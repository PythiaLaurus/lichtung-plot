//! `lichtung-plot::builder` — the `Chart<'a>` builder API.
//!
//! The primary user-facing entry point. Borrowers of a polars
//! `DataFrame` chain configuration calls (`.x(...)`, `.y(...)`,
//! `.theme(...)`, `.add_comparisons(...)`) and then terminate with
//! a plot type (`.scatter()`, `.bar()`, `.boxplot()`,
//! `.mass_spectrum()`) which produces a renderable that
//! `.render_to_svg(path)` / `.render_to_png(path)` consume.
//!
//! Pairwise annotations are caller-owned display inputs. The renderer never
//! performs hypothesis tests or adjusts p-values.

mod accessors;
mod configuration;
mod facets;
mod layout;
mod selection;

use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::layout::AxisTick;
use crate::render::backend::PointOutline;
use crate::spec::{
    AxisScale, BarAggregation, BarSpacing, CategoricalColors, DumbbellEndpoints,
    EffectBandSemantics, EstimateSource, FacetScales, HeatmapAnnotations, HeatmapLabelFit,
    HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale, HistogramBins, HistogramComposite,
    HistogramStat, IntervalSemantics, LegendPosition, LineMode, LinePathStyle, NumericLabelFormat,
    PanelTextAnnotation, PlotKind, QqLine, RawOverlay, RegressionBand, ValueSort, ViolinLayout,
    WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{ComparisonAnnotation, HistogramCompositeGeometryInput, RaincloudGeometryInput};

/// The chart Builder. Holds a borrow of the input `DataFrame`
/// and accumulated configuration. Plot methods consume `self` and
/// return a renderable handle.
#[derive(Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent chart visibility and rendering switches are typed builder options"
)]
pub struct Chart<'a> {
    data: &'a polars::prelude::DataFrame,
    source_rows: Option<&'a [usize]>,
    x_col: Option<&'a str>,
    y_col: Option<&'a str>,
    hue_col: Option<&'a str>,
    group_col: Option<&'a str>,
    title: Option<String>,
    subtitle: Option<String>,
    caption: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    labels: Labels,
    x_axis_spec: AxisSpec,
    x_limits_user_explicit: bool,
    x_tick_count_overridden: bool,
    x_ticks: Option<Vec<AxisTick>>,
    y_axis_spec: AxisSpec,
    y_limits_user_explicit: bool,
    y_tick_count_overridden: bool,
    y_ticks: Option<Vec<AxisTick>>,
    theme: Theme,
    comparisons: Vec<ComparisonAnnotation>,
    grouped_factor_annotations: Vec<crate::GroupedFactorAnnotation>,
    show_non_significant: bool,
    significance_star_limit: crate::SignificanceStarLimit,
    width: u32,
    height: u32,
    /// Peak markers for `PlotKind::MassSpectrum`. Owned
    /// `(m/z, intensity)` pairs supplied via `add_peaks`.
    /// Empty for all other plot kinds.
    peaks: Vec<(f64, f64)>,
    /// Optional scatter points for `PlotKind::Marginal`.
    /// Owned `(x, y)` pairs supplied via `add_marginal_coords`.
    /// When non-empty, the `Marginal` plot uses these points
    /// instead of extracting from `x_col` / `y_col` in the
    /// `DataFrame` (the canonical use case: pre-computed
    /// UMAP coordinates for immune-cell discovery plots).
    marginal_coords: Vec<(f64, f64)>,
    /// Ordered variable names for `PlotKind::Matrix` (pairplot).
    matrix_variables: Vec<String>,
    bar_aggregation: BarAggregation,
    bar_spacing: BarSpacing,
    histogram_bins: HistogramBins,
    histogram_stat: HistogramStat,
    histogram_composite: HistogramComposite,
    histogram_composite_geometry: Option<HistogramCompositeGeometryInput>,
    qq_line: QqLine,
    point_outline: Option<PointOutline>,
    legend_position: LegendPosition,
    legend_visible: Option<bool>,
    color_col: Option<&'a str>,
    color_label: Option<String>,
    heatmap_scale: HeatmapScale,
    heatmap_layout: HeatmapLayout,
    heatmap_label_fit: HeatmapLabelFit,
    heatmap_color_limits: Option<(f64, f64)>,
    beeswarm_color_limits: Option<(f64, f64)>,
    heatmap_annotations: HeatmapAnnotations,
    heatmap_null_row_annotations: Vec<HeatmapNullRowAnnotation>,
    heatmap_highlight_col: Option<&'a str>,
    estimate_labels: Option<NumericLabelFormat>,
    coefficient_significance_col: Option<&'a str>,
    bar_significance_col: Option<&'a str>,
    significance_mark_style: crate::SignificanceMarkStyle,
    value_sort: ValueSort,
    color_by_sign: bool,
    zero_line: bool,
    interval_lower_col: Option<&'a str>,
    interval_upper_col: Option<&'a str>,
    interval_semantics: Option<IntervalSemantics>,
    estimate_status_col: Option<&'a str>,
    effect_band_semantics: Option<EffectBandSemantics>,
    markers_visible: bool,
    regression_band: RegressionBand,
    line_mode: LineMode,
    line_path_style: LinePathStyle,
    raw_overlay: RawOverlay,
    raw_overlay_source_row_col: Option<&'a str>,
    violin_layout: ViolinLayout,
    raincloud_geometry: Option<RaincloudGeometryInput>,
    reference_line: Option<f64>,
    axis_scale: AxisScale,
    interaction_source: Option<EstimateSource>,
    interaction_connectors: Option<bool>,
    facet_row_col: Option<&'a str>,
    facet_col_col: Option<&'a str>,
    facet_wrap_col: Option<&'a str>,
    facet_wrap_columns: Option<usize>,
    facet_scales: FacetScales,
    resolved_facet_x_range: Option<(f64, f64)>,
    global_hue_levels: Option<Vec<String>>,
    global_group_levels: Option<Vec<String>>,
    global_line_condition_levels: Option<Vec<String>>,
    interaction_style_cols: Option<(&'a str, &'a str)>,
    categorical_colors: Option<CategoricalColors>,
    render_scope: Option<String>,
    axis_text_measured: bool,
    composed_axis_text_measured: bool,
    y_tick_labels_visible: bool,
    waterfall_base_col: Option<&'a str>,
    waterfall_terminal_col: Option<&'a str>,
    waterfall_labels: Option<WaterfallLabels>,
    dumbbell_right_col: Option<&'a str>,
    dumbbell_endpoints: Option<DumbbellEndpoints>,
    guides: Vec<GuideSpec>,
    point_layers: Vec<PointLayer<'a>>,
    bar_point_layers: Vec<BarPointLayer<'a>>,
    panel_text_annotations: Vec<PanelTextAnnotation>,
    /// Which plot type the terminal call selected. `None` until
    /// `.scatter()` / `.bar()` / `.boxplot()` is invoked; used by
    /// `render_to_svg` / `render_to_png` to dispatch to the right
    /// `plots::*` impl.
    plot_kind: Option<PlotKind>,
}

impl<'a> Chart<'a> {
    /// Construct a new `Chart` borrowing `df`. The borrow is
    /// kept until the chart is rendered; this lets the rendering
    /// pipeline see the data without copying.
    #[expect(
        clippy::too_many_lines,
        reason = "one explicit initializer keeps every typed chart default auditable"
    )]
    pub fn new(df: &'a polars::prelude::DataFrame) -> Self {
        Self {
            data: df,
            source_rows: None,
            x_col: None,
            y_col: None,
            hue_col: None,
            group_col: None,
            title: None,
            subtitle: None,
            caption: None,
            x_label: None,
            y_label: None,
            labels: Labels::default(),
            x_axis_spec: AxisSpec::default(),
            x_limits_user_explicit: false,
            x_tick_count_overridden: false,
            x_ticks: None,
            y_axis_spec: AxisSpec::default(),
            y_limits_user_explicit: false,
            y_tick_count_overridden: false,
            y_ticks: None,
            theme: Theme::Publication,
            comparisons: Vec::new(),
            grouped_factor_annotations: Vec::new(),
            show_non_significant: false,
            significance_star_limit: crate::SignificanceStarLimit::default(),
            width: 640,
            height: 480,
            peaks: Vec::new(),
            marginal_coords: Vec::new(),
            matrix_variables: Vec::new(),
            bar_aggregation: BarAggregation::default(),
            bar_spacing: BarSpacing::default(),
            histogram_bins: HistogramBins::default(),
            histogram_stat: HistogramStat::default(),
            histogram_composite: HistogramComposite::default(),
            histogram_composite_geometry: None,
            qq_line: QqLine::default(),
            point_outline: Some(PointOutline::default()),
            legend_position: LegendPosition::Auto,
            legend_visible: None,
            color_col: None,
            color_label: None,
            heatmap_scale: HeatmapScale::default(),
            heatmap_layout: HeatmapLayout::default(),
            heatmap_label_fit: HeatmapLabelFit::default(),
            heatmap_color_limits: None,
            beeswarm_color_limits: None,
            heatmap_annotations: HeatmapAnnotations::default(),
            heatmap_null_row_annotations: Vec::new(),
            heatmap_highlight_col: None,
            estimate_labels: None,
            coefficient_significance_col: None,
            bar_significance_col: None,
            significance_mark_style: crate::SignificanceMarkStyle::default(),
            value_sort: ValueSort::None,
            color_by_sign: false,
            zero_line: false,
            interval_lower_col: None,
            interval_upper_col: None,
            interval_semantics: None,
            estimate_status_col: None,
            effect_band_semantics: None,
            markers_visible: true,
            regression_band: RegressionBand::default(),
            line_mode: LineMode::default(),
            line_path_style: LinePathStyle::default(),
            raw_overlay: RawOverlay::default(),
            raw_overlay_source_row_col: None,
            violin_layout: ViolinLayout::default(),
            raincloud_geometry: None,
            reference_line: None,
            axis_scale: AxisScale::Linear,
            interaction_source: None,
            interaction_connectors: None,
            facet_row_col: None,
            facet_col_col: None,
            facet_wrap_col: None,
            facet_wrap_columns: None,
            facet_scales: FacetScales::Fixed,
            resolved_facet_x_range: None,
            global_hue_levels: None,
            global_group_levels: None,
            global_line_condition_levels: None,
            interaction_style_cols: None,
            categorical_colors: None,
            render_scope: None,
            axis_text_measured: false,
            composed_axis_text_measured: false,
            y_tick_labels_visible: true,
            waterfall_base_col: None,
            waterfall_terminal_col: None,
            waterfall_labels: None,
            dumbbell_right_col: None,
            dumbbell_endpoints: None,
            guides: Vec::new(),
            point_layers: Vec::new(),
            bar_point_layers: Vec::new(),
            panel_text_annotations: Vec::new(),
            plot_kind: None,
        }
    }
}
