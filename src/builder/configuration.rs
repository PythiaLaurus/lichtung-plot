use super::Plot;
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisTick;
use crate::spec::{
    AxisScale, BarSpacing, CategoryOrder, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout,
    HeatmapNullRowAnnotation, HistogramBins, HistogramComposite, HistogramStat, InteractionStyle,
    LegendPolicy, LinePathStyle, PanelTextAnnotation, QqLine, RawOverlay, RegressionBand,
    WaterfallLabels,
};
use crate::theme::Theme;
use crate::Labels;
use crate::{
    BooleanValues, Bounds, CategoryValues, EstimateStatus, NumericFormat, Significance, SourceRows,
};

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

    /// Replace numeric presentation for this Plot's continuous color guide.
    /// Shared Canvas guides require identical complete effective formats before fonts.
    /// ```compile_fail
    /// use lichtung_plot::{kind, NumericFormat, Plot};
    /// fn unsupported(plot: Plot<kind::Scatter>, format: NumericFormat) {
    ///     let _ = plot.color_label_format(format);
    /// }
    /// ```
    #[must_use]
    pub fn color_label_format(mut self, format: NumericFormat) -> Self
    where
        K: crate::kind::ContinuousColor,
    {
        self.color_label_format = Some(std::sync::Arc::new(format));
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

    /// Draw numeric labels beside point-only Coefficient estimates.
    /// One precision is selected from every available prepared estimate in the panel.
    /// ```compile_fail
    /// use lichtung_plot::{kind, NumericFormat, Plot};
    /// fn unsupported(plot: Plot<kind::Forest>, format: NumericFormat) {
    ///     let _ = plot.estimate_labels(format);
    /// }
    /// ```
    #[must_use]
    pub fn estimate_labels(mut self, format: NumericFormat) -> Self
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
    ///
    /// Plain strings replace content while retaining existing appearance and layout
    /// instructions. A supplied Text's present style or whole layout replaces that
    /// instruction; an explicit default layout resets it to owner defaults.
    #[must_use]
    pub fn title(mut self, title: impl Into<crate::Text>) -> Self {
        self.title.replace(title.into());
        self
    }

    /// Set the chart subtitle.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<crate::Text>) -> Self {
        self.subtitle.replace(subtitle.into());
        self
    }

    /// Align this Plot's title and each of its lines to a physical edge or center.
    ///
    /// Overrides the Canvas panel default and this Plot's complete theme. Repeated
    /// calls replace only this instruction; theme attachment order is independent.
    /// Start and End are physical left and right after the owning content insets.
    /// ```
    /// use lichtung_plot::{NumericValues, Plot, Theme, TextAlignment};
    /// let plot = Plot::scatter(
    ///     NumericValues::new("x", [1.0, 2.0]),
    ///     NumericValues::new("y", [2.0, 1.0]),
    /// ).title("Title").subtitle("Independent subtitle")
    ///  .title_alignment(TextAlignment::End)
    ///  .subtitle_alignment(TextAlignment::Start)
    ///  .theme(Theme::nature());
    /// # let _ = plot;
    /// ```
    #[must_use]
    pub const fn title_alignment(mut self, alignment: crate::TextAlignment) -> Self {
        self.title.edit_alignment(alignment);
        self
    }

    /// Align this Plot's subtitle independently of its title.
    ///
    /// Overrides the Canvas panel subtitle default, then the optional theme
    /// subtitle alignment, then the final effective title alignment. Omitting this
    /// call contributes no instruction; it does not bypass those lower owners.
    #[must_use]
    pub const fn subtitle_alignment(mut self, alignment: crate::TextAlignment) -> Self {
        self.subtitle.edit_alignment(alignment);
        self
    }

    /// Set the chart caption.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn caption(mut self, caption: impl Into<crate::Text>) -> Self {
        self.caption.replace(caption.into());
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

    /// Replace the complete generated x-tick selection.
    ///
    /// Exact [`Self::x_ticks`] take precedence in either setter order, independently
    /// of generated count limits. Targets allow nice-step deviation; maxima bound
    /// complete in-domain sources, including endpoints. Automatic resets a previous
    /// interval or count. The actual axis must be numeric; an interval must also
    /// match its linear/logarithmic scale. Mismatches reject before fonts.
    /// ```
    /// use lichtung_plot::{NumericValues, Plot, TickGeneration};
    /// let plot = Plot::scatter(NumericValues::new("x", [0.0, 10.0]),
    ///     NumericValues::new("y", [0.0, 4.0]))
    ///     .x_tick_generation(TickGeneration::target(5)?)
    ///     .y_tick_generation(TickGeneration::maximum(4)?)
    ///     .x_tick_generation(TickGeneration::automatic());
    /// # let _ = plot;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn categorical(plot: Plot<kind::Bar>, generation: TickGeneration) {
    ///     let _ = plot.x_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{Plot, TickGeneration};
    /// fn erased(plot: Plot, generation: TickGeneration) {
    ///     let _ = plot.x_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn matrix(plot: Plot<kind::Matrix>, generation: TickGeneration) {
    ///     let _ = plot.x_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn heatmap(plot: Plot<kind::Heatmap>, generation: TickGeneration) {
    ///     let _ = plot.x_tick_generation(generation);
    /// }
    /// ```
    #[must_use]
    pub fn x_tick_generation(mut self, generation: crate::TickGeneration) -> Self
    where
        K: crate::kind::NumericX,
    {
        self.x_axis_spec.generation = generation;
        self
    }

    /// Replace the complete generated y-tick selection, independently of x.
    ///
    /// Exact [`Self::y_ticks`] take precedence in either setter order. Only the
    /// existing sealed numeric-y families admit this instruction. Automatic resets
    /// any previous interval or count; actual categorical axes reject other choices.
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn categorical(plot: Plot<kind::Beeswarm>, generation: TickGeneration) {
    ///     let _ = plot.y_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{Plot, TickGeneration};
    /// fn erased(plot: Plot, generation: TickGeneration) {
    ///     let _ = plot.y_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn matrix(plot: Plot<kind::Matrix>, generation: TickGeneration) {
    ///     let _ = plot.y_tick_generation(generation);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot, TickGeneration};
    /// fn heatmap(plot: Plot<kind::Heatmap>, generation: TickGeneration) {
    ///     let _ = plot.y_tick_generation(generation);
    /// }
    /// ```
    #[must_use]
    pub fn y_tick_generation(mut self, generation: crate::TickGeneration) -> Self
    where
        K: crate::kind::NumericY,
    {
        self.y_axis_spec.generation = generation;
        self
    }

    /// Replace generated numeric x-tick presentation; exact tick text takes precedence.
    /// Forest Log10 ticks format mathematical powers, retaining exponent-space positions.
    /// An actual categorical axis rejects this instruction before fonts.
    /// ```compile_fail
    /// use lichtung_plot::{kind, NumericFormat, Plot};
    /// fn unsupported(plot: Plot<kind::Matrix>, format: NumericFormat) {
    ///     let _ = plot.x_tick_format(format);
    /// }
    /// ```
    #[must_use]
    pub fn x_tick_format(mut self, format: NumericFormat) -> Self
    where
        K: crate::kind::NumericX,
    {
        self.x_axis_spec.format = Some(std::sync::Arc::new(format));
        self
    }

    /// Replace generated numeric y-tick presentation; exact tick text takes precedence.
    /// This changes displayed numbers without changing scientific coordinates.
    /// ```compile_fail
    /// use lichtung_plot::{kind, NumericFormat, Plot};
    /// fn unsupported(plot: Plot<kind::Waterfall>, format: NumericFormat) {
    ///     let _ = plot.y_tick_format(format);
    /// }
    /// ```
    #[must_use]
    pub fn y_tick_format(mut self, format: NumericFormat) -> Self
    where
        K: crate::kind::NumericY,
    {
        self.y_axis_spec.format = Some(std::sync::Arc::new(format));
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

    /// Replace the complete categorical palette/map selection for active raw identities.
    #[must_use]
    ///
    /// The retired map-only setter is unavailable:
    /// ```compile_fail
    /// use lichtung_plot::{kind, CategoricalColorScale, Plot};
    /// fn old_setter(plot: Plot<kind::Scatter>, scale: CategoricalColorScale) {
    ///     let _ = plot.categorical_colors(scale);
    /// }
    /// ```
    /// This attachment is not an accent-only or erased-family capability:
    /// ```compile_fail
    /// use lichtung_plot::{kind, CategoricalColorScale, Plot};
    /// fn wrong_family(plot: Plot<kind::Regression>, scale: CategoricalColorScale) {
    ///     let _ = plot.categorical_color_scale(scale);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{CategoricalColorScale, Plot};
    /// fn erased(plot: Plot, scale: CategoricalColorScale) {
    ///     let _ = plot.categorical_color_scale(scale);
    /// }
    /// ```
    pub fn categorical_color_scale(mut self, scale: crate::CategoricalColorScale) -> Self
    where
        K: crate::kind::CategoricalScale,
    {
        self.categorical_color_scale = Some(scale);
        self
    }

    /// Replace the complete continuous kind, optional gradient and optional exact domain.
    #[must_use]
    ///
    /// Exact limits belong to the complete typed selection:
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot};
    /// fn old_limits(plot: Plot<kind::Heatmap>) {
    ///     let _ = plot.color_limits(0.0, 1.0);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, Plot};
    /// fn old_limits(plot: Plot<kind::Beeswarm>) {
    ///     let _ = plot.color_limits(0.0, 1.0);
    /// }
    /// ```
    /// ```compile_fail
    /// use lichtung_plot::{kind, ContinuousColorScale, Plot};
    /// fn old_selector(plot: Plot<kind::Heatmap>, scale: ContinuousColorScale) {
    ///     let _ = plot.scale(scale);
    /// }
    /// ```
    pub fn continuous_color_scale(mut self, scale: crate::ContinuousColorScale) -> Self
    where
        K: crate::kind::ContinuousColor,
    {
        self.continuous_color_scale = Some(scale);
        self
    }

    /// Add a caller-owned text block at one corner inside the chart panel.
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
    /// If omitted, a standalone Plot uses [`Theme::publication`] and a Plot in
    /// a [`crate::Canvas`] inherits the Canvas root theme.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
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

    /// Allocate exactly this width in CSS pixels and derive height from content.
    ///
    /// Without a sizing call, both dimensions are derived from content. This
    /// replaces the complete request, clearing any height set by [`Self::size`].
    /// Zero is rejected during preflight, without a setter panic. An embedded
    /// Plot's standalone request does not control its Canvas cell.
    ///
    /// ```no_run
    /// use lichtung_plot::{NumericValues, Plot};
    /// let output = Plot::scatter(
    ///     NumericValues::new("x", [0.0, 1.0]),
    ///     NumericValues::new("y", [1.0, 0.0]),
    /// )
    /// .size(600, 1)
    /// .width(600)
    /// .svg_preserve_text("figure")?;
    /// assert_eq!(output.width(), 600);
    /// assert!(output.height() > 0);
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    #[must_use]
    pub const fn width(mut self, width: u32) -> Self {
        self.root_size = crate::layout::RootSize::width(width);
        self
    }

    /// Allocate exactly this width and height in CSS pixels.
    ///
    /// This replaces the complete sizing request. Content must fit without
    /// changing either dimension; zero dimensions reject during preflight.
    /// Final canonical dimensions are available on [`crate::Output`].
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.root_size = crate::layout::RootSize::new(width, height);
        self
    }

    /// Set the legend visibility and placement policy.
    ///
    /// Native Heatmap/Beeswarm keep their right guide track and Matrix its trailing
    /// grid track; Hidden suppresses their guide. Standard panels use the selected
    /// inside/outside edge. All exact families can explain declared data alpha.
    #[must_use]
    pub fn legend(mut self, policy: LegendPolicy) -> Self
    where
        K: crate::kind::Legend,
    {
        self.legend_policy = policy;
        self
    }

    /// Replace discrete guide composition; the default is Combined.
    ///
    /// This changes only section semantics, independently of theme, labels,
    /// visibility, placement and sizing. Facets use this owning Plot selection.
    /// Canvas Shared uses its own selection; PerPlot uses each Plot's value.
    /// See [`crate::LegendComposition::Separate`] for truthful factorization.
    /// ```
    /// use lichtung_plot::{CategoryValues, LegendComposition, NumericValues, Plot};
    /// let plot = Plot::scatter(NumericValues::new("x", [0.0, 1.0]),
    ///     NumericValues::new("y", [1.0, 0.0]))
    ///     .hue(CategoryValues::new("cohort", ["control", "treated"])?)
    ///     .legend_composition(LegendComposition::Separate);
    /// # let _ = plot;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    /// All exact families can explain declared alpha. Erased plots do not expose this setter:
    /// ```compile_fail
    /// use lichtung_plot::{LegendComposition, Plot};
    /// fn erased(plot: Plot) { let _ = plot.legend_composition(LegendComposition::Separate); }
    /// ```
    #[must_use]
    pub const fn legend_composition(mut self, composition: crate::LegendComposition) -> Self
    where
        K: crate::kind::Legend,
    {
        self.legend_composition = composition;
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
}

impl Plot<crate::kind::Forest> {
    /// Set the numeric axis scale used by a forest plot.
    #[must_use]
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.axis_scale = scale;
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
    /// Replace numeric presentation for window-eligible base and terminal totals.
    /// Verbatim names from [`Self::total_labels`] join the formatted number with one space.
    /// ```compile_fail
    /// use lichtung_plot::{kind, NumericFormat, Plot};
    /// fn unsupported(plot: Plot<kind::HorizontalBar>, format: NumericFormat) {
    ///     let _ = plot.total_format(format);
    /// }
    /// ```
    #[must_use]
    pub fn total_format(mut self, format: NumericFormat) -> Self {
        self.total_format = Some(std::sync::Arc::new(format));
        self
    }

    /// Set caller-owned labels for base and terminal annotations.
    #[must_use]
    pub fn total_labels(mut self, labels: WaterfallLabels) -> Self {
        self.waterfall_labels = Some(labels);
        self
    }
}
