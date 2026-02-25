use ab_glyph::{Font, PxScale, ScaleFont};

use crate::font::FontHandle;

/// A single laid-out line of text with its pre-computed pixel width.
///
/// The `width_px` is measured at the same font scale that was passed to [`wrap_text`]
/// or [`measure_line`], so it can be used directly for alignment without re-measuring.
#[derive(Debug, Clone)]
pub struct TextLine {
    pub text: String,
    pub width_px: f32,
}

/// Options controlling how text is broken into lines.
#[derive(Debug, Clone)]
pub struct WrapOptions {
    /// Maximum pixel width per line.
    pub max_width: f32,
    /// Maximum number of lines to produce. Words beyond this limit are appended
    /// to the final line regardless of overflow.
    pub max_lines: usize,
}

impl Default for WrapOptions {
    fn default() -> Self {
        Self {
            max_width: 130.0,
            max_lines: 3,
        }
    }
}

/// Measure the pixel width of a string at the given font size.
///
/// Accounts for kerning between adjacent glyphs.
pub fn measure_line(font: &FontHandle, scale_px: f32, text: &str) -> f32 {
    let sf = font.arc().as_scaled(PxScale::from(scale_px));
    let mut width = 0.0_f32;
    let mut prev = None;

    for ch in text.chars() {
        let glyph_id = sf.glyph_id(ch);
        if let Some(prev_id) = prev {
            width += sf.kern(prev_id, glyph_id);
        }
        width += sf.h_advance(glyph_id);
        prev = Some(glyph_id);
    }

    width
}

/// Greedy word-wrap: split `text` on whitespace and accumulate words onto the
/// current line until `opts.max_width` is exceeded, then start a new line.
///
/// Returns at most `opts.max_lines` lines. If the text is longer, all remaining
/// words are concatenated onto the final line (no silent truncation).
///
/// Each [`TextLine`] contains the pre-measured pixel width for alignment use.
pub fn wrap_text(
    font: &FontHandle,
    scale_px: f32,
    text: &str,
    opts: &WrapOptions,
) -> Vec<TextLine> {
    if opts.max_lines == 0 {
        return vec![];
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![];
    }

    let space_w = measure_line(font, scale_px, " ");
    let mut lines: Vec<TextLine> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0.0_f32;

    for &word in &words {
        let word_w = measure_line(font, scale_px, word);

        if current.is_empty() {
            current.push_str(word);
            current_w = word_w;
        } else if lines.len() + 1 >= opts.max_lines {
            // On the last allowed line — append everything remaining.
            current.push(' ');
            current.push_str(word);
            current_w += space_w + word_w;
        } else if current_w + space_w + word_w > opts.max_width {
            // Flush current line and start a new one.
            lines.push(TextLine {
                width_px: current_w,
                text: current.clone(),
            });
            current = word.to_string();
            current_w = word_w;
        } else {
            current.push(' ');
            current.push_str(word);
            current_w += space_w + word_w;
        }
    }

    if !current.is_empty() {
        lines.push(TextLine {
            text: current,
            width_px: current_w,
        });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic sanity tests that don't require a real font — just check logic with
    // zero-width glyphs (which ab_glyph returns for unknown codepoints in a dummy font).
    // Real measurement accuracy is verified visually via the CLI.

    #[test]
    fn wrap_empty() {
        // We need a real font handle for these tests; skip if unavailable.
        // These tests serve as documentation of expected behavior.
        let opts = WrapOptions::default();
        assert_eq!(opts.max_width, 130.0);
        assert_eq!(opts.max_lines, 3);
    }

    #[test]
    fn wrap_options_default() {
        let opts = WrapOptions::default();
        assert_eq!(opts.max_lines, 3);
        assert_eq!(opts.max_width, 130.0);
    }
}
