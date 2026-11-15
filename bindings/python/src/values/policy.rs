//! Native appearance selections and exact group/significance policies.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::theme::{
    text_weight, PyDivergingGradient, PyOpaqueColor, PyQualitativePalette, PySequentialGradient,
};
use super::{is_omitted, omitted};
use crate::native_error;

fn map_entries(value: &Bound<'_, PyAny>) -> PyResult<Vec<(String, core::OpaqueColor)>> {
    let entries: Vec<(String, PyRef<'_, PyOpaqueColor>)> = value.extract()?;
    Ok(entries
        .into_iter()
        .map(|(raw, color)| {
            let native = color.native;
            drop(color);
            (raw, native)
        })
        .collect())
}

#[pyclass(
    name = "CategoricalColorMap",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyCategoricalColorMap {
    pub(crate) native: core::CategoricalColorMap,
}

#[pymethods]
impl PyCategoricalColorMap {
    #[new]
    fn new(entries: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            native: core::CategoricalColorMap::new(map_entries(entries)?).map_err(native_error)?,
        })
    }

    #[getter]
    fn entries<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let entries = self
            .native
            .entries()
            .map(|(raw, native)| Ok((raw, Py::new(py, PyOpaqueColor { native })?)))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, entries)
    }

    fn color(&self, raw: &str) -> Option<PyOpaqueColor> {
        self.native
            .color(raw)
            .map(|native| PyOpaqueColor { native })
    }

    #[pyo3(signature = (*, entries = omitted()),
        text_signature = "($self, *, entries=...)")]
    fn copy_with(&self, py: Python<'_>, entries: Py<PyAny>) -> PyResult<Self> {
        let entries = &entries.into_bound(py);
        if is_omitted(entries) {
            Ok(self.clone())
        } else {
            Self::new(entries)
        }
    }
}

#[pyclass(
    name = "CategoricalColorScale",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyCategoricalColorScale {
    pub(crate) native: core::CategoricalColorScale,
}

#[pymethods]
impl PyCategoricalColorScale {
    #[staticmethod]
    fn from_palette(palette: PyRef<'_, PyQualitativePalette>) -> Self {
        let native = palette.native.clone();
        drop(palette);
        Self {
            native: core::CategoricalColorScale::from_palette(native),
        }
    }

    #[staticmethod]
    fn from_map(map: PyRef<'_, PyCategoricalColorMap>) -> Self {
        let native = map.native.clone();
        drop(map);
        Self {
            native: core::CategoricalColorScale::from_map(native),
        }
    }

    #[getter]
    fn kind(&self) -> &'static str {
        if self.native.palette().is_some() {
            "palette"
        } else {
            "map"
        }
    }

    #[getter]
    fn palette(&self) -> Option<PyQualitativePalette> {
        self.native
            .palette()
            .cloned()
            .map(|native| PyQualitativePalette { native })
    }

    #[getter]
    fn map(&self) -> Option<PyCategoricalColorMap> {
        self.native
            .map()
            .cloned()
            .map(|native| PyCategoricalColorMap { native })
    }

    #[pyo3(signature = (*, palette = omitted(), map = omitted()),
        text_signature = "($self, *, palette=..., map=...)")]
    fn copy_with(&self, py: Python<'_>, palette: Py<PyAny>, map: Py<PyAny>) -> PyResult<Self> {
        let palette = &palette.into_bound(py);
        let map = &map.into_bound(py);
        if let Some(inherited) = self.native.palette() {
            if !is_omitted(map) {
                return Err(PyTypeError::new_err(
                    "copy_with edits only the current palette branch",
                ));
            }
            let palette = if is_omitted(palette) {
                inherited.clone()
            } else {
                palette
                    .extract::<PyRef<'_, PyQualitativePalette>>()?
                    .native
                    .clone()
            };
            Ok(Self {
                native: core::CategoricalColorScale::from_palette(palette),
            })
        } else {
            if !is_omitted(palette) {
                return Err(PyTypeError::new_err(
                    "copy_with edits only the current map branch",
                ));
            }
            let map = if is_omitted(map) {
                self.native.map().expect("native map branch").clone()
            } else {
                map.extract::<PyRef<'_, PyCategoricalColorMap>>()?
                    .native
                    .clone()
            };
            Ok(Self {
                native: core::CategoricalColorScale::from_map(map),
            })
        }
    }
}

#[pyclass(
    name = "ColorDomain",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyColorDomain {
    pub(crate) native: core::ColorDomain,
}

#[pymethods]
impl PyColorDomain {
    #[new]
    fn new(minimum: f64, maximum: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::ColorDomain::exact(minimum, maximum).map_err(native_error)?,
        })
    }

    #[getter]
    fn minimum(&self) -> f64 {
        self.native.minimum()
    }

    #[getter]
    fn maximum(&self) -> f64 {
        self.native.maximum()
    }

    #[pyo3(signature = (*, minimum = omitted(), maximum = omitted()),
        text_signature = "($self, *, minimum=..., maximum=...)")]
    fn copy_with(&self, py: Python<'_>, minimum: Py<PyAny>, maximum: Py<PyAny>) -> PyResult<Self> {
        let minimum = &minimum.into_bound(py);
        let maximum = &maximum.into_bound(py);
        Self::new(
            if is_omitted(minimum) {
                self.native.minimum()
            } else {
                minimum.extract()?
            },
            if is_omitted(maximum) {
                self.native.maximum()
            } else {
                maximum.extract()?
            },
        )
    }
}

#[pyclass(
    name = "ContinuousColorScale",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyContinuousColorScale {
    pub(crate) native: core::ContinuousColorScale,
}

#[pymethods]
impl PyContinuousColorScale {
    #[staticmethod]
    #[pyo3(signature = (*, gradient = None, domain = None))]
    fn sequential(
        gradient: Option<PyRef<'_, PySequentialGradient>>,
        domain: Option<PyRef<'_, PyColorDomain>>,
    ) -> PyResult<Self> {
        let gradient = gradient.map(|value| value.native.clone());
        let domain = domain.map(|value| value.native.clone());
        Ok(Self {
            native: core::ContinuousColorScale::sequential(gradient, domain)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (center, *, gradient = None, domain = None))]
    fn diverging(
        center: f64,
        gradient: Option<PyRef<'_, PyDivergingGradient>>,
        domain: Option<PyRef<'_, PyColorDomain>>,
    ) -> PyResult<Self> {
        let gradient = gradient.map(|value| value.native.clone());
        let domain = domain.map(|value| value.native.clone());
        Ok(Self {
            native: core::ContinuousColorScale::diverging(center, gradient, domain)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn kind(&self) -> &'static str {
        match self.native.kind() {
            core::ContinuousScaleKind::Sequential => "sequential",
            core::ContinuousScaleKind::Diverging => "diverging",
        }
    }

    #[getter]
    fn domain(&self) -> Option<PyColorDomain> {
        self.native
            .get_domain()
            .cloned()
            .map(|native| PyColorDomain { native })
    }

    #[getter]
    fn center(&self) -> Option<f64> {
        self.native.get_center()
    }

    #[getter]
    fn sequential_gradient(&self) -> Option<PySequentialGradient> {
        self.native
            .get_sequential_gradient()
            .cloned()
            .map(|native| PySequentialGradient { native })
    }

    #[getter]
    fn diverging_gradient(&self) -> Option<PyDivergingGradient> {
        self.native
            .get_diverging_gradient()
            .cloned()
            .map(|native| PyDivergingGradient { native })
    }

    #[pyo3(signature = (*, domain = omitted(), center = omitted(),
        sequential_gradient = omitted(), diverging_gradient = omitted()),
        text_signature = "($self, *, domain=..., center=..., sequential_gradient=..., diverging_gradient=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        domain: Py<PyAny>,
        center: Py<PyAny>,
        sequential_gradient: Py<PyAny>,
        diverging_gradient: Py<PyAny>,
    ) -> PyResult<Self> {
        let domain = &domain.into_bound(py);
        let center = &center.into_bound(py);
        let sequential_gradient = &sequential_gradient.into_bound(py);
        let diverging_gradient = &diverging_gradient.into_bound(py);
        match self.native.kind() {
            core::ContinuousScaleKind::Sequential => {
                if !is_omitted(center) || !is_omitted(diverging_gradient) {
                    return Err(PyTypeError::new_err(
                        "copy_with edits only the current sequential branch",
                    ));
                }
                let domain = if is_omitted(domain) {
                    self.native.get_domain().cloned()
                } else {
                    domain
                        .extract::<Option<PyRef<'_, PyColorDomain>>>()?
                        .map(|value| value.native.clone())
                };
                let gradient = if is_omitted(sequential_gradient) {
                    self.native.get_sequential_gradient().cloned()
                } else {
                    sequential_gradient
                        .extract::<Option<PyRef<'_, PySequentialGradient>>>()?
                        .map(|value| value.native.clone())
                };
                Ok(Self {
                    native: core::ContinuousColorScale::sequential(gradient, domain)
                        .map_err(native_error)?,
                })
            }
            core::ContinuousScaleKind::Diverging => {
                if !is_omitted(sequential_gradient) {
                    return Err(PyTypeError::new_err(
                        "copy_with edits only the current diverging branch",
                    ));
                }
                let domain = if is_omitted(domain) {
                    self.native.get_domain().cloned()
                } else {
                    domain
                        .extract::<Option<PyRef<'_, PyColorDomain>>>()?
                        .map(|value| value.native.clone())
                };
                let center = if is_omitted(center) {
                    self.native.get_center().expect("native diverging branch")
                } else {
                    center.extract()?
                };
                let gradient = if is_omitted(diverging_gradient) {
                    self.native.get_diverging_gradient().cloned()
                } else {
                    diverging_gradient
                        .extract::<Option<PyRef<'_, PyDivergingGradient>>>()?
                        .map(|value| value.native.clone())
                };
                // Validate the final combination once: either edit alone may be
                // invalid against the old counterpart while both together are legal.
                Ok(Self {
                    native: core::ContinuousColorScale::diverging(center, gradient, domain)
                        .map_err(native_error)?,
                })
            }
        }
    }
}

fn point_shape(value: &str) -> PyResult<core::PointShape> {
    match value {
        "circle" => Ok(core::PointShape::Circle),
        "triangle_up" => Ok(core::PointShape::TriangleUp),
        _ => Err(PyValueError::new_err(
            "point_shape must be 'circle' or 'triangle_up'",
        )),
    }
}

fn point_shape_name(value: core::PointShape) -> &'static str {
    match value {
        core::PointShape::Circle => "circle",
        core::PointShape::TriangleUp => "triangle_up",
    }
}

fn line_pattern(value: &str) -> PyResult<core::LinePattern> {
    match value {
        "solid" => Ok(core::LinePattern::Solid),
        "dashed" => Ok(core::LinePattern::Dashed),
        _ => Err(PyValueError::new_err(
            "line_pattern must be 'solid' or 'dashed'",
        )),
    }
}

fn line_pattern_name(value: core::LinePattern) -> &'static str {
    match value {
        core::LinePattern::Solid => "solid",
        core::LinePattern::Dashed => "dashed",
    }
}

#[pyclass(
    name = "InteractionStyle",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyInteractionStyle {
    pub(crate) native: core::InteractionStyle,
}

#[pymethods]
impl PyInteractionStyle {
    #[new]
    #[pyo3(signature = (group, *, point_shape = "circle", line_pattern = "solid"))]
    fn new(group: String, point_shape: &str, line_pattern: &str) -> PyResult<Self> {
        let point_shape = self::point_shape(point_shape)?;
        let line_pattern = self::line_pattern(line_pattern)?;
        Ok(Self {
            native: core::InteractionStyle::new(group)
                .and_then(|value| value.point_shape(point_shape))
                .and_then(|value| value.line_pattern(line_pattern))
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn group(&self) -> &str {
        self.native.group()
    }

    #[getter]
    fn point_shape(&self) -> &'static str {
        point_shape_name(self.native.get_point_shape())
    }

    #[getter]
    fn line_pattern(&self) -> &'static str {
        line_pattern_name(self.native.get_line_pattern())
    }

    #[pyo3(signature = (*, group = omitted(), point_shape = omitted(), line_pattern = omitted()),
        text_signature = "($self, *, group=..., point_shape=..., line_pattern=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        group: Py<PyAny>,
        point_shape: Py<PyAny>,
        line_pattern: Py<PyAny>,
    ) -> PyResult<Self> {
        let group = &group.into_bound(py);
        let point_shape = &point_shape.into_bound(py);
        let line_pattern = &line_pattern.into_bound(py);
        Self::new(
            if is_omitted(group) {
                self.native.group().to_owned()
            } else {
                group.extract()?
            },
            if is_omitted(point_shape) {
                self.point_shape()
            } else {
                point_shape.extract()?
            },
            if is_omitted(line_pattern) {
                self.line_pattern()
            } else {
                line_pattern.extract()?
            },
        )
    }
}

#[pyclass(
    name = "SignificanceMarkStyle",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PySignificanceMarkStyle {
    pub(crate) native: core::SignificanceMarkStyle,
}

#[pymethods]
impl PySignificanceMarkStyle {
    #[new]
    #[pyo3(signature = (*, font_size = 16.0, weight = "normal", gap = 8.0))]
    fn new(font_size: f64, weight: &str, gap: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::SignificanceMarkStyle::new(font_size, text_weight(weight)?, gap)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn font_size(&self) -> f64 {
        self.native.get_font_size()
    }

    #[getter]
    fn weight(&self) -> &'static str {
        match self.native.get_weight() {
            core::TextWeight::Normal => "normal",
            core::TextWeight::Semibold => "semibold",
            core::TextWeight::Bold => "bold",
        }
    }

    #[getter]
    fn gap(&self) -> f64 {
        self.native.get_gap()
    }

    #[pyo3(signature = (*, font_size = omitted(), weight = omitted(), gap = omitted()),
        text_signature = "($self, *, font_size=..., weight=..., gap=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        font_size: Py<PyAny>,
        weight: Py<PyAny>,
        gap: Py<PyAny>,
    ) -> PyResult<Self> {
        let font_size = &font_size.into_bound(py);
        let weight = &weight.into_bound(py);
        let gap = &gap.into_bound(py);
        Self::new(
            if is_omitted(font_size) {
                self.native.get_font_size()
            } else {
                font_size.extract()?
            },
            if is_omitted(weight) {
                self.weight()
            } else {
                weight.extract()?
            },
            if is_omitted(gap) {
                self.native.get_gap()
            } else {
                gap.extract()?
            },
        )
    }
}

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<super::PyCategoricalColorMap>()?;
    m.add_class::<super::PyCategoricalColorScale>()?;
    m.add_class::<super::PyColorDomain>()?;
    m.add_class::<super::PyContinuousColorScale>()?;
    m.add_class::<super::PyInteractionStyle>()?;
    m.add_class::<super::PySignificanceMarkStyle>()
}
