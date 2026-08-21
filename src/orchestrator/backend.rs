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
    begun: bool,
}

impl<'a> DeferredBackend<'a> {
    pub(crate) fn new(backend: &'a mut dyn Backend, width: u32, height: u32) -> Self {
        Self {
            backend,
            width,
            height,
            begun: false,
        }
    }

    fn ensure_started(&mut self) -> Result<(), Error> {
        if !self.begun {
            self.backend.begin(self.width, self.height)?;
            self.begun = true;
        }
        Ok(())
    }

    pub(crate) fn finish<T>(mut self, rendered: Result<T, Error>, scope: &str) -> Result<T, Error> {
        if rendered.is_ok() && !self.begun {
            self.ensure_started()?;
        }
        if !self.begun {
            return rendered;
        }
        let ended = self.backend.end();
        backend::finish_scope(rendered, ended, scope)
    }
}

impl Backend for DeferredBackend<'_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if self.begun {
            return Err(Error::Render("backend canvas has already begun".into()));
        }
        self.backend.begin(width, height)?;
        self.begun = true;
        Ok(())
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

    fn draw_text(
        &mut self,
        x_pixel: u32,
        y_pixel: u32,
        text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_text(x_pixel, y_pixel, text, style)
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
        self.backend.end()
    }
}

pub(super) struct NullBackend;

impl Backend for NullBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        backend::validate_clip_region(region)
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        backend::validate_points(points, "scatter")?;
        backend::validate_point_style(style)
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        if points.len() < 2 {
            return Ok(());
        }
        backend::validate_points(points, "line")?;
        backend::validate_line_style(style)
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        if points.len() < 3 {
            return Ok(());
        }
        backend::validate_points(points, "polygon")?;
        backend::validate_fill_style(style)
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        backend::validate_linear_gradient(bounds, start, end, stops)
    }

    fn draw_text(
        &mut self,
        _x_pixel: u32,
        _y_pixel: u32,
        _text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        backend::validate_text_style(style)
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
        backend::validate_fill_style(style)
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        backend::validate_bracket(bracket, style)
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
