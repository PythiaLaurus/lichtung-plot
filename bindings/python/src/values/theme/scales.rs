//! Native ordered palettes and gradients, without Python-side repair or maps.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{is_omitted, omitted, PyOpaqueColor};
use crate::native_error;

fn colors(value: &Bound<'_, PyAny>) -> PyResult<Vec<core::OpaqueColor>> {
    let values: Vec<PyRef<'_, PyOpaqueColor>> = value.extract()?;
    Ok(values.iter().map(|value| value.native).collect())
}

fn color_tuple<'py>(
    py: Python<'py>,
    values: &[core::OpaqueColor],
) -> PyResult<Bound<'py, PyTuple>> {
    let values = values
        .iter()
        .map(|&native| Py::new(py, PyOpaqueColor { native }))
        .collect::<PyResult<Vec<_>>>()?;
    PyTuple::new(py, values)
}

#[pyclass(
    name = "AccentPalette",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyAccentPalette {
    pub(crate) native: core::AccentPalette,
}

#[pymethods]
impl PyAccentPalette {
    #[new]
    fn new(seeds: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            native: core::AccentPalette::new(colors(seeds)?).map_err(native_error)?,
        })
    }

    #[getter]
    fn seeds<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        color_tuple(py, self.native.seeds())
    }

    #[pyo3(signature = (*, seeds = omitted()),
        text_signature = "($self, *, seeds=...)")]
    fn copy_with(&self, py: Python<'_>, seeds: Py<PyAny>) -> PyResult<Self> {
        let seeds = &seeds.into_bound(py);
        if is_omitted(seeds) {
            Ok(self.clone())
        } else {
            Self::new(seeds)
        }
    }
}

#[pyclass(
    name = "QualitativePalette",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyQualitativePalette {
    pub(crate) native: core::QualitativePalette,
}

impl PyQualitativePalette {
    fn from_parts(colors: Vec<core::OpaqueColor>, capacity: &str) -> PyResult<Self> {
        let native = match capacity {
            "fixed" => core::QualitativePalette::fixed(colors),
            "extendable" => core::QualitativePalette::extendable(colors),
            _ => {
                return Err(PyValueError::new_err(
                    "capacity must be 'fixed' or 'extendable'",
                ))
            }
        };
        Ok(Self {
            native: native.map_err(native_error)?,
        })
    }
}

#[pymethods]
impl PyQualitativePalette {
    #[staticmethod]
    fn fixed(colors: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::from_parts(self::colors(colors)?, "fixed")
    }

    #[staticmethod]
    fn extendable(colors: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::from_parts(self::colors(colors)?, "extendable")
    }

    #[getter]
    fn colors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        color_tuple(py, self.native.colors())
    }

    #[getter]
    fn capacity(&self) -> &'static str {
        match self.native.capacity() {
            core::PaletteCapacity::Fixed => "fixed",
            core::PaletteCapacity::Extendable => "extendable",
        }
    }

    #[pyo3(signature = (*, colors = omitted(), capacity = omitted()),
        text_signature = "($self, *, colors=..., capacity=...)")]
    fn copy_with(&self, py: Python<'_>, colors: Py<PyAny>, capacity: Py<PyAny>) -> PyResult<Self> {
        let colors = &colors.into_bound(py);
        let capacity = &capacity.into_bound(py);
        let colors = if is_omitted(colors) {
            self.native.colors().to_vec()
        } else {
            self::colors(colors)?
        };
        let capacity = if is_omitted(capacity) {
            self.capacity()
        } else {
            capacity.extract::<&str>()?
        };
        Self::from_parts(colors, capacity)
    }
}

#[pyclass(
    name = "ColorStop",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyColorStop {
    pub(crate) native: core::ColorStop,
}

#[pymethods]
impl PyColorStop {
    #[new]
    fn new(offset: f64, color: PyRef<'_, PyOpaqueColor>) -> PyResult<Self> {
        let paint = color.native;
        drop(color);
        Ok(Self {
            native: core::ColorStop::new(offset, paint).map_err(native_error)?,
        })
    }

    #[getter]
    fn offset(&self) -> f64 {
        self.native.offset()
    }

    #[getter]
    fn color(&self) -> PyOpaqueColor {
        PyOpaqueColor {
            native: self.native.color(),
        }
    }

    #[pyo3(signature = (*, offset = omitted(), color = omitted()),
        text_signature = "($self, *, offset=..., color=...)")]
    fn copy_with(&self, py: Python<'_>, offset: Py<PyAny>, color: Py<PyAny>) -> PyResult<Self> {
        let offset = &offset.into_bound(py);
        let color = &color.into_bound(py);
        let offset = if is_omitted(offset) {
            self.native.offset()
        } else {
            offset.extract()?
        };
        let color = if is_omitted(color) {
            self.native.color()
        } else {
            color.extract::<PyRef<'_, PyOpaqueColor>>()?.native
        };
        Ok(Self {
            native: core::ColorStop::new(offset, color).map_err(native_error)?,
        })
    }
}

fn stops(value: &Bound<'_, PyAny>) -> PyResult<Vec<core::ColorStop>> {
    let values: Vec<PyRef<'_, PyColorStop>> = value.extract()?;
    Ok(values.iter().map(|value| value.native).collect())
}

fn stop_tuple<'py>(py: Python<'py>, values: &[core::ColorStop]) -> PyResult<Bound<'py, PyTuple>> {
    let values = values
        .iter()
        .map(|&native| Py::new(py, PyColorStop { native }))
        .collect::<PyResult<Vec<_>>>()?;
    PyTuple::new(py, values)
}

macro_rules! gradient {
    ($wrapper:ident, $name:literal, $native:ident, {$($extra:item)*}) => {
        #[pyclass(name = $name, module = "lichtung_plot", frozen, eq, skip_from_py_object)]
        #[derive(Clone, PartialEq)]
        pub(crate) struct $wrapper {
            pub(crate) native: core::$native,
        }

        #[pymethods]
        impl $wrapper {
            #[new]
            fn new(stops: &Bound<'_, PyAny>) -> PyResult<Self> {
                Ok(Self {
                    native: core::$native::new(self::stops(stops)?).map_err(native_error)?,
                })
            }

            #[getter]
            fn stops<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
                stop_tuple(py, self.native.stops())
            }

            #[pyo3(signature = (*, stops = omitted()),
                text_signature = "($self, *, stops=...)")]
            fn copy_with(&self, py: Python<'_>, stops: Py<PyAny>) -> PyResult<Self> {
                let stops = &stops.into_bound(py);
                if is_omitted(stops) {
                    Ok(self.clone())
                } else {
                    Self::new(stops)
                }
            }

            $($extra)*
        }
    };
}

gradient!(
    PySequentialGradient,
    "SequentialGradient",
    SequentialGradient,
    {}
);
gradient!(
    PyDivergingGradient,
    "DivergingGradient",
    DivergingGradient,
    {
        #[getter]
        fn neutral(&self) -> PyOpaqueColor {
            PyOpaqueColor {
                native: self.native.neutral(),
            }
        }
    }
);

theme_record!(PyColorScales, "ColorScales", ColorScales, {
    qualitative: PyRef<'_, PyQualitativePalette> => get_qualitative (PyQualitativePalette) [borrow PyQualitativePalette],
    sequential: PyRef<'_, PySequentialGradient> => get_sequential (PySequentialGradient) [borrow PySequentialGradient],
    diverging: PyRef<'_, PyDivergingGradient> => get_diverging (PyDivergingGradient) [borrow PyDivergingGradient],
});

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAccentPalette>()?;
    m.add_class::<PyQualitativePalette>()?;
    m.add_class::<PyColorStop>()?;
    m.add_class::<PySequentialGradient>()?;
    m.add_class::<PyDivergingGradient>()?;
    m.add_class::<PyColorScales>()
}
