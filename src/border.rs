use crate::color::Color;

/// How to draw the rounded-rectangle frame around the canvas.
#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle {
    /// No border.
    None,

    /// Solid rounded-rect stroke.
    Solid {
        /// Stroke thickness in pixels.
        thickness: f32,
        /// Corner radius in pixels.
        radius: f32,
        /// Stroke color.
        color: Color,
    },

    /// Vignette that fades from `color` at the canvas edge inward over `width` pixels.
    ///
    /// The `color.a` value controls the peak alpha at the very edge. Alpha reaches
    /// zero at `width` pixels inward using a quadratic ease-out falloff.
    Vignette {
        /// How far inward (in pixels) the fade extends before reaching full transparency.
        width: f32,
        /// Corner radius in pixels.
        radius: f32,
        /// Edge color (alpha = peak alpha at the very edge).
        color: Color,
    },
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::None
    }
}

/// Signed-distance-field distance from pixel center `(px, py)` to the nearest point
/// on the edge of a rounded rectangle.
///
/// The rectangle is axis-aligned, centered at `(cx, cy)` with half-extents `(hw, hh)`
/// and corner radius `r`.
///
/// - Returns **negative** values inside the rounded rect.
/// - Returns **zero** on the boundary.
/// - Returns **positive** values outside.
///
/// Uses the Inigo Quilez box-SDF formulation.
pub(crate) fn rrect_sdf(
    px: f32,
    py: f32,
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    r: f32,
) -> f32 {
    // Translate to box-centered coordinates and fold to first quadrant.
    let qx = (px - cx).abs() - hw + r;
    let qy = (py - cy).abs() - hh + r;

    // Standard SDF for a rounded rect:
    //   outside corners: Euclidean distance to corner circle centre − r
    //   inside + on-axis: max(qx, qy) − r  (negative inside)
    f32::hypot(qx.max(0.0), qy.max(0.0)) + qx.min(0.0).max(qy.min(0.0)) - r
}

/// Cubic smoothstep: maps `t` from `[edge0, edge1]` to `[0, 1]` with smooth ends.
#[inline]
pub(crate) fn smoothstep(edge0: f32, edge1: f32, t: f32) -> f32 {
    let t = ((t - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdf_center_is_negative() {
        // Centre of a 144×144 canvas should be well inside → negative distance.
        let d = rrect_sdf(72.0, 72.0, 72.0, 72.0, 72.0, 72.0, 8.0);
        assert!(d < 0.0, "centre should be inside (d={d})");
    }

    #[test]
    fn sdf_outside_is_positive() {
        // Far outside the canvas → positive.
        let d = rrect_sdf(200.0, 200.0, 72.0, 72.0, 72.0, 72.0, 8.0);
        assert!(d > 0.0, "outside should be positive (d={d})");
    }

    #[test]
    fn sdf_edge_near_zero() {
        // A point exactly on the straight edge of the rect (not at a corner).
        // With hw=72, hh=72, r=8: the straight edge is at x=0 or x=144.
        // At x=0.5, y=72 we should be very close to 0 (just inside).
        let d = rrect_sdf(0.5, 72.0, 72.0, 72.0, 72.0, 72.0, 8.0);
        assert!(d.abs() < 1.0, "edge should be near zero (d={d})");
    }

    #[test]
    fn smoothstep_clamps() {
        assert_eq!(smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 2.0), 1.0);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }
}
