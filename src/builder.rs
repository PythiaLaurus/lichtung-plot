//! `lichtung-plot::builder` — the native `Plot` builder API.
//!
//! Semantic constructors accept immutable Rust-native values. Configuration is
//! chained before `.svg(path)` or `.png(path)` consumes the plot.
//!
//! Significance annotations are caller-owned display inputs. The renderer never
//! performs hypothesis tests or adjusts p-values.

use std::marker::PhantomData;
use std::sync::Arc;

mod accessors;
mod configuration;
mod facets;
mod layout;

use crate::input::{AxisValues, PlotInput};
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::render::backend::PointOutline;
use crate::spec::{
    AxisScale, BarSpacing, CategoricalColors, CategoryOrder, DumbbellEndpoints, FacetSpec,
    HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale,
    HistogramBins, HistogramComposite, HistogramStat, LegendPolicy, LinePathStyle,
    NumericLabelFormat, PanelTextAnnotation, PlotKind, QqLine, RawOverlay, RegressionBand,
    WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{
    BarInput, BooleanValues, Bounds, CategoryValues, CoefficientInput, HistogramInput,
    NumericValues, RaincloudInput, SourceRows,
};

/// One configured semantic plot backed by immutable Rust-native values.
///
/// `K` is a zero-sized plot-family marker from [`crate::kind`]. Omitting it
/// means [`crate::kind::Any`], the erased form used by heterogeneous storage.
/// Family-specific methods are available only through exact-marker inherent
/// implementations or a corresponding sealed capability.
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn removed_identity_alias(plot: &Plot<kind::Scatter>) {
///     let _ = plot.plot_kind();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn internal_family_options(plot: &Plot<kind::Histogram>) {
///     let _ = plot.histogram_bins_spec();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn internal_renderer_configuration(plot: &Plot<kind::Scatter>) {
///     let _ = plot.x_axis_spec();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn internal_palette(plot: &Plot<kind::Scatter>) {
///     let _ = plot.palette();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn internal_layout(plot: &Plot<kind::Scatter>) {
///     let _ = plot.canvas_layout();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{NumericValues, Plot};
/// let plot = Plot::scatter(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.x_tick_count(3);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{NumericValues, Plot};
/// let plot = Plot::scatter(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.y_tick_count(3);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{HistogramBins, NumericValues, Plot};
/// let plot = Plot::scatter(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.bins(HistogramBins::Count(4));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{HistogramBins, HistogramInput, NumericValues, Plot};
/// let plot = Plot::histogram(HistogramInput::samples(
///     NumericValues::new("value", [1.0]),
/// )).erase();
/// let _ = plot.bins(HistogramBins::Count(4));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{BarInput, CategoryValues, HeatmapScale, NumericValues, Plot};
/// let plot = Plot::bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     BarInput::estimates(NumericValues::new("value", [1.0])),
/// );
/// let _ = plot.scale(HeatmapScale::Sequential);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{HistogramInput, NumericValues, Plot, RawOverlay};
/// let plot = Plot::histogram(HistogramInput::samples(NumericValues::new("value", [1.0])));
/// let _ = plot.raw_overlay(RawOverlay::Points);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{Bounds, NumericValues, Plot};
/// let plot = Plot::regression(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let bounds = Bounds::new(
///     NumericValues::new("lower", [0.0]),
///     NumericValues::new("upper", [3.0]),
/// );
/// let _ = plot.band(bounds);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{NumericValues, Plot, Significance, SignificanceMark};
/// let plot = Plot::scatter(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.significance([
///     Significance::at("a", SignificanceMark::OneStar),
/// ]);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{NumericValues, Plot, Significance, SignificanceMark};
/// let plot = Plot::scatter(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.significance([
///     Significance::between("a", "b", SignificanceMark::OneStar),
/// ]);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot, Significance, SignificanceMark};
/// let plot = Plot::boxplot(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
/// );
/// let _ = plot.significance([
///     Significance::at("a", SignificanceMark::OneStar),
/// ]);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, CoefficientInput, NumericValues, Plot, Significance, SignificanceMark};
/// let plot = Plot::coefficient(
///     CategoryValues::new("term", ["a", "b"]).unwrap(),
///     CoefficientInput::estimates(NumericValues::new("estimate", [1.0, 2.0])),
/// );
/// let _ = plot.significance([
///     Significance::between("a", "b", SignificanceMark::OneStar),
/// ]);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, CoefficientInput, NumericValues, Plot};
/// struct Wrapped(CoefficientInput);
/// impl Into<CoefficientInput> for Wrapped {
///     fn into(self) -> CoefficientInput { self.0 }
/// }
/// let _ = Plot::coefficient(
///     CategoryValues::new("term", ["a"]).unwrap(),
///     Wrapped(CoefficientInput::estimates(NumericValues::new("estimate", [1.0]))),
/// );
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{Bounds, CategoryValues, NumericValues, Plot};
/// let plot = Plot::trajectory(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("subject", ["a", "a"]).unwrap(),
/// );
/// let _ = plot.band(Bounds::new(
///     NumericValues::new("lower", [1.0, 2.0]),
///     NumericValues::new("upper", [3.0, 4.0]),
/// ));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::trajectory(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("subject", ["a", "a"]).unwrap(),
/// );
/// let _ = plot.markers(false);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::trajectory(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("subject", ["a", "a"]).unwrap(),
/// );
/// let _ = plot.group(CategoryValues::new("replacement", ["b", "b"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{Bounds, CategoryValues, NumericValues, Plot};
/// let plot = Plot::line_path(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("path", ["a", "a"]).unwrap(),
/// );
/// let _ = plot.band(Bounds::new(
///     NumericValues::new("lower", [1.0, 2.0]),
///     NumericValues::new("upper", [3.0, 4.0]),
/// ));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot, PointLayer};
/// let plot = Plot::line_path(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("path", ["a", "a"]).unwrap(),
/// );
/// let layer = PointLayer::new(
///     NumericValues::new("x", [1.5]),
///     NumericValues::new("y", [2.5]),
/// );
/// let _ = plot.point_layer(layer);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::line_path(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
///     CategoryValues::new("path", ["a", "a"]).unwrap(),
/// );
/// let _ = plot.group(CategoryValues::new("replacement", ["b", "b"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{Bounds, CategoryValues, NumericValues, Plot};
/// let plot = Plot::interaction(
///     CategoryValues::new("condition", ["before", "after"]).unwrap(),
///     NumericValues::new("estimate", [1.0, 2.0]),
///     CategoryValues::new("group", ["a", "a"]).unwrap(),
///     Bounds::new(
///         NumericValues::new("lower", [0.5, 1.5]),
///         NumericValues::new("upper", [1.5, 2.5]),
///     ),
/// );
/// let _ = plot.group(CategoryValues::new("replacement", ["b", "b"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{LinePathStyle, NumericValues, Plot};
/// let plot = Plot::line(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
/// );
/// let _ = plot.style(LinePathStyle::Hue);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let _ = Plot::line(
///     CategoryValues::new("x", ["before", "after"]).unwrap(),
///     NumericValues::new("y", [1.0, 2.0]),
/// );
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let _ = Plot::dumbbell(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("left", [1.0]),
///     NumericValues::new("right", [2.0]),
/// );
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{BarSpacing, CategoryValues, NumericValues, Plot};
/// let plot = Plot::stacked_bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
///     CategoryValues::new("group", ["g"]).unwrap(),
/// );
/// let _ = plot.spacing(BarSpacing::Compact);
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::stacked_bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
///     CategoryValues::new("series", ["g"]).unwrap(),
/// );
/// let _ = plot.series(CategoryValues::new("replacement", ["h"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::stacked_horizontal_bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
///     CategoryValues::new("series", ["g"]).unwrap(),
/// );
/// let _ = plot.series(CategoryValues::new("replacement", ["h"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{BarInput, CategoryValues, NumericValues, Plot};
/// let plot = Plot::bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     BarInput::estimates(NumericValues::new("value", [1.0])),
/// );
/// let _ = plot.group(CategoryValues::new("series", ["g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::horizontal_bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     lichtung_plot::BarInput::estimates(NumericValues::new("value", [1.0])),
/// );
/// let _ = plot.hue(CategoryValues::new("series", ["g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::boxplot(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
/// );
/// let _ = plot.group(CategoryValues::new("series", ["g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::violin(
///     CategoryValues::new("category", ["a", "a"]).unwrap(),
///     NumericValues::new("value", [1.0, 2.0]),
/// );
/// let _ = plot.hue(CategoryValues::new("series", ["g", "g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot, RaincloudInput};
/// let plot = Plot::raincloud(RaincloudInput::samples(
///     CategoryValues::new("category", ["a", "a"]).unwrap(),
///     NumericValues::new("value", [1.0, 2.0]),
/// ));
/// let _ = plot.hue(CategoryValues::new("series", ["g", "g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::line(
///     NumericValues::new("x", [1.0, 2.0]),
///     NumericValues::new("y", [2.0, 3.0]),
/// );
/// let _ = plot.series(CategoryValues::new("series", ["g", "g"]).unwrap());
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::heatmap(
///     CategoryValues::new("x", ["a"]).unwrap(),
///     CategoryValues::new("y", ["b"]).unwrap(),
///     NumericValues::new("fill", [1.0]),
/// );
/// let _ = plot.color(NumericValues::new("replacement", [2.0]));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, NumericValues, Plot};
/// let plot = Plot::beeswarm(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     NumericValues::new("value", [1.0]),
///     NumericValues::new("color", [2.0]),
/// );
/// let _ = plot.color(NumericValues::new("replacement", [3.0]));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, FacetSpec, NumericValues, Plot};
/// let plot = Plot::matrix(vec![
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// ]);
/// let _ = plot.facet(FacetSpec::column(
///     CategoryValues::new("panel", ["a"]).unwrap(),
/// ));
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{CategoryValues, FacetSpec, NumericValues, Plot};
/// let plot = Plot::marginal(
///     NumericValues::new("x", [1.0]),
///     NumericValues::new("y", [2.0]),
/// );
/// let _ = plot.facet(FacetSpec::column(
///     CategoryValues::new("panel", ["a"]).unwrap(),
/// ));
/// ```
#[derive(Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent chart visibility and rendering switches are typed builder options"
)]
pub struct Plot<K: crate::kind::Marker = crate::kind::Any> {
    input: PlotInput,
    source_rows: Option<SourceRows>,
    series: Option<CategoryValues>,
    hue: Option<CategoryValues>,
    group: Option<CategoryValues>,
    title: Option<String>,
    subtitle: Option<String>,
    pub(crate) title_alignment: Option<crate::theme::TitleAlignment>,
    pub(crate) subtitle_alignment: Option<crate::theme::TitleAlignment>,
    caption: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    labels: Labels,
    x_axis_spec: AxisSpec,
    x_limits_user_explicit: bool,
    y_axis_spec: AxisSpec,
    y_limits_user_explicit: bool,
    theme: Option<Theme>,
    significance: Vec<crate::significance::SignificanceAnnotation>,
    width: u32,
    height: u32,
    /// Ordered variable names for `PlotKind::Matrix` (pairplot).
    matrix_variables: Vec<String>,
    bar_spacing: BarSpacing,
    histogram_bins: HistogramBins,
    histogram_stat: HistogramStat,
    histogram_composite: HistogramComposite,
    qq_line: QqLine,
    point_outline: Option<PointOutline>,
    legend_policy: LegendPolicy,
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
    significance_mark_style: Option<crate::SignificanceMarkStyle>,
    category_orders: CategoryOrders,
    color_by_sign: bool,
    zero_line: bool,
    line_band: Option<Bounds>,
    estimate_status: Option<Arc<[crate::EstimateStatus]>>,
    markers_visible: bool,
    regression_band: RegressionBand,
    line_path_style: LinePathStyle,
    raw_overlay: RawOverlay,
    coefficient_reference_line: Option<f64>,
    forest_reference: f64,
    axis_scale: AxisScale,
    interaction_connectors: Option<bool>,
    facet: Option<FacetSpec>,
    resolved_facet_x_range: Option<(f64, f64)>,
    global_hue_levels: Option<Vec<String>>,
    global_group_levels: Option<Vec<String>>,
    global_collective_pairs: Option<Vec<crate::data::CollectivePair>>,
    global_line_condition_levels: Option<Vec<String>>,
    interaction_styles: Option<Vec<crate::InteractionStyle>>,
    categorical_colors: Option<CategoricalColors>,
    render_scope: Option<String>,
    pub(crate) prepared_axes:
        std::sync::Arc<Vec<(Option<String>, crate::plots::shared_x::PreparedAxes)>>,
    pub(crate) prepared_matrix_axes:
        Option<std::sync::Arc<crate::plots::matrix::PreparedMatrixAxes>>,
    axis_gutter_owner: AxisGutterOwner,
    y_tick_labels_visible: bool,
    waterfall_labels: Option<WaterfallLabels>,
    guides: Vec<GuideSpec>,
    point_layers: Vec<PointLayer>,
    bar_point_layers: Vec<BarPointLayer>,
    panel_text_annotations: Vec<PanelTextAnnotation>,
    /// Plot family selected by the semantic constructor and used for
    /// validation and rendering dispatch.
    kind: PlotKind,
    marker: PhantomData<K>,
}

#[derive(Clone, Default)]
struct CategoryOrders {
    primary: CategoryOrder,
    x: CategoryOrder,
    y: CategoryOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisGutterOwner {
    Root,
    Composition,
}

/// Only the values that parent coordination may replace for a fixed child owner.
pub(crate) struct ComposedInputs {
    x_axis_spec: AxisSpec,
    y_axis_spec: AxisSpec,
    resolved_facet_x_range: Option<(f64, f64)>,
    global_hue_levels: Option<Vec<String>>,
    global_group_levels: Option<Vec<String>>,
    global_collective_pairs: Option<Vec<crate::data::CollectivePair>>,
    global_line_condition_levels: Option<Vec<String>>,
    axis_gutter_owner: AxisGutterOwner,
    y_tick_labels_visible: bool,
    render_scope: Option<String>,
    legend_policy: LegendPolicy,
    width: u32,
    height: u32,
}

impl Plot<crate::kind::Any> {
    fn new<K: crate::kind::Marker>(input: PlotInput, kind: PlotKind) -> Plot<K> {
        Plot {
            input,
            source_rows: None,
            series: None,
            hue: None,
            group: None,
            title: None,
            subtitle: None,
            title_alignment: None,
            subtitle_alignment: None,
            caption: None,
            x_label: None,
            y_label: None,
            labels: Labels::default(),
            x_axis_spec: AxisSpec::default(),
            x_limits_user_explicit: false,
            y_axis_spec: AxisSpec::default(),
            y_limits_user_explicit: false,
            theme: None,
            significance: Vec::new(),
            width: 640,
            height: 480,
            matrix_variables: Vec::new(),
            bar_spacing: BarSpacing::default(),
            histogram_bins: HistogramBins::default(),
            histogram_stat: HistogramStat::default(),
            histogram_composite: HistogramComposite::default(),
            qq_line: QqLine::default(),
            point_outline: Some(PointOutline::default()),
            legend_policy: LegendPolicy::Automatic,
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
            significance_mark_style: None,
            category_orders: CategoryOrders::default(),
            color_by_sign: false,
            zero_line: false,
            line_band: None,
            estimate_status: None,
            markers_visible: true,
            regression_band: RegressionBand::default(),
            line_path_style: LinePathStyle::default(),
            raw_overlay: RawOverlay::default(),
            coefficient_reference_line: None,
            forest_reference: 0.0,
            axis_scale: AxisScale::Linear,
            interaction_connectors: None,
            facet: None,
            resolved_facet_x_range: None,
            global_hue_levels: None,
            global_group_levels: None,
            global_collective_pairs: None,
            global_line_condition_levels: None,
            interaction_styles: None,
            categorical_colors: None,
            render_scope: None,
            prepared_axes: std::sync::Arc::new(Vec::new()),
            prepared_matrix_axes: None,
            axis_gutter_owner: AxisGutterOwner::Root,
            y_tick_labels_visible: true,
            waterfall_labels: None,
            guides: Vec::new(),
            point_layers: Vec::new(),
            bar_point_layers: Vec::new(),
            panel_text_annotations: Vec::new(),
            kind,
            marker: PhantomData,
        }
    }

    /// Construct a scatter plot.
    #[must_use]
    pub fn scatter(x: NumericValues, y: NumericValues) -> Plot<crate::kind::Scatter> {
        Self::new(PlotInput::Scatter { x, y }, PlotKind::Scatter)
    }

    /// Construct a histogram.
    #[must_use]
    pub fn histogram(input: HistogramInput) -> Plot<crate::kind::Histogram> {
        let geometry = input.geometry_input().is_some();
        let mut plot = Self::new(PlotInput::Histogram { input }, PlotKind::Histogram);
        if geometry {
            plot.histogram_stat = HistogramStat::Density;
        }
        plot
    }

    /// Construct a Gaussian density plot.
    #[must_use]
    pub fn density(values: NumericValues) -> Plot<crate::kind::Density> {
        Self::new(PlotInput::Density { values }, PlotKind::Density)
    }

    /// Construct a normal Q-Q plot.
    #[must_use]
    pub fn qq(values: NumericValues) -> Plot<crate::kind::Qq> {
        Self::new(PlotInput::Qq { values }, PlotKind::Qq)
    }

    /// Construct a point-interval plot.
    #[must_use]
    pub fn point_interval(
        category: CategoryValues,
        estimate: NumericValues,
        bounds: Bounds,
    ) -> Plot<crate::kind::PointInterval> {
        Self::new(
            PlotInput::PointInterval {
                estimate,
                category,
                bounds,
            },
            PlotKind::PointInterval,
        )
    }

    /// Construct an ordered line plot.
    #[must_use]
    pub fn line(x: NumericValues, y: NumericValues) -> Plot<crate::kind::Line> {
        Self::new(
            PlotInput::Line {
                x: AxisValues::Numeric(x),
                y,
            },
            PlotKind::Line,
        )
    }

    /// Construct explicit-group repeated trajectories.
    #[must_use]
    pub fn trajectory(
        x: impl Into<AxisValues>,
        y: NumericValues,
        group: CategoryValues,
    ) -> Plot<crate::kind::Trajectory> {
        Self::new(
            PlotInput::Trajectory {
                x: x.into(),
                y,
                group,
            },
            PlotKind::Trajectory,
        )
    }

    /// Construct explicit-group numeric paths in source-row order.
    #[must_use]
    pub fn line_path(
        x: NumericValues,
        y: NumericValues,
        group: CategoryValues,
    ) -> Plot<crate::kind::LinePath> {
        Self::new(PlotInput::LinePath { x, y, group }, PlotKind::LinePath)
    }

    /// Construct a precomputed interaction-profile plot.
    #[must_use]
    pub fn interaction(
        condition: CategoryValues,
        estimate: NumericValues,
        group: CategoryValues,
        bounds: Bounds,
    ) -> Plot<crate::kind::Interaction> {
        Self::new(
            PlotInput::Interaction {
                condition,
                estimate,
                group,
                bounds,
            },
            PlotKind::Interaction,
        )
    }

    /// Construct a model-coefficient plot.
    #[must_use]
    pub fn coefficient(
        term: CategoryValues,
        input: CoefficientInput,
    ) -> Plot<crate::kind::Coefficient> {
        Self::new(
            PlotInput::Coefficient { input, term },
            PlotKind::Coefficient,
        )
    }

    /// Construct a forest plot with complete bounds and a scientific reference.
    #[must_use]
    pub fn forest(
        term: CategoryValues,
        estimate: NumericValues,
        bounds: Bounds,
        reference: f64,
    ) -> Plot<crate::kind::Forest> {
        let mut plot = Self::new(
            PlotInput::Forest {
                estimate,
                term,
                bounds,
            },
            PlotKind::Forest,
        );
        plot.forest_reference = reference;
        plot
    }

    /// Construct a horizontal paired-endpoint dumbbell plot.
    #[must_use]
    pub fn dumbbell(
        category: CategoryValues,
        left: NumericValues,
        right: NumericValues,
        endpoints: DumbbellEndpoints,
    ) -> Plot<crate::kind::Dumbbell> {
        Self::new(
            PlotInput::Dumbbell {
                category,
                left,
                right,
                endpoints,
            },
            PlotKind::Dumbbell,
        )
    }

    /// Construct a categorical heatmap.
    #[must_use]
    pub fn heatmap(
        x: CategoryValues,
        y: CategoryValues,
        fill: NumericValues,
    ) -> Plot<crate::kind::Heatmap> {
        Self::new(PlotInput::Heatmap { x, y, fill }, PlotKind::Heatmap)
    }

    /// Construct a vertical bar plot.
    #[must_use]
    pub fn bar(category: CategoryValues, input: impl Into<BarInput>) -> Plot<crate::kind::Bar> {
        Self::new(
            PlotInput::Bar {
                category,
                input: input.into(),
            },
            PlotKind::Bar,
        )
    }

    /// Construct a horizontal bar plot.
    #[must_use]
    pub fn horizontal_bar(
        category: CategoryValues,
        input: impl Into<BarInput>,
    ) -> Plot<crate::kind::HorizontalBar> {
        Self::new(
            PlotInput::HorizontalBar {
                category,
                input: input.into(),
            },
            PlotKind::HorizontalBar,
        )
    }

    /// Construct a stacked vertical bar plot.
    #[must_use]
    pub fn stacked_bar(
        category: CategoryValues,
        value: NumericValues,
        series: CategoryValues,
    ) -> Plot<crate::kind::StackedBar> {
        Self::new(
            PlotInput::StackedBar {
                category,
                value,
                series,
            },
            PlotKind::StackedBar,
        )
    }

    /// Construct a stacked horizontal bar plot.
    #[must_use]
    pub fn stacked_horizontal_bar(
        category: CategoryValues,
        value: NumericValues,
        series: CategoryValues,
    ) -> Plot<crate::kind::StackedHorizontalBar> {
        Self::new(
            PlotInput::StackedHorizontalBar {
                category,
                value,
                series,
            },
            PlotKind::StackedHorizontalBar,
        )
    }

    /// Construct a box-and-whisker plot.
    #[must_use]
    pub fn boxplot(category: CategoryValues, value: NumericValues) -> Plot<crate::kind::Boxplot> {
        Self::new(PlotInput::Boxplot { category, value }, PlotKind::Boxplot)
    }

    /// Construct a violin plot.
    #[must_use]
    pub fn violin(category: CategoryValues, value: NumericValues) -> Plot<crate::kind::Violin> {
        Self::new(PlotInput::Violin { category, value }, PlotKind::Violin)
    }

    /// Construct a raincloud plot.
    #[must_use]
    pub fn raincloud(input: RaincloudInput) -> Plot<crate::kind::Raincloud> {
        Self::new(PlotInput::Raincloud { input }, PlotKind::Raincloud)
    }

    /// Construct a regression plot.
    #[must_use]
    pub fn regression(x: NumericValues, y: NumericValues) -> Plot<crate::kind::Regression> {
        Self::new(PlotInput::Regression { x, y }, PlotKind::Regression)
    }

    /// Construct a marginal plot.
    #[must_use]
    pub fn marginal(x: NumericValues, y: NumericValues) -> Plot<crate::kind::Marginal> {
        Self::new(PlotInput::Marginal { x, y }, PlotKind::Marginal)
    }

    /// Construct a scatter matrix from ordered named numeric variables.
    #[must_use]
    pub fn matrix(variables: Vec<NumericValues>) -> Plot<crate::kind::Matrix> {
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
    pub fn beeswarm(
        category: CategoryValues,
        value: NumericValues,
        color: NumericValues,
    ) -> Plot<crate::kind::Beeswarm> {
        Self::new(
            PlotInput::Beeswarm {
                value,
                category,
                color,
            },
            PlotKind::Beeswarm,
        )
    }

    /// Construct a cumulative signed-delta waterfall plot.
    #[must_use]
    pub fn waterfall(
        labels: CategoryValues,
        deltas: NumericValues,
        base: f64,
    ) -> Plot<crate::kind::Waterfall> {
        Self::new(
            PlotInput::Waterfall {
                labels,
                deltas,
                base,
            },
            PlotKind::Waterfall,
        )
    }
}

impl<K: crate::kind::ExactFamily> Plot<K> {
    /// Consume this plot and erase its compile-time family marker.
    ///
    /// Every owned input and configuration field is moved unchanged.
    ///
    /// An erased plot cannot be erased a second time:
    ///
    /// ```compile_fail
    /// use lichtung_plot::{NumericValues, Plot};
    /// let erased: Plot = Plot::scatter(
    ///     NumericValues::new("x", [1.0]),
    ///     NumericValues::new("y", [2.0]),
    /// ).erase();
    /// let _ = erased.erase();
    /// ```
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "explicit full-field reconstruction makes marker erasure mechanically complete"
    )]
    pub fn erase(self) -> Plot<crate::kind::Any> {
        let Plot {
            input,
            source_rows,
            series,
            hue,
            group,
            title,
            subtitle,
            caption,
            x_label,
            y_label,
            labels,
            x_axis_spec,
            x_limits_user_explicit,
            y_axis_spec,
            y_limits_user_explicit,
            theme,
            significance,
            width,
            height,
            matrix_variables,
            bar_spacing,
            histogram_bins,
            histogram_stat,
            histogram_composite,
            qq_line,
            point_outline,
            legend_policy,
            color_label,
            heatmap_scale,
            heatmap_layout,
            heatmap_label_fit,
            heatmap_color_limits,
            beeswarm_color_limits,
            heatmap_annotations,
            heatmap_null_row_annotations,
            heatmap_highlight,
            estimate_labels,
            significance_mark_style,
            category_orders,
            color_by_sign,
            zero_line,
            line_band,
            estimate_status,
            markers_visible,
            regression_band,
            line_path_style,
            raw_overlay,
            coefficient_reference_line,
            forest_reference,
            axis_scale,
            interaction_connectors,
            facet,
            resolved_facet_x_range,
            global_hue_levels,
            global_group_levels,
            global_collective_pairs,
            global_line_condition_levels,
            interaction_styles,
            categorical_colors,
            render_scope,
            axis_gutter_owner,
            y_tick_labels_visible,
            waterfall_labels,
            guides,
            point_layers,
            bar_point_layers,
            panel_text_annotations,
            kind,
            marker: _,
            title_alignment,
            subtitle_alignment,
            prepared_axes,
            prepared_matrix_axes,
        } = self;
        Plot {
            prepared_axes,
            prepared_matrix_axes,
            title_alignment,
            subtitle_alignment,
            input,
            source_rows,
            series,
            hue,
            group,
            title,
            subtitle,
            caption,
            x_label,
            y_label,
            labels,
            x_axis_spec,
            x_limits_user_explicit,
            y_axis_spec,
            y_limits_user_explicit,
            theme,
            significance,
            width,
            height,
            matrix_variables,
            bar_spacing,
            histogram_bins,
            histogram_stat,
            histogram_composite,
            qq_line,
            point_outline,
            legend_policy,
            color_label,
            heatmap_scale,
            heatmap_layout,
            heatmap_label_fit,
            heatmap_color_limits,
            beeswarm_color_limits,
            heatmap_annotations,
            heatmap_null_row_annotations,
            heatmap_highlight,
            estimate_labels,
            significance_mark_style,
            category_orders,
            color_by_sign,
            zero_line,
            line_band,
            estimate_status,
            markers_visible,
            regression_band,
            line_path_style,
            raw_overlay,
            coefficient_reference_line,
            forest_reference,
            axis_scale,
            interaction_connectors,
            facet,
            resolved_facet_x_range,
            global_hue_levels,
            global_group_levels,
            global_collective_pairs,
            global_line_condition_levels,
            interaction_styles,
            categorical_colors,
            render_scope,
            axis_gutter_owner,
            y_tick_labels_visible,
            waterfall_labels,
            guides,
            point_layers,
            bar_point_layers,
            panel_text_annotations,
            kind,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{HistogramDimensions, RaincloudDimensions};
    use crate::Significance;

    fn numeric(name: &str) -> NumericValues {
        NumericValues::new(name, [1.0])
    }

    fn category(name: &str) -> CategoryValues {
        CategoryValues::new(name, ["a"]).unwrap()
    }

    fn bounds() -> Bounds {
        Bounds::new(numeric("lower"), numeric("upper"))
    }

    fn assert_marker<K: crate::kind::Marker>(_: Plot<K>) {}

    fn assert_constructor<K: crate::kind::ExactFamily>(plot: Plot<K>, expected_kind: PlotKind) {
        assert_eq!(plot.kind(), expected_kind);
        assert_eq!(plot.theme_spec(), None);
        let erased = plot.erase();
        assert_eq!(erased.kind(), expected_kind);
        assert_eq!(erased.theme_spec(), None);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive constructor inventory verifies every exact family marker"
    )]
    fn constructors_return_exact_family_markers() {
        assert_constructor::<crate::kind::Scatter>(
            Plot::scatter(numeric("x"), numeric("y")),
            PlotKind::Scatter,
        );
        assert_constructor::<crate::kind::Histogram>(
            Plot::histogram(HistogramInput::samples(numeric("value"))),
            PlotKind::Histogram,
        );
        assert_constructor::<crate::kind::Histogram>(
            Plot::histogram(HistogramInput::geometry(crate::HistogramGeometry {
                dimensions: HistogramDimensions {
                    value: "value".into(),
                    density: "density".into(),
                },
                bins: Vec::new(),
                kde: Vec::new(),
                rugs: Vec::new(),
            })),
            PlotKind::Histogram,
        );
        assert_constructor::<crate::kind::Density>(
            Plot::density(numeric("value")),
            PlotKind::Density,
        );
        assert_constructor::<crate::kind::Qq>(Plot::qq(numeric("value")), PlotKind::Qq);
        assert_constructor::<crate::kind::PointInterval>(
            Plot::point_interval(category("category"), numeric("estimate"), bounds()),
            PlotKind::PointInterval,
        );
        assert_constructor::<crate::kind::Line>(
            Plot::line(numeric("x"), numeric("y")),
            PlotKind::Line,
        );
        assert_constructor::<crate::kind::Trajectory>(
            Plot::trajectory(numeric("x"), numeric("y"), category("group")),
            PlotKind::Trajectory,
        );
        assert_constructor::<crate::kind::LinePath>(
            Plot::line_path(numeric("x"), numeric("y"), category("group")),
            PlotKind::LinePath,
        );
        assert_constructor::<crate::kind::Interaction>(
            Plot::interaction(
                category("condition"),
                numeric("estimate"),
                category("group"),
                bounds(),
            ),
            PlotKind::Interaction,
        );
        assert_constructor::<crate::kind::Coefficient>(
            Plot::coefficient(
                category("term"),
                CoefficientInput::estimates(numeric("estimate")),
            ),
            PlotKind::Coefficient,
        );
        assert_constructor::<crate::kind::Forest>(
            Plot::forest(category("term"), numeric("estimate"), bounds(), 0.0),
            PlotKind::Forest,
        );
        assert_constructor::<crate::kind::Dumbbell>(
            Plot::dumbbell(
                category("category"),
                numeric("left"),
                numeric("right"),
                DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
            ),
            PlotKind::Dumbbell,
        );
        assert_constructor::<crate::kind::Heatmap>(
            Plot::heatmap(category("x"), category("y"), numeric("fill")),
            PlotKind::Heatmap,
        );
        assert_constructor::<crate::kind::Bar>(
            Plot::bar(category("category"), BarInput::estimates(numeric("value"))),
            PlotKind::Bar,
        );
        assert_constructor::<crate::kind::HorizontalBar>(
            Plot::horizontal_bar(category("category"), BarInput::estimates(numeric("value"))),
            PlotKind::HorizontalBar,
        );
        assert_constructor::<crate::kind::StackedBar>(
            Plot::stacked_bar(category("category"), numeric("value"), category("group")),
            PlotKind::StackedBar,
        );
        assert_constructor::<crate::kind::StackedHorizontalBar>(
            Plot::stacked_horizontal_bar(category("category"), numeric("value"), category("group")),
            PlotKind::StackedHorizontalBar,
        );
        assert_constructor::<crate::kind::Boxplot>(
            Plot::boxplot(category("category"), numeric("value")),
            PlotKind::Boxplot,
        );
        assert_constructor::<crate::kind::Violin>(
            Plot::violin(category("category"), numeric("value")),
            PlotKind::Violin,
        );
        assert_constructor::<crate::kind::Raincloud>(
            Plot::raincloud(RaincloudInput::samples(
                category("category"),
                numeric("value"),
            )),
            PlotKind::Raincloud,
        );
        assert_constructor::<crate::kind::Raincloud>(
            Plot::raincloud(RaincloudInput::geometry(crate::RaincloudGeometry {
                dimensions: RaincloudDimensions {
                    category: "category".into(),
                    value: "value".into(),
                    hue: None,
                },
                groups: Vec::new(),
            })),
            PlotKind::Raincloud,
        );
        assert_constructor::<crate::kind::Regression>(
            Plot::regression(numeric("x"), numeric("y")),
            PlotKind::Regression,
        );
        assert_constructor::<crate::kind::Marginal>(
            Plot::marginal(numeric("x"), numeric("y")),
            PlotKind::Marginal,
        );
        assert_constructor::<crate::kind::Matrix>(
            Plot::matrix(vec![numeric("x"), numeric("y")]),
            PlotKind::Matrix,
        );
        assert_constructor::<crate::kind::Beeswarm>(
            Plot::beeswarm(category("category"), numeric("value"), numeric("color")),
            PlotKind::Beeswarm,
        );
        assert_constructor::<crate::kind::Waterfall>(
            Plot::waterfall(category("category"), numeric("delta"), 0.0),
            PlotKind::Waterfall,
        );
    }

    #[test]
    fn erase_preserves_configuration_and_returns_any() {
        let erased: Plot<crate::kind::Any> = Plot::scatter(numeric("x"), numeric("y"))
            .title("Configured")
            .size(320, 240)
            .erase();

        assert_eq!(erased.kind(), PlotKind::Scatter);
        assert_eq!(erased.title.as_deref(), Some("Configured"));
        assert_eq!((erased.width, erased.height), (320, 240));
        assert_eq!(erased.theme_spec(), None);

        let explicitly_themed: Plot<crate::kind::Any> = Plot::scatter(numeric("x"), numeric("y"))
            .theme(Theme::Publication)
            .erase();
        assert_eq!(explicitly_themed.theme_spec(), Some(Theme::Publication));
    }

    #[test]
    fn significance_placements_compile_for_exact_supported_families() {
        let categories = || CategoryValues::new("category", ["a", "b"]).unwrap();
        let values = || NumericValues::new("value", [1.0, 2.0]);
        let groups = || CategoryValues::new("group", ["first", "second"]).unwrap();
        let attached = || Significance::at("a", crate::SignificanceMark::OneStar);
        let pairwise = || Significance::between("a", "b", crate::SignificanceMark::TwoStars);

        assert_marker::<crate::kind::Bar>(
            Plot::bar(categories(), BarInput::estimates(values()))
                .significance([attached()])
                .significance([pairwise()]),
        );
        assert_marker::<crate::kind::HorizontalBar>(
            Plot::horizontal_bar(categories(), BarInput::estimates(values()))
                .significance([attached()])
                .significance([pairwise()]),
        );
        assert_marker::<crate::kind::Coefficient>(
            Plot::coefficient(categories(), CoefficientInput::estimates(values()))
                .significance([attached()]),
        );
        assert_marker::<crate::kind::StackedBar>(
            Plot::stacked_bar(categories(), values(), groups()).significance([pairwise()]),
        );
        assert_marker::<crate::kind::StackedHorizontalBar>(
            Plot::stacked_horizontal_bar(categories(), values(), groups())
                .significance([pairwise()]),
        );
        assert_marker::<crate::kind::Boxplot>(
            Plot::boxplot(categories(), values()).significance([pairwise()]),
        );
        assert_marker::<crate::kind::Violin>(
            Plot::violin(categories(), values()).significance([pairwise()]),
        );
        assert_marker::<crate::kind::Raincloud>(
            Plot::raincloud(RaincloudInput::samples(categories(), values()))
                .significance([pairwise()]),
        );
    }

    #[test]
    fn canvas_accepts_distinct_typed_plot_families() {
        let canvas = crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, Plot::scatter(numeric("x"), numeric("y")))
            .unwrap()
            .panel(
                0,
                1,
                Plot::bar(category("category"), BarInput::estimates(numeric("value"))),
            );

        assert!(canvas.is_ok());
    }
}
