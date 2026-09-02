//! Plot terminal output rendering.

use std::path::Path;

use crate::builder::Plot;
use crate::render::svg_backend::SvgBackend;
use crate::scene::RenderedScene;
use crate::{Error, Output};

impl Plot {
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        let plan = super::lifecycle::PlotPlan::new(self, self.kind())?;
        let (width, height) = plan.dimensions();
        let mut backend =
            SvgBackend::in_memory_with_fonts(width, height, plan.font_environment()?.clone());
        plan.render(&mut backend)?;
        RenderedScene::new(width, height, backend.into_output())
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    /// Render once, publish an outlined SVG, and return a reusable output.
    /// The output filename is completed with `.svg` after conversion.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?);
        output.svg(path)
    }

    /// Render once, publish SVG with selectable text, and return a reusable output.
    /// The output filename is completed with `.svg` after rendering.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?);
        output.svg_preserve_text(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?);
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?);
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?);
        output.png_with_options(path, options)
    }
}
