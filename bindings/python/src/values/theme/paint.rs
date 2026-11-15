//! Complete compound paints and the native fixed/mapped outline choice.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::{is_omitted, omitted, PyColor, PyRelativeSpacing};
use crate::native_error;

fn stroke_pattern(value: &str) -> PyResult<core::StrokePattern> {
    match value {
        "solid" => Ok(core::StrokePattern::Solid),
        "short_dash" => Ok(core::StrokePattern::ShortDash),
        "medium_dash" => Ok(core::StrokePattern::MediumDash),
        "long_dash" => Ok(core::StrokePattern::LongDash),
        _ => Err(PyValueError::new_err(
            "pattern must be 'solid', 'short_dash', 'medium_dash', or 'long_dash'",
        )),
    }
}

fn stroke_pattern_name(value: core::StrokePattern) -> &'static str {
    match value {
        core::StrokePattern::Solid => "solid",
        core::StrokePattern::ShortDash => "short_dash",
        core::StrokePattern::MediumDash => "medium_dash",
        core::StrokePattern::LongDash => "long_dash",
    }
}

fn point_shape(value: &str) -> PyResult<core::PointShape> {
    match value {
        "circle" => Ok(core::PointShape::Circle),
        "triangle_up" => Ok(core::PointShape::TriangleUp),
        _ => Err(PyValueError::new_err(
            "shape must be 'circle' or 'triangle_up'",
        )),
    }
}

fn point_shape_name(value: core::PointShape) -> &'static str {
    match value {
        core::PointShape::Circle => "circle",
        core::PointShape::TriangleUp => "triangle_up",
    }
}

theme_record!(PyStrokeTheme, "StrokeTheme", StrokeTheme, {
    color: PyRef<'_, PyColor> => get_color (PyColor) [value PyColor],
    width: f32 => get_width (f32) [scalar],
    pattern: &str => get_pattern (&'static str) [choice stroke_pattern stroke_pattern_name],
});

theme_record!(PySolidStrokeTheme, "SolidStrokeTheme", SolidStrokeTheme, {
    color: PyRef<'_, PyColor> => get_color (PyColor) [value PyColor],
    width: f32 => get_width (f32) [scalar],
});

theme_record!(PyOutlineTheme, "OutlineTheme", OutlineTheme, {
    color: PyRef<'_, PyColor> => get_color (PyColor) [value PyColor],
    width: f32 => get_width (f32) [scalar],
});

theme_record!(PyPointOutlineTheme, "PointOutlineTheme", PointOutlineTheme, {
    color: PyRef<'_, PyColor> => get_color (PyColor) [value PyColor],
    width_ratio: PyRef<'_, PyRelativeSpacing> => get_width_ratio (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

#[pyclass(
    name = "DataOutlineTheme",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyDataOutlineTheme {
    pub(crate) native: core::DataOutlineTheme,
}

#[pymethods]
impl PyDataOutlineTheme {
    #[staticmethod]
    fn fixed(stroke: PyRef<'_, PyOutlineTheme>) -> Self {
        let outline = stroke.native.clone();
        drop(stroke);
        Self {
            native: core::DataOutlineTheme::fixed(outline),
        }
    }

    #[staticmethod]
    fn mapped(width: f32, alpha: u8) -> PyResult<Self> {
        Ok(Self {
            native: core::DataOutlineTheme::mapped(width, alpha).map_err(native_error)?,
        })
    }

    #[getter]
    fn kind(&self) -> &'static str {
        if self.native.fixed_stroke().is_some() {
            "fixed"
        } else {
            "mapped"
        }
    }

    #[getter]
    fn fixed_stroke(&self) -> Option<PyOutlineTheme> {
        self.native
            .fixed_stroke()
            .cloned()
            .map(|native| PyOutlineTheme { native })
    }

    #[getter]
    fn mapped_width(&self) -> Option<f32> {
        self.native.mapped_width()
    }

    #[getter]
    fn mapped_alpha(&self) -> Option<u8> {
        self.native.mapped_alpha()
    }

    #[pyo3(signature = (*, fixed_stroke = omitted(), mapped_width = omitted(), mapped_alpha = omitted()),
        text_signature = "($self, *, fixed_stroke=..., mapped_width=..., mapped_alpha=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        fixed_stroke: Py<PyAny>,
        mapped_width: Py<PyAny>,
        mapped_alpha: Py<PyAny>,
    ) -> PyResult<Self> {
        let fixed_stroke = &fixed_stroke.into_bound(py);
        let mapped_width = &mapped_width.into_bound(py);
        let mapped_alpha = &mapped_alpha.into_bound(py);
        if let Some(original) = self.native.fixed_stroke() {
            if !is_omitted(mapped_width) || !is_omitted(mapped_alpha) {
                return Err(PyValueError::new_err("fixed outline has no mapped payload"));
            }
            let stroke = if is_omitted(fixed_stroke) {
                original.clone()
            } else {
                fixed_stroke
                    .extract::<PyRef<'_, PyOutlineTheme>>()?
                    .native
                    .clone()
            };
            Ok(Self {
                native: core::DataOutlineTheme::fixed(stroke),
            })
        } else {
            if !is_omitted(fixed_stroke) {
                return Err(PyValueError::new_err("mapped outline has no fixed stroke"));
            }
            let width = if is_omitted(mapped_width) {
                self.native.mapped_width().expect("mapped branch")
            } else {
                mapped_width.extract()?
            };
            let alpha = if is_omitted(mapped_alpha) {
                self.native.mapped_alpha().expect("mapped branch")
            } else {
                mapped_alpha.extract()?
            };
            Self::mapped(width, alpha)
        }
    }
}

theme_record!(PyDataFillTheme, "DataFillTheme", DataFillTheme, {
    fill_alpha: u8 => get_fill_alpha (u8) [scalar],
    outline: Option<PyRef<'_, PyDataOutlineTheme>> = None => get_outline (Option<PyDataOutlineTheme>) [optional PyDataOutlineTheme],
});

theme_record!(PyFixedFillTheme, "FixedFillTheme", FixedFillTheme, {
    color: PyRef<'_, PyColor> => get_color (PyColor) [value PyColor],
    outline: Option<PyRef<'_, PyOutlineTheme>> = None => get_outline (Option<PyOutlineTheme>) [optional PyOutlineTheme],
});

theme_record!(PyPointTheme, "PointTheme", PointTheme, {
    fill_alpha: u8 => get_fill_alpha (u8) [scalar],
    shape: &str => get_shape (&'static str) [choice point_shape point_shape_name],
    outline: Option<PyRef<'_, PyPointOutlineTheme>> = None => get_outline (Option<PyPointOutlineTheme>) [optional PyPointOutlineTheme],
});

theme_record!(PyDataLineTheme, "DataLineTheme", DataLineTheme, {
    width: f32 => get_width (f32) [scalar],
    alpha: u8 => get_alpha (u8) [scalar],
    pattern: &str => get_pattern (&'static str) [choice stroke_pattern stroke_pattern_name],
});

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStrokeTheme>()?;
    m.add_class::<PySolidStrokeTheme>()?;
    m.add_class::<PyOutlineTheme>()?;
    m.add_class::<PyPointOutlineTheme>()?;
    m.add_class::<PyDataOutlineTheme>()?;
    m.add_class::<PyDataFillTheme>()?;
    m.add_class::<PyFixedFillTheme>()?;
    m.add_class::<PyPointTheme>()?;
    m.add_class::<PyDataLineTheme>()
}
