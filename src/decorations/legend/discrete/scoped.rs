//! Exact operands actually consumed by a scoped guide/child measurement.

use std::sync::Arc;

use super::{GuideSlot, PreparedGuide};
use crate::layout::kernel::{ChildFeedbackInputs, FeedbackRound, LayoutScalar, SpatialRounding};
use crate::layout::{ComponentAddress, PanelBox};
use crate::{Error, LegendComposition, Plot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuideUse {
    Hidden,
    SharedSuppressed,
    Local,
}

/// Borrowed call operands; retained keys own immutable handles separately.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GuideConsumption<'a> {
    pub(crate) use_kind: GuideUse,
    pub(crate) local_guide: Option<&'a GuideSlot>,
    pub(crate) dependencies: &'a [ConsumedGuideDependency],
}

pub(crate) const SUPPRESSED_GUIDES: GuideConsumption<'static> = GuideConsumption {
    use_kind: GuideUse::SharedSuppressed,
    local_guide: None,
    dependencies: &[],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuideOrientation {
    Horizontal,
    Vertical,
}

impl GuideOrientation {
    pub(crate) const fn from_horizontal(horizontal: bool) -> Self {
        if horizontal {
            Self::Horizontal
        } else {
            Self::Vertical
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuideAllocation {
    Width(i64),
    Box([i64; 4]),
}

impl GuideAllocation {
    pub(crate) fn width(width: f64) -> Result<Self, Error> {
        let width = LayoutScalar::from_css_pixels(width, SpatialRounding::Floor)?.quanta();
        if width < 0 {
            return Err(Error::Render("negative consumed guide width".into()));
        }
        Ok(Self::Width(width))
    }

    pub(crate) fn bounds(bounds: PanelBox) -> Result<Self, Error> {
        let mut edges = [0; 4];
        for (target, source) in edges
            .iter_mut()
            .zip([bounds.x0, bounds.x1, bounds.y0, bounds.y1])
        {
            *target = LayoutScalar::from_css_pixels(source, SpatialRounding::NearestEven)?.quanta();
        }
        if edges[1] < edges[0] || edges[3] < edges[2] {
            return Err(Error::Render("inverted consumed guide allocation".into()));
        }
        Ok(Self::Box(edges))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DependencyUse {
    AllocationOnly,
    DescriptorAndAllocation {
        descriptor: Arc<PreparedGuide>,
        orientation: GuideOrientation,
        compact: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumedGuideDependency {
    pub(crate) owner: ComponentAddress,
    use_kind: DependencyUse,
    allocation: GuideAllocation,
    feedback: Arc<ChildFeedbackInputs>,
}

impl ConsumedGuideDependency {
    pub(crate) fn new(
        owner: ComponentAddress,
        use_kind: DependencyUse,
        allocation: GuideAllocation,
        round: &FeedbackRound,
    ) -> Result<Self, Error> {
        let feedback = round.child_inputs(std::slice::from_ref(&owner))?;
        Ok(Self {
            owner,
            use_kind,
            allocation,
            feedback: Arc::new(feedback),
        })
    }

    fn stable(&self, round: &FeedbackRound) -> bool {
        let owners = std::slice::from_ref(&self.owner);
        round.child_inputs_equal(&self.feedback, owners) && round.child_stable(owners)
    }
}

/// A source contribution can be transported without local placement. Keep it
/// distinct from the descriptor actually drawn by a routed/Shared owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopedGuideInputs {
    composition: LegendComposition,
    use_kind: GuideUse,
    contribution_prepared: bool,
    contribution: Option<Arc<PreparedGuide>>,
    local_guide: Option<GuideSlot>,
    dependencies: Arc<[ConsumedGuideDependency]>,
}

impl ScopedGuideInputs {
    pub(crate) fn new(
        plot: &Plot<impl crate::kind::Marker>,
        use_kind: GuideUse,
        local_guide: Option<&GuideSlot>,
        dependencies: &[ConsumedGuideDependency],
    ) -> Result<Self, Error> {
        if use_kind != GuideUse::Local && local_guide.is_some() {
            return Err(Error::Render(
                "suppressed guide scope has a local placement descriptor".into(),
            ));
        }
        let contribution = plot
            .prepared_discrete_guides
            .as_ref()
            .map(|guides| guides.for_scope(plot.axis_scope()))
            .transpose()?
            .flatten()
            .cloned();
        let composition = plot.effective_legend_composition();
        if contribution
            .iter()
            .chain(local_guide.map(|slot| &slot.descriptor))
            .any(|guide| guide.composition != composition)
        {
            return Err(Error::Render(
                "consumed guide composition differs from its effective source".into(),
            ));
        }
        if dependencies
            .windows(2)
            .any(|pair| pair[0].owner >= pair[1].owner)
        {
            return Err(Error::Render(
                "consumed guide dependencies are not unique canonical owners".into(),
            ));
        }
        let mut retained = Vec::new();
        crate::layout::fit::reserve(
            &mut retained,
            dependencies.len(),
            "consumed guide dependencies",
        )?;
        retained.extend_from_slice(dependencies);
        Ok(Self {
            composition,
            use_kind,
            contribution_prepared: plot.prepared_discrete_guides.is_some(),
            contribution,
            local_guide: local_guide.cloned(),
            dependencies: super::shared_slice(retained)?,
        })
    }

    pub(crate) fn matches_source(&self, plot: &Plot<impl crate::kind::Marker>) -> bool {
        self.composition == plot.effective_legend_composition()
            && self.contribution_prepared == plot.prepared_discrete_guides.is_some()
            && match &plot.prepared_discrete_guides {
                Some(guides) => guides
                    .for_scope(plot.axis_scope())
                    .is_ok_and(|guide| guide == self.contribution.as_ref()),
                None => self.contribution.is_none(),
            }
    }

    pub(crate) fn stable(&self, round: &FeedbackRound) -> bool {
        self.dependencies
            .iter()
            .all(|dependency| dependency.stable(round))
    }

    pub(crate) fn matches_consumption(
        &self,
        use_kind: GuideUse,
        local_guide: Option<&GuideSlot>,
        dependencies: &[ConsumedGuideDependency],
    ) -> bool {
        self.use_kind == use_kind
            && self.local_guide.as_ref() == local_guide
            && self.dependencies.as_ref() == dependencies
    }

    #[cfg(test)]
    pub(crate) fn for_test(plot: &Plot) -> Self {
        Self::new(plot, GuideUse::SharedSuppressed, None, &[]).unwrap()
    }
}
