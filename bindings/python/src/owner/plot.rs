//! Mutable Python Plot handles over one exact-family native configuration.

use std::path::PathBuf;
use std::sync::Arc;

use lichtung_plot::{self as core, kind};
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use pyo3::{PyTraverseError, PyVisit};

use super::{native_snapshot, policy, CallbackSlot, NativePlot, PlotRoots};
use crate::callbacks::{self, CallbackRoots};
use crate::input;
use crate::output::{PyOutput, Terminal};
use crate::values::numeric::{PyNumericFormat, PyTickGeneration};
use crate::values::{
    self, PyCategoricalColorScale, PyColor, PyContinuousColorScale, PyInteractionStyle, PyLabels,
    PyPngOptions, PySignificanceMarkStyle, PyTextLayout, PyTextPosition, PyTheme, TextIntent,
};

// Typed input preparation has a separate boundary from pure native edits. Each
// arm calls D's exact producer and wraps its matching Config<K>/roots together.
macro_rules! prepare_all {
    ([$($kind:ident),+] $native:expr, $config:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind($config) =>
                $body.map(|prepared| PreparedPlot::new(prepared, NativePlot::$kind))),+
        }
    };
}

macro_rules! prepare_kinds {
    ($native:expr, [$($kind:ident),+], $config:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind($config) =>
                $body.map(|prepared| PreparedPlot::new(prepared, NativePlot::$kind))),+,
            _ => Err(PyTypeError::new_err(
                "this exact plot family does not support that input operation",
            )),
        }
    };
}

macro_rules! prepare_capability {
    (All, $native:expr, $config:ident => $body:expr) => {
        all_families!(prepare_all, $native, $config => $body)
    };
    (Hue, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [Scatter, Density, Line, Trajectory, LinePath,
            Interaction, Marginal, Matrix], $config => $body)
    };
    (Group, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [Density, Line, Marginal, Matrix, Dumbbell], $config => $body)
    };
    (Series, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [Bar, HorizontalBar, Boxplot, Violin, Raincloud], $config => $body)
    };
    (Facet, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [Scatter, Histogram, Density, Qq, PointInterval,
            Line, Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell,
            Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot,
            Violin, Raincloud, Regression, Beeswarm, Waterfall], $config => $body)
    };
    (EstimateStatus, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [PointInterval, Coefficient, Forest], $config => $body)
    };
    ($kind:ident, $native:expr, $config:ident => $body:expr) => {
        prepare_kinds!($native, [$kind], $config => $body)
    };
}

// Compile-time method plumbing only. Signatures and exact Args fields remain
// explicit; there is no runtime keyword bag, role inference or builder protocol.
macro_rules! owner_api {
    (
        constructors {
            $(
                $(#[$attribute:meta])*
                fn $constructor:ident($($argument:ident: $argument_type:ty),* $(,)?)
                    => $kind:ident, $args:ident { $($field:ident),* $(,)? };
            )*
        }
        inputs ($slf:ident, $native:ident, $config:ident, $roots:ident) {
            $(
                fn $method:ident($($parameter:ident: $parameter_type:ty),* $(,)?)
                    : $capability:ident => $producer:expr;
            )*
        }
        api { $($api:tt)* }
        manual { $($manual:item)* }
    ) => {
        plot_api! {
            $($api)*
            manual {
                $(
                    #[staticmethod]
                    $(#[$attribute])*
                    fn $constructor(
                        data: Option<&Bound<'_, PyAny>>,
                        $($argument: $argument_type),*
                    ) -> PyResult<Self> {
                        let prepared = input::$constructor(data, input::$args { $($field),* })?;
                        Ok(Self::from_prepared(PreparedPlot::new(prepared, NativePlot::$kind)))
                    }
                )*
                $(
                    fn $method(
                        $slf: &Bound<'_, Self>,
                        $($parameter: $parameter_type),*
                    ) -> PyResult<()> {
                        Self::prepare_input($slf, |$native, $roots| {
                            prepare_capability!($capability, $native, $config => $producer)
                        })
                    }
                )*
                $($manual)*
            }
        }
    };
}

fn geometry_label(
    value: &Bound<'_, PyAny>,
    unset: &Bound<'_, PyAny>,
) -> PyResult<Option<TextIntent>> {
    if value.is(unset) {
        Ok(None)
    } else {
        // In particular None is not absence or a clear command at construction.
        values::text_intent(value, unset, unset, unset).map(Some)
    }
}

macro_rules! apply_label {
    ($plot:expr, $intent:expr, $setter:ident, $style:ident, $layout:ident) => {{
        let mut plot = $plot;
        if let Some(intent) = $intent {
            plot = plot.$setter(intent.native);
            if intent.clear_style {
                plot = plot.$style();
            }
            if intent.clear_layout {
                plot = plot.$layout();
            }
        }
        plot
    }};
}

#[pyclass(name = "_PlotOwner", module = "lichtung_plot._native")]
pub(crate) struct PyPlotOwner {
    // None has exactly one meaning: GC has cleared this owner.
    native: Option<Arc<NativePlot>>,
    roots: PlotRoots,
    inputs: input::InputRoots,
}

/// A prepared input transaction, never retained as a second configuration.
struct PreparedPlot {
    native: NativePlot,
    roots: input::InputRoots,
}

impl PreparedPlot {
    fn new<K: kind::Marker>(
        prepared: input::Prepared<K>,
        wrap: fn(input::Config<K>) -> NativePlot,
    ) -> Self {
        Self {
            native: wrap(prepared.config),
            roots: prepared.roots,
        }
    }
}

impl PyPlotOwner {
    fn from_prepared(prepared: PreparedPlot) -> Self {
        Self {
            native: Some(Arc::new(prepared.native)),
            roots: PlotRoots::default(),
            inputs: prepared.roots,
        }
    }

    pub(super) fn snapshot(&self, py: Python<'_>) -> PyResult<(Arc<NativePlot>, CallbackRoots)> {
        Ok((native_snapshot(self.native.as_ref())?, self.roots.pins(py)?))
    }

    fn input_snapshot(slf: &Bound<'_, Self>) -> PyResult<(Arc<NativePlot>, input::InputRoots)> {
        let owner = slf.try_borrow()?;
        let native = native_snapshot(owner.native.as_ref())?;
        let roots = owner.inputs.clone_ref(slf.py());
        Ok((native, roots))
    }

    fn prepare_input(
        slf: &Bound<'_, Self>,
        prepare: impl FnOnce(&NativePlot, &input::InputRoots) -> PyResult<PreparedPlot>,
    ) -> PyResult<()> {
        let (expected, roots) = Self::input_snapshot(slf)?;
        let prepared = prepare(expected.as_ref(), &roots)?;
        Self::commit_input(slf, &expected, prepared)
    }

    fn commit_input(
        slf: &Bound<'_, Self>,
        expected: &Arc<NativePlot>,
        prepared: PreparedPlot,
    ) -> PyResult<()> {
        let incoming = Arc::new(prepared.native);
        let mut inputs = Some(prepared.roots);
        let mut retired = None;
        let result = (|| {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            if !Arc::ptr_eq(&current, expected) {
                return Err(PyRuntimeError::new_err(
                    "plot changed during input preparation; the stale binding was not committed",
                ));
            }
            retired = Some((
                owner.native.replace(incoming),
                std::mem::replace(
                    &mut owner.inputs,
                    inputs.take().expect("prepared inputs are committed once"),
                ),
            ));
            Ok(())
        })();
        // Both rejected and retired Python roots can run reentrant finalizers.
        drop(retired);
        drop(inputs);
        result
    }

    fn update(
        slf: &Bound<'_, Self>,
        edit: impl FnOnce(NativePlot) -> PyResult<NativePlot>,
    ) -> PyResult<()> {
        let expected = {
            let owner = slf.try_borrow()?;
            native_snapshot(owner.native.as_ref())?
        };
        let incoming = Arc::new(edit(expected.as_ref().clone())?);
        let mut retired = None;
        let result = (|| {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            if !Arc::ptr_eq(&current, &expected) {
                return Err(PyRuntimeError::new_err(
                    "plot changed during configuration preparation; the stale edit was not committed",
                ));
            }
            retired = owner.native.replace(incoming);
            Ok(())
        })();
        drop(retired);
        result
    }

    fn update_callback(
        slf: &Bound<'_, Self>,
        slot: CallbackSlot,
        roots: CallbackRoots,
        edit: impl FnOnce(NativePlot) -> PyResult<NativePlot>,
    ) -> PyResult<()> {
        let expected = {
            let owner = slf.try_borrow()?;
            native_snapshot(owner.native.as_ref())?
        };
        let incoming = Arc::new(edit(expected.as_ref().clone())?);
        let mut incoming_roots = Some(roots);
        let mut retired = None;
        let result = (|| {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            if !Arc::ptr_eq(&current, &expected) {
                return Err(PyRuntimeError::new_err(
                    "plot changed during configuration preparation; the stale edit was not committed",
                ));
            }
            let native = owner.native.replace(incoming);
            let roots = owner.roots.replace(
                slot,
                incoming_roots
                    .take()
                    .expect("prepared callback roots are committed once"),
            );
            retired = Some((native, roots));
            Ok(())
        })();
        // A released callable may run a finalizer which reenters this owner.
        drop(retired);
        drop(incoming_roots);
        result
    }

    fn publish(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        terminal: Terminal,
    ) -> PyResult<PyOutput> {
        let path = path.extract::<PathBuf>()?;
        let (native, pins) = {
            let owner = slf.try_borrow()?;
            owner.snapshot(slf.py())?
        };
        callbacks::with_operation(slf.py(), pins, move || {
            all_families!(call_configs, native.as_ref(), config =>
                terminal.plot(&config.native, &path))
        })
        .map(PyOutput::new)
    }
}

owner_api! {
    constructors {
        #[pyo3(signature = (data=None, *, value, hue=None, group=None))]
        fn density(
            value: &Bound<'_, PyAny>, hue: Option<&Bound<'_, PyAny>>,
            group: Option<&Bound<'_, PyAny>>,
        ) => Density, DensityArgs { value, hue, group };
        #[pyo3(signature = (data=None, *, value))]
        fn qq(value: &Bound<'_, PyAny>) => Qq, QqArgs { value };
        #[pyo3(signature = (data=None, *, category, estimate, lower, upper))]
        fn point_interval(
            category: &Bound<'_, PyAny>, estimate: &Bound<'_, PyAny>,
            lower: &Bound<'_, PyAny>, upper: &Bound<'_, PyAny>,
        ) => PointInterval, PointIntervalArgs { category, estimate, lower, upper };
        #[pyo3(signature = (data=None, *, x, y, hue=None, group=None,
            by_row_position=false, numeric_only=false, unset))]
        #[allow(clippy::too_many_arguments, reason = "complete Line roles and two explicit modes")]
        fn line(
            x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>,
            hue: Option<&Bound<'_, PyAny>>, group: Option<&Bound<'_, PyAny>>,
            by_row_position: bool, numeric_only: bool, unset: &Bound<'_, PyAny>,
        ) => Line, LineArgs { x, y, hue, group, by_row_position, numeric_only, unset };
        #[pyo3(signature = (data=None, *, x, y, group, hue=None))]
        fn trajectory(
            x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>, group: &Bound<'_, PyAny>,
            hue: Option<&Bound<'_, PyAny>>,
        ) => Trajectory, TrajectoryArgs { x, y, group, hue };
        #[pyo3(signature = (data=None, *, x, y, group, hue=None))]
        fn line_path(
            x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>, group: &Bound<'_, PyAny>,
            hue: Option<&Bound<'_, PyAny>>,
        ) => LinePath, LinePathArgs { x, y, group, hue };
        #[pyo3(signature = (data=None, *, condition, estimate, group, lower, upper, hue=None))]
        fn interaction(
            condition: &Bound<'_, PyAny>, estimate: &Bound<'_, PyAny>,
            group: &Bound<'_, PyAny>, lower: &Bound<'_, PyAny>,
            upper: &Bound<'_, PyAny>, hue: Option<&Bound<'_, PyAny>>,
        ) => Interaction, InteractionArgs { condition, estimate, group, lower, upper, hue };
        #[pyo3(signature = (data=None, *, term, estimate, lower, upper, unset))]
        fn coefficient(
            term: &Bound<'_, PyAny>, estimate: &Bound<'_, PyAny>,
            lower: &Bound<'_, PyAny>, upper: &Bound<'_, PyAny>, unset: &Bound<'_, PyAny>,
        ) => Coefficient, CoefficientArgs { term, estimate, lower, upper, unset };
        #[pyo3(signature = (data=None, *, term, estimate, lower, upper, reference))]
        fn forest(
            term: &Bound<'_, PyAny>, estimate: &Bound<'_, PyAny>,
            lower: &Bound<'_, PyAny>, upper: &Bound<'_, PyAny>, reference: &Bound<'_, PyAny>,
        ) => Forest, ForestArgs { term, estimate, lower, upper, reference };
        #[pyo3(signature = (data=None, *, grid, x, y, fill, unset))]
        fn heatmap(
            grid: &Bound<'_, PyAny>, x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>,
            fill: &Bound<'_, PyAny>, unset: &Bound<'_, PyAny>,
        ) => Heatmap, HeatmapArgs { grid, x, y, fill, unset };
        #[pyo3(signature = (data=None, *, category, samples, estimate, lower, upper,
            error_bar, series=None, unset))]
        #[allow(clippy::too_many_arguments, reason = "complete sample/estimate Bar alternatives")]
        fn bar(
            category: &Bound<'_, PyAny>, samples: &Bound<'_, PyAny>,
            estimate: &Bound<'_, PyAny>, lower: &Bound<'_, PyAny>,
            upper: &Bound<'_, PyAny>, error_bar: &Bound<'_, PyAny>,
            series: Option<&Bound<'_, PyAny>>, unset: &Bound<'_, PyAny>,
        ) => Bar, BarArgs { category, samples, estimate, lower, upper, error_bar, series, unset };
        #[pyo3(signature = (data=None, *, category, samples, estimate, lower, upper,
            error_bar, series=None, unset))]
        #[allow(clippy::too_many_arguments, reason = "complete sample/estimate HorizontalBar alternatives")]
        fn horizontal_bar(
            category: &Bound<'_, PyAny>, samples: &Bound<'_, PyAny>,
            estimate: &Bound<'_, PyAny>, lower: &Bound<'_, PyAny>,
            upper: &Bound<'_, PyAny>, error_bar: &Bound<'_, PyAny>,
            series: Option<&Bound<'_, PyAny>>, unset: &Bound<'_, PyAny>,
        ) => HorizontalBar, HorizontalBarArgs {
            category, samples, estimate, lower, upper, error_bar, series, unset
        };
        #[pyo3(signature = (data=None, *, category, value, series))]
        fn stacked_bar(
            category: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, series: &Bound<'_, PyAny>,
        ) => StackedBar, StackedBarArgs { category, value, series };
        #[pyo3(signature = (data=None, *, category, value, series))]
        fn stacked_horizontal_bar(
            category: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, series: &Bound<'_, PyAny>,
        ) => StackedHorizontalBar, StackedHorizontalBarArgs { category, value, series };
        #[pyo3(signature = (data=None, *, category, value, series=None,
            by_column_name=false, numeric_only=false, unset))]
        fn boxplot(
            category: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>,
            series: Option<&Bound<'_, PyAny>>, by_column_name: bool,
            numeric_only: bool, unset: &Bound<'_, PyAny>,
        ) => Boxplot, BoxplotArgs { category, value, series, by_column_name, numeric_only, unset };
        #[pyo3(signature = (data=None, *, category, value, series=None,
            by_column_name=false, numeric_only=false, unset))]
        fn violin(
            category: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>,
            series: Option<&Bound<'_, PyAny>>, by_column_name: bool,
            numeric_only: bool, unset: &Bound<'_, PyAny>,
        ) => Violin, ViolinArgs { category, value, series, by_column_name, numeric_only, unset };
        #[pyo3(signature = (data=None, *, x, y))]
        fn regression(
            x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>,
        ) => Regression, RegressionArgs { x, y };
        #[pyo3(signature = (data=None, *, x, y, hue=None, group=None))]
        fn marginal(
            x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>,
            hue: Option<&Bound<'_, PyAny>>, group: Option<&Bound<'_, PyAny>>,
        ) => Marginal, MarginalArgs { x, y, hue, group };
        #[pyo3(signature = (data=None, *, variables, hue=None, group=None, numeric_only=false, unset))]
        fn matrix(
            variables: &Bound<'_, PyAny>, hue: Option<&Bound<'_, PyAny>>,
            group: Option<&Bound<'_, PyAny>>, numeric_only: bool, unset: &Bound<'_, PyAny>,
        ) => Matrix, MatrixArgs { variables, hue, group, numeric_only, unset };
        #[pyo3(signature = (data=None, *, category, value, color))]
        fn beeswarm(
            category: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, color: &Bound<'_, PyAny>,
        ) => Beeswarm, BeeswarmArgs { category, value, color };
        #[pyo3(signature = (data=None, *, category, delta, base))]
        fn waterfall(
            category: &Bound<'_, PyAny>, delta: &Bound<'_, PyAny>, base: &Bound<'_, PyAny>,
        ) => Waterfall, WaterfallArgs { category, delta, base };
    }
    inputs (slf, native, config, roots) {
        fn hue_cells(values: &Bound<'_, PyAny>): Hue => input::hue_cells(config, roots, values);
        fn hue_columns(): Hue => input::hue_columns(config, roots, slf.py());
        fn group(values: &Bound<'_, PyAny>): Group => input::group(config, roots, values);
        fn group_cells(values: &Bound<'_, PyAny>): Group => input::group_cells(config, roots, values);
        fn group_columns(): Group => input::group_columns(config, roots, slf.py());
        fn series(values: &Bound<'_, PyAny>): Series => input::series(config, roots, values);
        fn series_cells(values: &Bound<'_, PyAny>): Series => input::series_cells(config, roots, values);
        fn series_columns(): Series => input::series_columns(config, roots, slf.py());
        fn source_rows(values: &Bound<'_, PyAny>): All => input::source_rows(config, roots, values);
        fn source_rows_cells(values: &Bound<'_, PyAny>): All => input::source_rows_cells(config, roots, values);
        fn column_labels(labels: &Bound<'_, PyAny>): All => input::column_labels(config, roots, labels);
        fn estimate_status(values: &Bound<'_, PyAny>): EstimateStatus => input::estimate_status(config, roots, values);
        fn estimate_status_boolean(values: &Bound<'_, PyAny>): EstimateStatus => input::estimate_status_boolean(config, roots, values);
        fn estimate_status_categorical(values: &Bound<'_, PyAny>, mapping: &Bound<'_, PyAny>): EstimateStatus =>
            input::estimate_status_categorical(config, roots, values, mapping);
        fn x_labels(labels: &Bound<'_, PyAny>): Heatmap => input::x_labels(config, roots, labels);
        fn y_labels(labels: &Bound<'_, PyAny>): Heatmap => input::y_labels(config, roots, labels);
        fn row_labels(values: &Bound<'_, PyAny>): Heatmap => input::row_labels(config, roots, values);
        fn heatmap_highlight(values: &Bound<'_, PyAny>): Heatmap => input::heatmap_highlight(config, roots, values);
        fn heatmap_highlight_cells(values: &Bound<'_, PyAny>): Heatmap => input::heatmap_highlight_cells(config, roots, values);
        fn line_band(lower: &Bound<'_, PyAny>, upper: &Bound<'_, PyAny>): Line => input::line_band(config, roots, lower, upper);
        fn facet_row(values: &Bound<'_, PyAny>): Facet => input::facet_row(config, roots, values);
        fn facet_column(values: &Bound<'_, PyAny>): Facet => input::facet_column(config, roots, values);
        fn facet_row_cells(values: &Bound<'_, PyAny>): Facet => input::facet_row_cells(config, roots, values);
        fn facet_column_cells(values: &Bound<'_, PyAny>): Facet => input::facet_column_cells(config, roots, values);
        fn facet_wrap(values: &Bound<'_, PyAny>, columns: usize): Facet => input::facet_wrap(config, roots, values, columns);
        fn facet_wrap_cells(values: &Bound<'_, PyAny>, columns: usize): Facet => input::facet_wrap_cells(config, roots, values, columns);
        fn facet_grid(row: &Bound<'_, PyAny>, column: &Bound<'_, PyAny>): Facet => input::facet_grid(config, roots, row, column);
        fn facet_grid_row_cells(row: &Bound<'_, PyAny>, column: &Bound<'_, PyAny>): Facet => input::facet_grid_row_cells(config, roots, row, column);
        fn facet_grid_column_cells(row: &Bound<'_, PyAny>, column: &Bound<'_, PyAny>): Facet => input::facet_grid_column_cells(config, roots, row, column);
        fn facet_grid_cells(row: &Bound<'_, PyAny>, column: &Bound<'_, PyAny>): Facet => input::facet_grid_cells(config, roots, row, column);
        fn facet_columns_row(): Facet => input::facet_columns_row(config, roots, slf.py());
        fn facet_columns_column(): Facet => input::facet_columns_column(config, roots, slf.py());
        fn facet_columns_wrap(columns: usize): Facet => input::facet_columns_wrap(config, roots, columns, slf.py());
        fn facet_grid_columns_row(column: &Bound<'_, PyAny>): Facet => input::facet_grid_columns_row(config, roots, column);
        fn facet_grid_columns_column(row: &Bound<'_, PyAny>): Facet => input::facet_grid_columns_column(config, roots, row);
        fn facet_grid_columns_row_cells(column: &Bound<'_, PyAny>): Facet => input::facet_grid_columns_row_cells(config, roots, column);
        fn facet_grid_columns_column_cells(row: &Bound<'_, PyAny>): Facet => input::facet_grid_columns_column_cells(config, roots, row);
    }
    api {
    text {
        title: All => clear_title_style, clear_title_layout;
        subtitle: All => clear_subtitle_style, clear_subtitle_layout;
        caption: All => clear_caption_style, clear_caption_layout;
        x_label: All => clear_x_label_style, clear_x_label_layout;
        y_label: All => clear_y_label_style, clear_y_label_layout;
        color_label: ContinuousColor => clear_color_label_style, clear_color_label_layout;
        hue_label: Raincloud => clear_hue_label_style, clear_hue_label_layout;
    }
    clear {
        clear_title_style: All;
        clear_title_layout: All;
        clear_subtitle_style: All;
        clear_subtitle_layout: All;
        clear_caption_style: All;
        clear_caption_layout: All;
        clear_x_label_style: All;
        clear_x_label_layout: All;
        clear_y_label_style: All;
        clear_y_label_layout: All;
        clear_color_label_style: ContinuousColor;
        clear_color_label_layout: ContinuousColor;
        clear_hue_label_style: Raincloud;
        clear_hue_label_layout: Raincloud;
        color_by_sign: HorizontalBar;
        zero_line: ZeroLine;
        hide_point_outline: PointOutline;
    }
    layout {
        x_tick_text_layout: All;
        y_tick_text_layout: All;
        facet_row_text_layout: Facet;
        facet_column_text_layout: Facet;
        estimate_detail_text_layout: EstimateDetailText;
        significance_text_layout: SignificanceStyle;
        matrix_row_text_layout: Matrix;
        matrix_column_text_layout: Matrix;
        value_text_layout: Heatmap;
        null_row_text_layout: Heatmap;
        total_text_layout: Waterfall;
    }
    format {
        x_tick_format: NumericX => XFormat;
        y_tick_format: NumericY => YFormat;
        color_label_format: ContinuousColor => ColorFormat;
        estimate_labels: Coefficient => EstimateLabels;
        total_format: Waterfall => TotalFormat;
    }
    generation {
        x_tick_generation: NumericX => XTick;
        y_tick_generation: NumericY => YTick;
    }
    flag {
        markers: Markers;
        connectors: Interaction;
    }
    }
    manual {
    #[staticmethod]
    #[pyo3(signature = (data=None, *, x, y, hue=None, by_row_position=false, numeric_only=false, unset))]
    fn scatter(
        data: Option<&Bound<'_, PyAny>>,
        x: &Bound<'_, PyAny>,
        y: &Bound<'_, PyAny>,
        hue: Option<&Bound<'_, PyAny>>,
        by_row_position: bool,
        numeric_only: bool,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let prepared = input::scatter(data, x, y, hue, by_row_position, numeric_only, unset)?;
        Ok(Self::from_prepared(PreparedPlot::new(prepared, NativePlot::Scatter)))
    }

    #[staticmethod]
    #[pyo3(signature = (data=None, *, category, left, right, left_label, left_color,
        right_label, right_color, group=None))]
    #[allow(clippy::too_many_arguments, reason = "complete endpoints are independent literal labels and RGBA paints")]
    fn dumbbell(
        data: Option<&Bound<'_, PyAny>>,
        category: &Bound<'_, PyAny>,
        left: &Bound<'_, PyAny>,
        right: &Bound<'_, PyAny>,
        left_label: String,
        left_color: PyRef<'_, PyColor>,
        right_label: String,
        right_color: PyRef<'_, PyColor>,
        group: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let native_left_color = left_color.native;
        let native_right_color = right_color.native;
        drop(left_color);
        drop(right_color);
        let endpoints = core::DumbbellEndpoints::new(
            left_label, native_left_color, right_label, native_right_color,
        ).map_err(crate::native_error)?;
        let prepared = input::dumbbell(data, input::DumbbellArgs {
            category, left, right, endpoints, group,
        })?;
        Ok(Self::from_prepared(PreparedPlot::new(prepared, NativePlot::Dumbbell)))
    }

    #[staticmethod]
    #[pyo3(signature = (data=None, *, samples, bin_lower, bin_upper, bin_density,
        kde_x, kde_y, rug_x, rug_baseline, rug_top, kde_data, rug_data,
        value_label, density_label, unset))]
    #[allow(clippy::too_many_arguments, reason = "flat complete geometry keeps independent component contexts and real Text labels")]
    fn histogram(
        data: Option<&Bound<'_, PyAny>>,
        samples: &Bound<'_, PyAny>,
        bin_lower: &Bound<'_, PyAny>,
        bin_upper: &Bound<'_, PyAny>,
        bin_density: &Bound<'_, PyAny>,
        kde_x: &Bound<'_, PyAny>,
        kde_y: &Bound<'_, PyAny>,
        rug_x: &Bound<'_, PyAny>,
        rug_baseline: &Bound<'_, PyAny>,
        rug_top: &Bound<'_, PyAny>,
        kde_data: Option<&Bound<'_, PyAny>>,
        rug_data: Option<&Bound<'_, PyAny>>,
        value_label: &Bound<'_, PyAny>,
        density_label: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        // Snapshot/validate authored content before D can touch any source.
        let value_label = geometry_label(value_label, unset)?;
        let density_label = geometry_label(density_label, unset)?;
        if !samples.is(unset) && (value_label.is_some() || density_label.is_some()) {
            return Err(PyTypeError::new_err(
                "constructor display labels belong to complete histogram geometry",
            ));
        }
        let mut prepared = input::histogram(data, input::HistogramArgs {
            samples, bin_lower, bin_upper, bin_density, kde_data, kde_x, kde_y,
            rug_data, rug_x, rug_baseline, rug_top, unset,
        })?;
        prepared.config = prepared.config.map_native(|plot| {
            let plot = apply_label!(plot, value_label, x_label, clear_x_label_style, clear_x_label_layout);
            apply_label!(plot, density_label, y_label, clear_y_label_style, clear_y_label_layout)
        });
        Ok(Self::from_prepared(PreparedPlot::new(prepared, NativePlot::Histogram)))
    }

    #[staticmethod]
    #[pyo3(signature = (data=None, *, category, value, series=None, by_column_name=false,
        numeric_only=false, q1, median, q3, density_category, density_y, normalized_width,
        point_category, point_y, normalized_strip, hue=None, density_hue=None, point_hue=None,
        density_data, point_data, category_label, value_label, hue_label, unset))]
    #[allow(clippy::too_many_arguments, reason = "complete flat sample/geometry forms retain three independent alignment domains")]
    fn raincloud(
        data: Option<&Bound<'_, PyAny>>,
        category: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        series: Option<&Bound<'_, PyAny>>,
        by_column_name: bool,
        numeric_only: bool,
        q1: &Bound<'_, PyAny>,
        median: &Bound<'_, PyAny>,
        q3: &Bound<'_, PyAny>,
        density_category: &Bound<'_, PyAny>,
        density_y: &Bound<'_, PyAny>,
        normalized_width: &Bound<'_, PyAny>,
        point_category: &Bound<'_, PyAny>,
        point_y: &Bound<'_, PyAny>,
        normalized_strip: &Bound<'_, PyAny>,
        hue: Option<&Bound<'_, PyAny>>,
        density_hue: Option<&Bound<'_, PyAny>>,
        point_hue: Option<&Bound<'_, PyAny>>,
        density_data: Option<&Bound<'_, PyAny>>,
        point_data: Option<&Bound<'_, PyAny>>,
        category_label: &Bound<'_, PyAny>,
        value_label: &Bound<'_, PyAny>,
        hue_label: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let category_label = geometry_label(category_label, unset)?;
        let value_label = geometry_label(value_label, unset)?;
        let hue_label = geometry_label(hue_label, unset)?;
        if (!value.is(unset) || by_column_name || numeric_only)
            && (category_label.is_some() || value_label.is_some() || hue_label.is_some())
        {
            return Err(PyTypeError::new_err(
                "constructor display labels belong to complete raincloud geometry",
            ));
        }
        let mut prepared = input::raincloud(data, input::RaincloudArgs {
            category, value, series, by_column_name, numeric_only, q1, median, q3,
            density_data, density_category, density_y, normalized_width,
            point_data, point_category, point_y, normalized_strip,
            hue, density_hue, point_hue, unset,
        })?;
        prepared.config = prepared.config.map_native(|plot| {
            let plot = apply_label!(plot, category_label, x_label, clear_x_label_style, clear_x_label_layout);
            let plot = apply_label!(plot, value_label, y_label, clear_y_label_style, clear_y_label_layout);
            // Raincloud's authored guide title is its real native slot, not an
            // alias or a replacement for raw hue dimension metadata.
            apply_label!(plot, hue_label, hue_label, clear_hue_label_style, clear_hue_label_layout)
        });
        Ok(Self::from_prepared(PreparedPlot::new(prepared, NativePlot::Raincloud)))
    }

    fn copy(slf: &Bound<'_, Self>) -> PyResult<Self> {
        let mut roots = PlotRoots::default();
        let (native, inputs) = {
            let owner = slf.try_borrow()?;
            let native = native_snapshot(owner.native.as_ref())?;
            owner.roots.clone_into(slf.py(), &mut roots)?;
            (native, owner.inputs.clone_ref(slf.py()))
        };
        Ok(Self {
            native: Some(native),
            roots,
            inputs,
        })
    }

    fn theme(slf: &Bound<'_, Self>, theme: PyRef<'_, PyTheme>) -> PyResult<()> {
        let theme_native = theme.native.clone();
        drop(theme);
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.theme(theme_native))
        })
    }

    fn labels(slf: &Bound<'_, Self>, labels: PyRef<'_, PyLabels>) -> PyResult<()> {
        let labels_native = labels.native.clone();
        drop(labels);
        Self::update(slf, |native| {
            Ok(all_families!(map_configs, native, config => config.labels(labels_native)))
        })
    }

    fn width(slf: &Bound<'_, Self>, width: u32) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.width(width))
        })
    }

    fn max_width(slf: &Bound<'_, Self>, maximum: u32) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.max_width(maximum))
        })
    }

    fn size(slf: &Bound<'_, Self>, width: u32, height: u32) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.size(width, height))
        })
    }

    fn hue(slf: &Bound<'_, Self>, values: &Bound<'_, PyAny>) -> PyResult<()> {
        let legacy_native =
            if values.is_instance_of::<PyList>() || values.is_instance_of::<PyTuple>() {
                match slf.try_borrow() {
                    Ok(owner) => match owner.native.as_deref() {
                        Some(NativePlot::Scatter(config)) => config.is_core(),
                        Some(_) => false,
                        None => true,
                    },
                    // Keep routing diagnostics behind the existing native
                    // conversion; update reports the owner failure afterwards.
                    Err(_) => true,
                }
            } else {
                false
            };
        if legacy_native {
            let values = input::category(values, "hue")?;
            return Self::update(slf, |native| {
                map_kinds!(native, [Scatter], plot => plot.hue(values))
            });
        }
        Self::prepare_input(slf, |native, roots| {
            prepare_capability!(Hue, native, config => input::hue(config, roots, values))
        })
    }

    fn facet_scales(slf: &Bound<'_, Self>, scales: &str) -> PyResult<()> {
        let scales = match scales {
            "fixed" => core::FacetScales::Fixed,
            "free_x" => core::FacetScales::FreeX,
            "free_y" => core::FacetScales::FreeY,
            "free" => core::FacetScales::Free,
            _ => return Err(PyValueError::new_err("invalid facet scales")),
        };
        Self::prepare_input(slf, |native, roots| {
            prepare_capability!(Facet, native, config =>
                input::facet_scales(config, roots, scales, slf.py()))
        })
    }

    fn composite_precomputed_kde(
        slf: &Bound<'_, Self>,
        x: &Bound<'_, PyAny>,
        y: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::prepare_input(slf, |native, roots| {
            prepare_capability!(Histogram, native, config =>
                input::composite_precomputed_kde(config, roots, None, x, y))
        })
    }

    fn composite_precomputed_kde_from_data(
        slf: &Bound<'_, Self>,
        data: &Bound<'_, PyAny>,
        x: &Bound<'_, PyAny>,
        y: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::prepare_input(slf, |native, roots| {
            prepare_capability!(Histogram, native, config =>
                input::composite_precomputed_kde(config, roots, Some(data), x, y))
        })
    }

    fn x_limits(slf: &Bound<'_, Self>, minimum: f64, maximum: f64) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(NumericX, native, plot => plot.x_limits(minimum, maximum))
        })
    }

    fn y_limits(slf: &Bound<'_, Self>, minimum: f64, maximum: f64) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(NumericY, native, plot => plot.y_limits(minimum, maximum))
        })
    }

    fn x_ticks(slf: &Bound<'_, Self>, ticks: &Bound<'_, PyAny>) -> PyResult<()> {
        let ticks = input::ticks(ticks)?;
        Self::update(slf, |native| {
            map_capability!(NumericX, native, plot => plot.x_ticks(ticks))
        })
    }

    fn y_ticks(slf: &Bound<'_, Self>, ticks: &Bound<'_, PyAny>) -> PyResult<()> {
        let ticks = input::ticks(ticks)?;
        Self::update(slf, |native| {
            map_capability!(NumericY, native, plot => plot.y_ticks(ticks))
        })
    }

    fn title_alignment(slf: &Bound<'_, Self>, alignment: &str) -> PyResult<()> {
        let alignment = policy::alignment(alignment)?;
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.title_alignment(alignment))
        })
    }

    fn subtitle_alignment(slf: &Bound<'_, Self>, alignment: &str) -> PyResult<()> {
        let alignment = policy::alignment(alignment)?;
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.subtitle_alignment(alignment))
        })
    }

    fn guide_text_layout(
        slf: &Bound<'_, Self>,
        role: &str,
        layout: PyRef<'_, PyTextLayout>,
    ) -> PyResult<()> {
        let layout_native = layout.native.clone();
        drop(layout);
        let role = policy::guide_role(role)?;
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.guide_text_layout(role, layout_native))
        })
    }

    fn legend(
        slf: &Bound<'_, Self>,
        policy: &str,
        position: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let policy = policy::legend(policy, position, unset)?;
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.legend(policy))
        })
    }

    fn legend_composition(slf: &Bound<'_, Self>, composition: &str) -> PyResult<()> {
        let composition = policy::composition(composition)?;
        Self::update(slf, |native| {
            map_capability!(All, native, plot => plot.legend_composition(composition))
        })
    }

    fn category_order(slf: &Bound<'_, Self>, order: &Bound<'_, PyAny>) -> PyResult<()> {
        let order = policy::category_order(order)?;
        Self::update(slf, |native| {
            map_capability!(CategoryOrder, native, plot => plot.category_order(order))
        })
    }

    fn x_category_order(slf: &Bound<'_, Self>, order: &Bound<'_, PyAny>) -> PyResult<()> {
        let order = policy::category_order(order)?;
        Self::update(slf, |native| {
            map_capability!(Heatmap, native, plot => plot.x_category_order(order))
        })
    }

    fn y_category_order(slf: &Bound<'_, Self>, order: &Bound<'_, PyAny>) -> PyResult<()> {
        let order = policy::category_order(order)?;
        Self::update(slf, |native| {
            map_capability!(Heatmap, native, plot => plot.y_category_order(order))
        })
    }

    #[pyo3(signature = (*, every=1, offset=0, endpoints="none"))]
    fn x_category_labels(
        slf: &Bound<'_, Self>, every: usize, offset: usize, endpoints: &str,
    ) -> PyResult<()> {
        let selection = policy::category_labels(every, offset, endpoints)?;
        Self::update(slf, |native| {
            map_capability!(CategoricalX, native, plot => plot.x_category_labels(selection))
        })
    }

    #[pyo3(signature = (*, every=1, offset=0, endpoints="none"))]
    fn y_category_labels(
        slf: &Bound<'_, Self>, every: usize, offset: usize, endpoints: &str,
    ) -> PyResult<()> {
        let selection = policy::category_labels(every, offset, endpoints)?;
        Self::update(slf, |native| {
            map_capability!(CategoricalY, native, plot => plot.y_category_labels(selection))
        })
    }

    fn categorical_color_scale(
        slf: &Bound<'_, Self>, scale: PyRef<'_, PyCategoricalColorScale>,
    ) -> PyResult<()> {
        let scale_native = scale.native.clone();
        drop(scale);
        Self::update(slf, |native| {
            map_capability!(CategoricalScale, native, plot => plot.categorical_color_scale(scale_native))
        })
    }

    fn continuous_color_scale(
        slf: &Bound<'_, Self>, scale: PyRef<'_, PyContinuousColorScale>,
    ) -> PyResult<()> {
        let scale_native = scale.native.clone();
        drop(scale);
        Self::update(slf, |native| {
            map_capability!(ContinuousColor, native, plot => plot.continuous_color_scale(scale_native))
        })
    }

    fn significance_mark_style(
        slf: &Bound<'_, Self>, style: PyRef<'_, PySignificanceMarkStyle>,
    ) -> PyResult<()> {
        let style_native = style.native;
        drop(style);
        Self::update(slf, |native| {
            map_capability!(SignificanceStyle, native, plot => plot.significance_mark_style(style_native))
        })
    }

    fn styles(slf: &Bound<'_, Self>, entries: &Bound<'_, PyAny>) -> PyResult<()> {
        let entries = entries.extract::<Vec<PyRef<'_, PyInteractionStyle>>>()?;
        let styles = entries.iter().map(|entry| entry.native.clone()).collect::<Vec<_>>();
        drop(entries);
        Self::update(slf, |native| {
            map_capability!(Interaction, native, plot => plot.styles(styles))
        })
    }

    fn raw_overlay(slf: &Bound<'_, Self>, overlay: &str) -> PyResult<()> {
        let overlay = policy::raw_overlay(overlay)?;
        Self::update(slf, |native| {
            map_capability!(RawOverlay, native, plot => plot.raw_overlay(overlay))
        })
    }

    fn spacing(slf: &Bound<'_, Self>, spacing: &str) -> PyResult<()> {
        let spacing = policy::spacing(spacing)?;
        Self::update(slf, |native| {
            map_capability!(BarSpacing, native, plot => plot.spacing(spacing))
        })
    }

    fn bins(
        slf: &Bound<'_, Self>,
        count: &Bound<'_, PyAny>,
        width: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let count_present = !count.is(unset);
        let width_present = !width.is(unset);
        let bins = match (count_present, width_present) {
            (false, false) => core::HistogramBins::Auto,
            (true, false) => core::HistogramBins::Count(count.extract()?),
            (false, true) => core::HistogramBins::Width(width.extract()?),
            (true, true) => return Err(PyTypeError::new_err(
                "histogram count and width are mutually exclusive",
            )),
        };
        Self::update(slf, |native| {
            map_capability!(Histogram, native, plot => plot.bins(bins))
        })
    }

    fn stat(slf: &Bound<'_, Self>, stat: &str) -> PyResult<()> {
        let stat = policy::histogram_stat(stat)?;
        Self::update(slf, |native| {
            map_capability!(Histogram, native, plot => plot.stat(stat))
        })
    }

    fn composite(slf: &Bound<'_, Self>, composite: &str) -> PyResult<()> {
        let composite = match composite {
            "histogram_only" => core::HistogramComposite::HistogramOnly,
            "kde_and_rug" => core::HistogramComposite::KdeAndRug(core::HistogramKde::Internal),
            _ => return Err(PyValueError::new_err(
                "composite must be 'histogram_only' or 'kde_and_rug'",
            )),
        };
        Self::update(slf, |native| {
            map_capability!(Histogram, native, plot => plot.composite(composite))
        })
    }

    fn qq_reference_line(slf: &Bound<'_, Self>, line: &str) -> PyResult<()> {
        let line = policy::qq_line(line)?;
        Self::update(slf, |native| {
            map_capability!(Qq, native, plot => plot.reference_line(line))
        })
    }

    fn coefficient_reference_line(slf: &Bound<'_, Self>, value: f64) -> PyResult<()> {
        Self::update(slf, |native| {
            map_capability!(Coefficient, native, plot => plot.reference_line(value))
        })
    }

    fn regression_band(slf: &Bound<'_, Self>, band: &str) -> PyResult<()> {
        let band = policy::regression_band(band)?;
        Self::update(slf, |native| {
            map_capability!(Regression, native, plot => plot.band(band))
        })
    }

    fn style(slf: &Bound<'_, Self>, style: &str) -> PyResult<()> {
        let style = policy::line_path_style(style)?;
        Self::update(slf, |native| {
            map_capability!(LinePath, native, plot => plot.style(style))
        })
    }

    fn scale(slf: &Bound<'_, Self>, scale: &str) -> PyResult<()> {
        let scale = policy::axis_scale(scale)?;
        Self::update(slf, |native| {
            map_capability!(Forest, native, plot => plot.scale(scale))
        })
    }

    fn layout(
        slf: &Bound<'_, Self>,
        layout: &str,
        diagonal: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let layout = match layout {
            "cartesian" if diagonal.is(unset) => core::HeatmapLayout::Cartesian,
            "cartesian" => return Err(PyTypeError::new_err(
                "cartesian layout does not accept diagonal",
            )),
            "lower_triangle" => core::HeatmapLayout::LowerTriangle { diagonal: diagonal.extract()? },
            "upper_triangle" => core::HeatmapLayout::UpperTriangle { diagonal: diagonal.extract()? },
            _ => return Err(PyValueError::new_err("invalid heatmap layout")),
        };
        Self::update(slf, |native| {
            map_capability!(Heatmap, native, plot => plot.layout(layout))
        })
    }

    fn annotations(slf: &Bound<'_, Self>, annotations: &Bound<'_, PyAny>) -> PyResult<()> {
        let (annotations, roots) = if annotations.is_instance_of::<pyo3::types::PyString>() {
            if annotations.extract::<&str>()? != "hidden" {
                return Err(PyValueError::new_err("annotations string must be 'hidden'"));
            }
            (core::HeatmapAnnotations::Hidden, Vec::new())
        } else {
            let format = annotations.extract::<PyRef<'_, PyNumericFormat>>()?;
            let (native, roots) = format.snapshot(slf.py())?;
            drop(format);
            (core::HeatmapAnnotations::Values(native), roots)
        };
        Self::update_callback(slf, CallbackSlot::HeatmapValues, roots, |native| {
            map_capability!(Heatmap, native, plot => plot.annotations(annotations))
        })
    }

    fn heatmap_null_row_annotation(
        slf: &Bound<'_, Self>,
        row_label: String,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let text = values::text_intent(text, style, layout, unset)?.native;
        let annotation = core::HeatmapNullRowAnnotation::new(row_label, text)
            .map_err(crate::native_error)?;
        Self::update(slf, |native| {
            map_capability!(Heatmap, native, plot => plot.heatmap_null_row_annotation(annotation))
        })
    }

    fn total_labels(
        slf: &Bound<'_, Self>,
        base: &Bound<'_, PyAny>,
        terminal: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let base = values::text_intent(base, unset, unset, unset)?.native;
        let terminal = values::text_intent(terminal, unset, unset, unset)?.native;
        let labels = core::WaterfallLabels::new(base, terminal).map_err(crate::native_error)?;
        Self::update(slf, |native| {
            map_capability!(Waterfall, native, plot => plot.total_labels(labels))
        })
    }

    fn guide_horizontal(
        slf: &Bound<'_, Self>, y: f64, color: PyRef<'_, PyColor>, width: f32, pattern: &str,
    ) -> PyResult<()> {
        let native_color = color.native;
        drop(color);
        let style = core::GuideStyle::new(native_color, width, policy::line_pattern(pattern)?)
            .map_err(crate::native_error)?;
        let guide = core::GuideSpec::horizontal(y, style);
        Self::update(slf, |native| {
            map_capability!(Guides, native, plot => plot.guide(guide))
        })
    }

    fn guide_vertical(
        slf: &Bound<'_, Self>, x: f64, color: PyRef<'_, PyColor>, width: f32, pattern: &str,
    ) -> PyResult<()> {
        let native_color = color.native;
        drop(color);
        let style = core::GuideStyle::new(native_color, width, policy::line_pattern(pattern)?)
            .map_err(crate::native_error)?;
        let guide = core::GuideSpec::vertical(x, style);
        Self::update(slf, |native| {
            map_capability!(Guides, native, plot => plot.guide(guide))
        })
    }

    #[allow(clippy::too_many_arguments, reason = "flat endpoints and complete guide style are independent inputs")]
    fn guide_segment(
        slf: &Bound<'_, Self>, x_start: f64, y_start: f64, x_end: f64, y_end: f64,
        color: PyRef<'_, PyColor>, width: f32, pattern: &str,
    ) -> PyResult<()> {
        let native_color = color.native;
        drop(color);
        let style = core::GuideStyle::new(native_color, width, policy::line_pattern(pattern)?)
            .map_err(crate::native_error)?;
        let guide = core::GuideSpec::segment(x_start, y_start, x_end, y_end, style);
        Self::update(slf, |native| {
            map_capability!(Guides, native, plot => plot.guide(guide))
        })
    }

    fn text_annotation(
        slf: &Bound<'_, Self>,
        position: PyRef<'_, PyTextPosition>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let position_native = position.native.clone();
        drop(position);
        let text = values::text_intent(text, style, layout, unset)?.native;
        let annotation = core::PlotTextAnnotation::at(position_native, text)
            .map_err(crate::native_error)?;
        Self::update(slf, |native| {
            map_capability!(TextAnnotation, native, plot => plot.text_annotation(annotation))
        })
    }

    fn svg(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::Svg)
    }

    fn svg_preserve_text(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::SvgPreserveText)
    }

    fn png(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::Png)
    }

    fn png_preview(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::PngPreview)
    }

    fn png_with_options(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        options: PyRef<'_, PyPngOptions>,
    ) -> PyResult<PyOutput> {
        let native = options.native;
        drop(options);
        Self::publish(slf, path, Terminal::PngWithOptions(native))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.roots.traverse(&visit)?;
        self.inputs.traverse(&visit)
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            (
                owner.native.take(),
                std::mem::take(&mut owner.roots),
                std::mem::take(&mut owner.inputs),
            )
        };
        drop(retired);
        Ok(())
    }
    }
}
