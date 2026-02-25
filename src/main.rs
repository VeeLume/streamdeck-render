use std::path::PathBuf;

use clap::Parser;
use streamdeck_render::{
    BorderStyle, Canvas, Color, FontRegistry, TextOptions, WrapOptions, wrap_text,
};

/// Render a Stream Deck button icon with custom text and a TrueType/OpenType font.
///
/// The output is a transparent-background PNG sized for Stream Deck key icons.
/// Use `--bg-color` to add a solid background, and `--border` for visual framing.
#[derive(Parser)]
#[command(name = "streamdeck-render", version, about, long_about = None)]
struct Cli {
    /// Path to a TrueType (.ttf) or OpenType (.otf) font file.
    #[arg(short, long)]
    font: PathBuf,

    /// Text to render. Use literal `\n` (backslash-n) for explicit line breaks.
    #[arg(short, long)]
    text: String,

    /// Output PNG file path.
    #[arg(short, long)]
    output: PathBuf,

    /// Font size in pixels.
    #[arg(long, default_value_t = 28.0)]
    size: f32,

    /// Canvas dimensions as WxH (e.g. `144x144` or `72x72`).
    #[arg(long, default_value = "144x144")]
    canvas: String,

    /// Text color as `#RRGGBB` or `#RRGGBBAA`.
    #[arg(long, default_value = "#ffffff")]
    color: String,

    /// Background fill color as `#RRGGBB` or `#RRGGBBAA`.
    /// Omit for a fully transparent background.
    #[arg(long)]
    bg_color: Option<String>,

    /// Maximum number of lines for word-wrap.
    #[arg(long, default_value_t = 3)]
    max_lines: usize,

    /// Border style: `none`, `solid`, or `vignette`.
    #[arg(long, default_value = "none")]
    border: String,

    /// Border color as `#RRGGBB` or `#RRGGBBAA`.
    #[arg(long, default_value = "#ffffffff")]
    border_color: String,

    /// Stroke thickness in pixels (only used when `--border solid`).
    #[arg(long, default_value_t = 4.0)]
    border_thickness: f32,

    /// Corner radius in pixels (used by both solid and vignette borders).
    #[arg(long, default_value_t = 8.0)]
    border_radius: f32,

    /// Vignette fade width in pixels (only used when `--border vignette`).
    #[arg(long, default_value_t = 10.0)]
    vignette_width: f32,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    // ── Parse canvas dimensions ───────────────────────────────────────────────
    let (w, h) = parse_canvas(&cli.canvas).ok_or_else(|| {
        anyhow::anyhow!(
            "invalid canvas size '{}' — expected WxH (e.g. 144x144)",
            cli.canvas
        )
    })?;

    // ── Parse colors ──────────────────────────────────────────────────────────
    let text_color = Color::from_hex(&cli.color)
        .ok_or_else(|| anyhow::anyhow!("invalid text color '{}'", cli.color))?;

    let border_color = Color::from_hex(&cli.border_color)
        .ok_or_else(|| anyhow::anyhow!("invalid border color '{}'", cli.border_color))?;

    let bg_color = cli
        .bg_color
        .as_deref()
        .map(|s| Color::from_hex(s).ok_or_else(|| anyhow::anyhow!("invalid bg-color '{s}'")))
        .transpose()?;

    // ── Load font ─────────────────────────────────────────────────────────────
    let mut fonts = FontRegistry::new();
    let font = fonts.load_file("main", &cli.font).map_err(|e| {
        anyhow::anyhow!("failed to load font '{}': {e}", cli.font.display())
    })?;

    // ── Build canvas ──────────────────────────────────────────────────────────
    let mut canvas = Canvas::new(w, h);

    if let Some(bg) = bg_color {
        canvas.fill(bg);
    }

    // ── Wrap and render text ──────────────────────────────────────────────────
    // Replace literal "\n" sequences in the CLI argument with real newlines,
    // then treat each resulting line as a hard-break boundary.
    let text = cli.text.replace("\\n", "\n");
    let wrap_opts = WrapOptions {
        max_width: w as f32 - 14.0, // 7px padding each side
        max_lines: cli.max_lines,
    };

    // Process hard line breaks: split on '\n', wrap each segment independently,
    // then concatenate all resulting lines.
    let lines: Vec<_> = text
        .split('\n')
        .flat_map(|segment| wrap_text(&font, cli.size, segment, &wrap_opts))
        .collect();

    canvas
        .draw_text(&lines, &TextOptions::new(font, cli.size).color(text_color))
        .map_err(|e| anyhow::anyhow!("text rendering failed: {e}"))?;

    // ── Border ────────────────────────────────────────────────────────────────
    let border_style = match cli.border.as_str() {
        "none" => BorderStyle::None,
        "solid" => BorderStyle::Solid {
            thickness: cli.border_thickness,
            radius: cli.border_radius,
            color: border_color,
        },
        "vignette" => BorderStyle::Vignette {
            width: cli.vignette_width,
            radius: cli.border_radius,
            color: border_color,
        },
        other => anyhow::bail!("unknown border style '{other}' — choose none, solid, or vignette"),
    };
    canvas.draw_border(&border_style);

    // ── Save ──────────────────────────────────────────────────────────────────
    let rendered = canvas.finish();
    rendered
        .save(&cli.output)
        .map_err(|e| anyhow::anyhow!("failed to save '{}': {e}", cli.output.display()))?;

    println!(
        "Saved {}×{} icon to '{}'",
        rendered.width(),
        rendered.height(),
        cli.output.display()
    );

    Ok(())
}

fn parse_canvas(s: &str) -> Option<(u32, u32)> {
    let (ws, hs) = s.split_once('x')?;
    let w = ws.trim().parse().ok()?;
    let h = hs.trim().parse().ok()?;
    Some((w, h))
}
