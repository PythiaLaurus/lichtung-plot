//! Faceted chart coordination.

use crate::Error;

use super::backend::validate_with_null_backend;
use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::canvas::allocate_composed_grid;
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};
use crate::theme::ResolvedTheme;

mod data;
mod legend;
mod planning;
mod rendering;

pub(crate) fn text_census_labels(source: &Plot) -> Result<Vec<String>, Error> {
    let selectors = data::facet_selectors(source)?;
    let slots = selectors
        .rows
        .checked_mul(selectors.columns)
        .ok_or_else(|| Error::Render("facet text census grid size overflowed".into()))?;
    let provisional = vec![crate::layout::PanelBox::new(0.0, 1.0, 0.0, 1.0); slots];
    data::extract_panels(source, &selectors, &provisional).map(|panels| {
        panels
            .iter()
            .map(|panel| rendering::facet_label(source, &panel.cell))
            .collect()
    })
}

pub(crate) struct FacetTextCensusPanel {
    plot: Plot,
    rows: Vec<usize>,
    source_rows: Vec<usize>,
}

impl FacetTextCensusPanel {
    pub(crate) fn plot(&self) -> &Plot {
        &self.plot
    }

    pub(crate) fn data(&self) -> crate::data::PlotData<'_> {
        crate::data::PlotData::with_rows(&self.plot, &self.rows, &self.source_rows)
    }
}

pub(crate) fn text_census_panels(
    source: &Plot,
    kind: PlotKind,
) -> Result<Vec<FacetTextCensusPanel>, Error> {
    let selectors = data::facet_selectors(source)?;
    let slots = selectors
        .rows
        .checked_mul(selectors.columns)
        .ok_or_else(|| Error::Render("facet text census grid size overflowed".into()))?;
    let provisional = vec![crate::layout::PanelBox::new(0.0, 1.0, 0.0, 1.0); slots];
    let panels = data::extract_panels(source, &selectors, &provisional)?;
    let (x_label, y_label) = source.resolved_axis_labels(kind);
    let base = data::base_plot(source, kind, !x_label.is_empty(), !y_label.is_empty())?;
    panels
        .into_iter()
        .map(|panel| {
            Ok(FacetTextCensusPanel {
                plot: base
                    .clone()
                    .with_render_scope(panel.render_scope())
                    .with_significance_subset(panel.significance_selectors()),
                rows: panel.rows,
                source_rows: panel.source_rows,
            })
        })
        .collect()
}

pub(super) struct FacetPlan {
    plot: Plot,
    panels: Vec<data::FacetPanel>,
    panel_plans: Vec<crate::decorations::plan::PanelPlan>,
    horizontal_axis_label: Option<String>,
    vertical_axis_label: Option<String>,
    canvas_layout: crate::layout::CanvasLayout,
    outer: crate::layout::PanelBox,
    shared_axis_origin: (f64, f64),
}

impl FacetPlan {
    #[expect(
        clippy::too_many_lines,
        reason = "facet coordination keeps both balanced validation sessions explicit"
    )]
    pub(super) fn new(source: &Plot, kind: PlotKind, theme: &ResolvedTheme) -> Result<Self, Error> {
        if !source.significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}"
            )));
        }

        let selectors = data::facet_selectors(source)?;
        let (horizontal_axis_label, vertical_axis_label) = source.resolved_axis_labels(kind);
        let horizontal_axis_label =
            (!horizontal_axis_label.is_empty()).then_some(horizontal_axis_label);
        let vertical_axis_label = (!vertical_axis_label.is_empty()).then_some(vertical_axis_label);
        let canvas_layout = planning::canvas_layout(
            source,
            theme,
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
        let base_plot = data::base_plot(
            source,
            kind,
            horizontal_axis_label.is_some(),
            vertical_axis_label.is_some(),
        )?;
        let mut panels = data::extract_panels(source, &selectors, &provisional)?;
        data::validate_significance_facet_anchors(source, &panels)?;
        let facet_scales = source.facet_scales_spec();

        let (base_plot, preflight) = validate_with_null_backend(
            source.width(),
            source.height(),
            "facet panel preflight",
            |backend| {
                legend::preflight_coordinated_panels(
                    source, base_plot, kind, &panels, theme, backend,
                )
            },
        )?;
        let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
        let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
        let horizontal_domain = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical_domain = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let shared_plot =
            base_plot.with_resolved_facet_ranges(kind, horizontal_domain, vertical_domain);
        let (reserved_outer, mut outside_legend, legend_entries) =
            legend::reserve_shared_legend(source, &shared_plot, &preflight, outer, theme)?;
        outer = planning::reserve_rewrapped_shared_axis_titles(
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
            theme,
            canvas_layout.panel_box,
            reserved_outer,
        )?;
        let shared_axis_origin = position_shared_axis_guides(
            source.legend_position_spec(),
            canvas_layout.panel_box,
            reserved_outer,
            outer,
            outside_legend.as_mut(),
        );

        validate_with_null_backend(
            source.width(),
            source.height(),
            "facet layout preflight",
            |backend| {
                planning::layout_panels(
                    &shared_plot,
                    kind,
                    &mut panels,
                    outer,
                    (selectors.rows, selectors.columns),
                    facet_scales,
                    theme,
                    backend,
                )
            },
        )?;
        let shared_plot = shared_plot.with_measured_axis_text();
        let (shared_plot, panel_plans) = planning::plan_panels(
            shared_plot,
            kind,
            &panels,
            outside_legend,
            &legend_entries,
            facet_scales,
            theme,
        )?;
        Ok(Self {
            plot: shared_plot,
            panels,
            panel_plans,
            horizontal_axis_label,
            vertical_axis_label,
            canvas_layout,
            outer,
            shared_axis_origin,
        })
    }

    pub(super) fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        rendering::render_scene(
            source,
            &self.plot,
            kind,
            &self.panels,
            &self.panel_plans,
            self.horizontal_axis_label.as_deref(),
            self.vertical_axis_label.as_deref(),
            &self.canvas_layout,
            self.outer,
            self.shared_axis_origin,
            theme,
            backend,
        )
    }
}

pub(super) fn validate_before_font_preparation(source: &Plot, kind: PlotKind) -> Result<(), Error> {
    if !source.significance_annotations().is_empty() && !kind.supports_significance_annotations() {
        return Err(Error::Data(format!(
            "significance annotations are not supported for {kind:?}"
        )));
    }
    let selectors = data::facet_selectors(source)?;
    let cell_count = selectors
        .rows
        .checked_mul(selectors.columns)
        .ok_or_else(|| Error::Render("facet semantic grid size overflowed".into()))?;
    let mut provisional = Vec::new();
    provisional
        .try_reserve_exact(cell_count)
        .map_err(|_| Error::Render("facet semantic grid allocation failed".into()))?;
    provisional.resize(cell_count, crate::layout::PanelBox::new(0.0, 1.0, 0.0, 1.0));
    let panels = data::extract_panels(source, &selectors, &provisional)?;
    data::validate_significance_facet_anchors(source, &panels)
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
