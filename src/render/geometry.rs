//! Validating backend that records conservative bounds for every primitive.

use crate::Error;

use crate::layout::{estimated_text_width, PanelBox};
use crate::render::backend::{
    self, Backend, BracketOrientation, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D,
    PointStyle, SignificanceBracket, TextAnchor, TextStyle,
};
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
        (bounds.left <= bounds.right && bounds.bottom <= bounds.top).then_some(bounds)
    }

    pub(crate) fn overlap_area(self, other: Self, margin: f64) -> f64 {
        let width =
            (self.right.min(other.right + margin) - self.left.max(other.left - margin)).max(0.0);
        let height =
            (self.top.min(other.top + margin) - self.bottom.max(other.bottom - margin)).max(0.0);
        width * height * self.weight
    }
}

#[derive(Default)]
pub(crate) struct GeometryBackend {
    bounds: Vec<GeometryBox>,
    clips: Vec<PanelBox>,
}

impl GeometryBackend {
    pub(crate) fn bounds(&self) -> &[GeometryBox] {
        &self.bounds
    }

    fn record(&mut self, bounds: Option<GeometryBox>) {
        if let Some(bounds) = bounds.and_then(|bounds| bounds.clipped(self.clips.last().copied())) {
            self.bounds.push(bounds);
        }
    }
}

impl Backend for GeometryBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        let region = backend::validate_clip_region(region)?;
        self.clips.push(region.panel());
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clips
            .pop()
            .ok_or_else(|| Error::Render("geometry backend clip scope is unbalanced".into()))?;
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
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
        if points.len() < 2 {
            return Ok(());
        }
        let points = backend::validate_line_points(points)?;
        backend::validate_line_style(style)?;
        let points = points
            .into_iter()
            .map(super::coordinates::CanonicalPoint::value)
            .collect::<Vec<_>>();
        for segment in points.windows(2) {
            self.record(GeometryBox::from_points(
                segment,
                f64::from(style.width) / 2.0,
            ));
        }
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        if points.len() < 3 {
            return Ok(());
        }
        let points = backend::validate_polygon_points(points)?;
        backend::validate_fill_style(style)?;
        let points = points
            .into_iter()
            .map(super::coordinates::CanonicalPoint::value)
            .collect::<Vec<_>>();
        self.record(GeometryBox::from_points(
            &points,
            f64::from(style.stroke_width) / 2.0,
        ));
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
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
        let rounded = backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
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
                top: label_position.y + f64::from(style.font_size),
                weight: ANNOTATION_COLLISION_WEIGHT,
            },
            BracketOrientation::Horizontal => {
                let label_half_height = f64::from(style.font_size) / 2.0;
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_text_style() -> TextStyle {
        TextStyle {
            color: (0, 0, 0),
            font_family: "sans-serif".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        }
    }

    #[test]
    fn scatter_and_polyline_record_individual_local_obstacles() {
        let mut backend = GeometryBackend::default();
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

        assert_eq!(backend.bounds().len(), 4);
        assert!(backend.bounds()[0].right < backend.bounds()[1].left);
        assert!((backend.bounds()[2].right - 91.0).abs() < f64::EPSILON);
        assert!((backend.bounds()[3].left - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn geometry_records_the_same_converted_subprecision_points_as_svg() {
        let mut backend = GeometryBackend::default();
        backend
            .draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0006, 0.0006)],
                &LineStyle::solid((0, 0, 0), 0.0),
            )
            .unwrap();

        assert_eq!(
            backend.bounds(),
            &[GeometryBox {
                left: 0.0,
                right: 0.001,
                bottom: 0.0,
                top: 0.001,
                weight: 1.0,
            }]
        );
    }

    #[test]
    fn geometry_records_signed_text_without_unsigned_saturation() {
        let mut backend = GeometryBackend::default();
        backend
            .draw_text(Point2D::new(-7.0, 125.0), "", &test_text_style())
            .unwrap();

        let bounds = backend.bounds()[0];
        assert!((f64::midpoint(bounds.left, bounds.right) + 7.0).abs() < f64::EPSILON);
        assert!((f64::midpoint(bounds.bottom, bounds.top) - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn geometry_and_svg_reject_the_same_collapsed_line() {
        let points = [Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)];
        let style = LineStyle::solid((0, 0, 0), 0.0);
        let mut geometry = GeometryBackend::default();
        let geometry_error = geometry.draw_line(&points, &style).unwrap_err();
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
        let mut geometry = GeometryBackend::default();
        geometry.draw_scatter(&points, &style).unwrap();
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
        let mut backend = GeometryBackend::default();
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
            color: (0, 0, 0),
            width: 1.0,
            font_family: "sans-serif".into(),
            font_size: 20,
            weight: crate::TextWeight::Normal,
            label_gap: 5.0,
        };
        let mut backend = GeometryBackend::default();
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

        assert!(backend.bounds[0].left < 49.0 && backend.bounds[0].right > 51.0);
        assert!(backend.bounds[1].bottom < 49.0 && backend.bounds[1].top > 51.0);
    }
}
