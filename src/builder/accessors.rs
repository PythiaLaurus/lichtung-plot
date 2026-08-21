use crate::Error;

use super::Chart;
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::layout::AxisTick;
use crate::render::backend::PointStyle;
use crate::spec::{
    AxisScale, BarAggregation, BarSpacing, CategoricalColors, DumbbellEndpoints,
    EffectBandSemantics, EstimateSource, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout,
    HeatmapNullRowAnnotation, HeatmapScale, HistogramBins, HistogramComposite, HistogramStat,
    IntervalSemantics, LegendPosition, LineMode, LinePathStyle, NumericLabelFormat,
    PanelTextAnnotation, PlotKind, QqLine, RawOverlay, RegressionBand, ValueSort, ViolinLayout,
    WaterfallLabels,
};
use crate::Labels;
use crate::{ComparisonAnnotation, HistogramCompositeGeometryInput, RaincloudGeometryInput};

impl<'a> Chart<'a> {
    /// Borrow the input DataFrame.
    #[must_use]
    pub fn data(&self) -> &'a polars::prelude::DataFrame {
        self.data
    }

    pub(crate) const fn source_row_map(&self) -> Option<&'a [usize]> {
        self.source_rows
    }

    /// Currently-configured x column, if any.
    #[must_use]
    pub fn x_col(&self) -> Option<&'a str> {
        self.x_col
    }

    /// Currently-configured y column, if any.
    #[must_use]
    pub fn y_col(&self) -> Option<&'a str> {
        self.y_col
    }

    /// Currently-configured hue column, if any.
    #[must_use]
    pub fn hue_col(&self) -> Option<&'a str> {
        self.hue_col
    }

    /// Currently-configured explicit graphical-group column, if any.
    #[must_use]
    pub fn group_col(&self) -> Option<&'a str> {
        self.group_col
    }

    /// Currently configured continuous color column.
    #[must_use]
    pub fn color_col(&self) -> Option<&'a str> {
        self.color_col
    }

    /// Heatmap colorbar title, if explicitly configured.
    #[must_use]
    pub fn color_label_text(&self) -> Option<&str> {
        self.color_label.as_deref()
    }

    /// Requested heatmap color scale.
    #[must_use]
    pub fn heatmap_scale_spec(&self) -> HeatmapScale {
        self.heatmap_scale
    }

    /// Requested heatmap coordinate layout.
    #[must_use]
    pub fn heatmap_layout_spec(&self) -> HeatmapLayout {
        self.heatmap_layout
    }

    /// Requested categorical heatmap label-fit policy.
    #[must_use]
    pub const fn heatmap_label_fit_spec(&self) -> HeatmapLabelFit {
        self.heatmap_label_fit
    }

    /// Exact heatmap color limits, if configured.
    #[must_use]
    pub fn heatmap_color_limits_spec(&self) -> Option<(f64, f64)> {
        self.heatmap_color_limits
    }

    /// Exact beeswarm continuous-color limits, if configured.
    #[must_use]
    pub const fn beeswarm_color_limits_spec(&self) -> Option<(f64, f64)> {
        self.beeswarm_color_limits
    }

    /// Requested heatmap annotation policy.
    #[must_use]
    pub const fn heatmap_annotations_spec(&self) -> &HeatmapAnnotations {
        &self.heatmap_annotations
    }

    /// Caller-owned annotations for fully-null heatmap rows.
    #[must_use]
    pub fn heatmap_null_row_annotations_spec(&self) -> &[HeatmapNullRowAnnotation] {
        &self.heatmap_null_row_annotations
    }

    pub(crate) const fn heatmap_highlight_col(&self) -> Option<&'a str> {
        self.heatmap_highlight_col
    }

    /// Fixed numeric labels configured for point-only coefficient estimates.
    #[must_use]
    pub const fn estimate_labels_spec(&self) -> Option<&NumericLabelFormat> {
        self.estimate_labels.as_ref()
    }

    pub(crate) const fn coefficient_significance_col(&self) -> Option<&'a str> {
        self.coefficient_significance_col
    }

    pub(crate) const fn bar_significance_col(&self) -> Option<&'a str> {
        self.bar_significance_col
    }

    pub(crate) const fn significance_mark_style_spec(&self) -> crate::SignificanceMarkStyle {
        self.significance_mark_style
    }

    pub(crate) fn grouped_factor_annotations(&self) -> &[crate::GroupedFactorAnnotation] {
        &self.grouped_factor_annotations
    }

    /// Requested categorical value ordering.
    #[must_use]
    pub fn value_sort_spec(&self) -> ValueSort {
        self.value_sort
    }

    /// Requested histogram bin-selection strategy.
    #[must_use]
    pub fn histogram_bins_spec(&self) -> HistogramBins {
        self.histogram_bins
    }

    /// Requested histogram vertical-axis normalization.
    #[must_use]
    pub fn histogram_stat_spec(&self) -> HistogramStat {
        self.histogram_stat
    }

    /// Requested optional histogram density-and-rug composition.
    #[must_use]
    pub const fn histogram_composite_spec(&self) -> &HistogramComposite {
        &self.histogram_composite
    }

    pub(crate) const fn histogram_composite_geometry_input(
        &self,
    ) -> Option<&HistogramCompositeGeometryInput> {
        self.histogram_composite_geometry.as_ref()
    }

    /// Requested normal Q-Q reference-line policy.
    #[must_use]
    pub fn qq_line_spec(&self) -> QqLine {
        self.qq_line
    }

    /// Whether values should be colored according to their sign.
    #[must_use]
    pub fn colors_by_sign(&self) -> bool {
        self.color_by_sign
    }

    /// Whether the renderer should draw a zero reference line.
    #[must_use]
    pub fn zero_line_visible(&self) -> bool {
        self.zero_line
    }

    /// Lower and upper bound columns for a point-interval plot.
    #[must_use]
    pub fn interval_bound_cols(&self) -> Option<(&'a str, &'a str)> {
        self.interval_lower_col.zip(self.interval_upper_col)
    }

    /// Typed interpretation of configured interval bounds.
    #[must_use]
    pub fn interval_semantics_spec(&self) -> Option<&IntervalSemantics> {
        self.interval_semantics.as_ref()
    }

    /// Configured estimate-status column, if any.
    #[must_use]
    pub const fn estimate_status_col(&self) -> Option<&'a str> {
        self.estimate_status_col
    }

    /// Typed model-adjusted effect-band semantics, when configured.
    #[must_use]
    pub fn effect_band_semantics_spec(&self) -> Option<&EffectBandSemantics> {
        self.effect_band_semantics.as_ref()
    }

    pub(crate) const fn markers_visible(&self) -> bool {
        self.markers_visible
    }

    /// Configured confidence-band policy for generated regression plots.
    #[must_use]
    pub const fn regression_band_spec(&self) -> RegressionBand {
        self.regression_band
    }

    /// Configured semantic mode for line plots.
    #[must_use]
    pub const fn line_mode_spec(&self) -> LineMode {
        self.line_mode
    }

    /// Configured complete-path color and legend policy.
    #[must_use]
    pub const fn line_path_style_spec(&self) -> LinePathStyle {
        self.line_path_style
    }

    /// Configured original-observation overlay.
    #[must_use]
    pub const fn raw_overlay_spec(&self) -> RawOverlay {
        self.raw_overlay
    }

    pub(crate) const fn raw_overlay_source_row_col(&self) -> Option<&'a str> {
        self.raw_overlay_source_row_col
    }

    /// Requested violin-family geometry.
    #[must_use]
    pub const fn violin_layout_spec(&self) -> ViolinLayout {
        self.violin_layout
    }

    pub(crate) const fn raincloud_geometry_input(&self) -> Option<&RaincloudGeometryInput> {
        self.raincloud_geometry.as_ref()
    }

    pub(crate) const fn reference_line_value(&self) -> Option<f64> {
        self.reference_line
    }

    pub(crate) const fn axis_scale_spec(&self) -> AxisScale {
        self.axis_scale
    }

    /// Estimate provenance configured for an interaction plot.
    #[must_use]
    pub const fn interaction_source_spec(&self) -> Option<EstimateSource> {
        self.interaction_source
    }

    pub(crate) const fn interaction_connectors_spec(&self) -> Option<bool> {
        self.interaction_connectors
    }

    pub(crate) const fn interaction_connectors_visible(&self) -> bool {
        match self.interaction_connectors {
            Some(visible) => visible,
            None => true,
        }
    }

    pub(crate) const fn interaction_style_cols(&self) -> Option<(&'a str, &'a str)> {
        self.interaction_style_cols
    }

    pub(crate) const fn categorical_colors_spec(&self) -> Option<&CategoricalColors> {
        self.categorical_colors.as_ref()
    }

    /// Caller-owned panel-corner text annotations.
    #[must_use]
    pub fn panel_text_annotations(&self) -> &[PanelTextAnnotation] {
        &self.panel_text_annotations
    }

    pub(crate) fn categorical_color(
        &self,
        family: &str,
        label: &str,
        default_index: usize,
    ) -> Result<(u8, u8, u8), Error> {
        if let Some(colors) = self.categorical_colors_spec() {
            colors.color(label).ok_or_else(|| {
                Error::Data(format!(
                    "{family}: category {label:?} has no caller-owned color"
                ))
            })
        } else {
            let palette = self.palette();
            Ok(palette.colors[default_index % palette.colors.len()])
        }
    }

    /// Base and terminal metadata columns for a waterfall.
    #[must_use]
    pub fn waterfall_total_cols(&self) -> Option<(&'a str, &'a str)> {
        self.waterfall_base_col.zip(self.waterfall_terminal_col)
    }

    /// Caller-owned waterfall annotation labels, if configured.
    #[must_use]
    pub const fn waterfall_labels_spec(&self) -> Option<&WaterfallLabels> {
        self.waterfall_labels.as_ref()
    }

    /// Second endpoint value column for a dumbbell plot.
    #[must_use]
    pub const fn dumbbell_right_col(&self) -> Option<&'a str> {
        self.dumbbell_right_col
    }

    /// Required endpoint labels and colors for a dumbbell plot.
    #[must_use]
    pub const fn dumbbell_endpoints_spec(&self) -> Option<&DumbbellEndpoints> {
        self.dumbbell_endpoints.as_ref()
    }

    /// The chart title, if any.
    #[must_use]
    pub fn title_text(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The chart subtitle, if any.
    #[must_use]
    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// The chart caption, if any.
    #[must_use]
    pub fn caption_text(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    /// The x-axis label, if any.
    #[must_use]
    pub fn x_label_text(&self) -> Option<&str> {
        self.x_label.as_deref()
    }

    /// The y-axis label, if any.
    #[must_use]
    pub fn y_label_text(&self) -> Option<&str> {
        self.y_label.as_deref()
    }

    /// Borrow the presentation rules for automatically sourced labels.
    #[must_use]
    pub const fn labels_spec(&self) -> &Labels {
        &self.labels
    }

    pub(crate) fn resolve_label(&self, raw: &str) -> String {
        self.labels.resolve(raw)
    }

    pub(crate) fn resolve_label_domain<'b>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'b str>,
    ) -> Result<Vec<String>, crate::Error> {
        self.labels.resolve_domain(domain, raw_labels)
    }

    pub(crate) fn resolved_axis_labels(&self, kind: PlotKind) -> (String, String) {
        let inferred =
            |raw: Option<&str>| raw.map_or_else(String::new, |raw| self.resolve_label(raw));
        let x = self.x_label.clone().unwrap_or_else(|| match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell => inferred(self.y_col),
            PlotKind::Qq => "Theoretical normal quantiles".to_owned(),
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || inferred(self.x_col),
                |input| self.resolve_label(&input.dimensions.value),
            ),
            PlotKind::Violin => self.raincloud_geometry_input().map_or_else(
                || inferred(self.x_col),
                |input| self.resolve_label(&input.dimensions.category),
            ),
            PlotKind::Matrix => String::new(),
            _ => inferred(self.x_col),
        });
        let y = self.y_label.clone().unwrap_or_else(|| match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell
            | PlotKind::Qq => inferred(self.x_col),
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || match self.histogram_stat {
                    HistogramStat::Count => "Count".to_owned(),
                    HistogramStat::Probability => "Probability".to_owned(),
                    HistogramStat::Density => "Density".to_owned(),
                },
                |input| self.resolve_label(&input.dimensions.density),
            ),
            PlotKind::Violin => self.raincloud_geometry_input().map_or_else(
                || inferred(self.y_col),
                |input| self.resolve_label(&input.dimensions.value),
            ),
            PlotKind::Density => "Density".to_owned(),
            PlotKind::Matrix => String::new(),
            _ => inferred(self.y_col),
        });
        (x, y)
    }

    pub(crate) fn resolved_color_label(&self) -> String {
        self.color_label.clone().unwrap_or_else(|| {
            self.color_col
                .map_or_else(String::new, |raw| self.resolve_label(raw))
        })
    }

    pub(crate) fn resolved_legend_title(&self) -> String {
        if self.interaction_style_cols.is_some() || self.hue_dimension_key().is_some() {
            self.resolve_label(self.legend_title_key())
        } else {
            "series".to_owned()
        }
    }

    pub(crate) fn legend_title_key(&self) -> &str {
        self.interaction_style_cols
            .and(self.group_col)
            .or_else(|| self.hue_dimension_key())
            .unwrap_or("series")
    }

    pub(crate) fn hue_dimension_key(&self) -> Option<&str> {
        self.hue_col.or_else(|| {
            self.raincloud_geometry
                .as_ref()
                .and_then(|input| input.dimensions.hue.as_deref())
        })
    }

    /// Borrow x-axis scale configuration.
    #[must_use]
    pub fn x_axis_spec(&self) -> &AxisSpec {
        &self.x_axis_spec
    }

    pub(crate) const fn x_tick_count_overridden(&self) -> bool {
        self.x_tick_count_overridden
    }

    pub(crate) fn explicit_x_ticks(&self) -> Option<&[AxisTick]> {
        self.x_ticks.as_deref()
    }

    /// Borrow y-axis scale configuration.
    #[must_use]
    pub fn y_axis_spec(&self) -> &AxisSpec {
        &self.y_axis_spec
    }

    pub(crate) const fn y_tick_count_overridden(&self) -> bool {
        self.y_tick_count_overridden
    }

    pub(crate) fn explicit_y_ticks(&self) -> Option<&[AxisTick]> {
        self.y_ticks.as_deref()
    }

    pub(crate) fn guides(&self) -> &[GuideSpec] {
        &self.guides
    }

    pub(crate) fn point_layers(&self) -> &[PointLayer<'a>] {
        &self.point_layers
    }

    pub(crate) fn bar_point_layers(&self) -> &[BarPointLayer<'a>] {
        &self.bar_point_layers
    }

    /// Borrow the registered explicit pairwise comparisons.
    #[must_use]
    pub fn comparisons(&self) -> &[ComparisonAnnotation] {
        &self.comparisons
    }

    pub(crate) fn shows_non_significant(&self) -> bool {
        self.show_non_significant
    }

    /// Configured maximum significance-star count.
    #[must_use]
    pub const fn significance_star_limit_spec(&self) -> crate::SignificanceStarLimit {
        self.significance_star_limit
    }

    /// Output width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Output height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Borrow the peak markers for `PlotKind::MassSpectrum`.
    /// Returns an empty slice for all other kinds.
    #[must_use]
    pub fn peaks(&self) -> &[(f64, f64)] {
        &self.peaks
    }

    /// Borrow the pre-computed scatter points for
    /// `PlotKind::Marginal`. Returns an empty slice if
    /// the data-column path should be used.
    #[must_use]
    pub fn marginal_coords(&self) -> &[(f64, f64)] {
        &self.marginal_coords
    }

    /// Borrow the ordered variable names for `PlotKind::Matrix`.
    #[must_use]
    pub(crate) fn matrix_variable_names(&self) -> &[String] {
        &self.matrix_variables
    }

    /// Grouped-bar spacing preset.
    #[must_use]
    pub fn bar_spacing_spec(&self) -> BarSpacing {
        self.bar_spacing
    }

    /// Configured value construction for ordinary and grouped bars.
    #[must_use]
    pub const fn bar_aggregation_spec(&self) -> BarAggregation {
        self.bar_aggregation
    }

    /// Build a point style using this chart's point-outline policy.
    #[must_use]
    pub(crate) fn point_style(&self, color: (u8, u8, u8), radius: u32) -> PointStyle {
        PointStyle::with_outline(color, radius, self.point_outline)
    }

    /// Active plot kind, if a terminal method (`.scatter()` /
    /// `.bar()` / `.boxplot()`) has been called.
    #[must_use]
    pub fn plot_kind(&self) -> Option<PlotKind> {
        self.plot_kind
    }

    /// Currently requested legend placement.
    #[must_use]
    pub fn legend_position_spec(&self) -> LegendPosition {
        self.legend_position
    }

    pub(crate) fn legend_visibility(&self) -> Option<bool> {
        self.legend_visible
    }

    pub(crate) fn with_suppressed_legend(mut self) -> Self {
        self.legend_visible = Some(false);
        self
    }

    pub(crate) const fn axis_text_is_measured(&self) -> bool {
        self.axis_text_measured
    }

    pub(crate) const fn composed_axis_text_is_measured(&self) -> bool {
        self.composed_axis_text_measured
    }

    pub(crate) const fn y_tick_labels_visible(&self) -> bool {
        self.y_tick_labels_visible
    }

    pub(crate) const fn with_y_tick_labels_visible(mut self, visible: bool) -> Self {
        self.y_tick_labels_visible = visible;
        self
    }

    pub(crate) fn with_planned_y_limits(mut self, limits: (f64, f64)) -> Self {
        self.y_axis_spec.limits = Some(limits);
        self
    }

    pub(crate) fn with_planned_x_limits(mut self, limits: (f64, f64)) -> Self {
        self.x_axis_spec.limits = Some(limits);
        self
    }

    pub(crate) const fn user_x_limits(&self) -> Option<(f64, f64)> {
        if self.x_limits_user_explicit {
            self.x_axis_spec.limits
        } else {
            None
        }
    }

    pub(crate) const fn user_y_limits(&self) -> Option<(f64, f64)> {
        if self.y_limits_user_explicit {
            self.y_axis_spec.limits
        } else {
            None
        }
    }
}
