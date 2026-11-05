use super::{context, Context, PlotAdapter};
use crate::error::{length, BindingResult};
use crate::{
    kind, AdapterError, ArraySource, AxisInput, AxisValues, BarInput, Bounds, CategoryInput,
    CategoryValues, CoefficientInput, ColumnsInput, DumbbellEndpoints, ErrorBar, HistogramGeometry,
    HistogramInput, NumericInput, NumericValues, Plot, RaincloudGeometry, RaincloudInput,
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
    ($kind:ident,$new:ident,$from:ident,$bind:ident,
        ($first:ident:$first_trait:ident=>$first_method:ident $(,$arg:ident:$input:ident=>$method:ident)*),
        ($($policy:ident:$ty:ty),*),$plot:expr) => {
        impl<'source> PlotAdapter<'source,kind::$kind> {
            #[doc=concat!("Bind direct inputs for native ",stringify!($kind)," ",stringify!($new)," mode.")]
            pub fn $new($first:impl $first_trait,$($arg:impl $input,)*$($policy:$ty),*)->std::result::Result<Self,AdapterError> { Self::$bind(None,$first,$($arg,)*$($policy),*) }
            #[doc=concat!("Bind logical source columns/direct inputs for native ",stringify!($kind)," ",stringify!($new)," mode.")]
            pub fn $from(source:impl ArraySource+'source,$first:impl $first_trait,$($arg:impl $input,)*$($policy:$ty),*)->std::result::Result<Self,AdapterError> { Self::$bind(context(source)?,$first,$($arg,)*$($policy),*) }
            fn $bind(source:Context<'source>,$first:impl $first_trait,$($arg:impl $input,)*$($policy:$ty),*)->BindingResult<Self> {
                let $first=$first.$first_method(source.as_deref(),stringify!($first))?; let len=$first.slots();
                $(let $arg=$arg.$method(source.as_deref(),stringify!($arg))?; length(stringify!($arg),$arg.name(),len,$arg.slots())?;)*
                Ok(Self::bound($plot,source,Some(len)))
            }
        }
    };
}
records!(Regression,new,from_data,coordinates,(x:NumericInput=>bind_numeric,y:NumericInput=>bind_numeric),(),Plot::regression(x,y));
records!(Marginal,new,from_data,coordinates,(x:NumericInput=>bind_numeric,y:NumericInput=>bind_numeric),(),Plot::marginal(x,y));
records!(Histogram,new,from_data,samples_input,(samples:NumericInput=>bind_numeric),(),Plot::histogram(HistogramInput::samples(samples)));
records!(Density,new,from_data,samples_input,(value:NumericInput=>bind_numeric),(),Plot::density(value));
records!(Qq,new,from_data,samples_input,(value:NumericInput=>bind_numeric),(),Plot::qq(value));
records!(PointInterval,new,from_data,estimates_input,(category:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric,lower:NumericInput=>bind_numeric,upper:NumericInput=>bind_numeric),(),Plot::point_interval(category,estimate,Bounds::new(lower,upper)));
records!(Trajectory,new,from_data,coordinates,(x:AxisInput=>bind_axis,y:NumericInput=>bind_numeric,group:CategoryInput=>bind_category),(),Plot::trajectory(x,y,group));
records!(Trajectory,new_numeric_x,from_data_numeric_x,numeric_coordinates,(x:NumericInput=>bind_numeric,y:NumericInput=>bind_numeric,group:CategoryInput=>bind_category),(),Plot::trajectory(x,y,group));
records!(Trajectory,new_categorical_x,from_data_categorical_x,category_coordinates,(x:CategoryInput=>bind_category,y:NumericInput=>bind_numeric,group:CategoryInput=>bind_category),(),Plot::trajectory(x,y,group));
records!(LinePath,new,from_data,coordinates,(x:NumericInput=>bind_numeric,y:NumericInput=>bind_numeric,group:CategoryInput=>bind_category),(),Plot::line_path(x,y,group));
records!(Interaction,new,from_data,estimates_input,(condition:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric,group:CategoryInput=>bind_category,lower:NumericInput=>bind_numeric,upper:NumericInput=>bind_numeric),(),Plot::interaction(condition,estimate,group,Bounds::new(lower,upper)));
records!(Coefficient,new,from_data,estimates_input,(term:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric),(),Plot::coefficient(term,CoefficientInput::estimates(estimate)));
records!(Coefficient,with_intervals,with_intervals_from_data,interval_input,(term:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric,lower:NumericInput=>bind_numeric,upper:NumericInput=>bind_numeric),(),Plot::coefficient(term,CoefficientInput::estimates_with_intervals(estimate,Bounds::new(lower,upper))));
records!(Forest,new,from_data,estimates_input,(term:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric,lower:NumericInput=>bind_numeric,upper:NumericInput=>bind_numeric),(reference:f64),Plot::forest(term,estimate,Bounds::new(lower,upper),reference));
records!(Dumbbell,new,from_data,paired_input,(category:CategoryInput=>bind_category,left:NumericInput=>bind_numeric,right:NumericInput=>bind_numeric),(endpoints:DumbbellEndpoints),Plot::dumbbell(category,left,right,endpoints));
records!(Heatmap,records,records_from_data,records_input,(x:CategoryInput=>bind_category,y:CategoryInput=>bind_category,fill:NumericInput=>bind_numeric),(),Plot::heatmap(x,y,fill));
records!(StackedBar,new,from_data,estimates_input,(category:CategoryInput=>bind_category,value:NumericInput=>bind_numeric,series:CategoryInput=>bind_category),(),Plot::stacked_bar(category,value,series));
records!(StackedHorizontalBar,new,from_data,estimates_input,(category:CategoryInput=>bind_category,value:NumericInput=>bind_numeric,series:CategoryInput=>bind_category),(),Plot::stacked_horizontal_bar(category,value,series));
records!(Beeswarm,new,from_data,samples_input,(category:CategoryInput=>bind_category,value:NumericInput=>bind_numeric,color:NumericInput=>bind_numeric),(),Plot::beeswarm(category,value,color));
records!(Waterfall,new,from_data,deltas_input,(category:CategoryInput=>bind_category,delta:NumericInput=>bind_numeric),(base:f64),Plot::waterfall(category,delta,base));
macro_rules! bar_modes {
    ($family:ident,$native:ident) => {
        records!($family,new,from_data,samples_input,(category:CategoryInput=>bind_category,samples:NumericInput=>bind_numeric),(),Plot::$native(category,BarInput::samples(samples)));
        records!($family,with_error_bar,with_error_bar_from_data,configured_samples_input,(category:CategoryInput=>bind_category,samples:NumericInput=>bind_numeric),(policy:impl Into<ErrorBar>),Plot::$native(category,BarInput::samples(samples).error_bar(policy)));
        records!($family,estimates,estimates_from_data,estimates_input,(category:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric),(),Plot::$native(category,BarInput::estimates(estimate)));
        records!($family,estimates_with_error_bars,estimates_with_error_bars_from_data,interval_input,(category:CategoryInput=>bind_category,estimate:NumericInput=>bind_numeric,lower:NumericInput=>bind_numeric,upper:NumericInput=>bind_numeric),(),Plot::$native(category,BarInput::estimates_with_error_bars(estimate,Bounds::new(lower,upper))));
    };
}
bar_modes!(Bar, bar);
bar_modes!(HorizontalBar, horizontal_bar);
macro_rules! geometry {
    ($family:ident,$input:ident,$geometry:ty,$native:ident) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Own caller-computed native geometry without installing an array source.
            pub fn geometry(geometry: $geometry) -> std::result::Result<Self, AdapterError> {
                Ok(Self::bound(
                    Plot::$native($input::geometry(geometry)),
                    None,
                    None,
                ))
            }
            /// Own native geometry and retain a source for later admitted bindings.
            pub fn geometry_from_data(
                source: impl ArraySource + 'source,
                geometry: $geometry,
            ) -> std::result::Result<Self, AdapterError> {
                Ok(Self::bound(
                    Plot::$native($input::geometry(geometry)),
                    context(source)?,
                    None,
                ))
            }
        }
    };
}
geometry!(Histogram, HistogramInput, HistogramGeometry, histogram);
geometry!(Raincloud, RaincloudInput, RaincloudGeometry, raincloud);

impl PlotAdapter<'_, kind::Line> {
    /// Bind bounds with exactly the line's single-variable or selected-column shape.
    pub fn band(
        mut self,
        lower: impl crate::NumericSeriesInput,
        upper: impl crate::NumericSeriesInput,
    ) -> std::result::Result<Self, AdapterError> {
        let native_lower = lower.native_slots();
        let native_upper = upper.native_slots();
        let (lower, lower_shape) = lower.bind_series(self.source.as_deref(), "lower")?;
        let (upper, upper_shape) = upper.bind_series(self.source.as_deref(), "upper")?;
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
    /// Use direct matrix columns as variables and rows as observations.
    pub fn new(variables: impl ColumnsInput) -> std::result::Result<Self, AdapterError> {
        Self::matrix_input(None, variables)
    }
    /// Select ordered logical columns, or `..` for every column.
    pub fn from_data(
        source: impl ArraySource + 'source,
        columns: impl ColumnsInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::matrix_input(context(source)?, columns)
    }
    /// Explicit numeric discovery on a homogeneous source uses all columns.
    pub fn numeric_from_data(
        source: impl ArraySource + 'source,
    ) -> std::result::Result<Self, AdapterError> {
        Self::from_data(source, ..)
    }
    fn matrix_input(source: Context<'source>, variables: impl ColumnsInput) -> BindingResult<Self> {
        let local = variables.local_identities();
        let (rows, columns) = variables.bind_columns(source.as_deref())?;
        let keys = columns
            .iter()
            .map(|(index, _)| {
                if local {
                    format!("__lichtung_matrix_{index}")
                } else {
                    crate::source::name(*index)
                }
            })
            .collect::<Vec<_>>();
        let labels = columns
            .iter()
            .map(|(index, _)| crate::source::name(*index))
            .collect::<Vec<_>>();
        let values = columns
            .into_iter()
            .zip(&keys)
            .map(|((_, values), key)| NumericValues::nullable(key, values))
            .collect();
        let mut adapter = Self::bound(Plot::matrix(values), source, Some(rows));
        adapter.matrix_keys = Some(keys);
        if local {
            adapter.column_labels(labels)
        } else {
            Ok(adapter)
        }
    }
}
