//! Construction is the only application-function boundary. All consumers retain
//! final bytes, scientific membership, numeric text and complete mapping recipes.

use super::{
    collect_input, numeric_key, validated_name, AlphaDeclaration, AlphaMembers, AlphaValues,
    ContinuousRule,
};
use crate::data::source::{collect_source, copy_source_string, reserve_source, sort_source_by};
use crate::numeric::{mapping::normalized_fraction, set::guide_presentation};
use crate::{CategoryValues, Error, NumericValues, Result};

fn domain(domain: (f64, f64)) -> Result<(f64, f64)> {
    if !domain.0.is_finite() || !domain.1.is_finite() || domain.0 >= domain.1 {
        return Err(Error::Data(
            "alpha domain endpoints must be finite and strictly ascending".into(),
        ));
    }
    Ok((
        f64::from_bits(numeric_key(domain.0)),
        f64::from_bits(numeric_key(domain.1)),
    ))
}

fn numeric_members(source: &NumericValues, positive: bool) -> Result<Vec<u64>> {
    collect_source((0..source.len()).map(|index| {
        let value = source.get(index).flatten().ok_or_else(|| {
            Error::Data(format!(
                "alpha source {:?} is missing at index {index}",
                source.name()
            ))
        })?;
        if !value.is_finite() || (positive && value <= 0.0) {
            return Err(Error::Data(format!(
                "alpha source {:?} at index {index} must be {}finite",
                source.name(),
                if positive {
                    "strictly positive and "
                } else {
                    ""
                },
            )));
        }
        Ok(numeric_key(value))
    }))
}

fn representatives(domain: (f64, f64)) -> Result<Vec<f64>> {
    let midpoint = f64::midpoint(domain.0, domain.1);
    let mut values = collect_source(
        [
            domain.0,
            f64::midpoint(domain.0, midpoint),
            midpoint,
            f64::midpoint(midpoint, domain.1),
            domain.1,
        ]
        .into_iter()
        .map(|value| Ok(f64::from_bits(numeric_key(value)))),
    )?;
    values.dedup_by(|a, b| a.to_bits() == b.to_bits());
    Ok(values)
}

fn clamped_key(member: u64, domain: (f64, f64)) -> u64 {
    numeric_key(f64::from_bits(member).clamp(domain.0, domain.1))
}

fn evaluations(
    members: &[u64],
    representatives: &[f64],
    domain: (f64, f64),
    mut evaluate: impl FnMut(f64) -> Result<u8>,
) -> Result<Vec<(u64, u8)>> {
    let mut keys = collect_source(
        members
            .iter()
            .map(|&member| Ok(clamped_key(member, domain))),
    )?;
    reserve_source(&mut keys, representatives.len())?;
    keys.extend(representatives.iter().map(|&value| numeric_key(value)));
    sort_source_by(&mut keys, |a, b| {
        f64::from_bits(*a).total_cmp(&f64::from_bits(*b))
    });
    keys.dedup();
    collect_source(
        keys.iter()
            .map(|&key| Ok((key, evaluate(f64::from_bits(key))?))),
    )
}

fn evaluated(table: &[(u64, u8)], key: u64) -> u8 {
    let index = table
        .binary_search_by(|(value, _)| f64::from_bits(*value).total_cmp(&f64::from_bits(key)))
        .expect("validated materialized alpha coordinate");
    table[index].1
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the rounded finite integer is checked against the complete byte range before conversion"
)]
fn quantize(fraction: f64, range: (u8, u8)) -> Result<u8> {
    let value = (f64::from(range.0) + fraction * (f64::from(range.1) - f64::from(range.0)))
        .round_ties_even();
    if !value.is_finite() || !(0.0..=255.0).contains(&value) {
        return Err(Error::Data("mapped alpha is outside the byte range".into()));
    }
    Ok(value as u8)
}

pub(super) fn builtin(
    source: &NumericValues,
    raw_domain: (f64, f64),
    range: (u8, u8),
    logarithmic: bool,
) -> Result<AlphaValues> {
    let domain = domain(raw_domain)?;
    let name = validated_name(copy_source_string(source.name())?)?;
    let coordinates = if logarithmic {
        if domain.0 <= 0.0 {
            return Err(Error::Data(
                "log10 alpha domain must be strictly positive".into(),
            ));
        }
        let coordinates = (
            crate::numeric::log10(domain.0)?,
            crate::numeric::log10(domain.1)?,
        );
        if !coordinates.0.is_finite()
            || !coordinates.1.is_finite()
            || coordinates.0 >= coordinates.1
        {
            return Err(Error::Data(
                "log10 alpha domain coordinates must be finite and distinguishable".into(),
            ));
        }
        coordinates
    } else {
        domain
    };
    let members = numeric_members(source, logarithmic)?;
    let examples = guide_presentation(&representatives(domain)?, None)?;
    let example_values =
        collect_source(examples.members.iter().map(|&key| Ok(f64::from_bits(key))))?;
    let table = evaluations(&members, &example_values, domain, |value| {
        let fraction = if value <= domain.0 {
            0.0
        } else if value >= domain.1 {
            1.0
        } else {
            let coordinate = if logarithmic {
                crate::numeric::log10(value)?
            } else {
                value
            };
            normalized_fraction(coordinates, coordinate)?
        };
        quantize(fraction, range)
    })?;
    let rule = if logarithmic {
        ContinuousRule::Log10 {
            range,
            coordinates: [numeric_key(coordinates.0), numeric_key(coordinates.1)],
        }
    } else {
        ContinuousRule::Linear(range)
    };
    finish_continuous(name, members, domain, |_| rule, examples, table)
}

pub(super) fn continuous(
    source: &NumericValues,
    raw_domain: (f64, f64),
    mut f: impl FnMut(f64) -> u8,
) -> Result<AlphaValues> {
    let domain = domain(raw_domain)?;
    let name = validated_name(copy_source_string(source.name())?)?;
    let members = numeric_members(source, false)?;
    let representatives = representatives(domain)?;
    let examples = guide_presentation(&representatives, None)?;
    let table = evaluations(&members, &representatives, domain, |value| Ok(f(value)))?;
    finish_continuous(
        name,
        members,
        domain,
        ContinuousRule::Custom,
        examples,
        table,
    )
}

fn finish_continuous(
    name: String,
    members: Vec<u64>,
    domain: (f64, f64),
    rule: impl FnOnce(Vec<(u64, u8)>) -> ContinuousRule,
    examples: crate::numeric::set::FormattedSet,
    table: Vec<(u64, u8)>,
) -> Result<AlphaValues> {
    let values = collect_source(
        members
            .iter()
            .map(|&member| Ok(evaluated(&table, clamped_key(member, domain)))),
    )?;
    let alphas = collect_source(
        examples
            .members
            .iter()
            .map(|&key| Ok(evaluated(&table, key))),
    )?;
    AlphaValues::from_parts(
        name,
        values,
        AlphaMembers::Numeric(members),
        AlphaDeclaration::Continuous {
            domain: [numeric_key(domain.0), numeric_key(domain.1)],
            rule: rule(table),
            examples,
            alphas,
        },
    )
}

pub(super) fn categorical(
    source: &CategoryValues,
    mut f: impl FnMut(&str) -> u8,
) -> Result<AlphaValues> {
    let name = validated_name(copy_source_string(source.name())?)?;
    // Validate the complete source before invoking any application function.
    let raw = collect_source((0..source.len()).map(|index| {
        source.get(index).flatten().ok_or_else(|| {
            Error::Data(format!("alpha source {name:?} is missing at index {index}"))
        })
    }))?;
    let mut indices = std::collections::HashMap::new();
    indices
        .try_reserve(source.levels().len())
        .map_err(|error| Error::Render(format!("alpha category index reservation: {error}")))?;
    for (index, category) in source.levels().iter().enumerate() {
        indices.insert(category.as_str(), index);
    }
    let members = collect_source(raw.iter().map(|category| Ok(indices[category])))?;
    let entries = collect_source(source.levels().iter().map(|category| {
        let retained = copy_source_string(category)?;
        Ok((retained, f(category)))
    }))?;
    let values = collect_source(members.iter().map(|&index| Ok(entries[index].1)))?;
    AlphaValues::from_parts(
        name,
        values,
        AlphaMembers::Categorical(members),
        AlphaDeclaration::Categorical(entries),
    )
}

pub(super) fn threshold(
    source: &NumericValues,
    boundaries: impl IntoIterator<Item = f64>,
    alphas: impl IntoIterator<Item = u8>,
) -> Result<AlphaValues> {
    let name = validated_name(copy_source_string(source.name())?)?;
    let boundaries = collect_input(boundaries)?;
    if boundaries.iter().any(|value| !value.is_finite())
        || boundaries.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(Error::Data(
            "alpha thresholds must be finite and strictly ascending".into(),
        ));
    }
    let alphas = collect_input(alphas)?;
    if boundaries.len().checked_add(1) != Some(alphas.len()) {
        return Err(Error::Data(
            "alpha thresholds require one more byte than boundaries".into(),
        ));
    }
    let source_members = numeric_members(source, false)?;
    let members = collect_source(source_members.iter().map(|&key| {
        Ok((
            key,
            boundaries.partition_point(|&bound| f64::from_bits(key) >= bound),
        ))
    }))?;
    let values = collect_source(members.iter().map(|&(_, index)| Ok(alphas[index])))?;
    let boundaries = collect_source(
        boundaries
            .iter()
            .map(|&value| Ok(f64::from_bits(numeric_key(value)))),
    )?;
    let boundaries = guide_presentation(&boundaries, None)?;
    let mut labels = Vec::new();
    reserve_source(&mut labels, alphas.len())?;
    if boundaries.labels.is_empty() {
        labels.push(copy_source_string("all values")?);
    } else {
        labels.push(format!("< {}", boundaries.labels[0]));
        for pair in boundaries.labels.windows(2) {
            labels.push(format!("[{}, {})", pair[0], pair[1]));
        }
        labels.push(format!(
            ">= {}",
            boundaries.labels.last().expect("nonempty bounds")
        ));
    }
    AlphaValues::from_parts(
        name,
        values,
        AlphaMembers::Threshold(members),
        AlphaDeclaration::Threshold {
            boundaries,
            alphas,
            labels,
        },
    )
}
