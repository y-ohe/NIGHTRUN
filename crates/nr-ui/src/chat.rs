//! Chat screen: status bar, scrollback with `user:` / `llama:` turns,
//! input line with cursor.

use alloc::string::String;
use alloc::vec::Vec;

use nr_gfx::{draw, theme, Surface};

use crate::fonts::Fonts;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Llama,
    System,
}

impl Role {
    /// Prefix text; the assistant's name comes from the model family.
    pub fn prefix_len(&self, assistant: &str) -> usize {
        match self {
            Role::User => "user: ".len(),
            Role::Llama => assistant.len() + 2,
            Role::System => 0,
        }
    }

    fn color(&self) -> u32 {
        match self {
            Role::User => theme::NEON_CYAN,
            Role::Llama => theme::NEON_PINK,
            Role::System => theme::TEXT_DIM,
        }
    }
}

pub struct Turn {
    pub role: Role,
    pub text: String,
}

pub struct Stats<'a> {
    pub model: &'a str,
    /// Assistant display name ("llama", "qwen").
    pub assistant: &'a str,
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    /// Generation rate, milli-tokens per second; 0 hides the readout.
    pub tok_s_milli: u32,
    /// Prompt-processing rate, milli-tokens per second; 0 hides it.
    pub pp_milli: u32,
    /// First-token latency of the last turn, ms; 0 hides it.
    pub ftl_ms: u32,
    pub ctx_used: u32,
    pub ctx_max: u32,
    pub cores: u32,
    pub generating: bool,
}

const MARGIN: i32 = 24;
const STATUS_H: i32 = 40;
const INPUT_H: i32 = 46;

/// Draw the chat screen. `scroll` is how many lines the view is scrolled
/// up from the newest; the clamped value is returned so callers can keep
/// their scroll state within range.
#[allow(clippy::too_many_arguments)] // flat screen-state API is deliberate
pub fn draw(
    surf: &mut Surface,
    fonts: &Fonts,
    turns: &[Turn],
    input: &str,
    caret: usize,
    cursor_on: bool,
    stats: &Stats,
    scroll: usize,
) -> usize {
    surf.clear(theme::BG_DEEP);
    let content_col = "USER:".len().max(stats.assistant.len() + 1) + 1;
    status_bar(surf, fonts, stats);
    input_bar(surf, fonts, input, caret, cursor_on, stats.generating);
    let scroll = scrollback(
        surf,
        fonts,
        turns,
        stats.generating,
        cursor_on,
        scroll,
        stats.assistant,
        content_col,
    );
    draw::scanlines(surf, 26);
    scroll
}

/// Lines the screen can show at once (for page-sized scroll steps).
pub fn page_lines(surf: &Surface, fonts: &Fonts) -> usize {
    let line_h = fonts.body.height as i32 + 2;
    (((surf.height as i32 - INPUT_H - 10) - (STATUS_H + 10)) / line_h).max(1) as usize
}

fn status_bar(surf: &mut Surface, fonts: &Fonts, stats: &Stats) {
    let w = surf.width as i32;
    surf.fill_rect(0, 0, w, STATUS_H, theme::BG_PANEL);
    // Sunset separator line.
    for x in 0..w {
        let c = nr_gfx::color::gradient(
            &[
                (0, theme::NEON_CYAN),
                (500, theme::NEON_MAGENTA),
                (1000, theme::NEON_ORANGE),
            ],
            x as u32 * 1000 / w.max(1) as u32,
        );
        surf.fill_rect(x, STATUS_H - 2, 1, 2, c);
    }

    let f = &fonts.small;
    let ty = (STATUS_H - f.height as i32) / 2 - 1;
    draw::text_glow(
        surf,
        f,
        MARGIN,
        ty,
        "NIGHTRUN",
        1,
        2,
        0,
        theme::NEON_MAGENTA,
        2,
    );
    draw::text(surf, f, MARGIN, ty, "NIGHTRUN", theme::TEXT_PRIMARY, 1, 2);

    let model_x = MARGIN + draw::text_width(f, "NIGHTRUN", 1, 2) + 28;
    let mut buf = String::new();
    buf.push_str("// ");
    buf.push_str(stats.model);
    draw::text(surf, f, model_x, ty, &buf, theme::TEXT_DIM, 1, 0);

    // Right side: cores | memory | context | prefill | generation rate.
    let mut right = String::new();
    fmt_u32(&mut right, stats.cores);
    right.push_str("c  ");
    fmt_mem(&mut right, stats.mem_used_mb, stats.mem_total_mb);
    right.push_str("  ctx ");
    fmt_u32(&mut right, stats.ctx_used);
    right.push('/');
    fmt_u32(&mut right, stats.ctx_max);
    right.push_str("  ");
    if stats.pp_milli > 0 {
        right.push_str("pp ");
        fmt_milli(&mut right, stats.pp_milli);
        right.push_str("  ");
    }
    if stats.ftl_ms > 0 {
        right.push_str("ftl ");
        fmt_u32(&mut right, stats.ftl_ms / 1000);
        right.push('.');
        fmt_u32(&mut right, (stats.ftl_ms % 1000) / 100);
        right.push_str("s  ");
    }
    if stats.tok_s_milli > 0 {
        fmt_milli(&mut right, stats.tok_s_milli);
        right.push_str(" tok/s");
    } else if stats.generating {
        right.push_str("... tok/s");
    } else {
        right.push_str("idle");
    }
    let rx = surf.width as i32 - MARGIN - draw::text_width(f, &right, 1, 0);
    let rate_col = if stats.generating {
        theme::NEON_YELLOW
    } else {
        theme::TEXT_DIM
    };
    draw::text(surf, f, rx, ty, &right, rate_col, 1, 0);
}

fn input_bar(
    surf: &mut Surface,
    fonts: &Fonts,
    input: &str,
    caret: usize,
    cursor_on: bool,
    generating: bool,
) {
    let w = surf.width as i32;
    let h = surf.height as i32;
    let y0 = h - INPUT_H;
    surf.fill_rect(0, y0, w, INPUT_H, theme::BG_PANEL);
    surf.fill_rect(0, y0, w, 1, theme::BORDER);

    let f = &fonts.body;
    let jp = fonts.japanese_body();
    let ty = y0 + (INPUT_H - f.height as i32) / 2;
    if generating {
        draw::text(
            surf,
            f,
            MARGIN,
            ty,
            ">> generating - press ESC to stop",
            theme::TEXT_DIM,
            1,
            0,
        );
        return;
    }
    // Input follows the label after a single standard space (unlike the
    // transcript, which aligns to the shared content column).
    draw::text(surf, f, MARGIN, ty, "USER:", theme::NEON_CYAN, 1, 0);
    let tx = MARGIN + ("USER:".len() as i32 + 1) * f.width as i32;

    // Window the text so the caret is always visible; the block cursor
    // sits ON the caret cell (classic terminal), overlaying the char there.
    let chars: Vec<char> = input.chars().collect();
    let caret = caret.min(chars.len());
    let available = (w - tx - MARGIN).max(f.width as i32) as usize;
    let cursor_width = chars
        .get(caret)
        .map(|ch| draw::glyph_width(f, jp, *ch))
        .unwrap_or(f.width);
    let mut start = caret;
    let mut left_width = 0;
    while start > 0 {
        let next_width = draw::glyph_width(f, jp, chars[start - 1]);
        if left_width + next_width + cursor_width > available {
            break;
        }
        left_width += next_width;
        start -= 1;
    }
    let mut end = caret;
    let mut right_width = 0;
    while end < chars.len() {
        let next_width = draw::glyph_width(f, jp, chars[end]);
        if left_width + cursor_width + right_width + next_width > available {
            break;
        }
        right_width += next_width;
        end += 1;
    }
    let shown: String = chars[start..end].iter().collect();
    draw::text_with_fallback(surf, f, jp, tx, ty, &shown, theme::TEXT_PRIMARY, 1, 0);
    if cursor_on {
        let prefix: String = chars[start..caret].iter().collect();
        let cx = tx + draw::text_width_with_fallback(f, jp, &prefix, 1, 0);
        surf.fill_rect(
            cx,
            ty + 2,
            cursor_width as i32 - 2,
            f.height as i32 - 4,
            theme::NEON_CYAN,
        );
        if caret < chars.len() {
            // Character under the cursor, inverted.
            let under: String = chars[caret..caret + 1].iter().collect();
            draw::text_with_fallback(surf, f, jp, cx, ty, &under, theme::BG_DEEP, 1, 0);
        }
    }
}

#[allow(clippy::too_many_arguments)] // flat screen-state API is deliberate
fn scrollback(
    surf: &mut Surface,
    fonts: &Fonts,
    turns: &[Turn],
    generating: bool,
    cursor_on: bool,
    scroll: usize,
    assistant: &str,
    content_col: usize,
) -> usize {
    let f = &fonts.body;
    let jp = fonts.japanese_body();
    let w = surf.width as i32;
    let h = surf.height as i32;
    let available_width = (w - 2 * MARGIN).max(f.width as i32) as usize;
    let top = STATUS_H + 10;
    let bottom = h - INPUT_H - 10;

    // All message text starts at one fixed content column, regardless of
    // which label (user/llama/qwen/granite/none) precedes it.
    let _ = assistant;
    let content_width = available_width
        .saturating_sub(content_col * f.width)
        .max(f.width);

    // Wrap all turns into (label_color, body_color, label, body, at_margin)
    // records; continuation lines carry an empty label. System text (the
    // intro/help/status lines) starts at the label column, not the content
    // column.
    let mut lines: Vec<(u32, u32, String, String, bool)> = Vec::new();
    for (i, turn) in turns.iter().enumerate() {
        if i > 0 {
            lines.push((0, 0, String::new(), String::new(), false)); // separator
        }
        let label = match turn.role {
            Role::User => String::from("USER:"),
            Role::Llama => {
                let mut p = assistant.to_ascii_uppercase();
                p.push(':');
                p
            }
            Role::System => String::new(),
        };
        // System text keeps its accent color on every line; user/assistant
        // bodies render in primary.
        let (body_color, at_margin, width) = match turn.role {
            Role::System => (turn.role.color(), true, available_width),
            _ => (theme::TEXT_PRIMARY, false, content_width),
        };
        let mut first = true;
        for line in wrap(&turn.text, width, f, jp) {
            let l = if first { label.clone() } else { String::new() };
            lines.push((turn.role.color(), body_color, l, line, at_margin));
            first = false;
        }
        if first {
            // Empty turn (streaming just started): show the bare label.
            lines.push((
                turn.role.color(),
                body_color,
                label,
                String::new(),
                at_margin,
            ));
        }
    }

    // Bottom-anchored window, shifted up by `scroll` lines.
    let line_h = f.height as i32 + 2;
    let max_lines = ((bottom - top) / line_h).max(1) as usize;
    let max_scroll = lines.len().saturating_sub(max_lines);
    let scroll = scroll.min(max_scroll);
    let end = lines.len() - scroll;
    let start = end.saturating_sub(max_lines);
    let mut y = top;
    let body_x = MARGIN + content_col as i32 * f.width as i32;
    for (label_color, body_color, label, body, at_margin) in &lines[start..end] {
        if !label.is_empty() {
            draw::text_with_fallback(surf, f, jp, MARGIN, y, label, *label_color, 1, 0);
        }
        if !body.is_empty() {
            let x = if *at_margin { MARGIN } else { body_x };
            draw::text_with_fallback(surf, f, jp, x, y, body, *body_color, 1, 0);
        }
        y += line_h;
    }

    // While waiting for the FIRST token, a blinking block sits right after
    // the model label ("GRANITE █" — thinking). It disappears the moment
    // text starts streaming; the streamed output carries no cursor. The
    // input field shows the generating notice for the whole duration, and
    // the cursor returns there when the reply completes.
    let thinking = generating
        && turns
            .last()
            .is_some_and(|t| t.role == Role::Llama && t.text.is_empty());
    if thinking && cursor_on && scroll == 0 && end > start {
        let cy = y - line_h;
        if cy + line_h <= bottom {
            surf.fill_rect(
                body_x,
                cy + 2,
                f.width as i32 - 2,
                f.height as i32 - 4,
                theme::NEON_PINK,
            );
        }
    }

    // History indicator while scrolled up.
    if scroll > 0 {
        let f = &fonts.small;
        let mut label = String::from("^ history  ");
        fmt_u32(&mut label, scroll as u32);
        label.push_str(" lines below - DOWN for newest");
        let tw = draw::text_width(f, &label, 1, 0);
        let x = surf.width as i32 - MARGIN - tw - 12;
        surf.fill_rect(x - 8, top, tw + 16, f.height as i32 + 8, theme::BG_PANEL);
        surf.fill_rect(x - 8, top, tw + 16, 1, theme::BORDER);
        surf.fill_rect(x - 8, top + f.height as i32 + 7, tw + 16, 1, theme::BORDER);
        draw::text(surf, f, x, top + 4, &label, theme::NEON_CYAN, 1, 0);
    }
    scroll
}

/// Word wrap to a pixel width.
fn wrap(
    text: &str,
    max_width: usize,
    font: &nr_gfx::PsfFont,
    fallback: Option<&nr_gfx::PsfFont>,
) -> Vec<String> {
    // Degenerate surfaces must not loop forever in push_word; render nothing.
    if max_width == 0 {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let line_width = draw::text_width_with_fallback(font, fallback, &line, 1, 0) as usize;
            if line_width == 0 {
                push_word(&mut out, &mut line, word, max_width, font, fallback);
            } else if line_width
                + font.width
                + draw::text_width_with_fallback(font, fallback, word, 1, 0) as usize
                <= max_width
            {
                line.push(' ');
                line.push_str(word);
            } else {
                out.push(core::mem::take(&mut line));
                push_word(&mut out, &mut line, word, max_width, font, fallback);
            }
        }
        out.push(line);
    }
    out
}

/// Push a word onto the current line, hard-splitting by rendered width.
fn push_word(
    out: &mut Vec<String>,
    line: &mut String,
    word: &str,
    max_width: usize,
    font: &nr_gfx::PsfFont,
    fallback: Option<&nr_gfx::PsfFont>,
) {
    let mut chunk = String::new();
    let mut chunk_width = 0;
    for ch in word.chars() {
        let char_width = draw::glyph_width(font, fallback, ch);
        if !chunk.is_empty() && chunk_width + char_width > max_width {
            out.push(core::mem::take(&mut chunk));
            chunk_width = 0;
        }
        chunk.push(ch);
        chunk_width += char_width;
    }
    line.push_str(&chunk);
}

fn fmt_mem(out: &mut String, used: u32, total: u32) {
    fmt_u32(out, used);
    out.push('/');
    fmt_u32(out, total);
    out.push_str(" MB");
}

fn fmt_u32(out: &mut String, v: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    let mut v = v;
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    out.push_str(core::str::from_utf8(&buf[i..]).unwrap());
}

fn fmt_milli(out: &mut String, milli: u32) {
    fmt_u32(out, milli / 1000);
    out.push('.');
    fmt_u32(out, (milli % 1000) / 100);
}
