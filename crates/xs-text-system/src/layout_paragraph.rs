use ab_glyph::{Font, Glyph, Point, ScaleFont, point};

fn is_breakable(c: char) -> bool {
    c.is_whitespace() || matches!(c, '-')
}

/// Simple paragraph layout for glyphs into `target`.
/// Starts at position `(0, ascent)`.
///
/// This is for testing and examples.
pub fn layout_paragraph<F, SF>(
    font: SF,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    let mut break_at: usize = 0;
    let mut break_caret: Point = caret;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                break_at = target.len();
                break_caret = caret;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;
        let advance = font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + max_width {
            if break_at > 0 && break_at < target.len() {
                let rest: Vec<Glyph> = target.drain(break_at..).collect();
                caret = point(position.x, break_caret.y + v_advance);
                let mut prev: Option<Glyph> = None;
                for mut g in rest {
                    if let Some(p) = prev.take() {
                        caret.x += font.kern(p.id, g.id);
                    }
                    g.position = caret;
                    caret.x += font.h_advance(g.id);
                    prev = Some(g.clone());
                    target.push(g);
                }
                if let Some(p) = prev {
                    caret.x += font.kern(p.id, glyph.id);
                }
                glyph.position = caret;
                caret.x += advance;
            } else {
                caret = point(position.x, caret.y + v_advance);
                glyph.position = caret;
                caret.x += advance;
            }
            last_glyph = Some(glyph.clone());
            target.push(glyph);
            break_at = target.len();
            break_caret = caret;
            continue;
        }

        last_glyph = Some(glyph.clone());
        caret.x += advance;
        target.push(glyph);

        if is_breakable(c) {
            break_at = target.len();
            break_caret = caret;
        }
    }
}
