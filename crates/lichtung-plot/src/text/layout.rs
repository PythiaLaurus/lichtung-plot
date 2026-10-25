//! Validated text-local constraints, distinct from scientific references.

use crate::{Error, LabelEndpoints, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Finite(u64);
impl Finite {
    pub(crate) const fn constant(value: f64) -> Self {
        Self(value.to_bits())
    }
    pub(crate) fn new(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(Error::Data("text layout coordinates must be finite".into()));
        }
        Ok(Self(if value == 0.0 { 0 } else { value.to_bits() }))
    }
    pub(crate) const fn get(self) -> f64 {
        f64::from_bits(self.0)
    }
}

/// A point of the complete measured block, independent of its line alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextAnchor {
    pub(crate) horizontal: Finite,
    pub(crate) vertical: VerticalAnchor,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalAnchor {
    Fraction(Finite),
    FirstBaseline,
    CenterBaseline,
}
impl TextAnchor {
    /// Arbitrary finite justification in a top-down block frame; no 0..1 clamp.
    pub fn fractions(horizontal: f64, vertical: f64) -> Result<Self> {
        Ok(Self {
            horizontal: Finite::new(horizontal)?,
            vertical: VerticalAnchor::Fraction(Finite::new(vertical)?),
        })
    }
    /// Attach to the actual first baseline at the chosen horizontal fraction.
    pub fn first_baseline(horizontal: f64) -> Result<Self> {
        Ok(Self {
            horizontal: Finite::new(horizontal)?,
            vertical: VerticalAnchor::FirstBaseline,
        })
    }
    /// Attach to the midpoint of the first and last measured baselines.
    pub fn center_baseline(horizontal: f64) -> Result<Self> {
        Ok(Self {
            horizontal: Finite::new(horizontal)?,
            vertical: VerticalAnchor::CenterBaseline,
        })
    }
}
macro_rules! anchors {
    ($($name:ident: ($x:literal,$y:literal)),+ $(,)?) => {impl TextAnchor {$(
        #[doc = concat!("Attach the block's `",stringify!($name),"` point.")]
        pub const fn $name() -> Self {Self {horizontal:Finite(($x as f64).to_bits()),vertical:VerticalAnchor::Fraction(Finite(($y as f64).to_bits()))}}
    )+}};
}
anchors! {top_left:(0.0,0.0),top_center:(0.5,0.0),top_right:(1.0,0.0),center_left:(0.0,0.5),center:(0.5,0.5),center_right:(1.0,0.5),bottom_left:(0.0,1.0),bottom_center:(0.5,1.0),bottom_right:(1.0,1.0)}

/// Coordinate frame for a signed CSS-pixel displacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOffsetFrame {
    /// Physical screen axes, positive y down.
    Screen,
    /// Axes rotating with the measured text block.
    TextLocal,
}
/// One exact represented finite displacement, not a data coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextOffset {
    pub(crate) dx: Finite,
    pub(crate) dy: Finite,
    pub(crate) frame: TextOffsetFrame,
}
impl TextOffset {
    /// Construct finite signed CSS offsets in the selected frame.
    pub fn new(dx: f64, dy: f64, frame: TextOffsetFrame) -> Result<Self> {
        Ok(Self {
            dx: Finite::new(dx)?,
            dy: Finite::new(dy)?,
            frame,
        })
    }
}

/// Explicit text-local finite maxima, independent of the parent's assigned region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextBounds {
    pub(crate) width: Option<Finite>,
    pub(crate) height: Option<Finite>,
    pub(crate) lines: Option<usize>,
}
impl TextBounds {
    /// None leaves a maximum unspecified; zero is a real requested constraint.
    pub fn new(
        max_width: Option<f64>,
        max_height: Option<f64>,
        max_lines: Option<usize>,
    ) -> Result<Self> {
        let bound = |value: f64| {
            if value < 0.0 {
                return Err(Error::Data("text bounds must be nonnegative".into()));
            }
            Finite::new(value)
        };
        Ok(Self {
            width: max_width.map(bound).transpose()?,
            height: max_height.map(bound).transpose()?,
            lines: max_lines,
        })
    }
}
/// Soft-wrap policy; both choices preserve caller-authored mandatory breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrapping {
    /// Use ordinary Unicode legal soft breaks.
    Legal,
    /// Keep each mandatory paragraph intact.
    NoSoftWrap,
}
/// Base paragraph direction, independent of physical line alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    /// Infer each paragraph from its original source.
    #[default]
    Auto,
    /// Use a left-to-right paragraph base.
    LeftToRight,
    /// Use a right-to-left paragraph base.
    RightToLeft,
}
/// Which end of the original source an ellipsis removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextElision {
    /// Remove the logical beginning.
    Start,
    /// Remove an interior span, retaining both ends.
    Middle,
    /// Remove the logical end.
    End,
    /// Remove the physical left end.
    Left,
    /// Remove the physical right end.
    Right,
}
/// An explicit derived-display program; the original source is never replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEllipsis {
    pub(crate) position: TextElision,
    pub(crate) marker: String,
    pub(crate) lines: Option<usize>,
    pub(crate) distinct: Option<bool>,
}
impl TextEllipsis {
    /// Use U+2026 and a one-line cap; distinctness follows the actual owner.
    pub fn new(position: TextElision) -> Self {
        Self {
            position,
            marker: "…".into(),
            lines: Some(1),
            distinct: None,
        }
    }
    /// Replace the marker, including an empty string.
    #[must_use]
    pub fn marker(mut self, text: impl Into<String>) -> Self {
        self.marker = text.into();
        self
    }
    /// Replace the extra ellipsis line cap.
    #[must_use]
    pub const fn max_lines(mut self, value: Option<usize>) -> Self {
        self.lines = value;
        self
    }
    /// Require distinct displayed views in the actual label group.
    #[must_use]
    pub const fn distinct(mut self, value: bool) -> Self {
        self.distinct = Some(value);
        self
    }
}
/// Whether actual label/annotation ink must avoid its peer owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverlap {
    /// Refuse intersecting required text.
    RequireNonOverlap,
    /// Deliberately permit intersecting text.
    Allow,
}
/// Terminal action for an actual text-fit miss, never for font/projection errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFailure {
    /// Refuse the complete operation.
    Error,
    /// Keep source but do not paint this text.
    Hide,
    /// Clip paint to the actual text region.
    Clip,
    /// Permit text overhang subject to the final output boundary.
    Visible,
}
/// Complete-text presentation and its terminal fit action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextOverflow {
    pub(crate) failure: TextFailure,
    pub(crate) ellipsis: Option<TextEllipsis>,
}
impl TextOverflow {
    /// Retain the complete text and refuse a real fit miss.
    pub const fn complete() -> Self {
        Self {
            failure: TextFailure::Error,
            ellipsis: None,
        }
    }
    /// Retain the complete text, clipping its paint to the assigned region.
    pub const fn clip() -> Self {
        Self {
            failure: TextFailure::Clip,
            ellipsis: None,
        }
    }
    /// Retain complete text while allowing local overhang.
    pub const fn visible() -> Self {
        Self {
            failure: TextFailure::Visible,
            ellipsis: None,
        }
    }
    /// Retain complete source and hide text only when it cannot fit.
    pub const fn hide() -> Self {
        Self {
            failure: TextFailure::Hide,
            ellipsis: None,
        }
    }
    /// Try an explicitly abbreviated display before the selected terminal action.
    pub fn ellipsis(value: TextEllipsis, failure: TextFailure) -> Self {
        Self {
            failure,
            ellipsis: Some(value),
        }
    }
}
/// Explicit text eligibility, independent of scientific ticks and marks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextVisibility {
    pub(crate) hidden: bool,
    pub(crate) program: VisibilityProgram,
    pub(crate) endpoints: crate::LabelEndpoints,
    pub(crate) separation: Finite,
    pub(crate) boundary: Option<Finite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisibilityProgram {
    All,
    Every { stride: usize, offset: usize },
    Greedy,
    Parity,
}

impl TextVisibility {
    /// Keep every otherwise eligible text occurrence.
    pub const fn all() -> Self {
        Self {
            hidden: false,
            program: VisibilityProgram::All,
            endpoints: LabelEndpoints::None,
            separation: Finite(0),
            boundary: None,
        }
    }
    /// Suppress this text without deleting its source or scientific geometry.
    pub const fn hidden() -> Self {
        Self {
            hidden: true,
            ..Self::all()
        }
    }

    /// Select original occurrence indices modulo a positive stride, plus protected endpoints.
    pub fn every(stride: usize, offset: usize, endpoints: LabelEndpoints) -> Result<Self> {
        if stride == 0 || offset >= stride {
            return Err(Error::Data(
                "text visibility requires positive stride and offset below stride".into(),
            ));
        }
        Ok(Self {
            program: VisibilityProgram::Every { stride, offset },
            endpoints,
            ..Self::all()
        })
    }

    /// Retain protected labels, then nonintersecting candidates in stable geometric order.
    pub const fn greedy(endpoints: LabelEndpoints) -> Self {
        Self {
            program: VisibilityProgram::Greedy,
            endpoints,
            ..Self::all()
        }
    }

    /// Repeatedly remove alternating unprotected candidates until remaining labels fit.
    pub const fn parity(endpoints: LabelEndpoints) -> Self {
        Self {
            program: VisibilityProgram::Parity,
            endpoints,
            ..Self::all()
        }
    }

    /// Set additional physical clearance for Greedy or Parity reduction only.
    pub fn separation(mut self, value: f64) -> Result<Self> {
        if !matches!(
            self.program,
            VisibilityProgram::Greedy | VisibilityProgram::Parity
        ) || value < 0.0
        {
            return Err(Error::Data(
                "text separation requires nonnegative clearance and an overlap reduction program"
                    .into(),
            ));
        }
        self.separation = Finite::new(value)?;
        Ok(self)
    }

    /// Hide labels outside the actual group boundary, allowing signed CSS overhang.
    pub fn boundary_tolerance(mut self, value: Option<f64>) -> Result<Self> {
        self.boundary = value.map(Finite::new).transpose()?;
        Ok(self)
    }

    pub(crate) fn reduces(&self) -> bool {
        !matches!(self.program, VisibilityProgram::All) || self.boundary.is_some()
    }

    pub(crate) fn protects(&self, index: usize, eligible: &[usize]) -> bool {
        (matches!(self.endpoints, LabelEndpoints::First | LabelEndpoints::Both)
            && eligible.first() == Some(&index))
            || (matches!(self.endpoints, LabelEndpoints::Last | LabelEndpoints::Both)
                && eligible.last() == Some(&index))
    }

    pub(crate) fn includes(&self, index: usize, eligible: &[usize]) -> bool {
        !self.hidden
            && (self.protects(index, eligible)
                || match self.program {
                    VisibilityProgram::Every { stride, offset } => index % stride == offset,
                    _ => true,
                })
    }
}
