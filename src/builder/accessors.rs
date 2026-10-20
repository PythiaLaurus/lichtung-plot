use crate::Error;

use super::Plot;
use crate::data::{
    DataColumn, BAR_INTERVAL_LOWER, BAR_INTERVAL_UPPER, BAR_UNITS, COLOR, DUMBBELL_RIGHT,
    FACET_COLUMN, FACET_ROW, FACET_WRAP, GROUP, HEATMAP_HIGHLIGHT, HUE, INTERVAL_LOWER,
    INTERVAL_UPPER, SERIES, X, Y,
};
use crate::input::{AxisValues, PlotInput};
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::layout::AxisTick;
use crate::render::backend::PointStyle;
use crate::spec::{
    AxisScale, BarInput, BarSpacing, Bootstrap, CategoryOrder, ConfidenceIntervalAlgorithm,
    DumbbellEndpoints, ErrorBarKind, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout,
    HeatmapNullRowAnnotation, HistogramBins, HistogramComposite, HistogramStat, LegendPolicy,
    LegendPosition, LinePathStyle, PanelTextAnnotation, PlotKind, QqLine, RawOverlay,
    RegressionBand, WaterfallLabels,
};
use crate::{HistogramGeometry, NumericFormat, RaincloudGeometry};

impl Plot {
    pub(crate) const fn data(&self) -> crate::data::PlotData<'_> {
        crate::data::PlotData::new(self)
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    pub(crate) fn theme_spec(&self) -> Option<&crate::theme::Theme> {
        self.theme.as_deref()
    }

    pub(crate) fn resolve_theme(&self) -> Result<crate::theme::ResolvedTheme, Error> {
        match self.theme_spec() {
            Some(theme) => theme.resolve(),
            None => crate::theme::Theme::publication().resolve(),
        }
    }

    pub(crate) fn source_row_map(&self) -> Option<&[usize]> {
        self.source_rows.as_ref().map(crate::SourceRows::as_slice)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.input.len()
    }

    fn series_values(&self) -> Option<&crate::CategoryValues> {
        self.input.required_series().or(self.series.as_ref())
    }

    fn group_values(&self) -> Option<&crate::CategoryValues> {
        self.input.required_group().or(self.group.as_ref())
    }

    fn color_values(&self) -> Option<&crate::NumericValues> {
        self.input.continuous_color()
    }

    pub(crate) fn x_col(&self) -> Option<&'static str> {
        self.input.x_name().map(|_| X)
    }

    pub(crate) fn y_col(&self) -> Option<&'static str> {
        self.input.y_name().map(|_| Y)
    }

    pub(crate) fn hue_col(&self) -> Option<&'static str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(|_| SERIES)
        } else {
            self.hue.as_ref().map(|_| HUE)
        }
    }

    pub(crate) fn hue_name(&self) -> Option<&str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(crate::CategoryValues::name)
        } else {
            self.hue.as_ref().map(crate::CategoryValues::name)
        }
    }

    pub(crate) fn group_col(&self) -> Option<&'static str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(|_| SERIES)
        } else {
            self.group_values().map(|_| GROUP)
        }
    }

    pub(crate) fn group_name(&self) -> Option<&str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(crate::CategoryValues::name)
        } else {
            self.group_values().map(crate::CategoryValues::name)
        }
    }

    pub(crate) fn color_col(&self) -> Option<&'static str> {
        self.color_values().map(|_| COLOR)
    }

    pub(crate) fn data_column(&self, key: &str) -> Option<DataColumn<'_>> {
        if key == X {
            return self.x_data_column();
        }
        if key == Y {
            return self.y_data_column();
        }
        match key {
            HUE => self.hue.as_ref().map(DataColumn::Category),
            GROUP => self.group_values().map(DataColumn::Category),
            SERIES => self.series_values().map(DataColumn::Category),
            COLOR => self.color_values().map(DataColumn::Numeric),
            HEATMAP_HIGHLIGHT => self.heatmap_highlight.as_ref().map(DataColumn::Boolean),
            INTERVAL_LOWER => self
                .drawable_bounds()
                .map(|bounds| DataColumn::Numeric(bounds.lower())),
            INTERVAL_UPPER => self
                .drawable_bounds()
                .map(|bounds| DataColumn::Numeric(bounds.upper())),
            BAR_INTERVAL_LOWER => self
                .bar_input()
                .and_then(BarInput::bounds)
                .map(|bounds| DataColumn::Numeric(bounds.lower())),
            BAR_INTERVAL_UPPER => self
                .bar_input()
                .and_then(BarInput::bounds)
                .map(|bounds| DataColumn::Numeric(bounds.upper())),
            BAR_UNITS => self
                .bar_input()
                .and_then(BarInput::error_bar)
                .and_then(|error_bar| match error_bar.kind() {
                    ErrorBarKind::ConfidenceInterval(interval) => match &interval.algorithm {
                        ConfidenceIntervalAlgorithm::Bootstrap(Bootstrap {
                            units: Some(units),
                            ..
                        }) => Some(DataColumn::Category(units)),
                        ConfidenceIntervalAlgorithm::Bootstrap(Bootstrap {
                            units: None, ..
                        })
                        | ConfidenceIntervalAlgorithm::StudentT => None,
                    },
                    ErrorBarKind::Hidden
                    | ErrorBarKind::StandardDeviation(_)
                    | ErrorBarKind::PercentileInterval(_)
                    | ErrorBarKind::StandardError(_) => None,
                }),
            DUMBBELL_RIGHT => match self.input.as_ref() {
                PlotInput::Dumbbell { right, .. } => Some(DataColumn::Numeric(right)),
                _ => None,
            },
            FACET_ROW => self
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::row_values)
                .map(DataColumn::Category),
            FACET_COLUMN => self
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::column_values)
                .map(DataColumn::Category),
            FACET_WRAP => self
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::wrap_values)
                .map(DataColumn::Category),
            _ => self
                .facet
                .as_ref()
                .and_then(|facet| facet.values().find(|values| values.name() == key))
                .map(DataColumn::Category)
                .or_else(|| match self.input.as_ref() {
                    PlotInput::Matrix { variables } => self
                        .matrix_variables
                        .iter()
                        .position(|name| name == key)
                        .and_then(|index| variables.get(index))
                        .map(DataColumn::Numeric),
                    _ => None,
                }),
        }
    }

    fn x_data_column(&self) -> Option<DataColumn<'_>> {
        match self.input.as_ref() {
            PlotInput::Scatter { x, .. }
            | PlotInput::Regression { x, .. }
            | PlotInput::Marginal { x, .. }
            | PlotInput::LinePath { x, .. } => Some(DataColumn::Numeric(x)),
            PlotInput::Histogram { input } => input.samples_input().map(DataColumn::Numeric),
            PlotInput::Density { values } | PlotInput::Qq { values } => {
                Some(DataColumn::Numeric(values))
            }
            PlotInput::PointInterval { estimate, .. } | PlotInput::Forest { estimate, .. } => {
                Some(DataColumn::Numeric(estimate))
            }
            PlotInput::Coefficient { input, .. } => Some(DataColumn::Numeric(input.values())),
            PlotInput::Line { x, .. } | PlotInput::Trajectory { x, .. } => match x {
                AxisValues::Numeric(values) => Some(DataColumn::Numeric(values)),
                AxisValues::Category(values) => Some(DataColumn::Category(values)),
            },
            PlotInput::Interaction { condition, .. }
            | PlotInput::Dumbbell {
                category: condition,
                ..
            }
            | PlotInput::Heatmap { x: condition, .. }
            | PlotInput::Bar {
                category: condition,
                ..
            }
            | PlotInput::HorizontalBar {
                category: condition,
                ..
            }
            | PlotInput::StackedBar {
                category: condition,
                ..
            }
            | PlotInput::StackedHorizontalBar {
                category: condition,
                ..
            }
            | PlotInput::Boxplot {
                category: condition,
                ..
            }
            | PlotInput::Violin {
                category: condition,
                ..
            }
            | PlotInput::Waterfall {
                labels: condition, ..
            } => Some(DataColumn::Category(condition)),
            PlotInput::Raincloud { input } => input
                .samples_input()
                .map(|(category, _)| DataColumn::Category(category)),
            PlotInput::Beeswarm { value, .. } => Some(DataColumn::Numeric(value)),
            PlotInput::Matrix { .. } => None,
        }
    }

    fn y_data_column(&self) -> Option<DataColumn<'_>> {
        match self.input.as_ref() {
            PlotInput::Scatter { y, .. }
            | PlotInput::Line { y, .. }
            | PlotInput::Trajectory { y, .. }
            | PlotInput::LinePath { y, .. }
            | PlotInput::Regression { y, .. }
            | PlotInput::Marginal { y, .. } => Some(DataColumn::Numeric(y)),
            PlotInput::PointInterval { category, .. }
            | PlotInput::Coefficient { term: category, .. }
            | PlotInput::Forest { term: category, .. }
            | PlotInput::Beeswarm { category, .. } => Some(DataColumn::Category(category)),
            PlotInput::Interaction { estimate, .. }
            | PlotInput::Dumbbell { left: estimate, .. }
            | PlotInput::StackedBar {
                value: estimate, ..
            }
            | PlotInput::StackedHorizontalBar {
                value: estimate, ..
            }
            | PlotInput::Boxplot {
                value: estimate, ..
            }
            | PlotInput::Violin {
                value: estimate, ..
            }
            | PlotInput::Waterfall {
                deltas: estimate, ..
            } => Some(DataColumn::Numeric(estimate)),
            PlotInput::Raincloud { input } => input
                .samples_input()
                .map(|(_, values)| DataColumn::Numeric(values)),
            PlotInput::Bar { input, .. } | PlotInput::HorizontalBar { input, .. } => {
                Some(DataColumn::Numeric(input.values()))
            }
            PlotInput::Heatmap { y, .. } => Some(DataColumn::Category(y)),
            PlotInput::Histogram { .. }
            | PlotInput::Density { .. }
            | PlotInput::Qq { .. }
            | PlotInput::Matrix { .. } => None,
        }
    }

    pub(crate) fn automatic_axis_category_values(
        &self,
        kind: PlotKind,
        x_axis: bool,
    ) -> Option<&crate::CategoryValues> {
        let column = if x_axis {
            match kind {
                PlotKind::Interaction
                | PlotKind::Heatmap
                | PlotKind::Bar
                | PlotKind::StackedBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Raincloud
                | PlotKind::Line
                | PlotKind::Trajectory
                | PlotKind::LinePath => self.x_data_column(),
                _ => None,
            }
        } else {
            match kind {
                PlotKind::PointInterval
                | PlotKind::Coefficient
                | PlotKind::Forest
                | PlotKind::Beeswarm
                | PlotKind::Heatmap => self.y_data_column(),
                PlotKind::Dumbbell
                | PlotKind::HorizontalBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Waterfall => self.x_data_column(),
                _ => None,
            }
        };
        match column {
            Some(DataColumn::Category(values)) => Some(values),
            Some(DataColumn::Numeric(_) | DataColumn::Boolean(_)) | None => None,
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive closed-input match keeps every family's aligned channels auditable"
    )]
    pub(crate) fn aligned_columns(&self) -> Vec<DataColumn<'_>> {
        let mut columns = match self.input.as_ref() {
            PlotInput::Scatter { x, y }
            | PlotInput::Regression { x, y }
            | PlotInput::Marginal { x, y } => {
                vec![DataColumn::Numeric(x), DataColumn::Numeric(y)]
            }
            PlotInput::Histogram { input } => input
                .samples_input()
                .map_or_else(Vec::new, |values| vec![DataColumn::Numeric(values)]),
            PlotInput::Density { values } | PlotInput::Qq { values } => {
                vec![DataColumn::Numeric(values)]
            }
            PlotInput::PointInterval {
                estimate, category, ..
            }
            | PlotInput::Forest {
                estimate,
                term: category,
                ..
            } => vec![
                DataColumn::Numeric(estimate),
                DataColumn::Category(category),
            ],
            PlotInput::Coefficient { input, term } => vec![
                DataColumn::Numeric(input.values()),
                DataColumn::Category(term),
            ],
            PlotInput::Line { x, y } => {
                let x = match x {
                    AxisValues::Numeric(values) => DataColumn::Numeric(values),
                    AxisValues::Category(values) => DataColumn::Category(values),
                };
                vec![x, DataColumn::Numeric(y)]
            }
            PlotInput::Trajectory { x, y, group } => {
                let x = match x {
                    AxisValues::Numeric(values) => DataColumn::Numeric(values),
                    AxisValues::Category(values) => DataColumn::Category(values),
                };
                vec![x, DataColumn::Numeric(y), DataColumn::Category(group)]
            }
            PlotInput::LinePath { x, y, group } => vec![
                DataColumn::Numeric(x),
                DataColumn::Numeric(y),
                DataColumn::Category(group),
            ],
            PlotInput::Interaction {
                condition,
                estimate,
                group,
                ..
            } => vec![
                DataColumn::Category(condition),
                DataColumn::Numeric(estimate),
                DataColumn::Category(group),
            ],
            PlotInput::Dumbbell {
                category,
                left,
                right,
                ..
            } => vec![
                DataColumn::Category(category),
                DataColumn::Numeric(left),
                DataColumn::Numeric(right),
            ],
            PlotInput::Heatmap { x, y, fill } => vec![
                DataColumn::Category(x),
                DataColumn::Category(y),
                DataColumn::Numeric(fill),
            ],
            PlotInput::Bar { category, input } | PlotInput::HorizontalBar { category, input } => {
                let mut columns = vec![
                    DataColumn::Category(category),
                    DataColumn::Numeric(input.values()),
                ];
                if let Some(bounds) = input.bounds() {
                    columns.push(DataColumn::Numeric(bounds.lower()));
                    columns.push(DataColumn::Numeric(bounds.upper()));
                }
                if let Some(DataColumn::Category(units)) = self.data_column(BAR_UNITS) {
                    columns.push(DataColumn::Category(units));
                }
                columns
            }
            PlotInput::StackedBar {
                category,
                value,
                series,
            }
            | PlotInput::StackedHorizontalBar {
                category,
                value,
                series,
            } => vec![
                DataColumn::Category(category),
                DataColumn::Numeric(value),
                DataColumn::Category(series),
            ],
            PlotInput::Boxplot { category, value } | PlotInput::Violin { category, value } => {
                vec![DataColumn::Category(category), DataColumn::Numeric(value)]
            }
            PlotInput::Beeswarm {
                category,
                value,
                color,
            } => vec![
                DataColumn::Category(category),
                DataColumn::Numeric(value),
                DataColumn::Numeric(color),
            ],
            PlotInput::Raincloud { input } => {
                input.samples_input().map_or_else(Vec::new, |input| {
                    vec![DataColumn::Category(input.0), DataColumn::Numeric(input.1)]
                })
            }
            PlotInput::Matrix { variables } => variables.iter().map(DataColumn::Numeric).collect(),
            PlotInput::Waterfall { labels, deltas, .. } => {
                vec![DataColumn::Category(labels), DataColumn::Numeric(deltas)]
            }
        };
        columns.extend(self.series.iter().map(DataColumn::Category));
        columns.extend(self.hue.iter().map(DataColumn::Category));
        columns.extend(self.group.iter().map(DataColumn::Category));
        if let Some(bounds) = self.drawable_bounds() {
            columns.push(DataColumn::Numeric(bounds.lower()));
            columns.push(DataColumn::Numeric(bounds.upper()));
        }
        columns.extend(self.heatmap_highlight.iter().map(DataColumn::Boolean));
        if let Some(facet) = &self.facet {
            columns.extend(facet.values().map(DataColumn::Category));
        }
        columns
    }

    /// Heatmap colorbar title, if explicitly configured.
    #[must_use]
    pub(crate) fn color_label_text(&self) -> Option<&str> {
        self.color_label.as_deref()
    }

    /// Requested heatmap coordinate layout.
    #[must_use]
    pub(crate) fn heatmap_layout_spec(&self) -> HeatmapLayout {
        self.heatmap_layout
    }

    /// Requested categorical heatmap label-fit policy.
    #[must_use]
    pub(crate) const fn heatmap_label_fit_spec(&self) -> HeatmapLabelFit {
        self.heatmap_label_fit
    }

    pub(crate) fn continuous_color_scale_spec(&self) -> Option<&crate::ContinuousColorScale> {
        self.continuous_color_scale.as_ref()
    }

    /// Resolve the complete caller selection against its owning default mapping.
    pub(crate) fn continuous_selection_in(
        &self,
        owner: &crate::theme::ColorScales,
    ) -> Result<crate::theme::ContinuousColorScale, crate::Error> {
        use crate::theme::ContinuousColorScale;
        let family = match self.kind() {
            crate::PlotKind::Heatmap => "heatmap",
            crate::PlotKind::Beeswarm => "beeswarm",
            _ => {
                return Err(crate::Error::Render(
                    "continuous selection requires Heatmap or Beeswarm".into(),
                ))
            }
        };
        let context = |error| match error {
            crate::Error::Data(message) => crate::Error::Data(format!("{family}: {message}")),
            error => error,
        };
        let default;
        let selection = if let Some(selection) = self.continuous_color_scale_spec() {
            selection
        } else {
            default = ContinuousColorScale::sequential(None, None)?;
            &default
        };
        selection.resolved_in(owner).map_err(context)
    }

    /// Requested heatmap annotation policy.
    #[must_use]
    pub(crate) const fn heatmap_annotations_spec(&self) -> &HeatmapAnnotations {
        &self.heatmap_annotations
    }

    /// Caller-owned annotations for fully-null heatmap rows.
    #[must_use]
    pub(crate) fn heatmap_null_row_annotations_spec(&self) -> &[HeatmapNullRowAnnotation] {
        &self.heatmap_null_row_annotations
    }

    pub(crate) fn heatmap_highlight_col(&self) -> Option<&'static str> {
        self.heatmap_highlight.as_ref().map(|_| HEATMAP_HIGHLIGHT)
    }

    /// Fixed numeric labels configured for point-only coefficient estimates.
    #[must_use]
    pub(crate) const fn estimate_labels_spec(&self) -> Option<&NumericFormat> {
        self.estimate_labels.as_ref()
    }

    pub(crate) const fn significance_mark_style_override(
        &self,
    ) -> Option<crate::SignificanceMarkStyle> {
        self.significance_mark_style
    }

    pub(crate) const fn category_order_spec(&self) -> &CategoryOrder {
        &self.category_orders.primary
    }

    pub(crate) const fn x_category_order_spec(&self) -> &CategoryOrder {
        &self.category_orders.x
    }

    pub(crate) const fn y_category_order_spec(&self) -> &CategoryOrder {
        &self.category_orders.y
    }

    /// Requested histogram bin-selection strategy.
    #[must_use]
    pub(crate) fn histogram_bins_spec(&self) -> HistogramBins {
        self.histogram_bins
    }

    /// Requested histogram vertical-axis normalization.
    #[must_use]
    pub(crate) fn histogram_stat_spec(&self) -> HistogramStat {
        self.histogram_stat
    }

    /// Requested optional histogram density-and-rug composition.
    #[must_use]
    pub(crate) fn histogram_composite_spec(&self) -> &HistogramComposite {
        &self.histogram_composite
    }

    pub(crate) fn histogram_composite_geometry_input(&self) -> Option<&HistogramGeometry> {
        match self.input.as_ref() {
            PlotInput::Histogram { input } => input.geometry_input(),
            _ => None,
        }
    }

    /// Requested normal Q-Q reference-line policy.
    #[must_use]
    pub(crate) fn qq_line_spec(&self) -> QqLine {
        self.qq_line
    }

    /// Whether values should be colored according to their sign.
    #[must_use]
    pub(crate) fn colors_by_sign(&self) -> bool {
        self.color_by_sign
    }

    /// Whether the renderer should draw a zero reference line.
    #[must_use]
    pub(crate) fn zero_line_visible(&self) -> bool {
        self.zero_line
    }

    pub(crate) fn interval_bound_cols(&self) -> Option<(&'static str, &'static str)> {
        self.drawable_bounds()
            .map(|_| (INTERVAL_LOWER, INTERVAL_UPPER))
    }

    fn drawable_bounds(&self) -> Option<&crate::Bounds> {
        match self.input.as_ref() {
            PlotInput::PointInterval { bounds, .. }
            | PlotInput::Interaction { bounds, .. }
            | PlotInput::Forest { bounds, .. } => Some(bounds),
            PlotInput::Coefficient { input, .. } => input.bounds(),
            _ => self.line_band.as_ref(),
        }
    }

    pub(crate) const fn line_band_configured(&self) -> bool {
        self.line_band.is_some()
    }

    pub(crate) fn bar_input(&self) -> Option<&BarInput> {
        match self.input.as_ref() {
            PlotInput::Bar { input, .. } | PlotInput::HorizontalBar { input, .. } => Some(input),
            _ => None,
        }
    }

    pub(crate) fn bar_bound_cols(&self) -> Option<(&'static str, &'static str)> {
        self.bar_input()
            .and_then(BarInput::bounds)
            .map(|_| (BAR_INTERVAL_LOWER, BAR_INTERVAL_UPPER))
    }

    pub(crate) fn bar_units_col(&self) -> Option<&'static str> {
        self.data_column(BAR_UNITS).map(|_| BAR_UNITS)
    }

    pub(crate) fn estimate_status_values(&self) -> Option<&[crate::EstimateStatus]> {
        self.estimate_status.as_deref()
    }

    pub(crate) const fn markers_visible(&self) -> bool {
        self.markers_visible
    }

    /// Configured confidence-band policy for generated regression plots.
    #[must_use]
    pub(crate) const fn regression_band_spec(&self) -> RegressionBand {
        self.regression_band
    }

    /// Configured complete-path color and legend policy.
    #[must_use]
    pub(crate) const fn line_path_style_spec(&self) -> LinePathStyle {
        self.line_path_style
    }

    /// Configured original-observation overlay.
    #[must_use]
    pub(crate) const fn raw_overlay_spec(&self) -> RawOverlay {
        self.raw_overlay
    }

    pub(crate) fn raincloud_geometry_input(&self) -> Option<&RaincloudGeometry> {
        match self.input.as_ref() {
            PlotInput::Raincloud { input } => input.geometry_input(),
            _ => None,
        }
    }

    pub(crate) const fn coefficient_reference_line_value(&self) -> Option<f64> {
        self.coefficient_reference_line
    }

    pub(crate) const fn forest_reference_value(&self) -> f64 {
        self.forest_reference
    }

    pub(crate) const fn axis_scale_spec(&self) -> AxisScale {
        self.axis_scale
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

    pub(crate) fn interaction_styles_spec(&self) -> Option<&[crate::InteractionStyle]> {
        self.interaction_styles.as_deref()
    }

    pub(crate) fn categorical_color_scale_spec(&self) -> Option<&crate::CategoricalColorScale> {
        self.categorical_color_scale.as_ref()
    }

    pub(crate) fn categorical_map_spec(&self) -> Option<&crate::CategoricalColorMap> {
        self.categorical_color_scale_spec()
            .and_then(crate::CategoricalColorScale::map)
    }

    /// Caller-owned panel-corner text annotations.
    #[must_use]
    pub(crate) fn panel_text_annotations(&self) -> &[PanelTextAnnotation] {
        &self.panel_text_annotations
    }

    pub(crate) fn waterfall_base(&self) -> Option<f64> {
        match self.input.as_ref() {
            PlotInput::Waterfall { base, .. } => Some(*base),
            _ => None,
        }
    }

    /// Caller-owned waterfall annotation labels, if configured.
    #[must_use]
    pub(crate) const fn waterfall_labels_spec(&self) -> Option<&WaterfallLabels> {
        self.waterfall_labels.as_ref()
    }

    pub(crate) fn total_format_spec(
        &self,
    ) -> Option<&std::sync::Arc<crate::numeric::format::NumericFormat>> {
        self.total_format.as_ref()
    }

    pub(crate) fn dumbbell_right_col(&self) -> Option<&'static str> {
        matches!(self.input.as_ref(), PlotInput::Dumbbell { .. }).then_some(DUMBBELL_RIGHT)
    }

    /// Required endpoint labels and colors for a dumbbell plot.
    #[must_use]
    pub(crate) fn dumbbell_endpoints_spec(&self) -> Option<&DumbbellEndpoints> {
        match self.input.as_ref() {
            PlotInput::Dumbbell { endpoints, .. } => Some(endpoints),
            _ => None,
        }
    }

    /// The chart title, if any.
    #[must_use]
    pub(crate) fn title_text(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The chart subtitle, if any.
    #[must_use]
    pub(crate) fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// The chart caption, if any.
    #[must_use]
    pub(crate) fn caption_text(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    pub(crate) fn resolve_label(&self, raw: &str) -> Result<String, Error> {
        self.labels.resolve(raw)
    }

    pub(crate) fn resolve_axis_label_domain<'b>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'b str>,
    ) -> Result<Vec<String>, Error> {
        self.labels.resolve_domain(domain, raw_labels)
    }

    pub(crate) fn prepare_axis_labels(
        &self,

        policy: Option<&crate::Labels>,
    ) -> Result<[String; 2], Error> {
        let mut labels = [String::new(), String::new()];
        for (target, (raw, inferred)) in labels.iter_mut().zip(self.axis_label_sources(self.kind)) {
            *target = if inferred {
                policy.unwrap_or(&self.labels).resolve(raw)?
            } else {
                crate::data::source::copy_source_string(raw)?
            };
        }
        Ok(labels)
    }

    pub(crate) fn resolve_label_domain<'b>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'b str>,
    ) -> Result<Vec<String>, crate::Error> {
        #[cfg(test)]
        if domain.contains("axis categories") {
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::AxisSourcePreparation,
            );
        }
        self.labels.resolve_domain(domain, raw_labels)
    }

    pub(crate) fn resolved_axis_labels(&self, kind: PlotKind) -> (String, String) {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        );
        let [x, y] = self.axis_label_sources(kind).map(|(raw, inferred)| {
            if inferred {
                self.resolve_label(raw).expect("label allocation")
            } else {
                raw.to_owned()
            }
        });
        (x, y)
    }

    fn axis_label_sources(&self, kind: PlotKind) -> [(&str, bool); 2] {
        let x_name = self.input.x_name().map_or(("", false), |name| (name, true));
        let y_name = self.input.y_name().map_or(("", false), |name| (name, true));
        let x = self.x_label.as_deref().map_or_else(
            || match kind {
                PlotKind::HorizontalBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Waterfall
                | PlotKind::Dumbbell => y_name,
                PlotKind::Qq => ("Theoretical normal quantiles", false),
                PlotKind::Histogram => self
                    .histogram_composite_geometry_input()
                    .map_or_else(|| x_name, |input| (&input.dimensions.value, true)),
                PlotKind::Raincloud => self
                    .raincloud_geometry_input()
                    .map_or_else(|| x_name, |input| (&input.dimensions.category, true)),
                PlotKind::Matrix => ("", false),
                _ => x_name,
            },
            |raw| (raw, false),
        );
        let y = self.y_label.as_deref().map_or_else(
            || match kind {
                PlotKind::HorizontalBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Waterfall
                | PlotKind::Dumbbell
                | PlotKind::Qq => x_name,
                PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                    || match self.histogram_stat {
                        HistogramStat::Count => ("Count", false),
                        HistogramStat::Probability => ("Probability", false),
                        HistogramStat::Density => ("Density", false),
                    },
                    |input| (&input.dimensions.density, true),
                ),
                PlotKind::Raincloud => self
                    .raincloud_geometry_input()
                    .map_or_else(|| y_name, |input| (&input.dimensions.value, true)),
                PlotKind::Density => ("Density", false),
                PlotKind::Matrix => ("", false),
                _ => y_name,
            },
            |raw| (raw, false),
        );
        [x, y]
    }

    pub(crate) fn prepare_color_label(
        &self,

        policy: Option<&crate::Labels>,
    ) -> Result<String, Error> {
        if let Some(label) = self.color_label_text() {
            crate::data::source::copy_source_string(label)
        } else if let Some(values) = self.color_values() {
            policy.unwrap_or(&self.labels).resolve(values.name())
        } else {
            Ok(String::new())
        }
    }

    pub(crate) fn color_dimension_key(&self) -> Option<&str> {
        self.color_values().map(crate::NumericValues::name)
    }

    pub(crate) fn legend_title_key(&self) -> &str {
        self.interaction_styles
            .as_ref()
            .and(self.group_values())
            .map(crate::CategoryValues::name)
            .or_else(|| self.hue_dimension_key())
            .unwrap_or("series")
    }

    pub(crate) fn collective_legend_identity(
        &self,
        pair: &crate::data::CollectivePair,
    ) -> Result<crate::decorations::legend::GuideEntryIdentity, Error> {
        match (
            self.group_name(),
            pair.group.as_deref(),
            self.hue_name(),
            pair.hue.as_deref(),
        ) {
            (Some(group_field), Some(group_value), Some(hue_field), Some(hue_value))
                if group_field == hue_field =>
            {
                if group_value != hue_value {
                    return Err(Error::Render(
                        "same-field collective legend identity contains different values".into(),
                    ));
                }
                crate::decorations::legend::GuideEntryIdentity::category(group_field, group_value)
            }
            (Some(group_field), Some(group_value), Some(hue_field), Some(hue_value)) => {
                crate::decorations::legend::GuideEntryIdentity::pair(
                    group_field,
                    group_value,
                    hue_field,
                    hue_value,
                )
            }
            (Some(group_field), Some(group_value), _, None) => {
                crate::decorations::legend::GuideEntryIdentity::category(group_field, group_value)
            }
            (_, None, Some(hue_field), Some(hue_value)) => {
                crate::decorations::legend::GuideEntryIdentity::category(hue_field, hue_value)
            }
            _ => Err(Error::Render(
                "collective legend identity has no resolved group or hue value".into(),
            )),
        }
    }

    /// Payload identities own both strings under the projection's gate.
    pub(crate) fn point_legend_identity(
        &self,
        label: &str,
    ) -> Result<crate::decorations::legend::GuideEntryIdentity, Error> {
        if self.interaction_styles.is_none() && self.hue_dimension_key().is_none() {
            return Err(Error::Render(
                "categorical guide entry has no prepared field".into(),
            ));
        }
        crate::decorations::legend::GuideEntryIdentity::category(self.legend_title_key(), label)
    }

    pub(crate) fn hue_dimension_key(&self) -> Option<&str> {
        self.hue_name().or_else(|| {
            self.raincloud_geometry_input()
                .and_then(|input| input.dimensions.hue.as_deref())
        })
    }

    pub(crate) fn categorical_color_field(&self) -> Option<&str> {
        if self.colors_by_sign() {
            None
        } else {
            self.hue_dimension_key().or_else(|| {
                (self.kind() == crate::PlotKind::Interaction)
                    .then(|| self.group_name())
                    .flatten()
            })
        }
    }

    /// Borrow x-axis scale configuration.
    #[must_use]
    pub(crate) fn x_axis_spec(&self) -> &AxisSpec {
        &self.x_axis_spec
    }

    pub(crate) fn prepared_axis_inputs(&self) -> Option<&crate::plots::shared_x::PreparedAxes> {
        self.prepared_axes
            .iter()
            .find(|(scope, _)| scope == &self.render_scope)
            .map(|(_, axes)| axes)
    }

    pub(crate) fn prepared_discrete_guide(
        &self,
    ) -> Result<
        Option<std::sync::Arc<crate::decorations::legend::discrete::PreparedDiscreteGuide>>,
        Error,
    > {
        self.prepared_discrete_guides
            .as_ref()
            .ok_or_else(|| Error::Render("discrete guide semantics were not prepared".into()))?
            .for_scope(self.axis_scope())
            .map(Option::<&_>::cloned)
    }

    #[cfg(test)]
    pub(crate) fn with_test_axis_ticks(mut self, side: usize, ticks: Vec<crate::AxisTick>) -> Self {
        if side == 0 {
            self.x_axis_spec.exact_ticks = Some(ticks);
        } else {
            self.y_axis_spec.exact_ticks = Some(ticks);
        }
        self
    }

    pub(crate) fn axis_scope(&self) -> Option<&str> {
        self.render_scope.as_deref()
    }

    pub(crate) fn explicit_x_ticks(&self) -> Option<&[AxisTick]> {
        self.x_axis_spec.exact_ticks.as_deref()
    }

    /// Borrow y-axis scale configuration.
    #[must_use]
    pub(crate) fn y_axis_spec(&self) -> &AxisSpec {
        &self.y_axis_spec
    }

    pub(crate) fn explicit_y_ticks(&self) -> Option<&[AxisTick]> {
        self.y_axis_spec.exact_ticks.as_deref()
    }

    pub(crate) fn guides(&self) -> &[GuideSpec] {
        &self.guides
    }

    pub(crate) fn point_layers(&self) -> &[PointLayer] {
        &self.point_layers
    }

    pub(crate) fn bar_point_layers(&self) -> &[BarPointLayer] {
        &self.bar_point_layers
    }

    pub(crate) fn significance_annotations(
        &self,
    ) -> &[crate::significance::SignificanceAnnotation] {
        &self.significance
    }

    pub(crate) const fn sizing_request(&self) -> crate::layout::RootSize {
        self.root_size
    }

    /// Assigned geometry is available only after the root or parent allocates it.
    pub(crate) fn canvas_dimensions(&self) -> Result<(u32, u32), Error> {
        self.root_size.dimensions()
    }

    /// Borrow the ordered variable names for `PlotKind::Matrix`.
    #[must_use]
    pub(crate) fn matrix_variable_names(&self) -> &[String] {
        &self.matrix_variables
    }

    /// Grouped-bar spacing preset.
    #[must_use]
    pub(crate) fn bar_spacing_spec(&self) -> BarSpacing {
        self.bar_spacing
    }

    /// Build a point style using this chart's point-outline policy.
    #[must_use]
    pub(crate) fn point_style(
        &self,
        theme: &crate::theme::ResolvedTheme,
        color: (u8, u8, u8),
        radius: u32,
    ) -> PointStyle {
        theme
            .marks
            .points()
            .primary()
            .backend_style(color, radius, self.point_outline_visible)
    }

    /// Materialize one selected distribution role with the same outline policy as its guide.
    pub(crate) fn distribution_point_style(
        &self,
        role: &crate::theme::mark::SizedPointTheme,
        color: (u8, u8, u8),
    ) -> PointStyle {
        role.backend_style(color, self.point_outline_visible)
    }

    /// Selected semantic plot kind.
    #[must_use]
    pub const fn kind(&self) -> PlotKind {
        self.kind
    }

    pub(crate) const fn legend_policy_spec(&self) -> LegendPolicy {
        self.legend_policy
    }

    pub(crate) const fn effective_legend_composition(
        &self,
    ) -> crate::decorations::legend::discrete::LegendComposition {
        match self.parent_legend_composition {
            Some(value) => value,
            None => self.legend_composition,
        }
    }

    pub(crate) const fn with_parent_legend_composition(
        mut self,
        value: crate::decorations::legend::discrete::LegendComposition,
    ) -> Self {
        self.parent_legend_composition = Some(value);
        self
    }

    pub(crate) const fn legend_position_spec(&self) -> LegendPosition {
        self.legend_policy.position()
    }

    pub(crate) fn with_legend_position(mut self, position: LegendPosition) -> Self {
        self.legend_policy = LegendPolicy::Visible(position);
        self
    }

    pub(crate) fn with_suppressed_legend(mut self) -> Self {
        self.legend_policy = LegendPolicy::Hidden;
        self
    }

    pub(crate) fn with_suppressed_continuous_guide(mut self) -> Self {
        self.suppress_continuous_guide = true;
        self
    }

    pub(crate) const fn continuous_guide_suppressed(&self) -> bool {
        self.suppress_continuous_guide
    }

    pub(crate) fn color_label_format_spec(
        &self,
    ) -> Option<&std::sync::Arc<crate::numeric::format::NumericFormat>> {
        self.color_label_format.as_ref()
    }

    pub(crate) const fn parent_owns_axis_gutters(&self) -> bool {
        matches!(self.axis_gutter_owner, super::AxisGutterOwner::Composition)
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
