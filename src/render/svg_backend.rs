//! Native SVG backend for deterministic publication-oriented output.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::Error;

use super::backend::{
    validate_bracket, validate_clip_region, validate_fill_style, validate_line_style,
    validate_linear_gradient, validate_point_style, validate_points, validate_rounded_rect,
    validate_text_style, Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle,
    Point2D, PointStyle, SignificanceBracket, TextAnchor, TextStyle,
};
use crate::layout::PanelBox;
#[cfg(feature = "png")]
use crate::spec::{PngCompression, PngOptions, PngSizing, SVG_CSS_DPI};
use crate::theme::rgb_to_hex;
use crate::{PointShape, DEFAULT_FONT_FAMILY_STACK};

impl SvgBackend {
    /// Create an SVG backend that writes to `path`.
    #[must_use]
    pub fn new(path: PathBuf, width: u32, height: u32) -> Self {
        Self {
            destination: SvgDestination::File(path),
            width,
            height,
            body: String::new(),
            grid_buf: String::new(),
            clip_defs: String::new(),
            gradient_defs: String::new(),
            group_stack: Vec::new(),
            next_clip_id: 0,
            next_gradient_id: 0,
            begun: false,
        }
    }

    pub(crate) fn in_memory(width: u32, height: u32) -> Self {
        Self {
            destination: SvgDestination::Memory(None),
            width,
            height,
            body: String::new(),
            grid_buf: String::new(),
            clip_defs: String::new(),
            gradient_defs: String::new(),
            group_stack: Vec::new(),
            next_clip_id: 0,
            next_gradient_id: 0,
            begun: false,
        }
    }

    pub(crate) fn into_bytes(self) -> Result<Vec<u8>, Error> {
        match self.destination {
            SvgDestination::Memory(Some(bytes)) => Ok(bytes),
            SvgDestination::Memory(None) => Err(Error::Render(
                "in-memory SVG backend did not finish rendering".into(),
            )),
            SvgDestination::File(_) => Err(Error::Render(
                "file SVG backend cannot return in-memory bytes".into(),
            )),
        }
    }

    fn y_mark(&self, y: f64) -> f64 {
        f64::from(self.height) - y
    }

    /// Returns a reference to the active output buffer (grid or main body).
    fn buf(&mut self) -> &mut String {
        if self
            .group_stack
            .iter()
            .any(|state| matches!(state, GroupState::Grid))
        {
            &mut self.grid_buf
        } else {
            &mut self.body
        }
    }

    fn fmt_points(&self, points: &[Point2D]) -> String {
        let mut out = String::new();
        for (i, p) in points.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let _ = write!(out, "{:.3},{:.3}", p.x, self.y_mark(p.y));
        }
        out
    }

    fn escape_text(text: &str) -> String {
        text.chars()
            .flat_map(|ch| match ch {
                '&' => "&amp;".chars().collect::<Vec<_>>(),
                '<' => "&lt;".chars().collect(),
                '>' => "&gt;".chars().collect(),
                '"' => "&quot;".chars().collect(),
                '\'' => "&apos;".chars().collect(),
                _ => vec![ch],
            })
            .collect()
    }

    fn ensure_begun(&self, operation: &str) -> Result<(), Error> {
        if self.begun {
            Ok(())
        } else {
            Err(Error::Render(format!(
                "svg backend {operation} called outside begin/end"
            )))
        }
    }
}

enum SvgDestination {
    File(PathBuf),
    Memory(Option<Vec<u8>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupState {
    Grid,
    Layer,
    Clip,
}

/// Convert canonical SVG scene text to vector paths using installed fonts.
#[cfg(feature = "text-as-path")]
pub(crate) fn convert_svg_text_to_paths(
    svg_data: &[u8],
    width_css_px: u32,
    height_css_px: u32,
) -> Result<Vec<u8>, Error> {
    let mut options = resvg::usvg::Options::default();
    crate::font::configure_system_fonts(options.fontdb_mut())?;

    let tree = resvg::usvg::Tree::from_data(svg_data, &options)
        .map_err(|error| Error::Render(format!("parse svg for text outlining: {error}")))?;
    let source_size = tree.size().to_int_size();
    if source_size.width() != width_css_px || source_size.height() != height_css_px {
        return Err(Error::Render(
            "rendered SVG scene dimensions do not match its canonical CSS dimensions".into(),
        ));
    }
    let outlined = tree.to_string(&resvg::usvg::WriteOptions::default());
    if outlined.contains("<text") {
        return Err(Error::Render(
            "SVG text outlining left text elements in the converted scene".into(),
        ));
    }
    Ok(outlined.into_bytes())
}

/// Rasterize canonical SVG scene bytes to PNG bytes with explicit export options.
#[cfg(feature = "png")]
pub(crate) fn rasterize_svg_bytes_to_png(
    svg_data: &[u8],
    width_css_px: u32,
    height_css_px: u32,
    png_options: &PngOptions,
) -> Result<Vec<u8>, Error> {
    validate_png_options(png_options)?;
    validate_png_target_for_source(width_css_px, height_css_px, png_options.sizing)?;

    let mut options = resvg::usvg::Options::default();
    crate::font::configure_system_fonts(options.fontdb_mut())?;

    let tree = resvg::usvg::Tree::from_data(svg_data, &options)
        .map_err(|e| Error::Render(format!("parse svg for png rasterization: {e}")))?;
    let source_size = tree.size().to_int_size();
    if source_size.width() != width_css_px || source_size.height() != height_css_px {
        return Err(Error::Render(
            "rendered SVG scene dimensions do not match its canonical CSS dimensions".into(),
        ));
    }
    let size = png_target_size(source_size, png_options.sizing)?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| Error::Render("allocate png pixmap".into()))?;
    let transform = png_transform(source_size, size);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    encode_png(pixmap, png_options)
}

#[cfg(feature = "png")]
fn validate_png_options(options: &PngOptions) -> Result<(), Error> {
    match options.sizing {
        PngSizing::Source => Ok(()),
        PngSizing::Dpi(dpi) => ensure_positive_finite(f64::from(dpi), "png dpi"),
        PngSizing::Scale(scale) => ensure_positive_finite(f64::from(scale), "png scale"),
        PngSizing::Width(width) => ensure_positive_dimension(width, "png width"),
        PngSizing::Height(height) => ensure_positive_dimension(height, "png height"),
    }
}

#[cfg(feature = "png")]
fn ensure_positive_finite(value: f64, label: &str) -> Result<(), Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "{label} must be positive and finite; got {value}"
        )))
    }
}

#[cfg(feature = "png")]
fn ensure_positive_dimension(value: u32, label: &str) -> Result<(), Error> {
    if value > 0 {
        Ok(())
    } else {
        Err(Error::Render(format!("{label} must be positive")))
    }
}

#[cfg(feature = "png")]
fn png_target_size(
    source_size: resvg::tiny_skia::IntSize,
    sizing: PngSizing,
) -> Result<resvg::tiny_skia::IntSize, Error> {
    validate_png_target_for_source(source_size.width(), source_size.height(), sizing)?;
    let size = match sizing {
        PngSizing::Source => Some(source_size),
        PngSizing::Dpi(dpi) => source_size.scale_by(dpi / SVG_CSS_DPI),
        PngSizing::Scale(scale) => source_size.scale_by(scale),
        PngSizing::Width(width) => source_size.scale_to_width(width),
        PngSizing::Height(height) => source_size.scale_to_height(height),
    };
    let size = size.ok_or_else(|| Error::Render("computed zero-sized png output".into()))?;
    crate::scene::validate_png_target_dimensions(
        f64::from(size.width()),
        f64::from(size.height()),
    )?;
    Ok(size)
}

#[cfg(feature = "png")]
fn validate_png_target_for_source(
    source_width: u32,
    source_height: u32,
    sizing: PngSizing,
) -> Result<(), Error> {
    let source_width = f64::from(source_width);
    let source_height = f64::from(source_height);
    let (target_width, target_height) = match sizing {
        PngSizing::Source => (source_width, source_height),
        PngSizing::Dpi(dpi) => {
            let scale = f64::from(dpi / SVG_CSS_DPI);
            (source_width * scale, source_height * scale)
        }
        PngSizing::Scale(scale) => {
            let scale = f64::from(scale);
            (source_width * scale, source_height * scale)
        }
        PngSizing::Width(width) => {
            let width = f64::from(width);
            (width, source_height * width / source_width)
        }
        PngSizing::Height(height) => {
            let height = f64::from(height);
            (source_width * height / source_height, height)
        }
    };
    crate::scene::validate_png_target_dimensions(target_width, target_height)
}

#[cfg(feature = "png")]
fn png_transform(
    source_size: resvg::tiny_skia::IntSize,
    target_size: resvg::tiny_skia::IntSize,
) -> resvg::tiny_skia::Transform {
    let source = source_size.to_size();
    let target = target_size.to_size();
    resvg::tiny_skia::Transform::from_scale(
        target.width() / source.width(),
        target.height() / source.height(),
    )
}

#[cfg(feature = "png")]
fn encode_png(pixmap: resvg::tiny_skia::Pixmap, options: &PngOptions) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, pixmap.width(), pixmap.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(map_png_compression(options.compression));
        if let PngSizing::Dpi(dpi) = options.sizing {
            encoder.set_pixel_dims(Some(png::PixelDimensions {
                xppu: dpi_to_pixels_per_meter(dpi)?,
                yppu: dpi_to_pixels_per_meter(dpi)?,
                unit: png::Unit::Meter,
            }));
        }

        let mut writer = encoder
            .write_header()
            .map_err(|e| Error::Render(format!("write png header: {e}")))?;
        writer
            .write_image_data(&pixmap.take_demultiplied())
            .map_err(|e| Error::Render(format!("write png data: {e}")))?;
        writer
            .finish()
            .map_err(|e| Error::Render(format!("finish png: {e}")))?;
    }
    Ok(bytes)
}

#[cfg(feature = "png")]
fn map_png_compression(compression: PngCompression) -> png::Compression {
    match compression {
        PngCompression::Default | PngCompression::Balanced => png::Compression::Balanced,
        PngCompression::None => png::Compression::NoCompression,
        PngCompression::Fast => png::Compression::Fast,
        PngCompression::High => png::Compression::High,
    }
}

#[cfg(feature = "png")]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "DPI metadata is range-checked before rounding to PNG pixels-per-meter"
)]
fn dpi_to_pixels_per_meter(dpi: f32) -> Result<u32, Error> {
    let pixels_per_meter = f64::from(dpi) / 0.0254;
    if !(pixels_per_meter.is_finite() && pixels_per_meter > 0.0) {
        return Err(Error::Render(format!(
            "png dpi must be positive and finite; got {dpi}"
        )));
    }
    if pixels_per_meter > f64::from(u32::MAX) {
        return Err(Error::Render(format!(
            "png dpi is too large to encode: {dpi}"
        )));
    }
    Ok(pixels_per_meter.round() as u32)
}

/// SVG-backed `Backend` implementation.
///
/// Plot modules currently pass geometry in the same bottom-up
/// coordinate convention used by the original Plotters shim. This
/// backend converts those marks to SVG's top-down coordinate system
/// at the final drawing boundary. Marks, axes, text, and clips all use
/// the same math-style canvas coordinates at the trait boundary.
///
/// Supports balanced nested layer and clip scopes which emit `<g>` groups.
/// Top-level layers carrying [`LayerRole::Grid`] are buffered separately and
/// emitted before all other content at `end()` time, ensuring grid lines
/// always appear behind data marks without constraining their unique IDs.
pub struct SvgBackend {
    destination: SvgDestination,
    width: u32,
    height: u32,
    body: String,
    /// Buffered grid layer content (emitted before body at end).
    grid_buf: String,
    clip_defs: String,
    gradient_defs: String,
    group_stack: Vec<GroupState>,
    next_clip_id: usize,
    next_gradient_id: usize,
    begun: bool,
}

impl Backend for SvgBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if self.begun {
            return Err(Error::Render(
                "svg backend begin called while a canvas is already open".into(),
            ));
        }
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "svg backend canvas dimensions must be positive".into(),
            ));
        }
        self.width = width;
        self.height = height;
        self.body.clear();
        self.grid_buf.clear();
        self.clip_defs.clear();
        self.gradient_defs.clear();
        self.group_stack.clear();
        self.next_clip_id = 0;
        self.next_gradient_id = 0;
        self.begun = true;
        Ok(())
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.begin_layer_with_role(name, LayerRole::Generic)
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.ensure_begun("begin_layer")?;
        if role == LayerRole::Grid {
            if !self.group_stack.is_empty() {
                return Err(Error::Render(
                    "svg grid layer must begin at the top level".into(),
                ));
            }
            let escaped = Self::escape_text(name);
            let _ = writeln!(self.grid_buf, "<g id=\"{escaped}\">");
            self.group_stack.push(GroupState::Grid);
        } else {
            let escaped = Self::escape_text(name);
            let _ = writeln!(self.buf(), "<g id=\"{escaped}\">");
            self.group_stack.push(GroupState::Layer);
        }
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.ensure_begun("end_layer")?;
        match self.group_stack.last() {
            Some(GroupState::Grid) => {
                let _ = writeln!(self.grid_buf, "</g>");
                self.group_stack.pop();
            }
            Some(GroupState::Layer) => {
                let _ = writeln!(self.buf(), "</g>");
                self.group_stack.pop();
            }
            Some(GroupState::Clip) => {
                return Err(Error::Render(
                    "svg end_layer called while a clip scope is active".into(),
                ));
            }
            None => {
                return Err(Error::Render(
                    "svg end_layer called without a matching begin_layer".into(),
                ));
            }
        }
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.ensure_begun("begin_clip")?;
        validate_clip_region(region)?;
        let id = self.next_clip_id;
        self.next_clip_id += 1;
        let svg_y = f64::from(self.height) - region.y1;
        let width = region.width();
        let height = region.height();
        let _ = writeln!(
            self.clip_defs,
            "<clipPath id=\"clip-{id}\"><rect x=\"{:.3}\" y=\"{svg_y:.3}\" width=\"{width:.3}\" height=\"{height:.3}\"/></clipPath>",
            region.x0,
        );
        let _ = writeln!(self.buf(), "<g clip-path=\"url(#clip-{id})\">");
        self.group_stack.push(GroupState::Clip);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.ensure_begun("end_clip")?;
        match self.group_stack.last() {
            Some(GroupState::Clip) => {
                let _ = writeln!(self.buf(), "</g>");
                self.group_stack.pop();
                Ok(())
            }
            Some(GroupState::Grid | GroupState::Layer) => Err(Error::Render(
                "svg end_clip called while a layer scope is active".into(),
            )),
            None => Err(Error::Render(
                "svg end_clip called without a matching begin_clip".into(),
            )),
        }
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.ensure_begun("draw_scatter")?;
        validate_points(points, "scatter")?;
        validate_point_style(style)?;
        let color = rgb_to_hex(style.color);
        let outline = style.outline.map(|outline| {
            (
                rgb_to_hex(outline.color),
                outline.width_for_radius(style.radius),
            )
        });
        let h = f64::from(self.height);
        match style.shape {
            PointShape::Circle => {
                let buf = self.buf();
                for p in points {
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{}\" fill=\"{color}\" fill-opacity=\"{}\" stroke=\"{outline_color}\" stroke-width=\"{outline_width:.1}\"/>",
                            p.x,
                            h - p.y,
                            style.radius,
                            style.opacity
                        );
                    } else {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{}\" fill=\"{color}\" fill-opacity=\"{}\"/>",
                            p.x,
                            h - p.y,
                            style.radius,
                            style.opacity
                        );
                    }
                }
            }
            PointShape::TriangleUp => {
                let radius = f64::from(style.radius);
                let half_width = radius * 0.866_025_403_784_438_6;
                for p in points {
                    let triangle = [
                        Point2D::new(p.x, p.y + radius),
                        Point2D::new(p.x - half_width, p.y - radius * 0.5),
                        Point2D::new(p.x + half_width, p.y - radius * 0.5),
                    ];
                    let vertices = self.fmt_points(&triangle);
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            self.buf(),
                            "<polygon points=\"{vertices}\" fill=\"{color}\" fill-opacity=\"{}\" stroke=\"{outline_color}\" stroke-width=\"{outline_width:.1}\"/>",
                            style.opacity
                        );
                    } else {
                        let _ = writeln!(
                            self.buf(),
                            "<polygon points=\"{vertices}\" fill=\"{color}\" fill-opacity=\"{}\"/>",
                            style.opacity
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_begun("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        validate_points(points, "line")?;
        validate_line_style(style)?;
        let color = rgb_to_hex(style.color);
        let pts_str = self.fmt_points(points);
        let buf = self.buf();
        if style.dash.is_empty() {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                style.width
            );
        } else {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"{}\"/>",
                style.width, style.dash
            );
        }
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_begun("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        validate_points(points, "polygon")?;
        validate_fill_style(style)?;
        let fill = rgb_to_hex(style.color);
        let stroke = rgb_to_hex(style.stroke);
        let pts_str = self.fmt_points(points);
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<polygon points=\"{pts_str}\" fill=\"{fill}\" fill-opacity=\"{:.2}\" stroke=\"{stroke}\" stroke-width=\"{}\"/>",
            style.opacity, style.stroke_width
        );
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.ensure_begun("draw_linear_gradient")?;
        validate_linear_gradient(bounds, start, end, stops)?;
        let id = self.next_gradient_id;
        self.next_gradient_id += 1;
        let start_y = self.y_mark(start.y);
        let end_y = self.y_mark(end.y);
        let _ = writeln!(
            self.gradient_defs,
            "<linearGradient id=\"linear-gradient-{id}\" gradientUnits=\"userSpaceOnUse\" x1=\"{:.3}\" y1=\"{start_y:.3}\" x2=\"{:.3}\" y2=\"{end_y:.3}\">",
            start.x, end.x
        );
        for stop in stops {
            let color = rgb_to_hex(stop.color);
            let _ = writeln!(
                self.gradient_defs,
                "<stop offset=\"{:.6}\" stop-color=\"{color}\"/>",
                stop.offset
            );
        }
        let _ = writeln!(self.gradient_defs, "</linearGradient>");
        let svg_y = self.y_mark(bounds.y1);
        let width = bounds.width();
        let height = bounds.height();
        let _ = writeln!(
            self.buf(),
            "<rect x=\"{:.3}\" y=\"{svg_y:.3}\" width=\"{width:.3}\" height=\"{height:.3}\" fill=\"url(#linear-gradient-{id})\"/>",
            bounds.x0
        );
        Ok(())
    }

    fn draw_rounded_rect(
        &mut self,
        rect_x: f64,
        rect_y: f64,
        rect_w: f64,
        rect_h: f64,
        radius: f64,
        style: &FillStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_rounded_rect")?;
        validate_rounded_rect(rect_x, rect_y, rect_w, rect_h, radius)?;
        validate_fill_style(style)?;
        let fill = rgb_to_hex(style.color);
        let stroke = rgb_to_hex(style.stroke);
        // Convert math coords to SVG coords: top-left in SVG is (x, height - (y + h))
        let svg_x = rect_x;
        let svg_y = self.y_mark(rect_y + rect_h);
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<rect x=\"{svg_x:.2}\" y=\"{svg_y:.2}\" width=\"{rect_w:.2}\" height=\"{rect_h:.2}\" rx=\"{radius:.1}\" ry=\"{radius:.1}\" fill=\"{fill}\" fill-opacity=\"{:.2}\" stroke=\"{stroke}\" stroke-width=\"{}\"/>",
            style.opacity, style.stroke_width
        );
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn draw_text(
        &mut self,
        x_pixel: u32,
        y_pixel: u32,
        text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_text")?;
        validate_text_style(style)?;
        let color = rgb_to_hex(style.color);
        let escaped = Self::escape_text(text);
        let x = x_pixel;
        let h = f64::from(self.height);
        let y = (h - f64::from(y_pixel)).round() as u32;
        let anchor = match style.anchor {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        };
        let weight = if style.weight == crate::TextWeight::Normal {
            String::new()
        } else {
            format!(" font-weight=\"{}\"", style.weight.svg_value())
        };
        let preserve_space = if text.contains("  ") {
            " xml:space=\"preserve\""
        } else {
            ""
        };
        let buf = self.buf();
        if style.rotation.abs() > f64::EPSILON {
            let _ = writeln!(
                buf,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{preserve_space} fill=\"{color}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\" transform=\"rotate({:.1},{},{})\">{escaped}</text>",
                x, y,
                Self::escape_text(&style.font_family),
                style.font_size,
                style.rotation,
                x, y
            );
        } else {
            let _ = writeln!(
                buf,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{preserve_space} fill=\"{color}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\">{escaped}</text>",
                x, y,
                Self::escape_text(&style.font_family),
                style.font_size
            );
        }
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_bracket")?;
        validate_bracket(bracket, style)?;
        let metrics = super::backend::SIGNIFICANCE_BRACKET_METRICS;
        let color = rgb_to_hex(style.color);
        let h = f64::from(self.height);
        let label = Self::escape_text(&bracket.label);
        let buf = self.buf();
        match bracket.orientation {
            super::backend::BracketOrientation::Vertical => {
                let y = h - bracket.y;
                let _ = writeln!(
                    buf,
                    "<path class=\"significance-bracket\" d=\"M {:.3} {:.3} L {:.3} {:.3} M {:.3} {:.3} L {:.3} {:.3} M {:.3} {:.3} L {:.3} {:.3}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                    bracket.x_left, y, bracket.x_right, y, bracket.x_left, y,
                    bracket.x_left, y + metrics.tick_height, bracket.x_right, y,
                    bracket.x_right, y + metrics.tick_height, style.width
                );
                let _ = writeln!(
                    buf,
                    "<text x=\"{:.3}\" y=\"{:.3}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{color}\">{label}</text>",
                    f64::midpoint(bracket.x_left, bracket.x_right), y - metrics.label_gap,
                    DEFAULT_FONT_FAMILY_STACK, metrics.label_font_size,
                );
            }
            super::backend::BracketOrientation::Horizontal => {
                let x = bracket.y;
                let bottom = h - bracket.x_left;
                let top = h - bracket.x_right;
                let _ = writeln!(
                    buf,
                    "<path class=\"significance-bracket horizontal\" d=\"M {:.3} {:.3} L {:.3} {:.3} M {:.3} {:.3} L {:.3} {:.3} M {:.3} {:.3} L {:.3} {:.3}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                    x, bottom, x, top, x, bottom, x - metrics.tick_height, bottom,
                    x, top, x - metrics.tick_height, top, style.width
                );
                let _ = writeln!(
                    buf,
                    "<text x=\"{:.3}\" y=\"{:.3}\" text-anchor=\"start\" dominant-baseline=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{color}\">{label}</text>",
                    x + metrics.label_gap, f64::midpoint(bottom, top), DEFAULT_FONT_FAMILY_STACK,
                    metrics.label_font_size,
                );
            }
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        if !self.begun {
            return Err(Error::Render("svg backend ended before begin".into()));
        }
        if !self.group_stack.is_empty() {
            return Err(Error::Render(format!(
                "svg backend ended with {} unclosed layer/clip scopes",
                self.group_stack.len()
            )));
        }
        let mut svg = String::new();
        let mut content = String::new();
        content.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n");
        if !self.grid_buf.is_empty() {
            content.push_str(&self.grid_buf);
        }
        content.push_str(&self.body);
        let _ = writeln!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\">",
            self.width, self.height, self.width, self.height
        );
        let _ = writeln!(
            svg,
            "<style>line,path,polyline,polygon,circle{{vector-effect:non-scaling-stroke}}</style>"
        );
        if !self.gradient_defs.is_empty() || !self.clip_defs.is_empty() {
            svg.push_str("<defs>\n");
            svg.push_str(&self.gradient_defs);
            svg.push_str(&self.clip_defs);
            svg.push_str("</defs>\n");
        }
        svg.push_str(&content);
        svg.push_str("</svg>\n");
        match &mut self.destination {
            SvgDestination::File(path) => {
                std::fs::write(path, svg).map_err(Error::Io)?;
            }
            SvgDestination::Memory(bytes) => *bytes = Some(svg.into_bytes()),
        }
        self.begun = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lichtung-svg-backend-{name}-{}.svg",
            std::process::id()
        ))
    }

    #[test]
    fn clip_scopes_emit_deterministic_ids_and_balanced_groups() {
        let path = output_path("clips");
        let mut backend = SvgBackend::new(path.clone(), 200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 190.0, 20.0, 80.0))
            .unwrap();
        backend
            .draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(200.0, 100.0)],
                &LineStyle::solid((0, 0, 0), 1.0),
            )
            .unwrap();
        backend.end_clip().unwrap();
        backend.end_layer().unwrap();
        backend
            .begin_clip(PanelBox::new(20.0, 180.0, 30.0, 70.0))
            .unwrap();
        backend.end_clip().unwrap();
        backend.end().unwrap();

        let svg = std::fs::read_to_string(&path).unwrap();
        assert!(svg.contains("<clipPath id=\"clip-0\"") && svg.contains("clip-1"));
        assert!(svg.contains("<g id=\"marks\">\n<g clip-path=\"url(#clip-0)\">"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn begin_clip_rejects_non_finite_or_non_positive_rectangles() {
        let mut backend = SvgBackend::new(output_path("invalid-clip"), 200, 100);
        backend.begin(200, 100).unwrap();

        assert!(backend
            .begin_clip(PanelBox::new(f64::NAN, 10.0, 0.0, 10.0))
            .is_err());
        assert!(backend
            .begin_clip(PanelBox::new(10.0, 10.0, 0.0, 10.0))
            .is_err());
    }

    #[test]
    fn end_rejects_unclosed_group_state() {
        let mut backend = SvgBackend::new(output_path("unclosed"), 200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();

        assert!(backend.end().is_err());
    }

    #[test]
    fn nested_layer_and_clip_scopes_require_lifo_order() {
        let path = output_path("nested-order");
        let mut backend = SvgBackend::new(path.clone(), 200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 190.0, 20.0, 80.0))
            .unwrap();

        assert!(backend.end_layer().is_err());
        backend.end_clip().unwrap();
        backend.end_layer().unwrap();
        backend.end().unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn linear_gradients_emit_native_deterministic_defs_and_math_space_geometry() {
        let first_path = output_path("gradient-first");
        let second_path = output_path("gradient-second");
        let stops = [
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.25, (247, 247, 247)),
            GradientStop::new(1.0, (222, 45, 38)),
        ];
        for path in [&first_path, &second_path] {
            let mut backend = SvgBackend::new(path.clone(), 200, 100);
            backend.begin(200, 100).unwrap();
            backend
                .draw_linear_gradient(
                    PanelBox::new(10.0, 22.0, 20.0, 80.0),
                    Point2D::new(16.0, 20.0),
                    Point2D::new(16.0, 80.0),
                    &stops,
                )
                .unwrap();
            backend.end().unwrap();
        }

        let first = std::fs::read_to_string(&first_path).unwrap();
        let second = std::fs::read_to_string(&second_path).unwrap();
        assert_eq!(first, second);
        assert!(first.contains(
            "<linearGradient id=\"linear-gradient-0\" gradientUnits=\"userSpaceOnUse\" x1=\"16.000\" y1=\"80.000\" x2=\"16.000\" y2=\"20.000\">"
        ));
        assert!(first.contains("<stop offset=\"0.250000\" stop-color=\"#F7F7F7\"/>"));
        assert!(first.contains(
            "<rect x=\"10.000\" y=\"20.000\" width=\"12.000\" height=\"60.000\" fill=\"url(#linear-gradient-0)\"/>"
        ));
        let _ = std::fs::remove_file(first_path);
        let _ = std::fs::remove_file(second_path);
    }
}
