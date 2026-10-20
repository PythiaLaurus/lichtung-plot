//! The single immutable discrete guide: semantic sections, resolved keys and
//! actual source occurrences, prepared before fonts or panel assignment.

mod composition;
mod projection;
pub(crate) mod scoped;
#[cfg(test)]
mod source_tests;

use std::sync::Arc;

use super::identity::{PrimaryTitleIdentity, RawField};
use super::source::{LiteralDomain, Projection, SourceMapping};
use super::{GuideEntryIdentity, ResolvedGuideKey};
use crate::layout::{AddressRole, ComponentAddress, PreparedTextRequirement, PreparedTextRole};
use crate::{Error, LegendComposition, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideSectionIdentity {
    Combined(PrimaryTitleIdentity),
    GroupStyle(RawField),
    Joint(RawField),
    HueColor(RawField),
    Literal(LiteralDomain),
    SupplementalSeries,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuideKeySource {
    producer: crate::PlotKind,
    contributor_address: ComponentAddress,
    source_ordinal: u32,
    mapping: SourceMapping,
    projection: Projection,
    group_neutral_rgb: Option<crate::theme::OpaqueColor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedGuideEntry {
    pub(crate) identity: GuideEntryIdentity,
    pub(crate) display_label: Arc<str>,
    pub(crate) key: ResolvedGuideKey,
    sources: Arc<[GuideKeySource]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedGuideSection {
    pub(crate) identity: GuideSectionIdentity,
    pub(crate) display_title: Arc<str>,
    pub(crate) entries: Arc<[PreparedGuideEntry]>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GuideCounts {
    pub(crate) sections: u32,
    pub(crate) entries: u32,
    pub(crate) points: u32,
    pub(crate) lines: u32,
    pub(crate) fills: u32,
    pub(crate) primary_source_keys: u32,
    pub(crate) observed_pair_source_keys: u32,
    pub(crate) labeled_layer_source_keys: u32,
    pub(crate) source_occurrences: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreparedDiscreteGuide {
    pub(crate) composition: LegendComposition,
    pub(crate) counts: GuideCounts,
    pub(crate) sections: Arc<[PreparedGuideSection]>,
}

/// A consumed guide retains its semantic owner even when routed to a facet
/// cell. Placement never reparents its section/title/entry addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscreteGuideSlot {
    pub(crate) descriptor: Arc<PreparedDiscreteGuide>,
    pub(crate) owner: ComponentAddress,
}

impl DiscreteGuideSlot {
    pub(crate) fn new(
        descriptor: Arc<PreparedDiscreteGuide>,
        parent: &ComponentAddress,
        position: crate::LegendPosition,
    ) -> Result<Self> {
        Ok(Self {
            descriptor,
            owner: parent.child(super::address_role(position), 0)?,
        })
    }
}

fn count(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| Error::Render("discrete guide count overflow".into()))
}

pub(super) fn shared_slice<T>(values: Vec<T>) -> Result<Arc<[T]>> {
    let storage = std::alloc::Layout::array::<T>(values.len())
        .map_err(|_| Error::Render("discrete guide storage overflow".into()))?;
    std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(storage)
        .map_err(|_| Error::Render("discrete guide Arc layout overflow".into()))?;
    Ok(values.into())
}

impl PreparedDiscreteGuide {
    #[cfg(test)]
    pub(crate) fn prepare_for_test(
        plot: &crate::Plot,
        theme: &crate::theme::ResolvedTheme,
        composition: LegendComposition,
    ) -> Result<Option<Arc<Self>>> {
        let operation = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(plot, &operation, Some(theme))?;
        let data = plot.data().with_operation(&operation);
        let keys = if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            Vec::new()
        } else if let Some(matrix) = &inputs.matrix {
            matrix.guide_sources(plot, theme, data)?
        } else {
            inputs.guide_sources(plot, theme, data, plot.axis_scope())?
        };
        Self::prepare(
            plot,
            theme,
            &ComponentAddress::root().child(AddressRole::PlotBody, 0)?,
            keys,
            composition,
        )
    }

    pub(crate) fn append_requirements(
        &self,
        requirements: &mut Vec<PreparedTextRequirement>,
        theme: &crate::theme::ResolvedTheme,
        owner: &ComponentAddress,
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<()> {
        for (ordinal, section) in self.sections.iter().enumerate() {
            let section_owner = owner.child(AddressRole::LegendSection, count(ordinal)?)?;
            if !section.display_title.is_empty() {
                PreparedTextRequirement::push_into(
                    requirements,
                    section_owner.child(AddressRole::LegendTitle, 0)?,
                    PreparedTextRole::LegendTitle,
                    0,
                    section.display_title.as_ref(),
                    &[],
                    &theme.typography.guides.legend_title,
                    stage.as_deref_mut(),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
            }
            for (index, entry) in section.entries.iter().enumerate() {
                if entry.display_label.is_empty() {
                    continue;
                }
                PreparedTextRequirement::push_into(
                    requirements,
                    section_owner
                        .child(AddressRole::LegendEntry, count(index)?)?
                        .child(AddressRole::LegendLabel, 0)?,
                    PreparedTextRole::LegendLabel,
                    0,
                    entry.display_label.as_ref(),
                    &[],
                    &theme.typography.guides.legend_label,
                    stage.as_deref_mut(),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
            }
        }
        Ok(())
    }
}

fn add(target: &mut u32, value: u32) -> Result<()> {
    *target = u32::try_from(u128::from(*target) + u128::from(value))
        .map_err(|_| Error::Render("discrete guide count overflow".into()))?;
    Ok(())
}

fn incompatible(context: &str) -> Error {
    Error::Render(format!("{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"))
}

fn merge_entry(
    entries: &mut Vec<PreparedGuideEntry>,
    entry: PreparedGuideEntry,
    context: &str,
) -> Result<()> {
    for existing in entries.iter_mut() {
        if existing.identity == entry.identity {
            if existing.display_label != entry.display_label {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles: conflicting labels for raw identity {:?}",
                    entry.identity
                )));
            }
            if existing.key != entry.key {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles: raw identity {:?} from {:?} has incompatible {}; glyphs must match exactly",
                    entry.identity, entry.sources.first(), existing.key.mismatched_component(&entry.key),
                )));
            }
            let length = existing
                .sources
                .len()
                .checked_add(entry.sources.len())
                .ok_or_else(|| Error::Render("guide source occurrence count overflow".into()))?;
            let mut sources = Vec::new();
            crate::data::source::reserve_source(&mut sources, length)?;
            sources.extend(existing.sources.iter().chain(entry.sources.iter()).cloned());
            existing.sources = shared_slice(sources)?;
            return Ok(());
        }
        if existing.display_label == entry.display_label {
            return Err(Error::Render(format!(
                "{context} has distinct raw identities resolving to display label {:?}",
                entry.display_label
            )));
        }
    }
    crate::data::source::reserve_source(entries, 1)?;
    entries.push(entry);
    Ok(())
}

/// Explicit per-scope contribution and one owning root union. None means actual
/// noncontribution; an absent preparation owner is a different invariant state.
#[derive(Debug)]
pub(crate) struct PreparedDiscreteGuides {
    pub(crate) root: Option<Arc<PreparedDiscreteGuide>>,
    pub(crate) panels: Vec<ScopedDiscreteGuide>,
}

#[derive(Debug)]
pub(crate) struct ScopedDiscreteGuide {
    pub(crate) scope: Option<String>,
    pub(crate) descriptor: Option<Arc<PreparedDiscreteGuide>>,
}

impl PreparedDiscreteGuides {
    pub(crate) fn for_scope(
        &self,
        scope: Option<&str>,
    ) -> Result<Option<&Arc<PreparedDiscreteGuide>>> {
        self.panels
            .iter()
            .find(|panel| panel.scope.as_deref() == scope)
            .map(|panel| panel.descriptor.as_ref())
            .ok_or_else(|| Error::Render("discrete guide is missing its prepared scope".into()))
    }
}
