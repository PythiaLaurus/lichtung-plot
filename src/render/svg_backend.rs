//! Native SVG backend for deterministic publication-oriented output.

use std::fmt::{self, Write as _};
use std::ops::Range;

use crate::Error;

use super::backend::{
    validate_bracket, validate_clip_region, validate_fill_style, validate_rounded_rect,
    validate_surface_bounds, validate_text, Backend, BracketStyle, FillStyle, GradientStop,
    LayerRole, LineStyle, PlannedSignificanceCall, PlannedTextPlacement, Point2D, PointStyle, Rgba,
    SignificanceBracket, SurfaceRole, TextAnchor, TextStyle, POLYGON_MITER_LIMIT,
};
use super::coordinates::{self, CanonicalPoint};
use super::primitives::{
    self, CanonicalFillStyle, CanonicalGradientFill, CanonicalLineStyle, CanonicalMarkers,
    CanonicalPlannedBracket, CanonicalPlannedText, CanonicalScatter, PathKind,
};
use crate::font::canonical::{PreparedFontEnvironment, ShapingDirection};
#[cfg(test)]
use crate::layout::CSS_PIXEL_QUANTA;
use crate::layout::{PanelBox, ResolvedTextPlan};
use crate::spec::{PngCompression, PngOptions, PngSizing, SVG_CSS_DPI};
#[cfg(test)]
use crate::PointShape;

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

    #[cfg(test)]
    pub(crate) fn into_outlined_bytes(self) -> Result<Vec<u8>, Error> {
        let output = self.into_output().ok_or_else(|| {
            Error::Render("in-memory SVG backend did not finish rendering".into())
        })?;
        output
            .outline_plan
            .derive(output.width_css_px, output.height_css_px)
            .map_err(Error::Render)
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
                    let scale = f64::from(text.plan.style.font_size) / f64::from(run.units_per_em);
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

    #[cfg(test)]
    pub(crate) fn from_test_bytes(width_css_px: u32, height_css_px: u32, bytes: &[u8]) -> Self {
        Self {
            width_css_px,
            height_css_px,
            bytes: bytes.into(),
            outline_plan: SvgOutlinePlan::Failed(
                "parse svg for text outlining: invalid finalized test SVG".into(),
            ),
        }
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
struct SvgPaint(Rgba);

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
        if paint.0.alpha() != 255 {
            let opacity = f64::from(paint.0.alpha()) / 255.0;
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
    fn draw_canonical_surface(bounds: coordinates::CanonicalBox, paint: Rgba, role: SurfaceRole);
    fn draw_canonical_rounded_rect(rounded: coordinates::CanonicalRoundedBox);
    fn begin(width: u32, height: u32);
    fn end();
    fn begin_layer(name: &str);
    fn begin_layer_with_role(name: &str, role: LayerRole);
    fn end_layer();
    fn begin_clip(region: PanelBox);
    fn end_clip();
    fn draw_surface(bounds: PanelBox, paint: Rgba, role: SurfaceRole);
    fn draw_scatter(points: &[Point2D], style: &PointStyle);
    fn draw_line(points: &[Point2D], style: &LineStyle);
    fn draw_polygon(points: &[Point2D], style: &FillStyle);
    fn draw_linear_gradient(bounds: PanelBox, start: Point2D, end: Point2D, stops: &[GradientStop]);
    fn draw_text(position: Point2D, text: &str, style: &TextStyle);
    fn draw_planned_text(placement: PlannedTextPlacement, plan: &ResolvedTextPlan);
    fn draw_rounded_rect(x: f64, y: f64, width: f64, height: f64, radius: f64, style: &FillStyle);
    fn draw_bracket(bracket: &SignificanceBracket, style: &BracketStyle);
    fn draw_planned_bracket(call: PlannedSignificanceCall<'_>);
}

impl From<(u8, u8, u8)> for SvgPaint {
    fn from(color: (u8, u8, u8)) -> Self {
        Self(color.into())
    }
}

impl From<Rgba> for SvgPaint {
    fn from(color: Rgba) -> Self {
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
        paint: Rgba,
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
        paint: Rgba,
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
                Self::escape_text(&style.font_family),
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
                Self::escape_text(&style.font_family),
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
        let font_family = Self::escape_text(&style.font_family);
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
        let paint = SvgPaint(Rgba::new(1, 2, 3, 128)).attributes("fill");
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
        let style = LineStyle::solid((1, 2, 3), 1.0);
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
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 12,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: Rgba::rgb(1, 2, 3),
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
        assert!(
            PngTargetPlan::new(source_size(16_385, 1), &options(PngSizing::Scale(2.0)))
                .unwrap_err()
                .to_string()
                .contains("conversion limit")
        );
        assert!(
            PngTargetPlan::new(source_size(10_000, 5_001), &options(PngSizing::Scale(2.0)))
                .unwrap_err()
                .to_string()
                .contains("work")
        );
    }

    #[test]
    fn png_dpi_plan_preserves_f32_sizing_and_checks_target_before_metadata() {
        assert_eq!(target_size(10_000, 10_000, PngSizing::Dpi(4.2)), (438, 438));

        let error = PngTargetPlan::new(
            source_size(10_000, 10_000),
            &PngOptions {
                sizing: PngSizing::Dpi(0.01),
                compression: PngCompression::Balanced,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("too small to encode"));

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
            color: (0, 0, 0).into(),
            font_family: "sans-serif".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Start,
            rotation: 0.0,
        }
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

    fn test_fill_style() -> FillStyle {
        FillStyle::new(Rgba::rgb(1, 2, 3), Rgba::rgb(4, 5, 6), 0.0)
    }

    #[test]
    fn multiline_planned_text_serializes_stored_lines_without_raw_svg_text_calls() {
        let plan = PreparedFontEnvironment::synthetic()
            .plan_text(TextPlanInput {
                source: "A\nA",
                stack: "Test Sans",
                weight: crate::TextWeight::Semibold,
                font_size: 12,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Center,
                paint: (1, 2, 3).into(),
                provenance: TextPlanProvenance {
                    pair_ordinal: 1,
                    owner: "multiline SVG test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        let placement =
            PlannedTextPlacement::middle(Point2D::new(50.0, 50.0), TextAnchor::Middle, 0.0);
        SVG_RAW_TEXT_CALLS.with(|calls| calls.set(0));
        SVG_PLANNED_TEXT_CALLS.with(|calls| calls.set(0));
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();

        backend.draw_planned_text(placement, &plan).unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();

        SVG_RAW_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        SVG_PLANNED_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        let lines = svg
            .lines()
            .filter(|line| line.starts_with("<text "))
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with(">A</text>"));
        assert!(lines[1].ends_with(">A</text>"));
        assert_ne!(lines[0], lines[1]);
        assert!(lines
            .iter()
            .all(|line| line.contains("font-family=\"Test Sans\"")
                && line.contains("font-size=\"12\"")
                && line.contains("font-weight=\"600\"")
                && line.contains("fill=\"#010203\"")));
    }

    #[test]
    fn rtl_preserve_text_uses_physical_left_anchor() {
        let mut plan = PreparedFontEnvironment::synthetic()
            .plan_text(TextPlanInput {
                source: "A",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 12,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: (1, 2, 3).into(),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "RTL SVG anchor test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        plan.edit_for_test().lines[0].base_direction = ShapingDirection::RightToLeft;
        let mut backend = SvgBackend::in_memory(100, 40);
        backend.begin(100, 40).unwrap();
        backend
            .draw_planned_text(
                PlannedTextPlacement::middle(Point2D::new(8.0, 20.0), TextAnchor::Start, 0.0),
                &plan,
            )
            .unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();

        assert!(svg.contains("text-anchor=\"end\" direction=\"rtl\""));
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
        let latest = String::from_utf8(backend.output.as_ref().unwrap().bytes.clone()).unwrap();
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
        assert!(svg.contains("points=\"0,100 0.0009765625,99.9990234375\""));
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
        assert!(svg.contains("points=\"10.0009765625,79 9.134765625,80.5 10.8662109375,80.5\""));
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
                "<circle cx=\"10\" cy=\"80\" r=\"2\" fill=\"#000000\" fill-opacity=\"0.8196078431372549\"/>",
                "<polygon points=\"20,78 18.267578125,81 21.732421875,81\" fill=\"#010203\" fill-opacity=\"0.8196078431372549\"/>",
                "<circle cx=\"15\" cy=\"80\" r=\"2\" fill=\"#070809\" fill-opacity=\"0.8196078431372549\" stroke=\"#FFFFFF\" stroke-width=\"0.2001953125\"/>",
                "<polygon points=\"30,78 28.267578125,81 31.732421875,81\" fill=\"#040506\" fill-opacity=\"0.8196078431372549\" stroke=\"#FFFFFF\" stroke-width=\"0.2001953125\" stroke-linejoin=\"miter\" stroke-miterlimit=\"4.0\"/>",
                "<polygon points=\"10,90 20,90 10,80\" fill=\"#010203\" stroke=\"#040506\" stroke-width=\"0\" stroke-linejoin=\"miter\" stroke-miterlimit=\"4.0\"/>",
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
                &FillStyle::new(Rgba::rgb(0, 0, 0), Rgba::rgb(0, 0, 0), f32::NAN),
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
                "<polyline points=\"1,99 2,98\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                "<polyline points=\"1,99 2,98\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"3 2\"/>",
                "<polyline points=\"1,99 2,98\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"4 3\"/>",
                "<polyline points=\"1,99 2,98\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.25\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-dasharray=\"5 3\"/>",
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
                &FillStyle::new(Rgba::new(1, 2, 3, 64), Rgba::rgb(4, 5, 6), 1.5),
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
            [
                "<rect x=\"10\" y=\"74\" width=\"30\" height=\"20\" rx=\"3\" ry=\"3\" fill=\"#010203\" fill-opacity=\"0.25098039215686274\" stroke=\"#040506\" stroke-width=\"1.5\"/>"
            ]
        );
        assert!(!svg.contains("<polygon"));
    }

    #[test]
    fn filled_paths_emit_independent_fill_and_stroke_alphas() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        let style = FillStyle::new(Rgba::new(1, 2, 3, 128), Rgba::new(4, 5, 6, 64), 1.0);
        backend
            .draw_polygon(
                &[
                    Point2D::new(10.0, 10.0),
                    Point2D::new(20.0, 10.0),
                    Point2D::new(10.0, 20.0),
                ],
                &style,
            )
            .unwrap();
        backend
            .draw_rounded_rect(30.0, 10.0, 20.0, 20.0, 2.0, &style)
            .unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        for tag in svg
            .lines()
            .filter(|line| line.starts_with("<polygon ") || line.starts_with("<rect "))
        {
            assert!(tag.contains("fill=\"#010203\" fill-opacity=\"0.5019607843137255\""));
            assert!(tag.contains("stroke=\"#040506\" stroke-opacity=\"0.25098039215686274\""));
            assert!(!tag.contains(" opacity="));
        }
    }

    #[test]
    fn outlined_points_emit_independent_fill_and_outline_alphas() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        let mut style = PointStyle::with_outline(
            (1, 2, 3),
            2,
            Some(crate::render::backend::PointOutline {
                color: Rgba::new(4, 5, 6, 64),
                width_ratio: 0.5,
            }),
        );
        style.color = Rgba::new(1, 2, 3, 128);
        backend
            .draw_scatter(&[Point2D::new(10.0, 10.0)], &style)
            .unwrap();
        backend
            .draw_scatter(
                &[Point2D::new(30.0, 30.0)],
                &style.with_shape(crate::PointShape::TriangleUp),
            )
            .unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        for tag in svg
            .lines()
            .filter(|line| line.starts_with("<circle ") || line.starts_with("<polygon "))
        {
            assert!(tag.contains("fill=\"#010203\" fill-opacity=\"0.5019607843137255\""));
            assert!(tag.contains("stroke=\"#040506\" stroke-opacity=\"0.25098039215686274\""));
        }
    }

    #[test]
    fn xml_special_characters_are_escaped_in_every_current_context() {
        let special = "<&\"'>";
        let escaped = "&lt;&amp;&quot;&apos;&gt;";
        let mut text_style = test_text_style();
        text_style.font_family = special.into();
        let bracket_style = BracketStyle {
            color: (0, 0, 0).into(),
            width: 1.0,
            label_color: (0, 0, 0).into(),
            font_family: special.into(),
            font_size: 12,
            line_height: 14.4,
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
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"2\" height=\"3\" viewBox=\"0 0 2 3\" role=\"img\">\n<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>\n</svg>\n"
        );
    }

    #[test]
    fn opaque_and_transparent_surfaces_have_exact_role_geometry_and_paint() {
        let mut backend = SvgBackend::in_memory(100, 80);
        backend.begin(100, 80).unwrap();
        backend
            .draw_surface(
                PanelBox::new(10.0, 30.0, 20.0, 40.0),
                Rgba::rgb(7, 8, 9),
                SurfaceRole::PlotPanel,
            )
            .unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 100.0, 0.0, 80.0),
                Rgba::rgb(1, 2, 3),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        backend
            .draw_surface(
                PanelBox::new(12.0, 32.0, 22.0, 42.0),
                Rgba::new(10, 11, 12, 0),
                SurfaceRole::PlotPanel,
            )
            .unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 100.0, 0.0, 80.0),
                Rgba::new(4, 5, 6, 0),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let surfaces = svg
            .lines()
            .filter(|line| line.starts_with("<rect "))
            .collect::<Vec<_>>();
        assert_eq!(
            surfaces,
            [
                "<rect width=\"100%\" height=\"100%\" fill=\"#010203\"/>",
                "<rect width=\"100%\" height=\"100%\" fill=\"#040506\" fill-opacity=\"0\"/>",
                "<rect x=\"10\" y=\"40\" width=\"20\" height=\"20\" fill=\"#070809\"/>",
                "<rect x=\"12\" y=\"38\" width=\"20\" height=\"20\" fill=\"#0A0B0C\" fill-opacity=\"0\"/>",
            ]
        );
    }

    #[test]
    fn rgba_surface_alpha_is_emitted_once_on_the_fill_property() {
        let mut backend = SvgBackend::in_memory(10, 10);
        backend.begin(10, 10).unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 10.0, 0.0, 10.0),
                Rgba::new(17, 34, 51, 128),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let surface = svg.lines().find(|line| line.starts_with("<rect ")).unwrap();
        assert_eq!(surface.matches("fill-opacity=").count(), 1);
        assert_eq!(surface.matches("opacity=").count(), 1);
        assert!(!surface.contains(" opacity="));
        assert!(surface.contains("fill=\"#112233\""));
    }

    #[test]
    fn migrated_rgba_paints_share_property_specific_alpha_serialization() {
        let mut backend = SvgBackend::in_memory(100, 100);
        backend.begin(100, 100).unwrap();
        backend
            .draw_line(
                &[Point2D::new(1.0, 1.0), Point2D::new(20.0, 20.0)],
                &LineStyle {
                    color: Rgba::new(17, 34, 51, 128),
                    width: 1.0,
                    pattern: super::super::backend::StrokePattern::Solid,
                },
            )
            .unwrap();
        backend
            .draw_text(
                Point2D::new(10.0, 10.0),
                "label",
                &TextStyle {
                    color: Rgba::new(68, 85, 102, 64),
                    ..test_text_style()
                },
            )
            .unwrap();
        backend
            .draw_bracket(
                &SignificanceBracket {
                    x_left: 20.0,
                    x_right: 40.0,
                    y: 40.0,
                    label: "*".into(),
                    orientation: super::super::backend::BracketOrientation::Vertical,
                },
                &BracketStyle {
                    color: Rgba::new(119, 136, 153, 32),
                    width: 1.0,
                    label_color: Rgba::new(170, 187, 204, 16),
                    font_family: "sans-serif".into(),
                    font_size: 12,
                    line_height: 14.4,
                    weight: crate::TextWeight::Normal,
                    label_gap: 5.0,
                },
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains("stroke=\"#112233\" stroke-opacity=\"0.5019607843137255\""));
        assert!(svg.contains("fill=\"#445566\" fill-opacity=\"0.25098039215686274\""));
        assert!(svg.contains("stroke=\"#778899\" stroke-opacity=\"0.12549019607843137\""));
        assert!(svg.contains("fill=\"#AABBCC\" fill-opacity=\"0.06274509803921569\""));
        assert!(!svg.contains(" opacity=\""));
    }

    #[test]
    fn png_preserves_surface_alpha_and_panel_over_root_compositing() {
        let mut backend = SvgBackend::in_memory(1, 1);
        backend.begin(1, 1).unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 1.0, 0.0, 1.0),
                Rgba::new(255, 0, 0, 128),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 1.0, 0.0, 1.0),
                Rgba::new(0, 0, 255, 128),
                SurfaceRole::PlotPanel,
            )
            .unwrap();
        backend.end().unwrap();
        let svg = backend.into_bytes().unwrap();

        let png = rasterize_svg_bytes_to_png(
            &svg,
            PngTargetPlan::new(source_size(1, 1), &PngOptions::default()).unwrap(),
            &PngOptions::default(),
        )
        .unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();

        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(info.bit_depth, png::BitDepth::Eight);
        assert_eq!((info.width, info.height), (3, 3));
        assert!(pixels[..info.buffer_size()]
            .chunks_exact(4)
            .all(|pixel| pixel == [85, 0, 170, 192]));
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
            "<clipPath id=\"clip-0\"><rect x=\"0\" y=\"99.9990234375\" width=\"0.0009765625\" height=\"0.0009765625\"/></clipPath>"
        ));
        assert!(svg.contains("<stop offset=\"0.000001\" stop-color=\"#7F7F7F\"/>"));
        assert!(svg.contains(
            "<rect x=\"0\" y=\"99.9990234375\" width=\"0.0009765625\" height=\"0.0009765625\" fill=\"url(#linear-gradient-0)\"/>"
        ));
        assert!(svg.contains(
            "<rect x=\"0\" y=\"99.994140625\" width=\"0.005859375\" height=\"0.005859375\" rx=\"0\" ry=\"0\""
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
                    color: (0, 0, 0).into(),
                    width: 1.0,
                    label_color: (0, 0, 0).into(),
                    font_family: "sans-serif".into(),
                    font_size: 12,
                    line_height: 14.4,
                    weight: crate::TextWeight::Normal,
                    label_gap: 5.0,
                },
            )
            .unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert!(svg.contains(
            "d=\"M 0 80 L 0.0009765625 80 M 0 80 L 0 87 M 0.0009765625 80 L 0.0009765625 87\""
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
    fn defs_style_surfaces_grid_and_body_have_fixed_role_order() {
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
        backend
            .draw_surface(
                PanelBox::new(10.0, 190.0, 20.0, 80.0),
                Rgba::rgb(4, 5, 6),
                SurfaceRole::PlotPanel,
            )
            .unwrap();
        backend
            .draw_surface(
                PanelBox::new(0.0, 200.0, 0.0, 100.0),
                Rgba::rgb(1, 2, 3),
                SurfaceRole::CanvasRoot,
            )
            .unwrap();
        backend
            .begin_clip(PanelBox::new(10.0, 190.0, 20.0, 80.0))
            .unwrap();
        backend.end_clip().unwrap();
        backend.end().unwrap();

        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let style = svg.find("<style>").unwrap();
        let defs = svg.find("<defs>").unwrap();
        let root = svg.find("fill=\"#010203\"").unwrap();
        let panel = svg.find("fill=\"#040506\"").unwrap();
        let grid = svg.find("<g id=\"grid\">").unwrap();
        let body = svg.find("<g id=\"marks\">").unwrap();

        assert!(style < root);
        assert!(defs < root);
        assert!(root < panel);
        assert!(panel < grid);
        assert!(grid < body);
    }

    #[test]
    fn planned_text_outlining_uses_retained_glyph_geometry() {
        let environment = PreparedFontEnvironment::synthetic();
        let plan = environment
            .plan_text(TextPlanInput {
                source: "A",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 20,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: (1, 2, 3).into(),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "outline test".into(),
                },
                exact_override: false,
            })
            .unwrap();
        crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow_mut().clear());
        let shape_calls = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let planned_glyphs = plan.lines[0].runs[0]
            .glyphs
            .iter()
            .map(|glyph| (glyph.face, glyph.glyph_id))
            .collect::<Vec<_>>();
        let mut backend = SvgBackend::in_memory_with_fonts(120, 40, environment);
        backend.begin(120, 40).unwrap();
        backend
            .draw_planned_text(
                PlannedTextPlacement::middle(Point2D::new(8.0, 20.0), TextAnchor::Start, 0.0),
                &plan,
            )
            .unwrap();
        backend.end().unwrap();
        let output = backend.into_output().unwrap();
        let outlined = String::from_utf8(
            output
                .outline_plan
                .derive(output.width_css_px, output.height_css_px)
                .unwrap(),
        )
        .unwrap();
        let preserve = String::from_utf8(output.bytes).unwrap();

        assert_eq!(
            preserve,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"40\" viewBox=\"0 0 120 40\" role=\"img\">\n<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>\n<text x=\"8\" y=\"26\" font-family=\"Test Sans\" font-size=\"20\" fill=\"#010203\" text-anchor=\"start\" direction=\"ltr\">A</text>\n</svg>\n"
        );
        assert!(!outlined.contains("<text"));
        assert!(outlined.contains("<path d=\"M"));
        assert!(outlined.contains('L') && outlined.contains('Z'));
        assert!(outlined.contains("fill=\"#010203\""));
        crate::font::canonical::FONT_OUTLINE_GLYPHS
            .with(|glyphs| assert_eq!(&*glyphs.borrow(), &planned_glyphs.repeat(2)));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), shape_calls));
    }

    #[test]
    fn reversed_rotated_block_preserves_existing_column_positions_in_png() {
        let environment = PreparedFontEnvironment::synthetic();
        let input = |source| TextPlanInput {
            source,
            stack: "Test Sans",
            weight: crate::TextWeight::Normal,
            font_size: 10,
            line_height: TextLineHeight::Multiplier(1.6),
            maximum_width_quanta: i64::MAX,
            alignment: PhysicalTextAlignment::Center,
            paint: Rgba::rgb(1, 2, 3),
            provenance: TextPlanProvenance {
                pair_ordinal: 0,
                owner: "rotated columns".into(),
            },
            exact_override: false,
        };
        let plan = environment.plan_text(input("A\nAA")).unwrap();
        let singles = [
            environment.plan_text(input("A")).unwrap(),
            environment.plan_text(input("AA")).unwrap(),
        ];
        let block = crate::render::backend::PlacedText::from_deferred_first_line(
            plan.clone(),
            Point2D::new(34.0, 50.0),
            TextAnchor::Middle,
            -90.0,
        )
        .with_reversed_lines();
        assert_eq!(
            block
                .placement
                .line_y(&plan, plan.lines[0].translation_y_quanta)
                .unwrap(),
            plan.lines[1].translation_y_quanta
        );
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let mut outlined = Vec::new();
        for whole in [true, false] {
            let mut backend = SvgBackend::in_memory_with_fonts(120, 100, environment.clone());
            backend.begin(120, 100).unwrap();
            if whole {
                block.draw(&mut backend).unwrap();
            } else {
                for (single, x) in singles.iter().zip([50.0, 34.0]) {
                    backend
                        .draw_planned_text(
                            PlannedTextPlacement::middle(
                                Point2D::new(x, 50.0),
                                TextAnchor::Middle,
                                -90.0,
                            ),
                            single,
                        )
                        .unwrap();
                }
            }
            backend.end().unwrap();
            outlined.push(backend.into_outlined_bytes().unwrap());
        }
        let options = PngOptions {
            sizing: PngSizing::Source,
            compression: PngCompression::Balanced,
        };
        let rasterize = |svg: &[u8]| {
            rasterize_svg_bytes_to_png(
                svg,
                PngTargetPlan::new(source_size(120, 100), &options).unwrap(),
                &options,
            )
            .unwrap()
        };
        assert_eq!(rasterize(&outlined[0]), rasterize(&outlined[1]));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert_eq!(block.plan, plan);
    }

    #[test]
    fn outline_uses_absolute_glyph_origins_without_summing_rounded_advances() {
        let environment = PreparedFontEnvironment::synthetic();
        let mut plan = environment
            .plan_text(TextPlanInput {
                source: "AA",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 20,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: Rgba::rgb(1, 2, 3),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "absolute origins".into(),
                },
                exact_override: false,
            })
            .unwrap();
        for (glyph, x) in plan.edit_for_test().lines[0].runs[0]
            .glyphs
            .iter_mut()
            .zip([5, 17])
        {
            glyph.glyph_origin_x_quanta = x * CSS_PIXEL_QUANTA;
            glyph.glyph_origin_y_quanta = 0;
            glyph.x_advance_quanta = i64::MAX;
        }
        let mut backend = SvgBackend::in_memory_with_fonts(120, 40, environment);
        backend.begin(120, 40).unwrap();
        backend
            .draw_planned_text(
                PlannedTextPlacement::baseline(Point2D::new(0.0, 0.0), TextAnchor::Start, 0.0),
                &plan,
            )
            .unwrap();
        backend.end().unwrap();
        let outlined = String::from_utf8(backend.into_outlined_bytes().unwrap()).unwrap();
        assert!(
            outlined.contains("matrix(0.02 0 0 -0.02 5 40)"),
            "{outlined}"
        );
        assert!(
            outlined.contains("matrix(0.02 0 0 -0.02 17 40)"),
            "{outlined}"
        );
    }

    #[test]
    fn wrapped_bidi_preserve_text_isolates_frozen_visual_runs_and_keeps_logical_source() {
        let environment = PreparedFontEnvironment::synthetic();
        let source = "A \u{202b}\u{5d0} \u{5d0} \u{202c}A";
        let plan = environment
            .plan_text(TextPlanInput {
                source,
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 10,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: 8_704,
                alignment: PhysicalTextAlignment::Start,
                paint: Rgba::rgb(1, 2, 3),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "paragraph bidi".into(),
                },
                exact_override: false,
            })
            .unwrap();
        assert!(plan.lines.len() >= 3);
        let mut backend = SvgBackend::in_memory_with_fonts(120, 100, environment);
        backend.begin(120, 100).unwrap();
        backend
            .draw_planned_text(
                PlannedTextPlacement::middle(Point2D::new(10.0, 50.0), TextAnchor::Start, 0.0),
                &plan,
            )
            .unwrap();
        backend.end().unwrap();
        let output = backend.into_output().unwrap();
        let svg = std::str::from_utf8(&output.bytes).unwrap();
        assert!(svg.contains("unicode-bidi=\"bidi-override\""));
        assert!(
            svg.contains("direction=\"rtl\" unicode-bidi=\"bidi-override\" text-anchor=\"end\"")
        );
        for line in &plan.lines {
            let logical = &plan.source
                [line.source_span.start_byte as usize..line.source_span.end_byte as usize];
            assert!(
                svg.contains(&format!("aria-label=\"{}\"", EscapedText(logical)))
                    || svg.contains(&format!(">{}</text>", EscapedText(logical)))
            );
        }
        assert_eq!(plan.line_texts().concat(), source);
        assert!(!contains_svg_text(
            &output.outline_plan.derive(120, 100).unwrap()
        ));
    }

    #[test]
    fn bidi_run_dom_order_preserves_selectable_logical_text() {
        let environment = PreparedFontEnvironment::synthetic_faces(&[
            ("Latin", &['A', ' ']),
            ("Hebrew", &['\u{5d0}']),
        ]);
        let source = "A \u{202b}\u{5d0} A\u{202c} A";
        let plan = environment
            .plan_text(TextPlanInput {
                source,
                stack: "Latin, Hebrew",
                weight: crate::TextWeight::Normal,
                font_size: 10,
                line_height: TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: Rgba::rgb(1, 2, 3),
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "logical selection".into(),
                },
                exact_override: false,
            })
            .unwrap();
        assert!(plan.lines[0]
            .runs
            .windows(2)
            .any(|pair| pair[0].source_span.start_byte > pair[1].source_span.start_byte));
        let first_face = plan.lines[0].runs[0].face;
        assert!(plan.lines[0].runs.iter().any(|run| run.face != first_face));
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let mut backend = SvgBackend::in_memory_with_fonts(120, 100, environment);
        backend.begin(120, 100).unwrap();
        backend
            .draw_planned_text(
                PlannedTextPlacement::middle(Point2D::new(10.0, 50.0), TextAnchor::Start, 0.0),
                &plan,
            )
            .unwrap();
        backend.end().unwrap();
        let output = backend.into_output().unwrap();
        let svg = std::str::from_utf8(&output.bytes).unwrap();
        assert!(svg.contains("font-family=\"Latin, Hebrew\""));
        assert!(!svg.contains("@font-face"));
        assert!(!contains_svg_text(
            &output.outline_plan.derive(120, 100).unwrap()
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), shapes));
        let selected = svg
            .split("<tspan ")
            .skip(1)
            .map(|part| {
                part.split_once('>')
                    .unwrap()
                    .1
                    .split_once("</tspan>")
                    .unwrap()
                    .0
            })
            .collect::<String>();
        assert_eq!(selected, source);
        let line = &plan.lines[0];
        let run = line
            .runs
            .iter()
            .find(|run| {
                run.bidi_level % 2 == 1
                    && plan.source
                        [run.source_span.start_byte as usize..run.source_span.end_byte as usize]
                        .contains('\u{5d0}')
            })
            .unwrap();
        let glyph = &run.glyphs[0];
        let expected_x =
            10 * CSS_PIXEL_QUANTA + line.translation_x_quanta + glyph.glyph_origin_x_quanta
                - glyph.x_offset_quanta;
        let run_source =
            &plan.source[run.source_span.start_byte as usize..run.source_span.end_byte as usize];
        let tag = svg
            .split("<tspan ")
            .skip(1)
            .find(|tag| {
                tag.split_once('>')
                    .unwrap()
                    .1
                    .split_once("</tspan>")
                    .unwrap()
                    .0
                    == run_source
            })
            .unwrap();
        assert!(tag.starts_with(&format!(
            "x=\"{}\" ",
            coordinates::CanonicalCoordinate::from_quanta(expected_x)
        )));
    }

    #[test]
    fn selected_soft_hyphen_paints_ascii_hyphen_without_rewriting_accessible_source() {
        for (environment, stack, selected_runs) in [
            (
                PreparedFontEnvironment::synthetic_faces(&[("Soft", &['A', '-'])]),
                "Soft",
                1,
            ),
            (
                PreparedFontEnvironment::synthetic_faces(&[
                    ("Letters", &['A']),
                    ("Hyphens", &['-']),
                ]),
                "Letters, Hyphens",
                2,
            ),
        ] {
            for (source, maximum_width_quanta, selected) in [
                ("A\u{ad}AA", 12_288, true),
                ("A\u{ad}AA", i64::MAX, false),
                ("A\u{ad}", i64::MAX, false),
            ] {
                let plan = environment
                    .plan_text(TextPlanInput {
                        source,
                        stack,
                        weight: crate::TextWeight::Normal,
                        font_size: 10,
                        line_height: TextLineHeight::Multiplier(1.2),
                        maximum_width_quanta,
                        alignment: PhysicalTextAlignment::Start,
                        paint: Rgba::rgb(1, 2, 3),
                        provenance: TextPlanProvenance {
                            pair_ordinal: 0,
                            owner: "soft hyphen SVG".into(),
                        },
                        exact_override: false,
                    })
                    .unwrap();
                assert_eq!(plan.source, source);
                assert_eq!(plan.lines[0].selected_soft_hyphen.is_some(), selected);
                let placement =
                    PlannedTextPlacement::middle(Point2D::new(8.0, 20.0), TextAnchor::Start, 0.0);
                let mut svg = SvgBackend::in_memory_with_fonts(100, 60, environment.clone());
                Backend::begin(&mut svg, 100, 60).unwrap();
                Backend::draw_planned_text(&mut svg, placement, &plan).unwrap();
                Backend::end(&mut svg).unwrap();
                let output = svg.into_output().unwrap();
                let text = std::str::from_utf8(&output.bytes).unwrap();
                if selected {
                    assert_eq!(plan.lines[0].runs.len(), selected_runs);
                    assert!(text.contains("aria-label=\"A\u{ad}\""), "{text}");
                    assert!(text.contains("-</tspan>"), "{text}");
                    assert!(!text.contains(">\u{ad}</tspan>"));
                    let mut invalid = plan.clone();
                    invalid.edit_for_test().lines[0].selected_soft_hyphen =
                        Some(plan.lines[0].source_span);
                    assert!(
                        super::super::backend::validate_planned_text(placement, &invalid).is_err()
                    );
                } else {
                    assert!(text.contains(source));
                    assert!(!text.contains(">-</tspan>"));
                }
                let count = output.outline_plan.count_bytes(100, 60).unwrap();
                let outlined = output.outline_plan.derive_counted(100, 60, count).unwrap();
                assert_eq!(outlined.len(), count);
                assert!(!contains_svg_text(&outlined));
                assert_eq!(plan.source, source);
            }
        }
    }

    #[test]
    fn no_outline_glyph_is_valid_only_without_planned_ink() {
        let environment = PreparedFontEnvironment::synthetic();
        let input = |source| TextPlanInput {
            source,
            stack: "Test Sans",
            weight: crate::TextWeight::Normal,
            font_size: 20,
            line_height: TextLineHeight::Multiplier(1.2),
            maximum_width_quanta: i64::MAX,
            alignment: PhysicalTextAlignment::Start,
            paint: (1, 2, 3).into(),
            provenance: TextPlanProvenance {
                pair_ordinal: 0,
                owner: "no-outline test".into(),
            },
            exact_override: false,
        };
        let placement =
            PlannedTextPlacement::middle(Point2D::new(8.0, 20.0), TextAnchor::Start, 0.0);

        let whitespace = environment.plan_text(input(" ")).unwrap();
        assert!(whitespace.lines[0].runs[0].glyphs[0].design_ink.is_none());
        let mut backend = SvgBackend::in_memory_with_fonts(120, 40, environment.clone());
        backend.begin(120, 40).unwrap();
        backend.draw_planned_text(placement, &whitespace).unwrap();
        backend.end().unwrap();
        let outlined = String::from_utf8(backend.into_outlined_bytes().unwrap()).unwrap();
        assert!(!outlined.contains("<text") && !outlined.contains("<path d="));

        let mut inconsistent = environment.plan_text(input("A")).unwrap();
        inconsistent.edit_for_test().lines[0].runs[0].glyphs[0].glyph_id = 2;
        let mut backend = SvgBackend::in_memory_with_fonts(120, 40, environment);
        backend.begin(120, 40).unwrap();
        backend.draw_planned_text(placement, &inconsistent).unwrap();
        backend.end().unwrap();
        let error = backend.into_outlined_bytes().unwrap_err();
        assert!(matches!(error, Error::Render(_)));
        assert!(error
            .to_string()
            .contains("planned glyph has ink but its retained face has no outline"));
    }

    #[test]
    fn png_conversion_keeps_parse_and_dimension_failures_route_specific() {
        let options = PngOptions::default();
        let text_svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><text x="1" y="8">late text</text></svg>"#;
        let target = PngTargetPlan::new(source_size(10, 10), &options).unwrap();
        let error = rasterize_svg_bytes_to_png(text_svg, target, &options).unwrap_err();
        assert!(error.to_string().contains("path-only SVG scene"));

        let error = rasterize_svg_bytes_to_png(b"not svg", target, &options).unwrap_err();
        assert!(error
            .to_string()
            .contains("parse svg for png rasterization"));

        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="11" height="10" viewBox="0 0 11 10"/>"#;
        let error = rasterize_svg_bytes_to_png(svg, target, &options).unwrap_err();
        assert!(error
            .to_string()
            .contains("dimensions do not match its canonical CSS dimensions"));
    }

    #[test]
    fn dpi_metadata_rejects_values_outside_png_range() {
        let error = dpi_to_pixels_per_meter(0.01).unwrap_err();
        assert!(error.to_string().contains("too small to encode"));
        assert_eq!(dpi_to_pixels_per_meter(0.02).unwrap(), 1);

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
            "<linearGradient id=\"linear-gradient-0\" gradientUnits=\"userSpaceOnUse\" x1=\"16\" y1=\"80\" x2=\"16\" y2=\"20\">\n<stop offset=\"0.000000\" stop-color=\"#3182BD\"/>\n<stop offset=\"0.250000\" stop-color=\"#F7F7F7\"/>\n<stop offset=\"1.000000\" stop-color=\"#DE2D26\"/>\n</linearGradient>\n"
        ));
        assert!(first.contains(
            "<rect x=\"10\" y=\"20\" width=\"12\" height=\"60\" fill=\"url(#linear-gradient-0)\"/>"
        ));
        let first_gradient = first.find("id=\"linear-gradient-0\"").unwrap();
        let second_gradient = first.find("id=\"linear-gradient-1\"").unwrap();
        let first_fill = first.find("fill=\"url(#linear-gradient-0)\"").unwrap();
        let second_fill = first.find("fill=\"url(#linear-gradient-1)\"").unwrap();
        assert!(first_gradient < second_gradient);
        assert!(second_gradient < first_fill);
        assert!(first_fill < second_fill);
    }
}
