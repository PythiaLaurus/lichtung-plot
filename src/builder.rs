//! `lichtung-plot::builder` — the native `Plot` builder API.
//!
//! Semantic constructors accept immutable Rust-native values. Configuration is
//! chained before `.svg(path)` or `.png(path)` consumes the plot.
//!
//! Pairwise annotations are caller-owned display inputs. The renderer never
//! performs hypothesis tests or adjusts p-values.

mod accessors;
mod configuration;
mod facets;
mod layout;

use crate::input::{AxisValues, PlotInput};
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
use crate::{
    BooleanValues, CategoryValues, ComparisonAnnotation, HistogramCompositeGeometryInput,
    NumericValues, RaincloudGeometryInput, SourceRows,
};

/// One configured semantic plot backed by immutable Rust-native values.
#[derive(Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent chart visibility and rendering switches are typed builder options"
)]
pub struct Plot {
    input: PlotInput,
    source_rows: Option<SourceRows>,
    hue: Option<CategoryValues>,
    group: Option<CategoryValues>,
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
    color: Option<NumericValues>,
    color_label: Option<String>,
    heatmap_scale: HeatmapScale,
    heatmap_layout: HeatmapLayout,
    heatmap_label_fit: HeatmapLabelFit,
    heatmap_color_limits: Option<(f64, f64)>,
    beeswarm_color_limits: Option<(f64, f64)>,
    heatmap_annotations: HeatmapAnnotations,
    heatmap_null_row_annotations: Vec<HeatmapNullRowAnnotation>,
    heatmap_highlight: Option<BooleanValues>,
    estimate_labels: Option<NumericLabelFormat>,
    coefficient_significance: Option<CategoryValues>,
    bar_significance: Option<CategoryValues>,
    significance_mark_style: crate::SignificanceMarkStyle,
    value_sort: ValueSort,
    color_by_sign: bool,
    zero_line: bool,
    interval_lower: Option<NumericValues>,
    interval_upper: Option<NumericValues>,
    interval_semantics: Option<IntervalSemantics>,
    estimate_status: Option<CategoryValues>,
    effect_band_semantics: Option<EffectBandSemantics>,
    markers_visible: bool,
    regression_band: RegressionBand,
    line_mode: LineMode,
    line_path_style: LinePathStyle,
    raw_overlay: RawOverlay,
    raw_overlay_source_rows: Option<SourceRows>,
    violin_layout: ViolinLayout,
    raincloud_geometry: Option<RaincloudGeometryInput>,
    reference_line: Option<f64>,
    axis_scale: AxisScale,
    interaction_source: Option<EstimateSource>,
    interaction_connectors: Option<bool>,
    facet_row: Option<CategoryValues>,
    facet_col: Option<CategoryValues>,
    facet_wrap: Option<CategoryValues>,
    facet_wrap_columns: Option<usize>,
    facet_scales: FacetScales,
    resolved_facet_x_range: Option<(f64, f64)>,
    global_hue_levels: Option<Vec<String>>,
    global_group_levels: Option<Vec<String>>,
    global_line_condition_levels: Option<Vec<String>>,
    interaction_styles: Option<(CategoryValues, CategoryValues)>,
    categorical_colors: Option<CategoricalColors>,
    render_scope: Option<String>,
    axis_text_measured: bool,
    composed_axis_text_measured: bool,
    y_tick_labels_visible: bool,
    waterfall_base: Option<f64>,
    waterfall_terminal: Option<f64>,
    waterfall_labels: Option<WaterfallLabels>,
    dumbbell_right: Option<NumericValues>,
    dumbbell_endpoints: Option<DumbbellEndpoints>,
    guides: Vec<GuideSpec>,
    point_layers: Vec<PointLayer>,
    bar_point_layers: Vec<BarPointLayer>,
    panel_text_annotations: Vec<PanelTextAnnotation>,
    png_options: crate::PngOptions,
    svg_text: crate::SvgText,
    /// Plot family selected by the semantic constructor and used for
    /// validation and rendering dispatch.
    kind: Option<PlotKind>,
}

impl Plot {
    fn new(input: PlotInput, kind: PlotKind) -> Self {
        Self {
            input,
            source_rows: None,
            hue: None,
            group: None,
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
            color: None,
            color_label: None,
            heatmap_scale: HeatmapScale::default(),
            heatmap_layout: HeatmapLayout::default(),
            heatmap_label_fit: HeatmapLabelFit::default(),
            heatmap_color_limits: None,
            beeswarm_color_limits: None,
            heatmap_annotations: HeatmapAnnotations::default(),
            heatmap_null_row_annotations: Vec::new(),
            heatmap_highlight: None,
            estimate_labels: None,
            coefficient_significance: None,
            bar_significance: None,
            significance_mark_style: crate::SignificanceMarkStyle::default(),
            value_sort: ValueSort::None,
            color_by_sign: false,
            zero_line: false,
            interval_lower: None,
            interval_upper: None,
            interval_semantics: None,
            estimate_status: None,
            effect_band_semantics: None,
            markers_visible: true,
            regression_band: RegressionBand::default(),
            line_mode: LineMode::default(),
            line_path_style: LinePathStyle::default(),
            raw_overlay: RawOverlay::default(),
            raw_overlay_source_rows: None,
            violin_layout: ViolinLayout::default(),
            raincloud_geometry: None,
            reference_line: None,
            axis_scale: AxisScale::Linear,
            interaction_source: None,
            interaction_connectors: None,
            facet_row: None,
            facet_col: None,
            facet_wrap: None,
            facet_wrap_columns: None,
            facet_scales: FacetScales::Fixed,
            resolved_facet_x_range: None,
            global_hue_levels: None,
            global_group_levels: None,
            global_line_condition_levels: None,
            interaction_styles: None,
            categorical_colors: None,
            render_scope: None,
            axis_text_measured: false,
            composed_axis_text_measured: false,
            y_tick_labels_visible: true,
            waterfall_base: None,
            waterfall_terminal: None,
            waterfall_labels: None,
            dumbbell_right: None,
            dumbbell_endpoints: None,
            guides: Vec::new(),
            point_layers: Vec::new(),
            bar_point_layers: Vec::new(),
            panel_text_annotations: Vec::new(),
            png_options: crate::PngOptions::default(),
            svg_text: crate::SvgText::default(),
            kind: Some(kind),
        }
    }

    /// Construct a scatter plot.
    #[must_use]
    pub fn scatter(x: NumericValues, y: NumericValues) -> Self {
        Self::new(PlotInput::Scatter { x, y }, PlotKind::Scatter)
    }

    /// Construct a histogram.
    #[must_use]
    pub fn histogram(values: NumericValues) -> Self {
        Self::new(PlotInput::Histogram { values }, PlotKind::Histogram)
    }

    /// Construct a histogram from complete caller-computed geometry.
    #[must_use]
    pub fn histogram_geometry(input: HistogramCompositeGeometryInput) -> Self {
        let mut plot = Self::new(PlotInput::HistogramGeometry, PlotKind::Histogram);
        plot.histogram_stat = HistogramStat::Density;
        plot.histogram_composite_geometry = Some(input);
        plot
    }

    /// Construct a Gaussian density plot.
    #[must_use]
    pub fn density(values: NumericValues) -> Self {
        Self::new(PlotInput::Density { values }, PlotKind::Density)
    }

    /// Construct a normal Q-Q plot.
    #[must_use]
    pub fn qq(values: NumericValues) -> Self {
        Self::new(PlotInput::Qq { values }, PlotKind::Qq)
    }

    /// Construct a point-interval plot.
    #[must_use]
    pub fn point_interval(
        category: CategoryValues,
        estimate: NumericValues,
        lower: NumericValues,
        upper: NumericValues,
    ) -> Self {
        let mut plot = Self::new(
            PlotInput::PointInterval { estimate, category },
            PlotKind::PointInterval,
        );
        plot.interval_lower = Some(lower);
        plot.interval_upper = Some(upper);
        plot
    }

    /// Construct an ordered line plot.
    #[must_use]
    pub fn line(x: impl Into<AxisValues>, y: NumericValues) -> Self {
        Self::new(PlotInput::Line { x: x.into(), y }, PlotKind::Line)
    }

    /// Construct a precomputed interaction-profile plot.
    #[must_use]
    pub fn interaction(
        condition: CategoryValues,
        estimate: NumericValues,
        group: CategoryValues,
        lower: NumericValues,
        upper: NumericValues,
    ) -> Self {
        let mut plot = Self::new(
            PlotInput::Interaction {
                condition,
                estimate,
            },
            PlotKind::Interaction,
        );
        plot.group = Some(group);
        plot.interval_lower = Some(lower);
        plot.interval_upper = Some(upper);
        plot
    }

    /// Construct a model-coefficient plot.
    #[must_use]
    pub fn coefficient(term: CategoryValues, estimate: NumericValues) -> Self {
        Self::new(
            PlotInput::Coefficient { estimate, term },
            PlotKind::Coefficient,
        )
    }

    /// Construct a forest plot with complete intervals.
    #[must_use]
    pub fn forest(
        term: CategoryValues,
        estimate: NumericValues,
        lower: NumericValues,
        upper: NumericValues,
    ) -> Self {
        let mut plot = Self::new(PlotInput::Forest { estimate, term }, PlotKind::Forest);
        plot.interval_lower = Some(lower);
        plot.interval_upper = Some(upper);
        plot
    }

    /// Construct a horizontal paired-endpoint dumbbell plot.
    #[must_use]
    pub fn dumbbell(category: CategoryValues, left: NumericValues, right: NumericValues) -> Self {
        let mut plot = Self::new(PlotInput::Dumbbell { category, left }, PlotKind::Dumbbell);
        plot.dumbbell_right = Some(right);
        plot
    }

    /// Construct a categorical heatmap.
    #[must_use]
    pub fn heatmap(x: CategoryValues, y: CategoryValues, fill: NumericValues) -> Self {
        let mut plot = Self::new(PlotInput::Heatmap { x, y }, PlotKind::Heatmap);
        plot.color = Some(fill);
        plot
    }

    /// Construct a vertical bar plot.
    #[must_use]
    pub fn bar(category: CategoryValues, value: NumericValues) -> Self {
        Self::new(PlotInput::Bar { category, value }, PlotKind::Bar)
    }

    /// Construct a horizontal bar plot.
    #[must_use]
    pub fn horizontal_bar(category: CategoryValues, value: NumericValues) -> Self {
        Self::new(
            PlotInput::HorizontalBar { category, value },
            PlotKind::HorizontalBar,
        )
    }

    /// Construct a stacked vertical bar plot.
    #[must_use]
    pub fn stacked_bar(
        category: CategoryValues,
        value: NumericValues,
        group: CategoryValues,
    ) -> Self {
        let mut plot = Self::new(
            PlotInput::StackedBar { category, value },
            PlotKind::StackedBar,
        );
        plot.group = Some(group);
        plot
    }

    /// Construct a stacked horizontal bar plot.
    #[must_use]
    pub fn stacked_horizontal_bar(
        category: CategoryValues,
        value: NumericValues,
        group: CategoryValues,
    ) -> Self {
        let mut plot = Self::new(
            PlotInput::StackedHorizontalBar { category, value },
            PlotKind::StackedHorizontalBar,
        );
        plot.group = Some(group);
        plot
    }

    /// Construct a box-and-whisker plot.
    #[must_use]
    pub fn boxplot(category: CategoryValues, value: NumericValues) -> Self {
        Self::new(PlotInput::Boxplot { category, value }, PlotKind::Boxplot)
    }

    /// Construct a violin plot.
    #[must_use]
    pub fn violin(category: CategoryValues, value: NumericValues) -> Self {
        Self::new(PlotInput::Violin { category, value }, PlotKind::Violin)
    }

    /// Construct a raincloud plot from complete caller-computed geometry.
    #[must_use]
    pub fn raincloud(input: RaincloudGeometryInput) -> Self {
        let mut plot = Self::new(PlotInput::RaincloudGeometry, PlotKind::Violin);
        plot.violin_layout = ViolinLayout::Raincloud;
        plot.raincloud_geometry = Some(input);
        plot
    }

    /// Construct a regression plot.
    #[must_use]
    pub fn regression(x: NumericValues, y: NumericValues) -> Self {
        Self::new(PlotInput::Regression { x, y }, PlotKind::Regression)
    }

    /// Construct a marginal plot.
    #[must_use]
    pub fn marginal(x: NumericValues, y: NumericValues) -> Self {
        Self::new(PlotInput::Marginal { x, y }, PlotKind::Marginal)
    }

    /// Construct a scatter matrix from ordered named numeric variables.
    #[must_use]
    pub fn matrix(variables: Vec<NumericValues>) -> Self {
        let names = variables
            .iter()
            .map(|values| values.name().to_owned())
            .collect();
        let mut plot = Self::new(PlotInput::Matrix { variables }, PlotKind::Matrix);
        plot.matrix_variables = names;
        plot
    }

    /// Construct a categorical beeswarm plot with continuous color values.
    #[must_use]
    pub fn beeswarm(category: CategoryValues, value: NumericValues, color: NumericValues) -> Self {
        let mut plot = Self::new(PlotInput::Beeswarm { value, category }, PlotKind::Beeswarm);
        plot.color = Some(color);
        plot
    }

    /// Construct a cumulative signed-delta waterfall plot.
    #[must_use]
    pub fn waterfall(
        labels: CategoryValues,
        deltas: NumericValues,
        base: f64,
        terminal: f64,
    ) -> Self {
        let mut plot = Self::new(PlotInput::Waterfall { labels, deltas }, PlotKind::Waterfall);
        plot.waterfall_base = Some(base);
        plot.waterfall_terminal = Some(terminal);
        plot
    }
}
