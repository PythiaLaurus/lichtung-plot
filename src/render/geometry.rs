//! Validating backend that records conservative bounds for every primitive.

use crate::Error;

use crate::layout::{estimated_text_width, PanelBox};
use crate::render::backend::{
    self, Backend, BracketOrientation, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D,
    PointStyle, SignificanceBracket, TextAnchor, TextStyle, SIGNIFICANCE_BRACKET_METRICS,
};

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
        backend::validate_clip_region(region)?;
        self.clips.push(region);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clips
            .pop()
            .ok_or_else(|| Error::Render("geometry backend clip scope is unbalanced".into()))?;
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        backend::validate_points(points, "scatter")?;
        backend::validate_point_style(style)?;
        for point in points {
            self.record(GeometryBox::from_points(
                std::slice::from_ref(point),
                f64::from(style.radius),
            ));
        }
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        if points.len() < 2 {
            return Ok(());
        }
        backend::validate_points(points, "line")?;
        backend::validate_line_style(style)?;
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
        backend::validate_points(points, "polygon")?;
        backend::validate_fill_style(style)?;
        self.record(GeometryBox::from_points(
            points,
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
        backend::validate_linear_gradient(bounds, start, end, stops)?;
        self.record(Some(GeometryBox {
            left: bounds.x0,
            right: bounds.x1,
            bottom: bounds.y0,
            top: bounds.y1,
            weight: 1.0,
        }));
        Ok(())
    }

    fn draw_text(&mut self, x: u32, y: u32, text: &str, style: &TextStyle) -> Result<(), Error> {
        backend::validate_text_style(style)?;
        let width = estimated_text_width(text, style.font_size);
        let height = f64::from(style.font_size) * 1.2;
        let x = f64::from(x);
        let y = f64::from(y);
        let left = match style.anchor {
            TextAnchor::Start => x,
            TextAnchor::Middle => x - width / 2.0,
            TextAnchor::End => x - width,
        };
        if style.rotation == 0.0 {
            self.record(Some(GeometryBox {
                left,
                right: left + width,
                bottom: y - height / 2.0,
                top: y + height / 2.0,
                weight: ANNOTATION_COLLISION_WEIGHT,
            }));
        } else {
            let radians = style.rotation.to_radians();
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
        backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
        let padding = f64::from(style.stroke_width) / 2.0;
        self.record(Some(GeometryBox {
            left: x - padding,
            right: x + width + padding,
            bottom: y - padding,
            top: y + height + padding,
            weight: 1.0,
        }));
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        backend::validate_bracket(bracket, style)?;
        let metrics = SIGNIFICANCE_BRACKET_METRICS;
        let label_width = estimated_text_width(&bracket.label, metrics.label_font_size);
        let bounds = match bracket.orientation {
            BracketOrientation::Vertical => GeometryBox {
                left: bracket.x_left.min(bracket.x_right),
                right: bracket.x_left.max(bracket.x_right),
                bottom: bracket.y - metrics.tick_height,
                top: bracket.y + metrics.label_gap + f64::from(metrics.label_font_size),
                weight: ANNOTATION_COLLISION_WEIGHT,
            },
            BracketOrientation::Horizontal => GeometryBox {
                left: bracket.y - metrics.tick_height,
                right: bracket.y + metrics.label_gap + label_width,
                bottom: bracket.x_left.min(bracket.x_right),
                top: bracket.x_left.max(bracket.x_right),
                weight: ANNOTATION_COLLISION_WEIGHT,
            },
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
}
