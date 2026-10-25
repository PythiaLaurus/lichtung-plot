//! Truthful explanation operands from the owned declaration and actual consumers.

use super::{collect_input, AlphaDeclaration, AlphaMembers, AlphaValues, ResolvedAlpha};
use crate::data::source::{collect_source, reserve_source};
use crate::{Error, Result};

pub(crate) struct AlphaExample<'a> {
    pub(crate) ordinal: usize,
    pub(crate) label: &'a str,
    pub(crate) alpha: u8,
}

impl AlphaValues {
    pub(crate) fn same_declaration(&self, other: &Self) -> bool {
        self.name == other.name && self.declaration == other.declaration
    }

    pub(crate) fn guide_examples(
        &self,
        consumed: impl IntoIterator<Item = ResolvedAlpha>,
    ) -> Result<Vec<AlphaExample<'_>>> {
        let mut members = Vec::new();
        let count = match self.declaration.as_ref() {
            AlphaDeclaration::Undeclared => return Ok(Vec::new()),
            AlphaDeclaration::Continuous { examples, .. } => examples.labels.len(),
            AlphaDeclaration::Categorical(table) => table.len(),
            AlphaDeclaration::Threshold { alphas, .. } => alphas.len(),
        };
        reserve_source(&mut members, count)?;
        members.resize(count, false);
        let mut present = false;
        for member in consumed {
            if self.get(member.index) != Some(member.alpha) {
                return Err(Error::Render(
                    "retained alpha member differs from its complete input".into(),
                ));
            }
            present = true;
            let ordinal = match &self.data.members {
                AlphaMembers::Categorical(values) => Some(values[member.index]),
                AlphaMembers::Threshold(values) => Some(values[member.index].1),
                AlphaMembers::Numeric(_) | AlphaMembers::Undeclared => None,
            };
            if let Some(ordinal) = ordinal {
                *members.get_mut(ordinal).ok_or_else(|| {
                    Error::Render("alpha declaration member is outside its table".into())
                })? = true;
            }
        }
        if !present {
            return Ok(Vec::new());
        }
        match self.declaration.as_ref() {
            AlphaDeclaration::Undeclared => unreachable!("undeclared alpha returned above"),
            AlphaDeclaration::Continuous {
                examples, alphas, ..
            } => collect_source(examples.labels.iter().zip(alphas).enumerate().map(
                |(ordinal, (label, &alpha))| {
                    Ok(AlphaExample {
                        ordinal,
                        label,
                        alpha,
                    })
                },
            )),
            AlphaDeclaration::Categorical(table) => collect_input(
                table
                    .iter()
                    .enumerate()
                    .filter(|(ordinal, _)| members[*ordinal])
                    .map(|(ordinal, (label, alpha))| AlphaExample {
                        ordinal,
                        label,
                        alpha: *alpha,
                    }),
            ),
            AlphaDeclaration::Threshold { labels, alphas, .. } => collect_input(
                labels
                    .iter()
                    .zip(alphas)
                    .enumerate()
                    .filter(|(ordinal, _)| members[*ordinal])
                    .map(|(ordinal, (label, &alpha))| AlphaExample {
                        ordinal,
                        label,
                        alpha,
                    }),
            ),
        }
    }
}
