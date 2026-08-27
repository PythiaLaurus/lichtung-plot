//! Native SVG backend for deterministic publication-oriented output.

use std::fmt::Write as _;

use crate::Error;

use super::backend::{
    validate_bracket, validate_clip_region, validate_fill_style, validate_line_points,
    validate_line_style, validate_linear_gradient, validate_point_style, validate_points,
    validate_polygon_points, validate_rounded_rect, validate_text, validate_triangle_points,
    Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextAnchor, TextStyle, POLYGON_MITER_LIMIT,
};
use super::coordinates::CanonicalPoint;
use crate::layout::PanelBox;
use crate::spec::{PngCompression, PngOptions, PngSizing, SVG_CSS_DPI};
use crate::theme::rgb_to_hex;
use crate::PointShape;

#[cfg(test)]
thread_local! {
    pub(crate) static PNG_TARGET_PLAN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl SvgBackend {
    pub(crate) fn in_memory(width: u32, height: u32) -> Self {
        Self {
            output: None,
            width,
            height,
            body: String::new(),
            grid_buf: String::new(),
            clip_defs: String::new(),
            gradient_defs: String::new(),
            group_stack: Vec::new(),
            next_clip_id: 0,
            next_gradient_id: 0,
            state: SvgCanvasState::Fresh,
        }
    }

    pub(crate) fn into_output(self) -> Option<FinalizedSvgOutput> {
        self.output
    }

    #[cfg(test)]
    pub(crate) fn into_bytes(self) -> Result<Vec<u8>, Error> {
        self.into_output()
            .map(|output| output.bytes.into_vec())
            .ok_or_else(|| Error::Render("in-memory SVG backend did not finish rendering".into()))
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

    fn fmt_points(&self, points: &[CanonicalPoint]) -> Result<String, Error> {
        let mut out = String::new();
        for (i, p) in points.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let y = p.y.inverted(self.height)?;
            let _ = write!(out, "{},{}", p.x, y);
        }
        Ok(out)
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
        if self.state == SvgCanvasState::Open {
            Ok(())
        } else {
            Err(Error::Render(format!(
                "svg backend {operation} called outside begin/end"
            )))
        }
    }
}

pub(crate) struct FinalizedSvgOutput {
    width_css_px: u32,
    height_css_px: u32,
    bytes: Box<[u8]>,
}

impl FinalizedSvgOutput {
    pub(crate) fn into_parts(self) -> (u32, u32, Box<[u8]>) {
        (self.width_css_px, self.height_css_px, self.bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupState {
    Grid,
    Layer,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SvgCanvasState {
    Fresh,
    Open,
    Closed,
}

/// Convert canonical SVG scene text to vector paths using installed fonts.
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
pub(crate) fn rasterize_svg_bytes_to_png(
    svg_data: &[u8],
    source_size: resvg::tiny_skia::IntSize,
    png_options: &PngOptions,
) -> Result<Vec<u8>, Error> {
    let plan = PngTargetPlan::new(source_size, png_options)?;

    let mut options = resvg::usvg::Options::default();
    crate::font::configure_system_fonts(options.fontdb_mut())?;

    let tree = resvg::usvg::Tree::from_data(svg_data, &options)
        .map_err(|e| Error::Render(format!("parse svg for png rasterization: {e}")))?;
    let source_size = tree.size().to_int_size();
    if source_size != plan.source_size {
        return Err(Error::Render(
            "rendered SVG scene dimensions do not match its canonical CSS dimensions".into(),
        ));
    }
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(plan.target_size.width(), plan.target_size.height())
            .ok_or_else(|| Error::Render("allocate png pixmap".into()))?;
    let transform = png_transform(plan.source_size, plan.target_size);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    encode_png(pixmap, png_options.compression, plan.dpi)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PngTargetPlan {
    source_size: resvg::tiny_skia::IntSize,
    target_size: resvg::tiny_skia::IntSize,
    dpi: Option<u32>,
}

impl PngTargetPlan {
    fn new(source_size: resvg::tiny_skia::IntSize, options: &PngOptions) -> Result<Self, Error> {
        #[cfg(test)]
        PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        validate_png_options(options)?;
        let target_size = match options.sizing {
            PngSizing::Source => Some(source_size),
            PngSizing::Dpi(dpi) => source_size.scale_by(dpi / SVG_CSS_DPI),
            PngSizing::Scale(scale) => source_size.scale_by(scale),
            PngSizing::Width(width) => source_size.scale_to_width(width),
            PngSizing::Height(height) => source_size.scale_to_height(height),
        }
        .ok_or_else(|| Error::Render("computed zero-sized png output".into()))?;
        crate::scene::validate_png_target_dimensions(
            f64::from(target_size.width()),
            f64::from(target_size.height()),
        )?;
        let dpi = match options.sizing {
            PngSizing::Dpi(dpi) => Some(dpi_to_pixels_per_meter(dpi)?),
            _ => None,
        };
        Ok(Self {
            source_size,
            target_size,
            dpi,
        })
    }
}

fn validate_png_options(options: &PngOptions) -> Result<(), Error> {
    match options.sizing {
        PngSizing::Source => Ok(()),
        PngSizing::Dpi(dpi) => ensure_positive_finite(f64::from(dpi), "png dpi"),
        PngSizing::Scale(scale) => ensure_positive_finite(f64::from(scale), "png scale"),
        PngSizing::Width(width) => ensure_positive_dimension(width, "png width"),
        PngSizing::Height(height) => ensure_positive_dimension(height, "png height"),
    }
}

fn ensure_positive_finite(value: f64, label: &str) -> Result<(), Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "{label} must be positive and finite; got {value}"
        )))
    }
}

fn ensure_positive_dimension(value: u32, label: &str) -> Result<(), Error> {
    if value > 0 {
        Ok(())
    } else {
        Err(Error::Render(format!("{label} must be positive")))
    }
}

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

fn encode_png(
    pixmap: resvg::tiny_skia::Pixmap,
    compression: PngCompression,
    dpi: Option<u32>,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, pixmap.width(), pixmap.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(map_png_compression(compression));
        if let Some(dpi) = dpi {
            encoder.set_pixel_dims(Some(png::PixelDimensions {
                xppu: dpi,
                yppu: dpi,
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

fn map_png_compression(compression: PngCompression) -> png::Compression {
    match compression {
        PngCompression::None => png::Compression::NoCompression,
        PngCompression::Fast => png::Compression::Fast,
        PngCompression::Balanced => png::Compression::Balanced,
        PngCompression::High => png::Compression::High,
    }
}

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
    output: Option<FinalizedSvgOutput>,
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
    state: SvgCanvasState,
}

impl Backend for SvgBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.output = None;
        if self.state == SvgCanvasState::Open {
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
        self.state = SvgCanvasState::Open;
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
        let region = validate_clip_region(region)?;
        let id = self.next_clip_id;
        self.next_clip_id += 1;
        let svg_y = region.top.inverted(self.height)?;
        let _ = writeln!(
            self.clip_defs,
            "<clipPath id=\"clip-{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>",
            region.left, svg_y, region.width, region.height,
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
        let canonical_points = validate_points(points, "scatter")?;
        validate_point_style(style)?;
        let color = rgb_to_hex(style.color);
        let outline = style.outline.map(|outline| {
            (
                rgb_to_hex(outline.color),
                outline.width_for_radius(style.radius),
            )
        });
        let height = self.height;
        match style.shape {
            PointShape::Circle => {
                let buf = self.buf();
                for p in canonical_points {
                    let y = p.y.inverted(height)?;
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{color}\" fill-opacity=\"{}\" stroke=\"{outline_color}\" stroke-width=\"{outline_width:.1}\"/>",
                            p.x, y,
                            style.radius,
                            style.opacity
                        );
                    } else {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{color}\" fill-opacity=\"{}\"/>",
                            p.x, y,
                            style.radius,
                            style.opacity
                        );
                    }
                }
            }
            PointShape::TriangleUp => {
                for triangle in validate_triangle_points(points, style.radius)? {
                    let vertices = self.fmt_points(&triangle)?;
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            self.buf(),
                            "<polygon points=\"{vertices}\" fill=\"{color}\" fill-opacity=\"{}\" stroke=\"{outline_color}\" stroke-width=\"{outline_width:.1}\" stroke-linejoin=\"miter\" stroke-miterlimit=\"{POLYGON_MITER_LIMIT:.1}\"/>",
                            style.opacity,
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
        let points = validate_line_points(points)?;
        validate_line_style(style)?;
        let color = rgb_to_hex(style.color);
        let pts_str = self.fmt_points(&points)?;
        let buf = self.buf();
        if let Some(dasharray) = style.pattern.svg_dasharray() {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"{dasharray}\"/>",
                style.width,
            );
        } else {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                style.width
            );
        }
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_begun("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        let points = validate_polygon_points(points)?;
        validate_fill_style(style)?;
        let fill = rgb_to_hex(style.color);
        let stroke = rgb_to_hex(style.stroke);
        let pts_str = self.fmt_points(&points)?;
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<polygon points=\"{pts_str}\" fill=\"{fill}\" fill-opacity=\"{:.2}\" stroke=\"{stroke}\" stroke-width=\"{}\" stroke-linejoin=\"miter\" stroke-miterlimit=\"{POLYGON_MITER_LIMIT:.1}\"/>",
            style.opacity, style.stroke_width,
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
        let gradient = validate_linear_gradient(bounds, start, end, stops)?;
        let id = self.next_gradient_id;
        self.next_gradient_id += 1;
        let start_y = gradient.start.y.inverted(self.height)?;
        let end_y = gradient.end.y.inverted(self.height)?;
        let _ = writeln!(
            self.gradient_defs,
            "<linearGradient id=\"linear-gradient-{id}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">",
            gradient.start.x, start_y, gradient.end.x, end_y,
        );
        for stop in gradient.stops {
            let color = rgb_to_hex(stop.color);
            let _ = writeln!(
                self.gradient_defs,
                "<stop offset=\"{}\" stop-color=\"{color}\"/>",
                stop.offset
            );
        }
        let _ = writeln!(self.gradient_defs, "</linearGradient>");
        let svg_y = gradient.bounds.top.inverted(self.height)?;
        let _ = writeln!(
            self.buf(),
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#linear-gradient-{id})\"/>",
            gradient.bounds.left,
            svg_y,
            gradient.bounds.width,
            gradient.bounds.height,
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
        let rounded = validate_rounded_rect(rect_x, rect_y, rect_w, rect_h, radius)?;
        validate_fill_style(style)?;
        let fill = rgb_to_hex(style.color);
        let stroke = rgb_to_hex(style.stroke);
        let svg_y = rounded.rect.top.inverted(self.height)?;
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{fill}\" fill-opacity=\"{:.2}\" stroke=\"{stroke}\" stroke-width=\"{}\"/>",
            rounded.rect.left,
            svg_y,
            rounded.rect.width,
            rounded.rect.height,
            rounded.radius,
            rounded.radius,
            style.opacity,
            style.stroke_width,
        );
        Ok(())
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.ensure_begun("draw_text")?;
        let canonical = validate_text(position, style)?;
        let color = rgb_to_hex(style.color);
        let escaped = Self::escape_text(text);
        let x = canonical.position.x;
        let y = canonical.position.y.inverted(self.height)?;
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
        if canonical.rotation.units() != 0 {
            let _ = writeln!(
                buf,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{preserve_space} fill=\"{color}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\" transform=\"rotate({},{},{})\">{escaped}</text>",
                x, y,
                Self::escape_text(&style.font_family),
                style.font_size,
                canonical.rotation,
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
        let canonical = validate_bracket(bracket, style)?;
        let color = rgb_to_hex(style.color);
        let label = Self::escape_text(&bracket.label);
        let font_family = Self::escape_text(&style.font_family);
        let weight = if style.weight == crate::TextWeight::Normal {
            String::new()
        } else {
            format!(" font-weight=\"{}\"", style.weight.svg_value())
        };
        let height = self.height;
        let line_start_y = canonical.line_start.y.inverted(height)?;
        let line_end_y = canonical.line_end.y.inverted(height)?;
        let first_tick_y = canonical.first_tick_end.y.inverted(height)?;
        let second_tick_y = canonical.second_tick_end.y.inverted(height)?;
        let label_y = canonical.label_position.y.inverted(height)?;
        let buf = self.buf();
        match canonical.orientation {
            super::backend::BracketOrientation::Vertical => {
                let _ = writeln!(
                    buf,
                    "<path class=\"significance-bracket\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                    canonical.line_start.x,
                    line_start_y,
                    canonical.line_end.x,
                    line_end_y,
                    canonical.line_start.x,
                    line_start_y,
                    canonical.first_tick_end.x,
                    first_tick_y,
                    canonical.line_end.x,
                    line_end_y,
                    canonical.second_tick_end.x,
                    second_tick_y,
                    style.width,
                );
                let _ = writeln!(
                    buf,
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"{font_family}\" font-size=\"{}\"{weight} fill=\"{color}\">{label}</text>",
                    canonical.label_position.x,
                    label_y,
                    style.font_size,
                );
            }
            super::backend::BracketOrientation::Horizontal => {
                let _ = writeln!(
                    buf,
                    "<path class=\"significance-bracket horizontal\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
                    canonical.line_start.x,
                    line_start_y,
                    canonical.line_end.x,
                    line_end_y,
                    canonical.line_start.x,
                    line_start_y,
                    canonical.first_tick_end.x,
                    first_tick_y,
                    canonical.line_end.x,
                    line_end_y,
                    canonical.second_tick_end.x,
                    second_tick_y,
                    style.width,
                );
                let _ = writeln!(
                    buf,
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"start\" dominant-baseline=\"middle\" font-family=\"{font_family}\" font-size=\"{}\"{weight} fill=\"{color}\">{label}</text>",
                    canonical.label_position.x,
                    label_y,
                    style.font_size,
                );
            }
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.state != SvgCanvasState::Open {
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
        self.output = Some(FinalizedSvgOutput {
            width_css_px: self.width,
            height_css_px: self.height,
            bytes: svg.into_bytes().into_boxed_slice(),
        });
        self.state = SvgCanvasState::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_size(width: u32, height: u32) -> resvg::tiny_skia::IntSize {
        resvg::tiny_skia::IntSize::from_wh(width, height).unwrap()
    }

    fn target_size(width: u32, height: u32, sizing: PngSizing) -> (u32, u32) {
        let plan = PngTargetPlan::new(
            source_size(width, height),
            &PngOptions {
                sizing,
                compression: PngCompression::Balanced,
            },
        )
        .unwrap();
        (plan.target_size.width(), plan.target_size.height())
    }

    #[test]
    fn png_target_plan_computes_each_sizing_mode_once() {
        PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(0));
        crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
        assert_eq!(target_size(319, 241, PngSizing::Source), (319, 241));
        assert_eq!(target_size(319, 241, PngSizing::Dpi(300.0)), (997, 753));
        assert_eq!(target_size(319, 241, PngSizing::Scale(1.5)), (479, 362));
        assert_eq!(target_size(319, 241, PngSizing::Width(500)), (500, 378));
        assert_eq!(target_size(319, 241, PngSizing::Height(500)), (662, 500));
        assert_eq!(PNG_TARGET_PLAN_CALLS.with(std::cell::Cell::get), 5);
        assert_eq!(
            crate::scene::PNG_TARGET_VALIDATION_CALLS.with(std::cell::Cell::get),
            5
        );
    }

    #[test]
    fn png_target_plan_rejects_rounded_zero_and_exact_limit_overflow() {
        let options = |sizing| PngOptions {
            sizing,
            compression: PngCompression::Balanced,
        };
        assert!(PngTargetPlan::new(
            source_size(320, 240),
            &options(PngSizing::Scale(f32::MIN_POSITIVE))
        )
        .unwrap_err()
        .to_string()
        .contains("zero-sized"));
        assert!(PngTargetPlan::new(source_size(32_768, 1), &options(PngSizing::Source),).is_ok());
        assert!(
            PngTargetPlan::new(source_size(32_769, 1), &options(PngSizing::Source),)
                .unwrap_err()
                .to_string()
                .contains("conversion limit")
        );
        assert!(
            PngTargetPlan::new(source_size(10_000, 10_000), &options(PngSizing::Source)).is_ok()
        );
        assert!(
            PngTargetPlan::new(source_size(10_000, 10_001), &options(PngSizing::Source))
                .unwrap_err()
                .to_string()
                .contains("work")
        );
    }

    #[test]
    fn png_dpi_plan_preserves_f32_sizing_and_checks_target_before_metadata() {
        assert_eq!(target_size(10_000, 10_000, PngSizing::Dpi(4.2)), (438, 438));

        let error = PngTargetPlan::new(
            source_size(1, 1),
            &PngOptions {
                sizing: PngSizing::Dpi(120_000_000.0),
                compression: PngCompression::Balanced,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("conversion limit"));
    }

    fn draw_test_line(backend: &mut SvgBackend) {
        backend
            .draw_line(
                &[Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)],
                &LineStyle::solid((0, 0, 0), 1.0),
            )
            .unwrap();
    }

    fn test_text_style() -> TextStyle {
        TextStyle {
            color: (0, 0, 0),
            font_family: "sans-serif".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Start,
            rotation: 0.0,
        }
    }

    fn test_fill_style() -> FillStyle {
        FillStyle {
            color: (1, 2, 3),
            stroke: (4, 5, 6),
            stroke_width: 0.0,
            opacity: 1.0,
        }
    }

    #[test]
    fn canvas_lifecycle_reuses_clean_state_and_invalidates_stale_output() {
        let mut backend = SvgBackend::in_memory(200, 100);
        let points = [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        let style = LineStyle::solid((0, 0, 0), 1.0);

        assert!(backend.end().is_err());
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.begin(0, 100).is_err());

        backend.begin(200, 100).unwrap();
        assert!(backend.begin(200, 100).is_err());
        draw_test_line(&mut backend);
        backend.end().unwrap();
        assert!(backend.output.is_some());
        assert!(backend.end().is_err());
        assert!(backend.draw_line(&points, &style).is_err());

        backend.begin(120, 80).unwrap();
        backend.end().unwrap();
        let latest = String::from_utf8(backend.output.as_ref().unwrap().bytes.to_vec()).unwrap();
        assert!(latest.contains("width=\"120\" height=\"80\""));
        assert!(!latest.contains("<polyline"));

        assert!(backend.begin(120, 0).is_err());
        assert!(backend.output.is_none());
        assert!(backend.into_bytes().is_err());
    }

    #[test]
    fn signed_text_positions_preserve_off_canvas_values_and_y_orientation() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_text(Point2D::new(-7.0, 125.0), "off canvas", &test_text_style())
            .unwrap();
        backend
            .draw_text(Point2D::new(10.0, 25.0), "low", &test_text_style())
            .unwrap();
        backend
            .draw_text(Point2D::new(10.0, 75.0), "high", &test_text_style())
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains("<text x=\"-7\" y=\"-25\""));
        assert!(svg.contains("<text x=\"10\" y=\"75\""));
        assert!(svg.contains("<text x=\"10\" y=\"25\""));
    }

    #[test]
    fn subprecision_points_use_the_shared_canonical_spelling() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0006, 0.0006)],
                &LineStyle::solid((0, 0, 0), 0.0),
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains("points=\"0.000,100.000 0.001,99.999\""));
    }

    #[test]
    fn triangle_vertices_are_derived_before_the_single_canonical_conversion() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_scatter(
                &[Point2D::new(10.00049, 20.0)],
                &PointStyle::plain((0, 0, 0), 1).with_shape(PointShape::TriangleUp),
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains("points=\"10.000,79.000 9.134,80.500 10.867,80.500\""));
    }

    #[test]
    fn point_shapes_and_polygon_joins_have_exact_closed_serialization() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_scatter(
                &[Point2D::new(10.0, 20.0)],
                &PointStyle::plain((0, 0, 0), 2),
            )
            .unwrap();
        backend
            .draw_scatter(
                &[Point2D::new(20.0, 20.0)],
                &PointStyle::plain((1, 2, 3), 2).with_shape(PointShape::TriangleUp),
            )
            .unwrap();
        backend
            .draw_scatter(&[Point2D::new(15.0, 20.0)], &PointStyle::new((7, 8, 9), 2))
            .unwrap();
        backend
            .draw_scatter(
                &[Point2D::new(30.0, 20.0)],
                &PointStyle::new((4, 5, 6), 2).with_shape(PointShape::TriangleUp),
            )
            .unwrap();
        backend
            .draw_polygon(
                &[
                    Point2D::new(10.0, 10.0),
                    Point2D::new(20.0, 10.0),
                    Point2D::new(10.0, 20.0),
                ],
                &test_fill_style(),
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let primitives = svg
            .lines()
            .filter(|line| line.starts_with("<circle ") || line.starts_with("<polygon "))
            .collect::<Vec<_>>();
        assert_eq!(
            primitives,
            [
                "<circle cx=\"10.000\" cy=\"80.000\" r=\"2\" fill=\"#000000\" fill-opacity=\"0.82\"/>",
                "<polygon points=\"20.000,78.000 18.268,81.000 21.732,81.000\" fill=\"#010203\" fill-opacity=\"0.82\"/>",
                "<circle cx=\"15.000\" cy=\"80.000\" r=\"2\" fill=\"#070809\" fill-opacity=\"0.82\" stroke=\"#FFFFFF\" stroke-width=\"0.2\"/>",
                "<polygon points=\"30.000,78.000 28.268,81.000 31.732,81.000\" fill=\"#040506\" fill-opacity=\"0.82\" stroke=\"#FFFFFF\" stroke-width=\"0.2\" stroke-linejoin=\"miter\" stroke-miterlimit=\"4.0\"/>",
                "<polygon points=\"10.000,90.000 20.000,90.000 10.000,80.000\" fill=\"#010203\" fill-opacity=\"1.00\" stroke=\"#040506\" stroke-width=\"0\" stroke-linejoin=\"miter\" stroke-miterlimit=\"4.0\"/>",
            ]
        );
    }

    #[test]
    fn line_patterns_are_exact_and_degenerate_invalid_carriers_are_noops() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_line(
                &[Point2D::new(f64::NAN, f64::NAN)],
                &LineStyle::solid((0, 0, 0), f32::NAN),
            )
            .unwrap();
        backend
            .draw_polygon(
                &[
                    Point2D::new(f64::NAN, f64::NAN),
                    Point2D::new(f64::NAN, f64::NAN),
                ],
                &FillStyle {
                    color: (0, 0, 0),
                    stroke: (0, 0, 0),
                    stroke_width: f32::NAN,
                    opacity: f64::NAN,
                },
            )
            .unwrap();
        let points = [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        for style in [
            LineStyle::solid((0, 0, 0), 1.25),
            LineStyle::short_dashed((0, 0, 0), 1.25),
            LineStyle::medium_dashed((0, 0, 0), 1.25),
            LineStyle::long_dashed((0, 0, 0), 1.25),
        ] {
            backend.draw_line(&points, &style).unwrap();
        }
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let lines = svg
            .lines()
            .filter(|line| line.starts_with("<polyline "))
            .collect::<Vec<_>>();
        assert_eq!(
            lines,
            [
                "<polyline points=\"1.000,99.000 2.000,98.000\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                "<polyline points=\"1.000,99.000 2.000,98.000\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"3 2\"/>",
                "<polyline points=\"1.000,99.000 2.000,98.000\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"4 3\"/>",
                "<polyline points=\"1.000,99.000 2.000,98.000\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"5 3\"/>",
            ]
        );
        assert!(!svg.contains("fill-opacity=\"NaN\""));
    }

    #[test]
    fn rounded_rect_uses_native_geometry() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_rounded_rect(
                10.0,
                6.0,
                30.0,
                20.0,
                3.0,
                &FillStyle {
                    color: (1, 2, 3),
                    stroke: (4, 5, 6),
                    stroke_width: 1.5,
                    opacity: 0.25,
                },
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let rounded_rects = svg
            .lines()
            .filter(|line| line.starts_with("<rect x="))
            .collect::<Vec<_>>();
        assert_eq!(
            rounded_rects,
            ["<rect x=\"10.00\" y=\"74.00\" width=\"30.00\" height=\"20.00\" rx=\"3.0\" ry=\"3.0\" fill=\"#010203\" fill-opacity=\"0.25\" stroke=\"#040506\" stroke-width=\"1.5\"/>"]
        );
        assert!(!svg.contains("<polygon"));
    }

    #[test]
    fn xml_special_characters_are_escaped_in_every_current_context() {
        let special = "<&\"'>";
        let escaped = "&lt;&amp;&quot;&apos;&gt;";
        let mut text_style = test_text_style();
        text_style.font_family = special.into();
        let bracket_style = BracketStyle {
            color: (0, 0, 0),
            width: 1.0,
            font_family: special.into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            label_gap: 5.0,
        };
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend.begin_layer(special).unwrap();
        backend
            .draw_text(Point2D::new(10.0, 10.0), special, &text_style)
            .unwrap();
        backend.end_layer().unwrap();
        backend
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 10.0,
                    x_right: 20.0,
                    y: 30.0,
                    label: special.into(),
                    orientation: super::super::backend::BracketOrientation::Vertical,
                },
                &bracket_style,
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains(&format!("<g id=\"{escaped}\">")));
        assert_eq!(
            svg.matches(&format!("font-family=\"{escaped}\"")).count(),
            2
        );
        assert_eq!(svg.matches(&format!(">{escaped}</text>")).count(), 2);
        assert!(!svg.contains(special));
    }

    #[test]
    fn empty_canvas_has_exact_canonical_document_framing() {
        let mut backend = SvgBackend::in_memory(2, 3);
        backend.begin(2, 3).unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert_eq!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"2\" height=\"3\" viewBox=\"0 0 2 3\" role=\"img\">\n<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>\n<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n</svg>\n"
        );
    }

    #[test]
    fn positive_subprecision_extents_and_stops_remain_representable() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .begin_clip(PanelBox::new(0.0, 0.0006, 0.0, 0.0006))
            .unwrap();
        backend.end_clip().unwrap();
        backend
            .draw_linear_gradient(
                PanelBox::new(0.0, 0.0006, 0.0, 0.0006),
                Point2D::new(0.0, 0.0),
                Point2D::new(0.0006, 0.0006),
                &[
                    GradientStop::new(0.0, (0, 0, 0)),
                    GradientStop::new(0.000_000_6, (127, 127, 127)),
                    GradientStop::new(1.0, (255, 255, 255)),
                ],
            )
            .unwrap();
        backend
            .draw_rounded_rect(0.0, 0.0, 0.006, 0.006, 0.0, &test_fill_style())
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains(
            "<clipPath id=\"clip-0\"><rect x=\"0.000\" y=\"99.999\" width=\"0.001\" height=\"0.001\"/></clipPath>"
        ));
        assert!(svg.contains("<stop offset=\"0.000001\" stop-color=\"#7F7F7F\"/>"));
        assert!(svg.contains(
            "<rect x=\"0.000\" y=\"99.999\" width=\"0.001\" height=\"0.001\" fill=\"url(#linear-gradient-0)\"/>"
        ));
        assert!(svg.contains(
            "<rect x=\"0.00\" y=\"99.99\" width=\"0.01\" height=\"0.01\" rx=\"0.0\" ry=\"0.0\""
        ));
    }

    #[test]
    fn subprecision_bracket_anchors_remain_distinct_after_conversion() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 0.0,
                    x_right: 0.0006,
                    y: 20.0,
                    label: "*".into(),
                    orientation: super::super::backend::BracketOrientation::Vertical,
                },
                &BracketStyle {
                    color: (0, 0, 0),
                    width: 1.0,
                    font_family: "sans-serif".into(),
                    font_size: 12,
                    weight: crate::TextWeight::Normal,
                    label_gap: 5.0,
                },
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains(
            "d=\"M 0.000 80.000 L 0.001 80.000 M 0.000 80.000 L 0.000 87.000 M 0.001 80.000 L 0.001 87.000\""
        ));
    }

    #[test]
    fn canvas_end_requires_balanced_scopes() {
        let mut backend = SvgBackend::in_memory(200, 100);
        backend.begin(200, 100).unwrap();

        assert!(backend.end_clip().is_err());
        backend
            .begin_clip(PanelBox::new(10.0, 190.0, 20.0, 80.0))
            .unwrap();
        assert!(backend.end().is_err());
        backend.end_clip().unwrap();
        backend.end().unwrap();
    }

    #[test]
    fn grid_role_serializes_before_generic_layers() {
        let mut backend = SvgBackend::in_memory(200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();
        draw_test_line(&mut backend);
        backend.end_layer().unwrap();
        backend
            .begin_layer_with_role("grid", LayerRole::Grid)
            .unwrap();
        draw_test_line(&mut backend);
        backend.end_layer().unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.find("<g id=\"grid\">").unwrap() < svg.find("<g id=\"marks\">").unwrap());
    }

    #[test]
    fn text_outlining_preserves_glyph_geometry() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="40" viewBox="0 0 120 40"><text x="8" y="28" font-family="sans-serif" font-size="20">Visible</text></svg>"#;
        let outlined = convert_svg_text_to_paths(svg, 120, 40).unwrap();
        let outlined = String::from_utf8(outlined).unwrap();

        assert!(!outlined.contains("<text"));
        assert!(outlined.contains("<path"));
    }

    #[test]
    fn png_conversion_keeps_parse_and_dimension_failures_route_specific() {
        let options = PngOptions::default();
        let error =
            rasterize_svg_bytes_to_png(b"not svg", source_size(10, 10), &options).unwrap_err();
        assert!(error
            .to_string()
            .contains("parse svg for png rasterization"));

        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="11" height="10" viewBox="0 0 11 10"/>"#;
        let error = rasterize_svg_bytes_to_png(svg, source_size(10, 10), &options).unwrap_err();
        assert!(error
            .to_string()
            .contains("dimensions do not match its canonical CSS dimensions"));
    }

    #[test]
    fn dpi_metadata_rejects_values_outside_png_range() {
        let error = dpi_to_pixels_per_meter(f32::MAX).unwrap_err();
        assert!(error.to_string().contains("too large to encode"));
    }

    #[test]
    fn png_compression_presets_keep_their_encoder_mappings() {
        assert_eq!(
            std::mem::discriminant(&map_png_compression(PngCompression::None)),
            std::mem::discriminant(&png::Compression::NoCompression)
        );
        assert_eq!(
            std::mem::discriminant(&map_png_compression(PngCompression::Fast)),
            std::mem::discriminant(&png::Compression::Fast)
        );
        assert_eq!(
            std::mem::discriminant(&map_png_compression(PngCompression::Balanced)),
            std::mem::discriminant(&png::Compression::Balanced)
        );
        assert_eq!(
            std::mem::discriminant(&map_png_compression(PngCompression::High)),
            std::mem::discriminant(&png::Compression::High)
        );
    }

    #[test]
    fn clip_scopes_emit_deterministic_ids_and_balanced_groups() {
        let mut backend = SvgBackend::in_memory(200, 100);
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

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains("<clipPath id=\"clip-0\"") && svg.contains("clip-1"));
        assert!(svg.contains("<g id=\"marks\">\n<g clip-path=\"url(#clip-0)\">"));
    }

    #[test]
    fn begin_clip_rejects_non_finite_or_non_positive_rectangles() {
        let mut backend = SvgBackend::in_memory(200, 100);
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
        let mut backend = SvgBackend::in_memory(200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();

        assert!(backend.end().is_err());
    }

    #[test]
    fn nested_layer_and_clip_scopes_require_lifo_order() {
        let mut backend = SvgBackend::in_memory(200, 100);
        backend.begin(200, 100).unwrap();
        backend.begin_layer("marks").unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 190.0, 20.0, 80.0))
            .unwrap();

        assert!(backend.end_layer().is_err());
        backend.end_clip().unwrap();
        backend.end_layer().unwrap();
        backend.end().unwrap();
    }

    #[test]
    fn linear_gradients_emit_native_deterministic_defs_and_math_space_geometry() {
        let stops = [
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.25, (247, 247, 247)),
            GradientStop::new(1.0, (222, 45, 38)),
        ];
        let render = || {
            let mut backend = SvgBackend::in_memory(200, 100);
            backend.begin(200, 100).unwrap();
            backend
                .draw_linear_gradient(
                    PanelBox::new(10.0, 22.0, 20.0, 80.0),
                    Point2D::new(16.0, 20.0),
                    Point2D::new(16.0, 80.0),
                    &stops,
                )
                .unwrap();
            backend
                .draw_linear_gradient(
                    PanelBox::new(30.0, 42.0, 20.0, 80.0),
                    Point2D::new(36.0, 20.0),
                    Point2D::new(36.0, 80.0),
                    &stops,
                )
                .unwrap();
            backend.end().unwrap();
            String::from_utf8(backend.into_bytes().unwrap()).unwrap()
        };

        let first = render();
        let second = render();
        assert_eq!(first, second);
        assert!(first.contains(
            "<linearGradient id=\"linear-gradient-0\" gradientUnits=\"userSpaceOnUse\" x1=\"16.000\" y1=\"80.000\" x2=\"16.000\" y2=\"20.000\">\n<stop offset=\"0.000000\" stop-color=\"#3182BD\"/>\n<stop offset=\"0.250000\" stop-color=\"#F7F7F7\"/>\n<stop offset=\"1.000000\" stop-color=\"#DE2D26\"/>\n</linearGradient>\n"
        ));
        assert!(first.contains(
            "<rect x=\"10.000\" y=\"20.000\" width=\"12.000\" height=\"60.000\" fill=\"url(#linear-gradient-0)\"/>"
        ));
        let first_gradient = first.find("id=\"linear-gradient-0\"").unwrap();
        let second_gradient = first.find("id=\"linear-gradient-1\"").unwrap();
        let background = first.find("<rect width=\"100%\"").unwrap();
        let first_fill = first.find("fill=\"url(#linear-gradient-0)\"").unwrap();
        let second_fill = first.find("fill=\"url(#linear-gradient-1)\"").unwrap();
        assert!(first_gradient < second_gradient);
        assert!(second_gradient < background);
        assert!(background < first_fill);
        assert!(first_fill < second_fill);
    }
}
