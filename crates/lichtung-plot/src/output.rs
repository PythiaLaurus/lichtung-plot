//! Opaque terminal output handle.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use crate::scene::RenderedScene;
use crate::{Error, PngCompression, PngOptions, PngSizing};

/// Opaque handle to one scene that has already been rendered.
///
/// Each consuming output method publishes its requested file immediately and
/// returns the same handle so another format can reuse the exact scene.
///
/// Terminal methods complete the final native filename with their conventional
/// lowercase suffix. A filename already ending in the complete expected suffix
/// under ASCII case-insensitive comparison is preserved exactly; otherwise the
/// suffix is appended literally. Other filename text is never interpreted or
/// replaced, so `plot.v1` becomes `plot.v1.svg` and `plot.png` passed to
/// [`Output::svg`] becomes `plot.png.svg`.
///
/// Suffix completion happens only after format conversion succeeds. The caller
/// path must have a filename and must not currently identify a directory. Its
/// parent must already exist.
pub struct Output {
    scene: RenderedScene,
}

impl Output {
    pub(crate) const fn new(scene: RenderedScene) -> Self {
        Self { scene }
    }

    /// Frozen canonical scene width in integral CSS pixels.
    ///
    /// This is the SVG root width, independent of PNG raster sizing. Inspecting
    /// it does not measure, render, or alter the retained scene.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.scene.dimensions().0
    }

    /// Frozen canonical scene height in integral CSS pixels.
    ///
    /// This is the SVG root height, independent of PNG raster sizing. All
    /// chained publications retain the same canonical dimensions.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.scene.dimensions().1
    }

    /// Publish a self-contained outlined SVG atomically.
    ///
    /// After text outlining succeeds, the final native filename is completed
    /// with `.svg`. A complete ASCII case-insensitive match such as `.SVG` is
    /// preserved exactly; otherwise `.svg` is appended without interpreting an
    /// existing extension.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        #[cfg(test)]
        let timer = crate::orchestrator::measurements::Timer::new(5);
        let bytes = self.scene.svg_with_text_as_paths()?;
        #[cfg(test)]
        drop(timer);
        let path = complete_suffix(path.as_ref(), ".svg")?;
        crate::orchestrator::write_atomic(&path, &bytes)?;
        Ok(self)
    }

    /// Publish SVG with selectable and searchable text atomically.
    ///
    /// After borrowing the canonical SVG bytes, the final native filename is
    /// completed with `.svg` using the literal rule documented on [`Output`].
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = self.scene.svg_bytes();
        let path = complete_suffix(path.as_ref(), ".svg")?;
        crate::orchestrator::write_atomic(&path, bytes)?;
        Ok(self)
    }

    /// Publish a 300-DPI balanced-compression PNG atomically.
    ///
    /// The final native filename is completed with `.png` by
    /// [`Output::png_with_options`] after rasterization succeeds.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.png_with_options(path, PngOptions::default())
    }

    /// Publish a source-sized balanced-compression PNG without density metadata.
    ///
    /// The final native filename is completed with `.png` by
    /// [`Output::png_with_options`] after rasterization succeeds.
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
    ///
    /// After rasterization succeeds, the final native filename is completed
    /// with `.png`. A complete ASCII case-insensitive match such as `.PNG` is
    /// preserved exactly; otherwise `.png` is appended without interpreting an
    /// existing extension.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: PngOptions,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        let timer = crate::orchestrator::measurements::Timer::new(6);
        let bytes = self.scene.png_bytes(&options)?;
        #[cfg(test)]
        drop(timer);
        let path = complete_suffix(path.as_ref(), ".png")?;
        crate::orchestrator::write_atomic(&path, &bytes)?;
        Ok(self)
    }
}

fn complete_suffix(path: &Path, expected: &str) -> Result<PathBuf, Error> {
    if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "render output path {} identifies a directory",
                path.display()
            ),
        )
        .into());
    }
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("render output path {} has no file name", path.display()),
        )
    })?;
    let encoded = file_name.as_encoded_bytes();
    if encoded
        .get(encoded.len().saturating_sub(expected.len())..)
        .is_some_and(|ending| ending.eq_ignore_ascii_case(expected.as_bytes()))
    {
        return Ok(path.to_path_buf());
    }

    let mut completed = OsString::from(file_name);
    completed.push(expected);
    Ok(path.with_file_name(completed))
}
