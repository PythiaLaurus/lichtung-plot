//! Call-local alpha declarations. Foreign preparation never borrows a plot owner.
//!
//! The original adapter methods own mapping, expansion and selector validation.
//! Their lookup contexts below borrow only Rust-owned cells and raw identities.

use lichtung_plot::{
    self as core, kind, AlphaEncoding, AlphaValues, CategoryValues, NumericValues,
};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use ndarray::Array2;
use polars_core::prelude::{Column, DataFrame, DataType, IntoColumn, Series};
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyString};

use super::array::{self, Role, Scratch};
use super::binding::{self, Data, Direct, Source};
use super::columns::{self, Scalar};
use super::families::{self, from_nd, from_pl};
use super::selectors::{self, Selection};
use super::{pandas, Config, InputRoots, Origin, Prepared};

type Input<'a, 'py> = &'a Bound<'py, PyAny>;

pub(crate) enum AlphaSourceArgs<'a, 'py> {
    Main,
    Independent(Option<Input<'a, 'py>>),
}

pub(crate) struct AlphaMapArgs<'a, 'py> {
    pub(crate) linear: Input<'a, 'py>,
    pub(crate) log10: Input<'a, 'py>,
    pub(crate) threshold: Input<'a, 'py>,
    pub(crate) continuous: Input<'a, 'py>,
    pub(crate) unset: Input<'a, 'py>,
}

pub(crate) enum AlphaInputArgs<'a, 'py> {
    Raw(Input<'a, 'py>),
    Numeric {
        input: Input<'a, 'py>,
        map: AlphaMapArgs<'a, 'py>,
    },
    Categories {
        input: Input<'a, 'py>,
        mapper: Input<'a, 'py>,
    },
}

pub(crate) enum AlphaRequestArgs<'a, 'py> {
    Row(AlphaInputArgs<'a, 'py>),
    Cells(AlphaInputArgs<'a, 'py>),
    Linear {
        input: Input<'a, 'py>,
        domain: Input<'a, 'py>,
        range: Input<'a, 'py>,
    },
    Single {
        value: Input<'a, 'py>,
        input: Input<'a, 'py>,
        unset: Input<'a, 'py>,
    },
    SingleMap {
        input: Input<'a, 'py>,
        map: AlphaMapArgs<'a, 'py>,
    },
    SingleCategories {
        input: Input<'a, 'py>,
        mapper: Input<'a, 'py>,
    },
    Keyed {
        source: AlphaSourceArgs<'a, 'py>,
        keys: Input<'a, 'py>,
        input: AlphaInputArgs<'a, 'py>,
    },
    Columns {
        source: AlphaSourceArgs<'a, 'py>,
        column_key: Input<'a, 'py>,
        other_keys: Option<Input<'a, 'py>>,
        input: AlphaInputArgs<'a, 'py>,
    },
}

pub(crate) struct AlphaBinsArgs<'a, 'py> {
    pub(crate) source: AlphaSourceArgs<'a, 'py>,
    pub(crate) lower: Input<'a, 'py>,
    pub(crate) upper: Input<'a, 'py>,
    pub(crate) facets: Option<Input<'a, 'py>>,
    pub(crate) input: AlphaInputArgs<'a, 'py>,
}

pub(crate) fn point_alpha<'py, K: kind::PointAlpha>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: AlphaRequestArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    apply_point(
        py,
        config,
        Call::prepare(py, roots, args, matches!(&config.origin, Origin::Nd(_)))?,
    )
}

pub(crate) fn line_alpha<'py, K: kind::LineAlpha>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: AlphaRequestArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    apply_line(
        py,
        config,
        Call::prepare(py, roots, args, matches!(&config.origin, Origin::Nd(_)))?,
    )
}

pub(crate) fn fill_alpha<'py, K: kind::FillAlpha>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: AlphaRequestArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    apply_fill(
        py,
        config,
        Call::prepare(py, roots, args, matches!(&config.origin, Origin::Nd(_)))?,
    )
}

pub(crate) fn fill_alpha_bins<'py>(
    py: Python<'py>,
    config: &Config<kind::Histogram>,
    roots: &InputRoots,
    args: AlphaBinsArgs<'_, 'py>,
) -> PyResult<Prepared<kind::Histogram>> {
    apply_bins(py, config, Call::prepare_bins(py, roots, args)?)
}

enum NumericFactory<'py> {
    Linear((f64, f64), (u8, u8)),
    Log10((f64, f64), (u8, u8)),
    Threshold(Vec<f64>, Vec<u8>),
    Continuous((f64, f64), Bound<'py, PyAny>),
}

enum Mapping<'py> {
    Raw,
    Numeric(NumericFactory<'py>),
    Categories(Bound<'py, PyAny>),
}

fn byte(value: &Bound<'_, PyAny>) -> PyResult<u8> {
    if value.is_instance_of::<PyBool>() || !value.is_instance_of::<PyInt>() {
        return Err(PyTypeError::new_err(
            "alpha bytes must be integers, not bool or float",
        ));
    }
    value.extract()
}

fn pair<'py>(value: &Bound<'py, PyAny>, role: &str) -> PyResult<[Bound<'py, PyAny>; 2]> {
    let values = super::entries(value, role)?;
    values
        .try_into()
        .map_err(|_| PyTypeError::new_err(format!("{role} requires exactly two entries")))
}

fn domain(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let [lower, upper] = pair(value, "alpha domain")?;
    Ok((
        columns::literal(&lower, "alpha domain endpoint")?,
        columns::literal(&upper, "alpha domain endpoint")?,
    ))
}

fn byte_range(value: &Bound<'_, PyAny>) -> PyResult<(u8, u8)> {
    let [lower, upper] = pair(value, "alpha byte range")?;
    Ok((byte(&lower)?, byte(&upper)?))
}

fn callable<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if !value.is_callable() {
        return Err(PyTypeError::new_err("alpha mapper must be callable"));
    }
    Ok(value.clone())
}

impl<'py> NumericFactory<'py> {
    fn parse(args: &AlphaMapArgs<'_, 'py>) -> PyResult<Self> {
        let options = [args.linear, args.log10, args.threshold, args.continuous];
        if options.iter().filter(|value| !value.is(args.unset)).count() != 1 {
            return Err(PyTypeError::new_err(
                "supply exactly one of linear, log10, threshold or continuous",
            ));
        }
        if !args.linear.is(args.unset) {
            let [limits, range] = pair(args.linear, "linear alpha mapping")?;
            Ok(Self::Linear(domain(&limits)?, byte_range(&range)?))
        } else if !args.log10.is(args.unset) {
            let [limits, range] = pair(args.log10, "log10 alpha mapping")?;
            Ok(Self::Log10(domain(&limits)?, byte_range(&range)?))
        } else if !args.threshold.is(args.unset) {
            let [boundaries, alphas] = pair(args.threshold, "threshold alpha mapping")?;
            let boundaries = super::entries(&boundaries, "alpha thresholds")?
                .iter()
                .map(|value| columns::literal(value, "alpha threshold"))
                .collect::<PyResult<Vec<_>>>()?;
            let alphas = super::entries(&alphas, "threshold alpha bytes")?
                .iter()
                .map(byte)
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Self::Threshold(boundaries, alphas))
        } else {
            let [limits, mapper] = pair(args.continuous, "continuous alpha mapping")?;
            Ok(Self::Continuous(domain(&limits)?, callable(&mapper)?))
        }
    }

    fn apply(
        &self,
        values: NumericValues,
        failure: &mut Option<PyErr>,
    ) -> core::Result<AlphaValues> {
        match self {
            Self::Linear(domain, range) => AlphaValues::linear(values, *domain, *range),
            Self::Log10(domain, range) => AlphaValues::log10(values, *domain, *range),
            Self::Threshold(boundaries, alphas) => {
                AlphaValues::threshold(values, boundaries.iter().copied(), alphas.iter().copied())
            }
            Self::Continuous(domain, mapper) => AlphaValues::continuous(values, *domain, |value| {
                if failure.is_some() {
                    return 0;
                }
                callback_byte(mapper.call1((value,)), failure)
            }),
        }
    }
}

fn callback_byte(result: PyResult<Bound<'_, PyAny>>, failure: &mut Option<PyErr>) -> u8 {
    match result.and_then(|value| byte(&value)) {
        Ok(value) => value,
        Err(error) => {
            *failure = Some(error);
            0
        }
    }
}

fn map_categories(
    mapper: &Bound<'_, PyAny>,
    values: CategoryValues,
    failure: &mut Option<PyErr>,
) -> core::Result<AlphaValues> {
    AlphaValues::categorical(values, |value| {
        if failure.is_some() {
            return 0;
        }
        callback_byte(mapper.call1((value,)), failure)
    })
}

struct InputArgs<'a, 'py> {
    value: Input<'a, 'py>,
    mapping: Mapping<'py>,
}

impl<'a, 'py> InputArgs<'a, 'py> {
    fn parse(args: AlphaInputArgs<'a, 'py>) -> PyResult<Self> {
        let (value, mapping) = match args {
            AlphaInputArgs::Raw(value) => (value, Mapping::Raw),
            AlphaInputArgs::Numeric { input, map } => {
                (input, Mapping::Numeric(NumericFactory::parse(&map)?))
            }
            AlphaInputArgs::Categories { input, mapper } => {
                (input, Mapping::Categories(callable(mapper)?))
            }
        };
        families::required(&[("alpha input", value)])?;
        Ok(Self { value, mapping })
    }
}

type FieldArgs<'py> = Vec<(String, Bound<'py, PyAny>)>;

/// A pinned snapshot of the original owner's registry, never the staged additions.
struct Fields<'py>(Vec<(String, Bound<'py, PyAny>)>);

impl<'py> Fields<'py> {
    fn new(py: Python<'py>, roots: &InputRoots) -> PyResult<Self> {
        let entries = roots
            .keys
            .as_ref()
            .map(|keys| {
                keys.bind(py)
                    .iter()
                    .map(|(raw, key)| Ok((raw.extract::<String>()?, key)))
                    .collect::<PyResult<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self(entries))
    }

    fn field(&self, value: &Bound<'_, PyAny>) -> PyResult<String> {
        if value.is_instance_of::<PyString>() {
            return value.extract();
        }
        let Some(Selection::One(query)) = selectors::description(value)? else {
            return Err(PyTypeError::new_err(
                "an alpha semantic field must be a raw string or col[original_key]",
            ));
        };
        let query = query.bind(value.py());
        let mut matched = None;
        for (raw, key) in &self.0 {
            if pandas::same_key(key, query)? {
                if matched.is_some() {
                    return Err(PyKeyError::new_err(
                        "the alpha semantic pandas key has more than one bound identity",
                    ));
                }
                matched = Some(raw.clone());
            }
        }
        matched.ok_or_else(|| PyKeyError::new_err("the alpha semantic pandas key is not bound"))
    }

    fn pairs(&self, value: Option<&Bound<'py, PyAny>>) -> PyResult<FieldArgs<'py>> {
        let Some(value) = value else {
            return Ok(Vec::new());
        };
        super::entries(value, "alpha selector pairs")?
            .iter()
            .map(|value| {
                let [field, input] = pair(value, "alpha selector pair")?;
                Ok((self.field(&field)?, input))
            })
            .collect()
    }
}

enum ScopeArgs<'a, 'py> {
    Row,
    Cells,
    Linear,
    Single,
    Literal(u8),
    Keyed(FieldArgs<'py>),
    Columns {
        column_key: Input<'a, 'py>,
        other_keys: FieldArgs<'py>,
    },
    Bins {
        lower: Input<'a, 'py>,
        upper: Input<'a, 'py>,
        facets: FieldArgs<'py>,
    },
}

struct Request<'a, 'py> {
    source: AlphaSourceArgs<'a, 'py>,
    input: Option<InputArgs<'a, 'py>>,
    scope: ScopeArgs<'a, 'py>,
}

impl<'a, 'py> Request<'a, 'py> {
    fn parse(args: AlphaRequestArgs<'a, 'py>, fields: &Fields<'py>) -> PyResult<Self> {
        match args {
            AlphaRequestArgs::Row(input) => Self::main(input, ScopeArgs::Row),
            AlphaRequestArgs::Cells(input) => Self::main(input, ScopeArgs::Cells),
            AlphaRequestArgs::Linear {
                input,
                domain: limits,
                range,
            } => {
                families::required(&[("alpha input", input)])?;
                Ok(Self {
                    source: AlphaSourceArgs::Main,
                    input: Some(InputArgs {
                        value: input,
                        mapping: Mapping::Numeric(NumericFactory::Linear(
                            domain(limits)?,
                            byte_range(range)?,
                        )),
                    }),
                    scope: ScopeArgs::Linear,
                })
            }
            AlphaRequestArgs::Single {
                value,
                input,
                unset,
            } => Self::single(value, input, unset),
            AlphaRequestArgs::SingleMap { input, map } => {
                Self::main(AlphaInputArgs::Numeric { input, map }, ScopeArgs::Single)
            }
            AlphaRequestArgs::SingleCategories { input, mapper } => Self::main(
                AlphaInputArgs::Categories { input, mapper },
                ScopeArgs::Single,
            ),
            AlphaRequestArgs::Keyed {
                source,
                keys,
                input,
            } => {
                let input = InputArgs::parse(input)?;
                Ok(Self {
                    source,
                    input: Some(input),
                    scope: ScopeArgs::Keyed(fields.pairs(Some(keys))?),
                })
            }
            AlphaRequestArgs::Columns {
                source,
                column_key,
                other_keys,
                input,
            } => {
                let input = InputArgs::parse(input)?;
                families::required(&[("column_key", column_key)])?;
                Ok(Self {
                    source,
                    input: Some(input),
                    scope: ScopeArgs::Columns {
                        column_key,
                        other_keys: fields.pairs(other_keys)?,
                    },
                })
            }
        }
    }

    fn main(input: AlphaInputArgs<'a, 'py>, scope: ScopeArgs<'a, 'py>) -> PyResult<Self> {
        Ok(Self {
            source: AlphaSourceArgs::Main,
            input: Some(InputArgs::parse(input)?),
            scope,
        })
    }

    fn single(
        value: Input<'a, 'py>,
        input: Input<'a, 'py>,
        unset: Input<'a, 'py>,
    ) -> PyResult<Self> {
        if value.is(unset) == input.is(unset) {
            return Err(PyTypeError::new_err(
                "single alpha requires exactly one of value or input",
            ));
        }
        if input.is(unset) {
            Ok(Self {
                source: AlphaSourceArgs::Main,
                input: None,
                scope: ScopeArgs::Literal(byte(value)?),
            })
        } else {
            Self::main(AlphaInputArgs::Raw(input), ScopeArgs::Single)
        }
    }

    fn bins(args: AlphaBinsArgs<'a, 'py>, fields: &Fields<'py>) -> PyResult<Self> {
        let input = InputArgs::parse(args.input)?;
        families::required(&[("lower", args.lower), ("upper", args.upper)])?;
        Ok(Self {
            source: args.source,
            input: Some(input),
            scope: ScopeArgs::Bins {
                lower: args.lower,
                upper: args.upper,
                facets: fields.pairs(args.facets)?,
            },
        })
    }
}

struct Variable<'py> {
    values: Scalar,
    selected: Option<usize>,
    mapping: Mapping<'py>,
}

fn shaped<T>(rows: usize, values: Vec<Option<T>>) -> PyResult<Array2<Option<T>>> {
    Array2::from_shape_vec((rows, 1), values)
        .map_err(|error| crate::DataError::new_err(error.to_string()))
}

fn scalar(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    role: Role,
    name: &str,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<(Scalar, Option<usize>)> {
    if super::instance(value, "pandas", "DataFrame")?
        || super::instance(value, "polars", "DataFrame")?
    {
        // Scalar alpha roles do not admit collections. Never enter the frozen
        // numeric rectangle transport merely to reject its rank afterwards.
        return Err(PyTypeError::new_err(format!(
            "{name} requires one variable, not a DataFrame",
        )));
    }
    if let Some(selected) = source
        .map(|source| source.resolve(value))
        .transpose()?
        .flatten()
    {
        let source = source.expect("a resolved selection has a source");
        let values = Scalar::selected(source, &selected, role, name, roots, aliases)?;
        Ok((values, Some(selected.indices[0])))
    } else {
        Ok((
            Scalar::prepare(None, value, role, name, roots, aliases)?,
            None,
        ))
    }
}

fn raw_scalar(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<(Scalar, Option<usize>)> {
    if super::native_sequence(value) {
        let cells = super::entries(value, "alpha bytes")?
            .iter()
            .map(|value| byte(value).map(Some))
            .collect::<PyResult<Vec<_>>>()?;
        return Ok((
            Scalar {
                values: Scratch::U8(shaped(cells.len(), cells)?),
                name: "Alpha".to_owned(),
                named: false,
            },
            None,
        ));
    }
    // Numeric is used only for the dtype-preserving foreign transport. No numeric
    // role extraction, normalization or cast is allowed on this exact-byte path.
    let (values, selected) = scalar(source, value, Role::Numeric, "Alpha", roots, aliases)?;
    if !matches!(&values.values, Scratch::U8(_)) {
        return Err(PyTypeError::new_err(
            "foreign alpha bytes require the original u8/UInt8 logical dtype",
        ));
    }
    Ok((values, selected))
}

fn numeric_scalar(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    name: &str,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<(Scalar, Option<usize>)> {
    let (mut values, selected) = scalar(source, value, Role::Numeric, name, roots, aliases)?;
    // Ordinary numeric admission only: nulls and present nonfinite values survive
    // unchanged for the native factory/bounds producer. This is not AlphaValues.
    values.values = Scratch::F64(values.values.numeric()?);
    Ok((values, selected))
}

fn category_scalar(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    name: &str,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<(Scalar, Option<usize>)> {
    let (mut values, selected) = scalar(source, value, Role::Category, name, roots, aliases)?;
    // Decode the ordinary category role without checking alpha selector nulls,
    // lengths, duplicate fields/tuples, schemas or coverage.
    let cells = values.values.category()?;
    values.values = Scratch::String(shaped(cells.len(), cells)?);
    Ok((values, selected))
}

impl<'py> Variable<'py> {
    fn prepare(
        args: InputArgs<'_, 'py>,
        source: Option<&Source<'_>>,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        let (values, selected) = match &args.mapping {
            Mapping::Raw => raw_scalar(source, args.value, roots, aliases)?,
            Mapping::Numeric(_) => numeric_scalar(source, args.value, "alpha map", roots, aliases)?,
            Mapping::Categories(_) => {
                category_scalar(source, args.value, "alpha map", roots, aliases)?
            }
        };
        Ok(Self {
            values,
            selected,
            mapping: args.mapping,
        })
    }

    fn bytes(&self) -> ndarray::ArrayView1<'_, Option<u8>> {
        let Scratch::U8(values) = &self.values.values else {
            unreachable!("raw alpha preparation retains exact bytes")
        };
        values.column(0)
    }

    fn numbers(&self) -> ndarray::ArrayView1<'_, Option<f64>> {
        numeric_cells(&self.values)
    }

    fn categories(&self) -> ndarray::ArrayView1<'_, Option<String>> {
        category_cells(&self.values)
    }

    fn numeric_source(&self, bound: NumericValues) -> NumericValues {
        if bound.name() == self.values.name.as_str() {
            bound
        } else {
            // Direct ND vectors have a protocol role name. When the original
            // columns namespace must remain installed, preserve the imported
            // variable's physical identity inside the factory, not by adding a
            // synthetic field to that namespace or reading opaque native values.
            NumericValues::nullable(&self.values.name, self.numbers().iter().copied())
        }
    }

    fn category_source(&self, bound: CategoryValues) -> core::Result<CategoryValues> {
        if bound.name() == self.values.name.as_str() {
            Ok(bound)
        } else {
            CategoryValues::nullable(&self.values.name, self.categories().iter().cloned())
        }
    }

    fn raw(&self) -> PyResult<AlphaValues> {
        self.values.with_nd(|projection| {
            nd::AlphaValuesInput::bind_alpha_values(0usize, Some(&projection))
                .map_err(super::nd_error)
        })
    }

    fn mapped(&self, failure: &mut Option<PyErr>) -> PyResult<AlphaValues> {
        match &self.mapping {
            Mapping::Numeric(factory) => factory
                .apply(self.values.numeric("alpha map")?, failure)
                .map_err(crate::native_error),
            Mapping::Categories(mapper) => {
                map_categories(mapper, self.values.category("alpha map")?, failure)
                    .map_err(crate::native_error)
            }
            Mapping::Raw => self.raw(),
        }
    }
}

fn numeric_cells(value: &Scalar) -> ndarray::ArrayView1<'_, Option<f64>> {
    let Scratch::F64(values) = &value.values else {
        unreachable!("ordinary numeric preparation uses the native f64 role")
    };
    values.column(0)
}

fn category_cells(value: &Scalar) -> ndarray::ArrayView1<'_, Option<String>> {
    let Scratch::String(values) = &value.values else {
        unreachable!("ordinary category preparation retains decoded raw strings")
    };
    values.column(0)
}

struct Keys(Vec<(String, Scalar)>);

impl Keys {
    fn prepare(
        args: FieldArgs<'_>,
        source: Option<&Source<'_>>,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        args.into_iter()
            .map(|(field, value)| {
                let (values, _) =
                    category_scalar(source, &value, "alpha selector", roots, aliases)?;
                Ok((field, values))
            })
            .collect::<PyResult<Vec<_>>>()
            .map(Self)
    }

    fn nd(&self) -> impl Iterator<Item = (&str, ndarray::ArrayView1<'_, Option<String>>)> {
        self.0
            .iter()
            .map(|(field, value)| (field.as_str(), category_cells(value)))
    }

    fn columns(&self) -> Vec<Column> {
        self.0.iter().map(|(_, value)| value.column()).collect()
    }

    fn pl<'s>(&'s self, columns: &'s [Column]) -> impl Iterator<Item = (&'s str, &'s Column)> {
        self.0
            .iter()
            .zip(columns)
            .map(|((field, _), column)| (field.as_str(), column))
    }

    fn native(&self) -> core::Result<Vec<CategoryValues>> {
        self.0
            .iter()
            .map(|(field, value)| {
                CategoryValues::nullable(field, category_cells(value).iter().cloned())
            })
            .collect()
    }
}

/// Raw typed cells and their original rank; no IDs or generated keys are resolved
/// here. In particular, a same-product rectangle cannot become a scalar variable.
struct ColumnKey {
    data: Vec<Scratch>,
    names: Vec<String>,
    nulls: Vec<bool>,
    rank: usize,
}

impl ColumnKey {
    fn prepare(
        source: Option<&Source<'_>>,
        value: &Bound<'_, PyAny>,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        if let Some(selected) = source
            .map(|source| source.resolve(value))
            .transpose()?
            .flatten()
        {
            let source = source.expect("a resolved selection has a source");
            let mut data = Vec::new();
            let mut names = Vec::new();
            let mut nulls = Vec::new();
            for index in selected.indices {
                let name = match source {
                    Source::Numpy { .. } => format!("Column {}", index + 1),
                    Source::Pandas(table) => binding::key_identity(table, index, roots, aliases)?,
                    Source::Polars { names, .. } => names[index].clone(),
                };
                if matches!(source, Source::Polars { .. }) {
                    let column = source.polars_column(index, Role::Numeric)?;
                    nulls.push(column.dtype() == &DataType::Null);
                    data.push(array::from_polars(&column, Role::Numeric)?);
                } else {
                    nulls.push(false);
                    let role = if matches!(source, Source::Pandas(_)) {
                        Role::Category
                    } else {
                        Role::Numeric
                    };
                    data.push(source.numpy_column(index, role)?);
                }
                names.push(name);
            }
            return Ok(Self {
                data,
                names,
                nulls,
                rank: if selected.collection { 2 } else { 1 },
            });
        }
        let (role, untyped_null) = native_column_key_role(value)?;
        let mut direct = Direct::new(value, role, "column alpha selector")?;
        direct.bind_name(value.py(), roots, aliases)?;
        let (data, nulls) = match direct.data {
            Data::Nd(values) => (vec![values], vec![untyped_null]),
            Data::NdColumns(values) => {
                let nulls = vec![false; values.len()];
                (values, nulls)
            }
            Data::Polars(columns) => {
                let nulls = columns
                    .iter()
                    .map(|column| column.dtype() == &DataType::Null)
                    .collect();
                let data = columns
                    .iter()
                    .map(|column| array::from_polars(column, Role::Numeric))
                    .collect::<PyResult<Vec<_>>>()?;
                (data, nulls)
            }
        };
        Ok(Self {
            data,
            names: direct.names,
            nulls,
            rank: direct.rank,
        })
    }

    fn column(&self) -> PyResult<Column> {
        if self.rank != 1 {
            return Err(PyTypeError::new_err(
                "column_key requires one variable, not a rectangle",
            ));
        }
        let name = &self.names[0];
        if self.nulls[0] {
            Ok(Series::full_null(
                name.as_str().into(),
                self.data[0].shape().0,
                &DataType::Null,
            )
            .into_column())
        } else {
            Ok(self.data[0].columns(std::slice::from_ref(name)).remove(0))
        }
    }
}

fn native_column_key_role(value: &Bound<'_, PyAny>) -> PyResult<(Role, bool)> {
    if !super::native_sequence(value) {
        let role = if super::instance(value, "pandas", "Series")? {
            Role::Category
        } else {
            Role::Ids
        };
        return Ok((role, false));
    }
    let cells = super::entries(value, "column alpha selector")?;
    let first = cells.iter().find(|value| !value.is_none());
    Ok((
        if first.is_some_and(pyo3::types::PyAnyMethods::is_instance_of::<PyString>) {
            Role::Category
        } else if first.is_some_and(pyo3::types::PyAnyMethods::is_instance_of::<PyFloat>) {
            Role::Numeric
        } else {
            Role::Ids
        },
        first.is_none(),
    ))
}

struct Namespace {
    shape: (usize, usize),
    identities: Option<Vec<(usize, String)>>,
}

impl Namespace {
    fn prepare(
        source: &Source<'_>,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        let identities = match source {
            Source::Numpy { .. } => None,
            Source::Pandas(table) => Some(
                (0..table.shape.1)
                    .map(|index| Ok((index, binding::key_identity(table, index, roots, aliases)?)))
                    .collect::<PyResult<Vec<_>>>()?,
            ),
            Source::Polars { names, .. } => Some(names.iter().cloned().enumerate().collect()),
        };
        Ok(Self {
            shape: source.shape(),
            identities,
        })
    }
}

enum Scope {
    Row,
    Cells,
    Linear,
    Single,
    Literal(u8),
    Keyed(Keys),
    Columns {
        column_key: ColumnKey,
        other_keys: Keys,
    },
    Bins {
        lower: Scalar,
        upper: Scalar,
        facets: Keys,
    },
}

struct Call<'py> {
    roots: InputRoots,
    aliases: Vec<(String, String)>,
    variable: Option<Variable<'py>>,
    scope: Scope,
    namespace: Option<Namespace>,
    source_rows: usize,
    independent: bool,
}

impl<'py> Call<'py> {
    fn prepare(
        py: Python<'py>,
        roots: &InputRoots,
        args: AlphaRequestArgs<'_, 'py>,
        nd_namespace: bool,
    ) -> PyResult<Self> {
        let fields = Fields::new(py, roots)?;
        Self::snapshot(py, roots, Request::parse(args, &fields)?, nd_namespace)
    }

    fn prepare_bins(
        py: Python<'py>,
        roots: &InputRoots,
        args: AlphaBinsArgs<'_, 'py>,
    ) -> PyResult<Self> {
        let fields = Fields::new(py, roots)?;
        Self::snapshot(py, roots, Request::bins(args, &fields)?, false)
    }

    fn snapshot(
        py: Python<'py>,
        original: &InputRoots,
        request: Request<'_, 'py>,
        nd_namespace: bool,
    ) -> PyResult<Self> {
        let mut roots = original.stage(py)?;
        let mut aliases = Vec::new();
        let independent = matches!(&request.source, AlphaSourceArgs::Independent(_));
        let source = request.source(py, original, nd_namespace)?;
        let source_rows = source.as_ref().map_or(0, |source| source.shape().0);
        let variable = request
            .input
            .map(|input| Variable::prepare(input, source.as_ref(), &mut roots, &mut aliases))
            .transpose()?;
        let namespace = if nd_namespace && matches!(&request.scope, ScopeArgs::Columns { .. }) {
            source
                .as_ref()
                .map(|source| Namespace::prepare(source, &mut roots, &mut aliases))
                .transpose()?
        } else {
            None
        };
        let scope = prepare_scope(request.scope, source.as_ref(), &mut roots, &mut aliases)?;
        if let Some(source) = &source {
            source.verify()?;
        }
        // All Source/Table objects, borrowed NumPy views, imported C-data handles
        // and capsules are gone before the caller can enter any Python mapper.
        drop(source);
        Ok(Self {
            roots,
            aliases,
            variable,
            scope,
            namespace,
            source_rows,
            independent,
        })
    }

    fn variable(&self) -> &Variable<'py> {
        self.variable
            .as_ref()
            .expect("a data-bearing alpha scope has an input")
    }

    fn finish<K: kind::Marker>(self, config: Config<K>) -> Prepared<K> {
        families::finish(config, self.roots, self.aliases)
    }
}

fn prepare_scope(
    args: ScopeArgs<'_, '_>,
    source: Option<&Source<'_>>,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<Scope> {
    Ok(match args {
        ScopeArgs::Row => Scope::Row,
        ScopeArgs::Cells => Scope::Cells,
        ScopeArgs::Linear => Scope::Linear,
        ScopeArgs::Single => Scope::Single,
        ScopeArgs::Literal(value) => Scope::Literal(value),
        ScopeArgs::Keyed(keys) => Scope::Keyed(Keys::prepare(keys, source, roots, aliases)?),
        ScopeArgs::Columns {
            column_key,
            other_keys,
        } => Scope::Columns {
            column_key: ColumnKey::prepare(source, column_key, roots, aliases)?,
            other_keys: Keys::prepare(other_keys, source, roots, aliases)?,
        },
        ScopeArgs::Bins {
            lower,
            upper,
            facets,
        } => Scope::Bins {
            lower: numeric_scalar(source, lower, "bin lower", roots, aliases)?.0,
            upper: numeric_scalar(source, upper, "bin upper", roots, aliases)?.0,
            facets: Keys::prepare(facets, source, roots, aliases)?,
        },
    })
}

impl<'py> Request<'_, 'py> {
    fn source(
        &self,
        py: Python<'py>,
        roots: &InputRoots,
        nd_namespace: bool,
    ) -> PyResult<Option<Source<'py>>> {
        match &self.source {
            AlphaSourceArgs::Independent(data) => data
                .filter(|data| !data.is_none())
                .map(Source::new)
                .transpose(),
            AlphaSourceArgs::Main => {
                let lookup = (nd_namespace && matches!(&self.scope, ScopeArgs::Columns { .. }))
                    || self.needs_lookup()?;
                if lookup {
                    roots
                        .source
                        .as_ref()
                        .map(|source| Source::new(source.bind(py)))
                        .transpose()
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn needs_lookup(&self) -> PyResult<bool> {
        let mut values: Vec<&Bound<'_, PyAny>> = Vec::new();
        if let Some(input) = &self.input {
            values.push(input.value);
        }
        match &self.scope {
            ScopeArgs::Keyed(keys) => values.extend(keys.iter().map(|(_, value)| value)),
            ScopeArgs::Columns {
                column_key,
                other_keys,
            } => {
                values.push(column_key);
                values.extend(other_keys.iter().map(|(_, value)| value));
            }
            ScopeArgs::Bins {
                lower,
                upper,
                facets,
            } => {
                values.extend([*lower, *upper]);
                values.extend(facets.iter().map(|(_, value)| value));
            }
            _ => {}
        }
        for value in values {
            if !direct(value)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn direct(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(super::native_sequence(value)
        || super::instance(value, "numpy", "ndarray")?
        || super::instance(value, "pandas", "Series")?
        || super::instance(value, "pandas", "DataFrame")?
        || super::instance(value, "polars", "Series")?
        || super::instance(value, "polars", "DataFrame")?)
}

impl Call<'_> {
    fn with_column_namespace<R>(
        &self,
        apply: impl FnOnce(Option<nd::ColumnProjection<'_>>) -> PyResult<R>,
    ) -> PyResult<R> {
        let Some(namespace) = &self.namespace else {
            return apply(None);
        };
        let identities = namespace.identities.as_ref().map(|identities| {
            identities
                .iter()
                .map(|(index, raw)| (*index, raw.as_str()))
                .collect::<Vec<_>>()
        });
        let variable = self.variable();
        if let Some(index) = variable.selected {
            let values = variable.values.values.view();
            let entries = [(index, values.as_ref())];
            let projection =
                nd::ColumnProjection::new(namespace.shape, &entries).map_err(super::nd_error)?;
            let projection = if let Some(identities) = &identities {
                projection
                    .with_identities(identities)
                    .map_err(super::nd_error)?
            } else {
                projection
            };
            apply(Some(projection))
        } else if let Some(identities) = &identities {
            // A truthful current-context namespace with no bound source values.
            // Neither direct alpha cells nor direct column IDs become fake fields.
            let projection = nd::ColumnProjection::from_identities(namespace.shape, identities)
                .map_err(super::nd_error)?;
            apply(Some(projection))
        } else {
            // Ordinary positional sources have no explicit identities or target
            // bounds check in source::identity. With every input already direct,
            // source=None is that same Column-N protocol, not an empty ID table.
            apply(None)
        }
    }
}

fn no_generated_dimension() -> PyErr {
    super::nd_error(nd::AdapterError::Binding {
        role: "column alpha",
        detail: "no generated variable dimension".to_owned(),
    })
}

fn core_encoding(call: &Call<'_>, failure: &mut Option<PyErr>) -> PyResult<AlphaEncoding> {
    match &call.scope {
        Scope::Literal(value) => {
            AlphaEncoding::single(AlphaValues::new("Alpha", [*value]).map_err(crate::native_error)?)
                .map_err(crate::native_error)
        }
        Scope::Row | Scope::Cells | Scope::Linear => Ok(AlphaEncoding::observations(
            call.variable().mapped(failure)?,
        )),
        Scope::Single => {
            AlphaEncoding::single(call.variable().mapped(failure)?).map_err(crate::native_error)
        }
        Scope::Keyed(keys) => {
            let variable = call.variable();
            let (keys, values) = if matches!(&variable.mapping, Mapping::Raw) {
                (keys.native().map_err(crate::native_error)?, variable.raw()?)
            } else {
                let values = variable.mapped(failure)?;
                (keys.native().map_err(crate::native_error)?, values)
            };
            AlphaEncoding::keyed(keys, values).map_err(crate::native_error)
        }
        Scope::Columns { .. } => {
            // Core has no generated dimension. Match the adapter's mapped order:
            // a factory (and any original PyErr) precedes this scope failure.
            if !matches!(&call.variable().mapping, Mapping::Raw) {
                call.variable().mapped(failure)?;
            }
            Err(no_generated_dimension())
        }
        Scope::Bins {
            lower,
            upper,
            facets,
        } => {
            let variable = call.variable();
            let mapped = if matches!(&variable.mapping, Mapping::Raw) {
                None
            } else {
                Some(variable.mapped(failure)?)
            };
            let bounds =
                core::Bounds::new(lower.numeric("bin lower")?, upper.numeric("bin upper")?);
            let facets = facets.native().map_err(crate::native_error)?;
            let values = mapped.map_or_else(|| variable.raw(), Ok)?;
            AlphaEncoding::bins(bounds, facets, values).map_err(crate::native_error)
        }
    }
}

// Both arms are ordinary inputs. Only a real selected alpha field is addressed
// through the restored source; direct vectors never install synthetic identities.
macro_rules! nd_column_value {
    ($variable:expr, $view:ident, |$input:ident| $body:expr) => {{
        let variable = $variable;
        if let Some(index) = variable.selected {
            let $input = index;
            $body
        } else {
            let $input = variable.$view();
            $body
        }
    }};
}

// SourceRowsInput deliberately sees the original integer dtype and rank. Its
// native column_key implementation, not this bridge, checks exact IDs and grids.
macro_rules! nd_column_key {
    ($key:expr, |$input:ident| $body:expr) => {{
        let key = $key;
        macro_rules! bind {
            ($values:expr) => {{
                if key.rank == 1 {
                    let $input = $values.column(0);
                    $body
                } else {
                    let $input = $values.view();
                    $body
                }
            }};
        }
        match key.data.as_slice() {
            [Scratch::I8(values)] => bind!(values),
            [Scratch::I16(values)] => bind!(values),
            [Scratch::I32(values)] => bind!(values),
            [Scratch::I64(values)] => bind!(values),
            [Scratch::I128(values)] => bind!(values),
            [Scratch::U8(values)] => bind!(values),
            [Scratch::U16(values)] => bind!(values),
            [Scratch::U32(values)] => bind!(values),
            [Scratch::U64(values)] => bind!(values),
            [Scratch::U128(values)] => bind!(values),
            [Scratch::F32(values)] => bind!(values),
            [Scratch::F64(values)] => bind!(values),
            [Scratch::Bool(values)] => bind!(values),
            [Scratch::String(values)] => bind!(values),
            _ => {
                // A heterogeneous rectangle remains a collection. The original
                // producer rejects selected_columns before attempting ID reads.
                let indices = (0..key.names.len()).collect::<Vec<_>>();
                let $input = indices.as_slice();
                $body
            }
        }
    }};
}

macro_rules! nd_columns_call {
    ($config:expr, $call:expr, $source:expr, $method:ident, $from:ident,
        $key:expr, $keys:expr, $input:expr $(, $mapper:expr)?) => {{
        let config = $config;
        let call = $call;
        let adapter = match $source {
            Some(source) if call.independent => {
                nd::PlotAdapter::from_parts(config.native.clone(), match &config.origin {
                    Origin::Nd(metadata) => metadata.clone(),
                    _ => unreachable!("ND dispatch retains its origin"),
                })
                .$from(source, $key, $keys.nd(), $input $(, $mapper)?)
            }
            Some(source) => nd::PlotAdapter::from_parts_with_data(
                config.native.clone(),
                match &config.origin {
                    Origin::Nd(metadata) => metadata.clone(),
                    _ => unreachable!("ND dispatch retains its origin"),
                },
                source,
            )
            .map_err(super::nd_error)?
            .$method($key, $keys.nd(), $input $(, $mapper)?),
            None => nd::PlotAdapter::from_parts(
                config.native.clone(),
                match &config.origin {
                    Origin::Nd(metadata) => metadata.clone(),
                    _ => unreachable!("ND dispatch retains its origin"),
                },
            )
            .$method($key, $keys.nd(), $input $(, $mapper)?),
        }
        .map_err(super::nd_error)?;
        Ok(from_nd(adapter))
    }};
}

macro_rules! alpha_target {
    ($cap:ident, $apply:ident, $nd:ident, $nd_columns:ident,
        $pl:ident, $pl_keyed:ident, $pl_columns:ident,
        $base:ident, $cells:ident, $linear:ident, $map:ident, $categories:ident,
        $cells_map:ident, $cells_categories:ident,
        $single:ident, $single_map:ident, $single_categories:ident,
        $keyed:ident, $keyed_data:ident, $keyed_map:ident, $keyed_map_data:ident,
        $keyed_categories:ident, $keyed_categories_data:ident,
        $columns:ident, $columns_data:ident, $columns_map:ident, $columns_map_data:ident,
        $columns_categories:ident, $columns_categories_data:ident) => {
        fn $apply<K: kind::$cap>(
            _py: Python<'_>,
            config: &Config<K>,
            call: Call<'_>,
        ) -> PyResult<Prepared<K>> {
            let mut failure = None;
            let candidate = match &config.origin {
                Origin::Core => core_encoding(&call, &mut failure)
                    .map(|encoding| config.clone().map_native(|native| native.$base(encoding))),
                Origin::Nd(_) => $nd(config, &call, &mut failure),
                Origin::Polars(_) => $pl(config, &call, &mut failure),
            };
            // A native error or reentrant edit cannot replace the original PyErr.
            // All adapter/projection borrows have ended before either return.
            if let Some(error) = failure {
                return Err(error);
            }
            Ok(call.finish(candidate?))
        }

        fn $nd<K: kind::$cap>(
            config: &Config<K>,
            call: &Call<'_>,
            failure: &mut Option<PyErr>,
        ) -> PyResult<Config<K>> {
            let Origin::Nd(metadata) = &config.origin else {
                unreachable!("ND dispatch retains its origin")
            };
            match &call.scope {
                Scope::Literal(value) => {
                    return Ok(from_nd(
                        nd::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                            .$single(*value)
                            .map_err(super::nd_error)?,
                    ));
                }
                Scope::Columns { .. } => return $nd_columns(config, call, failure),
                _ => {}
            }
            let variable = call.variable();
            variable.values.with_nd(|projection| {
                if let Scope::Keyed(keys) = &call.scope {
                    if call.independent {
                        let adapter =
                            nd::PlotAdapter::from_parts(config.native.clone(), metadata.clone());
                        let adapter = match &variable.mapping {
                            Mapping::Raw => adapter.$keyed_data(projection, keys.nd(), 0usize),
                            Mapping::Numeric(factory) => {
                                adapter.$keyed_map_data(projection, keys.nd(), 0usize, |values| {
                                    factory.apply(values, failure)
                                })
                            }
                            Mapping::Categories(mapper) => adapter.$keyed_categories_data(
                                projection,
                                keys.nd(),
                                0usize,
                                |values| map_categories(mapper, values, failure),
                            ),
                        }
                        .map_err(super::nd_error)?;
                        return Ok(from_nd(adapter));
                    }
                }
                let adapter = nd::PlotAdapter::from_parts_with_data(
                    config.native.clone(),
                    metadata.clone(),
                    projection,
                )
                .map_err(super::nd_error)?;
                let adapter = match (&call.scope, &variable.mapping) {
                    (Scope::Row, Mapping::Raw) => adapter.$base(0usize),
                    (Scope::Cells, Mapping::Raw) => adapter.$cells(0usize),
                    (Scope::Single, Mapping::Raw) => adapter.$single(0usize),
                    (Scope::Row, Mapping::Numeric(factory)) => {
                        adapter.$map(0usize, |values| factory.apply(values, failure))
                    }
                    (Scope::Cells, Mapping::Numeric(factory)) => {
                        adapter.$cells_map(0usize, |values| factory.apply(values, failure))
                    }
                    (Scope::Single, Mapping::Numeric(factory)) => {
                        adapter.$single_map(0usize, |values| factory.apply(values, failure))
                    }
                    (Scope::Linear, Mapping::Numeric(NumericFactory::Linear(domain, range))) => {
                        adapter.$linear(0usize, *domain, *range)
                    }
                    (Scope::Row, Mapping::Categories(mapper)) => adapter
                        .$categories(0usize, |values| map_categories(mapper, values, failure)),
                    (Scope::Cells, Mapping::Categories(mapper)) => adapter
                        .$cells_categories(0usize, |values| {
                            map_categories(mapper, values, failure)
                        }),
                    (Scope::Single, Mapping::Categories(mapper)) => adapter
                        .$single_categories(0usize, |values| {
                            map_categories(mapper, values, failure)
                        }),
                    (Scope::Keyed(keys), Mapping::Raw) => adapter.$keyed(keys.nd(), 0usize),
                    (Scope::Keyed(keys), Mapping::Numeric(factory)) => {
                        adapter
                            .$keyed_map(keys.nd(), 0usize, |values| factory.apply(values, failure))
                    }
                    (Scope::Keyed(keys), Mapping::Categories(mapper)) => {
                        adapter.$keyed_categories(keys.nd(), 0usize, |values| {
                            map_categories(mapper, values, failure)
                        })
                    }
                    _ => unreachable!("the closed request admits only these target scopes"),
                }
                .map_err(super::nd_error)?;
                Ok(from_nd(adapter))
            })
        }

        fn $nd_columns<K: kind::$cap>(
            config: &Config<K>,
            call: &Call<'_>,
            failure: &mut Option<PyErr>,
        ) -> PyResult<Config<K>> {
            let Scope::Columns {
                column_key,
                other_keys,
            } = &call.scope
            else {
                unreachable!("columns dispatch receives the columns scope")
            };
            let variable = call.variable();
            call.with_column_namespace(|source| {
                nd_column_key!(column_key, |key| {
                    match &variable.mapping {
                        Mapping::Raw => nd_column_value!(variable, bytes, |input| {
                            nd_columns_call!(
                                config,
                                call,
                                source,
                                $columns,
                                $columns_data,
                                key,
                                other_keys,
                                input
                            )
                        }),
                        Mapping::Numeric(factory) => nd_column_value!(variable, numbers, |input| {
                            nd_columns_call!(
                                config,
                                call,
                                source,
                                $columns_map,
                                $columns_map_data,
                                key,
                                other_keys,
                                input,
                                |values| factory.apply(variable.numeric_source(values), failure)
                            )
                        }),
                        Mapping::Categories(mapper) => {
                            nd_column_value!(variable, categories, |input| {
                                nd_columns_call!(
                                    config,
                                    call,
                                    source,
                                    $columns_categories,
                                    $columns_categories_data,
                                    key,
                                    other_keys,
                                    input,
                                    |values| map_categories(
                                        mapper,
                                        variable.category_source(values)?,
                                        failure
                                    )
                                )
                            })
                        }
                    }
                })
            })
        }

        fn $pl<K: kind::$cap>(
            config: &Config<K>,
            call: &Call<'_>,
            failure: &mut Option<PyErr>,
        ) -> PyResult<Config<K>> {
            let Origin::Polars(metadata) = &config.origin else {
                unreachable!("PL dispatch retains its origin")
            };
            let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone());
            match &call.scope {
                Scope::Literal(value) => {
                    return Ok(from_pl(adapter.$single(*value).map_err(super::pl_error)?));
                }
                Scope::Keyed(_) => return $pl_keyed(adapter, call, failure),
                Scope::Columns { .. } => return $pl_columns(adapter, call, failure),
                _ => {}
            }
            let variable = call.variable();
            let input = variable.values.column();
            let adapter = match (&call.scope, &variable.mapping) {
                (Scope::Row, Mapping::Raw) => adapter.$base(&input),
                (Scope::Cells, Mapping::Raw) => adapter.$cells(&input),
                (Scope::Single, Mapping::Raw) => adapter.$single(&input),
                (Scope::Row, Mapping::Numeric(factory)) => {
                    adapter.$map(&input, |values| factory.apply(values, failure))
                }
                (Scope::Cells, Mapping::Numeric(factory)) => {
                    adapter.$cells_map(&input, |values| factory.apply(values, failure))
                }
                (Scope::Single, Mapping::Numeric(factory)) => {
                    adapter.$single_map(&input, |values| factory.apply(values, failure))
                }
                (Scope::Linear, Mapping::Numeric(NumericFactory::Linear(domain, range))) => {
                    adapter.$linear(&input, *domain, *range)
                }
                (Scope::Row, Mapping::Categories(mapper)) => {
                    adapter.$categories(&input, |values| map_categories(mapper, values, failure))
                }
                (Scope::Cells, Mapping::Categories(mapper)) => adapter
                    .$cells_categories(&input, |values| map_categories(mapper, values, failure)),
                (Scope::Single, Mapping::Categories(mapper)) => adapter
                    .$single_categories(&input, |values| map_categories(mapper, values, failure)),
                _ => unreachable!("the closed request admits only these target scopes"),
            }
            .map_err(super::pl_error)?;
            Ok(from_pl(adapter))
        }

        fn $pl_keyed<K: kind::$cap>(
            adapter: pl::PlotAdapter<'static, K>,
            call: &Call<'_>,
            failure: &mut Option<PyErr>,
        ) -> PyResult<Config<K>> {
            let Scope::Keyed(keys) = &call.scope else {
                unreachable!("keyed dispatch receives the keyed scope")
            };
            let variable = call.variable();
            let input = variable.values.column();
            let columns = keys.columns();
            // Every selector has resolved in its actual Python context. These
            // original from-data calls receive only Rust-owned Column inputs;
            // their ColumnInput implementation never looks up a frame field.
            let source = DataFrame::empty_with_height(call.source_rows);
            let adapter = match (&variable.mapping, call.independent) {
                (Mapping::Raw, false) => adapter.$keyed(keys.pl(&columns), &input),
                (Mapping::Raw, true) => adapter.$keyed_data(&source, keys.pl(&columns), &input),
                (Mapping::Numeric(factory), false) => {
                    adapter.$keyed_map(keys.pl(&columns), &input, |values| {
                        factory.apply(values, failure)
                    })
                }
                (Mapping::Numeric(factory), true) => {
                    adapter.$keyed_map_data(&source, keys.pl(&columns), &input, |values| {
                        factory.apply(values, failure)
                    })
                }
                (Mapping::Categories(mapper), false) => {
                    adapter.$keyed_categories(keys.pl(&columns), &input, |values| {
                        map_categories(mapper, values, failure)
                    })
                }
                (Mapping::Categories(mapper), true) => {
                    adapter.$keyed_categories_data(&source, keys.pl(&columns), &input, |values| {
                        map_categories(mapper, values, failure)
                    })
                }
            }
            .map_err(super::pl_error)?;
            Ok(from_pl(adapter))
        }

        fn $pl_columns<K: kind::$cap>(
            adapter: pl::PlotAdapter<'static, K>,
            call: &Call<'_>,
            failure: &mut Option<PyErr>,
        ) -> PyResult<Config<K>> {
            let Scope::Columns {
                column_key,
                other_keys,
            } = &call.scope
            else {
                unreachable!("columns dispatch receives the columns scope")
            };
            let variable = call.variable();
            let input = variable.values.column();
            let key = column_key.column()?;
            let columns = other_keys.columns();
            let source = DataFrame::empty_with_height(call.source_rows);
            let adapter = match (&variable.mapping, call.independent) {
                (Mapping::Raw, false) => adapter.$columns(&key, other_keys.pl(&columns), &input),
                (Mapping::Raw, true) => {
                    adapter.$columns_data(&source, &key, other_keys.pl(&columns), &input)
                }
                (Mapping::Numeric(factory), false) => {
                    adapter.$columns_map(&key, other_keys.pl(&columns), &input, |values| {
                        factory.apply(values, failure)
                    })
                }
                (Mapping::Numeric(factory), true) => adapter.$columns_map_data(
                    &source,
                    &key,
                    other_keys.pl(&columns),
                    &input,
                    |values| factory.apply(values, failure),
                ),
                (Mapping::Categories(mapper), false) => {
                    adapter.$columns_categories(&key, other_keys.pl(&columns), &input, |values| {
                        map_categories(mapper, values, failure)
                    })
                }
                (Mapping::Categories(mapper), true) => adapter.$columns_categories_data(
                    &source,
                    &key,
                    other_keys.pl(&columns),
                    &input,
                    |values| map_categories(mapper, values, failure),
                ),
            }
            .map_err(super::pl_error)?;
            Ok(from_pl(adapter))
        }
    };
}

alpha_target!(
    PointAlpha,
    apply_point,
    point_nd,
    point_nd_columns,
    point_pl,
    point_pl_keyed,
    point_pl_columns,
    point_alpha,
    point_alpha_cells,
    point_alpha_linear,
    point_alpha_map,
    point_alpha_map_categories,
    point_alpha_cells_map,
    point_alpha_cells_map_categories,
    point_alpha_single,
    point_alpha_single_map,
    point_alpha_single_map_categories,
    point_alpha_keyed,
    point_alpha_keyed_from_data,
    point_alpha_keyed_map,
    point_alpha_keyed_map_from_data,
    point_alpha_keyed_map_categories,
    point_alpha_keyed_map_categories_from_data,
    point_alpha_columns,
    point_alpha_columns_from_data,
    point_alpha_columns_map,
    point_alpha_columns_map_from_data,
    point_alpha_columns_map_categories,
    point_alpha_columns_map_categories_from_data
);
alpha_target!(
    LineAlpha,
    apply_line,
    line_nd,
    line_nd_columns,
    line_pl,
    line_pl_keyed,
    line_pl_columns,
    line_alpha,
    line_alpha_cells,
    line_alpha_linear,
    line_alpha_map,
    line_alpha_map_categories,
    line_alpha_cells_map,
    line_alpha_cells_map_categories,
    line_alpha_single,
    line_alpha_single_map,
    line_alpha_single_map_categories,
    line_alpha_keyed,
    line_alpha_keyed_from_data,
    line_alpha_keyed_map,
    line_alpha_keyed_map_from_data,
    line_alpha_keyed_map_categories,
    line_alpha_keyed_map_categories_from_data,
    line_alpha_columns,
    line_alpha_columns_from_data,
    line_alpha_columns_map,
    line_alpha_columns_map_from_data,
    line_alpha_columns_map_categories,
    line_alpha_columns_map_categories_from_data
);
alpha_target!(
    FillAlpha,
    apply_fill,
    fill_nd,
    fill_nd_columns,
    fill_pl,
    fill_pl_keyed,
    fill_pl_columns,
    fill_alpha,
    fill_alpha_cells,
    fill_alpha_linear,
    fill_alpha_map,
    fill_alpha_map_categories,
    fill_alpha_cells_map,
    fill_alpha_cells_map_categories,
    fill_alpha_single,
    fill_alpha_single_map,
    fill_alpha_single_map_categories,
    fill_alpha_keyed,
    fill_alpha_keyed_from_data,
    fill_alpha_keyed_map,
    fill_alpha_keyed_map_from_data,
    fill_alpha_keyed_map_categories,
    fill_alpha_keyed_map_categories_from_data,
    fill_alpha_columns,
    fill_alpha_columns_from_data,
    fill_alpha_columns_map,
    fill_alpha_columns_map_from_data,
    fill_alpha_columns_map_categories,
    fill_alpha_columns_map_categories_from_data
);

fn apply_bins(
    _py: Python<'_>,
    config: &Config<kind::Histogram>,
    call: Call<'_>,
) -> PyResult<Prepared<kind::Histogram>> {
    let mut failure = None;
    let candidate = match &config.origin {
        Origin::Core => core_encoding(&call, &mut failure).map(|encoding| {
            config
                .clone()
                .map_native(|native| native.fill_alpha(encoding))
        }),
        Origin::Nd(_) => bins_nd(config, &call, &mut failure),
        Origin::Polars(_) => bins_pl(config, &call, &mut failure),
    };
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(call.finish(candidate?))
}

fn bins_nd(
    config: &Config<kind::Histogram>,
    call: &Call<'_>,
    failure: &mut Option<PyErr>,
) -> PyResult<Config<kind::Histogram>> {
    let Origin::Nd(metadata) = &config.origin else {
        unreachable!("ND dispatch retains its origin")
    };
    let Scope::Bins {
        lower,
        upper,
        facets,
    } = &call.scope
    else {
        unreachable!("bins dispatch receives the bins scope")
    };
    let variable = call.variable();
    variable.values.with_nd(|projection| {
        let lower = numeric_cells(lower);
        let upper = numeric_cells(upper);
        let adapter = if call.independent {
            let adapter = nd::Histogram::from_parts(config.native.clone(), metadata.clone());
            match &variable.mapping {
                Mapping::Raw => {
                    adapter.fill_alpha_bins_from_data(projection, lower, upper, facets.nd(), 0usize)
                }
                Mapping::Numeric(factory) => adapter.fill_alpha_bins_map_from_data(
                    projection,
                    lower,
                    upper,
                    facets.nd(),
                    0usize,
                    |values| factory.apply(values, failure),
                ),
                Mapping::Categories(mapper) => adapter.fill_alpha_bins_map_categories_from_data(
                    projection,
                    lower,
                    upper,
                    facets.nd(),
                    0usize,
                    |values| map_categories(mapper, values, failure),
                ),
            }
        } else {
            let adapter = nd::Histogram::from_parts_with_data(
                config.native.clone(),
                metadata.clone(),
                projection,
            )
            .map_err(super::nd_error)?;
            match &variable.mapping {
                Mapping::Raw => adapter.fill_alpha_bins(lower, upper, facets.nd(), 0usize),
                Mapping::Numeric(factory) => {
                    adapter.fill_alpha_bins_map(lower, upper, facets.nd(), 0usize, |values| {
                        factory.apply(values, failure)
                    })
                }
                Mapping::Categories(mapper) => adapter.fill_alpha_bins_map_categories(
                    lower,
                    upper,
                    facets.nd(),
                    0usize,
                    |values| map_categories(mapper, values, failure),
                ),
            }
        }
        .map_err(super::nd_error)?;
        Ok(from_nd(adapter))
    })
}

fn bins_pl(
    config: &Config<kind::Histogram>,
    call: &Call<'_>,
    failure: &mut Option<PyErr>,
) -> PyResult<Config<kind::Histogram>> {
    let Origin::Polars(metadata) = &config.origin else {
        unreachable!("PL dispatch retains its origin")
    };
    let Scope::Bins {
        lower,
        upper,
        facets,
    } = &call.scope
    else {
        unreachable!("bins dispatch receives the bins scope")
    };
    let variable = call.variable();
    let lower = lower.column();
    let upper = upper.column();
    let input = variable.values.column();
    let columns = facets.columns();
    let source = DataFrame::empty_with_height(call.source_rows);
    let adapter = pl::Histogram::from_parts(config.native.clone(), metadata.clone());
    let adapter = match (&variable.mapping, call.independent) {
        (Mapping::Raw, false) => {
            adapter.fill_alpha_bins(&lower, &upper, facets.pl(&columns), &input)
        }
        (Mapping::Raw, true) => {
            adapter.fill_alpha_bins_from_data(&source, &lower, &upper, facets.pl(&columns), &input)
        }
        (Mapping::Numeric(factory), false) => {
            adapter.fill_alpha_bins_map(&lower, &upper, facets.pl(&columns), &input, |values| {
                factory.apply(values, failure)
            })
        }
        (Mapping::Numeric(factory), true) => adapter.fill_alpha_bins_map_from_data(
            &source,
            &lower,
            &upper,
            facets.pl(&columns),
            &input,
            |values| factory.apply(values, failure),
        ),
        (Mapping::Categories(mapper), false) => adapter.fill_alpha_bins_map_categories(
            &lower,
            &upper,
            facets.pl(&columns),
            &input,
            |values| map_categories(mapper, values, failure),
        ),
        (Mapping::Categories(mapper), true) => adapter.fill_alpha_bins_map_categories_from_data(
            &source,
            &lower,
            &upper,
            facets.pl(&columns),
            &input,
            |values| map_categories(mapper, values, failure),
        ),
    }
    .map_err(super::pl_error)?;
    Ok(from_pl(adapter))
}
