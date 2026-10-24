//! One construction and union path for complete and factorized sections.

use super::{
    add, count, incompatible, merge_entry, shared_slice, GuideCounts, GuideKeySource,
    GuideSectionBody, GuideSectionIdentity, PreparedGuide, PreparedGuideEntry,
    PreparedGuideSection,
};
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
    title: Arc<crate::Text>,
    entries: Vec<PreparedGuideEntry>,
}

impl GuideSectionIdentity {
    fn title(&self, plot: &Plot) -> Result<Arc<crate::Text>> {
        let source = match self {
            Self::Combined(primary) => primary.resolved_text(plot),
            Self::GroupStyle(field) | Self::Joint(field) | Self::HueColor(field) => {
                plot.resolve_label(field.as_str())
            }
            Self::Literal(LiteralDomain::BarSign) => {
                crate::data::source::copy_source_string("Change")
            }
            Self::Literal(_) | Self::SupplementalSeries => {
                crate::data::source::copy_source_string("series")
            }
            Self::ContinuousColor(_) => {
                Err(Error::Render("continuous title is producer-owned".into()))
            }
            Self::Alpha(_) => Err(Error::Render("alpha title is declaration-owned".into())),
        }?;
        crate::data::source::source_arc(crate::Text::from(source))
    }

    fn class(&self) -> u8 {
        match self {
            Self::Combined(_) | Self::GroupStyle(_) => 0,
            Self::Joint(_) => 1,
            Self::HueColor(_) => 2,
            Self::Literal(_) => 3,
            Self::SupplementalSeries => 4,
            Self::ContinuousColor(_) => 5,
            Self::Alpha(identity) => match identity.target {
                crate::alpha::AlphaTarget::Point => 6,
                crate::alpha::AlphaTarget::Line => 7,
                crate::alpha::AlphaTarget::Fill => 8,
            },
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

impl PreparedGuide {
    /// Source keys are consumed once. Only section entries and actual source
    /// occurrence records survive preparation; no complete-pair adapter remains.
    pub(crate) fn prepare(
        plot: &Plot,
        theme: &crate::theme::ResolvedTheme,
        owner: &ComponentAddress,
        keys: Vec<GuideSourceKey>,
        continuous: Option<Arc<super::super::descriptor::ContinuousDescriptor>>,
        alpha: Vec<PreparedGuideSection>,
        composition: LegendComposition,
    ) -> Result<Option<Arc<Self>>> {
        if (keys.is_empty() && continuous.is_none() && alpha.is_empty())
            || plot.legend_policy_spec() == crate::LegendPolicy::Hidden
        {
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
                    source.authored_label,
                )?;
            } else {
                separate_source(
                    &mut sections,
                    plot,
                    theme,
                    source.identity,
                    source.key,
                    record,
                    source.authored_label,
                )?;
            }
        }
        finish(composition, sections, continuous, alpha, counts, &[]).map(Some)
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
        let mut primary: Option<(PrimaryTitleIdentity, Arc<crate::Text>)> = None;
        let mut counts = GuideCounts::default();
        let mut continuous: Option<Arc<super::super::descriptor::ContinuousDescriptor>> = None;
        let mut alpha = Vec::new();
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
                if let GuideSectionBody::Alpha(_) = &section.body {
                    merge_alpha(&mut alpha, section, context)?;
                    continue;
                }
                if let GuideSectionBody::ContinuousColor(color) = &section.body {
                    if continuous
                        .as_ref()
                        .is_some_and(|existing| existing != color)
                    {
                        return Err(incompatible(context));
                    }
                    continuous = Some(Arc::clone(color));
                    continue;
                }
                if guide.composition == LegendComposition::Combined {
                    merge_combined(&mut sections, &mut primary, section, context)?;
                } else {
                    for entry in section.entries() {
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
        if continuous.is_some() && !sections.is_empty() {
            return Err(Error::Data(
                "shared Canvas guide mixes discrete and continuous contributors".into(),
            ));
        }
        if composition == LegendComposition::Combined && !sections.is_empty() {
            let (identity, title) = primary.map_or_else(
                || {
                    Ok::<_, Error>((
                        PrimaryTitleIdentity::Series,
                        crate::data::source::source_arc(crate::Text::from(
                            crate::data::source::copy_source_string("series")?,
                        ))?,
                    ))
                },
                Ok,
            )?;
            let section = sections
                .first_mut()
                .ok_or_else(|| Error::Render("empty Combined guide".into()))?;
            section.identity = GuideSectionIdentity::Combined(identity);
            section.title = title;
        }
        finish(composition, sections, continuous, alpha, counts, order).map(Some)
    }
}

fn merge_combined(
    sections: &mut Vec<Section>,
    primary: &mut Option<(PrimaryTitleIdentity, Arc<crate::Text>)>,
    section: &PreparedGuideSection,
    context: &str,
) -> Result<()> {
    let GuideSectionIdentity::Combined(identity) = &section.identity else {
        return Err(incompatible(context));
    };
    if section
        .entries()
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
    for entry in section.entries() {
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
    authored: Option<crate::Text>,
) -> Result<()> {
    let existing = sections.iter().find(|current| current.identity == section);
    let title = existing.map_or_else(
        || section.title(plot),
        |current| Ok(Arc::clone(&current.title)),
    )?;
    let label = if let Some(text) = authored {
        let resolved = identity.resolved_label(plot, source.mapping.layer_label_context)?;
        crate::data::source::source_arc(text.try_copy_with_source(resolved)?)?
    } else {
        existing
            .and_then(|current| {
                current
                    .entries
                    .iter()
                    .find(|entry| entry.identity == identity)
            })
            .map_or_else(
                || {
                    crate::data::source::source_arc(crate::Text::from(
                        identity.resolved_label(plot, source.mapping.layer_label_context)?,
                    ))
                },
                |entry| Ok(Arc::clone(&entry.display_label)),
            )?
    };
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
    title: Arc<crate::Text>,
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
    if !title.as_str().is_empty()
        && sections
            .iter()
            .any(|section| section.title.as_str() == title.as_str())
    {
        return Err(Error::Render(format!(
            "{context}: distinct section identities resolve to title {:?}",
            title.as_str()
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
    authored: Option<crate::Text>,
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
            let paint = theme.typography.guides.legend_label.get_color();
            let neutral = crate::theme::OpaqueColor::rgb(paint.red(), paint.green(), paint.blue());
            let group_key = super::projection::project(&key, &source.mapping, Projection::GroupStyle, Some(neutral))?;
            let mut group_source = source.clone();
            group_source.projection = Projection::GroupStyle;
            group_source.group_neutral_rgb = Some(neutral);
            insert_source(sections, plot, GuideSectionIdentity::GroupStyle(group.clone()),
                GuideEntryIdentity::Category { field: group, value: group_value }, group_key, group_source, None)?;
            let hue_key = super::projection::project(&key, &source.mapping, Projection::HueColor, None)?;
            source.projection = Projection::HueColor;
            return insert_source(sections, plot, GuideSectionIdentity::HueColor(hue.clone()),
                GuideEntryIdentity::Category { field: hue, value: hue_value }, hue_key, source, None);
        }
        GuideEncoding::HueOnly { field } | GuideEncoding::RowHue { field } => {
            let key = super::projection::project(&key, &source.mapping, Projection::HueColor, None)?;
            source.projection = Projection::HueColor;
            return insert_source(sections, plot, GuideSectionIdentity::HueColor(field), identity, key, source, None);
        }
        GuideEncoding::Joint { field } => GuideSectionIdentity::Joint(field),
        GuideEncoding::Literal(domain) => GuideSectionIdentity::Literal(domain),
        GuideEncoding::SupplementalLayer => GuideSectionIdentity::SupplementalSeries,
    };
    insert_source(sections, plot, section, identity, key, source, authored)
}

fn finish(
    composition: LegendComposition,
    sections: Vec<Section>,
    continuous: Option<Arc<super::super::descriptor::ContinuousDescriptor>>,
    alpha: Vec<PreparedGuideSection>,
    mut counts: GuideCounts,
    order: &[GuideEntryIdentity],
) -> Result<Arc<PreparedGuide>> {
    if sections.is_empty() && continuous.is_none() && alpha.is_empty() {
        return Err(Error::Render("a prepared guide must be nonempty".into()));
    }
    counts.sections = count(sections.len())?;
    add(&mut counts.sections, u32::from(continuous.is_some()))?;
    add(&mut counts.sections, count(alpha.len())?)?;
    let mut retained = Vec::new();
    crate::data::source::reserve_source(
        &mut retained,
        usize::try_from(counts.sections)
            .map_err(|_| Error::Render("guide section count overflow".into()))?,
    )?;
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
            body: GuideSectionBody::Symbols(shared_slice(section.entries)?),
        });
    }
    if let Some(color) = continuous {
        let display_title = Arc::clone(&color.title);
        if !display_title.as_str().is_empty()
            && retained
                .iter()
                .any(|section| section.display_title.as_str() == display_title.as_str())
        {
            return Err(Error::Render(format!(
                "prepared guide: distinct section identities resolve to title {:?}",
                display_title.as_str()
            )));
        }
        retained.push(PreparedGuideSection {
            identity: GuideSectionIdentity::ContinuousColor(super::super::identity::RawField::new(
                &color.field,
            )?),
            display_title,
            body: GuideSectionBody::ContinuousColor(color),
        });
    }
    retain_alpha_sections(&mut retained, alpha, &mut counts)?;
    order_sections(&mut retained);
    crate::data::source::source_arc(PreparedGuide {
        composition,
        counts,
        sections: shared_slice(retained)?,
    })
}

fn order_sections(sections: &mut [PreparedGuideSection]) {
    for index in 1..sections.len() {
        let mut current = index;
        while current > 0
            && sections[current - 1].identity.class() > sections[current].identity.class()
        {
            sections.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn retain_alpha_sections(
    retained: &mut Vec<PreparedGuideSection>,
    alpha: Vec<PreparedGuideSection>,
    counts: &mut GuideCounts,
) -> Result<()> {
    for section in alpha {
        if !matches!(section.identity, GuideSectionIdentity::Alpha(_))
            || section.entries().is_empty()
        {
            return Err(Error::Render(
                "alpha section has no declared members".into(),
            ));
        }
        if retained
            .iter()
            .any(|existing| existing.display_title.as_str() == section.display_title.as_str())
        {
            return Err(Error::Render(format!(
                "prepared guide: distinct section identities resolve to title {:?}",
                section.display_title.as_str()
            )));
        }
        add(&mut counts.entries, count(section.entries().len())?)?;
        for entry in section.entries() {
            add(&mut counts.points, u32::from(entry.key.point.is_some()))?;
            add(&mut counts.lines, u32::from(entry.key.line.is_some()))?;
            add(&mut counts.fills, u32::from(entry.key.fill.is_some()))?;
        }
        let GuideSectionBody::Alpha(body) = &section.body else {
            return Err(Error::Render("alpha section lost its typed body".into()));
        };
        add(
            &mut counts.source_occurrences,
            count(body.source_occurrences()?)?,
        )?;
        retained.push(section);
    }
    Ok(())
}

fn merge_alpha(
    sections: &mut Vec<PreparedGuideSection>,
    incoming: &PreparedGuideSection,
    context: &str,
) -> Result<()> {
    if let Some(existing) = sections
        .iter_mut()
        .find(|section| section.identity == incoming.identity)
    {
        if existing.display_title != incoming.display_title {
            return Err(incompatible(context));
        }
        let (GuideSectionBody::Alpha(left), GuideSectionBody::Alpha(right)) =
            (&existing.body, &incoming.body)
        else {
            return Err(Error::Render(
                "alpha section has inconsistent typed bodies".into(),
            ));
        };
        existing.body = GuideSectionBody::Alpha(left.merge(right, context)?);
    } else {
        crate::data::source::reserve_source(sections, 1)?;
        sections.push(incoming.clone());
    }
    Ok(())
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
