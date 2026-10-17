use super::Plot;
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisTick;
use crate::spec::{
    AxisScale, BarSpacing, CategoricalColors, CategoryOrder, HeatmapAnnotations, HeatmapLabelFit,
    HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale, HistogramBins, HistogramComposite,
    HistogramStat, InteractionStyle, LegendPolicy, LinePathStyle, NumericLabelFormat,
    PanelTextAnnotation, QqLine, RawOverlay, RegressionBand, WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{BooleanValues, Bounds, CategoryValues, EstimateStatus, Significance, SourceRows};

impl<K: crate::kind::Marker> Plot<K> {
    /// Set original source-row identities aligned one-to-one with the chart data.
    #[must_use]
    pub fn source_rows(mut self, source_rows: SourceRows) -> Self
    where
        K: crate::kind::SourceRows,
    {
        self.source_rows = Some(source_rows);
        self
    }

    /// Set categorical color identity independently of graphical grouping.
    #[must_use]
    pub fn hue(mut self, values: CategoryValues) -> Self
    where
        K: crate::kind::Hue,
    {
        self.hue = Some(values);
        self
    }

    /// Set the graphical-group column used to partition collective marks.
    #[must_use]
    pub fn group(mut self, values: CategoryValues) -> Self
    where
        K: crate::kind::Group,
    {
        self.group = Some(values);
        self
    }

    /// Set one positioned categorical series that owns dodge, color, legend,
    /// and significance identity where supported.
    #[must_use]
    pub fn series(mut self, values: CategoryValues) -> Self
    where
        K: crate::kind::Series,
    {
        self.series = Some(values);
        self
    }

    /// Set the title shown beside a continuous color scale.
    #[must_use]
    pub fn color_label(mut self, label: impl Into<String>) -> Self
    where
        K: crate::kind::ContinuousColor,
    {
        self.color_label = Some(label.into());
        self
    }

    /// Configure whether crowded heatmap category labels are elided or wrapped.
    #[must_use]
    pub fn heatmap_label_fit(mut self, fit: HeatmapLabelFit) -> Self
    where
        K: crate::kind::HeatmapOptions,
    {
        self.heatmap_label_fit = fit;
        self
    }

    /// Add caller-owned wrapped text for one fully-null Cartesian heatmap row.
    #[must_use]
    pub fn heatmap_null_row_annotation(mut self, annotation: HeatmapNullRowAnnotation) -> Self
    where
        K: crate::kind::HeatmapOptions,
    {
        self.heatmap_null_row_annotations.push(annotation);
        self
    }

    /// Set a strict Boolean column whose true heatmap cells receive a prominent border.
    #[must_use]
    pub fn heatmap_highlight(mut self, values: BooleanValues) -> Self
    where
        K: crate::kind::HeatmapOptions,
    {
        self.heatmap_highlight = Some(values);
        self
    }

    /// Draw fixed-format numeric labels beside point-only coefficient estimates.
    #[must_use]
    pub fn estimate_labels(mut self, format: NumericLabelFormat) -> Self
    where
        K: crate::kind::CoefficientOptions,
    {
        self.estimate_labels = Some(format);
        self
    }

    /// Configure significance typography independently of axis text.
    #[must_use]
    pub const fn significance_mark_style(mut self, style: crate::SignificanceMarkStyle) -> Self
    where
        K: crate::kind::SignificanceStyle,
    {
        self.significance_mark_style = Some(style);
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
    pub fn x_limits(mut self, min: f64, max: f64) -> Self
    where
        K: crate::kind::NumericX,
    {
        self.x_axis_spec = self.x_axis_spec.limits(min, max);
        self.x_limits_user_explicit = true;
        self
    }

    /// Set explicit y-axis limits.
    #[must_use]
    pub fn y_limits(mut self, min: f64, max: f64) -> Self
    where
        K: crate::kind::NumericY,
    {
        self.y_axis_spec = self.y_axis_spec.limits(min, max);
        self.y_limits_user_explicit = true;
        self
    }

    /// Replace automatic x-axis ticks with caller-labeled numeric positions.
    #[must_use]
    pub fn x_ticks(mut self, ticks: Vec<AxisTick>) -> Self
    where
        K: crate::kind::NumericX,
    {
        self.x_axis_spec.exact_ticks = Some(ticks);
        self
    }

    /// Replace automatic y-axis ticks with caller-labeled numeric positions.
    #[must_use]
    pub fn y_ticks(mut self, ticks: Vec<AxisTick>) -> Self
    where
        K: crate::kind::NumericY,
    {
        self.y_axis_spec.exact_ticks = Some(ticks);
        self
    }

    /// Append a numeric guide rendered in data space.
    #[must_use]
    pub fn guide(mut self, guide: GuideSpec) -> Self
    where
        K: crate::kind::Guides,
    {
        self.guides.push(guide);
        self
    }

    /// Set the primary categorical display order.
    ///
    /// Passing [`crate::ValueSort`] selects value-derived ordering after the
    /// supported plot family has prepared one scalar value per category. This
    /// option is supported by bar families and point-interval plots; heatmaps
    /// use [`Self::x_category_order`] and [`Self::y_category_order`].
    #[must_use]
    pub fn category_order(mut self, order: impl Into<CategoryOrder>) -> Self
    where
        K: crate::kind::CategoryOrder,
    {
        self.category_orders.primary = order.into();
        self
    }

    /// Set the horizontal categorical order for a heatmap.
    ///
    /// Heatmap axes accept observed or explicit ordering. Value-derived
    /// ordering is ambiguous because one axis category owns multiple cells.
    #[must_use]
    pub fn x_category_order(mut self, order: impl Into<CategoryOrder>) -> Self
    where
        K: crate::kind::HeatmapOptions,
    {
        self.category_orders.x = order.into();
        self
    }

    /// Set the vertical categorical order for a heatmap.
    ///
    /// Heatmap axes accept observed or explicit ordering. Value-derived
    /// ordering is ambiguous because one axis category owns multiple cells.
    #[must_use]
    pub fn y_category_order(mut self, order: impl Into<CategoryOrder>) -> Self
    where
        K: crate::kind::HeatmapOptions,
    {
        self.category_orders.y = order.into();
        self
    }

    /// Color positive and negative bars with distinct theme colors.
    #[must_use]
    pub fn color_by_sign(mut self) -> Self
    where
        K: crate::kind::SignedBars,
    {
        self.color_by_sign = true;
        self
    }

    /// Draw a reference line at zero on the continuous axis.
    #[must_use]
    pub fn zero_line(mut self) -> Self
    where
        K: crate::kind::ZeroLine,
    {
        self.zero_line = true;
        self
    }

    /// Set one typed availability value per aligned estimate row.
    #[must_use]
    pub fn estimate_status(mut self, statuses: impl IntoIterator<Item = EstimateStatus>) -> Self
    where
        K: crate::kind::EstimateStatus,
    {
        self.estimate_status = Some(statuses.into_iter().collect::<Vec<_>>().into());
        self
    }

    /// Set whether line-like plot families draw markers at estimates.
    #[must_use]
    pub fn markers(mut self, visible: bool) -> Self
    where
        K: crate::kind::Markers,
    {
        self.markers_visible = visible;
        self
    }

    /// Sets exact caller-owned colors for every categorical level used by the plot.
    #[must_use]
    pub fn categorical_colors(mut self, colors: CategoricalColors) -> Self
    where
        K: crate::kind::CategoricalColors,
    {
        self.categorical_colors = Some(colors);
        self
    }

    /// Add caller-owned single-line text at one corner inside the chart panel.
    #[must_use]
    pub fn panel_text_annotation(mut self, annotation: PanelTextAnnotation) -> Self
    where
        K: crate::kind::PanelText,
    {
        self.panel_text_annotations.push(annotation);
        self
    }

    /// Render scatter-like point marks without the default white outline.
    #[must_use]
    pub fn hide_point_outline(mut self) -> Self
    where
        K: crate::kind::PointOutline,
    {
        self.point_outline_visible = false;
        self
    }

    /// Set a complete explicit Plot theme.
    ///
    /// If omitted, a standalone Plot uses [`Theme::Publication`] and a Plot in
    /// a [`crate::Canvas`] inherits the Canvas root theme.
    #[must_use]
    pub fn theme(self, theme: Theme) -> Self {
        self.with_theme_value(theme.into_value())
    }

    pub(crate) fn with_theme_value(mut self, theme: crate::theme::ThemeValue) -> Self {
        self.theme = Some(std::sync::Arc::new(theme));
        self
    }

    /// Append caller-owned significance annotations of one placement.
    ///
    /// Ordinary bars accept attached and pairwise annotations through separate
    /// calls. Other plot families expose only their supported placement.
    #[must_use]
    pub fn significance<P>(mut self, annotations: impl IntoIterator<Item = Significance<P>>) -> Self
    where
        K: crate::kind::Significance<P>,
    {
        self.significance
            .extend(annotations.into_iter().map(Significance::into_annotation));
        self
    }

    /// Set the output pixel size. Default 640x480.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the legend visibility and placement policy.
    #[must_use]
    pub fn legend(mut self, policy: LegendPolicy) -> Self
    where
        K: crate::kind::Legend,
    {
        self.legend_policy = policy;
        self
    }

    /// Select a centered or deterministic source-row-displaced raw-observation
    /// overlay for a boxplot or violin plot.
    #[must_use]
    pub const fn raw_overlay(mut self, overlay: RawOverlay) -> Self
    where
        K: crate::kind::RawOverlay,
    {
        self.raw_overlay = overlay;
        self
    }
}

impl Plot<crate::kind::Heatmap> {
    /// Set the heatmap color-scale policy.
    #[must_use]
    pub fn scale(mut self, scale: HeatmapScale) -> Self {
        self.heatmap_scale = scale;
        self
    }

    /// Select Cartesian or structural triangular heatmap cells.
    #[must_use]
    pub fn layout(mut self, layout: HeatmapLayout) -> Self {
        self.heatmap_layout = layout;
        self
    }

    /// Configure optional heatmap cell-value annotations.
    #[must_use]
    pub fn annotations(mut self, annotations: HeatmapAnnotations) -> Self {
        self.heatmap_annotations = annotations;
        self
    }

    /// Set exact heatmap color-domain limits. Values outside them saturate.
    #[must_use]
    pub fn color_limits(mut self, min: f64, max: f64) -> Self {
        self.heatmap_color_limits = Some((min, max));
        self
    }
}

impl Plot<crate::kind::Forest> {
    /// Set the numeric axis scale used by a forest plot.
    #[must_use]
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.axis_scale = scale;
        self
    }
}

impl Plot<crate::kind::Beeswarm> {
    /// Set exact continuous-color limits. Values outside them saturate.
    #[must_use]
    pub fn color_limits(mut self, min: f64, max: f64) -> Self {
        self.beeswarm_color_limits = Some((min, max));
        self
    }
}

impl Plot<crate::kind::Bar> {
    /// Set spacing between series inside each category.
    #[must_use]
    pub fn spacing(mut self, spacing: BarSpacing) -> Self {
        self.bar_spacing = spacing;
        self
    }

    /// Append a strict point layer positioned at prepared bar centers.
    #[must_use]
    pub fn point_layer(mut self, layer: BarPointLayer) -> Self {
        self.bar_point_layers.push(layer);
        self
    }
}

impl Plot<crate::kind::HorizontalBar> {
    /// Set spacing between series inside each category.
    #[must_use]
    pub fn spacing(mut self, spacing: BarSpacing) -> Self {
        self.bar_spacing = spacing;
        self
    }

    /// Append a strict point layer positioned at prepared bar centers.
    #[must_use]
    pub fn point_layer(mut self, layer: BarPointLayer) -> Self {
        self.bar_point_layers.push(layer);
        self
    }
}

impl Plot<crate::kind::Histogram> {
    /// Set the histogram bin-selection strategy.
    #[must_use]
    pub fn bins(mut self, bins: HistogramBins) -> Self {
        self.histogram_bins = bins;
        self
    }

    /// Set the histogram vertical-axis normalization.
    #[must_use]
    pub fn stat(mut self, stat: HistogramStat) -> Self {
        self.histogram_stat = stat;
        self
    }

    /// Add a KDE density curve and complete source-observation rug.
    #[must_use]
    pub fn composite(mut self, composite: HistogramComposite) -> Self {
        self.histogram_composite = std::sync::Arc::new(composite);
        self
    }
}

impl Plot<crate::kind::Qq> {
    /// Set the normal Q-Q reference-line policy.
    #[must_use]
    pub fn reference_line(mut self, line: QqLine) -> Self {
        self.qq_line = line;
        self
    }
}

impl Plot<crate::kind::Line> {
    /// Append an independently owned supplemental point layer.
    ///
    /// Layers accept arbitrary coordinates independent from the primary line
    /// input. Per-point labels annotate coordinates, while a layer legend label
    /// identifies the complete layer once.
    #[must_use]
    pub fn point_layer(mut self, layer: PointLayer) -> Self {
        self.point_layers.push(layer);
        self
    }

    /// Add a drawable band to the line series.
    #[must_use]
    pub fn band(mut self, bounds: Bounds) -> Self {
        self.line_band = Some(bounds);
        self
    }
}

impl Plot<crate::kind::Regression> {
    /// Select the generated confidence-band policy.
    #[must_use]
    pub const fn band(mut self, band: RegressionBand) -> Self {
        self.regression_band = band;
        self
    }
}

impl Plot<crate::kind::LinePath> {
    /// Select the complete-path color and legend policy.
    #[must_use]
    pub const fn style(mut self, style: LinePathStyle) -> Self {
        self.line_path_style = style;
        self
    }
}

impl Plot<crate::kind::Coefficient> {
    /// Set the scientific reference/null line.
    #[must_use]
    pub fn reference_line(mut self, value: f64) -> Self {
        self.coefficient_reference_line = Some(value);
        self
    }
}

impl Plot<crate::kind::Interaction> {
    /// Set whether connectors are drawn between category estimates.
    #[must_use]
    pub const fn connectors(mut self, visible: bool) -> Self {
        self.interaction_connectors = Some(visible);
        self
    }

    /// Set exactly one typed point-and-connector style per interaction group.
    #[must_use]
    pub fn styles(mut self, styles: impl IntoIterator<Item = InteractionStyle>) -> Self {
        self.interaction_styles = Some(styles.into_iter().collect());
        self
    }
}

impl Plot<crate::kind::Waterfall> {
    /// Set caller-owned labels for base and terminal annotations.
    #[must_use]
    pub fn total_labels(mut self, labels: WaterfallLabels) -> Self {
        self.waterfall_labels = Some(labels);
        self
    }
}
