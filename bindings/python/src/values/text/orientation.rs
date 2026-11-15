//! Normalized angles and variant-preserving authored orientation requests.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::position::PyDataPosition;
use super::{core, is_omitted, native_error, omitted};

#[pyclass(
    name = "TextRotation",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextRotation {
    pub(super) native: core::TextRotation,
}

#[pymethods]
impl PyTextRotation {
    #[new]
    fn new(degrees: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextRotation::degrees(degrees).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn horizontal() -> Self {
        Self {
            native: core::TextRotation::horizontal(),
        }
    }

    #[staticmethod]
    fn tilt_up() -> Self {
        Self {
            native: core::TextRotation::tilt_up(),
        }
    }

    #[staticmethod]
    fn tilt_down() -> Self {
        Self {
            native: core::TextRotation::tilt_down(),
        }
    }

    #[staticmethod]
    fn vertical_up() -> Self {
        Self {
            native: core::TextRotation::vertical_up(),
        }
    }

    #[staticmethod]
    fn vertical_down() -> Self {
        Self {
            native: core::TextRotation::vertical_down(),
        }
    }

    #[getter]
    fn degrees(&self) -> f64 {
        self.native.get_degrees()
    }

    #[pyo3(signature = (*, degrees = omitted()),
        text_signature = "($self, *, degrees=...)")]
    fn copy_with(&self, py: Python<'_>, degrees: Py<PyAny>) -> PyResult<Self> {
        let degrees = &degrees.into_bound(py);
        if is_omitted(degrees) {
            Ok(self.clone())
        } else {
            Self::new(degrees.extract()?)
        }
    }
}

fn rotations(value: &Bound<'_, PyAny>) -> PyResult<Vec<core::TextRotation>> {
    Ok(value
        .extract::<Vec<PyRef<'_, PyTextRotation>>>()?
        .iter()
        .map(|rotation| rotation.native)
        .collect())
}

fn require_omitted(arguments: &[&Bound<'_, PyAny>]) -> PyResult<()> {
    if arguments.iter().any(|value| !is_omitted(value)) {
        return Err(PyTypeError::new_err(
            "copy_with edits only payloads of the current orientation branch",
        ));
    }
    Ok(())
}

#[pyclass(
    name = "TextOrientation",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextOrientation {
    pub(super) native: core::TextOrientation,
}

#[pymethods]
impl PyTextOrientation {
    #[staticmethod]
    fn fixed(rotation: PyRef<'_, PyTextRotation>) -> Self {
        let angle = rotation.native;
        drop(rotation);
        Self {
            native: core::TextOrientation::fixed(angle),
        }
    }

    #[staticmethod]
    fn data_direction(from_: PyRef<'_, PyDataPosition>, to: PyRef<'_, PyDataPosition>) -> Self {
        let start = from_.native.clone();
        let end = to.native.clone();
        drop(from_);
        drop(to);
        Self {
            native: core::TextOrientation::data_direction(start, end),
        }
    }

    #[staticmethod]
    fn automatic_ticks(angles: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            native: core::TextOrientation::automatic_ticks(rotations(angles)?),
        })
    }

    #[getter]
    fn rotation(&self) -> Option<PyTextRotation> {
        self.native
            .get_fixed_rotation()
            .map(|native| PyTextRotation { native })
    }

    #[getter]
    fn data_from(&self) -> Option<PyDataPosition> {
        self.native
            .get_data_direction()
            .map(|(from, _)| PyDataPosition {
                native: from.clone(),
            })
    }

    #[getter]
    fn data_to(&self) -> Option<PyDataPosition> {
        self.native
            .get_data_direction()
            .map(|(_, to)| PyDataPosition { native: to.clone() })
    }

    #[getter]
    fn angles<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyTuple>>> {
        self.native
            .get_automatic_angles()
            .map(|angles| {
                let angles = angles
                    .iter()
                    .map(|&native| Py::new(py, PyTextRotation { native }))
                    .collect::<PyResult<Vec<_>>>()?;
                PyTuple::new(py, angles)
            })
            .transpose()
    }

    #[pyo3(signature = (*, rotation = omitted(), data_from = omitted(), data_to = omitted(), angles = omitted()),
        text_signature = "($self, *, rotation=..., data_from=..., data_to=..., angles=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        rotation: Py<PyAny>,
        data_from: Py<PyAny>,
        data_to: Py<PyAny>,
        angles: Py<PyAny>,
    ) -> PyResult<Self> {
        let rotation = &rotation.into_bound(py);
        let data_from = &data_from.into_bound(py);
        let data_to = &data_to.into_bound(py);
        let angles = &angles.into_bound(py);
        let native = if let Some(inherited) = self.native.get_fixed_rotation() {
            require_omitted(&[data_from, data_to, angles])?;
            let rotation = if is_omitted(rotation) {
                inherited
            } else {
                rotation.extract::<PyRef<'_, PyTextRotation>>()?.native
            };
            core::TextOrientation::fixed(rotation)
        } else if let Some((from, to)) = self.native.get_data_direction() {
            require_omitted(&[rotation, angles])?;
            let from = if is_omitted(data_from) {
                from.clone()
            } else {
                data_from
                    .extract::<PyRef<'_, PyDataPosition>>()?
                    .native
                    .clone()
            };
            let to = if is_omitted(data_to) {
                to.clone()
            } else {
                data_to
                    .extract::<PyRef<'_, PyDataPosition>>()?
                    .native
                    .clone()
            };
            core::TextOrientation::data_direction(from, to)
        } else {
            require_omitted(&[rotation, data_from, data_to])?;
            let angles = if is_omitted(angles) {
                self.native
                    .get_automatic_angles()
                    .expect("a native orientation has one authored branch")
                    .to_vec()
            } else {
                rotations(angles)?
            };
            core::TextOrientation::automatic_ticks(angles)
        };
        Ok(Self { native })
    }
}
