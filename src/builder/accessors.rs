use crate::Error;

use super::Plot;
use crate::data::{
    DataColumn, BAR_INTERVAL_LOWER, BAR_INTERVAL_UPPER, BAR_UNITS, COLOR, DUMBBELL_RIGHT,
    FACET_COLUMN, FACET_ROW, FACET_WRAP, GROUP, HEATMAP_HIGHLIGHT, HUE, INTERACTION_LINE_PATTERN,
    INTERACTION_POINT_SHAPE, INTERVAL_LOWER, INTERVAL_UPPER, SERIES, STATUS, X, Y,
};
use crate::input::{AxisValues, PlotInput};
use crate::layers::{BarPointLayer, GuideSpec, PointLayer};
use crate::layout::AxisSpec;
use crate::layout::AxisTick;
use crate::render::backend::PointStyle;
use crate::spec::{
    AxisScale, BarInput, BarSpacing, Bootstrap, CategoricalColors, CategoryOrder,
    ConfidenceIntervalAlgorithm, DumbbellEndpoints, ErrorBarKind, HeatmapAnnotations,
    HeatmapLabelFit, HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale, HistogramBins,
    HistogramComposite, HistogramStat, LegendPosition, LinePathStyle, NumericLabelFormat,
    PanelTextAnnotation, PlotKind, QqLine, RawOverlay, RegressionBand, WaterfallLabels,
};
use crate::Labels;
use crate::{HistogramCompositeGeometryInput, RaincloudGeometryInput};

impl Plot {
    pub(crate) const fn data(&self) -> crate::data::PlotData<'_> {
        crate::data::PlotData::new(self)
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    pub(crate) fn source_row_map(&self) -> Option<&[usize]> {
        self.source_rows.as_ref().map(crate::SourceRows::as_slice)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.input.len()
    }

    pub(crate) fn x_col(&self) -> Option<&'static str> {
        self.input.x_name().map(|_| X)
    }

    pub(crate) fn y_col(&self) -> Option<&'static str> {
        self.input.y_name().map(|_| Y)
    }

    pub(crate) fn hue_col(&self) -> Option<&'static str> {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series.as_ref().map(|_| SERIES)
        } else {
            self.hue.as_ref().map(|_| HUE)
        }
    }

    pub(crate) fn hue_name(&self) -> Option<&str> {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series.as_ref().map(crate::CategoryValues::name)
        } else {
            self.hue.as_ref().map(crate::CategoryValues::name)
        }
    }

    pub(crate) fn group_col(&self) -> Option<&'static str> {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series.as_ref().map(|_| SERIES)
        } else {
            self.group.as_ref().map(|_| GROUP)
        }
    }

    pub(crate) fn group_name(&self) -> Option<&str> {
        if self.kind.is_some_and(crate::PlotKind::uses_unified_series) {
            self.series.as_ref().map(crate::CategoryValues::name)
        } else {
            self.group.as_ref().map(crate::CategoryValues::name)
        }
    }

    pub(crate) fn color_col(&self) -> Option<&'static str> {
        self.color.as_ref().map(|_| COLOR)
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
            GROUP => self.group.as_ref().map(DataColumn::Category),
            SERIES => self.series.as_ref().map(DataColumn::Category),
            COLOR => self.color.as_ref().map(DataColumn::Numeric),
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
            STATUS => self.estimate_status.as_ref().map(DataColumn::Category),
            INTERACTION_POINT_SHAPE => self
                .interaction_styles
                .as_ref()
                .map(|(values, _)| DataColumn::Category(values)),
            INTERACTION_LINE_PATTERN => self
                .interaction_styles
                .as_ref()
                .map(|(_, values)| DataColumn::Category(values)),
            DUMBBELL_RIGHT => self.dumbbell_right.as_ref().map(DataColumn::Numeric),
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
                .or_else(|| match &self.input {
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
        match &self.input {
            PlotInput::Scatter { x, .. }
            | PlotInput::Regression { x, .. }
            | PlotInput::Marginal { x, .. } => Some(DataColumn::Numeric(x)),
            PlotInput::Histogram { input } => input.samples_input().map(DataColumn::Numeric),
            PlotInput::Density { values } | PlotInput::Qq { values } => {
                Some(DataColumn::Numeric(values))
            }
            PlotInput::PointInterval { estimate, .. } | PlotInput::Forest { estimate, .. } => {
                Some(DataColumn::Numeric(estimate))
            }
            PlotInput::Coefficient { input, .. } => Some(DataColumn::Numeric(input.values())),
            PlotInput::Line { x, .. } => match x {
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
        match &self.input {
            PlotInput::Scatter { y, .. }
            | PlotInput::Line { y, .. }
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

    pub(crate) fn aligned_columns(&self) -> Vec<DataColumn<'_>> {
        let mut columns = match &self.input {
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
            PlotInput::Interaction {
                condition,
                estimate,
                ..
            } => vec![
                DataColumn::Category(condition),
                DataColumn::Numeric(estimate),
            ],
            PlotInput::Dumbbell { category, left } => {
                vec![DataColumn::Category(category), DataColumn::Numeric(left)]
            }
            PlotInput::Heatmap { x, y } => {
                vec![DataColumn::Category(x), DataColumn::Category(y)]
            }
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
            PlotInput::StackedBar { category, value }
            | PlotInput::StackedHorizontalBar { category, value }
            | PlotInput::Boxplot { category, value }
            | PlotInput::Violin { category, value }
            | PlotInput::Beeswarm { category, value } => {
                vec![DataColumn::Category(category), DataColumn::Numeric(value)]
            }
            PlotInput::Raincloud { input } => {
                input.samples_input().map_or_else(Vec::new, |input| {
                    vec![DataColumn::Category(input.0), DataColumn::Numeric(input.1)]
                })
            }
            PlotInput::Matrix { variables } => variables.iter().map(DataColumn::Numeric).collect(),
            PlotInput::Waterfall { labels, deltas } => {
                vec![DataColumn::Category(labels), DataColumn::Numeric(deltas)]
            }
        };
        columns.extend(self.series.iter().map(DataColumn::Category));
        columns.extend(self.hue.iter().map(DataColumn::Category));
        columns.extend(self.group.iter().map(DataColumn::Category));
        columns.extend(self.color.iter().map(DataColumn::Numeric));
        if let Some(bounds) = self.drawable_bounds() {
            columns.push(DataColumn::Numeric(bounds.lower()));
            columns.push(DataColumn::Numeric(bounds.upper()));
        }
        columns.extend(self.estimate_status.iter().map(DataColumn::Category));
        columns.extend(self.heatmap_highlight.iter().map(DataColumn::Boolean));
        if let Some(facet) = &self.facet {
            columns.extend(facet.values().map(DataColumn::Category));
        }
        columns.extend(self.dumbbell_right.iter().map(DataColumn::Numeric));
        if let Some((point_shapes, line_patterns)) = &self.interaction_styles {
            columns.push(DataColumn::Category(point_shapes));
            columns.push(DataColumn::Category(line_patterns));
        }
        columns
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

    pub(crate) fn heatmap_highlight_col(&self) -> Option<&'static str> {
        self.heatmap_highlight.as_ref().map(|_| HEATMAP_HIGHLIGHT)
    }

    /// Fixed numeric labels configured for point-only coefficient estimates.
    #[must_use]
    pub const fn estimate_labels_spec(&self) -> Option<&NumericLabelFormat> {
        self.estimate_labels.as_ref()
    }

    pub(crate) const fn significance_mark_style_spec(&self) -> crate::SignificanceMarkStyle {
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
        match &self.input {
            PlotInput::Histogram { input } => input.geometry_input(),
            _ => None,
        }
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

    pub(crate) fn interval_bound_cols(&self) -> Option<(&'static str, &'static str)> {
        self.drawable_bounds()
            .map(|_| (INTERVAL_LOWER, INTERVAL_UPPER))
    }

    fn drawable_bounds(&self) -> Option<&crate::Bounds> {
        match &self.input {
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

    pub(crate) const fn bar_input(&self) -> Option<&BarInput> {
        match &self.input {
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

    pub(crate) fn estimate_status_col(&self) -> Option<&'static str> {
        self.estimate_status.as_ref().map(|_| STATUS)
    }

    pub(crate) const fn markers_visible(&self) -> bool {
        self.markers_visible
    }

    /// Configured confidence-band policy for generated regression plots.
    #[must_use]
    pub const fn regression_band_spec(&self) -> RegressionBand {
        self.regression_band
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

    pub(crate) fn raw_overlay_source_row_map(&self) -> Option<&[usize]> {
        self.raw_overlay_source_rows
            .as_ref()
            .map(crate::SourceRows::as_slice)
    }

    pub(crate) const fn raincloud_geometry_input(&self) -> Option<&RaincloudGeometryInput> {
        match &self.input {
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

    pub(crate) fn interaction_style_cols(&self) -> Option<(&'static str, &'static str)> {
        self.interaction_styles
            .as_ref()
            .map(|_| (INTERACTION_POINT_SHAPE, INTERACTION_LINE_PATTERN))
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

    pub(crate) fn waterfall_totals(&self) -> Option<(f64, f64)> {
        self.waterfall_base.zip(self.waterfall_terminal)
    }

    /// Caller-owned waterfall annotation labels, if configured.
    #[must_use]
    pub const fn waterfall_labels_spec(&self) -> Option<&WaterfallLabels> {
        self.waterfall_labels.as_ref()
    }

    pub(crate) fn dumbbell_right_col(&self) -> Option<&'static str> {
        self.dumbbell_right.as_ref().map(|_| DUMBBELL_RIGHT)
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
        let x_name = self.input.x_name();
        let y_name = self.input.y_name();
        let x = self.x_label.clone().unwrap_or_else(|| match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell => inferred(y_name),
            PlotKind::Qq => "Theoretical normal quantiles".to_owned(),
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || inferred(x_name),
                |input| self.resolve_label(&input.dimensions.value),
            ),
            PlotKind::Raincloud => self.raincloud_geometry_input().map_or_else(
                || inferred(x_name),
                |input| self.resolve_label(&input.dimensions.category),
            ),
            PlotKind::Matrix => String::new(),
            _ => inferred(x_name),
        });
        let y = self.y_label.clone().unwrap_or_else(|| match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell
            | PlotKind::Qq => inferred(x_name),
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || match self.histogram_stat {
                    HistogramStat::Count => "Count".to_owned(),
                    HistogramStat::Probability => "Probability".to_owned(),
                    HistogramStat::Density => "Density".to_owned(),
                },
                |input| self.resolve_label(&input.dimensions.density),
            ),
            PlotKind::Raincloud => self.raincloud_geometry_input().map_or_else(
                || inferred(y_name),
                |input| self.resolve_label(&input.dimensions.value),
            ),
            PlotKind::Density => "Density".to_owned(),
            PlotKind::Matrix => String::new(),
            _ => inferred(y_name),
        });
        (x, y)
    }

    pub(crate) fn resolved_color_label(&self) -> String {
        self.color_label.clone().unwrap_or_else(|| {
            self.color
                .as_ref()
                .map_or_else(String::new, |values| self.resolve_label(values.name()))
        })
    }

    pub(crate) fn resolved_legend_title(&self) -> String {
        if self.interaction_styles.is_some() || self.hue_dimension_key().is_some() {
            self.resolve_label(self.legend_title_key())
        } else {
            "series".to_owned()
        }
    }

    pub(crate) fn legend_title_key(&self) -> &str {
        self.interaction_styles
            .as_ref()
            .and(self.group.as_ref())
            .map(crate::CategoryValues::name)
            .or_else(|| self.hue_dimension_key())
            .unwrap_or("series")
    }

    pub(crate) fn hue_dimension_key(&self) -> Option<&str> {
        self.hue_name().or_else(|| {
            self.raincloud_geometry_input()
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

    /// Build a point style using this chart's point-outline policy.
    #[must_use]
    pub(crate) fn point_style(&self, color: (u8, u8, u8), radius: u32) -> PointStyle {
        PointStyle::with_outline(color, radius, self.point_outline)
    }

    /// Active plot kind, if a terminal method (`.scatter()` /
    /// `.bar()` / `.boxplot()`) has been called.
    #[must_use]
    pub fn plot_kind(&self) -> Option<PlotKind> {
        self.kind
    }

    /// Currently requested legend placement.
    #[must_use]
    pub fn legend_position_spec(&self) -> LegendPosition {
        self.legend_position
    }

    pub(crate) fn legend_visibility(&self) -> Option<bool> {
        self.legend_visible
    }

    pub(crate) fn with_legend_position(mut self, position: LegendPosition) -> Self {
        self.legend_position = position;
        self.legend_visible = Some(position != LegendPosition::Hidden);
        self
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

    pub(crate) const fn output_options(&self) -> (crate::PngOptions, crate::SvgText) {
        (self.png_options, self.svg_text)
    }
}
