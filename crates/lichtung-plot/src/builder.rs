//! `lichtung-plot::builder` — the native `Plot` builder API.
//!
//! Semantic constructors accept immutable Rust-native values. Configuration is
//! chained before `.svg(path)` or `.png(path)` borrows the configured plot.
//!
//! Significance annotations are caller-owned display inputs. The renderer never
//! performs hypothesis tests or adjusts p-values.

use std::marker::PhantomData;
use std::sync::Arc;

mod accessors;
mod alpha;
mod configuration;
mod facets;
mod layout;
mod text_clear;
mod text_layout;

use crate::input::{AxisValues, PlotInput};
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::spec::{
    AxisScale, BarSpacing, CategoryOrder, DumbbellEndpoints, FacetSpec, HeatmapAnnotations,
    HeatmapLayout, HeatmapNullRowAnnotation, HistogramBins, HistogramComposite, HistogramStat,
    LegendPolicy, LinePathStyle, PlotKind, PlotTextAnnotation, QqLine, RawOverlay, RegressionBand,
    WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{
    BarInput, BooleanValues, Bounds, CategoryValues, CoefficientInput, HistogramInput,
    NumericFormat, NumericValues, RaincloudInput, SourceRows,
};

/// One configured semantic plot backed by immutable Rust-native values.
///
/// `K` is a zero-sized plot-family marker from [`crate::kind`]. Omitting it
/// means [`crate::kind::Any`], the erased form used by heterogeneous storage.
/// Family-specific methods are available only through exact-marker inherent
/// implementations or a corresponding sealed capability.
/// Without a sizing call, dimensions derive from content. [`Self::width`] fixes
/// width with content-derived height; [`Self::size`] fixes both dimensions.
/// The latest sizing call replaces the complete request. Final dimensions are
/// inspected on [`crate::Output`], after the scene has been frozen.
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn removed_width_getter(plot: &Plot<kind::Scatter>) {
///     let _ = plot.width();
/// }
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{kind, Plot};
/// fn removed_height_getter(plot: &Plot<kind::Scatter>) {
///     let _ = plot.height();
/// }
/// ```
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
/// use lichtung_plot::{BarInput, CategoryValues, ContinuousColorScale, NumericValues, Plot};
/// let plot = Plot::bar(
///     CategoryValues::new("category", ["a"]).unwrap(),
///     BarInput::estimates(NumericValues::new("value", [1.0])),
/// );
/// let _ = plot.continuous_color_scale(ContinuousColorScale::sequential(None, None).unwrap());
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
    description: PlotDescription,
    pub(crate) output_dimensions: Option<(u32, u32)>,
    x_limits_user_explicit: bool,
    y_limits_user_explicit: bool,
    parent_legend_composition: Option<crate::LegendComposition>,
    /// Canvas Shared owns the continuous color guide for this prepared child.
    suppress_continuous_guide: bool,
    resolved_facet_x_range: Option<(f64, f64)>,
    global_hue_levels: Option<Vec<String>>,
    global_group_levels: Option<Vec<String>>,
    global_collective_pairs: Option<Vec<crate::data::CollectivePair>>,
    global_line_condition_levels: Option<Vec<String>>,
    render_scope: Option<String>,
    pub(crate) canvas_owner: Option<Arc<[usize; 6]>>,
    pub(crate) prepared_discrete_guides:
        Option<Arc<crate::decorations::legend::discrete::PreparedGuides>>,
    pub(crate) prepared_axes:
        std::sync::Arc<Vec<(Option<String>, crate::plots::shared_x::PreparedAxes)>>,
    pub(crate) prepared_matrix_axes:
        Option<std::sync::Arc<crate::plots::matrix::PreparedMatrixAxes>>,
    pub(crate) prepared_families: std::sync::Arc<
        Vec<(
            Option<String>,
            crate::orchestrator::registration::PreparedFamily,
        )>,
    >,
    pub(crate) prepared_marginal: Option<std::sync::Arc<crate::plots::marginal::PreparedMarginal>>,
    pub(crate) prepared_waterfall:
        Option<std::sync::Arc<crate::plots::waterfall::PreparedWaterfallInputs>>,
    pub(crate) prepared_beeswarm:
        Option<std::sync::Arc<crate::plots::beeswarm::PreparedBeeswarmInputs>>,
    pub(crate) prepared_heatmap:
        Option<std::sync::Arc<crate::plots::heatmap::PreparedHeatmapInputs>>,
    axis_gutter_owner: AxisGutterOwner,
    y_tick_labels_visible: bool,
    marker: PhantomData<K>,
}

mod description;
pub(crate) use description::PlotDescription;

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

/// Coordinated replacements and complete numeric policies for a fixed child owner.
pub(crate) struct ComposedInputs {
    output_dimensions: Option<(u32, u32)>,
    text_layouts: crate::text::roles::TextRoles,
    prose: [crate::Text; 3],
    axis_titles: [crate::Text; 2],
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
    suppress_continuous_guide: bool,
    root_size: crate::layout::RootRequest,
    numeric: crate::numeric::inputs::NumericInputs,
    alpha: crate::alpha::AlphaInputs,
    pub(crate) guides: crate::decorations::legend::discrete::scoped::ScopedGuideInputs,
}

impl Plot<crate::kind::Any> {
    fn new<K: crate::kind::Marker>(input: PlotInput, kind: PlotKind) -> Plot<K> {
        Plot {
            description: PlotDescription::new(input, kind),
            output_dimensions: None,
            x_limits_user_explicit: false,
            y_limits_user_explicit: false,
            parent_legend_composition: None,
            suppress_continuous_guide: false,
            resolved_facet_x_range: None,
            global_hue_levels: None,
            global_group_levels: None,
            global_collective_pairs: None,
            global_line_condition_levels: None,
            render_scope: None,
            canvas_owner: None,
            prepared_discrete_guides: None,
            prepared_axes: std::sync::Arc::new(Vec::new()),
            prepared_families: std::sync::Arc::new(Vec::new()),
            prepared_marginal: None,
            prepared_heatmap: None,
            prepared_beeswarm: None,
            prepared_waterfall: None,
            prepared_matrix_axes: None,
            axis_gutter_owner: AxisGutterOwner::Root,
            y_tick_labels_visible: true,
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
            plot.description.payload_mut().histogram_stat = HistogramStat::Density;
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
        Self::new(
            PlotInput::Forest {
                estimate,
                term,
                bounds,
                reference,
            },
            PlotKind::Forest,
        )
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
        Self::new(PlotInput::Matrix { variables }, PlotKind::Matrix)
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
    pub fn erase(self) -> Plot<crate::kind::Any> {
        let Plot {
            description,
            output_dimensions,
            x_limits_user_explicit,
            y_limits_user_explicit,
            parent_legend_composition,
            suppress_continuous_guide,
            resolved_facet_x_range,
            global_hue_levels,
            global_group_levels,
            global_collective_pairs,
            global_line_condition_levels,
            render_scope,
            canvas_owner,
            axis_gutter_owner,
            y_tick_labels_visible,
            marker: _,
            prepared_axes,
            prepared_matrix_axes,
            prepared_discrete_guides,
            prepared_families,

            prepared_marginal,
            prepared_heatmap,
            prepared_beeswarm,
            prepared_waterfall,
        } = self;
        Plot {
            prepared_waterfall,
            prepared_beeswarm,
            prepared_marginal,
            prepared_heatmap,
            prepared_families,
            prepared_axes,
            prepared_matrix_axes,
            prepared_discrete_guides,
            description,
            output_dimensions,
            x_limits_user_explicit,
            y_limits_user_explicit,
            parent_legend_composition,
            suppress_continuous_guide,
            resolved_facet_x_range,
            global_hue_levels,
            global_group_levels,
            global_collective_pairs,
            global_line_condition_levels,
            render_scope,
            canvas_owner,
            axis_gutter_owner,
            y_tick_labels_visible,
            marker: PhantomData,
        }
    }
}
