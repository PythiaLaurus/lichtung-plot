//! Retained primitive ink and collision geometry. The validating backend is a test oracle only.

use std::collections::HashMap;

use crate::Error;

#[cfg(test)]
use crate::font::canonical::PreparedFontEnvironment;
#[cfg(test)]
use crate::layout::{PhysicalTextAlignment, TextLineHeight, TextPlanInput, TextPlanProvenance};
use crate::layout::{PixelRect, ResolvedTextPlan, CSS_PIXEL_QUANTA};
use crate::render::backend::{
    self, FillStyle, GradientStop, LineStyle, PlannedSignificanceCall, PlannedTextOrigin,
    PlannedTextPlacement, Point2D, PointStyle, TextAnchor, POLYGON_MITER_LIMIT,
};
#[cfg(test)]
use crate::render::backend::{
    Backend, BracketOrientation, BracketStyle, SignificanceBracket, SurfaceRole, TextStyle,
};
#[cfg(test)]
use crate::render::coordinates::CanonicalBracket;
use crate::render::coordinates::CanonicalPoint;
#[cfg(test)]
use crate::theme::Color;
use crate::PointShape;

// Labels and annotation glyphs are less tolerable legend targets than translucent marks.
const ANNOTATION_COLLISION_WEIGHT: f64 = 100.0;

#[cfg(test)]
thread_local! {
    pub(crate) static GEOMETRY_BACKEND_BEGINS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GeometryBox {
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
    pub(crate) top: f64,
    pub(crate) weight: f64,
}

impl GeometryBox {
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "compare the rounded float with the exact wide integer before directed correction"
    )]
    fn from_half_quanta(edges: [i128; 4]) -> Result<Self, Error> {
        let mut values = [0.0; 4];
        for (index, edge) in edges.into_iter().enumerate() {
            let upper = index % 2 == 1;
            let quanta = edge.div_euclid(2) + i128::from(upper && edge.rem_euclid(2) != 0);
            let quanta = i64::try_from(quanta)
                .map_err(|_| Error::Render("painted bound exceeds signed quantum range".into()))?;
            let mut value = quanta as f64;
            if upper && (value as i128) < i128::from(quanta) {
                value = value.next_up();
            } else if !upper && (value as i128) > i128::from(quanta) {
                value = value.next_down();
            }
            if value as i128 > i128::from(i64::MAX) || (value as i128) < i128::from(i64::MIN) {
                return Err(Error::Render(
                    "painted collision bound exceeds signed quantum range".into(),
                ));
            }
            values[index] = value / 1024.0;
        }
        Ok(Self {
            left: values[0],
            right: values[1],
            bottom: values[2],
            top: values[3],
            weight: 1.0,
        })
    }

    fn from_canonical_points(
        points: &[CanonicalPoint],
        padding_half_quanta: i128,
    ) -> Result<Option<Self>, Error> {
        if !(0..=i128::from(u64::MAX)).contains(&padding_half_quanta) {
            return Err(Error::Render(
                "painted padding exceeds signed quantum envelope".into(),
            ));
        }
        let Some(first) = points.first() else {
            return Ok(None);
        };
        let mut edges = [
            first.x.units(),
            first.x.units(),
            first.y.units(),
            first.y.units(),
        ];
        for point in &points[1..] {
            edges[0] = edges[0].min(point.x.units());
            edges[1] = edges[1].max(point.x.units());
            edges[2] = edges[2].min(point.y.units());
            edges[3] = edges[3].max(point.y.units());
        }
        // All operands are signed quanta; i128 preserves odd-width half-quanta.
        Self::from_half_quanta([
            2 * i128::from(edges[0]) - padding_half_quanta,
            2 * i128::from(edges[1]) + padding_half_quanta,
            2 * i128::from(edges[2]) - padding_half_quanta,
            2 * i128::from(edges[3]) + padding_half_quanta,
        ])
        .map(Some)
    }

    fn outward(self) -> Self {
        Self {
            left: (self.left * 1024.0).floor() / 1024.0,
            right: (self.right * 1024.0).ceil() / 1024.0,
            bottom: (self.bottom * 1024.0).floor() / 1024.0,
            top: (self.top * 1024.0).ceil() / 1024.0,
            ..self
        }
    }

    fn from_points(points: &[Point2D], padding: f64) -> Option<Self> {
        let mut left = f64::INFINITY;
        let mut right = f64::NEG_INFINITY;
        let mut bottom = f64::INFINITY;
        let mut top = f64::NEG_INFINITY;
        for point in points {
            left = left.min(point.x);
            right = right.max(point.x);
            bottom = bottom.min(point.y);
            top = top.max(point.y);
        }
        (!points.is_empty()).then_some(Self {
            left: left - padding,
            right: right + padding,
            bottom: bottom - padding,
            top: top + padding,
            weight: 1.0,
        })
    }

    fn clipped(self, clip: Option<PixelRect>) -> Option<Self> {
        let Some(clip) = clip else { return Some(self) };
        let bounds = Self {
            left: self.left.max(clip.x0),
            right: self.right.min(clip.x1),
            bottom: self.bottom.max(clip.y0),
            top: self.top.min(clip.y1),
            weight: self.weight,
        };
        (bounds.left < bounds.right && bounds.bottom < bounds.top).then_some(bounds)
    }

    pub(crate) fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            bottom: self.bottom.min(other.bottom),
            top: self.top.max(other.top),
            weight: self.weight,
        }
    }

    pub(crate) fn overlap_area(self, other: Self, margin: f64) -> f64 {
        let width =
            (self.right.min(other.right + margin) - self.left.max(other.left - margin)).max(0.0);
        let height =
            (self.top.min(other.top + margin) - self.bottom.max(other.bottom - margin)).max(0.0);
        width * height * self.weight
    }
}

#[cfg(test)]
pub(crate) struct GeometryBackend {
    snapshot: GeometrySnapshot,
    font_environment: PreparedFontEnvironment,
    clips: Vec<PixelRect>,
    scopes: Vec<GeometryScope>,
    state: GeometryCanvasState,
}

/// Retained consequences of selected geometry, independent of backend sessions.
#[derive(Debug)]
pub(crate) struct GeometrySnapshot {
    bounds: Vec<GeometryBox>,
    text: Vec<PositionedText>,
}

#[derive(Debug)]
struct PositionedText {
    placement: PlannedTextPlacement,
    plan: ResolvedTextPlan,
    clip: Option<PixelRect>,
    bounds_index: Option<usize>,
}

#[cfg(test)]
impl std::ops::Deref for GeometryBackend {
    type Target = GeometrySnapshot;
    fn deref(&self) -> &Self::Target {
        &self.snapshot
    }
}

#[cfg(test)]
impl std::ops::DerefMut for GeometryBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.snapshot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
enum GeometryScope {
    Layer,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(test)]
enum GeometryCanvasState {
    #[default]
    Fresh,
    Open,
    Closed,
}

#[cfg(test)]
impl GeometryBackend {
    fn with_environment(font_environment: PreparedFontEnvironment) -> Self {
        Self {
            snapshot: GeometrySnapshot {
                bounds: Vec::new(),
                text: Vec::new(),
            },
            font_environment,
            clips: Vec::new(),
            scopes: Vec::new(),
            state: GeometryCanvasState::Fresh,
        }
    }
}

impl GeometrySnapshot {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Vec::new(),
            text: Vec::new(),
        }
    }

    pub(crate) fn record(
        &mut self,
        bounds: Option<GeometryBox>,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        if let Some(bounds) = bounds.and_then(|bounds| bounds.clipped(clip)) {
            backend::reserve_text_records(&mut self.bounds, 1)?;
            self.bounds.push(bounds);
        }
        Ok(())
    }

    pub(crate) fn scatter(
        &mut self,
        points: &[Point2D],
        style: &PointStyle,
        clip: PixelRect,
    ) -> Result<(), Error> {
        self.scatter_with_clip(points, style, Some(clip))
    }

    pub(crate) fn scatter_with_clip(
        &mut self,
        points: &[Point2D],
        style: &PointStyle,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        scatter_bounds(points, style, |bounds| self.record(bounds, clip))
    }

    pub(crate) fn rounded_rect(
        &mut self,
        [x, y, width, height, radius]: [f64; 5],
        style: &FillStyle,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        let rounded = backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
        if rounded.rect.width.value() == 0.0
            || rounded.rect.height.value() == 0.0
            || (style.color.get_alpha() == 0
                && (style.stroke.get_alpha() == 0 || style.stroke_width == 0.0))
        {
            return Ok(());
        }
        let padding = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(if style.stroke.get_alpha() == 0 {
                0.0
            } else {
                style.stroke_width
            }),
            "rectangle stroke width",
        )?
        .units();
        self.record(
            GeometryBox::from_canonical_points(
                &[
                    CanonicalPoint {
                        x: rounded.rect.left,
                        y: rounded.rect.bottom,
                    },
                    CanonicalPoint {
                        x: rounded.rect.right,
                        y: rounded.rect.top,
                    },
                ],
                i128::from(padding),
            )?,
            clip,
        )
    }

    pub(crate) fn line(
        &mut self,
        points: &[Point2D],
        style: &LineStyle,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        line_bounds(points, style, |bounds| self.record(bounds, clip))
    }

    pub(crate) fn text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        let canonical = backend::validate_planned_text(placement, plan)?;
        let placement = PlannedTextPlacement {
            position: canonical.position.value(),
            rotation: canonical.rotation.value(),
            ..placement
        };
        backend::reserve_text_records(&mut self.text, 1)?;
        let before = self.bounds.len();
        self.record(
            planned_text_ink_bounds(
                plan,
                placement.position,
                placement.anchor,
                placement.rotation,
                placement.origin,
                placement.reverse_lines,
            ),
            clip,
        )?;
        self.text.push(PositionedText {
            placement,
            plan: plan.clone(),
            clip,
            bounds_index: (before != self.bounds.len()).then_some(before),
        });
        Ok(())
    }

    pub(crate) fn polygon(
        &mut self,
        points: &[Point2D],
        style: &FillStyle,
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        if points.len() < 3 {
            return Ok(());
        }

        let points = backend::validate_polygon_points(points)?;
        let style = super::primitives::CanonicalFillStyle::new(style)?;
        let ring = polygon_ring(&points);
        let fill = (style.color.get_alpha() > 0 && polygon_has_positive_area(&ring)?)
            .then(|| GeometryBox::from_canonical_points(&ring, 0))
            .transpose()?
            .flatten();
        let stroke = if style.stroke.get_alpha() > 0 {
            polygon_stroke_bounds(&ring, style.stroke_width.units())?
        } else {
            None
        };
        self.record(
            match (fill, stroke) {
                (Some(fill), Some(stroke)) => Some(fill.union(stroke)),
                (fill, stroke) => fill.or(stroke),
            },
            clip,
        )
    }

    pub(crate) fn linear_gradient(
        &mut self,
        bounds: PixelRect,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
        clip: Option<PixelRect>,
    ) -> Result<(), Error> {
        let gradient = backend::validate_linear_gradient(bounds, start, end, stops)?;
        let bounds = gradient.bounds;
        self.record(
            Some(GeometryBox {
                left: bounds.left.value(),
                right: bounds.right.value(),
                bottom: bounds.bottom.value(),
                top: bounds.top.value(),
                weight: 1.0,
            }),
            clip,
        )
    }

    pub(crate) fn bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        let canonical = backend::validate_planned_bracket(call)?;
        let width = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(call.style.width),
            "bracket stroke width",
        )?;
        let path = (width.units() > 0)
            .then(|| {
                GeometryBox::from_canonical_points(
                    &[
                        canonical.line_start,
                        canonical.line_end,
                        canonical.first_tick_end,
                        canonical.second_tick_end,
                    ],
                    i128::from(width.units()),
                )
            })
            .transpose()?
            .flatten()
            .map(|bounds| GeometryBox {
                weight: ANNOTATION_COLLISION_WEIGHT,
                ..bounds
            });
        let placement = backend::planned_bracket_placement(canonical, &call.label.plan)?;
        backend::reserve_text_records(&mut self.text, 1)?;
        self.text.push(PositionedText {
            placement,
            plan: call.label.plan.clone(),
            clip: None,
            bounds_index: None,
        });
        let label = planned_text_ink_bounds(
            &call.label.plan,
            placement.position,
            placement.anchor,
            placement.rotation,
            placement.origin,
            placement.reverse_lines,
        );
        self.record(
            match (path, label) {
                (Some(path), Some(label)) => Some(path.union(label)),
                (path, label) => path.or(label),
            },
            None,
        )
    }

    pub(crate) fn append(&mut self, other: &Self) -> Result<(), Error> {
        backend::reserve_text_records(&mut self.bounds, other.bounds.len())?;
        backend::reserve_text_records(&mut self.text, other.text.len())?;
        let offset = self.bounds.len();
        self.bounds.extend_from_slice(&other.bounds);
        self.text
            .extend(other.text.iter().map(|text| PositionedText {
                placement: text.placement,
                plan: text.plan.clone(),
                clip: text.clip,
                bounds_index: text.bounds_index.map(|index| offset + index),
            }));
        Ok(())
    }

    pub(crate) fn bounds(&self) -> &[GeometryBox] {
        &self.bounds
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.observe_count(self.bounds.len())?;
        for bounds in &self.bounds {
            round.observe_box(PixelRect::new(
                bounds.left,
                bounds.right,
                bounds.bottom,
                bounds.top,
            ))?;
            round.observe_number(bounds.weight)?;
        }
        round.observe_count(self.text.len())?;
        for text in &self.text {
            round.observe_placement(text.placement)?;
            round.observe_text(&text.plan)?;
            round.observe_count(usize::from(text.clip.is_some()))?;
            if let Some(clip) = text.clip {
                round.observe_box(clip)?;
            }
            round.observe_count(usize::from(text.bounds_index.is_some()))?;
            if let Some(index) = text.bounds_index {
                round.observe_count(index)?;
            }
        }
        Ok(())
    }

    pub(crate) fn overlap_after_assignment(
        &self,
        before: PixelRect,
        after: PixelRect,
        legend: GeometryBox,
        margin: f64,
        screen_text: impl Fn(&ResolvedTextPlan) -> Option<PlannedTextPlacement>,
    ) -> Result<f64, Error> {
        let mut text = self
            .text
            .iter()
            .filter(|text| text.bounds_index.is_some())
            .peekable();
        let mut overlap = 0.0;
        for (index, bounds) in self.bounds.iter().enumerate() {
            let placed = if text
                .peek()
                .is_some_and(|text| text.bounds_index == Some(index))
            {
                let text = text.next().expect("matched text bounds");
                screen_text(&text.plan).and_then(|placement| {
                    planned_text_ink_bounds(
                        &text.plan,
                        placement.position,
                        placement.anchor,
                        placement.rotation,
                        placement.origin,
                        placement.reverse_lines,
                    )
                })
            } else {
                None
            };
            let Some(bounds) = bounds.clipped(Some(before)) else {
                continue;
            };
            let predicted = if let Some(placed) = placed {
                placed
            } else {
                let x = |value| after.x0 + (value - before.x0) / before.width() * after.width();
                let y = |value| after.y0 + (value - before.y0) / before.height() * after.height();
                let left = x(bounds.left);
                let right = x(bounds.right);
                let bottom = y(bounds.bottom);
                let top = y(bounds.top);
                // Reserves do not shrink pixel-sized glyphs or stroke footprints.
                let extra_x = ((bounds.right - bounds.left) - (right - left)).max(0.0) / 2.0;
                let extra_y = ((bounds.top - bounds.bottom) - (top - bottom)).max(0.0) / 2.0;
                GeometryBox {
                    left: left - extra_x,
                    right: right + extra_x,
                    bottom: bottom - extra_y,
                    top: top + extra_y,
                    weight: bounds.weight,
                }
            };
            overlap += predicted.overlap_area(legend, margin);
        }
        if overlap.is_finite() {
            Ok(overlap)
        } else {
            Err(Error::Render(
                "assignment collision arithmetic overflowed".into(),
            ))
        }
    }

    /// Test only declared data-anchored annotation text against a pending
    /// projection. Unrelated painted overflow does not inherit its remedy.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the clonable iterator is an allocation-free view over declared annotation owners"
    )]
    pub(crate) fn text_overflow_resolved<'a>(
        &self,
        bounds: GeometryBox,
        root: PixelRect,
        current: &crate::layout::Projection,
        next: &crate::layout::Projection,
        declared: impl Iterator<Item = &'a ResolvedTextPlan> + Clone,
    ) -> Result<bool, Error> {
        for text in &self.text {
            let mut found = false;
            for plan in declared.clone() {
                if plan == &text.plan {
                    found = true;
                    break;
                }
            }
            if !found {
                continue;
            }
            let Some(ink) = planned_text_ink_bounds(
                &text.plan,
                text.placement.position,
                text.placement.anchor,
                text.placement.rotation,
                text.placement.origin,
                text.placement.reverse_lines,
            ) else {
                continue;
            };
            let same_overflow = (bounds.left >= root.x0
                || ink.left.to_bits() == bounds.left.to_bits())
                && (bounds.right <= root.x1 || ink.right.to_bits() == bounds.right.to_bits())
                && (bounds.bottom >= root.y0 || ink.bottom.to_bits() == bounds.bottom.to_bits())
                && (bounds.top <= root.y1 || ink.top.to_bits() == bounds.top.to_bits());
            if !same_overflow {
                continue;
            }
            let before = current.panel_box();
            let after = next.panel_box();
            let position = Point2D::new(
                feedback_coordinate(
                    text.placement.position.x,
                    (before.x0, before.x1),
                    (after.x0, after.x1),
                    current.x_range(),
                    next.x_range(),
                )?,
                feedback_coordinate(
                    text.placement.position.y,
                    (before.y0, before.y1),
                    (after.y0, after.y1),
                    current.y_range(),
                    next.y_range(),
                )?,
            );
            let placement = PlannedTextPlacement {
                // This predictor is limited to deferred data-anchored annotations.
                position: Point2D::new(position.x.round_ties_even(), position.y.round_ties_even()),
                ..text.placement
            };
            let canonical = backend::validate_planned_text(placement, &text.plan)?;
            if let Some(ink) = planned_text_ink_bounds(
                &text.plan,
                canonical.position.value(),
                placement.anchor,
                canonical.rotation.value(),
                placement.origin,
                placement.reverse_lines,
            ) {
                if ink.left >= root.x0
                    && ink.right <= root.x1
                    && ink.bottom >= root.y0
                    && ink.top <= root.y1
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

/// Exact painted extent of one origin-relative point. The same canonical
/// geometry path measures pre-selection point requirements and selected ink.
pub(crate) fn point_ink_dimensions(style: &PointStyle) -> Result<(f64, f64), Error> {
    Ok(point_ink_bounds(style)?.map_or((0.0, 0.0), |bounds| {
        (bounds.right - bounds.left, bounds.top - bounds.bottom)
    }))
}

pub(crate) fn point_ink_bounds(style: &PointStyle) -> Result<Option<GeometryBox>, Error> {
    let mut ink = GeometrySnapshot::new();
    ink.scatter_with_clip(&[Point2D::new(0.0, 0.0)], style, None)?;
    Ok(ink.bounds().first().copied())
}

/// Painted thickness of an origin-relative one-pixel path, independent of the
/// prepared scientific length and orientation.
pub(crate) fn line_ink_width(style: &LineStyle) -> Result<f64, Error> {
    Ok(line_ink_bounds(style)?.map_or(0.0, |bounds| bounds.top - bounds.bottom))
}

/// Relative painted extents of an axis-aligned path. The synthetic one-pixel
/// segment itself is removed from the right end; both caps stay in the ink.
pub(crate) fn line_ink_bounds(style: &LineStyle) -> Result<Option<GeometryBox>, Error> {
    let mut ink = GeometrySnapshot::new();
    ink.line(
        &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)],
        style,
        None,
    )?;
    Ok(ink.bounds().first().copied().map(|mut bounds| {
        bounds.right -= 1.0;
        bounds
    }))
}

/// Outward painted size beyond a unit, axis-aligned bin body. This is the
/// histogram's actual rectangular polygon grammar, including mitered stroke.
pub(crate) fn rectangular_fill_ink_outset(style: &FillStyle) -> Result<(f64, f64), Error> {
    Ok(
        rectangular_fill_ink_bounds(style)?.map_or((0.0, 0.0), |bounds| {
            (
                (bounds.right - bounds.left).max(0.0),
                (bounds.top - bounds.bottom).max(0.0),
            )
        }),
    )
}

pub(crate) fn rectangular_fill_ink_bounds(style: &FillStyle) -> Result<Option<GeometryBox>, Error> {
    let mut ink = GeometrySnapshot::new();
    ink.polygon(
        &[
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ],
        style,
        None,
    )?;
    Ok(ink.bounds().first().copied().map(|mut bounds| {
        bounds.right -= 1.0;
        bounds.top -= 1.0;
        bounds
    }))
}

#[cfg(test)]
impl GeometryBackend {
    fn record_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        backend::reserve_text_records(&mut self.snapshot.text, 1)?;

        let clip = self.clips.iter().copied().reduce(|a, b| {
            let left = a.x0.max(b.x0);
            let bottom = a.y0.max(b.y0);
            PixelRect::new(
                left,
                a.x1.min(b.x1).max(left),
                bottom,
                a.y1.min(b.y1).max(bottom),
            )
        });
        self.text.push(PositionedText {
            placement,
            plan: plan.clone(),
            clip,
            bounds_index: None,
        });
        Ok(())
    }

    fn plan_raw_text(
        &self,
        text: &str,
        style: &TextStyle,
        alignment: PhysicalTextAlignment,
        owner: &str,
    ) -> Result<ResolvedTextPlan, Error> {
        self.font_environment
            .plan_text(TextPlanInput {
                size_trace: None,
                direction: crate::TextDirection::Auto,
                source: text,
                stack: &style.font_family,
                weight: style.weight,
                font_size: style.font_size,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment,
                paint: style.color,
                provenance: TextPlanProvenance {
                    pair_ordinal: match style.weight {
                        crate::TextWeight::Normal => 0,
                        crate::TextWeight::Semibold => 1,
                        crate::TextWeight::Bold => 2,
                    },
                    owner: owner.to_owned(),
                },
                exact_override: false,
            })
            .map_err(|error| Error::Render(format!("{owner} text planning: {error}")))
    }

    fn ensure_open(&self, operation: &str) -> Result<(), Error> {
        if self.state == GeometryCanvasState::Open {
            Ok(())
        } else {
            Err(Error::Render(format!(
                "geometry backend {operation} called outside begin/end"
            )))
        }
    }

    fn record(&mut self, bounds: Option<GeometryBox>) {
        let bounds = self.clips.iter().fold(bounds, |bounds, clip| {
            bounds.and_then(|bounds| bounds.clipped(Some(*clip)))
        });
        if let Some(bounds) = bounds {
            self.bounds.push(bounds);
        }
    }

    fn record_planned_bracket(
        &mut self,
        canonical: CanonicalBracket,
        style: &BracketStyle,
        label: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        let width = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(style.width),
            "bracket stroke width",
        )?;
        let path = (width.units() > 0)
            .then(|| {
                GeometryBox::from_canonical_points(
                    &[
                        canonical.line_start,
                        canonical.line_end,
                        canonical.first_tick_end,
                        canonical.second_tick_end,
                    ],
                    i128::from(width.units()),
                )
            })
            .transpose()?
            .flatten()
            .map(|bounds| GeometryBox {
                weight: ANNOTATION_COLLISION_WEIGHT,
                ..bounds
            });
        let placement = backend::planned_bracket_placement(canonical, label)?;
        self.record_text(placement, label)?;
        let label = planned_text_ink_bounds(
            label,
            placement.position,
            placement.anchor,
            placement.rotation,
            placement.origin,
            placement.reverse_lines,
        );
        self.record(match (path, label) {
            (Some(path), Some(label)) => Some(path.union(label)),
            (path, label) => path.or(label),
        });
        Ok(())
    }
}

fn feedback_coordinate(
    value: f64,
    before: (f64, f64),
    after: (f64, f64),
    domain: (f64, f64),
    next: (f64, f64),
) -> Result<f64, Error> {
    let fraction = (value - before.0) / (before.1 - before.0);
    let next_fraction = if domain == next {
        fraction
    } else {
        let scale = domain
            .0
            .abs()
            .max(domain.1.abs())
            .max(next.0.abs())
            .max(next.1.abs())
            .max(f64::MIN_POSITIVE);
        let data = domain.0 / scale + fraction * (domain.1 / scale - domain.0 / scale);
        (data - next.0 / scale) / (next.1 / scale - next.0 / scale)
    };
    let result = after.0 + next_fraction * (after.1 - after.0);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(Error::Render(
            "annotation feedback projection arithmetic overflowed".into(),
        ))
    }
}

#[cfg(test)]
fn physical_alignment(anchor: TextAnchor) -> PhysicalTextAlignment {
    match anchor {
        TextAnchor::Start => PhysicalTextAlignment::Start,
        TextAnchor::Middle => PhysicalTextAlignment::Center,
        TextAnchor::End => PhysicalTextAlignment::End,
    }
}

fn scatter_bounds(
    points: &[Point2D],
    style: &PointStyle,

    mut record: impl FnMut(Option<GeometryBox>) -> Result<(), Error>,
) -> Result<(), Error> {
    let canonical_points = backend::validate_points(points, "scatter")?;
    backend::validate_point_style(style)?;
    let outline_width = style
        .outline
        .as_ref()
        .filter(|outline| outline.color.get_alpha() > 0)
        .map(|outline| {
            outline
                .width_for_radius(style.radius)
                .map(super::coordinates::CanonicalCoordinate::units)
        })
        .transpose()?
        .unwrap_or(0);
    match style.shape {
        PointShape::Circle => {
            let radius = super::coordinates::CanonicalCoordinate::nonnegative_length(
                f64::from(style.radius),
                "scatter radius",
            )?
            .units();
            for point in canonical_points {
                if style.color.get_alpha() == 0 && outline_width == 0 {
                    continue;
                }
                record(GeometryBox::from_canonical_points(
                    std::slice::from_ref(&point),
                    2 * i128::from(radius) + i128::from(outline_width),
                )?)?;
            }
        }
        PointShape::TriangleUp => {
            for triangle in backend::validate_triangle_points(points, style.radius)? {
                let fill = (style.color.get_alpha() > 0)
                    .then(|| GeometryBox::from_canonical_points(&triangle, 0))
                    .transpose()?
                    .flatten();
                let stroke = polygon_stroke_bounds(&triangle, outline_width)?;
                record(match (fill, stroke) {
                    (Some(fill), Some(stroke)) => Some(fill.union(stroke)),
                    (fill, stroke) => fill.or(stroke),
                })?;
            }
        }
    }
    Ok(())
}

fn line_bounds(
    points: &[Point2D],
    style: &LineStyle,
    mut record: impl FnMut(Option<GeometryBox>) -> Result<(), Error>,
) -> Result<(), Error> {
    if points.len() < 2 {
        return Ok(());
    }
    let points = backend::validate_line_points(points)?;
    let style = super::primitives::CanonicalLineStyle::new(style)?;
    if style.color.get_alpha() == 0 || style.width.units() == 0 {
        return Ok(());
    }
    for segment in points.windows(2) {
        record(GeometryBox::from_canonical_points(
            segment,
            i128::from(style.width.units()),
        )?)?;
    }
    Ok(())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "planned text retains exact integer quanta until collision conversion"
)]
pub(crate) fn planned_text_ink_bounds(
    plan: &ResolvedTextPlan,
    position: Point2D,
    anchor: TextAnchor,
    rotation: f64,
    vertical_origin: PlannedTextOrigin,
    reverse_lines: bool,
) -> Option<GeometryBox> {
    let quanta = CSS_PIXEL_QUANTA as f64;
    let width = plan.width_quanta as f64 / quanta;
    let anchor_offset = match anchor {
        TextAnchor::Start => 0.0,
        TextAnchor::Middle => -backend::nearest_even_text_half(plan.width_quanta)? as f64 / quanta,
        TextAnchor::End => -width,
    };
    let vertical_reference = match vertical_origin {
        PlannedTextOrigin::Middle => {
            backend::nearest_even_text_half(plan.height_quanta)? as f64 / quanta
        }
        PlannedTextOrigin::Baseline => plan.first_baseline_quanta.unwrap_or(0) as f64 / quanta,
    };
    let (sine, cosine) = match rotation {
        0.0 => (0.0, 1.0),
        -90.0 => (-1.0, 0.0),
        90.0 => (1.0, 0.0),
        _ => rotation.to_radians().sin_cos(),
    };
    let mut bounds: Option<GeometryBox> = None;

    for line in &plan.lines {
        let Some(ink) = line.ink_quanta else {
            continue;
        };
        let line_x = line.translation_x_quanta as f64 / quanta;
        let left = anchor_offset + line_x + ink.left as f64 / quanta;
        let right = anchor_offset + line_x + ink.right as f64 / quanta;
        let placement = PlannedTextPlacement {
            position,
            anchor,
            rotation,
            origin: vertical_origin,
            reverse_lines,
        };
        let baseline = vertical_reference
            - placement.line_y(plan, line.translation_y_quanta).ok()? as f64 / quanta;
        let bottom = baseline + ink.bottom as f64 / quanta;
        let top = baseline + ink.top as f64 / quanta;
        let corners = [(left, bottom), (left, top), (right, bottom), (right, top)].map(|(x, y)| {
            Point2D::new(
                position.x + x * cosine + y * sine,
                position.y - x * sine + y * cosine,
            )
        });
        let line_bounds = GeometryBox::from_points(&corners, 0.0).map(|bounds| GeometryBox {
            weight: ANNOTATION_COLLISION_WEIGHT,
            ..bounds
        });
        bounds = match (bounds, line_bounds) {
            (Some(current), Some(line)) => Some(current.union(line)),
            (current, line) => current.or(line),
        };
    }
    let clip = crate::layout::text::placement::paint_clip(
        PlannedTextPlacement {
            position,
            anchor,
            rotation,
            origin: vertical_origin,
            reverse_lines,
        },
        plan,
    )
    .ok()?;
    bounds
        .map(|bounds| {
            let mut bounds = GeometryBox::outward(bounds);
            if plan.request.as_ref().is_some_and(|request| {
                request.instructions().overlap == Some(crate::TextOverlap::Allow)
            }) {
                bounds.weight = 0.0;
            }
            bounds
        })
        .and_then(|bounds| bounds.clipped(clip))
}

fn polygon_ring(points: &[CanonicalPoint]) -> Vec<CanonicalPoint> {
    let mut ring = Vec::with_capacity(points.len());
    for point in points {
        if ring.last() != Some(point) {
            ring.push(*point);
        }
    }
    if ring.len() > 1 && ring.first() == ring.last() {
        ring.pop();
    }
    ring
}

fn polygon_has_positive_area(points: &[CanonicalPoint]) -> Result<bool, Error> {
    if points.len() < 3 {
        return Ok(false);
    }
    // A nonzero-fill path is empty only when its directed edges cancel exactly.
    // Canonical integer line intervals avoid floating cancellation at retraced bands.
    let mut line_events = HashMap::<(i128, i128, i128), Vec<(i128, i32)>>::new();
    for (start, end) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let dx = i128::from(end.x.units()) - i128::from(start.x.units());
        let dy = i128::from(end.y.units()) - i128::from(start.y.units());
        if dx == 0 && dy == 0 {
            continue;
        }
        let divisor = i128::from(greatest_common_divisor(
            u64::try_from(dx.unsigned_abs()).expect("i64 endpoint difference fits u64"),
            u64::try_from(dy.unsigned_abs()).expect("i64 endpoint difference fits u64"),
        ));
        let raw_x = dx / divisor;
        let raw_y = dy / divisor;
        let forward = raw_x > 0 || (raw_x == 0 && raw_y > 0);
        let (direction_x, direction_y, direction) = if forward {
            (raw_x, raw_y, 1_i32)
        } else {
            (-raw_x, -raw_y, -1_i32)
        };
        let overflow = || Error::Render("polygon canonical area arithmetic overflowed".into());
        let checked_product = |direction: i128, coordinate: i64| {
            direction
                .checked_mul(i128::from(coordinate))
                .ok_or_else(overflow)
        };
        let line = checked_product(direction_y, start.x.units())?
            .checked_sub(checked_product(direction_x, start.y.units())?)
            .ok_or_else(overflow)?;
        let start_projection = checked_product(direction_x, start.x.units())?
            .checked_add(checked_product(direction_y, start.y.units())?)
            .ok_or_else(overflow)?;
        let end_projection = checked_product(direction_x, end.x.units())?
            .checked_add(checked_product(direction_y, end.y.units())?)
            .ok_or_else(overflow)?;
        let (low, high) = if start_projection < end_projection {
            (start_projection, end_projection)
        } else {
            (end_projection, start_projection)
        };
        let events = line_events
            .entry((direction_x, direction_y, line))
            .or_default();
        events.push((low, direction));
        events.push((high, -direction));
    }

    Ok(line_events.into_values().any(|mut events| {
        events.sort_unstable_by_key(|event| event.0);
        let mut coverage = 0_i32;
        let mut index = 0;
        while index < events.len() {
            let projection = events[index].0;
            while index < events.len() && events[index].0 == projection {
                coverage += events[index].1;
                index += 1;
            }
            if coverage != 0 && index < events.len() && projection < events[index].0 {
                return true;
            }
        }
        false
    }))
}

fn greatest_common_divisor(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

// Polygon offsets are evaluated locally with directed interval arithmetic. Never
// add a floating normal or miter to an absolute (possibly >2^53-quanta) vertex.
#[derive(Clone, Copy)]
struct StrokeInterval {
    low: f64,
    high: f64,
}

impl StrokeInterval {
    const ZERO: Self = Self {
        low: 0.0,
        high: 0.0,
    };
    const ONE: Self = Self {
        low: 1.0,
        high: 1.0,
    };

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "directed enclosure of bounded canonical integer differences and widths"
    )]
    fn integer(value: i128) -> Self {
        let rounded = value as f64;
        Self {
            low: if rounded as i128 > value {
                rounded.next_down()
            } else {
                rounded
            },
            high: if (rounded as i128) < value {
                rounded.next_up()
            } else {
                rounded
            },
        }
    }

    fn neg(self) -> Self {
        Self {
            low: -self.high,
            high: -self.low,
        }
    }

    fn add(self, other: Self) -> Self {
        if other.low == 0.0 && other.high == 0.0 {
            return self;
        }
        if self.low == 0.0 && self.high == 0.0 {
            return other;
        }
        Self {
            low: (self.low + other.low).next_down(),
            high: (self.high + other.high).next_up(),
        }
    }

    #[expect(
        clippy::float_cmp,
        reason = "only exact singleton identities preserve interval endpoints"
    )]
    fn mul(self, other: Self) -> Self {
        if (self.low == 0.0 && self.high == 0.0) || (other.low == 0.0 && other.high == 0.0) {
            return Self::ZERO;
        }
        if self.low == 1.0 && self.high == 1.0 {
            return other;
        }
        if other.low == 1.0 && other.high == 1.0 {
            return self;
        }
        if self.low == -1.0 && self.high == -1.0 {
            return other.neg();
        }
        if other.low == -1.0 && other.high == -1.0 {
            return self.neg();
        }
        let products = [
            self.low * other.low,
            self.low * other.high,
            self.high * other.low,
            self.high * other.high,
        ];
        Self {
            low: products
                .into_iter()
                .fold(f64::INFINITY, f64::min)
                .next_down(),
            high: products
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max)
                .next_up(),
        }
    }

    #[expect(
        clippy::float_cmp,
        reason = "only an exact singleton identity preserves interval endpoints"
    )]
    fn div_positive(self, other: Self) -> Self {
        debug_assert!(other.low > 0.0);
        if other.low == 1.0 && other.high == 1.0 {
            return self;
        }
        self.mul(Self {
            low: (1.0 / other.high).next_down(),
            high: (1.0 / other.low).next_up(),
        })
    }

    fn sqrt(self) -> Self {
        Self {
            low: self.low.sqrt().next_down(),
            high: self.high.sqrt().next_up(),
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::float_cmp,
        reason = "finite local offsets are range checked before wide-integer conversion"
    )]
    fn translated(self, anchor: i64, width: i64) -> Result<[i128; 2], Error> {
        // Axis normals and right-angle miters use exact integer half-quanta,
        // including odd widths. No floating width or absolute vertex is needed.
        if self.low == self.high && [0.0, 1.0, -1.0].contains(&self.low) {
            let edge = 2 * i128::from(anchor) + self.low as i128 * i128::from(width);
            return Ok([edge; 2]);
        }
        let offset = self.mul(Self::integer(i128::from(width)));
        // Other offsets are half-quanta; round locally, then add the exact vertex.
        let low = (offset.low / 2.0).floor();
        let high = (offset.high / 2.0).ceil();
        let limit = 2.0_f64.powi(126);
        if !low.is_finite() || !high.is_finite() || low <= -limit || high >= limit {
            return Err(Error::Render("painted polygon offset overflowed".into()));
        }
        Ok([
            2 * (i128::from(anchor) + low as i128),
            2 * (i128::from(anchor) + high as i128),
        ])
    }
}

fn polygon_stroke_bounds(
    points: &[CanonicalPoint],
    stroke_width: i64,
) -> Result<Option<GeometryBox>, Error> {
    if stroke_width == 0 || points.len() < 2 {
        return Ok(None);
    }
    let mut bounds: Option<GeometryBox> = None;
    let mut include = |point: CanonicalPoint, offset: [StrokeInterval; 2]| -> Result<(), Error> {
        let [left, right] = offset[0].translated(point.x.units(), stroke_width)?;
        let [bottom, top] = offset[1].translated(point.y.units(), stroke_width)?;
        let next = GeometryBox::from_half_quanta([left, right, bottom, top])?;
        bounds = Some(bounds.map_or(next, |bounds| bounds.union(next)));
        Ok(())
    };
    let mut directions = Vec::with_capacity(points.len());
    for (start, end) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let dx = i128::from(end.x.units()) - i128::from(start.x.units());
        let dy = i128::from(end.y.units()) - i128::from(start.y.units());
        if dx == 0 && dy == 0 {
            directions.push(None);
            continue;
        }
        let direction = if dx == 0 || dy == 0 {
            [
                StrokeInterval::integer(dx.signum()),
                StrokeInterval::integer(dy.signum()),
            ]
        } else {
            let x = StrokeInterval::integer(dx);
            let y = StrokeInterval::integer(dy);
            let length = x.mul(x).add(y.mul(y)).sqrt();
            [x.div_positive(length), y.div_positive(length)]
        };
        let normal = [direction[1].neg(), direction[0]];
        for point in [start, end] {
            include(point, normal)?;
            include(point, normal.map(StrokeInterval::neg))?;
        }
        directions.push(Some(direction));
    }

    for index in 0..points.len() {
        let Some(incoming) = directions[(index + points.len() - 1) % points.len()] else {
            continue;
        };
        let Some(outgoing) = directions[index] else {
            continue;
        };
        let turn = incoming[0]
            .mul(outgoing[1])
            .add(incoming[1].mul(outgoing[0]).neg());
        let denominator = StrokeInterval::ONE.add(
            incoming[0]
                .mul(outgoing[0])
                .add(incoming[1].mul(outgoing[1])),
        );
        // miter_ratio^2 = 2 / (1 + dot). An ambiguous limit comparison
        // includes the miter; it must never discard potentially painted ink.
        let minimum_denominator = 2.0 / (POLYGON_MITER_LIMIT * POLYGON_MITER_LIMIT);
        if (turn.low == 0.0 && turn.high == 0.0) || denominator.high < minimum_denominator {
            continue;
        }
        if denominator.low <= 0.0 {
            return Err(Error::Render(
                "painted polygon miter denominator is not positive".into(),
            ));
        }
        let offset = [
            incoming[1].add(outgoing[1]).neg(),
            incoming[0].add(outgoing[0]),
        ]
        .map(|value| value.div_positive(denominator));
        if turn.low <= 0.0 {
            include(points[index], offset)?;
        }
        if turn.high >= 0.0 {
            include(points[index], offset.map(StrokeInterval::neg))?;
        }
    }

    Ok(bounds)
}

#[cfg(test)]
impl Backend for GeometryBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        #[cfg(test)]
        GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(calls.get() + 1));
        if self.state == GeometryCanvasState::Open {
            return Err(Error::Render(
                "geometry backend begin called while a canvas is already open".into(),
            ));
        }
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "geometry backend canvas dimensions must be positive".into(),
            ));
        }
        self.bounds.clear();
        self.text.clear();
        self.clips.clear();
        self.scopes.clear();
        self.state = GeometryCanvasState::Open;
        Ok(())
    }

    fn begin_layer(&mut self, _name: &str) -> Result<(), Error> {
        self.ensure_open("begin_layer")?;
        self.scopes.push(GeometryScope::Layer);
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.ensure_open("end_layer")?;
        match self.scopes.last() {
            Some(GeometryScope::Layer) => {
                self.scopes.pop();
                Ok(())
            }
            Some(GeometryScope::Clip) => Err(Error::Render(
                "geometry backend end_layer called while a clip scope is active".into(),
            )),
            None => Err(Error::Render(
                "geometry backend end_layer called without a matching begin_layer".into(),
            )),
        }
    }

    fn begin_clip(&mut self, region: PixelRect) -> Result<(), Error> {
        self.ensure_open("begin_clip")?;
        let region = backend::validate_clip_region(region)?;
        self.clips.push(region.panel());
        self.scopes.push(GeometryScope::Clip);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.ensure_open("end_clip")?;
        match self.scopes.last() {
            Some(GeometryScope::Clip) => {
                self.scopes.pop();
                self.clips.pop();
                Ok(())
            }
            Some(GeometryScope::Layer) => Err(Error::Render(
                "geometry backend end_clip called while a layer scope is active".into(),
            )),
            None => Err(Error::Render(
                "geometry backend end_clip called without a matching begin_clip".into(),
            )),
        }
    }

    fn draw_surface(
        &mut self,
        bounds: PixelRect,
        _paint: Color,
        _role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_open("draw_surface")?;
        backend::validate_surface_bounds(bounds)?;
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.ensure_open("draw_scatter")?;
        scatter_bounds(points, style, |bounds| {
            self.record(bounds);
            Ok(())
        })
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_open("draw_line")?;
        line_bounds(points, style, |bounds| {
            self.record(bounds);
            Ok(())
        })
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_open("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        let points = backend::validate_polygon_points(points)?;
        let style = super::primitives::CanonicalFillStyle::new(style)?;
        let ring = polygon_ring(&points);
        let fill_bounds = (style.color.get_alpha() > 0 && polygon_has_positive_area(&ring)?)
            .then(|| GeometryBox::from_canonical_points(&ring, 0))
            .transpose()?
            .flatten();
        let stroke_bounds = if style.stroke.get_alpha() > 0 {
            polygon_stroke_bounds(&ring, style.stroke_width.units())?
        } else {
            None
        };
        self.record(match (fill_bounds, stroke_bounds) {
            (Some(fill), Some(stroke)) => Some(fill.union(stroke)),
            (fill, stroke) => fill.or(stroke),
        });
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PixelRect,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.ensure_open("draw_linear_gradient")?;
        let gradient = backend::validate_linear_gradient(bounds, start, end, stops)?;
        let bounds = gradient.bounds;
        self.record(Some(GeometryBox {
            left: bounds.left.value(),
            right: bounds.right.value(),
            bottom: bounds.bottom.value(),
            top: bounds.top.value(),
            weight: 1.0,
        }));
        Ok(())
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.ensure_open("draw_text")?;
        let canonical = backend::validate_text(position, style)?;
        let plan = self.plan_raw_text(
            text,
            style,
            physical_alignment(style.anchor),
            "geometry raw text",
        )?;
        self.record_text(
            PlannedTextPlacement::middle(
                canonical.position.value(),
                style.anchor,
                canonical.rotation.value(),
            ),
            &plan,
        )?;
        let index = self.bounds.len();
        self.record(planned_text_ink_bounds(
            &plan,
            canonical.position.value(),
            style.anchor,
            canonical.rotation.value(),
            PlannedTextOrigin::Middle,
            false,
        ));
        if self.bounds.len() > index {
            self.text.last_mut().expect("recorded text").bounds_index = Some(index);
        }
        Ok(())
    }

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.ensure_open("draw_planned_text")?;
        let canonical = backend::validate_planned_text(placement, plan)?;
        self.record_text(
            PlannedTextPlacement {
                position: canonical.position.value(),
                rotation: canonical.rotation.value(),
                ..placement
            },
            plan,
        )?;
        let index = self.bounds.len();
        self.record(planned_text_ink_bounds(
            plan,
            canonical.position.value(),
            placement.anchor,
            canonical.rotation.value(),
            placement.origin,
            placement.reverse_lines,
        ));
        if self.bounds.len() > index {
            self.text.last_mut().expect("recorded text").bounds_index = Some(index);
        }
        Ok(())
    }

    fn draw_rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
        style: &FillStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_rounded_rect")?;
        let rounded = backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
        if rounded.rect.width.value() == 0.0
            || rounded.rect.height.value() == 0.0
            || (style.color.get_alpha() == 0
                && (style.stroke.get_alpha() == 0 || style.stroke_width == 0.0))
        {
            return Ok(());
        }
        let padding = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(if style.stroke.get_alpha() == 0 {
                0.0
            } else {
                style.stroke_width
            }),
            "rectangle stroke width",
        )?
        .units();
        self.record(GeometryBox::from_canonical_points(
            &[
                CanonicalPoint {
                    x: rounded.rect.left,
                    y: rounded.rect.bottom,
                },
                CanonicalPoint {
                    x: rounded.rect.right,
                    y: rounded.rect.top,
                },
            ],
            i128::from(padding),
        )?);
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_bracket")?;
        let canonical = backend::validate_bracket(bracket, style)?;
        let (anchor, alignment) = match canonical.orientation {
            BracketOrientation::Vertical => (TextAnchor::Middle, PhysicalTextAlignment::Center),
            BracketOrientation::Horizontal => (TextAnchor::Start, PhysicalTextAlignment::Start),
        };
        let text_style = TextStyle {
            color: style.label_color,
            font_family: style.font_family.clone(),
            font_size: style.font_size,
            weight: style.weight,
            anchor,
            rotation: 0.0,
        };
        let label = self.plan_raw_text(
            &bracket.label,
            &text_style,
            alignment,
            "geometry significance bracket label",
        )?;
        self.record_planned_bracket(canonical, style, &label)?;
        Ok(())
    }

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.ensure_open("draw_planned_bracket")?;
        let canonical = backend::validate_planned_bracket(call)?;
        self.record_planned_bracket(canonical, call.style, &call.label.plan)?;
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ensure_open("end")?;
        if !self.scopes.is_empty() {
            return Err(Error::Render(format!(
                "geometry backend ended with {} unclosed layer/clip scopes",
                self.scopes.len()
            )));
        }
        self.state = GeometryCanvasState::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw_quantum_stroke(
        geometry: &mut GeometryBackend,
        kind: &str,
        x: f64,
    ) -> Result<(), Error> {
        let width = 1.0 / 1024.0;
        match kind {
            "line" => geometry.draw_line(
                &[Point2D::new(x, 10.0), Point2D::new(x, 20.0)],
                &LineStyle::solid(Color::rgb(0, 0, 0), width),
            ),
            "circle" => {
                let mut style = PointStyle::new((0, 0, 0), 1);
                style.color = style.color.alpha(0);
                style.outline = Some(backend::PointOutline {
                    color: Color::rgb(0, 0, 0),
                    width_ratio: crate::theme::RelativeSpacing::new(f64::from(width)).unwrap(),
                });
                geometry.draw_scatter(&[Point2D::new(x, 15.0)], &style)
            }
            "bracket" => geometry.draw_bracket(
                &SignificanceBracket {
                    x_left: x,
                    x_right: x + 2.0,
                    y: 20.0,
                    label: String::new(),
                    orientation: BracketOrientation::Vertical,
                },
                &BracketStyle {
                    color: Color::rgb(0, 0, 0),
                    width,
                    label_color: Color::rgb(0, 0, 0),
                    font_family: crate::font::canonical::named_stack(&["Test Sans"]),
                    font_size: 12.0,
                    line_height: 14.4,
                    weight: crate::TextWeight::Normal,
                    label_gap: 4.0,
                    tick_height: 7.0,
                },
            ),
            "polygon" => geometry.draw_polygon(
                &[
                    Point2D::new(x, 10.0),
                    Point2D::new(x + 2.0, 10.0),
                    Point2D::new(x + 2.0, 20.0),
                    Point2D::new(x, 20.0),
                ],
                &test_fill_style(0, width),
            ),
            "rectangle" => {
                geometry.draw_rounded_rect(x, 10.0, 2.0, 10.0, 0.0, &test_fill_style(0, width))
            }
            _ => unreachable!("test primitive"),
        }
    }

    #[test]
    fn large_one_quantum_strokes_keep_exact_outward_envelopes_and_clipped_ink() {
        for x in [2.0_f64.powi(43), -2.0_f64.powi(43)] {
            for kind in ["line", "circle", "bracket", "polygon", "rectangle"] {
                let (left, right) = match kind {
                    "line" => (x, x),
                    "circle" => (x - 1.0, x + 1.0),
                    _ => (x, x + 2.0),
                };
                let mut geometry = begun_backend();
                draw_quantum_stroke(&mut geometry, kind, x).unwrap();
                assert_eq!(
                    geometry.bounds()[0].left.to_bits(),
                    left.next_down().to_bits(),
                    "{kind}"
                );
                assert_eq!(
                    geometry.bounds()[0].right.to_bits(),
                    right.next_up().to_bits(),
                    "{kind}"
                );
                geometry
                    .begin_clip(PixelRect::new(right, right + 1.0, 9.0, 21.0))
                    .unwrap();
                draw_quantum_stroke(&mut geometry, kind, x).unwrap();
                assert_eq!(
                    geometry.bounds().len(),
                    2,
                    "clipped {kind} stroke must survive"
                );
                assert_eq!(geometry.bounds()[1].left.to_bits(), right.to_bits());
                assert_eq!(
                    geometry.bounds()[1].right.to_bits(),
                    right.next_up().to_bits()
                );
                assert!(geometry.bounds()[1].overlap_area(geometry.bounds()[1], 0.0) > 0.0);
            }
            let mut geometry = begun_backend();
            geometry
                .draw_line(
                    &[Point2D::new(10.0, x), Point2D::new(20.0, x)],
                    &LineStyle::solid(Color::rgb(0, 0, 0), 1.0 / 1024.0),
                )
                .unwrap();
            assert_eq!(
                geometry.bounds()[0].bottom.to_bits(),
                x.next_down().to_bits()
            );
            assert_eq!(geometry.bounds()[0].top.to_bits(), x.next_up().to_bits());
        }
    }

    fn begun_backend() -> GeometryBackend {
        let mut backend = GeometryBackend::with_environment(PreparedFontEnvironment::synthetic());
        backend.begin(100, 100).unwrap();
        backend
    }

    fn test_fill_style(alpha: u8, stroke_width: f32) -> FillStyle {
        FillStyle::new(
            Color::rgba(0, 0, 0, alpha),
            Color::rgb(0, 0, 0),
            stroke_width,
        )
    }

    #[test]
    fn nested_noncontained_clips_use_their_full_positive_area_intersection() {
        let mut backend = begun_backend();
        let style = test_fill_style(255, 0.0);
        backend
            .begin_clip(PixelRect::new(0.0, 10.0, 0.0, 10.0))
            .unwrap();
        backend
            .begin_clip(PixelRect::new(8.0, 20.0, 8.0, 20.0))
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 20.0, 20.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .begin_clip(PixelRect::new(20.0, 30.0, 20.0, 30.0))
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 30.0, 30.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .begin_clip(PixelRect::new(10.0, 20.0, 0.0, 10.0))
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 20.0, 20.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 20.0, 20.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend.end().unwrap();

        assert_eq!(
            backend.bounds(),
            [
                GeometryBox {
                    left: 8.0,
                    right: 10.0,
                    bottom: 8.0,
                    top: 10.0,
                    weight: 1.0,
                },
                GeometryBox {
                    left: 0.0,
                    right: 10.0,
                    bottom: 0.0,
                    top: 10.0,
                    weight: 1.0,
                },
            ]
        );
    }
}
