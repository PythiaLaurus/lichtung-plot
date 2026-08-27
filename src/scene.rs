//! Immutable canonical SVG scenes and optional format conversion.

use std::fmt;

use crate::render::svg_backend::FinalizedSvgOutput;
use crate::{Error, PngOptions};

const MAX_EXPORT_DIMENSION: u32 = 32_768;
const MAX_EXPORT_PIXELS: u64 = 100_000_000;
const MAX_EXPORT_PIXELS_F64: f64 = 100_000_000.0;
const MAX_CONVERSION_SCENE_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
thread_local! {
    pub(crate) static SCENE_CONSTRUCTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SCENE_ROUTE_EVIDENCE: std::cell::RefCell<Vec<(&'static str, usize, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static PNG_TARGET_VALIDATION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// One internal fully rendered SVG scene shared by terminal outputs.
pub(crate) struct RenderedScene {
    size: CanonicalSceneSize,
    svg: Box<[u8]>,
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
        let (output_width, output_height, svg) = output.into_parts();
        if (output_width, output_height) != (width_css_px, height_css_px) {
            return Err(Error::Render(format!(
                "finalized SVG dimensions {output_width}x{output_height} do not match canonical scene dimensions {width_css_px}x{height_css_px}"
            )));
        }
        Ok(Self { size, svg })
    }

    /// Borrow the exact canonical SVG bytes.
    #[must_use]
    pub(crate) fn svg_bytes(&self) -> &[u8] {
        #[cfg(test)]
        SCENE_ROUTE_EVIDENCE.with(|evidence| {
            evidence
                .borrow_mut()
                .push(("preserve", self.svg.as_ptr().addr(), self.svg.len()));
        });
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
    pub(crate) fn svg_with_text_as_paths(&self) -> Result<Vec<u8>, Error> {
        #[cfg(test)]
        SCENE_ROUTE_EVIDENCE.with(|evidence| {
            evidence
                .borrow_mut()
                .push(("outline", self.svg.as_ptr().addr(), self.svg.len()));
        });
        crate::render::svg_backend::convert_svg_text_to_paths(
            &self.svg,
            self.size.dimensions.width(),
            self.size.dimensions.height(),
        )
    }

    /// Rasterize the scene to deterministic PNG bytes with explicit options.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid or excessive sizing, unavailable rendering
    /// fonts, SVG parsing failures, allocation failure, or PNG encoding failure.
    pub(crate) fn png_bytes(&self, options: &PngOptions) -> Result<Vec<u8>, Error> {
        #[cfg(test)]
        SCENE_ROUTE_EVIDENCE.with(|evidence| {
            evidence
                .borrow_mut()
                .push(("png", self.svg.as_ptr().addr(), self.svg.len()));
        });
        self.validate_png_source_limits()?;
        crate::render::svg_backend::rasterize_svg_bytes_to_png(
            &self.svg,
            self.size.dimensions,
            options,
        )
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
        if self.svg.len() > MAX_CONVERSION_SCENE_BYTES {
            return Err(Error::Render(format!(
                "PNG SVG scene size exceeds the {MAX_CONVERSION_SCENE_BYTES}-byte conversion limit"
            )));
        }
        Ok(())
    }
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
    use crate::render::backend::Backend;
    use crate::render::svg_backend::SvgBackend;

    fn finalized(width: u32, height: u32) -> FinalizedSvgOutput {
        let mut backend = SvgBackend::in_memory(width, height);
        backend.begin(width, height).unwrap();
        backend.end().unwrap();
        backend.into_output().unwrap()
    }

    fn direct_scene(width: u32, height: u32, byte_count: usize) -> RenderedScene {
        RenderedScene {
            size: CanonicalSceneSize {
                dimensions: resvg::tiny_skia::IntSize::from_wh(width, height).unwrap(),
            },
            svg: vec![0; byte_count].into_boxed_slice(),
        }
    }

    #[test]
    fn canonical_scene_rejects_incomplete_zero_and_mismatched_output() {
        assert!(RenderedScene::new(320, 240, None)
            .unwrap_err()
            .to_string()
            .contains("did not finish"));
        assert!(RenderedScene::new(0, 240, Some(finalized(320, 240)))
            .unwrap_err()
            .to_string()
            .contains("dimensions must be positive"));
        assert!(RenderedScene::new(320, 0, Some(finalized(320, 240)))
            .unwrap_err()
            .to_string()
            .contains("dimensions must be positive"));
        assert!(RenderedScene::new(321, 240, Some(finalized(320, 240)))
            .unwrap_err()
            .to_string()
            .contains("do not match canonical scene dimensions"));
    }

    #[test]
    fn canonical_scene_owns_exact_finalized_bytes() {
        let (_, _, expected) = finalized(320, 240).into_parts();
        let scene = RenderedScene::new(320, 240, Some(finalized(320, 240))).unwrap();

        assert_eq!(scene.svg_bytes(), &*expected);
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
            .unwrap_err()
            .to_string()
            .contains("scene size"));
    }

    #[test]
    fn svg_routes_bypass_png_source_limits() {
        let scene = RenderedScene::new(
            MAX_EXPORT_DIMENSION + 1,
            1,
            Some(finalized(MAX_EXPORT_DIMENSION + 1, 1)),
        )
        .unwrap();

        assert!(scene.svg_bytes().starts_with(b"<svg"));
        assert!(scene.svg_with_text_as_paths().is_ok());
        let error = scene
            .png_bytes(&PngOptions {
                sizing: crate::PngSizing::Scale(f32::NAN),
                compression: crate::PngCompression::Balanced,
            })
            .unwrap_err();
        assert!(error.to_string().contains("scene dimensions"));
    }
}
