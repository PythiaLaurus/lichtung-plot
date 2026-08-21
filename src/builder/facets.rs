use crate::Error;

use super::Chart;
use crate::spec::{AxisScale, FacetScales, PlotKind};

impl<'a> Chart<'a> {
    /// Split the selected chart into rows by a categorical column.
    #[must_use]
    pub fn facet_row(mut self, column: &'a str) -> Self {
        self.facet_row_col = Some(column);
        self
    }

    /// Split the selected chart into columns by a categorical column.
    #[must_use]
    pub fn facet_col(mut self, column: &'a str) -> Self {
        self.facet_col_col = Some(column);
        self
    }

    /// Wrap one categorical facet variable into the requested number of columns.
    #[must_use]
    pub fn facet_wrap(mut self, column: &'a str, columns: usize) -> Self {
        self.facet_wrap_col = Some(column);
        self.facet_wrap_columns = Some(columns);
        self
    }

    /// Configure whether facet panels share numeric axes.
    #[must_use]
    pub fn facet_scales(mut self, scales: FacetScales) -> Self {
        self.facet_scales = scales;
        self
    }

    pub(crate) const fn facet_scales_spec(&self) -> FacetScales {
        self.facet_scales
    }

    pub(crate) const fn facet_columns(
        &self,
    ) -> (
        Option<&'a str>,
        Option<&'a str>,
        Option<&'a str>,
        Option<usize>,
    ) {
        (
            self.facet_row_col,
            self.facet_col_col,
            self.facet_wrap_col,
            self.facet_wrap_columns,
        )
    }

    pub(crate) fn without_facets(mut self) -> Self {
        self.facet_row_col = None;
        self.facet_col_col = None;
        self.facet_wrap_col = None;
        self.facet_wrap_columns = None;
        self
    }

    pub(crate) fn with_comparison_subset(
        mut self,
        selectors: &[(String, String)],
    ) -> Result<Self, Error> {
        for comparison in &self.comparisons {
            for (variable, _) in selectors {
                if !comparison
                    .facet
                    .iter()
                    .any(|facet| &facet.variable == variable)
                {
                    return Err(Error::Render(format!(
                        "faceted significance comparison must name facet variable {variable:?}; annotations are never replicated across facets"
                    )));
                }
            }
            if comparison.facet.iter().any(|facet| {
                !selectors
                    .iter()
                    .any(|(variable, _)| variable == &facet.variable)
            }) {
                return Err(Error::Render(
                    "significance comparison references a facet variable not configured on this chart"
                        .into(),
                ));
            }
        }
        self.comparisons.retain(|comparison| {
            selectors.iter().all(|(variable, level)| {
                comparison
                    .facet
                    .iter()
                    .any(|facet| facet.variable == *variable && facet.level == *level)
            })
        });
        Ok(self)
    }

    pub(crate) const fn has_facets(&self) -> bool {
        self.facet_row_col.is_some()
            || self.facet_col_col.is_some()
            || self.facet_wrap_col.is_some()
    }

    pub(crate) fn with_resolved_facet_ranges(
        mut self,
        kind: PlotKind,
        x_range: Option<(f64, f64)>,
        y_range: Option<(f64, f64)>,
    ) -> Self {
        if let Some(range) = x_range {
            if kind == PlotKind::Forest && self.axis_scale == AxisScale::Log10 {
                self.resolved_facet_x_range = Some(range);
            } else {
                self.x_axis_spec.limits = Some(range);
            }
        }
        if let Some(range) = y_range {
            self.y_axis_spec.limits = Some(range);
        }
        self
    }

    pub(crate) const fn resolved_facet_x_range(&self) -> Option<(f64, f64)> {
        self.resolved_facet_x_range
    }

    pub(crate) fn with_global_hue_levels(mut self, levels: Vec<String>) -> Self {
        self.global_hue_levels = Some(levels);
        self
    }

    pub(crate) fn resolved_hue_levels(&self, observed: Vec<String>) -> Vec<String> {
        self.global_hue_levels.clone().unwrap_or(observed)
    }

    pub(crate) fn global_hue_levels(&self) -> Option<&[String]> {
        self.global_hue_levels.as_deref()
    }

    pub(crate) fn with_global_group_levels(mut self, levels: Vec<String>) -> Self {
        self.global_group_levels = Some(levels);
        self
    }

    pub(crate) fn resolved_group_levels(&self, observed: Vec<String>) -> Vec<String> {
        self.global_group_levels.clone().unwrap_or(observed)
    }

    pub(crate) fn with_global_line_condition_levels(mut self, levels: Vec<String>) -> Self {
        self.global_line_condition_levels = Some(levels);
        self
    }

    pub(crate) fn resolved_line_condition_levels(&self, observed: Vec<String>) -> Vec<String> {
        self.global_line_condition_levels
            .clone()
            .unwrap_or(observed)
    }

    pub(crate) fn hue_color_index(&self, label: &str, default: usize) -> usize {
        self.global_hue_levels
            .as_ref()
            .and_then(|levels| levels.iter().position(|level| level == label))
            .unwrap_or(default)
    }

    pub(crate) fn with_render_scope(mut self, scope: impl Into<String>) -> Self {
        self.render_scope = Some(scope.into());
        self
    }

    pub(crate) fn with_measured_axis_text(mut self) -> Self {
        self.axis_text_measured = true;
        self
    }

    pub(crate) fn with_measured_composed_axis_text(mut self) -> Self {
        self.axis_text_measured = true;
        self.composed_axis_text_measured = true;
        self
    }

    pub(crate) fn layer_name(&self, name: &str) -> String {
        self.render_scope
            .as_ref()
            .map_or_else(|| name.to_owned(), |scope| format!("{scope}-{name}"))
    }

    pub(crate) fn with_canvas_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}
