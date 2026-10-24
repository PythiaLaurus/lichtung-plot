//! Native SVG backend for deterministic publication-oriented output.

use std::fmt::{self, Write as _};
use std::ops::Range;

use crate::theme::{color_from_rgb, Color};
use crate::Error;

#[cfg(test)]
use super::backend::{validate_bracket, validate_text, TextAnchor};
use super::backend::{
    validate_clip_region, validate_fill_style, validate_rounded_rect, validate_surface_bounds,
    Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle, PlannedSignificanceCall,
    PlannedTextPlacement, Point2D, PointStyle, SignificanceBracket, SurfaceRole, TextStyle,
    POLYGON_MITER_LIMIT,
};
use super::coordinates::{self, CanonicalPoint};
use super::primitives::{
    self, CanonicalFillStyle, CanonicalGradientFill, CanonicalLineStyle, CanonicalMarkers,
    CanonicalPlannedBracket, CanonicalPlannedText, CanonicalScatter, PathKind,
};
use crate::font::canonical::{PreparedFontEnvironment, ShapingDirection};
use crate::layout::{PanelBox, ResolvedTextPlan};
use crate::spec::{PngCompression, PngOptions, PngSizing, SVG_CSS_DPI};

#[cfg(test)]
thread_local! {
    pub(crate) static PNG_TARGET_PLAN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SVG_RAW_TEXT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SVG_PLANNED_TEXT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SVG_BEGIN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SVG_END_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static REFUSE_STORAGE: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
    pub(crate) static DERIVED_BUFFER_RESERVATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn reserve_storage<T>(
    stage: &mut crate::layout::kernel::FinalStage<'_>,
    values: &mut Vec<T>,
    count: usize,
    payload: &str,
) -> Result<(), Error> {
    if count == 0 {
        return Ok(());
    }
    #[cfg(test)]
    REFUSE_STORAGE.with(|refuse| {
        if refuse.get() == Some(payload) {
            refuse.set(None);
            crate::layout::kernel::force_reservation_failure(
                crate::layout::kernel::ReservationStage::Final,
            );
        }
    });
    stage.reserve_final(values, count, payload)
}

impl SvgBackend {
    #[cfg(test)]
    pub(crate) fn reserved_storage_addresses(&self) -> [(usize, usize); 8] {
        let address = |text: &String| (text.as_ptr().addr(), text.capacity());
        [
            address(&self.body.text),
            address(&self.root_surfaces.text),
            address(&self.panel_surfaces.text),
            address(&self.grid_buf.text),
            address(&self.clip_defs.text),
            address(&self.gradient_defs.text),
            address(&self.document),
            (
                self.text_replacements.as_ptr().addr(),
                self.text_replacements.capacity(),
            ),
        ]
    }
    #[cfg(test)]
    pub(crate) fn in_memory(width: u32, height: u32) -> Self {
        Self {
            output: None,
            width,
            height,
            body: WriteBuffer::default(),
            root_surfaces: WriteBuffer::default(),
            panel_surfaces: WriteBuffer::default(),
            grid_buf: WriteBuffer::default(),
            clip_defs: WriteBuffer::default(),
            gradient_defs: WriteBuffer::default(),
            group_stack: Vec::new(),
            next_clip_id: 0,
            next_gradient_id: 0,
            state: SvgCanvasState::Fresh,
            font_environment: None,
            text_replacements: Vec::new(),
            replacement_count: 0,
            fixed: false,
            document: String::new(),
            expected_document_bytes: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn in_memory_with_fonts(
        width: u32,
        height: u32,
        font_environment: PreparedFontEnvironment,
    ) -> Self {
        let mut backend = Self::in_memory(width, height);
        backend.font_environment = Some(font_environment);
        backend
    }

    pub(crate) fn into_output(self) -> Option<FinalizedSvgOutput> {
        self.output
    }

    #[cfg(test)]
    pub(crate) fn into_bytes(self) -> Result<Vec<u8>, Error> {
        self.into_output()
            .map(|output| output.bytes)
            .ok_or_else(|| Error::Render("in-memory SVG backend did not finish rendering".into()))
    }
}

impl<S: SvgSink> SvgSerializer<S> {
    /// Returns a reference to the active output buffer (grid or main body).
    fn buf(&mut self) -> &mut S {
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

    fn active_text_buffer(&self) -> TextBuffer {
        if self
            .group_stack
            .iter()
            .any(|state| matches!(state, GroupState::Grid))
        {
            TextBuffer::Grid
        } else {
            TextBuffer::Body
        }
    }

    fn append_text_replacement(
        &mut self,
        start: usize,
        outline: Option<PlannedLineOutline>,
    ) -> Result<(), Error> {
        let buffer = self.active_text_buffer();
        let target = match buffer {
            TextBuffer::Body => &mut self.body,
            TextBuffer::Grid => &mut self.grid_buf,
        };
        let end = target.len();
        self.replacement_count = self
            .replacement_count
            .checked_add(1)
            .ok_or_else(|| Error::Render("SVG replacement count overflow".into()))?;
        if S::COUNTING {
            return Ok(());
        }
        if self.text_replacements.len() == self.text_replacements.capacity() {
            if self.fixed {
                return Err(Error::Render("SVG exceeded reserved replacements".into()));
            }
            self.text_replacements
                .try_reserve(1)
                .map_err(|error| Error::Render(format!("SVG replacement reservation: {error}")))?;
        }
        self.text_replacements.push(TextReplacement {
            buffer,
            range: start..end,
            outline,
        });
        Ok(())
    }

    fn push_group(&mut self, group: GroupState) -> Result<(), Error> {
        if self.group_stack.len() == self.group_stack.capacity() {
            if self.fixed {
                return Err(Error::Render("SVG exceeded reserved scopes".into()));
            }
            self.group_stack
                .try_reserve(1)
                .map_err(|error| Error::Render(format!("SVG scope reservation: {error}")))?;
        }
        self.group_stack.push(group);
        Ok(())
    }

    pub(crate) fn check_buffers(&self) -> Result<(), Error> {
        for buffer in [
            &self.body,
            &self.root_surfaces,
            &self.panel_surfaces,
            &self.grid_buf,
            &self.clip_defs,
            &self.gradient_defs,
        ] {
            buffer.check()?;
        }
        Ok(())
    }

    fn escape_text(text: &str) -> EscapedText<'_> {
        EscapedText(text)
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

impl SvgBackend {
    fn outline_planned_line<S: SvgSink>(
        environment: &PreparedFontEnvironment,
        outline: &PlannedLineOutline,
        outlined: &mut S,
    ) -> Result<(), Error> {
        let text = &outline.text.0;
        let line = &text.plan.lines[outline.line_index];
        let geometry = &text.lines[outline.line_index];
        let rotation_center = (text.center.x.value(), text.center.y.value());
        let fill = SvgPaint::from(text.plan.style.paint).attributes("fill");

        for (run, geometry) in line.runs.iter().zip(&geometry.runs) {
            for (glyph, origin) in run.glyphs.iter().zip(&geometry.glyphs) {
                if glyph.face != run.face {
                    return Err(Error::Render(
                        "planned glyph face does not match its immutable shaped run".into(),
                    ));
                }
                let mut builder = SvgGlyphPath::<S>::default();
                let (metrics, retained_ink) = environment
                    .outline_glyph(glyph.face, glyph.glyph_id, &mut builder)
                    .map_err(|error| Error::Render(format!("planned glyph outlining: {error}")))?;
                if metrics.units_per_em != run.units_per_em
                    || metrics.ascender != run.design_ascender
                    || metrics.descender != run.design_descender
                {
                    return Err(Error::Render(
                        "retained outline face metrics do not match the immutable text plan".into(),
                    ));
                }
                if retained_ink != glyph.design_ink {
                    let reason = if retained_ink.is_none() && glyph.design_ink.is_some() {
                        "planned glyph has ink but its retained face has no outline"
                    } else {
                        "retained glyph outline ink does not match the immutable text plan"
                    };
                    return Err(Error::Render(reason.into()));
                }
                if builder.invalid {
                    return Err(Error::Render(
                        "retained glyph outline contains a non-finite coordinate".into(),
                    ));
                }
                if builder.allocation_failed {
                    return Err(Error::Render("glyph path buffer reservation failed".into()));
                }
                if retained_ink.is_some() && builder.data.len() == 0 {
                    return Err(Error::Render(
                        "planned glyph has ink but its retained outline path is empty".into(),
                    ));
                }
                if builder.data.len() != 0 {
                    let path_origin = (origin.x.value(), origin.y.value());
                    let scale = text.plan.style.font_size / f64::from(run.units_per_em);
                    let transform = glyph_transform(
                        scale,
                        path_origin.0,
                        path_origin.1,
                        text.rotation_sin_cos,
                        rotation_center,
                    );
                    let _ = outlined.write_str("<path d=\"");
                    outlined.append(&builder.data);
                    let _ = writeln!(
                        outlined,
                        "\" {fill} transform=\"matrix({} {} {} {} {} {})\"/>",
                        transform[0],
                        transform[1],
                        transform[2],
                        transform[3],
                        transform[4],
                        transform[5],
                    );
                    outlined.check()?;
                }
            }
        }
        Ok(())
    }
}

impl<S: SvgSink> SvgSerializer<S> {
    fn draw_multiline_planned_text(&mut self, text: &CanonicalPlannedText) -> Result<(), Error> {
        if text.0.suppressed {
            return Ok(());
        }
        if let Some(clip) = text.0.clip {
            self.begin_canonical_clip(clip)?;
        }
        let result = self.draw_planned_lines(text);
        let end = if text.0.clip.is_some() {
            self.end_clip()
        } else {
            Ok(())
        };
        crate::render::backend::finish_scope(result, end, "text-local clip")
    }

    fn draw_planned_lines(&mut self, text: &CanonicalPlannedText) -> Result<(), Error> {
        let plan = &text.0.plan;
        let fill = SvgPaint::from(plan.style.paint).attributes("fill");
        let family = Self::escape_text(&plan.style.stack_source);
        let weight = FontWeight(plan.style.weight);
        let rotation = text.0.rotation;
        let rotated = rotation.is_rotated();
        for (line_index, line) in plan.lines.iter().enumerate() {
            let geometry = &text.0.lines[line_index];
            let start = line.source_span.start_byte as usize;
            let end = line.source_span.end_byte as usize;
            let source = &plan.source[start..end];
            let escaped = Self::escape_text(source);
            let preserve_space = if source.chars().any(char::is_whitespace) {
                " xml:space=\"preserve\""
            } else {
                ""
            };
            let (direction, physical_anchor) = match line.base_direction {
                ShapingDirection::LeftToRight => ("ltr", "start"),
                ShapingDirection::RightToLeft => ("rtl", "end"),
            };
            let preserve = match self.active_text_buffer() {
                TextBuffer::Body => &mut self.body,
                TextBuffer::Grid => &mut self.grid_buf,
            };
            let start = preserve.len();
            if rotated {
                let _ = write!(
                    preserve,
                    "<text x=\"{}\" y=\"{}\" font-family=\"{family}\" font-size=\"{}\"{weight}{preserve_space} {fill} text-anchor=\"{physical_anchor}\" direction=\"{direction}\" transform=\"rotate({rotation},{},{})\"",
                    geometry.origin.x,
                    geometry.origin.y,
                    plan.style.font_size,
                    text.0.center.x,
                    text.0.center.y,
                );
            } else {
                let _ = write!(
                    preserve,
                    "<text x=\"{}\" y=\"{}\" font-family=\"{family}\" font-size=\"{}\"{weight}{preserve_space} {fill} text-anchor=\"{physical_anchor}\" direction=\"{direction}\"",
                    geometry.origin.x,
                    geometry.origin.y,
                    plan.style.font_size,
                );
            }
            if geometry.isolate_runs {
                let _ = write!(preserve, " aria-label=\"{}\">", EscapedAttribute(source));
                for &index in &geometry.order {
                    let run = &line.runs[index];
                    let Some(origin) = geometry.runs[index].origin else {
                        continue;
                    };
                    let rtl = run.bidi_level % 2 != 0;
                    let run_source = &plan.source
                        [run.source_span.start_byte as usize..run.source_span.end_byte as usize];
                    let (run_direction, anchor) = if rtl {
                        // SVG end anchoring in RTL keeps x at the physical left edge.
                        ("rtl", "end")
                    } else {
                        ("ltr", "start")
                    };
                    let _ = write!(preserve, "<tspan x=\"{}\" y=\"{}\" direction=\"{run_direction}\" unicode-bidi=\"bidi-override\" text-anchor=\"{anchor}\">", origin.x, origin.y);
                    if let Some(soft) = line.selected_soft_hyphen.filter(|soft| {
                        soft.start_byte >= run.source_span.start_byte
                            && soft.end_byte <= run.source_span.end_byte
                    }) {
                        let _ = write!(
                            preserve,
                            "{}-{}",
                            EscapedText(
                                &plan.source
                                    [run.source_span.start_byte as usize..soft.start_byte as usize]
                            ),
                            EscapedText(
                                &plan.source
                                    [soft.end_byte as usize..run.source_span.end_byte as usize]
                            )
                        );
                    } else {
                        let _ = write!(preserve, "{}", EscapedText(run_source));
                    }
                    let _ = preserve.write_str("</tspan>");
                }
                let _ = writeln!(preserve, "</text>");
            } else {
                let _ = writeln!(preserve, ">{escaped}</text>");
            }
            self.append_text_replacement(
                start,
                (!S::COUNTING).then(|| PlannedLineOutline {
                    text: text.clone(),
                    line_index,
                }),
            )?;
        }
        Ok(())
    }
}

pub(crate) struct FinalizedSvgOutput {
    width_css_px: u32,
    height_css_px: u32,
    bytes: Vec<u8>,
    outline_plan: SvgOutlinePlan,
}

type FinalizedSvgParts = (u32, u32, Vec<u8>, SvgOutlinePlan);

impl FinalizedSvgOutput {
    #[cfg(test)]
    pub(crate) fn storage_addresses(&self) -> [(usize, usize); 8] {
        let SvgOutlinePlan::Ready(source) = &self.outline_plan else {
            panic!("expected real SVG source");
        };
        let address = |text: &String| (text.as_ptr().addr(), text.capacity());
        [
            address(&source.body),
            address(&source.root_surfaces),
            address(&source.panel_surfaces),
            address(&source.grid),
            address(&source.clip_defs),
            address(&source.gradient_defs),
            (self.bytes.as_ptr().addr(), self.bytes.capacity()),
            (
                source.replacements.as_ptr().addr(),
                source.replacements.capacity(),
            ),
        ]
    }
    pub(crate) fn into_parts(self) -> FinalizedSvgParts {
        (
            self.width_css_px,
            self.height_css_px,
            self.bytes,
            self.outline_plan,
        )
    }
}

pub(crate) enum SvgOutlinePlan {
    Ready(SvgOutlineSource),
    #[cfg(test)]
    Failed(String),
}

impl SvgOutlinePlan {
    pub(crate) fn count_bytes(&self, width: u32, height: u32) -> Result<usize, String> {
        let mut count = CountBuffer::default();
        self.emit(&mut count, width, height)?;
        Ok(count.len())
    }

    pub(crate) fn count_parser_bytes(
        &self,
        width: u32,
        height: u32,
        limit: usize,
    ) -> Result<usize, String> {
        let mut count = CountBuffer {
            limit: Some(limit),
            ..CountBuffer::default()
        };
        self.emit(&mut count, width, height)?;
        Ok(count.len())
    }

    pub(crate) fn derive(&self, width_css_px: u32, height_css_px: u32) -> Result<Vec<u8>, String> {
        let count = self.count_bytes(width_css_px, height_css_px)?;
        self.derive_counted(width_css_px, height_css_px, count)
    }

    pub(crate) fn derive_counted(
        &self,
        width_css_px: u32,
        height_css_px: u32,
        count: usize,
    ) -> Result<Vec<u8>, String> {
        let mut output = WriteBuffer::default();
        #[cfg(test)]
        DERIVED_BUFFER_RESERVATIONS.with(|calls| calls.set(calls.get() + 1));
        output
            .text
            .try_reserve_exact(count)
            .map_err(|error| format!("outlined SVG reservation: {error}"))?;
        output.fixed = true;
        self.emit(&mut output, width_css_px, height_css_px)?;
        if output.len() != count {
            return Err("outlined SVG differs from counted tokens".into());
        }
        if contains_svg_text(output.text.as_bytes()) {
            return Err("finalized outlined SVG still contains text elements".into());
        }
        Ok(output.text.into_bytes())
    }

    fn emit<S: SvgSink>(&self, output: &mut S, width: u32, height: u32) -> Result<(), String> {
        let source = match self {
            Self::Ready(source) => source,
            #[cfg(test)]
            Self::Failed(error) => return Err(error.clone()),
        };
        emit_svg_header(output, width, height)
            .map_err(|_| "outlined SVG header emission failed".to_owned())?;
        if !source.gradient_defs.is_empty() || !source.clip_defs.is_empty() {
            let _ = output.write_str("<defs>\n");
            let _ = output.write_str(&source.gradient_defs);
            let _ = output.write_str(&source.clip_defs);
            let _ = output.write_str("</defs>\n");
        }
        let _ = output.write_str(&source.root_surfaces);
        let _ = output.write_str(&source.panel_surfaces);
        output.check().map_err(|error| error.to_string())?;
        apply_text_replacements(
            &source.grid,
            TextBuffer::Grid,
            &source.replacements,
            source.font_environment.as_ref(),
            output,
        )?;
        apply_text_replacements(
            &source.body,
            TextBuffer::Body,
            &source.replacements,
            source.font_environment.as_ref(),
            output,
        )?;
        let _ = output.write_str("</svg>\n");
        output.check().map_err(|error| error.to_string())
    }

    pub(crate) fn replacement_count(&self) -> usize {
        match self {
            Self::Ready(source) => source.replacements.len(),
            #[cfg(test)]
            Self::Failed(_) => 0,
        }
    }
}

pub(crate) struct SvgOutlineSource {
    font_environment: Option<PreparedFontEnvironment>,
    root_surfaces: String,
    panel_surfaces: String,
    grid: String,
    body: String,
    gradient_defs: String,
    clip_defs: String,
    replacements: Vec<TextReplacement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextBuffer {
    Body,
    Grid,
}

struct TextReplacement {
    buffer: TextBuffer,
    range: Range<usize>,
    outline: Option<PlannedLineOutline>,
}

struct PlannedLineOutline {
    text: CanonicalPlannedText,
    line_index: usize,
}

#[derive(Default)]
struct SvgGlyphPath<S: SvgSink> {
    data: S,
    invalid: bool,
    allocation_failed: bool,
}

impl<S: SvgSink> SvgGlyphPath<S> {
    fn append(&mut self, tokens: fmt::Arguments<'_>) {
        if !self.allocation_failed && self.data.write_fmt(tokens).is_err() {
            self.allocation_failed = true;
        }
    }

    fn coordinates_are_finite(&mut self, coordinates: &[f32]) -> bool {
        let finite = coordinates.iter().all(|coordinate| coordinate.is_finite());
        self.invalid |= !finite;
        finite
    }
}

impl<S: SvgSink> ttf_parser::OutlineBuilder for SvgGlyphPath<S> {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.coordinates_are_finite(&[x, y]) {
            self.append(format_args!("M{x} {y}"));
        }
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if self.coordinates_are_finite(&[x, y]) {
            self.append(format_args!("L{x} {y}"));
        }
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        if self.coordinates_are_finite(&[x1, y1, x, y]) {
            self.append(format_args!("Q{x1} {y1} {x} {y}"));
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        if self.coordinates_are_finite(&[x1, y1, x2, y2, x, y]) {
            self.append(format_args!("C{x1} {y1} {x2} {y2} {x} {y}"));
        }
    }

    fn close(&mut self) {
        self.append(format_args!("Z"));
    }
}

fn glyph_transform(
    scale: f64,
    origin_x: f64,
    origin_y: f64,
    rotation_sin_cos: (f64, f64),
    rotation_center: (f64, f64),
) -> [f64; 6] {
    let (sin, cos) = rotation_sin_cos;
    let (center_x, center_y) = rotation_center;
    [
        cos * scale,
        sin * scale,
        sin * scale,
        -cos * scale,
        cos * origin_x - sin * origin_y + center_x - cos * center_x + sin * center_y,
        sin * origin_x + cos * origin_y + center_y - sin * center_x - cos * center_y,
    ]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SvgPaint(Color);

impl SvgPaint {
    fn attributes(self, property: &str) -> PaintAttributes<'_> {
        PaintAttributes {
            paint: self,
            property,
        }
    }
}

struct PaintAttributes<'a> {
    paint: SvgPaint,
    property: &'a str,
}

impl fmt::Display for PaintAttributes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { paint, property } = self;
        write!(formatter, "{property}=\"{paint}\"")?;
        if paint.0.get_alpha() != 255 {
            let opacity = f64::from(paint.0.get_alpha()) / 255.0;
            write!(formatter, " {property}-opacity=\"{opacity}\"")?;
        }
        Ok(())
    }
}

struct EscapedText<'a>(&'a str);

struct EscapedAttribute<'a>(&'a str);

impl fmt::Display for EscapedAttribute<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            match ch {
                '\t' => formatter.write_str("&#9;")?,
                '\n' => formatter.write_str("&#10;")?,
                '\r' => formatter.write_str("&#13;")?,
                _ => {
                    let mut bytes = [0; 4];
                    EscapedText(ch.encode_utf8(&mut bytes)).fmt(formatter)?;
                }
            }
        }
        Ok(())
    }
}

struct PointTokens<'a>(&'a [CanonicalPoint]);

impl fmt::Display for PointTokens<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, point) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(" ")?;
            }
            write!(formatter, "{},{}", point.x, point.y)?;
        }
        Ok(())
    }
}

struct FontWeight(crate::TextWeight);

impl fmt::Display for FontWeight {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 != crate::TextWeight::Normal {
            write!(formatter, " font-weight=\"{}\"", self.0.svg_value())?;
        }
        Ok(())
    }
}

impl fmt::Display for EscapedText<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            match ch {
                '&' => formatter.write_str("&amp;")?,
                '<' => formatter.write_str("&lt;")?,
                '>' => formatter.write_str("&gt;")?,
                '"' => formatter.write_str("&quot;")?,
                '\'' => formatter.write_str("&apos;")?,
                _ => formatter.write_char(ch)?,
            }
        }
        Ok(())
    }
}

/// Internal token sink shared by nonrendering counting and concrete writing.
pub trait SvgSink: Default + fmt::Write {
    /// Whether this sink only counts tokens.
    const COUNTING: bool;
    /// Exact emitted byte count.
    fn len(&self) -> usize;
    /// Report a previous checked-write failure.
    fn check(&self) -> Result<(), Error>;
    /// Append an already emitted section without changing its tokens.
    fn append(&mut self, source: &Self);
}

/// Reserved canonical UTF-8 storage. Fixed writers never grow during replay.
#[derive(Default)]
pub struct WriteBuffer {
    text: String,
    fixed: bool,
    failed: bool,
}

impl WriteBuffer {
    fn prepare(
        bytes: usize,
        stage: &mut crate::layout::kernel::FinalStage<'_>,
        payload: &str,
    ) -> Result<Self, Error> {
        let mut storage = Vec::new();
        reserve_storage(stage, &mut storage, bytes, payload)?;
        let text = String::from_utf8(storage)
            .map_err(|error| Error::Render(format!("{payload}: {error}")))?;
        Ok(Self {
            text,
            fixed: true,
            failed: false,
        })
    }

    fn clear(&mut self) {
        self.text.clear();
        self.failed = false;
    }
}

impl std::ops::Deref for WriteBuffer {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl fmt::Write for WriteBuffer {
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        let result = fmt::write(self, args);
        self.failed |= result.is_err();
        result
    }
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let Some(end) = self.text.len().checked_add(text.len()) else {
            self.failed = true;
            return Err(fmt::Error);
        };
        if end > self.text.capacity() && (self.fixed || self.text.try_reserve(text.len()).is_err())
        {
            self.failed = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}

impl SvgSink for WriteBuffer {
    const COUNTING: bool = false;
    fn len(&self) -> usize {
        self.text.len()
    }
    fn check(&self) -> Result<(), Error> {
        if self.failed {
            Err(Error::Render(
                "SVG token write exceeded reserved storage".into(),
            ))
        } else {
            Ok(())
        }
    }
    fn append(&mut self, source: &Self) {
        let _ = self.write_str(&source.text);
    }
}

#[derive(Default)]
pub(crate) struct CountBuffer {
    bytes: usize,
    failed: bool,
    limit: Option<usize>,
    limit_exceeded: bool,
}

impl fmt::Write for CountBuffer {
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        let result = fmt::write(self, args);
        self.failed |= result.is_err();
        result
    }
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.add(text.len())
    }
}

impl CountBuffer {
    fn add(&mut self, bytes: usize) -> fmt::Result {
        let Some(total) = self.bytes.checked_add(bytes) else {
            self.failed = true;
            return Err(fmt::Error);
        };
        if self.limit.is_some_and(|limit| total > limit) {
            self.limit_exceeded = true;
            return Err(fmt::Error);
        }
        self.bytes = total;
        Ok(())
    }
}

impl SvgSink for CountBuffer {
    const COUNTING: bool = true;
    fn len(&self) -> usize {
        self.bytes
    }
    fn check(&self) -> Result<(), Error> {
        if self.limit_exceeded {
            return Err(Error::Render(format!(
                "PNG SVG parser input size exceeds the {}-byte conversion limit",
                self.limit.unwrap_or(usize::MAX)
            )));
        }
        if self.failed {
            Err(Error::Render("SVG token count overflow".into()))
        } else {
            Ok(())
        }
    }
    fn append(&mut self, source: &Self) {
        let _ = self.add(source.bytes);
    }
}

fn emit_svg_parts<S: SvgSink>(
    out: &mut S,
    width: u32,
    height: u32,
    gradients: &S,
    clips: &S,
    content: &[&S],
) -> Result<(), Error> {
    emit_svg_header(out, width, height)
        .map_err(|_| Error::Render("SVG header emission failed".into()))?;
    if gradients.len() != 0 || clips.len() != 0 {
        let _ = out.write_str("<defs>\n");
        out.append(gradients);
        out.append(clips);
        let _ = out.write_str("</defs>\n");
    }
    for part in content {
        out.append(part);
    }
    let _ = out.write_str("</svg>\n");
    out.check()
}

macro_rules! svg_backend_methods {
    ($(fn $name:ident($($arg:ident: $ty:ty),*);)*) => {
        impl Backend for SvgBackend {
            #[cfg(test)]
            fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
                SvgSerializer::draw_text(self, position, text, style)?;
                self.check_buffers()
            }
            #[cfg(test)]
            fn draw_bracket(&mut self, bracket: &SignificanceBracket, style: &BracketStyle) -> Result<(), Error> {
                SvgSerializer::draw_bracket(self, bracket, style)?;
                self.check_buffers()
            }
            #[cfg(not(test))]
            fn draw_text(&mut self, _position: Point2D, _text: &str, _style: &TextStyle) -> Result<(), Error> {
                Err(Error::Render("SVG text publication requires a resolved text plan".into()))
            }
            #[cfg(not(test))]
            fn draw_bracket(&mut self, _bracket: &SignificanceBracket, _style: &BracketStyle) -> Result<(), Error> {
                Err(Error::Render("SVG bracket publication requires a resolved label plan".into()))
            }
            $(fn $name(&mut self, $($arg: $ty),*) -> Result<(), Error> {
                SvgSerializer::$name(self, $($arg),*)?;
                self.check_buffers()
            })*
        }
    };
}

svg_backend_methods! {
    fn draw_canonical_text(text: &CanonicalPlannedText);
    fn draw_canonical_bracket(bracket: &CanonicalPlannedBracket);
    fn draw_canonical_scatter(scatter: &CanonicalScatter);
    fn draw_canonical_line(points: &[CanonicalPoint], style: &CanonicalLineStyle);
    fn draw_canonical_polygon(points: &[CanonicalPoint], style: &CanonicalFillStyle);
    fn draw_canonical_gradient(gradient: &CanonicalGradientFill);
    fn begin_canonical_clip(region: coordinates::CanonicalBox);
    fn draw_canonical_surface(bounds: coordinates::CanonicalBox, paint: Color, role: SurfaceRole);
    fn draw_canonical_rounded_rect(rounded: coordinates::CanonicalRoundedBox);
    fn begin(width: u32, height: u32);
    fn end();
    fn begin_layer(name: &str);
    fn begin_layer_with_role(name: &str, role: LayerRole);
    fn end_layer();
    fn begin_clip(region: PanelBox);
    fn end_clip();
    fn draw_surface(bounds: PanelBox, paint: Color, role: SurfaceRole);
    fn draw_scatter(points: &[Point2D], style: &PointStyle);
    fn draw_line(points: &[Point2D], style: &LineStyle);
    fn draw_polygon(points: &[Point2D], style: &FillStyle);
    fn draw_linear_gradient(bounds: PanelBox, start: Point2D, end: Point2D, stops: &[GradientStop]);
    fn draw_planned_text(placement: PlannedTextPlacement, plan: &ResolvedTextPlan);
    fn draw_rounded_rect(x: f64, y: f64, width: f64, height: f64, radius: f64, style: &FillStyle);
    fn draw_planned_bracket(call: PlannedSignificanceCall<'_>);
}

impl From<(u8, u8, u8)> for SvgPaint {
    fn from(color: (u8, u8, u8)) -> Self {
        Self(color_from_rgb(color))
    }
}

impl From<Color> for SvgPaint {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

impl fmt::Display for SvgPaint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02X}{:02X}{:02X}",
            self.0.red(),
            self.0.green(),
            self.0.blue()
        )
    }
}

/// Rasterize one finalized path-only SVG scene with explicit export options.
pub(crate) fn rasterize_svg_bytes_to_png(
    svg_data: &[u8],
    plan: PngTargetPlan,
    png_options: &PngOptions,
) -> Result<Vec<u8>, Error> {
    crate::scene::validate_png_parser_bytes(svg_data.len())?;
    if contains_svg_text(svg_data) {
        return Err(Error::Render(
            "PNG rasterization requires the finalized path-only SVG scene".into(),
        ));
    }
    let tree = resvg::usvg::Tree::from_data(svg_data, &resvg::usvg::Options::default())
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

fn contains_svg_text(svg_data: &[u8]) -> bool {
    svg_data
        .windows(b"<text".len())
        .any(|window| window == b"<text")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PngTargetPlan {
    source_size: resvg::tiny_skia::IntSize,
    target_size: resvg::tiny_skia::IntSize,
    dpi: Option<u32>,
}

impl PngTargetPlan {
    pub(crate) fn new(
        source_size: resvg::tiny_skia::IntSize,
        options: &PngOptions,
    ) -> Result<Self, Error> {
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

struct PngBuffer(Vec<u8>);

impl std::io::Write for PngBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("PNG output byte count overflow"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(std::io::Error::other)?;
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn encode_png(
    pixmap: resvg::tiny_skia::Pixmap,
    compression: PngCompression,
    dpi: Option<u32>,
) -> Result<Vec<u8>, Error> {
    let mut bytes = PngBuffer(Vec::new());
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
    Ok(bytes.0)
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
    let pixels_per_meter = pixels_per_meter.round();
    if pixels_per_meter < 1.0 {
        return Err(Error::Render(format!(
            "png dpi is too small to encode: {dpi}"
        )));
    }
    if pixels_per_meter > f64::from(u32::MAX) {
        return Err(Error::Render(format!(
            "png dpi is too large to encode: {dpi}"
        )));
    }
    Ok(pixels_per_meter as u32)
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
/// emitted after semantic surfaces but before ordinary content at `end()` time,
/// ensuring grid lines always appear behind data marks without constraining
/// their unique IDs.
pub struct SvgSerializer<S: SvgSink> {
    output: Option<FinalizedSvgOutput>,
    width: u32,
    height: u32,
    body: S,
    /// Buffered Canvas root surfaces (emitted before panel surfaces).
    root_surfaces: S,
    /// Buffered Plot panel surfaces (emitted before grid and body content).
    panel_surfaces: S,
    /// Buffered grid layer content (emitted before body at end).
    grid_buf: S,
    clip_defs: S,
    gradient_defs: S,
    group_stack: Vec<GroupState>,
    next_clip_id: usize,
    next_gradient_id: usize,
    state: SvgCanvasState,
    font_environment: Option<PreparedFontEnvironment>,
    text_replacements: Vec<TextReplacement>,
    replacement_count: usize,
    fixed: bool,
    document: String,
    expected_document_bytes: Option<usize>,
}

/// Concrete canonical SVG writer.
pub type SvgBackend = SvgSerializer<WriteBuffer>;

pub(crate) struct SvgStorage {
    writer: SvgBackend,
}

#[derive(Clone, Copy)]
pub(crate) struct SvgStorageRequirements {
    pub(crate) scopes: usize,
}

impl SvgStorage {
    pub(crate) fn prepare(
        width: u32,
        height: u32,
        environment: PreparedFontEnvironment,
        requirements: SvgStorageRequirements,
        stage: &mut crate::layout::kernel::FinalStage<'_>,
        visit: impl FnOnce(&mut SvgSerializer<CountBuffer>) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        let mut groups = Vec::new();
        reserve_storage(
            stage,
            &mut groups,
            requirements.scopes,
            "canonical SVG scopes",
        )?;
        let mut count = SvgSerializer {
            output: None,
            width,
            height,
            body: CountBuffer::default(),
            root_surfaces: CountBuffer::default(),
            panel_surfaces: CountBuffer::default(),
            grid_buf: CountBuffer::default(),
            clip_defs: CountBuffer::default(),
            gradient_defs: CountBuffer::default(),
            group_stack: groups,
            next_clip_id: 0,
            next_gradient_id: 0,
            state: SvgCanvasState::Open,
            font_environment: None,
            text_replacements: Vec::new(),
            replacement_count: 0,
            fixed: true,
            document: String::new(),
            expected_document_bytes: None,
        };
        visit(&mut count)?;
        count.check_buffers()?;
        if !count.group_stack.is_empty() {
            return Err(Error::Render(
                "canonical SVG count ended with unclosed scopes".into(),
            ));
        }
        let mut document = CountBuffer::default();
        emit_svg_parts(
            &mut document,
            width,
            height,
            &count.gradient_defs,
            &count.clip_defs,
            &[
                &count.root_surfaces,
                &count.panel_surfaces,
                &count.grid_buf,
                &count.body,
            ],
        )?;
        let mut replacements = Vec::new();
        reserve_storage(
            stage,
            &mut replacements,
            count.replacement_count,
            "canonical SVG replacements",
        )?;
        let document_storage =
            WriteBuffer::prepare(document.len(), stage, "canonical SVG document")?;
        Ok(Self {
            writer: SvgSerializer {
                output: None,
                width,
                height,
                body: WriteBuffer::prepare(count.body.len(), stage, "canonical SVG body")?,
                root_surfaces: WriteBuffer::prepare(
                    count.root_surfaces.len(),
                    stage,
                    "canonical SVG root surfaces",
                )?,
                panel_surfaces: WriteBuffer::prepare(
                    count.panel_surfaces.len(),
                    stage,
                    "canonical SVG panel surfaces",
                )?,
                grid_buf: WriteBuffer::prepare(count.grid_buf.len(), stage, "canonical SVG grid")?,
                clip_defs: WriteBuffer::prepare(
                    count.clip_defs.len(),
                    stage,
                    "canonical SVG clips",
                )?,
                gradient_defs: WriteBuffer::prepare(
                    count.gradient_defs.len(),
                    stage,
                    "canonical SVG gradients",
                )?,
                group_stack: count.group_stack,
                next_clip_id: 0,
                next_gradient_id: 0,
                state: SvgCanvasState::Fresh,
                font_environment: Some(environment),
                text_replacements: replacements,
                replacement_count: 0,
                fixed: true,
                document: document_storage.text,
                expected_document_bytes: Some(document.len()),
            },
        })
    }

    pub(crate) fn into_backend(self) -> SvgBackend {
        self.writer
    }
}

impl SvgBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        #[cfg(test)]
        SVG_BEGIN_CALLS.with(|calls| calls.set(calls.get() + 1));
        self.output = None;
        if self.state == SvgCanvasState::Open {
            return Err(Error::Render(
                "svg backend begin called while a canvas is already open".into(),
            ));
        }
        if self.fixed && self.state == SvgCanvasState::Closed {
            return Err(Error::Render(
                "prepared SVG storage permits only one concrete lifecycle".into(),
            ));
        }
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "svg backend canvas dimensions must be positive".into(),
            ));
        }
        if self.fixed && (width, height) != (self.width, self.height) {
            return Err(Error::Render(
                "SVG dimensions must match prepared storage".into(),
            ));
        }
        self.width = width;
        self.height = height;
        self.body.clear();
        self.root_surfaces.clear();
        self.panel_surfaces.clear();
        self.grid_buf.clear();
        self.clip_defs.clear();
        self.gradient_defs.clear();
        self.group_stack.clear();
        self.next_clip_id = 0;
        self.next_gradient_id = 0;
        self.text_replacements.clear();
        self.replacement_count = 0;
        self.state = SvgCanvasState::Open;
        Ok(())
    }
}

impl<S: SvgSink> SvgSerializer<S> {
    pub(crate) fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.begin_layer_with_role(name, LayerRole::Generic)
    }

    pub(crate) fn begin_layer_with_role(
        &mut self,
        name: &str,
        role: LayerRole,
    ) -> Result<(), Error> {
        self.ensure_begun("begin_layer")?;
        if role == LayerRole::Grid {
            if !self.group_stack.is_empty() {
                return Err(Error::Render(
                    "svg grid layer must begin at the top level".into(),
                ));
            }
            let escaped = Self::escape_text(name);
            let _ = writeln!(self.grid_buf, "<g id=\"{escaped}\">");
            self.push_group(GroupState::Grid)?;
        } else {
            let escaped = Self::escape_text(name);
            let _ = writeln!(self.buf(), "<g id=\"{escaped}\">");
            self.push_group(GroupState::Layer)?;
        }
        Ok(())
    }

    pub(crate) fn end_layer(&mut self) -> Result<(), Error> {
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

    pub(crate) fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.ensure_begun("begin_clip")?;
        let region = validate_clip_region(region)?;
        self.begin_canonical_clip(coordinates::CanonicalBox::from_bottom_up(
            region,
            self.height,
        )?)
    }

    pub(crate) fn begin_canonical_clip(
        &mut self,
        region: coordinates::CanonicalBox,
    ) -> Result<(), Error> {
        self.ensure_begun("begin_clip")?;
        let id = self.next_clip_id;
        self.next_clip_id = self
            .next_clip_id
            .checked_add(1)
            .ok_or_else(|| Error::Render("SVG clip identifier overflow".into()))?;
        let _ = writeln!(
            self.clip_defs,
            "<clipPath id=\"clip-{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>",
            region.x, region.y, region.width, region.height,
        );
        let _ = writeln!(self.buf(), "<g clip-path=\"url(#clip-{id})\">");
        self.push_group(GroupState::Clip)?;
        Ok(())
    }

    pub(crate) fn end_clip(&mut self) -> Result<(), Error> {
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

    pub(crate) fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: Color,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_surface")?;
        let bounds = validate_surface_bounds(bounds)?;
        self.draw_canonical_surface(
            coordinates::CanonicalBox::from_bottom_up(bounds, self.height)?,
            paint,
            role,
        )
    }

    pub(crate) fn draw_canonical_surface(
        &mut self,
        bounds: coordinates::CanonicalBox,
        paint: Color,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_surface")?;
        let paint = SvgPaint(paint).attributes("fill");
        if role == SurfaceRole::CanvasRoot
            && bounds.x.units() == 0
            && bounds.y.units() == 0
            && bounds.width.units() == i64::from(self.width) * 1024
            && bounds.height.units() == i64::from(self.height) * 1024
        {
            let _ = writeln!(
                self.root_surfaces,
                "<rect width=\"100%\" height=\"100%\" {paint}/>"
            );
        } else {
            let buffer = match role {
                SurfaceRole::CanvasRoot => &mut self.root_surfaces,
                SurfaceRole::PlotPanel => &mut self.panel_surfaces,
            };
            let _ = writeln!(
                buffer,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {paint}/>",
                bounds.x, bounds.y, bounds.width, bounds.height,
            );
        }
        Ok(())
    }

    pub(crate) fn draw_scatter(
        &mut self,
        points: &[Point2D],
        style: &PointStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_scatter")?;
        let scatter = CanonicalScatter::new(points, style, self.height, None)?;
        self.draw_canonical_scatter(&scatter)
    }

    pub(crate) fn draw_canonical_scatter(
        &mut self,
        scatter: &CanonicalScatter,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_scatter")?;
        let color = SvgPaint::from(scatter.color).attributes("fill");
        let outline = scatter
            .outline
            .map(|(color, width)| (SvgPaint::from(color).attributes("stroke"), width));
        match &scatter.markers {
            CanonicalMarkers::Circles { points, radius } => {
                let buf = self.buf();
                for p in points {
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" {color} {outline_color} stroke-width=\"{outline_width}\"/>",
                            p.x, p.y, radius
                        );
                    } else {
                        let _ = writeln!(
                            buf,
                            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" {color}/>",
                            p.x, p.y, radius
                        );
                    }
                }
            }
            CanonicalMarkers::Triangles(triangles) => {
                for triangle in triangles {
                    let vertices = PointTokens(triangle);
                    if let Some((outline_color, outline_width)) = &outline {
                        let _ = writeln!(
                            self.buf(),
                            "<polygon points=\"{vertices}\" {color} {outline_color} stroke-width=\"{outline_width}\" stroke-linejoin=\"miter\" stroke-miterlimit=\"{POLYGON_MITER_LIMIT:.1}\"/>",
                        );
                    } else {
                        let _ = writeln!(self.buf(), "<polygon points=\"{vertices}\" {color}/>");
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_begun("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        let points = primitives::path(points, PathKind::Line, self.height, None)?;
        let style = CanonicalLineStyle::new(style)?;
        self.draw_canonical_line(&points, &style)
    }

    pub(crate) fn draw_canonical_line(
        &mut self,
        points: &[CanonicalPoint],
        style: &CanonicalLineStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_line")?;
        let stroke = SvgPaint::from(style.color).attributes("stroke");
        let pts_str = PointTokens(points);
        let buf = self.buf();
        if let Some(dasharray) = style.pattern.svg_dasharray() {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" {stroke} stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"{dasharray}\"/>",
                style.width,
            );
        } else {
            let _ = writeln!(
                buf,
                "<polyline points=\"{pts_str}\" fill=\"none\" {stroke} stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                style.width
            );
        }
        Ok(())
    }

    pub(crate) fn draw_polygon(
        &mut self,
        points: &[Point2D],
        style: &FillStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        let points = primitives::path(points, PathKind::Polygon, self.height, None)?;
        let style = CanonicalFillStyle::new(style)?;
        self.draw_canonical_polygon(&points, &style)
    }

    pub(crate) fn draw_canonical_polygon(
        &mut self,
        points: &[CanonicalPoint],
        style: &CanonicalFillStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_polygon")?;
        let fill = SvgPaint::from(style.color).attributes("fill");
        let stroke = SvgPaint::from(style.stroke).attributes("stroke");
        let pts_str = PointTokens(points);
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<polygon points=\"{pts_str}\" {fill} {stroke} stroke-width=\"{}\" stroke-linejoin=\"miter\" stroke-miterlimit=\"{POLYGON_MITER_LIMIT:.1}\"/>",
            style.stroke_width,
        );
        Ok(())
    }

    pub(crate) fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.ensure_begun("draw_linear_gradient")?;
        let gradient = CanonicalGradientFill::new(bounds, start, end, stops, self.height, None)?;
        self.draw_canonical_gradient(&gradient)
    }

    pub(crate) fn draw_canonical_gradient(
        &mut self,
        gradient: &CanonicalGradientFill,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_linear_gradient")?;
        let id = self.next_gradient_id;
        self.next_gradient_id = self
            .next_gradient_id
            .checked_add(1)
            .ok_or_else(|| Error::Render("SVG gradient identifier overflow".into()))?;
        let _ = writeln!(
            self.gradient_defs,
            "<linearGradient id=\"linear-gradient-{id}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">",
            gradient.start.x, gradient.start.y, gradient.end.x, gradient.end.y,
        );
        for stop in &gradient.stops {
            let offset = stop.offset;
            let color = SvgPaint::from(stop.color);
            let _ = writeln!(
                self.gradient_defs,
                "<stop offset=\"{offset}\" stop-color=\"{color}\"/>"
            );
        }
        let _ = writeln!(self.gradient_defs, "</linearGradient>");
        let _ = writeln!(
            self.buf(),
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#linear-gradient-{id})\"/>",
            gradient.bounds.x,
            gradient.bounds.y,
            gradient.bounds.width,
            gradient.bounds.height,
        );
        Ok(())
    }

    pub(crate) fn draw_rounded_rect(
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
        self.draw_canonical_rounded_rect(coordinates::CanonicalRoundedBox::from_bottom_up(
            rounded,
            style,
            self.height,
        )?)
    }

    pub(crate) fn draw_canonical_rounded_rect(
        &mut self,
        rounded: coordinates::CanonicalRoundedBox,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_rounded_rect")?;
        let fill = SvgPaint::from(rounded.color).attributes("fill");
        let stroke = SvgPaint::from(rounded.stroke).attributes("stroke");
        let buf = self.buf();
        let _ = writeln!(
            buf,
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" ry=\"{}\" {fill} {stroke} stroke-width=\"{}\"/>",
            rounded.bounds.x,
            rounded.bounds.y,
            rounded.bounds.width,
            rounded.bounds.height,
            rounded.radius,
            rounded.radius,
            rounded.stroke_width,
        );
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn draw_text(
        &mut self,
        position: Point2D,
        text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        #[cfg(test)]
        if !S::COUNTING {
            SVG_RAW_TEXT_CALLS.with(|calls| calls.set(calls.get() + 1));
        }
        self.ensure_begun("draw_text")?;
        let canonical = validate_text(position, style)?;
        let fill = SvgPaint::from(style.color).attributes("fill");
        let escaped = Self::escape_text(text);
        let x = canonical.position.x;
        let y = canonical.position.y.inverted(self.height)?;
        let anchor = match style.anchor {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        };
        let weight = FontWeight(style.weight);
        let preserve_space = if text.contains("  ") {
            " xml:space=\"preserve\""
        } else {
            ""
        };
        let buf = self.buf();
        if canonical.rotation.is_rotated() {
            let _ = writeln!(
                buf,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{preserve_space} {fill} text-anchor=\"{anchor}\" dominant-baseline=\"middle\" transform=\"rotate({},{},{})\">{escaped}</text>",
                x,
                y,
                Self::escape_text(&style.font_family.css().map_err(|error| Error::Render(error.to_string()))?),
                style.font_size,
                canonical.rotation,
                x,
                y
            );
        } else {
            let _ = writeln!(
                buf,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{preserve_space} {fill} text-anchor=\"{anchor}\" dominant-baseline=\"middle\">{escaped}</text>",
                x,
                y,
                Self::escape_text(&style.font_family.css().map_err(|error| Error::Render(error.to_string()))?),
                style.font_size
            );
        }
        Ok(())
    }

    pub(crate) fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_planned_text")?;
        let text = CanonicalPlannedText::new(placement, plan, self.height, None)?;
        self.draw_canonical_text(&text)
    }

    pub(crate) fn draw_canonical_text(&mut self, text: &CanonicalPlannedText) -> Result<(), Error> {
        #[cfg(test)]
        if !S::COUNTING {
            SVG_PLANNED_TEXT_CALLS.with(|calls| calls.set(calls.get() + 1));
        }
        self.ensure_begun("draw_planned_text")?;
        self.draw_multiline_planned_text(text)
    }

    #[cfg(test)]
    pub(crate) fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_bracket")?;
        let canonical = validate_bracket(bracket, style)?;
        let stroke = SvgPaint::from(style.color).attributes("stroke");
        let fill = SvgPaint::from(style.label_color).attributes("fill");
        let label = Self::escape_text(&bracket.label);
        let css = style
            .font_family
            .css()
            .map_err(|error| Error::Render(error.to_string()))?;
        let font_family = Self::escape_text(&css);
        let weight = FontWeight(style.weight);
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
                    "<path class=\"significance-bracket\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\" fill=\"none\" {stroke} stroke-width=\"{}\" stroke-linecap=\"round\"/>",
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
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"{font_family}\" font-size=\"{}\"{weight} {fill}>{label}</text>",
                    canonical.label_position.x, label_y, style.font_size,
                );
            }
            super::backend::BracketOrientation::Horizontal => {
                let _ = writeln!(
                    buf,
                    "<path class=\"significance-bracket horizontal\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\" fill=\"none\" {stroke} stroke-width=\"{}\" stroke-linecap=\"round\"/>",
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
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"start\" dominant-baseline=\"middle\" font-family=\"{font_family}\" font-size=\"{}\"{weight} {fill}>{label}</text>",
                    canonical.label_position.x, label_y, style.font_size,
                );
            }
        }
        Ok(())
    }

    pub(crate) fn draw_planned_bracket(
        &mut self,
        call: PlannedSignificanceCall<'_>,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_planned_bracket")?;
        let bracket = CanonicalPlannedBracket::new(call, self.height, None)?;
        self.draw_canonical_bracket(&bracket)
    }

    pub(crate) fn draw_canonical_bracket(
        &mut self,
        bracket: &CanonicalPlannedBracket,
    ) -> Result<(), Error> {
        self.ensure_begun("draw_planned_bracket")?;
        let stroke = SvgPaint::from(bracket.color).attributes("stroke");
        let suffix = match bracket.orientation {
            super::backend::BracketOrientation::Vertical => "",
            super::backend::BracketOrientation::Horizontal => " horizontal",
        };
        let [start, end, first, second] = bracket.points;
        let _ = writeln!(self.buf(),
            "<path class=\"significance-bracket{suffix}\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\" fill=\"none\" {stroke} stroke-width=\"{}\" stroke-linecap=\"round\"/>",
            start.x, start.y, end.x, end.y, start.x, start.y, first.x, first.y, end.x, end.y, second.x, second.y, bracket.width);
        self.draw_canonical_text(&bracket.text)
    }
}

impl SvgBackend {
    fn end(&mut self) -> Result<(), Error> {
        #[cfg(test)]
        SVG_END_CALLS.with(|calls| calls.set(calls.get() + 1));
        if self.state != SvgCanvasState::Open {
            return Err(Error::Render("svg backend ended before begin".into()));
        }
        if !self.group_stack.is_empty() {
            return Err(Error::Render(format!(
                "svg backend ended with {} unclosed layer/clip scopes",
                self.group_stack.len()
            )));
        }
        self.check_buffers()?;
        let mut svg = WriteBuffer {
            text: std::mem::take(&mut self.document),
            fixed: self.fixed,
            failed: false,
        };
        emit_svg_parts(
            &mut svg,
            self.width,
            self.height,
            &self.gradient_defs,
            &self.clip_defs,
            &[
                &self.root_surfaces,
                &self.panel_surfaces,
                &self.grid_buf,
                &self.body,
            ],
        )?;
        if self
            .expected_document_bytes
            .is_some_and(|expected| expected != svg.len())
        {
            return Err(Error::Render(
                "canonical SVG differs from counted byte size".into(),
            ));
        }
        let outline_plan = SvgOutlinePlan::Ready(SvgOutlineSource {
            font_environment: self.font_environment.clone(),
            root_surfaces: std::mem::take(&mut self.root_surfaces).text,
            panel_surfaces: std::mem::take(&mut self.panel_surfaces).text,
            grid: std::mem::take(&mut self.grid_buf).text,
            body: std::mem::take(&mut self.body).text,
            gradient_defs: std::mem::take(&mut self.gradient_defs).text,
            clip_defs: std::mem::take(&mut self.clip_defs).text,
            replacements: std::mem::take(&mut self.text_replacements),
        });
        self.output = Some(FinalizedSvgOutput {
            width_css_px: self.width,
            height_css_px: self.height,
            bytes: svg.text.into_bytes(),
            outline_plan,
        });
        self.state = SvgCanvasState::Closed;
        Ok(())
    }
}

fn apply_text_replacements<S: SvgSink>(
    source: &str,
    buffer: TextBuffer,
    replacements: &[TextReplacement],
    font_environment: Option<&PreparedFontEnvironment>,
    outlined: &mut S,
) -> Result<(), String> {
    let mut cursor = 0_usize;
    for replacement in replacements
        .iter()
        .filter(|replacement| replacement.buffer == buffer)
    {
        if replacement.range.start < cursor
            || replacement.range.end < replacement.range.start
            || replacement.range.end > source.len()
        {
            return Err("planned SVG text replacement ranges are invalid".into());
        }
        let _ = outlined.write_str(&source[cursor..replacement.range.start]);
        outlined.check().map_err(|error| error.to_string())?;
        if let Some(outline) = &replacement.outline {
            let environment = font_environment.ok_or_else(|| {
                "planned SVG text was serialized without its retained font environment".to_owned()
            })?;
            SvgBackend::outline_planned_line(environment, outline, outlined)
                .map_err(|error| error.to_string())?;
        }
        cursor = replacement.range.end;
    }
    let _ = outlined.write_str(&source[cursor..]);
    outlined.check().map_err(|error| error.to_string())
}

fn emit_svg_header(svg: &mut impl fmt::Write, width: u32, height: u32) -> fmt::Result {
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\">"
    )?;
    writeln!(
        svg,
        "<style>line,path,polyline,polygon,circle{{vector-effect:non-scaling-stroke}}</style>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::PreparedFontEnvironment;
    use crate::layout::{PhysicalTextAlignment, TextLineHeight, TextPlanInput, TextPlanProvenance};

    #[test]
    fn shared_svg_tokens_count_escaped_utf8_and_refuse_overflow() {
        let escaped = EscapedText("<&\"'\u{03b1}");
        let attribute = EscapedAttribute("\t\n\r<&");
        let paint = SvgPaint(Color::rgba(1, 2, 3, 128)).attributes("fill");
        let mut count = CountBuffer::default();
        write!(count, "{escaped} {paint} {attribute}").unwrap();
        let mut output = WriteBuffer::default();
        write!(output, "{escaped} {paint} {attribute}").unwrap();
        assert_eq!(count.len(), output.len());
        assert!(output.starts_with("&lt;&amp;&quot;&apos;"));
        assert!(output.ends_with("&#9;&#10;&#13;&lt;&amp;"));
        let mut overflow = CountBuffer {
            bytes: usize::MAX,
            ..CountBuffer::default()
        };
        assert!(overflow.write_str("x").is_err());
        assert_eq!(overflow.len(), usize::MAX);
    }

    #[test]
    fn formatter_errors_remain_latched_through_enclosing_serialization() {
        struct FailingDisplay;
        impl fmt::Display for FailingDisplay {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("prefix")?;
                Err(fmt::Error)
            }
        }

        let mut count = CountBuffer::default();
        assert!(write!(count, "{FailingDisplay}").is_err());
        count.write_str("suffix").unwrap();
        assert_eq!(count.len(), 12);
        assert!(!count.limit_exceeded);
        assert!(count.check().is_err());
        assert!(emit_svg_parts(
            &mut count,
            100,
            100,
            &CountBuffer::default(),
            &CountBuffer::default(),
            &[]
        )
        .is_err());

        let mut buffer = WriteBuffer {
            text: String::with_capacity(32),
            fixed: true,
            failed: false,
        };
        assert!(write!(buffer, "{FailingDisplay}").is_err());
        buffer.write_str("suffix").unwrap();
        assert_eq!(&*buffer, "prefixsuffix");
        assert!(buffer.len() < buffer.text.capacity());
        assert!(buffer.check().is_err());
        let mut svg = SvgBackend::in_memory(100, 100);
        Backend::begin(&mut svg, 100, 100).unwrap();
        svg.body = buffer;
        assert!(Backend::end(&mut svg).is_err());
        assert!(svg.into_output().is_none());
    }

    #[test]
    fn fixed_primitive_and_document_overflows_reject_enclosing_svg_output() {
        let points = [Point2D::new(10.0, 20.0), Point2D::new(30.0, 40.0)];
        let style = LineStyle::solid(Color::rgb(1, 2, 3), 1.0);
        let mut svg = SvgBackend::in_memory(100, 100);
        Backend::begin(&mut svg, 100, 100).unwrap();
        svg.body.fixed = true;
        assert!(Backend::draw_line(&mut svg, &points, &style).is_err());
        assert!(svg.body.failed);
        assert_eq!(svg.body.text.capacity(), 0);
        assert!(Backend::end(&mut svg).is_err());
        assert!(svg.into_output().is_none());

        let mut reference = SvgBackend::in_memory(100, 100);
        Backend::begin(&mut reference, 100, 100).unwrap();
        Backend::draw_line(&mut reference, &points, &style).unwrap();
        Backend::end(&mut reference).unwrap();
        let required = reference.into_bytes().unwrap().len();
        let mut svg = SvgBackend::in_memory(100, 100);
        Backend::begin(&mut svg, 100, 100).unwrap();
        Backend::draw_line(&mut svg, &points, &style).unwrap();
        svg.document = String::with_capacity(required - 1);
        assert!(svg.document.capacity() < required);
        svg.fixed = true;
        let error = Backend::end(&mut svg).unwrap_err();
        assert!(error.to_string().contains("reserved storage"));
        assert!(svg.into_output().is_none());
    }

    #[test]
    fn fixed_glyph_and_outline_overflows_reject_derived_output() {
        let environment = PreparedFontEnvironment::synthetic();
        let plan = environment
            .plan_text(TextPlanInput {
                source: "A",
                stack: crate::font::canonical::test_sans_stack(),
                weight: crate::TextWeight::Normal,
                font_size: 12.0,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: Color::rgb(1, 2, 3),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "outline overflow".into(),
                },
                exact_override: false,
            })
            .unwrap();
        let placement =
            PlannedTextPlacement::middle(Point2D::new(8.0, 20.0), TextAnchor::Start, 0.0);
        let outline = PlannedLineOutline {
            text: CanonicalPlannedText::new(placement, &plan, 40, None).unwrap(),
            line_index: 0,
        };
        let mut buffer = WriteBuffer {
            fixed: true,
            ..WriteBuffer::default()
        };
        let error =
            SvgBackend::outline_planned_line(&environment, &outline, &mut buffer).unwrap_err();
        assert!(error.to_string().contains("reserved storage"));
        assert!(buffer.failed);
        assert_eq!(buffer.text.capacity(), 0);

        let mut svg = SvgBackend::in_memory_with_fonts(100, 40, environment);
        Backend::begin(&mut svg, 100, 40).unwrap();
        Backend::draw_planned_text(&mut svg, placement, &plan).unwrap();
        Backend::end(&mut svg).unwrap();
        let output = svg.into_output().unwrap();
        let required = output.outline_plan.count_bytes(100, 40).unwrap();
        let error = output
            .outline_plan
            .derive_counted(100, 40, required - 1)
            .unwrap_err();
        assert!(error.contains("reserved storage"), "{error}");
        assert_eq!(
            output
                .outline_plan
                .derive_counted(100, 40, required)
                .unwrap()
                .len(),
            required
        );
    }

    #[test]
    fn canonical_quantum_spelling_is_exact_and_minimal() {
        use coordinates::CanonicalCoordinate;
        assert_eq!(CanonicalCoordinate::from_quanta(0).to_string(), "0");
        assert_eq!(
            CanonicalCoordinate::from_quanta(1).to_string(),
            "0.0009765625"
        );
        assert_eq!(CanonicalCoordinate::from_quanta(512).to_string(), "0.5");
        assert_eq!(
            CanonicalCoordinate::from_quanta(1023).to_string(),
            "0.9990234375"
        );
        assert_eq!(CanonicalCoordinate::from_quanta(-1536).to_string(), "-1.5");
        assert_eq!(
            CanonicalCoordinate::from_quanta(i64::MIN).to_string(),
            "-9007199254740992"
        );
    }
}
