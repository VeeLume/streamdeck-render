//! Render Stream Deck button icons as PNG images with custom fonts.
//!
//! Stream Deck hardware does not support embedded fonts in SVGs, so custom-font
//! icons must be rendered as PNGs programmatically. This crate provides the
//! building blocks: font loading, text layout, a pixel canvas, and PNG/base64
//! output — sized for Stream Deck key icons (144 × 144 px high-DPI).
//!
//! # Quick start
//!
//! ```rust,ignore
//! use streamdeck_render::{Canvas, Color, FontRegistry, TextOptions, WrapOptions, wrap_text};
//!
//! let mut fonts = FontRegistry::new();
//! let font = fonts.load_bytes("sans", include_bytes!("../fonts/Inter-Regular.ttf")).unwrap();
//!
//! let mut canvas = Canvas::key_icon(); // 144×144 transparent
//!
//! let lines = wrap_text(&font, 28.0, "Hello World", &WrapOptions::default());
//! canvas.draw_text(&lines, &TextOptions::new(font, 28.0)).unwrap();
//!
//! let rendered = canvas.finish();
//!
//! // Use with streamdeck-lib:
//! // cx.sd().set_image_b64(event.context(), rendered.to_base64().unwrap());
//! ```

pub mod border;
pub mod canvas;
pub mod color;
pub mod error;
pub mod font;
pub mod layout;
pub mod output;

// Flatten the most-used items to the crate root for ergonomic imports.
pub use border::BorderStyle;
pub use canvas::{Canvas, HAlign, TextOptions, VAlign};
pub use color::Color;
pub use error::RenderError;
pub use font::{FontHandle, FontRegistry};
pub use layout::{TextLine, WrapOptions, measure_line, wrap_text};
pub use output::RenderedImage;
