//! Typed caller-computed results through the original standalone producers.

use lichtung_plot::{self as core, kind};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use ndarray::{Array1, ArrayView1};
use polars_core::prelude::{Column, DataFrame};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};

use super::array::Role;
use super::binding::{self, Source};
use super::columns::{self, Scalar};
use super::families;
use super::{Config, InputRoots, Prepared};

type Input<'a, 'py> = &'a Bound<'py, PyAny>;

#[derive(Clone, Copy)]
pub(crate) enum ResultSourceArgs<'a, 'py> {
    Main,
    Independent(Option<Input<'a, 'py>>),
}

#[derive(Clone, Copy)]
pub(crate) struct ResultTargetArgs<'a, 'py> {
    pub(crate) input: Input<'a, 'py>,
    pub(crate) series: Input<'a, 'py>,
    pub(crate) value: Input<'a, 'py>,
}

#[derive(Clone, Copy)]
pub(crate) struct ResultMarksArgs<'a, 'py> {
    pub(crate) marks: Input<'a, 'py>,
    pub(crate) mapping: Input<'a, 'py>,
    pub(crate) mark: Input<'a, 'py>,
}

#[derive(Clone, Copy)]
pub(crate) enum ResultScopeArgs<'a, 'py> {
    Unfaceted,
    Facet {
        facet: Input<'a, 'py>,
        facet_value: Input<'a, 'py>,
    },
    Grid {
        facet_row: Input<'a, 'py>,
        facet_column: Input<'a, 'py>,
        facet_value: Input<'a, 'py>,
    },
}

#[derive(Clone, Copy)]
pub(crate) struct AttachedArgs<'a, 'py> {
    pub(crate) source: ResultSourceArgs<'a, 'py>,
    pub(crate) target: ResultTargetArgs<'a, 'py>,
    pub(crate) marks: ResultMarksArgs<'a, 'py>,
    pub(crate) scope: ResultScopeArgs<'a, 'py>,
    pub(crate) unset: Input<'a, 'py>,
}

#[derive(Clone, Copy)]
pub(crate) struct PairwiseArgs<'a, 'py> {
    pub(crate) source: ResultSourceArgs<'a, 'py>,
    pub(crate) left: ResultTargetArgs<'a, 'py>,
    pub(crate) right: ResultTargetArgs<'a, 'py>,
    pub(crate) marks: ResultMarksArgs<'a, 'py>,
    pub(crate) scope: ResultScopeArgs<'a, 'py>,
    pub(crate) unset: Input<'a, 'py>,
}

pub(crate) fn significance_attached<'py, K: kind::Significance<core::Attached>>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: AttachedArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    append_attached(py, config, roots, args)
}

pub(crate) fn significance_pairwise<'py, K: kind::Significance<core::Pairwise>>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: PairwiseArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    append_pairwise(py, config, roots, args)
}

// Each of these closed forms is call-local. Ordinary variables remain ordinary;
// only explicit literal branches become native scalar arguments.
enum Target<C> {
    Column(C),
    Series(C, C),
    Literal(core::SignificanceTarget),
}

type Mapping = Vec<(String, core::SignificanceMark)>;

enum Marks<C> {
    Numeric(C),
    Categorical(C, Mapping),
    Literal(core::SignificanceMark),
}

enum Scope<C> {
    Unfaceted,
    Facet(C),
    Grid(C, C),
    Literal(core::FacetCell),
}

fn validate_target(
    args: &ResultTargetArgs<'_, '_>,
    unset: &Bound<'_, PyAny>,
    name: &str,
) -> PyResult<()> {
    let column = !args.input.is(unset);
    let literal = !args.value.is(unset);
    if column == literal {
        return Err(PyTypeError::new_err(format!(
            "supply exactly one of {name} or {name}_value"
        )));
    }
    let series = !args.series.is(unset) && !args.series.is_none();
    if literal && series {
        return Err(PyTypeError::new_err(format!(
            "{name}_series modifies only the {name} column form"
        )));
    }
    families::required(&[(name, if column { args.input } else { args.value })])
}

fn validate_marks(args: &ResultMarksArgs<'_, '_>, unset: &Bound<'_, PyAny>) -> PyResult<()> {
    let column = !args.marks.is(unset);
    let literal = !args.mark.is(unset);
    let mapping = !args.mapping.is(unset);
    if column == literal || (literal && mapping) {
        return Err(PyTypeError::new_err(
            "supply numeric marks, categorical marks with mapping, or literal mark, exclusively",
        ));
    }
    if mapping && args.mapping.is_none() {
        return Err(PyTypeError::new_err("mapping cannot be None"));
    }
    families::required(&[(
        if column { "marks" } else { "mark" },
        if column { args.marks } else { args.mark },
    )])
}

fn validate_scope(scope: &ResultScopeArgs<'_, '_>, unset: &Bound<'_, PyAny>) -> PyResult<()> {
    match *scope {
        ResultScopeArgs::Unfaceted => Ok(()),
        ResultScopeArgs::Facet { facet, facet_value } => {
            let column = !facet.is(unset);
            let literal = !facet_value.is(unset);
            if column == literal {
                return Err(PyTypeError::new_err(
                    "supply exactly one of facet or facet_value",
                ));
            }
            families::required(&[(
                if column { "facet" } else { "facet_value" },
                if column { facet } else { facet_value },
            )])
        }
        ResultScopeArgs::Grid {
            facet_row,
            facet_column,
            facet_value,
        } => {
            let row = !facet_row.is(unset);
            let column = !facet_column.is(unset);
            let literal = !facet_value.is(unset);
            if row != column || row == literal {
                return Err(PyTypeError::new_err(
                    "supply facet_row and facet_column together, or facet_value, exclusively",
                ));
            }
            if literal {
                families::required(&[("facet_value", facet_value)])
            } else {
                families::required(&[("facet_row", facet_row), ("facet_column", facet_column)])
            }
        }
    }
}

fn literal_pair(value: &Bound<'_, PyAny>, name: &str) -> PyResult<(String, String)> {
    let pair = value
        .cast::<PyTuple>()
        .map_err(|_| PyTypeError::new_err(format!("{name} requires a pair of literal strings")))?;
    if pair.len() != 2 {
        return Err(PyTypeError::new_err(format!(
            "{name} requires a pair of literal strings"
        )));
    }
    Ok((pair.get_item(0)?.extract()?, pair.get_item(1)?.extract()?))
}

fn target_form<'a, 'py>(
    args: ResultTargetArgs<'a, 'py>,
    unset: &Bound<'py, PyAny>,
) -> PyResult<Target<Input<'a, 'py>>> {
    if !args.value.is(unset) {
        let target = if args.value.is_instance_of::<PyString>() {
            core::SignificanceTarget::category(args.value.extract::<String>()?)
        } else {
            let (category, series) = literal_pair(args.value, "target value")?;
            core::SignificanceTarget::category_series(category, series)
        };
        Ok(Target::Literal(target))
    } else if !args.series.is(unset) && !args.series.is_none() {
        Ok(Target::Series(args.input, args.series))
    } else {
        Ok(Target::Column(args.input))
    }
}

fn literal_mark(value: &Bound<'_, PyAny>) -> PyResult<core::SignificanceMark> {
    use lichtung_plot::SignificanceMark;
    if value.is_instance_of::<PyString>() {
        return match value.extract::<String>()?.as_str() {
            "ns" => Ok(SignificanceMark::NotSignificant),
            "*" => Ok(SignificanceMark::OneStar),
            "**" => Ok(SignificanceMark::TwoStars),
            "***" => Ok(SignificanceMark::ThreeStars),
            "****" => Ok(SignificanceMark::FourStars),
            _ => Err(PyTypeError::new_err(
                "a literal significance mark requires a number or ns/*/**/***/****",
            )),
        };
    }
    // Do not call glyph or check p-range/nonfinite values here, even for unused
    // mapping entries. Those remain native annotation/preflight decisions.
    Ok(SignificanceMark::PValue(columns::literal(
        value,
        "significance mark",
    )?))
}

fn mapping(value: &Bound<'_, PyAny>) -> PyResult<Mapping> {
    // Pin all Python entries before any mark conversion can reenter an owner.
    let pairs = if let Ok(mapping) = value.cast::<PyDict>() {
        mapping.iter().collect::<Vec<_>>()
    } else {
        let entries = if super::native_sequence(value) {
            super::entries(value, "significance mapping")?
        } else {
            let abc = PyModule::import(value.py(), "collections.abc")?;
            if !value.is_instance(&abc.getattr("Mapping")?)? {
                return Err(PyTypeError::new_err(
                    "mapping requires a Mapping or ordered pairs",
                ));
            }
            value
                .call_method0("items")?
                .try_iter()?
                .collect::<PyResult<Vec<_>>>()?
        };
        entries
            .iter()
            .map(|entry| {
                let pair = super::entries(entry, "significance mapping pair")?;
                if pair.len() != 2 {
                    return Err(PyTypeError::new_err(
                        "significance mapping pairs require exactly two entries",
                    ));
                }
                Ok((pair[0].clone(), pair[1].clone()))
            })
            .collect::<PyResult<Vec<_>>>()?
    };
    pairs
        .into_iter()
        .map(|(key, mark)| Ok((key.extract::<String>()?, literal_mark(&mark)?)))
        .collect()
}

fn marks_form<'a, 'py>(
    args: ResultMarksArgs<'a, 'py>,
    unset: &Bound<'py, PyAny>,
) -> PyResult<Marks<Input<'a, 'py>>> {
    if !args.mark.is(unset) {
        Ok(Marks::Literal(literal_mark(args.mark)?))
    } else if !args.mapping.is(unset) {
        Ok(Marks::Categorical(args.marks, mapping(args.mapping)?))
    } else {
        Ok(Marks::Numeric(args.marks))
    }
}

fn scope_form<'a, 'py>(
    args: ResultScopeArgs<'a, 'py>,
    unset: &Bound<'py, PyAny>,
) -> PyResult<Scope<Input<'a, 'py>>> {
    match args {
        ResultScopeArgs::Unfaceted => Ok(Scope::Unfaceted),
        ResultScopeArgs::Facet { facet, facet_value } => {
            if facet_value.is(unset) {
                Ok(Scope::Facet(facet))
            } else {
                Ok(Scope::Literal(core::FacetCell::single(
                    facet_value.extract::<String>()?,
                )))
            }
        }
        ResultScopeArgs::Grid {
            facet_row,
            facet_column,
            facet_value,
        } => {
            if facet_value.is(unset) {
                Ok(Scope::Grid(facet_row, facet_column))
            } else {
                let (row, column) = literal_pair(facet_value, "grid facet_value")?;
                Ok(Scope::Literal(core::FacetCell::grid(row, column)))
            }
        }
    }
}

enum LookupSource<'py> {
    Main,
    Independent(Option<Source<'py>>),
}

struct Variable {
    scalar: Scalar,
    role: Role,
    name: &'static str,
}

/// Only call-local foreign lookup and Rust-owned variables. No result field IDs
/// or auxiliary Python roots are published to the owner.
struct Lookup<'py> {
    source: LookupSource<'py>,
    roots: InputRoots,
    aliases: Vec<(String, String)>,
    variables: Vec<Variable>,
    source_fields: Vec<(usize, usize)>,
    source_polars: Option<bool>,
    direct_polars: bool,
}

impl<'py> Lookup<'py> {
    fn new(py: Python<'py>, args: ResultSourceArgs<'_, 'py>, roots: &InputRoots) -> PyResult<Self> {
        let (source, local_roots, source_polars) = match args {
            ResultSourceArgs::Main => (
                LookupSource::Main,
                InputRoots {
                    source: roots.source.as_ref().map(|source| source.clone_ref(py)),
                    keys: None,
                },
                None,
            ),
            ResultSourceArgs::Independent(data) => {
                let (source, roots) = families::context(data)?;
                let polars = source
                    .as_ref()
                    .map(|source| matches!(source, Source::Polars { .. }));
                (LookupSource::Independent(source), roots, polars)
            }
        };
        Ok(Self {
            source,
            roots: local_roots,
            aliases: Vec::new(),
            variables: Vec::new(),
            source_fields: Vec::new(),
            source_polars,
            direct_polars: false,
        })
    }

    fn scalar(
        &mut self,
        value: &Bound<'py, PyAny>,
        role: Role,
        name: &'static str,
    ) -> PyResult<usize> {
        let selected = match &self.source {
            LookupSource::Main => {
                // No retained-source discovery for direct variables or literals.
                if let Some((source, selected)) = binding::fresh(&self.roots, value)? {
                    self.source_polars = Some(matches!(&source, Source::Polars { .. }));
                    Some((
                        Scalar::selected(
                            &source,
                            &selected,
                            role,
                            name,
                            &mut self.roots,
                            &mut self.aliases,
                        )?,
                        None,
                    ))
                } else {
                    None
                }
            }
            LookupSource::Independent(Some(source)) => {
                if let Some(selected) = source.resolve(value)? {
                    let scalar = Scalar::selected(
                        source,
                        &selected,
                        role,
                        name,
                        &mut self.roots,
                        &mut self.aliases,
                    )?;
                    Some((scalar, Some(selected.indices[0])))
                } else {
                    None
                }
            }
            LookupSource::Independent(None) => None,
        };
        let (scalar, field) = if let Some(selected) = selected {
            selected
        } else {
            self.direct_polars |= columns::polars_route(None, &[value])?;
            (
                Scalar::prepare(None, value, role, name, &mut self.roots, &mut self.aliases)?,
                None,
            )
        };
        let slot = self.variables.len();
        self.variables.push(Variable { scalar, role, name });
        if let Some(field) = field {
            if !self.source_fields.iter().any(|&(known, _)| known == field) {
                self.source_fields.push((field, slot));
            }
        }
        Ok(slot)
    }

    fn polars(&self) -> bool {
        self.source_polars.unwrap_or(self.direct_polars)
    }

    fn verify(&self) -> PyResult<()> {
        if let LookupSource::Independent(Some(source)) = &self.source {
            source.verify()?;
        }
        Ok(())
    }

    fn with_nd_source<R>(
        &self,
        apply: impl FnOnce(Option<nd::ColumnProjection<'_>>) -> PyResult<R>,
    ) -> PyResult<R> {
        let LookupSource::Independent(Some(source)) = &self.source else {
            return apply(None);
        };
        let views = self
            .source_fields
            .iter()
            .map(|&(_, slot)| self.variables[slot].scalar.values.view())
            .collect::<Vec<_>>();
        let entries = self
            .source_fields
            .iter()
            .zip(&views)
            .map(|(&(index, _), view)| (index, view.as_ref()))
            .collect::<Vec<_>>();
        let identities = self
            .source_fields
            .iter()
            .map(|&(index, slot)| (index, self.variables[slot].scalar.name.as_str()))
            .collect::<Vec<_>>();
        // Only actual selected source fields are projected. A literal/direct-only
        // call needs no values, but still carries the independent source's shape.
        let projection = if entries.is_empty() {
            nd::ColumnProjection::from_identities(source.shape(), &identities)
        } else {
            nd::ColumnProjection::new(source.shape(), &entries)
                .and_then(|projection| projection.with_identities(&identities))
        }
        .map_err(super::nd_error)?;
        apply(Some(projection))
    }

    fn pl_source(&self, variables: &PlVariables) -> PyResult<Option<DataFrame>> {
        let LookupSource::Independent(Some(source)) = &self.source else {
            return Ok(None);
        };
        let columns = self
            .source_fields
            .iter()
            .map(|&(_, slot)| variables.0[slot].clone())
            .collect::<Vec<_>>();
        let source = if columns.is_empty() {
            DataFrame::empty_with_height(source.shape().0)
        } else {
            DataFrame::new(source.shape().0, columns)
                .map_err(|error| crate::DataError::new_err(error.to_string()))?
        };
        Ok(Some(source))
    }
}

fn prepare_scope<'py>(
    scope: Scope<Input<'_, 'py>>,
    lookup: &mut Lookup<'py>,
) -> PyResult<Scope<usize>> {
    Ok(match scope {
        Scope::Unfaceted => Scope::Unfaceted,
        Scope::Facet(facet) => {
            Scope::Facet(lookup.scalar(facet, Role::Category, "significance facet")?)
        }
        Scope::Grid(row, column) => Scope::Grid(
            lookup.scalar(row, Role::Category, "significance facet row")?,
            lookup.scalar(column, Role::Category, "significance facet column")?,
        ),
        Scope::Literal(facet) => Scope::Literal(facet),
    })
}

fn prepare_target<'py>(
    target: Target<Input<'_, 'py>>,
    lookup: &mut Lookup<'py>,
) -> PyResult<Target<usize>> {
    Ok(match target {
        Target::Column(category) => {
            Target::Column(lookup.scalar(category, Role::Category, "significance target")?)
        }
        Target::Series(category, series) => Target::Series(
            lookup.scalar(category, Role::Category, "significance category")?,
            lookup.scalar(series, Role::Category, "significance series")?,
        ),
        Target::Literal(target) => Target::Literal(target),
    })
}

fn prepare_marks<'py>(
    marks: Marks<Input<'_, 'py>>,
    lookup: &mut Lookup<'py>,
) -> PyResult<Marks<usize>> {
    Ok(match marks {
        Marks::Numeric(values) => {
            Marks::Numeric(lookup.scalar(values, Role::Numeric, "significance p-values")?)
        }
        Marks::Categorical(values, mapping) => Marks::Categorical(
            lookup.scalar(values, Role::Category, "significance marks")?,
            mapping,
        ),
        Marks::Literal(mark) => Marks::Literal(mark),
    })
}

enum NdVariable {
    Category(Array1<Option<String>>),
    Numeric(Array1<Option<f64>>),
}

struct NdVariables(Vec<NdVariable>);

impl NdVariables {
    fn prepare(variables: &[Variable]) -> PyResult<Self> {
        let variables = variables
            .iter()
            .map(|variable| {
                let source = variable.scalar.values.view();
                // Raw role extraction only: retain every missing slot and leave
                // missing/category-pair lengths to the standalone producer.
                Ok(match variable.role {
                    Role::Category => NdVariable::Category(Array1::from(
                        source
                            .category_column(0, variable.name)
                            .map_err(super::nd_error)?,
                    )),
                    Role::Numeric => NdVariable::Numeric(Array1::from(
                        source
                            .numeric_column(0, variable.name)
                            .map_err(super::nd_error)?,
                    )),
                    Role::Ids => unreachable!("results have no source-row ID variables"),
                })
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self(variables))
    }

    fn category(&self, slot: usize) -> ArrayView1<'_, Option<String>> {
        match &self.0[slot] {
            NdVariable::Category(values) => values.view(),
            NdVariable::Numeric(_) => unreachable!("category role was prepared as numeric"),
        }
    }

    fn numeric(&self, slot: usize) -> ArrayView1<'_, Option<f64>> {
        match &self.0[slot] {
            NdVariable::Numeric(values) => values.view(),
            NdVariable::Category(_) => unreachable!("numeric role was prepared as category"),
        }
    }
}

struct PlVariables(Vec<Column>);

impl PlVariables {
    fn prepare(variables: &[Variable]) -> Self {
        Self(
            variables
                .iter()
                .map(|variable| variable.scalar.column())
                .collect(),
        )
    }

    fn category(&self, slot: usize) -> &Column {
        &self.0[slot]
    }

    fn numeric(&self, slot: usize) -> &Column {
        &self.0[slot]
    }
}

// Sealed native protocols require concrete argument types. These continuations
// select only the closed column/pair/literal forms, never bind targets or count.
macro_rules! with_target {
    ($target:expr, $variables:expr, |$input:ident| $then:expr) => {
        match $target {
            Target::Column(slot) => {
                let $input = ($variables).category(*slot);
                $then
            }
            Target::Series(category, series) => {
                let $input = (
                    ($variables).category(*category),
                    ($variables).category(*series),
                );
                $then
            }
            Target::Literal(target) => {
                let $input = target.clone();
                $then
            }
        }
    };
}

macro_rules! with_marks {
    ($marks:expr, $variables:expr, |$input:ident| $then:expr) => {
        match $marks {
            Marks::Numeric(slot) => {
                let $input = ($variables).numeric(*slot);
                $then
            }
            Marks::Categorical(slot, mapping) => {
                let $input = (($variables).category(*slot), mapping.as_slice());
                $then
            }
            Marks::Literal(mark) => {
                let $input = *mark;
                $then
            }
        }
    };
}

macro_rules! produce_scoped {
    (
        $adapter:ident, $source:expr, $scope:expr, $variables:expr,
        $base:ident, $from:ident, $facet:ident, $facet_from:ident,
        $grid:ident, $grid_from:ident, ($($input:expr),+)
    ) => {
        match ($source, $scope) {
            (None, Scope::Unfaceted) => $adapter::$base($($input),+),
            (Some(source), Scope::Unfaceted) => $adapter::$from(source, $($input),+),
            (None, Scope::Facet(slot)) => {
                $adapter::$facet($($input),+, ($variables).category(*slot))
            }
            (Some(source), Scope::Facet(slot)) => {
                $adapter::$facet_from(source, $($input),+, ($variables).category(*slot))
            }
            (None, Scope::Grid(row, column)) => {
                $adapter::$grid(
                    $($input),+,
                    ($variables).category(*row),
                    ($variables).category(*column),
                )
            }
            (Some(source), Scope::Grid(row, column)) => {
                $adapter::$grid_from(
                    source, $($input),+,
                    ($variables).category(*row),
                    ($variables).category(*column),
                )
            }
            // A literal grid FacetCell must take the scalar-capable facet path,
            // not _in_grid's two ordinary, non-scalar category variables.
            (None, Scope::Literal(facet)) => $adapter::$facet($($input),+, facet.clone()),
            (Some(source), Scope::Literal(facet)) => {
                $adapter::$facet_from(source, $($input),+, facet.clone())
            }
        }
    };
}

macro_rules! route_producers {
    ($adapter:ident, $source:ty, $variables:ty, $attached:ident, $pairwise:ident, $error:ident) => {
        fn $attached<T: $adapter::SignificanceTargets, M: $adapter::SignificanceMarks>(
            source: $source,
            target: T,
            marks: M,
            scope: &Scope<usize>,
            variables: &$variables,
        ) -> PyResult<Vec<core::Significance<core::Attached>>> {
            produce_scoped!(
                $adapter,
                source,
                scope,
                variables,
                significance_attached,
                significance_attached_from_data,
                significance_attached_in_facet,
                significance_attached_in_facet_from_data,
                significance_attached_in_grid,
                significance_attached_in_grid_from_data,
                (target, marks)
            )
            .map_err(super::$error)
        }

        fn $pairwise<
            L: $adapter::SignificanceTargets,
            R: $adapter::SignificanceTargets,
            M: $adapter::SignificanceMarks,
        >(
            source: $source,
            left: L,
            right: R,
            marks: M,
            scope: &Scope<usize>,
            variables: &$variables,
        ) -> PyResult<Vec<core::Significance<core::Pairwise>>> {
            produce_scoped!(
                $adapter,
                source,
                scope,
                variables,
                significance_pairwise,
                significance_pairwise_from_data,
                significance_pairwise_in_facet,
                significance_pairwise_in_facet_from_data,
                significance_pairwise_in_grid,
                significance_pairwise_in_grid_from_data,
                (left, right, marks)
            )
            .map_err(super::$error)
        }
    };
}

route_producers!(
    nd,
    Option<nd::ColumnProjection<'_>>,
    NdVariables,
    nd_attached,
    nd_pairwise,
    nd_error
);
route_producers!(
    pl,
    Option<&DataFrame>,
    PlVariables,
    pl_attached,
    pl_pairwise,
    pl_error
);

fn attached_results(
    lookup: &Lookup<'_>,
    target: &Target<usize>,
    marks: &Marks<usize>,
    scope: &Scope<usize>,
) -> PyResult<Vec<core::Significance<core::Attached>>> {
    if lookup.polars() {
        let variables = PlVariables::prepare(&lookup.variables);
        let source = lookup.pl_source(&variables)?;
        with_target!(target, variables, |target| {
            with_marks!(marks, variables, |marks| {
                pl_attached(source.as_ref(), target, marks, scope, &variables)
            })
        })
    } else {
        let variables = NdVariables::prepare(&lookup.variables)?;
        lookup.with_nd_source(|source| {
            with_target!(target, variables, |target| {
                with_marks!(marks, variables, |marks| {
                    nd_attached(source, target, marks, scope, &variables)
                })
            })
        })
    }
}

fn pairwise_results(
    lookup: &Lookup<'_>,
    left: &Target<usize>,
    right: &Target<usize>,
    marks: &Marks<usize>,
    scope: &Scope<usize>,
) -> PyResult<Vec<core::Significance<core::Pairwise>>> {
    if lookup.polars() {
        let variables = PlVariables::prepare(&lookup.variables);
        let source = lookup.pl_source(&variables)?;
        with_target!(left, variables, |left| {
            with_target!(right, variables, |right| {
                with_marks!(marks, variables, |marks| {
                    pl_pairwise(source.as_ref(), left, right, marks, scope, &variables)
                })
            })
        })
    } else {
        let variables = NdVariables::prepare(&lookup.variables)?;
        lookup.with_nd_source(|source| {
            with_target!(left, variables, |left| {
                with_target!(right, variables, |right| {
                    with_marks!(marks, variables, |marks| {
                        nd_pairwise(source, left, right, marks, scope, &variables)
                    })
                })
            })
        })
    }
}

fn append_attached<'py, K: kind::Significance<core::Attached>>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: AttachedArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    // Finish the entire presence decision before parsing a Mapping or opening
    // either lookup context; malformed later fields cannot trigger an earlier
    // result-column lookup.
    validate_target(&args.target, args.unset, "target")?;
    validate_marks(&args.marks, args.unset)?;
    validate_scope(&args.scope, args.unset)?;
    let scope = scope_form(args.scope, args.unset)?;
    let target = target_form(args.target, args.unset)?;
    let marks = marks_form(args.marks, args.unset)?;
    let mut lookup = Lookup::new(py, args.source, roots)?;
    let scope = prepare_scope(scope, &mut lookup)?;
    let target = prepare_target(target, &mut lookup)?;
    let marks = prepare_marks(marks, &mut lookup)?;
    lookup.verify()?;
    // Original producers bind facet(s), then target, marks and result count.
    let values = attached_results(&lookup, &target, &marks, &scope)?;
    Ok(Prepared {
        config: config
            .clone()
            .map_native(|native| native.significance(values)),
        roots: roots.clone_ref(py),
    })
}

fn append_pairwise<'py, K: kind::Significance<core::Pairwise>>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: PairwiseArgs<'_, 'py>,
) -> PyResult<Prepared<K>> {
    validate_target(&args.left, args.unset, "left")?;
    validate_target(&args.right, args.unset, "right")?;
    validate_marks(&args.marks, args.unset)?;
    validate_scope(&args.scope, args.unset)?;
    let scope = scope_form(args.scope, args.unset)?;
    let left = target_form(args.left, args.unset)?;
    let right = target_form(args.right, args.unset)?;
    let marks = marks_form(args.marks, args.unset)?;
    let mut lookup = Lookup::new(py, args.source, roots)?;
    let scope = prepare_scope(scope, &mut lookup)?;
    let left = prepare_target(left, &mut lookup)?;
    let right = prepare_target(right, &mut lookup)?;
    let marks = prepare_marks(marks, &mut lookup)?;
    lookup.verify()?;
    // No target materialization/broadcasting or scientific preflight here.
    let values = pairwise_results(&lookup, &left, &right, &marks, &scope)?;
    // All call-local sources, registries, foreign-backed imports and native
    // source projections die on return, outside the caller's paired commit.
    Ok(Prepared {
        config: config
            .clone()
            .map_native(|native| native.significance(values)),
        roots: roots.clone_ref(py),
    })
}
