//! `cmap`, the character to glyph map.
//!
//! Format 4 covers the Basic Multilingual Plane and is what almost every font
//! ships; format 12 covers everything above it and is needed for anything
//! outside Latin/Greek/Cyrillic. Formats 0, 6 and 13 exist but are symbol-font
//! and legacy trimming special cases that nothing here needs, so they are
//! reported as unsupported rather than guessed at.

use crate::{
    FontError, Result,
    sfnt::{Sfnt, read_u16, read_u32},
};

/// One subtable's entry in the encoding records.
#[derive(Clone, Copy, Debug)]
struct Record {
    offset: usize,
    format: u16,
}

/// Every subtable in the font, already filtered to the ones we can read.
#[derive(Clone, Debug, Default)]
pub struct Cmap {
    records: Vec<Record>,
}

/// `cmap` format 4: segment mapping, used for the BMP.
const FORMAT_4: u16 = 4;
/// `cmap` format 12: segmented coverage, used for everything.
const FORMAT_12: u16 = 12;

impl Cmap {
    /// Reads every subtable header in `table`.
    ///
    /// # Errors
    ///
    /// [`FontError::Truncated`] when the encoding records or a subtable header
    /// is cut short.
    pub fn parse(table: &[u8]) -> Result<Self> {
        if table.len() < 4 {
            return Err(FontError::Truncated("cmap header"));
        }
        let count = read_u16(table, 2)? as usize;
        let needed = 4 + count.checked_mul(8).ok_or(FontError::Truncated("cmap"))?;
        if table.len() < needed {
            return Err(FontError::Truncated("cmap encoding records"));
        }

        let mut records = Vec::with_capacity(count);
        for index in 0..count {
            let at = 4 + index * 8;
            let platform = read_u16(table, at)?;
            let encoding = read_u16(table, at + 2)?;
            let offset = read_u32(table, at + 4)? as usize;
            let Some(format) = table.get(offset..offset + 2) else {
                // A subtable pointing past the end is skipped, not fatal: the
                // font may have others that are intact.
                continue;
            };
            let format = u16::from_be_bytes([format[0], format[1]]);
            if !matches!(format, FORMAT_4 | FORMAT_12) {
                continue;
            }
            let _ = (platform, encoding);
            records.push(Record { offset, format });
        }
        Ok(Self { records })
    }

    /// Whether any subtable can be used.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        !self.records.is_empty()
    }

    /// The glyph `c` maps to.
    ///
    /// # Errors
    ///
    /// [`FontError::UnsupportedFormat`] when the font has no format 4 or 12
    /// subtable.
    pub fn glyph_index(&self, sfnt: &Sfnt<'_>, c: char) -> Result<Option<u16>> {
        let codepoint = c as u32;
        let Some(table) = sfnt.cmap() else {
            return Err(FontError::MissingTable("cmap"));
        };
        // A format 12 subtable can answer for any codepoint, so prefer it: the
        // format 4 table cannot cover anything above the BMP.
        let best = self
            .records
            .iter()
            .min_by_key(|record| u8::from(record.format != FORMAT_12));
        let Some(record) = best else {
            return Err(FontError::UnsupportedFormat(
                "cmap without a format 4 or 12 subtable",
            ));
        };
        let data = table
            .get(record.offset..)
            .ok_or(FontError::Truncated("cmap subtable"))?;
        let glyph = match record.format {
            FORMAT_4 => {
                if codepoint > 0xFFFF {
                    return Ok(None);
                }
                format_4(data, codepoint as u16)?
            }
            _ => format_12(data, codepoint)?,
        };
        Ok(glyph.filter(|glyph| *glyph != 0))
    }
}

/// `cmap` format 4: an array of segments, each mapping `[start, end]` to glyphs.
fn format_4(data: &[u8], codepoint: u16) -> Result<Option<u16>> {
    let length = read_u16(data, 2)? as usize;
    let table = data.get(..length).unwrap_or(data);
    let seg_count_x2 = read_u16(table, 6)? as usize;
    if seg_count_x2 == 0 || !seg_count_x2.is_multiple_of(2) {
        return Err(FontError::Malformed("cmap 4: segCountX2"));
    }
    let segments = seg_count_x2 / 2;

    let ends_at = 14;
    let starts_at = ends_at + seg_count_x2 + 2;
    let deltas_at = starts_at + seg_count_x2;
    let ranges_at = deltas_at + seg_count_x2;
    if ranges_at > table.len() {
        return Err(FontError::Truncated("cmap 4"));
    }

    for segment in 0..segments {
        let end = read_u16(table, ends_at + segment * 2)?;
        if codepoint > end {
            continue;
        }
        let start = read_u16(table, starts_at + segment * 2)?;
        if codepoint < start {
            // Segments are sorted, so the codepoint is simply not covered.
            return Ok(None);
        }
        let delta = read_u16(table, deltas_at + segment * 2)?;
        let range_offset = read_u16(table, ranges_at + segment * 2)? as usize;
        if range_offset == 0 {
            return Ok(Some(codepoint.wrapping_add(delta)));
        }
        // idRangeOffset is a byte offset from its own slot, not an index.
        let at = ranges_at + segment * 2 + range_offset + (codepoint - start) as usize * 2;
        let Some(entry) = table.get(at..at + 2) else {
            return Err(FontError::Truncated("cmap 4 idRangeOffset"));
        };
        let glyph = u16::from_be_bytes([entry[0], entry[1]]);
        if glyph == 0 {
            return Ok(None);
        }
        return Ok(Some(glyph.wrapping_add(delta)));
    }
    Ok(None)
}

/// `cmap` format 12: groups of `[startChar, endChar, startGlyphID]`.
fn format_12(data: &[u8], codepoint: u32) -> Result<Option<u16>> {
    let length = read_u32(data, 4)? as usize;
    let table = data.get(..length).unwrap_or(data);
    let groups = read_u32(table, 12)? as usize;
    for group in 0..groups {
        let at = 16 + group * 12;
        let Some(entry) = table.get(at..at + 12) else {
            return Err(FontError::Truncated("cmap 12 groups"));
        };
        let start = u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]);
        let end = u32::from_be_bytes([entry[4], entry[5], entry[6], entry[7]]);
        if codepoint < start {
            // Groups are sorted by start code.
            return Ok(None);
        }
        if codepoint <= end {
            let glyph = u32::from_be_bytes([entry[8], entry[9], entry[10], entry[11]]);
            let index = glyph + (codepoint - start);
            return u16::try_from(index)
                .map(Some)
                .map_err(|_| FontError::GlyphOutOfRange(u16::MAX));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfnt::test_view;

    const FONT: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    fn cmap() -> (Cmap, Sfnt<'static>) {
        let font = test_view(FONT);
        let cmap = Cmap::parse(font.cmap().unwrap()).unwrap();
        (cmap, font)
    }

    #[test]
    fn reads_the_ascii_range() {
        let (cmap, font) = cmap();
        // Independent parse of the same table.
        for (c, expected) in [('X', 53), ('A', 30), ('g', 8), ('@', 81), (' ', 994)] {
            let actual = cmap.glyph_index(&font, c).unwrap();
            assert_eq!(actual, Some(expected), "{c:?}");
        }
    }

    #[test]
    fn reaches_beyond_ascii() {
        let (cmap, font) = cmap();
        assert_eq!(cmap.glyph_index(&font, 'é').unwrap(), Some(252));
    }

    #[test]
    fn an_unmapped_codepoint_is_none_rather_than_an_error() {
        let (cmap, font) = cmap();
        // U+E000 is a private use area an ordinary text font has no glyph for.
        assert_eq!(cmap.glyph_index(&font, '\u{E000}').unwrap(), None);
    }

    #[test]
    fn glyph_zero_means_unmapped() {
        let (cmap, font) = cmap();
        // .notdef is glyph 0 and must not come back as a hit.
        assert_eq!(cmap.glyph_index(&font, '\u{0}').unwrap(), None);
    }

    #[test]
    fn a_cmap_with_no_usable_subtable_is_reported() {
        // Structurally valid but empty: the failure belongs to the lookup, not
        // to the header.
        let cmap = Cmap::parse(&[0, 0, 0, 0]).unwrap();
        assert!(!cmap.is_usable());
        let font = test_view(FONT);
        let actual = cmap.glyph_index(&font, 'X').unwrap_err().to_string();
        assert!(actual.contains("format 4 or 12"), "{actual}");
    }

    #[test]
    fn a_truncated_cmap_is_reported_rather_than_read_past_the_end() {
        let font = test_view(FONT);
        let table = font.cmap().unwrap();
        assert!(matches!(
            Cmap::parse(&table[..2]),
            Err(FontError::Truncated(_))
        ));
        // Cut inside the encoding record array, which the count says continues.
        let count = u16::from_be_bytes([table[2], table[3]]) as usize;
        let cut = 4 + count * 8 - 1;
        assert!(
            matches!(Cmap::parse(&table[..cut]), Err(FontError::Truncated(_))),
            "cut at {cut}"
        );
    }
}
