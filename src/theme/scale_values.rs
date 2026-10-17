//! Immutable, validated opaque scale inputs. Backend stops are publication operands.

use std::sync::Arc;

use crate::spec::CategoricalColorMap;
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

/// Whether categorical seeds are a fixed supply or may be extended deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCapacity {
    /// Reject if the active domain requires more colors than supplied.
    Fixed,
    /// Extend the exact prefix using the first-party deterministic color algorithm.
    Extendable,
}

/// Nonempty opaque categorical colors and their explicit capacity policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualitativePalette {
    colors: Arc<Vec<OpaqueColor>>,
    capacity: PaletteCapacity,
}

impl QualitativePalette {
    /// Construct a fixed nonempty color supply, preserving its exact order.
    pub fn fixed(colors: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            colors: retain(colors)?,
            capacity: PaletteCapacity::Fixed,
        })
    }
    /// Construct nonempty ordered seeds for deterministic extension without cycling.
    pub fn extendable(colors: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            colors: retain(colors)?,
            capacity: PaletteCapacity::Extendable,
        })
    }
    /// Borrow the supplied ordered colors, including any deliberate repeats.
    pub fn colors(&self) -> &[OpaqueColor] {
        &self.colors
    }
    /// Inspect the explicit supply policy.
    pub fn capacity(&self) -> PaletteCapacity {
        self.capacity
    }
}

/// Independent nonempty mark-accent seeds, always deterministically extendable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccentPalette {
    seeds: Arc<Vec<OpaqueColor>>,
}

impl AccentPalette {
    /// Retain nonempty ordered opaque seeds independently of categorical scales.
    pub fn new(seeds: impl IntoIterator<Item = OpaqueColor>) -> Result<Self> {
        Ok(Self {
            seeds: retain(seeds)?,
        })
    }
    /// Borrow the supplied ordered seeds.
    pub fn seeds(&self) -> &[OpaqueColor] {
        &self.seeds
    }
}

/// One complete categorical selection: an ordered palette or an exact raw-key map.
///
/// ```
/// use lichtung_plot::{CategoricalColorMap, CategoricalColorScale, OpaqueColor};
/// let map = CategoricalColorMap::new([
///     ("A  B".into(), OpaqueColor::rgb(17, 53, 89)),
///     ("A B".into(), OpaqueColor::rgb(131, 47, 29)),
/// ])?;
/// let scale = CategoricalColorScale::from_map(map);
/// assert!(scale.palette().is_none());
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Data scale colors do not silently discard element alpha:
/// ```compile_fail
/// use lichtung_plot::{Color, QualitativePalette};
/// let _ = QualitativePalette::fixed([Color::rgba(17, 53, 89, 128)]);
/// ```
/// ```compile_fail
/// use lichtung_plot::CategoricalColors;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoricalColorScale {
    source: CategoricalSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CategoricalSource {
    Palette(QualitativePalette),
    Map(CategoricalColorMap),
}

impl CategoricalColorScale {
    /// Select a complete palette and its capacity policy.
    pub fn from_palette(palette: QualitativePalette) -> Self {
        Self {
            source: CategoricalSource::Palette(palette),
        }
    }
    /// Select an exact raw-key map; every active identity must be present.
    pub fn from_map(map: CategoricalColorMap) -> Self {
        Self {
            source: CategoricalSource::Map(map),
        }
    }
    /// Borrow the palette, or None for a map selection.
    pub fn palette(&self) -> Option<&QualitativePalette> {
        match &self.source {
            CategoricalSource::Palette(palette) => Some(palette),
            CategoricalSource::Map(_) => None,
        }
    }
    /// Borrow the exact map, or None for a palette selection.
    pub fn map(&self) -> Option<&CategoricalColorMap> {
        match &self.source {
            CategoricalSource::Map(map) => Some(map),
            CategoricalSource::Palette(_) => None,
        }
    }
}

/// An exact semantic gradient offset in zero through one and an opaque color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    offset: f64,
    color: OpaqueColor,
}
impl Eq for ColorStop {}

impl ColorStop {
    /// Construct a finite offset in zero through one, normalizing signed zero.
    pub fn new(offset: f64, color: OpaqueColor) -> Result<Self> {
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
    /// Inspect the exact normalized semantic offset.
    pub fn offset(&self) -> f64 {
        self.offset
    }
    /// Inspect the opaque chromatic mapping color.
    pub fn color(&self) -> OpaqueColor {
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
        #[doc = concat!("Complete strictly ordered opaque `", stringify!($name), "` with exact zero/one endpoints.")]
        pub struct $name {
            stops: Arc<Vec<ColorStop>>,
        }
        impl $name {
            /// Retain and validate every ordered semantic stop without sorting or repair.
            /// Diverging input also requires one explicit neutral stop at exactly 0.5.
            pub fn new(stops: impl IntoIterator<Item = ColorStop>) -> Result<Self> {
                let stops = retain(stops)?;
                validate_stops(&stops, $diverging)?;
                Ok(Self { stops })
            }
            /// Borrow the complete ordered semantic stop sequence.
            pub fn stops(&self) -> &[ColorStop] {
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
    /// Inspect the unique explicitly supplied neutral stop color at exactly 0.5.
    pub fn neutral(&self) -> OpaqueColor {
        self.stops()
            .iter()
            .find(|s| s.offset.to_bits() == 0.5_f64.to_bits())
            .expect("validated neutral stop")
            .color()
    }
}

/// Complete default qualitative, sequential and diverging mappings of one theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorScales {
    qualitative: QualitativePalette,
    sequential: SequentialGradient,
    diverging: DivergingGradient,
}

impl ColorScales {
    /// Construct three complete validated default mappings.
    pub fn new(
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
    /// Borrow the default categorical palette, independent of mark accents.
    pub fn qualitative(&self) -> &QualitativePalette {
        &self.qualitative
    }
    /// Borrow the default sequential gradient.
    pub fn sequential(&self) -> &SequentialGradient {
        &self.sequential
    }
    /// Borrow the default diverging gradient.
    pub fn diverging(&self) -> &DivergingGradient {
        &self.diverging
    }
    /// Replace only the complete default categorical palette.
    pub fn with_qualitative(self, value: QualitativePalette) -> Result<Self> {
        Self::new(value, self.sequential, self.diverging)
    }
    /// Replace only the complete default sequential gradient.
    pub fn with_sequential(self, value: SequentialGradient) -> Result<Self> {
        Self::new(self.qualitative, value, self.diverging)
    }
    /// Replace only the complete default diverging gradient.
    pub fn with_diverging(self, value: DivergingGradient) -> Result<Self> {
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

fn zero(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

/// Finite exact color limits, including a constant domain.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorDomain {
    minimum: f64,
    maximum: f64,
}
impl Eq for ColorDomain {}

impl ColorDomain {
    /// Construct finite minimum/maximum with maximum at least minimum; normalize zero.
    pub fn exact(minimum: f64, maximum: f64) -> Result<Self> {
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
    /// Inspect the finite normalized minimum.
    pub fn minimum(&self) -> f64 {
        self.minimum
    }
    /// Inspect the finite normalized maximum.
    pub fn maximum(&self) -> f64 {
        self.maximum
    }
    pub(crate) fn as_pair(&self) -> (f64, f64) {
        (self.minimum(), self.maximum())
    }
}

/// Closed kind of a complete continuous-color selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuousScaleKind {
    /// Sequential chromatic mapping.
    Sequential,
    /// Two semantic branches around an exact neutral center.
    Diverging,
}

/// One complete continuous kind, optional gradient and optional exact domain.
///
/// ```
/// use lichtung_plot::{ColorDomain, ColorStop, ContinuousColorScale,
///     DivergingGradient, OpaqueColor};
/// let gradient = DivergingGradient::new([
///     ColorStop::new(0.0, OpaqueColor::hex("#204080")?)?,
///     ColorStop::new(0.5, OpaqueColor::hex("#EEEEEE")?)?,
///     ColorStop::new(1.0, OpaqueColor::hex("#A04020")?)?,
/// ])?;
/// let scale = ContinuousColorScale::diverging(
///     0.0, Some(gradient), Some(ColorDomain::exact(-1.0, 3.0)?),
/// )?;
/// assert_eq!(scale.center(), Some(0.0));
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// The replaced Heatmap-only selector is removed:
/// ```compile_fail
/// use lichtung_plot::HeatmapScale;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousColorScale {
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
    /// Select sequential mapping; None gradient/domain means owner default/automatic domain.
    pub fn sequential(
        gradient: Option<SequentialGradient>,
        domain: Option<ColorDomain>,
    ) -> Result<Self> {
        Ok(Self {
            selection: ContinuousSelection::Sequential(gradient),
            domain,
        })
    }
    /// Select a finite center and optional complete gradient/domain containing that center.
    pub fn diverging(
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
    /// Inspect the closed mapping kind.
    pub fn kind(&self) -> ContinuousScaleKind {
        match self.selection {
            ContinuousSelection::Sequential(_) => ContinuousScaleKind::Sequential,
            ContinuousSelection::Diverging { .. } => ContinuousScaleKind::Diverging,
        }
    }
    /// Inspect the exact diverging center, or None for sequential mapping.
    pub fn center(&self) -> Option<f64> {
        match self.selection {
            ContinuousSelection::Sequential(_) => None,
            ContinuousSelection::Diverging { center, .. } => Some(center),
        }
    }
    /// Borrow the optional exact domain; None means automatic from active finite values.
    pub fn domain(&self) -> Option<&ColorDomain> {
        self.domain.as_ref()
    }
    /// Borrow a supplied sequential gradient, if this is its selected kind.
    pub fn sequential_gradient(&self) -> Option<&SequentialGradient> {
        match &self.selection {
            ContinuousSelection::Sequential(gradient) => gradient.as_ref(),
            ContinuousSelection::Diverging { .. } => None,
        }
    }
    /// Borrow a supplied diverging gradient, if this is its selected kind.
    pub fn diverging_gradient(&self) -> Option<&DivergingGradient> {
        match &self.selection {
            ContinuousSelection::Diverging { gradient, .. } => gradient.as_ref(),
            ContinuousSelection::Sequential(_) => None,
        }
    }
    /// Replace the optional exact domain, preserving kind and any center containment.
    pub fn with_domain(mut self, domain: Option<ColorDomain>) -> Result<Self> {
        self.domain = domain;
        self.validate()?;
        Ok(self)
    }
    /// Replace a diverging center; reject sequential kind or an excluding exact domain.
    pub fn with_center(mut self, value: f64) -> Result<Self> {
        let ContinuousSelection::Diverging { center, .. } = &mut self.selection else {
            return Err(invalid("sequential scale has no center"));
        };
        *center = zero(value);
        self.validate()?;
        Ok(self)
    }
    /// Replace the optional sequential gradient; reject a diverging selection.
    pub fn with_sequential_gradient(mut self, value: Option<SequentialGradient>) -> Result<Self> {
        let ContinuousSelection::Sequential(gradient) = &mut self.selection else {
            return Err(invalid("diverging scale cannot take a sequential gradient"));
        };
        *gradient = value;
        self.validate()?;
        Ok(self)
    }
    /// Replace the optional diverging gradient; reject a sequential selection.
    pub fn with_diverging_gradient(mut self, value: Option<DivergingGradient>) -> Result<Self> {
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
