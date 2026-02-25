use std::{collections::HashMap, path::Path, sync::Arc};

use ab_glyph::FontArc;

use crate::error::RenderError;

/// A cheap-to-clone handle to a loaded font.
///
/// Internally backed by an `Arc`, so cloning is O(1) and the font data is shared.
#[derive(Debug, Clone)]
pub struct FontHandle(pub(crate) Arc<FontArc>);

impl FontHandle {
    pub(crate) fn arc(&self) -> &FontArc {
        &self.0
    }
}

/// Stores named fonts. Load fonts once at startup, then retrieve handles by name.
///
/// # Example
/// ```rust,ignore
/// use streamdeck_render::FontRegistry;
///
/// let mut fonts = FontRegistry::new();
/// let font = fonts.load_bytes("mono", include_bytes!("../fonts/MyFont.ttf")).unwrap();
/// ```
#[derive(Default, Clone)]
pub struct FontRegistry {
    fonts: HashMap<String, FontHandle>,
}

impl FontRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a font from a static byte slice (e.g. via `include_bytes!`).
    pub fn load_bytes(
        &mut self,
        name: impl Into<String>,
        bytes: &'static [u8],
    ) -> Result<FontHandle, RenderError> {
        let font = FontArc::try_from_slice(bytes)?;
        let handle = FontHandle(Arc::new(font));
        self.fonts.insert(name.into(), handle.clone());
        Ok(handle)
    }

    /// Register a font from an owned `Vec<u8>` (e.g. read from disk at runtime).
    pub fn load_vec(
        &mut self,
        name: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Result<FontHandle, RenderError> {
        let font = FontArc::try_from_vec(bytes)?;
        let handle = FontHandle(Arc::new(font));
        self.fonts.insert(name.into(), handle.clone());
        Ok(handle)
    }

    /// Load a font from a file path and register it under `name`.
    pub fn load_file(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<FontHandle, RenderError> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref).map_err(|e| RenderError::FontLoadIo {
            path: path_ref.display().to_string(),
            source: e,
        })?;
        let name = name.into();
        self.load_vec(name, bytes)
    }

    /// Retrieve a previously registered font by name.
    pub fn get(&self, name: &str) -> Option<FontHandle> {
        self.fonts.get(name).cloned()
    }

    /// Retrieve a font by name, returning an error if not found.
    pub fn require(&self, name: &str) -> Result<FontHandle, RenderError> {
        self.get(name)
            .ok_or_else(|| RenderError::FontNotFound(name.to_string()))
    }
}
