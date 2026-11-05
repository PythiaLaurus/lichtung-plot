//! Record-form constructors bind each role before calling its native constructor.

use polars_core::prelude::DataFrame;

use super::PlotAdapter;
use crate::error::{length, BindingResult};
use crate::{
    kind, AdapterError, AxisInput, AxisValues, BarInput, Bounds, CategoryInput, CategoryValues,
    CoefficientInput, ColumnsInput, DumbbellEndpoints, ErrorBar, HistogramGeometry, HistogramInput,
    NumericInput, NumericValues, Plot, RaincloudGeometry, RaincloudInput,
};

trait Slots {
    fn slots(&self) -> usize;
}
impl Slots for NumericValues {
    fn slots(&self) -> usize {
        self.len()
    }
}
impl Slots for CategoryValues {
    fn slots(&self) -> usize {
        self.len()
    }
}
impl Slots for AxisValues {
    fn slots(&self) -> usize {
        match self {
            Self::Numeric(values) => values.len(),
            Self::Category(values) => values.len(),
        }
    }
}

macro_rules! records {
    ($kind:ident, $new:ident, $from:ident, $bind:ident,
        ($first:ident: $first_trait:ident => $first_method:ident $(, $arg:ident: $input:ident => $method:ident)* $(,)?),
        ($($policy:ident: $ty:ty),* $(,)?), $plot:expr) => {
        impl<'source> PlotAdapter<'source, kind::$kind> {
            #[doc = concat!("Bind direct variables for the native ", stringify!($kind), " `", stringify!($new), "` mode.")]
            pub fn $new($first: impl $first_trait, $($arg: impl $input,)* $($policy: $ty),*) -> std::result::Result<Self, AdapterError> {
                Self::$bind(None, $first, $($arg,)* $($policy),*)
            }
            #[doc = concat!("Bind fields/direct variables in a DataFrame for the native ", stringify!($kind), " `", stringify!($new), "` mode.")]
            pub fn $from(source: &'source DataFrame, $first: impl $first_trait, $($arg: impl $input,)* $($policy: $ty),*) -> std::result::Result<Self, AdapterError> {
                Self::$bind(Some(source), $first, $($arg,)* $($policy),*)
            }
            fn $bind(source: Option<&'source DataFrame>, $first: impl $first_trait, $($arg: impl $input,)* $($policy: $ty),*) -> BindingResult<Self> {
                let $first = $first.$first_method(source, stringify!($first))?;
                let len = $first.slots();
                $(let $arg = $arg.$method(source, stringify!($arg))?;
                length(stringify!($arg), $arg.name(), len, $arg.slots())?;)*
                Ok(Self::bound($plot, source, Some(len)))
            }
        }
    };
}
records!(Regression, new, from_data, coordinates,
    (x: NumericInput => bind_numeric, y: NumericInput => bind_numeric), (), Plot::regression(x, y));
records!(Marginal, new, from_data, coordinates,
    (x: NumericInput => bind_numeric, y: NumericInput => bind_numeric), (), Plot::marginal(x, y));
records!(Histogram, new, from_data, samples_input,
    (samples: NumericInput => bind_numeric), (), Plot::histogram(HistogramInput::samples(samples)));
records!(Density, new, from_data, samples_input,
    (value: NumericInput => bind_numeric), (), Plot::density(value));
records!(Qq, new, from_data, samples_input,
    (value: NumericInput => bind_numeric), (), Plot::qq(value));
records!(PointInterval, new, from_data, estimates_input,
    (category: CategoryInput => bind_category, estimate: NumericInput => bind_numeric,
        lower: NumericInput => bind_numeric, upper: NumericInput => bind_numeric), (),
    Plot::point_interval(category, estimate, Bounds::new(lower, upper)));
records!(Trajectory, new, from_data, coordinates,
    (x: AxisInput => bind_axis, y: NumericInput => bind_numeric, group: CategoryInput => bind_category), (),
    Plot::trajectory(x, y, group));
records!(Trajectory, new_numeric_x, from_data_numeric_x, numeric_coordinates,
    (x: NumericInput => bind_numeric, y: NumericInput => bind_numeric, group: CategoryInput => bind_category), (),
    Plot::trajectory(x, y, group));
records!(Trajectory, new_categorical_x, from_data_categorical_x, category_coordinates,
    (x: CategoryInput => bind_category, y: NumericInput => bind_numeric, group: CategoryInput => bind_category), (),
    Plot::trajectory(x, y, group));
records!(LinePath, new, from_data, coordinates,
    (x: NumericInput => bind_numeric, y: NumericInput => bind_numeric, group: CategoryInput => bind_category), (),
    Plot::line_path(x, y, group));
records!(Interaction, new, from_data, estimates_input,
    (condition: CategoryInput => bind_category, estimate: NumericInput => bind_numeric, group: CategoryInput => bind_category,
        lower: NumericInput => bind_numeric, upper: NumericInput => bind_numeric), (),
    Plot::interaction(condition, estimate, group, Bounds::new(lower, upper)));
records!(Coefficient, new, from_data, estimates_input,
    (term: CategoryInput => bind_category, estimate: NumericInput => bind_numeric), (),
    Plot::coefficient(term, CoefficientInput::estimates(estimate)));
records!(Coefficient, with_intervals, with_intervals_from_data, interval_input,
    (term: CategoryInput => bind_category, estimate: NumericInput => bind_numeric,
        lower: NumericInput => bind_numeric, upper: NumericInput => bind_numeric), (),
    Plot::coefficient(term, CoefficientInput::estimates_with_intervals(estimate, Bounds::new(lower, upper))));
records!(Forest, new, from_data, estimates_input,
    (term: CategoryInput => bind_category, estimate: NumericInput => bind_numeric,
        lower: NumericInput => bind_numeric, upper: NumericInput => bind_numeric), (reference: f64),
    Plot::forest(term, estimate, Bounds::new(lower, upper), reference));
records!(Dumbbell, new, from_data, paired_input,
    (category: CategoryInput => bind_category, left: NumericInput => bind_numeric, right: NumericInput => bind_numeric),
    (endpoints: DumbbellEndpoints), Plot::dumbbell(category, left, right, endpoints));
records!(Heatmap, records, records_from_data, records_input,
    (x: CategoryInput => bind_category, y: CategoryInput => bind_category, fill: NumericInput => bind_numeric), (),
    Plot::heatmap(x, y, fill));
records!(StackedBar, new, from_data, estimates_input,
    (category: CategoryInput => bind_category, value: NumericInput => bind_numeric, series: CategoryInput => bind_category), (),
    Plot::stacked_bar(category, value, series));
records!(StackedHorizontalBar, new, from_data, estimates_input,
    (category: CategoryInput => bind_category, value: NumericInput => bind_numeric, series: CategoryInput => bind_category), (),
    Plot::stacked_horizontal_bar(category, value, series));
records!(Beeswarm, new, from_data, samples_input,
    (category: CategoryInput => bind_category, value: NumericInput => bind_numeric, color: NumericInput => bind_numeric), (),
    Plot::beeswarm(category, value, color));
records!(Waterfall, new, from_data, deltas_input,
    (category: CategoryInput => bind_category, delta: NumericInput => bind_numeric), (base: f64), Plot::waterfall(category, delta, base));

macro_rules! bar_modes {
    ($kind:ident, $native:ident) => {
        records!($kind, new, from_data, samples_input,
            (category: CategoryInput => bind_category, samples: NumericInput => bind_numeric), (),
            Plot::$native(category, BarInput::samples(samples)));
        records!($kind, with_error_bar, with_error_bar_from_data, configured_samples_input,
            (category: CategoryInput => bind_category, samples: NumericInput => bind_numeric), (policy: impl Into<ErrorBar>),
            Plot::$native(category, BarInput::samples(samples).error_bar(policy)));
        records!($kind, estimates, estimates_from_data, estimates_input,
            (category: CategoryInput => bind_category, estimate: NumericInput => bind_numeric), (),
            Plot::$native(category, BarInput::estimates(estimate)));
        records!($kind, estimates_with_error_bars, estimates_with_error_bars_from_data, interval_input,
            (category: CategoryInput => bind_category, estimate: NumericInput => bind_numeric,
                lower: NumericInput => bind_numeric, upper: NumericInput => bind_numeric), (),
            Plot::$native(category, BarInput::estimates_with_error_bars(estimate, Bounds::new(lower, upper))));
    };
}
bar_modes!(Bar, bar);
bar_modes!(HorizontalBar, horizontal_bar);

macro_rules! geometry {
    ($family:ident, $input:ident, $geometry:ty, $native:ident) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Bind complete caller-computed native geometry without installing a source.
            pub fn geometry(geometry: $geometry) -> std::result::Result<Self, AdapterError> {
                Ok(Self::bound(
                    Plot::$native($input::geometry(geometry)),
                    None,
                    None,
                ))
            }
            /// Bind complete geometry and retain a source for subsequent admitted bindings.
            pub fn geometry_from_data(
                source: &'source DataFrame,
                geometry: $geometry,
            ) -> std::result::Result<Self, AdapterError> {
                Ok(Self::bound(
                    Plot::$native($input::geometry(geometry)),
                    Some(source),
                    None,
                ))
            }
        }
    };
}
geometry!(Histogram, HistogramInput, HistogramGeometry, histogram);
geometry!(Raincloud, RaincloudInput, RaincloudGeometry, raincloud);

impl PlotAdapter<'_, kind::Line> {
    /// Bind independently supplied lower/upper bounds, not a new interval estimator.
    pub fn band(
        mut self,
        lower: impl crate::NumericSeriesInput,
        upper: impl crate::NumericSeriesInput,
    ) -> std::result::Result<Self, AdapterError> {
        let native_lower = lower.native_slots();
        let native_upper = upper.native_slots();
        let (lower, lower_shape) = lower.bind_series(self.source, "lower")?;
        let (upper, upper_shape) = upper.bind_series(self.source, "upper")?;
        for (shape, native) in [(&lower_shape, native_lower), (&upper_shape, native_upper)] {
            match (&self.expansion, shape) {
                (_, None) if native => {}
                (None, None) => {}
                (Some(expansion), Some((rows, columns, _)))
                    if *rows == expansion.rows && columns.len() == expansion.keys.len() => {}
                _ => {
                    return Err(crate::error::binding(
                        "band",
                        "bounds must match the line's single-variable or selected-column shape",
                    ))
                }
            }
        }
        self.aligned("lower", lower.name(), lower.len())?;
        self.aligned("upper", upper.name(), upper.len())?;
        self.plot = self.plot.band(Bounds::new(lower, upper));
        Ok(self)
    }
}

impl<'source> PlotAdapter<'source, kind::Matrix> {
    /// Bind direct ordered Polars variables as a native scatter matrix.
    pub fn new(variables: impl ColumnsInput) -> std::result::Result<Self, AdapterError> {
        Self::matrix_input(None, variables)
    }
    /// Bind an ordered field selection; `..` includes every logical field.
    pub fn from_data(
        source: &'source DataFrame,
        columns: impl ColumnsInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::matrix_input(Some(source), columns)
    }
    /// Explicitly discover supported numeric fields, in schema order.
    pub fn numeric_from_data(
        source: &'source DataFrame,
    ) -> std::result::Result<Self, AdapterError> {
        Self::matrix_input(Some(source), super::wide::numeric_names(source).as_slice())
    }
    fn matrix_input(
        source: Option<&'source DataFrame>,
        variables: impl ColumnsInput,
    ) -> BindingResult<Self> {
        let columns = variables.bind_columns(source)?;
        let len = columns[0].len();
        let mut values = crate::snapshot::allocate(columns.len(), "matrix")?;
        for (index, column) in columns.iter().enumerate() {
            length("matrix", column.name().as_str(), len, column.len())?;
            values.push(NumericValues::nullable(
                format!("__lichtung_matrix_{index}"),
                crate::numeric::values(column)?,
            ));
        }
        let mut adapter = Self::bound(Plot::matrix(values), source, Some(len));
        adapter.matrix_keys = Some(
            (0..columns.len())
                .map(|index| format!("__lichtung_matrix_{index}"))
                .collect(),
        );
        adapter.column_labels(columns.iter().enumerate().map(|(index, column)| {
            if column.name().is_empty() {
                format!("Column {}", index + 1)
            } else {
                column.name().to_string()
            }
        }))
    }
}
