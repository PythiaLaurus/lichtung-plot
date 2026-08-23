use super::Plot;
use crate::data::{FACET_COLUMN, FACET_ROW, FACET_WRAP};
use crate::spec::{AxisScale, FacetScales, FacetSpec, PlotKind};

impl<K: crate::kind::Marker> Plot<K> {
    /// Apply one complete generic-facet layout.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: crate::kind::Facet,
    {
        self.facet = Some(facet);
        self
    }

    pub(crate) const fn facet_scales_spec(&self) -> FacetScales {
        match &self.facet {
            Some(facet) => facet.scale_policy(),
            None => FacetScales::Fixed,
        }
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
            self.facet
                .as_ref()
                .and_then(FacetSpec::row_values)
                .map(|_| FACET_ROW),
            self.facet
                .as_ref()
                .and_then(FacetSpec::column_values)
                .map(|_| FACET_COLUMN),
            self.facet
                .as_ref()
                .and_then(FacetSpec::wrap_values)
                .map(|_| FACET_WRAP),
            self.facet.as_ref().and_then(FacetSpec::wrap_columns),
        )
    }

    pub(crate) fn facet_names(&self) -> (Option<&str>, Option<&str>, Option<&str>) {
        (
            self.facet
                .as_ref()
                .and_then(FacetSpec::row_values)
                .map(crate::CategoryValues::name),
            self.facet
                .as_ref()
                .and_then(FacetSpec::column_values)
                .map(crate::CategoryValues::name),
            self.facet
                .as_ref()
                .and_then(FacetSpec::wrap_values)
                .map(crate::CategoryValues::name),
        )
    }

    pub(crate) fn without_facets(mut self) -> Self {
        self.facet = None;
        self
    }

    pub(crate) fn with_significance_subset<'a>(
        mut self,
        selectors: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let selectors = selectors.into_iter().collect::<Vec<_>>();
        self.significance.retain(|annotation| {
            annotation
                .facet_cell()
                .is_some_and(|cell| cell.matches(selectors.iter().map(|(_, level)| *level)))
        });
        self
    }

    pub(crate) const fn has_facets(&self) -> bool {
        self.facet.is_some()
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
