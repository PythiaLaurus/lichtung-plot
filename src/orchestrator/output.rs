//! Chart output staging, rendering, rasterization, and publication.

use std::path::Path;

use crate::Error;

#[cfg(feature = "png")]
use super::artifacts::{
    paths_resolve_to_same_destination, publish_staged_pair, reserve_staging_pair,
};
use super::artifacts::{
    publish_staged, reserve_unique_sibling_temp_path, with_temporary_artifacts,
};
use crate::builder::Chart;
use crate::render::svg_backend::SvgBackend;
use crate::scene::RenderedSvgScene;
#[cfg(feature = "png")]
use crate::spec::PngOptions;

impl Chart<'_> {
    /// Render one immutable canonical SVG scene without creating a file.
    ///
    /// # Errors
    ///
    /// Returns an error when chart preflight or SVG rendering fails.
    pub fn render_scene(&self) -> Result<RenderedSvgScene, Error> {
        self.preflight()?;
        let mut backend = SvgBackend::in_memory(self.width(), self.height());
        self.render_with_backend(&mut backend)?;
        Ok(RenderedSvgScene::new(
            self.width(),
            self.height(),
            backend.into_bytes()?,
        ))
    }

    /// Render the canonical SVG scene to owned bytes without creating a file.
    ///
    /// # Errors
    ///
    /// Returns an error when chart preflight or SVG rendering fails.
    pub fn render_to_svg_bytes(&self) -> Result<Vec<u8>, Error> {
        self.render_scene().map(RenderedSvgScene::into_svg_bytes)
    }

    /// Render to an SVG file at `path`. Per `02_CRATES.md` §2.4
    /// public API surface.
    pub fn render_to_svg(&self, path: &Path) -> Result<(), Error> {
        let scene = self.render_scene()?;
        let staged_svg = reserve_unique_sibling_temp_path(path, "svg")?;
        with_temporary_artifacts(vec![staged_svg.clone()], || {
            std::fs::write(&staged_svg, scene.svg_bytes()).map_err(Error::Io)?;
            publish_staged(&staged_svg, path)
        })
    }

    /// Render the canonical scene to default-sized PNG bytes.
    #[cfg(feature = "png")]
    pub fn render_to_png_bytes(&self) -> Result<Vec<u8>, Error> {
        self.render_to_png_bytes_with_options(&PngOptions::default())
    }

    /// Render the canonical scene to PNG bytes with explicit rasterization options.
    #[cfg(feature = "png")]
    pub fn render_to_png_bytes_with_options(&self, options: &PngOptions) -> Result<Vec<u8>, Error> {
        self.render_scene()?.png_bytes(options)
    }

    /// Render to a PNG file at `path`. Per `02_CRATES.md` §2.4
    /// public API surface. Requires the `png` feature; the PNG
    /// is rasterized from the native SVG renderer through
    /// `resvg` + `tiny-skia`.
    #[cfg(feature = "png")]
    pub fn render_to_png(&self, path: &Path) -> Result<(), Error> {
        self.render_to_png_with_options(path, &PngOptions::default())
    }

    /// Render to a PNG file with explicit rasterization options.
    #[cfg(feature = "png")]
    pub fn render_to_png_with_options(
        &self,
        path: &Path,
        options: &PngOptions,
    ) -> Result<(), Error> {
        let png = self.render_to_png_bytes_with_options(options)?;
        let staged_png = reserve_unique_sibling_temp_path(path, "png")?;
        with_temporary_artifacts(vec![staged_png.clone()], || {
            std::fs::write(&staged_png, &png).map_err(Error::Io)?;
            publish_staged(&staged_png, path)
        })
    }

    /// Render once to persistent SVG and rasterize that SVG to PNG.
    ///
    /// This avoids rendering the same chart twice when callers need both
    /// vector and raster artifacts.
    #[cfg(feature = "png")]
    pub fn render_to_svg_and_png(&self, svg_path: &Path, png_path: &Path) -> Result<(), Error> {
        self.render_to_svg_and_png_with_options(svg_path, png_path, &PngOptions::default())
    }

    /// Render once to persistent SVG and rasterize that SVG to PNG with
    /// explicit rasterization options.
    #[cfg(feature = "png")]
    pub fn render_to_svg_and_png_with_options(
        &self,
        svg_path: &Path,
        png_path: &Path,
        options: &PngOptions,
    ) -> Result<(), Error> {
        if paths_resolve_to_same_destination(svg_path, png_path)? {
            return Err(Error::Data(
                "SVG and PNG output paths must be distinct".into(),
            ));
        }
        let scene = self.render_scene()?;
        let png = scene.png_bytes(options)?;
        let (staged_svg, staged_png) = reserve_staging_pair(svg_path, "svg", png_path, "png")?;
        with_temporary_artifacts(vec![staged_svg.clone(), staged_png.clone()], || {
            std::fs::write(&staged_svg, scene.svg_bytes()).map_err(Error::Io)?;
            std::fs::write(&staged_png, &png).map_err(Error::Io)?;
            publish_staged_pair([(&staged_svg, svg_path), (&staged_png, png_path)])
        })
    }
}
