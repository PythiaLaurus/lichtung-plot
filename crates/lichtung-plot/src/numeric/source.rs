//! Operation and semantic-owner binding, distinct from the normalized policy.

use super::context::{
    ContextInfo, ContextValue, CoordinateAxis, DomainInfo, Identity, NumericPurpose,
    NumericRangeOrigin, NumericRole, OwnerInfo,
};
use crate::{Error, Plot, Result};

#[derive(Debug, Clone, Copy)]
pub(crate) struct NumericBinding<'v> {
    pub(crate) operation: &'v crate::font::canonical::OperationContext,
    pub(crate) owner: &'v OwnerInfo,
    pub(crate) plot_id: Option<&'v crate::orchestrator::registration::PlotOccurrenceId>,
}

impl NumericBinding<'_> {
    pub(crate) fn check_data(self, data: crate::data::PlotData<'_, '_>) -> Result<()> {
        data.check_numeric_binding(self)
    }

    pub(crate) fn context(
        self,
        role: NumericRole,
        purpose: NumericPurpose,
        values: Vec<ContextValue>,
        domain: Option<DomainInfo>,
    ) -> Result<ContextInfo> {
        let owner = self.owner.try_copy()?;
        Ok(ContextInfo {
            owner,
            role,
            purpose,
            values,
            domain,
            generation: None,
            window: None,
        })
    }

    pub(crate) fn indexed(
        self,
        values: &[super::operand::Operand],
        role: NumericRole,
        purpose: NumericPurpose,
        domain: Option<DomainInfo>,
    ) -> Result<ContextInfo> {
        use super::context::CoordinateAxis;
        let values = crate::data::source::collect_source(values.iter().enumerate().map(
            |(index, value)| {
                let identity = if purpose == NumericPurpose::IntrinsicProbe {
                    Identity::Probe(index)
                } else {
                    match role {
                        NumericRole::XAxis => Identity::Tick(CoordinateAxis::X, index),
                        NumericRole::YAxis => Identity::Tick(CoordinateAxis::Y, index),
                        NumericRole::ContinuousColor => Identity::GuideTick(index),
                        _ => {
                            return Err(Error::Render(
                                "numeric indexed source has no occurrence identity".into(),
                            ))
                        }
                    }
                };
                Ok(ContextValue {
                    operand: value.clone(),
                    identity,
                })
            },
        ))?;
        self.context(role, purpose, values, domain)
    }
}

impl<'v> NumericBinding<'v> {
    pub(crate) fn shared(
        root: &crate::orchestrator::registration::RootIdentity,
        operation: &'v crate::font::canonical::OperationContext,
        owner: &'v OwnerInfo,
    ) -> Result<Self> {
        if *root != operation.root_identity() {
            return Err(Error::Render(
                "row source belongs to a different source or operation".into(),
            ));
        }
        Ok(Self {
            operation,
            owner,
            plot_id: None,
        })
    }
}

impl OwnerInfo {
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            kind: self.kind,
            canvas: self.canvas,
            facet: self
                .facet
                .as_ref()
                .map(crate::FacetCell::try_copy)
                .transpose()?,
            shared: self.shared,
            scope: self
                .scope
                .as_deref()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
        })
    }
    pub(crate) fn for_plot(plot: &Plot, facet: Option<&crate::FacetCell>) -> Result<Self> {
        Ok(Self {
            kind: Some(plot.kind()),
            canvas: None,
            facet: facet.map(crate::FacetCell::try_copy).transpose()?,
            shared: false,
            scope: plot
                .axis_scope()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AxisNumericPhase {
    Base,
    Actual { parallel_span: f64 },
}

#[derive(Debug)]
pub(crate) struct AxisRequest<'v> {
    pub(crate) binding: NumericBinding<'v>,
    pub(crate) format: Option<&'v crate::NumericFormat>,
    pub(crate) generation: &'v crate::TickGeneration,
    pub(crate) axis: CoordinateAxis,
    pub(crate) domain: DomainInfo,
    pub(crate) phase: AxisNumericPhase,
}

impl<'v> AxisRequest<'v> {
    pub(crate) fn base(
        binding: NumericBinding<'v>,
        axis: crate::orchestrator::registration::BaseAxisRef<'v>,
        side: crate::layout::AxisSide,
    ) -> Result<Option<Self>> {
        let Some(authored) = axis.authored() else {
            return Ok(None);
        };
        let range = axis.extent()?.pair();
        let scale = match axis.witness().space {
            crate::orchestrator::registration::AxisSpace::Categorical => crate::AxisScale::Linear,
            crate::orchestrator::registration::AxisSpace::Continuous(scale) => scale,
        };
        Ok(Some(Self {
            binding: NumericBinding {
                plot_id: Some(&axis.witness().plot_id),
                ..binding
            },
            format: authored.format.as_deref(),
            generation: &authored.generation,
            axis: match side {
                crate::layout::AxisSide::Bottom => CoordinateAxis::X,
                crate::layout::AxisSide::Left => CoordinateAxis::Y,
            },
            domain: DomainInfo::axis(
                range,
                scale,
                axis.provenance(),
                if scale == crate::AxisScale::Log10 {
                    axis.raw_window().map(|window| window.pair())
                } else {
                    Some(range)
                },
            ),
            phase: AxisNumericPhase::Base,
        }))
    }

    pub(crate) fn actual(
        binding: NumericBinding<'v>,
        axis: crate::orchestrator::assignment::AssignedAxisRef<'v>,
        projection: &crate::layout::Projection,
        side: crate::layout::AxisSide,
    ) -> Result<Option<Self>> {
        let Some(mut request) = Self::base(binding, axis.base, side)? else {
            return Ok(None);
        };
        let range = axis.domain.extent.pair();
        request.domain = DomainInfo::axis(
            range,
            request.domain.scale,
            axis.domain.provenance,
            if request.domain.scale == crate::AxisScale::Log10 {
                axis.base.raw_window().map(|window| window.pair())
            } else {
                Some(range)
            },
        );
        request.phase = AxisNumericPhase::Actual {
            parallel_span: match side {
                crate::layout::AxisSide::Bottom => projection.panel_box().width(),
                crate::layout::AxisSide::Left => projection.panel_box().height(),
            },
        };
        Ok(Some(request))
    }

    pub(crate) fn empty_fixed_source(&self) -> Result<bool> {
        empty_fixed_source(
            self.generation,
            Some(crate::orchestrator::registration::DisplayExtent {
                lower: f64::from_bits(self.domain.limits.0),
                upper: f64::from_bits(self.domain.limits.1),
            }),
            self.domain.scale == crate::AxisScale::Log10,
            self.domain
                .data
                .map(|(a, b)| (f64::from_bits(a), f64::from_bits(b))),
        )
    }

    pub(crate) fn probe_context(
        &self,
        operands: &[super::operand::Operand],
    ) -> Result<ContextInfo> {
        self.context(operands, NumericPurpose::IntrinsicProbe, None)
    }

    pub(crate) fn actual_context(
        &self,
        operands: &[super::operand::Operand],
        source_indices: Option<&[usize]>,
    ) -> Result<ContextInfo> {
        self.context(operands, NumericPurpose::ActualSet, source_indices)
    }

    fn context(
        &self,
        operands: &[super::operand::Operand],
        purpose: NumericPurpose,
        source_indices: Option<&[usize]>,
    ) -> Result<ContextInfo> {
        let role = match self.axis {
            CoordinateAxis::X => NumericRole::XAxis,
            CoordinateAxis::Y => NumericRole::YAxis,
        };
        let mut info = self
            .binding
            .indexed(operands, role, purpose, Some(self.domain.clone()))?;
        if let Some(indices) = source_indices {
            if indices.len() != info.values.len() {
                return Err(Error::Render(
                    "numeric source occurrence count mismatch".into(),
                ));
            }
            for (value, &index) in info.values.iter_mut().zip(indices) {
                let Identity::Tick(axis, _) = value.identity else {
                    return Err(Error::Render("indexed axis identity missing".into()));
                };
                value.identity = Identity::Tick(axis, index);
            }
        }
        info.generation = Some(self.generation.clone());
        Ok(info)
    }

    pub(crate) fn position_view(&self) -> Result<crate::spec::ticks::function::View> {
        let AxisNumericPhase::Actual { parallel_span } = self.phase else {
            unreachable!("position functions consume an actual axis view")
        };
        crate::layout::kernel::required_length(parallel_span)?;
        Ok(crate::spec::ticks::function::View {
            owner: self.binding.owner.try_copy()?,
            domain: self.domain.clone(),
            axis: self.axis,
            span: parallel_span.to_bits(),
        })
    }
}

/// The original source-emptiness program is shared by intrinsic admission and requests.
pub(crate) fn empty_fixed_source(
    generation: &crate::TickGeneration,
    extent: Option<crate::orchestrator::registration::DisplayExtent>,
    log: bool,
    raw: Option<(f64, f64)>,
) -> Result<bool> {
    if generation
        .exact_parameters()
        .is_some_and(|(count, _, _)| count == 0)
    {
        return Ok(true);
    }
    let range = extent.expect("prepared axis display seed").pair();
    let maximum = generation.get_maximum_count().unwrap_or(usize::MAX);
    if let Some((anchor, ratio, requirement)) = generation.multiplicative_parameters() {
        let count =
            crate::numeric::lattice::generation::LatticeRange::new(range, log, anchor, ratio)?
                .count();
        if requirement == crate::TickRequirement::Fixed && count > num_bigint::BigInt::from(maximum)
        {
            return Err(Error::Render("fixed multiplicative count exceeds its configured maximum or platform representation".into()));
        }
        return Ok(count == num_bigint::BigInt::ZERO);
    }
    if let Some(parameters) = generation.interval_parameters() {
        return crate::layout::fixed_interval_is_empty(range, raw, log, parameters, maximum);
    }
    Ok(false)
}

/// Primitive numeric-source fixtures keep the request inside its real operation.
#[cfg(test)]
pub(crate) fn with_axis_binding_for_test<T>(visit: impl FnOnce(NumericBinding<'_>) -> T) -> T {
    let operation = crate::font::canonical::OperationContext::new();
    let owner = OwnerInfo {
        kind: None,
        canvas: None,
        facet: None,
        shared: false,
        scope: None,
    };
    visit(NumericBinding {
        operation: &operation,
        owner: &owner,
        plot_id: None,
    })
}

#[cfg(test)]
pub(crate) fn axis_request_for_test<'v>(
    binding: NumericBinding<'v>,
    format: Option<&'v crate::NumericFormat>,
    generation: &'v crate::TickGeneration,
    projection: &crate::layout::Projection,
    side: crate::layout::AxisSide,
    log: bool,
) -> AxisRequest<'v> {
    let (range, axis, parallel_span) = match side {
        crate::layout::AxisSide::Bottom => (
            projection.x_range(),
            CoordinateAxis::X,
            projection.panel_box().width(),
        ),
        crate::layout::AxisSide::Left => (
            projection.y_range(),
            CoordinateAxis::Y,
            projection.panel_box().height(),
        ),
    };
    AxisRequest {
        binding,
        format,
        generation,
        axis,
        domain: DomainInfo::axis(
            range,
            if log {
                crate::AxisScale::Log10
            } else {
                crate::AxisScale::Linear
            },
            crate::layout::RangeProvenance::Automatic,
            (!log).then_some(range),
        ),
        phase: AxisNumericPhase::Actual { parallel_span },
    }
}

pub(crate) fn collect_values(
    count: usize,
    values: impl Iterator<Item = Result<ContextValue>>,
) -> Result<Vec<ContextValue>> {
    let mut result = Vec::new();
    crate::data::source::reserve_source(&mut result, count)?;
    for value in values {
        if result.len() == count {
            return Err(Error::Render(
                "numeric occurrence count changed during materialization".into(),
            ));
        }
        result.push(value?);
    }
    if result.len() != count {
        return Err(Error::Render(
            "numeric occurrence count changed during materialization".into(),
        ));
    }
    Ok(result)
}

impl DomainInfo {
    pub(crate) fn axis(
        range: (f64, f64),
        scale: crate::AxisScale,
        provenance: crate::layout::RangeProvenance,
        raw: Option<(f64, f64)>,
    ) -> Self {
        let origin = match provenance {
            crate::layout::RangeProvenance::Automatic => NumericRangeOrigin::Automatic,
            crate::layout::RangeProvenance::Configured => NumericRangeOrigin::Explicit,
            crate::layout::RangeProvenance::Shared => NumericRangeOrigin::ParentResolved,
        };
        Self {
            limits: (range.0.to_bits(), range.1.to_bits()),
            data: raw.map(|(a, b)| (a.to_bits(), b.to_bits())),
            scale,
            origin,
        }
    }
}
