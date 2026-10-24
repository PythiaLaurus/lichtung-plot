//! Resolve complete text constraints before parent geometry and frozen placement.

use super::ResolvedTextPlan;
use crate::layout::{
    fit::{self, FitOutcome, FitScope},
    PhysicalTextAlignment,
};
use crate::text::TextAppearance;
use crate::theme::ResolvedTheme;
use crate::{Error, TextFailure, TextWrapping};

#[cfg(test)]
mod tests;

#[expect(
    clippy::too_many_arguments,
    reason = "original occurrence eligibility is independent of the complete measured text request and assignment"
)]
pub(crate) fn measure_occurrence(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
    occurrence: (usize, &[usize]),
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    if appearance.includes(occurrence.0, occurrence.1) {
        measure_quanta(theme, source, appearance, width, alignment, owner, scope)
    } else {
        Ok(FitOutcome::Measured(ResolvedTextPlan::hidden_source(
            source, appearance, alignment, owner,
        )?))
    }
}

pub(crate) fn measure(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    measure_quanta(
        theme,
        source,
        appearance,
        capacity(width)?,
        alignment,
        owner,
        scope,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "one complete text request resolves independent spatial/line constraints and its explicit terminal action"
)]
pub(crate) fn measure_quanta(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    if appearance.hidden() {
        return Ok(FitOutcome::Measured(ResolvedTextPlan::hidden_source(
            source,
            appearance,
            appearance
                .layout
                .and_then(|layout| layout.alignment)
                .map_or(alignment, Into::into),
            owner,
        )?));
    }
    let Some(request) = appearance.layout else {
        return fit::outcome(theme.plan_text_quanta_scoped(
            source,
            appearance.theme,
            width,
            alignment,
            owner,
            appearance.exact_override,
            scope,
        ));
    };
    let instructions = request.instructions();
    let bounds = instructions.bounds;
    let width = bounds
        .and_then(|bounds| bounds.width)
        .map_or(Ok(width), |bound| bound_capacity(bound.get(), width))?;
    let height = bounds
        .and_then(|bounds| bounds.height)
        .map_or(Ok(i64::MAX), |bound| bound_capacity(bound.get(), i64::MAX))?;
    let alignment = request.alignment.map_or(alignment, Into::into);
    let wrap = instructions.wrapping != Some(TextWrapping::NoSoftWrap);
    let failure = instructions
        .overflow
        .as_ref()
        .map_or(TextFailure::Error, |overflow| overflow.failure);
    let hidden = instructions
        .visibility
        .as_ref()
        .is_some_and(|visibility| visibility.hidden);
    let maximum = if wrap { width } else { i64::MAX };
    let planned = fit::outcome(theme.plan_text_quanta_scoped(
        source,
        appearance.theme,
        maximum,
        alignment,
        owner,
        appearance.exact_override,
        scope,
    ))?;
    let (plan, miss) = match planned {
        FitOutcome::Measured(plan) => (plan, None),
        FitOutcome::Miss(miss) if failure != TextFailure::Error || hidden => (
            theme
                .plan_text_quanta_scoped(
                    source,
                    appearance.theme,
                    i64::MAX,
                    alignment,
                    owner,
                    appearance.exact_override,
                    scope,
                )
                .map_err(fit::PlanError::into_error)?,
            Some(miss),
        ),
        FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
    };
    let plan = plan.with_request(Some(request));
    if hidden {
        return Ok(FitOutcome::Measured(plan.hide()?));
    }
    let fits = miss.is_none()
        && plan.width_quanta <= width
        && plan.height_quanta <= height
        && bounds
            .and_then(|bounds| bounds.lines)
            .is_none_or(|limit| plan.lines.len() <= limit);
    if fits {
        return Ok(FitOutcome::Measured(plan));
    }
    match failure {
        TextFailure::Error => Ok(FitOutcome::Miss(miss.unwrap_or_else(|| {
            scope.failure(
                fit::FitKind::Text,
                format!("{owner}: complete text exceeds its declared bounds"),
            )
        }))),
        TextFailure::Hide => Ok(FitOutcome::Measured(plan.hide()?)),
        TextFailure::Visible => Ok(FitOutcome::Measured(plan)),
        TextFailure::Clip => {
            let lines = bounds
                .and_then(|bounds| bounds.lines)
                .map_or(height, |count| {
                    if count == 0 {
                        0
                    } else {
                        i64::try_from(count)
                            .ok()
                            .and_then(|count| plan.line_pitch_quanta.checked_mul(count))
                            .unwrap_or(i64::MAX)
                            .min(height)
                    }
                });
            Ok(FitOutcome::Measured(plan.clip_to(width, lines)))
        }
    }
}

fn capacity(value: f64) -> Result<i64, Error> {
    super::maximum_width_quanta(value).map_err(|error| Error::Render(error.to_string()))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a larger finite caller maximum does not consume geometry beyond the actual representable capacity"
)]
fn bound_capacity(value: f64, assigned: i64) -> Result<i64, Error> {
    if value >= assigned as f64 / 1024.0 {
        Ok(assigned)
    } else {
        capacity(value).map(|limit| assigned.min(limit))
    }
}
