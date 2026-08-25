//! Opaque terminal output handle.

use std::path::Path;

use crate::scene::RenderedScene;
use crate::{Error, PngCompression, PngOptions, PngSizing};

/// Opaque handle to one scene that has already been rendered.
///
/// Each consuming output method publishes its requested file immediately and
/// returns the same handle so another format can reuse the exact scene.
pub struct Output {
    scene: RenderedScene,
}

impl Output {
    pub(crate) const fn new(scene: RenderedScene) -> Self {
        Self { scene }
    }

    /// Publish a self-contained outlined SVG atomically to one path.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = self.scene.svg_with_text_as_paths()?;
        write_atomic(path.as_ref(), "svg", &bytes)?;
        Ok(self)
    }

    /// Publish SVG with selectable and searchable text atomically to one path.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        write_atomic(path.as_ref(), "svg", self.scene.svg_bytes())?;
        Ok(self)
    }

    /// Publish a 300-DPI balanced-compression PNG atomically to one path.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.png_with_options(path, PngOptions::default())
    }

    /// Publish a source-sized balanced-compression PNG without density metadata.
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

fn write_atomic(path: &Path, extension: &str, bytes: &[u8]) -> Result<(), Error> {
    let staged = crate::orchestrator::reserve_unique_sibling_temp_path(path, extension)?;
    crate::orchestrator::with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, bytes).map_err(Error::Io)?;
        crate::orchestrator::publish_staged(&staged, path)
    })
}
