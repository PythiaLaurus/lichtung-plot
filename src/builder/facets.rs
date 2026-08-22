use crate::Error;

use super::Plot;
use crate::data::{FACET_COLUMN, FACET_ROW, FACET_WRAP};
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::CategoryValues;

impl Plot {
    /// Split the selected plot into rows by categorical values.
    #[must_use]
    pub fn facet_row(mut self, values: CategoryValues) -> Self {
        self.facet_row = Some(values);
        self
    }

    /// Split the selected plot into columns by categorical values.
    #[must_use]
    pub fn facet_col(mut self, values: CategoryValues) -> Self {
        self.facet_col = Some(values);
        self
    }

    /// Wrap categorical facet values into the requested number of columns.
    #[must_use]
    pub fn facet_wrap(mut self, values: CategoryValues, columns: usize) -> Self {
        self.facet_wrap = Some(values);
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

    pub(crate) fn facet_columns(
        &self,
    ) -> (
        Option<&'static str>,
        Option<&'static str>,
        Option<&'static str>,
        Option<usize>,
    ) {
        (
            self.facet_row.as_ref().map(|_| FACET_ROW),
            self.facet_col.as_ref().map(|_| FACET_COLUMN),
            self.facet_wrap.as_ref().map(|_| FACET_WRAP),
            self.facet_wrap_columns,
        )
    }

    pub(crate) fn without_facets(mut self) -> Self {
        self.facet_row = None;
        self.facet_col = None;
        self.facet_wrap = None;
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
        self.facet_row.is_some() || self.facet_col.is_some() || self.facet_wrap.is_some()
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
