//! Plot terminal output rendering.

use std::path::Path;

use crate::builder::Plot;
use crate::render::svg_backend::SvgBackend;
use crate::scene::RenderedScene;
use crate::{Error, Output};

impl Plot {
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        self.preflight()?;
        let mut backend = SvgBackend::in_memory(self.width(), self.height());
        self.render_with_backend(&mut backend)?;
        Ok(RenderedScene::new(
            self.width(),
            self.height(),
            backend.into_bytes()?,
        ))
    }

    /// Render once, publish SVG immediately, and return an opaque reusable output.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let (png_options, svg_text) = self.output_options();
        let output = Output::new(self.render_scene()?, png_options, svg_text);
        output.svg(path)
    }

    /// Render once, publish PNG immediately, and return an opaque reusable output.
    #[cfg(feature = "png")]
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let (png_options, svg_text) = self.output_options();
        let output = Output::new(self.render_scene()?, png_options, svg_text);
        output.png(path)
    }

    /// Render once and transactionally publish SVG and PNG outputs.
    #[cfg(feature = "png")]
    pub fn svg_and_png(
        self,
        svg_path: impl AsRef<Path>,
        png_path: impl AsRef<Path>,
    ) -> Result<Output, Error> {
        let (png_options, svg_text) = self.output_options();
        let output = Output::new(self.render_scene()?, png_options, svg_text);
        output.svg_and_png(svg_path, png_path)
    }
}
