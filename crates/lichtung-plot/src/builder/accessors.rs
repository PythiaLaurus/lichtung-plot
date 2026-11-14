use crate::Error;

use super::{Plot, PlotDescription};
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
    DumbbellEndpoints, ErrorBarKind, HeatmapAnnotations, HeatmapLayout, HeatmapNullRowAnnotation,
    HistogramBins, HistogramComposite, HistogramStat, LegendPolicy, LegendPosition, LinePathStyle,
    PlotKind, PlotTextAnnotation, QqLine, RawOverlay, RegressionBand, WaterfallLabels,
};
use crate::{HistogramGeometry, NumericFormat, RaincloudGeometry};

impl PlotDescription {
    pub(crate) fn text_layouts(&self) -> &crate::text::roles::TextRoles {
        &self.payload.text_layouts
    }

    pub(crate) fn theme_spec(&self) -> Option<&crate::theme::Theme> {
        self.payload.theme.as_deref()
    }

    pub(crate) fn resolve_theme(&self) -> Result<crate::theme::ResolvedTheme, Error> {
        let mut theme = match self.theme_spec() {
            Some(theme) => theme.resolve(),
            None => crate::theme::Theme::publication().resolve(),
        }?;
        theme.text_layouts = self.payload.text_layouts.clone();
        theme.apply_text_defaults(self.kind());
        Ok(theme)
    }

    pub(crate) fn source_row_map(&self) -> Option<&crate::SourceRows> {
        self.payload.source_rows.as_ref()
    }

    pub(crate) fn configured_source_rows(&self) -> Result<Option<&crate::SourceRows>, Error> {
        if let Some(rows) = self.source_row_map() {
            if rows.len() != self.row_count() {
                return Err(Error::Data(format!(
                    "chart source-row map has length {}, but chart data has {} rows",
                    rows.len(),
                    self.row_count()
                )));
            }
        }
        Ok(self.source_row_map())
    }

    pub(crate) fn row_count(&self) -> usize {
        self.payload.input.len()
    }

    fn series_values(&self) -> Option<&crate::CategoryValues> {
        self.payload
            .input
            .required_series()
            .or(self.payload.series.as_ref())
    }

    fn group_values(&self) -> Option<&crate::CategoryValues> {
        self.payload
            .input
            .required_group()
            .or(self.payload.group.as_ref())
    }

    fn color_values(&self) -> Option<&crate::NumericValues> {
        self.payload.input.continuous_color()
    }

    pub(crate) fn x_col(&self) -> Option<&'static str> {
        self.payload.input.x_name().map(|_| X)
    }

    pub(crate) fn y_col(&self) -> Option<&'static str> {
        self.payload.input.y_name().map(|_| Y)
    }

    pub(crate) fn hue_col(&self) -> Option<&'static str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(|_| SERIES)
        } else {
            self.payload.hue.as_ref().map(|_| HUE)
        }
    }

    pub(crate) fn hue_name(&self) -> Option<&str> {
        if self.kind.uses_unified_series() {
            self.series_values().map(crate::CategoryValues::name)
        } else {
            self.payload.hue.as_ref().map(crate::CategoryValues::name)
        }
    }

    pub(crate) fn hue_field(&self) -> Option<crate::data::category::RawField> {
        if self.kind.uses_unified_series() {
            self.series_values().map(crate::CategoryValues::raw_field)
        } else {
            self.payload
                .hue
                .as_ref()
                .map(crate::CategoryValues::raw_field)
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
            HUE => self.payload.hue.as_ref().map(DataColumn::Category),
            GROUP => self.group_values().map(DataColumn::Category),
            SERIES => self.series_values().map(DataColumn::Category),
            COLOR => self.color_values().map(DataColumn::Numeric),
            HEATMAP_HIGHLIGHT => self
                .payload
                .heatmap_highlight
                .as_ref()
                .map(DataColumn::Boolean),
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
            DUMBBELL_RIGHT => match self.payload.input.as_ref() {
                PlotInput::Dumbbell { right, .. } => Some(DataColumn::Numeric(right)),
                _ => None,
            },
            FACET_ROW => self
                .payload
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::row_values)
                .map(DataColumn::Category),
            FACET_COLUMN => self
                .payload
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::column_values)
                .map(DataColumn::Category),
            FACET_WRAP => self
                .payload
                .facet
                .as_ref()
                .and_then(crate::FacetSpec::wrap_values)
                .map(DataColumn::Category),
            _ => self
                .payload
                .facet
                .as_ref()
                .and_then(|facet| facet.values().find(|values| values.name() == key))
                .map(DataColumn::Category)
                .or_else(|| match self.payload.input.as_ref() {
                    PlotInput::Matrix { variables } => variables
                        .iter()
                        .find(|values| values.name() == key)
                        .map(DataColumn::Numeric),
                    _ => None,
                }),
        }
    }

    fn x_data_column(&self) -> Option<DataColumn<'_>> {
        match self.payload.input.as_ref() {
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
        match self.payload.input.as_ref() {
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
        let mut columns = match self.payload.input.as_ref() {
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
        columns.extend(self.payload.series.iter().map(DataColumn::Category));
        columns.extend(self.payload.hue.iter().map(DataColumn::Category));
        columns.extend(self.payload.group.iter().map(DataColumn::Category));
        if let Some(bounds) = self.drawable_bounds() {
            columns.push(DataColumn::Numeric(bounds.lower()));
            columns.push(DataColumn::Numeric(bounds.upper()));
        }
        columns.extend(
            self.payload
                .heatmap_highlight
                .iter()
                .map(DataColumn::Boolean),
        );
        if let Some(facet) = &self.payload.facet {
            columns.extend(facet.values().map(DataColumn::Category));
        }
        columns
    }

    /// Heatmap colorbar title, if explicitly configured.
    #[must_use]
    pub(crate) fn color_label_text(&self) -> Option<&str> {
        self.payload.color_label.provided().map(crate::Text::as_str)
    }

    pub(crate) fn copy_color_title(&self) -> Result<crate::Text, Error> {
        self.payload.color_label.try_copy()
    }

    pub(crate) fn same_color_title(&self, title: &crate::Text) -> bool {
        self.payload.color_label == *title
    }

    /// Requested heatmap coordinate layout.
    #[must_use]
    pub(crate) fn heatmap_layout_spec(&self) -> HeatmapLayout {
        self.payload.heatmap_layout
    }

    /// Requested categorical heatmap label-fit policy.
    #[must_use]
    pub(crate) fn continuous_color_scale_spec(&self) -> Option<&crate::ContinuousColorScale> {
        self.payload.continuous_color_scale.as_ref()
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
    pub(crate) fn heatmap_annotations_spec(&self) -> &HeatmapAnnotations {
        &self.payload.heatmap_annotations
    }

    /// Caller-owned annotations for fully-null heatmap rows.
    #[must_use]
    pub(crate) fn heatmap_null_row_annotations_spec(&self) -> &[HeatmapNullRowAnnotation] {
        &self.payload.heatmap_null_row_annotations
    }

    pub(crate) fn heatmap_highlight_col(&self) -> Option<&'static str> {
        self.payload
            .heatmap_highlight
            .as_ref()
            .map(|_| HEATMAP_HIGHLIGHT)
    }

    /// Fixed numeric labels configured for point-only coefficient estimates.
    #[must_use]
    pub(crate) fn estimate_labels_spec(&self) -> Option<&NumericFormat> {
        self.payload.estimate_labels.as_ref()
    }

    pub(crate) const fn significance_mark_style_override(
        &self,
    ) -> Option<crate::SignificanceMarkStyle> {
        self.significance_mark_style
    }

    pub(crate) fn category_order_spec(&self) -> &CategoryOrder {
        &self.payload.category_orders.primary
    }

    pub(crate) fn x_category_order_spec(&self) -> &CategoryOrder {
        &self.payload.category_orders.x
    }

    pub(crate) fn y_category_order_spec(&self) -> &CategoryOrder {
        &self.payload.category_orders.y
    }

    /// Requested histogram bin-selection strategy.
    #[must_use]
    pub(crate) fn histogram_bins_spec(&self) -> HistogramBins {
        self.payload.histogram_bins
    }

    /// Requested histogram vertical-axis normalization.
    #[must_use]
    pub(crate) fn histogram_stat_spec(&self) -> HistogramStat {
        self.payload.histogram_stat
    }

    /// Requested optional histogram density-and-rug composition.
    #[must_use]
    pub(crate) fn histogram_composite_spec(&self) -> &HistogramComposite {
        &self.payload.histogram_composite
    }

    pub(crate) fn histogram_composite_geometry_input(&self) -> Option<&HistogramGeometry> {
        match self.payload.input.as_ref() {
            PlotInput::Histogram { input } => input.geometry_input(),
            _ => None,
        }
    }

    /// Requested normal Q-Q reference-line policy.
    #[must_use]
    pub(crate) fn qq_line_spec(&self) -> QqLine {
        self.payload.qq_line
    }

    /// Whether values should be colored according to their sign.
    #[must_use]
    pub(crate) fn colors_by_sign(&self) -> bool {
        self.payload.color_by_sign
    }

    /// Whether the renderer should draw a zero reference line.
    #[must_use]
    pub(crate) fn zero_line_visible(&self) -> bool {
        self.payload.zero_line
    }

    pub(crate) fn interval_bound_cols(&self) -> Option<(&'static str, &'static str)> {
        self.drawable_bounds()
            .map(|_| (INTERVAL_LOWER, INTERVAL_UPPER))
    }

    fn drawable_bounds(&self) -> Option<&crate::Bounds> {
        match self.payload.input.as_ref() {
            PlotInput::PointInterval { bounds, .. }
            | PlotInput::Interaction { bounds, .. }
            | PlotInput::Forest { bounds, .. } => Some(bounds),
            PlotInput::Coefficient { input, .. } => input.bounds(),
            _ => self.payload.line_band.as_ref(),
        }
    }

    pub(crate) fn line_band_configured(&self) -> bool {
        self.payload.line_band.is_some()
    }

    pub(crate) fn bar_input(&self) -> Option<&BarInput> {
        match self.payload.input.as_ref() {
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
        self.payload.estimate_status.as_deref()
    }

    pub(crate) fn markers_visible(&self) -> bool {
        self.payload.markers_visible
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
        match self.payload.input.as_ref() {
            PlotInput::Raincloud { input } => input.geometry_input(),
            _ => None,
        }
    }

    pub(crate) fn coefficient_reference_line_value(&self) -> Option<f64> {
        self.payload.coefficient_reference_line
    }

    pub(crate) fn forest_reference_value(&self) -> f64 {
        match self.payload.input.as_ref() {
            PlotInput::Forest { reference, .. } => *reference,
            _ => 0.0,
        }
    }

    pub(crate) fn axis_scale_spec(&self) -> AxisScale {
        self.payload.axis_scale
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
        self.payload.interaction_styles.as_deref()
    }

    pub(crate) fn categorical_color_scale_spec(&self) -> Option<&crate::CategoricalColorScale> {
        self.payload.categorical_color_scale.as_ref()
    }

    pub(crate) fn categorical_map_spec(&self) -> Option<&crate::CategoricalColorMap> {
        self.categorical_color_scale_spec()
            .and_then(crate::CategoricalColorScale::map)
    }

    /// Caller-owned panel-corner text annotations.
    #[must_use]
    pub(crate) fn panel_text_annotations(&self) -> &[PlotTextAnnotation] {
        &self.payload.panel_text_annotations
    }

    pub(crate) fn waterfall_base(&self) -> Option<f64> {
        match self.payload.input.as_ref() {
            PlotInput::Waterfall { base, .. } => Some(*base),
            _ => None,
        }
    }

    /// Caller-owned waterfall annotation labels, if configured.
    #[must_use]
    pub(crate) fn waterfall_labels_spec(&self) -> Option<&WaterfallLabels> {
        self.payload.waterfall_labels.as_ref()
    }

    pub(crate) fn total_format_spec(
        &self,
    ) -> Option<&std::sync::Arc<crate::numeric::format::NumericFormat>> {
        self.payload.total_format.as_ref()
    }

    pub(crate) fn dumbbell_right_col(&self) -> Option<&'static str> {
        matches!(self.payload.input.as_ref(), PlotInput::Dumbbell { .. }).then_some(DUMBBELL_RIGHT)
    }

    /// Required endpoint labels and colors for a dumbbell plot.
    #[must_use]
    pub(crate) fn dumbbell_endpoints_spec(&self) -> Option<&DumbbellEndpoints> {
        match self.payload.input.as_ref() {
            PlotInput::Dumbbell { endpoints, .. } => Some(endpoints),
            _ => None,
        }
    }

    /// The chart title, if any.
    #[must_use]
    pub(crate) fn title_text(&self) -> Option<&str> {
        self.title.provided().map(crate::Text::as_str)
    }

    /// The chart subtitle, if any.
    #[must_use]
    pub(crate) fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.provided().map(crate::Text::as_str)
    }

    /// The chart caption, if any.
    #[must_use]
    pub(crate) fn caption_text(&self) -> Option<&str> {
        self.payload.caption.provided().map(crate::Text::as_str)
    }

    pub(crate) fn prose(&self) -> [Option<&crate::Text>; 3] {
        [&self.title, &self.subtitle, &self.payload.caption].map(crate::Text::provided)
    }

    pub(crate) fn prose_alignments(&self) -> [Option<crate::TextAlignment>; 3] {
        [&self.title, &self.subtitle, &self.payload.caption].map(crate::Text::alignment_request)
    }

    pub(crate) fn copy_prose(&self) -> Result<[crate::Text; 3], Error> {
        Ok([
            self.title.try_copy()?,
            self.subtitle.try_copy()?,
            self.payload.caption.try_copy()?,
        ])
    }

    pub(crate) fn same_prose(&self, prose: &[crate::Text; 3]) -> bool {
        [&self.title, &self.subtitle, &self.payload.caption]
            .into_iter()
            .zip(prose)
            .all(|(left, right)| left == right)
    }

    pub(crate) fn validate_display_text(&self) -> Result<(), Error> {
        for (source, role) in [
            (self.title_text(), "plot title"),
            (self.subtitle_text(), "plot subtitle"),
            (self.caption_text(), "plot caption"),
            (
                self.payload.x_label.provided().map(crate::Text::as_str),
                "x-axis title",
            ),
            (
                self.payload.y_label.provided().map(crate::Text::as_str),
                "y-axis title",
            ),
            (self.color_label_text(), "color guide title"),
        ] {
            if let Some(source) = source {
                crate::text::validate_for(source, role)?;
            }
        }
        Ok(())
    }

    pub(crate) fn resolve_label(&self, raw: &str) -> Result<String, Error> {
        self.payload.labels.resolve(raw)
    }

    pub(crate) fn labels_policy(&self) -> &crate::Labels {
        &self.payload.labels
    }

    pub(crate) fn resolve_axis_label_domain<'b>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'b str>,
    ) -> Result<Vec<String>, Error> {
        self.payload.labels.resolve_domain(domain, raw_labels)
    }

    pub(crate) fn copy_axis_titles(&self) -> Result<[crate::Text; 2], Error> {
        Ok([
            self.payload.x_label.try_copy()?,
            self.payload.y_label.try_copy()?,
        ])
    }

    pub(crate) fn same_axis_titles(&self, titles: &[crate::Text; 2]) -> bool {
        self.payload.x_label == titles[0] && self.payload.y_label == titles[1]
    }

    pub(crate) fn prepare_axis_labels(
        &self,

        policy: Option<&crate::Labels>,
    ) -> Result<[Option<crate::Text>; 2], Error> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        );
        let mut labels = [None, None];
        for ((target, source), authored) in labels
            .iter_mut()
            .zip(self.axis_label_sources(self.kind))
            .zip([
                self.payload.x_label.provided(),
                self.payload.y_label.provided(),
            ])
        {
            *target = if let Some(text) = authored {
                text.required_source("axis title")?;
                Some(text.try_copy()?)
            } else {
                source
                    .map(|(raw, inferred)| {
                        if inferred {
                            policy
                                .unwrap_or(&self.payload.labels)
                                .resolve(raw)
                                .map(crate::Text::from)
                        } else {
                            crate::text::validate_for(raw, "axis title")?;
                            crate::data::source::copy_source_string(raw).map(crate::Text::from)
                        }
                    })
                    .transpose()?
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
        self.payload.labels.resolve_domain(domain, raw_labels)
    }

    fn axis_label_sources(&self, kind: PlotKind) -> [Option<(&str, bool)>; 2] {
        let x_name = self.payload.input.x_name().map(|name| (name, true));
        let y_name = self.payload.input.y_name().map(|name| (name, true));
        let x = match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell => y_name,
            PlotKind::Qq => Some(("Theoretical normal quantiles", false)),
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || x_name,
                |input| Some((input.value_dimension.as_str(), true)),
            ),
            PlotKind::Raincloud => self.raincloud_geometry_input().map_or_else(
                || x_name,
                |input| Some((input.category_dimension.as_str(), true)),
            ),
            PlotKind::Matrix => None,
            _ => x_name,
        };
        let y = match kind {
            PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Waterfall
            | PlotKind::Dumbbell
            | PlotKind::Qq => x_name,
            PlotKind::Histogram => self.histogram_composite_geometry_input().map_or_else(
                || match self.payload.histogram_stat {
                    HistogramStat::Count => Some(("Count", false)),
                    HistogramStat::Probability => Some(("Probability", false)),
                    HistogramStat::Density => Some(("Density", false)),
                },
                |input| Some((input.density_dimension.as_str(), true)),
            ),
            PlotKind::Raincloud => self.raincloud_geometry_input().map_or_else(
                || y_name,
                |input| Some((input.value_dimension.as_str(), true)),
            ),
            PlotKind::Density => Some(("Density", false)),
            PlotKind::Matrix => None,
            _ => y_name,
        };
        [x, y]
    }

    pub(crate) fn prepare_color_label(
        &self,

        policy: Option<&crate::Labels>,
    ) -> Result<crate::Text, Error> {
        if let Some(label) = self.payload.color_label.provided() {
            label.required_source("color guide title")?;
            label.try_copy()
        } else if let Some(values) = self.color_values() {
            policy
                .unwrap_or(&self.payload.labels)
                .resolve(values.name())
                .map(crate::Text::from)
        } else {
            Ok(crate::Text::new(String::new()))
        }
    }

    pub(crate) fn color_dimension_key(&self) -> Option<&str> {
        self.color_values().map(crate::NumericValues::name)
    }

    pub(crate) fn legend_title_key(&self) -> &str {
        self.payload
            .interaction_styles
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
        if self.payload.interaction_styles.is_none() && self.hue_dimension_key().is_none() {
            return Err(Error::Render(
                "categorical guide entry has no prepared field".into(),
            ));
        }
        crate::decorations::legend::GuideEntryIdentity::category(self.legend_title_key(), label)
    }

    pub(crate) fn hue_dimension_key(&self) -> Option<&str> {
        self.hue_name().or_else(|| {
            self.raincloud_geometry_input()
                .and_then(|input| input.hue_dimension.as_deref())
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

    pub(crate) fn categorical_field(&self) -> Option<crate::data::category::RawField> {
        self.hue_field()
            .or_else(|| {
                self.raincloud_geometry_input()
                    .and_then(|input| input.hue_dimension.as_deref())
                    .map(|field| {
                        crate::data::category::RawField::from_shared(std::sync::Arc::from(field))
                    })
            })
            .or_else(|| {
                if self.kind() == crate::PlotKind::Interaction {
                    self.group_values().map(crate::CategoryValues::raw_field)
                } else if matches!(
                    self.kind(),
                    crate::PlotKind::Bar
                        | crate::PlotKind::HorizontalBar
                        | crate::PlotKind::StackedBar
                        | crate::PlotKind::StackedHorizontalBar
                        | crate::PlotKind::Boxplot
                        | crate::PlotKind::Violin
                        | crate::PlotKind::Raincloud
                ) {
                    self.x_data_column()
                        .and_then(|column| match column {
                            DataColumn::Category(values) => Some(values.raw_field()),
                            _ => None,
                        })
                        .or_else(|| {
                            self.raincloud_geometry_input().map(|input| {
                                crate::data::category::RawField::from_shared(std::sync::Arc::from(
                                    input.category_dimension.as_str(),
                                ))
                            })
                        })
                } else {
                    None
                }
            })
    }

    /// Borrow x-axis scale configuration.
    #[must_use]
    pub(crate) fn x_axis_spec(&self) -> &AxisSpec {
        &self.payload.axes[0]
    }

    pub(crate) fn explicit_x_ticks(&self) -> Option<&[AxisTick]> {
        self.payload.axes[0].exact_ticks.as_deref()
    }

    pub(crate) fn y_axis_spec(&self) -> &AxisSpec {
        &self.payload.axes[1]
    }

    pub(crate) fn explicit_y_ticks(&self) -> Option<&[AxisTick]> {
        self.payload.axes[1].exact_ticks.as_deref()
    }

    pub(crate) fn guides(&self) -> &[GuideSpec] {
        &self.payload.guides
    }

    pub(crate) fn point_layers(&self) -> &[PointLayer] {
        &self.payload.point_layers
    }

    pub(crate) fn bar_point_layers(&self) -> &[BarPointLayer] {
        &self.payload.bar_point_layers
    }

    pub(crate) fn significance_annotations(
        &self,
    ) -> &[crate::significance::SignificanceAnnotation] {
        &self.payload.significance
    }

    pub(crate) const fn sizing_request(&self) -> crate::layout::RootRequest {
        self.root_size
    }

    pub(crate) fn matrix_variables(&self) -> Option<&[crate::NumericValues]> {
        match self.payload.input.as_ref() {
            PlotInput::Matrix { variables } => Some(variables),
            _ => None,
        }
    }

    pub(crate) fn bar_spacing_spec(&self) -> BarSpacing {
        self.payload.bar_spacing
    }

    pub(crate) fn point_style(
        &self,
        theme: &crate::theme::ResolvedTheme,
        color: (u8, u8, u8),
        radius: u32,
    ) -> PointStyle {
        theme.marks.get_points().get_primary().backend_style(
            color,
            radius,
            self.payload.point_outline_visible,
        )
    }

    pub(crate) fn distribution_point_style(
        &self,
        role: &crate::theme::mark::SizedPointTheme,
        color: (u8, u8, u8),
    ) -> PointStyle {
        role.backend_style(color, self.payload.point_outline_visible)
    }

    pub(crate) fn legend_policy_spec(&self) -> LegendPolicy {
        self.payload.legend_policy
    }

    pub(crate) fn legend_position_spec(&self) -> LegendPosition {
        self.payload.legend_policy.position()
    }

    pub(crate) fn color_label_format_spec(
        &self,
    ) -> Option<&std::sync::Arc<crate::numeric::format::NumericFormat>> {
        self.payload.color_label_format.as_ref()
    }

    pub(crate) const fn kind(&self) -> PlotKind {
        self.kind
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    pub(crate) const fn description(&self) -> &PlotDescription {
        &self.description
    }

    #[cfg(test)]
    pub(crate) fn with_test_axis_ticks(mut self, side: usize, ticks: Vec<crate::AxisTick>) -> Self {
        if side == 0 {
            self.description.payload_mut().axes[0].exact_ticks = Some(ticks);
        } else {
            self.description.payload_mut().axes[1].exact_ticks = Some(ticks);
        }
        self
    }

    /// Assigned geometry is available only after the root or parent allocates it.
    pub(crate) fn canvas_dimensions(&self) -> Result<(u32, u32), Error> {
        self.description.root_size.dimensions()
    }

    pub(crate) fn figure_dimensions(&self) -> Result<(u32, u32), Error> {
        self.output_dimensions
            .map_or_else(|| self.canvas_dimensions(), Ok)
    }

    pub(crate) fn measure_dimensions(
        &self,
    ) -> Result<crate::orchestrator::assignment::PlotMeasureDimensions, Error> {
        Ok(crate::orchestrator::assignment::PlotMeasureDimensions {
            canvas: self.description.root_size.assigned_size()?,
            output_override: self
                .output_dimensions
                .map(|(width, height)| crate::layout::RootSize::new(width, height)),
        })
    }

    /// Selected semantic plot kind.
    #[must_use]
    pub const fn kind(&self) -> PlotKind {
        self.description.kind
    }

    pub(crate) fn with_planned_y_limits(mut self, limits: (f64, f64)) -> Self {
        self.description.payload_mut().axes[1].window =
            Some(crate::layout::RawWindow::from_pair(limits));
        self
    }

    pub(crate) fn with_planned_x_limits(mut self, limits: (f64, f64)) -> Self {
        self.description.payload_mut().axes[0].window =
            Some(crate::layout::RawWindow::from_pair(limits));
        self
    }

    pub(crate) fn user_x_limits(&self) -> Option<(f64, f64)> {
        if self.x_limits_user_explicit {
            self.description.payload.axes[0]
                .window
                .map(crate::layout::RawWindow::pair)
        } else {
            None
        }
    }

    pub(crate) fn user_y_limits(&self) -> Option<(f64, f64)> {
        if self.y_limits_user_explicit {
            self.description.payload.axes[1]
                .window
                .map(crate::layout::RawWindow::pair)
        } else {
            None
        }
    }
}
