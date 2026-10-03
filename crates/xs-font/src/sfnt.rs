//! The `sfnt` container: the table directory every OpenType and TrueType file
//! starts with, and the four tables whose values are read straight out of it.
//!
//! References here are to the OpenType spec, chapter "Font File Tables".

use std::collections::HashMap;

use crate::{FontError, Result};

/// `sfntVersion`. `0x00010000` is the original TrueType, `OTTO` is CFF-flavoured
/// OpenType, `true`/`typ1` are the Apple variants of TrueType.
const VERSION_TRUETYPE: u32 = 0x0001_0000;
const VERSION_CFF: u32 = 0x4F54_544F;
const VERSION_TRUE: u32 = 0x7472_7565;
const VERSION_TYP1: u32 = 0x7479_7031;

const TAG_HEAD: u32 = tag(*b"head");
const TAG_HHEA: u32 = tag(*b"hhea");
const TAG_HMTX: u32 = tag(*b"hmtx");
const TAG_MAXP: u32 = tag(*b"maxp");
const TAG_LOCA: u32 = tag(*b"loca");
const TAG_GLYF: u32 = tag(*b"glyf");
const TAG_CMAP: u32 = tag(*b"cmap");

/// Packs four bytes into the big-endian tag the tables are keyed by.
const fn tag(name: [u8; 4]) -> u32 {
    u32::from_be_bytes(name)
}

/// One entry of the table directory.
#[derive(Clone, Copy, Debug)]
struct Table {
    offset: usize,
    length: usize,
}

/// Everything read out of the file once, at parse time.
///
/// Kept apart from the bytes so a `Font` can own it: the file outlives every
/// lookup but the directory does not need to be re-read for one, and rebuilding
/// a `HashMap` per glyph would show up in a text layout loop.
#[derive(Clone, Debug)]
pub struct Tables {
    directory: HashMap<u32, Table>,
    head: Head,
    hhea: Hhea,
    num_glyphs: u16,
}

impl Tables {
    /// Reads the directory and the metrics tables.
    ///
    /// # Errors
    ///
    /// [`FontError::Truncated`] when the directory or a required table is cut
    /// short, and [`FontError::UnsupportedFormat`] for anything that is not a
    /// TrueType-flavoured `sfnt`.
    pub fn parse(data: &[u8]) -> Result<Self> {
        /// Bytes before the first directory entry.
        const DIRECTORY: usize = 12;

        if data.len() < DIRECTORY {
            return Err(FontError::Truncated("sfnt header"));
        }
        let version = read_u32(data, 0)?;
        if !matches!(version, VERSION_TRUETYPE | VERSION_TRUE | VERSION_TYP1) {
            let kind = if version == VERSION_CFF {
                "CFF-flavoured OpenType"
            } else {
                "unknown sfnt version"
            };
            return Err(FontError::UnsupportedFormat(kind));
        }

        let count = read_u16(data, 4)? as usize;
        // Each entry is tag + checksum + offset + length.
        let needed = DIRECTORY
            .checked_add(
                count
                    .checked_mul(16)
                    .ok_or(FontError::Truncated("table count"))?,
            )
            .ok_or(FontError::Truncated("table count"))?;
        if data.len() < needed {
            return Err(FontError::Truncated("table directory"));
        }

        let mut directory: HashMap<u32, Table> = HashMap::with_capacity(count);
        for index in 0..count {
            let entry = DIRECTORY + index * 16;
            let name = read_u32(data, entry)?;
            let offset = read_u32(data, entry + 8)? as usize;
            let length = read_u32(data, entry + 12)? as usize;
            let end = offset
                .checked_add(length)
                .ok_or(FontError::Truncated("table extent"))?;
            // A directory may point past the end of the file for a truncated
            // download; drop that table rather than refusing the whole font.
            if end <= data.len() {
                directory.insert(name, Table { offset, length });
            }
        }

        let head_offset = table(&directory, data, TAG_HEAD, "head")?;
        let hhea_offset = table(&directory, data, TAG_HHEA, "hhea")?;
        let head = Head::parse(data, head_offset)?;
        let hhea = Hhea::parse(data, hhea_offset)?;

        // maxp is required too, and its glyph count bounds everything else.
        let maxp_offset = table(&directory, data, TAG_MAXP, "maxp")?;
        let num_glyphs = read_u16(data, maxp_offset + 4)? as usize;

        Ok(Self {
            directory,
            head,
            hhea,
            num_glyphs: num_glyphs as u16,
        })
    }
}

/// A borrowed view of a font: the parsed directory plus the bytes to read out of.
#[derive(Clone, Copy, Debug)]
pub struct Sfnt<'a> {
    tables: &'a Tables,
    data: &'a [u8],
}

impl<'a> Sfnt<'a> {
    /// Pairs a parsed directory with the bytes it describes.
    #[must_use]
    pub fn new(tables: &'a Tables, data: &'a [u8]) -> Self {
        Self { tables, data }
    }

    /// The bytes of `name`, or `None` when the font does not have it.
    #[must_use]
    pub fn table(&self, name: u32) -> Option<&'a [u8]> {
        let table = self.tables.directory.get(&name)?;
        Some(&self.data[table.offset..table.offset + table.length])
    }

    /// Whether the font has a `glyf` outline table.
    #[must_use]
    pub fn has_glyf(&self) -> bool {
        self.tables.directory.contains_key(&TAG_GLYF)
    }

    /// The design units per em.
    #[must_use]
    pub fn units_per_em(&self) -> u16 {
        self.tables.head.units_per_em
    }

    /// Whether `loca` stores half-word offsets.
    #[must_use]
    pub fn index_to_loc_is_short(&self) -> bool {
        self.tables.head.index_to_loc_format == 0
    }

    /// The number of glyphs the font holds.
    #[must_use]
    pub fn num_glyphs(&self) -> u16 {
        self.tables.num_glyphs
    }

    /// The ascent above the baseline, in font units.
    #[must_use]
    pub fn ascent(&self) -> f32 {
        self.tables.hhea.ascender()
    }

    /// The descent below the baseline, in font units, negative.
    #[must_use]
    pub fn descent(&self) -> f32 {
        self.tables.hhea.descender()
    }

    /// The extra leading between lines, in font units.
    #[must_use]
    pub fn line_gap(&self) -> f32 {
        self.tables.hhea.line_gap()
    }

    /// The advance width of `glyph`, in font units.
    ///
    /// `hmtx` stores a full record for the first `num_h_metrics` glyphs and
    /// then only left side bearings, so every glyph past that shares the last
    /// advance.
    ///
    /// # Errors
    ///
    /// [`FontError::MissingTable`] when `hmtx` is absent, or
    /// [`FontError::Truncated`] when the glyph index is past the end.
    pub fn advance(&self, glyph: u16) -> Result<u16> {
        let hmtx = self
            .table(TAG_HMTX)
            .ok_or(FontError::MissingTable("hmtx"))?;
        let metrics = self.tables.hhea.num_h_metrics;
        let index = if glyph < metrics {
            glyph as usize
        } else {
            (metrics as usize).saturating_sub(1)
        };
        // Each record is advanceWidth then leftSideBearing, both uint16.
        let offset = index.checked_mul(4).ok_or(FontError::Truncated("hmtx"))?;
        if offset + 2 > hmtx.len() {
            return Err(FontError::Truncated("hmtx"));
        }
        Ok(u16::from_be_bytes([hmtx[offset], hmtx[offset + 1]]))
    }

    /// The byte range of `glyph` in `glyf`, from `loca`.
    ///
    /// An empty range means the glyph has no outline (a space, say), which is
    /// legal rather than an error.
    ///
    /// # Errors
    ///
    /// [`FontError::MissingTable`] when `loca` or `glyf` is absent, and
    /// [`FontError::Truncated`] when the index is out of range.
    pub fn glyph_range(&self, glyph: u16) -> Result<(usize, usize)> {
        let index = glyph as usize;
        if index >= self.num_glyphs() as usize {
            return Err(FontError::GlyphOutOfRange(glyph));
        }
        let loca = self
            .table(TAG_LOCA)
            .ok_or(FontError::MissingTable("loca"))?;
        let glyf = self
            .table(TAG_GLYF)
            .ok_or(FontError::MissingTable("glyf"))?;

        let read = |i: usize| -> Result<usize> {
            if self.index_to_loc_is_short() {
                let at = i * 2;
                let entry = loca.get(at..at + 2).ok_or(FontError::Truncated("loca"))?;
                Ok(u16::from_be_bytes([entry[0], entry[1]]) as usize * 2)
            } else {
                let at = i * 4;
                let entry = loca.get(at..at + 4).ok_or(FontError::Truncated("loca"))?;
                Ok(u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]) as usize)
            }
        };
        let start = read(index)?;
        let end = read(index + 1)?;
        if end < start || end > glyf.len() {
            return Err(FontError::Truncated("loca"));
        }
        // loca offsets are relative to the start of glyf.
        Ok((start, end))
    }

    #[must_use]
    pub fn cmap(&self) -> Option<&'a [u8]> {
        self.table(TAG_CMAP)
    }
}

/// The handful of `head` fields that matter to outlines.
#[derive(Clone, Copy, Debug)]
struct Head {
    units_per_em: u16,
    index_to_loc_format: i16,
}

impl Head {
    fn parse(data: &[u8], offset: usize) -> Result<Self> {
        if data.len() < offset + 54 {
            return Err(FontError::Truncated("head"));
        }
        Ok(Self {
            units_per_em: read_u16(data, offset + 18)?,
            index_to_loc_format: read_i16(data, offset + 50)?,
        })
    }
}

/// The handful of `hhea` fields that matter to layout.
#[derive(Clone, Copy, Debug)]
struct Hhea {
    ascender: i16,
    descender: i16,
    line_gap: i16,
    num_h_metrics: u16,
}

impl Hhea {
    fn parse(data: &[u8], offset: usize) -> Result<Self> {
        if data.len() < offset + 36 {
            return Err(FontError::Truncated("hhea"));
        }
        Ok(Self {
            ascender: read_i16(data, offset + 4)?,
            descender: read_i16(data, offset + 6)?,
            line_gap: read_i16(data, offset + 8)?,
            num_h_metrics: read_u16(data, offset + 34)?,
        })
    }

    /// Ascent above the baseline, in font units.
    #[must_use]
    pub fn ascender(self) -> f32 {
        self.ascender as f32
    }

    /// Descent below the baseline, negative, in font units.
    #[must_use]
    pub fn descender(self) -> f32 {
        self.descender as f32
    }

    #[must_use]
    pub fn line_gap(self) -> f32 {
        self.line_gap as f32
    }
}

fn table(
    tables: &HashMap<u32, Table>,
    data: &[u8],
    name: u32,
    label: &'static str,
) -> Result<usize> {
    tables
        .get(&name)
        .map(|table| table.offset)
        .ok_or(FontError::MissingTable(label))
        .and_then(|offset| {
            if data.len() > offset {
                Ok(offset)
            } else {
                Err(FontError::Truncated(label))
            }
        })
}

pub fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    let at = data
        .get(offset..offset + 2)
        .ok_or(FontError::Truncated("u16"))?;
    Ok(u16::from_be_bytes([at[0], at[1]]))
}

pub fn read_i16(data: &[u8], offset: usize) -> Result<i16> {
    read_u16(data, offset).map(|value| value as i16)
}

pub fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let at = data
        .get(offset..offset + 4)
        .ok_or(FontError::Truncated("u32"))?;
    Ok(u32::from_be_bytes([at[0], at[1], at[2], at[3]]))
}

#[cfg(test)]
/// A borrowed view of a font for tests. The parsed directory is leaked because
/// `Sfnt` borrows it, and a handful of leaked directories is not worth the
/// self-referential struct that avoiding it would need.
pub(crate) fn test_view(bytes: &'static [u8]) -> Sfnt<'static> {
    let tables = Box::leak(Box::new(Tables::parse(bytes).expect("a real font")));
    Sfnt::new(tables, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The bundled font, parsed for real rather than through a fixture: these
    // numbers were read out of the file with an independent Python parser, so a
    // mistake in the offsets here cannot agree with them by accident.
    const FONT: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    #[test]
    fn reads_the_directory_and_the_metrics_tables() {
        let sfnt = test_view(FONT);
        assert_eq!(sfnt.units_per_em(), 1000);
        assert!(sfnt.index_to_loc_is_short());
        assert_eq!(sfnt.tables.hhea.num_h_metrics, 1018);
        assert!(sfnt.has_glyf());
        assert!(sfnt.cmap().is_some());
    }

    #[test]
    fn ascent_and_descent_match_the_header() {
        let sfnt = test_view(FONT);
        assert_eq!(sfnt.ascent(), 1025.0);
        assert_eq!(sfnt.descent(), -275.0);
        assert_eq!(sfnt.line_gap(), 0.0);
    }

    #[test]
    fn advances_match_the_hmtx_table() {
        let sfnt = test_view(FONT);
        // Checked against an independent parse of hmtx.
        assert_eq!(sfnt.advance(53).unwrap(), 613);
        assert_eq!(sfnt.advance(30).unwrap(), 641);
        assert_eq!(sfnt.advance(8).unwrap(), 528);
        assert_eq!(sfnt.advance(81).unwrap(), 891);
        assert_eq!(sfnt.advance(252).unwrap(), 549);
    }

    #[test]
    fn a_glyph_beyond_the_last_one_is_refused() {
        let sfnt = test_view(FONT);
        assert!(matches!(
            sfnt.glyph_range(5000),
            Err(FontError::GlyphOutOfRange(5000))
        ));
    }

    #[test]
    fn a_short_file_is_refused_rather_than_read_past_the_end() {
        assert!(matches!(Tables::parse(&[]), Err(FontError::Truncated(_))));
        assert!(matches!(
            Tables::parse(&FONT[..8]),
            Err(FontError::Truncated(_))
        ));
        assert!(matches!(
            Tables::parse(&FONT[..20]),
            Err(FontError::Truncated(_))
        ));
    }

    #[test]
    fn a_cff_font_is_named_in_the_error() {
        let mut data = FONT.to_vec();
        data[0..4].copy_from_slice(b"OTTO");
        let actual = Tables::parse(&data).unwrap_err().to_string();
        assert!(actual.contains("CFF"), "{actual}");
    }

    #[test]
    fn an_unknown_version_is_refused() {
        let mut data = FONT.to_vec();
        data[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(matches!(
            Tables::parse(&data),
            Err(FontError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn a_directory_pointing_past_the_end_drops_that_table() {
        // Corrupt the `glyf` entry's length so its extent runs off the end.
        let mut data = FONT.to_vec();
        let count = u16::from_be_bytes([data[4], data[5]]) as usize;
        let mut glyf = None;
        for index in 0..count {
            let entry = 12 + index * 16;
            if &data[entry..entry + 4] == b"glyf" {
                glyf = Some(entry);
            }
        }
        let entry = glyf.expect("the font has a glyf table");
        data[entry + 12..entry + 16].copy_from_slice(&u32::MAX.to_be_bytes());

        let tables = Box::leak(Box::new(Tables::parse(&data).expect("still readable")));
        let sfnt = Sfnt::new(tables, &data);
        assert!(
            !sfnt.has_glyf(),
            "the oversized table should have been dropped"
        );
        assert!(sfnt.cmap().is_some(), "other tables should survive");
        assert_eq!(sfnt.units_per_em(), 1000);
    }

    #[test]
    fn glyph_ranges_are_monotonic_and_inside_glyf() {
        let sfnt = test_view(FONT);
        let glyf = sfnt.table(0x676C_7966).unwrap(); // "glyf"
        for glyph in [0u16, 1, 8, 30, 53, 81, 252, 1017] {
            let (start, end) = sfnt.glyph_range(glyph).unwrap();
            assert!(start <= end, "glyph {glyph}: {start} > {end}");
            assert!(end <= glyf.len(), "glyph {glyph} past glyf");
        }
    }
}
