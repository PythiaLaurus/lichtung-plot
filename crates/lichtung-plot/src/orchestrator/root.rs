//! Operation-local measurement state, borrowed for one measurement round.

use crate::builder::PlotDescription;
use crate::decorations::legend::discrete::GuideSlot;
use crate::decorations::plan::children::ChildPlans;
use crate::decorations::title_block::plan::RootTitleBlockIntrinsics;
use crate::font::canonical::{OperationContext, SealedTextVerifier};
use crate::layout::kernel::{FeedbackRound, PreparedStage};
use crate::layout::{MeasuredLayout, RootRequest};
use crate::orchestrator::assignment::{
    AssignmentFactory, PlotMeasureAssignmentFactory, RootAssignment,
};
use crate::orchestrator::registration::{PlotOccurrenceId, PlotRegistration, RootIdentity};
use crate::text::TextService;
use crate::theme::ResolvedTheme;
use crate::{Canvas, Error, FacetCell};
use std::cell::{Cell, RefCell};

#[cfg(test)]
thread_local! {
    pub(crate) static FRAME_CACHE_READS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy)]
pub(crate) enum RootSource<'src> {
    Plot(&'src PlotDescription),
    Canvas(&'src Canvas),
}

impl RootSource<'_> {
    pub(crate) fn request(&self) -> RootRequest {
        match self {
            Self::Plot(source) => source.sizing_request(),
            Self::Canvas(source) => source.sizing_request(),
        }
    }
}

#[expect(
    clippy::struct_field_names,
    reason = "root_appearance and root_text distinguish root-owned appearance and title-block intrinsics from per-plot state and text services"
)]
pub(crate) struct Root<'src> {
    pub(crate) source: RootSource<'src>,
    pub(crate) identity: RootIdentity,
    pub(crate) operation: OperationContext,
    pub(crate) next_candidate: Cell<usize>,
    pub(crate) root_appearance: ResolvedTheme,
    pub(crate) plots: Vec<PlotRegistration<'src>>,
    pub(crate) composition: PreparedComposition,
    pub(crate) census: crate::layout::PreparedTextCensus,
    pub(crate) root_text: RootTitleBlockIntrinsics,
    pub(crate) text: TextService,
    pub(crate) plot_intrinsics: Vec<(PlotOccurrenceId, PlotIntrinsics)>,
    pub(crate) composition_intrinsics: CompositionIntrinsics,
    pub(crate) caches: RefCell<RootLayoutCaches>,
}

pub(crate) enum PreparedComposition {
    Single { plot_id: PlotOccurrenceId },
    Facets(PreparedFacetTopology),
    Canvas(PreparedCanvasTopology),
}

pub(crate) enum CanvasHeaderRole {
    Column,
    Row,
    RowTitle,
}

pub(crate) struct CanvasHeaderSource {
    pub(crate) role: CanvasHeaderRole,
    pub(crate) declaration: usize,
}

pub(crate) struct PreparedCanvasTopology {
    pub(crate) slots: Vec<Option<CanvasAnchor>>,
    pub(crate) producers: Vec<GridSlot>,
    pub(crate) headers: Vec<CanvasHeaderSource>,
    pub(crate) range_groups: Vec<Vec<PlotOccurrenceId>>,
    pub(crate) category_bands: Vec<Vec<PlotOccurrenceId>>,
    pub(crate) shared_guide: Option<GuideSlot>,
}

pub(crate) enum DeclaredFacetSelector {
    Single {
        #[expect(
            dead_code,
            reason = "retains the declared single-facet level ordinal as passive topology identity"
        )]
        level: usize,
    },
    Grid {
        #[expect(
            dead_code,
            reason = "retains the declared row-level ordinal as passive crossed-facet identity"
        )]
        row_level: usize,
        #[expect(
            dead_code,
            reason = "retains the declared column-level ordinal as passive crossed-facet identity"
        )]
        column_level: usize,
    },
}

pub(crate) struct FacetSlot {
    pub(crate) selector: Option<DeclaredFacetSelector>,
    pub(crate) plot_id: Option<PlotOccurrenceId>,
}

pub(crate) struct PreparedFacetTopology {
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) slots: Vec<FacetSlot>,
    pub(crate) producers: Vec<GridSlot>,
    pub(crate) shared_axis_members: [Vec<PlotOccurrenceId>; 2],
    pub(crate) shared_titles_from: PlotOccurrenceId,
    pub(crate) parent_guide: Option<GuideSlot>,
}

pub(crate) struct FacetTopologyShell {
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) slots: Vec<FacetSlot>,
}

pub(crate) struct FacetSelectors {
    pub(crate) shell: FacetTopologyShell,
    pub(crate) cells: Vec<(GridSlot, FacetCell)>,
}

pub(crate) enum CompositionIntrinsics {
    Single,
    Facets,
    Canvas {
        text: Vec<Option<crate::canvas::text_panel::TextPanelIntrinsics>>,
        prose: Vec<(PlotOccurrenceId, RootTitleBlockIntrinsics)>,
    },
}

impl<'src> Root<'src> {
    pub(crate) fn prepare(
        source: RootSource<'src>,
        stage: &mut PreparedStage<'_>,
    ) -> Result<Self, Error> {
        match source {
            RootSource::Plot(source) => super::lifecycle::prepare_plot_root(source, stage),
            RootSource::Canvas(source) => source.prepare_root(stage),
        }
    }

    pub(crate) fn request(&self) -> crate::text::TextRequestContext<'_, 'src> {
        crate::text::TextRequestContext::new(&self.root_appearance, &self.text)
    }

    pub(crate) fn assignment_factory(&self) -> AssignmentFactory<'_> {
        AssignmentFactory {
            identity: &self.identity,
            next_candidate: &self.next_candidate,
            original_request: self.source.request(),
        }
    }

    pub(crate) fn plot(&self, id: &PlotOccurrenceId) -> Result<&PlotRegistration<'src>, Error> {
        self.plots
            .get(id.slot)
            .filter(|plot| plot.scope.id == *id)
            .ok_or_else(|| {
                Error::Render("row source belongs to a different source or operation".into())
            })
    }

    pub(crate) fn plot_intrinsic(&self, id: &PlotOccurrenceId) -> Result<&PlotIntrinsics, Error> {
        self.plot_intrinsics
            .iter()
            .find(|(found, _)| found == id)
            .map(|(_, intrinsic)| intrinsic)
            .ok_or_else(|| Error::Render("prepared plot intrinsic scope missing".into()))
    }

    pub(crate) fn plot_assignment_factory<'v>(
        &'v self,
        root: &'v RootAssignment,
        plot_id: &PlotOccurrenceId,
    ) -> Result<PlotMeasureAssignmentFactory<'v, 'src>, Error> {
        Ok(PlotMeasureAssignmentFactory {
            issuer: self.assignment_factory(),
            root,
            registered: self.plot(plot_id)?,
            text: &self.text,
            canvas: match self.source {
                RootSource::Canvas(source) => Some(source),
                RootSource::Plot(_) => None,
            },
            operation: &self.operation,
            intrinsic: self.plot_intrinsic(plot_id)?,
            registrations: &self.plots,
            composition: &self.composition,
        })
    }

    pub(crate) fn sealed_verifier(&self) -> Result<SealedTextVerifier<'_>, Error> {
        Ok(SealedTextVerifier::new(
            self.text.prepared_font_environment()?,
        ))
    }

    pub(crate) fn measure(
        &self,
        assigned: RootRequest,
        round: &mut FeedbackRound,
    ) -> Result<MeasuredLayout<super::plan::MeasuredOwnedPlan>, Error> {
        let mut caches = self.caches.borrow_mut();
        let RootLayoutCaches {
            children,
            heights,
            frames,
        } = &mut *caches;
        let plan = match self.source {
            RootSource::Plot(source) => super::lifecycle::measure_plot_root(
                self, source, assigned, round, children, frames,
            )?,
            RootSource::Canvas(source) => {
                source.measure_root(self, assigned, round, children, heights)?
            }
        };
        children.finish_round(round);
        Ok(MeasuredLayout::new(plan.size, plan))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct GridSlot(pub(crate) usize);

pub(crate) enum CanvasAnchor {
    Plot(crate::orchestrator::registration::PlotOccurrenceId),
    Text,
}

/// One occurrence's post-font analysis. Native readers retain their original
/// lazy initialization; scientific axes own no measurement state.
#[derive(Debug)]
pub(crate) enum PlotIntrinsics {
    None,
    Coordinate {
        value: std::sync::OnceLock<crate::layout::interior::coordinate::Intrinsics>,
    },
    Heatmap {
        value: std::sync::OnceLock<crate::plots::heatmap::HeatmapIntrinsicCache>,
    },
}

impl PlotIntrinsics {
    pub(crate) const fn for_kind(kind: crate::PlotKind) -> Self {
        match kind {
            crate::PlotKind::Matrix => Self::None,
            crate::PlotKind::Heatmap => Self::Heatmap {
                value: std::sync::OnceLock::new(),
            },
            _ => Self::Coordinate {
                value: std::sync::OnceLock::new(),
            },
        }
    }
}

pub(crate) struct PlotIntrinsicInputs<'v, 'src> {
    pub(crate) registered: &'v crate::orchestrator::registration::PlotRegistration<'src>,
    pub(crate) request: crate::text::TextRequestContext<'v, 'src>,
    pub(crate) operation: &'v crate::font::canonical::OperationContext,
    pub(crate) y_visible: bool,
    pub(crate) diagnostic_scope: Option<&'v str>,
}

pub(crate) fn coordinate_intrinsic<'a>(
    slot: &'a PlotIntrinsics,
    input: &PlotIntrinsicInputs<'_, '_>,
) -> Result<&'a crate::layout::interior::coordinate::Intrinsics, crate::Error> {
    crate::layout::interior::coordinate::Intrinsics::prepare(slot, input)
}

pub(crate) fn prepare_plot_intrinsics(
    slot: &PlotIntrinsics,
    input: &PlotIntrinsicInputs<'_, '_>,
) -> Result<(), crate::Error> {
    let kind = input.registered.source.kind();
    if crate::layout::interior::InteriorRoute::classify(kind).is_coordinate() {
        coordinate_intrinsic(slot, input)?;
    } else if kind == crate::PlotKind::Heatmap {
        if !matches!(
            input.registered.family,
            crate::orchestrator::registration::PreparedFamily::Heatmap(_)
        ) {
            return Err(crate::Error::Render(
                "prepared heatmap intrinsic scope missing".into(),
            ));
        }
        crate::plots::heatmap::heatmap_minima(slot, input)?;
    }
    Ok(())
}

pub(crate) fn intrinsic_inputs<'v, 'src>(
    registered: Option<&'v crate::orchestrator::registration::PlotRegistration<'src>>,
    request: &crate::text::TextRequestContext<'v, 'src>,
    operation: &'v crate::font::canonical::OperationContext,
) -> Result<PlotIntrinsicInputs<'v, 'src>, crate::Error> {
    Ok(PlotIntrinsicInputs {
        registered: registered.ok_or_else(|| {
            crate::Error::Render("native coordinate panel requires root-prepared axes".into())
        })?,
        request: *request,
        operation,
        y_visible: crate::orchestrator::registration::axis_tick_labels_visible(registered, 1),
        diagnostic_scope: crate::orchestrator::registration::render_name(registered),
    })
}
pub(crate) struct RootLayoutCaches {
    pub(crate) children: ChildPlans,
    pub(crate) heights: crate::canvas::HeightMeasurements,
    pub(crate) frames: Option<CachedRootFrame>,
}

pub(crate) struct CachedRootFrame {
    pub(crate) prose: [crate::Text; 3],
    pub(crate) dimensions: (u32, u32),
    pub(crate) minimum: [u64; 2],
    pub(crate) frame: std::sync::Arc<crate::layout::RootFramePlan>,
}

impl RootLayoutCaches {
    pub(crate) fn new(children: ChildPlans) -> Self {
        Self {
            children,
            heights: crate::canvas::HeightMeasurements::default(),
            frames: None,
        }
    }
}
