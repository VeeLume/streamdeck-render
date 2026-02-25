# streamdeck-render

Render Stream Deck button icons as PNG images with custom fonts.

Stream Deck hardware does not support embedded or referenced fonts in SVGs. To use a custom font on a button, you must render the icon as a PNG programmatically. This crate provides the primitives to do that: font loading, text layout, a pixel canvas, and PNG/base64 output sized for Stream Deck key icons.

## Features

- Load fonts from a file path or embedded bytes (`include_bytes!`)
- Measure text width and wrap words into N lines to fit a bounding box
- Render anti-aliased text onto a transparent RGBA canvas
- Solid or vignette rounded-rectangle border effects
- Encode the result to PNG bytes or a base64 string
- Includes a CLI for previewing icons without writing plugin code

## Library usage

Add to your `Cargo.toml`:

```toml
[dependencies]
streamdeck-render = { git = "https://github.com/veelume/streamdeck-render", tag = "v0.1.0" }
```

### Quick start

```rust
use streamdeck_render::{
    BorderStyle, Canvas, Color, FontRegistry, TextOptions, WrapOptions, wrap_text,
};

// Load a font once at startup.
let mut fonts = FontRegistry::new();
let font = fonts.load_file("sans", "fonts/Inter-Bold.ttf")?;

// Create a 144×144 transparent canvas (high-DPI Stream Deck key icon).
let mut canvas = Canvas::key_icon();

// Word-wrap the label to fit the canvas width.
let lines = wrap_text(&font, 28.0, "Record Active", &WrapOptions::default());

// Render white centered text.
canvas.draw_text(&lines, &TextOptions::new(font, 28.0).color(Color::WHITE))?;

// Optional: add a vignette border.
canvas.draw_border(&BorderStyle::Vignette {
    width: 10.0,
    radius: 8.0,
    color: Color::rgba(255, 255, 255, 80),
});

// Encode for use with streamdeck-lib.
let b64 = canvas.finish().to_base64()?;
// cx.sd().set_image_b64(event.context(), b64);
```

### Font loading

```rust
let mut fonts = FontRegistry::new();

// From a file path (loaded at runtime).
let font = fonts.load_file("sans", "fonts/Inter-Regular.ttf")?;

// From owned bytes (e.g. read from disk).
let bytes = std::fs::read("fonts/Inter-Bold.ttf")?;
let bold = fonts.load_vec("bold", bytes)?;

// From a static byte slice (embedded in the binary at compile time).
let mono = fonts.load_bytes("mono", include_bytes!("../fonts/Mono.ttf"))?;

// Retrieve a previously registered font by name.
let font = fonts.require("sans")?;
```

### Canvas sizes

```rust
Canvas::key_icon()          // 144×144 — high-DPI (recommended)
Canvas::key_icon_standard() // 72×72  — standard resolution
Canvas::new(200, 100)       // arbitrary size
```

### Text layout

```rust
use streamdeck_render::{WrapOptions, wrap_text};

let opts = WrapOptions {
    max_width: 120.0, // pixels
    max_lines: 3,
};

let lines = wrap_text(&font, 24.0, "Some long label text", &opts);

// Or measure a single line without wrapping:
let width_px = streamdeck_render::measure_line(&font, 24.0, "Hello");
```

### Text options

```rust
use streamdeck_render::{HAlign, TextOptions, VAlign};

let opts = TextOptions::new(font, 28.0)
    .color(Color::rgb(255, 200, 50))  // amber text
    .h_align(HAlign::Center)          // default
    .v_align(VAlign::Center)          // default
    .line_gap(4.0);                   // extra px between lines
```

### Border styles

```rust
use streamdeck_render::{BorderStyle, Color};

// No border (default).
canvas.draw_border(&BorderStyle::None);

// Solid rounded-rect stroke.
canvas.draw_border(&BorderStyle::Solid {
    thickness: 4.0,
    radius: 8.0,
    color: Color::WHITE,
});

// Vignette: fades from the edge inward over `width` pixels.
canvas.draw_border(&BorderStyle::Vignette {
    width: 12.0,
    radius: 8.0,
    color: Color::rgba(255, 255, 255, 100), // alpha = peak brightness at edge
});
```

### Colors

```rust
use streamdeck_render::Color;

Color::WHITE                      // #ffffff
Color::BLACK                      // #000000
Color::TRANSPARENT                // #00000000
Color::rgb(255, 64, 64)           // opaque red
Color::rgba(255, 64, 64, 128)     // semi-transparent red
Color::from_hex("#ff4040")        // from hex string (#RRGGBB)
Color::from_hex("#ff404080")      // from hex string (#RRGGBBAA)
```

### Output

```rust
let rendered = canvas.finish();

// Base64 string — pass directly to streamdeck-lib.
let b64: String = rendered.to_base64()?;

// Raw PNG bytes.
let bytes: Vec<u8> = rendered.to_png_bytes()?;

// Save to a file.
rendered.save("button.png")?;
```

### Integration with streamdeck-lib

`streamdeck-render` does not depend on `streamdeck-lib`. The integration point is
`SdClient::set_image_b64`, which accepts a plain base64 string:

```rust
fn on_key_down(&mut self, cx: &Context, ev: &incoming::KeyDown) {
    if let Ok(b64) = self.render_icon("Record Active") {
        cx.sd().set_image_b64(&ev.context, b64);
    }
}

fn render_icon(&self, label: &str) -> Result<String, streamdeck_render::RenderError> {
    let mut canvas = Canvas::key_icon();
    let lines = wrap_text(&self.font, 28.0, label, &WrapOptions::default());
    canvas.draw_text(&lines, &TextOptions::new(self.font.clone(), 28.0))?;
    canvas.finish().to_base64()
}
```

---

## CLI usage

```
streamdeck-render --font <PATH> --text <TEXT> --output <PATH> [OPTIONS]
```

### Required arguments

| Argument | Description |
|---|---|
| `-f, --font <PATH>` | Path to a `.ttf` or `.otf` font file |
| `-t, --text <TEXT>` | Label text. Use `\n` for explicit line breaks |
| `-o, --output <PATH>` | Output PNG file path |

### Optional arguments

| Argument | Default | Description |
|---|---|---|
| `--size <f32>` | `28.0` | Font size in pixels |
| `--canvas <WxH>` | `144x144` | Canvas dimensions |
| `--color <#hex>` | `#ffffff` | Text color |
| `--bg-color <#hex>` | *(transparent)* | Background fill color |
| `--max-lines <n>` | `3` | Max word-wrap lines |
| `--border <style>` | `none` | Border style: `none`, `solid`, or `vignette` |
| `--border-color <#hex>` | `#ffffffff` | Border color |
| `--border-thickness <f32>` | `4.0` | Stroke width (solid only) |
| `--border-radius <f32>` | `8.0` | Corner radius |
| `--vignette-width <f32>` | `10.0` | Fade width in px (vignette only) |

### Examples

Render a simple white label on a transparent background:

```sh
streamdeck-render \
  --font Inter-Bold.ttf \
  --text "Hello World" \
  --output button.png
```

Multi-line label with a dark background and vignette border:

```sh
streamdeck-render \
  --font Inter-Bold.ttf \
  --text "Record\nActive" \
  --size 30 \
  --bg-color "#1a1a1a" \
  --border vignette \
  --border-color "#ffffff50" \
  --vignette-width 14 \
  --output record-active.png
```

Solid border with amber text at 72×72:

```sh
streamdeck-render \
  --font Inter-Bold.ttf \
  --text "OBS" \
  --canvas 72x72 \
  --size 22 \
  --color "#ffcc00" \
  --border solid \
  --border-thickness 3 \
  --border-radius 6 \
  --output obs.png
```

---

## Stream Deck image specifications

| Icon type | Standard | High-DPI | Method |
|---|---|---|---|
| Key icon | 72×72 px | 144×144 px | `Canvas::key_icon()` |
| Arbitrary | any | any | `Canvas::new(w, h)` |

Key icons support full color and transparent backgrounds. The Stream Deck software
composites the icon over its own background.

For reference: [Elgato image guidelines](https://docs.elgato.com/guidelines/streamdeck/plugins/images-and-layouts)

---

## License

MIT OR Apache-2.0
