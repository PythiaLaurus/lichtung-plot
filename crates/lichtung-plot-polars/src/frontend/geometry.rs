//! Independent column domains; scientific validation and grouping stay native.

use polars_core::prelude::DataFrame;

use super::PlotAdapter;
use crate::error::{length, BindingResult};
use crate::{
    kind, AdapterError, CategoryInput, HistogramComposite, HistogramGeometry, HistogramInput,
    HistogramKde, NumericInput, Plot, RaincloudGeometry, RaincloudInput,
};

// Each domain has its own selector context and length. Only the main context is
// moved into the frontend; auxiliary borrows end after these snapshots.
macro_rules! columnar_geometry {
    ($family:ident, $direct:ident, $from:ident, $bind:ident,
        ($first:ident: $input:ident => $method:ident $(, $arg:ident: $arg_input:ident => $arg_method:ident)*),
        ($(($aux:ident => $head:ident: $head_input:ident => $head_method:ident $(, $tail:ident: $tail_input:ident => $tail_method:ident)*)),+),
        ($($dimension:ident),+), $points:expr, $plot:expr) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Snapshot complete geometry from direct columns in main/auxiliary
            /// domain order, followed by scientific dimension names.
            #[expect(clippy::too_many_arguments, reason = "complete columnar geometry keeps every scientific role explicit")]
            pub fn $direct(
                $first: impl $input, $($arg: impl $arg_input,)*
                $($head: impl $head_input, $($tail: impl $tail_input,)*)+
                $($dimension: impl Into<String>),+
            ) -> std::result::Result<Self, AdapterError> {
                $(let $aux = None;)+
                Self::$bind(None, $($aux,)+ $first, $($arg,)* $($head, $($tail,)*)+ $($dimension),+)
            }

            /// Snapshot independently selected/direct component columns. The main
            /// source is retained; each optional auxiliary source is used only
            /// for its own domain, with no cross-domain selector fallback.
            #[expect(clippy::too_many_arguments, reason = "independent sources precede explicit complete geometry roles")]
            pub fn $from(
                source: &'source DataFrame, $($aux: Option<&DataFrame>,)+
                $first: impl $input, $($arg: impl $arg_input,)*
                $($head: impl $head_input, $($tail: impl $tail_input,)*)+
                $($dimension: impl Into<String>),+
            ) -> std::result::Result<Self, AdapterError> {
                Self::$bind(Some(source), $($aux,)+ $first, $($arg,)* $($head, $($tail,)*)+ $($dimension),+)
            }

            #[expect(clippy::too_many_arguments, reason = "private extraction mirrors the public complete geometry roles")]
            fn $bind(
                source: Option<&'source DataFrame>, $($aux: Option<&DataFrame>,)+
                $first: impl $input, $($arg: impl $arg_input,)*
                $($head: impl $head_input, $($tail: impl $tail_input,)*)+
                $($dimension: impl Into<String>),+
            ) -> BindingResult<Self> {
                let $first = $first.$method(source, stringify!($first))?;
                let len = $first.len();
                $(let $arg = $arg.$arg_method(source, stringify!($arg))?;
                length(stringify!($arg), $arg.name(), len, $arg.len())?;)*
                $(let $head = $head.$head_method($aux, stringify!($head))?;
                let len = $head.len();
                $(let $tail = $tail.$tail_method($aux, stringify!($tail))?;
                length(stringify!($tail), $tail.name(), len, $tail.len())?;)*)+
                let points = $points;
                let mut adapter = Self::bound($plot, source, None);
                adapter.geometry_points = points;
                Ok(adapter)
            }
        }
    };
}

columnar_geometry!(Histogram, geometry, geometry_from_data, histogram_geometry,
(bin_lower: NumericInput => bind_numeric, bin_upper: NumericInput => bind_numeric,
    bin_density: NumericInput => bind_numeric),
((kde_source => kde_x: NumericInput => bind_numeric, kde_y: NumericInput => bind_numeric),
 (rug_source => rug_x: NumericInput => bind_numeric, rug_baseline: NumericInput => bind_numeric,
    rug_top: NumericInput => bind_numeric)),
(value_dimension, density_dimension), None,
Plot::histogram(HistogramInput::geometry(HistogramGeometry {
    value_dimension: value_dimension.into(), density_dimension: density_dimension.into(),
    bin_lower, bin_upper, bin_density, kde_x, kde_y, rug_x, rug_baseline, rug_top,
})));

columnar_geometry!(Raincloud, geometry, geometry_from_data, raincloud_geometry,
(category: CategoryInput => bind_category, q1: NumericInput => bind_numeric,
    median: NumericInput => bind_numeric, q3: NumericInput => bind_numeric),
((density_source => density_category: CategoryInput => bind_category,
    density_y: NumericInput => bind_numeric, normalized_width: NumericInput => bind_numeric),
 (points_source => point_category: CategoryInput => bind_category,
    point_y: NumericInput => bind_numeric, normalized_strip: NumericInput => bind_numeric)),
(category_dimension, value_dimension), Some(point_y.len()),
Plot::raincloud(RaincloudInput::geometry(RaincloudGeometry {
    category_dimension: category_dimension.into(), value_dimension: value_dimension.into(),
    hue_dimension: None, category, hue: None, q1, median, q3,
    density_category, density_hue: None, density_y, normalized_width,
    point_category, point_hue: None, point_y, normalized_strip,
})));

columnar_geometry!(Raincloud, geometry_with_hue, geometry_with_hue_from_data, hued_raincloud_geometry,
(category: CategoryInput => bind_category, hue: CategoryInput => bind_category,
    q1: NumericInput => bind_numeric, median: NumericInput => bind_numeric, q3: NumericInput => bind_numeric),
((density_source => density_category: CategoryInput => bind_category, density_hue: CategoryInput => bind_category,
    density_y: NumericInput => bind_numeric, normalized_width: NumericInput => bind_numeric),
 (points_source => point_category: CategoryInput => bind_category, point_hue: CategoryInput => bind_category,
    point_y: NumericInput => bind_numeric, normalized_strip: NumericInput => bind_numeric)),
(category_dimension, value_dimension, hue_dimension), Some(point_y.len()),
Plot::raincloud(RaincloudInput::geometry(RaincloudGeometry {
    category_dimension: category_dimension.into(), value_dimension: value_dimension.into(),
    hue_dimension: Some(hue_dimension.into()), category, hue: Some(hue), q1, median, q3,
    density_category, density_hue: Some(density_hue), density_y, normalized_width,
    point_category, point_hue: Some(point_hue), point_y, normalized_strip,
})));

impl PlotAdapter<'_, kind::Histogram> {
    /// Snapshot direct KDE columns for the sample-owned KDE+rug composite.
    /// Samples, bins, stat and the main lookup context are unchanged.
    pub fn composite_precomputed_kde(
        self,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.bind_kde(None, x, y)
    }

    /// Snapshot KDE from an independent source, retaining no borrow of it and
    /// never using the main source as a selector fallback.
    pub fn composite_precomputed_kde_from_data(
        self,
        source: &DataFrame,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.bind_kde(Some(source), x, y)
    }

    fn bind_kde(
        mut self,
        source: Option<&DataFrame>,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> BindingResult<Self> {
        let x = x.bind_numeric(source, "KDE x")?;
        let y = y.bind_numeric(source, "KDE y")?;
        length("KDE y", y.name(), x.len(), y.len())?;
        self.plot = self
            .plot
            .composite(HistogramComposite::KdeAndRug(HistogramKde::Precomputed {
                x,
                y,
            }));
        Ok(self)
    }
}

#[cfg(test)]
mod tests;
