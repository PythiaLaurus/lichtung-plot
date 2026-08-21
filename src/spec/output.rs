//! Output-format options shared by plot renderers and exporters.

/// CSS pixel density used by SVG renderers.
#[cfg(feature = "png")]
pub(crate) const SVG_CSS_DPI: f32 = 96.0;

/// PNG export options for rasterizing the canonical SVG scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PngOptions {
    /// How the output PNG pixel dimensions are chosen.
    pub sizing: PngSizing,
    /// PNG encoder compression preset.
    pub compression: PngCompression,
}

impl Default for PngOptions {
    fn default() -> Self {
        Self {
            sizing: PngSizing::Source,
            compression: PngCompression::Default,
        }
    }
}

/// Mutually exclusive PNG sizing modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PngSizing {
    /// Preserve the SVG canvas pixel dimensions.
    Source,
    /// Render at a print-oriented DPI, treating SVG CSS pixels as 96 DPI.
    /// Also writes matching DPI metadata to the PNG file.
    Dpi(f32),
    /// Multiply SVG canvas width and height by this scale factor.
    Scale(f32),
    /// Set the output width in pixels and preserve aspect ratio.
    Width(u32),
    /// Set the output height in pixels and preserve aspect ratio.
    Height(u32),
}

/// High-level PNG compression presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PngCompression {
    /// Use the encoder's default balanced compression.
    #[default]
    Default,
    /// Do not compress. Fastest, largest files.
    None,
    /// Prefer encoding speed over file size.
    Fast,
    /// Balance encoding speed and file size.
    Balanced,
    /// Spend more time for smaller files.
    High,
}
