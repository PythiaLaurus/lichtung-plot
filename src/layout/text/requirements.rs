//! Canonically ordered text and font-pair preparation requirements.

use std::borrow::Cow;

#[cfg(test)]
use crate::font::canonical::RequestedWeight;
use crate::font::canonical::{
    self, CensusSource, FontPairPreparationRequest, FontStack, PreparedFontEnvironment,
};
use crate::layout::kernel::ComponentAddress;
use crate::theme::TextTheme;
use crate::{Error, Plot, PlotKind, TextWeight};

const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;
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
    PreferenceMetric,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedTextRequirement {
    pub(crate) component_address: ComponentAddress,
    pub(crate) text_role: PreparedTextRole,
    pub(crate) local_index: u32,
    pub(crate) source: String,
    pub(crate) generated_alphabet: Box<[char]>,
    pub(crate) stack: FontStack,
    pub(crate) weight: TextWeight,
    pub(crate) pair_ordinal: u32,
}

impl PreparedTextRequirement {
    /// Role-local `ch` metrics are known actual sources, prepared along with
    /// authored text before bounded layout starts.
    pub(crate) fn append_preference_metrics(
        requirements: &mut Vec<Self>,
        theme: &crate::theme::ResolvedTheme,
        owner: &ComponentAddress,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(), Error> {
        for (index, appearance) in [
            &theme.typography.root.title,
            &theme.typography.root.subtitle,
            &theme.typography.root.caption,
            &theme.typography.axes.tick_label,
            &theme.typography.guides.legend_title,
            &theme.typography.guides.legend_label,
            &theme.typography.composition.composition_header,
            &theme.typography.composition.text_panel_title,
            &theme.typography.composition.facet_header,
            &theme.typography.composition.body,
        ]
        .into_iter()
        .enumerate()
        {
            let mut used = false;
            for requirement in requirements.iter() {
                if requirement.weight == appearance.weight
                    && requirement.stack.same(&appearance.stack)
                {
                    used = true;
                    break;
                }
            }
            if !used {
                continue;
            }
            let address = owner.child(
                crate::layout::AddressRole::PreferenceMetric,
                u32::try_from(index).map_err(|_| {
                    Error::Render("preference metric census address overflowed".into())
                })?,
            )?;
            Self::push_into(
                requirements,
                address,
                PreparedTextRole::PreferenceMetric,
                0,
                "0",
                &[],
                appearance,
                Some(stage),
            )
            .map_err(|error| Error::Render(format!("preference metric census: {error}")))?;
        }
        Ok(())
    }
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
        appearance: &TextTheme,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), PreparedTextCensusError> {
        #[cfg(not(test))]
        if prepared_stage.is_none() {
            return Err(PreparedTextCensusError::MissingPreparedStage);
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
        appearance: &TextTheme,
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
        appearance: &TextTheme,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, PreparedTextCensusError> {
        if generated_alphabet.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(PreparedTextCensusError::GeneratedAlphabetOrder);
        }
        let source = source.into();
        let stack_source = &appearance.stack;

        let (source, formatted) = match source {
            CensusSource::Text(source) => (source, false),
            CensusSource::Formatted(arguments) => (
                Cow::Owned(canonical::format_census_source(
                    arguments,
                    prepared_stage.as_deref_mut(),
                )?),
                true,
            ),
        };
        crate::text::validate(&source).map_err(|error| {
            canonical::FontError::UnsupportedControl {
                scalar: error.scalar,
            }
        })?;
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
        let stack = stack_source.copy_with_stage(prepared_stage)?;
        retained_alphabet.extend_from_slice(generated_alphabet);
        Ok(Self {
            component_address,
            text_role,
            local_index,
            source: retained_source,
            generated_alphabet: retained_alphabet.into_boxed_slice(),
            stack,
            weight: appearance.weight,
            pair_ordinal: u32::MAX,
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
    pub(crate) stack: FontStack,
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
        #[cfg(not(test))]
        if prepared_stage.is_none() {
            return Err(PreparedTextCensusError::MissingPreparedStage);
        }

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
        let mut pair_count = 0;
        for index in 0..requirements.len() {
            if pair_first_use(&requirements, index) {
                pair_count += 1;
            }
        }
        let mut first_use_indices = Vec::new();
        reserve_census_vec(
            &mut first_use_indices,
            pair_count,
            "prepared font pair first uses",
            prepared_stage.as_deref_mut(),
        )?;
        for index in 0..requirements.len() {
            if pair_first_use(&requirements, index) {
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
            retained_stacks.push(retain_family_stack(
                &requirements[*index].stack,
                prepared_stage.as_deref_mut(),
            )?);
        }
        let mut pairs: Vec<PreparedFontPairRequirement> = Vec::new();
        reserve_census_vec(
            &mut pairs,
            pair_count,
            "prepared font pairs",
            prepared_stage.as_deref_mut(),
        )?;
        for (pair_index, (&requirement_index, stack)) in
            first_use_indices.iter().zip(retained_stacks).enumerate()
        {
            let requirement = &requirements[requirement_index];
            pairs.push(PreparedFontPairRequirement {
                pair_ordinal: u32::try_from(pair_index)
                    .map_err(|_| PreparedTextCensusError::Arithmetic("font pair ordinal"))?,
                stack,
                weight: requirement.weight,
                required_scalars: Vec::new(),
            });
        }
        for requirement_index in 0..requirements.len() {
            let mut pair_index = None;
            for (ordinal, index) in first_use_indices.iter().enumerate() {
                if requirements[*index].weight == requirements[requirement_index].weight
                    && requirements[*index]
                        .stack
                        .same(&requirements[requirement_index].stack)
                {
                    pair_index = Some(ordinal);
                    break;
                }
            }
            let pair_index = pair_index.ok_or(PreparedTextCensusError::MissingPair)?;
            requirements[requirement_index].pair_ordinal = u32::try_from(pair_index)
                .map_err(|_| PreparedTextCensusError::Arithmetic("font pair ordinal"))?;
        }
        let unique_pair_scalars = if let Some(presence) = scalar_presence {
            reserve_required_scalar_capacities(&requirements, &mut pairs, presence, prepared_stage)?
        } else {
            0
        };

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
        stack: &FontStack,
        weight: TextWeight,
    ) -> Result<u32, PreparedTextCensusError> {
        for pair in &self.pairs {
            if pair.weight == weight && pair.stack.same(stack) {
                return Ok(pair.pair_ordinal);
            }
        }
        Err(PreparedTextCensusError::MissingPair)
    }

    pub(crate) fn prepare_system_fonts(
        &mut self,
        mut presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<PreparedFontEnvironment, PreparedTextCensusError> {
        self.materialize_required_scalars(&mut presence, Some(prepared_stage))?;
        let environment =
            PreparedFontEnvironment::discover_for_operation(presence, prepared_stage)?;
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
                    pair.weight.into(),
                    &pair.required_scalars,
                )
            }),
            prepared_stage.as_deref_mut(),
        )?;
        for requirement in &self.requirements {
            environment.admit_text(
                &requirement.stack,
                requirement.weight.into(),
                &requirement.source,
                prepared_stage.as_deref_mut(),
            )?;
        }
        Ok(())
    }
}

fn pair_first_use(requirements: &[PreparedTextRequirement], index: usize) -> bool {
    let requirement = &requirements[index];
    for earlier in &requirements[..index] {
        if earlier.weight == requirement.weight && earlier.stack.same(&requirement.stack) {
            return false;
        }
    }
    true
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

fn retain_family_stack(
    source: &FontStack,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontStack, PreparedTextCensusError> {
    Ok(source.copy_with_stage(prepared_stage)?)
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

fn prepared_address_child(
    base: &ComponentAddress,
    role: crate::layout::kernel::AddressRole,
    slot_index: u32,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<ComponentAddress, Error> {
    prepared_stage
        .ok_or_else(|| Error::Render(PreparedTextCensusError::MissingPreparedStage.to_string()))?;
    base.child(role, slot_index)
}

#[expect(
    clippy::similar_names,
    clippy::too_many_arguments,
    reason = "the closed text census keeps both axis policies and canonical producer order explicit"
)]
pub(crate) fn foundational_plot_requirements(
    interiors: Option<&crate::layout::interior::PreparedInteriorInputs>,
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
        interiors,
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
        interiors: Option<&crate::layout::interior::PreparedInteriorInputs>,
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
        Self::push_authored(
            requirements,
            title_address,
            PreparedTextRole::Title,
            plot.prose()[0],
            &theme.typography.root.title,
            true,
            prepared_stage.as_deref_mut(),
        )?;
        let subtitle_address = prepared_address_child(
            base,
            AddressRole::RootSubtitle,
            0,
            prepared_stage.as_deref_mut(),
        )?;
        Self::push_authored(
            requirements,
            subtitle_address,
            PreparedTextRole::Subtitle,
            plot.prose()[1],
            &theme.typography.root.subtitle,
            true,
            prepared_stage.as_deref_mut(),
        )?;
        if kind != PlotKind::Matrix {
            let [x_title, y_title] = plot.prepare_axis_labels(None)?;
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
            for (address, title) in [(&x_axis, x_title.as_ref()), (&y_axis, y_title.as_ref())] {
                Self::push_authored(
                    requirements,
                    address.clone(),
                    PreparedTextRole::AxisTitle,
                    title,
                    &theme.typography.axes.axis_title,
                    false,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            push_plot_axis_ticks(
                interiors,
                requirements,
                plot,
                kind,
                theme,
                base,
                prepared_stage.as_deref_mut(),
            )?;
            if matches!(
                crate::layout::interior::InteriorRoute::classify(kind),
                crate::layout::interior::InteriorRoute::StandardCoordinate(_)
            ) {
                let address = prepared_address_child(
                    base,
                    AddressRole::PanelInterior,
                    0,
                    prepared_stage.as_deref_mut(),
                )?;
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::TickLabel,
                    0,
                    "0",
                    &[],
                    &theme.typography.axes.tick_label,
                    prepared_stage.as_deref_mut(),
                )?;
                let address = prepared_address_child(
                    base,
                    AddressRole::PanelInterior,
                    1,
                    prepared_stage.as_deref_mut(),
                )?;
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::TickLabel,
                    0,
                    "0.5",
                    &[],
                    &theme.typography.axes.tick_label,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }

        let significance = theme.effective_significance(plot.significance_mark_style_override())?;
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
                &significance.label,
                prepared_stage.as_deref_mut(),
            )?;
        }
        if plot.legend_policy_spec() != crate::LegendPolicy::Hidden {
            if let Some(interiors) = interiors {
                let guides = interiors.discrete_guides.get().ok_or_else(|| {
                    Error::Render("discrete guide font census requires prepared semantics".into())
                })?;
                if plot.has_facets() && matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) {
                    for (index, panel) in guides.panels.iter().enumerate() {
                        if let Some(guide) = &panel.descriptor {
                            let index = u32::try_from(index).map_err(|_| {
                                Error::Render("native guide facet index overflow".into())
                            })?;
                            let owner = base
                                .child(AddressRole::FacetCell, index)?
                                .child(AddressRole::OutsideGuideRight, 0)?;
                            guide.append_requirements(
                                requirements,
                                theme,
                                &owner,
                                prepared_stage.as_deref_mut(),
                            )?;
                        }
                    }
                } else if let Some(guide) = &guides.root {
                    let guide_parent =
                        if plot.has_facets() || base.leaf_role() == AddressRole::CanvasCell {
                            base.clone()
                        } else {
                            base.child(AddressRole::PlotBody, 0)?
                        };
                    let role = if kind == PlotKind::Matrix {
                        AddressRole::InsideGuide
                    } else if matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) {
                        AddressRole::OutsideGuideRight
                    } else {
                        crate::decorations::legend::address_role(plot.legend_position_spec())
                    };
                    guide.append_requirements(
                        requirements,
                        theme,
                        &guide_parent.child(role, 0)?,
                        prepared_stage.as_deref_mut(),
                    )?;
                }
            }
        }
        push_deferred_plot_requirements(
            interiors,
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
            Self::push_authored(
                requirements,
                caption_address,
                PreparedTextRole::Caption,
                plot.prose()[2],
                &theme.typography.root.caption,
                true,
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
    interiors: Option<&crate::layout::interior::PreparedInteriorInputs>,
    requirements: &mut Vec<PreparedTextRequirement>,
    plot: &Plot,
    kind: PlotKind,
    theme: &crate::theme::ResolvedTheme,
    base: &ComponentAddress,
    filter_point_label_x_to_limits: bool,
    filter_point_label_y_to_limits: bool,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    use crate::layout::kernel::AddressRole;

    let body_address = prepared_address_child(
        base,
        AddressRole::SpecializedInterior,
        0,
        prepared_stage.as_deref_mut(),
    )?;
    let mut body_index = 0_u32;

    for annotation in plot.panel_text_annotations() {
        let appearance = annotation
            .content()
            .appearance(&theme.typography.annotations.panel_text);
        push_deferred_body(
            requirements,
            &body_address,
            &mut body_index,
            annotation.text(),
            &[],
            appearance.theme,
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
                &theme.typography.composition.facet_header,
                prepared_stage.as_deref_mut(),
            )
        })?;
    }
    for layer in plot.point_layers() {
        if let Some(labels) = layer.label_values() {
            let appearance =
                layer.label_appearance(&theme.typography.annotations.point_layer_label)?;
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
                        &appearance,
                        prepared_stage.as_deref_mut(),
                    )?;
                }
            }
        }
    }

    if kind == PlotKind::Matrix {
        for variable in plot.matrix_variable_names() {
            for _ in 0..2 {
                let address = body_address.clone();
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::AxisTitle,
                    body_index,
                    plot.resolve_label(variable)?,
                    &[],
                    &theme.typography.axes.axis_title,
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
            &[],
            &theme.typography.axes.tick_label,
            prepared_stage,
        )?;
    } else if kind == PlotKind::Waterfall {
        let waterfall = interiors
            .and_then(|inputs| inputs.waterfall.as_deref())
            .ok_or_else(|| {
                Error::Render("waterfall census requires root-prepared semantics".into())
            })?;
        for (scope, marks) in &waterfall.panels {
            if plot.axis_scope().is_some() && scope.as_deref() != plot.axis_scope() {
                continue;
            }
            for label in marks.total_font_sources() {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    label.as_str(),
                    &[],
                    label
                        .appearance(&theme.typography.annotations.waterfall_total)
                        .theme,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    } else if kind == PlotKind::Heatmap {
        let heatmap = interiors
            .and_then(|inputs| inputs.heatmap.as_deref())
            .ok_or_else(|| {
                Error::Render("heatmap census requires root-prepared semantics".into())
            })?;
        for (_, marks) in &heatmap.panels {
            if let Some(text) = marks.numeric_explanation() {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    text.as_str(),
                    &[],
                    text.appearance(theme.contrasting_text(theme.surfaces.get_plot_panel())?)
                        .theme,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            for (label, fill) in marks.annotation_labels(theme.marks.get_cells().get_fill_alpha()) {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    label,
                    &[],
                    theme.contrasting_text(fill)?,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            for annotation in marks.null_row_sources() {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    annotation.as_str(),
                    &[],
                    annotation
                        .appearance(
                            theme.contrasting_text(theme.marks.get_cells().get_missing_fill())?,
                        )
                        .theme,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    } else if matches!(
        kind,
        PlotKind::Coefficient | PlotKind::Forest | PlotKind::PointInterval
    ) {
        let estimates = interiors
            .and_then(|inputs| inputs.estimate.as_deref())
            .ok_or_else(|| {
                Error::Render("estimate detail census requires root-prepared semantics".into())
            })?;
        for (scope, marks) in &estimates.panels {
            if plot.axis_scope().is_some() && scope.as_deref() != plot.axis_scope() {
                continue;
            }
            if let Some(text) = marks.numeric_explanation() {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    text.as_str(),
                    &[],
                    text.appearance(&theme.typography.annotations.estimate_detail)
                        .theme,
                    prepared_stage.as_deref_mut(),
                )?;
            }
            for source in marks.detail_sources() {
                push_deferred_body(
                    requirements,
                    &body_address,
                    &mut body_index,
                    source,
                    &[],
                    &theme.typography.annotations.estimate_detail,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    }

    Ok(())
}

fn push_deferred_body<'a>(
    requirements: &mut Vec<PreparedTextRequirement>,
    body_address: &ComponentAddress,
    body_index: &mut u32,
    source: impl Into<CensusSource<'a>>,
    generated: &[char],
    appearance: &TextTheme,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    let address = body_address.clone();
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

impl PreparedTextRequirement {
    pub(crate) fn push_authored(
        requirements: &mut Vec<PreparedTextRequirement>,
        address: ComponentAddress,
        role: PreparedTextRole,
        source: Option<&crate::Text>,
        appearance: &TextTheme,
        preferred_width_metric: bool,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), Error> {
        if let Some(source) = source {
            let appearance = source.appearance(appearance).theme;
            let metric_address = preferred_width_metric
                .then(|| address.child(crate::layout::AddressRole::PreferenceMetric, 0))
                .transpose()?;
            push_owned(
                requirements,
                address,
                role,
                0,
                source.as_str(),
                &[],
                appearance,
                prepared_stage.as_deref_mut(),
            )?;
            if let Some(address) = metric_address {
                push_owned(
                    requirements,
                    address,
                    PreparedTextRole::PreferenceMetric,
                    0,
                    "0",
                    &[],
                    appearance,
                    prepared_stage,
                )?;
            }
        }
        Ok(())
    }
}

fn push_plot_axis_ticks(
    interiors: Option<&crate::layout::interior::PreparedInteriorInputs>,
    requirements: &mut Vec<PreparedTextRequirement>,
    plot: &Plot,
    kind: PlotKind,
    theme: &crate::theme::ResolvedTheme,
    base: &ComponentAddress,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    let alphabet = (kind == PlotKind::Heatmap
        && plot.heatmap_label_fit_spec() == crate::HeatmapLabelFit::Elide)
        .then_some(HEATMAP_ELISION_ALPHABET);
    if let Some(interiors) = interiors {
        for (index, (scope, axes)) in interiors.axes.iter().enumerate() {
            if plot.axis_scope().is_some() && plot.axis_scope() != scope.as_deref() {
                continue;
            }
            let owner = if plot.has_facets() {
                base.child(
                    crate::layout::AddressRole::FacetCell,
                    u32::try_from(index)
                        .map_err(|_| Error::Render("facet axis census address overflow".into()))?,
                )?
            } else {
                base.clone()
            };
            for side in 0..2 {
                if side == 1 && !plot.y_tick_labels_visible() {
                    continue;
                }
                let address = owner.child(
                    if side == 0 {
                        crate::layout::AddressRole::AxisBottom
                    } else {
                        crate::layout::AddressRole::AxisLeft
                    },
                    0,
                )?;
                let categories = axes.display_categories[side]
                    .as_ref()
                    .map(|labels| {
                        crate::data::source::collect_source(
                            labels
                                .iter()
                                .map(|label| crate::data::source::copy_source_string(label)),
                        )
                    })
                    .transpose()?;
                push_axis_ticks(
                    requirements,
                    &address,
                    &axes.specs[side],
                    categories,
                    alphabet,
                    &theme.typography.axes.tick_label,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    } else {
        for (side, spec) in [plot.x_axis_spec(), plot.y_axis_spec()]
            .into_iter()
            .enumerate()
        {
            if side == 1 && !plot.y_tick_labels_visible() {
                continue;
            }
            let address = base.child(
                if side == 0 {
                    crate::layout::AddressRole::AxisBottom
                } else {
                    crate::layout::AddressRole::AxisLeft
                },
                0,
            )?;
            push_axis_ticks(
                requirements,
                &address,
                spec,
                automatic_category_labels(plot, kind, side == 0)?,
                alphabet,
                &theme.typography.axes.tick_label,
                prepared_stage.as_deref_mut(),
            )?;
        }
    }
    Ok(())
}

fn push_axis_ticks(
    requirements: &mut Vec<PreparedTextRequirement>,
    axis: &ComponentAddress,
    spec: &crate::layout::AxisSpec,
    automatic_categories: Option<Vec<String>>,
    category_generated_alphabet: Option<&[char]>,
    appearance: &TextTheme,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), Error> {
    let exact = spec.exact_ticks.as_deref();
    if let Some(ticks) = exact {
        for (index, tick) in ticks.iter().enumerate() {
            let address = axis.clone();
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
        let count = categories.len();
        for (index, category) in categories.into_iter().enumerate() {
            if !spec.category_labels.includes(index, count) {
                continue;
            }
            let address = axis.clone();
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
        let address = axis.clone();
        push_owned(
            requirements,
            address,
            PreparedTextRole::TickLabel,
            0,
            String::new(),
            &[],
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
    appearance: &TextTheme,
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

    #[test]
    fn text_census_without_an_operation_stage_refuses_before_font_or_address_work() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [2.0]),
        )
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let error = foundational_plot_requirements(
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
        .unwrap_err();
        assert!(error.to_string().contains("root prepared stage"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            discoveries
        );
    }

    fn with_prepared_stage<T>(
        prepare: impl FnOnce(&mut crate::layout::kernel::PreparedStage<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let mut result = None;
        crate::layout::plan_exact_root(
            1,
            1,
            |stage| {
                result = Some(prepare(stage)?);
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )?;
        Ok(result.expect("root preparation completed"))
    }
    use crate::layout::kernel::AddressRole;
    use crate::theme::Color;

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
        assert!(!environment.prepared_pairs_are_sealed().unwrap());
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
    fn soft_hyphen_borrowed_and_chunked_sources_include_derived_literal_once() {
        for formatted in [false, true] {
            for alphabet in [&[][..], &['-'][..]] {
                {
                    crate::layout::plan_exact_root(
                        320,
                        240,
                        |stage| {
                            // One copy (entry + nine name bytes), source scalars/units,
                            // one derived literal, and the generated alphabet occurrences.

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
                            {
                                let requirement = result.unwrap();
                                assert_eq!(requirement.source, "A\u{00AD}\u{00AD}");
                                assert_eq!(
                                    requirement.scalar_occurrence_count().unwrap(),
                                    4 + u64::try_from(alphabet.len()).unwrap()
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
    fn formatted_source_census_preserves_the_complete_source() {
        for formatted in [false, true] {
            crate::layout::plan_exact_root(
                320,
                240,
                |stage| {
                    let appearance = appearance("Test Sans", TextWeight::Normal);
                    let format = crate::NumericFormat::new(
                        crate::NumericNotation::Decimal,
                        crate::NumericPrecision::Fractional(0),
                    )
                    .unwrap()
                    .suffix("\u{0301}")
                    .unwrap();
                    let mut requirements = Vec::new();
                    let source = if formatted {
                        PreparedTextRequirement::new_with_stage(
                            ComponentAddress::root(),
                            PreparedTextRole::Body,
                            0,
                            format_args!("{}", format.format_singleton(1.0, false)?),
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
    fn staged_requirements_preserve_shared_and_distinct_addresses() {
        crate::layout::plan_exact_root(
            320,
            240,
            |stage| {
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
                PreparedTextRequirement::push_into(
                    &mut requirements,
                    other,
                    PreparedTextRole::Caption,
                    0,
                    "A",
                    &[],
                    &appearance,
                    Some(stage),
                )
                .unwrap();
                assert_eq!(requirements.len(), 3);
                assert_eq!(
                    PreparedTextCensus::structural_address_count(&requirements).unwrap(),
                    2
                );
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }

    #[test]
    fn census_retains_an_independent_utf8_stack_copy() {
        {
            with_prepared_stage(|stage| {
                let appearance = appearance("α", TextWeight::Normal);

                let result = PreparedTextRequirement::new_with_stage(
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    0,
                    "",
                    &[],
                    &appearance,
                    Some(stage),
                );
                let requirement = result.unwrap();
                assert_eq!(requirement.stack, appearance.stack);
                assert_ne!(
                    requirement.stack.families().as_ptr(),
                    appearance.stack.families().as_ptr()
                );
                Ok(())
            })
            .unwrap();
        }
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

    fn appearance(stack: &str, weight: TextWeight) -> TextTheme {
        TextTheme::new(
            canonical::named_stack(&[stack]),
            Color::rgb(0, 0, 0),
            12,
            weight,
            1.2,
        )
        .unwrap()
    }

    fn requirement(
        address: ComponentAddress,
        role: PreparedTextRole,
        index: u32,
        source: &str,
        generated: &[char],
        appearance: &TextTheme,
    ) -> PreparedTextRequirement {
        PreparedTextRequirement::new(address, role, index, source, generated, appearance).unwrap()
    }

    #[test]
    fn census_orders_records_and_assigns_structural_pairs_by_first_use() {
        let root = ComponentAddress::root();
        let panel = root.child(AddressRole::DataPanel, 0).unwrap();
        let mut normal = appearance("Test Sans", TextWeight::Normal);
        normal.stack = FontStack::new([
            canonical::FontFamily::named("Test Sans").unwrap(),
            canonical::FontFamily::generic(canonical::GenericFontFamily::SansSerif),
        ])
        .unwrap();
        let same_normal = normal.clone();
        let bold = TextTheme {
            weight: TextWeight::Bold,
            ..normal.clone()
        };
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
    fn census_pair_identity_keeps_literal_commas_keywords_order_and_duplicates() {
        use canonical::{FontFamily, GenericFontFamily};
        let literal = canonical::named_stack(&["A, B"]);
        let separate = canonical::named_stack(&["A", "B"]);
        let keyword = canonical::named_stack(&["serif"]);
        let generic = FontStack::new([FontFamily::generic(GenericFontFamily::Serif)]).unwrap();
        let reverse = canonical::named_stack(&["B", "A"]);
        let repeated = canonical::named_stack(&["A", "A", "B"]);
        let stacks = [
            literal.clone(),
            separate,
            keyword,
            generic,
            reverse,
            repeated,
            literal,
        ];
        let requirements = stacks
            .iter()
            .enumerate()
            .map(|(index, stack)| {
                let appearance = TextTheme {
                    stack: stack.clone(),
                    ..appearance("A", TextWeight::Normal)
                };
                requirement(
                    ComponentAddress::root(),
                    PreparedTextRole::Title,
                    u32::try_from(index).unwrap(),
                    "X",
                    &[],
                    &appearance,
                )
            })
            .collect();
        let census = PreparedTextCensus::materialize(requirements, None, None).unwrap();
        assert_eq!(census.pairs.len(), 6);
        assert_eq!(
            census
                .requirements
                .iter()
                .map(|entry| entry.pair_ordinal)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3, 4, 5, 0]
        );
        for (index, stack) in stacks[..6].iter().enumerate() {
            assert_eq!(census.pairs[index].stack, *stack);
        }
    }

    #[test]
    fn staged_census_preserves_distinct_names_and_ordinals() {
        {
            with_prepared_stage(|stage| {
                let requirements = ["Alpha0", "Alpha1"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, name)| {
                        PreparedTextRequirement::new_with_stage(
                            ComponentAddress::root(),
                            PreparedTextRole::Title,
                            u32::try_from(index).unwrap(),
                            "X",
                            &[],
                            &appearance(name, TextWeight::Normal),
                            Some(stage),
                        )
                        .unwrap()
                    })
                    .collect();

                let result = PreparedTextCensus::materialize(requirements, Some(stage), None);
                {
                    let census = result.unwrap();
                    assert_eq!(census.pairs.len(), 2);
                    assert_eq!(
                        census
                            .requirements
                            .iter()
                            .map(|entry| entry.pair_ordinal)
                            .collect::<Vec<_>>(),
                        [0, 1]
                    );
                }

                Ok(())
            })
            .unwrap();
        }
    }

    #[test]
    fn duplicate_scalar_occurrences_preserve_unique_pair_membership() {
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
    fn numeric_sources_admit_only_reached_affix_units_with_stable_choices() {
        let format = crate::numeric::format::NumericFormat::default()
            .precision(crate::NumericPrecision::Fractional(0))
            .unwrap()
            .suffix("\u{0301}")
            .unwrap();
        let mut appearance = appearance("Digits", TextWeight::Normal);
        appearance.stack = canonical::named_stack(&["Digits", "Marks", "Whole"]);
        let spec = crate::layout::AxisSpec {
            format: Some(std::sync::Arc::new(format.clone())),
            ..Default::default()
        };
        let requirements = with_prepared_stage(|stage| {
            let mut requirements = Vec::new();
            push_axis_ticks(
                &mut requirements,
                &ComponentAddress::root(),
                &spec,
                None,
                None,
                &appearance,
                Some(stage),
            )?;
            Ok(requirements)
        })
        .unwrap();
        let environment = PreparedFontEnvironment::synthetic_faces(&[
            ("Digits", &['0', '1']),
            ("Marks", &['\u{0301}']),
            ("Whole", &['0', '1', '\u{0301}']),
        ]);
        let mut census = PreparedTextCensus::materialize(requirements, None, None).unwrap();
        census
            .prepare_fonts_in(&environment, &mut vec![0; SCALAR_MAP_WORDS])
            .unwrap();
        assert!(!environment.prepared_pairs_are_sealed().unwrap());
        environment
            .admit_text(&appearance.stack, RequestedWeight::Normal, "1", None)
            .unwrap();
        let source = format.format_singleton(1.0, false).unwrap();
        assert_eq!(source, "1\u{0301}");
        let admissions = canonical::FONT_SOURCE_ADMISSIONS.with(std::cell::Cell::get);
        assert_eq!(
            environment
                .admit_text(&appearance.stack, RequestedWeight::Normal, &source, None)
                .unwrap(),
            0
        );
        let pair = environment
            .prepared_pair(&appearance.stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(
            pair.selection_for_unit("1", '1').unwrap(),
            canonical::ScalarFaceSelection::Face(pair.primary)
        );
        assert_eq!(
            pair.selection_for_unit(&source, '1').unwrap(),
            canonical::ScalarFaceSelection::Face(pair.provenance.candidates[2])
        );
        assert!(!pair
            .selected_scalars
            .iter()
            .any(|entry| entry.scalar == '0'));
        assert_eq!(pair.selected_units.len(), 1);
        let probes = canonical::FONT_SELECTION_PROBES.with(std::cell::Cell::get);
        environment
            .admit_text(&appearance.stack, RequestedWeight::Normal, &source, None)
            .unwrap();
        assert_eq!(
            canonical::FONT_SOURCE_ADMISSIONS.with(std::cell::Cell::get),
            admissions + 1
        );
        assert_eq!(
            canonical::FONT_SELECTION_PROBES.with(std::cell::Cell::get),
            probes
        );
        assert_eq!(
            environment
                .admit_text(&appearance.stack, RequestedWeight::Bold, "0", None)
                .unwrap(),
            1
        );
        assert_eq!(
            environment
                .prepared_pair(&appearance.stack, RequestedWeight::Normal)
                .unwrap()
                .unwrap(),
            pair
        );
        environment.operation().seal_fonts().unwrap();
        assert!(matches!(
            environment.admit_text(
                &appearance.stack,
                RequestedWeight::Normal,
                "0\u{0301}",
                None
            ),
            Err(canonical::FontError::PreparedPairAfterSeal { pair_ordinal: 0 })
        ));
    }

    #[test]
    fn numeric_axes_register_pairs_and_exact_ticks_suppress_replaced_format_sources() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .x_tick_format(
            crate::numeric::format::NumericFormat::default()
                .suffix("\u{10ffff}")
                .unwrap(),
        )
        .x_ticks(vec![crate::layout::AxisTick::new(1.0, "exact")])
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let requirements = with_prepared_stage(|stage| {
            foundational_plot_requirements(
                None,
                &plot,
                PlotKind::Scatter,
                &theme,
                &ComponentAddress::root(),
                true,
                true,
                true,
                Some(stage),
            )
        })
        .unwrap();
        let generated = requirements
            .iter()
            .filter(|requirement| !requirement.generated_alphabet.is_empty())
            .collect::<Vec<_>>();

        assert!(generated.is_empty());
        assert!(requirements.iter().any(|requirement| {
            requirement.component_address.leaf_role() == crate::layout::AddressRole::AxisLeft
                && requirement.text_role == PreparedTextRole::TickLabel
                && requirement.source.is_empty()
        }));
        assert!(requirements.iter().any(|requirement| {
            requirement.text_role == PreparedTextRole::TickLabel
                && requirement.source == "exact"
                && requirement.generated_alphabet.is_empty()
        }));
        assert!(requirements
            .iter()
            .all(|requirement| !requirement.source.contains('\u{10ffff}')));
    }

    #[test]
    fn heatmap_value_census_requests_only_the_contrast_role_chosen_for_each_fill() {
        for (value, label, alpha, chosen) in [
            (0.0, "0.00", 255, "Dark contrast font"),
            (1.0, "1.00", 255, "Light contrast font"),
            (1.0, "1.00", 0, "Dark contrast font"),
        ] {
            let plot = crate::Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", [value, value + 0.01]),
            )
            .continuous_color_scale(
                crate::ContinuousColorScale::diverging(
                    0.0,
                    None,
                    Some(crate::ColorDomain::exact(-1.0, 1.0).unwrap()),
                )
                .unwrap(),
            )
            .annotations(crate::HeatmapAnnotations::Values(
                crate::NumericFormat::default()
                    .precision(crate::NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
                    .unwrap(),
            ))
            .with_alpha(
                crate::alpha::AlphaTarget::Fill,
                crate::alpha::AlphaEncoding::observations(
                    crate::alpha::AlphaValues::new("cell confidence", [alpha; 2]).unwrap(),
                ),
            )
            .erase();
            let resources = crate::font::canonical::OperationContext::new();
            let inputs = crate::plots::shared_x::prepare_plot(
                &plot,
                &resources,
                Some(&crate::theme::Theme::publication()),
            )
            .unwrap();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            let cells = theme.marks.get_cells().clone().fill_alpha(0).unwrap();
            theme.marks = theme.marks.clone().cells(cells).unwrap();
            theme.typography.annotations.contrast.light.stack =
                canonical::named_stack(&["Light contrast font"]);
            theme.typography.annotations.contrast.dark.stack =
                canonical::named_stack(&["Dark contrast font"]);
            theme.typography.annotations.contrast.light.font_size = 19;
            theme.typography.annotations.contrast.dark.font_size = 9;
            inputs
                .prepare_discrete_guides(
                    &plot,
                    &theme,
                    &resources,
                    &ComponentAddress::root(),
                    false,
                )
                .unwrap();
            let requirements = with_prepared_stage(|stage| {
                foundational_plot_requirements(
                    Some(&inputs),
                    &plot,
                    PlotKind::Heatmap,
                    &theme,
                    &ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )
            })
            .unwrap();
            let selected = requirements
                .iter()
                .find(|requirement| requirement.source == label)
                .unwrap();
            assert_eq!(selected.stack, canonical::named_stack(&[chosen]));
            assert!(!requirements.iter().any(|requirement| {
                requirement.source == label
                    && requirement.stack != canonical::named_stack(&[chosen])
            }));
        }
    }

    #[test]
    fn transparent_missing_row_census_uses_the_final_fill_and_planning_backdrop() {
        use crate::theme::{Color, OpaqueColor};

        let plot = crate::Plot::heatmap(
            crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
            crate::CategoryValues::new("row", ["absent", "absent"]).unwrap(),
            crate::NumericValues::nullable("fill", [None, None]),
        )
        .continuous_color_scale(
            crate::ContinuousColorScale::sequential(
                None,
                Some(crate::ColorDomain::exact(0.0, 1.0).unwrap()),
            )
            .unwrap(),
        )
        .heatmap_null_row_annotation(
            crate::HeatmapNullRowAnnotation::new("absent", "Unavailable").unwrap(),
        )
        .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &resources,
            Some(&crate::theme::Theme::publication()),
        )
        .unwrap();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.surfaces.plot_panel = Color::rgba(0, 0, 0, 0);
        theme.surfaces.canvas_root = Color::rgba(0, 0, 0, 0);
        let cells = theme
            .marks
            .get_cells()
            .clone()
            .missing_fill(Color::rgba(20, 40, 60, 0))
            .unwrap();
        theme.marks = theme.marks.clone().cells(cells).unwrap();
        theme.typography.annotations.contrast.light.stack =
            canonical::named_stack(&["Light missing row"]);
        theme.typography.annotations.contrast.dark.stack =
            canonical::named_stack(&["Dark missing row"]);
        theme.typography.annotations.contrast.light.font_size = 39;
        theme.typography.annotations.contrast.dark.font_size = 9;
        inputs
            .prepare_discrete_guides(&plot, &theme, &resources, &ComponentAddress::root(), false)
            .unwrap();
        for (backdrop, expected) in [
            (OpaqueColor::rgb(0, 0, 0), "Light missing row"),
            (OpaqueColor::rgb(255, 255, 255), "Dark missing row"),
        ] {
            theme.annotations = theme
                .annotations
                .clone()
                .planning_backdrop(backdrop)
                .unwrap();
            let requirements = with_prepared_stage(|stage| {
                foundational_plot_requirements(
                    Some(&inputs),
                    &plot,
                    PlotKind::Heatmap,
                    &theme,
                    &ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )
            })
            .unwrap();
            let selected = requirements
                .iter()
                .filter(|requirement| requirement.source == "Unavailable")
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].stack, canonical::named_stack(&[expected]));
            assert_eq!(
                theme
                    .contrasting_text(theme.marks.get_cells().get_missing_fill())
                    .unwrap()
                    .font_size,
                if expected == "Light missing row" {
                    39
                } else {
                    9
                }
            );
        }
    }

    #[test]
    fn final_shared_heatmap_domain_reselects_complete_contrast_role_before_fonts() {
        let plot = crate::Plot::heatmap(
            crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
            crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
            crate::NumericValues::new("fill", [0.0, 1.0]),
        )
        .annotations(crate::HeatmapAnnotations::Values(
            crate::NumericFormat::default()
                .precision(crate::NumericPrecision::Fractional(2))
                .unwrap(),
        ))
        .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let mut inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &resources,
            Some(&crate::theme::Theme::publication()),
        )
        .unwrap();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.annotations.contrast.light.stack =
            canonical::named_stack(&["Selected light contrast"]);
        theme.typography.annotations.contrast.dark.stack =
            canonical::named_stack(&["Selected dark contrast"]);
        theme.typography.annotations.contrast.light.font_size = 39;
        theme.typography.annotations.contrast.dark.font_size = 9;
        inputs
            .prepare_discrete_guides(&plot, &theme, &resources, &ComponentAddress::root(), false)
            .unwrap();
        let selected_stack = |inputs: &crate::layout::interior::PreparedInteriorInputs| {
            let requirements = with_prepared_stage(|stage| {
                foundational_plot_requirements(
                    Some(inputs),
                    &plot,
                    PlotKind::Heatmap,
                    &theme,
                    &ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )
            })?;
            Ok::<_, Error>(
                requirements
                    .iter()
                    .find(|requirement| requirement.source == "1.00")
                    .unwrap()
                    .stack
                    .clone(),
            )
        };
        assert_eq!(
            selected_stack(&inputs).unwrap(),
            canonical::named_stack(&["Selected light contrast"])
        );
        let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
            selection: crate::theme::ContinuousColorScale::sequential(
                Some(
                    crate::theme::scale_values::test_defaults()
                        .get_sequential()
                        .clone(),
                ),
                None,
            )
            .unwrap(),
            field: "fill".into(),
            title: std::sync::Arc::new("Fill".into()),
            stops: vec![
                crate::render::backend::GradientStop::new(0.0, (247, 251, 255)),
                crate::render::backend::GradientStop::new(1.0, (8, 48, 107)),
            ],
            center: None,
            missing: None,
            domain: (0.0, 20.0),
            exact: false,
            format: None,
            numeric: crate::decorations::legend::descriptor::domain_presentation(
                (0.0, 20.0),
                None,
                None,
            )
            .unwrap(),
        };
        std::sync::Arc::get_mut(inputs.heatmap.as_mut().unwrap())
            .unwrap()
            .panels[0]
            .1
            .install_shared_domain(&descriptor)
            .unwrap();
        assert_eq!(
            selected_stack(&inputs).unwrap(),
            canonical::named_stack(&["Selected dark contrast"])
        );
    }

    #[test]
    fn estimate_detail_census_uses_complete_role_for_retained_values_and_unavailable_text() {
        let plot = crate::Plot::coefficient(
            crate::CategoryValues::new("term", ["available", "outside", "unavailable"]).unwrap(),
            crate::CoefficientInput::estimates(crate::NumericValues::nullable(
                "estimate",
                [Some(1.01), Some(1.02), None],
            )),
        )
        .estimate_labels(
            crate::NumericFormat::default()
                .precision(crate::NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
                .unwrap()
                .suffix("\u{0301} units")
                .unwrap(),
        )
        .estimate_status([
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::NotEstimable,
        ])
        .x_limits(1.0, 1.015)
        .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &resources,
            Some(&crate::theme::Theme::publication()),
        )
        .unwrap();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.annotations.estimate_detail.stack =
            canonical::named_stack(&["Estimate detail only"]);
        theme.typography.annotations.estimate_detail.font_size = 20;
        theme.typography.annotations.estimate_detail.paint = crate::theme::Color::rgb(5, 7, 9);
        inputs
            .prepare_discrete_guides(&plot, &theme, &resources, &ComponentAddress::root(), false)
            .unwrap();
        let requirements = with_prepared_stage(|stage| {
            foundational_plot_requirements(
                Some(&inputs),
                &plot,
                PlotKind::Coefficient,
                &theme,
                &ComponentAddress::root(),
                true,
                true,
                true,
                Some(stage),
            )
        })
        .unwrap();
        let prepared = &inputs.estimate.as_ref().unwrap().panels[0].1;
        assert_eq!(
            prepared.detail_sources().collect::<Vec<_>>(),
            ["Not estimable", "1.01\u{0301} units", "1.02\u{0301} units"]
        );
        for source in prepared.detail_sources() {
            let selected: Vec<_> = requirements
                .iter()
                .filter(|requirement| requirement.source == source)
                .collect();
            assert_eq!(selected.len(), 1);
            assert_eq!(
                selected[0].stack,
                canonical::named_stack(&["Estimate detail only"])
            );
        }
    }

    #[test]
    fn unavailable_unselected_heatmap_contrast_face_never_enters_font_preparation() {
        for (value, light_selected) in [(0.0, false), (1.0, true)] {
            let plot = crate::Plot::heatmap(
                crate::CategoryValues::new("column", ["a"]).unwrap(),
                crate::CategoryValues::new("row", ["one"]).unwrap(),
                crate::NumericValues::new("fill", [value]),
            )
            .continuous_color_scale(
                crate::ContinuousColorScale::diverging(
                    0.0,
                    None,
                    Some(crate::ColorDomain::exact(-1.0, 1.0).unwrap()),
                )
                .unwrap(),
            )
            .annotations(crate::HeatmapAnnotations::Values(
                crate::NumericFormat::default(),
            ))
            .erase();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            let available = theme.typography.annotations.contrast.light.stack.clone();
            let missing = canonical::named_stack(&["No Such Contrast Face In This Operation"]);
            theme.typography.annotations.contrast.light.stack = if light_selected {
                available.clone()
            } else {
                missing.clone()
            };
            theme.typography.annotations.contrast.dark.stack =
                if light_selected { missing } else { available };
            theme.typography.annotations.contrast.light.font_size =
                if light_selected { 64 } else { 7 };
            theme.typography.annotations.contrast.dark.font_size =
                if light_selected { 7 } else { 64 };
            crate::layout::plan_exact_root(
                800,
                600,
                |stage| {
                    let inputs = crate::plots::shared_x::prepare_plot(
                        &plot,
                        &stage.operation(),
                        Some(&crate::theme::Theme::publication()),
                    )?;
                    inputs.prepare_discrete_guides(
                        &plot,
                        &theme,
                        &stage.operation(),
                        &ComponentAddress::root(),
                        false,
                    )?;
                    let requirements = foundational_plot_requirements(
                        Some(&inputs),
                        &plot,
                        PlotKind::Heatmap,
                        &theme,
                        &ComponentAddress::root(),
                        true,
                        true,
                        true,
                        Some(stage),
                    )?;
                    let mut presence = Vec::new();
                    stage.reserve_prepared(&mut presence, SCALAR_MAP_WORDS, "contrast census")?;
                    presence.resize(SCALAR_MAP_WORDS, 0);
                    let mut census = PreparedTextCensus::materialize(
                        requirements,
                        Some(stage),
                        Some(&mut presence),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                    theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                    let marks = &inputs.heatmap.as_ref().unwrap().panels[0].1;
                    let minimum = crate::plots::heatmap::prepare_intrinsics(
                        marks,
                        &inputs.axes[0].1,
                        &theme,
                        None,
                    )?;
                    assert!(
                        minimum.1 > 60.0 && minimum.0 > 100.0,
                        "selected complete text size owns row height: {minimum:?}"
                    );
                    Ok(())
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            )
            .unwrap();
        }
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
                let stack = canonical::named_stack(&["Test Sans"]);
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

        let stack = canonical::named_stack(&["Test Sans"]);
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
            .plan_text(crate::layout::TextPlanInput {
                source: "A",
                stack: canonical::test_sans_stack(),
                weight: TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: crate::layout::PhysicalTextAlignment::Start,
                paint: Color::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: u32::MAX,
                    owner: "prepared census test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));

        environment.operation().seal_fonts().unwrap();
        let missing = environment
            .plan_text(crate::layout::TextPlanInput {
                source: "B",
                stack: canonical::test_sans_stack(),
                weight: TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: crate::layout::PhysicalTextAlignment::Start,
                paint: Color::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: u32::MAX,
                    owner: "uncensused scalar test".into(),
                },
                exact_override: false,
            })
            .unwrap_err();
        assert!(matches!(
            missing,
            crate::layout::TextPlanError::Font(canonical::FontError::PreparedPairAfterSeal {
                pair_ordinal: 0
            })
        ));
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }
}
