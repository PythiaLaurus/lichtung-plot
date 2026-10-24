//! One canonical measurement and finite allocation for all typed guide sections.

mod cache;
mod flow;
#[cfg(test)]
mod tests;
pub(crate) use cache::GuideMeasurements;

use std::cmp::Ordering;
use std::sync::Arc;

use super::continuous::ColorContent;
use super::discrete::{GuideSectionBody, PreparedGuide, PreparedGuideSection};
use super::{Legend, LegendLayout, LegendMetrics, SectionLayout};
use crate::layout::fit::{self, FitKind, FitOutcome, FitScope, PlanError};
use crate::layout::kernel::{allocate_tracks, LayoutScalar, SpatialRounding, TrackSpec};
use crate::layout::{AddressRole, ResolvedTextPlan, TextBlock};
use crate::theme::spacing::ExactLength;
use crate::theme::ResolvedTheme;
use crate::Error;

#[expect(
    clippy::cast_precision_loss,
    reason = "assigned plans admit exact pixel projections before retaining canonical quanta"
)]
pub(super) fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

fn length(quanta: i64) -> Result<ExactLength, Error> {
    ExactLength::quanta(LayoutScalar::from_quanta(quanta))
}

fn hard(value: &ExactLength) -> Result<i64, Error> {
    Ok(value.round(SpatialRounding::Ceil)?.quanta())
}

fn count_gaps(count: usize) -> Result<u32, Error> {
    let count = count
        .checked_sub(1)
        .ok_or_else(|| Error::Render("a measured guide section must be nonempty".into()))?;
    u32::try_from(count).map_err(|_| Error::Render("guide gap count overflow".into()))
}

fn add(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_add(right)
        .ok_or_else(|| Error::Render("guide geometry overflowed layout quanta".into()))
}

struct EntryIntrinsics {
    minimum: ExactLength,
    preferred: i64,
    key_origin: i64,
    chrome: ExactLength,
    ascent: i64,
    descent: i64,
}

struct SectionIntrinsics {
    title_minimum: i64,
    title_preferred: i64,
    entries: Vec<EntryIntrinsics>,
    preferred_column: i64,
    color_width: Option<ExactLength>,
}

fn guide_intrinsics(
    descriptor: &PreparedGuide,
    theme: &ResolvedTheme,
    metrics: &LegendMetrics,
) -> Result<Vec<SectionIntrinsics>, Error> {
    let mut sections = Vec::new();
    fit::reserve(
        &mut sections,
        descriptor.sections.len(),
        "guide section intrinsics",
    )?;
    for section in descriptor.sections.iter() {
        let (minimum, natural) = fit::intrinsic_widths(
            theme,
            section.display_title.as_str(),
            section
                .display_title
                .appearance(theme.guide_text_appearance(crate::GuideTextRole::Title)),
            "legend title intrinsic",
        )?;
        let mut entries = Vec::new();
        fit::reserve(
            &mut entries,
            section.entries().len(),
            "guide entry intrinsics",
        )?;
        let mut preferred_column = 0;
        let eligible = crate::data::source::collect_source((0..section.entries().len()).map(Ok))?;
        for (index, entry) in section.entries().iter().enumerate() {
            let appearance = entry
                .display_label
                .appearance(theme.guide_text_appearance(crate::GuideTextRole::Entry));
            let (minimum, preferred) = if appearance.includes(index, &eligible) {
                fit::intrinsic_widths(
                    theme,
                    entry.display_label.as_str(),
                    appearance,
                    "legend entry intrinsic",
                )?
            } else {
                (0.0, 0.0)
            };
            let ink = entry.key.glyph_ink(metrics)?;
            let left = LayoutScalar::from_css_pixels(ink.left, SpatialRounding::Floor)?.quanta();
            let right = LayoutScalar::from_css_pixels(ink.right, SpatialRounding::Ceil)?.quanta();
            let key_width = right
                .checked_sub(left)
                .ok_or_else(|| Error::Render("guide key span overflow".into()))?;
            let chrome = length(key_width)?.add_length(&metrics.exact.key_label_gap);
            let preferred = hard(&chrome.add_pixels(minimum.max(preferred))?)?;
            preferred_column = preferred_column.max(preferred);
            entries.push(EntryIntrinsics {
                minimum: chrome.add_pixels(minimum)?,
                preferred,
                key_origin: left
                    .checked_neg()
                    .ok_or_else(|| Error::Render("guide key origin overflow".into()))?,
                chrome,
                ascent: LayoutScalar::from_css_pixels(ink.top.max(0.0), SpatialRounding::Ceil)?
                    .quanta(),
                descent: LayoutScalar::from_css_pixels(
                    (-ink.bottom).max(0.0),
                    SpatialRounding::Ceil,
                )?
                .quanta(),
            });
        }
        let color_width = match &section.body {
            GuideSectionBody::ContinuousColor(color) => {
                Some(super::continuous::intrinsic_width(color, theme, metrics)?)
            }
            GuideSectionBody::Symbols(_) | GuideSectionBody::Alpha(_) => None,
        };
        sections.push(SectionIntrinsics {
            title_minimum: LayoutScalar::from_css_pixels(minimum, SpatialRounding::Ceil)?.quanta(),
            title_preferred: LayoutScalar::from_css_pixels(natural, SpatialRounding::Ceil)?
                .quanta(),
            entries,
            preferred_column,
            color_width,
        });
    }
    Ok(sections)
}

/// Advance/leading and actual glyph ink share the first baseline. Ink outside
/// design metrics contributes to occupancy before any row/capacity calculation.
fn text_extents(plan: &ResolvedTextPlan) -> Result<(i64, i64), Error> {
    let placement = crate::layout::text::placement::resolve(
        crate::render::backend::PlannedTextPlacement::baseline(
            crate::render::backend::Point2D::new(0.0, 0.0),
            crate::render::backend::TextAnchor::Start,
            0.0,
        ),
        plan,
        None,
    )?;
    let bounds = crate::layout::text::placement::footprint(placement, plan)?;
    Ok((
        crate::layout::kernel::ceil_spatial_quanta(bounds.y1.max(0.0))?,
        crate::layout::kernel::ceil_spatial_quanta((-bounds.y0).max(0.0))?,
    ))
}

struct MeasuredEntry {
    width: ExactLength,
    key_origin: i64,
    chrome: ExactLength,
    ascent: i64,
    descent: i64,
    block: TextBlock,
}

impl MeasuredEntry {
    fn height(&self) -> Result<i64, Error> {
        add(self.ascent, self.descent)
    }
}

struct MeasuredSection {
    title: TextBlock,
    title_gap: ExactLength,
    title_ascent: i64,
    title_width: ExactLength,
    title_occupancy: ExactLength,
    entries: Vec<MeasuredEntry>,
    color: Option<ColorSection>,
}

struct ColorSection {
    descriptor: Arc<super::descriptor::ContinuousDescriptor>,
    content: ColorContent,
    compact: bool,
}

impl MeasuredSection {
    fn assigned_color(
        &mut self,
        assignment: &Assignment,
        capacity: Option<i64>,
        metrics: &LegendMetrics,
        compact_changes: &mut bool,
        scope: &FitScope,
    ) -> Result<Option<super::continuous::MeasuredColorBody>, PlanError> {
        let Some(color) = self.color.take() else {
            return Ok(None);
        };
        let body_capacity = capacity
            .map(|capacity| {
                Ok::<_, Error>(
                    length(capacity)?
                        .subtract(&self.title_occupancy)
                        .unwrap_or(length(0)?),
                )
            })
            .transpose()?;
        color
            .content
            .assign(
                &color.descriptor,
                assignment.inner_width,
                body_capacity.as_ref(),
                color.compact,
                metrics,
                compact_changes,
                scope,
            )
            .map(Some)
    }

    fn track(&self, metrics: &LegendMetrics) -> Result<TrackSpec, Error> {
        if let Some(color) = &self.color {
            return color.content.track(&self.title_occupancy, metrics);
        }
        let mut maximum = 0;
        let mut preferred = self
            .title_occupancy
            .add_length(&metrics.exact.row_gap.times(count_gaps(self.entries.len())?));
        for entry in &self.entries {
            let height = entry.height()?;
            maximum = maximum.max(height);
            preferred = preferred.add_length(&length(height)?);
        }
        let preferred = hard(&preferred)?;
        Ok(TrackSpec {
            min: hard(&self.title_occupancy.add_length(&length(maximum)?))?,
            preferred,
            max: Some(preferred),
            flexible: true,
        })
    }
}

struct Assignment {
    width: i64,
    height: Option<i64>,
    inner_width: i64,
    inner_height: Option<i64>,
}

impl Assignment {
    fn frame_size(
        &self,
        content: [&ExactLength; 2],
        metrics: &LegendMetrics,
        minimum: f64,
        scope: &FitScope,
    ) -> Result<(i64, i64), PlanError> {
        // Inner capacity is integral quanta. Preserve its complete requirement
        // before adding exact outer edges, including cancelling fractions.
        let width = metrics
            .outer_width(&length(hard(content[0])?)?)?
            .max(minimum);
        let height = metrics
            .outer_height(&length(hard(content[1])?)?, true)?
            .max(minimum);
        if width > pixels(self.width) || self.height.is_some_and(|limit| height > pixels(limit)) {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "complete guide frame exceeds its assigned track",
                )
                .into());
        }
        Ok((
            LayoutScalar::from_css_pixels(width, SpatialRounding::Ceil)?.quanta(),
            LayoutScalar::from_css_pixels(height, SpatialRounding::Ceil)?.quanta(),
        ))
    }

    fn new(
        available: (f64, Option<f64>),
        metrics: &LegendMetrics,
        scope: &FitScope,
    ) -> Result<Self, PlanError> {
        let spatial = |value| {
            let value = LayoutScalar::from_css_pixels(value, SpatialRounding::Floor)?.quanta();
            if value.unsigned_abs() > 1_u64 << 53 {
                return Err(Error::Render(
                    "guide assignment cannot retain exact CSS quanta".into(),
                ));
            }
            Ok(value)
        };
        let width = spatial(available.0)?;
        let height = available.1.map(spatial).transpose()?;
        if width <= 0 || height.is_some_and(|height| height <= 0) {
            return Err(scope
                .failure(FitKind::LegendOrGuide, "guide has no positive allocation")
                .into());
        }
        let inner_width = length(width)?
            .subtract(&metrics.edge_reserve(false))
            .map(|value| value.round(SpatialRounding::Floor))
            .transpose()?
            .map_or(0, LayoutScalar::quanta);
        let inner_height = height
            .map(|height| {
                Ok::<_, Error>(
                    length(height)?
                        .subtract(&metrics.edge_reserve(true))
                        .map(|value| value.round(SpatialRounding::Floor))
                        .transpose()?
                        .map_or(0, LayoutScalar::quanta),
                )
            })
            .transpose()?;
        if inner_width <= 0 || inner_height.is_some_and(|height| height <= 0) {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "guide has no positive content extent",
                )
                .into());
        }
        Ok(Self {
            width,
            height,
            inner_width,
            inner_height,
        })
    }

    fn compact_cap(&self, metrics: &LegendMetrics) -> Result<i64, Error> {
        let gap = metrics
            .exact
            .column_gap
            .round(SpatialRounding::NearestEven)?
            .quanta();
        Ok(self
            .inner_width
            .checked_sub(gap)
            .filter(|width| *width > 0)
            .map_or(0, |width| width / 2))
    }
}

fn entry_allocation(
    entry: &EntryIntrinsics,
    section: &SectionIntrinsics,
    cap: i64,
    horizontal: bool,
) -> i64 {
    (if horizontal {
        entry.preferred
    } else {
        section.preferred_column
    })
    .min(cap)
}

fn measure_title(
    text: &crate::Text,
    width: i64,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<(ResolvedTextPlan, ExactLength), PlanError> {
    let appearance = text.appearance(theme.guide_text_appearance(crate::GuideTextRole::Title));
    let plan = fit::measured(fit::single_text(
        theme,
        text.as_str(),
        appearance,
        pixels(width),
        text.resolved_alignment(crate::TextAlignment::Start).into(),
        "legend section title",
        &scope.child(AddressRole::LegendTitle, 0)?,
    ))?;
    let gap = theme
        .legend
        .lengths(
            theme.typography.guides.legend_label.get_font_size(),
            appearance.theme.get_font_size(),
        )?
        .title_gap;
    Ok((plan, gap))
}

#[expect(
    clippy::too_many_arguments,
    reason = "section text measurement consumes the owner's fixed width variant and resolved role metrics"
)]
fn measure_section(
    section: &PreparedGuideSection,
    intrinsic: &SectionIntrinsics,
    cap: i64,
    horizontal: bool,
    theme: &ResolvedTheme,
    metrics: &LegendMetrics,
    scope: &FitScope,
    inner_width: i64,
    compact: bool,
) -> Result<MeasuredSection, PlanError> {
    if !matches!(section.body, GuideSectionBody::ContinuousColor(_)) {
        count_gaps(section.entries().len())?;
    }
    if intrinsic.title_minimum > inner_width {
        return Err(scope
            .child(AddressRole::LegendTitle, 0)?
            .failure(
                FitKind::Text,
                "guide title allocation is below its hard unbroken requirement",
            )
            .into());
    }
    // Check every complete hard width before shaping assigned label plans.
    for (index, entry) in intrinsic.entries.iter().enumerate() {
        let allocated = length(entry_allocation(entry, intrinsic, cap, horizontal))?;
        if entry.minimum.compare(&allocated) == Ordering::Greater {
            return Err(scope
                .child(AddressRole::LegendEntry, index)?
                .failure(
                    FitKind::Text,
                    "guide entry allocation is below its hard unbroken requirement",
                )
                .into());
        }
    }
    let (title, title_gap) = measure_title(&section.display_title, inner_width, theme, scope)?;
    let (title_ascent, title_descent) = text_extents(&title)?;
    let title_height = add(title_ascent, title_descent)?;
    let title_occupancy = length(title_height)?.add_length(&title_gap);
    let eligible = crate::data::source::collect_source((0..section.entries().len()).map(Ok))?;
    let entries = fit::collect(
        "measured guide entries",
        section
            .entries()
            .iter()
            .zip(&intrinsic.entries)
            .enumerate()
            .map(|(index, (entry, width))| {
                let allocated = length(entry_allocation(width, intrinsic, cap, horizontal))?;
                let label_scope = scope
                    .child(AddressRole::LegendEntry, index)?
                    .child(AddressRole::LegendLabel, 0)?;
                let text_capacity = allocated.subtract(&width.chrome).ok_or_else(|| {
                    label_scope.failure(
                        FitKind::Text,
                        "guide entry has no text capacity after exact chrome",
                    )
                })?;
                let appearance = entry
                    .display_label
                    .appearance(theme.guide_text_appearance(crate::GuideTextRole::Entry));
                let alignment = entry
                    .display_label
                    .resolved_alignment(crate::TextAlignment::Start)
                    .into();
                let plan = fit::measured(crate::layout::text::request::measure_occurrence(
                    theme,
                    entry.display_label.as_str(),
                    appearance,
                    text_capacity.round(SpatialRounding::Floor)?.quanta(),
                    alignment,
                    "legend section entry",
                    &label_scope,
                    (index, &eligible),
                ))?;
                let (ascent, descent) = text_extents(&plan)?;
                Ok(MeasuredEntry {
                    width: allocated,
                    key_origin: width.key_origin,
                    chrome: width.chrome.clone(),
                    ascent: ascent.max(width.ascent),
                    descent: descent.max(width.descent),
                    block: TextBlock::from_resolved(plan),
                })
            }),
    )?;
    Ok(MeasuredSection {
        title_gap,
        title_width: ExactLength::pixels(title.occupied_size().0.max(title.fit_width()))?,
        title: TextBlock::from_resolved(title),
        title_ascent,
        title_occupancy,
        entries,
        color: match &section.body {
            GuideSectionBody::Symbols(_) | GuideSectionBody::Alpha(_) => None,
            GuideSectionBody::ContinuousColor(descriptor) => Some(ColorSection {
                descriptor: Arc::clone(descriptor),
                content: ColorContent::prepare(
                    descriptor,
                    inner_width,
                    horizontal,
                    theme,
                    metrics,
                    scope,
                )?,
                compact,
            }),
        },
    })
}

/// The only finite vertical allocation. Maxima pin excess to unused slack;
/// a failed section never receives another section's unused capacity.
fn section_capacities(
    sections: &[MeasuredSection],
    height: i64,
    metrics: &LegendMetrics,
    scope: &FitScope,
) -> Result<Vec<i64>, PlanError> {
    let mut tracks = Vec::new();
    fit::reserve(&mut tracks, sections.len(), "guide section capacities")?;
    let mut minimum = 0_i128;
    for section in sections {
        let track = section.track(metrics)?;
        minimum = minimum
            .checked_add(i128::from(track.min))
            .ok_or_else(|| Error::Render("guide section minima overflow".into()))?;
        tracks.push(track);
    }
    let mut gaps = length(0)?;
    for section in sections.iter().skip(1) {
        gaps = gaps
            .add_length(&section.title_gap)
            .add_length(&metrics.exact.row_gap);
    }
    let reserve = hard(&gaps)?;
    let available = height
        .checked_sub(reserve)
        .filter(|value| *value >= 0)
        .ok_or_else(|| {
            scope.failure(
                FitKind::LegendOrGuide,
                "guide section gaps exceed the assigned height",
            )
        })?;
    if i128::from(available) < minimum {
        return Err(scope
            .failure(
                FitKind::LegendOrGuide,
                format!("guide cannot preserve every section's title and body minimum: {available} available quanta, {minimum} required"),
            )
            .into());
    }
    Ok(fit::measured(allocate_tracks(&tracks, available, [0, 0], 0, false, scope))?.sizes)
}

fn stack_sections(
    measured: Vec<MeasuredSection>,
    assignment: &Assignment,
    horizontal: bool,
    metrics: &LegendMetrics,
    frame_minimum: f64,
    scope: &FitScope,
) -> Result<LegendLayout, PlanError> {
    let capacities = if horizontal {
        None
    } else {
        assignment
            .inner_height
            .map(|height| section_capacities(&measured, height, metrics, scope))
            .transpose()?
    };
    let mut sections = Vec::new();
    fit::reserve(&mut sections, measured.len(), "measured guide sections")?;
    let mut top = length(0)?;
    let mut width = length(0)?;
    let mut compact_changes = false;
    for (ordinal, mut section) in measured.into_iter().enumerate() {
        if ordinal > 0 {
            top = top
                .add_length(&section.title_gap)
                .add_length(&metrics.exact.row_gap);
        }
        let section_scope = scope.child(AddressRole::LegendSection, ordinal)?;
        let capacity = capacities.as_ref().map(|values| values[ordinal]);
        let (mut flow, color) = if let Some(body) = section.assigned_color(
            assignment,
            capacity,
            metrics,
            &mut compact_changes,
            &section_scope,
        )? {
            let flow = flow::SectionFlow {
                width: body.width.times(1),
                height: body.height.times(1),
                columns: 0,
                slots: Vec::new(),
            };
            (flow, Some(body))
        } else {
            (
                flow::assigned_entries(
                    &section,
                    assignment.inner_width,
                    capacity,
                    &top,
                    metrics,
                    horizontal,
                    &section_scope,
                )?,
                None,
            )
        };
        let occupied = section.title_occupancy.add_length(&flow.height);
        let section_width = if section.title_width.compare(&flow.width) == Ordering::Greater {
            section.title_width
        } else {
            flow.width
        };
        let region = crate::layout::PanelBox::new(
            0.0,
            pixels(hard(&section_width)?),
            -pixels(hard(&top.add_length(&occupied))?),
            -pixels(hard(&top)?),
        );
        select_entry_visibility(&mut flow.slots, region, horizontal, &section_scope)?;
        if width.compare(&section_width) == Ordering::Less {
            width = section_width.clone();
        }
        sections.push(SectionLayout {
            title: section.title,
            title_baseline: top
                .add_length(&length(section.title_ascent)?)
                .before(0.0)?
                .quanta(),
            top: top.round(SpatialRounding::NearestEven)?.quanta(),
            width: hard(&section_width)?,
            height: hard(&occupied)?,
            capacity,
            columns: flow.columns,
            entries: flow.slots,
            body_top: top
                .add_length(&section.title_occupancy)
                .round(SpatialRounding::NearestEven)?
                .quanta(),
            color,
        });
        top = top.add_length(&occupied);
    }
    let (width, height) = assignment.frame_size([&width, &top], metrics, frame_minimum, scope)?;
    Ok(LegendLayout {
        width,
        height,
        sections,
        compact_changes,
    })
}

fn select_entry_visibility(
    slots: &mut [super::LegendSlot],
    region: crate::layout::PanelBox,
    horizontal: bool,
    scope: &FitScope,
) -> Result<(), PlanError> {
    let mut labels = crate::data::source::collect_source(slots.iter().map(|slot| {
        let plan = slot.block.plan();
        let (offset, anchor) = super::label_anchor(plan)?;
        let x = slot.label[0]
            .checked_add(offset)
            .ok_or_else(|| Error::Render("guide label anchor overflow".into()))?;
        let placement = crate::layout::text::placement::resolve(
            crate::render::backend::PlannedTextPlacement::baseline(
                crate::render::backend::Point2D::new(pixels(x), pixels(slot.label[1])),
                anchor,
                0.0,
            ),
            plan,
            None,
        )?;
        Ok(crate::render::backend::PlacedText {
            placement,
            plan: plan.clone(),
        })
    }))?;
    let indices = crate::data::source::collect_source((0..labels.len()).map(Ok))?;
    crate::layout::text::visibility::apply(&mut labels, &indices, region, !horizontal, scope)?;
    for (slot, label) in slots.iter_mut().zip(labels) {
        slot.block = TextBlock::from_resolved(label.plan);
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "one owner consumes fixed metrics, intrinsics, assignment and the global flow variant"
)]
fn measure_layout(
    descriptor: &PreparedGuide,
    intrinsic: &[SectionIntrinsics],
    assignment: &Assignment,
    horizontal: bool,
    compact: bool,
    theme: &ResolvedTheme,
    metrics: &LegendMetrics,
    frame_minimum: f64,
    compact_changes: &mut bool,
    scope: &FitScope,
) -> Result<LegendLayout, PlanError> {
    count_gaps(descriptor.sections.len())?;
    let compact_cap = assignment.compact_cap(metrics)?;
    if !compact {
        *compact_changes = intrinsic.iter().any(|section| {
            section.entries.iter().any(|entry| {
                entry_allocation(entry, section, assignment.inner_width, horizontal)
                    != entry_allocation(entry, section, compact_cap, horizontal)
            })
        });
    }
    let cap = if compact {
        compact_cap
    } else {
        assignment.inner_width
    };
    if cap <= 0 && descriptor.has_symbols() {
        return Err(scope
            .failure(
                FitKind::LegendOrGuide,
                "compact guide has no positive column width",
            )
            .into());
    }
    let measured = fit::collect(
        "measured guide sections",
        descriptor.sections.iter().zip(intrinsic).enumerate().map(
            |(ordinal, (section, intrinsic))| {
                measure_section(
                    section,
                    intrinsic,
                    cap,
                    horizontal,
                    theme,
                    metrics,
                    &scope.child(AddressRole::LegendSection, ordinal)?,
                    assignment.inner_width,
                    compact,
                )
            },
        ),
    )?;
    let layout = stack_sections(
        measured,
        assignment,
        horizontal,
        metrics,
        frame_minimum,
        scope,
    )?;
    if !compact {
        *compact_changes |= layout.compact_changes;
    }
    Ok(layout)
}

pub(super) fn natural_preference(
    descriptor: &PreparedGuide,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let (metrics, frame_minimum) = Legend::allocation_metrics(descriptor, theme)?;
    let intrinsic = guide_intrinsics(descriptor, theme, &metrics)?;
    // The text planner's conservative advance guard is a quantum, not a new
    // layout alternative. Include it once in the complete natural requirement.
    let width = metrics
        .outer_width(&length(hard(&preferred_content_width(&intrinsic)?)?)?)?
        .max(frame_minimum);
    let scope = FitScope::root();
    let assignment =
        Assignment::new((width, None), &metrics, &scope).map_err(PlanError::into_error)?;
    let layout = measure_layout(
        descriptor,
        &intrinsic,
        &assignment,
        false,
        false,
        theme,
        &metrics,
        frame_minimum,
        &mut false,
        &scope,
    )
    .map_err(PlanError::into_error)?;
    Ok((pixels(layout.width), pixels(layout.height)))
}

pub(super) fn width_requirements(
    descriptor: &PreparedGuide,
    theme: &ResolvedTheme,
) -> Result<super::GuideWidths, Error> {
    let (metrics, frame_minimum) = Legend::allocation_metrics(descriptor, theme)?;
    let intrinsics = guide_intrinsics(descriptor, theme, &metrics)?;
    let mut title = 0;
    let mut minimum = length(0)?;
    let preferred = preferred_content_width(&intrinsics)?;
    for section in intrinsics {
        title = title.max(section.title_minimum);
        for candidate in std::iter::once(length(section.title_minimum)?)
            .chain(section.color_width)
            .chain(section.entries.into_iter().map(|entry| entry.minimum))
        {
            if minimum.compare(&candidate).is_lt() {
                minimum = candidate;
            }
        }
    }
    Ok(super::GuideWidths {
        title_minimum: metrics.outer_width(&length(title)?)?.max(frame_minimum),
        minimum: metrics
            .outer_width(&length(hard(&minimum)?)?)?
            .max(frame_minimum),
        preferred: metrics
            .outer_width(&length(hard(&preferred)?)?)?
            .max(frame_minimum),
        outside_gap: metrics.outside_gap,
    })
}

fn preferred_content_width(intrinsics: &[SectionIntrinsics]) -> Result<ExactLength, Error> {
    let mut preferred = length(0)?;
    for section in intrinsics {
        let title = length(add(section.title_preferred, 1)?)?;
        let symbols = (!section.entries.is_empty())
            .then(|| length(add(section.preferred_column, 1)?))
            .transpose()?;
        for candidate in std::iter::once(&title)
            .chain(symbols.as_ref())
            .chain(section.color_width.as_ref())
        {
            if preferred.compare(candidate).is_lt() {
                preferred = candidate.times(1);
            }
        }
    }
    Ok(preferred)
}

pub(super) fn measure_assigned(
    descriptor: Arc<PreparedGuide>,
    available: (f64, Option<f64>),
    horizontal: bool,
    compact: bool,
    theme: &ResolvedTheme,
    compact_changes: &mut bool,
    scope: &FitScope,
) -> Result<FitOutcome<Legend>, Error> {
    cache::measure_assigned(
        descriptor,
        available,
        horizontal,
        compact,
        theme,
        compact_changes,
        scope,
    )
}

fn measure_uncached(
    descriptor: Arc<PreparedGuide>,
    available: (f64, Option<f64>),
    horizontal: bool,
    compact: bool,
    theme: &ResolvedTheme,
    compact_changes: &mut bool,
    scope: &FitScope,
) -> Result<FitOutcome<Legend>, Error> {
    if !compact {
        *compact_changes = false;
    }
    let (metrics, frame_minimum) = Legend::allocation_metrics(&descriptor, theme)?;
    let assignment = match Assignment::new(available, &metrics, scope) {
        Ok(value) => value,
        Err(PlanError::Fit(failure)) => return Ok(FitOutcome::Miss(failure)),
        Err(PlanError::Fatal(error)) => return Err(error),
    };
    let intrinsic = guide_intrinsics(&descriptor, theme, &metrics)?;
    let layout = match measure_layout(
        &descriptor,
        &intrinsic,
        &assignment,
        horizontal,
        compact,
        theme,
        &metrics,
        frame_minimum,
        compact_changes,
        scope,
    ) {
        Ok(value) => value,
        Err(PlanError::Fit(failure)) => return Ok(FitOutcome::Miss(failure)),
        Err(PlanError::Fatal(error)) => return Err(error),
    };
    let legend = Legend {
        descriptor,
        metrics: Arc::new(metrics),
        available_width: assignment.width,
        available_height: assignment.height,
        layout: Arc::new(layout),
    };
    let bounds = legend.box_at(0.0, 0.0);
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
    let (x, y) = bounds.anchor();
    legend.visit_calls(x, y, |call| call.geometry(&mut ink))?;
    if ink.bounds().iter().any(|ink| {
        ink.left < bounds.left
            || ink.right > bounds.right
            || ink.bottom < bounds.bottom
            || ink.top > bounds.top
    }) {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::LegendOrGuide,
            "complete discrete guide ink exceeds its assigned allocation",
        )));
    }
    Ok(FitOutcome::Measured(legend))
}
