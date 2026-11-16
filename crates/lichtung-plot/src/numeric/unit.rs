//! Units and the one authored-or-generated common explanation slot.

use std::sync::Arc;

use super::decimal::{NumericNotation, ResolvedFormat, ResolvedPrecision, TrailingZeros};
use super::emission::append;
use super::format::NumericFormat;
use super::locale::validate_token;
use super::prefixes::{selection::SelectedScale, NumericFactor};
use crate::{Error, Result, Text, TextLayout, TextTheme};

/// Whether unit/factor text repeats or occupies one required owner slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericUnitPlacement {
    /// Repeat the unit with each numeric label.
    PerLabel,
    /// Retain a common factor/unit explanation, without repeating it on labels.
    Shared,
}

/// Literal scientific unit text; it never infers conversion from its spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericUnit {
    text: String,
    separator: String,
    placement: NumericUnitPlacement,
}

impl NumericUnit {
    /// Construct a unit, with one ordinary space as its default separator.
    /// Content is validated when attached to a NumericFormat.
    pub fn new(text: impl Into<String>, placement: NumericUnitPlacement) -> Self {
        Self {
            text: text.into(),
            separator: " ".into(),
            placement,
        }
    }
    /// Replace the literal separator; an empty separator is valid.
    #[must_use]
    pub fn separator(mut self, value: impl Into<String>) -> Self {
        self.separator = value.into();
        self
    }
    /// Literal unit, which may be empty for factor-only explanations.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Literal separator.
    pub fn get_separator(&self) -> &str {
        &self.separator
    }
    /// Repeated or common placement.
    pub const fn placement(&self) -> NumericUnitPlacement {
        self.placement
    }
    pub(super) fn validate(&self) -> Result<()> {
        validate_token(&self.text, true)?;
        validate_token(&self.separator, true)
    }
    pub(super) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            text: crate::data::source::copy_source_string(&self.text)?,
            separator: crate::data::source::copy_source_string(&self.separator)?,
            placement: self.placement,
        })
    }
}

/// Instructions for one common explanation, retaining normal Text replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericExplanation(Arc<Text>);

impl NumericExplanation {
    pub(super) fn replace_callback(&self, current: Option<Text>, incoming: Text) -> Result<Text> {
        let mut slot = self.0.try_copy()?;
        if let Some(current) = current {
            slot.replace_from(current)?;
        }
        slot.replace_from(incoming)?;
        slot.resolve_source(String::new())
    }
    /// Generate content from the selected factor and shared unit.
    pub fn automatic() -> Self {
        Self(Arc::new(Text::UNSET))
    }
    /// Replace generated content with authored Text in the same slot.
    pub fn text(value: impl Into<Text>) -> Self {
        let mut slot = Text::UNSET;
        slot.replace_from(value.into())
            .expect("public Text always has a source");
        Self(Arc::new(slot))
    }
    /// Select the complete local appearance, without changing content.
    #[must_use]
    pub fn style(mut self, value: TextTheme) -> Self {
        Arc::make_mut(&mut self.0).edit_style(value);
        self
    }
    /// Select the local layout, without changing content.
    #[must_use]
    pub fn layout(mut self, value: TextLayout) -> Self {
        Arc::make_mut(&mut self.0).edit_layout(value);
        self
    }
    /// Remove the local appearance while preserving authored/generated content and layout.
    #[must_use]
    pub fn clear_style(mut self) -> Self {
        Arc::make_mut(&mut self.0).edit_clear_style();
        self
    }
    /// Remove the local layout request while preserving content and appearance.
    #[must_use]
    pub fn clear_layout(mut self) -> Self {
        Arc::make_mut(&mut self.0).edit_clear_layout();
        self
    }
    fn resolve(&self, generated: String) -> Result<Option<Text>> {
        let text = self.0.resolve_source(generated)?;
        if !self.0.is_provided() && text.as_str().is_empty() {
            return Ok(None);
        }
        text.required_source("numeric explanation")?;
        Ok(Some(text))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnitPolicy {
    pub(super) unit: Option<NumericUnit>,
    pub(super) explanation: NumericExplanation,
}

impl Default for UnitPolicy {
    fn default() -> Self {
        Self {
            unit: None,
            explanation: NumericExplanation::automatic(),
        }
    }
}

impl NumericFormat {
    pub(super) fn numeric_explanation(&self, scales: &[SelectedScale]) -> Result<Option<Text>> {
        let generated = if let Some(unit) = self
            .units
            .unit
            .as_ref()
            .filter(|unit| unit.placement == NumericUnitPlacement::Shared)
        {
            if scales.windows(2).any(|pair| pair[0] != pair[1]) {
                return Err(Error::Data(
                    "shared numeric unit requires one common factor and affix selection".into(),
                ));
            }
            scales
                .first()
                .map_or_else(|| Ok(String::new()), |scale| common_text(scale, unit))?
        } else {
            String::new()
        };
        self.units.explanation.resolve(generated)
    }
    /// Set or clear literal unit text, validating its single-line tokens.
    pub fn unit(mut self, value: Option<NumericUnit>) -> Result<Self> {
        if let Some(unit) = &value {
            unit.validate()?;
        }
        Arc::make_mut(&mut self.units).unit = value;
        self.validate()?;
        Ok(self)
    }
    /// Replace instructions/content for the one common explanation slot.
    #[must_use]
    pub fn explanation(mut self, value: NumericExplanation) -> Self {
        Arc::make_mut(&mut self.units).explanation = value;
        self
    }

    pub(super) fn decorate_scales(
        &self,
        scales: &[SelectedScale],
    ) -> Result<(Vec<SelectedScale>, Option<Text>)> {
        let Some(unit) = &self.units.unit else {
            return Ok((
                crate::data::source::collect_source(scales.iter().map(SelectedScale::try_copy))?,
                self.units.explanation.resolve(String::new())?,
            ));
        };
        if unit.placement == NumericUnitPlacement::Shared {
            if scales.windows(2).any(|pair| pair[0] != pair[1]) {
                return Err(Error::Data(
                    "shared numeric unit requires one common factor and affix selection".into(),
                ));
            }
            let generated = scales
                .first()
                .map_or_else(|| Ok(String::new()), |scale| common_text(scale, unit))?;
            let bare = crate::data::source::collect_source(scales.iter().map(|scale| {
                Ok(SelectedScale {
                    divisor: scale.divisor.try_copy()?,
                    prefix: String::new(),
                    suffix: String::new(),
                    si: scale.si,
                })
            }))?;
            return Ok((bare, self.units.explanation.resolve(generated)?));
        }
        let decorated = crate::data::source::collect_source(scales.iter().map(|scale| {
            let mut selected = scale.try_copy()?;
            if let Some(si) = scale.si {
                selected.suffix.clear();
                if !unit.text.is_empty() {
                    append(&mut selected.suffix, &unit.separator)?;
                }
                append(&mut selected.suffix, si.symbol())?;
                append(&mut selected.suffix, &unit.text)?;
            } else if !unit.text.is_empty() {
                append(&mut selected.suffix, &unit.separator)?;
                append(&mut selected.suffix, &unit.text)?;
            }
            Ok(selected)
        }))?;
        Ok((decorated, self.units.explanation.resolve(String::new())?))
    }
}

fn common_text(scale: &SelectedScale, unit: &NumericUnit) -> Result<String> {
    let mut text = String::new();
    if let Some(si) = scale.si {
        if !unit.text.is_empty() {
            append(&mut text, si.symbol())?;
            append(&mut text, &unit.text)?;
            return Ok(text);
        }
    }
    if !scale.prefix.is_empty() || (!scale.suffix.is_empty() && scale.si.is_none()) {
        append(&mut text, &scale.prefix)?;
        append(&mut text, &scale.suffix)?;
    } else {
        match &scale.divisor {
            NumericFactor::PowerOfTen(exponent) if exponent != "0" => {
                append(&mut text, "×10")?;
                for scalar in exponent.chars() {
                    append(
                        &mut text,
                        match scalar {
                            '-' => "⁻",
                            '0' => "⁰",
                            '1' => "¹",
                            '2' => "²",
                            '3' => "³",
                            '4' => "⁴",
                            '5' => "⁵",
                            '6' => "⁶",
                            '7' => "⁷",
                            '8' => "⁸",
                            '9' => "⁹",
                            _ => unreachable!("normalized factor exponent"),
                        },
                    )?;
                }
            }
            NumericFactor::Value(value) if value.to_bits() != 1.0_f64.to_bits() => {
                append(&mut text, "×")?;
                let factor = super::decimal::format_finite(
                    *value,
                    &ResolvedFormat {
                        notation: NumericNotation::General,
                        precision: ResolvedPrecision::Significant(17),
                        trailing_zeros: TrailingZeros::Trim,
                        ..ResolvedFormat::default()
                    },
                )?;
                append(&mut text, &factor)?;
            }
            _ => (),
        }
    }
    if !unit.text.is_empty() {
        if !text.is_empty() {
            append(&mut text, &unit.separator)?;
        }
        append(&mut text, &unit.text)?;
    }
    Ok(text)
}

impl UnitPolicy {
    pub(super) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        round.observe_count(usize::from(self.unit.is_some()))?;
        if let Some(unit) = &self.unit {
            round.observe_string(&unit.text)?;
            round.observe_string(&unit.separator)?;
            round.observe_count(unit.placement as usize)?;
        }
        self.explanation.0.observe_feedback(round)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NumericPrecision, NumericPrefixes, NumericPromotion, NumericSelection, SiPrefix};

    #[test]
    fn shared_factors_retain_occurrences_units_and_one_deterministic_explanation() {
        let millions =
            NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
                .unwrap()
                .prefixes(NumericPrefixes::factor(NumericFactor::PowerOfTen("6".into())).unwrap())
                .unwrap()
                .unit(Some(NumericUnit::new("", NumericUnitPlacement::Shared)))
                .unwrap();
        let set = millions
            .format_set(&[1e6, 2e6, 3e6, 1e6], false, true)
            .unwrap()
            .unwrap();
        assert_eq!(set.labels, ["1", "2", "3", "1"]);
        assert_eq!(set.explanation.unwrap().as_str(), "×10⁶");
        assert_eq!(set.scales.len(), set.members.len());
        let whole = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(1))
            .unwrap()
            .prefixes(
                NumericPrefixes::si(NumericSelection::WholeSet, NumericPromotion::KeepThreshold)
                    .unwrap(),
            )
            .unwrap()
            .unit(Some(NumericUnit::new("m", NumericUnitPlacement::Shared)))
            .unwrap();
        let set = whole
            .format_set(&[1250.0, 2500.0], false, true)
            .unwrap()
            .unwrap();
        assert_eq!(set.labels, ["1.2", "2.5"]);
        assert_eq!(set.explanation.unwrap().as_str(), "km");
        let repeated = whole
            .unit(Some(
                NumericUnit::new("m", NumericUnitPlacement::PerLabel).separator("\u{202f}"),
            ))
            .unwrap();
        assert_eq!(
            repeated.format_singleton(1250.0, false).unwrap(),
            "1.2\u{202f}km"
        );
        let mixed = repeated
            .prefixes(
                NumericPrefixes::si(NumericSelection::PerValue, NumericPromotion::KeepThreshold)
                    .unwrap(),
            )
            .unwrap()
            .unit(Some(NumericUnit::new("m", NumericUnitPlacement::Shared)))
            .unwrap();
        assert!(mixed.format_set(&[1.0, 1000.0], false, false).is_err());
        let tiny = millions
            .prefixes(
                NumericPrefixes::si(
                    NumericSelection::Fixed(SiPrefix::from_exponent(-6).unwrap()),
                    NumericPromotion::KeepThreshold,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            tiny.format_set(&[0.0], false, true)
                .unwrap()
                .unwrap()
                .explanation
                .unwrap()
                .as_str(),
            "×10⁻⁶"
        );
    }
}
