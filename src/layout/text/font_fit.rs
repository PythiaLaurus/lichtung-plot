//! Local font fitting over certified metric cells, never a sampled-size quota.

pub(crate) mod stability;
#[cfg(test)]
mod tests;

use super::{ExactRational, PhysicalTextAlignment, ResolvedTextPlan, TextPlanError};
use crate::layout::fit::{FitOutcome, FitScope};
use crate::text::TextAppearance;
use crate::theme::ResolvedTheme;
use crate::{Error, TextFailure, TextFitPriority, TextOverflow, TextTheme};
use std::sync::Arc;

/// The original preferred appearance and assignment are distinct from the
/// selected font size and display. They also make actual-group remeasurement
/// independent of formatters and source callbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FitInput {
    pub(crate) appearance: TextTheme,
    pub(crate) width: i64,
    pub(crate) exact_override: bool,
    pub(crate) height: Option<i64>,
    pub(crate) padding: Option<(crate::RelativeSpacing, crate::RelativeSpacing)>,
    contrast_role: Option<bool>,
}

#[expect(
    clippy::too_many_lines,
    reason = "the single local transaction orders complete/marked size programs, their certified exclusions and terminal floor handling"
)]
pub(crate) fn measure(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let request = appearance.layout.expect("font fit request");
    let policy = request.instructions().fit.expect("font fit policy");
    let preferred = appearance.theme.get_font_size();
    let (minimum, maximum) = policy.bounds(preferred)?;
    let permitted = preferred.clamp(minimum, maximum);
    let mut complete = complete_request(request);
    complete.edit_instructions().fit = None;
    let mut marked = request.clone();
    marked.edit_instructions().fit = None;
    if let Some(overflow) = marked.edit_instructions().overflow.as_mut() {
        overflow.failure = TextFailure::Error;
    }
    let priority = request.instructions().fit_priority.unwrap_or_default();
    let ellipsis = request
        .instructions()
        .overflow
        .as_ref()
        .and_then(|overflow| overflow.ellipsis.as_ref());
    let mut basis_request = complete.clone();
    let instructions = basis_request.edit_instructions();
    instructions.bounds = None;
    instructions.orientation = None;
    instructions.wrapping = Some(crate::TextWrapping::NoSoftWrap);
    let basis_style = appearance.theme.clone().font_size(1.0)?;
    let mut basis_theme = theme.clone();
    basis_theme.text_height = None;
    basis_theme.text_size_trial = None;
    basis_theme.text_padding = None;
    let basis = match super::request::measure_current(
        &basis_theme,
        source,
        TextAppearance {
            theme: &basis_style,
            layout: Some(&basis_request),
            ..appearance
        },
        i64::MAX,
        alignment,
        owner,
        scope,
    )? {
        FitOutcome::Measured(plan) => plan,
        FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
    };
    let evidence = BoundsEvidence::new(&basis, theme.prepared_font_environment()?, appearance)
        .map_err(metric_error)?;
    let selected = |layout: &crate::TextLayout, size: f64| -> Result<_, Error> {
        let style = appearance.theme.clone().font_size(size)?;
        let trace = Arc::new(stability::Stability::new(size, minimum));
        let mut trial = theme.clone();
        trial.text_size_trial = Some(Arc::clone(&trace));
        let outcome = super::request::measure_unfitted(
            &trial,
            source,
            TextAppearance {
                theme: &style,
                layout: Some(layout),
                ..appearance
            },
            width,
            alignment,
            owner,
            scope,
        )?;
        Ok((outcome, trace))
    };
    let finish = |mut plan: ResolvedTextPlan| -> Result<_, Error> {
        // The selected glyphs are enough for capture. Do not keep trial analyses
        // (and their failed-size certificates) alive as an all-candidate cache.
        plan.1 = None;
        plan.payload_mut().fit_input = Some(Arc::new(FitInput {
            appearance: appearance.theme.clone(),
            width,
            exact_override: appearance.exact_override,
            height: theme.text_height,
            padding: theme.text_padding.clone(),
            // These references identify deliberate appearance owners, never
            // equality inferred from equal current glyphs or preferred sizes.
            contrast_role: if std::ptr::eq(
                appearance.theme,
                theme
                    .typography
                    .get_annotations()
                    .get_contrast()
                    .get_light(),
            ) {
                Some(true)
            } else if std::ptr::eq(
                appearance.theme,
                theme.typography.get_annotations().get_contrast().get_dark(),
            ) {
                Some(false)
            } else {
                None
            },
        }));
        Ok(FitOutcome::Measured(plan.with_request(Some(request))))
    };
    if priority == TextFitPriority::PreferredSizeFirst && ellipsis.is_some() {
        if let (FitOutcome::Measured(plan), _) = selected(&marked, permitted)? {
            return finish(plan);
        }
    }
    for layout in std::iter::once(&complete).chain(ellipsis.map(|_| &marked)) {
        let Some(ceiling) = evidence.ceiling(
            layout,
            width,
            minimum,
            maximum,
            theme.text_height,
            layout
                .instructions()
                .overflow
                .as_ref()
                .is_none_or(|overflow| overflow.ellipsis.is_none()),
        )?
        else {
            continue;
        };
        let target = permitted.min(ceiling);
        let mut proposal = target;
        if basis.width() > 0.0 {
            proposal = proposal.min(pixels(width) / basis.width());
        }
        if let Some(height) = layout
            .instructions()
            .bounds
            .and_then(|bounds| bounds.height)
        {
            proposal = proposal.min(height.get() / basis.height());
        }
        if let Some(width) = layout.instructions().bounds.and_then(|bounds| bounds.width) {
            if basis.width() > 0.0 {
                proposal = proposal.min(width.get() / basis.width());
            }
        }
        proposal = proposal.clamp(minimum, ceiling);
        // Endpoints and the real clamped preference are measured explicitly.
        for size in [ceiling, target, proposal, minimum] {
            if let (FitOutcome::Measured(plan), _) = selected(layout, size)? {
                return finish(plan);
            }
        }
        let mut size = ceiling;
        loop {
            let (outcome, trace) = selected(layout, size)?;
            if let FitOutcome::Measured(plan) = outcome {
                return finish(plan);
            }
            let lower = trace
                .lower()
                .map_err(|error| Error::Render(error.to_string()))?;
            if lower <= minimum {
                break;
            }
            // Every reached rounded metric, wrap decision and derived-view
            // decision is unchanged on [lower,size]. Exclude exactly that cell.
            // At least one positive finite bit pattern is removed each time.
            size = f64::from_bits(lower.to_bits() - 1);
        }
    }
    let style = appearance.theme.clone().font_size(minimum)?;
    let mut terminal = request.clone();
    terminal.edit_instructions().fit = None;
    let outcome = super::request::measure_unfitted(
        theme,
        source,
        TextAppearance {
            theme: &style,
            layout: Some(&terminal),
            ..appearance
        },
        width,
        alignment,
        owner,
        scope,
    )?;
    match outcome {
        FitOutcome::Measured(plan) => finish(plan),
        FitOutcome::Miss(miss) => Ok(FitOutcome::Miss(miss)),
    }
}

/// Optimistic, exact design-space lower bounds prune only proven nonfits.
/// Rounded glyph origins move an ideal full-line ink span by at most one
/// quantum. Each real line also retains at least one admitted face's metrics.
struct BoundsEvidence {
    width: ExactRational,
    metric: ExactRational,
    paragraphs: usize,
    unbreakable: bool,
}
impl BoundsEvidence {
    fn new(
        basis: &ResolvedTextPlan,
        environment: &crate::font::canonical::PreparedFontEnvironment,
        appearance: TextAppearance<'_>,
    ) -> Result<Self, TextPlanError> {
        let mut width = ExactRational::ZERO;
        for line in &basis.lines {
            let mut pen_x = ExactRational::ZERO;
            let mut pen_y = ExactRational::ZERO;
            let mut ink: Option<super::ExactInk> = None;
            for run in &line.runs {
                for glyph in &run.glyphs {
                    if let Some(design) = glyph.design_ink {
                        let x = pen_x.add(
                            &ExactRational::design(
                                glyph.design_x_offset.into(),
                                1.0,
                                run.units_per_em,
                            )?,
                            "ideal glyph",
                        )?;
                        let y = pen_y.add(
                            &ExactRational::design(
                                glyph.design_y_offset.into(),
                                1.0,
                                run.units_per_em,
                            )?,
                            "ideal glyph",
                        )?;
                        let next =
                            super::exact_glyph_ink(design, 0, 0, 1.0.into(), run.units_per_em)?
                                .translated(&x, &y)?;
                        ink = Some(match ink {
                            Some(current) => current.union(next),
                            None => next,
                        });
                    }
                    pen_x = pen_x.add(
                        &ExactRational::design(
                            glyph.design_x_advance.into(),
                            1.0,
                            run.units_per_em,
                        )?,
                        "ideal pen",
                    )?;
                    pen_y = pen_y.add(
                        &ExactRational::design(
                            glyph.design_y_advance.into(),
                            1.0,
                            run.units_per_em,
                        )?,
                        "ideal pen",
                    )?;
                }
            }
            if let Some(ink) = ink {
                width = width.max(ink.right.add(&ink.left.negated()?, "ideal span")?);
            }
        }
        let pair = environment
            .prepared_pair(
                appearance.theme.get_stack(),
                appearance.theme.get_weight().into(),
            )?
            .ok_or(TextPlanError::MissingPreparedPair)?;
        let mut metric = None;
        for &face in &pair.provenance.candidates {
            let metrics = environment.face_metrics(face)?;
            let next = ExactRational::design(
                i64::from(metrics.ascender) - i64::from(metrics.descender),
                1.0,
                metrics.units_per_em,
            )?;
            metric = Some(match metric {
                Some(current) => ExactRational::min(current, next),
                None => next,
            });
        }
        let mut unbreakable = true;
        if let Some(analysis) = &basis.1 {
            for paragraph in &analysis.paragraphs {
                unbreakable &= super::legal_line_breaks_at_graphemes(
                    &basis.source[paragraph.span.range()],
                    &paragraph.grapheme_ends,
                )?
                .len()
                    <= 1;
            }
        }
        Ok(Self {
            width,
            metric: metric.ok_or(TextPlanError::MissingPreparedPair)?,
            paragraphs: basis.lines.len(),
            unbreakable,
        })
    }
    fn ceiling(
        &self,
        request: &crate::TextLayout,
        width: i64,
        minimum: f64,
        maximum: f64,
        assigned_height: Option<i64>,
        complete: bool,
    ) -> Result<Option<f64>, Error> {
        let bounds = request.instructions().bounds;
        if bounds.and_then(|bounds| bounds.lines) == Some(0)
            || (complete
                && bounds
                    .and_then(|bounds| bounds.lines)
                    .is_some_and(|lines| lines < self.paragraphs))
        {
            return Ok(None);
        }
        let possible = |size: f64| -> Result<bool, TextPlanError> {
            let factor = ExactRational::design(1, size, 1024)?;
            let metric = self
                .metric
                .multiply(&factor)?
                .multiply(&ExactRational::new(
                    if complete {
                        i128::try_from(self.paragraphs)
                            .map_err(|_| TextPlanError::Arithmetic("font fit paragraph count"))?
                    } else {
                        1
                    },
                    1,
                )?)?;
            if request.instructions().orientation.is_none()
                && assigned_height
                    .is_some_and(|height| metric.greater(&ExactRational::Small(height.into(), 1)))
            {
                return Ok(false);
            }
            if let Some(height) = bounds.and_then(|bounds| bounds.height) {
                if metric.greater(&ExactRational::design(
                    1,
                    height.get().max(f64::from_bits(1)),
                    1,
                )?) || height.get() == 0.0
                {
                    return Ok(false);
                }
            }
            if complete
                && (self.unbreakable
                    || bounds.and_then(|bounds| bounds.lines) == Some(self.paragraphs)
                    || request.instructions().wrapping == Some(crate::TextWrapping::NoSoftWrap))
            {
                let span = self
                    .width
                    .multiply(&factor)?
                    .add(&ExactRational::new(-1, 1)?, "ideal span error")?;
                if let Some(bound) = bounds.and_then(|bounds| bounds.width) {
                    let limit = if bound.get() == 0.0 {
                        ExactRational::ZERO
                    } else {
                        ExactRational::design(1, bound.get(), 1)?
                    };
                    if span.greater(&limit) {
                        return Ok(false);
                    }
                }
                if request.instructions().orientation.is_none()
                    && span.greater(&ExactRational::new(width.into(), 1)?)
                {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        if !possible(minimum).map_err(metric_error)? {
            return Ok(None);
        }
        if possible(maximum).map_err(metric_error)? {
            return Ok(Some(maximum));
        }
        let mut lo = minimum.to_bits();
        let mut hi = maximum.to_bits();
        while hi - lo > 1 {
            let middle = lo + (hi - lo) / 2;
            if possible(f64::from_bits(middle)).map_err(metric_error)? {
                lo = middle;
            } else {
                hi = middle;
            }
        }
        Ok(Some(f64::from_bits(lo)))
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Result::map_err adapter consumes the failed metric"
)]
fn metric_error(error: TextPlanError) -> Error {
    Error::Render(format!("font fit metric evidence: {error}"))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the actual assignment is represented in canonical text quanta"
)]
fn pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupMember {
    source: String,
    input: Arc<FitInput>,
    request: crate::TextLayout,
}

/// Called by one actual role before its selected metrics allocate geometry.
/// Independent slots never call this together. Different contrast appearances
/// remain different groups even when their preferred sizes happen to match.
#[expect(
    clippy::too_many_lines,
    reason = "actual group membership, common interval and joint certified trials are frozen together"
)]
pub(crate) fn uniform(
    theme: &ResolvedTheme,
    plans: &mut [ResolvedTextPlan],
    scope: &FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    let mut visited = crate::data::source::collect_source(plans.iter().map(|_| Ok(false)))?;
    for start in 0..plans.len() {
        if visited[start] || !is_uniform(&plans[start]) {
            continue;
        }
        let indices = crate::data::source::collect_source(
            (start..plans.len())
                .filter(|&index| {
                    is_uniform(&plans[index]) && same_appearance(&plans[start], &plans[index])
                })
                .map(Ok),
        )?;
        for &index in &indices {
            visited[index] = true;
        }
        let mut minimum = 0.0_f64;
        let mut maximum = f64::MAX;
        let mut target = f64::MAX;
        let mut members = Vec::new();
        crate::data::source::reserve_source(&mut members, indices.len())?;
        for &index in &indices {
            let plan = &plans[index];
            let input = plan.fit_input.as_ref().expect("uniform fit input");
            let request = plan.request.as_ref().expect("uniform request");
            let (lower, upper) = request
                .instructions()
                .fit
                .expect("uniform fit")
                .bounds(input.appearance.get_font_size())?;
            minimum = minimum.max(lower);
            maximum = maximum.min(upper);
            target = target.min(plan.style.font_size);
            members.push(GroupMember {
                source: crate::data::source::copy_source_string(plan.original_source())?,
                input: Arc::clone(input),
                request: request.clone(),
            });
        }
        if minimum > maximum {
            return Err(scope
                .failure(
                    crate::layout::fit::FitKind::Text,
                    "uniform text group has no common permitted font size",
                )
                .into());
        }
        target = target.clamp(minimum, maximum);
        if indices
            .iter()
            .all(|&index| plans[index].style.font_size.to_bits() == target.to_bits())
        {
            let identity = crate::data::source::source_arc((scope.address.clone(), members))?;
            for &index in &indices {
                plans[index].payload_mut().fit_group = Some(Arc::clone(&identity));
            }
            continue;
        }
        let mut accepted = None;
        for abbreviated in [false, true] {
            if abbreviated
                && !members.iter().any(|member| {
                    member
                        .request
                        .instructions()
                        .overflow
                        .as_ref()
                        .is_some_and(|overflow| overflow.ellipsis.is_some())
                })
            {
                continue;
            }
            let mut initial = Some(target);
            let mut size = target;
            loop {
                let trace = Arc::new(stability::Stability::new(size, minimum));
                let mut trial = theme.clone();
                trial.text_size_trial = Some(Arc::clone(&trace));
                let mut selected = Vec::new();
                crate::data::source::reserve_source(&mut selected, indices.len())?;
                let mut miss = false;
                for (&index, member) in indices.iter().zip(&members) {
                    let mut request = member.request.clone();
                    trial.text_height = member.input.height;
                    trial.text_padding.clone_from(&member.input.padding);
                    request.edit_instructions().fit = None;
                    if abbreviated
                        || (member.request.instructions().fit_priority
                            == Some(TextFitPriority::PreferredSizeFirst)
                            && size.to_bits()
                                == member
                                    .input
                                    .appearance
                                    .get_font_size()
                                    .clamp(minimum, maximum)
                                    .to_bits())
                    {
                        if let Some(overflow) = request.edit_instructions().overflow.as_mut() {
                            overflow.failure = TextFailure::Error;
                        }
                    } else {
                        request = complete_request(&request);
                    }
                    let style = member.input.appearance.clone().font_size(size)?;
                    match super::request::measure_unfitted(
                        &trial,
                        &member.source,
                        TextAppearance {
                            theme: &style,
                            exact_override: member.input.exact_override,
                            layout: Some(&request),
                        },
                        member.input.width,
                        plans[index].style.alignment,
                        &plans[index].style.provenance.owner,
                        scope,
                    )? {
                        FitOutcome::Measured(mut plan) => {
                            plan.1 = None;
                            plan.payload_mut().fit_input = Some(Arc::clone(&member.input));
                            selected.push(plan.with_request(Some(&member.request)));
                        }
                        FitOutcome::Miss(_) => miss = true,
                    }
                }
                if !miss {
                    accepted = Some(selected);
                    break;
                }
                if initial.take().is_some() {
                    size = maximum;
                    continue;
                }
                let lower = trace.lower().map_err(metric_error)?;
                if lower <= minimum {
                    break;
                }
                size = f64::from_bits(lower.to_bits() - 1);
            }
            if accepted.is_some() {
                break;
            }
        }
        let selected = if let Some(selected) = accepted {
            selected
        } else {
            crate::layout::fit::collect(
                "uniform text floor",
                indices.iter().zip(&members).map(|(&index, member)| {
                    let request = member
                        .request
                        .clone()
                        .fit(crate::TextFit::range(minimum, minimum)?);
                    let result = super::request::measure_quanta(
                        theme,
                        &member.source,
                        TextAppearance {
                            theme: &member.input.appearance,
                            exact_override: member.input.exact_override,
                            layout: Some(&request),
                        },
                        member.input.width,
                        plans[index].style.alignment,
                        &plans[index].style.provenance.owner,
                        scope,
                    )?;
                    crate::layout::fit::measured(Ok(result))
                        .map(|plan| plan.with_request(Some(&member.request)))
                }),
            )?
        };
        let identity = crate::data::source::source_arc((scope.address.clone(), members))?;
        for (&index, mut plan) in indices.iter().zip(selected) {
            plan.payload_mut().fit_group = Some(Arc::clone(&identity));
            plans[index] = plan.freeze_orientation()?;
        }
    }
    Ok(())
}

fn is_uniform(plan: &ResolvedTextPlan) -> bool {
    plan.fit_group.is_none()
        && plan.fit_input.is_some()
        && plan
            .request
            .as_ref()
            .and_then(|request| request.instructions().fit)
            .is_some_and(|fit| fit.grouping == crate::TextFitGrouping::Uniform)
}

pub(crate) fn uniform_options(
    theme: &ResolvedTheme,
    plans: &mut [Option<ResolvedTextPlan>],
    scope: &FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    let mut selected =
        crate::data::source::collect_source(plans.iter().flatten().cloned().map(Ok))?;
    uniform(theme, &mut selected, scope)?;
    for (target, plan) in plans.iter_mut().filter(|plan| plan.is_some()).zip(selected) {
        *target = Some(plan);
    }
    Ok(())
}

pub(crate) fn uniform_placed(
    theme: &ResolvedTheme,
    text: &mut [crate::render::backend::PlacedText],
    scope: &FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    if !text.iter().any(|text| is_uniform(&text.plan)) {
        return Ok(());
    }
    let mut plans =
        crate::data::source::collect_source(text.iter().map(|text| Ok(text.plan.clone())))?;
    uniform(theme, &mut plans, scope)?;
    for (text, plan) in text.iter_mut().zip(plans) {
        text.placement = super::placement::remeasure(text.placement, &text.plan, &plan)?;
        text.plan = plan;
    }
    Ok(())
}
fn same_appearance(a: &ResolvedTextPlan, b: &ResolvedTextPlan) -> bool {
    if a.fit_input.as_ref().expect("fit input").contrast_role
        != b.fit_input.as_ref().expect("fit input").contrast_role
    {
        return false;
    }
    let a = &a.fit_input.as_ref().expect("fit input").appearance;
    let b = &b.fit_input.as_ref().expect("fit input").appearance;
    a.get_stack() == b.get_stack()
        && a.get_color() == b.get_color()
        && a.get_weight() == b.get_weight()
        && a.get_line_height().to_bits() == b.get_line_height().to_bits()
}

pub(crate) fn minimum_height(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
) -> Result<Option<i64>, Error> {
    let Some(request) = appearance
        .layout
        .filter(|request| request.instructions().fit.is_some())
    else {
        return Ok(None);
    };
    if appearance.hidden()
        || request
            .instructions()
            .overflow
            .as_ref()
            .is_some_and(|overflow| overflow.failure != TextFailure::Error)
    {
        return Ok(Some(0));
    }
    let minimum = request
        .instructions()
        .fit
        .expect("fit policy")
        .bounds(appearance.theme.get_font_size())?
        .0;
    let environment = theme.prepared_font_environment()?;
    let pair = environment
        .prepared_pair(
            appearance.theme.get_stack(),
            appearance.theme.get_weight().into(),
        )
        .map_err(|error| metric_error(error.into()))?
        .ok_or_else(|| Error::Render("text minimum has no admitted font pair".into()))?;
    let mut metric = None;
    for &face in &pair.provenance.candidates {
        let metrics = environment
            .face_metrics(face)
            .map_err(|error| metric_error(error.into()))?;
        let value = ExactRational::design(
            i64::from(metrics.ascender) - i64::from(metrics.descender),
            minimum,
            metrics.units_per_em,
        )
        .map_err(metric_error)?;
        metric = Some(metric.map_or(value.clone(), |prior: ExactRational| prior.min(value)));
    }
    let count = if request
        .instructions()
        .overflow
        .as_ref()
        .is_some_and(|overflow| overflow.ellipsis.is_some())
    {
        1
    } else {
        super::mandatory_paragraph_seeds(source)
            .map_err(metric_error)?
            .len()
    };
    let count = i128::try_from(count)
        .map_err(|_| Error::Render("text minimum paragraph count overflow".into()))?;
    Ok(Some(
        metric
            .ok_or_else(|| Error::Render("text minimum has no font metrics".into()))?
            .multiply(&ExactRational::new(count, 1).map_err(metric_error)?)
            .map_err(metric_error)?
            .ceil_i64("minimum text height")
            .map_err(metric_error)?,
    ))
}

pub(crate) fn assign_height(
    theme: &ResolvedTheme,
    plan: &ResolvedTextPlan,
    height: f64,
    scope: &FitScope,
) -> Result<ResolvedTextPlan, crate::layout::fit::PlanError> {
    let Some(input) = &plan.fit_input else {
        return Ok(plan.clone());
    };
    let mut assigned = theme.clone();
    assigned.text_padding.clone_from(&input.padding);
    crate::layout::fit::measured(super::request::measure_height(
        &assigned,
        plan.original_source(),
        TextAppearance {
            theme: &input.appearance,
            exact_override: input.exact_override,
            layout: plan.request.as_ref(),
        },
        pixels(input.width),
        height,
        plan.style.alignment,
        &plan.style.provenance.owner,
        scope,
    ))
}

pub(crate) fn hard_height(
    theme: &ResolvedTheme,
    plan: &ResolvedTextPlan,
) -> Result<(f64, f64), Error> {
    let Some(input) = &plan.fit_input else {
        return Ok((plan.occupied_size().1, plan.style.font_size));
    };
    let request = plan.request.as_ref().expect("fitted request");
    let size = request
        .instructions()
        .fit
        .expect("fitted policy")
        .bounds(input.appearance.get_font_size())?
        .0;
    let height = minimum_height(
        theme,
        plan.original_source(),
        TextAppearance {
            theme: &input.appearance,
            exact_override: input.exact_override,
            layout: Some(request),
        },
    )?
    .unwrap_or(plan.occupied_height_quanta());
    Ok((pixels(height), size))
}

fn complete_request(request: &crate::TextLayout) -> crate::TextLayout {
    let mut complete = request.clone().overflow(TextOverflow::complete());
    if let Some(lines) = request
        .instructions()
        .overflow
        .as_ref()
        .and_then(|overflow| overflow.ellipsis.as_ref())
        .and_then(|ellipsis| ellipsis.lines)
    {
        let bounds = complete.instructions().bounds.unwrap_or(crate::TextBounds {
            width: None,
            height: None,
            lines: None,
        });
        complete.edit_instructions().bounds = Some(crate::TextBounds {
            lines: Some(bounds.lines.map_or(lines, |value| value.min(lines))),
            ..bounds
        });
    }
    complete
}
