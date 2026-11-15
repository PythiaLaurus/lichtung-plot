//! Scalar snapshots and small call-local projection helpers, not retained inputs.

use lichtung_plot::{AxisValues, CategoryValues, NumericValues};
use lichtung_plot_ndarray as nd;
use polars_core::prelude::{Column, DataType};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyString};

use super::array::{self, Role, Scratch};
use super::binding::{self, Data, Direct, Selected, Source};
use super::{pandas, InputRoots};

pub(super) struct Scalar {
    pub(super) values: Scratch,
    pub(super) name: String,
    pub(super) named: bool,
}

impl Scalar {
    pub(super) fn prepare(
        source: Option<&Source<'_>>,
        value: &Bound<'_, PyAny>,
        role: Role,
        name: &str,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        if value.is_none() {
            return Err(PyTypeError::new_err(format!("{name} cannot be None")));
        }
        if let Some(selected) = source
            .map(|source| source.resolve(value))
            .transpose()?
            .flatten()
        {
            let source = source.expect("selection has a source");
            return Self::selected(source, &selected, role, name, roots, aliases);
        }
        let mut direct = Direct::new(value, role, name)?;
        if direct.rank != 1 {
            return Err(PyTypeError::new_err(format!(
                "{name} requires a one-dimensional variable"
            )));
        }
        direct.bind_name(value.py(), roots, aliases)?;
        let named = direct.pandas_name.is_some() || matches!(&direct.data, Data::Polars(_));
        let name = direct.names[0].clone();
        let values = match direct.data {
            Data::Nd(values) => values,
            Data::Polars(columns) => array::from_polars(&columns[0], role)?,
            Data::NdColumns(_) => unreachable!("scalar direct input has rank one"),
        };
        Ok(Self {
            values,
            name,
            named,
        })
    }

    pub(super) fn selected(
        source: &Source<'_>,
        selected: &Selected,
        role: Role,
        name: &str,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        if selected.collection {
            return Err(PyTypeError::new_err(format!(
                "{name} requires one variable, not a collection"
            )));
        }
        let index = selected.indices[0];
        let values = source.numpy_column(index, role)?;
        let name = match source {
            Source::Numpy { .. } => format!("Column {}", index + 1),
            Source::Pandas(table) => binding::key_identity(table, index, roots, aliases)?,
            Source::Polars { names, .. } => names[index].clone(),
        };
        Ok(Self {
            values,
            name,
            named: true,
        })
    }

    pub(super) fn fresh(
        roots: &mut InputRoots,
        value: &Bound<'_, PyAny>,
        role: Role,
        name: &str,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        if let Some((source, selected)) = binding::fresh(roots, value)? {
            Self::selected(&source, &selected, role, name, roots, aliases)
        } else {
            Self::prepare(None, value, role, name, roots, aliases)
        }
    }

    pub(super) fn with_nd<R>(
        &self,
        apply: impl FnOnce(nd::ColumnProjection<'_>) -> PyResult<R>,
    ) -> PyResult<R> {
        binding::with_nd_identity(self.values.view().as_ref(), &self.name, apply)
    }

    pub(super) fn column(&self) -> Column {
        self.values
            .columns(std::slice::from_ref(&self.name))
            .remove(0)
    }

    pub(super) fn dimension(&self, fallback: &str) -> String {
        if self.named {
            self.name.clone()
        } else {
            fallback.to_owned()
        }
    }

    pub(super) fn numeric(&self, role: &'static str) -> PyResult<NumericValues> {
        self.with_nd(|projection| {
            nd::NumericInput::bind_numeric(0usize, Some(&projection), role).map_err(super::nd_error)
        })
    }

    pub(super) fn category(&self, role: &'static str) -> PyResult<CategoryValues> {
        self.category_repeated(role, 1)
    }

    pub(super) fn category_repeated(
        &self,
        role: &'static str,
        copies: usize,
    ) -> PyResult<CategoryValues> {
        self.with_nd(|projection| {
            nd::CategoryInput::bind_category_repeated(0usize, Some(&projection), role, copies)
                .map_err(super::nd_error)
        })
    }

    pub(super) fn numeric_repeated(
        &self,
        role: &'static str,
        copies: usize,
    ) -> PyResult<NumericValues> {
        self.with_nd(|projection| {
            nd::NumericInput::bind_numeric_repeated(0usize, Some(&projection), role, copies)
                .map_err(super::nd_error)
        })
    }
}

pub(super) fn polars_route(
    source: Option<&Source<'_>>,
    values: &[&Bound<'_, PyAny>],
) -> PyResult<bool> {
    if let Some(source) = source {
        return Ok(matches!(source, Source::Polars { .. }));
    }
    for value in values {
        if super::instance(value, "polars", "Series")?
            || super::instance(value, "polars", "DataFrame")?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn literal(value: &Bound<'_, PyAny>, role: &str) -> PyResult<f64> {
    if value.is_none()
        || value.is_instance_of::<PyBool>()
        || super::instance(value, "numpy", "bool_")?
    {
        return Err(PyTypeError::new_err(format!(
            "{role} requires a literal number, not bool or None"
        )));
    }
    value.extract::<f64>()
}

/// Axis classification uses dtype/elements before the one selected native
/// protocol. In particular a Null import must not turn into numeric F64 scratch.
pub(super) fn axis(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<AxisValues> {
    let selected = source
        .map(|source| source.resolve(value))
        .transpose()?
        .flatten();
    let object = pandas_object_axis(source, selected.as_ref(), value, roots)?;
    if let Some(selected) = &selected {
        if selected.collection {
            return Err(PyTypeError::new_err(
                "x requires one variable, not a collection",
            ));
        }
        if let Some(source @ Source::Polars { .. }) = source {
            let column = source.polars_column(selected.indices[0], Role::Category)?;
            if column.dtype() == &DataType::Null {
                return Err(PyTypeError::new_err(
                    "untyped Null x is ambiguous; provide a typed variable",
                ));
            }
            let values = Scalar {
                name: column.name().to_string(),
                values: array::from_polars(&column, Role::Category)?,
                named: true,
            };
            return axis_values(&values, false);
        }
        let values = Scalar::selected(
            source.expect("selection has a source"),
            selected,
            Role::Category,
            "x",
            roots,
            aliases,
        )?;
        if object != pandas_object_axis(source, Some(selected), value, roots)? {
            return Err(crate::DataError::new_err(
                "pandas x dtype changed during axis admission",
            ));
        }
        return axis_values(&values, object);
    } else if super::native_sequence(value) {
        let entries = super::entries(value, "x")?;
        let Some(first) = entries.iter().find(|entry| !entry.is_none()) else {
            return Err(PyTypeError::new_err(
                "empty or all-missing x is ambiguous; provide a typed variable",
            ));
        };
        let role = if first.is_instance_of::<PyString>() {
            Role::Category
        } else {
            Role::Numeric
        };
        return axis_values(
            &Scalar::prepare(None, value, role, "x", roots, aliases)?,
            false,
        );
    } else if super::instance(value, "polars", "Series")? {
        let column = super::polars::import_series(value, "x")?;
        if column.dtype() == &DataType::Null {
            return Err(PyTypeError::new_err(
                "untyped Null x is ambiguous; provide a typed variable",
            ));
        }
        let values = Scalar {
            name: column.name().to_string(),
            values: array::from_polars(&column, Role::Category)?,
            named: true,
        };
        return axis_values(&values, false);
    }
    // Category permits admitted pandas object-string carriers without coercing
    // numeric dtypes. The native ArraySource axis method distinguishes both.
    let values = Scalar::prepare(None, value, Role::Category, "x", roots, aliases)?;
    if object != pandas_object_axis(None, None, value, roots)? {
        return Err(crate::DataError::new_err(
            "pandas x dtype changed during axis admission",
        ));
    }
    axis_values(&values, object)
}

fn pandas_object_axis(
    source: Option<&Source<'_>>,
    selected: Option<&Selected>,
    value: &Bound<'_, PyAny>,
    roots: &InputRoots,
) -> PyResult<bool> {
    let dtype = if let (Some(Source::Pandas(_)), Some(selected)) = (source, selected) {
        let object = roots
            .source
            .as_ref()
            .expect("constructor source is retained")
            .bind(value.py());
        object
            .getattr("dtypes")?
            .getattr("iloc")?
            .get_item(selected.indices[0])?
    } else if selected.is_none() && super::instance(value, "pandas", "Series")? {
        value.getattr("dtype")?
    } else {
        return Ok(false);
    };
    Ok(pandas::logical_dtype(&dtype)?.is_none()
        && dtype.getattr("kind")?.extract::<String>()? == "O")
}

fn axis_values(values: &Scalar, untyped_object: bool) -> PyResult<AxisValues> {
    if untyped_object
        && matches!(&values.values, Scratch::String(values) if values.iter().all(Option::is_none))
    {
        return Err(PyTypeError::new_err(
            "empty or all-missing object x is ambiguous; provide a typed variable",
        ));
    }
    values.with_nd(|projection| {
        nd::AxisInput::bind_axis(0usize, Some(&projection), "x").map_err(super::nd_error)
    })
}

pub(crate) fn bootstrap_units(
    data: Option<&Bound<'_, PyAny>>,
    units: &Bound<'_, PyAny>,
) -> PyResult<CategoryValues> {
    let (source, mut roots) = super::families::context(data)?;
    let mut aliases = Vec::new();
    Scalar::prepare(
        source.as_ref(),
        units,
        Role::Category,
        "bootstrap units",
        &mut roots,
        &mut aliases,
    )?
    .category("bootstrap units")
}
