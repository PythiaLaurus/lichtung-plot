//! Canonically ordered text and font-pair preparation requirements.

use crate::font::canonical::{
    self, FamilyPreference, FontPairPreparationRequest, PreparedFontEnvironment, RequestedWeight,
};
use crate::layout::kernel::ComponentAddress;
use crate::theme::TextAppearance;
use crate::{Error, Plot, PlotKind, TextWeight};

const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;
const AUTOMATIC_NUMERIC_ALPHABET: &[char] = &[
    '-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'M', '^', 'e', 'k',
];
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
}

impl PreparedTextRequirement {
    pub(crate) fn new(
        component_address: ComponentAddress,
        text_role: PreparedTextRole,
        local_index: u32,
        source: impl AsRef<str>,
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

    pub(crate) fn new_with_stage(
        component_address: ComponentAddress,
        text_role: PreparedTextRole,
        local_index: u32,
        source: impl AsRef<str>,
        generated_alphabet: &[char],
        appearance: &TextAppearance,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, PreparedTextCensusError> {
        if generated_alphabet.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(PreparedTextCensusError::GeneratedAlphabetOrder);
        }
        let source = source.as_ref();
        let mut retained_source = String::new();
        reserve_census_string(
            &mut retained_source,
            source.len(),
            "prepared text source",
            prepared_stage.as_deref_mut(),
        )?;
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
        let stack = canonical::parse_family_stack_with_stage(&appearance.stack, prepared_stage)?;
        retained_source.push_str(source);
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
        })
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
        requirements.sort_by(|left, right| {
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
        let mut retained_requirement_shells = if let Some(stage) = prepared_stage.as_deref_mut() {
            Some(reserve_prepared_requirement_shells(&requirements, stage)?)
        } else {
            None
        };

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
        if let Some(mut retained) = retained_requirement_shells.take() {
            fill_prepared_requirement_shells(&mut retained, &requirements)?;
            requirements = retained;
        }

        if pairs.len() != pair_count {
            return Err(PreparedTextCensusError::Arithmetic(
                "prepared font pair count/materialization mismatch",
            ));
        }
        let structural_addresses = Self::structural_address_count(&requirements)?;
        let scalar_occurrences = requirements.iter().try_fold(0_u128, |total, requirement| {
            let source = u128::try_from(requirement.source.chars().count()).map_err(|_| {
                PreparedTextCensusError::Arithmetic("prepared text scalar occurrences")
            })?;
            let generated = u128::try_from(requirement.generated_alphabet.len()).map_err(|_| {
                PreparedTextCensusError::Arithmetic("prepared text scalar occurrences")
            })?;
            total
                .checked_add(source)
                .and_then(|total| total.checked_add(generated))
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
        let scalar_occurrences = self.scalar_occurrence_count()?;
        let discovery = PreparedFontEnvironment::reserve_system_discovery(prepared_stage)?;
        prepared_stage
            .font_resources()
            .claim_work(scalar_occurrences)?;
        self.materialize_required_scalars(&mut presence, Some(prepared_stage))?;
        let environment = PreparedFontEnvironment::discover_system_with_prepared_resources(
            presence,
            prepared_stage.font_resources(),
            prepared_stage.generation_resources(),
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
        capacities
            .try_reserve_exact(self.pairs.len())
            .map_err(|_| PreparedTextCensusError::Allocation("required pair scalar capacities"))?;
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
        environment.claim_preflight_work(self.scalar_occurrence_count()?)?;
        self.materialize_required_scalars(presence, None)?;
        self.prepare_pairs_in(environment, None)
    }

    fn prepare_pairs_in(
        &self,
        environment: &PreparedFontEnvironment,
        prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
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
            prepared_stage,
        )?;
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

fn reserve_prepared_requirement_shells(
    source: &[PreparedTextRequirement],
    stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Vec<PreparedTextRequirement>, PreparedTextCensusError> {
    let mut retained = Vec::new();
    reserve_census_vec(
        &mut retained,
        source.len(),
        "prepared text requirements",
        Some(stage),
    )?;
    for requirement in source {
        let mut retained_source = String::new();
        reserve_census_string(
            &mut retained_source,
            requirement.source.len(),
            "prepared text source",
            Some(stage),
        )?;
        let mut retained_alphabet = Vec::new();
        reserve_census_vec(
            &mut retained_alphabet,
            requirement.generated_alphabet.len(),
            "prepared generated alphabet",
            Some(stage),
        )?;
        retained_alphabet.resize(requirement.generated_alphabet.len(), '\0');
        let mut stack_source = String::new();
        reserve_census_string(
            &mut stack_source,
            requirement.stack_source.len(),
            "prepared font stack source",
            Some(stage),
        )?;
        retained.push(PreparedTextRequirement {
            component_address: requirement
                .component_address
                .prepared_copy_shell(stage)
                .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?,
            text_role: requirement.text_role,
            local_index: requirement.local_index,
            source: retained_source,
            generated_alphabet: retained_alphabet.into_boxed_slice(),
            stack_source,
            stack: reserve_family_stack_shell(&requirement.stack, Some(stage))?,
            weight: requirement.weight,
            pair_ordinal: u32::MAX,
        });
    }
    Ok(retained)
}

fn fill_prepared_requirement_shells(
    retained: &mut [PreparedTextRequirement],
    source: &[PreparedTextRequirement],
) -> Result<(), PreparedTextCensusError> {
    if retained.len() != source.len() {
        return Err(PreparedTextCensusError::Arithmetic(
            "prepared text requirements",
        ));
    }
    for (retained, source) in retained.iter_mut().zip(source) {
        retained
            .component_address
            .fill_prepared_copy(&source.component_address)
            .map_err(|error| PreparedTextCensusError::Reservation(error.to_string()))?;
        retained.source.push_str(&source.source);
        retained
            .generated_alphabet
            .copy_from_slice(&source.generated_alphabet);
        retained.stack_source.push_str(&source.stack_source);
        fill_family_stack_shell(&mut retained.stack, &source.stack)?;
        retained.pair_ordinal = source.pair_ordinal;
    }
    Ok(())
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
    capacities
        .try_reserve_exact(pairs.len())
        .map_err(|_| PreparedTextCensusError::Allocation("required pair scalar capacities"))?;
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
    clippy::too_many_lines,
    reason = "the closed text census keeps both axis policies and canonical producer order explicit"
)]
pub(crate) fn foundational_plot_requirements(
    plot: &Plot,
    kind: PlotKind,
    theme: &crate::theme::ResolvedTheme,
    base: &ComponentAddress,
    include_caption: bool,
    filter_point_label_x_to_limits: bool,
    filter_point_label_y_to_limits: bool,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<PreparedTextRequirement>, Error> {
    use crate::layout::kernel::AddressRole;

    let mut requirements = Vec::new();
    let title_address = prepared_address_child(
        base,
        AddressRole::RootTitle,
        0,
        prepared_stage.as_deref_mut(),
    )?;
    push_present(
        &mut requirements,
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
        &mut requirements,
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
                &mut requirements,
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
                &mut requirements,
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
            &mut requirements,
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
            &mut requirements,
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
            &mut requirements,
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
        &mut requirements,
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
            &mut requirements,
            caption_address,
            PreparedTextRole::Caption,
            plot.caption_text(),
            &theme.typography.caption,
            prepared_stage,
        )?;
    }
    Ok(requirements)
}

#[expect(
    clippy::similar_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one closed plot-kind match keeps both range filters and every deferred T5b/T5e census source auditable"
)]
fn push_deferred_plot_requirements(
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
        for label in crate::orchestrator::facet_text_census_labels(plot)? {
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
                let (_, label) = plot.legend_identity(label).resolved_text(plot);
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
                        crate::data::observed_levels_for_finite_pairs(
                            plot.data(),
                            x,
                            y,
                            hue,
                            "scatter",
                        )?
                    } else {
                        crate::data::observed_levels_for_retained_finite_pairs(
                            plot.data(),
                            x,
                            y,
                            hue,
                            "marginal",
                        )?
                    }
                } else {
                    values.levels().to_vec()
                };
                for value in &levels {
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
    } else if plot.has_facets()
        && matches!(
            kind,
            PlotKind::Heatmap
                | PlotKind::Beeswarm
                | PlotKind::Coefficient
                | PlotKind::Forest
                | PlotKind::PointInterval
                | PlotKind::Waterfall
        )
    {
        for panel in crate::orchestrator::facet_text_census_panels(plot, kind)? {
            for source in specialized_text_census(
                panel.plot(),
                kind,
                panel.data(),
                filter_point_label_x_to_limits,
            )? {
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
    } else {
        for source in
            specialized_text_census(plot, kind, plot.data(), filter_point_label_x_to_limits)?
        {
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

    Ok(())
}

fn push_deferred_body(
    requirements: &mut Vec<PreparedTextRequirement>,
    body_address: &ComponentAddress,
    body_index: &mut u32,
    source: impl AsRef<str>,
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

fn legend_title_census(plot: &Plot, kind: PlotKind) -> Vec<String> {
    if legend_uses_pair_identity(plot, kind) {
        return vec![
            plot.resolve_label(plot.group_name().expect("pair identity has a group")),
            plot.resolve_label(plot.hue_name().expect("pair identity has a hue")),
            " × ".to_owned(),
        ];
    }
    let literal = matches!(kind, PlotKind::Dumbbell | PlotKind::Regression)
        || (kind == PlotKind::Line && plot.hue_dimension_key().is_none());
    vec![if literal {
        "series".to_owned()
    } else {
        plot.resolve_label(plot.legend_title_key())
    }]
}

fn specialized_text_census(
    plot: &Plot,
    kind: PlotKind,
    data: crate::data::PlotData<'_>,
    filter_x_to_limits: bool,
) -> Result<Vec<String>, Error> {
    use crate::data::DataColumn;

    let mut sources = Vec::new();
    match kind {
        PlotKind::Heatmap => {
            sources.push(plot.resolved_color_label());
            let heatmap = crate::plots::heatmap::text_census(plot, data)?;
            sources.extend(heatmap.colorbar_values);
            sources.extend(heatmap.value_annotations);
            if heatmap.has_missing {
                sources.push("Missing".to_owned());
            }
            sources.extend(
                plot.heatmap_null_row_annotations_spec()
                    .iter()
                    .map(|annotation| annotation.text().to_owned()),
            );
        }
        PlotKind::Beeswarm => {
            sources.push(plot.resolved_color_label());
            sources.extend(crate::plots::beeswarm::text_census_colorbar_values(
                plot, data,
            )?);
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
                sources.push("Not estimable".to_owned());
            }
            if let Some(format) = plot.estimate_labels_spec() {
                if let Ok(DataColumn::Numeric(values)) = data.column(crate::data::X) {
                    for row in 0..data.height() {
                        if let Some(Some(value)) = values.get(data.row_index(row)) {
                            sources.push(format.format(value));
                        }
                    }
                }
            }
        }
        PlotKind::Waterfall => {
            sources.extend(crate::plots::waterfall::text_census_annotations(
                plot,
                data,
                filter_x_to_limits,
            )?);
        }
        _ => {}
    }
    Ok(sources)
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
                tick.label.clone(),
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
            let levels = values
                .levels()
                .iter()
                .map(|value| {
                    if normalized {
                        crate::data::collapse_whitespace(value)
                    } else {
                        value.clone()
                    }
                })
                .collect::<Vec<_>>();
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
fn push_owned(
    requirements: &mut Vec<PreparedTextRequirement>,
    address: ComponentAddress,
    role: PreparedTextRole,
    local_index: u32,
    source: impl AsRef<str>,
    generated_alphabet: &[char],
    appearance: &TextAppearance,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    reserve_census_vec(
        requirements,
        1,
        "prepared text requirements",
        prepared_stage.as_deref_mut(),
    )
    .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
    requirements.push(
        PreparedTextRequirement::new_with_stage(
            address,
            role,
            local_index,
            source,
            generated_alphabet,
            appearance,
            prepared_stage,
        )
        .map_err(|error| Error::Render(format!("prepared text census: {error}")))?,
    );
    Ok(())
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
        for scalar in requirement.source.chars() {
            visit(scalar)?;
        }
        for &scalar in &requirement.generated_alphabet {
            visit(scalar)?;
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PreparedTextCensusError {
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
