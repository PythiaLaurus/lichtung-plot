//! Deferred and validating backend implementations used by render orchestration.

use crate::Error;

use crate::layout::PanelBox;
use crate::render::backend::{
    self, Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle, Point2D,
    PointStyle, SignificanceBracket, TextStyle,
};

pub(crate) struct DeferredBackend<'a> {
    backend: &'a mut dyn Backend,
    width: u32,
    height: u32,
    state: DeferredState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeferredState {
    Fresh,
    Open,
    Closed,
}

impl<'a> DeferredBackend<'a> {
    pub(crate) fn new(backend: &'a mut dyn Backend, width: u32, height: u32) -> Self {
        Self {
            backend,
            width,
            height,
            state: DeferredState::Fresh,
        }
    }

    fn ensure_started(&mut self) -> Result<(), Error> {
        match self.state {
            DeferredState::Fresh => {
                self.state = DeferredState::Closed;
                self.backend.begin(self.width, self.height)?;
                self.state = DeferredState::Open;
                Ok(())
            }
            DeferredState::Open => Ok(()),
            DeferredState::Closed => Err(Error::Render(
                "deferred backend operation called after end".into(),
            )),
        }
    }

    fn end_open_canvas(&mut self) -> Result<(), Error> {
        debug_assert!(self.state == DeferredState::Open);
        self.state = DeferredState::Closed;
        self.backend.end()
    }

    pub(crate) fn finish<T>(mut self, rendered: Result<T, Error>, scope: &str) -> Result<T, Error> {
        if rendered.is_ok() && self.state == DeferredState::Fresh {
            self.ensure_started()?;
        }
        if self.state != DeferredState::Open {
            return rendered;
        }
        let ended = self.end_open_canvas();
        backend::finish_scope(rendered, ended, scope)
    }
}

impl Backend for DeferredBackend<'_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        match self.state {
            DeferredState::Open => {
                return Err(Error::Render("backend canvas has already begun".into()));
            }
            DeferredState::Closed => {
                return Err(Error::Render("backend canvas has already ended".into()));
            }
            DeferredState::Fresh => {}
        }
        if (width, height) != (self.width, self.height) {
            return Err(Error::Render(format!(
                "backend canvas dimensions must match deferred dimensions {}x{}; got {width}x{height}",
                self.width, self.height
            )));
        }
        self.ensure_started()
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.begin_layer(name)
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.begin_layer_with_role(name, role)
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.end_layer()
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.begin_clip(region)
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.end_clip()
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_scatter(points, style)
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_line(points, style)
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_polygon(points, style)
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_linear_gradient(bounds, start, end, stops)
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_text(position, text, style)
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
        self.ensure_started()?;
        self.backend
            .draw_rounded_rect(x, y, width, height, radius, style)
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_bracket(bracket, style)
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ensure_started()?;
        self.end_open_canvas()
    }
}

#[derive(Default)]
pub(super) struct NullBackend {
    state: NullCanvasState,
    scopes: Vec<NullScope>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum NullCanvasState {
    #[default]
    Fresh,
    Open,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NullScope {
    Layer,
    Clip,
}

impl NullBackend {
    fn ensure_open(&self, operation: &str) -> Result<(), Error> {
        if self.state == NullCanvasState::Open {
            Ok(())
        } else {
            Err(Error::Render(format!(
                "null backend {operation} called outside begin/end"
            )))
        }
    }
}

pub(super) fn validate_with_null_backend<T>(
    width: u32,
    height: u32,
    scope: &str,
    validate: impl FnOnce(&mut NullBackend) -> Result<T, Error>,
) -> Result<T, Error> {
    let mut backend = NullBackend::default();
    backend.begin(width, height)?;
    let validated = validate(&mut backend);
    let ended = backend.end();
    backend::finish_scope(validated, ended, scope)
}

impl Backend for NullBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if self.state == NullCanvasState::Open {
            return Err(Error::Render(
                "null backend begin called while a canvas is already open".into(),
            ));
        }
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "null backend canvas dimensions must be positive".into(),
            ));
        }
        self.scopes.clear();
        self.state = NullCanvasState::Open;
        Ok(())
    }

    fn begin_layer(&mut self, _name: &str) -> Result<(), Error> {
        self.ensure_open("begin_layer")?;
        self.scopes.push(NullScope::Layer);
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.ensure_open("end_layer")?;
        match self.scopes.last() {
            Some(NullScope::Layer) => {
                self.scopes.pop();
                Ok(())
            }
            Some(NullScope::Clip) => Err(Error::Render(
                "null backend end_layer called while a clip scope is active".into(),
            )),
            None => Err(Error::Render(
                "null backend end_layer called without a matching begin_layer".into(),
            )),
        }
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.ensure_open("begin_clip")?;
        backend::validate_clip_region(region)?;
        self.scopes.push(NullScope::Clip);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.ensure_open("end_clip")?;
        match self.scopes.last() {
            Some(NullScope::Clip) => {
                self.scopes.pop();
                Ok(())
            }
            Some(NullScope::Layer) => Err(Error::Render(
                "null backend end_clip called while a layer scope is active".into(),
            )),
            None => Err(Error::Render(
                "null backend end_clip called without a matching begin_clip".into(),
            )),
        }
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.ensure_open("draw_scatter")?;
        backend::validate_points(points, "scatter")?;
        backend::validate_point_style(style)
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_open("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        backend::validate_line_points(points)?;
        backend::validate_line_style(style)
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_open("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        backend::validate_polygon_points(points)?;
        backend::validate_fill_style(style)
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.ensure_open("draw_linear_gradient")?;
        backend::validate_linear_gradient(bounds, start, end, stops)?;
        Ok(())
    }

    fn draw_text(
        &mut self,
        position: Point2D,
        _text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_text")?;
        backend::validate_text(position, style)?;
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
        backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_bracket")?;
        backend::validate_bracket(bracket, style)?;
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ensure_open("end")?;
        if !self.scopes.is_empty() {
            return Err(Error::Render(format!(
                "null backend ended with {} unclosed layer/clip scopes",
                self.scopes.len()
            )));
        }
        self.state = NullCanvasState::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_backend_preserves_degenerate_line_and_polygon_noops() {
        let mut backend = NullBackend::default();
        let invalid_line = LineStyle::solid((0, 0, 0), f32::NAN);
        let invalid_fill = FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width: f32::NAN,
            opacity: f64::NAN,
        };

        backend.begin(100, 100).unwrap();
        assert!(backend
            .draw_line(&[Point2D::new(f64::NAN, f64::NAN)], &invalid_line)
            .is_ok());
        assert!(backend
            .draw_polygon(
                &[
                    Point2D::new(f64::NAN, f64::NAN),
                    Point2D::new(f64::NAN, f64::NAN),
                ],
                &invalid_fill,
            )
            .is_ok());
        backend.end().unwrap();
    }

    #[test]
    fn null_backend_enforces_canvas_lifecycle_and_allows_reuse() {
        let mut backend = NullBackend::default();
        let points = [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        let style = LineStyle::solid((0, 0, 0), 1.0);

        assert!(backend.begin(0, 100).is_err());
        assert!(backend.begin(100, 0).is_err());
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.end().is_err());

        backend.begin(100, 100).unwrap();
        assert!(backend.begin(100, 100).is_err());
        backend.draw_line(&points, &style).unwrap();
        backend.end().unwrap();
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.end().is_err());

        backend.begin(200, 150).unwrap();
        backend.end().unwrap();
    }

    #[test]
    fn null_backend_enforces_balanced_lifo_scopes() {
        let mut backend = NullBackend::default();
        backend.begin(100, 100).unwrap();

        assert!(backend.end_clip().is_err());
        assert!(backend.end_layer().is_err());
        backend.begin_layer("marks").unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 90.0, 10.0, 90.0))
            .unwrap();
        assert!(backend.end_layer().is_err());
        assert!(backend.end().is_err());
        backend.end_clip().unwrap();
        assert!(backend.end().is_err());
        backend.end_layer().unwrap();
        backend.end().unwrap();
    }
}
