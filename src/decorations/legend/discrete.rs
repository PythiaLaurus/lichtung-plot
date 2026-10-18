//! The single immutable discrete guide: semantic sections, resolved keys and
//! actual source occurrences, prepared before fonts or panel assignment.

use std::sync::Arc;

use super::identity::PrimaryTitleIdentity;
use super::{GuideEntryIdentity, GuideSourceKey, ResolvedGuideKey};
use crate::layout::{AddressRole, ComponentAddress, PreparedTextRequirement, PreparedTextRole};
use crate::{Error, Plot, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LegendComposition {
    #[default]
    Combined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideSectionIdentity {
    Combined(PrimaryTitleIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuideKeySource {
    producer: crate::PlotKind,
    contributor_address: ComponentAddress,
    source_ordinal: u32,
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
    /// Source keys are transient producer operands, never a second retained
    /// flat guide. Only this constructor resolves their display semantics.
    pub(crate) fn prepare(
        plot: &Plot,
        owner: &ComponentAddress,
        keys: Vec<GuideSourceKey>,
    ) -> Result<Option<Arc<Self>>> {
        if keys.is_empty() {
            return Ok(None);
        }
        let mut primary = None;
        let mut counts = GuideCounts::default();
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, keys.len())?;
        for (ordinal, source) in keys.into_iter().enumerate() {
            if let Some(title) = source.identity.primary_title() {
                if primary.as_ref().is_some_and(|expected| *expected != title) {
                    return Err(incompatible("plot legend"));
                }
                primary = Some(title);
                add(&mut counts.primary_source_keys, 1)?;
            } else {
                add(&mut counts.labeled_layer_source_keys, 1)?;
            }
            if matches!(source.identity, GuideEntryIdentity::Pair { .. }) {
                add(&mut counts.observed_pair_source_keys, 1)?;
            }
            let label = source.identity.resolved_label(plot)?;
            entries.push(PreparedGuideEntry {
                identity: source.identity,
                display_label: super::identity::shared_text(&label)?,
                key: source.key,
                sources: shared_slice(crate::data::source::collect_source(std::iter::once(Ok(
                    GuideKeySource {
                        producer: plot.kind(),
                        contributor_address: owner.clone(),
                        source_ordinal: count(ordinal)?,
                    },
                )))?)?,
            });
        }
        let primary = primary.unwrap_or(PrimaryTitleIdentity::Series);
        let title = primary.resolved_text(plot)?;
        let guide = Self::finish(
            primary,
            super::identity::shared_text(&title)?,
            merge_entries(&entries, "plot legend")?,
            counts,
        )?;
        Ok(Some(guide))
    }

    /// Union already resolved sections. Layer-only contributors cannot choose
    /// a primary domain or cause first-contributor title/label resolution.
    pub(crate) fn merge<'a>(
        guides: impl IntoIterator<Item = &'a Arc<Self>>,
        order: &[GuideEntryIdentity],
        context: &str,
    ) -> Result<Option<Arc<Self>>> {
        let mut primary: Option<(PrimaryTitleIdentity, Arc<str>)> = None;
        let mut entries = Vec::new();
        let mut counts = GuideCounts::default();
        for guide in guides {
            add(
                &mut counts.primary_source_keys,
                guide.counts.primary_source_keys,
            )?;
            add(
                &mut counts.observed_pair_source_keys,
                guide.counts.observed_pair_source_keys,
            )?;
            add(
                &mut counts.labeled_layer_source_keys,
                guide.counts.labeled_layer_source_keys,
            )?;
            for section in guide.sections.iter() {
                let GuideSectionIdentity::Combined(identity) = &section.identity;
                if section
                    .entries
                    .iter()
                    .any(|entry| entry.identity.primary_title().is_some())
                {
                    if let Some((expected, title)) = &primary {
                        if expected != identity || *title != section.display_title {
                            return Err(incompatible(context));
                        }
                    } else {
                        primary = Some((identity.clone(), Arc::clone(&section.display_title)));
                    }
                }
                crate::data::source::reserve_source(&mut entries, section.entries.len())?;
                entries.extend(section.entries.iter().cloned());
            }
        }
        if entries.is_empty() {
            return Ok(None);
        }
        let (identity, title) = primary.map_or_else(
            || {
                Ok::<_, Error>((
                    PrimaryTitleIdentity::Series,
                    super::identity::shared_text("series")?,
                ))
            },
            Ok,
        )?;
        let mut merged = merge_entries(&entries, context)?;
        if !order.is_empty() {
            let mut positions = crate::data::source::collect_source(merged.iter().map(|entry| {
                Ok(order
                    .iter()
                    .position(|identity| *identity == entry.identity)
                    .unwrap_or(usize::MAX))
            }))?;
            for index in 1..merged.len() {
                let mut current = index;
                while current > 0 && positions[current - 1] > positions[current] {
                    positions.swap(current - 1, current);
                    merged.swap(current - 1, current);
                    current -= 1;
                }
            }
        }
        Self::finish(identity, title, merged, counts).map(Some)
    }

    fn finish(
        identity: PrimaryTitleIdentity,
        title: Arc<str>,
        entries: Vec<PreparedGuideEntry>,
        mut counts: GuideCounts,
    ) -> Result<Arc<Self>> {
        if entries.is_empty() {
            return Err(Error::Render("a discrete section must be nonempty".into()));
        }
        counts.sections = 1;
        counts.entries = count(entries.len())?;
        for entry in &entries {
            add(&mut counts.points, u32::from(entry.key.point.is_some()))?;
            add(&mut counts.lines, u32::from(entry.key.line.is_some()))?;
            add(&mut counts.fills, u32::from(entry.key.fill.is_some()))?;
            add(&mut counts.source_occurrences, count(entry.sources.len())?)?;
        }
        crate::data::source::source_arc(Self {
            composition: LegendComposition::Combined,
            counts,
            sections: shared_slice(crate::data::source::collect_source(std::iter::once(Ok(
                PreparedGuideSection {
                    identity: GuideSectionIdentity::Combined(identity),
                    display_title: title,
                    entries: shared_slice(entries)?,
                },
            )))?)?,
        })
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

fn merge_entries(entries: &[PreparedGuideEntry], context: &str) -> Result<Vec<PreparedGuideEntry>> {
    let mut merged = Vec::new();
    for supplemental in [false, true] {
        for entry in entries
            .iter()
            .filter(|entry| entry.identity.primary_title().is_none() == supplemental)
        {
            merge_entry(&mut merged, entry.clone(), context)?;
        }
    }
    Ok(merged)
}

fn merge_entry(
    entries: &mut Vec<PreparedGuideEntry>,
    entry: PreparedGuideEntry,
    context: &str,
) -> Result<()> {
    for existing in entries.iter_mut() {
        if existing.identity == entry.identity {
            if existing.display_label != entry.display_label || existing.key != entry.key {
                return Err(incompatible(context));
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
