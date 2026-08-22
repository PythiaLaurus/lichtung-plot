//! Opaque terminal output handle.

use std::path::Path;

use crate::scene::RenderedScene;
use crate::{Error, PngOptions};

/// SVG text representation selected before the first terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SvgText {
    /// Preserve selectable and searchable SVG text elements.
    #[default]
    Selectable,
    /// Convert every SVG text element to a vector path.
    Paths,
}

/// Opaque handle to one scene that has already been rendered.
///
/// Each consuming output method publishes its requested file immediately and
/// returns the same handle so another format can reuse the exact scene.
pub struct Output {
    scene: RenderedScene,
    #[cfg(feature = "png")]
    png_options: PngOptions,
    svg_text: SvgText,
}

impl Output {
    pub(crate) const fn new(
        scene: RenderedScene,
        png_options: PngOptions,
        svg_text: SvgText,
    ) -> Self {
        #[cfg(not(feature = "png"))]
        let _ = png_options;
        Self {
            scene,
            #[cfg(feature = "png")]
            png_options,
            svg_text,
        }
    }

    /// Publish SVG output atomically to one path.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = match self.svg_text {
            SvgText::Selectable => self.scene.svg_bytes().to_vec(),
            SvgText::Paths => outlined_svg(&self.scene)?,
        };
        write_atomic(path.as_ref(), "svg", &bytes)?;
        Ok(self)
    }

    /// Publish PNG output atomically to one path.
    #[cfg(feature = "png")]
    pub fn png(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = self.scene.png_bytes(&self.png_options)?;
        write_atomic(path.as_ref(), "png", &bytes)?;
        Ok(self)
    }
}

#[cfg(feature = "text-as-path")]
fn outlined_svg(scene: &RenderedScene) -> Result<Vec<u8>, Error> {
    scene.svg_with_text_as_paths()
}

#[cfg(not(feature = "text-as-path"))]
fn outlined_svg(_scene: &RenderedScene) -> Result<Vec<u8>, Error> {
    Err(Error::Render(
        "SvgText::Paths requires the text-as-path feature".into(),
    ))
}

fn write_atomic(path: &Path, extension: &str, bytes: &[u8]) -> Result<(), Error> {
    let staged = crate::orchestrator::reserve_unique_sibling_temp_path(path, extension)?;
    crate::orchestrator::with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, bytes).map_err(Error::Io)?;
        crate::orchestrator::publish_staged(&staged, path)
    })
}
