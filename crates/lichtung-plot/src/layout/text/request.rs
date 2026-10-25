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

/// A real parent row assignment, never an invented automatic-height constraint.
#[expect(
    clippy::too_many_arguments,
    reason = "the parent supplies an independent finite height in addition to the original text request"
)]
pub(crate) fn measure_height(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: f64,
    height: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let Some(original) = appearance
        .layout
        .filter(|request| request.instructions().fit.is_some())
    else {
        return measure(theme, source, appearance, width, alignment, owner, scope);
    };
    let mut assigned = theme.clone();
    assigned.text_height = Some(capacity(height)?);
    Ok(
        match measure(
            &assigned, source, appearance, width, alignment, owner, scope,
        )? {
            FitOutcome::Measured(mut plan) => {
                if let Some(input) = plan.payload_mut().fit_input.as_mut() {
                    std::sync::Arc::make_mut(input).height = Some(capacity(height)?);
                }
                FitOutcome::Measured(plan.with_request(Some(original)))
            }
            FitOutcome::Miss(miss) => FitOutcome::Miss(miss),
        },
    )
}

pub(crate) fn measure_quanta(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    if !appearance.hidden()
        && appearance
            .layout
            .is_some_and(|request| request.instructions().fit.is_some())
    {
        return Ok(
            match super::font_fit::measure(
                theme, source, appearance, width, alignment, owner, scope,
            )? {
                FitOutcome::Measured(plan) => FitOutcome::Measured(plan.freeze_orientation()?),
                FitOutcome::Miss(miss) => FitOutcome::Miss(miss),
            },
        );
    }
    measure_unfitted(theme, source, appearance, width, alignment, owner, scope)
}

pub(super) fn measure_unfitted(
    theme: &ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let measured = if appearance.hidden() {
        None
    } else {
        appearance
            .layout
            .and_then(|request| request.instructions().overflow.as_ref())
            .and_then(|overflow| overflow.ellipsis.as_ref())
    };
    Ok(
        match if let Some(ellipsis) = measured {
            super::ellipsis::measure(
                theme, source, appearance, width, alignment, owner, scope, ellipsis,
            )?
        } else {
            measure_current(theme, source, appearance, width, alignment, owner, scope)?
        } {
            FitOutcome::Measured(plan) => FitOutcome::Measured(plan.freeze_orientation()?),
            FitOutcome::Miss(miss) => FitOutcome::Miss(miss),
        },
    )
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "one complete text request resolves independent spatial/line constraints and its explicit terminal action"
)]
pub(super) fn measure_current(
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
    let request = appearance.layout.unwrap_or(&crate::TextLayout::EMPTY);
    let mut width = width;
    let mut assigned_height = theme.text_height;
    if let Some((horizontal, vertical)) = &theme.text_padding {
        let padding =
            |spacing: &crate::RelativeSpacing, size: f64| -> Result<i64, super::TextPlanError> {
                Ok(spacing
                    .product(crate::theme::LocalMetric::css_pixels(size)?)
                    .round(crate::layout::kernel::SpatialRounding::Ceil)?
                    .quanta())
            };
        let h = padding(horizontal, appearance.theme.get_font_size())
            .map_err(|error| Error::Render(error.to_string()))?;
        let v = padding(vertical, appearance.theme.get_font_size())
            .map_err(|error| Error::Render(error.to_string()))?;
        if let Some(trace) = &theme.text_size_trial {
            trace
                .observe(h, |size| padding(horizontal, size))
                .map_err(|error| Error::Render(error.to_string()))?;
            trace
                .observe(v, |size| padding(vertical, size))
                .map_err(|error| Error::Render(error.to_string()))?;
        }
        width = width.saturating_sub(h).max(0);
        assigned_height = assigned_height.map(|height| height.saturating_sub(v).max(0));
    }
    if let Some(crate::text::orientation::Orientation::Data(from, to)) = request
        .instructions()
        .orientation
        .as_ref()
        .map(|orientation| &orientation.0)
    {
        let frame = theme.text_direction.as_ref().ok_or_else(|| {
            Error::Data("data text direction requires actual scientific axes".into())
        })?;
        let resolved = request.clone().rotation(frame.rotation(from, to, theme)?);
        return Ok(
            match measure_quanta(
                theme,
                source,
                TextAppearance {
                    layout: Some(&resolved),
                    ..appearance
                },
                width,
                alignment,
                owner,
                scope,
            )? {
                FitOutcome::Measured(plan) => {
                    FitOutcome::Measured(plan.with_request(Some(request)))
                }
                FitOutcome::Miss(miss) => FitOutcome::Miss(miss),
            },
        );
    }
    let instructions = request.instructions();
    let rotated = match instructions
        .orientation
        .as_ref()
        .map(|orientation| &orientation.0)
    {
        Some(crate::text::orientation::Orientation::Fixed(rotation)) => Some(rotation.0.get()),
        Some(crate::text::orientation::Orientation::Data(..)) => {
            return Err(Error::Data(
                "data direction has no actual scientific projection".into(),
            ))
        }
        Some(crate::text::orientation::Orientation::Automatic(..)) => {
            return Err(Error::Data(
                "automatic orientation requires an actual tick group".into(),
            ))
        }
        None => None,
    };
    let assigned = width;
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
    let maximum = if wrap {
        let projected = if let Some(angle) = rotated {
            let (_, cosine) = crate::text::orientation::components(angle);
            if cosine == 0.0 {
                i64::MAX
            } else {
                bound_capacity((assigned as f64 / 1024.0) / cosine.abs(), i64::MAX)?
            }
        } else {
            width
        };
        projected.min(
            bounds
                .and_then(|bounds| bounds.width)
                .map_or(Ok(i64::MAX), |bound| bound_capacity(bound.get(), i64::MAX))?,
        )
    } else {
        i64::MAX
    };
    let planned = fit::outcome(theme.plan_text_quanta_scoped(
        source,
        appearance.theme,
        maximum,
        alignment,
        owner,
        appearance.exact_override,
        instructions.direction.unwrap_or_default(),
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
                    instructions.direction.unwrap_or_default(),
                    scope,
                )
                .map_err(fit::PlanError::into_error)?,
            Some(miss),
        ),
        FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
    };
    let mut plan = plan.with_request(appearance.layout);
    if let Some(angle) = rotated.filter(|_| wrap) {
        let (sine, cosine) = crate::text::orientation::components(angle);
        let mut previous = maximum;
        loop {
            if cosine == 0.0 {
                break;
            }
            let available = assigned as f64 / 1024.0 - plan.height() * sine.abs();
            let next = bound_capacity((available / cosine.abs()).max(0.0), previous)?;
            if next >= previous {
                break;
            }
            match fit::outcome(theme.plan_text_quanta_scoped(
                source,
                appearance.theme,
                next,
                alignment,
                owner,
                appearance.exact_override,
                instructions.direction.unwrap_or_default(),
                scope,
            ))? {
                FitOutcome::Measured(value) => {
                    plan = value.with_request(appearance.layout);
                    previous = next;
                }
                FitOutcome::Miss(_) => break,
            }
        }
    }
    if hidden {
        return Ok(FitOutcome::Measured(plan.hide()?));
    }
    let physical_width = if rotated.is_some() {
        let placement = super::placement::at(
            &plan,
            crate::render::backend::Point2D::new(0.0, 0.0),
            crate::TextAnchor::center(),
            0.0,
            false,
            None,
        )?;
        super::placement::logical_bounds(placement, &plan)?.width()
    } else {
        plan.width()
    };
    let physical_height = if let Some(angle) = rotated {
        let (sine, cosine) = crate::text::orientation::components(angle);
        plan.width() * sine.abs() + plan.height() * cosine.abs()
    } else {
        plan.height()
    };
    let fits = miss.is_none()
        && physical_width <= assigned as f64 / 1024.0
        && bounds
            .and_then(|bounds| bounds.width)
            .is_none_or(|bound| plan.width() <= bound.get())
        && plan.height_quanta <= height
        && assigned_height.is_none_or(|height| physical_height <= height as f64 / 1024.0)
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
                format!("{owner}: complete text exceeds its declared bounds (projected width {physical_width}, assigned width {})",assigned as f64/1024.0),
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
