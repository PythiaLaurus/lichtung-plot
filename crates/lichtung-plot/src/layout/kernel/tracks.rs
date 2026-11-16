//! Quantized track allocation for active sequence and later Grid owners.

use crate::layout::fit::{FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
use crate::layout::ComponentAddress;
use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TrackSpec {
    pub min: i64,
    pub preferred: i64,
    pub max: Option<i64>,
    pub flexible: bool,
}

#[derive(Debug)]
pub(crate) struct TrackAllocation {
    pub sizes: Vec<i64>,
    pub trailing_slack: i64,
}

/// One child's requirement on a Grid axis. The caller provides the canonical
/// component address, independent of sparse coordinate insertion order.
pub(crate) struct GridRequirement<'a> {
    pub start: usize,
    pub span: usize,
    pub hard: i64,
    pub preferred: i64,
    pub fill: bool,
    pub address: &'a ComponentAddress,
}

fn admitted<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Render("track allocation failed".into()))?;
    Ok(values)
}

fn checked(value: i128) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Render("track length overflowed layout quanta".into()))
}

/// Allocate once in canonical order. Excess after all maxima is visible parent
/// slack, while unsatisfied hard requirements are typed fit misses.
#[expect(
    clippy::too_many_lines,
    reason = "one proportional shrink and one capped surplus distribution share checked track arithmetic"
)]
pub(crate) fn allocate_tracks(
    specs: &[TrackSpec],
    total: i64,
    insets: [i64; 2],
    gap: i64,
    require_full: bool,

    scope: &FitScope,
) -> Result<FitOutcome<TrackAllocation>, Error> {
    if total < 0 || gap < 0 || insets.into_iter().any(|inset| inset < 0) {
        return Err(Error::Render(
            "track constraints must be nonnegative".into(),
        ));
    }
    let mut sizes = admitted(specs.len())?;
    let mut flexible = admitted(specs.len())?;
    let mut inflexible_total = 0_i128;
    let mut flexible_min = 0_i128;
    let mut flexible_preferred = 0_i128;
    for (index, spec) in specs.iter().enumerate() {
        if spec.min < 0 || spec.preferred < spec.min || spec.max.is_some_and(|max| max < spec.min) {
            return Err(Error::Render(
                "inconsistent track minimum, preference, or maximum".into(),
            ));
        }
        let preferred = spec
            .max
            .map_or(spec.preferred, |max| spec.preferred.min(max));
        sizes.push(preferred);
        if spec.flexible {
            flexible.push(index);
            flexible_min += i128::from(spec.min);
            flexible_preferred += i128::from(preferred);
        } else {
            inflexible_total += i128::from(preferred);
        }
    }
    let gaps = i128::try_from(specs.len().saturating_sub(1))
        .map_err(|_| Error::Render("track gap count overflowed".into()))?
        .checked_mul(i128::from(gap))
        .ok_or_else(|| Error::Render("track gaps overflowed".into()))?;
    let remainder =
        i128::from(total) - i128::from(insets[0]) - i128::from(insets[1]) - gaps - inflexible_total;
    if remainder < 0 || remainder < flexible_min {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "assigned tracks cannot preserve their hard requirements",
        )));
    }
    let available = checked(remainder)?;
    if remainder < flexible_preferred {
        let deficit = checked(flexible_preferred - remainder)?;
        let capacity = checked(flexible_preferred - flexible_min)?;
        let mut remainders = admitted(flexible.len())?;
        let mut spent = 0_i64;
        for &index in &flexible {
            let cap = sizes[index] - specs[index].min;
            let numerator = u128::try_from(deficit)
                .unwrap()
                .checked_mul(u128::try_from(cap).unwrap())
                .ok_or_else(|| Error::Render("proportional shrink overflowed".into()))?;
            let denominator = u128::try_from(capacity).unwrap();
            let whole = i64::try_from(numerator / denominator)
                .map_err(|_| Error::Render("proportional shrink overflowed".into()))?;
            sizes[index] -= whole;
            spent = spent
                .checked_add(whole)
                .ok_or_else(|| Error::Render("proportional shrink overflowed".into()))?;
            remainders.push((numerator % denominator, index));
        }

        remainders.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let mut leftover = deficit - spent;
        for &(_, index) in &remainders {
            if leftover == 0 {
                break;
            }
            if sizes[index] > specs[index].min {
                sizes[index] -= 1;
                leftover -= 1;
            }
        }
        if leftover != 0 {
            return Err(Error::Render(
                "proportional shrink left undistributed quanta".into(),
            ));
        }
    } else {
        let mut surplus = checked(remainder - flexible_preferred)?;
        let mut active = admitted(flexible.len())?;
        active.extend(flexible);
        while surplus > 0 && !active.is_empty() {
            let count = i64::try_from(active.len())
                .map_err(|_| Error::Render("track count overflowed".into()))?;
            let quotient = surplus / count;
            let remainder = surplus % count;
            let mut spent = 0_i64;
            for (order, &index) in active.iter().enumerate() {
                let order = i64::try_from(order)
                    .map_err(|_| Error::Render("track index overflowed".into()))?;
                let grant = quotient + i64::from(order < remainder);
                let headroom = specs[index]
                    .max
                    .map_or(i64::MAX - sizes[index], |max| max - sizes[index]);
                let grant = grant.min(headroom);
                sizes[index] = sizes[index]
                    .checked_add(grant)
                    .ok_or_else(|| Error::Render("track surplus overflowed".into()))?;
                spent = spent
                    .checked_add(grant)
                    .ok_or_else(|| Error::Render("track surplus overflowed".into()))?;
            }
            surplus -= spent;
            active.retain(|&index| specs[index].max.is_none_or(|max| sizes[index] < max));
        }
        if require_full && surplus > 0 {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::TrackOrResidual,
                "assigned flexible tracks reached their maxima before filling the frame",
            )));
        }
        return Ok(FitOutcome::Measured(TrackAllocation {
            sizes,
            trailing_slack: surplus,
        }));
    }
    let trailing_slack = available
        - sizes
            .iter()
            .enumerate()
            .filter(|(index, _)| specs[*index].flexible)
            .map(|(_, size)| *size)
            .sum::<i64>();
    Ok(FitOutcome::Measured(TrackAllocation {
        sizes,
        trailing_slack,
    }))
}

/// Raise only the flexible tracks covered by one span. Return false when the
/// span has no remaining headroom; hard failure and capped soft preference are
/// distinguished by the caller.
fn raise_span(
    tracks: &mut [TrackSpec],
    start: usize,
    span: usize,
    target: i64,
    gap: i64,
    hard: bool,
) -> Result<bool, Error> {
    let mut current = i128::from(gap)
        * i128::try_from(span - 1).map_err(|_| Error::Render("Grid span overflowed".into()))?;
    for track in &tracks[start..start + span] {
        current += i128::from(if hard { track.min } else { track.preferred });
    }
    let mut deficit = i128::from(target) - current;
    if deficit <= 0 {
        return Ok(true);
    }
    let mut active = admitted(span)?;
    active.extend((start..start + span).filter(|&index| tracks[index].flexible));
    while deficit > 0 && !active.is_empty() {
        let count = i128::try_from(active.len())
            .map_err(|_| Error::Render("Grid span count overflowed".into()))?;
        let quotient = deficit / count;
        let remainder = deficit % count;
        let mut spent = 0_i128;
        for (order, &index) in active.iter().enumerate() {
            let proposal = quotient
                + i128::from(
                    i128::try_from(order)
                        .map_err(|_| Error::Render("Grid span index overflowed".into()))?
                        < remainder,
                );
            let track = &mut tracks[index];
            let size = if hard {
                &mut track.min
            } else {
                &mut track.preferred
            };
            let headroom = i128::from(track.max.unwrap_or(i64::MAX) - *size);
            let grant = proposal.min(headroom);
            *size = checked(i128::from(*size) + grant)?;
            spent += grant;
        }
        deficit -= spent;
        active.retain(|&index| {
            let track = &tracks[index];
            let size = if hard { track.min } else { track.preferred };
            track.max.is_none_or(|max| size < max)
        });
    }
    Ok(deficit == 0)
}

/// One Grid dimension. Call columns before rows after measuring the children at
/// their assigned column widths. Span order is length, then canonical address.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "canonical topology admission, hard/preferred spans, allocation, and hard recheck form one Grid axis"
)]
pub(crate) fn allocate_grid_axis(
    base: &[TrackSpec],
    children: &[GridRequirement<'_>],
    total: impl Into<Option<i64>>,
    insets: [i64; 2],
    gap: i64,
    declared_em: Option<i64>,
    require_full: bool,

    scope: &FitScope,
) -> Result<FitOutcome<TrackAllocation>, Error> {
    if base.is_empty() || gap < 0 || declared_em.is_some_and(|em| em < 0) {
        return Err(Error::Render(
            "Grid axis has invalid topology or spacing".into(),
        ));
    }
    for track in base {
        if track.min < 0
            || track.preferred < track.min
            || track.max.is_some_and(|max| max < track.min)
        {
            return Err(Error::Render(
                "Grid track declaration is inconsistent".into(),
            ));
        }
    }
    let mut order = admitted(children.len())?;
    for (index, child) in children.iter().enumerate() {
        if child.span == 0
            || child
                .start
                .checked_add(child.span)
                .is_none_or(|end| end > base.len())
            || child.hard < 0
            || child.preferred < child.hard
        {
            return Err(Error::Render(
                "Grid child has invalid span or requirement".into(),
            ));
        }
        order.push(index);
    }

    order.sort_unstable_by(|&left, &right| children[left].address.cmp(children[right].address));
    if order
        .windows(2)
        .any(|pair| children[pair[0]].address == children[pair[1]].address)
    {
        return Err(Error::Render("Grid child address is duplicated".into()));
    }
    order.sort_unstable_by(|&left, &right| {
        children[left]
            .span
            .cmp(&children[right].span)
            .then_with(|| children[left].address.cmp(children[right].address))
    });
    let mut tracks = admitted(base.len())?;
    tracks.extend_from_slice(base);
    if let Some(em) = declared_em {
        for track in &mut tracks {
            track.min = track.min.max(1);
            track.preferred = track.preferred.max(track.min).max(em);
            track.flexible = true;
        }
    }
    for child in children {
        if child.fill {
            for track in &mut tracks[child.start..child.start + child.span] {
                track.flexible = true;
            }
        }
    }
    for &index in &order {
        let child = &children[index];
        if child.span == 1 {
            let track = &mut tracks[child.start];
            if track.max.is_some_and(|max| child.hard > max) {
                return Ok(FitOutcome::Miss(FitScope::new(child.address, 0).failure(
                    FitKind::TrackOrResidual,
                    "Grid child hard minimum exceeds its track maximum",
                )));
            }
            track.min = track.min.max(child.hard);
        } else if !raise_span(&mut tracks, child.start, child.span, child.hard, gap, true)? {
            return Ok(FitOutcome::Miss(FitScope::new(child.address, 0).failure(
                FitKind::TrackOrResidual,
                "Grid span has no flexible hard headroom",
            )));
        }
    }
    for (track, original) in tracks.iter_mut().zip(base) {
        if track.max.is_some_and(|max| track.min > max) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::TrackOrResidual,
                "Grid topology exceeds an explicit track maximum",
            )));
        }
        track.preferred = track.min.max(track.preferred).max(original.preferred);
        if let Some(max) = track.max {
            track.preferred = track.preferred.min(max);
        }
    }
    for &index in &order {
        let child = &children[index];
        if child.span == 1 {
            let track = &mut tracks[child.start];
            track.preferred = track.preferred.max(child.preferred).max(track.min);
            if let Some(max) = track.max {
                track.preferred = track.preferred.min(max);
            }
        } else {
            // Preferences cap at maxima; infeasible soft surplus is not a hard miss.
            let _ = raise_span(
                &mut tracks,
                child.start,
                child.span,
                child.preferred,
                gap,
                false,
            )?;
        }
    }
    let total = match total.into() {
        Some(total) => total,
        None => checked(
            tracks
                .iter()
                .map(|track| i128::from(track.preferred))
                .sum::<i128>()
                + i128::from(insets[0])
                + i128::from(insets[1])
                + i128::from(gap)
                    * i128::try_from(tracks.len().saturating_sub(1))
                        .map_err(|_| Error::Render("natural Grid gap count overflowed".into()))?,
        )?,
    };
    let result = allocate_tracks(&tracks, total, insets, gap, require_full, scope)?;
    if let FitOutcome::Measured(allocation) = &result {
        for &index in &order {
            let child = &children[index];
            let width = i128::from(gap)
                * i128::try_from(child.span - 1)
                    .map_err(|_| Error::Render("Grid span overflowed".into()))?
                + allocation.sizes[child.start..child.start + child.span]
                    .iter()
                    .map(|&size| i128::from(size))
                    .sum::<i128>();
            if width < i128::from(child.hard) {
                return Ok(FitOutcome::Miss(FitScope::new(child.address, 0).failure(
                    FitKind::TrackOrResidual,
                    "assigned Grid tracks violate a spanning hard requirement",
                )));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::AddressRole;

    fn grid(
        address: &ComponentAddress,
        start: usize,
        span: usize,
        hard: i64,
        preferred: i64,
    ) -> GridRequirement<'_> {
        GridRequirement {
            start,
            span,
            hard,
            preferred,
            fill: false,
            address,
        }
    }

    #[test]
    fn declared_sparse_tracks_remain_noncollapsing_while_empty_slots_stay_zero() {
        let FitOutcome::Measured(declared) = allocate_grid_axis(
            &[TrackSpec {
                min: 0,
                preferred: 0,
                max: None,
                flexible: false,
            }; 3],
            &[],
            12,
            [0, 0],
            0,
            Some(4),
            true,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("declared trailing tracks fill");
        };
        assert_eq!(declared.sizes, [4, 4, 4]);
        let empty = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 0)
            .unwrap();
        let FitOutcome::Measured(unoccupied) = allocate_grid_axis(
            &[TrackSpec {
                min: 0,
                preferred: 0,
                max: None,
                flexible: false,
            }; 2],
            &[grid(&empty, 0, 1, 0, 0)],
            5,
            [0, 0],
            0,
            None,
            false,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("empty slots do not allocate content");
        };
        assert_eq!(unoccupied.sizes, [0, 0]);
        assert_eq!(unoccupied.trailing_slack, 5);

        let filled = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 2)
            .unwrap();
        let FitOutcome::Measured(sparse) = allocate_grid_axis(
            &[TrackSpec {
                min: 0,
                preferred: 0,
                max: None,
                flexible: false,
            }; 4],
            &[grid(&filled, 2, 1, 7, 7)],
            10,
            [0, 0],
            0,
            Some(4),
            true,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("sparse index two retains its seven-quanta minimum");
        };
        assert_eq!(sparse.sizes, [1, 1, 7, 1]);
    }
}
