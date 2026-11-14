use crate::decorations::title_block::plan::RootAlignments;
use crate::font::canonical::OperationContext;
use crate::layout::fit::{FitScope, PlanError};
use crate::layout::{RootFramePlan, RootSize};
use crate::orchestrator::assignment::PlotMeasureAssignment;
use crate::orchestrator::registration::PlotRegistration;
use crate::orchestrator::root::PlotIntrinsics;
use crate::text::TextRequestContext;

#[derive(Clone, Copy)]
pub(crate) enum RootFrameDomainInput<'v> {
    Prepared,
    Selected(&'v PlotMeasureAssignment),
}

pub(crate) fn measure_root_frame(
    registered: &PlotRegistration<'_>,
    size: RootSize,
    domains: RootFrameDomainInput<'_>,
    request: &TextRequestContext<'_, '_>,
    intrinsic: &PlotIntrinsics,
    operation: &OperationContext,
    _scope: &FitScope,
) -> Result<RootFramePlan, PlanError> {
    #[cfg(test)]
    crate::orchestrator::root::FRAME_CACHE_READS
        .with(|events| events.borrow_mut().push("frame minimum"));
    let minimum = crate::layout::interior::intrinsic_body_minimum(
        registered, domains, intrinsic, operation, request,
    )?;
    let source = registered.source;
    let mut root = RootFramePlan::measure(
        size.dimensions()?,
        request,
        source.prose(),
        RootAlignments::resolve(request, source.prose_alignments(), [None; 2]),
        minimum,
    )?;
    root.native_preference = match crate::layout::interior::InteriorRoute::classify(source.kind()) {
        crate::layout::interior::InteriorRoute::StandardCoordinate(_) => Some(
            crate::layout::interior::preference::CoordinateSizePreference::Standard(
                crate::layout::interior::preference::mark_preference(
                    registered, domains, request, minimum,
                )?,
            ),
        ),
        crate::layout::interior::InteriorRoute::OneCategoricalAxis(_) => Some(
            crate::layout::interior::preference::CoordinateSizePreference::Categorical(
                crate::layout::interior::preference::categorical_preference(
                    registered, domains, request, minimum,
                )?,
            ),
        ),
        crate::layout::interior::InteriorRoute::SpecializedGrid(_) => None,
    };
    Ok(root)
}
