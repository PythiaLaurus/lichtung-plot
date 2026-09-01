//! Validating backend that records conservative bounds for every primitive.

use std::collections::HashMap;

use crate::Error;

use crate::layout::{estimated_text_width, PanelBox};
use crate::render::backend::{
    self, Backend, BracketOrientation, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D,
    PointStyle, Rgba, SignificanceBracket, SurfaceRole, TextAnchor, TextStyle, POLYGON_MITER_LIMIT,
};
use crate::render::coordinates::CanonicalPoint;
use crate::PointShape;

// Labels and annotation glyphs are less tolerable legend targets than translucent marks.
const ANNOTATION_COLLISION_WEIGHT: f64 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GeometryBox {
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
    pub(crate) top: f64,
    pub(crate) weight: f64,
}

impl GeometryBox {
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
    bounds: Vec<GeometryBox>,
    clips: Vec<PanelBox>,
    scopes: Vec<GeometryScope>,
    state: GeometryCanvasState,
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

impl Default for GeometryBackend {
    fn default() -> Self {
        Self {
            bounds: Vec::new(),
            clips: Vec::new(),
            scopes: Vec::new(),
            state: GeometryCanvasState::Fresh,
        }
    }
}

impl GeometryBackend {
    pub(crate) fn bounds(&self) -> &[GeometryBox] {
        &self.bounds
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

fn polygon_has_positive_area(points: &[CanonicalPoint]) -> bool {
    if points.len() < 3 {
        return false;
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
        let dx = end.x.units() - start.x.units();
        let dy = end.y.units() - start.y.units();
        if dx == 0 && dy == 0 {
            continue;
        }
        let divisor = i128::from(greatest_common_divisor(
            dx.unsigned_abs(),
            dy.unsigned_abs(),
        ));
        let raw_x = i128::from(dx) / divisor;
        let raw_y = i128::from(dy) / divisor;
        let forward = raw_x > 0 || (raw_x == 0 && raw_y > 0);
        let (direction_x, direction_y, direction) = if forward {
            (raw_x, raw_y, 1_i32)
        } else {
            (-raw_x, -raw_y, -1_i32)
        };
        let line =
            direction_y * i128::from(start.x.units()) - direction_x * i128::from(start.y.units());
        let start_projection =
            direction_x * i128::from(start.x.units()) + direction_y * i128::from(start.y.units());
        let end_projection =
            direction_x * i128::from(end.x.units()) + direction_y * i128::from(end.y.units());
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

    line_events.into_values().any(|mut events| {
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
    })
}

fn greatest_common_divisor(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn cross(first: Point2D, second: Point2D) -> f64 {
    first.x * second.y - first.y * second.x
}

fn polygon_stroke_bounds(points: &[Point2D], stroke_width: f64) -> Option<GeometryBox> {
    if stroke_width == 0.0 || points.len() < 2 {
        return None;
    }
    let half_width = stroke_width / 2.0;
    let mut stroke_points = Vec::with_capacity(points.len() * 5);
    let mut directions = Vec::with_capacity(points.len());
    for (start, end) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = dx.hypot(dy);
        if length == 0.0 {
            directions.push(None);
            continue;
        }
        let direction = Point2D::new(dx / length, dy / length);
        let normal = Point2D::new(-direction.y * half_width, direction.x * half_width);
        stroke_points.extend([
            Point2D::new(start.x + normal.x, start.y + normal.y),
            Point2D::new(start.x - normal.x, start.y - normal.y),
            Point2D::new(end.x + normal.x, end.y + normal.y),
            Point2D::new(end.x - normal.x, end.y - normal.y),
        ]);
        directions.push(Some(direction));
    }

    for index in 0..points.len() {
        let Some(incoming) = directions[(index + points.len() - 1) % points.len()] else {
            continue;
        };
        let Some(outgoing) = directions[index] else {
            continue;
        };
        let turn = cross(incoming, outgoing);
        let denominator = 1.0 + incoming.x * outgoing.x + incoming.y * outgoing.y;
        if turn == 0.0 || denominator <= f64::EPSILON {
            continue;
        }
        let incoming_normal = Point2D::new(-incoming.y, incoming.x);
        let outgoing_normal = Point2D::new(-outgoing.y, outgoing.x);
        let outer_sign = if turn > 0.0 { -1.0 } else { 1.0 };
        let offset = Point2D::new(
            outer_sign * half_width * (incoming_normal.x + outgoing_normal.x) / denominator,
            outer_sign * half_width * (incoming_normal.y + outgoing_normal.y) / denominator,
        );
        let miter_ratio = offset.x.hypot(offset.y) / half_width;
        if miter_ratio <= POLYGON_MITER_LIMIT {
            stroke_points.push(Point2D::new(
                points[index].x + offset.x,
                points[index].y + offset.y,
            ));
        }
    }

    GeometryBox::from_points(&stroke_points, 0.0)
}

impl Backend for GeometryBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
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
        let canonical_points = backend::validate_points(points, "scatter")?;
        backend::validate_point_style(style)?;
        match style.shape {
            PointShape::Circle => {
                for point in canonical_points {
                    let point = point.value();
                    self.record(GeometryBox::from_points(
                        std::slice::from_ref(&point),
                        f64::from(style.radius),
                    ));
                }
            }
            PointShape::TriangleUp => {
                let outline_padding = style
                    .outline
                    .map_or(0.0, |outline| outline.width_for_radius(style.radius) / 2.0);
                for triangle in backend::validate_triangle_points(points, style.radius)? {
                    let triangle = triangle.map(super::coordinates::CanonicalPoint::value);
                    self.record(GeometryBox::from_points(&triangle, outline_padding));
                }
            }
        }
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_open("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        let points = backend::validate_line_points(points)?;
        backend::validate_line_style(style)?;
        let points = points
            .into_iter()
            .map(super::coordinates::CanonicalPoint::value)
            .collect::<Vec<_>>();
        if style.width == 0.0 {
            return Ok(());
        }
        for segment in points.windows(2) {
            self.record(GeometryBox::from_points(
                segment,
                f64::from(style.width) / 2.0,
            ));
        }
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_open("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        let points = backend::validate_polygon_points(points)?;
        backend::validate_fill_style(style)?;
        let ring = polygon_ring(&points);
        let point_values = ring
            .iter()
            .copied()
            .map(super::coordinates::CanonicalPoint::value)
            .collect::<Vec<_>>();
        let fill_bounds = (style.opacity > 0.0 && polygon_has_positive_area(&ring))
            .then(|| GeometryBox::from_points(&point_values, 0.0))
            .flatten();
        let stroke_bounds = polygon_stroke_bounds(&point_values, f64::from(style.stroke_width));
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
        let width = estimated_text_width(text, style.font_size);
        let height = f64::from(style.font_size) * 1.2;
        let x = canonical.position.x.value();
        let y = canonical.position.y.value();
        let left = match style.anchor {
            TextAnchor::Start => x,
            TextAnchor::Middle => x - width / 2.0,
            TextAnchor::End => x - width,
        };
        let rotation = canonical.rotation.value();
        if rotation == 0.0 {
            self.record(Some(GeometryBox {
                left,
                right: left + width,
                bottom: y - height / 2.0,
                top: y + height / 2.0,
                weight: ANNOTATION_COLLISION_WEIGHT,
            }));
        } else {
            let radians = rotation.to_radians();
            let projected_width = width * radians.cos().abs() + height * radians.sin().abs();
            let projected_height = width * radians.sin().abs() + height * radians.cos().abs();
            self.record(Some(GeometryBox {
                left: x - projected_width / 2.0,
                right: x + projected_width / 2.0,
                bottom: y - projected_height / 2.0,
                top: y + projected_height / 2.0,
                weight: ANNOTATION_COLLISION_WEIGHT,
            }));
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
        let padding = f64::from(style.stroke_width) / 2.0;
        self.record(Some(GeometryBox {
            left: rounded.rect.left.value() - padding,
            right: rounded.rect.right.value() + padding,
            bottom: rounded.rect.bottom.value() - padding,
            top: rounded.rect.top.value() + padding,
            weight: 1.0,
        }));
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_bracket")?;
        let canonical = backend::validate_bracket(bracket, style)?;
        let label_width = estimated_text_width(&bracket.label, style.font_size);
        let line_start = canonical.line_start.value();
        let line_end = canonical.line_end.value();
        let first_tick_end = canonical.first_tick_end.value();
        let second_tick_end = canonical.second_tick_end.value();
        let label_position = canonical.label_position.value();
        let bounds = match canonical.orientation {
            BracketOrientation::Vertical => GeometryBox {
                left: line_start
                    .x
                    .min(line_end.x)
                    .min(label_position.x - label_width / 2.0),
                right: line_start
                    .x
                    .max(line_end.x)
                    .max(label_position.x + label_width / 2.0),
                bottom: first_tick_end.y.min(second_tick_end.y),
                top: label_position.y + style.line_height,
                weight: ANNOTATION_COLLISION_WEIGHT,
            },
            BracketOrientation::Horizontal => {
                let label_half_height = style.line_height / 2.0;
                GeometryBox {
                    left: first_tick_end.x.min(second_tick_end.x),
                    right: label_position.x + label_width,
                    bottom: line_start
                        .y
                        .min(line_end.y)
                        .min(label_position.y - label_half_height),
                    top: line_start
                        .y
                        .max(line_end.y)
                        .max(label_position.y + label_half_height),
                    weight: ANNOTATION_COLLISION_WEIGHT,
                }
            }
        };
        self.record(Some(bounds));
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

    fn test_text_style() -> TextStyle {
        TextStyle {
            color: (0, 0, 0).into(),
            font_family: "sans-serif".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        }
    }

    fn begun_backend() -> GeometryBackend {
        let mut backend = GeometryBackend::default();
        backend.begin(100, 100).unwrap();
        backend
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
        polygon_has_positive_area(&polygon_ring(&points))
    }

    #[test]
    fn geometry_lifecycle_requires_one_balanced_session_and_fresh_reuse() {
        let points = [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        let style = LineStyle::solid((0, 0, 0), 1.0);
        let mut backend = GeometryBackend::default();

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

        assert!((admitted.right - 13.0).abs() < 1.0e-12);
        assert!((limited.right - (12.0 + 5.0 / 13.0)).abs() < 1.0e-12);
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
                right: 1.001,
                bottom: -1.0,
                top: 1.001,
                weight: 1.0,
            }]
        );
    }

    #[test]
    fn geometry_records_signed_text_without_unsigned_saturation() {
        let mut backend = begun_backend();
        backend
            .draw_text(Point2D::new(-7.0, 125.0), "", &test_text_style())
            .unwrap();
        backend.end().unwrap();

        let bounds = backend.bounds()[0];
        assert!((f64::midpoint(bounds.left, bounds.right) + 7.0).abs() < f64::EPSILON);
        assert!((f64::midpoint(bounds.bottom, bounds.top) - 125.0).abs() < f64::EPSILON);
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
                left: 9.134,
                right: 10.866,
                bottom: 19.5,
                top: 21.0,
                weight: 1.0,
            }]
        );
        assert!(output.contains("points=\"10.000,79.000 9.134,80.500 10.866,80.500\""));
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
            font_family: "sans-serif".into(),
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
                    label: "****".into(),
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
                    label: "****".into(),
                    orientation: BracketOrientation::Horizontal,
                },
                &style,
            )
            .unwrap();
        backend.end().unwrap();

        assert!(backend.bounds[0].left < 49.0 && backend.bounds[0].right > 51.0);
        assert!((backend.bounds[0].top - 109.0).abs() < f64::EPSILON);
        assert!((backend.bounds[1].bottom - 38.0).abs() < f64::EPSILON);
        assert!((backend.bounds[1].top - 62.0).abs() < f64::EPSILON);
    }
}
