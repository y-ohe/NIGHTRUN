//! Drawing primitives: gradients, text (plain / sheared / glowing), lines.

use crate::color;
use crate::font::PsfFont;
use crate::surface::Surface;

pub fn vertical_gradient(surf: &mut Surface, x: i32, y: i32, w: i32, h: i32, stops: &[(u32, u32)]) {
    if h <= 0 {
        return;
    }
    for row in 0..h {
        let t = row as u32 * 1000 / h as u32;
        let c = color::gradient(stops, t);
        surf.fill_rect(x, y + row, w, 1, c);
    }
}

/// Anti-aliased-ish line via simple DDA with blending.
pub fn line(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, colr: u32, alpha: u32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);
    for i in 0..=steps {
        let x = x0 + (x1 - x0) * i / steps;
        let y = y0 + (y1 - y0) * i / steps;
        surf.blend(x, y, colr, alpha);
    }
}

pub fn text_width(font: &PsfFont, s: &str, scale: usize, tracking: i32) -> i32 {
    let n = s.chars().count() as i32;
    if n == 0 {
        return 0;
    }
    n * (font.width * scale) as i32 + (n - 1) * tracking
}

pub fn text_width_with_fallback(
    font: &PsfFont,
    fallback: Option<&PsfFont>,
    s: &str,
    scale: usize,
    tracking: i32,
) -> i32 {
    let mut width = 0;
    let mut count = 0;
    for ch in s.chars() {
        if count > 0 {
            width += tracking;
        }
        width += glyph_width(font, fallback, ch) as i32 * scale as i32;
        count += 1;
    }
    width
}

pub fn glyph_width(font: &PsfFont, fallback: Option<&PsfFont>, ch: char) -> usize {
    if font.glyph(ch).is_some() {
        font.width
    } else if let Some(fallback) = fallback.filter(|fallback| fallback.glyph(ch).is_some()) {
        fallback.width
    } else if let Some(substitute) = substitute(ch) {
        if font.glyph(substitute).is_some() {
            font.width
        } else if let Some(fallback) =
            fallback.filter(|fallback| fallback.glyph(substitute).is_some())
        {
            fallback.width
        } else {
            font.width
        }
    } else {
        font.width
    }
}

/// Draw text; `tracking` is extra pixels between glyphs.
#[allow(clippy::too_many_arguments)] // flat draw API is deliberate
pub fn text(
    surf: &mut Surface,
    font: &PsfFont,
    x: i32,
    y: i32,
    s: &str,
    colr: u32,
    scale: usize,
    tracking: i32,
) {
    text_with_fallback(surf, font, None, x, y, s, colr, scale, tracking);
}

pub fn text_with_fallback(
    surf: &mut Surface,
    font: &PsfFont,
    fallback: Option<&PsfFont>,
    x: i32,
    y: i32,
    s: &str,
    colr: u32,
    scale: usize,
    tracking: i32,
) {
    text_fx_with_fallback(
        surf,
        font,
        fallback,
        x,
        y,
        s,
        scale,
        tracking,
        0,
        |_, _| colr,
        255,
    );
}

/// Full-control text: shear (italic slant, pixels shifted right at the top
/// row, tapering to 0 at the bottom), per-row color, global alpha.
#[allow(clippy::too_many_arguments)] // flat draw API is deliberate
pub fn text_fx(
    surf: &mut Surface,
    font: &PsfFont,
    x: i32,
    y: i32,
    s: &str,
    scale: usize,
    tracking: i32,
    shear: i32,
    row_color: impl FnMut(usize, usize) -> u32,
    alpha: u32,
) {
    text_fx_with_fallback(
        surf, font, None, x, y, s, scale, tracking, shear, row_color, alpha,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn text_fx_with_fallback(
    surf: &mut Surface,
    font: &PsfFont,
    fallback: Option<&PsfFont>,
    x: i32,
    y: i32,
    s: &str,
    scale: usize,
    tracking: i32,
    shear: i32,
    mut row_color: impl FnMut(usize, usize) -> u32,
    alpha: u32,
) {
    let mut pen_x = x;
    for ch in s.chars() {
        let selected = font
            .glyph(ch)
            .map(|glyph| (font, glyph))
            .or_else(|| fallback.and_then(|fallback| fallback.glyph(ch).map(|g| (fallback, g))))
            .or_else(|| {
                let replacement = substitute(ch)?;
                font.glyph(replacement)
                    .map(|glyph| (font, glyph))
                    .or_else(|| {
                        fallback.and_then(|fallback| {
                            fallback.glyph(replacement).map(|glyph| (fallback, glyph))
                        })
                    })
            });
        let Some((glyph_font, glyph)) = selected else {
            pen_x += (font.width * scale) as i32 + tracking;
            continue;
        };
        let gh = glyph_font.height * scale;
        let gw = glyph_font.width * scale;
        for gy in 0..gh {
            let src_y = gy / scale;
            let shift = shear * (gh - 1 - gy) as i32 / gh.max(1) as i32;
            let c = row_color(gy, gh);
            for gx in 0..gw {
                if glyph_font.pixel(glyph, gx / scale, src_y) {
                    if alpha >= 255 {
                        surf.put(pen_x + gx as i32 + shift, y + gy as i32, c);
                    } else {
                        surf.blend(pen_x + gx as i32 + shift, y + gy as i32, c, alpha);
                    }
                }
            }
        }
        pen_x += gw as i32 + tracking;
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::text_with_fallback;
    use crate::{PsfFont, Surface};

    fn psf2_font(width: usize, height: usize, codepoint: char, bitmap: &[u8]) -> Vec<u8> {
        let bytes_per_row = width.div_ceil(8);
        let bytes_per_glyph = bytes_per_row * height;
        assert_eq!(bitmap.len(), bytes_per_glyph);

        let mut data = Vec::new();
        for field in [
            0x864a_b572u32,
            0,
            32,
            1,
            1,
            bytes_per_glyph as u32,
            height as u32,
            width as u32,
        ] {
            data.extend_from_slice(&field.to_le_bytes());
        }
        data.extend_from_slice(bitmap);
        data.extend_from_slice(codepoint.encode_utf8(&mut [0; 4]).as_bytes());
        data.push(0xff);
        data
    }

    #[test]
    fn fallback_draw_uses_fallback_font_stride_and_dimensions() {
        let primary_data = psf2_font(8, 8, '?', &[0xff; 8]);
        let fallback_data = psf2_font(16, 16, '日', &[0xff; 32]);
        let primary = PsfFont::parse(&primary_data).unwrap();
        let fallback = PsfFont::parse(&fallback_data).unwrap();
        let mut surface = Surface::new(16, 16);

        text_with_fallback(
            &mut surface,
            &primary,
            Some(&fallback),
            0,
            0,
            "日",
            0x00ff_ffff,
            1,
            0,
        );

        assert!(surface.rows().iter().all(|&pixel| pixel == 0x00ff_ffff));
    }
}

/// ASCII lookalikes for typographic characters the fonts lack.
pub fn substitute(ch: char) -> Option<char> {
    Some(match ch {
        '\u{2014}' | '\u{2015}' | '\u{2212}' => '-', // em dash, horizontal bar, minus
        '\u{00a0}' | '\u{2009}' | '\u{202f}' => ' ', // nbsp, thin spaces
        _ => return None,
    })
}

/// Soft neon glow behind text: several blended, offset copies.
#[allow(clippy::too_many_arguments)] // flat draw API is deliberate
pub fn text_glow(
    surf: &mut Surface,
    font: &PsfFont,
    x: i32,
    y: i32,
    s: &str,
    scale: usize,
    tracking: i32,
    shear: i32,
    colr: u32,
    radius: i32,
) {
    for r in 1..=radius {
        let a = (46 / r as u32).max(10);
        for (dx, dy) in [
            (r, 0),
            (-r, 0),
            (0, r),
            (0, -r),
            (r, r),
            (-r, r),
            (r, -r),
            (-r, -r),
        ] {
            text_fx(
                surf,
                font,
                x + dx,
                y + dy,
                s,
                scale,
                tracking,
                shear,
                |_, _| colr,
                a,
            );
        }
    }
}

/// Darken every third row for a CRT scanline feel.
pub fn scanlines(surf: &mut Surface, alpha: u32) {
    let (w, h) = (surf.width as i32, surf.height as i32);
    let mut y = 0;
    while y < h {
        surf.blend_rect(0, y, w, 1, 0x000000, alpha);
        y += 3;
    }
}
