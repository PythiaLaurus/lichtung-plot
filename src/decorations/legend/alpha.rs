//! Actual component membership and declared alpha-only symbol explanations.

use std::sync::Arc;

use crate::alpha::{
    marks::ObservationAlpha, AlphaEncoding, AlphaSemantics, AlphaTarget, AlphaUnitKind,
    AlphaValues, ResolvedAlpha,
};
use crate::data::source::{
    collect_source, copy_source_string, reserve_source, reserve_source_string, source_arc,
};
use crate::layout::ComponentAddress;
use crate::render::backend::{FillStyle, LineStyle, PointStyle};
use crate::theme::{Color, OpaqueColor, ResolvedTheme};
use crate::{Error, Plot, PlotKind, Result};

use super::discrete::{
    shared_slice, GuideSectionBody, GuideSectionIdentity, PreparedGuideEntry, PreparedGuideSection,
};
use super::identity::{shared_text, RawField};
use super::source::{
    AlphaExplanation, AlphaExplanationMask, GuideExplanation, GuideExplanationMask, RgbExplanation,
    StyleExplanation,
};
use super::{GuideEntryIdentity, ResolvedGuideKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlphaSectionIdentity {
    pub(crate) field: RawField,
    pub(crate) semantics: AlphaSemantics,
    pub(crate) target: AlphaTarget,
    pub(crate) unit: AlphaUnitKind,
}

/// Roles establish a producer's deterministic component order, independently of
/// row order, alpha magnitude or category labels. All active glyphs are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AlphaGlyphRole {
    Primary,
    Outlier,
    RawOverlay,
    MandatoryStrip,
    LeftEndpoint,
    RightEndpoint,
    Line,
    Summary,
    Median,
    Fill,
    Highlight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AlphaUnit {
    Observation { slot: usize, source_row: usize },
    Keyed(Arc<[(String, String)]>),
    Bin([u64; 2]),
    Single,
}

impl AlphaUnit {
    pub(crate) const fn observation(alpha: ObservationAlpha) -> Self {
        Self::Observation {
            slot: alpha.slot,
            source_row: alpha.source_row,
        }
    }

    pub(crate) fn bin(bounds: (f64, f64)) -> Self {
        let key = |value: f64| if value == 0.0 { 0 } else { value.to_bits() };
        Self::Bin([key(bounds.0), key(bounds.1)])
    }

    pub(crate) fn keyed(keys: &[(&str, &str)]) -> Result<Self> {
        let mut keys =
            collect_source(keys.iter().map(|&(field, value)| {
                Ok((copy_source_string(field)?, copy_source_string(value)?))
            }))?;
        crate::data::source::sort_source_by(&mut keys, |left, right| left.0.cmp(&right.0));
        keys.dedup();
        Ok(Self::Keyed(shared_slice(keys)?))
    }

    pub(crate) fn series(
        plot: &Plot,
        identity: Option<&crate::data::CollectivePair>,
    ) -> Result<Self> {
        let group = plot
            .group_name()
            .zip(identity.and_then(|pair| pair.group.as_deref()));
        let hue = plot
            .hue_name()
            .zip(identity.and_then(|pair| pair.hue.as_deref()));
        match (group, hue) {
            (Some(group), Some(hue)) => Self::keyed(&[group, hue]),
            (Some(group), None) => Self::keyed(&[group]),
            (None, Some(hue)) => Self::keyed(&[hue]),
            (None, None) => Self::keyed(&[]),
        }
    }

    pub(crate) fn aggregate(plot: &Plot, category: &str, series: Option<&str>) -> Result<Self> {
        let field = plot
            .x_col()
            .and_then(|key| plot.data_column(key))
            .ok_or_else(|| Error::Render("alpha aggregate guide has no category field".into()))?;
        let category = (field.name(), category);
        if let Some(series) = plot.group_name().zip(series) {
            Self::keyed(&[category, series])
        } else {
            Self::keyed(&[category])
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlphaActualMember {
    pub(crate) unit: AlphaUnit,
    pub(crate) value: ResolvedAlpha,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlphaGlyphSource {
    role: AlphaGlyphRole,
    pub(crate) key: ResolvedGuideKey,
    pub(crate) members: Arc<[AlphaActualMember]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlphaGuideSource {
    pub(crate) producer: PlotKind,
    pub(crate) owner: ComponentAddress,
    pub(crate) scope: Option<Arc<str>>,
    pub(crate) input: AlphaEncoding,
    pub(crate) neutral: OpaqueColor,
    pub(crate) glyphs: Arc<[AlphaGlyphSource]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedAlphaSection {
    pub(crate) declaration: AlphaValues,
    pub(crate) entries: Arc<[PreparedGuideEntry]>,
    pub(crate) sources: Arc<[AlphaGuideSource]>,
}

impl PreparedAlphaSection {
    pub(crate) fn source_occurrences(&self) -> Result<usize> {
        self.sources
            .iter()
            .flat_map(|source| source.glyphs.iter())
            .try_fold(0_usize, |count, glyph| {
                count
                    .checked_add(glyph.members.len())
                    .ok_or_else(|| Error::Render("alpha guide membership count overflow".into()))
            })
    }

    pub(super) fn merge(&self, other: &Self, context: &str) -> Result<Arc<Self>> {
        if !self.declaration.same_declaration(&other.declaration) || !compatible_glyphs(self, other)
        {
            return Err(Error::Render(format!("{context} requires compatible semantic titles, labels, and styles: alpha declarations and complete projected glyphs must match")));
        }
        let mut entries = collect_source(self.entries.iter().cloned().map(Ok))?;
        for entry in other.entries.iter() {
            if let Some(existing) = entries
                .iter()
                .find(|existing| existing.identity == entry.identity)
            {
                if existing.display_label != entry.display_label || existing.key != entry.key {
                    return Err(Error::Render(format!("{context} requires compatible semantic titles, labels, and styles: alpha member labels and glyphs must match")));
                }
            } else {
                if entries
                    .iter()
                    .any(|existing| existing.display_label == entry.display_label)
                {
                    return Err(Error::Render(format!("{context} has distinct declared alpha members resolving to display label {:?}", entry.display_label)));
                }
                reserve_source(&mut entries, 1)?;
                entries.push(entry.clone());
            }
        }
        crate::data::source::sort_source_by(&mut entries, |left, right| {
            ordinal(left).cmp(&ordinal(right))
        });
        let mut sources = collect_source(self.sources.iter().cloned().map(Ok))?;
        reserve_source(&mut sources, other.sources.len())?;
        sources.extend_from_slice(&other.sources);
        source_arc(Self {
            declaration: self.declaration.clone(),
            entries: shared_slice(entries)?,
            sources: shared_slice(sources)?,
        })
    }
}

fn ordinal(entry: &PreparedGuideEntry) -> u32 {
    let GuideEntryIdentity::AlphaMember { ordinal } = entry.identity else {
        unreachable!("alpha section retains alpha entry identities")
    };
    ordinal
}

fn compatible_glyphs(left: &PreparedAlphaSection, right: &PreparedAlphaSection) -> bool {
    fn glyphs(section: &PreparedAlphaSection) -> Option<&[AlphaGlyphSource]> {
        section.sources.first().map(|source| source.glyphs.as_ref())
    }
    let (Some(left), Some(right)) = (glyphs(left), glyphs(right)) else {
        return false;
    };
    left.iter()
        .all(|glyph| right.iter().any(|other| other.key == glyph.key))
        && right
            .iter()
            .all(|glyph| left.iter().any(|other| other.key == glyph.key))
}

struct PendingGlyph {
    role: AlphaGlyphRole,
    key: ResolvedGuideKey,
    members: Vec<AlphaActualMember>,
}

struct PendingTarget {
    unit: AlphaUnitKind,
    glyphs: Vec<PendingGlyph>,
}

pub(crate) struct AlphaGuideBuilder<'a> {
    plot: &'a Plot,
    owner: &'a ComponentAddress,
    neutral: OpaqueColor,
    targets: [Option<PendingTarget>; 3],
}

impl<'a> AlphaGuideBuilder<'a> {
    pub(crate) fn new(plot: &'a Plot, theme: &ResolvedTheme, owner: &'a ComponentAddress) -> Self {
        let paint = theme.typography.guides.legend_label.color();
        Self {
            plot,
            owner,
            neutral: OpaqueColor::rgb(paint.red(), paint.green(), paint.blue()),
            targets: [None, None, None],
        }
    }

    pub(crate) fn enabled(&self, target: AlphaTarget) -> bool {
        self.plot
            .alpha_inputs()
            .get(target)
            .values()
            .is_some_and(|values| values.semantics() != AlphaSemantics::Undeclared)
    }

    pub(crate) fn any_enabled(&self) -> bool {
        [AlphaTarget::Point, AlphaTarget::Line, AlphaTarget::Fill]
            .into_iter()
            .any(|target| self.enabled(target))
    }

    pub(crate) fn point(
        &mut self,
        role: AlphaGlyphRole,
        alpha: ObservationAlpha,
        mut style: PointStyle,
        mapped_shape: bool,
    ) -> Result<()> {
        if !self.enabled(AlphaTarget::Point) {
            return Ok(());
        }
        style.color = Color::from(self.neutral);
        if mapped_shape {
            style.shape = crate::PointShape::Circle;
        }
        let key = ResolvedGuideKey {
            explanation: GuideExplanation::Projected(GuideExplanationMask {
                alpha: AlphaExplanationMask {
                    point_fill: Some(AlphaExplanation::Explained),
                    point_outline: style.outline.as_ref().map(|_| AlphaExplanation::Constant),
                    ..AlphaExplanationMask::default()
                },
                point_fill: Some(RgbExplanation::NeutralizedColor),
                point_outline: style.outline.as_ref().map(|_| RgbExplanation::Constant),
                point_shape: Some(if mapped_shape {
                    StyleExplanation::NeutralizedGroup
                } else {
                    StyleExplanation::Constant
                }),
                line_stroke: None,
                fill_paint: None,
                fill_outline: None,
                line_pattern: None,
            }),
            point: Some(style),
            ..ResolvedGuideKey::default()
        };
        self.push(
            AlphaTarget::Point,
            AlphaUnitKind::Observation,
            role,
            AlphaUnit::Observation {
                slot: alpha.slot,
                source_row: alpha.source_row,
            },
            alpha.value,
            key,
        )
    }

    pub(crate) fn line(
        &mut self,
        role: AlphaGlyphRole,
        unit: AlphaUnitKind,
        identity: AlphaUnit,
        alpha: Option<ResolvedAlpha>,
        mut style: LineStyle,
        mapped_pattern: bool,
    ) -> Result<()> {
        if !self.enabled(AlphaTarget::Line) {
            return Ok(());
        }
        style.color = Color::from(self.neutral);
        if mapped_pattern {
            style.pattern = crate::StrokePattern::Solid;
        }
        let key = ResolvedGuideKey {
            explanation: GuideExplanation::Projected(GuideExplanationMask {
                alpha: AlphaExplanationMask {
                    line_stroke: Some(AlphaExplanation::Explained),
                    ..AlphaExplanationMask::default()
                },
                line_stroke: Some(RgbExplanation::NeutralizedColor),
                line_pattern: Some(if mapped_pattern {
                    StyleExplanation::NeutralizedGroup
                } else {
                    StyleExplanation::Constant
                }),
                point_fill: None,
                point_outline: None,
                fill_paint: None,
                fill_outline: None,
                point_shape: None,
            }),
            line: Some(style),
            ..ResolvedGuideKey::default()
        };
        self.push(AlphaTarget::Line, unit, role, identity, alpha, key)
    }

    pub(crate) fn fill(
        &mut self,
        role: AlphaGlyphRole,
        unit: AlphaUnitKind,
        identity: AlphaUnit,
        alpha: Option<ResolvedAlpha>,
        mut style: FillStyle,
        mapped_outline: bool,
    ) -> Result<()> {
        if !self.enabled(AlphaTarget::Fill) {
            return Ok(());
        }
        style.color = Color::from(self.neutral);
        if mapped_outline {
            style.stroke = Color::from(self.neutral).with_alpha(style.stroke.alpha());
        }
        let outline = style.stroke_width > 0.0;
        let key = ResolvedGuideKey {
            explanation: GuideExplanation::Projected(GuideExplanationMask {
                alpha: AlphaExplanationMask {
                    fill_paint: Some(AlphaExplanation::Explained),
                    fill_outline: outline.then_some(AlphaExplanation::Constant),
                    ..AlphaExplanationMask::default()
                },
                fill_paint: Some(RgbExplanation::NeutralizedColor),
                fill_outline: outline.then_some(if mapped_outline {
                    RgbExplanation::NeutralizedColor
                } else {
                    RgbExplanation::Constant
                }),
                point_fill: None,
                point_outline: None,
                line_stroke: None,
                point_shape: None,
                line_pattern: None,
            }),
            fill: Some(style),
            ..ResolvedGuideKey::default()
        };
        self.push(AlphaTarget::Fill, unit, role, identity, alpha, key)
    }

    fn push(
        &mut self,
        target: AlphaTarget,
        unit: AlphaUnitKind,
        role: AlphaGlyphRole,
        identity: AlphaUnit,
        alpha: Option<ResolvedAlpha>,
        key: ResolvedGuideKey,
    ) -> Result<()> {
        let value = alpha.ok_or_else(|| {
            Error::Render("actual alpha guide component has no retained member".into())
        })?;
        let pending = self.targets[target.index()].get_or_insert_with(|| PendingTarget {
            unit,
            glyphs: Vec::new(),
        });
        if pending.unit != unit {
            return Err(Error::Render(
                "one alpha target has conflicting scientific units".into(),
            ));
        }
        let index = if let Some(index) = pending
            .glyphs
            .iter()
            .position(|glyph| glyph.role == role && glyph.key == key)
        {
            index
        } else {
            reserve_source(&mut pending.glyphs, 1)?;
            pending.glyphs.push(PendingGlyph {
                role,
                key,
                members: Vec::new(),
            });
            pending.glyphs.len() - 1
        };
        let members = &mut pending.glyphs[index].members;
        if members.len() == members.capacity() {
            let additional = members.capacity().max(1);
            reserve_source(members, additional)?;
        }
        members.push(AlphaActualMember {
            unit: identity,
            value,
        });
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<Vec<PreparedGuideSection>> {
        let mut sections = Vec::new();
        reserve_source(&mut sections, self.targets.iter().flatten().count())?;
        for (target, pending) in [AlphaTarget::Point, AlphaTarget::Line, AlphaTarget::Fill]
            .into_iter()
            .zip(self.targets)
        {
            let Some(mut pending) = pending else { continue };
            crate::data::source::sort_source_by(&mut pending.glyphs, |left, right| {
                left.role.cmp(&right.role)
            });
            let input = self.plot.alpha_inputs().get(target);
            let values = input.values().expect("enabled alpha retains owned values");
            let examples = values.guide_examples(
                pending
                    .glyphs
                    .iter()
                    .flat_map(|glyph| glyph.members.iter().map(|member| member.value)),
            )?;
            let prototype = &pending
                .glyphs
                .first()
                .ok_or_else(|| Error::Render("alpha section has no actual component glyph".into()))?
                .key;
            let entries = collect_source(examples.into_iter().map(|example| {
                let label = if values.semantics() == AlphaSemantics::Categorical {
                    self.plot.resolve_label(example.label)?
                } else {
                    copy_source_string(example.label)?
                };
                Ok(PreparedGuideEntry {
                    identity: GuideEntryIdentity::AlphaMember {
                        ordinal: u32::try_from(example.ordinal)
                            .map_err(|_| Error::Render("alpha guide ordinal overflow".into()))?,
                    },
                    display_label: shared_text(&label)?,
                    key: sample_key(prototype, target, example.alpha),
                    sources: shared_slice(Vec::new())?,
                })
            }))?;
            for (index, entry) in entries.iter().enumerate() {
                if entries[..index]
                    .iter()
                    .any(|other| other.display_label == entry.display_label)
                {
                    return Err(Error::Render(format!(
                        "alpha guide has distinct declared members resolving to display label {:?}",
                        entry.display_label
                    )));
                }
            }
            let glyphs = collect_source(pending.glyphs.into_iter().map(|glyph| {
                Ok(AlphaGlyphSource {
                    role: glyph.role,
                    key: glyph.key,
                    members: shared_slice(glyph.members)?,
                })
            }))?;
            let source = AlphaGuideSource {
                producer: self.plot.kind(),
                owner: self.owner.clone(),
                scope: self.plot.axis_scope().map(shared_text).transpose()?,
                input: input.clone(),
                neutral: self.neutral,
                glyphs: shared_slice(glyphs)?,
            };
            let identity = AlphaSectionIdentity {
                field: RawField::new(values.name())?,
                semantics: values.semantics(),
                target,
                unit: pending.unit,
            };
            let title = title(self.plot, values.name(), target, pending.unit)?;
            sections.push(PreparedGuideSection {
                identity: GuideSectionIdentity::Alpha(identity),
                display_title: shared_text(&title)?,
                body: GuideSectionBody::Alpha(source_arc(PreparedAlphaSection {
                    declaration: values.clone(),
                    entries: shared_slice(entries)?,
                    sources: shared_slice(collect_source(std::iter::once(Ok(source)))?)?,
                })?),
            });
        }
        Ok(sections)
    }
}

fn sample_key(prototype: &ResolvedGuideKey, target: AlphaTarget, alpha: u8) -> ResolvedGuideKey {
    let mut key = prototype.clone();
    match target {
        AlphaTarget::Point => {
            let point = key
                .point
                .as_mut()
                .expect("point alpha has its actual glyph");
            point.color = point.color.with_alpha(alpha);
        }
        AlphaTarget::Line => {
            let line = key.line.as_mut().expect("line alpha has its actual glyph");
            line.color = line.color.with_alpha(alpha);
        }
        AlphaTarget::Fill => {
            let fill = key.fill.as_mut().expect("fill alpha has its actual glyph");
            fill.color = fill.color.with_alpha(alpha);
        }
    }
    key
}

fn title(plot: &Plot, source: &str, target: AlphaTarget, unit: AlphaUnitKind) -> Result<String> {
    let target = match target {
        AlphaTarget::Point => "point",
        AlphaTarget::Line => "line",
        AlphaTarget::Fill => "fill",
    };
    let unit = match unit {
        AlphaUnitKind::Observation => "observation",
        AlphaUnitKind::Series => "series",
        AlphaUnitKind::Aggregate => "aggregate",
        AlphaUnitKind::Bin => "bin",
        AlphaUnitKind::Single => "single",
    };
    let mut title = plot.resolve_label(source)?;
    let extra = unit
        .len()
        .checked_add(target.len())
        .and_then(|count| count.checked_add(" (  alpha)".len()))
        .ok_or_else(|| Error::Render("alpha guide title length overflow".into()))?;
    reserve_source_string(&mut title, extra)?;
    title.push_str(" (");
    title.push_str(unit);
    title.push(' ');
    title.push_str(target);
    title.push_str(" alpha)");
    Ok(title)
}
