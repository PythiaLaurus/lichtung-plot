//! Only the parent operands consumed by a particular Canvas cell.

use super::Canvas;
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, DependencyUse, GuideAllocation,
};
use crate::layout::kernel::FeedbackRound;
use crate::layout::{AddressRole, ComponentAddress};
use crate::Error;

impl Canvas {
    pub(super) fn consumed_guide_dependencies(
        &self,
        index: usize,
        width: f64,
        shared_allocation: Option<GuideAllocation>,
        round: &FeedbackRound,
    ) -> Result<Vec<ConsumedGuideDependency>, Error> {
        let count = self
            .x_range_groups
            .iter()
            .filter(|group| group.contains(&index))
            .count()
            .checked_add(usize::from(shared_allocation.is_some()))
            .ok_or_else(|| Error::Render("Canvas dependency count overflowed".into()))?;
        let mut dependencies = Vec::new();
        crate::layout::fit::reserve(&mut dependencies, count, "Canvas consumed dependencies")?;
        // CanvasGrid precedes CanvasSharedLegend in canonical address order.
        // Only groups containing this cell are operands; other groups/siblings
        // do not enter either the snapshot or its stability check.
        let root = ComponentAddress::root();
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            if group.contains(&index) {
                let owner = root.child(AddressRole::CanvasGrid, 0)?.child(
                    AddressRole::CanvasRangeGroup,
                    u32::try_from(group_index)
                        .map_err(|_| Error::Render("Canvas group address overflowed".into()))?,
                )?;
                dependencies.push(ConsumedGuideDependency::new(
                    owner,
                    DependencyUse::AllocationOnly,
                    GuideAllocation::width(width)?,
                    round,
                )?);
            }
        }
        if let Some(allocation) = shared_allocation {
            dependencies.push(ConsumedGuideDependency::new(
                root.child(AddressRole::CanvasSharedLegend, 0)?,
                DependencyUse::AllocationOnly,
                allocation,
                round,
            )?);
        }
        Ok(dependencies)
    }
}
