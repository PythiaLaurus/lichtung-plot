//! Opaque terminal output handle.

use std::borrow::Cow;
use std::path::Path;

use crate::scene::RenderedScene;
use crate::Error;
#[cfg(feature = "png")]
use crate::{PngCompression, PngOptions, PngSizing};

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
    svg_text: SvgText,
}

impl Output {
    pub(crate) const fn new(scene: RenderedScene, svg_text: SvgText) -> Self {
        Self { scene, svg_text }
    }

    /// Publish SVG output atomically to one path.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = match self.svg_text {
            SvgText::Selectable => Cow::Borrowed(self.scene.svg_bytes()),
            SvgText::Paths => Cow::Owned(outlined_svg(&self.scene)?),
        };
        write_atomic(path.as_ref(), "svg", &bytes)?;
        Ok(self)
    }

    /// Publish a 300-DPI balanced-compression PNG atomically to one path.
    #[cfg(feature = "png")]
    pub fn png(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.png_with_options(path, PngOptions::default())
    }

    /// Publish a source-sized balanced-compression PNG without density metadata.
    #[cfg(feature = "png")]
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.png_with_options(
            path,
            PngOptions {
                sizing: PngSizing::Source,
                compression: PngCompression::Balanced,
            },
        )
    }

    /// Publish a PNG atomically using the exact caller-provided options.
    #[cfg(feature = "png")]
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: PngOptions,
    ) -> Result<Self, Error> {
        let bytes = self.scene.png_bytes(&options)?;
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
