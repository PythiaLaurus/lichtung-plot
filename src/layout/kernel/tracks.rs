//! Quantized track allocation for active sequence and later Grid owners.

use crate::font::canonical::OperationGenerationResources;
use crate::layout::fit::{FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
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

fn admitted<T>(resources: &OperationGenerationResources, count: usize) -> Result<Vec<T>, Error> {
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Render("track storage overflowed".into()))?;
    resources
        .claim_bytes(bytes)
        .map_err(|error| Error::Render(format!("track allocation admission: {error:?}")))?;
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
    resources: &OperationGenerationResources,
    scope: &FitScope,
) -> Result<FitOutcome<TrackAllocation>, Error> {
    if total < 0 || gap < 0 || insets.into_iter().any(|inset| inset < 0) {
        return Err(Error::Render(
            "track constraints must be nonnegative".into(),
        ));
    }
    let mut sizes = admitted(resources, specs.len())?;
    let mut flexible = admitted(resources, specs.len())?;
    let mut inflexible_total = 0_i128;
    let mut flexible_min = 0_i128;
    let mut flexible_preferred = 0_i128;
    for (index, spec) in specs.iter().enumerate() {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("track specification admission: {error:?}")))?;
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
        let mut remainders = admitted(resources, flexible.len())?;
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
        resources
            .claim_visits(
                u128::try_from(remainders.len())
                    .map_err(|_| Error::Render("track sort count overflowed".into()))?
                    .checked_mul(u128::from(
                        remainders.len().checked_ilog2().unwrap_or(0) + 1,
                    ))
                    .ok_or_else(|| Error::Render("track sort count overflowed".into()))?,
            )
            .map_err(|error| Error::Render(format!("track shrink sort admission: {error:?}")))?;
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
        let mut active = admitted(resources, flexible.len())?;
        active.extend(flexible);
        while surplus > 0 && !active.is_empty() {
            resources
                .claim_visits(
                    u128::try_from(active.len())
                        .map_err(|_| Error::Render("track surplus count overflowed".into()))?
                        .checked_mul(2)
                        .ok_or_else(|| Error::Render("track surplus count overflowed".into()))?,
                )
                .map_err(|error| Error::Render(format!("track surplus admission: {error:?}")))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn flex(min: i64, preferred: i64, max: Option<i64>) -> TrackSpec {
        TrackSpec {
            min,
            preferred,
            max,
            flexible: true,
        }
    }

    fn alloc(specs: &[TrackSpec], total: i64, require_full: bool) -> FitOutcome<TrackAllocation> {
        allocate_tracks(
            specs,
            total,
            [0; 2],
            0,
            require_full,
            &OperationGenerationResources::new(),
            &FitScope::root(),
        )
        .unwrap()
    }

    #[test]
    fn shrink_uses_one_largest_remainder_tie_by_track_index() {
        let FitOutcome::Measured(allocation) =
            alloc(&[flex(2, 4, None), flex(2, 4, None)], 7, true)
        else {
            panic!("the hard requirements fit");
        };
        assert_eq!(allocation.sizes, [3, 4]);
        assert_eq!(allocation.trailing_slack, 0);
        let FitOutcome::Measured(allocation) =
            alloc(&[flex(1, 5, None), flex(2, 4, None)], 6, true)
        else {
            panic!("proportional shrink must fit");
        };
        assert_eq!(allocation.sizes, [3, 3]);
        assert!(matches!(
            alloc(&[flex(2, 4, None), flex(2, 4, None)], 3, false),
            FitOutcome::Miss(_)
        ));
    }

    #[test]
    fn capped_surplus_redistributes_once_then_retains_trailing_slack() {
        let FitOutcome::Measured(allocation) =
            alloc(&[flex(1, 2, Some(3)), flex(1, 2, None)], 11, true)
        else {
            panic!("the second track absorbs the capped surplus");
        };
        assert_eq!(allocation.sizes, [3, 8]);
        let specs = [flex(1, 2, Some(3)), flex(1, 2, Some(3))];
        let FitOutcome::Measured(allocation) = alloc(&specs, 11, false) else {
            panic!("a nonfill owner may retain slack");
        };
        assert_eq!(allocation.sizes, [3, 3]);
        assert_eq!(allocation.trailing_slack, 5);
        assert!(matches!(alloc(&specs, 11, true), FitOutcome::Miss(_)));
    }

    #[test]
    fn inflexible_sequence_occupies_its_content_height_not_the_parent() {
        let specs = [
            TrackSpec {
                min: 20,
                preferred: 20,
                max: None,
                flexible: false,
            },
            TrackSpec {
                min: 30,
                preferred: 30,
                max: None,
                flexible: false,
            },
        ];
        let FitOutcome::Measured(allocation) = allocate_tracks(
            &specs,
            200,
            [16, 16],
            12,
            false,
            &OperationGenerationResources::new(),
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("sequence content fits");
        };
        assert_eq!(allocation.sizes, [20, 30]);
        assert_eq!(allocation.trailing_slack, 106);
        assert!(matches!(
            allocate_tracks(
                &specs,
                93,
                [16, 16],
                12,
                false,
                &OperationGenerationResources::new(),
                &FitScope::root()
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));
    }

    #[test]
    fn allocation_refuses_one_visit_or_byte_below_its_exact_admitted_cost() {
        let specs = [flex(1, 4, Some(6)), flex(1, 3, None)];
        let run = |limit: Option<(u128, u64)>| {
            let resources = OperationGenerationResources::new();
            if let Some((visits, bytes)) = limit {
                resources.set_test_limits(visits, bytes);
            }
            let result = allocate_tracks(&specs, 8, [0, 0], 0, true, &resources, &FitScope::root());
            (resources, result)
        };
        let (resources, result) = run(None);
        assert!(matches!(result.unwrap(), FitOutcome::Measured(_)));
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        assert!(visits > 0 && bytes > 0);
        assert!(run(Some((visits - 1, bytes))).1.is_err());
        assert!(run(Some((visits, bytes - 1))).1.is_err());
        assert!(matches!(
            run(Some((visits, bytes))).1.unwrap(),
            FitOutcome::Measured(_)
        ));
    }
}
