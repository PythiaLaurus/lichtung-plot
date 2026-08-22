//! Pre-backend chart configuration validation.

use std::collections::HashSet;

use crate::Error;

use crate::builder::Plot;
use crate::spec::{
    BarAggregation, HistogramBins, HistogramComposite, HistogramStat, LineMode, LinePathStyle,
    PlotKind, RegressionBand, ViolinLayout,
};

impl Plot {
    pub(crate) fn validate_value_alignment(&self) -> Result<(), Error> {
        let expected = self.row_count();
        for values in self.aligned_columns() {
            if values.len() != expected {
                return Err(Error::Data(format!(
                    "plot value {:?} has length {}, expected {expected}",
                    values.name(),
                    values.len()
                )));
            }
        }
        if let Some(rows) = self.source_row_map() {
            if rows.len() != expected {
                return Err(Error::Data(format!(
                    "plot source-row map has length {}, expected {expected}",
                    rows.len()
                )));
            }
        }
        if let Some(rows) = self.raw_overlay_source_row_map() {
            if rows.len() != expected {
                return Err(Error::Data(format!(
                    "raw-overlay source-row map has length {}, expected {expected}",
                    rows.len()
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_facet_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        let (row, column, wrap, wrap_columns) = self.facet_columns();
        if wrap.is_some() && (row.is_some() || column.is_some()) {
            return Err(Error::Data(
                "facet_wrap cannot be combined with facet_row or facet_col".into(),
            ));
        }
        if row.is_some() && row == column {
            return Err(Error::Data(
                "facet_row and facet_col must use different columns".into(),
            ));
        }
        if wrap.is_some() && wrap_columns == Some(0) {
            return Err(Error::Data(
                "facet_wrap columns must be greater than zero".into(),
            ));
        }
        if self.has_facets() && matches!(kind, PlotKind::Matrix | PlotKind::Marginal) {
            return Err(Error::Render(format!(
                "{kind:?} manages its own panels and cannot be nested in generic faceting"
            )));
        }
        if self.has_facets() && !self.panel_text_annotations().is_empty() {
            return Err(Error::Data(
                "panel text annotations are not supported with faceting; each annotation requires one unambiguous chart panel owner"
                    .into(),
            ));
        }
        if !self.has_facets()
            && self
                .comparisons()
                .iter()
                .any(|comparison| !comparison.facet.is_empty())
        {
            return Err(Error::Render(
                "facet-owned significance comparisons require facet_row, facet_col, or facet_wrap; ownership is never ignored on an unfaceted chart".into(),
            ));
        }
        for comparison in self.comparisons() {
            for (index, facet) in comparison.facet.iter().enumerate() {
                if let Some(previous) = comparison.facet[..index]
                    .iter()
                    .find(|previous| previous.variable == facet.variable)
                {
                    return Err(Error::Render(format!(
                        "significance comparison has contradictory duplicate ownership for facet variable {:?}: {:?} and {:?}; name each facet variable exactly once",
                        facet.variable, previous.level, facet.level
                    )));
                }
            }
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive validator keeps mutually exclusive generated and caller interval contracts together"
    )]
    pub(crate) fn validate_interval_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        if let Some(effect) = self.effect_band_semantics_spec() {
            effect.validate()?;
            if kind != PlotKind::Line {
                return Err(Error::Data(format!(
                    "effect-band semantics are only supported by Line, not {kind:?}"
                )));
            }
            if self.line_mode_spec() != LineMode::Series {
                return Err(Error::Data(
                    "effect-band semantics require LineMode::Series".into(),
                ));
            }
            if self.interval_bound_cols().is_none() {
                return Err(Error::Data(
                    "effect-band semantics require complete lower and upper bound columns".into(),
                ));
            }
            if self.interval_semantics_spec() != Some(effect.interval()) {
                return Err(Error::Data(
                    "effect-band interval semantics do not match the configured interval semantics"
                        .into(),
                ));
            }
        }
        if self.line_mode_spec() != LineMode::Series
            && (self.interval_bound_cols().is_some() || self.interval_semantics_spec().is_some())
        {
            return Err(Error::Data(
                "non-series line modes do not support ribbons or interval bounds".into(),
            ));
        }
        if matches!(kind, PlotKind::Bar | PlotKind::HorizontalBar) {
            let has_bounds = self.interval_bound_cols().is_some();
            let has_semantics = self.interval_semantics_spec().is_some();
            if self.bar_aggregation_spec() == BarAggregation::Mean && (has_bounds || has_semantics)
            {
                return Err(Error::Data(format!(
                    "{kind:?} generated mean intervals are owned by BarAggregation and reject caller-provided bounds and interval semantics"
                )));
            }
            if self.bar_aggregation_spec() == BarAggregation::Identity
                && has_bounds != has_semantics
            {
                return Err(Error::Data(format!(
                    "{kind:?} identity interval bounds and IntervalSemantics must be configured together"
                )));
            }
            if let Some(semantics) = self.interval_semantics_spec() {
                semantics.validate()?;
            }
            return crate::plots::bar_data::validate_aggregation_cells(
                self,
                self.data(),
                self.bar_aggregation_spec(),
            );
        }
        if matches!(kind, PlotKind::StackedBar | PlotKind::StackedHorizontalBar) {
            if self.bar_aggregation_spec() == BarAggregation::Identity {
                return Err(Error::Data(format!(
                    "BarAggregation::Identity is not consumed by {kind:?}"
                )));
            }
            if self.interval_bound_cols().is_some() || self.interval_semantics_spec().is_some() {
                return Err(Error::Data(format!(
                    "explicit interval bounds and semantics are not consumed by {kind:?}"
                )));
            }
        } else if self.bar_aggregation_spec() != BarAggregation::Mean {
            return Err(Error::Data(format!(
                "BarAggregation is supported only by Bar and HorizontalBar, not {kind:?}"
            )));
        }
        if kind != PlotKind::Regression
            && self.regression_band_spec() != RegressionBand::PointwiseMean95
        {
            return Err(Error::Data(format!(
                "RegressionBand is supported only by Regression, not {kind:?}"
            )));
        }
        let Some(semantics) = self.interval_semantics_spec() else {
            return Ok(());
        };
        semantics.validate()?;
        if self.interval_bound_cols().is_none() {
            return Err(Error::Data(format!(
                "interval semantics for {kind:?} require complete lower and upper bound columns"
            )));
        }
        if !matches!(
            kind,
            PlotKind::PointInterval
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Coefficient
                | PlotKind::Forest
        ) {
            return Err(Error::Data(format!(
                "interval semantics are not consumed by {kind:?}"
            )));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive pre-backend validator covers every typed chart channel"
    )]
    pub(crate) fn validate_channel_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        let family = kind.key_segment();
        if !self.panel_text_annotations().is_empty() {
            if kind == PlotKind::Matrix {
                return Err(Error::Data(
                    "matrix: panel text annotations require one standard chart panel".into(),
                ));
            }
            let mut corners = HashSet::new();
            for annotation in self.panel_text_annotations() {
                if !corners.insert(annotation.corner()) {
                    return Err(Error::Data(format!(
                        "{family}: duplicate panel text annotation corner {:?}",
                        annotation.corner()
                    )));
                }
            }
        }
        if self.histogram_composite_geometry_input().is_some() {
            if kind != PlotKind::Histogram {
                return Err(Error::Data(format!(
                    "{family}: precomputed histogram composite geometry is supported only by Histogram"
                )));
            }
            if self.has_facets() {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry is not supported with faceting"
                        .into(),
                ));
            }
            if self.histogram_stat_spec() != HistogramStat::Density {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry requires HistogramStat::Density"
                        .into(),
                ));
            }
            if self.histogram_bins_spec() != HistogramBins::Auto
                || self.histogram_composite_spec() != &HistogramComposite::HistogramOnly
            {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry cannot be combined with sample-derived HistogramBins or HistogramComposite options"
                        .into(),
                ));
            }
            if self.x_col().is_some()
                || self.y_col().is_some()
                || self.hue_col().is_some()
                || self.group_col().is_some()
            {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry is self-contained and does not accept x, y, hue, or group data channels"
                        .into(),
                ));
            }
            if self.source_row_map().is_some() || self.raw_overlay_source_row_map().is_some() {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry does not accept source-row inputs"
                        .into(),
                ));
            }
        }
        if self.raincloud_geometry_input().is_some() {
            if kind != PlotKind::Violin {
                return Err(Error::Data(format!(
                    "{family}: precomputed raincloud geometry is supported only by Violin"
                )));
            }
            if self.violin_layout_spec() != ViolinLayout::Raincloud {
                return Err(Error::Data(
                    "violin: precomputed raincloud geometry requires ViolinLayout::Raincloud"
                        .into(),
                ));
            }
            if self.has_facets() {
                return Err(Error::Data(
                    "violin: precomputed raincloud geometry is not supported with faceting".into(),
                ));
            }
            if self.x_col().is_some()
                || self.y_col().is_some()
                || self.hue_col().is_some()
                || self.group_col().is_some()
            {
                return Err(Error::Data(
                    "violin: precomputed raincloud geometry is self-contained and does not accept x, y, hue, or group data channels"
                        .into(),
                ));
            }
            if self.source_row_map().is_some() || self.raw_overlay_source_row_map().is_some() {
                return Err(Error::Data(
                    "violin: precomputed raincloud geometry does not accept source-row inputs"
                        .into(),
                ));
            }
            if !self.comparisons().is_empty() {
                return Err(Error::Data(
                    "violin: precomputed raincloud geometry does not support significance annotations"
                        .into(),
                ));
            }
        }
        if self.histogram_composite_spec().is_enabled() {
            if kind != PlotKind::Histogram {
                return Err(Error::Data(format!(
                    "{family}: HistogramComposite is supported only by Histogram"
                )));
            }
            if self.histogram_stat_spec() != HistogramStat::Density {
                return Err(Error::Data(
                    "histogram: HistogramComposite::KdeAndRug requires HistogramStat::Density so bars and curve share one density axis"
                        .into(),
                ));
            }
        }
        if self.violin_layout_spec() != ViolinLayout::Mirrored && kind != PlotKind::Violin {
            return Err(Error::Data(format!(
                "{family}: ViolinLayout::Raincloud is supported only by Violin"
            )));
        }
        if self.violin_layout_spec() == ViolinLayout::Raincloud
            && self.raw_overlay_spec().is_visible()
        {
            return Err(Error::Data(
                "violin: ViolinLayout::Raincloud already includes every raw observation and cannot be combined with a visible RawOverlay"
                    .into(),
            ));
        }
        if self.raw_overlay_spec().is_visible()
            && !matches!(kind, PlotKind::Boxplot | PlotKind::Violin)
        {
            return Err(Error::Data(format!(
                "{family}: visible RawOverlay is supported only by Boxplot and Violin"
            )));
        }
        let raincloud_uses_source_rows = kind == PlotKind::Violin
            && self.violin_layout_spec() == ViolinLayout::Raincloud
            && self.raincloud_geometry_input().is_none();
        if self.raw_overlay_source_row_map().is_some()
            && !self.raw_overlay_spec().is_visible()
            && !raincloud_uses_source_rows
        {
            return Err(Error::Data(format!(
                "{family}: a raw-observation source-row column requires a visible RawOverlay or ViolinLayout::Raincloud"
            )));
        }
        if matches!(self.line_mode_spec(), LineMode::Trajectory | LineMode::Path) {
            let mode = if self.line_mode_spec() == LineMode::Path {
                "path"
            } else {
                "trajectory"
            };
            if kind != PlotKind::Line {
                return Err(Error::Data(format!(
                    "{family}: LineMode::{:?} is only supported by Line",
                    self.line_mode_spec()
                )));
            }
            if self.group_col().is_none() {
                return Err(Error::Data(format!(
                    "line {mode}: an explicit graphical group column is required; hue is color-only"
                )));
            }
            if self.line_mode_spec() == LineMode::Trajectory && !self.markers_visible() {
                return Err(Error::Data(
                    "line trajectory: markers(false) is not supported because observations must remain visible"
                        .into(),
                ));
            }
            if self.line_mode_spec() == LineMode::Path {
                self.validate_path_x_axis()?;
            } else {
                self.validate_trajectory_x_axis()?;
            }
        }
        if self.line_path_style_spec() != LinePathStyle::Neutral {
            if kind != PlotKind::Line || self.line_mode_spec() != LineMode::Path {
                return Err(Error::Data(format!(
                    "{family}: LinePathStyle::Hue is supported only by LineMode::Path"
                )));
            }
            if self.hue_col().is_none() {
                return Err(Error::Data(
                    "line path: LinePathStyle::Hue requires a categorical hue column".into(),
                ));
            }
        }
        let supports_hue = matches!(
            kind,
            PlotKind::Scatter
                | PlotKind::Density
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Marginal
                | PlotKind::Matrix
        );
        if let Some(column) = self.hue_name().filter(|_| !supports_hue) {
            if kind == PlotKind::Histogram {
                return Err(Error::Data(format!(
                    "histogram: hue column {column:?} is not supported yet; render separate single-series histograms"
                )));
            }
            return Err(Error::Data(format!(
                "{family}: hue column {column:?} is not supported"
            )));
        }
        let supports_group = matches!(
            kind,
            PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Density
                | PlotKind::Marginal
                | PlotKind::Matrix
                | PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Dumbbell
        );
        if let Some(column) = self.group_col().filter(|_| !supports_group) {
            return Err(Error::Data(format!(
                "{family}: graphical group column {column:?} is not supported"
            )));
        }
        if matches!(
            kind,
            PlotKind::Interaction | PlotKind::StackedBar | PlotKind::StackedHorizontalBar
        ) && self.group_col().is_none()
        {
            return Err(Error::Data(format!(
                "{family}: an explicit graphical group column is required"
            )));
        }
        if self.interaction_style_cols().is_some() {
            if kind != PlotKind::Interaction {
                return Err(Error::Data(format!(
                    "{family}: interaction style columns are supported only by Interaction"
                )));
            }
            if !self.markers_visible() {
                return Err(Error::Data(
                    "interaction: style columns require visible markers".into(),
                ));
            }
            if self.has_facets() {
                return Err(Error::Data(
                    "interaction: style columns are not supported with faceting in this focused style phase"
                        .into(),
                ));
            }
        }
        if let Some(connectors) = self.interaction_connectors_spec() {
            if kind != PlotKind::Interaction {
                return Err(Error::Data(format!(
                    "{family}: interaction_connectors is supported only by Interaction"
                )));
            }
            if !connectors && self.interaction_style_cols().is_some() {
                return Err(Error::Data(
                    "interaction: interaction_connectors(false) is incompatible with line-pattern style columns"
                        .into(),
                ));
            }
        }
        if kind == PlotKind::Interaction
            && !self.markers_visible()
            && !self.interaction_connectors_visible()
        {
            return Err(Error::Data(
                "interaction: markers and connectors cannot both be hidden".into(),
            ));
        }
        if matches!(kind, PlotKind::Boxplot | PlotKind::Violin)
            && self
                .hue_col()
                .is_some_and(|column| Some(column) == self.x_col())
        {
            return Err(Error::Data(format!(
                "{family}: hue must use a different column from the x category; a redundant hue channel is not silently ignored"
            )));
        }

        let supports_color = matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm);
        if let Some(column) = self.color_col().filter(|_| !supports_color) {
            return Err(Error::Data(format!(
                "{family}: continuous color column {column:?} is not supported"
            )));
        }
        if self.color_label_text().is_some() && !supports_color {
            return Err(Error::Data(format!(
                "{family}: a continuous color label is not supported"
            )));
        }
        if let Some((minimum, maximum)) = self.beeswarm_color_limits_spec() {
            if kind != PlotKind::Beeswarm {
                return Err(Error::Data(format!(
                    "{family}: beeswarm color limits are supported only by Beeswarm"
                )));
            }
            validate_beeswarm_color_limits(minimum, maximum)?;
        }
        if self.heatmap_highlight_col().is_some() && kind != PlotKind::Heatmap {
            return Err(Error::Data(format!(
                "{family}: a heatmap highlight column is not supported"
            )));
        }
        if self.estimate_labels_spec().is_some() && kind != PlotKind::Coefficient {
            return Err(Error::Data(format!(
                "{family}: numeric estimate labels are supported only by Coefficient"
            )));
        }
        if self.coefficient_significance_col().is_some() && kind != PlotKind::Coefficient {
            return Err(Error::Data(format!(
                "{family}: coefficient significance marks are supported only by Coefficient"
            )));
        }
        if self.bar_significance_col().is_some()
            && !matches!(kind, PlotKind::Bar | PlotKind::HorizontalBar)
        {
            return Err(Error::Data(format!(
                "{family}: per-bar significance marks are supported only by ordinary Bar and HorizontalBar"
            )));
        }
        if self.bar_significance_col().is_some()
            && self.bar_aggregation_spec() != crate::BarAggregation::Identity
        {
            return Err(Error::Data(format!(
                "{family}: per-bar significance marks require identity aggregation"
            )));
        }
        if !self.grouped_factor_annotations().is_empty() {
            if !matches!(kind, PlotKind::Bar | PlotKind::HorizontalBar) {
                return Err(Error::Data(format!(
                    "{family}: grouped-factor annotations are supported only by ordinary Bar and HorizontalBar"
                )));
            }
            if self.bar_aggregation_spec() != BarAggregation::Identity {
                return Err(Error::Data(format!(
                    "{family}: grouped-factor annotations require identity aggregation"
                )));
            }
            if self.group_col().is_none() {
                return Err(Error::Data(format!(
                    "{family}: grouped-factor annotations require a graphical group column"
                )));
            }
            if self.has_facets() {
                return Err(Error::Data(format!(
                    "{family}: grouped-factor annotations are not supported with faceting"
                )));
            }
            let mut factors = HashSet::new();
            for annotation in self.grouped_factor_annotations() {
                if !factors.insert(annotation.factor()) {
                    return Err(Error::Data(format!(
                        "{family}: grouped-factor annotation repeats factor identity {:?}",
                        annotation.factor()
                    )));
                }
            }
        }
        if self.estimate_status_col().is_some()
            && !matches!(
                kind,
                PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
            )
        {
            return Err(Error::Data(format!(
                "{family}: estimate status is supported only by PointInterval, Coefficient, and Forest"
            )));
        }
        if self.dumbbell_right_col().is_some() != self.dumbbell_endpoints_spec().is_some() {
            return Err(Error::Data(format!(
                "{family}: dumbbell right-value and endpoint options must be configured together"
            )));
        }
        if kind != PlotKind::Dumbbell
            && (self.dumbbell_right_col().is_some() || self.dumbbell_endpoints_spec().is_some())
        {
            return Err(Error::Data(format!(
                "{family}: dumbbell endpoint options are not supported"
            )));
        }

        let supports_bounds = matches!(
            kind,
            PlotKind::PointInterval
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Coefficient
                | PlotKind::Forest
                | PlotKind::Bar
                | PlotKind::HorizontalBar
        );
        if self.interval_bound_cols().is_some() && !supports_bounds {
            return Err(Error::Data(format!(
                "{family}: interval bound columns are not supported"
            )));
        }
        if self.waterfall_totals().is_some() && kind != PlotKind::Waterfall {
            return Err(Error::Data(format!(
                "{family}: waterfall base and terminal channels are not supported"
            )));
        }
        if self.waterfall_labels_spec().is_some() && kind != PlotKind::Waterfall {
            return Err(Error::Data(format!(
                "{family}: waterfall annotation labels are supported only by Waterfall"
            )));
        }
        if kind == PlotKind::Matrix {
            self.validate_matrix_variables()?;
        } else if !self.matrix_variable_names().is_empty() {
            return Err(Error::Data(format!(
                "{family}: matrix variables are not supported"
            )));
        }
        Ok(())
    }

    fn validate_matrix_variables(&self) -> Result<(), Error> {
        if self.x_col().is_some() || self.y_col().is_some() {
            return Err(Error::Data(
                "matrix: x and y channels are not supported; configure only the ordered matrix variable list"
                    .into(),
            ));
        }
        let variables = self.matrix_variable_names();
        if variables.len() < 2 {
            return Err(Error::Data(
                "matrix: at least two ordered variables are required".into(),
            ));
        }
        let mut seen = HashSet::new();
        for variable in variables {
            if variable.trim().is_empty() {
                return Err(Error::Data(
                    "matrix: variable names must be nonempty".into(),
                ));
            }
            if !seen.insert(variable.as_str()) {
                return Err(Error::Data(format!(
                    "matrix: duplicate variable name {variable:?}"
                )));
            }
        }
        Ok(())
    }

    fn validate_trajectory_x_axis(&self) -> Result<(), Error> {
        let Some(x_column) = self.x_col() else {
            return Ok(());
        };
        let source = self.data().column(x_column).map_err(|error| {
            Error::Data(format!("line trajectory: column {x_column:?}: {error}"))
        })?;
        if source.is_numeric() {
            return Ok(());
        }
        if self.x_axis_spec().limits.is_some() {
            return Err(Error::Data(
                "line trajectory: categorical condition x does not support numeric x limits".into(),
            ));
        }
        if self.x_tick_count_overridden() {
            return Err(Error::Data(
                "line trajectory: categorical condition x does not support numeric x tick-count overrides"
                    .into(),
            ));
        }
        Ok(())
    }

    fn validate_path_x_axis(&self) -> Result<(), Error> {
        let Some(x_column) = self.x_col() else {
            return Ok(());
        };
        let source = self
            .data()
            .column(x_column)
            .map_err(|error| Error::Data(format!("line path: column {x_column:?}: {error}")))?;
        if source.is_numeric() {
            Ok(())
        } else {
            Err(Error::Data(
                "line path: x must be numeric so source-ordered path geometry remains explicit"
                    .into(),
            ))
        }
    }
}

fn validate_beeswarm_color_limits(minimum: f64, maximum: f64) -> Result<(), Error> {
    if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
        return Err(Error::Data(format!(
            "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
        )));
    }
    Ok(())
}
