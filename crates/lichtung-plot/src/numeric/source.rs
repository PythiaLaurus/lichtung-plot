//! Operation and semantic-owner binding, distinct from the normalized policy.

use super::context::{
    ContextInfo, ContextValue, DomainInfo, Identity, NumericPurpose, NumericRangeOrigin,
    NumericRole, OwnerInfo,
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

#[derive(Debug, Clone)]
pub(crate) struct AxisRequest<'a> {
    pub(crate) format: Option<&'a crate::NumericFormat>,
    pub(crate) binding: NumericBinding<'a>,
    pub(crate) raw_windows: [Option<&'a (f64, f64)>; 2],
    pub(crate) provenance: crate::layout::RangeProvenance,
    pub(crate) generation: crate::TickGeneration,
    pub(crate) source_indices: Option<&'a [usize]>,
}

impl AxisRequest<'_> {
    pub(crate) fn raw_limits(&self, side: crate::layout::AxisSide) -> Option<(f64, f64)> {
        self.raw_windows[usize::from(side == crate::layout::AxisSide::Left)].copied()
    }
    pub(crate) fn materialize(
        &self,
        positions: &[super::operand::Operand],
        log: bool,
        side: crate::layout::AxisSide,
        range: (f64, f64),
        purpose: NumericPurpose,
    ) -> Result<Option<super::set::FormattedSet>> {
        let Some(format) = self.format else {
            return Ok(None);
        };
        format.materialize(
            self.binding.operation,
            self.context(positions, log, side, range, purpose)?,
            purpose == NumericPurpose::ActualSet,
        )
    }

    pub(crate) fn context(
        &self,
        positions: &[super::operand::Operand],
        log: bool,
        side: crate::layout::AxisSide,
        range: (f64, f64),
        purpose: NumericPurpose,
    ) -> Result<ContextInfo> {
        let index = usize::from(side == crate::layout::AxisSide::Left);
        let role = if index == 0 {
            NumericRole::XAxis
        } else {
            NumericRole::YAxis
        };
        let scale = if log {
            crate::AxisScale::Log10
        } else {
            crate::AxisScale::Linear
        };
        let raw = if log {
            self.raw_windows[index].copied()
        } else {
            Some(range)
        };
        let domain = DomainInfo::axis(range, scale, self.provenance, raw);
        let mut info = self
            .binding
            .indexed(positions, role, purpose, Some(domain))?;
        if let Some(indices) = self.source_indices {
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
}

/// Primitive numeric-source fixtures keep the request inside its real operation.
#[cfg(test)]
pub(crate) fn with_axis_request_for_test<T>(
    format: &crate::NumericFormat,
    visit: impl FnOnce(AxisRequest<'_>) -> T,
) -> T {
    let operation = crate::font::canonical::OperationContext::new();
    let owner = OwnerInfo {
        kind: None,
        canvas: None,
        facet: None,
        shared: false,
        scope: None,
    };
    visit(AxisRequest {
        format: Some(format),
        binding: NumericBinding {
            operation: &operation,
            owner: &owner,
            plot_id: None,
        },
        raw_windows: [None; 2],
        provenance: crate::layout::RangeProvenance::Automatic,
        generation: crate::TickGeneration::automatic(),
        source_indices: None,
    })
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
