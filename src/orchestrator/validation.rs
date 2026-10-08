//! Pre-backend chart configuration validation.

use std::collections::HashSet;

use crate::Error;

use crate::builder::Plot;
use crate::spec::{
    CategoryOrder, HeatmapLabelFit, HeatmapLayout, HistogramBins, HistogramComposite,
    HistogramStat, LinePathStyle, PlotKind, RegressionBand,
};

impl Plot {
    pub(crate) fn validate_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        self.x_axis_spec().validate("x-axis")?;
        self.y_axis_spec().validate("y-axis")?;
        for (spec, logarithmic) in [
            (
                self.x_axis_spec(),
                kind == PlotKind::Forest && self.axis_scale_spec() == crate::spec::AxisScale::Log10,
            ),
            (self.y_axis_spec(), false),
        ] {
            if spec.interval.is_some_and(|interval| {
                !matches!(
                    (logarithmic, interval),
                    (false, crate::layout::TickInterval::Additive(_))
                        | (true, crate::layout::TickInterval::Log10Decades(_))
                )
            }) {
                return Err(Error::Data(
                    "tick interval does not match the axis scale".into(),
                ));
            }
        }
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        if kind != PlotKind::Heatmap && self.heatmap_layout_spec() != HeatmapLayout::Cartesian {
            return Err(Error::Data(format!(
                "{kind:?}: non-Cartesian HeatmapLayout is supported only by PlotKind::Heatmap"
            )));
        }
        if kind != PlotKind::Heatmap && self.heatmap_label_fit_spec() != HeatmapLabelFit::Elide {
            return Err(Error::Data(format!(
                "{kind:?}: HeatmapLabelFit::Wrap is supported only by PlotKind::Heatmap"
            )));
        }
        if kind != PlotKind::Heatmap && !self.heatmap_null_row_annotations_spec().is_empty() {
            return Err(Error::Data(format!(
                "{kind:?}: fully-null heatmap row annotations are supported only by PlotKind::Heatmap"
            )));
        }
        if self.raincloud_geometry_input().is_some() && self.group_col().is_some() {
            return Err(Error::Data(
                "raincloud: Plot::series requires RaincloudInput::samples; remove .series(...) or construct the raincloud from samples"
                    .into(),
            ));
        }
        self.validate_value_alignment()?;
        if kind == PlotKind::Trajectory && self.automatic_axis_category_values(kind, true).is_some()
        {
            let condition = self
                .x_col()
                .ok_or_else(|| Error::Data("trajectory: condition column not set".into()))?;
            crate::data::strict_normalized_strings(
                self.data(),
                condition,
                "trajectory",
                "condition",
            )?;
        }
        self.validate_category_order_configuration(kind)?;
        self.validate_channel_configuration(kind)?;
        self.validate_interval_configuration(kind)?;
        self.validate_facet_configuration()?;
        self.validate_supplemental_configuration(kind)?;
        crate::layout::interior::InteriorRoute::classify(kind).validate(self)
    }

    pub(crate) fn validate_category_order_configuration(
        &self,
        kind: PlotKind,
    ) -> Result<(), Error> {
        if !matches!(self.category_order_spec(), CategoryOrder::Observed)
            && !matches!(
                kind,
                PlotKind::PointInterval
                    | PlotKind::Heatmap
                    | PlotKind::Bar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedBar
                    | PlotKind::StackedHorizontalBar
                    | PlotKind::Dumbbell
            )
        {
            return Err(Error::Data(format!(
                "{kind:?}: primary category ordering is not supported"
            )));
        }
        if kind != PlotKind::Heatmap
            && (!matches!(self.x_category_order_spec(), CategoryOrder::Observed)
                || !matches!(self.y_category_order_spec(), CategoryOrder::Observed))
        {
            return Err(Error::Data(format!(
                "{kind:?}: x_category_order and y_category_order are supported only by Heatmap"
            )));
        }
        if self.has_facets() && matches!(self.category_order_spec(), CategoryOrder::Value(_)) {
            return Err(Error::Data(
                "value-derived category order is not supported with faceting; each panel can prepare a different value for the same category, so supply an explicit order"
                    .into(),
            ));
        }
        if self.has_facets() {
            self.validate_faceted_explicit_category_orders(kind)?;
        }
        Ok(())
    }

    fn validate_faceted_explicit_category_orders(&self, kind: PlotKind) -> Result<(), Error> {
        let configured_source_rows = self.configured_source_rows()?;
        let data = configured_source_rows
            .map_or_else(|| self.data(), |rows| self.data().map_source_rows(rows));
        match kind {
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let column = self
                    .x_col()
                    .ok_or_else(|| Error::Data("bar: category column not set".into()))?;
                let observed = crate::data::stable_levels(&crate::data::strict_strings(
                    data,
                    column,
                    "faceted bar category order",
                )?);
                validate_full_explicit_order(
                    self.category_order_spec(),
                    &observed,
                    "bar category order",
                    false,
                )?;
            }
            PlotKind::PointInterval => {
                let column = self
                    .y_col()
                    .ok_or_else(|| Error::Data("point_interval: category column not set".into()))?;
                let observed = crate::data::stable_levels(&crate::data::strict_normalized_strings(
                    data,
                    column,
                    "faceted point_interval category order",
                    "category",
                )?);
                validate_full_explicit_order(
                    self.category_order_spec(),
                    &observed,
                    "point_interval category order",
                    true,
                )?;
            }
            PlotKind::Heatmap => self.validate_faceted_heatmap_category_orders(data)?,
            _ => {}
        }
        Ok(())
    }

    fn validate_faceted_heatmap_category_orders(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        let x_column = self
            .x_col()
            .ok_or_else(|| Error::Data("heatmap: x category column not set".into()))?;
        let y_column = self
            .y_col()
            .ok_or_else(|| Error::Data("heatmap: y category column not set".into()))?;
        let x_observed = crate::data::stable_levels(&crate::data::strict_normalized_strings(
            data,
            x_column,
            "faceted heatmap category order",
            "x",
        )?);
        let y_observed = crate::data::stable_levels(&crate::data::strict_normalized_strings(
            data,
            y_column,
            "faceted heatmap category order",
            "y",
        )?);
        let family = (self.heatmap_layout_spec() != HeatmapLayout::Cartesian).then(|| {
            let mut family = x_observed.clone();
            for level in &y_observed {
                if !family.contains(level) {
                    family.push(level.clone());
                }
            }
            family
        });
        for (axis, observed, order) in [
            (
                "x",
                family.as_deref().unwrap_or(&x_observed),
                self.x_category_order_spec(),
            ),
            (
                "y",
                family.as_deref().unwrap_or(&y_observed),
                self.y_category_order_spec(),
            ),
        ] {
            validate_full_explicit_order(
                order,
                observed,
                &format!("heatmap {axis} category order"),
                true,
            )?;
        }
        if self.heatmap_layout_spec() != HeatmapLayout::Cartesian {
            validate_matching_triangular_orders(
                self.x_category_order_spec(),
                self.y_category_order_spec(),
            )?;
        }
        Ok(())
    }

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
        if let Some(statuses) = self.estimate_status_values() {
            if statuses.len() != expected {
                return Err(Error::Data(format!(
                    "plot estimate status values have length {}, expected {expected}",
                    statuses.len()
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
        Ok(())
    }

    fn validate_facet_configuration(&self) -> Result<(), Error> {
        if self.has_facets() {
            crate::layout::interior::CompositionChildPolicy::Facet.validate_child(self)?;
        }
        if self.has_facets() && !self.panel_text_annotations().is_empty() {
            return Err(Error::Data(
                "panel text annotations are not supported with faceting; each annotation requires one unambiguous chart panel owner"
                    .into(),
            ));
        }
        for annotation in self.significance_annotations() {
            annotation.validate()?;
        }
        if !self.has_facets()
            && self
                .significance_annotations()
                .iter()
                .any(|annotation| annotation.facet_cell().is_some())
        {
            return Err(Error::Render(
                "facet-owned significance annotations require Plot::facet; ownership is never ignored on an unfaceted chart".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_interval_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        if self.line_band_configured() && kind != PlotKind::Line {
            return Err(Error::Data(format!(
                "{kind:?}: band is supported only by Line"
            )));
        }
        if matches!(kind, PlotKind::Bar | PlotKind::HorizontalBar) {
            if self.interval_bound_cols().is_some() {
                return Err(Error::Data(format!(
                    "{kind:?} does not consume line bands; use BarInput::estimates_with_error_bars for precomputed bar bounds"
                )));
            }
            return crate::plots::bar_data::validate_cells(self, self.data());
        }
        if matches!(kind, PlotKind::StackedBar | PlotKind::StackedHorizontalBar)
            && self.interval_bound_cols().is_some()
        {
            return Err(Error::Data(format!(
                "line bands are not consumed by {kind:?}"
            )));
        }
        if kind != PlotKind::Regression
            && self.regression_band_spec() != RegressionBand::PointwiseMean95
        {
            return Err(Error::Data(format!(
                "RegressionBand is supported only by Regression, not {kind:?}"
            )));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive pre-backend validator covers every typed chart channel"
    )]
    pub(crate) fn validate_channel_configuration(&self, kind: PlotKind) -> Result<(), Error> {
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
                        "plot kind {kind:?}: duplicate panel text annotation corner {:?}",
                        annotation.corner()
                    )));
                }
            }
        }
        if self.histogram_composite_geometry_input().is_some() {
            if kind != PlotKind::Histogram {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: precomputed histogram composite geometry is supported only by Histogram"
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
            if self.source_row_map().is_some() {
                return Err(Error::Data(
                    "histogram: precomputed composite geometry does not accept source-row inputs"
                        .into(),
                ));
            }
        }
        if self.raincloud_geometry_input().is_some() {
            if kind != PlotKind::Raincloud {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: precomputed raincloud geometry is supported only by Raincloud"
                )));
            }
            if self.has_facets() {
                return Err(Error::Data(
                    "raincloud: precomputed geometry is not supported with faceting".into(),
                ));
            }
            if self.x_col().is_some()
                || self.y_col().is_some()
                || self.hue_col().is_some()
                || self.group_col().is_some()
            {
                return Err(Error::Data(
                    "raincloud: precomputed geometry is self-contained and does not accept external sample data channels"
                        .into(),
                ));
            }
            if self.source_row_map().is_some() {
                return Err(Error::Data(
                    "raincloud: precomputed geometry does not accept source-row inputs".into(),
                ));
            }
            if !self.significance_annotations().is_empty() {
                return Err(Error::Data(
                    "raincloud: precomputed geometry does not support significance annotations"
                        .into(),
                ));
            }
        }
        if self.histogram_composite_spec().is_enabled() {
            if kind != PlotKind::Histogram {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: HistogramComposite is supported only by Histogram"
                )));
            }
            if self.histogram_stat_spec() != HistogramStat::Density {
                return Err(Error::Data(
                    "histogram: HistogramComposite::KdeAndRug requires HistogramStat::Density so bars and curve share one density axis"
                        .into(),
                ));
            }
        }
        if self.raw_overlay_spec().is_visible()
            && !matches!(kind, PlotKind::Boxplot | PlotKind::Violin)
        {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: visible RawOverlay is supported only by Boxplot and Violin"
            )));
        }
        if matches!(kind, PlotKind::Trajectory | PlotKind::LinePath) {
            let mode = if kind == PlotKind::LinePath {
                "path"
            } else {
                "trajectory"
            };
            if self.group_col().is_none() {
                return Err(Error::Data(format!(
                    "line {mode}: an explicit graphical group column is required; hue is color-only"
                )));
            }
            if kind == PlotKind::LinePath {
                self.validate_path_x_axis()?;
            } else {
                self.validate_trajectory_x_axis()?;
            }
        }
        if self.line_path_style_spec() != LinePathStyle::Neutral {
            if kind != PlotKind::LinePath {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: LinePathStyle::Hue is supported only by LinePath"
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
                | PlotKind::Trajectory
                | PlotKind::LinePath
                | PlotKind::Interaction
                | PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Raincloud
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
                "plot kind {kind:?}: hue column {column:?} is not supported"
            )));
        }
        let supports_group = matches!(
            kind,
            PlotKind::Line
                | PlotKind::Trajectory
                | PlotKind::LinePath
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
                | PlotKind::Raincloud
                | PlotKind::Dumbbell
        );
        if let Some(column) = self.group_col().filter(|_| !supports_group) {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: graphical group column {column:?} is not supported"
            )));
        }
        if matches!(
            kind,
            PlotKind::Trajectory
                | PlotKind::LinePath
                | PlotKind::Interaction
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
        ) && self.group_col().is_none()
        {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: an explicit graphical group column is required"
            )));
        }
        if self.interaction_styles_spec().is_some() {
            if kind != PlotKind::Interaction {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: interaction styles are supported only by Interaction"
                )));
            }
            if !self.markers_visible() {
                return Err(Error::Data(
                    "interaction: configured styles require visible markers".into(),
                ));
            }
        }
        if let Some(connectors) = self.interaction_connectors_spec() {
            if kind != PlotKind::Interaction {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: connectors is supported only by Interaction"
                )));
            }
            if !connectors && self.interaction_styles_spec().is_some() {
                return Err(Error::Data(
                    "interaction: connectors(false) is incompatible with configured styles".into(),
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
                "plot kind {kind:?}: hue must use a different column from the x category; a redundant hue channel is not silently ignored"
            )));
        }

        let supports_color = matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm);
        if let Some(column) = self.color_col().filter(|_| !supports_color) {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: continuous color column {column:?} is not supported"
            )));
        }
        if self.color_label_text().is_some() && !supports_color {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: a continuous color label is not supported"
            )));
        }
        if let Some((minimum, maximum)) = self.beeswarm_color_limits_spec() {
            if kind != PlotKind::Beeswarm {
                return Err(Error::Data(format!(
                    "plot kind {kind:?}: beeswarm color limits are supported only by Beeswarm"
                )));
            }
            validate_beeswarm_color_limits(minimum, maximum)?;
        }
        if self.heatmap_highlight_col().is_some() && kind != PlotKind::Heatmap {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: a heatmap highlight column is not supported"
            )));
        }
        if self.estimate_labels_spec().is_some() && kind != PlotKind::Coefficient {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: numeric estimate labels are supported only by Coefficient"
            )));
        }
        if self.coefficient_reference_line_value().is_some() && kind != PlotKind::Coefficient {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: reference_line is supported only by Coefficient; Forest receives its reference in Plot::forest"
            )));
        }
        if self.estimate_status_values().is_some()
            && !matches!(
                kind,
                PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
            )
        {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: estimate status is supported only by PointInterval, Coefficient, and Forest"
            )));
        }
        let supports_bounds = matches!(
            kind,
            PlotKind::PointInterval
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Coefficient
                | PlotKind::Forest
        );
        if self.interval_bound_cols().is_some() && !supports_bounds {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: interval bound columns are not supported"
            )));
        }
        if self.waterfall_labels_spec().is_some() && kind != PlotKind::Waterfall {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: waterfall annotation labels are supported only by Waterfall"
            )));
        }
        if kind == PlotKind::Matrix {
            self.validate_matrix_variables()?;
        } else if !self.matrix_variable_names().is_empty() {
            return Err(Error::Data(format!(
                "plot kind {kind:?}: matrix variables are not supported"
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
        let source = self
            .data()
            .column(x_column)
            .map_err(|error| Error::Data(format!("trajectory: column {x_column:?}: {error}")))?;
        if source.is_numeric() {
            return Ok(());
        }
        if self.x_axis_spec().limits.is_some() {
            return Err(Error::Data(
                "trajectory: categorical condition x does not support numeric x limits".into(),
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

fn validate_full_explicit_order(
    order: &CategoryOrder,
    observed: &[String],
    context: &str,
    normalize: bool,
) -> Result<(), Error> {
    let CategoryOrder::Explicit(requested) = order else {
        return Ok(());
    };
    let requested = if normalize {
        requested
            .iter()
            .map(|level| crate::data::collapse_whitespace(level))
            .collect::<Vec<_>>()
    } else {
        requested.clone()
    };
    crate::data::explicit_category_order(observed, &requested, context, false)?;
    Ok(())
}

fn validate_matching_triangular_orders(x: &CategoryOrder, y: &CategoryOrder) -> Result<(), Error> {
    let (CategoryOrder::Explicit(x), CategoryOrder::Explicit(y)) = (x, y) else {
        return Ok(());
    };
    let normalized = |levels: &[String]| {
        levels
            .iter()
            .map(|level| crate::data::collapse_whitespace(level))
            .collect::<Vec<_>>()
    };
    if normalized(x) != normalized(y) {
        return Err(Error::Data(
            "heatmap: triangular x and y category orders must resolve to the same family order"
                .into(),
        ));
    }
    Ok(())
}

fn validate_beeswarm_color_limits(minimum: f64, maximum: f64) -> Result<(), Error> {
    if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
        return Err(Error::Data(format!(
            "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
        )));
    }
    Ok(())
}
