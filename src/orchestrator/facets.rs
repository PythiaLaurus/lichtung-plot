//! Faceted chart coordination.

use crate::Error;

use super::backend::NullBackend;
use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::canvas::allocate_composed_grid;
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};

mod data;
mod legend;
mod planning;
mod rendering;

impl Plot {
    pub(super) fn render_facets(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}"
            )));
        }

        let selectors = data::facet_selectors(self)?;
        let palette = self.palette();
        let (horizontal_axis_label, vertical_axis_label) = self.resolved_axis_labels(kind);
        let horizontal_axis_label = (palette.show_x_label && !horizontal_axis_label.is_empty())
            .then_some(horizontal_axis_label);
        let vertical_axis_label = (palette.show_y_label && !vertical_axis_label.is_empty())
            .then_some(vertical_axis_label);
        let canvas_layout = planning::canvas_layout(
            self,
            &palette,
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
        )?;
        let mut outer = canvas_layout.panel_box;
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
        data::validate_significance_facet_anchors(self, &panels)?;
        let facet_scales = self.facet_scales_spec();

        let mut null_backend = NullBackend;
        let (base, preflight) =
            legend::preflight_coordinated_panels(self, base, kind, &panels, &mut null_backend)?;
        let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
        let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
        let horizontal_domain = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical_domain = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let shared = base.with_resolved_facet_ranges(kind, horizontal_domain, vertical_domain);
        let (reserved_outer, mut outside_legend, legend_entries) =
            legend::reserve_shared_legend(self, &shared, &preflight, outer)?;
        outer = planning::reserve_rewrapped_shared_axis_titles(
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
            &palette,
            canvas_layout.panel_box,
            reserved_outer,
        )?;
        let shared_axis_origin = position_shared_axis_guides(
            self.legend_position_spec(),
            canvas_layout.panel_box,
            reserved_outer,
            outer,
            outside_legend.as_mut(),
        );

        planning::layout_panels(
            &shared,
            kind,
            &mut panels,
            outer,
            (selectors.rows, selectors.columns),
            facet_scales,
            &mut null_backend,
        )?;
        let shared = shared.with_measured_axis_text();
        let (shared, plans) = planning::plan_panels(
            shared,
            kind,
            &panels,
            outside_legend,
            &legend_entries,
            facet_scales,
        )?;
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
    canvas_panel: crate::layout::PanelBox,
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
            origin.0 = canvas_panel.x0 + extra_left;
            legend.bounds = legend.bounds.translated(extra_left, 0.0);
        }
        LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => {
            origin.1 = canvas_panel.y0 + extra_bottom;
            legend.bounds = legend.bounds.translated(0.0, extra_bottom);
        }
        _ => {}
    }
    origin
}
