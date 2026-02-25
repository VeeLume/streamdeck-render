use std::io::Cursor;
use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use image::{ImageFormat, RgbaImage};

use crate::error::RenderError;

/// A finished, immutable rendered image ready to encode and ship.
///
/// Obtain via [`crate::Canvas::finish`].
pub struct RenderedImage {
    pub(crate) buf: RgbaImage,
}

impl RenderedImage {
    /// Encode to PNG bytes. Allocates once per call.
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, RenderError> {
        let mut out = Vec::new();
        self.buf
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)?;
        Ok(out)
    }

    /// Encode to a plain base64 string (no `data:` URI prefix).
    pub fn to_base64(&self) -> Result<String, RenderError> {
        Ok(BASE64_STANDARD.encode(self.to_png_bytes()?))
    }

    /// Encode to a full PNG data URI (`data:image/png;base64,…`).
    ///
    /// Pass the result directly to `streamdeck-lib`'s `SdClient::set_image()`.
    pub fn to_data_url(&self) -> Result<String, RenderError> {
        Ok(format!("data:image/png;base64,{}", self.to_base64()?))
    }

    /// Save the image to a file. Format is inferred from the file extension.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), RenderError> {
        self.buf.save(path).map_err(RenderError::PngEncode)
    }

    /// Image width in pixels.
    pub fn width(&self) -> u32 {
        self.buf.width()
    }

    /// Image height in pixels.
    pub fn height(&self) -> u32 {
        self.buf.height()
    }
}
