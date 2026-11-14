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
    MatrixVariable(usize),
    MatrixDiagonalDensity(usize),
    MarginalCentral(AxisSide),
    MarginalStripDensity(MarginalStrip),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MarginalStrip {
    Top,
    Right,
}

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the closed native axes owner retains its fixed source arrays by value"
)]
pub(crate) enum PreparedAxesOwner {
    Pair(PreparedAxes),
    Matrix {
        variables: Vec<PreparedAxisSource>,
        diagonal_density: Vec<PreparedAxisSource>,
    },
    Marginal {
        central: [PreparedAxisSource; 2],
        strip_density: [PreparedAxisSource; 2],
    },
}

#[derive(Debug)]
pub(crate) struct PreparedAxes {
    pub(crate) sources: [PreparedAxisSource; 2],
}

impl PreparedAxesOwner {
    pub(crate) fn matrix(&self) -> Option<(&[PreparedAxisSource], &[PreparedAxisSource])> {
        match self {
            Self::Matrix {
                variables,
                diagonal_density,
            } => Some((variables, diagonal_density)),
            _ => None,
        }
    }

    pub(crate) fn coordinate(&self) -> Option<&Self> {
        match self {
            Self::Pair(_) | Self::Marginal { .. } => Some(self),
            Self::Matrix { .. } => None,
        }
    }

    pub(crate) fn sources(&self) -> &[PreparedAxisSource; 2] {
        match self {
            Self::Pair(axes) => &axes.sources,
            Self::Marginal { central, .. } => central,
            Self::Matrix { .. } => unreachable!("Matrix has no single panel axis owner"),
        }
    }

    pub(crate) fn sources_mut(&mut self) -> &mut [PreparedAxisSource; 2] {
        match self {
            Self::Pair(axes) => &mut axes.sources,
            Self::Marginal { central, .. } => central,
            Self::Matrix { .. } => unreachable!("Matrix has no single panel axis owner"),
        }
    }

    pub(crate) fn axis<'a>(
        &'a self,
        source: &'a crate::builder::PlotDescription,
        side: usize,
    ) -> BaseAxisRef<'a> {
        self.axis_at(source, self.sources()[side].witness.slot)
    }

    pub(crate) fn axis_at<'a>(
        &'a self,
        source: &'a crate::builder::PlotDescription,
        slot: AxisSlot,
    ) -> BaseAxisRef<'a> {
        let (axis, intent) = match (self, slot) {
            (Self::Pair(axes), AxisSlot::Pair(side)) => (
                &axes.sources[usize::from(side == AxisSide::Left)],
                AxisIntentRef::Authored(match side {
                    AxisSide::Bottom => source.x_axis_spec(),
                    AxisSide::Left => source.y_axis_spec(),
                }),
            ),
            (Self::Marginal { central, .. }, AxisSlot::MarginalCentral(side)) => (
                &central[usize::from(side == AxisSide::Left)],
                AxisIntentRef::Authored(match side {
                    AxisSide::Bottom => source.x_axis_spec(),
                    AxisSide::Left => source.y_axis_spec(),
                }),
            ),
            (Self::Matrix { variables, .. }, AxisSlot::MatrixVariable(index)) => {
                (&variables[index], AxisIntentRef::MatrixVariable)
            }
            (
                Self::Matrix {
                    diagonal_density, ..
                },
                AxisSlot::MatrixDiagonalDensity(index),
            ) => (&diagonal_density[index], AxisIntentRef::NativeAmplitude),
            (Self::Marginal { strip_density, .. }, AxisSlot::MarginalStripDensity(strip)) => (
                &strip_density[match strip {
                    MarginalStrip::Top => 0,
                    MarginalStrip::Right => 1,
                }],
                AxisIntentRef::NativeAmplitude,
            ),
            _ => unreachable!("axis slot belongs to its prepared native owner"),
        };
        BaseAxisRef {
            source: axis,
            intent,
        }
    }

    pub(crate) fn ranges(&self) -> [(f64, f64); 2] {
        self.sources()
            .each_ref()
            .map(|source| source.base_seed.expect("prepared axis display seed").pair())
    }

    pub(crate) fn logarithmic(&self) -> bool {
        matches!(
            self.sources()[0].witness.space,
            AxisSpace::Continuous(AxisScale::Log10)
        )
    }

    pub(crate) fn categories(&self, side: usize) -> Option<&PreparedCategorySource> {
        self.sources()[side].categories.as_ref()
    }

    pub(crate) fn label(
        &self,
        registered: &super::PlotRegistration<'_>,
        side: usize,
    ) -> Option<&Text> {
        axis_label(registered, &self.sources()[side], side)
    }

    pub(crate) fn copy_label(
        &self,
        registered: &super::PlotRegistration<'_>,
        side: usize,
    ) -> Result<Option<Text>, Error> {
        self.label(registered, side).map(Text::try_copy).transpose()
    }

    pub(crate) fn retain_axis_labels(&self, labels: [Option<&Text>; 2]) -> Result<(), Error> {
        for (source, label) in self.sources().iter().zip(labels) {
            if source.label.get().is_none() {
                let label = label.map_or(Ok(Text::UNSET), Text::try_copy)?;
                let _ = source.label.set(label);
            }
        }
        Ok(())
    }
}

pub(crate) fn axis_label<'v>(
    registered: &super::PlotRegistration<'_>,
    source: &'v PreparedAxisSource,
    side: usize,
) -> Option<&'v Text> {
    if !registered.policy.axis_text.titles[side] {
        return None;
    }
    source.label.get().and_then(Text::provided)
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

#[expect(
    clippy::unnecessary_wraps,
    reason = "source construction preserves the fallible producer boundary without adding early validation"
)]
pub(crate) fn prepare_axis_source(
    build: &super::PlotRegistrationBuild<'_>,
    slot: AxisSlot,
    seed: Option<DisplayExtent>,
    range_policy: RangePolicy,
    categories: Option<PreparedCategorySource>,
    label: Option<Text>,
) -> Result<PreparedAxisSource, Error> {
    let provenance = match slot {
        AxisSlot::Pair(side) | AxisSlot::MarginalCentral(side) => match side {
            AxisSide::Bottom => build.source.x_axis_spec().provenance,
            AxisSide::Left => build.source.y_axis_spec().provenance,
        },
        _ => RangeProvenance::Automatic,
    };
    let source = PreparedAxisSource {
        witness: AxisSourceWitness {
            plot_id: build.scope.id.clone(),
            slot,
            space: if categories.is_some() {
                AxisSpace::Categorical
            } else {
                AxisSpace::Continuous(
                    if matches!(slot, AxisSlot::Pair(AxisSide::Bottom))
                        && build.source.kind() == crate::PlotKind::Forest
                    {
                        build.source.axis_scale_spec()
                    } else {
                        AxisScale::Linear
                    },
                )
            },
        },
        range_policy,
        base_seed: seed,
        base_provenance: provenance,
        categories,
        projected_exact_positions: None,
        label: std::sync::OnceLock::new(),
    };
    if let Some(label) = label {
        let _ = source.label.set(label);
    }
    Ok(source)
}

#[derive(Clone, Copy)]
pub(crate) enum AxisIntentRef<'a> {
    Authored(&'a AxisSpec),
    MatrixVariable,
    NativeAmplitude,
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
    pub(crate) const fn witness(self) -> &'a AxisSourceWitness {
        &self.source.witness
    }

    pub(crate) const fn authored(self) -> Option<&'a AxisSpec> {
        match self.intent {
            AxisIntentRef::Authored(spec) => Some(spec),
            AxisIntentRef::MatrixVariable | AxisIntentRef::NativeAmplitude => None,
        }
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "the base reader preserves the original prepared-seed boundary without adding an earlier validator"
    )]
    pub(crate) fn extent(self) -> Result<DisplayExtent, Error> {
        Ok(self.source.base_seed.expect("prepared axis display seed"))
    }

    pub(crate) const fn provenance(self) -> RangeProvenance {
        self.source.base_provenance
    }

    pub(crate) fn raw_window(self) -> Option<&'a crate::layout::RawWindow> {
        self.authored().and_then(|spec| {
            spec.window
                .as_ref()
                .filter(|_| spec.provenance == RangeProvenance::Configured)
        })
    }

    pub(crate) fn can_expand(self) -> bool {
        self.raw_window().is_none()
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "exact sources retain the shared fallible reader boundary without a new validation pass"
    )]
    pub(crate) fn exact_ticks(self) -> Result<Option<ExactTicksRef<'a>>, Error> {
        let Some(raw) = self.authored().and_then(|spec| spec.exact_ticks.as_deref()) else {
            return Ok(None);
        };
        Ok(Some(ExactTicksRef {
            raw,
            projected: self.source.projected_exact_positions.as_deref(),
        }))
    }
}

impl<'a> crate::orchestrator::assignment::AssignedAxisRef<'a> {
    pub(crate) fn semantic_inputs(self) -> crate::layout::AxisSemanticInputs {
        crate::layout::AxisSemanticInputs::new(
            self.base.intent,
            self.domain.provenance,
            match self.base.source.witness.space {
                AxisSpace::Categorical => AxisScale::Linear,
                AxisSpace::Continuous(scale) => scale,
            },
            self.base.source.range_policy,
        )
    }

    pub(crate) fn layout_ticks(
        self,
        supplied: Vec<AxisTick>,
        projection: &crate::layout::Projection,
        side: AxisSide,
        theme: &crate::text::TextRequestContext<'_, '_>,
        scope: &crate::layout::fit::FitScope,
        binding: crate::numeric::source::NumericBinding<'a>,
    ) -> Result<crate::layout::NumericTickPlan, crate::layout::fit::PlanError> {
        crate::layout::layout_axis_ticks(
            self.base.exact_ticks()?,
            &self
                .base
                .authored()
                .expect("coordinate authored intent")
                .generation,
            supplied,
            projection,
            side,
            theme,
            scope,
            || crate::numeric::source::AxisRequest::actual(binding, self, projection, side),
        )
    }
}
