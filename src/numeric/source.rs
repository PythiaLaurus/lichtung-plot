//! Operation and semantic-owner binding, distinct from the normalized policy.

use super::context::{
    ContextInfo, ContextValue, DomainInfo, Identity, NumericPurpose, NumericRangeOrigin,
    NumericRole, OwnerInfo,
};
use crate::{Error, Plot, Result};

#[derive(Debug, Clone)]
pub(crate) struct NumericSource {
    pub(crate) operation: crate::font::canonical::OperationContext,
    pub(crate) owner: OwnerInfo,
    raw_limits: [Option<(u64, u64)>; 2],
}
impl PartialEq for NumericSource {
    fn eq(&self, other: &Self) -> bool {
        self.operation.same_operation(&other.operation)
            && self.owner == other.owner
            && self.raw_limits == other.raw_limits
    }
}
impl Eq for NumericSource {}

impl OwnerInfo {
    fn try_copy(&self) -> Result<Self> {
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
            canvas: plot.numeric_cell.as_deref().copied(),
            facet: facet.map(crate::FacetCell::try_copy).transpose()?,
            shared: false,
            scope: plot
                .axis_scope()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
        })
    }
}

impl NumericSource {
    pub(crate) fn raw_limits(&self, side: crate::layout::AxisSide) -> Option<(f64, f64)> {
        self.raw_limits[usize::from(side == crate::layout::AxisSide::Left)]
            .map(|(a, b)| (f64::from_bits(a), f64::from_bits(b)))
    }
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            operation: self.operation.clone(),
            owner: self.owner.try_copy()?,
            raw_limits: self.raw_limits,
        })
    }
    pub(crate) fn for_data(plot: &Plot, data: crate::data::PlotData<'_>) -> Result<Self> {
        Self::for_plot(plot, data.source_resources()?, data.numeric_facet())
    }
    pub(crate) fn for_plot(
        plot: &Plot,
        operation: &crate::font::canonical::OperationContext,
        facet: Option<&crate::FacetCell>,
    ) -> Result<Self> {
        Ok(Self {
            operation: operation.clone(),
            owner: OwnerInfo::for_plot(plot, facet)?,
            raw_limits: [plot.user_x_limits(), plot.user_y_limits()]
                .map(|limits| limits.map(|(a, b)| (a.to_bits(), b.to_bits()))),
        })
    }
    pub(crate) fn context(
        &self,
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
        &self,
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

#[derive(Debug, Clone)]
pub(crate) struct AxisRequest<'a> {
    pub(crate) format: &'a crate::NumericFormat,
    pub(crate) source: std::sync::Arc<NumericSource>,
    pub(crate) provenance: crate::layout::RangeProvenance,
    pub(crate) generation: crate::TickGeneration,
}

impl AxisRequest<'_> {
    #[cfg(test)]
    pub(crate) fn for_test(format: &crate::NumericFormat) -> AxisRequest<'_> {
        AxisRequest {
            format,
            source: std::sync::Arc::new(NumericSource {
                operation: crate::font::canonical::OperationContext::new(),
                owner: OwnerInfo {
                    kind: None,
                    canvas: None,
                    facet: None,
                    shared: false,
                    scope: None,
                },
                raw_limits: [None; 2],
            }),
            provenance: crate::layout::RangeProvenance::Automatic,
            generation: crate::TickGeneration::automatic(),
        }
    }
    pub(crate) fn materialize(
        &self,
        positions: &[super::operand::Operand],
        log: bool,
        side: crate::layout::AxisSide,
        range: (f64, f64),
        purpose: NumericPurpose,
    ) -> Result<Option<super::set::FormattedSet>> {
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
            self.source.raw_limits[index].map(|(a, b)| (f64::from_bits(a), f64::from_bits(b)))
        } else {
            Some(range)
        };
        let domain = DomainInfo::axis(range, scale, self.provenance, raw);
        let mut info = self
            .source
            .indexed(positions, role, purpose, Some(domain))?;
        info.generation = Some(self.generation.clone());
        self.format.materialize(
            &self.source.operation,
            info,
            purpose == NumericPurpose::ActualSet,
        )
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
