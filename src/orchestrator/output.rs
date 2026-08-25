//! Plot terminal output rendering.

use std::path::Path;

use crate::builder::Plot;
use crate::render::svg_backend::SvgBackend;
use crate::scene::RenderedScene;
use crate::{Error, Output};

impl Plot {
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        self.preflight_erased()?;
        let mut backend = SvgBackend::in_memory(self.width(), self.height());
        self.render_with_backend_erased(&mut backend)?;
        Ok(RenderedScene::new(
            self.width(),
            self.height(),
            backend.into_bytes()?,
        ))
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    /// Render once, publish SVG immediately, and return an opaque reusable output.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?, plot.output_svg_text());
        output.svg(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    #[cfg(feature = "png")]
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?, plot.output_svg_text());
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    #[cfg(feature = "png")]
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?, plot.output_svg_text());
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    #[cfg(feature = "png")]
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let plot = crate::kind::into_erased(self);
        let output = Output::new(plot.render_scene()?, plot.output_svg_text());
        output.png_with_options(path, options)
    }
}
