use ab_glyph::{Font, PxScale, ScaleFont};
use image::{ImageBuffer, Rgba, RgbaImage};

use crate::{
    border::{BorderStyle, rrect_sdf, smoothstep},
    color::Color,
    error::RenderError,
    font::FontHandle,
    layout::TextLine,
    output::RenderedImage,
};

/// Vertical alignment of the text block within the canvas.
#[derive(Debug, Clone, Copy, Default)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
    /// Place the first baseline at an absolute pixel Y coordinate.
    ///
    /// Useful for precise multi-group layouts where two `draw_text` calls at
    /// different font sizes need to align to specific positions.
    Baseline(f32),
}

/// Horizontal alignment of each text line within the canvas.
#[derive(Debug, Clone, Copy, Default)]
pub enum HAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Options controlling how text is rendered onto the canvas.
#[derive(Debug, Clone)]
pub struct TextOptions {
    pub font: FontHandle,
    /// Font size in pixels (passed as `PxScale`).
    pub size: f32,
    pub color: Color,
    pub h_align: HAlign,
    pub v_align: VAlign,
    /// Extra pixels of vertical spacing added between lines on top of the
    /// font's natural line gap.
    pub line_gap: f32,
}

impl TextOptions {
    /// Create options with white, centered text at the given size.
    pub fn new(font: FontHandle, size: f32) -> Self {
        Self {
            font,
            size,
            color: Color::WHITE,
            h_align: HAlign::Center,
            v_align: VAlign::Center,
            line_gap: 0.0,
        }
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn h_align(mut self, a: HAlign) -> Self {
        self.h_align = a;
        self
    }

    pub fn v_align(mut self, a: VAlign) -> Self {
        self.v_align = a;
        self
    }

    pub fn line_gap(mut self, g: f32) -> Self {
        self.line_gap = g;
        self
    }
}

/// An RGBA canvas for compositing text and border effects.
///
/// Create with [`Canvas::new`], [`Canvas::key_icon`], or [`Canvas::key_icon_standard`],
/// then call methods to render content, and finally call [`Canvas::finish`] to get the
/// encoded [`RenderedImage`].
pub struct Canvas {
    buf: RgbaImage,
    width: u32,
    height: u32,
}

impl Canvas {
    /// Create a new transparent canvas of the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let buf = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));
        Self { buf, width, height }
    }

    /// 144×144 high-DPI Stream Deck key icon (recommended for modern hardware).
    pub fn key_icon() -> Self {
        Self::new(144, 144)
    }

    /// 72×72 standard Stream Deck key icon.
    pub fn key_icon_standard() -> Self {
        Self::new(72, 72)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Fill the entire canvas with a solid color.
    ///
    /// Use [`Color::TRANSPARENT`] to reset to a fully transparent background.
    pub fn fill(&mut self, color: Color) {
        for pixel in self.buf.pixels_mut() {
            *pixel = Rgba([color.r, color.g, color.b, color.a]);
        }
    }

    /// Render pre-wrapped lines of text onto the canvas.
    ///
    /// Lines are laid out according to `opts.h_align` and `opts.v_align`.
    /// Glyphs are composited using Porter-Duff "source over destination".
    pub fn draw_text(&mut self, lines: &[TextLine], opts: &TextOptions) -> Result<(), RenderError> {
        if lines.is_empty() {
            return Ok(());
        }

        let scale = PxScale::from(opts.size);
        let sf = opts.font.arc().as_scaled(scale);

        let ascent = sf.ascent();
        let descent = sf.descent(); // negative
        let font_line_gap = sf.line_gap();
        let line_h = ascent - descent + font_line_gap + opts.line_gap;

        let n = lines.len() as f32;
        // Total height of the text block: N lines of ascent+descent, (N-1) gaps.
        let total_h = ascent - descent + (n - 1.0) * line_h;

        let w = self.width as f32;
        let h = self.height as f32;

        // Y of the first baseline.
        let first_baseline_y = match opts.v_align {
            VAlign::Top => ascent,
            VAlign::Center => (h - total_h) / 2.0 + ascent,
            VAlign::Bottom => h - (total_h - ascent),
            VAlign::Baseline(y) => y,
        };

        for (i, line) in lines.iter().enumerate() {
            let baseline_y = first_baseline_y + i as f32 * line_h;

            let start_x = match opts.h_align {
                HAlign::Left => 0.0,
                HAlign::Center => (w - line.width_px) / 2.0,
                HAlign::Right => w - line.width_px,
            };

            draw_text_line(
                &mut self.buf,
                &line.text,
                &opts.font,
                scale,
                start_x,
                baseline_y,
                opts.color,
                self.width,
                self.height,
            );
        }

        Ok(())
    }

    /// Draw a rounded-rectangle border effect over the canvas.
    ///
    /// Uses the SDF from [`crate::border::rrect_sdf`] for smooth anti-aliasing.
    pub fn draw_border(&mut self, style: &BorderStyle) {
        match style {
            BorderStyle::None => {}
            BorderStyle::Solid { thickness, radius, color } => {
                self.draw_solid_border(*thickness, *radius, *color);
            }
            BorderStyle::Vignette { width, radius, color } => {
                self.draw_vignette_border(*width, *radius, *color);
            }
        }
    }

    /// Draw a 1px horizontal line across the canvas at pixel row `y`.
    ///
    /// Useful for separators in multi-section button layouts.
    pub fn draw_horizontal_line(&mut self, y: u32, color: Color) {
        if y >= self.height {
            return;
        }
        for px in 0..self.width {
            let pixel = self.buf.get_pixel_mut(px, y);
            composite_over(pixel, color, color.a as f32 / 255.0);
        }
    }

    /// Consume the canvas and return a [`RenderedImage`] ready for encoding.
    pub fn finish(self) -> RenderedImage {
        RenderedImage { buf: self.buf }
    }

    // ── private helpers ─────────────────────────────────────────────────────

    fn draw_solid_border(&mut self, thickness: f32, radius: f32, color: Color) {
        let w = self.width as f32;
        let h = self.height as f32;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let hw = w / 2.0;
        let hh = h / 2.0;

        for py in 0..self.height {
            for px in 0..self.width {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dist = rrect_sdf(fx, fy, cx, cy, hw, hh, radius);

                // Paint the band [−thickness, 0] (from inner edge to outer edge).
                // AA: 1px smooth transition on each side.
                let outer_aa = smoothstep(1.0, 0.0, dist);           // 1 just inside, 0 outside
                let inner_aa = smoothstep(-thickness - 1.0, -thickness, dist); // 0 deep inside, 1 at inner edge

                let alpha = outer_aa * inner_aa * (color.a as f32 / 255.0);
                if alpha > 0.0 {
                    let pixel = self.buf.get_pixel_mut(px, py);
                    composite_over(pixel, color, alpha);
                }
            }
        }
    }

    fn draw_vignette_border(&mut self, width: f32, radius: f32, color: Color) {
        let w = self.width as f32;
        let h = self.height as f32;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let hw = w / 2.0;
        let hh = h / 2.0;

        let peak_alpha = color.a as f32 / 255.0;

        for py in 0..self.height {
            for px in 0..self.width {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dist = rrect_sdf(fx, fy, cx, cy, hw, hh, radius);

                // Only pixels inside the shape (dist < 0) are painted.
                if dist >= 1.0 {
                    continue;
                }

                // inset: how far inside the boundary we are (0 at edge, positive inside)
                let inset = -dist;
                // Clamp to vignette width
                if inset > width {
                    continue;
                }

                // Outer shape AA (handles the 1px anti-aliased edge of the rounded rect itself)
                let shape_aa = smoothstep(1.0, 0.0, dist);

                // Quadratic falloff: bright at edge (t=0), zero at width (t=1)
                let t = (inset / width).clamp(0.0, 1.0);
                let falloff = (1.0 - t) * (1.0 - t);

                let alpha = peak_alpha * falloff * shape_aa;
                if alpha > 0.0 {
                    let pixel = self.buf.get_pixel_mut(px, py);
                    composite_over(pixel, color, alpha);
                }
            }
        }
    }
}

/// Porter-Duff "source over destination" compositing.
///
/// `src_alpha` is the pre-multiplied effective alpha of the source (already in `[0,1]`).
#[inline]
fn composite_over(dst: &mut Rgba<u8>, src_color: Color, src_alpha: f32) {
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_alpha + dst_a * (1.0 - src_alpha);

    if out_a <= 0.0 {
        return;
    }

    let blend = |src_c: u8, dst_c: u8| -> u8 {
        let s = src_c as f32 / 255.0;
        let d = dst_c as f32 / 255.0;
        ((s * src_alpha + d * dst_a * (1.0 - src_alpha)) / out_a * 255.0).round() as u8
    };

    dst[0] = blend(src_color.r, dst[0]);
    dst[1] = blend(src_color.g, dst[1]);
    dst[2] = blend(src_color.b, dst[2]);
    dst[3] = (out_a * 255.0).round() as u8;
}

/// Rasterize a single line of text into `img` at the given baseline position.
#[allow(clippy::too_many_arguments)]
fn draw_text_line(
    img: &mut RgbaImage,
    text: &str,
    font: &FontHandle,
    scale: PxScale,
    start_x: f32,
    baseline_y: f32,
    color: Color,
    img_w: u32,
    img_h: u32,
) {
    let sf = font.arc().as_scaled(scale);
    let mut cursor_x = start_x;
    let mut prev = None;

    for ch in text.chars() {
        let glyph_id = sf.glyph_id(ch);
        if let Some(prev_id) = prev {
            cursor_x += sf.kern(prev_id, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(
            scale,
            ab_glyph::point(cursor_x, baseline_y),
        );
        cursor_x += sf.h_advance(glyph_id);
        prev = Some(glyph_id);

        if let Some(og) = font.arc().outline_glyph(glyph) {
            let bounds = og.px_bounds();
            og.draw(|dx, dy, coverage| {
                let px = bounds.min.x as i32 + dx as i32;
                let py = bounds.min.y as i32 + dy as i32;
                if px >= 0 && py >= 0 && (px as u32) < img_w && (py as u32) < img_h {
                    let cov = coverage.clamp(0.0, 1.0);
                    if cov > 0.0 {
                        let pixel = img.get_pixel_mut(px as u32, py as u32);
                        composite_over(pixel, color, cov * (color.a as f32 / 255.0));
                    }
                }
            });
        }
    }
}
