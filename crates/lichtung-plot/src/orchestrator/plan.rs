//! Owning selected root content, independent of the operation that measured it.

pub(crate) struct MeasuredOwnedPlan {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "retains the initial root allocation separately from completion; Figure uses the selected plot dimensions"
        )
    )]
    pub(crate) measurement: super::assignment::RootAssignment,
    pub(crate) size: crate::layout::RootSize,
    pub(crate) frame: std::sync::Arc<crate::layout::RootFramePlan>,
    pub(crate) root_paint: crate::Color,
    pub(crate) content: MeasuredRootContent,
}

pub(crate) enum MeasuredRootContent {
    Single {
        plot: std::sync::Arc<crate::decorations::plan::MeasuredPlotPlan>,
        legend: Option<(String, crate::decorations::legend::LegendDrawPlan)>,
    },
    Facets(super::facets::OwnedFacetPlan),
    Canvas(crate::canvas::OwnedCanvasPlan),
}
