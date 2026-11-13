//! Prepared axis sources own scientific coordinates, not authored policies.

use crate::layout::{AxisSide, AxisSpec, AxisTick, RangePolicy, RangeProvenance};
use crate::{AxisScale, Error, Text};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DisplayExtent {
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}

impl DisplayExtent {
    pub(crate) const fn pair(self) -> (f64, f64) {
        (self.lower, self.upper)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AxisSlot {
    Pair(AxisSide),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AxisSpace {
    Categorical,
    Continuous(AxisScale),
}

#[derive(Debug)]
pub(crate) struct AxisSourceWitness {
    pub(crate) plot_id: super::PlotOccurrenceId,
    pub(crate) slot: AxisSlot,
    pub(crate) space: AxisSpace,
}

#[derive(Debug)]
pub(crate) struct PreparedAxisSource {
    pub(crate) witness: AxisSourceWitness,
    pub(crate) range_policy: RangePolicy,
    pub(crate) base_seed: Option<DisplayExtent>,
    pub(crate) base_provenance: RangeProvenance,
    pub(crate) categories: Option<PreparedCategorySource>,
    pub(crate) projected_exact_positions: Option<Vec<f64>>,
    pub(crate) label: std::sync::OnceLock<Text>,
}

#[derive(Debug)]
pub(crate) struct PreparedCategorySource {
    pub(crate) raw: Vec<String>,
    pub(crate) ticks: Vec<AxisTick>,
}

#[derive(Clone, Copy)]
pub(crate) enum AxisIntentRef<'a> {
    Authored(&'a AxisSpec),
}

#[derive(Clone, Copy)]
pub(crate) struct BaseAxisRef<'a> {
    pub(crate) source: &'a PreparedAxisSource,
    pub(crate) intent: AxisIntentRef<'a>,
}

pub(crate) struct ExactTicksRef<'a> {
    pub(crate) raw: &'a [AxisTick],
    pub(crate) projected: Option<&'a [f64]>,
}

impl ExactTicksRef<'_> {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the prepared exact source keeps the common fallible tick-reader boundary"
    )]
    pub(crate) fn tick(&self, index: usize) -> Result<(f64, &str), Error> {
        let tick = &self.raw[index];
        Ok((
            self.projected
                .map_or(tick.position, |positions| positions[index]),
            &tick.label,
        ))
    }
}

impl<'a> BaseAxisRef<'a> {
    pub(crate) const fn authored(self) -> &'a AxisSpec {
        let AxisIntentRef::Authored(spec) = self.intent;
        spec
    }

    pub(crate) fn extent(self) -> DisplayExtent {
        self.source.base_seed.expect("prepared axis display seed")
    }

    pub(crate) const fn provenance(self) -> RangeProvenance {
        self.source.base_provenance
    }

    pub(crate) fn exact_ticks(self) -> Option<ExactTicksRef<'a>> {
        Some(ExactTicksRef {
            raw: self.authored().exact_ticks.as_deref()?,
            projected: self.source.projected_exact_positions.as_deref(),
        })
    }

    pub(crate) fn semantic_inputs(self) -> crate::layout::AxisSemanticInputs {
        let mut input = crate::layout::AxisSemanticInputs::new(
            self.authored(),
            match self.source.witness.space {
                AxisSpace::Categorical => AxisScale::Linear,
                AxisSpace::Continuous(scale) => scale,
            },
            self.source.range_policy,
        );
        input.provenance = self.authored().provenance;
        input
    }

    pub(crate) fn numeric_request(
        self,
        binding: crate::numeric::source::NumericBinding<'a>,
        windows: [Option<&'a (f64, f64)>; 2],
    ) -> Result<Option<crate::numeric::source::AxisRequest<'a>>, Error> {
        let mut request = self.authored().numeric_request(
            Some(crate::numeric::source::NumericBinding {
                plot_id: Some(&self.source.witness.plot_id),
                ..binding
            }),
            windows,
        )?;
        if let Some(request) = &mut request {
            request.provenance = self.provenance();
        }
        Ok(request)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one borrowed source selects exact or numeric ticks at its original operation and fit scope"
    )]
    pub(crate) fn layout_ticks(
        self,
        supplied: Vec<AxisTick>,
        projection: &crate::layout::Projection,
        side: AxisSide,
        theme: &crate::text::TextRequestContext<'_, '_>,
        scope: &crate::layout::fit::FitScope,
        binding: crate::numeric::source::NumericBinding<'a>,
        windows: [Option<&'a (f64, f64)>; 2],
    ) -> Result<crate::layout::NumericTickPlan, crate::layout::fit::PlanError> {
        crate::layout::layout_axis_ticks(
            self.exact_ticks(),
            &self.authored().generation,
            supplied,
            projection,
            side,
            theme,
            scope,
            || self.numeric_request(binding, windows),
        )
    }
}
