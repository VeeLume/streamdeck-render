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
    ///
    /// The result is suitable for direct use with `streamdeck-lib`'s
    /// `SdClient::set_image_b64(context, base64_string)`.
    pub fn to_base64(&self) -> Result<String, RenderError> {
        Ok(BASE64_STANDARD.encode(self.to_png_bytes()?))
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
