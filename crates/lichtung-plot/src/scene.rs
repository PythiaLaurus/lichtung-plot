//! Immutable canonical SVG scenes and optional format conversion.

use std::fmt;
use std::sync::OnceLock;

use crate::render::svg_backend::{FinalizedSvgOutput, SvgOutlinePlan};
use crate::{Error, PngOptions};

const MAX_EXPORT_DIMENSION: u32 = 32_768;
const MAX_EXPORT_PIXELS: u64 = 100_000_000;
const MAX_EXPORT_PIXELS_F64: f64 = 100_000_000.0;
const MAX_CONVERSION_SCENE_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
thread_local! {
    pub(crate) static SCENE_CONSTRUCTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static PNG_TARGET_VALIDATION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// One internal fully rendered SVG scene shared by terminal outputs.
pub(crate) struct RenderedScene {
    size: CanonicalSceneSize,
    svg: Vec<u8>,
    outline_plan: SvgOutlinePlan,
    outlined_svg: OnceLock<Result<Vec<u8>, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalSceneSize {
    dimensions: resvg::tiny_skia::IntSize,
}

impl fmt::Debug for RenderedScene {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedScene")
            .field("width_css_px", &self.size.dimensions.width())
            .field("height_css_px", &self.size.dimensions.height())
            .field("svg_bytes", &self.svg.len())
            .field(
                "outline_replacements",
                &self.outline_plan.replacement_count(),
            )
            .field("outlined_svg_cached", &self.outlined_svg.get().is_some())
            .finish()
    }
}

impl RenderedScene {
    pub(crate) fn new(
        width_css_px: u32,
        height_css_px: u32,
        output: Option<FinalizedSvgOutput>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(calls.get() + 1));
        let size = CanonicalSceneSize {
            dimensions: resvg::tiny_skia::IntSize::from_wh(width_css_px, height_css_px)
                .ok_or_else(|| {
                    Error::Render("canonical scene dimensions must be positive".into())
                })?,
        };
        let output = output.ok_or_else(|| {
            Error::Render("in-memory SVG backend did not finish rendering".into())
        })?;
        let (output_width, output_height, svg, outline_plan) = output.into_parts();
        if (output_width, output_height) != (width_css_px, height_css_px) {
            return Err(Error::Render(format!(
                "finalized SVG dimensions {output_width}x{output_height} do not match canonical scene dimensions {width_css_px}x{height_css_px}"
            )));
        }
        Ok(Self {
            size,
            svg,
            outline_plan,
            outlined_svg: OnceLock::new(),
        })
    }

    pub(crate) fn dimensions(&self) -> (u32, u32) {
        (self.size.dimensions.width(), self.size.dimensions.height())
    }

    /// Borrow the exact canonical SVG bytes.
    #[must_use]
    pub(crate) fn svg_bytes(&self) -> &[u8] {
        &self.svg
    }

    /// Return SVG bytes with every text element converted to vector paths.
    ///
    /// The returned bytes are derived from immutable planned glyphs and the
    /// scene's retained font faces and contain no selectable text.
    ///
    /// # Errors
    ///
    /// Returns a render failure if immutable planned glyph outlining cannot be
    /// completed from the retained scene state.
    pub(crate) fn svg_with_text_as_paths(&self) -> Result<Vec<u8>, Error> {
        let outlined = self.outlined_svg_bytes()?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(outlined.len())
            .map_err(|error| Error::Render(format!("outlined SVG copy reservation: {error}")))?;
        bytes.extend_from_slice(outlined);
        Ok(bytes)
    }

    /// Rasterize the scene to deterministic PNG bytes with explicit options.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid or excessive sizing, planned-glyph failures,
    /// SVG parsing failures, allocation failure, or PNG encoding failure.
    pub(crate) fn png_bytes(&self, options: &PngOptions) -> Result<Vec<u8>, Error> {
        self.png_bytes_with_parser_limit(options, MAX_CONVERSION_SCENE_BYTES)
    }

    fn png_bytes_with_parser_limit(
        &self,
        options: &PngOptions,
        parser_limit: usize,
    ) -> Result<Vec<u8>, Error> {
        self.validate_png_source_limits()?;
        let target = crate::render::svg_backend::PngTargetPlan::new(self.size.dimensions, options)?;
        let bytes = match self.outlined_svg.get() {
            Some(Ok(bytes)) => bytes.len(),
            Some(Err(error)) => return Err(Error::Render(error.clone())),
            None => self
                .outline_plan
                .count_parser_bytes(
                    self.size.dimensions.width(),
                    self.size.dimensions.height(),
                    parser_limit,
                )
                .map_err(Error::Render)?,
        };
        validate_parser_bytes(bytes, parser_limit)?;
        let outlined = self
            .outlined_svg
            .get_or_init(|| {
                self.outline_plan.derive_counted(
                    self.size.dimensions.width(),
                    self.size.dimensions.height(),
                    bytes,
                )
            })
            .as_deref()
            .map_err(|error| Error::Render(error.clone()))?;
        crate::render::svg_backend::rasterize_svg_bytes_to_png(outlined, target, options)
    }

    fn validate_png_source_limits(&self) -> Result<(), Error> {
        let width_css_px = self.size.dimensions.width();
        let height_css_px = self.size.dimensions.height();
        if width_css_px > MAX_EXPORT_DIMENSION || height_css_px > MAX_EXPORT_DIMENSION {
            return Err(Error::Render(format!(
                "PNG scene dimensions {width_css_px}x{height_css_px} exceed the {MAX_EXPORT_DIMENSION}-pixel conversion limit"
            )));
        }
        let pixels = u64::from(width_css_px) * u64::from(height_css_px);
        if pixels > MAX_EXPORT_PIXELS {
            return Err(Error::Render(format!(
                "PNG scene work of {pixels} CSS pixels exceeds the {MAX_EXPORT_PIXELS}-pixel conversion limit"
            )));
        }
        Ok(())
    }

    fn outlined_svg_bytes(&self) -> Result<&[u8], Error> {
        self.outlined_svg
            .get_or_init(|| {
                self.outline_plan
                    .derive(self.size.dimensions.width(), self.size.dimensions.height())
            })
            .as_deref()
            .map_err(|error| Error::Render(error.clone()))
    }
}

pub(crate) fn validate_png_parser_bytes(bytes: usize) -> Result<(), Error> {
    validate_parser_bytes(bytes, MAX_CONVERSION_SCENE_BYTES)
}

fn validate_parser_bytes(bytes: usize, limit: usize) -> Result<(), Error> {
    if bytes > limit {
        return Err(Error::Render(format!(
            "PNG SVG parser input size exceeds the {limit}-byte conversion limit"
        )));
    }
    Ok(())
}

pub(crate) fn validate_png_target_dimensions(width: f64, height: f64) -> Result<(), Error> {
    #[cfg(test)]
    PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(calls.get() + 1));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn direct_scene(width: u32, height: u32, byte_count: usize) -> RenderedScene {
        RenderedScene {
            size: CanonicalSceneSize {
                dimensions: resvg::tiny_skia::IntSize::from_wh(width, height).unwrap(),
            },
            svg: vec![0; byte_count],
            outline_plan: SvgOutlinePlan::Failed("not used by source-limit tests".into()),
            outlined_svg: OnceLock::new(),
        }
    }

    #[test]
    fn png_source_limits_keep_exact_axis_work_and_byte_boundaries() {
        assert!(
            direct_scene(MAX_EXPORT_DIMENSION, 1, MAX_CONVERSION_SCENE_BYTES)
                .validate_png_source_limits()
                .is_ok()
        );
        assert!(direct_scene(MAX_EXPORT_DIMENSION + 1, 1, 1)
            .validate_png_source_limits()
            .unwrap_err()
            .to_string()
            .contains("scene dimensions"));
        assert!(direct_scene(10_000, 10_000, 1)
            .validate_png_source_limits()
            .is_ok());
        assert!(direct_scene(10_000, 10_001, 1)
            .validate_png_source_limits()
            .unwrap_err()
            .to_string()
            .contains("scene work"));
        assert!(direct_scene(1, 1, MAX_CONVERSION_SCENE_BYTES + 1)
            .validate_png_source_limits()
            .is_ok());
        assert!(validate_png_parser_bytes(MAX_CONVERSION_SCENE_BYTES).is_ok());
        assert!(validate_png_parser_bytes(MAX_CONVERSION_SCENE_BYTES + 1).is_err());
    }
}
