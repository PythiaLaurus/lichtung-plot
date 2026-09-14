//! Validating backend that records conservative bounds for every primitive.

use std::collections::HashMap;

use crate::Error;

use crate::font::canonical::PreparedFontEnvironment;
use crate::layout::{
    PanelBox, PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight, TextPlanInput,
    TextPlanProvenance, CSS_PIXEL_QUANTA,
};
use crate::render::backend::{
    self, Backend, BracketOrientation, BracketStyle, FillStyle, GradientStop, LineStyle,
    PlannedSignificanceCall, PlannedTextOrigin, PlannedTextPlacement, Point2D, PointStyle, Rgba,
    SignificanceBracket, SurfaceRole, TextAnchor, TextStyle, POLYGON_MITER_LIMIT,
};
use crate::render::coordinates::{CanonicalBracket, CanonicalPoint};
use crate::PointShape;

// Labels and annotation glyphs are less tolerable legend targets than translucent marks.
const ANNOTATION_COLLISION_WEIGHT: f64 = 100.0;

#[cfg(test)]
thread_local! {
    static GEOMETRY_RAW_TEXT_PLAN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static GEOMETRY_BACKEND_BEGINS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static GEOMETRY_BACKEND_CONSTRUCTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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

    fn clipped(self, clip: Option<PanelBox>) -> Option<Self> {
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

    fn union(self, other: Self) -> Self {
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

pub(crate) struct GeometryBackend {
    snapshot: GeometrySnapshot,
    clips: Vec<PanelBox>,
    scopes: Vec<GeometryScope>,
    state: GeometryCanvasState,
    operation_default_stack: String,
}

/// Retained consequences of selected geometry, independent of backend sessions.
#[derive(Debug)]
pub(crate) struct GeometrySnapshot {
    bounds: Vec<GeometryBox>,
    text: Vec<PositionedText>,
    font_environment: PreparedFontEnvironment,
}

#[derive(Debug)]
struct PositionedText {
    placement: PlannedTextPlacement,
    plan: ResolvedTextPlan,
    clip: Option<PanelBox>,
    bounds_index: Option<usize>,
}

impl std::ops::Deref for GeometryBackend {
    type Target = GeometrySnapshot;
    fn deref(&self) -> &Self::Target {
        &self.snapshot
    }
}

impl std::ops::DerefMut for GeometryBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.snapshot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeometryScope {
    Layer,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum GeometryCanvasState {
    #[default]
    Fresh,
    Open,
    Closed,
}

impl GeometryBackend {
    pub(crate) fn new(theme: &crate::theme::ResolvedTheme) -> Result<Self, Error> {
        Ok(Self::with_environment(
            theme.prepared_font_environment()?.clone(),
            theme.font_family.clone(),
        ))
    }

    fn with_environment(
        font_environment: PreparedFontEnvironment,
        operation_default_stack: String,
    ) -> Self {
        #[cfg(test)]
        GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(calls.get() + 1));
        Self {
            snapshot: GeometrySnapshot {
                bounds: Vec::new(),
                text: Vec::new(),
                font_environment,
            },
            clips: Vec::new(),
            scopes: Vec::new(),
            state: GeometryCanvasState::Fresh,
            operation_default_stack,
        }
    }

    pub(crate) fn into_snapshot(self) -> GeometrySnapshot {
        self.snapshot
    }
}

impl GeometrySnapshot {
    fn admit_scratch<T>(&self, count: usize) -> Result<(), Error> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("geometry scratch storage overflow".into()))?;
        self.font_environment
            .claim_processing_visits(count as u128)
            .map_err(|error| Error::Render(format!("geometry scratch work: {error:?}")))?;
        self.font_environment
            .claim_unicode_bytes(bytes)
            .map_err(|error| Error::Render(format!("geometry scratch storage: {error}")))
    }
    pub(crate) fn new(theme: &crate::theme::ResolvedTheme) -> Result<Self, Error> {
        Ok(Self {
            bounds: Vec::new(),
            text: Vec::new(),
            font_environment: theme.prepared_font_environment()?.clone(),
        })
    }

    pub(crate) fn record(
        &mut self,
        bounds: Option<GeometryBox>,
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        if let Some(bounds) = bounds.and_then(|bounds| bounds.clipped(clip)) {
            backend::reserve_text_records(&mut self.bounds, 1, &self.font_environment)?;
            self.bounds.push(bounds);
        }
        Ok(())
    }

    pub(crate) fn scatter(
        &mut self,
        points: &[Point2D],
        style: &PointStyle,
        clip: PanelBox,
    ) -> Result<(), Error> {
        self.scatter_with_clip(points, style, Some(clip))
    }

    pub(crate) fn scatter_with_clip(
        &mut self,
        points: &[Point2D],
        style: &PointStyle,
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        self.admit_scratch::<CanonicalPoint>(points.len())?;
        if style.shape == PointShape::TriangleUp {
            self.admit_scratch::<[CanonicalPoint; 3]>(points.len())?;
            self.admit_scratch::<[Option<[StrokeInterval; 2]>; 3]>(points.len())?;
        }
        scatter_bounds(points, style, |bounds| self.record(bounds, clip))
    }

    pub(crate) fn rounded_rect(
        &mut self,
        [x, y, width, height, radius]: [f64; 5],
        style: &FillStyle,
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        let rounded = backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
        if rounded.rect.width.value() == 0.0
            || rounded.rect.height.value() == 0.0
            || (style.opacity == 0.0 && style.stroke_width == 0.0)
        {
            return Ok(());
        }
        let padding = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(style.stroke_width),
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
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        self.admit_scratch::<CanonicalPoint>(points.len())?;
        line_bounds(points, style, |bounds| self.record(bounds, clip))
    }

    pub(crate) fn text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        let canonical = backend::validate_planned_text(placement, plan)?;
        let placement = PlannedTextPlacement {
            position: canonical.position.value(),
            rotation: canonical.rotation.value(),
            ..placement
        };
        backend::reserve_text_records(&mut self.text, 1, &self.font_environment)?;
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
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        if points.len() < 3 {
            return Ok(());
        }
        self.admit_scratch::<CanonicalPoint>(points.len())?;
        self.admit_scratch::<CanonicalPoint>(points.len())?;
        self.admit_scratch::<Option<[StrokeInterval; 2]>>(points.len())?;
        let points = backend::validate_polygon_points(points)?;
        let style = super::primitives::CanonicalFillStyle::new(style)?;
        let ring = polygon_ring(&points);
        let fill = (style.opacity > 0.0 && polygon_has_positive_area(&ring)?)
            .then(|| GeometryBox::from_canonical_points(&ring, 0))
            .transpose()?
            .flatten();
        let stroke = polygon_stroke_bounds(&ring, style.stroke_width.units())?;
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
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
        clip: Option<PanelBox>,
    ) -> Result<(), Error> {
        self.admit_scratch::<super::coordinates::CanonicalGradientStop>(stops.len())?;
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
        let placement = backend::planned_bracket_placement(canonical);
        backend::reserve_text_records(&mut self.text, 1, &self.font_environment)?;
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
        backend::reserve_text_records(
            &mut self.bounds,
            other.bounds.len(),
            &self.font_environment,
        )?;
        backend::reserve_text_records(&mut self.text, other.text.len(), &self.font_environment)?;
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
            round.observe_box(PanelBox::new(
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
        before: PanelBox,
        after: PanelBox,
        legend: GeometryBox,
        margin: f64,
        screen_text: impl Fn(&ResolvedTextPlan) -> Option<PlannedTextPlacement>,
    ) -> Result<f64, Error> {
        self.font_environment
            .claim_processing_visits(self.bounds.len() as u128 + self.text.len() as u128)
            .map_err(|error| Error::Render(format!("assignment feedback work: {error:?}")))?;
        let mut text = self
            .text
            .iter()
            .filter(|text| text.bounds_index.is_some())
            .peekable();
        let mut overlap = 0.0;
        for (index, bounds) in self.bounds.iter().enumerate() {
            #[cfg(test)]
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::GeometryOverlapScan,
            );
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
        root: PanelBox,
        current: &crate::layout::Projection,
        next: &crate::layout::Projection,
        declared: impl Iterator<Item = &'a ResolvedTextPlan> + Clone,
    ) -> Result<bool, Error> {
        for text in &self.text {
            self.font_environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("annotation feedback visits: {error:?}")))?;
            let mut found = false;
            for plan in declared.clone() {
                self.font_environment
                    .claim_processing_visits(1)
                    .map_err(|error| {
                        Error::Render(format!("annotation feedback visits: {error:?}"))
                    })?;
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

impl GeometryBackend {
    fn record_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        backend::reserve_text_records(&mut self.snapshot.text, 1, &self.snapshot.font_environment)?;
        self.font_environment
            .claim_processing_visits(self.clips.len() as u128)
            .map_err(|error| Error::Render(format!("geometry text admission: {error:?}")))?;
        let clip = self.clips.iter().copied().reduce(|a, b| {
            let left = a.x0.max(b.x0);
            let bottom = a.y0.max(b.y0);
            PanelBox::new(
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
        #[cfg(test)]
        GEOMETRY_RAW_TEXT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        // Point-layer labels still carry the legacy first-family token until
        // their renderer owns immutable text plans; use the operation policy.
        let stack = if style.font_family == crate::DEFAULT_FONT_FAMILY {
            &self.operation_default_stack
        } else {
            &style.font_family
        };
        self.font_environment
            .plan_prepared_text(TextPlanInput {
                source: text,
                stack,
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
        let placement = backend::planned_bracket_placement(canonical);
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
        .map(|outline| {
            super::coordinates::CanonicalCoordinate::nonnegative_length(
                outline.width_for_radius(style.radius),
                "scatter outline width",
            )
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
                if style.opacity == 0.0 && outline_width == 0 {
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
                let fill = (style.opacity > 0.0)
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
    if style.width.units() == 0 {
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
    bounds.map(GeometryBox::outward)
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
        let product = |direction: i128, coordinate: i64| {
            direction
                .checked_mul(i128::from(coordinate))
                .ok_or_else(overflow)
        };
        let line = product(direction_y, start.x.units())?
            .checked_sub(product(direction_x, start.y.units())?)
            .ok_or_else(overflow)?;
        let start_projection = product(direction_x, start.x.units())?
            .checked_add(product(direction_y, start.y.units())?)
            .ok_or_else(overflow)?;
        let end_projection = product(direction_x, end.x.units())?
            .checked_add(product(direction_y, end.y.units())?)
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

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
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
        bounds: PanelBox,
        _paint: Rgba,
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
        let fill_bounds = (style.opacity > 0.0 && polygon_has_positive_area(&ring)?)
            .then(|| GeometryBox::from_canonical_points(&ring, 0))
            .transpose()?
            .flatten();
        let stroke_bounds = polygon_stroke_bounds(&ring, style.stroke_width.units())?;
        self.record(match (fill_bounds, stroke_bounds) {
            (Some(fill), Some(stroke)) => Some(fill.union(stroke)),
            (fill, stroke) => fill.or(stroke),
        });
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
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
            || (style.opacity == 0.0 && style.stroke_width == 0.0)
        {
            return Ok(());
        }
        let padding = super::coordinates::CanonicalCoordinate::nonnegative_length(
            f64::from(style.stroke_width),
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
                &LineStyle::solid((0, 0, 0), width),
            ),
            "circle" => geometry.draw_scatter(
                &[Point2D::new(x, 15.0)],
                &PointStyle::with_outline(
                    (0, 0, 0),
                    1,
                    Some(backend::PointOutline {
                        color: (0, 0, 0),
                        width_ratio: f64::from(width),
                    }),
                )
                .with_opacity(0.0),
            ),
            "bracket" => geometry.draw_bracket(
                &SignificanceBracket {
                    x_left: x,
                    x_right: x + 2.0,
                    y: 20.0,
                    label: String::new(),
                    orientation: BracketOrientation::Vertical,
                },
                &BracketStyle {
                    color: (0, 0, 0).into(),
                    width,
                    label_color: (0, 0, 0).into(),
                    font_family: "Test Sans".into(),
                    font_size: 12,
                    line_height: 14.4,
                    weight: crate::TextWeight::Normal,
                    label_gap: 4.0,
                },
            ),
            "polygon" => geometry.draw_polygon(
                &[
                    Point2D::new(x, 10.0),
                    Point2D::new(x + 2.0, 10.0),
                    Point2D::new(x + 2.0, 20.0),
                    Point2D::new(x, 20.0),
                ],
                &test_fill_style(0.0, width),
            ),
            "rectangle" => {
                geometry.draw_rounded_rect(x, 10.0, 2.0, 10.0, 0.0, &test_fill_style(0.0, width))
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
                    .begin_clip(PanelBox::new(right, right + 1.0, 9.0, 21.0))
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
                    &LineStyle::solid((0, 0, 0), 1.0 / 1024.0),
                )
                .unwrap();
            assert_eq!(
                geometry.bounds()[0].bottom.to_bits(),
                x.next_down().to_bits()
            );
            assert_eq!(geometry.bounds()[0].top.to_bits(), x.next_up().to_bits());
        }
    }

    #[test]
    fn painted_edges_reject_signed_quantum_and_directed_float_overflow() {
        let point = |x| CanonicalPoint {
            x: super::super::coordinates::CanonicalCoordinate::from_quanta(x),
            y: super::super::coordinates::CanonicalCoordinate::from_quanta(0),
        };
        for x in [i64::MIN, i64::MAX] {
            assert!(GeometryBox::from_canonical_points(&[point(x)], 1)
                .unwrap_err()
                .to_string()
                .contains("painted bound exceeds signed quantum range"));
        }
        for padding in [i128::MIN, i128::MAX] {
            assert!(GeometryBox::from_canonical_points(&[point(0)], padding).is_err());
        }
        let last_float_quanta = i64::MAX - 1023;
        assert!(GeometryBox::from_canonical_points(&[point(last_float_quanta)], 0).is_ok());
        assert!(
            GeometryBox::from_canonical_points(&[point(last_float_quanta)], 1)
                .unwrap_err()
                .to_string()
                .contains("painted collision bound exceeds signed quantum range")
        );
        // An unrepresentable canonical vertex must not be rounded before expansion.
        let bounds = GeometryBox::from_canonical_points(&[point((1_i64 << 53) + 1)], 1)
            .unwrap()
            .unwrap();
        let x = 2.0_f64.powi(43);
        assert_eq!(bounds.left.to_bits(), x.to_bits());
        assert_eq!(bounds.right.to_bits(), x.next_up().to_bits());
        for kind in ["line", "circle", "bracket", "polygon", "rectangle"] {
            let mut geometry = begun_backend();
            // Valid canonical geometry at i64::MIN, but its stroke crosses the edge.
            let error = draw_quantum_stroke(&mut geometry, kind, -2.0_f64.powi(53)).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("painted bound exceeds signed quantum range"),
                "{kind}: {error}"
            );
            assert!(geometry.bounds().is_empty());
        }
    }

    #[test]
    fn local_polygon_miters_stay_conservative_under_large_translations() {
        for x in [0.0, 2.0_f64.powi(43), -2.0_f64.powi(43)] {
            let bounds = recorded_polygon(
                &[
                    Point2D::new(x, 0.0),
                    Point2D::new(x + 10.0, 0.0),
                    Point2D::new(x + 6.0, 3.0),
                ],
                &test_fill_style(0.0, 2.0),
            )
            .unwrap();
            // The 3-4-5 edge produces an exact three-pixel miter at the right tip.
            assert!(bounds.right >= x + 13.0);
            assert!(bounds.right <= (x + 13.0 + 1.0 / 1024.0).next_up());
            let points = backend::validate_polygon_points(&[
                Point2D::new(x, 0.0),
                Point2D::new(x + 10.0, 0.0),
                Point2D::new(x + 6.0, 3.0),
            ])
            .unwrap();
            // Thin diagonal stroke offsets survive the same local-to-absolute path.
            let thin = polygon_stroke_bounds(&points, 1).unwrap().unwrap();
            assert!(thin.right > x + 10.0);
            assert!(thin.bottom < 0.0);
        }
        let high = 2.0_f64.powi(53) - 2.0;
        let points = backend::validate_polygon_points(&[
            Point2D::new(high - 10.0, 0.0),
            Point2D::new(high, 0.0),
            Point2D::new(high - 4.0, 3.0),
        ])
        .unwrap();
        assert!(polygon_stroke_bounds(&points, 2048)
            .unwrap_err()
            .to_string()
            .contains("painted bound exceeds signed quantum range"));
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "canonical quanta must agree exactly")]
    fn raw_fractional_text_matches_planned_quantum_placement() {
        let style = test_text_style();
        for x in [-1.2345, 0.5 / 1024.0, 1.5 / 1024.0, 20.125] {
            let position = Point2D::new(x, 30.2345);
            let mut raw = begun_backend();
            raw.draw_text(position, "A", &style).unwrap();
            let mut planned = begun_backend();
            let plan = planned
                .plan_raw_text("A", &style, PhysicalTextAlignment::Center, "test")
                .unwrap();
            let placement = PlannedTextPlacement::middle(position, style.anchor, style.rotation);
            planned.draw_planned_text(placement, &plan).unwrap();
            assert_eq!(raw.bounds(), planned.bounds());
            let canonical =
                super::super::primitives::CanonicalPlannedText::new(placement, &plan, 100, None)
                    .unwrap();
            assert_eq!(
                raw.text[0].placement.position.x,
                canonical.0.center.x.value()
            );
            assert_eq!(
                raw.text[0].placement.position.y,
                100.0 - canonical.0.center.y.value()
            );
        }
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "outward quantum edges must agree exactly")]
    fn painted_marker_outlines_survive_fill_excluding_clips() {
        for shape in [PointShape::Circle, PointShape::TriangleUp] {
            let mut geometry = begun_backend();
            geometry
                .begin_clip(PanelBox::new(8.0, 12.0, 12.0, 13.0))
                .unwrap();
            geometry
                .draw_scatter(
                    &[Point2D::new(10.0, 10.0)],
                    &PointStyle::new((0, 0, 0), 2)
                        .with_shape(shape)
                        .with_opacity(0.0),
                )
                .unwrap();
            let bounds = geometry.bounds()[0];
            assert_eq!(bounds.bottom, 12.0);
            assert!(bounds.top > 12.0);
            // Triangle tip miter extends a full stroke width, not half.
            assert_eq!(
                bounds.top,
                if shape == PointShape::Circle {
                    12.100_585_937_5
                } else {
                    12.200_195_312_5
                }
            );
        }
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "outward quantum edges must agree exactly")]
    fn odd_quantum_strokes_round_outward_before_clipping() {
        let mut geometry = begun_backend();
        geometry
            .draw_line(
                &[Point2D::new(1.0, 2.0), Point2D::new(3.0, 2.0)],
                &LineStyle::solid((0, 0, 0), 1.0 / 1024.0),
            )
            .unwrap();
        let bounds = geometry.bounds()[0];
        assert_eq!(bounds.left, 1.0 - 1.0 / 1024.0);
        assert_eq!(bounds.top, 2.0 + 1.0 / 1024.0);
        geometry
            .draw_rounded_rect(1.0, 1.0, 2.0, 2.0, 0.0, &test_fill_style(0.0, 1.0 / 1024.0))
            .unwrap();
        assert_eq!(geometry.bounds()[1].left, bounds.left);
    }

    #[test]
    fn bracket_uses_quantized_stroke_before_outward_clipped_bounds() {
        let mut geometry = begun_backend();
        let style = BracketStyle {
            color: (0, 0, 0).into(),
            width: 0.8009,
            label_color: (0, 0, 0).into(),
            font_family: "Test Sans".into(),
            font_size: 12,
            line_height: 14.4,
            weight: crate::TextWeight::Normal,
            label_gap: 4.0,
        };
        geometry
            .begin_clip(PanelBox::new(9.0, 10.0, 29.0, 30.0))
            .unwrap();
        geometry
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 10.0,
                    x_right: 20.0,
                    y: 30.0,
                    label: String::new(),
                    orientation: BracketOrientation::Vertical,
                },
                &style,
            )
            .unwrap();
        // 0.8009 becomes 820 quanta: its half is exactly 410, not 411.
        assert_eq!(
            geometry.bounds()[0].left.to_bits(),
            (10.0_f64 - 410.0 / 1024.0).to_bits()
        );
        assert_eq!(geometry.bounds()[0].right.to_bits(), 10.0_f64.to_bits());
    }

    #[test]
    fn selected_geometry_admits_scratch_and_bounds_before_allocating() {
        for remaining in [0, std::mem::size_of::<CanonicalPoint>() as u64] {
            let theme = crate::Theme::Publication.resolve();
            let mut geometry = GeometrySnapshot::new(&theme).unwrap();
            let used = geometry
                .font_environment
                .resource_usage()
                .unwrap()
                .font_bytes;
            geometry
                .font_environment
                .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap() - remaining)
                .unwrap();
            let style = PointStyle::new((0, 0, 0), 3);
            let error = geometry
                .scatter(
                    &[Point2D::new(50.0, 50.0)],
                    &style,
                    PanelBox::new(0.0, 100.0, 0.0, 100.0),
                )
                .unwrap_err();
            assert!(error.to_string().contains("bytes"), "{error}");
            assert_eq!(geometry.bounds.capacity(), 0);
            assert!(geometry.bounds.is_empty());
        }
    }

    #[test]
    fn geometry_text_bytes_refuse_before_allocation_or_clip_processing() {
        let mut geometry = begun_backend();
        let plan = geometry
            .plan_raw_text(
                "A",
                &test_text_style(),
                PhysicalTextAlignment::Start,
                "test",
            )
            .unwrap();
        geometry.clips.push(PanelBox::new(0.0, 100.0, 0.0, 100.0));
        let before = geometry.font_environment.resource_usage().unwrap();
        geometry
            .font_environment
            .claim_unicode_bytes(u64::try_from(536_870_912 - before.font_bytes).unwrap())
            .unwrap();
        assert_eq!(geometry.text.capacity(), 0);
        let placement =
            PlannedTextPlacement::middle(Point2D::new(10.0, 10.0), TextAnchor::Start, 0.0);
        assert!(geometry
            .record_text(placement, &plan)
            .unwrap_err()
            .to_string()
            .contains("system-font bytes"));
        assert_eq!(geometry.text.capacity(), 0);
        assert!(geometry.text.is_empty());
        assert_eq!(
            geometry
                .font_environment
                .resource_usage()
                .unwrap()
                .processing_visits,
            before.processing_visits
        );
    }

    #[test]
    fn collision_pairs_share_the_font_processing_cap() {
        let mut geometry = begun_backend();
        let bounds = GeometryBox {
            left: 1.0,
            right: 2.0,
            bottom: 1.0,
            top: 2.0,
            weight: 1.0,
        };
        geometry.bounds = vec![bounds; 2];
        let environment = &geometry.font_environment;
        let used = environment.resource_usage().unwrap().processing_visits;
        environment
            .claim_processing_visits(1_000_000 - used - 2)
            .unwrap();
        let panel = PanelBox::new(0.0, 100.0, 0.0, 100.0);
        assert_eq!(
            geometry
                .overlap_after_assignment(panel, panel, bounds, 0.0, |_| None)
                .unwrap()
                .to_bits(),
            2.0_f64.to_bits()
        );
        assert_eq!(
            environment.resource_usage().unwrap().processing_visits,
            1_000_000
        );
        assert!(geometry
            .overlap_after_assignment(panel, panel, bounds, 0.0, |_| None)
            .unwrap_err()
            .to_string()
            .contains("root processing visits"));
    }

    fn test_text_style() -> TextStyle {
        TextStyle {
            color: (0, 0, 0).into(),
            font_family: "Test Sans".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        }
    }

    fn begun_backend() -> GeometryBackend {
        let mut backend = GeometryBackend::with_environment(
            PreparedFontEnvironment::synthetic(),
            "Test Sans".into(),
        );
        backend.begin(100, 100).unwrap();
        backend
    }

    #[test]
    fn feedback_observes_annotation_placement_even_when_clipped_ink_is_unchanged() {
        let mut producer = begun_backend();
        let plan = producer
            .plan_raw_text(
                "A",
                &test_text_style(),
                PhysicalTextAlignment::Center,
                "deferred annotation",
            )
            .unwrap();
        producer.end().unwrap();
        let mut calls = 0;
        let mut observations = None;
        crate::layout::plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let owner = crate::layout::ComponentAddress::root();
                round.request_insets(
                    &owner,
                    &owner,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                let mut geometry = begun_backend();
                geometry.begin_clip(PanelBox::new(0.0, 1.0, 0.0, 1.0))?;
                geometry.draw_planned_text(
                    PlannedTextPlacement::middle(
                        Point2D::new(if calls == 1 { 50.0 } else { 60.0 }, 50.0),
                        TextAnchor::Middle,
                        0.0,
                    ),
                    &plan,
                )?;
                geometry.end_clip()?;
                geometry.end()?;
                assert!(geometry.bounds().is_empty());
                let start = round.child_checkpoint();
                geometry.observe_feedback(round)?;
                let current = round.retain_child_feedback(start)?.unwrap();
                if let Some(previous) = &observations {
                    assert_ne!(&current, previous);
                }
                observations = Some(current);
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(calls, 2);
    }

    #[test]
    fn root_text_overflow_uses_only_a_linked_resolving_domain_request() {
        use crate::layout::kernel::{FeedbackKind, FitKind};
        let mut producer = begun_backend();
        let plan = producer
            .plan_raw_text(
                "A",
                &test_text_style(),
                PhysicalTextAlignment::Center,
                "deferred annotation",
            )
            .unwrap();
        producer.end().unwrap();
        for (declared, requested_axis) in [
            (true, FeedbackKind::AutomaticDomainX),
            (false, FeedbackKind::AutomaticDomainX),
            (true, FeedbackKind::AutomaticDomainY),
        ] {
            let mut calls = 0;
            let root = crate::layout::plan_exact_root(
                100,
                100,
                |_| Ok(()),
                |(), assigned, round| {
                    calls += 1;
                    let address = crate::layout::ComponentAddress::root();
                    let domain = round
                        .domain(&address, FeedbackKind::AutomaticDomainX)
                        .unwrap_or((0.0, 1.0));
                    let panel = PanelBox::new(0.0, 100.0, 0.0, 100.0);
                    let current =
                        crate::layout::Projection::new(domain, (0.0, 1.0), panel.as_tuple());
                    let (x, y) = current.project(1.1, 0.5);
                    let mut geometry = begun_backend();
                    geometry.draw_planned_text(
                        PlannedTextPlacement::middle(Point2D::new(x, y), TextAnchor::Middle, 0.0),
                        &plan,
                    )?;
                    geometry.end()?;
                    geometry.observe_feedback(round)?;
                    let bounds = geometry.bounds()[0];
                    if bounds.right > panel.x1 {
                        let request =
                            round.request_domain(&address, &address, requested_axis, (0.0, 2.0))?;
                        let next = crate::layout::Projection::new(
                            if requested_axis == FeedbackKind::AutomaticDomainX {
                                (0.0, 2.0)
                            } else {
                                domain
                            },
                            if requested_axis == FeedbackKind::AutomaticDomainY {
                                (0.0, 2.0)
                            } else {
                                (0.0, 1.0)
                            },
                            panel.as_tuple(),
                        );
                        let remedy = geometry.text_overflow_resolved(
                            bounds,
                            panel,
                            &current,
                            &next,
                            declared.then_some(&plan).into_iter(),
                        )?;
                        let requests = [request];
                        round.miss(
                            &address,
                            FitKind::OverlayCollision,
                            if remedy { &requests } else { &[] },
                            "root annotation overflow",
                        )?;
                    }
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            );
            if declared && requested_axis == FeedbackKind::AutomaticDomainX {
                assert!(root.is_ok());
                assert_eq!(calls, 2);
            } else {
                assert!(root
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("root annotation overflow"));
                assert_eq!(calls, 1);
            }
        }
    }

    fn test_fill_style(opacity: f64, stroke_width: f32) -> FillStyle {
        FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width,
            opacity,
        }
    }

    fn recorded_polygon(points: &[Point2D], style: &FillStyle) -> Option<GeometryBox> {
        let mut backend = begun_backend();
        backend.draw_polygon(points, style).unwrap();
        backend.end().unwrap();
        backend.bounds().first().copied()
    }

    fn has_polygon_fill(points: &[Point2D]) -> bool {
        let points = backend::validate_polygon_points(points).unwrap();
        polygon_has_positive_area(&polygon_ring(&points)).unwrap()
    }

    #[test]
    fn geometry_lifecycle_requires_one_balanced_session_and_fresh_reuse() {
        let points = [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        let style = LineStyle::solid((0, 0, 0), 1.0);
        let mut backend = GeometryBackend::with_environment(
            PreparedFontEnvironment::synthetic(),
            "Test Sans".into(),
        );

        assert!(backend.begin(0, 100).is_err());
        assert!(backend.draw_line(&points, &style).is_err());
        backend.begin(100, 100).unwrap();
        assert!(backend.begin(100, 100).is_err());
        backend.draw_line(&points, &style).unwrap();
        backend
            .begin_clip(PanelBox::new(0.0, 10.0, 0.0, 10.0))
            .unwrap();
        assert!(backend.end().is_err());
        backend.end_clip().unwrap();
        backend.end().unwrap();
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.end().is_err());
        assert_eq!(backend.bounds().len(), 1);

        backend.begin(200, 150).unwrap();
        assert!(backend.bounds().is_empty());
        backend.end().unwrap();
    }

    #[test]
    fn geometry_scopes_require_balanced_lifo_closure() {
        let mut backend = begun_backend();

        assert!(backend.end_layer().is_err());
        assert!(backend.end_clip().is_err());
        backend.begin_layer("marks").unwrap();
        assert!(backend.end_clip().is_err());
        assert!(backend.end().is_err());
        backend
            .begin_clip(PanelBox::new(0.0, 10.0, 0.0, 10.0))
            .unwrap();
        assert!(backend.end_layer().is_err());
        backend.end_clip().unwrap();
        backend.end_layer().unwrap();
        backend.end().unwrap();
    }

    #[test]
    fn surfaces_validate_without_becoming_collision_obstacles() {
        let mut backend = begun_backend();

        backend
            .draw_surface(
                PanelBox::new(0.0, 100.0, 0.0, 100.0),
                Rgba::new(1, 2, 3, 128),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        assert!(backend
            .draw_surface(
                PanelBox::new(1.0, 1.0, 2.0, 3.0),
                Rgba::rgb(4, 5, 6),
                SurfaceRole::PlotPanel,
            )
            .is_err());
        backend.end().unwrap();

        assert!(backend.bounds().is_empty());
    }

    #[test]
    fn nested_noncontained_clips_use_their_full_positive_area_intersection() {
        let mut backend = begun_backend();
        let style = test_fill_style(1.0, 0.0);
        backend
            .begin_clip(PanelBox::new(0.0, 10.0, 0.0, 10.0))
            .unwrap();
        backend
            .begin_clip(PanelBox::new(8.0, 20.0, 8.0, 20.0))
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 20.0, 20.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .begin_clip(PanelBox::new(20.0, 30.0, 20.0, 30.0))
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 30.0, 30.0, 0.0, &style)
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 20.0, 0.0, 10.0))
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

    #[test]
    fn point_envelopes_and_weighted_overlap_keep_the_exact_local_rules() {
        assert!(GeometryBox::from_points(&[], 2.0).is_none());
        let bounds =
            GeometryBox::from_points(&[Point2D::new(2.0, 3.0), Point2D::new(5.0, 7.0)], 1.0)
                .unwrap();
        let weighted = GeometryBox {
            weight: 100.0,
            ..bounds
        };
        let other = GeometryBox {
            left: 5.0,
            right: 8.0,
            bottom: 6.0,
            top: 9.0,
            weight: 1.0,
        };

        assert_eq!(
            bounds,
            GeometryBox {
                left: 1.0,
                right: 6.0,
                bottom: 2.0,
                top: 8.0,
                weight: 1.0,
            }
        );
        assert!((bounds.overlap_area(other, 0.0) - 2.0).abs() < f64::EPSILON);
        assert!((weighted.overlap_area(other, 0.0) - 200.0).abs() < f64::EPSILON);
        assert!((bounds.overlap_area(other, 1.0) - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_width_lines_are_omitted_after_geometry_validation() {
        let mut backend = begun_backend();
        backend
            .draw_line(
                &[Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)],
                &LineStyle::solid((0, 0, 0), 0.0),
            )
            .unwrap();
        let error = backend
            .draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)],
                &LineStyle::solid((0, 0, 0), 0.0),
            )
            .unwrap_err();
        backend.end().unwrap();

        assert!(backend.bounds().is_empty());
        assert!(error.to_string().contains("distinct points"));
    }

    #[test]
    fn solid_and_dashed_lines_keep_local_round_cap_and_join_bounds() {
        let mut backend = begun_backend();
        backend
            .draw_line(
                &[
                    Point2D::new(10.0, 20.0),
                    Point2D::new(20.0, 20.0),
                    Point2D::new(20.0, 30.0),
                ],
                &LineStyle::solid((0, 0, 0), 4.0),
            )
            .unwrap();
        backend
            .draw_line(
                &[Point2D::new(10.0, 40.0), Point2D::new(30.0, 40.0)],
                &LineStyle::short_dashed((0, 0, 0), 4.0),
            )
            .unwrap();
        backend.end().unwrap();

        assert_eq!(
            backend.bounds(),
            [
                GeometryBox {
                    left: 8.0,
                    right: 22.0,
                    bottom: 18.0,
                    top: 22.0,
                    weight: 1.0,
                },
                GeometryBox {
                    left: 18.0,
                    right: 22.0,
                    bottom: 18.0,
                    top: 32.0,
                    weight: 1.0,
                },
                GeometryBox {
                    left: 8.0,
                    right: 32.0,
                    bottom: 38.0,
                    top: 42.0,
                    weight: 1.0,
                },
            ]
        );
    }

    #[test]
    fn polygon_bounds_union_only_current_painted_fill_and_stroke() {
        let triangle = [
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(0.0, 10.0),
        ];
        let fill = recorded_polygon(&triangle, &test_fill_style(1.0, 0.0)).unwrap();
        let stroke = recorded_polygon(&triangle, &test_fill_style(0.0, 2.0)).unwrap();
        let union = recorded_polygon(&triangle, &test_fill_style(1.0, 2.0)).unwrap();

        assert_eq!(
            fill,
            GeometryBox {
                left: 0.0,
                right: 10.0,
                bottom: 0.0,
                top: 10.0,
                weight: 1.0,
            }
        );
        assert_eq!(stroke, union);
        assert!(stroke.left < -0.9 && stroke.right > 12.4);
        assert!(stroke.bottom < -0.9 && stroke.top > 12.4);
        assert!(recorded_polygon(&triangle, &test_fill_style(0.0, 0.0)).is_none());
        assert!(recorded_polygon(
            &[
                Point2D::new(0.0, 0.0),
                Point2D::new(5.0, 0.0),
                Point2D::new(10.0, 0.0),
            ],
            &test_fill_style(1.0, 0.0),
        )
        .is_none());
    }

    #[test]
    fn polygon_miters_include_admitted_acute_tips_and_bevel_over_limit_tips() {
        let admitted = recorded_polygon(
            &[
                Point2D::new(0.0, 0.0),
                Point2D::new(10.0, 0.0),
                Point2D::new(6.0, 3.0),
            ],
            &test_fill_style(0.0, 2.0),
        )
        .unwrap();
        let limited = recorded_polygon(
            &[
                Point2D::new(0.0, 0.0),
                Point2D::new(12.0, 0.0),
                Point2D::new(0.0, 5.0),
            ],
            &test_fill_style(0.0, 2.0),
        )
        .unwrap();

        // Directed local arithmetic may enclose an exactly integral tip by one quantum.
        assert!(admitted.right >= 13.0 && admitted.right <= 13.0 + 1.0 / 1024.0);
        let expected = ((12.0_f64 + 5.0 / 13.0) * 1024.0).ceil() / 1024.0;
        assert_eq!(limited.right.to_bits(), expected.to_bits());
    }

    #[test]
    fn nonzero_fill_detection_handles_zero_signed_area_paths() {
        assert!(has_polygon_fill(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 2.0),
            Point2D::new(0.0, 2.0),
            Point2D::new(2.0, 0.0),
        ]));
        assert!(!has_polygon_fill(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(0.0, 0.0),
        ]));
        assert!(!has_polygon_fill(&[
            Point2D::new(497.985, -43.074),
            Point2D::new(959.176, 507.531),
            Point2D::new(460.608, 541.168),
            Point2D::new(959.176, 507.531),
        ]));
    }

    #[test]
    fn rounded_rect_bounds_follow_current_painted_union_and_positive_area() {
        let record = |width, height, radius, style: FillStyle| {
            let mut backend = begun_backend();
            backend
                .draw_rounded_rect(10.0, 20.0, width, height, radius, &style)
                .unwrap();
            backend.end().unwrap();
            backend.bounds().first().copied()
        };
        let fill = record(30.0, 20.0, 0.0, test_fill_style(1.0, 0.0)).unwrap();
        let stroke = record(30.0, 20.0, 0.0, test_fill_style(0.0, 2.0)).unwrap();
        let union = record(30.0, 20.0, 10.0, test_fill_style(1.0, 2.0)).unwrap();

        assert_eq!(
            fill,
            GeometryBox {
                left: 10.0,
                right: 40.0,
                bottom: 20.0,
                top: 40.0,
                weight: 1.0,
            }
        );
        assert_eq!(stroke, union);
        assert_eq!(
            stroke,
            GeometryBox {
                left: 9.0,
                right: 41.0,
                bottom: 19.0,
                top: 41.0,
                weight: 1.0,
            }
        );
        assert!(record(30.0, 20.0, 0.0, test_fill_style(0.0, 0.0)).is_none());
        assert!(record(0.0, 20.0, 0.0, test_fill_style(1.0, 2.0)).is_none());
        assert!(record(30.0, 0.0, 0.0, test_fill_style(1.0, 2.0)).is_none());
    }

    #[test]
    fn scatter_and_polyline_record_individual_local_obstacles() {
        let mut backend = begun_backend();
        let points = [
            Point2D::new(10.0, 10.0),
            Point2D::new(90.0, 90.0),
            Point2D::new(10.0, 90.0),
        ];

        backend
            .draw_scatter(&points[..2], &PointStyle::plain((0, 0, 0), 2))
            .unwrap();
        backend
            .draw_line(&points, &LineStyle::solid((0, 0, 0), 2.0))
            .unwrap();
        backend.end().unwrap();

        assert_eq!(backend.bounds().len(), 4);
        assert!(backend.bounds()[0].right < backend.bounds()[1].left);
        assert!((backend.bounds()[2].right - 91.0).abs() < f64::EPSILON);
        assert!((backend.bounds()[3].left - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn geometry_records_the_same_converted_subprecision_points_as_svg() {
        let mut backend = begun_backend();
        backend
            .draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0006, 0.0006)],
                &LineStyle::solid((0, 0, 0), 2.0),
            )
            .unwrap();
        backend.end().unwrap();

        assert_eq!(
            backend.bounds(),
            &[GeometryBox {
                left: -1.0,
                right: 1.000_976_562_5,
                bottom: -1.0,
                top: 1.000_976_562_5,
                weight: 1.0,
            }]
        );
    }

    #[test]
    fn geometry_records_signed_text_without_unsigned_saturation() {
        let mut backend = begun_backend();
        backend
            .draw_text(Point2D::new(-7.0, 125.0), "A", &test_text_style())
            .unwrap();
        backend.end().unwrap();

        let bounds = backend.bounds()[0];
        assert!((f64::midpoint(bounds.left, bounds.right) + 7.0).abs() < f64::EPSILON);
        assert!(bounds.bottom > 100.0 && bounds.top > bounds.bottom);
    }

    #[test]
    fn text_anchors_and_rotation_follow_exact_planned_ink() {
        let mut backend = begun_backend();
        for (y, anchor) in [
            (20.0, TextAnchor::Start),
            (40.0, TextAnchor::Middle),
            (60.0, TextAnchor::End),
        ] {
            backend
                .draw_text(
                    Point2D::new(50.0, y),
                    "A",
                    &TextStyle {
                        anchor,
                        ..test_text_style()
                    },
                )
                .unwrap();
        }
        backend
            .draw_text(
                Point2D::new(50.0, 80.0),
                "A",
                &TextStyle {
                    rotation: 90.0,
                    ..test_text_style()
                },
            )
            .unwrap();
        backend.end().unwrap();

        let [start, middle, end, rotated] = backend.bounds() else {
            panic!("expected four exact text bounds");
        };
        let width = start.right - start.left;
        let height = start.top - start.bottom;
        assert!((start.left - 50.0).abs() < f64::EPSILON);
        assert!((f64::midpoint(middle.left, middle.right) - 50.0).abs() < f64::EPSILON);
        assert!((end.right - 50.0).abs() < f64::EPSILON);
        assert!((rotated.right - rotated.left - height).abs() < 1.0e-12);
        assert!((rotated.top - rotated.bottom - width).abs() < 1.0e-12);
    }

    #[test]
    fn planned_geometry_uses_stored_ink_without_replanning() {
        let plan = PreparedFontEnvironment::synthetic()
            .plan_text(TextPlanInput {
                source: "A",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 12,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Center,
                paint: (0, 0, 0).into(),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "planned geometry test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        let placement =
            PlannedTextPlacement::middle(Point2D::new(50.0, 50.0), TextAnchor::Middle, 0.0);
        GEOMETRY_RAW_TEXT_PLAN_CALLS.with(|calls| calls.set(0));
        let mut backend = begun_backend();

        backend.draw_planned_text(placement, &plan).unwrap();
        backend.end().unwrap();

        GEOMETRY_RAW_TEXT_PLAN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert_eq!(backend.bounds.len(), 1);
        assert_eq!(backend.text.len(), 1);
        assert!(backend.text[0].plan.shares_payload(&plan));
        assert_eq!(
            backend.bounds[0],
            planned_text_ink_bounds(
                &plan,
                placement.position,
                placement.anchor,
                placement.rotation,
                placement.origin,
                placement.reverse_lines,
            )
            .unwrap()
        );
        backend.begin(100, 100).unwrap();
        assert!(backend.text.is_empty());
        backend.end().unwrap();
    }

    #[test]
    fn geometry_and_svg_reject_the_same_collapsed_line() {
        let points = [Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)];
        let style = LineStyle::solid((0, 0, 0), 0.0);
        let mut geometry = begun_backend();
        let geometry_error = geometry.draw_line(&points, &style).unwrap_err();
        geometry.end().unwrap();
        let mut svg = crate::render::svg_backend::SvgBackend::in_memory(100, 100);
        svg.begin(100, 100).unwrap();
        let svg_error = svg.draw_line(&points, &style).unwrap_err();

        assert_eq!(geometry_error.to_string(), svg_error.to_string());
        assert!(geometry.bounds().is_empty());
    }

    #[test]
    fn geometry_and_svg_share_final_triangle_coordinates() {
        let points = [Point2D::new(10.0004, 20.0004)];
        let style = PointStyle::plain((0, 0, 0), 1).with_shape(PointShape::TriangleUp);
        let mut geometry = begun_backend();
        geometry.draw_scatter(&points, &style).unwrap();
        geometry.end().unwrap();
        let mut svg = crate::render::svg_backend::SvgBackend::in_memory(100, 100);
        svg.begin(100, 100).unwrap();
        svg.draw_scatter(&points, &style).unwrap();
        svg.end().unwrap();
        let output = String::from_utf8(svg.into_bytes().unwrap()).unwrap();

        assert_eq!(
            geometry.bounds(),
            &[GeometryBox {
                left: 9.134_765_625,
                right: 10.866_210_937_5,
                bottom: 19.5,
                top: 21.0,
                weight: 1.0,
            }]
        );
        assert!(output.contains("points=\"10,79 9.134765625,80.5 10.8662109375,80.5\""));
    }

    #[test]
    fn linear_gradient_records_its_rectangular_bounds() {
        let mut backend = begun_backend();
        let bounds = PanelBox::new(10.0, 22.0, 20.0, 80.0);

        backend
            .draw_linear_gradient(
                bounds,
                Point2D::new(16.0, 20.0),
                Point2D::new(16.0, 80.0),
                &[
                    GradientStop::new(0.0, (0, 0, 0)),
                    GradientStop::new(1.0, (255, 255, 255)),
                ],
            )
            .unwrap();
        backend.end().unwrap();

        assert_eq!(
            backend.bounds(),
            &[GeometryBox {
                left: 10.0,
                right: 22.0,
                bottom: 20.0,
                top: 80.0,
                weight: 1.0,
            }]
        );
    }

    #[test]
    fn brackets_record_label_overhang_in_both_orientations() {
        let style = BracketStyle {
            color: (0, 0, 0).into(),
            width: 1.0,
            label_color: (0, 0, 0).into(),
            font_family: "Test Sans".into(),
            font_size: 20,
            line_height: 24.0,
            weight: crate::TextWeight::Normal,
            label_gap: 5.0,
        };
        let mut backend = begun_backend();
        backend
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 49.0,
                    x_right: 51.0,
                    y: 80.0,
                    label: "A".into(),
                    orientation: BracketOrientation::Vertical,
                },
                &style,
            )
            .unwrap();
        backend
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 49.0,
                    x_right: 51.0,
                    y: 120.0,
                    label: "A".into(),
                    orientation: BracketOrientation::Horizontal,
                },
                &style,
            )
            .unwrap();
        backend.end().unwrap();

        assert!(backend.bounds[0].left < 49.0 && backend.bounds[0].right > 51.0);
        assert!(backend.bounds[0].top > 85.0);
        assert!(backend.bounds[1].left < 113.0 && backend.bounds[1].right > 125.0);
        assert!(backend.bounds[1].bottom < 50.0 && backend.bounds[1].top > 50.0);
    }
}
