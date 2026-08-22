use super::Plot;
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisTick;
use crate::spec::{
    AxisScale, BarAggregation, BarSpacing, CategoricalColors, CategoryOrder, DumbbellEndpoints,
    EffectBandSemantics, EstimateSource, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout,
    HeatmapNullRowAnnotation, HeatmapScale, HistogramBins, HistogramComposite, HistogramStat,
    IntervalSemantics, LegendPosition, LineMode, LinePathStyle, NumericLabelFormat,
    PanelTextAnnotation, QqLine, RawOverlay, RegressionBand, ViolinLayout, WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{
    BooleanValues, CategoryValues, HistogramCompositeGeometryInput, NumericValues,
    RaincloudGeometryInput, Significance, SourceRows,
};

impl Plot {
    /// Configure PNG sizing and compression before the first terminal output.
    #[must_use]
    pub const fn png_options(mut self, options: crate::PngOptions) -> Self {
        self.png_options = options;
        self
    }

    /// Configure selectable text or outlined text for SVG output.
    #[must_use]
    pub const fn svg_text(mut self, text: crate::SvgText) -> Self {
        self.svg_text = text;
        self
    }

    /// Set original source-row identities aligned one-to-one with the chart data.
    ///
    /// An explicitly configured raw-overlay source-row column takes precedence
    /// for charts that configure both forms of identity.
    #[must_use]
    pub fn source_rows(mut self, source_rows: SourceRows) -> Self {
        self.source_rows = Some(source_rows);
        self
    }

    /// Set categorical color identity.
    ///
    /// For bars, boxplots, and sample-derived violins, this configures the same
    /// positioned, colored series as [`Self::group`]; the last call wins.
    #[must_use]
    pub fn hue(mut self, values: CategoryValues) -> Self {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series = Some(values);
        } else {
            self.hue = Some(values);
        }
        self
    }

    /// Set the graphical-group column used to partition collective marks.
    ///
    /// For bars, boxplots, and sample-derived violins, this configures the same
    /// positioned, colored series as [`Self::hue`]; the last call wins.
    #[must_use]
    pub fn group(mut self, values: CategoryValues) -> Self {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series = Some(values);
        } else {
            self.group = Some(values);
        }
        self
    }

    /// Set a continuous color-value column for plot kinds that support it.
    #[must_use]
    pub fn color(mut self, values: NumericValues) -> Self {
        self.color = Some(values);
        self
    }

    /// Set the title shown beside a heatmap color scale.
    #[must_use]
    pub fn color_label(mut self, label: impl Into<String>) -> Self {
        self.color_label = Some(label.into());
        self
    }

    /// Set the heatmap color-scale policy.
    #[must_use]
    pub fn heatmap_scale(mut self, scale: HeatmapScale) -> Self {
        self.heatmap_scale = scale;
        self
    }

    /// Select Cartesian or structural triangular heatmap cells.
    #[must_use]
    pub fn heatmap_layout(mut self, layout: HeatmapLayout) -> Self {
        self.heatmap_layout = layout;
        self
    }

    /// Configure whether crowded heatmap category labels are elided or wrapped.
    #[must_use]
    pub fn heatmap_label_fit(mut self, fit: HeatmapLabelFit) -> Self {
        self.heatmap_label_fit = fit;
        self
    }

    /// Set exact heatmap color-domain limits. Values outside them saturate.
    #[must_use]
    pub fn heatmap_color_limits(mut self, min: f64, max: f64) -> Self {
        self.heatmap_color_limits = Some((min, max));
        self
    }

    /// Set exact beeswarm continuous-color limits. Values outside them saturate.
    #[must_use]
    pub fn beeswarm_color_limits(mut self, min: f64, max: f64) -> Self {
        self.beeswarm_color_limits = Some((min, max));
        self
    }

    /// Configure optional heatmap cell-value annotations.
    #[must_use]
    pub fn heatmap_annotations(mut self, annotations: HeatmapAnnotations) -> Self {
        self.heatmap_annotations = annotations;
        self
    }

    /// Add caller-owned wrapped text for one fully-null Cartesian heatmap row.
    #[must_use]
    pub fn heatmap_null_row_annotation(mut self, annotation: HeatmapNullRowAnnotation) -> Self {
        self.heatmap_null_row_annotations.push(annotation);
        self
    }

    /// Set a strict Boolean column whose true heatmap cells receive a prominent border.
    #[must_use]
    pub fn heatmap_highlight(mut self, values: BooleanValues) -> Self {
        self.heatmap_highlight = Some(values);
        self
    }

    /// Draw fixed-format numeric labels beside point-only coefficient estimates.
    #[must_use]
    pub fn estimate_labels(mut self, format: NumericLabelFormat) -> Self {
        self.estimate_labels = Some(format);
        self
    }

    /// Configure significance typography independently of axis text.
    #[must_use]
    pub const fn significance_mark_style(mut self, style: crate::SignificanceMarkStyle) -> Self {
        self.significance_mark_style = style;
        self
    }

    /// Set the chart title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the chart subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the chart caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Set the x-axis label.
    #[must_use]
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the y-axis label.
    #[must_use]
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set presentation rules for every label inferred from data identities.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Set explicit x-axis limits.
    #[must_use]
    pub fn x_limits(mut self, min: f64, max: f64) -> Self {
        self.x_axis_spec = self.x_axis_spec.limits(min, max);
        self.x_limits_user_explicit = true;
        self
    }

    /// Set explicit y-axis limits.
    #[must_use]
    pub fn y_limits(mut self, min: f64, max: f64) -> Self {
        self.y_axis_spec = self.y_axis_spec.limits(min, max);
        self.y_limits_user_explicit = true;
        self
    }

    /// Set target x-axis tick count.
    #[must_use]
    pub fn x_tick_count(mut self, count: usize) -> Self {
        self.x_axis_spec = self.x_axis_spec.tick_count(count);
        self.x_tick_count_overridden = true;
        self
    }

    /// Replace automatic x-axis ticks with caller-labeled numeric positions.
    #[must_use]
    pub fn x_ticks(mut self, ticks: Vec<AxisTick>) -> Self {
        self.x_ticks = Some(ticks);
        self
    }

    /// Set target y-axis tick count.
    #[must_use]
    pub fn y_tick_count(mut self, count: usize) -> Self {
        self.y_axis_spec = self.y_axis_spec.tick_count(count);
        self.y_tick_count_overridden = true;
        self
    }

    /// Replace automatic y-axis ticks with caller-labeled numeric positions.
    #[must_use]
    pub fn y_ticks(mut self, ticks: Vec<AxisTick>) -> Self {
        self.y_ticks = Some(ticks);
        self
    }

    /// Append a numeric guide rendered in data space.
    #[must_use]
    pub fn guide(mut self, guide: GuideSpec) -> Self {
        self.guides.push(guide);
        self
    }

    /// Append a separately borrowed strict point layer to a numeric line chart.
    #[must_use]
    pub fn point_layer(mut self, layer: PointLayer) -> Self {
        self.point_layers.push(layer);
        self
    }

    /// Append a separately borrowed strict point layer to an ordinary bar chart.
    #[must_use]
    pub fn bar_point_layer(mut self, layer: BarPointLayer) -> Self {
        self.bar_point_layers.push(layer);
        self
    }

    /// Set grouped-bar spacing.
    #[must_use]
    pub fn bar_spacing(mut self, spacing: BarSpacing) -> Self {
        self.bar_spacing = spacing;
        self
    }

    /// Select sample-mean inference or one-row identity values for ordinary bars.
    #[must_use]
    pub const fn bar_aggregation(mut self, aggregation: BarAggregation) -> Self {
        self.bar_aggregation = aggregation;
        self
    }

    /// Set the histogram bin-selection strategy.
    #[must_use]
    pub fn histogram_bins(&mut self, bins: HistogramBins) -> &mut Self {
        self.histogram_bins = bins;
        self
    }

    /// Set the histogram vertical-axis normalization.
    #[must_use]
    pub fn histogram_stat(&mut self, stat: HistogramStat) -> &mut Self {
        self.histogram_stat = stat;
        self
    }

    /// Add a KDE density curve and complete source-observation rug to a histogram.
    #[must_use]
    pub fn histogram_composite(mut self, composite: HistogramComposite) -> Self {
        self.histogram_composite = composite;
        self
    }

    /// Use complete caller-computed density-axis histogram composite geometry.
    ///
    /// This selects [`HistogramStat::Density`]. Rendering maps the supplied bins,
    /// KDE points, and rug segments to the panel without deriving sample statistics.
    #[must_use]
    pub fn histogram_composite_geometry(mut self, input: HistogramCompositeGeometryInput) -> Self {
        self.histogram_stat = HistogramStat::Density;
        self.histogram_composite_geometry = Some(input);
        self
    }

    /// Set the normal Q-Q reference-line policy.
    #[must_use]
    pub fn qq_line(&mut self, line: QqLine) -> &mut Self {
        self.qq_line = line;
        self
    }

    /// Set the primary categorical display order.
    ///
    /// Passing [`crate::ValueSort`] selects value-derived ordering after the
    /// supported plot family has prepared one scalar value per category. This
    /// option is supported by bar families and point-interval plots; heatmaps
    /// use [`Self::x_category_order`] and [`Self::y_category_order`].
    #[must_use]
    pub fn category_order(mut self, order: impl Into<CategoryOrder>) -> Self {
        self.category_orders.primary = order.into();
        self
    }

    /// Set the horizontal categorical order for a heatmap.
    ///
    /// Heatmap axes accept observed or explicit ordering. Value-derived
    /// ordering is ambiguous because one axis category owns multiple cells.
    #[must_use]
    pub fn x_category_order(mut self, order: impl Into<CategoryOrder>) -> Self {
        self.category_orders.x = order.into();
        self
    }

    /// Set the vertical categorical order for a heatmap.
    ///
    /// Heatmap axes accept observed or explicit ordering. Value-derived
    /// ordering is ambiguous because one axis category owns multiple cells.
    #[must_use]
    pub fn y_category_order(mut self, order: impl Into<CategoryOrder>) -> Self {
        self.category_orders.y = order.into();
        self
    }

    /// Color positive and negative bars with distinct theme colors.
    #[must_use]
    pub fn color_by_sign(mut self) -> Self {
        self.color_by_sign = true;
        self
    }

    /// Draw a reference line at zero on the continuous axis.
    #[must_use]
    pub fn zero_line(mut self) -> Self {
        self.zero_line = true;
        self
    }

    /// Set columns containing lower and upper bounds for a bounds-consuming plot.
    /// Ordinary identity bars consume these columns with matching semantics;
    /// generated mean and stacked bars reject them.
    #[must_use]
    pub fn interval_bounds(mut self, lower: NumericValues, upper: NumericValues) -> Self {
        self.interval_lower = Some(lower);
        self.interval_upper = Some(upper);
        self
    }

    /// Attach the statistical interpretation and provenance of rendered intervals.
    #[must_use]
    pub fn interval_semantics(mut self, semantics: IntervalSemantics) -> Self {
        self.interval_semantics = Some(semantics);
        self
    }

    /// Set an exact `estimated`/`not_estimable` availability column for estimate rows.
    #[must_use]
    pub fn estimate_status(mut self, values: CategoryValues) -> Self {
        self.estimate_status = Some(values);
        self
    }

    /// Attach model-adjusted pointwise or simultaneous semantics to a Line ribbon.
    #[must_use]
    pub fn effect_band_semantics(mut self, semantics: EffectBandSemantics) -> Self {
        self.interval_semantics = Some(semantics.interval().clone());
        self.effect_band_semantics = Some(semantics);
        self
    }

    /// Set whether line-like plot families draw markers at estimates.
    #[must_use]
    pub fn markers(mut self, visible: bool) -> Self {
        self.markers_visible = visible;
        self
    }

    /// Select the generated regression confidence-band policy.
    #[must_use]
    pub const fn regression_band(mut self, band: RegressionBand) -> Self {
        self.regression_band = band;
        self
    }

    /// Select ordinary series, repeated trajectory, or source-ordered path semantics.
    #[must_use]
    pub const fn line_mode(mut self, mode: LineMode) -> Self {
        self.line_mode = mode;
        self
    }

    /// Select the complete-path color and legend policy for `LineMode::Path`.
    #[must_use]
    pub const fn line_path_style(mut self, style: LinePathStyle) -> Self {
        self.line_path_style = style;
        self
    }

    /// Set the scientific reference/null line for an estimate plot.
    #[must_use]
    pub fn reference_line(mut self, value: f64) -> Self {
        self.reference_line = Some(value);
        self
    }

    /// Set the numeric axis scale used by a forest plot.
    #[must_use]
    pub fn axis_scale(mut self, scale: AxisScale) -> Self {
        self.axis_scale = scale;
        self
    }

    /// Identify interaction estimates as raw summaries or model-adjusted means.
    #[must_use]
    pub fn interaction_source(mut self, source: EstimateSource) -> Self {
        self.interaction_source = Some(source);
        self
    }

    /// Set whether an interaction plot draws connectors between category estimates.
    #[must_use]
    pub const fn interaction_connectors(mut self, visible: bool) -> Self {
        self.interaction_connectors = Some(visible);
        self
    }

    /// Set exact categorical point-shape and line-pattern columns for interaction traces.
    ///
    /// Point-shape values must be [`crate::PointShape::as_str`] values and line-pattern
    /// values must be [`crate::LinePattern::as_str`] values. Each graphical trace must
    /// resolve to one constant value from each column.
    #[must_use]
    pub fn interaction_styles(
        mut self,
        point_shapes: CategoryValues,
        line_patterns: CategoryValues,
    ) -> Self {
        self.interaction_styles = Some((point_shapes, line_patterns));
        self
    }

    /// Sets exact caller-owned colors for every categorical level used by the plot.
    #[must_use]
    pub fn categorical_colors(mut self, colors: CategoricalColors) -> Self {
        self.categorical_colors = Some(colors);
        self
    }

    /// Add caller-owned single-line text at one corner inside the chart panel.
    #[must_use]
    pub fn panel_text_annotation(mut self, annotation: PanelTextAnnotation) -> Self {
        self.panel_text_annotations.push(annotation);
        self
    }

    /// Set caller-owned labels for waterfall base and terminal annotations.
    #[must_use]
    pub fn waterfall_labels(mut self, labels: WaterfallLabels) -> Self {
        self.waterfall_labels = Some(labels);
        self
    }

    /// Set required caller-owned labels and colors for both dumbbell endpoints.
    #[must_use]
    pub fn dumbbell_endpoints(mut self, endpoints: DumbbellEndpoints) -> Self {
        self.dumbbell_endpoints = Some(endpoints);
        self
    }

    /// Render scatter-like point marks without the default white outline.
    #[must_use]
    pub fn hide_point_outline(mut self) -> Self {
        self.point_outline = None;
        self
    }

    /// Set the visual theme.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Append caller-owned significance marks and pairwise brackets.
    #[must_use]
    pub fn significance(mut self, annotations: impl IntoIterator<Item = Significance>) -> Self {
        self.significance.extend(annotations);
        self
    }

    /// Set the output pixel size. Default 640x480.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Request a specific legend placement.
    #[must_use]
    pub fn legend_position(mut self, position: LegendPosition) -> Self {
        self.legend_position = position;
        self.legend_visible = Some(position != LegendPosition::Hidden);
        self
    }

    /// Force the legend to be rendered when the selected plot can produce one.
    #[must_use]
    pub fn show_legend(mut self) -> Self {
        self.legend_visible = Some(true);
        self
    }

    /// Suppress the legend even when the selected plot can produce one.
    #[must_use]
    pub fn hide_legend(mut self) -> Self {
        self.legend_visible = Some(false);
        self
    }

    /// Select a centered or deterministic source-row-displaced raw-observation
    /// overlay for a boxplot or violin plot.
    #[must_use]
    pub const fn raw_overlay(mut self, overlay: RawOverlay) -> Self {
        self.raw_overlay = overlay;
        self
    }

    /// Set the integer column carrying original row identities for a visible
    /// raw-observation overlay after the caller has filtered or compacted data.
    #[must_use]
    pub fn raw_overlay_source_rows(mut self, source_rows: SourceRows) -> Self {
        self.raw_overlay_source_rows = Some(source_rows);
        self
    }

    /// Select the mirrored or raincloud violin geometry.
    #[must_use]
    pub const fn violin_layout(mut self, layout: ViolinLayout) -> Self {
        self.violin_layout = layout;
        self
    }

    /// Use complete caller-computed raincloud geometry.
    ///
    /// This selects [`ViolinLayout::Raincloud`]. Rendering maps the supplied
    /// density boundaries, summaries, and strip points without KDE, quantile,
    /// or jitter calculation.
    #[must_use]
    pub fn raincloud_geometry(mut self, input: RaincloudGeometryInput) -> Self {
        self.violin_layout = ViolinLayout::Raincloud;
        self.raincloud_geometry = Some(input);
        self
    }
}
