//! Faceted chart coordination.

use crate::Error;

use super::backend::NullBackend;
use crate::builder::Chart;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::figure::allocate_composed_grid;
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};

mod data;
mod legend;
mod planning;
mod rendering;

impl Chart<'_> {
    pub(super) fn render_facets(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        if !self.comparisons().is_empty() && !kind.supports_significance() {
            return Err(Error::Render(format!(
                "significance comparisons are not supported for {kind:?}"
            )));
        }

        let selectors = data::facet_selectors(self)?;
        let palette = self.palette();
        let (horizontal_axis_label, vertical_axis_label) = self.resolved_axis_labels(kind);
        let horizontal_axis_label = (palette.show_x_label && !horizontal_axis_label.is_empty())
            .then_some(horizontal_axis_label);
        let vertical_axis_label = (palette.show_y_label && !vertical_axis_label.is_empty())
            .then_some(vertical_axis_label);
        let figure_layout = planning::figure_layout(
            self,
            &palette,
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
        )?;
        let mut outer = figure_layout.panel_box;
        let provisional = allocate_composed_grid(
            outer,
            selectors.rows,
            selectors.columns,
            &[],
            "facet layout",
        )?;
        let base = data::base_chart(
            self,
            kind,
            horizontal_axis_label.is_some(),
            vertical_axis_label.is_some(),
        )?;
        let mut panels = data::extract_panels(self, &selectors, &provisional)?;
        data::validate_comparison_facet_anchors(self, &panels)?;

        let mut null_backend = NullBackend;
        let preflight = legend::preflight_panels(&base, kind, &panels, &mut null_backend)?;
        let base = legend::coordinate_hue_levels(self, base, &preflight)?;
        let fix_x = matches!(
            self.facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeY
        );
        let fix_y = matches!(
            self.facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeX
        );
        let horizontal_domain = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical_domain = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let shared = base.with_resolved_facet_ranges(kind, horizontal_domain, vertical_domain);
        let (reserved_outer, mut outside_legend) =
            legend::reserve_shared_legend(self, &shared, &preflight, outer)?;
        outer = planning::reserve_rewrapped_shared_axis_titles(
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
            &palette,
            figure_layout.panel_box,
            reserved_outer,
        )?;
        let shared_axis_origin = position_shared_axis_guides(
            self.legend_position_spec(),
            figure_layout.panel_box,
            reserved_outer,
            outer,
            outside_legend.as_mut(),
        );

        planning::layout_panels(
            &shared,
            kind,
            &mut panels,
            outer,
            selectors.rows,
            selectors.columns,
            &mut null_backend,
        )?;
        let shared = shared.with_measured_axis_text();
        let (shared, plans) =
            planning::plan_panels(shared, kind, &panels, outside_legend, fix_x, fix_y)?;
        rendering::render_scene(
            self,
            &shared,
            kind,
            panels,
            plans,
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
            outer,
            shared_axis_origin,
            &palette,
            backend,
        )
    }
}

fn position_shared_axis_guides(
    position: LegendPosition,
    figure_panel: crate::layout::PanelBox,
    reserved_outer: crate::layout::PanelBox,
    outer: crate::layout::PanelBox,
    legend: Option<&mut ResolvedLegend>,
) -> (f64, f64) {
    let mut origin = (outer.x0, outer.y0);
    let Some(legend) = legend else {
        return origin;
    };
    let extra_left = outer.x0 - reserved_outer.x0;
    let extra_bottom = outer.y0 - reserved_outer.y0;
    match position {
        LegendPosition::OutsideLeftTop
        | LegendPosition::OutsideLeft
        | LegendPosition::OutsideLeftBottom => {
            origin.0 = figure_panel.x0 + extra_left;
            legend.bounds = legend.bounds.translated(extra_left, 0.0);
        }
        LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => {
            origin.1 = figure_panel.y0 + extra_bottom;
            legend.bounds = legend.bounds.translated(0.0, extra_bottom);
        }
        _ => {}
    }
    origin
}
