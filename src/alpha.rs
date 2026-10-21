//! Owned byte alpha and explicit, eagerly materialized explanatory semantics.

use std::sync::Arc;

use crate::data::source::{collect_source, reserve_source, source_arc};
use crate::numeric::set::FormattedSet;
use crate::{CategoryValues, Error, NumericValues, Result};

mod materialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlphaSemantics {
    Undeclared,
    Continuous,
    Categorical,
    Threshold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlphaValues {
    name: Arc<String>,
    data: Arc<AlphaData>,
    declaration: Arc<AlphaDeclaration>,
}

#[derive(Debug, PartialEq, Eq)]
struct AlphaData {
    values: Vec<u8>,
    members: AlphaMembers,
}

#[derive(Debug, PartialEq, Eq)]
enum AlphaMembers {
    Undeclared,
    Numeric(Vec<u64>),
    Categorical(Vec<usize>),
    Threshold(Vec<(u64, usize)>),
}

/// Recipes retain the complete declaration independently of selected occurrences.
#[derive(Debug, PartialEq, Eq)]
enum AlphaDeclaration {
    Undeclared,
    Continuous {
        domain: [u64; 2],
        rule: ContinuousRule,
        examples: FormattedSet,
        alphas: Vec<u8>,
    },
    Categorical(Vec<(String, u8)>),
    Threshold {
        boundaries: FormattedSet,
        alphas: Vec<u8>,
        labels: Vec<String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum ContinuousRule {
    Linear((u8, u8)),
    Log10 {
        range: (u8, u8),
        coordinates: [u64; 2],
    },
    Custom(Vec<(u64, u8)>),
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the final consuming constructor contract is staged privately before public cutover"
)]
impl AlphaValues {
    pub(crate) fn new(
        name: impl Into<String>,
        values: impl IntoIterator<Item = u8>,
    ) -> Result<Self> {
        let name = validated_name(name.into())?;
        Self::from_parts(
            name,
            collect_input(values)?,
            AlphaMembers::Undeclared,
            AlphaDeclaration::Undeclared,
        )
    }

    pub(crate) fn linear(
        source: NumericValues,
        domain: (f64, f64),
        range: (u8, u8),
    ) -> Result<Self> {
        materialize::builtin(&source, domain, range, false)
    }

    pub(crate) fn log10(
        source: NumericValues,
        domain: (f64, f64),
        range: (u8, u8),
    ) -> Result<Self> {
        materialize::builtin(&source, domain, range, true)
    }

    pub(crate) fn continuous(
        source: NumericValues,
        domain: (f64, f64),
        f: impl FnMut(f64) -> u8,
    ) -> Result<Self> {
        materialize::continuous(&source, domain, f)
    }

    pub(crate) fn categorical(source: CategoryValues, f: impl FnMut(&str) -> u8) -> Result<Self> {
        materialize::categorical(&source, f)
    }

    pub(crate) fn threshold(
        source: NumericValues,
        boundaries: impl IntoIterator<Item = f64>,
        alphas: impl IntoIterator<Item = u8>,
    ) -> Result<Self> {
        materialize::threshold(&source, boundaries, alphas)
    }

    pub(crate) fn select(&self, indices: impl IntoIterator<Item = usize>) -> Result<Self> {
        let indices = collect_input(indices)?;
        let values = collect_source(indices.iter().map(|&index| {
            self.get(index).ok_or_else(|| {
                Error::Data(format!(
                    "alpha values {:?} selection index {index} is outside length {}",
                    self.name(),
                    self.len(),
                ))
            })
        }))?;
        let members = match &self.data.members {
            AlphaMembers::Undeclared => AlphaMembers::Undeclared,
            AlphaMembers::Numeric(members) => {
                AlphaMembers::Numeric(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
            AlphaMembers::Categorical(members) => {
                AlphaMembers::Categorical(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
            AlphaMembers::Threshold(members) => {
                AlphaMembers::Threshold(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
        };
        Ok(Self {
            name: Arc::clone(&self.name),
            data: source_arc(AlphaData { values, members })?,
            declaration: Arc::clone(&self.declaration),
        })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn len(&self) -> usize {
        self.data.values.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.data.values.is_empty()
    }

    pub(crate) fn get(&self, index: usize) -> Option<u8> {
        self.data.values.get(index).copied()
    }

    pub(crate) fn semantics(&self) -> AlphaSemantics {
        match self.declaration.as_ref() {
            AlphaDeclaration::Undeclared => AlphaSemantics::Undeclared,
            AlphaDeclaration::Continuous { .. } => AlphaSemantics::Continuous,
            AlphaDeclaration::Categorical(_) => AlphaSemantics::Categorical,
            AlphaDeclaration::Threshold { .. } => AlphaSemantics::Threshold,
        }
    }

    fn from_parts(
        name: String,
        values: Vec<u8>,
        members: AlphaMembers,
        declaration: AlphaDeclaration,
    ) -> Result<Self> {
        Ok(Self {
            name: source_arc(name)?,
            data: source_arc(AlphaData { values, members })?,
            declaration: source_arc(declaration)?,
        })
    }
}

fn validated_name(name: String) -> Result<String> {
    if name.is_empty() || name.trim() != name || name.chars().any(char::is_control) {
        return Err(Error::Data(
            "alpha source names must be nonempty, already trimmed and control-free".into(),
        ));
    }
    Ok(name)
}

fn numeric_key(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn collect_input<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<T>> {
    let values = values.into_iter();
    let mut retained = Vec::new();
    let (minimum, maximum) = values.size_hint();
    if maximum == Some(minimum) {
        reserve_source(&mut retained, minimum)?;
    }
    for value in values {
        if retained.len() == retained.capacity() {
            let additional = retained.capacity().max(1);
            reserve_source(&mut retained, additional)?;
        }
        retained.push(value);
    }
    Ok(retained)
}

#[cfg(test)]
mod tests;
