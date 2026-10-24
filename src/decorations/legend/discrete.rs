//! The single immutable guide: typed sections, resolved keys and
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
    ContinuousColor(RawField),
    Alpha(super::alpha::AlphaSectionIdentity),
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
    pub(crate) display_label: Arc<crate::Text>,
    pub(crate) key: ResolvedGuideKey,
    pub(super) sources: Arc<[GuideKeySource]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedGuideSection {
    pub(crate) identity: GuideSectionIdentity,
    pub(crate) display_title: Arc<crate::Text>,
    pub(crate) body: GuideSectionBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideSectionBody {
    Symbols(Arc<[PreparedGuideEntry]>),
    ContinuousColor(Arc<super::descriptor::ContinuousDescriptor>),
    Alpha(Arc<super::alpha::PreparedAlphaSection>),
}

impl PreparedGuideSection {
    pub(crate) fn entries(&self) -> &[PreparedGuideEntry] {
        match &self.body {
            GuideSectionBody::Symbols(entries) => entries,
            GuideSectionBody::Alpha(alpha) => &alpha.entries,
            GuideSectionBody::ContinuousColor(_) => &[],
        }
    }
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
pub(crate) struct PreparedGuide {
    pub(crate) composition: LegendComposition,
    pub(crate) counts: GuideCounts,
    pub(crate) sections: Arc<[PreparedGuideSection]>,
}

/// A consumed guide retains its semantic owner even when routed to a facet
/// cell. Placement never reparents its section/title/entry addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuideSlot {
    pub(crate) descriptor: Arc<PreparedGuide>,
    pub(crate) owner: ComponentAddress,
}

impl GuideSlot {
    pub(crate) fn new(
        descriptor: Arc<PreparedGuide>,
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

impl PreparedGuide {
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
        let continuous = if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            None
        } else {
            inputs.continuous_source(plot, theme, plot.axis_scope())?
        };
        let alpha = if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            Vec::new()
        } else {
            inputs
                .alpha_sources(
                    plot,
                    theme,
                    plot.axis_scope(),
                    &ComponentAddress::root().child(AddressRole::PlotBody, 0)?,
                )?
                .sections(plot)?
        };
        Self::prepare(
            plot,
            theme,
            &ComponentAddress::root().child(AddressRole::PlotBody, 0)?,
            keys,
            continuous,
            alpha,
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
            PreparedTextRequirement::push_authored(
                requirements,
                section_owner.child(AddressRole::LegendTitle, 0)?,
                PreparedTextRole::LegendTitle,
                Some(&section.display_title),
                &theme.typography.guides.legend_title,
                false,
                stage.as_deref_mut(),
            )
            .map_err(|error| Error::Render(error.to_string()))?;
            for (index, entry) in section.entries().iter().enumerate() {
                PreparedTextRequirement::push_authored(
                    requirements,
                    section_owner
                        .child(AddressRole::LegendEntry, count(index)?)?
                        .child(AddressRole::LegendLabel, 0)?,
                    PreparedTextRole::LegendLabel,
                    Some(&entry.display_label),
                    &theme.typography.guides.legend_label,
                    false,
                    stage.as_deref_mut(),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
            }
            if let GuideSectionBody::ContinuousColor(color) = &section.body {
                for (index, label) in color.numeric.labels.iter().enumerate() {
                    PreparedTextRequirement::push_into(
                        requirements,
                        section_owner.child(AddressRole::GuideLabel, count(index)?)?,
                        PreparedTextRole::LegendLabel,
                        count(index)?,
                        label,
                        &[],
                        &theme.typography.guides.legend_label,
                        stage.as_deref_mut(),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                }
                PreparedTextRequirement::push_authored(
                    requirements,
                    section_owner
                        .child(AddressRole::GuideLabel, count(color.numeric.labels.len())?)?,
                    PreparedTextRole::LegendLabel,
                    color.numeric.explanation.as_ref(),
                    &theme.typography.guides.legend_label,
                    false,
                    stage.as_deref_mut(),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                if let Some(missing) = &color.missing {
                    PreparedTextRequirement::push_into(
                        requirements,
                        section_owner.child(AddressRole::GuideMissingBlock, 0)?,
                        PreparedTextRole::LegendLabel,
                        0,
                        &missing.label,
                        &[],
                        &theme.typography.guides.legend_label,
                        stage.as_deref_mut(),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                }
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
        if existing.display_label.as_str() == entry.display_label.as_str() {
            return Err(Error::Render(format!(
                "{context} has distinct raw identities resolving to display label {:?}",
                entry.display_label.as_str()
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
pub(crate) struct PreparedGuides {
    pub(crate) root: Option<Arc<PreparedGuide>>,
    pub(crate) panels: Vec<ScopedGuide>,
}

#[derive(Debug)]
pub(crate) struct ScopedGuide {
    pub(crate) scope: Option<String>,
    pub(crate) descriptor: Option<Arc<PreparedGuide>>,
    pub(crate) alpha: Arc<super::alpha::PreparedAlphaInputs>,
}

impl PreparedGuides {
    pub(crate) fn alpha_for_scope(
        &self,
        scope: Option<&str>,
    ) -> Result<&Arc<super::alpha::PreparedAlphaInputs>> {
        self.panels
            .iter()
            .find(|panel| panel.scope.as_deref() == scope)
            .map(|panel| &panel.alpha)
            .ok_or_else(|| Error::Render("alpha inputs are missing their prepared scope".into()))
    }

    pub(crate) fn for_scope(&self, scope: Option<&str>) -> Result<Option<&Arc<PreparedGuide>>> {
        self.panels
            .iter()
            .find(|panel| panel.scope.as_deref() == scope)
            .map(|panel| panel.descriptor.as_ref())
            .ok_or_else(|| Error::Render("discrete guide is missing its prepared scope".into()))
    }
}

impl PreparedGuide {
    pub(crate) fn continuous(&self) -> Option<&Arc<super::descriptor::ContinuousDescriptor>> {
        self.sections
            .iter()
            .find_map(|section| match &section.body {
                GuideSectionBody::ContinuousColor(color) => Some(color),
                GuideSectionBody::Symbols(_) | GuideSectionBody::Alpha(_) => None,
            })
    }

    pub(crate) fn has_symbols(&self) -> bool {
        self.sections
            .iter()
            .any(|section| !section.entries().is_empty())
    }
}
