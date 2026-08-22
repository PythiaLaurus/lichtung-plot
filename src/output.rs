//! Opaque terminal output handle.

use std::borrow::Cow;
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
            SvgText::Selectable => Cow::Borrowed(self.scene.svg_bytes()),
            SvgText::Paths => Cow::Owned(outlined_svg(&self.scene)?),
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

    /// Transactionally publish SVG and PNG outputs from the same scene.
    #[cfg(feature = "png")]
    pub fn svg_and_png(
        self,
        svg_path: impl AsRef<Path>,
        png_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let svg_path = svg_path.as_ref();
        let png_path = png_path.as_ref();
        if crate::orchestrator::paths_resolve_to_same_destination(svg_path, png_path)? {
            return Err(Error::Data(
                "SVG and PNG output paths must be distinct".into(),
            ));
        }
        let svg = match self.svg_text {
            SvgText::Selectable => Cow::Borrowed(self.scene.svg_bytes()),
            SvgText::Paths => Cow::Owned(outlined_svg(&self.scene)?),
        };
        let png = self.scene.png_bytes(&self.png_options)?;
        let (staged_svg, staged_png) =
            crate::orchestrator::reserve_staging_pair(svg_path, "svg", png_path, "png")?;
        crate::orchestrator::with_temporary_artifacts(
            vec![staged_svg.clone(), staged_png.clone()],
            || {
                std::fs::write(&staged_svg, &svg).map_err(Error::Io)?;
                std::fs::write(&staged_png, &png).map_err(Error::Io)?;
                crate::orchestrator::publish_staged_pair([
                    (&staged_svg, svg_path),
                    (&staged_png, png_path),
                ])
            },
        )?;
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
