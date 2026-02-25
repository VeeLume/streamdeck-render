use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("font not found: '{0}'")]
    FontNotFound(String),

    #[error("failed to load font from '{path}': {source}")]
    FontLoadIo {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse font data: {0}")]
    FontParse(#[from] ab_glyph::InvalidFont),

    #[error("PNG encoding failed: {0}")]
    PngEncode(#[from] image::ImageError),
}
