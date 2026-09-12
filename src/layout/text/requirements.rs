//! Canonically ordered text and font-pair preparation requirements.

use std::borrow::Cow;

use crate::font::canonical::{
    self, CensusSource, FamilyPreference, FontPairPreparationRequest, PreparedFontEnvironment,
    RequestedWeight,
};
use crate::layout::kernel::ComponentAddress;
use crate::theme::TextAppearance;
use crate::{Error, Plot, PlotKind, TextWeight};

const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;
use crate::layout::NUMERIC_ALPHABET as AUTOMATIC_NUMERIC_ALPHABET;
const HEATMAP_ELISION_ALPHABET: &[char] = &['.'];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PreparedTextRole {
    Title,
    Subtitle,
    Caption,
    AxisTitle,
    TickLabel,
    SignificanceLabel,
    LegendTitle,
    LegendLabel,
    Header,
    Annotation,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedTextRequirement {
    pub(crate) component_address: ComponentAddress,
    pub(crate) text_role: PreparedTextRole,
    pub(crate) local_index: u32,
    pub(crate) source: String,
    pub(crate) generated_alphabet: Box<[char]>,
    pub(crate) stack_source: String,
    pub(crate) stack: Vec<FamilyPreference>,
    pub(crate) weight: TextWeight,
    pub(crate) pair_ordinal: u32,
    source_work_admitted: bool,
}

impl PreparedTextRequirement {
    #[expect(
        clippy::too_many_arguments,
        reason = "admission receives the complete borrowed text identity"
    )]
    pub(crate) fn push_into<'a>(
        requirements: &mut Vec<Self>,
        address: ComponentAddress,
        role: PreparedTextRole,
        local_index: u32,
        source: impl Into<CensusSource<'a>>,
        generated_alphabet: &[char],
        appearance: &TextAppearance,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), PreparedTextCensusError> {
        #[cfg(not(test))]
        if prepared_stage.is_none() {
            return Err(PreparedTextCensusError::MissingPreparedStage);
        }
        if let Some(stage) = prepared_stage.as_deref_mut() {
            let mut present = false;
            for entry in requirements.iter() {
                stage.font_resources().claim_visits(1)?;
                if entry.component_address == address {
                    present = true;
                    break;
                }
            }
            if !present {
                stage
                    .claim_topology_slots(1)
                    .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?;
            }
        }
        reserve_census_vec(
            requirements,
            1,
            "prepared text requirements",
            prepared_stage.as_deref_mut(),
        )?;
        requirements.push(Self::new_with_stage(
            address,
            role,
            local_index,
            source,
            generated_alphabet,
            appearance,
            prepared_stage,
        )?);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn new<'a>(
        component_address: ComponentAddress,
        text_role: PreparedTextRole,
        local_index: u32,
        source: impl Into<CensusSource<'a>>,
        generated_alphabet: &[char],
        appearance: &TextAppearance,
    ) -> Result<Self, PreparedTextCensusError> {
        Self::new_with_stage(
            component_address,
            text_role,
            local_index,
            source,
            generated_alphabet,
            appearance,
            None,
        )
    }

    fn new_with_stage<'a>(
        component_address: ComponentAddress,
        text_role: PreparedTextRole,
        local_index: u32,
        source: impl Into<CensusSource<'a>>,
        generated_alphabet: &[char],
        appearance: &TextAppearance,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, PreparedTextCensusError> {
        if generated_alphabet.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(PreparedTextCensusError::GeneratedAlphabetOrder);
        }
        let source = source.into();
        // Source work is admitted before either copying or formatting output.
        if let Some(stage) = prepared_stage.as_deref_mut() {
            let work = appearance
                .stack
                .chars()
                .count()
                .checked_add(generated_alphabet.len())
                .ok_or(PreparedTextCensusError::Arithmetic(
                    "borrowed text census work",
                ))?;
            stage.font_resources().claim_visits(
                u64::try_from(work).map_err(|_| {
                    PreparedTextCensusError::Arithmetic("borrowed text census work")
                })?,
            )?;
        }
        let (source, formatted) = match source {
            CensusSource::Text(source) => {
                if let Some(stage) = prepared_stage.as_deref_mut() {
                    let work = source
                        .chars()
                        .count()
                        .checked_add(canonical::selection_units(&source).count())
                        .and_then(|work| work.checked_add(usize::from(source.contains('\u{00AD}'))))
                        .ok_or(PreparedTextCensusError::Arithmetic(
                            "borrowed text census work",
                        ))?;
                    stage
                        .font_resources()
                        .claim_visits(u64::try_from(work).map_err(|_| {
                            PreparedTextCensusError::Arithmetic("borrowed text census work")
                        })?)?;
                }
                (source, false)
            }
            CensusSource::Formatted(arguments) => (
                Cow::Owned(canonical::format_census_source(
                    arguments,
                    prepared_stage.as_deref_mut(),
                )?),
                true,
            ),
        };
        let retained_source = match source {
            Cow::Borrowed(source) => {
                let mut retained = String::new();
                reserve_census_string(
                    &mut retained,
                    source.len(),
                    "prepared text source",
                    prepared_stage.as_deref_mut(),
                )?;
                retained.push_str(source);
                retained
            }
            Cow::Owned(mut source) => {
                // Semantic display producers already own this string; transfer it,
                // rather than allocating a second census copy.
                if let Some(stage) = prepared_stage.as_deref_mut().filter(|_| !formatted) {
                    stage
                        .font_resources()
                        .claim_bytes(source.capacity() as u64)?;
                    stage
                        .reserve_prepared_string(&mut source, 0, "prepared semantic text source")
                        .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?;
                }
                source
            }
        };
        let mut retained_alphabet = Vec::new();
        reserve_census_vec(
            &mut retained_alphabet,
            generated_alphabet.len(),
            "prepared generated alphabet",
            prepared_stage.as_deref_mut(),
        )?;
        let mut stack_source = String::new();
        reserve_census_string(
            &mut stack_source,
            appearance.stack.len(),
            "prepared font stack source",
            prepared_stage.as_deref_mut(),
        )?;
        let source_work_admitted = prepared_stage.is_some();
        let stack = canonical::parse_family_stack_with_stage(&appearance.stack, prepared_stage)?;
        retained_alphabet.extend_from_slice(generated_alphabet);
        stack_source.push_str(&appearance.stack);
        Ok(Self {
            component_address,
            text_role,
            local_index,
            source: retained_source,
            generated_alphabet: retained_alphabet.into_boxed_slice(),
            stack_source,
            stack,
            weight: appearance.weight,
            pair_ordinal: u32::MAX,
            source_work_admitted,
        })
    }

    fn scalar_occurrence_count(&self) -> Result<u64, PreparedTextCensusError> {
        self.source
            .chars()
            .count()
            .checked_add(usize::from(self.source.contains('\u{00AD}')))
            .and_then(|count| count.checked_add(self.generated_alphabet.len()))
            .and_then(|count| u64::try_from(count).ok())
            .ok_or(PreparedTextCensusError::Arithmetic(
                "prepared text scalar occurrences",
            ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedFontPairRequirement {
    pub(crate) pair_ordinal: u32,
    pub(crate) stack_source: String,
    pub(crate) stack: Vec<FamilyPreference>,
    pub(crate) weight: TextWeight,
    pub(crate) required_scalars: Vec<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreparedTextCensusCounts {
    pub(crate) requirements: u128,
    pub(crate) pairs: u128,
    pub(crate) structural_addresses: u128,
    pub(crate) scalar_occurrences: u128,
    pub(crate) unique_pair_scalars: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedTextCensus {
    pub(crate) requirements: Vec<PreparedTextRequirement>,
    pub(crate) pairs: Vec<PreparedFontPairRequirement>,
    pub(crate) counts: PreparedTextCensusCounts,
}

impl PreparedTextCensus {
    pub(crate) fn structural_address_count(
        requirements: &[PreparedTextRequirement],
    ) -> Result<u128, PreparedTextCensusError> {
        requirements
            .iter()
            .enumerate()
            .filter(|(index, requirement)| {
                !requirements[..*index]
                    .iter()
                    .any(|earlier| earlier.component_address == requirement.component_address)
            })
            .try_fold(0_u128, |count, _| {
                count
                    .checked_add(1)
                    .ok_or(PreparedTextCensusError::Arithmetic(
                        "prepared text structural addresses",
                    ))
            })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the census reserves all pair shells before assigning canonical first-use ordinals"
    )]
    pub(crate) fn materialize(
        mut requirements: Vec<PreparedTextRequirement>,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
        scalar_presence: Option<&mut [u64]>,
    ) -> Result<Self, PreparedTextCensusError> {
        requirements.sort_unstable_by(|left, right| {
            left.component_address
                .canonical_text_cmp(&right.component_address)
                .then_with(|| left.text_role.cmp(&right.text_role))
                .then_with(|| left.local_index.cmp(&right.local_index))
        });
        if requirements.windows(2).any(|pair| {
            pair[0].component_address == pair[1].component_address
                && pair[0].text_role == pair[1].text_role
                && pair[0].local_index == pair[1].local_index
        }) {
            return Err(PreparedTextCensusError::DuplicateIdentity);
        }
        let pair_count = requirements
            .iter()
            .enumerate()
            .filter(|(index, requirement)| {
                !requirements[..*index].iter().any(|earlier| {
                    earlier.stack == requirement.stack && earlier.weight == requirement.weight
                })
            })
            .count();
        let mut first_use_indices = Vec::new();
        reserve_census_vec(
            &mut first_use_indices,
            pair_count,
            "prepared font pair first uses",
            prepared_stage.as_deref_mut(),
        )?;
        for (index, requirement) in requirements.iter().enumerate() {
            if !requirements[..index].iter().any(|earlier| {
                earlier.stack == requirement.stack && earlier.weight == requirement.weight
            }) {
                first_use_indices.push(index);
            }
        }
        let mut retained_stacks = Vec::new();
        reserve_census_vec(
            &mut retained_stacks,
            pair_count,
            "prepared font pair retained stacks",
            prepared_stage.as_deref_mut(),
        )?;
        for index in &first_use_indices {
            retained_stacks.push(reserve_family_stack_shell(
                &requirements[*index].stack,
                prepared_stage.as_deref_mut(),
            )?);
        }
        let mut retained_stack_sources = Vec::new();
        reserve_census_vec(
            &mut retained_stack_sources,
            pair_count,
            "prepared font pair stack sources",
            prepared_stage.as_deref_mut(),
        )?;
        for index in &first_use_indices {
            let mut source = String::new();
            reserve_census_string(
                &mut source,
                requirements[*index].stack_source.len(),
                "prepared font pair stack source",
                prepared_stage.as_deref_mut(),
            )?;
            retained_stack_sources.push(source);
        }
        let mut pairs: Vec<PreparedFontPairRequirement> = Vec::new();
        reserve_census_vec(
            &mut pairs,
            pair_count,
            "prepared font pairs",
            prepared_stage.as_deref_mut(),
        )?;
        for (pair_index, ((&requirement_index, stack), stack_source)) in first_use_indices
            .iter()
            .zip(retained_stacks)
            .zip(retained_stack_sources)
            .enumerate()
        {
            let requirement = &requirements[requirement_index];
            pairs.push(PreparedFontPairRequirement {
                pair_ordinal: u32::try_from(pair_index)
                    .map_err(|_| PreparedTextCensusError::Arithmetic("font pair ordinal"))?,
                stack_source,
                stack,
                weight: requirement.weight,
                required_scalars: Vec::new(),
            });
        }
        for requirement_index in 0..requirements.len() {
            let pair_index = first_use_indices
                .iter()
                .position(|index| {
                    requirements[*index].stack == requirements[requirement_index].stack
                        && requirements[*index].weight == requirements[requirement_index].weight
                })
                .ok_or(PreparedTextCensusError::MissingPair)?;
            requirements[requirement_index].pair_ordinal = u32::try_from(pair_index)
                .map_err(|_| PreparedTextCensusError::Arithmetic("font pair ordinal"))?;
        }
        let unique_pair_scalars = if let Some(presence) = scalar_presence {
            reserve_required_scalar_capacities(&requirements, &mut pairs, presence, prepared_stage)?
        } else {
            0
        };
        for (pair, &requirement_index) in pairs.iter_mut().zip(&first_use_indices) {
            fill_family_stack_shell(&mut pair.stack, &requirements[requirement_index].stack)?;
            pair.stack_source
                .push_str(&requirements[requirement_index].stack_source);
        }

        if pairs.len() != pair_count {
            return Err(PreparedTextCensusError::Arithmetic(
                "prepared font pair count/materialization mismatch",
            ));
        }
        let structural_addresses = Self::structural_address_count(&requirements)?;
        let scalar_occurrences = requirements.iter().try_fold(0_u128, |total, requirement| {
            total
                .checked_add(u128::from(requirement.scalar_occurrence_count()?))
                .ok_or(PreparedTextCensusError::Arithmetic(
                    "prepared text scalar occurrences",
                ))
        })?;

        Ok(Self {
            counts: PreparedTextCensusCounts {
                requirements: requirements.len() as u128,
                pairs: pairs.len() as u128,
                structural_addresses,
                scalar_occurrences,
                unique_pair_scalars,
            },
            requirements,
            pairs,
        })
    }

    pub(crate) fn pair_ordinal(
        &self,
        stack: &str,
        weight: TextWeight,
    ) -> Result<u32, PreparedTextCensusError> {
        let stack = canonical::parse_family_stack(stack)?;
        self.pairs
            .iter()
            .find(|pair| pair.stack == stack && pair.weight == weight)
            .map(|pair| pair.pair_ordinal)
            .ok_or(PreparedTextCensusError::MissingPair)
    }

    pub(crate) fn prepare_system_fonts(
        &mut self,
        mut presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<PreparedFontEnvironment, PreparedTextCensusError> {
        let discovery = PreparedFontEnvironment::reserve_system_discovery(prepared_stage)?;
        for requirement in &self.requirements {
            if !requirement.source_work_admitted {
                prepared_stage
                    .font_resources()
                    .claim_visits(requirement.scalar_occurrence_count()?)?;
            }
        }
        self.materialize_required_scalars(&mut presence, Some(prepared_stage))?;
        let environment = PreparedFontEnvironment::discover_system_with_prepared_resources(
            presence,
            prepared_stage.font_resources(),
            discovery,
            prepared_stage,
        )?;
        self.prepare_pairs_in(&environment, Some(prepared_stage))?;
        Ok(environment)
    }

    fn scalar_occurrence_count(&self) -> Result<u64, PreparedTextCensusError> {
        u64::try_from(self.counts.scalar_occurrences)
            .map_err(|_| PreparedTextCensusError::Arithmetic("prepared text scalar occurrences"))
    }

    fn materialize_required_scalars(
        &mut self,
        presence: &mut [u64],
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), PreparedTextCensusError> {
        if presence.len() != SCALAR_MAP_WORDS {
            return Err(PreparedTextCensusError::Arithmetic(
                "Unicode scalar presence map",
            ));
        }
        let mut capacities = Vec::new();
        reserve_census_vec(
            &mut capacities,
            self.pairs.len(),
            "required pair scalar capacities",
            prepared_stage.as_deref_mut(),
        )?;
        let mut unique_pair_scalars = 0_u128;
        for pair in &self.pairs {
            presence.fill(0);
            let mut unique_count = 0_usize;
            visit_pair_scalars(&self.requirements, pair.pair_ordinal, |scalar| {
                let value = scalar as usize;
                let word = &mut presence[value / u64::BITS as usize];
                let bit = 1_u64 << (value % u64::BITS as usize);
                if *word & bit == 0 {
                    *word |= bit;
                    unique_count =
                        unique_count
                            .checked_add(1)
                            .ok_or(PreparedTextCensusError::Arithmetic(
                                "prepared text unique scalars",
                            ))?;
                }
                Ok(())
            })?;
            unique_pair_scalars = unique_pair_scalars
                .checked_add(unique_count as u128)
                .ok_or(PreparedTextCensusError::Arithmetic(
                    "prepared text unique scalars",
                ))?;
            capacities.push(unique_count);
        }
        if capacities.len() != self.pairs.len() {
            return Err(PreparedTextCensusError::Arithmetic(
                "required pair scalar capacity count/materialization mismatch",
            ));
        }
        for (pair, unique_count) in self.pairs.iter_mut().zip(&capacities) {
            if pair.required_scalars.capacity() - pair.required_scalars.len() >= *unique_count {
                continue;
            }
            if let Some(stage) = prepared_stage.as_deref_mut() {
                stage
                    .reserve_prepared(
                        &mut pair.required_scalars,
                        *unique_count,
                        "required pair scalars",
                    )
                    .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?;
            } else {
                pair.required_scalars
                    .try_reserve_exact(*unique_count)
                    .map_err(|_| PreparedTextCensusError::Allocation("required pair scalars"))?;
            }
        }
        for (pair, unique_count) in self.pairs.iter_mut().zip(capacities) {
            presence.fill(0);
            visit_pair_scalars(&self.requirements, pair.pair_ordinal, |scalar| {
                let value = scalar as usize;
                let word = &mut presence[value / u64::BITS as usize];
                let bit = 1_u64 << (value % u64::BITS as usize);
                if *word & bit == 0 {
                    *word |= bit;
                    pair.required_scalars.push(scalar);
                }
                Ok(())
            })?;
            if pair.required_scalars.len() != unique_count {
                return Err(PreparedTextCensusError::Arithmetic(
                    "prepared scalar count/materialization mismatch",
                ));
            }
        }
        self.counts.unique_pair_scalars = unique_pair_scalars;
        Ok(())
    }

    fn prepare_fonts_in(
        &mut self,
        environment: &PreparedFontEnvironment,
        presence: &mut [u64],
    ) -> Result<(), PreparedTextCensusError> {
        environment.claim_preflight_visits(self.scalar_occurrence_count()?)?;
        self.materialize_required_scalars(presence, None)?;
        self.prepare_pairs_in(environment, None)
    }

    fn prepare_pairs_in(
        &self,
        environment: &PreparedFontEnvironment,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), PreparedTextCensusError> {
        environment.prepare_pairs(
            self.pairs.iter().map(|pair| {
                FontPairPreparationRequest::new(
                    pair.pair_ordinal,
                    &pair.stack,
                    requested_weight(pair.weight),
                    &pair.required_scalars,
                )
            }),
            prepared_stage.as_deref_mut(),
        )?;
        for requirement in &self.requirements {
            environment.prepare_source_units(
                requirement.pair_ordinal,
                &requirement.source,
                prepared_stage.as_deref_mut(),
            )?;
        }
        environment.seal_prepared_pairs()?;
        Ok(())
    }
}

fn reserve_census_vec<T>(
    values: &mut Vec<T>,
    additional: usize,
    payload: &'static str,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), PreparedTextCensusError> {
    if let Some(stage) = prepared_stage {
        super::claim_text_generation(stage.generation_resources().claim_slots(additional as u128))
            .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(PreparedTextCensusError::Arithmetic(payload))?;
        if required > values.capacity() {
            let bytes = required
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(PreparedTextCensusError::Arithmetic(payload))?;
            stage.font_resources().claim_bytes(
                u64::try_from(bytes).map_err(|_| PreparedTextCensusError::Arithmetic(payload))?,
            )?;
        }
        stage
            .reserve_prepared(values, additional, payload)
            .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))
    } else {
        values
            .try_reserve_exact(additional)
            .map_err(|_| PreparedTextCensusError::Allocation(payload))
    }
}

fn reserve_census_string(
    value: &mut String,
    additional: usize,
    payload: &'static str,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), PreparedTextCensusError> {
    if let Some(stage) = prepared_stage {
        let required = value
            .len()
            .checked_add(additional)
            .ok_or(PreparedTextCensusError::Arithmetic(payload))?;
        if required > value.capacity() {
            stage.font_resources().claim_bytes(
                u64::try_from(required)
                    .map_err(|_| PreparedTextCensusError::Arithmetic(payload))?,
            )?;
        }
        stage
            .reserve_prepared_string(value, additional, payload)
            .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))
    } else {
        value
            .try_reserve_exact(additional)
            .map_err(|_| PreparedTextCensusError::Allocation(payload))
    }
}

fn reserve_family_stack_shell(
    source: &[FamilyPreference],
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<FamilyPreference>, PreparedTextCensusError> {
    let mut retained = Vec::new();
    reserve_census_vec(
        &mut retained,
        source.len(),
        "prepared font pair stack",
        prepared_stage.as_deref_mut(),
    )?;
    for preference in source {
        retained.push(match preference {
            FamilyPreference::Named(name) => {
                let mut reserved = String::new();
                reserve_census_string(
                    &mut reserved,
                    name.len(),
                    "prepared font pair family name",
                    prepared_stage.as_deref_mut(),
                )?;
                FamilyPreference::Named(reserved)
            }
            FamilyPreference::Serif => FamilyPreference::Serif,
            FamilyPreference::SansSerif => FamilyPreference::SansSerif,
            FamilyPreference::Monospace => FamilyPreference::Monospace,
        });
    }
    Ok(retained)
}

fn reserve_required_scalar_capacities(
    requirements: &[PreparedTextRequirement],
    pairs: &mut [PreparedFontPairRequirement],
    presence: &mut [u64],
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<u128, PreparedTextCensusError> {
    if presence.len() != SCALAR_MAP_WORDS {
        return Err(PreparedTextCensusError::Arithmetic(
            "Unicode scalar presence map",
        ));
    }
    let mut capacities = Vec::new();
    reserve_census_vec(
        &mut capacities,
        pairs.len(),
        "required pair scalar capacities",
        prepared_stage.as_deref_mut(),
    )?;
    let mut unique_pair_scalars = 0_u128;
    for pair in pairs.iter() {
        presence.fill(0);
        let mut unique_count = 0_usize;
        visit_pair_scalars(requirements, pair.pair_ordinal, |scalar| {
            let value = scalar as usize;
            let word = &mut presence[value / u64::BITS as usize];
            let bit = 1_u64 << (value % u64::BITS as usize);
            if *word & bit == 0 {
                *word |= bit;
                unique_count =
                    unique_count
                        .checked_add(1)
                        .ok_or(PreparedTextCensusError::Arithmetic(
                            "prepared text unique scalars",
                        ))?;
            }
            Ok(())
        })?;
        unique_pair_scalars = unique_pair_scalars
            .checked_add(unique_count as u128)
            .ok_or(PreparedTextCensusError::Arithmetic(
                "prepared text unique scalars",
            ))?;
        capacities.push(unique_count);
    }
    for (pair, capacity) in pairs.iter_mut().zip(capacities) {
        reserve_census_vec(
            &mut pair.required_scalars,
            capacity,
            "required pair scalars",
            prepared_stage.as_deref_mut(),
        )?;
    }
    Ok(unique_pair_scalars)
}

fn fill_family_stack_shell(
    retained: &mut [FamilyPreference],
    source: &[FamilyPreference],
) -> Result<(), PreparedTextCensusError> {
    if retained.len() != source.len() {
        return Err(PreparedTextCensusError::Arithmetic(
            "prepared font pair stack",
        ));
    }
    for (retained, source) in retained.iter_mut().zip(source) {
        match (retained, source) {
            (FamilyPreference::Named(retained), FamilyPreference::Named(source)) => {
                if !retained.is_empty() || retained.capacity() < source.len() {
                    return Err(PreparedTextCensusError::Arithmetic(
                        "prepared font pair family name",
                    ));
                }
                retained.push_str(source);
            }
            (FamilyPreference::Serif, FamilyPreference::Serif)
            | (FamilyPreference::SansSerif, FamilyPreference::SansSerif)
            | (FamilyPreference::Monospace, FamilyPreference::Monospace) => {}
            _ => {
                return Err(PreparedTextCensusError::Arithmetic(
                    "prepared font pair stack",
                ));
            }
        }
    }
    Ok(())
}

fn prepared_address_child(
    base: &ComponentAddress,
    role: crate::layout::kernel::AddressRole,
    slot_index: u32,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<ComponentAddress, Error> {
    if let Some(stage) = prepared_stage {
        base.prepared_child(role, slot_index, stage)
    } else {
        base.child(role, slot_index)
    }
}

fn prepared_address_clone(
    address: &ComponentAddress,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<ComponentAddress, Error> {
    if let Some(stage) = prepared_stage {
        address.prepared_clone(stage)
    } else {
        Ok(address.clone())
    }
}

#[expect(
    clippy::similar_names,
    clippy::too_many_arguments,
    reason = "the closed text census keeps both axis policies and canonical producer order explicit"
)]
pub(crate) fn foundational_plot_requirements(
    heatmap: Option<&crate::plots::heatmap::PreparedHeatmapInputs>,
    plot: &Plot,
    kind: PlotKind,
    theme: &crate::theme::ResolvedTheme,
    base: &ComponentAddress,
    include_caption: bool,
    filter_point_label_x_to_limits: bool,
    filter_point_label_y_to_limits: bool,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<PreparedTextRequirement>, Error> {
    let mut requirements = Vec::new();
    PreparedTextRequirement::append_plot(
        heatmap,
        &mut requirements,
        plot,
        kind,
        theme,
        base,
        include_caption,
        filter_point_label_x_to_limits,
        filter_point_label_y_to_limits,
        prepared_stage,
    )?;
    Ok(requirements)
}

impl PreparedTextRequirement {
    #[expect(
        clippy::similar_names,
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "the closed producer traversal preserves axis policies and visible source identities"
    )]
    pub(crate) fn append_plot(
        heatmap: Option<&crate::plots::heatmap::PreparedHeatmapInputs>,
        requirements: &mut Vec<Self>,
        plot: &Plot,
        kind: PlotKind,
        theme: &crate::theme::ResolvedTheme,
        base: &ComponentAddress,
        include_caption: bool,
        filter_point_label_x_to_limits: bool,
        filter_point_label_y_to_limits: bool,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), Error> {
        use crate::layout::kernel::AddressRole;

        #[cfg(not(test))]
        if prepared_stage.is_none() {
            return Err(Error::Render(
                PreparedTextCensusError::MissingPreparedStage.to_string(),
            ));
        }
        let title_address = prepared_address_child(
            base,
            AddressRole::RootTitle,
            0,
            prepared_stage.as_deref_mut(),
        )?;
        push_present(
            requirements,
            title_address,
            PreparedTextRole::Title,
            plot.title_text(),
            &theme.typography.title,
            prepared_stage.as_deref_mut(),
        )?;
        let subtitle_address = prepared_address_child(
            base,
            AddressRole::RootSubtitle,
            0,
            prepared_stage.as_deref_mut(),
        )?;
        push_present(
            requirements,
            subtitle_address,
            PreparedTextRole::Subtitle,
            plot.subtitle_text(),
            &theme.typography.subtitle,
            prepared_stage.as_deref_mut(),
        )?;
        if kind != PlotKind::Matrix {
            let (x_title, y_title) = plot.resolved_axis_labels(kind);
            let x_axis = prepared_address_child(
                base,
                AddressRole::AxisBottom,
                0,
                prepared_stage.as_deref_mut(),
            )?;
            let y_axis = prepared_address_child(
                base,
                AddressRole::AxisLeft,
                0,
                prepared_stage.as_deref_mut(),
            )?;
            if !x_title.is_empty() {
                let address = prepared_address_clone(&x_axis, prepared_stage.as_deref_mut())?;
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::AxisTitle,
                    0,
                    x_title,
                    &[],
                    &theme.typography.axis_title,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            if !y_title.is_empty() {
                let address = prepared_address_clone(&y_axis, prepared_stage.as_deref_mut())?;
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::AxisTitle,
                    0,
                    y_title,
                    &[],
                    &theme.typography.axis_title,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            push_axis_ticks(
                requirements,
                &x_axis,
                plot.explicit_x_ticks(),
                automatic_category_labels(plot, kind, true)?,
                (kind == PlotKind::Heatmap
                    && plot.heatmap_label_fit_spec() == crate::HeatmapLabelFit::Elide)
                    .then_some(HEATMAP_ELISION_ALPHABET),
                &theme.typography.tick_label,
                prepared_stage.as_deref_mut(),
            )?;
            push_axis_ticks(
                requirements,
                &y_axis,
                plot.explicit_y_ticks(),
                automatic_category_labels(plot, kind, false)?,
                (kind == PlotKind::Heatmap
                    && plot.heatmap_label_fit_spec() == crate::HeatmapLabelFit::Elide)
                    .then_some(HEATMAP_ELISION_ALPHABET),
                &theme.typography.tick_label,
                prepared_stage.as_deref_mut(),
            )?;
        }

        let significance_appearance = theme
            .typography
            .effective_significance_label(plot.significance_mark_style_override());
        for (index, annotation) in plot.significance_annotations().iter().enumerate() {
            let address = prepared_address_child(
                base,
                AddressRole::Annotation,
                u32::try_from(index)
                    .map_err(|_| Error::Render("significance census index overflowed".into()))?,
                prepared_stage.as_deref_mut(),
            )?;
            push_owned(
                requirements,
                address,
                PreparedTextRole::SignificanceLabel,
                0,
                annotation.mark().glyph()?,
                &[],
                &significance_appearance,
                prepared_stage.as_deref_mut(),
            )?;
        }
        push_deferred_plot_requirements(
            heatmap,
            requirements,
            plot,
            kind,
            theme,
            base,
            filter_point_label_x_to_limits,
            filter_point_label_y_to_limits,
            prepared_stage.as_deref_mut(),
        )?;
        if include_caption {
            let caption_address = prepared_address_child(
                base,
                AddressRole::RootCaption,
                0,
                prepared_stage.as_deref_mut(),
            )?;
            push_present(
                requirements,
                caption_address,
                PreparedTextRole::Caption,
                plot.caption_text(),
                &theme.typography.caption,
                prepared_stage,
            )?;
        }
        Ok(())
    }
}

#[expect(
    clippy::similar_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one closed plot-kind match keeps both range filters and every deferred T5b/T5e census source auditable"
)]
fn push_deferred_plot_requirements(
    heatmap: Option<&crate::plots::heatmap::PreparedHeatmapInputs>,
    requirements: &mut Vec<PreparedTextRequirement>,
    plot: &Plot,
    kind: PlotKind,
    theme: &crate::theme::ResolvedTheme,
    base: &ComponentAddress,
    filter_point_label_x_to_limits: bool,
    filter_point_label_y_to_limits: bool,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    use crate::data::DataColumn;
    use crate::layout::kernel::AddressRole;

    let body = theme.legacy_text_appearance(theme.detail_font_size, TextWeight::Normal);
    let body_address = prepared_address_child(
        base,
        AddressRole::SpecializedInterior,
        0,
        prepared_stage.as_deref_mut(),
    )?;
    let mut body_index = 0_u32;

    for annotation in plot.panel_text_annotations() {
        push_deferred_body(
            requirements,
            &body_address,
            &mut body_index,
            annotation.text(),
            &[],
            &body,
            prepared_stage.as_deref_mut(),
        )?;
    }
    if plot.has_facets() {
        crate::orchestrator::facet_text_census_labels(plot, |label| {
            push_deferred_body(
                requirements,
                &body_address,
                &mut body_index,
                label,
                &[],
                &body,
                prepared_stage.as_deref_mut(),
            )
        })?;
    }
    for layer in plot.point_layers() {
        if let Some(labels) = layer.label_values() {
            for row in 0..labels.len() {
                let Some(Some(label)) = labels.get(row) else {
                    continue;
                };
                if point_layer_row_is_visible(
                    plot,
                    layer,
                    row,
                    filter_point_label_x_to_limits,
                    filter_point_label_y_to_limits,
                ) {
                    push_deferred_body(
                        requirements,
                        &body_address,
                        &mut body_index,
                        label,
                        &[],
                        &body,
                        prepared_stage.as_deref_mut(),
                    )?;
                }
            }
        }
        if let Some(label) = layer.legend_label_text() {
            if plot.legend_policy_spec() != crate::LegendPolicy::Hidden {
                let (_, label) = plot
                    .legend_identity(label)
                    .resolved_text(plot, plot.data())?;
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    label,
                    &[],
                    &body,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    }

    if plot_data_legend_is_visible(plot, kind) {
        for source in legend_title_census(plot, kind) {
            push_deferred_body(
                requirements,
                &body_address,
                &mut body_index,
                source,
                &[],
                &body,
                prepared_stage.as_deref_mut(),
            )?;
        }
        for column in plot.aligned_columns() {
            if let DataColumn::Category(values) = column {
                if !legend_dimension_is_visible(plot, kind, values.name()) {
                    continue;
                }
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    plot.resolve_label(values.name()),
                    &[],
                    &body,
                    prepared_stage.as_deref_mut(),
                )?;
                let levels = if matches!(kind, PlotKind::Scatter | PlotKind::Marginal)
                    && Some(values.name()) == plot.hue_name()
                {
                    let family = if kind == PlotKind::Scatter {
                        "scatter"
                    } else {
                        "marginal"
                    };
                    let x = plot
                        .x_col()
                        .ok_or_else(|| Error::Data(format!("{family}: x column not set")))?;
                    let y = plot
                        .y_col()
                        .ok_or_else(|| Error::Data(format!("{family}: y column not set")))?;
                    let hue = plot
                        .hue_col()
                        .ok_or_else(|| Error::Data(format!("{family}: hue column not set")))?;
                    if kind == PlotKind::Scatter {
                        Cow::Owned(crate::data::observed_levels_for_finite_pairs(
                            plot.data(),
                            x,
                            y,
                            hue,
                            "scatter",
                        )?)
                    } else {
                        Cow::Owned(crate::data::observed_levels_for_retained_finite_pairs(
                            plot.data(),
                            x,
                            y,
                            hue,
                            "marginal",
                        )?)
                    }
                } else {
                    Cow::Borrowed(values.levels())
                };
                for value in levels.iter() {
                    push_deferred_body(
                        requirements,
                        &body_address,
                        &mut body_index,
                        plot.resolve_label(value),
                        &[],
                        &body,
                        prepared_stage.as_deref_mut(),
                    )?;
                }
            }
        }
        if legend_uses_pair_identity(plot, kind) {
            push_deferred_body(
                requirements,
                &body_address,
                &mut body_index,
                " × , ",
                &[],
                &body,
                prepared_stage.as_deref_mut(),
            )?;
        }
        if kind == PlotKind::Regression
            && matches!(plot.legend_policy_spec(), crate::LegendPolicy::Visible(_))
        {
            for source in ["Observations", plot.regression_band_spec().legend_label()] {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    source,
                    &[],
                    &body,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
        if let Some(endpoints) = plot.dumbbell_endpoints_spec() {
            for source in [endpoints.left_label(), endpoints.right_label()] {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    source,
                    &[],
                    &body,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
        if let Some(input) = plot.raincloud_geometry_input() {
            let mut hues = Vec::new();
            for hue in input.groups.iter().filter_map(|group| group.hue.as_deref()) {
                if !hues.contains(&hue) {
                    hues.push(hue);
                    push_deferred_body(
                        requirements,
                        &body_address,
                        &mut body_index,
                        plot.resolve_label(hue),
                        &[],
                        &body,
                        prepared_stage.as_deref_mut(),
                    )?;
                }
            }
        }
    }

    if kind == PlotKind::Matrix {
        for variable in plot.matrix_variable_names() {
            for _ in 0..2 {
                let address = prepared_address_clone(&body_address, prepared_stage.as_deref_mut())?;
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::AxisTitle,
                    body_index,
                    plot.resolve_label(variable),
                    &[],
                    &theme.typography.axis_title,
                    prepared_stage.as_deref_mut(),
                )?;
                body_index = body_index
                    .checked_add(1)
                    .ok_or_else(|| Error::Render("deferred text census index overflowed".into()))?;
            }
        }
        push_owned(
            requirements,
            body_address,
            PreparedTextRole::TickLabel,
            body_index,
            String::new(),
            AUTOMATIC_NUMERIC_ALPHABET,
            &theme.typography.tick_label,
            prepared_stage,
        )?;
    } else if kind == PlotKind::Heatmap {
        let heatmap = heatmap.ok_or_else(|| {
            Error::Render("heatmap census requires root-prepared semantics".into())
        })?;
        let mut visit = |source: CensusSource<'_>| {
            push_deferred_body(
                requirements,
                &body_address,
                &mut body_index,
                source,
                &[],
                &body,
                prepared_stage.as_deref_mut(),
            )
        };
        for (_, marks) in &heatmap.panels {
            visit(marks.color_label.as_str().into())?;
            crate::plots::heatmap::text_census(marks, &mut visit)?;
            for annotation in plot.heatmap_null_row_annotations_spec() {
                visit(annotation.text().into())?;
            }
        }
    } else if plot.has_facets()
        && matches!(
            kind,
            PlotKind::Beeswarm
                | PlotKind::Coefficient
                | PlotKind::Forest
                | PlotKind::PointInterval
                | PlotKind::Waterfall
        )
    {
        let resources = prepared_stage
            .as_ref()
            .map(|stage| stage.generation_resources());
        let data = resources.as_ref().map_or_else(
            || plot.data(),
            |resources| plot.data().with_axis_resources(resources),
        );
        for panel in crate::orchestrator::facet_text_census_panels(plot, data)? {
            specialized_text_census(
                panel.plot(),
                kind,
                panel.data(),
                filter_point_label_x_to_limits,
                |source| {
                    push_deferred_body(
                        requirements,
                        &body_address,
                        &mut body_index,
                        source,
                        &[],
                        &body,
                        prepared_stage.as_deref_mut(),
                    )
                },
            )?;
        }
    } else {
        specialized_text_census(
            plot,
            kind,
            plot.data(),
            filter_point_label_x_to_limits,
            |source| {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    source,
                    &[],
                    &body,
                    prepared_stage.as_deref_mut(),
                )
            },
        )?;
    }

    Ok(())
}

fn push_deferred_body<'a>(
    requirements: &mut Vec<PreparedTextRequirement>,
    body_address: &ComponentAddress,
    body_index: &mut u32,
    source: impl Into<CensusSource<'a>>,
    generated: &[char],
    appearance: &TextAppearance,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    let address = prepared_address_clone(body_address, prepared_stage.as_deref_mut())?;
    push_owned(
        requirements,
        address,
        PreparedTextRole::Body,
        *body_index,
        source,
        generated,
        appearance,
        prepared_stage,
    )?;
    *body_index = body_index
        .checked_add(1)
        .ok_or_else(|| Error::Render("deferred text census index overflowed".into()))?;
    Ok(())
}

#[expect(
    clippy::similar_names,
    reason = "independent x and y limit filters are part of the Canvas shared-range contract"
)]
fn point_layer_row_is_visible(
    plot: &Plot,
    layer: &crate::PointLayer,
    row: usize,
    filter_x_to_limits: bool,
    filter_y_to_limits: bool,
) -> bool {
    let x = layer.x_values().get(row).flatten();
    let y = layer.y_values().get(row).flatten();
    x.is_some_and(|x| {
        x.is_finite()
            && (!filter_x_to_limits
                || plot
                    .x_axis_spec()
                    .limits
                    .is_none_or(|limits| x >= limits.0 && x <= limits.1))
    }) && y.is_some_and(|y| {
        y.is_finite()
            && (!filter_y_to_limits
                || plot
                    .y_axis_spec()
                    .limits
                    .is_none_or(|limits| y >= limits.0 && y <= limits.1))
    })
}

fn plot_data_legend_is_visible(plot: &Plot, kind: PlotKind) -> bool {
    if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
        return false;
    }
    match kind {
        PlotKind::Scatter
        | PlotKind::Density
        | PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Raincloud
        | PlotKind::Marginal
        | PlotKind::Matrix => plot.hue_dimension_key().is_some(),
        PlotKind::Line => {
            plot.hue_name().is_some()
                || plot
                    .point_layers()
                    .iter()
                    .any(|layer| layer.legend_label_text().is_some())
        }
        PlotKind::Trajectory => plot.hue_name().is_some(),
        PlotKind::LinePath => {
            plot.hue_name().is_some()
                && (plot.markers_visible()
                    || plot.line_path_style_spec() == crate::LinePathStyle::Hue)
        }
        PlotKind::Interaction => {
            plot.hue_name().is_some() || plot.interaction_styles_spec().is_some()
        }
        PlotKind::Dumbbell => true,
        PlotKind::Regression => {
            matches!(plot.legend_policy_spec(), crate::LegendPolicy::Visible(_))
        }
        PlotKind::Histogram
        | PlotKind::Qq
        | PlotKind::PointInterval
        | PlotKind::Coefficient
        | PlotKind::Forest
        | PlotKind::Heatmap
        | PlotKind::Beeswarm
        | PlotKind::Waterfall => false,
    }
}

fn legend_dimension_is_visible(plot: &Plot, kind: PlotKind, name: &str) -> bool {
    if kind == PlotKind::LinePath && plot.line_path_style_spec() == crate::LinePathStyle::Hue {
        Some(name) == plot.group_name() || Some(name) == plot.hue_name()
    } else {
        Some(name) == plot.hue_dimension_key()
            || (kind == PlotKind::Interaction
                && plot.interaction_styles_spec().is_some()
                && Some(name) == plot.group_name())
            || (legend_uses_pair_identity(plot, kind) && Some(name) == plot.group_name())
    }
}

fn legend_uses_pair_identity(plot: &Plot, kind: PlotKind) -> bool {
    (matches!(
        kind,
        PlotKind::Density
            | PlotKind::Line
            | PlotKind::Interaction
            | PlotKind::Marginal
            | PlotKind::Matrix
    ) || (kind == PlotKind::LinePath && plot.line_path_style_spec() == crate::LinePathStyle::Hue))
        && matches!(
            (plot.group_name(), plot.hue_name()),
            (Some(group), Some(hue)) if group != hue
        )
}

fn legend_title_census(plot: &Plot, kind: PlotKind) -> impl Iterator<Item = Cow<'_, str>> {
    if legend_uses_pair_identity(plot, kind) {
        [
            Some(Cow::Owned(plot.resolve_label(
                plot.group_name().expect("pair identity has a group"),
            ))),
            Some(Cow::Owned(plot.resolve_label(
                plot.hue_name().expect("pair identity has a hue"),
            ))),
            Some(Cow::Borrowed(" × ")),
        ]
    } else {
        let literal = matches!(kind, PlotKind::Dumbbell | PlotKind::Regression)
            || (kind == PlotKind::Line && plot.hue_dimension_key().is_none());
        [
            Some(if literal {
                Cow::Borrowed("series")
            } else {
                Cow::Owned(plot.resolve_label(plot.legend_title_key()))
            }),
            None,
            None,
        ]
    }
    .into_iter()
    .flatten()
}

fn specialized_text_census(
    plot: &Plot,
    kind: PlotKind,
    data: crate::data::PlotData<'_>,
    filter_x_to_limits: bool,
    mut visit: impl FnMut(CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    use crate::data::DataColumn;

    match kind {
        PlotKind::Heatmap => {
            return Err(Error::Render(
                "heatmap census requires retained traversal".into(),
            ));
        }
        PlotKind::Beeswarm => {
            visit(plot.resolved_color_label().into())?;
            crate::plots::beeswarm::text_census_colorbar_values(plot, data, &mut visit)?;
        }
        PlotKind::Coefficient | PlotKind::Forest | PlotKind::PointInterval => {
            let has_unavailable = data
                .estimate_statuses()?
                .is_some_and(|statuses| statuses.contains(&crate::EstimateStatus::NotEstimable))
                || data
                    .column(crate::data::X)
                    .ok()
                    .and_then(|column| match column {
                        DataColumn::Numeric(values) => Some(values),
                        DataColumn::Category(_) | DataColumn::Boolean(_) => None,
                    })
                    .is_some_and(|values| {
                        (0..data.height())
                            .any(|row| matches!(values.get(data.row_index(row)), Some(None)))
                    });
            if has_unavailable {
                visit("Not estimable".into())?;
            }
            if let Some(format) = plot.estimate_labels_spec() {
                if let Ok(DataColumn::Numeric(values)) = data.column(crate::data::X) {
                    for row in 0..data.height() {
                        if let Some(Some(value)) = values.get(data.row_index(row)) {
                            visit(format_args!("{}", format.display(value)).into())?;
                        }
                    }
                }
            }
        }
        PlotKind::Waterfall => {
            crate::plots::waterfall::text_census_annotations(
                plot,
                data,
                filter_x_to_limits,
                &mut visit,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn push_present(
    requirements: &mut Vec<PreparedTextRequirement>,
    address: ComponentAddress,
    role: PreparedTextRole,
    source: Option<&str>,
    appearance: &TextAppearance,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    if let Some(source) = source.filter(|source| !source.is_empty()) {
        push_owned(
            requirements,
            address,
            role,
            0,
            source,
            &[],
            appearance,
            prepared_stage,
        )?;
    }
    Ok(())
}

fn push_axis_ticks(
    requirements: &mut Vec<PreparedTextRequirement>,
    axis: &ComponentAddress,
    exact: Option<&[crate::layout::AxisTick]>,
    automatic_categories: Option<Vec<String>>,
    category_generated_alphabet: Option<&[char]>,
    appearance: &TextAppearance,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    if let Some(ticks) = exact {
        for (index, tick) in ticks.iter().enumerate() {
            let address = prepared_address_clone(axis, prepared_stage.as_deref_mut())?;
            push_owned(
                requirements,
                address,
                PreparedTextRole::TickLabel,
                u32::try_from(index)
                    .map_err(|_| Error::Render("exact tick census index overflowed".into()))?,
                &tick.label,
                &[],
                appearance,
                prepared_stage.as_deref_mut(),
            )?;
        }
    } else if let Some(categories) = automatic_categories {
        for (index, category) in categories.into_iter().enumerate() {
            let address = prepared_address_clone(axis, prepared_stage.as_deref_mut())?;
            push_owned(
                requirements,
                address,
                PreparedTextRole::TickLabel,
                u32::try_from(index).map_err(|_| {
                    Error::Render("categorical tick census index overflowed".into())
                })?,
                category,
                category_generated_alphabet.unwrap_or_default(),
                appearance,
                prepared_stage.as_deref_mut(),
            )?;
        }
    } else {
        let address = prepared_address_clone(axis, prepared_stage.as_deref_mut())?;
        push_owned(
            requirements,
            address,
            PreparedTextRole::TickLabel,
            0,
            String::new(),
            AUTOMATIC_NUMERIC_ALPHABET,
            appearance,
            prepared_stage,
        )?;
    }
    Ok(())
}

fn automatic_category_labels(
    plot: &Plot,
    kind: PlotKind,
    x_axis: bool,
) -> Result<Option<Vec<String>>, Error> {
    if kind == PlotKind::Raincloud && x_axis {
        if let Some(input) = plot.raincloud_geometry_input() {
            return plot
                .resolve_label_domain(
                    "x-axis categories",
                    input.groups.iter().map(|group| group.category.as_str()),
                )
                .map(Some);
        }
    }
    plot.automatic_axis_category_values(kind, x_axis)
        .map(|values| {
            let normalized = matches!(
                (kind, x_axis),
                (PlotKind::Trajectory | PlotKind::Heatmap, true)
                    | (
                        PlotKind::Dumbbell | PlotKind::PointInterval | PlotKind::Heatmap,
                        false
                    )
            );
            let levels = if normalized {
                Cow::Owned(
                    values
                        .levels()
                        .iter()
                        .map(|value| crate::data::collapse_whitespace(value))
                        .collect::<Vec<_>>(),
                )
            } else {
                Cow::Borrowed(values.levels())
            };
            plot.resolve_label_domain(
                if x_axis {
                    "x-axis categories"
                } else {
                    "y-axis categories"
                },
                levels.iter().map(String::as_str),
            )
        })
        .transpose()
}

#[expect(
    clippy::too_many_arguments,
    reason = "one census insertion keeps the complete structural text identity explicit"
)]
fn push_owned<'a>(
    requirements: &mut Vec<PreparedTextRequirement>,
    address: ComponentAddress,
    role: PreparedTextRole,
    local_index: u32,
    source: impl Into<CensusSource<'a>>,
    generated_alphabet: &[char],
    appearance: &TextAppearance,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    PreparedTextRequirement::push_into(
        requirements,
        address,
        role,
        local_index,
        source,
        generated_alphabet,
        appearance,
        prepared_stage,
    )
    .map_err(|error| Error::Render(format!("prepared text census: {error}")))
}

const fn requested_weight(weight: TextWeight) -> RequestedWeight {
    match weight {
        TextWeight::Normal => RequestedWeight::Normal,
        TextWeight::Semibold => RequestedWeight::Semibold,
        TextWeight::Bold => RequestedWeight::Bold,
    }
}

fn visit_pair_scalars(
    requirements: &[PreparedTextRequirement],
    pair_ordinal: u32,
    mut visit: impl FnMut(char) -> Result<(), PreparedTextCensusError>,
) -> Result<(), PreparedTextCensusError> {
    for requirement in requirements
        .iter()
        .filter(|requirement| requirement.pair_ordinal == pair_ordinal)
    {
        let mut needs_hyphen = false;
        for scalar in requirement.source.chars() {
            needs_hyphen |= scalar == '\u{00AD}';
            visit(scalar)?;
        }
        // A selected discretionary break paints this scalar in the owning pair.
        // Keep its occurrence in admission even if an explicit alphabet repeats it.
        if needs_hyphen {
            visit('-')?;
        }
        for &scalar in &requirement.generated_alphabet {
            visit(scalar)?;
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PreparedTextCensusError {
    #[error("production text census requires the root prepared stage")]
    MissingPreparedStage,
    #[error(transparent)]
    Stack(#[from] canonical::FamilyStackError),
    #[error(transparent)]
    Font(#[from] canonical::FontError),
    #[error("prepared generated text alphabet must be strictly ascending")]
    GeneratedAlphabetOrder,
    #[error("prepared text census contains a duplicate address/role/index identity")]
    DuplicateIdentity,
    #[error("prepared text census is missing the requested font stack/weight pair")]
    MissingPair,
    #[error("prepared text census arithmetic overflow while calculating {0}")]
    Arithmetic(&'static str),
    #[error("prepared text census could not reserve {0}")]
    Allocation(&'static str),
    #[error("{0}")]
    Reservation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::kernel::AddressRole;
    use crate::render::backend::Rgba;

    #[test]
    fn soft_hyphen_census_derives_one_literal_per_requirement_in_the_owning_pair() {
        let bold = appearance("Soft", TextWeight::Bold);
        let normal = appearance("Soft", TextWeight::Normal);
        let mut census = PreparedTextCensus::materialize(
            vec![
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    0,
                    "A\u{00AD}\u{00AD}-",
                    &['-'],
                    &bold,
                ),
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::Caption,
                    0,
                    "A\u{00AD}",
                    &[],
                    &bold,
                ),
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::TickLabel,
                    0,
                    "A",
                    &[],
                    &normal,
                ),
            ],
            None,
            None,
        )
        .unwrap();
        assert_eq!(census.counts.scalar_occurrences, 10);
        let environment = PreparedFontEnvironment::synthetic_faces(&[("Soft", &['A', '-'])]);
        census
            .prepare_fonts_in(&environment, &mut vec![0; SCALAR_MAP_WORDS])
            .unwrap();
        assert!(environment.prepared_pairs_are_sealed().unwrap());
        assert_eq!(census.counts.unique_pair_scalars, 4);
        let bold_pair = census
            .pairs
            .iter()
            .find(|pair| pair.weight == TextWeight::Bold)
            .unwrap();
        assert_eq!(bold_pair.required_scalars, ['A', '\u{00AD}', '-']);
        let normal_pair = census
            .pairs
            .iter()
            .find(|pair| pair.weight == TextWeight::Normal)
            .unwrap();
        assert_eq!(normal_pair.required_scalars, ['A']);
        let mut occurrences = Vec::new();
        visit_pair_scalars(&census.requirements, bold_pair.pair_ordinal, |scalar| {
            occurrences.push(scalar);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            occurrences.iter().filter(|scalar| **scalar == '-').count(),
            4
        );
    }

    #[test]
    fn soft_hyphen_missing_visible_literal_is_fatal_during_census_selection() {
        let mut census = PreparedTextCensus::materialize(
            vec![requirement(
                ComponentAddress::root(),
                PreparedTextRole::Title,
                0,
                "A\u{00AD}AA",
                &[],
                &appearance("Test Sans", TextWeight::Bold),
            )],
            None,
            None,
        )
        .unwrap();
        let environment = PreparedFontEnvironment::synthetic();
        let error = census
            .prepare_fonts_in(&environment, &mut vec![0; SCALAR_MAP_WORDS])
            .unwrap_err();
        assert!(matches!(
            error,
            PreparedTextCensusError::Font(canonical::FontError::MissingGlyph {
                pair_ordinal: 0,
                scalar: 0x2D
            })
        ));
        assert!(!environment.prepared_pairs_are_sealed().unwrap());
    }

    #[test]
    fn soft_hyphen_borrowed_and_chunked_format_admission_include_derived_work_once() {
        for formatted in [false, true] {
            for alphabet in [&[][..], &['-'][..]] {
                for refuse in [false, true] {
                    crate::layout::plan_exact_root(
                        320,
                        240,
                        |stage| {
                            // Nine stack bytes, three source scalars/units, one derived
                            // literal, and every explicit generated alphabet occurrence.
                            let work = 16 + u64::try_from(alphabet.len()).unwrap();
                            stage
                                .font_resources()
                                .claim_visits(1_000_000 - work + u64::from(refuse))
                                .unwrap();
                            let appearance = appearance("Test Sans", TextWeight::Bold);
                            let result = if formatted {
                                PreparedTextRequirement::new_with_stage(
                                    ComponentAddress::root(),
                                    PreparedTextRole::Title,
                                    0,
                                    format_args!("{}{}{}", "A", '\u{00AD}', '\u{00AD}'),
                                    alphabet,
                                    &appearance,
                                    Some(stage),
                                )
                            } else {
                                PreparedTextRequirement::new_with_stage(
                                    ComponentAddress::root(),
                                    PreparedTextRole::Title,
                                    0,
                                    "A\u{00AD}\u{00AD}",
                                    alphabet,
                                    &appearance,
                                    Some(stage),
                                )
                            };
                            if refuse {
                                assert!(matches!(
                                    result,
                                    Err(PreparedTextCensusError::Font(
                                        canonical::FontError::Limit { .. }
                                    ))
                                ));
                                assert_eq!(stage.font_resources().usage().unwrap().font_bytes, 0);
                            } else {
                                let requirement = result.unwrap();
                                assert_eq!(requirement.source, "A\u{00AD}\u{00AD}");
                                assert_eq!(
                                    requirement.scalar_occurrence_count().unwrap(),
                                    4 + u64::try_from(alphabet.len()).unwrap()
                                );
                                assert_eq!(
                                    stage.font_resources().usage().unwrap().processing_visits,
                                    1_000_000
                                );
                            }
                            Ok(())
                        },
                        |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                        |(), _, (), _| Ok(()),
                    )
                    .unwrap();
                }
            }
        }
    }

    #[test]
    fn formatted_source_census_charges_the_same_source_work_and_bytes_once() {
        for formatted in [false, true] {
            crate::layout::plan_exact_root(
                320,
                240,
                |stage| {
                    let appearance = appearance("Test Sans", TextWeight::Normal);
                    let format = crate::NumericLabelFormat::new(0)
                        .unwrap()
                        .suffix("\u{0301}")
                        .unwrap();
                    let mut requirements = Vec::new();
                    let source = if formatted {
                        PreparedTextRequirement::new_with_stage(
                            ComponentAddress::root(),
                            PreparedTextRole::Body,
                            0,
                            format_args!("{}", format.display(1.0)),
                            &[],
                            &appearance,
                            Some(stage),
                        )
                    } else {
                        PreparedTextRequirement::new_with_stage(
                            ComponentAddress::root(),
                            PreparedTextRole::Body,
                            0,
                            "1\u{0301}",
                            &[],
                            &appearance,
                            Some(stage),
                        )
                    }
                    .map_err(|error| Error::Render(error.to_string()))?;
                    assert_eq!(source.source, "1\u{0301}");
                    assert_eq!(
                        stage.font_resources().usage().unwrap().processing_visits,
                        12
                    );
                    let expected_bytes =
                        3 + 2 * "Test Sans".len() + std::mem::size_of::<FamilyPreference>();
                    assert_eq!(
                        stage.font_resources().usage().unwrap().font_bytes,
                        expected_bytes as u128
                    );
                    requirements.push(source);
                    let census = PreparedTextCensus::materialize(requirements, Some(stage), None)
                        .map_err(|error| Error::Render(error.to_string()))?;
                    assert_eq!(census.counts.scalar_occurrences, 2);
                    Ok(())
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            )
            .unwrap();
        }
    }

    #[test]
    fn staged_topology_is_admitted_before_requirement_storage_and_shared_addresses_count_once() {
        crate::layout::plan_exact_root(
            320,
            240,
            |stage| {
                // The exact-root driver already owns one structural slot.
                stage.claim_topology_slots(999_998)?;
                let appearance = appearance("Test Sans", TextWeight::Normal);
                let address = ComponentAddress::root();
                let mut requirements = Vec::new();
                for role in [PreparedTextRole::Title, PreparedTextRole::Subtitle] {
                    PreparedTextRequirement::push_into(
                        &mut requirements,
                        address.clone(),
                        role,
                        0,
                        "A",
                        &[],
                        &appearance,
                        Some(stage),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                }
                let other = address.child(AddressRole::RootCaption, 0)?;
                let capacity = requirements.capacity();
                let error = PreparedTextRequirement::push_into(
                    &mut requirements,
                    other,
                    PreparedTextRole::Caption,
                    0,
                    "A",
                    &[],
                    &appearance,
                    Some(stage),
                )
                .unwrap_err();
                assert!(error.to_string().contains("topology slots limit"));
                assert_eq!(requirements.len(), 2);
                assert_eq!(requirements.capacity(), capacity);
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }

    #[test]
    fn borrowed_source_work_refusal_precedes_source_retention() {
        crate::layout::plan_exact_root(
            320,
            240,
            |stage| {
                stage
                    .font_resources()
                    .claim_visits(1_000_000)
                    .map_err(|error| Error::Render(error.to_string()))?;
                let appearance = appearance("Test Sans", TextWeight::Normal);
                let error = PreparedTextRequirement::new_with_stage(
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    0,
                    "A\u{0301}",
                    &[],
                    &appearance,
                    Some(stage),
                )
                .unwrap_err();
                assert!(error.to_string().contains("root processing visits"));
                assert_eq!(stage.font_resources().usage().unwrap().font_bytes, 0);
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }

    #[test]
    fn staged_materialization_moves_semantic_sources_without_a_second_census_copy() {
        crate::layout::plan_exact_root(
            320,
            240,
            |stage| {
                let appearance = appearance("Test Sans", TextWeight::Normal);
                let source = String::from("A\u{0301}");
                let source_pointer = source.as_ptr();
                let mut requirements = Vec::new();
                PreparedTextRequirement::push_into(
                    &mut requirements,
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    0,
                    source,
                    &[],
                    &appearance,
                    Some(stage),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                let storage_pointer = requirements.as_ptr();
                assert_eq!(requirements[0].source.as_ptr(), source_pointer);
                let work = stage.font_resources().usage().unwrap().processing_visits;
                assert_eq!(work, 2 + 1 + "Test Sans".len() as u128);
                let mut presence = stage.reserve_scalar_presence_map()?;
                let census =
                    PreparedTextCensus::materialize(requirements, Some(stage), Some(&mut presence))
                        .map_err(|error| Error::Render(error.to_string()))?;
                assert_eq!(census.requirements.as_ptr(), storage_pointer);
                assert_eq!(census.requirements[0].source.as_ptr(), source_pointer);
                assert_eq!(census.counts.structural_addresses, 1);
                assert_eq!(census.counts.scalar_occurrences, 2);
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }

    fn appearance(stack: &str, weight: TextWeight) -> TextAppearance {
        TextAppearance {
            stack: stack.to_owned(),
            paint: Rgba::rgb(0, 0, 0),
            font_size: 12,
            weight,
            line_height_multiplier: 1.2,
        }
    }

    fn requirement(
        address: ComponentAddress,
        role: PreparedTextRole,
        index: u32,
        source: &str,
        generated: &[char],
        appearance: &TextAppearance,
    ) -> PreparedTextRequirement {
        PreparedTextRequirement::new(address, role, index, source, generated, appearance).unwrap()
    }

    #[test]
    fn census_orders_records_and_assigns_structural_pairs_by_first_use() {
        let root = ComponentAddress::root();
        let panel = root.child(AddressRole::DataPanel, 0).unwrap();
        let normal = appearance(" Test Sans , sans-serif ", TextWeight::Normal);
        let same_normal = appearance("Test Sans,sans-serif", TextWeight::Normal);
        let bold = appearance("Test Sans, sans-serif", TextWeight::Bold);
        let census = PreparedTextCensus::materialize(
            vec![
                requirement(
                    panel.clone(),
                    PreparedTextRole::SignificanceLabel,
                    0,
                    "*",
                    &[],
                    &bold,
                ),
                requirement(
                    root.clone(),
                    PreparedTextRole::Caption,
                    0,
                    "caption",
                    &[],
                    &same_normal,
                ),
                requirement(root, PreparedTextRole::Title, 0, "title", &[], &normal),
            ],
            None,
            None,
        )
        .unwrap();

        assert_eq!(census.requirements[0].text_role, PreparedTextRole::Title);
        assert_eq!(census.requirements[1].text_role, PreparedTextRole::Caption);
        assert_eq!(census.requirements[2].pair_ordinal, 1);
        assert_eq!(census.pairs.len(), 2);
        assert_eq!(census.counts.structural_addresses, 2);
        assert_eq!(
            census
                .pair_ordinal(&normal.stack, TextWeight::Normal)
                .unwrap(),
            0
        );
        assert_eq!(
            census.pair_ordinal(&bold.stack, TextWeight::Bold).unwrap(),
            1
        );
    }

    #[test]
    fn duplicate_scalar_occurrences_charge_work_but_pair_membership_deduplicates() {
        let appearance = appearance("Test Sans", TextWeight::Normal);
        let mut census = PreparedTextCensus::materialize(
            vec![requirement(
                ComponentAddress::root(),
                PreparedTextRole::TickLabel,
                0,
                "101",
                &['-', '.', '0', '1'],
                &appearance,
            )],
            None,
            None,
        )
        .unwrap();
        let mut presence = vec![0; SCALAR_MAP_WORDS];
        census
            .materialize_required_scalars(&mut presence, None)
            .unwrap();

        assert_eq!(census.counts.scalar_occurrences, 7);
        assert_eq!(census.counts.unique_pair_scalars, 4);
        assert_eq!(census.pairs[0].required_scalars, ['1', '0', '-', '.']);
    }

    #[test]
    fn empty_exact_label_retains_record_identity_without_scalars() {
        let appearance = appearance("Test Sans", TextWeight::Normal);
        let census = PreparedTextCensus::materialize(
            vec![requirement(
                ComponentAddress::root(),
                PreparedTextRole::TickLabel,
                4,
                "",
                &[],
                &appearance,
            )],
            None,
            None,
        )
        .unwrap();

        assert_eq!(census.counts.requirements, 1);
        assert_eq!(census.counts.scalar_occurrences, 0);
        assert!(census.pairs[0].required_scalars.is_empty());
    }

    #[test]
    fn automatic_numeric_alphabet_is_the_exact_sorted_closed_grammar() {
        assert_eq!(
            AUTOMATIC_NUMERIC_ALPHABET,
            ['-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'M', '^', 'e', 'k']
        );
        assert!(AUTOMATIC_NUMERIC_ALPHABET
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn automatic_numeric_axes_contribute_the_closed_alphabet_and_exact_ticks_do_not() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .x_ticks(vec![crate::layout::AxisTick::new(1.0, "exact")])
        .erase();
        let theme = crate::Theme::Publication.resolve();
        let requirements = foundational_plot_requirements(
            None,
            &plot,
            PlotKind::Scatter,
            &theme,
            &ComponentAddress::root(),
            true,
            true,
            true,
            None,
        )
        .unwrap();
        let generated = requirements
            .iter()
            .filter(|requirement| !requirement.generated_alphabet.is_empty())
            .collect::<Vec<_>>();

        assert_eq!(generated.len(), 1);
        assert_eq!(
            generated[0].generated_alphabet.as_ref(),
            AUTOMATIC_NUMERIC_ALPHABET
        );
        assert!(requirements.iter().any(|requirement| {
            requirement.text_role == PreparedTextRole::TickLabel
                && requirement.source == "exact"
                && requirement.generated_alphabet.is_empty()
        }));
    }

    #[test]
    fn generated_metric_scalars_are_selected_and_missing_generated_coverage_is_fatal() {
        let appearance = appearance("Test Sans", TextWeight::Normal);
        for (generated, succeeds) in [(&['A'][..], true), (&['M'][..], false)] {
            let mut census = PreparedTextCensus::materialize(
                vec![requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::Body,
                    0,
                    "",
                    generated,
                    &appearance,
                )],
                None,
                None,
            )
            .unwrap();
            let environment = PreparedFontEnvironment::synthetic();
            let result = census.prepare_fonts_in(&environment, &mut vec![0; SCALAR_MAP_WORDS]);
            if succeeds {
                result.unwrap();
                let stack = canonical::parse_family_stack("Test Sans").unwrap();
                let pair = environment
                    .prepared_pair(&stack, RequestedWeight::Normal)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    pair.selection_for_unit("A", 'A').unwrap(),
                    canonical::ScalarFaceSelection::Face(pair.primary)
                );
            } else {
                assert!(matches!(
                    result,
                    Err(PreparedTextCensusError::Font(
                        canonical::FontError::MissingGlyph { scalar: 0x4D, .. }
                    ))
                ));
            }
        }
    }

    #[test]
    fn census_resolves_each_pair_once_with_its_complete_scalar_union() {
        let normal = appearance("Test Sans", TextWeight::Normal);
        let bold = appearance("Test Sans", TextWeight::Bold);
        let mut census = PreparedTextCensus::materialize(
            vec![
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    0,
                    "A",
                    &[],
                    &normal,
                ),
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::SignificanceLabel,
                    0,
                    " ",
                    &[],
                    &bold,
                ),
            ],
            None,
            None,
        )
        .unwrap();
        let environment = PreparedFontEnvironment::synthetic();
        let mut presence = vec![0; SCALAR_MAP_WORDS];

        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
        census
            .prepare_fonts_in(&environment, &mut presence)
            .unwrap();

        let stack = canonical::parse_family_stack("Test Sans").unwrap();
        assert_eq!(
            environment
                .prepared_pair(&stack, RequestedWeight::Normal)
                .unwrap()
                .unwrap()
                .selected_scalars
                .len(),
            1
        );
        assert_eq!(
            environment
                .prepared_pair(&stack, RequestedWeight::Bold)
                .unwrap()
                .unwrap()
                .provenance
                .pair_ordinal,
            1
        );
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));

        environment
            .plan_prepared_text(crate::layout::TextPlanInput {
                source: "A",
                stack: "Test Sans",
                weight: TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: crate::layout::PhysicalTextAlignment::Start,
                paint: Rgba::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: u32::MAX,
                    owner: "prepared census test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));

        let missing = environment
            .plan_prepared_text(crate::layout::TextPlanInput {
                source: "B",
                stack: "Test Sans",
                weight: TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: crate::layout::PhysicalTextAlignment::Start,
                paint: Rgba::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: u32::MAX,
                    owner: "uncensused scalar test".into(),
                },
                exact_override: false,
            })
            .unwrap_err();
        assert!(matches!(
            missing,
            crate::layout::TextPlanError::MissingPreparedScalar(0x42)
        ));
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }
}
