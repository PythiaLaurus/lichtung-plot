//! Text-only group reduction; all source positions and original indices survive.

use crate::layout::{
    fit::{FitKind, FitScope, PlanError},
    PanelBox,
};
use crate::text::layout::VisibilityProgram;
use crate::{TextFailure, TextVisibility};

#[cfg(test)]
mod tests;

pub(crate) struct Candidate {
    pub(crate) index: usize,
    pub(crate) position: f64,
    pub(crate) bounds: PanelBox,
}

/// Freeze visibility once per actual owner group, retaining hidden source plans.
pub(crate) fn apply(
    plans: &mut [crate::render::backend::PlacedText],
    indices: &[usize],
    region: PanelBox,
    vertical: bool,
    scope: &FitScope,
) -> Result<(), PlanError> {
    let mut visited = crate::data::source::collect_source(plans.iter().map(|_| Ok(false)))?;
    for start in 0..plans.len() {
        if visited[start] {
            continue;
        }
        let Some(policy) = plans[start]
            .plan
            .request
            .as_ref()
            .and_then(|request| request.instructions().visibility.clone())
        else {
            continue;
        };
        let members = crate::data::source::collect_source(
            (start..plans.len())
                .filter(|&index| plans[index].plan.request == plans[start].plan.request)
                .map(Ok),
        )?;
        let eligible =
            crate::data::source::collect_source(members.iter().map(|&index| Ok(indices[index])))?;
        let candidates = crate::data::source::collect_source(members.iter().map(|&index| {
            let text = &plans[index];
            Ok(Candidate {
                index: indices[index],
                position: if vertical {
                    text.placement.position.y
                } else {
                    text.placement.position.x
                },
                bounds: super::placement::footprint(text.placement, &text.plan)?,
            })
        }))?;
        let failure = plans[start]
            .plan
            .request
            .as_ref()
            .and_then(|request| request.instructions().overflow.as_ref())
            .map_or(TextFailure::Error, |overflow| overflow.failure);
        let selected = reduce(&policy, &eligible, &candidates, region, failure, scope)?;
        for (&index, visible) in members.iter().zip(selected) {
            visited[index] = true;
            if !visible {
                plans[index].plan = plans[index].plan.clone().hide()?;
            } else if failure == TextFailure::Clip {
                plans[index].plan = plans[index].plan.clone().clip_region(region);
            }
        }
    }
    require_distinct(plans, scope)
}

pub(crate) fn require_distinct(
    plans: &mut [crate::render::backend::PlacedText],
    scope: &FitScope,
) -> Result<(), PlanError> {
    fn display(plan: &super::ResolvedTextPlan) -> impl Iterator<Item = char> + '_ {
        plan.source
            .chars()
            .filter(|&ch| !super::ellipsis::bidi_control(ch))
    }
    for index in 0..plans.len() {
        let requested = plans[index]
            .plan
            .request
            .as_ref()
            .and_then(|request| request.instructions().overflow.as_ref());
        if plans[index].plan.lines.is_empty()
            || requested
                .and_then(|overflow| overflow.ellipsis.as_ref())
                .is_none_or(|ellipsis| ellipsis.distinct != Some(true))
        {
            continue;
        }
        if (0..index).any(|prior| {
            !plans[prior].plan.lines.is_empty()
                && display(&plans[prior].plan).eq(display(&plans[index].plan))
        }) {
            match requested.map_or(TextFailure::Error, |overflow| overflow.failure) {
                TextFailure::Error => {
                    return Err(scope
                        .failure(FitKind::Text, "text views become ambiguous after ellipsis")
                        .into())
                }
                TextFailure::Hide => plans[index].plan = plans[index].plan.clone().hide()?,
                TextFailure::Clip | TextFailure::Visible => {}
            }
        }
    }
    Ok(())
}

/// Required groups use the same transformed footprint as reduction and painting.
pub(crate) fn require_nonoverlap(
    plans: &mut [crate::render::backend::PlacedText],
    scope: &FitScope,
) -> Result<(), PlanError> {
    let mut occupied: Vec<(usize, PanelBox)> = Vec::new();
    crate::data::source::reserve_source(&mut occupied, plans.len())?;
    for index in 0..plans.len() {
        let allows = |plan: &super::ResolvedTextPlan| {
            plan.request.as_ref().is_some_and(|request| {
                request.instructions().overlap == Some(crate::TextOverlap::Allow)
            })
        };
        if allows(&plans[index].plan) {
            continue;
        }
        let bounds = super::placement::footprint(plans[index].placement, &plans[index].plan)?;
        for &(prior, previous) in &occupied {
            if !intersects(bounds, previous, 0.0) {
                continue;
            }
            let failure = |plan: &super::ResolvedTextPlan| {
                plan.request
                    .as_ref()
                    .and_then(|request| request.instructions().overflow.as_ref())
                    .map_or(TextFailure::Error, |overflow| overflow.failure)
            };
            match (failure(&plans[index].plan), failure(&plans[prior].plan)) {
                (TextFailure::Hide, _) => {
                    plans[index].plan = plans[index].plan.clone().hide()?;
                    break;
                }
                (_, TextFailure::Hide) => plans[prior].plan = plans[prior].plan.clone().hide()?,
                (TextFailure::Visible | TextFailure::Clip, _)
                | (_, TextFailure::Visible | TextFailure::Clip) => {}
                _ => {
                    return Err(scope
                        .failure(FitKind::OverlayCollision, "required text labels overlap")
                        .into())
                }
            }
        }
        occupied.push((index, bounds));
    }
    Ok(())
}

fn intersects(a: PanelBox, b: PanelBox, gap: f64) -> bool {
    a.width() > 0.0
        && a.height() > 0.0
        && b.width() > 0.0
        && b.height() > 0.0
        && a.x0 < b.x1 + gap
        && b.x0 < a.x1 + gap
        && a.y0 < b.y1 + gap
        && b.y0 < a.y1 + gap
}

#[expect(
    clippy::too_many_lines,
    reason = "one deterministic group transaction applies boundary filtering, protected endpoints, the chosen reduction and its terminal action"
)]
pub(crate) fn reduce(
    policy: &TextVisibility,
    eligible: &[usize],
    candidates: &[Candidate],
    region: PanelBox,
    failure: TextFailure,
    scope: &FitScope,
) -> Result<Vec<bool>, PlanError> {
    let mut visible = crate::data::source::collect_source(candidates.iter().map(|candidate| {
        let in_bounds = policy.boundary.is_none_or(|tolerance| {
            let margin = tolerance.get();
            candidate.bounds.x0 >= region.x0 - margin
                && candidate.bounds.x1 <= region.x1 + margin
                && candidate.bounds.y0 >= region.y0 - margin
                && candidate.bounds.y1 <= region.y1 + margin
        });
        Ok(in_bounds && policy.includes(candidate.index, eligible))
    }))?;
    let mut order = crate::data::source::collect_source(
        (0..candidates.len())
            .filter(|&index| visible[index])
            .map(Ok),
    )?;
    crate::data::source::sort_source_by(&mut order, |a, b| {
        candidates[*a]
            .position
            .total_cmp(&candidates[*b].position)
            .then_with(|| candidates[*a].index.cmp(&candidates[*b].index))
    });
    let protected = |index: usize| policy.protects(candidates[index].index, eligible);
    let overlaps = |a: usize, b: usize| {
        intersects(
            candidates[a].bounds,
            candidates[b].bounds,
            policy.separation.get(),
        )
    };
    match policy.program {
        VisibilityProgram::All | VisibilityProgram::Every { .. } => return Ok(visible),
        VisibilityProgram::Greedy => {
            let mut kept = crate::data::source::collect_source(
                order
                    .iter()
                    .copied()
                    .filter(|&index| protected(index))
                    .map(Ok),
            )?;
            for &index in &order {
                if protected(index) {
                    continue;
                }
                if kept.iter().all(|&prior| !overlaps(index, prior)) {
                    crate::data::source::reserve_source(&mut kept, 1)?;
                    kept.push(index);
                } else {
                    visible[index] = false;
                }
            }
        }
        VisibilityProgram::Parity => {
            while order
                .iter()
                .enumerate()
                .any(|(index, &a)| order[index + 1..].iter().any(|&b| overlaps(a, b)))
            {
                let mut ordinal = 0;
                let before = order.len();
                order.retain(|&index| {
                    if protected(index) {
                        return true;
                    }
                    ordinal += 1;
                    let keep = ordinal % 2 == 0;
                    visible[index] = keep;
                    keep
                });
                if order.len() == before {
                    break;
                }
            }
        }
    }
    let conflicts = order
        .iter()
        .copied()
        .filter(|&index| visible[index])
        .enumerate()
        .any(|(ordinal, a)| {
            order
                .iter()
                .copied()
                .filter(|&index| visible[index])
                .skip(ordinal + 1)
                .any(|b| overlaps(a, b))
        });
    if conflicts {
        match failure {
            TextFailure::Error => {
                return Err(scope
                    .failure(
                        FitKind::Text,
                        "protected text labels cannot satisfy group separation",
                    )
                    .into())
            }
            TextFailure::Hide => visible.fill(false),
            TextFailure::Clip | TextFailure::Visible => {}
        }
    }
    Ok(visible)
}
