/// RGBA color. Alpha 255 = fully opaque, 0 = fully transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    /// Parse `#RRGGBB` or `#RRGGBBAA` hex color strings.
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                let a = u8::from_str_radix(&s[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Return this color with a modified alpha value.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_rgb() {
        assert_eq!(Color::from_hex("#ffffff"), Some(Color::WHITE));
        assert_eq!(Color::from_hex("#000000"), Some(Color::BLACK));
    }

    #[test]
    fn hex_rgba() {
        assert_eq!(
            Color::from_hex("#ff000080"),
            Some(Color::rgba(255, 0, 0, 128))
        );
    }

    #[test]
    fn hex_no_hash() {
        assert_eq!(Color::from_hex("ff0000"), Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn hex_invalid() {
        assert_eq!(Color::from_hex("#zzz"), None);
        assert_eq!(Color::from_hex("#fff"), None);
    }
}
