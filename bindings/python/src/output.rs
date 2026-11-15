//! Frozen outputs publish through the renderer without retaining Python owners.

use std::path::{Path, PathBuf};

use lichtung_plot::{kind, Canvas, Output, Plot, PngOptions};
use pyo3::prelude::*;

use crate::values::PyPngOptions;

#[derive(Clone, Copy)]
pub(crate) enum Terminal {
    Svg,
    SvgPreserveText,
    Png,
    PngPreview,
    PngWithOptions(PngOptions),
}

impl Terminal {
    pub(crate) fn plot<K: kind::Marker>(
        self,
        plot: &Plot<K>,
        path: &Path,
    ) -> lichtung_plot::Result<Output> {
        match self {
            Self::Svg => plot.svg(path),
            Self::SvgPreserveText => plot.svg_preserve_text(path),
            Self::Png => plot.png(path),
            Self::PngPreview => plot.png_preview(path),
            Self::PngWithOptions(options) => plot.png_with_options(path, options),
        }
    }

    pub(crate) fn canvas(self, canvas: &Canvas, path: &Path) -> lichtung_plot::Result<Output> {
        match self {
            Self::Svg => canvas.svg(path),
            Self::SvgPreserveText => canvas.svg_preserve_text(path),
            Self::Png => canvas.png(path),
            Self::PngPreview => canvas.png_preview(path),
            Self::PngWithOptions(options) => canvas.png_with_options(path, options),
        }
    }

    fn output(self, output: &Output, path: &Path) -> lichtung_plot::Result<Output> {
        match self {
            Self::Svg => output.svg(path),
            Self::SvgPreserveText => output.svg_preserve_text(path),
            Self::Png => output.png(path),
            Self::PngPreview => output.png_preview(path),
            Self::PngWithOptions(options) => output.png_with_options(path, options),
        }
    }
}

#[pyclass(frozen, name = "Output", module = "lichtung_plot._native")]
pub(crate) struct PyOutput {
    native: Output,
}

impl PyOutput {
    pub(crate) fn new(native: Output) -> Self {
        Self { native }
    }

    fn publish(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        terminal: Terminal,
    ) -> PyResult<Self> {
        // fspath and all other Python conversion precede the native snapshot.
        let path = path.extract::<PathBuf>()?;
        let native = slf.get().native.clone();
        slf.py()
            .detach(move || terminal.output(&native, &path))
            .map(Self::new)
            .map_err(crate::native_error)
    }
}

#[pymethods]
impl PyOutput {
    #[getter]
    fn width(&self) -> u32 {
        self.native.width()
    }

    #[getter]
    fn height(&self) -> u32 {
        self.native.height()
    }

    fn copy(&self) -> Self {
        Self::new(self.native.clone())
    }

    fn svg(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::publish(slf, path, Terminal::Svg)
    }

    fn svg_preserve_text(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::publish(slf, path, Terminal::SvgPreserveText)
    }

    fn png(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::publish(slf, path, Terminal::Png)
    }

    fn png_preview(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::publish(slf, path, Terminal::PngPreview)
    }

    fn png_with_options(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        options: PyRef<'_, PyPngOptions>,
    ) -> PyResult<Self> {
        let native = options.native;
        drop(options);
        Self::publish(slf, path, Terminal::PngWithOptions(native))
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyOutput>()
}
