//! One construction and union path for complete and factorized sections.

use super::{
    add, count, incompatible, merge_entry, shared_slice, GuideCounts, GuideKeySource,
    GuideSectionIdentity, PreparedDiscreteGuide, PreparedGuideEntry, PreparedGuideSection,
};
use crate::decorations::legend::identity::shared_text;
use crate::decorations::legend::identity::PrimaryTitleIdentity;
use crate::decorations::legend::source::{
    GuideEncoding, GuideExplanation, LiteralDomain, Projection,
};
use crate::decorations::legend::{GuideEntryIdentity, GuideSourceKey, ResolvedGuideKey};
use crate::layout::ComponentAddress;
use crate::LegendComposition;
use crate::{Error, Plot, Result};
use std::sync::Arc;

struct Section {
    identity: GuideSectionIdentity,
    title: Arc<str>,
    entries: Vec<PreparedGuideEntry>,
}

impl GuideSectionIdentity {
    fn title(&self, plot: &Plot) -> Result<Arc<str>> {
        match self {
            Self::Combined(primary) => shared_text(&primary.resolved_text(plot)?),
            Self::GroupStyle(field) | Self::Joint(field) | Self::HueColor(field) => {
                shared_text(&plot.resolve_label(field.as_str())?)
            }
            Self::Literal(LiteralDomain::BarSign) => shared_text("Change"),
            Self::Literal(_) | Self::SupplementalSeries => shared_text("series"),
        }
    }

    fn class(&self) -> u8 {
        match self {
            Self::Combined(_) | Self::GroupStyle(_) => 0,
            Self::Joint(_) => 1,
            Self::HueColor(_) => 2,
            Self::Literal(_) => 3,
            Self::SupplementalSeries => 4,
        }
    }

    fn validate_entry(&self, entry: &PreparedGuideEntry) -> Result<()> {
        let valid = match (self, &entry.identity) {
            (Self::Combined(primary), identity) => identity
                .primary_title()
                .is_none_or(|title| title == *primary),
            (
                Self::GroupStyle(field) | Self::Joint(field) | Self::HueColor(field),
                GuideEntryIdentity::Category { field: actual, .. },
            ) => field == actual,
            (Self::Literal(LiteralDomain::BarSign), GuideEntryIdentity::BarSign { .. })
            | (
                Self::Literal(LiteralDomain::DumbbellEndpoints),
                GuideEntryIdentity::DumbbellEndpoint { .. },
            )
            | (
                Self::Literal(LiteralDomain::RegressionReference),
                GuideEntryIdentity::RegressionObservation
                | GuideEntryIdentity::RegressionFit { .. },
            )
            | (Self::SupplementalSeries, GuideEntryIdentity::SupplementalLayer { .. }) => true,
            _ => false,
        };
        let projection = match self {
            Self::GroupStyle(_) => Projection::GroupStyle,
            Self::HueColor(_) => Projection::HueColor,
            _ => Projection::Complete,
        };
        if !valid
            || entry.sources.is_empty()
            || entry.sources.iter().any(|source| {
                source.projection != projection
                    || source.group_neutral_rgb.is_some() != (projection == Projection::GroupStyle)
            })
            || matches!(entry.key.explanation, GuideExplanation::Complete(_))
                != (projection == Projection::Complete)
        {
            return Err(Error::Render(
                "discrete section has inconsistent identity or projection".into(),
            ));
        }
        Ok(())
    }
}

impl PreparedDiscreteGuide {
    /// Source keys are consumed once. Only section entries and actual source
    /// occurrence records survive preparation; no complete-pair adapter remains.
    pub(crate) fn prepare(
        plot: &Plot,
        theme: &crate::theme::ResolvedTheme,
        owner: &ComponentAddress,
        keys: Vec<GuideSourceKey>,
        composition: LegendComposition,
    ) -> Result<Option<Arc<Self>>> {
        if keys.is_empty() || plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            return Ok(None);
        }
        let primary = if composition == LegendComposition::Combined {
            let mut primary = None;
            for source in &keys {
                if let Some(title) = source.identity.primary_title() {
                    if primary.as_ref().is_some_and(|expected| *expected != title) {
                        return Err(incompatible("plot legend"));
                    }
                    primary = Some(title);
                }
            }
            Some(GuideSectionIdentity::Combined(
                primary.unwrap_or(PrimaryTitleIdentity::Series),
            ))
        } else {
            None
        };
        let mut counts = GuideCounts::default();
        let mut sections = Vec::new();
        for (ordinal, mut source) in keys.into_iter().enumerate() {
            source.bind_alpha(plot)?;
            source.validate(plot)?;
            if source.identity.primary_title().is_some() {
                add(&mut counts.primary_source_keys, 1)?;
            } else {
                add(&mut counts.labeled_layer_source_keys, 1)?;
            }
            if matches!(source.identity, GuideEntryIdentity::Pair { .. }) {
                add(&mut counts.observed_pair_source_keys, 1)?;
            }
            let record = GuideKeySource {
                producer: plot.kind(),
                contributor_address: owner.clone(),
                source_ordinal: count(ordinal)?,
                mapping: source.mapping,
                projection: Projection::Complete,
                group_neutral_rgb: None,
            };
            if let Some(primary) = &primary {
                insert_source(
                    &mut sections,
                    plot,
                    primary.clone(),
                    source.identity,
                    source.key,
                    record,
                )?;
            } else {
                separate_source(
                    &mut sections,
                    plot,
                    theme,
                    source.identity,
                    source.key,
                    record,
                )?;
            }
        }
        finish(composition, sections, counts, &[]).map(Some)
    }

    /// Merge only prepared explanatory sections. Labels and key paints are
    /// never re-resolved under a first contributor or the parent palette.
    pub(crate) fn merge<'a>(
        guides: impl IntoIterator<Item = &'a Arc<Self>>,
        order: &[GuideEntryIdentity],
        context: &str,
    ) -> Result<Option<Arc<Self>>> {
        let mut composition = None;
        let mut sections: Vec<Section> = Vec::new();
        let mut primary: Option<(PrimaryTitleIdentity, Arc<str>)> = None;
        let mut counts = GuideCounts::default();
        for guide in guides {
            if composition.is_some_and(|expected| expected != guide.composition) {
                return Err(incompatible(context));
            }
            composition = Some(guide.composition);
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
                if guide.composition == LegendComposition::Combined {
                    merge_combined(&mut sections, &mut primary, section, context)?;
                } else {
                    for entry in section.entries.iter() {
                        insert(
                            &mut sections,
                            section.identity.clone(),
                            Arc::clone(&section.display_title),
                            entry.clone(),
                            context,
                        )?;
                    }
                }
            }
        }
        let Some(composition) = composition else {
            return Ok(None);
        };
        if composition == LegendComposition::Combined {
            let (identity, title) = primary.map_or_else(
                || Ok::<_, Error>((PrimaryTitleIdentity::Series, shared_text("series")?)),
                Ok,
            )?;
            let section = sections
                .first_mut()
                .ok_or_else(|| Error::Render("empty Combined guide".into()))?;
            section.identity = GuideSectionIdentity::Combined(identity);
            section.title = title;
        }
        finish(composition, sections, counts, order).map(Some)
    }
}

fn merge_combined(
    sections: &mut Vec<Section>,
    primary: &mut Option<(PrimaryTitleIdentity, Arc<str>)>,
    section: &PreparedGuideSection,
    context: &str,
) -> Result<()> {
    let GuideSectionIdentity::Combined(identity) = &section.identity else {
        return Err(incompatible(context));
    };
    if section
        .entries
        .iter()
        .any(|entry| entry.identity.primary_title().is_some())
    {
        if let Some((expected, title)) = primary {
            if expected != identity || *title != section.display_title {
                return Err(incompatible(context));
            }
        } else {
            *primary = Some((identity.clone(), Arc::clone(&section.display_title)));
        }
    }
    if sections.is_empty() {
        crate::data::source::reserve_source(sections, 1)?;
        sections.push(Section {
            identity: section.identity.clone(),
            title: Arc::clone(&section.display_title),
            entries: Vec::new(),
        });
    }
    for entry in section.entries.iter() {
        merge_entry(&mut sections[0].entries, entry.clone(), context)?;
    }
    Ok(())
}

fn insert_source(
    sections: &mut Vec<Section>,
    plot: &Plot,
    section: GuideSectionIdentity,
    identity: GuideEntryIdentity,
    key: ResolvedGuideKey,
    source: GuideKeySource,
) -> Result<()> {
    let existing = sections.iter().find(|current| current.identity == section);
    let title = existing.map_or_else(
        || section.title(plot),
        |current| Ok(Arc::clone(&current.title)),
    )?;
    let label = existing
        .and_then(|current| {
            current
                .entries
                .iter()
                .find(|entry| entry.identity == identity)
        })
        .map_or_else(
            || shared_text(&identity.resolved_label(plot, source.mapping.layer_label_context)?),
            |entry| Ok(Arc::clone(&entry.display_label)),
        )?;
    let entry = PreparedGuideEntry {
        identity,
        display_label: label,
        key,
        sources: shared_slice(crate::data::source::collect_source(std::iter::once(Ok(
            source,
        )))?)?,
    };
    insert(sections, section, title, entry, "plot legend")
}

fn insert(
    sections: &mut Vec<Section>,
    identity: GuideSectionIdentity,
    title: Arc<str>,
    entry: PreparedGuideEntry,
    context: &str,
) -> Result<()> {
    if let Some(section) = sections
        .iter_mut()
        .find(|section| section.identity == identity)
    {
        if section.title != title {
            return Err(incompatible(context));
        }
        return merge_entry(&mut section.entries, entry, context);
    }
    if !title.is_empty() && sections.iter().any(|section| section.title == title) {
        return Err(Error::Render(format!(
            "{context}: distinct section identities resolve to title {title:?}"
        )));
    }
    let mut entries = Vec::new();
    crate::data::source::reserve_source(&mut entries, 1)?;
    entries.push(entry);
    crate::data::source::reserve_source(sections, 1)?;
    sections.push(Section {
        identity,
        title,
        entries,
    });
    Ok(())
}

fn separate_source(
    sections: &mut Vec<Section>,
    plot: &Plot,
    theme: &crate::theme::ResolvedTheme,
    identity: GuideEntryIdentity,
    key: ResolvedGuideKey,
    mut source: GuideKeySource,
) -> Result<()> {
    let section = match source.mapping.encoding.clone() {
        GuideEncoding::UnsupportedPair { group, hue } => return Err(Error::Render(format!(
            "unsupported discrete guide factorization for {:?} at {:?}: group {:?} and hue {:?}, raw identity {identity:?}, have no configured group-style channel",
            source.producer, source.contributor_address, group.as_str(), hue.as_str(),
        ))),
        GuideEncoding::IndependentPair { group, hue } => {
            let GuideEntryIdentity::Pair { group_value, hue_value, .. } = identity else {
                return Err(Error::Render("independent guide source lost its pair identity".into()));
            };
            let paint = theme.typography.guides.legend_label.color();
            let neutral = crate::theme::OpaqueColor::rgb(paint.red(), paint.green(), paint.blue());
            let group_key = super::projection::project(&key, &source.mapping, Projection::GroupStyle, Some(neutral))?;
            let mut group_source = source.clone();
            group_source.projection = Projection::GroupStyle;
            group_source.group_neutral_rgb = Some(neutral);
            insert_source(sections, plot, GuideSectionIdentity::GroupStyle(group.clone()),
                GuideEntryIdentity::Category { field: group, value: group_value }, group_key, group_source)?;
            let hue_key = super::projection::project(&key, &source.mapping, Projection::HueColor, None)?;
            source.projection = Projection::HueColor;
            return insert_source(sections, plot, GuideSectionIdentity::HueColor(hue.clone()),
                GuideEntryIdentity::Category { field: hue, value: hue_value }, hue_key, source);
        }
        GuideEncoding::HueOnly { field } | GuideEncoding::RowHue { field } => {
            let key = super::projection::project(&key, &source.mapping, Projection::HueColor, None)?;
            source.projection = Projection::HueColor;
            return insert_source(sections, plot, GuideSectionIdentity::HueColor(field), identity, key, source);
        }
        GuideEncoding::Joint { field } => GuideSectionIdentity::Joint(field),
        GuideEncoding::Literal(domain) => GuideSectionIdentity::Literal(domain),
        GuideEncoding::SupplementalLayer => GuideSectionIdentity::SupplementalSeries,
    };
    insert_source(sections, plot, section, identity, key, source)
}

fn finish(
    composition: LegendComposition,
    mut sections: Vec<Section>,
    mut counts: GuideCounts,
    order: &[GuideEntryIdentity],
) -> Result<Arc<PreparedDiscreteGuide>> {
    if sections.is_empty() {
        return Err(Error::Render("a discrete guide must be nonempty".into()));
    }
    for index in 1..sections.len() {
        let mut current = index;
        while current > 0
            && sections[current - 1].identity.class() > sections[current].identity.class()
        {
            sections.swap(current - 1, current);
            current -= 1;
        }
    }
    counts.sections = count(sections.len())?;
    let mut retained = Vec::new();
    crate::data::source::reserve_source(&mut retained, sections.len())?;
    for mut section in sections {
        if section.entries.is_empty()
            || matches!(section.identity, GuideSectionIdentity::Combined(_))
                != (composition == LegendComposition::Combined)
        {
            return Err(Error::Render(
                "discrete guide has an empty or policy-inconsistent section".into(),
            ));
        }
        order_entries(&mut section.entries, order)?;
        add(&mut counts.entries, count(section.entries.len())?)?;
        for entry in &section.entries {
            section.identity.validate_entry(entry)?;
            add(&mut counts.points, u32::from(entry.key.point.is_some()))?;
            add(&mut counts.lines, u32::from(entry.key.line.is_some()))?;
            add(&mut counts.fills, u32::from(entry.key.fill.is_some()))?;
            add(&mut counts.source_occurrences, count(entry.sources.len())?)?;
        }
        retained.push(PreparedGuideSection {
            identity: section.identity,
            display_title: section.title,
            entries: shared_slice(section.entries)?,
        });
    }
    crate::data::source::source_arc(PreparedDiscreteGuide {
        composition,
        counts,
        sections: shared_slice(retained)?,
    })
}

fn order_entries(entries: &mut [PreparedGuideEntry], order: &[GuideEntryIdentity]) -> Result<()> {
    let mut positions = crate::data::source::collect_source(entries.iter().map(|entry| {
        let position = order
            .iter()
            .position(|identity| *identity == entry.identity)
            .or_else(|| {
                order
                    .iter()
                    .position(|identity| match (&entry.identity, identity) {
                        (
                            GuideEntryIdentity::Category { field, value },
                            GuideEntryIdentity::Pair {
                                group_field,
                                group_value,
                                hue_field,
                                hue_value,
                            },
                        ) => {
                            (field == group_field && value == group_value)
                                || (field == hue_field && value == hue_value)
                        }
                        _ => false,
                    })
            })
            .unwrap_or(usize::MAX);
        Ok((entry.identity.primary_title().is_none(), position))
    }))?;
    for index in 1..entries.len() {
        let mut current = index;
        while current > 0 && positions[current - 1] > positions[current] {
            positions.swap(current - 1, current);
            entries.swap(current - 1, current);
            current -= 1;
        }
    }
    Ok(())
}
