//! Parsing TrueType (`.ttf`) fonts: the `sfnt` tables, character mapping,
//! glyph outlines, and an antialiased rasterizer for them.
//!
//! This replaces `ab_glyph` plus the `ttf-parser` it wraps. Scope is deliberately
//! narrow, because that is all this workspace needs:
//!
//! * **TrueType outlines only** (`glyf` + `loca`). A CFF-flavoured OpenType
//!   file (`.otf`, `OTTO`) is rejected with [`FontError::UnsupportedFormat`]
//!   rather than half-read. CJK webfonts are usually CFF, so that is the one
//!   gap worth knowing about.
//! * **No shaping.** `glyph_index` maps a `char` to a glyph through `cmap` and
//!   `kerning` reads nothing, so Latin/Greek/Cyrillic text lays out correctly
//!   and anything relying on ligatures, reordering or bidi does not. That is
//!   not a regression: the font this workspace ships has a 115 KB `GPOS` table
//!   and the previous stack ignored it too.
//! * **No hinting.** `glyf` instructions (`fpgm`, `cvt`, `prep`) are skipped.
//!
//! # Examples
//!
//! ```
//! use xs_font::Font;
//!
//! let bytes = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
//! let font = Font::parse(bytes.to_vec()).unwrap();
//! assert_eq!(font.units_per_em(), 1000);
//!
//! let scaled = font.scaled(32.0);
//! let raster = scaled.glyph('X').unwrap().rasterize().unwrap();
//! println!("{}x{} pixels", raster.pixel_width(), raster.pixel_height());
//! raster.draw(|x, y, coverage| {
//!     // ... blit into an atlas
//!     # let _ = (x, y, coverage);
//! });
//! ```

#![warn(clippy::all, clippy::pedantic)]
// Every field in a font file has whatever width the format gives it, and the
// format is full of `u8`s and `i16`s that have to become `f32`s and offsets.
// Where a cast can actually go wrong the read is bounds or range checked
// first; the rest is the parsing, not an oversight.
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

mod cmap;
mod error;
mod glyf;
mod outline;
mod raster;
mod sfnt;

pub use error::{FontError, Result};
pub use outline::{Contour, Outline, Point, Segment};
pub use raster::{RasterizedGlyph, Rect};

use std::sync::Arc;

use crate::{cmap::Cmap, sfnt::Sfnt, sfnt::Tables};

/// A parsed font.
///
/// Owns its bytes so a font can be stored and shared without borrowing, which is
/// what lets a caller keep one per family/weight without leaking the file data
/// to get a `&'static` slice.
#[derive(Clone, Debug)]
pub struct Font {
    data: Arc<[u8]>,
    tables: Tables,
}

impl Font {
    /// Takes ownership of `data` and parses it.
    ///
    /// # Errors
    ///
    /// [`FontError`] when the file is truncated, is not TrueType, or is missing
    /// a table this implementation needs.
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        let data: Arc<[u8]> = Arc::from(data);
        // Parse eagerly so a bad font is rejected here rather than at first use,
        // and so no lookup has to re-read the directory.
        let tables = Tables::parse(&data)?;
        let sfnt = Sfnt::new(&tables, &data);
        // Only `glyf` outlines are implemented, so a CFF font is refused here
        // rather than parsing cleanly and then rendering nothing at all.
        if !sfnt.has_glyf() {
            return Err(FontError::UnsupportedFormat("only glyf outlines, not CFF"));
        }
        let cmap = sfnt.cmap().ok_or(FontError::MissingTable("cmap"))?;
        if !Cmap::parse(cmap)?.is_usable() {
            return Err(FontError::UnsupportedFormat(
                "a cmap with no format 4 or 12 subtable",
            ));
        }
        Ok(Self { data, tables })
    }

    /// Copies `data` and parses it.
    ///
    /// # Errors
    ///
    /// The same as [`Font::parse`].
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        Self::parse(data.to_vec())
    }

    /// The raw file.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// A view of this font at `px_per_em` pixels per em.
    #[must_use]
    pub fn scaled(&self, px_per_em: f32) -> ScaledFont<'_> {
        ScaledFont {
            sfnt: self.sfnt(),
            px_per_em,
        }
    }

    /// The design units per em, 1000 or 2048 for every real font.
    #[must_use]
    pub fn units_per_em(&self) -> u16 {
        self.sfnt().units_per_em()
    }

    /// The number of glyphs in the font.
    #[must_use]
    pub fn num_glyphs(&self) -> u16 {
        self.sfnt().num_glyphs()
    }

    /// The ascent above the baseline, in font units.
    #[must_use]
    pub fn ascent(&self) -> f32 {
        self.sfnt().ascent()
    }

    /// The descent below the baseline, in font units, negative.
    #[must_use]
    pub fn descent(&self) -> f32 {
        self.sfnt().descent()
    }

    /// The extra leading between lines, in font units.
    #[must_use]
    pub fn line_gap(&self) -> f32 {
        self.sfnt().line_gap()
    }

    /// One line's worth of vertical space, ascent to descent, in font units.
    ///
    /// This is the divisor `ab_glyph` used for `PxScale`, which is why the
    /// glyphs it rasterized were 25% smaller than the same nominal font size
    /// asked for here: a 32px `PxScale` divided 32 by 1300 rather than by
    /// [`units_per_em`](Self::units_per_em). See [`px_per_em_for_line_height`].
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascent() - self.descent()
    }

    /// The em size whose one line of text is `px` tall.
    ///
    /// Use this when a size is specified as a line height rather than an em,
    /// which is how this crate's predecessor was driven in practice.
    #[must_use]
    pub fn px_per_em_for_line_height(&self, px: f32) -> f32 {
        px * self.units_per_em() as f32 / self.line_height()
    }

    /// The advance width of `glyph`, in font units, or zero for an index the
    /// font does not have.
    #[must_use]
    pub fn advance(&self, glyph: u16) -> f32 {
        f32::from(self.sfnt().advance(glyph).unwrap_or(0))
    }

    /// The glyph `c` maps to, or `None` when the font has no coverage for it.
    #[must_use]
    pub fn glyph_index(&self, c: char) -> Option<u16> {
        self.sfnt().glyph_index(c).ok().flatten()
    }

    /// The outline of `glyph`, in font units, or `None` for a glyph with none
    /// (a space, say).
    #[must_use]
    pub fn outline(&self, glyph: u16) -> Option<Outline> {
        self.sfnt().outline(glyph).ok().flatten()
    }

    /// A view of the parsed directory over the bytes. Infallible: the directory
    /// was built and checked in `Font::parse` and the bytes cannot change since.
    fn sfnt(&self) -> Sfnt<'_> {
        Sfnt::new(&self.tables, &self.data)
    }
}

/// A font at a pixel size.
///
/// Scaling is a separate view rather than a wrapper type the caller has to
/// thread through: nothing here mutates, so every metric is a method that
/// multiplies by the size.
#[derive(Clone, Copy, Debug)]
pub struct ScaledFont<'a> {
    sfnt: Sfnt<'a>,
    px_per_em: f32,
}

impl<'a> ScaledFont<'a> {
    /// The size this view was created with, in pixels per em.
    #[must_use]
    pub fn px_per_em(&self) -> f32 {
        self.px_per_em
    }

    /// Font units to pixels.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.px_per_em / f32::from(self.sfnt.units_per_em())
    }

    /// The ascent above the baseline, in pixels.
    #[must_use]
    pub fn ascent(&self) -> f32 {
        self.sfnt.ascent() * self.scale()
    }

    /// The descent below the baseline, negative, in pixels.
    #[must_use]
    pub fn descent(&self) -> f32 {
        self.sfnt.descent() * self.scale()
    }

    /// The extra leading between lines, in pixels.
    #[must_use]
    pub fn line_gap(&self) -> f32 {
        self.sfnt.line_gap() * self.scale()
    }

    /// One line's worth of vertical space, ascent to descent, in pixels.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.ascent() - self.descent()
    }

    /// The `glyph` for `c`, positioned at the origin.
    ///
    /// `None` when the font has no glyph for `c` and no `.notdef` either. Use
    /// [`Self::glyph_or_notdef`] to lay out text that may not be covered.
    #[must_use]
    pub fn glyph(&self, c: char) -> Option<Glyph<'a>> {
        let id = self.sfnt.glyph_index(c).ok().flatten()?;
        Some(Glyph {
            scaled: *self,
            id,
            position: Point::ORIGIN,
        })
    }

    /// The `glyph` at `id`, positioned at the origin.
    #[must_use]
    pub fn glyph_by_id(&self, id: u16) -> Option<Glyph<'a>> {
        if id >= self.sfnt.num_glyphs() {
            return None;
        }
        Some(Glyph {
            scaled: *self,
            id,
            position: Point::ORIGIN,
        })
    }

    /// The `glyph` for `c`, falling back to `.notdef` when the font has nothing
    /// for it.
    ///
    /// Laying out text the font cannot cover should still move the caret along,
    /// which is what dropping the character would not do.
    #[must_use]
    pub fn glyph_or_notdef(&self, c: char) -> Option<Glyph<'a>> {
        self.glyph(c).or_else(|| self.glyph_by_id(0))
    }

    /// The `id` for `c`, without building a positioned glyph.
    #[must_use]
    pub fn glyph_index(&self, c: char) -> Option<u16> {
        self.sfnt.glyph_index(c).ok().flatten()
    }

    /// The advance width of `glyph`, in pixels.
    #[must_use]
    pub fn advance(&self, glyph: u16) -> f32 {
        f32::from(self.sfnt.advance(glyph).unwrap_or(0)) * self.scale()
    }

    /// The horizontal adjustment between two glyphs, in pixels.
    ///
    /// Always zero: kerning lives in `GPOS`, which this does not read. The
    /// previous stack read the legacy `kern` table, and the bundled font has
    /// none, so nothing here changes for the fonts in use.
    #[must_use]
    pub fn kerning(&self, _left: u16, _right: u16) -> f32 {
        0.0
    }

    /// The outline of `glyph` at this size, ready to rasterize.
    #[must_use]
    pub fn outline(&self, glyph: u16) -> Option<RasterizedGlyph> {
        let outline = self.sfnt.outline(glyph).ok().flatten()?;
        Some(RasterizedGlyph::from_outline(outline, self.scale()))
    }
}

/// A glyph placed on the line, in pixels from the pen position.
#[derive(Clone, Copy, Debug)]
pub struct Glyph<'a> {
    scaled: ScaledFont<'a>,
    id: u16,
    position: Point,
}

impl Glyph<'_> {
    /// The glyph index.
    #[must_use]
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Where the pen is, relative to the line's start.
    #[must_use]
    pub fn position(&self) -> Point {
        self.position
    }

    /// Moves the pen.
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    /// The advance width, in pixels.
    #[must_use]
    pub fn advance(&self) -> f32 {
        self.scaled.advance(self.id)
    }

    /// The rasterized outline, or `None` for a blank glyph.
    #[must_use]
    pub fn rasterize(&self) -> Option<RasterizedGlyph> {
        self.scaled.outline(self.id)
    }
}
