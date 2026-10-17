//! Immutable, validated opaque scale inputs. Backend stops are publication operands.

use std::sync::Arc;

use crate::spec::CategoricalColors;
use crate::{Error, Result};

use super::OpaqueColor;

fn invalid(message: &str) -> Error {
    Error::Data(message.into())
}

/// Checked, fallible construction of immutable scale inputs.
fn retain<T>(values: impl IntoIterator<Item = T>) -> Result<Arc<Vec<T>>> {
    let values = values.into_iter();
    let (minimum, maximum) = values.size_hint();
    let mut retained = Vec::new();
    if maximum == Some(minimum) && minimum > 0 {
        reserve_input(&mut retained, minimum)?;
    }
    for value in values {
        let next = retained
            .len()
            .checked_add(1)
            .ok_or_else(|| invalid("scale input length overflow"))?;

        if next > retained.capacity() {
            let capacity = retained
                .capacity()
                .checked_mul(2)
                .ok_or_else(|| invalid("scale input capacity overflow"))?
                .max(next);
            reserve_input(&mut retained, capacity)?;
        }
        retained.push(value);
    }
    if retained.is_empty() {
        return Err(invalid("scale input must be nonempty"));
    }
    Ok(Arc::new(retained))
}

fn reserve_input<T>(values: &mut Vec<T>, capacity: usize) -> Result<()> {
    values
        .try_reserve_exact(capacity - values.len())
        .map_err(|_| invalid("cannot reserve scale input"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteCapacity {
    Fixed,
    Extendable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QualitativePalette {
    colors: Arc<Vec<OpaqueColor>>,
    capacity: PaletteCapacity,
}

impl QualitativePalette {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "fixed construction is consumed privately before public palette attachment"
        )
    )]
    pub(crate) fn fixed(colors: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            colors: retain(colors)?,
            capacity: PaletteCapacity::Fixed,
        })
    }
    pub(crate) fn extendable(colors: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            colors: retain(colors)?,
            capacity: PaletteCapacity::Extendable,
        })
    }
    pub(crate) fn colors(&self) -> &[OpaqueColor] {
        &self.colors
    }
    pub(crate) fn capacity(&self) -> PaletteCapacity {
        self.capacity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccentPalette {
    seeds: Arc<Vec<OpaqueColor>>,
}

impl AccentPalette {
    pub(crate) fn new(seeds: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            seeds: retain(seeds)?,
        })
    }
    pub(crate) fn seeds(&self) -> &[OpaqueColor] {
        &self.seeds
    }
}

/// The current categorical map name remains CategoricalColors until its public cutover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CategoricalColorScale {
    source: CategoricalSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CategoricalSource {
    Palette(QualitativePalette),
    Map(CategoricalColors),
}

impl CategoricalColorScale {
    pub(crate) fn from_palette(palette: QualitativePalette) -> Self {
        Self {
            source: CategoricalSource::Palette(palette),
        }
    }
    pub(crate) fn from_map(map: CategoricalColors) -> Self {
        Self {
            source: CategoricalSource::Map(map),
        }
    }
    pub(crate) fn palette(&self) -> Option<&QualitativePalette> {
        match &self.source {
            CategoricalSource::Palette(palette) => Some(palette),
            CategoricalSource::Map(_) => None,
        }
    }
    pub(crate) fn map(&self) -> Option<&CategoricalColors> {
        match &self.source {
            CategoricalSource::Map(map) => Some(map),
            CategoricalSource::Palette(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ColorStop {
    offset: f64,
    color: OpaqueColor,
}
impl Eq for ColorStop {}

impl ColorStop {
    pub(crate) fn new(offset: f64, color: OpaqueColor) -> Result<Self> {
        if !offset.is_finite() || !(0.0..=1.0).contains(&offset) {
            return Err(invalid("gradient offset must be finite in 0..=1"));
        }
        Ok(Self {
            offset: zero(offset),
            color,
        })
    }
    const fn builtin(offset: f64, color: OpaqueColor) -> Self {
        assert!(offset.is_finite() && offset >= 0.0 && offset <= 1.0);
        Self {
            offset: if offset == 0.0 { 0.0 } else { offset },
            color,
        }
    }
    pub(crate) fn offset(&self) -> f64 {
        self.offset
    }
    pub(crate) fn color(&self) -> OpaqueColor {
        self.color
    }
    pub(crate) fn backend_stop(&self) -> crate::render::backend::GradientStop {
        crate::render::backend::GradientStop::new(self.offset(), self.color().as_rgb())
    }
}

pub(crate) const HEATMAP_SEQUENTIAL_STOPS: [ColorStop; 2] = [
    ColorStop::builtin(0.0, OpaqueColor::rgb(247, 251, 255)),
    ColorStop::builtin(1.0, OpaqueColor::rgb(8, 48, 107)),
];
pub(crate) const HEATMAP_DIVERGING_STOPS: [ColorStop; 3] = [
    ColorStop::builtin(0.0, OpaqueColor::rgb(49, 130, 189)),
    ColorStop::builtin(0.5, OpaqueColor::rgb(247, 247, 247)),
    ColorStop::builtin(1.0, OpaqueColor::rgb(222, 45, 38)),
];

fn validate_stops(stops: &[ColorStop], diverging: bool) -> Result<()> {
    if stops.len() < if diverging { 3 } else { 2 }
        || stops.first().is_none_or(|s| s.offset.to_bits() != 0)
        || stops
            .last()
            .is_none_or(|s| s.offset.to_bits() != 1.0_f64.to_bits())
        || stops
            .windows(2)
            .any(|pair| pair[0].offset >= pair[1].offset)
        || (diverging
            && !stops
                .iter()
                .any(|s| s.offset.to_bits() == 0.5_f64.to_bits()))
    {
        return Err(invalid("gradient must increase strictly from exactly 0 to 1 with an explicit diverging neutral center stop at 0.5"));
    }
    Ok(())
}

macro_rules! gradient {
    ($name:ident, $diverging:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) struct $name {
            stops: Arc<Vec<ColorStop>>,
        }
        impl $name {
            pub(crate) fn new(stops: impl IntoIterator<Item = ColorStop>) -> Result<Self> {
                let stops = retain(stops)?;
                validate_stops(&stops, $diverging)?;
                Ok(Self { stops })
            }
            pub(crate) fn stops(&self) -> &[ColorStop] {
                &self.stops
            }
        }
    };
}
gradient!(SequentialGradient, false);
gradient!(DivergingGradient, true);

impl SequentialGradient {
    /// Compare complete normalized content; immutable
    /// payload sharing is a constant-work witness for the same exact value.
    pub(crate) fn same_in(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.stops, &other.stops) {
            return true;
        }
        if self.stops.len() != other.stops.len() {
            return false;
        }
        for (left, right) in self.stops.iter().zip(other.stops.iter()) {
            if left != right {
                return false;
            }
        }
        true
    }
}

impl DivergingGradient {
    pub(crate) fn from_sequence(sequence: &SequentialGradient) -> Result<Self> {
        validate_stops(sequence.stops(), true)?;

        Ok(Self {
            stops: Arc::clone(&sequence.stops),
        })
    }
    pub(crate) fn neutral(&self) -> OpaqueColor {
        self.stops()
            .iter()
            .find(|s| s.offset.to_bits() == 0.5_f64.to_bits())
            .expect("validated neutral stop")
            .color()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ColorScales {
    qualitative: QualitativePalette,
    sequential: SequentialGradient,
    diverging: DivergingGradient,
}

impl ColorScales {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "complete scale component construction consumes already validated fields and uniformly returns Result"
    )]
    pub(crate) fn new(
        qualitative: QualitativePalette,
        sequential: SequentialGradient,
        diverging: DivergingGradient,
    ) -> Result<Self> {
        Ok(Self {
            qualitative,
            sequential,
            diverging,
        })
    }
    pub(crate) fn qualitative(&self) -> &QualitativePalette {
        &self.qualitative
    }
    pub(crate) fn sequential(&self) -> &SequentialGradient {
        &self.sequential
    }
    pub(crate) fn diverging(&self) -> &DivergingGradient {
        &self.diverging
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only complete scale edits await complete theme customization"
        )
    )]
    pub(crate) fn with_qualitative(self, value: QualitativePalette) -> Result<Self> {
        Self::new(value, self.sequential, self.diverging)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only complete scale edits await complete theme customization"
        )
    )]
    pub(crate) fn with_sequential(self, value: SequentialGradient) -> Result<Self> {
        Self::new(self.qualitative, value, self.diverging)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only complete scale edits await complete theme customization"
        )
    )]
    pub(crate) fn with_diverging(self, value: DivergingGradient) -> Result<Self> {
        Self::new(self.qualitative, self.sequential, value)
    }
}

#[cfg(test)]
pub(crate) fn test_defaults() -> ColorScales {
    ColorScales::new(
        QualitativePalette::extendable([OpaqueColor::rgb(0, 0, 0)]).unwrap(),
        SequentialGradient::new(HEATMAP_SEQUENTIAL_STOPS).unwrap(),
        DivergingGradient::new(HEATMAP_DIVERGING_STOPS).unwrap(),
    )
    .unwrap()
}

impl AccentPalette {}

fn zero(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ColorDomain {
    minimum: f64,
    maximum: f64,
}
impl Eq for ColorDomain {}

impl ColorDomain {
    pub(crate) fn exact(minimum: f64, maximum: f64) -> Result<Self> {
        if !minimum.is_finite() || !maximum.is_finite() || maximum < minimum {
            return Err(invalid(
                "color domain must be finite and ascending, including a constant domain",
            ));
        }
        Ok(Self {
            minimum: zero(minimum),
            maximum: zero(maximum),
        })
    }
    pub(crate) fn minimum(&self) -> f64 {
        self.minimum
    }
    pub(crate) fn maximum(&self) -> f64 {
        self.maximum
    }
    pub(crate) fn as_pair(&self) -> (f64, f64) {
        (self.minimum(), self.maximum())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContinuousScaleKind {
    Sequential,
    Diverging,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContinuousColorScale {
    selection: ContinuousSelection,
    domain: Option<ColorDomain>,
}
#[derive(Debug, Clone, PartialEq)]
enum ContinuousSelection {
    Sequential(Option<SequentialGradient>),
    Diverging {
        center: f64,
        gradient: Option<DivergingGradient>,
    },
}
impl Eq for ContinuousSelection {}

impl ContinuousColorScale {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the typed sequential selection participates in the uniform fallible scale construction contract"
    )]
    pub(crate) fn sequential(
        gradient: Option<SequentialGradient>,
        domain: Option<ColorDomain>,
    ) -> Result<Self> {
        Ok(Self {
            selection: ContinuousSelection::Sequential(gradient),
            domain,
        })
    }
    pub(crate) fn diverging(
        center: f64,
        gradient: Option<DivergingGradient>,
        domain: Option<ColorDomain>,
    ) -> Result<Self> {
        let value = Self {
            selection: ContinuousSelection::Diverging {
                center: zero(center),
                gradient,
            },
            domain,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        if let Some(center) = self.center() {
            if !center.is_finite()
                || self
                    .domain
                    .as_ref()
                    .is_some_and(|domain| center < domain.minimum || center > domain.maximum)
            {
                return Err(invalid(
                    "diverging center must be finite and the exact domain must contain center",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn kind(&self) -> ContinuousScaleKind {
        match self.selection {
            ContinuousSelection::Sequential(_) => ContinuousScaleKind::Sequential,
            ContinuousSelection::Diverging { .. } => ContinuousScaleKind::Diverging,
        }
    }
    pub(crate) fn center(&self) -> Option<f64> {
        match self.selection {
            ContinuousSelection::Sequential(_) => None,
            ContinuousSelection::Diverging { center, .. } => Some(center),
        }
    }
    pub(crate) fn domain(&self) -> Option<&ColorDomain> {
        self.domain.as_ref()
    }
    pub(crate) fn sequential_gradient(&self) -> Option<&SequentialGradient> {
        match &self.selection {
            ContinuousSelection::Sequential(gradient) => gradient.as_ref(),
            ContinuousSelection::Diverging { .. } => None,
        }
    }
    pub(crate) fn diverging_gradient(&self) -> Option<&DivergingGradient> {
        match &self.selection {
            ContinuousSelection::Diverging { gradient, .. } => gradient.as_ref(),
            ContinuousSelection::Sequential(_) => None,
        }
    }
    pub(crate) fn with_domain(mut self, domain: Option<ColorDomain>) -> Result<Self> {
        self.domain = domain;
        self.validate()?;
        Ok(self)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only center edits await private complete theme/scale customization"
        )
    )]
    pub(crate) fn with_center(mut self, value: f64) -> Result<Self> {
        let ContinuousSelection::Diverging { center, .. } = &mut self.selection else {
            return Err(invalid("sequential scale has no center"));
        };
        *center = zero(value);
        self.validate()?;
        Ok(self)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only gradient edits await complete scale attachment"
        )
    )]
    pub(crate) fn with_sequential_gradient(
        mut self,
        value: Option<SequentialGradient>,
    ) -> Result<Self> {
        let ContinuousSelection::Sequential(gradient) = &mut self.selection else {
            return Err(invalid("diverging scale cannot take a sequential gradient"));
        };
        *gradient = value;
        self.validate()?;
        Ok(self)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only gradient edits await complete scale attachment"
        )
    )]
    pub(crate) fn with_diverging_gradient(
        mut self,
        value: Option<DivergingGradient>,
    ) -> Result<Self> {
        let ContinuousSelection::Diverging { gradient, .. } = &mut self.selection else {
            return Err(invalid("sequential scale cannot take a diverging gradient"));
        };
        *gradient = value;
        self.validate()?;
        Ok(self)
    }
    pub(crate) fn stops(&self) -> Result<&[ColorStop]> {
        match &self.selection {
            ContinuousSelection::Sequential(Some(gradient)) => Ok(gradient.stops()),
            ContinuousSelection::Diverging {
                gradient: Some(gradient),
                ..
            } => Ok(gradient.stops()),
            _ => Err(Error::Render(
                "prepared continuous selection has no retained gradient".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests;
