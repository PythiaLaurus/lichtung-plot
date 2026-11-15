//! Read-only scoped views. Native reads finish before constructing Python values.

use lichtung_plot as core;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::scope::ContextKey;
use crate::values::numeric::PyNumericLabels;

#[derive(Clone, Copy)]
pub(super) enum OwnerSource {
    Numeric,
    Tick,
}

#[derive(Clone, Copy)]
pub(super) enum DomainSource {
    Numeric,
    Tick,
}

#[derive(Clone, Copy)]
pub(super) enum OperandSource {
    Value(usize),
    Lower(DomainSource),
    Upper(DomainSource),
}

#[derive(Clone, Copy)]
pub(super) enum FactorSource {
    Value,
    Common,
}

#[derive(Clone, Copy)]
pub(super) enum Request {
    Index,
    ValueBuiltin,
    SetLength,
    SetRole,
    SetPurpose,
    SetBuiltin,
    Owner(OwnerSource),
    Domain(DomainSource),
    Operand(OperandSource),
    Identity(usize),
    Factors(FactorSource),
    Locale,
    TickAxis,
    TickSpan,
}

pub(super) struct OwnedOwner {
    plot_kind: Option<&'static str>,
    canvas: Option<(usize, usize)>,
    facet: Option<core::FacetCell>,
    shared: bool,
}

pub(super) struct OwnedDomain {
    lower: OwnedOperand,
    upper: OwnedOperand,
    data: Option<(f64, f64)>,
    limits: (f64, f64),
    scale: &'static str,
    origin: &'static str,
}

pub(super) enum OwnedOperand {
    Binary64(f64),
    PowerOfTen(f64),
    Multiplicative {
        anchor: f64,
        ratio: f64,
        index: String,
    },
}

pub(super) struct OwnedIdentity {
    kind: &'static str,
    axis: Option<&'static str>,
    source_index: Option<usize>,
    row: Option<String>,
    column: Option<String>,
    category: Option<String>,
    index: Option<usize>,
}

pub(super) struct OwnedFactors {
    multiplier: f64,
    decimal_scale: i16,
    rate: &'static str,
    divisor: core::NumericFactor,
    prefix: String,
    suffix: String,
    unit: Option<core::NumericUnit>,
}

/// Every field is owned, including explanation Text and arbitrary exponent text.
pub(super) enum OwnedReply {
    Index(usize),
    Name(&'static str),
    Number(f64),
    Text(String),
    Labels(core::NumericLabels),
    Owner(OwnedOwner),
    Domain(Option<OwnedDomain>),
    Operand(OwnedOperand),
    Identity(OwnedIdentity),
    Factors(Option<OwnedFactors>),
    Locale(core::NumericLocale),
}

fn axis(value: core::CoordinateAxis) -> &'static str {
    match value {
        core::CoordinateAxis::X => "x",
        core::CoordinateAxis::Y => "y",
    }
}

fn plot_kind(value: core::PlotKind) -> &'static str {
    match value {
        core::PlotKind::Scatter => "scatter",
        core::PlotKind::Histogram => "histogram",
        core::PlotKind::Density => "density",
        core::PlotKind::Qq => "qq",
        core::PlotKind::PointInterval => "point_interval",
        core::PlotKind::Line => "line",
        core::PlotKind::Trajectory => "trajectory",
        core::PlotKind::LinePath => "line_path",
        core::PlotKind::Interaction => "interaction",
        core::PlotKind::Coefficient => "coefficient",
        core::PlotKind::Forest => "forest",
        core::PlotKind::Dumbbell => "dumbbell",
        core::PlotKind::Heatmap => "heatmap",
        core::PlotKind::Bar => "bar",
        core::PlotKind::HorizontalBar => "horizontal_bar",
        core::PlotKind::StackedBar => "stacked_bar",
        core::PlotKind::StackedHorizontalBar => "stacked_horizontal_bar",
        core::PlotKind::Boxplot => "boxplot",
        core::PlotKind::Violin => "violin",
        core::PlotKind::Raincloud => "raincloud",
        core::PlotKind::Regression => "regression",
        core::PlotKind::Marginal => "marginal",
        core::PlotKind::Matrix => "matrix",
        core::PlotKind::Beeswarm => "beeswarm",
        core::PlotKind::Waterfall => "waterfall",
    }
}

fn owner(value: &core::NumericOwner<'_>) -> OwnedOwner {
    OwnedOwner {
        plot_kind: value.plot_kind().map(plot_kind),
        canvas: value.canvas_cell(),
        facet: value.facet_cell().cloned(),
        shared: value.is_shared(),
    }
}

fn operand(value: core::NumericOperand<'_>) -> OwnedOperand {
    match value {
        core::NumericOperand::Binary64(value) => OwnedOperand::Binary64(value),
        core::NumericOperand::PowerOfTen { exponent } => OwnedOperand::PowerOfTen(exponent),
        core::NumericOperand::Multiplicative {
            anchor,
            ratio,
            index,
        } => OwnedOperand::Multiplicative {
            anchor,
            ratio,
            index: index.to_owned(),
        },
    }
}

fn domain(value: &core::NumericDomain<'_>) -> OwnedDomain {
    OwnedDomain {
        lower: operand(value.lower()),
        upper: operand(value.upper()),
        data: value.data_limits(),
        limits: value.scale_limits(),
        scale: match value.scale() {
            core::AxisScale::Linear => "linear",
            core::AxisScale::Log10 => "log10",
        },
        origin: match value.origin() {
            core::NumericRangeOrigin::Automatic => "automatic",
            core::NumericRangeOrigin::Explicit => "explicit",
            core::NumericRangeOrigin::ParentResolved => "parent_resolved",
        },
    }
}

fn identity(value: &core::NumericIdentity<'_>) -> OwnedIdentity {
    let mut owned = OwnedIdentity {
        kind: "",
        axis: None,
        source_index: None,
        row: None,
        column: None,
        category: None,
        index: None,
    };
    match value {
        core::NumericIdentity::Tick {
            axis: coordinate,
            source_index,
        } => {
            owned.kind = "tick";
            owned.axis = Some(axis(*coordinate));
            owned.source_index = Some(*source_index);
        }
        core::NumericIdentity::GuideTick { source_index } => {
            owned.kind = "guide_tick";
            owned.source_index = Some(*source_index);
        }
        core::NumericIdentity::HeatmapCell { row, column } => {
            owned.kind = "heatmap_cell";
            owned.row = Some((*row).to_owned());
            owned.column = Some((*column).to_owned());
        }
        core::NumericIdentity::EstimateRow { category } => {
            owned.kind = "estimate_row";
            owned.category = Some((*category).to_owned());
        }
        core::NumericIdentity::WaterfallBase => owned.kind = "waterfall_base",
        core::NumericIdentity::WaterfallTerminal => owned.kind = "waterfall_terminal",
        core::NumericIdentity::Probe { index } => {
            owned.kind = "probe";
            owned.index = Some(*index);
        }
    }
    owned
}

fn factors(value: &core::NumericFactors<'_>) -> OwnedFactors {
    OwnedFactors {
        multiplier: value.multiplier(),
        decimal_scale: value.decimal_scale(),
        rate: match value.rate() {
            core::NumericRate::Value => "value",
            core::NumericRate::Percent => "percent",
            core::NumericRate::PerMille => "per_mille",
        },
        divisor: value.divisor().clone(),
        prefix: value.prefix().to_owned(),
        suffix: value.suffix().to_owned(),
        unit: value.unit().cloned(),
    }
}

fn invalid_selection() -> core::Error {
    core::Error::Render("callback context selection does not match its active native view".into())
}

pub(super) fn read_value(
    context: &core::NumericValueContext<'_>,
    request: Request,
) -> core::Result<OwnedReply> {
    match request {
        Request::Index => Ok(OwnedReply::Index(context.index())),
        Request::ValueBuiltin => context.format_builtin().map(OwnedReply::Text),
        Request::Factors(FactorSource::Value) => {
            Ok(OwnedReply::Factors(Some(factors(&context.factors()))))
        }
        request => read_set(context.set(), request),
    }
}

pub(super) fn read_set(
    context: &core::NumericSetContext<'_>,
    request: Request,
) -> core::Result<OwnedReply> {
    match request {
        Request::SetLength => Ok(OwnedReply::Index(context.values().len())),
        Request::SetRole => Ok(OwnedReply::Name(match context.role() {
            core::NumericRole::XAxis => "x_axis",
            core::NumericRole::YAxis => "y_axis",
            core::NumericRole::ContinuousColor => "continuous_color",
            core::NumericRole::HeatmapValues => "heatmap_values",
            core::NumericRole::CoefficientLabels => "coefficient_labels",
            core::NumericRole::WaterfallTotals => "waterfall_totals",
        })),
        Request::SetPurpose => Ok(OwnedReply::Name(match context.purpose() {
            core::NumericPurpose::IntrinsicProbe => "intrinsic_probe",
            core::NumericPurpose::ActualSet => "actual_set",
        })),
        Request::SetBuiltin => context.format_builtin().map(OwnedReply::Labels),
        Request::Owner(OwnerSource::Numeric) => Ok(OwnedReply::Owner(owner(&context.owner()))),
        Request::Domain(DomainSource::Numeric) => {
            Ok(OwnedReply::Domain(context.domain().as_ref().map(domain)))
        }
        Request::Operand(OperandSource::Value(index)) => context
            .values()
            .get(index)
            .map(|value| OwnedReply::Operand(operand(value.operand())))
            .ok_or_else(invalid_selection),
        Request::Operand(OperandSource::Lower(DomainSource::Numeric)) => context
            .domain()
            .map(|value| OwnedReply::Operand(operand(value.lower())))
            .ok_or_else(invalid_selection),
        Request::Operand(OperandSource::Upper(DomainSource::Numeric)) => context
            .domain()
            .map(|value| OwnedReply::Operand(operand(value.upper())))
            .ok_or_else(invalid_selection),
        Request::Identity(index) => context
            .values()
            .get(index)
            .map(|value| OwnedReply::Identity(identity(&value.identity())))
            .ok_or_else(invalid_selection),
        Request::Factors(FactorSource::Common) => Ok(OwnedReply::Factors(
            context.common_factors().as_ref().map(factors),
        )),
        Request::Locale => Ok(OwnedReply::Locale(context.locale().clone())),
        _ => Err(invalid_selection()),
    }
}

pub(super) fn read_tick(
    context: &core::TickContext<'_>,
    request: Request,
) -> core::Result<OwnedReply> {
    match request {
        Request::TickAxis => Ok(OwnedReply::Name(axis(context.axis()))),
        Request::TickSpan => Ok(OwnedReply::Number(context.span())),
        Request::Owner(OwnerSource::Tick) => Ok(OwnedReply::Owner(owner(&context.owner()))),
        Request::Domain(DomainSource::Tick) => {
            Ok(OwnedReply::Domain(Some(domain(&context.domain()))))
        }
        Request::Operand(OperandSource::Lower(DomainSource::Tick)) => {
            Ok(OwnedReply::Operand(operand(context.domain().lower())))
        }
        Request::Operand(OperandSource::Upper(DomainSource::Tick)) => {
            Ok(OwnedReply::Operand(operand(context.domain().upper())))
        }
        _ => Err(invalid_selection()),
    }
}

fn invalid_reply() -> PyErr {
    PyRuntimeError::new_err("native callback context returned an incompatible owned reply")
}

macro_rules! reply {
    ($key:expr, $py:expr, $request:expr, $variant:ident) => {{
        let OwnedReply::$variant(value) = $key.read($py, $request)? else {
            return Err(invalid_reply());
        };
        value
    }};
}

#[pyclass(name = "NumericValueContext", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericValueContext {
    key: ContextKey,
}

impl PyNumericValueContext {
    pub(super) fn new(key: ContextKey) -> Self {
        Self { key }
    }
}

#[pymethods]
impl PyNumericValueContext {
    #[getter]
    fn set(&self, py: Python<'_>) -> PyResult<PyNumericSetContext> {
        self.key.check(py)?;
        Ok(PyNumericSetContext::new(self.key.clone()))
    }

    #[getter]
    fn index(&self, py: Python<'_>) -> PyResult<usize> {
        Ok(reply!(self.key, py, Request::Index, Index))
    }

    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<PyNumericValue> {
        let index = reply!(self.key, py, Request::Index, Index);
        Ok(PyNumericValue {
            key: self.key.clone(),
            index,
        })
    }

    #[getter]
    fn factors(&self, py: Python<'_>) -> PyResult<PyNumericFactors> {
        self.key.check(py)?;
        Ok(PyNumericFactors {
            key: self.key.clone(),
            source: FactorSource::Value,
        })
    }

    fn format_builtin(&self, py: Python<'_>) -> PyResult<String> {
        Ok(reply!(self.key, py, Request::ValueBuiltin, Text))
    }
}

#[pyclass(name = "NumericSetContext", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericSetContext {
    key: ContextKey,
}

impl PyNumericSetContext {
    pub(super) fn new(key: ContextKey) -> Self {
        Self { key }
    }
}

#[pymethods]
impl PyNumericSetContext {
    #[getter]
    fn owner(&self, py: Python<'_>) -> PyResult<PyNumericOwner> {
        self.key.check(py)?;
        Ok(PyNumericOwner {
            key: self.key.clone(),
            source: OwnerSource::Numeric,
        })
    }

    #[getter]
    fn role(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(reply!(self.key, py, Request::SetRole, Name))
    }

    #[getter]
    fn purpose(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(reply!(self.key, py, Request::SetPurpose, Name))
    }

    #[getter]
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let count = reply!(self.key, py, Request::SetLength, Index);
        PyTuple::new(
            py,
            (0..count).map(|index| PyNumericValue {
                key: self.key.clone(),
                index,
            }),
        )
    }

    #[getter]
    fn domain(&self, py: Python<'_>) -> PyResult<Option<PyNumericDomain>> {
        let domain = reply!(self.key, py, Request::Domain(DomainSource::Numeric), Domain);
        Ok(domain.map(|_| PyNumericDomain {
            key: self.key.clone(),
            source: DomainSource::Numeric,
        }))
    }

    #[getter]
    fn locale(&self, py: Python<'_>) -> PyResult<PyNumericLocaleView> {
        self.key.check(py)?;
        Ok(PyNumericLocaleView {
            key: self.key.clone(),
        })
    }

    #[getter]
    fn common_factors(&self, py: Python<'_>) -> PyResult<Option<PyNumericFactors>> {
        let factors = reply!(
            self.key,
            py,
            Request::Factors(FactorSource::Common),
            Factors
        );
        Ok(factors.map(|_| PyNumericFactors {
            key: self.key.clone(),
            source: FactorSource::Common,
        }))
    }

    fn format_builtin(&self, py: Python<'_>) -> PyResult<PyNumericLabels> {
        Ok(PyNumericLabels {
            native: reply!(self.key, py, Request::SetBuiltin, Labels),
        })
    }
}

#[pyclass(name = "TickContext", module = "lichtung_plot", frozen)]
pub(crate) struct PyTickContext {
    key: ContextKey,
}

impl PyTickContext {
    pub(super) fn new(key: ContextKey) -> Self {
        Self { key }
    }
}

#[pymethods]
impl PyTickContext {
    #[getter]
    fn axis(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(reply!(self.key, py, Request::TickAxis, Name))
    }

    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<f64> {
        Ok(reply!(self.key, py, Request::TickSpan, Number))
    }

    #[getter]
    fn owner(&self, py: Python<'_>) -> PyResult<PyNumericOwner> {
        self.key.check(py)?;
        Ok(PyNumericOwner {
            key: self.key.clone(),
            source: OwnerSource::Tick,
        })
    }

    #[getter]
    fn domain(&self, py: Python<'_>) -> PyResult<PyNumericDomain> {
        self.key.check(py)?;
        Ok(PyNumericDomain {
            key: self.key.clone(),
            source: DomainSource::Tick,
        })
    }
}

#[pyclass(name = "NumericOwner", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericOwner {
    key: ContextKey,
    source: OwnerSource,
}

impl PyNumericOwner {
    fn snapshot(&self, py: Python<'_>) -> PyResult<OwnedOwner> {
        Ok(reply!(self.key, py, Request::Owner(self.source), Owner))
    }
}

#[pymethods]
impl PyNumericOwner {
    #[getter]
    fn plot_kind(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        Ok(self.snapshot(py)?.plot_kind)
    }

    #[getter]
    fn canvas_cell(&self, py: Python<'_>) -> PyResult<Option<(usize, usize)>> {
        Ok(self.snapshot(py)?.canvas)
    }

    #[getter]
    fn is_shared(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.snapshot(py)?.shared)
    }

    #[getter]
    fn facet_cell(&self, py: Python<'_>) -> PyResult<Option<PyFacetCellView>> {
        Ok(self.snapshot(py)?.facet.map(|_| PyFacetCellView {
            key: self.key.clone(),
            source: self.source,
        }))
    }
}

#[pyclass(name = "_FacetCellView", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyFacetCellView {
    key: ContextKey,
    source: OwnerSource,
}

impl PyFacetCellView {
    fn snapshot(&self, py: Python<'_>) -> PyResult<core::FacetCell> {
        let owner = reply!(self.key, py, Request::Owner(self.source), Owner);
        owner.facet.ok_or_else(invalid_reply)
    }
}

#[pymethods]
impl PyFacetCellView {
    #[getter]
    fn single_level(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.snapshot(py)?.single_level().map(str::to_owned))
    }

    #[getter]
    fn grid_levels(&self, py: Python<'_>) -> PyResult<Option<(String, String)>> {
        Ok(self
            .snapshot(py)?
            .grid_levels()
            .map(|(row, column)| (row.to_owned(), column.to_owned())))
    }
}

#[pyclass(name = "NumericValue", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericValue {
    key: ContextKey,
    index: usize,
}

#[pymethods]
impl PyNumericValue {
    #[getter]
    fn operand(&self, py: Python<'_>) -> PyResult<PyNumericOperand> {
        self.key.check(py)?;
        Ok(PyNumericOperand {
            key: self.key.clone(),
            source: OperandSource::Value(self.index),
        })
    }

    #[getter]
    fn identity(&self, py: Python<'_>) -> PyResult<PyNumericIdentity> {
        self.key.check(py)?;
        Ok(PyNumericIdentity {
            key: self.key.clone(),
            index: self.index,
        })
    }
}

#[pyclass(name = "NumericOperand", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericOperand {
    key: ContextKey,
    source: OperandSource,
}

impl PyNumericOperand {
    fn snapshot(&self, py: Python<'_>) -> PyResult<OwnedOperand> {
        Ok(reply!(self.key, py, Request::Operand(self.source), Operand))
    }
}

#[pymethods]
impl PyNumericOperand {
    #[getter]
    fn kind(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::Binary64(_) => "binary64",
            OwnedOperand::PowerOfTen(_) => "power_of_ten",
            OwnedOperand::Multiplicative { .. } => "multiplicative",
        })
    }

    #[getter]
    fn binary64(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::Binary64(value) => Some(value),
            _ => None,
        })
    }

    #[getter]
    fn exponent(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::PowerOfTen(value) => Some(value),
            _ => None,
        })
    }

    #[getter]
    fn anchor(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::Multiplicative { anchor, .. } => Some(anchor),
            _ => None,
        })
    }

    #[getter]
    fn ratio(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::Multiplicative { ratio, .. } => Some(ratio),
            _ => None,
        })
    }

    #[getter]
    fn index(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(match self.snapshot(py)? {
            OwnedOperand::Multiplicative { index, .. } => Some(index),
            _ => None,
        })
    }
}

#[pyclass(name = "NumericIdentity", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericIdentity {
    key: ContextKey,
    index: usize,
}

impl PyNumericIdentity {
    fn snapshot(&self, py: Python<'_>) -> PyResult<OwnedIdentity> {
        Ok(reply!(
            self.key,
            py,
            Request::Identity(self.index),
            Identity
        ))
    }
}

#[pymethods]
impl PyNumericIdentity {
    #[getter]
    fn kind(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(self.snapshot(py)?.kind)
    }

    #[getter]
    fn axis(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        Ok(self.snapshot(py)?.axis)
    }

    #[getter]
    fn source_index(&self, py: Python<'_>) -> PyResult<Option<usize>> {
        Ok(self.snapshot(py)?.source_index)
    }

    #[getter]
    fn row(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.snapshot(py)?.row)
    }

    #[getter]
    fn column(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.snapshot(py)?.column)
    }

    #[getter]
    fn category(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.snapshot(py)?.category)
    }

    #[getter]
    fn index(&self, py: Python<'_>) -> PyResult<Option<usize>> {
        Ok(self.snapshot(py)?.index)
    }
}

#[pyclass(name = "NumericDomain", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericDomain {
    key: ContextKey,
    source: DomainSource,
}

impl PyNumericDomain {
    fn snapshot(&self, py: Python<'_>) -> PyResult<OwnedDomain> {
        let domain = reply!(self.key, py, Request::Domain(self.source), Domain);
        domain.ok_or_else(invalid_reply)
    }
}

#[pymethods]
impl PyNumericDomain {
    #[getter]
    fn lower(&self, py: Python<'_>) -> PyResult<PyNumericOperand> {
        // Reading this snapshot also verifies that the optional domain exists.
        drop(self.snapshot(py)?.lower);
        Ok(PyNumericOperand {
            key: self.key.clone(),
            source: OperandSource::Lower(self.source),
        })
    }

    #[getter]
    fn upper(&self, py: Python<'_>) -> PyResult<PyNumericOperand> {
        drop(self.snapshot(py)?.upper);
        Ok(PyNumericOperand {
            key: self.key.clone(),
            source: OperandSource::Upper(self.source),
        })
    }

    #[getter]
    fn data_limits(&self, py: Python<'_>) -> PyResult<Option<(f64, f64)>> {
        Ok(self.snapshot(py)?.data)
    }

    #[getter]
    fn scale_limits(&self, py: Python<'_>) -> PyResult<(f64, f64)> {
        Ok(self.snapshot(py)?.limits)
    }

    #[getter]
    fn scale(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(self.snapshot(py)?.scale)
    }

    #[getter]
    fn origin(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(self.snapshot(py)?.origin)
    }
}

#[pyclass(name = "NumericFactors", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericFactors {
    key: ContextKey,
    source: FactorSource,
}

impl PyNumericFactors {
    fn snapshot(&self, py: Python<'_>) -> PyResult<OwnedFactors> {
        let factors = reply!(self.key, py, Request::Factors(self.source), Factors);
        factors.ok_or_else(invalid_reply)
    }
}

#[pymethods]
impl PyNumericFactors {
    #[getter]
    fn multiplier(&self, py: Python<'_>) -> PyResult<f64> {
        Ok(self.snapshot(py)?.multiplier)
    }

    #[getter]
    fn decimal_scale(&self, py: Python<'_>) -> PyResult<i16> {
        Ok(self.snapshot(py)?.decimal_scale)
    }

    #[getter]
    fn rate(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(self.snapshot(py)?.rate)
    }

    #[getter]
    fn divisor(&self, py: Python<'_>) -> PyResult<PyNumericFactorView> {
        self.key.check(py)?;
        Ok(PyNumericFactorView {
            key: self.key.clone(),
            source: self.source,
        })
    }

    #[getter]
    fn prefix(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.prefix)
    }

    #[getter]
    fn suffix(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.suffix)
    }

    #[getter]
    fn unit(&self, py: Python<'_>) -> PyResult<Option<PyNumericUnitView>> {
        Ok(self.snapshot(py)?.unit.map(|_| PyNumericUnitView {
            key: self.key.clone(),
            source: self.source,
        }))
    }
}

#[pyclass(name = "_NumericFactorView", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyNumericFactorView {
    key: ContextKey,
    source: FactorSource,
}

impl PyNumericFactorView {
    fn snapshot(&self, py: Python<'_>) -> PyResult<core::NumericFactor> {
        let factors = reply!(self.key, py, Request::Factors(self.source), Factors);
        Ok(factors.ok_or_else(invalid_reply)?.divisor)
    }
}

#[pymethods]
impl PyNumericFactorView {
    #[getter]
    fn kind(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(match self.snapshot(py)? {
            core::NumericFactor::AutomaticDecimal => "automatic_decimal",
            core::NumericFactor::Value(_) => "value",
            core::NumericFactor::PowerOfTen(_) => "power_of_ten",
        })
    }

    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(match self.snapshot(py)? {
            core::NumericFactor::Value(value) => Some(value),
            _ => None,
        })
    }

    #[getter]
    fn exponent(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(match self.snapshot(py)? {
            core::NumericFactor::PowerOfTen(value) => Some(value),
            _ => None,
        })
    }
}

#[pyclass(name = "_NumericUnitView", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyNumericUnitView {
    key: ContextKey,
    source: FactorSource,
}

impl PyNumericUnitView {
    fn snapshot(&self, py: Python<'_>) -> PyResult<core::NumericUnit> {
        let factors = reply!(self.key, py, Request::Factors(self.source), Factors);
        factors
            .ok_or_else(invalid_reply)?
            .unit
            .ok_or_else(invalid_reply)
    }
}

#[pymethods]
impl PyNumericUnitView {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.text().to_owned())
    }

    #[getter]
    fn separator(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.get_separator().to_owned())
    }

    #[getter]
    fn placement(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(match self.snapshot(py)?.placement() {
            core::NumericUnitPlacement::PerLabel => "per_label",
            core::NumericUnitPlacement::Shared => "shared",
        })
    }
}

#[pyclass(name = "_NumericLocaleView", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyNumericLocaleView {
    key: ContextKey,
}

impl PyNumericLocaleView {
    fn snapshot(&self, py: Python<'_>) -> PyResult<core::NumericLocale> {
        Ok(reply!(self.key, py, Request::Locale, Locale))
    }
}

#[pymethods]
impl PyNumericLocaleView {
    #[getter]
    fn decimal(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.decimal().to_owned())
    }

    #[getter]
    fn digits<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.snapshot(py)?.digits())
    }

    #[getter]
    fn grouping(&self, py: Python<'_>) -> PyResult<Option<PyNumericGroupingView>> {
        Ok(self
            .snapshot(py)?
            .grouping()
            .map(|_| PyNumericGroupingView {
                key: self.key.clone(),
            }))
    }

    #[getter]
    fn symbols(&self, py: Python<'_>) -> PyResult<PyNumericSymbolsView> {
        self.key.check(py)?;
        Ok(PyNumericSymbolsView {
            key: self.key.clone(),
        })
    }

    #[getter]
    fn currency(&self, py: Python<'_>) -> PyResult<PyNumericCurrencyView> {
        self.key.check(py)?;
        Ok(PyNumericCurrencyView {
            key: self.key.clone(),
        })
    }
}

#[pyclass(
    name = "_NumericGroupingView",
    module = "lichtung_plot._native",
    frozen
)]
pub(crate) struct PyNumericGroupingView {
    key: ContextKey,
}

impl PyNumericGroupingView {
    fn snapshot(&self, py: Python<'_>) -> PyResult<core::NumericGrouping> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        locale.grouping().cloned().ok_or_else(invalid_reply)
    }
}

#[pymethods]
impl PyNumericGroupingView {
    #[getter]
    fn separator(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.snapshot(py)?.separator().to_owned())
    }

    #[getter]
    fn sizes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.snapshot(py)?.sizes())
    }

    #[getter]
    fn repeat(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(match self.snapshot(py)?.repeat() {
            core::NumericGroupRepeat::Cycle => "cycle",
            core::NumericGroupRepeat::Last => "last",
            core::NumericGroupRepeat::Stop => "stop",
        })
    }
}

#[pyclass(name = "_NumericSymbolsView", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyNumericSymbolsView {
    key: ContextKey,
}

#[pymethods]
impl PyNumericSymbolsView {
    #[getter]
    fn positive(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.symbols().positive().to_owned())
    }

    #[getter]
    fn negative(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.symbols().negative().to_owned())
    }

    #[getter]
    fn percent(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.symbols().percent().to_owned())
    }

    #[getter]
    fn per_mille(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.symbols().per_mille().to_owned())
    }
}

#[pyclass(
    name = "_NumericCurrencyView",
    module = "lichtung_plot._native",
    frozen
)]
pub(crate) struct PyNumericCurrencyView {
    key: ContextKey,
}

#[pymethods]
impl PyNumericCurrencyView {
    #[getter]
    fn prefix(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.currency().prefix().to_owned())
    }

    #[getter]
    fn suffix(&self, py: Python<'_>) -> PyResult<String> {
        let locale = reply!(self.key, py, Request::Locale, Locale);
        Ok(locale.currency().suffix().to_owned())
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyNumericValueContext>()?;
    module.add_class::<PyNumericSetContext>()?;
    module.add_class::<PyTickContext>()?;
    module.add_class::<PyNumericOwner>()?;
    module.add_class::<PyFacetCellView>()?;
    module.add_class::<PyNumericValue>()?;
    module.add_class::<PyNumericOperand>()?;
    module.add_class::<PyNumericIdentity>()?;
    module.add_class::<PyNumericDomain>()?;
    module.add_class::<PyNumericFactors>()?;
    module.add_class::<PyNumericFactorView>()?;
    module.add_class::<PyNumericUnitView>()?;
    module.add_class::<PyNumericLocaleView>()?;
    module.add_class::<PyNumericGroupingView>()?;
    module.add_class::<PyNumericSymbolsView>()?;
    module.add_class::<PyNumericCurrencyView>()
}
