use xs_font::{Glyph, Point, ScaledFont};

fn is_breakable(c: char) -> bool {
    c.is_whitespace() || matches!(c, '-')
}

/// Simple paragraph layout for glyphs into `target`.
/// Starts at position `(0, ascent)`.
///
/// This is for testing and examples.
pub fn layout_paragraph<'a>(
    font: &ScaledFont<'a>,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph<'a>>,
) {
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + Point::new(0.0, font.ascent());
    let mut last_glyph: Option<u16> = None;
    let mut break_at: usize = 0;
    let mut break_caret: Point = caret;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = Point::new(position.x, caret.y + v_advance);
                break_at = target.len();
                break_caret = caret;
            }
            continue;
        }
        let Some(mut glyph) = font.glyph_or_notdef(c) else {
            continue;
        };
        if let Some(previous) = last_glyph {
            caret.x += font.kerning(previous, glyph.id());
        }
        glyph.set_position(caret);
        let advance = font.advance(glyph.id());

        if !c.is_whitespace() && caret.x > position.x + max_width {
            if break_at > 0 && break_at < target.len() {
                let rest: Vec<Glyph<'a>> = target.drain(break_at..).collect();
                caret = Point::new(position.x, break_caret.y + v_advance);
                let mut prev: Option<u16> = None;
                for mut g in rest {
                    if let Some(p) = prev {
                        caret.x += font.kerning(p, g.id());
                    }
                    g.set_position(caret);
                    caret.x += font.advance(g.id());
                    prev = Some(g.id());
                    target.push(g);
                }
                if let Some(p) = prev {
                    caret.x += font.kerning(p, glyph.id());
                }
                glyph.set_position(caret);
                caret.x += advance;
            } else {
                caret = Point::new(position.x, caret.y + v_advance);
                glyph.set_position(caret);
                caret.x += advance;
            }
            last_glyph = Some(glyph.id());
            target.push(glyph);
            break_at = target.len();
            break_caret = caret;
            continue;
        }

        last_glyph = Some(glyph.id());
        caret.x += advance;
        target.push(glyph);

        if is_breakable(c) {
            break_at = target.len();
            break_caret = caret;
        }
    }
}
