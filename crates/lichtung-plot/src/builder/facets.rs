use super::{ComposedInputs, Plot, PlotDescription};
use crate::data::category::CategoryOrderRef;
use crate::data::{FACET_COLUMN, FACET_ROW, FACET_WRAP};
use crate::decorations::legend::discrete::scoped::ScopedGuideInputs;
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::kernel::{FeedbackKind, FeedbackRound};
use crate::layout::{AddressRole, RangeProvenance};
use crate::orchestrator::assignment::AxisDomainAssignments;
use crate::spec::{AxisScale, FacetScales, FacetSpec, PlotKind, PlotMeasureContext};

#[cfg(test)]
mod generation_tests;

impl<K: crate::kind::Marker> Plot<K> {
    /// Apply one complete generic-facet layout.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: crate::kind::Facet,
    {
        self.description.payload_mut().facet = Some(facet);
        self
    }
}

impl PlotDescription {
    pub(crate) fn facet_spec(&self) -> Option<&FacetSpec> {
        self.payload.facet.as_ref()
    }

    pub(crate) fn facet_scales_spec(&self) -> FacetScales {
        self.payload
            .facet
            .as_ref()
            .map_or(FacetScales::Fixed, FacetSpec::scale_policy)
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
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::row_values)
                .map(|_| FACET_ROW),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::column_values)
                .map(|_| FACET_COLUMN),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::wrap_values)
                .map(|_| FACET_WRAP),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::wrap_columns),
        )
    }

    pub(crate) fn has_facets(&self) -> bool {
        self.payload.facet.is_some()
    }
}

fn same_values<T: PartialEq>(a: Option<&[T]>, b: Option<&[T]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) if a.len() == b.len() => a.iter().zip(b).all(|(a, b)| a == b),
        _ => false,
    }
}

fn same_order(order: Option<CategoryOrderRef<'_>>, values: Option<&[String]>) -> bool {
    match (order, values) {
        (None, None) => true,
        (Some(order), Some(values)) if order.len() == values.len() => {
            order.values().zip(values).all(|(a, b)| a == b)
        }
        _ => false,
    }
}

/// Recover the old working-window snapshot, not merely its selected display
/// extent. Forest's resolved-x override deliberately remains a separate value.
#[expect(
    clippy::type_complexity,
    reason = "three original narrow snapshot operands, without another owner"
)]
fn working_axes(
    context: &PlotMeasureContext<'_, '_>,
    round: &FeedbackRound,
    measurement_phase: Option<MeasurementPhase>,
) -> (
    [Option<(f64, f64)>; 2],
    [RangeProvenance; 2],
    Option<(f64, f64)>,
) {
    let registered = context.registered;
    let source = registered.source;
    let authored = [source.x_axis_spec(), source.y_axis_spec()];
    let mut windows = authored.map(|axis| axis.window.map(crate::layout::RawWindow::pair));
    let mut provenance = authored.map(|axis| {
        if axis.window.is_some() {
            RangeProvenance::Configured
        } else {
            RangeProvenance::Automatic
        }
    });
    let mut resolved_x = None;
    let forest = source.kind() == PlotKind::Forest && source.axis_scale_spec() == AxisScale::Log10;
    if let Some(scales) = registered.policy.facet_scales {
        if context.fit_scope.address.leaf_role() != AddressRole::FacetCell {
            return (windows, provenance, resolved_x);
        }
        for (side, kind) in [
            FeedbackKind::AutomaticDomainX,
            FeedbackKind::AutomaticDomainY,
        ]
        .into_iter()
        .enumerate()
        {
            let fixed = matches!(scales, FacetScales::Fixed)
                || side == 0 && scales == FacetScales::FreeY
                || side == 1 && scales == FacetScales::FreeX;
            if fixed && registered.axes.categories(side).is_none() {
                let shared = registered.axes.sources()[side]
                    .base_seed
                    .expect("prepared shared facet seed")
                    .pair();
                if authored[side].window.is_none() {
                    provenance[side] = RangeProvenance::Shared;
                }
                if side == 0 && forest {
                    resolved_x = Some(shared);
                } else {
                    windows[side] = Some(shared);
                }
            }
            let feedback = if fixed {
                round.facet_feedback_domain(kind)
            } else {
                round.feedback_domain(&registered.scope.address, kind)
            };
            if let Some(range) = feedback {
                windows[side] = Some(range);
            }
        }
    } else if let Some(canvas) = context.canvas {
        let initial = measurement_phase == Some(MeasurementPhase::Legend);
        let grouped = canvas.range_group_contains(
            usize::try_from(registered.scope.address.leaf_slot_index())
                .expect("Canvas slot fits usize"),
        );
        let pair = match &context.assignment.local.domains {
            AxisDomainAssignments::Pair(pair) => pair,
            AxisDomainAssignments::Marginal { central, .. } => central,
            AxisDomainAssignments::Matrix { .. } => return (windows, provenance, resolved_x),
        };
        if grouped {
            let shared = if initial {
                registered.axes.sources()[0]
                    .base_seed
                    .expect("prepared shared Canvas seed")
                    .pair()
            } else {
                pair[0].extent.pair()
            };
            if authored[0].window.is_none() {
                provenance[0] = RangeProvenance::Shared;
            }
            if forest {
                resolved_x = Some(shared);
            } else {
                windows[0] = Some(shared);
            }
        }
        if initial {
            return (windows, provenance, resolved_x);
        }
        if !grouped {
            if let Some(range) =
                round.feedback_domain(&registered.scope.address, FeedbackKind::AutomaticDomainX)
            {
                windows[0] = Some(range);
            }
        }
        if let Some(range) =
            round.feedback_domain(&registered.scope.address, FeedbackKind::AutomaticDomainY)
        {
            windows[1] = Some(range);
        }
    }
    (windows, provenance, resolved_x)
}

impl ComposedInputs {
    /// Compare only the original computed inputs of a root-local child owner.
    pub(crate) fn matches(
        &self,
        context: &PlotMeasureContext<'_, '_>,
        round: &FeedbackRound,
        guides: &ScopedGuideInputs,
        measurement_phase: Option<MeasurementPhase>,
    ) -> bool {
        let registration = context.registered;
        let registered = Some(registration);
        let source = registration.source;
        let (windows, provenance, resolved_x) = working_axes(context, round, measurement_phase);
        for (side, other) in [&self.x_axis_spec, &self.y_axis_spec]
            .into_iter()
            .enumerate()
        {
            let axis = &source.payload.axes[side];
            if windows[side] != other.0
                || provenance[side] != other.1
                || axis.generation != other.2
                || !same_values(axis.exact_ticks.as_deref(), other.5.as_deref())
            {
                return false;
            }
        }
        let range = |value: Option<(f64, f64)>| value.map(|(a, b)| (a.to_bits(), b.to_bits()));
        source.same_prose(&self.prose)
            && source.same_axis_titles(&self.axis_titles)
            && range(windows[0]) == range(self.x_axis_spec.0)
            && range(windows[1]) == range(self.y_axis_spec.0)
            && range(resolved_x) == range(self.resolved_facet_x_range)
            && same_order(
                crate::orchestrator::registration::parent_hues(registered)
                    .map(CategoryOrderRef::Keys),
                self.global_hue_levels.as_deref(),
            )
            && same_order(
                crate::orchestrator::registration::composed_group_levels(registered),
                self.global_group_levels.as_deref(),
            )
            && same_values(
                crate::orchestrator::registration::collective_pairs(registered),
                self.global_collective_pairs.as_deref(),
            )
            && same_values(
                crate::orchestrator::registration::line_condition_levels(registered),
                self.global_line_condition_levels.as_deref(),
            )
            && registration.policy.gutter_owner == self.axis_gutter_owner
            && registration.policy.axis_text.tick_labels[1] == self.y_tick_labels_visible
            && registration.scope.render_name.as_deref() == self.render_scope.as_deref()
            && crate::orchestrator::registration::legend_policy(source, registered)
                == self.legend_policy
            && crate::orchestrator::registration::continuous_guide_suppressed(registered)
                == self.suppress_continuous_guide
            && context
                .assignment
                .local
                .dimensions
                .canvas
                .dimensions()
                .is_ok_and(|(width, height)| {
                    self.root_size == crate::layout::RootRequest::ExactSize { width, height }
                })
            && self.numeric.matches(registration, context.canvas)
            && source.payload.alpha == self.alpha
            && source.payload.text_layouts == self.text_layouts
            && context
                .assignment
                .local
                .dimensions
                .output_override
                .map(|size| size.dimensions().expect("assigned output dimensions"))
                == self.output_dimensions
            && guides.matches_source(registration)
            && *guides == self.guides
    }

    #[expect(
        clippy::too_many_lines,
        reason = "original narrow copies retain their fallible order"
    )]
    pub(crate) fn retain(
        context: &PlotMeasureContext<'_, '_>,
        round: &FeedbackRound,
        guides: ScopedGuideInputs,
        measurement_phase: Option<MeasurementPhase>,
    ) -> Result<Self, crate::Error> {
        let registration = context.registered;
        let registered = Some(registration);
        let source = registration.source;
        if !guides.matches_source(registration) {
            return Err(crate::Error::Render(
                "retained guide inputs do not belong to their computed source".into(),
            ));
        }
        let copy = |source: &str| -> Result<String, crate::Error> {
            let mut value = String::new();
            value
                .try_reserve_exact(source.len())
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            value.push_str(source);
            Ok(value)
        };
        let (windows, provenance, resolved_x) = working_axes(context, round, measurement_phase);
        let mut axes = std::array::from_fn::<_, 2, _>(|_| {
            (
                None,
                RangeProvenance::Automatic,
                crate::TickGeneration::automatic(),
                crate::CategoryLabelSelection::all(),
                None,
                None,
            )
        });
        for (side, (target, axis)) in axes.iter_mut().zip(&source.payload.axes).enumerate() {
            target.0 = windows[side];
            target.1 = provenance[side];
            target.2.clone_from(&axis.generation);
            target.3.clone_from(&axis.category_labels);
            target.4.clone_from(&axis.format);
            if let Some(ticks) = &axis.exact_ticks {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, ticks.len())?;
                for tick in ticks {
                    retained.push(crate::AxisTick {
                        position: tick.position,
                        label: copy(&tick.label)?,
                    });
                }
                target.5 = Some(retained);
            }
        }
        let mut levels = [None, None, None];
        for (target, source) in levels.iter_mut().zip([
            crate::orchestrator::registration::parent_hues(registered).map(CategoryOrderRef::Keys),
            crate::orchestrator::registration::composed_group_levels(registered),
            crate::orchestrator::registration::line_condition_levels(registered)
                .map(CategoryOrderRef::Raw),
        ]) {
            if let Some(source) = source {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, source.len())?;
                for value in source.values() {
                    retained.push(copy(value)?);
                }
                *target = Some(retained);
            }
        }
        let pairs =
            if let Some(source) = crate::orchestrator::registration::collective_pairs(registered) {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, source.len())?;
                for pair in source {
                    retained.push(crate::data::CollectivePair {
                        group: pair.group.as_deref().map(&copy).transpose()?,
                        hue: pair.hue.as_deref().map(&copy).transpose()?,
                    });
                }
                Some(retained)
            } else {
                None
            };
        let [x_axis_spec, y_axis_spec] = axes;
        let [global_hue_levels, global_group_levels, global_line_condition_levels] = levels;
        let dimensions = context.assignment.local.dimensions;
        let (width, height) = dimensions
            .canvas
            .dimensions()
            .expect("assigned canvas dimensions");
        Ok(Self {
            text_layouts: source.payload.text_layouts.clone(),
            output_dimensions: dimensions
                .output_override
                .map(|size| size.dimensions().expect("assigned output dimensions")),
            prose: source.copy_prose()?,
            axis_titles: source.copy_axis_titles()?,
            x_axis_spec,
            y_axis_spec,
            global_hue_levels,
            global_group_levels,
            global_line_condition_levels,
            global_collective_pairs: pairs,
            resolved_facet_x_range: resolved_x,
            axis_gutter_owner: registration.policy.gutter_owner,
            y_tick_labels_visible: registration.policy.axis_text.tick_labels[1],
            render_scope: registration
                .scope
                .render_name
                .as_deref()
                .map(copy)
                .transpose()?,
            legend_policy: crate::orchestrator::registration::legend_policy(source, registered),
            suppress_continuous_guide:
                crate::orchestrator::registration::continuous_guide_suppressed(registered),
            root_size: crate::layout::RootRequest::ExactSize { width, height },
            numeric: crate::numeric::inputs::NumericInputs::new(registration, context.canvas)?,
            alpha: source.payload.alpha.clone(),
            guides,
        })
    }
}
