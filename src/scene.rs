//! Immutable canonical SVG scenes and optional format conversion.

use std::fmt;

#[cfg(feature = "text-as-path")]
use crate::Error;

#[cfg(feature = "png")]
use crate::PngOptions;

#[cfg(feature = "png")]
const MAX_EXPORT_DIMENSION: u32 = 32_768;
#[cfg(feature = "png")]
const MAX_EXPORT_PIXELS: u64 = 100_000_000;
#[cfg(feature = "png")]
const MAX_EXPORT_PIXELS_F64: f64 = 100_000_000.0;
#[cfg(feature = "png")]
const MAX_CONVERSION_SCENE_BYTES: usize = 64 * 1024 * 1024;

/// One internal fully rendered SVG scene shared by terminal outputs.
pub(crate) struct RenderedScene {
    width_css_px: u32,
    height_css_px: u32,
    svg: Box<[u8]>,
}

impl fmt::Debug for RenderedScene {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedScene")
            .field("width_css_px", &self.width_css_px)
            .field("height_css_px", &self.height_css_px)
            .field("svg_bytes", &self.svg.len())
            .finish()
    }
}

impl RenderedScene {
    pub(crate) fn new(width_css_px: u32, height_css_px: u32, svg: Vec<u8>) -> Self {
        Self {
            width_css_px,
            height_css_px,
            svg: svg.into_boxed_slice(),
        }
    }

    /// Borrow the exact canonical SVG bytes.
    #[must_use]
    pub(crate) fn svg_bytes(&self) -> &[u8] {
        &self.svg
    }

    /// Return SVG bytes with every text element converted to vector paths.
    ///
    /// The conversion uses the first available family from the renderer's
    /// system-font policy. The returned SVG is self-contained but no longer
    /// contains selectable or searchable text.
    ///
    /// # Errors
    ///
    /// Returns an error when no system font is available, the canonical SVG
    /// cannot be parsed, its dimensions change, or any text remains after
    /// conversion.
    #[cfg(feature = "text-as-path")]
    pub(crate) fn svg_with_text_as_paths(&self) -> Result<Vec<u8>, Error> {
        crate::render::svg_backend::convert_svg_text_to_paths(
            &self.svg,
            self.width_css_px,
            self.height_css_px,
        )
    }

    /// Rasterize the scene to deterministic PNG bytes with explicit options.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid or excessive sizing, unavailable rendering
    /// fonts, SVG parsing failures, allocation failure, or PNG encoding failure.
    #[cfg(feature = "png")]
    pub(crate) fn png_bytes(&self, options: &PngOptions) -> Result<Vec<u8>, Error> {
        self.validate_conversion_input("PNG")?;
        crate::render::svg_backend::rasterize_svg_bytes_to_png(
            &self.svg,
            self.width_css_px,
            self.height_css_px,
            options,
        )
    }

    #[cfg(feature = "png")]
    fn validate_conversion_input(&self, format: &str) -> Result<(), Error> {
        if self.width_css_px == 0 || self.height_css_px == 0 {
            return Err(Error::Render(format!(
                "{format} scene dimensions must be positive"
            )));
        }
        if self.width_css_px > MAX_EXPORT_DIMENSION || self.height_css_px > MAX_EXPORT_DIMENSION {
            return Err(Error::Render(format!(
                "{format} scene dimensions {}x{} exceed the {MAX_EXPORT_DIMENSION}-pixel conversion limit",
                self.width_css_px, self.height_css_px
            )));
        }
        let pixels = u64::from(self.width_css_px) * u64::from(self.height_css_px);
        if pixels > MAX_EXPORT_PIXELS {
            return Err(Error::Render(format!(
                "{format} scene work of {pixels} CSS pixels exceeds the {MAX_EXPORT_PIXELS}-pixel conversion limit"
            )));
        }
        if self.svg.len() > MAX_CONVERSION_SCENE_BYTES {
            return Err(Error::Render(format!(
                "{format} SVG scene size exceeds the {MAX_CONVERSION_SCENE_BYTES}-byte conversion limit"
            )));
        }
        Ok(())
    }
}

#[cfg(feature = "png")]
pub(crate) fn validate_png_target_dimensions(width: f64, height: f64) -> Result<(), Error> {
    if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
        return Err(Error::Render(format!(
            "computed PNG dimensions must be positive and finite; got {width}x{height}"
        )));
    }
    if width > f64::from(MAX_EXPORT_DIMENSION) || height > f64::from(MAX_EXPORT_DIMENSION) {
        return Err(Error::Render(format!(
            "computed PNG dimensions {width}x{height} exceed the {MAX_EXPORT_DIMENSION}-pixel conversion limit"
        )));
    }
    if width * height > MAX_EXPORT_PIXELS_F64 {
        return Err(Error::Render(format!(
            "computed PNG work exceeds the {MAX_EXPORT_PIXELS}-pixel conversion limit"
        )));
    }
    Ok(())
}
