//! Compile-time exact-kind dispatch; no erased scientific owner or runtime DSL.

// The body is type-checked independently for each exact Config<K>.
macro_rules! call_configs {
    ([$($kind:ident),+] $native:expr, $config:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind($config) => $body),+
        }
    };
}

macro_rules! map_configs {
    ([$($kind:ident),+] $native:expr, $config:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind($config) => NativePlot::$kind($body)),+
        }
    };
}

macro_rules! map_all {
    ([$($kind:ident),+] $native:expr, $plot:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind(config) =>
                NativePlot::$kind(config.map_native(|$plot| $body))),+
        }
    };
}

macro_rules! map_kinds {
    ($native:expr, [$($kind:ident),+], $plot:ident => $body:expr) => {
        match $native {
            $(NativePlot::$kind(config) =>
                Ok(NativePlot::$kind(config.map_native(|$plot| $body)))),+,
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "this exact plot family does not support that operation",
            )),
        }
    };
}

// These are the sealed sets in core kind.rs, not inferred visual similarities.
macro_rules! map_capability {
    (All, $native:expr, $plot:ident => $body:expr) => {
        Ok(all_families!(map_all, $native, $plot => $body))
    };
    (NumericX, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Histogram, Density, Qq, PointInterval, Line,
            Trajectory, LinePath, Coefficient, Forest, Dumbbell, HorizontalBar,
            StackedHorizontalBar, Regression, Marginal, Beeswarm, Waterfall],
            $plot => $body)
    };
    (NumericY, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Histogram, Density, Qq, Line, Trajectory,
            LinePath, Interaction, Bar, StackedBar, Boxplot, Violin, Raincloud,
            Regression, Marginal], $plot => $body)
    };
    (CategoricalX, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Trajectory, Interaction, Heatmap, Bar, StackedBar,
            Boxplot, Violin, Raincloud], $plot => $body)
    };
    (CategoricalY, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [PointInterval, Coefficient, Forest, Dumbbell, Heatmap,
            HorizontalBar, StackedHorizontalBar, Beeswarm, Waterfall], $plot => $body)
    };
    (ContinuousColor, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Heatmap, Beeswarm], $plot => $body)
    };
    (Facet, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Histogram, Density, Qq, PointInterval, Line,
            Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell,
            Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot,
            Violin, Raincloud, Regression, Beeswarm, Waterfall], $plot => $body)
    };
    (EstimateDetailText, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [PointInterval, Coefficient, Forest], $plot => $body)
    };
    (SignificanceStyle, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Bar, HorizontalBar, StackedBar, StackedHorizontalBar,
            Boxplot, Violin, Raincloud, Coefficient], $plot => $body)
    };
    (Guides, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Histogram, Density, Qq, PointInterval, Line,
            Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell, Bar,
            HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
            Regression, Raincloud, Marginal, Beeswarm, Waterfall], $plot => $body)
    };
    (TextAnnotation, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Histogram, Density, Qq, PointInterval, Line,
            Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell,
            Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot,
            Violin, Raincloud, Regression, Marginal, Beeswarm, Waterfall], $plot => $body)
    };
    (CategoryOrder, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [PointInterval, Bar, HorizontalBar, StackedBar,
            StackedHorizontalBar], $plot => $body)
    };
    (ZeroLine, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [PointInterval, HorizontalBar, StackedHorizontalBar,
            Beeswarm], $plot => $body)
    };
    (Markers, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Line, LinePath, Interaction], $plot => $body)
    };
    (PointOutline, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Density, Qq, PointInterval, Line, Trajectory,
            LinePath, Interaction, Coefficient, Forest, Dumbbell, Boxplot, Violin,
            Raincloud, Regression, Marginal, Matrix, Beeswarm], $plot => $body)
    };
    (CategoricalScale, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Scatter, Line, Trajectory, LinePath, Density, Bar,
            HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Interaction,
            Violin, Raincloud, Marginal, Matrix], $plot => $body)
    };
    (RawOverlay, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Boxplot, Violin], $plot => $body)
    };
    (BarSpacing, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [Bar, HorizontalBar], $plot => $body)
    };
    // Exact-only native methods, not invented capability traits.
    ($kind:ident, $native:expr, $plot:ident => $body:expr) => {
        map_kinds!($native, [$kind], $plot => $body)
    };
}
