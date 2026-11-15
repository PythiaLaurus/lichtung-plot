//! Complete native anchors, abbreviation, overflow and visibility programs.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::{core, is_omitted, native_error, omitted};

string_choice!(elision, elision_name, core::TextElision, "position", {
    "start" => core::TextElision::Start,
    "middle" => core::TextElision::Middle,
    "end" => core::TextElision::End,
    "left" => core::TextElision::Left,
    "right" => core::TextElision::Right,
});
string_choice!(failure, failure_name, core::TextFailure, "failure", {
    "error" => core::TextFailure::Error,
    "hide" => core::TextFailure::Hide,
    "clip" => core::TextFailure::Clip,
    "visible" => core::TextFailure::Visible,
});
string_choice!(endpoints, endpoints_name, core::LabelEndpoints, "endpoints", {
    "none" => core::LabelEndpoints::None,
    "first" => core::LabelEndpoints::First,
    "last" => core::LabelEndpoints::Last,
    "both" => core::LabelEndpoints::Both,
});

#[pyclass(
    name = "TextAnchor",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextAnchor {
    pub(super) native: core::TextAnchor,
}

#[pymethods]
impl PyTextAnchor {
    #[staticmethod]
    fn fractions(horizontal: f64, vertical: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextAnchor::fractions(horizontal, vertical).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn first_baseline(horizontal: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextAnchor::first_baseline(horizontal).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn center_baseline(horizontal: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextAnchor::center_baseline(horizontal).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn top_left() -> Self {
        Self {
            native: core::TextAnchor::top_left(),
        }
    }

    #[staticmethod]
    fn top_center() -> Self {
        Self {
            native: core::TextAnchor::top_center(),
        }
    }

    #[staticmethod]
    fn top_right() -> Self {
        Self {
            native: core::TextAnchor::top_right(),
        }
    }

    #[staticmethod]
    fn center_left() -> Self {
        Self {
            native: core::TextAnchor::center_left(),
        }
    }

    #[staticmethod]
    fn center() -> Self {
        Self {
            native: core::TextAnchor::center(),
        }
    }

    #[staticmethod]
    fn center_right() -> Self {
        Self {
            native: core::TextAnchor::center_right(),
        }
    }

    #[staticmethod]
    fn bottom_left() -> Self {
        Self {
            native: core::TextAnchor::bottom_left(),
        }
    }

    #[staticmethod]
    fn bottom_center() -> Self {
        Self {
            native: core::TextAnchor::bottom_center(),
        }
    }

    #[staticmethod]
    fn bottom_right() -> Self {
        Self {
            native: core::TextAnchor::bottom_right(),
        }
    }

    #[getter]
    fn horizontal(&self) -> f64 {
        self.native.get_horizontal()
    }

    #[getter]
    fn vertical_fraction(&self) -> Option<f64> {
        self.native.get_vertical_fraction()
    }

    #[getter]
    fn baseline(&self) -> Option<&'static str> {
        if self.native.get_vertical_fraction().is_some() {
            None
        } else if self.native.is_first_baseline() {
            Some("first")
        } else {
            Some("center")
        }
    }

    #[pyo3(signature = (*, horizontal = omitted()),
        text_signature = "($self, *, horizontal=...)")]
    fn copy_with(&self, py: Python<'_>, horizontal: Py<PyAny>) -> PyResult<Self> {
        let horizontal = &horizontal.into_bound(py);
        let horizontal = if is_omitted(horizontal) {
            self.native.get_horizontal()
        } else {
            horizontal.extract()?
        };
        if let Some(vertical) = self.native.get_vertical_fraction() {
            Self::fractions(horizontal, vertical)
        } else if self.native.is_first_baseline() {
            Self::first_baseline(horizontal)
        } else {
            Self::center_baseline(horizontal)
        }
    }
}

fn ellipsis(
    position: core::TextElision,
    marker: String,
    max_lines: Option<usize>,
    distinct: Option<bool>,
) -> core::TextEllipsis {
    let mut native = core::TextEllipsis::new(position)
        .marker(marker)
        .max_lines(max_lines);
    if let Some(distinct) = distinct {
        native = native.distinct(distinct);
    }
    native
}

#[pyclass(
    name = "TextEllipsis",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextEllipsis {
    pub(super) native: core::TextEllipsis,
}

#[pymethods]
impl PyTextEllipsis {
    #[new]
    #[pyo3(signature = (position, *, marker = "…", max_lines = Some(1), distinct = None))]
    fn new(
        position: &str,
        marker: &str,
        max_lines: Option<usize>,
        distinct: Option<bool>,
    ) -> PyResult<Self> {
        Ok(Self {
            native: ellipsis(elision(position)?, marker.to_owned(), max_lines, distinct),
        })
    }

    #[getter]
    fn position(&self) -> &'static str {
        elision_name(self.native.get_position())
    }

    #[getter]
    fn marker(&self) -> &str {
        self.native.get_marker()
    }

    #[getter]
    fn max_lines(&self) -> Option<usize> {
        self.native.get_max_lines()
    }

    #[getter]
    fn distinct(&self) -> Option<bool> {
        self.native.get_distinct()
    }

    #[pyo3(signature = (*, position = omitted(), marker = omitted(), max_lines = omitted(), distinct = omitted()),
        text_signature = "($self, *, position=..., marker=..., max_lines=..., distinct=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        position: Py<PyAny>,
        marker: Py<PyAny>,
        max_lines: Py<PyAny>,
        distinct: Py<PyAny>,
    ) -> PyResult<Self> {
        let position = &position.into_bound(py);
        let marker = &marker.into_bound(py);
        let max_lines = &max_lines.into_bound(py);
        let distinct = &distinct.into_bound(py);
        let position = if is_omitted(position) {
            self.native.get_position()
        } else {
            elision(position.extract::<&str>()?)?
        };
        let marker = if is_omitted(marker) {
            self.native.get_marker().to_owned()
        } else {
            marker.extract()?
        };
        let max_lines = if is_omitted(max_lines) {
            self.native.get_max_lines()
        } else {
            max_lines.extract()?
        };
        let distinct = if is_omitted(distinct) {
            self.native.get_distinct()
        } else {
            distinct.extract()?
        };
        Ok(Self {
            native: ellipsis(position, marker, max_lines, distinct),
        })
    }
}

fn overflow(
    failure: core::TextFailure,
    ellipsis: Option<core::TextEllipsis>,
) -> core::TextOverflow {
    if let Some(ellipsis) = ellipsis {
        core::TextOverflow::ellipsis(ellipsis, failure)
    } else {
        match failure {
            core::TextFailure::Error => core::TextOverflow::complete(),
            core::TextFailure::Hide => core::TextOverflow::hide(),
            core::TextFailure::Clip => core::TextOverflow::clip(),
            core::TextFailure::Visible => core::TextOverflow::visible(),
        }
    }
}

#[pyclass(
    name = "TextOverflow",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextOverflow {
    pub(super) native: core::TextOverflow,
}

#[pymethods]
impl PyTextOverflow {
    #[new]
    #[pyo3(signature = (*, failure = "error", ellipsis = None))]
    fn new(failure: &str, ellipsis: Option<PyRef<'_, PyTextEllipsis>>) -> PyResult<Self> {
        Ok(Self {
            native: overflow(
                self::failure(failure)?,
                ellipsis.map(|value| value.native.clone()),
            ),
        })
    }

    #[getter]
    fn failure(&self) -> &'static str {
        failure_name(self.native.get_failure())
    }

    #[getter]
    fn ellipsis(&self) -> Option<PyTextEllipsis> {
        self.native
            .get_ellipsis()
            .cloned()
            .map(|native| PyTextEllipsis { native })
    }

    #[pyo3(signature = (*, failure = omitted(), ellipsis = omitted()),
        text_signature = "($self, *, failure=..., ellipsis=...)")]
    fn copy_with(&self, py: Python<'_>, failure: Py<PyAny>, ellipsis: Py<PyAny>) -> PyResult<Self> {
        let failure = &failure.into_bound(py);
        let ellipsis = &ellipsis.into_bound(py);
        let failure = if is_omitted(failure) {
            self.native.get_failure()
        } else {
            self::failure(failure.extract::<&str>()?)?
        };
        let ellipsis = if is_omitted(ellipsis) {
            self.native.get_ellipsis().cloned()
        } else if ellipsis.is_none() {
            None
        } else {
            Some(
                ellipsis
                    .extract::<PyRef<'_, PyTextEllipsis>>()?
                    .native
                    .clone(),
            )
        };
        Ok(Self {
            native: overflow(failure, ellipsis),
        })
    }
}

#[pyclass(
    name = "TextVisibility",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextVisibility {
    pub(super) native: core::TextVisibility,
}

#[pymethods]
impl PyTextVisibility {
    #[staticmethod]
    #[pyo3(signature = (*, boundary_tolerance = None))]
    fn all(boundary_tolerance: Option<f64>) -> PyResult<Self> {
        Ok(Self {
            native: core::TextVisibility::all()
                .boundary_tolerance(boundary_tolerance)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (*, boundary_tolerance = None))]
    fn hidden(boundary_tolerance: Option<f64>) -> PyResult<Self> {
        Ok(Self {
            native: core::TextVisibility::hidden()
                .boundary_tolerance(boundary_tolerance)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (stride, offset, *, endpoints, boundary_tolerance = None))]
    fn every(
        stride: usize,
        offset: usize,
        endpoints: &str,
        boundary_tolerance: Option<f64>,
    ) -> PyResult<Self> {
        let endpoints = self::endpoints(endpoints)?;
        Ok(Self {
            native: core::TextVisibility::every(stride, offset, endpoints)
                .map_err(native_error)?
                .boundary_tolerance(boundary_tolerance)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (*, endpoints, separation = 0.0, boundary_tolerance = None))]
    fn greedy(endpoints: &str, separation: f64, boundary_tolerance: Option<f64>) -> PyResult<Self> {
        let endpoints = self::endpoints(endpoints)?;
        Ok(Self {
            native: core::TextVisibility::greedy(endpoints)
                .separation(separation)
                .map_err(native_error)?
                .boundary_tolerance(boundary_tolerance)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (*, endpoints, separation = 0.0, boundary_tolerance = None))]
    fn parity(endpoints: &str, separation: f64, boundary_tolerance: Option<f64>) -> PyResult<Self> {
        let endpoints = self::endpoints(endpoints)?;
        Ok(Self {
            native: core::TextVisibility::parity(endpoints)
                .separation(separation)
                .map_err(native_error)?
                .boundary_tolerance(boundary_tolerance)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn kind(&self) -> &'static str {
        if self.native.is_hidden() {
            "hidden"
        } else if self.native.get_every().is_some() {
            "every"
        } else if self.native.is_greedy() {
            "greedy"
        } else if self.native.is_parity() {
            "parity"
        } else {
            "all"
        }
    }

    #[getter]
    fn stride(&self) -> Option<usize> {
        self.native.get_every().map(|(stride, _)| stride)
    }

    #[getter]
    fn offset(&self) -> Option<usize> {
        self.native.get_every().map(|(_, offset)| offset)
    }

    #[getter]
    fn endpoints(&self) -> &'static str {
        endpoints_name(self.native.get_endpoints())
    }

    #[getter]
    fn separation(&self) -> f64 {
        self.native.get_separation()
    }

    #[getter]
    fn boundary_tolerance(&self) -> Option<f64> {
        self.native.get_boundary_tolerance()
    }

    #[pyo3(signature = (*, separation = omitted(), boundary_tolerance = omitted()),
        text_signature = "($self, *, separation=..., boundary_tolerance=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        separation: Py<PyAny>,
        boundary_tolerance: Py<PyAny>,
    ) -> PyResult<Self> {
        let separation = &separation.into_bound(py);
        let boundary_tolerance = &boundary_tolerance.into_bound(py);
        let mut native = self.native.clone();
        if !is_omitted(separation) {
            native = native
                .separation(separation.extract()?)
                .map_err(native_error)?;
        }
        if !is_omitted(boundary_tolerance) {
            native = native
                .boundary_tolerance(boundary_tolerance.extract()?)
                .map_err(native_error)?;
        }
        Ok(Self { native })
    }
}
