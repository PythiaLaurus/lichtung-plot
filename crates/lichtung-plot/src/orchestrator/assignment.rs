//! Per-call dimensions, distinct from the root's completed output size.

use std::cell::Cell;

use crate::decorations::legend::discrete::scoped::{GuideAllocation, GuideConsumption};
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::fit::FitScope;
use crate::layout::kernel::{FeedbackKind, FeedbackRound};
use crate::layout::{PixelRect, RangeProvenance, RootRequest, RootSize};
use crate::orchestrator::registration::{
    BaseAxisRef, CandidateId, DisplayExtent, PlotOccurrenceId, PlotRegistration, PreparedAxesOwner,
    RootIdentity,
};
use crate::orchestrator::root::{PlotIntrinsics, PreparedComposition};
use crate::spec::{AxisScale, FacetScales, LegendPosition, PlotKind, PlotMeasureContext};
use crate::Error;

/// A selected domain owns scale coordinates, never authored raw endpoints.
#[derive(Debug, Clone)]
pub(crate) struct AxisDomainAssignment {
    pub(crate) extent: DisplayExtent,
    pub(crate) provenance: crate::layout::RangeProvenance,
}

#[derive(Clone, Copy)]
pub(crate) struct AssignedAxisRef<'v> {
    pub(crate) base: BaseAxisRef<'v>,
    pub(crate) domain: &'v AxisDomainAssignment,
}

#[derive(Clone, Copy)]
pub(crate) struct AxisPairRef<'v> {
    pub(crate) axes: [AssignedAxisRef<'v>; 2],
}

pub(crate) enum AxisPairScope {
    Pair,
    MatrixCell { row: usize, column: usize },
    MatrixDiagonalDensity { variable: usize },
    MarginalCentral,
    MarginalTop,
    MarginalRight,
}

#[derive(Debug, Clone)]
pub(crate) enum AxisDomainAssignments {
    Pair([AxisDomainAssignment; 2]),
    Matrix {
        variables: Vec<AxisDomainAssignment>,
        diagonal_density: Vec<AxisDomainAssignment>,
    },
    Marginal {
        central: [AxisDomainAssignment; 2],
        strip_density: [AxisDomainAssignment; 2],
    },
}

pub(crate) struct RootAssignment {
    pub(crate) candidate: CandidateId,
    pub(crate) request: RootRequest,
    pub(crate) measurement_size: RootSize,
}

#[derive(Clone, Copy)]
pub(crate) struct RootPlotAssignmentRef<'a> {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "retains root lineage for the plot view; drawing dimensions belong to the local assignment"
        )
    )]
    pub(crate) root: &'a RootAssignment,
    pub(crate) local: &'a PlotMeasureAssignment,
}

/// The selected allocation remains distinct from its completed root frame.
#[derive(Debug, Clone)]
pub(crate) struct PlotMeasureAssignment {
    pub(crate) plot_id: PlotOccurrenceId,
    #[expect(
        dead_code,
        reason = "retains a pure candidate label on the selected allocation, not a rendering input"
    )]
    pub(crate) candidate: CandidateId,
    pub(crate) dimensions: PlotMeasureDimensions,
    pub(crate) cell: PixelRect,
    pub(crate) frame: PixelRect,
    pub(crate) body: PixelRect,
    pub(crate) axis_label_width: f64,
    pub(crate) content_height: bool,
    pub(crate) fill_content_height: bool,
    pub(crate) content_preference: Option<f64>,
    pub(crate) insets: ParentInsets,
    pub(crate) domains: AxisDomainAssignments,
    pub(crate) guide: GuideTrialAssignment,
}

impl PlotMeasureAssignment {
    pub(crate) fn ranges(&self) -> Option<[(f64, f64); 2]> {
        match &self.domains {
            AxisDomainAssignments::Pair(pair) => {
                Some(pair.each_ref().map(|axis| axis.extent.pair()))
            }
            AxisDomainAssignments::Marginal { central, .. } => {
                Some(central.each_ref().map(|axis| axis.extent.pair()))
            }
            AxisDomainAssignments::Matrix { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParentInsets {
    pub(crate) axis_gutter: [i64; 4],
    pub(crate) owned_ink: [i64; 4],
    pub(crate) shared_alignment: [i64; 4],
}

impl ParentInsets {
    pub(crate) fn consumed(
        round: &crate::layout::kernel::FeedbackRound,
        address: &crate::layout::ComponentAddress,
    ) -> Self {
        use crate::layout::kernel::FeedbackKind;
        Self {
            axis_gutter: round.insets_quanta(address, FeedbackKind::AxisGutter),
            owned_ink: round.insets_quanta(address, FeedbackKind::OwnedInk),
            shared_alignment: round.insets_quanta(address, FeedbackKind::SharedAlignment),
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "consumed insets retain the original separate exact-root-bounded pixel conversions before addition"
    )]
    pub(crate) fn assigned_gutters(&self) -> [f64; 4] {
        let ink = self.owned_ink_pixels();
        std::array::from_fn(|side| self.axis_gutter[side] as f64 / 1024.0 + ink[side])
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "consumed owned ink is exact-root-bounded layout quanta"
    )]
    pub(crate) fn owned_ink_pixels(&self) -> [f64; 4] {
        self.owned_ink.map(|value| value as f64 / 1024.0)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GuideTrialAssignment {
    pub(crate) edge: LegendPosition,
    pub(crate) compact: bool,
    pub(crate) allocation: Option<GuideAllocation>,
}

pub(crate) struct AssignmentFactory<'r> {
    pub(crate) identity: &'r RootIdentity,
    pub(crate) next_candidate: &'r Cell<usize>,
    pub(crate) original_request: RootRequest,
}

impl AssignmentFactory<'_> {
    pub(crate) fn assign(&self, measurement_size: RootSize) -> RootAssignment {
        let serial = self.next_candidate.get();
        self.next_candidate.set(serial + 1);
        RootAssignment {
            candidate: CandidateId {
                root: self.identity.clone(),
                serial,
            },
            request: self.original_request,
            measurement_size,
        }
    }
}

pub(crate) struct PlotMeasureAssignmentFactory<'r, 'src> {
    pub(crate) issuer: AssignmentFactory<'r>,
    pub(crate) root: &'r RootAssignment,
    pub(crate) registered: &'r PlotRegistration<'src>,
    pub(crate) text: &'r crate::text::TextService,
    pub(crate) canvas: Option<&'src crate::Canvas>,
    pub(crate) operation: &'r crate::font::canonical::OperationContext,
    pub(crate) intrinsic: &'r PlotIntrinsics,
    pub(crate) registrations: &'r [PlotRegistration<'src>],
    pub(crate) composition: &'r PreparedComposition,
}

pub(crate) struct PlotAllocationInputs {
    pub(crate) dimensions: PlotMeasureDimensions,
    pub(crate) cell: PixelRect,
    pub(crate) frame: PixelRect,
    pub(crate) body: PixelRect,
    pub(crate) axis_label_width: f64,
    pub(crate) content_height: bool,
    pub(crate) fill_content_height: bool,
    pub(crate) content_preference: Option<f64>,
    pub(crate) insets: ParentInsets,
    pub(crate) guide: GuideTrialAssignment,
}

#[cfg(test)]
impl PlotAllocationInputs {
    pub(crate) fn for_test(
        size: RootSize,
        panel: PixelRect,
        round: &FeedbackRound,
        owner: &crate::layout::ComponentAddress,
    ) -> Self {
        Self {
            dimensions: PlotMeasureDimensions {
                canvas: size,
                output_override: None,
            },
            cell: panel,
            frame: panel,
            body: panel,
            axis_label_width: panel.width(),
            content_height: false,
            fill_content_height: false,
            content_preference: None,
            insets: ParentInsets::consumed(round, owner),
            guide: GuideTrialAssignment {
                edge: LegendPosition::Auto,
                compact: false,
                allocation: None,
            },
        }
    }
}

pub(crate) struct PlotMeasureBranch {
    pub(crate) root: RootAssignment,
    pub(crate) entry: PlotMeasureAssignment,
}

pub(crate) struct GuideTrialChoice {
    pub(crate) edge: LegendPosition,
    pub(crate) compact: bool,
}

pub(crate) enum PlotAssignmentChange {
    ContentPreference {
        frame: PixelRect,
        preferred: f64,
    },
    OutsideGuide {
        body: PixelRect,
        axis_label_width: f64,
        allocation: GuideAllocation,
    },
    AxisInsets {
        body: PixelRect,
        axis_label_width: f64,
    },
    NumericExplanation {
        body: PixelRect,
    },
}

impl<'r, 'src: 'r> PlotMeasureAssignmentFactory<'r, 'src> {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the fixed factory interface returns Result while context creation only borrows prepared operands"
    )]
    pub(crate) fn context<'c>(
        &'c self,
        local: &'c PlotMeasureAssignment,
        guides: GuideConsumption<'c>,
        fit_scope: &'c FitScope,
    ) -> Result<PlotMeasureContext<'c, 'src>, Error> {
        Ok(PlotMeasureContext {
            registered: self.registered,
            assignment: RootPlotAssignmentRef {
                root: self.root,
                local,
            },
            canvas: self.canvas,
            text: self.text,
            operation: self.operation,
            intrinsic: self.intrinsic,
            guides,
            fit_scope,
        })
    }

    pub(crate) fn assign(
        &self,
        inputs: PlotAllocationInputs,
        consumed: &FeedbackRound,
        measurement_phase: Option<MeasurementPhase>,
    ) -> Result<PlotMeasureAssignment, Error> {
        let domains = self.entry_domains(consumed, measurement_phase)?;
        Ok(PlotMeasureAssignment {
            plot_id: self.registered.scope.id.clone(),
            candidate: self.root.candidate.clone(),
            dimensions: inputs.dimensions,
            cell: inputs.cell,
            frame: inputs.frame,
            body: inputs.body,
            axis_label_width: inputs.axis_label_width,
            content_height: inputs.content_height,
            fill_content_height: inputs.fill_content_height,
            content_preference: inputs.content_preference,
            domains,
            insets: inputs.insets,
            guide: inputs.guide,
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "a guide trial owns its choice input alongside the returned root and local assignment"
    )]
    pub(crate) fn fork(
        &self,
        entry: &PlotMeasureAssignment,
        choice: GuideTrialChoice,
        consumed: &FeedbackRound,
    ) -> Result<PlotMeasureBranch, Error> {
        let root = self.issuer.assign(self.root.measurement_size);
        let mut domains = entry.domains.clone();
        // A trial starts with the caller's selected domains, not another
        // unconditional reduction of the prepared bases.
        self.attempt_domains(&mut domains, consumed)?;
        let local = PlotMeasureAssignment {
            plot_id: entry.plot_id.clone(),
            candidate: root.candidate.clone(),
            dimensions: entry.dimensions,
            cell: entry.cell,
            frame: entry.frame,
            body: entry.body,
            axis_label_width: entry.axis_label_width,
            content_height: entry.content_height,
            fill_content_height: entry.fill_content_height,
            content_preference: entry.content_preference,
            domains,
            insets: ParentInsets {
                axis_gutter: entry.insets.axis_gutter,
                owned_ink: entry.insets.owned_ink,
                shared_alignment: entry.insets.shared_alignment,
            },
            guide: GuideTrialAssignment {
                edge: choice.edge,
                compact: choice.compact,
                allocation: entry.guide.allocation,
            },
        };
        Ok(PlotMeasureBranch { root, entry: local })
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "the branch factory keeps the fixed fallible interface while borrowing an existing root"
    )]
    pub(crate) fn for_branch<'b>(
        &'b self,
        root: &'b RootAssignment,
    ) -> Result<PlotMeasureAssignmentFactory<'b, 'src>, Error> {
        Ok(PlotMeasureAssignmentFactory {
            issuer: AssignmentFactory {
                identity: self.issuer.identity,
                next_candidate: self.issuer.next_candidate,
                original_request: self.issuer.original_request,
            },
            root,
            registered: self.registered,
            text: self.text,
            canvas: self.canvas,
            operation: self.operation,
            intrinsic: self.intrinsic,
            registrations: self.registrations,
            composition: self.composition,
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        clippy::unnecessary_wraps,
        clippy::unused_self,
        reason = "residual changes use the factory's fixed fallible by-value API even when installing only copied geometry"
    )]
    pub(crate) fn apply_change(
        &self,
        mut from: PlotMeasureAssignment,
        change: PlotAssignmentChange,
    ) -> Result<PlotMeasureAssignment, Error> {
        match change {
            PlotAssignmentChange::ContentPreference { frame, preferred } => {
                from.frame = frame;
                from.body = frame;
                from.content_preference = Some(preferred);
            }
            PlotAssignmentChange::OutsideGuide {
                body,
                axis_label_width,
                allocation,
            } => {
                from.body = body;
                from.axis_label_width = axis_label_width;
                from.guide.allocation = Some(allocation);
            }
            PlotAssignmentChange::AxisInsets {
                body,
                axis_label_width,
            } => {
                from.body = body;
                from.axis_label_width = axis_label_width;
            }
            PlotAssignmentChange::NumericExplanation { body } => from.body = body,
        }
        Ok(from)
    }

    fn entry_domains(
        &self,
        consumed: &FeedbackRound,
        phase: Option<MeasurementPhase>,
    ) -> Result<AxisDomainAssignments, Error> {
        let preflight = matches!(
            (self.composition, phase),
            (
                PreparedComposition::Facets(_),
                Some(MeasurementPhase::Probe | MeasurementPhase::Hue)
            )
        );
        let coordinate = match (self.composition, phase) {
            (PreparedComposition::Single { .. }, _)
            | (PreparedComposition::Canvas(_), Some(MeasurementPhase::Legend)) => false,
            _ => !preflight,
        };
        let pair = |sources: &[crate::orchestrator::registration::PreparedAxisSource; 2]| {
            let mut domains = [
                self.initial_axis(&sources[0], 0, preflight)?,
                self.initial_axis(&sources[1], 1, preflight)?,
            ];
            if coordinate {
                self.coordinate_pair(&mut domains, consumed)?;
            }
            Ok::<_, Error>(domains)
        };
        match &self.registered.axes {
            PreparedAxesOwner::Pair(axes) => Ok(AxisDomainAssignments::Pair(pair(&axes.sources)?)),
            PreparedAxesOwner::Marginal {
                central,
                strip_density,
            } => Ok(AxisDomainAssignments::Marginal {
                central: pair(central)?,
                strip_density: [
                    seed_domain(&strip_density[0]),
                    seed_domain(&strip_density[1]),
                ],
            }),
            PreparedAxesOwner::Matrix {
                variables,
                diagonal_density,
            } => Ok(AxisDomainAssignments::Matrix {
                variables: variables.iter().map(seed_domain).collect(),
                diagonal_density: diagonal_density.iter().map(seed_domain).collect(),
            }),
        }
    }

    fn initial_axis(
        &self,
        source: &crate::orchestrator::registration::PreparedAxisSource,
        side: usize,
        preflight: bool,
    ) -> Result<AxisDomainAssignment, Error> {
        let mut domain = seed_domain(source);
        let authored = if side == 0 {
            self.registered.source.x_axis_spec()
        } else {
            self.registered.source.y_axis_spec()
        };
        if preflight {
            domain.provenance = if authored.window.is_some() {
                RangeProvenance::Configured
            } else {
                RangeProvenance::Automatic
            };
        }
        if let Some(window) = authored.window {
            domain.extent = self.window_extent(side, window.pair())?;
        }
        Ok(domain)
    }

    fn window_extent(&self, side: usize, range: (f64, f64)) -> Result<DisplayExtent, Error> {
        let (lower, upper) = if side == 0
            && self.registered.source.kind() == PlotKind::Forest
            && self.registered.source.axis_scale_spec() == AxisScale::Log10
        {
            (
                crate::numeric::log10(range.0)?,
                crate::numeric::log10(range.1)?,
            )
        } else {
            range
        };
        Ok(DisplayExtent { lower, upper })
    }

    fn coordinate_pair(
        &self,
        domains: &mut [AxisDomainAssignment; 2],
        consumed: &FeedbackRound,
    ) -> Result<(), Error> {
        match self.composition {
            PreparedComposition::Single { .. } => {}
            PreparedComposition::Facets(topology) => {
                for (side, kind) in [
                    FeedbackKind::AutomaticDomainX,
                    FeedbackKind::AutomaticDomainY,
                ]
                .into_iter()
                .enumerate()
                {
                    let fixed = match self.registered.policy.facet_scales {
                        Some(FacetScales::Fixed) => true,
                        Some(FacetScales::FreeY) => side == 0,
                        Some(FacetScales::FreeX) => side == 1,
                        _ => false,
                    };
                    let mut shared = None;
                    if fixed {
                        for id in &topology.shared_axis_members[side] {
                            let registered = self.registration(id);
                            if let Some(range) =
                                consumed.feedback_domain(&registered.scope.address, kind)
                            {
                                shared = Some(union(shared, range));
                            }
                        }
                    } else {
                        shared = consumed.feedback_domain(&self.registered.scope.address, kind);
                    }
                    if let Some(range) = shared {
                        domains[side].extent = self.window_extent(side, range)?;
                    }
                    // The original Forest resolved-x override is applied after
                    // conversion of its working raw window, including feedback.
                    if fixed && self.resolved_forest_x(side) {
                        domains[side].extent = self.registered.axes.sources()[side]
                            .base_seed
                            .expect("prepared axis display seed");
                    }
                }
            }
            PreparedComposition::Canvas(topology) => {
                let mut grouped = false;
                for group in &topology.range_groups {
                    if !group.contains(&self.registered.scope.id) {
                        continue;
                    }
                    let mut shared = None;
                    for id in group {
                        let registered = self.registration(id);
                        let base = registered.axes.sources()[0]
                            .base_seed
                            .expect("prepared shared x member")
                            .pair();
                        let range = consumed
                            .feedback_domain(
                                &registered.scope.address,
                                FeedbackKind::AutomaticDomainX,
                            )
                            .map_or(base, |range| union(Some(base), range));
                        shared = Some(union(shared, range));
                    }
                    let range = shared.expect("validated nonempty x-range group");
                    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
                        return Err(Error::Render(
                            "Canvas shared x domain must be finite and ascending".into(),
                        ));
                    }
                    domains[0].extent = DisplayExtent {
                        lower: range.0,
                        upper: range.1,
                    };
                    grouped = true;
                }
                if !grouped {
                    if let Some(range) = consumed.feedback_domain(
                        &self.registered.scope.address,
                        FeedbackKind::AutomaticDomainX,
                    ) {
                        domains[0].extent = self.window_extent(0, range)?;
                    }
                }
                if let Some(range) = consumed.feedback_domain(
                    &self.registered.scope.address,
                    FeedbackKind::AutomaticDomainY,
                ) {
                    domains[1].extent = self.window_extent(1, range)?;
                }
            }
        }
        Ok(())
    }

    fn registration(&self, id: &PlotOccurrenceId) -> &PlotRegistration<'src> {
        self.registrations
            .iter()
            .find(|registered| registered.scope.id == *id)
            .expect("prepared sharing member")
    }

    fn resolved_forest_x(&self, side: usize) -> bool {
        side == 0
            && self.registered.source.kind() == PlotKind::Forest
            && self.registered.source.axis_scale_spec() == AxisScale::Log10
    }

    fn attempt_domains(
        &self,
        domains: &mut AxisDomainAssignments,
        consumed: &FeedbackRound,
    ) -> Result<(), Error> {
        let pair = match domains {
            AxisDomainAssignments::Pair(pair) => pair,
            AxisDomainAssignments::Marginal { central, .. } => central,
            AxisDomainAssignments::Matrix { .. } => return Ok(()),
        };
        for (side, kind) in [
            FeedbackKind::AutomaticDomainX,
            FeedbackKind::AutomaticDomainY,
        ]
        .into_iter()
        .enumerate()
        {
            let Some(range) = consumed.feedback_domain(&self.registered.scope.address, kind) else {
                continue;
            };
            let forest = self.resolved_forest_x(side);
            let resolved = forest
                && match self.composition {
                    PreparedComposition::Single { .. } => false,
                    PreparedComposition::Facets(_) => matches!(
                        self.registered.policy.facet_scales,
                        Some(FacetScales::Fixed | FacetScales::FreeY)
                    ),
                    PreparedComposition::Canvas(topology) => topology
                        .range_groups
                        .iter()
                        .any(|group| group.contains(&self.registered.scope.id)),
                };
            let working = match self.composition {
                PreparedComposition::Single { .. } => {
                    let source = self.registered.source;
                    if side == 0 {
                        source.x_axis_spec()
                    } else {
                        source.y_axis_spec()
                    }
                    .window
                    .map(crate::layout::RawWindow::pair)
                }
                PreparedComposition::Facets(topology) if resolved => {
                    let mut shared = None;
                    for id in &topology.shared_axis_members[side] {
                        if let Some(range) =
                            consumed.feedback_domain(&self.registration(id).scope.address, kind)
                        {
                            shared = Some(union(shared, range));
                        }
                    }
                    shared
                }
                // A free Forest caller has already installed this raw feedback
                // window. Its assigned extent is logarithmic, not that window.
                _ if forest => None,
                _ => Some(pair[side].extent.pair()),
            };
            let converted = self.window_extent(side, union(working, range))?;
            // The original helper converts the raw working window first, even
            // when the independently resolved Forest x extent then overrides it.
            if !resolved {
                pair[side].extent = converted;
            }
        }
        Ok(())
    }

    /// The original height prepass installs feedback directly. Its numeric
    /// native readers are Linear; coordinate intrinsic readers still use their
    /// prepared seeds and must not eagerly convert a Forest working window.
    pub(crate) fn content_ranges(
        &self,
        entry: &PlotMeasureAssignment,
        consumed: &FeedbackRound,
    ) -> Option<[(f64, f64); 2]> {
        let pair = match &entry.domains {
            AxisDomainAssignments::Pair(pair) => pair,
            AxisDomainAssignments::Marginal { central, .. } => central,
            AxisDomainAssignments::Matrix { .. } => return None,
        };
        Some(std::array::from_fn(|side| {
            consumed
                .feedback_domain(
                    &self.registered.scope.address,
                    if side == 0 {
                        FeedbackKind::AutomaticDomainX
                    } else {
                        FeedbackKind::AutomaticDomainY
                    },
                )
                .unwrap_or(pair[side].extent.pair())
        }))
    }
}

fn seed_domain(
    source: &crate::orchestrator::registration::PreparedAxisSource,
) -> AxisDomainAssignment {
    AxisDomainAssignment {
        extent: source.base_seed.expect("prepared axis display seed"),
        provenance: source.base_provenance,
    }
}

fn union(prior: Option<(f64, f64)>, range: (f64, f64)) -> (f64, f64) {
    prior.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1)))
}

pub(crate) struct PlotFrameHeightPreference {
    pub(crate) height: f64,
}

pub(crate) fn completion_frame(
    kind: crate::PlotKind,
    selected: &PlotMeasureAssignment,
    result: &crate::spec::PlotRenderResult,
) -> PixelRect {
    let frame = selected.frame;
    if selected.fill_content_height {
        PixelRect::new(
            frame.x0,
            frame.x1,
            frame.y0,
            frame.y0 + selected.content_preference.unwrap_or(frame.height()),
        )
    } else if selected.content_height
        && matches!(
            kind,
            crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm | crate::PlotKind::Matrix
        )
    {
        PixelRect::new(
            frame.x0,
            frame.x1,
            frame.y0,
            result.layout.projection.panel_box().y1 + selected.insets.assigned_gutters()[3],
        )
    } else {
        frame
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotMeasureDimensions {
    pub(crate) canvas: RootSize,
    pub(crate) output_override: Option<RootSize>,
}

impl PlotMeasureDimensions {
    pub(crate) const fn canvas_size(self) -> RootSize {
        self.canvas
    }

    pub(crate) fn figure_size(self) -> RootSize {
        self.output_override.unwrap_or(self.canvas_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one real consumed round witnesses precoordination, full entry, attempt union and the direct content prepass"
    )]
    fn facet_probe_full_entry_and_attempt_keep_the_original_direct_and_union_domains() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, 10.0, 20.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
        )
        .facet(
            crate::FacetSpec::column(
                crate::CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            )
            .scales(FacetScales::FreeX),
        )
        .size(1200, 800);
        let requested = [[(-5.0, 8.0), (-2.0, 10.0)], [(5.0, 30.0), (0.0, 20.0)]];
        let mut witnessed = false;
        crate::layout::plan_root(
            source.description().sizing_request(),
            |stage| {
                crate::orchestrator::root::Root::prepare(
                    crate::orchestrator::root::RootSource::Plot(source.description()),
                    stage,
                )
            },
            |root, assigned, round| {
                let size = assigned.assigned_size()?;
                let owner = &root.plots[0].scope.address;
                if !witnessed
                    && round
                        .feedback_domain(owner, FeedbackKind::AutomaticDomainX)
                        .is_some()
                {
                    let reads = crate::data::SOURCE_COLUMN_READS.with(Cell::get);
                    let measurement = root.assignment_factory().assign(size);
                    for (registered, local_ranges) in root.plots.iter().zip(requested) {
                        let factory =
                            root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                        let panel = PixelRect::new(80.0, 560.0, 80.0, 680.0);
                        let allocation = || {
                            PlotAllocationInputs::for_test(
                                size,
                                panel,
                                round,
                                &registered.scope.address,
                            )
                        };
                        for phase in [MeasurementPhase::Probe, MeasurementPhase::Hue] {
                            let probe = factory.assign(allocation(), round, Some(phase))?;
                            assert_eq!(probe.ranges(), Some(registered.axes.ranges()));
                            let AxisDomainAssignments::Pair(pair) = probe.domains else {
                                unreachable!("Scatter pair")
                            };
                            assert!(pair
                                .iter()
                                .all(|axis| axis.provenance == RangeProvenance::Automatic));
                        }
                        let entry = factory.assign(allocation(), round, None)?;
                        let expected = [local_ranges[0], (-2.0, 20.0)];
                        assert_eq!(entry.ranges(), Some(expected));
                        assert_eq!(factory.content_ranges(&entry, round), Some(local_ranges));
                        let branch = factory.fork(
                            &entry,
                            GuideTrialChoice {
                                edge: LegendPosition::Auto,
                                compact: false,
                            },
                            round,
                        )?;
                        assert_eq!(branch.entry.ranges(), Some(expected));
                        let AxisDomainAssignments::Pair(pair) = &branch.entry.domains else {
                            unreachable!("Scatter pair")
                        };
                        assert_eq!(pair[0].provenance, RangeProvenance::Automatic);
                        assert_eq!(pair[1].provenance, RangeProvenance::Shared);
                        let scope = FitScope::new(&registered.scope.address, 0);
                        let branch_factory = factory.for_branch(&branch.root)?;
                        let context = branch_factory.context(
                            &branch.entry,
                            crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
                            &scope,
                        )?;
                        assert!(std::ptr::eq(
                            std::ptr::from_ref(context.operation),
                            std::ptr::from_ref(&root.operation),
                        ));
                        assert!(std::ptr::eq(context.registered, registered));
                    }
                    assert_eq!(crate::data::SOURCE_COLUMN_READS.with(Cell::get), reads);
                    witnessed = true;
                }
                for (registered, ranges) in root.plots.iter().zip(requested) {
                    for (kind, range) in [
                        FeedbackKind::AutomaticDomainX,
                        FeedbackKind::AutomaticDomainY,
                    ]
                    .into_iter()
                    .zip(ranges)
                    {
                        round.request_domain(
                            &registered.scope.address,
                            &registered.scope.address,
                            kind,
                            range,
                        )?;
                    }
                }
                Ok(crate::layout::MeasuredLayout::new(size, ()))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        assert!(witnessed);
    }

    #[test]
    fn completion_keeps_the_selected_origin_and_original_fill_and_native_top_branches() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .size(800, 600);
        crate::layout::plan_root(
            source.description().sizing_request(),
            |stage| {
                crate::orchestrator::root::Root::prepare(
                    crate::orchestrator::root::RootSource::Plot(source.description()),
                    stage,
                )
            },
            |root, assigned, round| {
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let factory =
                    root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                let frame = PixelRect::new(5.0, 105.0, 20.0, 220.0);
                let result = crate::spec::PlotRenderResult::new(
                    crate::layout::Layout::envelope(PixelRect::new(10.0, 100.0, 30.0, 80.0)),
                    None,
                );
                let mut selected = factory.assign(
                    PlotAllocationInputs {
                        dimensions: PlotMeasureDimensions {
                            canvas: RootSize::new(800, 600),
                            output_override: None,
                        },
                        cell: frame,
                        frame,
                        body: frame,
                        axis_label_width: frame.width(),
                        content_height: true,
                        fill_content_height: true,
                        content_preference: Some(25.5),
                        insets: ParentInsets {
                            axis_gutter: [0, 0, 0, 3328],
                            owned_ink: [0, 0, 0, 1536],
                            shared_alignment: [0; 4],
                        },
                        guide: GuideTrialAssignment {
                            edge: LegendPosition::Auto,
                            compact: false,
                            allocation: None,
                        },
                    },
                    round,
                    None,
                )?;
                assert_eq!(
                    completion_frame(crate::PlotKind::Scatter, &selected, &result),
                    PixelRect::new(5.0, 105.0, 20.0, 45.5)
                );
                selected.content_preference = None;
                assert_eq!(
                    completion_frame(crate::PlotKind::Heatmap, &selected, &result),
                    frame
                );
                selected.fill_content_height = false;
                for kind in [
                    crate::PlotKind::Heatmap,
                    crate::PlotKind::Beeswarm,
                    crate::PlotKind::Matrix,
                ] {
                    assert_eq!(
                        completion_frame(kind, &selected, &result),
                        PixelRect::new(5.0, 105.0, 20.0, 84.75)
                    );
                }
                assert_eq!(
                    completion_frame(crate::PlotKind::Scatter, &selected, &result),
                    frame
                );
                selected.content_height = false;
                assert_eq!(
                    completion_frame(crate::PlotKind::Heatmap, &selected, &result),
                    frame
                );
                Ok(crate::layout::MeasuredLayout::new(
                    measurement.measurement_size,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
    }
}
