//! Deferred and validating backend implementations used by render orchestration.

use crate::Error;
use std::cell::RefCell;

use crate::font::canonical::PreparedFontEnvironment;
use crate::layout::{
    DeferredAnnotationTextPlan, PanelBox, PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight,
    TextPlanInput, TextPlanProvenance,
};
use crate::render::backend::{
    self, Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle,
    PlannedSignificanceCall, PlannedTextPlacement, Point2D, PointStyle, Rgba, SignificanceBracket,
    SurfaceRole, TextAnchor, TextStyle,
};

#[cfg(test)]
thread_local! {
    pub(crate) static TEXT_PLAN_CAPTURE_SESSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static TEXT_REPLAY_SESSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static NULL_BEGIN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static NULL_END_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) struct DeferredBackend<'a> {
    backend: &'a mut dyn Backend,
    width: u32,
    height: u32,
    state: DeferredState,
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum DeferredState {
    Fresh,
    Open,
    Closed,
}

#[cfg(test)]
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

#[cfg(test)]
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

    fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: Rgba,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_surface(bounds, paint, role)
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

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_planned_text(placement, plan)
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

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.ensure_started()?;
        self.backend.draw_planned_bracket(call)
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ensure_started()?;
        self.end_open_canvas()
    }
}

pub(crate) struct TextReplayPlan {
    calls: Vec<PlannedCall>,
    storage: RefCell<PreparedReplayStorage>,
}

impl std::fmt::Debug for TextReplayPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextReplayPlan")
            .field("calls", &self.calls)
            .finish_non_exhaustive()
    }
}

struct PreparedReplayStorage {
    svg: Option<crate::render::svg_backend::SvgStorage>,
    null: Option<NullBackend>,
}

impl TextReplayPlan {
    pub(crate) fn take_svg_backend(&self) -> Result<crate::render::svg_backend::SvgBackend, Error> {
        let mut storage = self
            .storage
            .try_borrow_mut()
            .map_err(|_| Error::Render("prepared scene storage is already borrowed".into()))?;
        if storage.null.is_some() {
            return Err(Error::Render(
                "prepared SVG requires successful final Null validation".into(),
            ));
        }
        storage
            .svg
            .take()
            .map(crate::render::svg_backend::SvgStorage::into_backend)
            .ok_or_else(|| Error::Render("prepared SVG storage was already consumed".into()))
    }

    #[cfg(test)]
    pub(crate) fn call_count(&self) -> usize {
        self.calls.len()
    }

    #[cfg(test)]
    pub(crate) fn retained_text(&self) -> Vec<(PlannedTextPlacement, &ResolvedTextPlan)> {
        self.calls
            .iter()
            .filter_map(|call| match call {
                PlannedCall::PlannedText(text) => Some((text.placement, &text.plan)),
                PlannedCall::RawText(_) => panic!("measured producer used raw final capture"),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn replay(
        &self,
        width: u32,
        height: u32,
        backend: &mut dyn Backend,
        scope: &str,
    ) -> Result<(), Error> {
        #[cfg(test)]
        TEXT_REPLAY_SESSIONS.with(|calls| calls.set(calls.get() + 1));
        backend.begin(width, height)?;
        let replayed = self.replay_open(backend);
        let ended = backend.end();
        backend::finish_scope(replayed, ended, scope)
    }

    pub(crate) fn replay_open(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for (index, call) in self.calls.iter().enumerate() {
            if let Err(error) = Self::replay_call(call, backend) {
                let cleanup = self.cleanup_open_scopes(index, backend);
                return backend::finish_scope(Err(error), cleanup, "prepared scene scopes");
            }
        }
        Ok(())
    }

    fn replay_call(call: &PlannedCall, backend: &mut dyn Backend) -> Result<(), Error> {
        match call {
            PlannedCall::BeginLayer(name) => backend.begin_layer(name)?,
            PlannedCall::BeginLayerWithRole(name, role) => {
                backend.begin_layer_with_role(name, *role)?;
            }
            PlannedCall::EndLayer => backend.end_layer()?,
            PlannedCall::BeginClip(region) => backend.begin_clip(*region)?,
            PlannedCall::EndClip => backend.end_clip()?,
            PlannedCall::Surface {
                bounds,
                paint,
                role,
            } => backend.draw_surface(*bounds, *paint, *role)?,
            PlannedCall::Scatter { points, style } => {
                backend.draw_scatter(points, style)?;
            }
            PlannedCall::Line { points, style } => backend.draw_line(points, style)?,
            PlannedCall::Polygon { points, style } => backend.draw_polygon(points, style)?,
            PlannedCall::LinearGradient {
                bounds,
                start,
                end,
                stops,
            } => backend.draw_linear_gradient(*bounds, *start, *end, stops)?,
            PlannedCall::RawText(call) | PlannedCall::PlannedText(call) => {
                backend.draw_planned_text(call.placement, &call.plan)?;
            }
            PlannedCall::RoundedRect {
                x,
                y,
                width,
                height,
                radius,
                style,
            } => backend.draw_rounded_rect(*x, *y, *width, *height, *radius, style)?,
            PlannedCall::RawBracket(call) | PlannedCall::PlannedBracket(call) => {
                backend.draw_planned_bracket(PlannedSignificanceCall {
                    bracket: &call.bracket,
                    style: &call.style,
                    label: &call.label,
                })?;
            }
        }
        Ok(())
    }

    fn cleanup_open_scopes(
        &self,
        failed_index: usize,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let mut closed_depth = 0_usize;
        for call in self.calls[..failed_index].iter().rev() {
            match call {
                PlannedCall::EndLayer | PlannedCall::EndClip => {
                    closed_depth = closed_depth.checked_add(1).ok_or_else(|| {
                        Error::Render("prepared scene cleanup depth overflowed".into())
                    })?;
                }
                PlannedCall::BeginLayer(_) | PlannedCall::BeginLayerWithRole(_, _) => {
                    if closed_depth == 0 {
                        backend.end_layer()?;
                    } else {
                        closed_depth -= 1;
                    }
                }
                PlannedCall::BeginClip(_) => {
                    if closed_depth == 0 {
                        backend.end_clip()?;
                    } else {
                        closed_depth -= 1;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum PlannedCall {
    BeginLayer(String),
    BeginLayerWithRole(String, LayerRole),
    EndLayer,
    BeginClip(PanelBox),
    EndClip,
    Surface {
        bounds: PanelBox,
        paint: Rgba,
        role: SurfaceRole,
    },
    Scatter {
        points: Vec<Point2D>,
        style: PointStyle,
    },
    Line {
        points: Vec<Point2D>,
        style: LineStyle,
    },
    Polygon {
        points: Vec<Point2D>,
        style: FillStyle,
    },
    LinearGradient {
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: Vec<GradientStop>,
    },
    RawText(PlannedTextCall),
    PlannedText(PlannedTextCall),
    RoundedRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
        style: FillStyle,
    },
    RawBracket(PlannedBracketCall),
    PlannedBracket(PlannedBracketCall),
}

impl PlannedCall {
    fn emit_svg<S: crate::render::svg_backend::SvgSink>(
        &self,
        serializer: &mut crate::render::svg_backend::SvgSerializer<S>,
    ) -> Result<(), Error> {
        match self {
            Self::BeginLayer(name) => serializer.begin_layer(name)?,
            Self::BeginLayerWithRole(name, role) => {
                serializer.begin_layer_with_role(name, *role)?;
            }
            Self::EndLayer => serializer.end_layer()?,
            Self::BeginClip(region) => serializer.begin_clip(*region)?,
            Self::EndClip => serializer.end_clip()?,
            Self::Surface {
                bounds,
                paint,
                role,
            } => serializer.draw_surface(*bounds, *paint, *role)?,
            Self::Scatter { points, style } => serializer.draw_scatter(points, style)?,
            Self::Line { points, style } => serializer.draw_line(points, style)?,
            Self::Polygon { points, style } => serializer.draw_polygon(points, style)?,
            Self::LinearGradient {
                bounds,
                start,
                end,
                stops,
            } => serializer.draw_linear_gradient(*bounds, *start, *end, stops)?,
            Self::RawText(call) | Self::PlannedText(call) => {
                serializer.draw_planned_text(call.placement, &call.plan)?;
            }
            Self::RoundedRect {
                x,
                y,
                width,
                height,
                radius,
                style,
            } => serializer.draw_rounded_rect(*x, *y, *width, *height, *radius, style)?,
            Self::RawBracket(call) | Self::PlannedBracket(call) => serializer
                .draw_planned_bracket(PlannedSignificanceCall {
                    bracket: &call.bracket,
                    style: &call.style,
                    label: &call.label,
                })?,
        }
        serializer.check_buffers()
    }
}

#[cfg(test)]
impl PlannedCall {
    const fn kind(&self) -> &'static str {
        match self {
            Self::RawText(_) => "raw text",
            Self::PlannedText(_) => "planned text",
            Self::RawBracket(_) => "raw bracket",
            Self::PlannedBracket(_) => "planned bracket",
            _ => "non-text draw call",
        }
    }
}

#[derive(Debug)]
struct PlannedTextCall {
    placement: PlannedTextPlacement,
    plan: ResolvedTextPlan,
}

#[derive(Debug)]
struct PlannedBracketCall {
    bracket: SignificanceBracket,
    style: BracketStyle,
    label: DeferredAnnotationTextPlan,
}

pub(super) struct TextPlanningBackend<'a, 'b> {
    recorder: CallCountingBackend,
    font_environment: PreparedFontEnvironment,
    operation_default_stack: String,
    calls: Vec<PlannedCall>,
    final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
}

impl<'a, 'b> TextPlanningBackend<'a, 'b> {
    fn new(
        theme: &crate::theme::ResolvedTheme,
        call_count: usize,
        final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(calls.get() + 1));
        let mut calls = Vec::new();
        final_stage.reserve_final(&mut calls, call_count, "prepared scene calls")?;
        let mut stack = Vec::new();
        final_stage.reserve_final(
            &mut stack,
            theme.font_family.len(),
            "prepared default font stack",
        )?;
        stack.extend_from_slice(theme.font_family.as_bytes());
        Ok(Self {
            recorder: CallCountingBackend {
                call_count: 0,
                limit: call_count,
            },
            font_environment: theme.prepared_font_environment()?.clone(),
            operation_default_stack: String::from_utf8(stack)
                .map_err(|error| Error::Render(format!("prepared default font stack: {error}")))?,
            calls,
            final_stage,
        })
    }

    fn stack_for<'c>(&'c self, style: &'c TextStyle) -> &'c str {
        if style.font_family == crate::DEFAULT_FONT_FAMILY {
            &self.operation_default_stack
        } else {
            &style.font_family
        }
    }

    fn plan_text(
        &self,
        source: &str,
        style: &TextStyle,
        line_height: TextLineHeight,
        alignment: PhysicalTextAlignment,
        owner: &str,
    ) -> Result<ResolvedTextPlan, Error> {
        self.font_environment
            .plan_prepared_text(TextPlanInput {
                source,
                stack: self.stack_for(style),
                weight: style.weight,
                font_size: style.font_size,
                line_height,
                maximum_width_quanta: i64::MAX,
                alignment,
                paint: style.color,
                provenance: TextPlanProvenance {
                    pair_ordinal: pair_ordinal(style.weight),
                    owner: owner.to_owned(),
                },
                exact_override: false,
            })
            .map_err(|error| Error::Render(format!("{owner}: {error}")))
    }

    fn plan_raw_bracket(
        &self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<DeferredAnnotationTextPlan, Error> {
        let alignment = match bracket.orientation {
            backend::BracketOrientation::Vertical => PhysicalTextAlignment::Center,
            backend::BracketOrientation::Horizontal => PhysicalTextAlignment::Start,
        };
        let stack = if style.font_family == crate::DEFAULT_FONT_FAMILY {
            &self.operation_default_stack
        } else {
            &style.font_family
        };
        let plan = self
            .font_environment
            .plan_prepared_text(TextPlanInput {
                source: &bracket.label,
                stack,
                font_size: style.font_size,
                weight: style.weight,
                line_height: TextLineHeight::Multiplier(
                    style.line_height / f64::from(style.font_size),
                ),
                maximum_width_quanta: i64::MAX,
                alignment,
                paint: style.label_color,
                provenance: TextPlanProvenance {
                    pair_ordinal: pair_ordinal(style.weight),
                    owner: "significance bracket label capture".into(),
                },
                exact_override: false,
            })
            .map_err(|error| {
                Error::Render(format!("significance bracket label capture: {error}"))
            })?;
        Ok(DeferredAnnotationTextPlan::from_resolved(plan))
    }

    fn into_plan(self, width: u32, height: u32) -> Result<TextReplayPlan, Error> {
        let calls = self.calls;
        let mut depth = 0_usize;
        let mut scopes = 0_usize;
        let mut points = 0_usize;
        let mut runs = 0_usize;
        for call in &calls {
            let text = match call {
                PlannedCall::RawText(call) | PlannedCall::PlannedText(call) => Some(&call.plan),
                PlannedCall::RawBracket(call) | PlannedCall::PlannedBracket(call) => {
                    Some(&call.label.plan)
                }
                _ => None,
            };
            if let Some(text) = text {
                for line in &text.lines {
                    if line.runs.len() > 1 {
                        runs = runs.max(line.runs.len());
                    }
                }
            }
            match call {
                PlannedCall::BeginLayer(_)
                | PlannedCall::BeginLayerWithRole(_, _)
                | PlannedCall::BeginClip(_) => {
                    depth = depth.checked_add(1).ok_or_else(|| {
                        Error::Render("prepared scene scope count overflow".into())
                    })?;
                    scopes = scopes.max(depth);
                }
                PlannedCall::EndLayer | PlannedCall::EndClip => {
                    depth = depth.checked_sub(1).ok_or_else(|| {
                        Error::Render("prepared scene scope closes without opening".into())
                    })?;
                }
                PlannedCall::Line { points: values, .. } if values.len() >= 2 => {
                    points = points.max(values.len());
                }
                PlannedCall::Polygon { points: values, .. } if values.len() >= 3 => {
                    points = points.max(values.len());
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(Error::Render("prepared scene has unclosed scopes".into()));
        }
        let svg = crate::render::svg_backend::SvgStorage::prepare(
            width,
            height,
            self.font_environment,
            crate::render::svg_backend::SvgStorageRequirements {
                scopes,
                points,
                runs,
            },
            self.final_stage,
            |serializer| {
                for call in &calls {
                    call.emit_svg(serializer)?;
                }
                Ok(())
            },
        )?;
        let mut null = NullBackend::default();
        crate::render::svg_backend::reserve_storage(
            self.final_stage,
            &mut null.recorder.scopes,
            scopes,
            "final Null scopes",
        )?;
        null.recorder.validation.prepare(
            points,
            self.final_stage,
            "final Null validation points",
        )?;
        null.recorder.fixed = true;
        Ok(TextReplayPlan {
            calls,
            storage: RefCell::new(PreparedReplayStorage {
                svg: Some(svg),
                null: Some(null),
            }),
        })
    }

    fn copy_slice<T: Copy>(&mut self, source: &[T], payload: &str) -> Result<Vec<T>, Error> {
        let mut values = Vec::new();
        self.final_stage
            .reserve_final(&mut values, source.len(), payload)?;
        values.extend_from_slice(source);
        Ok(values)
    }

    fn copy_string(&mut self, source: &str, payload: &str) -> Result<String, Error> {
        String::from_utf8(self.copy_slice(source.as_bytes(), payload)?)
            .map_err(|error| Error::Render(format!("{payload}: {error}")))
    }

    fn copy_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(SignificanceBracket, BracketStyle), Error> {
        let label = self.copy_string(&bracket.label, "prepared bracket label")?;
        let font_family = self.copy_string(&style.font_family, "prepared bracket font family")?;
        Ok((
            SignificanceBracket { label, ..*bracket },
            BracketStyle {
                font_family,
                ..*style
            },
        ))
    }
}

const fn pair_ordinal(weight: crate::TextWeight) -> u32 {
    match weight {
        crate::TextWeight::Normal => 0,
        crate::TextWeight::Semibold => 1,
        crate::TextWeight::Bold => 2,
    }
}

const fn alignment(anchor: TextAnchor) -> PhysicalTextAlignment {
    match anchor {
        TextAnchor::Start => PhysicalTextAlignment::Start,
        TextAnchor::Middle => PhysicalTextAlignment::Center,
        TextAnchor::End => PhysicalTextAlignment::End,
    }
}

struct CallCountingBackend {
    call_count: usize,
    limit: usize,
}

impl Default for CallCountingBackend {
    fn default() -> Self {
        Self {
            call_count: 0,
            limit: usize::MAX,
        }
    }
}

impl CallCountingBackend {
    fn claim_call(&mut self) -> Result<(), Error> {
        if self.call_count >= self.limit {
            return Err(Error::Render(
                "prepared scene exceeded counted call storage before retention".into(),
            ));
        }
        self.call_count = self
            .call_count
            .checked_add(1)
            .ok_or_else(|| Error::Render("prepared scene call count overflowed".into()))?;
        Ok(())
    }
}

impl Backend for CallCountingBackend {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "prepared scene dimensions must be positive".into(),
            ));
        }
        Ok(())
    }

    fn begin_layer(&mut self, _name: &str) -> Result<(), Error> {
        self.claim_call()
    }

    fn begin_layer_with_role(&mut self, _name: &str, _role: LayerRole) -> Result<(), Error> {
        self.claim_call()
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.claim_call()
    }

    fn begin_clip(&mut self, _region: PanelBox) -> Result<(), Error> {
        self.claim_call()
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_surface(
        &mut self,
        _bounds: PanelBox,
        _paint: Rgba,
        _role: SurfaceRole,
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_scatter(&mut self, _points: &[Point2D], _style: &PointStyle) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_line(&mut self, _points: &[Point2D], _style: &LineStyle) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_polygon(&mut self, _points: &[Point2D], _style: &FillStyle) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_linear_gradient(
        &mut self,
        _bounds: PanelBox,
        _start: Point2D,
        _end: Point2D,
        _stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_text(
        &mut self,
        _position: Point2D,
        _text: &str,
        _style: &TextStyle,
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_planned_text(
        &mut self,
        _placement: PlannedTextPlacement,
        _plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_rounded_rect(
        &mut self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _radius: f64,
        _style: &FillStyle,
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_bracket(
        &mut self,
        _bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        self.claim_call()
    }

    fn draw_planned_bracket(&mut self, _call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.claim_call()
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) fn capture_text_replay_plan(
    width: u32,
    height: u32,
    scope: &str,
    theme: &crate::theme::ResolvedTheme,
    final_stage: &mut crate::layout::kernel::FinalStage<'_>,
    mut capture: impl FnMut(&mut dyn Backend) -> Result<(), Error>,
) -> Result<TextReplayPlan, Error> {
    let mut counter = CallCountingBackend::default();
    if width == 0 || height == 0 {
        return Err(Error::Render(
            "prepared scene dimensions must be positive".into(),
        ));
    }
    capture(&mut counter)?;
    let call_count = counter.call_count;

    let mut backend = TextPlanningBackend::new(theme, call_count, final_stage)?;
    capture(&mut backend)?;
    if backend.calls.len() != call_count {
        return Err(Error::Render(format!(
            "{scope}: counted {call_count} prepared scene calls but materialized {}",
            backend.calls.len()
        )));
    }
    backend.into_plan(width, height)
}

impl Backend for TextPlanningBackend<'_, '_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.recorder.begin(width, height)
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.recorder.begin_layer(name)?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayer(name));
        Ok(())
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.recorder.begin_layer_with_role(name, role)?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayerWithRole(name, role));
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.recorder.end_layer()?;
        self.calls.push(PlannedCall::EndLayer);
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.recorder.begin_clip(region)?;
        self.calls.push(PlannedCall::BeginClip(region));
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.recorder.end_clip()?;
        self.calls.push(PlannedCall::EndClip);
        Ok(())
    }

    fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: Rgba,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.recorder.draw_surface(bounds, paint, role)?;
        self.calls.push(PlannedCall::Surface {
            bounds,
            paint,
            role,
        });
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.recorder.draw_scatter(points, style)?;
        let points = self.copy_slice(points, "prepared scatter points")?;
        self.calls.push(PlannedCall::Scatter {
            points,
            style: style.clone(),
        });
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.recorder.draw_line(points, style)?;
        let points = self.copy_slice(points, "prepared line points")?;
        self.calls.push(PlannedCall::Line {
            points,
            style: style.clone(),
        });
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.recorder.draw_polygon(points, style)?;
        let points = self.copy_slice(points, "prepared polygon points")?;
        self.calls.push(PlannedCall::Polygon {
            points,
            style: style.clone(),
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
        self.recorder
            .draw_linear_gradient(bounds, start, end, stops)?;
        let stops = self.copy_slice(stops, "prepared gradient stops")?;
        self.calls.push(PlannedCall::LinearGradient {
            bounds,
            start,
            end,
            stops,
        });
        Ok(())
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.recorder.draw_text(position, text, style)?;
        let placement = PlannedTextPlacement::middle(position, style.anchor, style.rotation);
        let plan = self.plan_text(
            text,
            style,
            TextLineHeight::Multiplier(1.2),
            alignment(style.anchor),
            "raw text capture",
        )?;
        self.calls
            .push(PlannedCall::RawText(PlannedTextCall { placement, plan }));
        Ok(())
    }

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.recorder.draw_planned_text(placement, plan)?;
        self.calls.push(PlannedCall::PlannedText(PlannedTextCall {
            placement,
            plan: plan.clone(),
        }));
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
        self.recorder
            .draw_rounded_rect(x, y, width, height, radius, style)?;
        self.calls.push(PlannedCall::RoundedRect {
            x,
            y,
            width,
            height,
            radius,
            style: style.clone(),
        });
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.recorder.draw_bracket(bracket, style)?;
        let label = self.plan_raw_bracket(bracket, style)?;
        let (bracket, style) = self.copy_bracket(bracket, style)?;
        self.calls.push(PlannedCall::RawBracket(PlannedBracketCall {
            bracket,
            style,
            label,
        }));
        Ok(())
    }

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.recorder.draw_planned_bracket(call)?;
        let (bracket, style) = self.copy_bracket(call.bracket, call.style)?;
        self.calls
            .push(PlannedCall::PlannedBracket(PlannedBracketCall {
                bracket,
                style,
                label: call.label.clone(),
            }));
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.recorder.end()
    }
}

#[cfg(test)]
pub(crate) struct TextReplayBackend<'a> {
    backend: &'a mut dyn Backend,
    plan: &'a TextReplayPlan,
    next: usize,
}

#[cfg(test)]
impl<'a> TextReplayBackend<'a> {
    pub(crate) fn new(backend: &'a mut dyn Backend, plan: &'a TextReplayPlan) -> Self {
        #[cfg(test)]
        TEXT_REPLAY_SESSIONS.with(|calls| calls.set(calls.get() + 1));
        Self {
            backend,
            plan,
            next: 0,
        }
    }

    fn mismatch(&self, expected: &PlannedCall, actual: &str) -> Error {
        Error::Render(format!(
            "text replay call {} expected {} but received {actual}",
            self.next,
            expected.kind()
        ))
    }

    fn verify_consumed(&self) -> Result<(), Error> {
        if self.next == self.plan.calls.len() {
            Ok(())
        } else {
            Err(Error::Render(format!(
                "text replay consumed {} of {} planned calls",
                self.next,
                self.plan.calls.len()
            )))
        }
    }
}

#[cfg(test)]
impl Backend for TextReplayBackend<'_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.backend.begin(width, height)
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.backend.begin_layer(name)
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.backend.begin_layer_with_role(name, role)
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.backend.end_layer()
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.backend.begin_clip(region)
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.backend.end_clip()
    }

    fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: Rgba,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.backend.draw_surface(bounds, paint, role)
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.backend.draw_scatter(points, style)
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.backend.draw_line(points, style)
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.backend.draw_polygon(points, style)
    }

    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.backend.draw_linear_gradient(bounds, start, end, stops)
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        let call_index = self.next;
        let expected = self.plan.calls.get(call_index).ok_or_else(|| {
            Error::Render(format!(
                "text replay received unexpected raw text call {call_index} after all planned calls were consumed"
            ))
        })?;
        let PlannedCall::RawText(expected) = expected else {
            return Err(self.mismatch(expected, "raw text"));
        };
        let _ = text;
        if !same_placement(
            expected.placement,
            PlannedTextPlacement::middle(position, style.anchor, style.rotation),
        ) {
            return Err(Error::Render(format!(
                "text replay raw text call {} does not match its captured source, style, or placement",
                self.next
            )));
        }
        self.next = call_index + 1;
        self.backend
            .draw_planned_text(expected.placement, &expected.plan)
    }

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        let call_index = self.next;
        let expected = self.plan.calls.get(call_index).ok_or_else(|| {
            Error::Render(format!(
                "text replay received unexpected planned text call {call_index} after all planned calls were consumed"
            ))
        })?;
        let PlannedCall::PlannedText(expected) = expected else {
            return Err(self.mismatch(expected, "planned text"));
        };
        if !same_placement(expected.placement, placement) || expected.plan != *plan {
            return Err(Error::Render(format!(
                "text replay planned text call {} does not match its captured plan or placement",
                self.next
            )));
        }
        self.next = call_index + 1;
        self.backend
            .draw_planned_text(expected.placement, &expected.plan)
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
        self.backend
            .draw_rounded_rect(x, y, width, height, radius, style)
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        let call_index = self.next;
        let expected = self.plan.calls.get(call_index).ok_or_else(|| {
            Error::Render(format!(
                "text replay received unexpected raw bracket call {call_index} after all planned calls were consumed"
            ))
        })?;
        let PlannedCall::RawBracket(expected) = expected else {
            return Err(self.mismatch(expected, "raw bracket"));
        };
        if !same_bracket(&expected.bracket, bracket) || !same_bracket_style(&expected.style, style)
        {
            return Err(Error::Render(format!(
                "text replay raw bracket call {} does not match its captured geometry, label, or style",
                self.next
            )));
        }
        self.next = call_index + 1;
        self.backend.draw_planned_bracket(PlannedSignificanceCall {
            bracket: &expected.bracket,
            style: &expected.style,
            label: &expected.label,
        })
    }

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        let call_index = self.next;
        let expected = self.plan.calls.get(call_index).ok_or_else(|| {
            Error::Render(format!(
                "text replay received unexpected planned bracket call {call_index} after all planned calls were consumed"
            ))
        })?;
        let PlannedCall::PlannedBracket(expected) = expected else {
            return Err(self.mismatch(expected, "planned bracket"));
        };
        if !same_bracket(&expected.bracket, call.bracket)
            || !same_bracket_style(&expected.style, call.style)
            || expected.label != *call.label
        {
            return Err(Error::Render(format!(
                "text replay planned bracket call {} does not match its captured geometry, style, or deferred label plan",
                self.next
            )));
        }
        self.next = call_index + 1;
        self.backend.draw_planned_bracket(PlannedSignificanceCall {
            bracket: &expected.bracket,
            style: &expected.style,
            label: &expected.label,
        })
    }

    fn end(&mut self) -> Result<(), Error> {
        backend::finish_scope(self.verify_consumed(), self.backend.end(), "text replay")
    }
}

#[cfg(test)]
fn same_point(left: Point2D, right: Point2D) -> bool {
    left.x.to_bits() == right.x.to_bits() && left.y.to_bits() == right.y.to_bits()
}

#[cfg(test)]
fn same_placement(left: PlannedTextPlacement, right: PlannedTextPlacement) -> bool {
    same_point(left.position, right.position)
        && left.anchor == right.anchor
        && left.rotation.to_bits() == right.rotation.to_bits()
        && left.origin == right.origin
}

#[cfg(test)]
fn same_bracket(left: &SignificanceBracket, right: &SignificanceBracket) -> bool {
    left.x_left.to_bits() == right.x_left.to_bits()
        && left.x_right.to_bits() == right.x_right.to_bits()
        && left.y.to_bits() == right.y.to_bits()
        && left.label == right.label
        && left.orientation == right.orientation
}

#[cfg(test)]
fn same_bracket_style(left: &BracketStyle, right: &BracketStyle) -> bool {
    left.color == right.color
        && left.label_color == right.label_color
        && left.width.to_bits() == right.width.to_bits()
        && left.font_family == right.font_family
        && left.font_size == right.font_size
        && left.line_height.to_bits() == right.line_height.to_bits()
        && left.weight == right.weight
        && left.label_gap.to_bits() == right.label_gap.to_bits()
}

#[derive(Default)]
pub(super) struct NullBackend {
    recorder: ValidationRecorder,
}

#[derive(Default)]
struct ValidationRecorder {
    state: NullCanvasState,
    scopes: Vec<NullScope>,
    validation: crate::render::coordinates::ValidationScratch,
    fixed: bool,
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

impl ValidationRecorder {
    fn push_scope(&mut self, scope: NullScope) -> Result<(), Error> {
        if self.scopes.len() == self.scopes.capacity() {
            if self.fixed {
                return Err(Error::Render("Null exceeded reserved scopes".into()));
            }
            self.scopes
                .try_reserve(1)
                .map_err(|error| Error::Render(format!("Null scope reservation: {error}")))?;
        }
        self.scopes.push(scope);
        Ok(())
    }
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
    validate: impl FnOnce(&mut dyn Backend) -> Result<T, Error>,
) -> Result<T, Error> {
    if width == 0 || height == 0 {
        return Err(Error::Render(
            "planning validation dimensions must be positive".into(),
        ));
    }
    // Measurement validates commands without opening a final backend lifecycle.
    let mut backend = ValidationRecorder {
        state: NullCanvasState::Open,
        ..ValidationRecorder::default()
    };
    let validated = validate(&mut backend);
    let ended = if backend.scopes.is_empty() {
        Ok(())
    } else {
        Err(Error::Render(
            "planning validation has unclosed scopes".into(),
        ))
    };
    backend::finish_scope(validated, ended, scope)
}

pub(crate) fn validate_text_replay_plan(
    width: u32,
    height: u32,
    plan: &TextReplayPlan,
    scope: &str,
) -> Result<(), Error> {
    let mut storage = plan
        .storage
        .try_borrow_mut()
        .map_err(|_| Error::Render("prepared scene storage is already borrowed".into()))?;
    let backend = storage
        .null
        .as_mut()
        .ok_or_else(|| Error::Render("prepared scene was already validated".into()))?;
    backend.begin(width, height)?;
    let result = plan.replay_open(backend);
    let result = backend::finish_scope(result, backend.end(), scope);
    if result.is_ok() {
        storage.null = None;
    }
    result
}

impl Backend for ValidationRecorder {
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
        self.push_scope(NullScope::Layer)?;
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
        self.push_scope(NullScope::Clip)?;
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
        for point in crate::render::coordinates::canonical_points_iter(points, "scatter") {
            point?;
        }
        backend::validate_point_style(style)?;
        if style.shape == crate::PointShape::TriangleUp {
            for (index, point) in points.iter().copied().enumerate() {
                crate::render::coordinates::canonical_triangle(point, style.radius, index)?;
            }
        }
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_open("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        self.validation.line(points)?;
        backend::validate_line_style(style)
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_open("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        self.validation.polygon(points)?;
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
        crate::render::coordinates::validate_gradient_geometry(bounds, start, end, stops)?;
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

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.ensure_open("draw_planned_text")?;
        backend::validate_planned_text(placement, plan)?;
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

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.ensure_open("draw_planned_bracket")?;
        backend::validate_planned_bracket(call)?;
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

macro_rules! null_backend_methods {
    ($(fn $name:ident($($arg:ident: $ty:ty),*);)*) => {
        impl Backend for NullBackend {
            fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
                #[cfg(test)]
                NULL_BEGIN_CALLS.with(|calls| calls.set(calls.get() + 1));
                self.recorder.begin(width, height)
            }
            fn end(&mut self) -> Result<(), Error> {
                #[cfg(test)]
                NULL_END_CALLS.with(|calls| calls.set(calls.get() + 1));
                self.recorder.end()
            }
            $(fn $name(&mut self, $($arg: $ty),*) -> Result<(), Error> {
                self.recorder.$name($($arg),*)
            })*
        }
    };
}

null_backend_methods! {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::backend::{BracketOrientation, Rgba, SurfaceRole};
    use crate::render::svg_backend::{SvgBackend, SVG_PLANNED_TEXT_CALLS, SVG_RAW_TEXT_CALLS};

    fn paired_draw_result(
        draw: impl Fn(&mut dyn Backend) -> Result<(), Error>,
    ) -> (Result<(), String>, String) {
        let mut null = NullBackend::default();
        null.begin(100, 100).unwrap();
        let null_result = draw(&mut null).map_err(|error| error.to_string());
        null.end().unwrap();

        let mut svg = SvgBackend::in_memory(100, 100);
        svg.begin(100, 100).unwrap();
        let svg_result = draw(&mut svg).map_err(|error| error.to_string());
        svg.end().unwrap();
        let output = String::from_utf8(svg.into_bytes().unwrap()).unwrap();

        assert_eq!(null_result, svg_result);
        (null_result, output)
    }

    fn assert_paired_rejection(draw: impl Fn(&mut dyn Backend) -> Result<(), Error>) {
        assert!(paired_draw_result(draw).0.is_err());
    }

    fn assert_paired_success(draw: impl Fn(&mut dyn Backend) -> Result<(), Error>) {
        assert_eq!(paired_draw_result(draw).0, Ok(()));
    }

    fn replay_text_style(theme: &crate::theme::ResolvedTheme) -> TextStyle {
        theme
            .typography
            .title
            .backend_style(TextAnchor::Middle, 0.0)
    }

    fn capture_test_plan(
        theme: &crate::theme::ResolvedTheme,
        mut capture: impl FnMut(&mut dyn Backend) -> Result<(), Error>,
    ) -> crate::layout::FrozenRoot<TextReplayPlan> {
        crate::layout::plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), assigned, (), final_stage| {
                let (width, height) = assigned.dimensions();
                capture_text_replay_plan(
                    width,
                    height,
                    "test capture",
                    theme,
                    final_stage,
                    &mut capture,
                )
            },
        )
        .unwrap()
    }

    fn storage_fixture(
        backend: &mut dyn Backend,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(), Error> {
        let bounds = PanelBox::new(0.0, 100.0, 0.0, 100.0);
        let points = [
            Point2D::new(10.0, 10.0),
            Point2D::new(30.0, 40.0),
            Point2D::new(50.0, 10.0),
        ];
        let line = LineStyle::short_dashed((1, 2, 3), 0.8);
        let fill = FillStyle {
            color: (10, 20, 30),
            stroke: (1, 2, 3),
            stroke_width: 0.5,
            opacity: 0.75,
        };
        backend.draw_surface(bounds, Rgba::rgb(255, 255, 255), SurfaceRole::CanvasRoot)?;
        backend.draw_surface(bounds, Rgba::new(1, 2, 3, 128), SurfaceRole::PlotPanel)?;
        backend.begin_layer_with_role("grid<&", LayerRole::Grid)?;
        backend.draw_line(&points, &line)?;
        backend.end_layer()?;
        backend.begin_layer("body<&")?;
        backend.begin_clip(bounds)?;
        backend.draw_scatter(&points, &PointStyle::new((1, 2, 3), 2))?;
        backend.draw_scatter(
            &points,
            &PointStyle::plain((3, 2, 1), 2).with_shape(crate::PointShape::TriangleUp),
        )?;
        backend.draw_polygon(&points, &fill)?;
        backend.draw_rounded_rect(5.0, 5.0, 20.0, 20.0, 2.0, &fill)?;
        backend.draw_linear_gradient(
            bounds,
            points[0],
            points[1],
            &[
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
        )?;
        backend.draw_text(
            Point2D::new(50.0, 50.0),
            "A<&\u{03a9}",
            &replay_text_style(theme),
        )?;
        backend.draw_bracket(
            &SignificanceBracket {
                x_left: 10.0,
                x_right: 30.0,
                y: 60.0,
                label: "*".into(),
                orientation: BracketOrientation::Vertical,
            },
            &BracketStyle {
                color: Rgba::rgb(1, 2, 3),
                label_color: Rgba::rgb(3, 2, 1),
                width: 0.8,
                font_family: theme.font_family.clone(),
                font_size: 12,
                line_height: 14.4,
                weight: crate::TextWeight::Normal,
                label_gap: 4.0,
            },
        )?;
        backend.end_clip()?;
        backend.end_layer()
    }

    #[test]
    fn prepared_svg_replay_moves_the_actual_reserved_buffers_unchanged() {
        let theme = crate::Theme::Publication.resolve();
        crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow_mut().clear());
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        NULL_END_CALLS.with(|calls| calls.set(0));
        crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| calls.set(0));
        crate::render::svg_backend::SVG_END_CALLS.with(|calls| calls.set(0));
        let plan = capture_test_plan(&theme, |backend| storage_fixture(backend, &theme));
        let plan = plan.plan();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        NULL_END_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        crate::render::svg_backend::SVG_END_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert!(plan.take_svg_backend().is_err());
        validate_text_replay_plan(100, 100, plan, "reserved Null").unwrap();
        assert!(validate_text_replay_plan(100, 100, plan, "duplicate Null").is_err());
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        NULL_END_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        let mut reserved = plan.take_svg_backend().unwrap();
        let addresses = reserved.reserved_storage_addresses();
        SVG_RAW_TEXT_CALLS.with(|calls| calls.set(0));
        plan.replay(100, 100, &mut reserved, "reserved SVG")
            .unwrap();
        let output = reserved.into_output().unwrap();
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), shapes));
        crate::font::canonical::FONT_OUTLINE_GLYPHS
            .with(|glyphs| assert!(glyphs.borrow().is_empty()));
        crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        crate::render::svg_backend::SVG_END_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        assert_eq!(addresses, output.storage_addresses());
        SVG_RAW_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        let mut reference = SvgBackend::in_memory_with_fonts(
            100,
            100,
            theme.prepared_font_environment().unwrap().clone(),
        );
        plan.replay(100, 100, &mut reference, "reference SVG")
            .unwrap();
        let reference = reference.into_output().unwrap();
        let scene = crate::scene::RenderedScene::new(100, 100, Some(output)).unwrap();
        let reference = crate::scene::RenderedScene::new(100, 100, Some(reference)).unwrap();
        assert_eq!(scene.svg_bytes(), reference.svg_bytes());
        assert_eq!(
            scene.svg_with_text_as_paths().unwrap(),
            reference.svg_with_text_as_paths().unwrap()
        );
        assert_eq!(
            scene.png_bytes(&crate::PngOptions::default()).unwrap(),
            reference.png_bytes(&crate::PngOptions::default()).unwrap()
        );
    }

    #[test]
    fn prepared_storage_preserves_legacy_primitive_svg_and_png_bytes() {
        use sha2::{Digest as _, Sha256};
        let theme = crate::Theme::Publication.resolve();
        let root = capture_test_plan(&theme, |backend| {
            backend.draw_surface(
                PanelBox::new(0.0, 100.0, 0.0, 100.0),
                Rgba::rgb(255, 255, 255),
                SurfaceRole::CanvasRoot,
            )?;
            backend.draw_scatter(
                &[Point2D::new(10.0, 20.0)],
                &PointStyle::plain((1, 2, 3), 2).with_opacity(1.0),
            )
        });
        let plan = root.plan();
        validate_text_replay_plan(100, 100, plan, "legacy fixture Null").unwrap();
        let mut writer = plan.take_svg_backend().unwrap();
        plan.replay(100, 100, &mut writer, "legacy fixture SVG")
            .unwrap();
        let scene = crate::scene::RenderedScene::new(100, 100, writer.into_output()).unwrap();
        let legacy = concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\" viewBox=\"0 0 100 100\" role=\"img\">\n",
            "<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>\n",
            "<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n",
            "<circle cx=\"10.000\" cy=\"80.000\" r=\"2\" fill=\"#010203\" fill-opacity=\"1\"/>\n",
            "</svg>\n",
        ).as_bytes();
        assert_eq!(scene.svg_bytes(), legacy);
        assert_eq!(scene.svg_with_text_as_paths().unwrap(), legacy);
        let options = crate::PngOptions {
            sizing: crate::PngSizing::Source,
            compression: crate::PngCompression::Balanced,
        };
        let target = crate::render::svg_backend::PngTargetPlan::new(
            resvg::tiny_skia::IntSize::from_wh(100, 100).unwrap(),
            &options,
        )
        .unwrap();
        let legacy_png =
            crate::render::svg_backend::rasterize_svg_bytes_to_png(legacy, target, &options)
                .unwrap();
        let png = scene.png_bytes(&options).unwrap();
        assert_eq!(png, legacy_png);
        println!(
            "r4 legacy SVG SHA-256: {:x}",
            Sha256::digest(scene.svg_bytes())
        );
        println!("r4 legacy PNG SHA-256: {:x}", Sha256::digest(&png));
    }

    #[test]
    fn raw_font_stack_failure_precedes_final_backend_lifecycles() {
        let theme = crate::Theme::Publication.resolve();
        for bracket in [false, true] {
            NULL_BEGIN_CALLS.with(|calls| calls.set(0));
            crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| calls.set(0));
            let result = crate::layout::plan_exact_root(
                100,
                100,
                |_| Ok(()),
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), stage| {
                    capture_text_replay_plan(
                        100,
                        100,
                        "invalid raw font",
                        &theme,
                        stage,
                        |backend| {
                            if bracket {
                                backend.draw_bracket(
                                    &SignificanceBracket {
                                        x_left: 10.0,
                                        x_right: 30.0,
                                        y: 60.0,
                                        label: "*".into(),
                                        orientation: BracketOrientation::Vertical,
                                    },
                                    &BracketStyle {
                                        color: Rgba::rgb(0, 0, 0),
                                        label_color: Rgba::rgb(0, 0, 0),
                                        width: 1.0,
                                        font_family: String::new(),
                                        font_size: 12,
                                        line_height: 14.4,
                                        weight: crate::TextWeight::Normal,
                                        label_gap: 4.0,
                                    },
                                )
                            } else {
                                let mut style = replay_text_style(&theme);
                                style.font_family.clear();
                                backend.draw_text(Point2D::new(50.0, 50.0), "raw text", &style)
                            }
                        },
                    )
                },
            );
            let Err(error) = result else {
                panic!("an empty raw font stack must be rejected");
            };
            assert!(error.to_string().contains("capture"), "{error}");
            NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }

    #[test]
    fn every_nonempty_canonical_storage_refusal_precedes_null() {
        let theme = crate::Theme::Publication.resolve();
        for payload in [
            "canonical SVG scopes",
            "canonical SVG validation points",
            "canonical SVG run-order scratch",
            "canonical SVG replacements",
            "canonical SVG document",
            "canonical SVG body",
            "canonical SVG root surfaces",
            "canonical SVG panel surfaces",
            "canonical SVG grid",
            "canonical SVG clips",
            "canonical SVG gradients",
            "final Null scopes",
            "final Null validation points",
        ] {
            NULL_BEGIN_CALLS.with(|calls| calls.set(0));
            crate::render::svg_backend::REFUSE_STORAGE.with(|refuse| refuse.set(Some(payload)));
            let result = crate::layout::plan_exact_root(
                100,
                100,
                |_| Ok(()),
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), stage| {
                    capture_text_replay_plan(
                        100,
                        100,
                        "storage refusal",
                        &theme,
                        stage,
                        |backend| storage_fixture(backend, &theme),
                    )
                },
            );
            let Err(error) = result else {
                panic!("storage reservation must fail: {payload}");
            };
            assert!(error.to_string().contains(payload), "{error}");
            NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            crate::render::svg_backend::REFUSE_STORAGE
                .with(|refuse| assert_eq!(refuse.get(), None));
        }
    }

    #[test]
    fn all_root_preflights_reserve_before_the_only_null_lifecycle() {
        let scatter = || {
            crate::Plot::scatter(
                crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
                crate::NumericValues::new("y", [2.0, 4.0, 1.0, 3.0]),
            )
            .size(960, 640)
        };
        let plot = scatter();
        let faceted = scatter().facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap(),
        ));
        let canvas = crate::Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, scatter())
            .unwrap();
        for root in 0..3 {
            for payload in [
                "canonical SVG body",
                "canonical SVG document",
                "canonical SVG replacements",
                "canonical SVG scopes",
                "canonical SVG validation points",
                "final Null scopes",
                "final Null validation points",
            ] {
                NULL_BEGIN_CALLS.with(|calls| calls.set(0));
                crate::render::svg_backend::REFUSE_STORAGE.with(|refuse| refuse.set(Some(payload)));
                let result = match root {
                    0 => plot.preflight(),
                    1 => faceted.preflight(),
                    _ => canvas.preflight(),
                };
                let error = result.unwrap_err();
                assert!(
                    error.to_string().contains(payload),
                    "root {root}, {payload}: {error}"
                );
                NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            }
            NULL_BEGIN_CALLS.with(|calls| calls.set(0));
            match root {
                0 => plot.preflight(),
                1 => faceted.preflight(),
                _ => canvas.preflight(),
            }
            .unwrap();
            NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        }
    }

    #[test]
    fn raw_text_capture_is_immutable_and_svg_replay_uses_only_planned_calls() {
        let theme = crate::Theme::Publication.resolve();
        let style = replay_text_style(&theme);
        let position = Point2D::new(40.0, 60.0);
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        let plan = capture_test_plan(&theme, |backend| {
            backend.draw_text(position, "captured text", &style)
        });
        let plan = plan.plan();
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));

        assert_eq!(plan.call_count(), 1);
        for replay_plan in [plan, plan] {
            SVG_RAW_TEXT_CALLS.with(|calls| calls.set(0));
            SVG_PLANNED_TEXT_CALLS.with(|calls| calls.set(0));
            let mut svg = SvgBackend::in_memory(100, 100);
            let mut replay = TextReplayBackend::new(&mut svg, replay_plan);
            replay.begin(100, 100).unwrap();
            replay.draw_text(position, "captured text", &style).unwrap();
            replay.end().unwrap();

            SVG_RAW_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            SVG_PLANNED_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        }
    }

    #[test]
    fn capture_rejects_extra_calls_before_retaining_or_shaping_the_payload() {
        let theme = crate::Theme::Publication.resolve();
        let mut pass = 0;
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), final_stage| {
                capture_text_replay_plan(
                    100,
                    100,
                    "extra call test",
                    &theme,
                    final_stage,
                    |backend| {
                        pass += 1;
                        if pass == 2 {
                            let mut invalid = replay_text_style(&theme);
                            invalid.font_size = 0;
                            backend.draw_text(Point2D::new(0.0, 0.0), "not shaped", &invalid)?;
                        }
                        Ok(())
                    },
                )
            },
        );
        let Err(error) = result else {
            panic!("capture must reject a call absent from counting");
        };
        assert!(error.to_string().contains("before retention"));
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn retained_capture_payload_refusal_precedes_null() {
        let theme = crate::Theme::Publication.resolve();
        for payload in [
            "prepared layer name",
            "prepared scatter points",
            "prepared gradient stops",
        ] {
            let mut pass = 0;
            NULL_BEGIN_CALLS.with(|calls| calls.set(0));
            let result = crate::layout::plan_exact_root(
                100,
                100,
                |_| Ok(()),
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), final_stage| {
                    capture_text_replay_plan(
                        100,
                        100,
                        "payload refusal",
                        &theme,
                        final_stage,
                        |backend| {
                            pass += 1;
                            if pass == 2 {
                                crate::layout::kernel::force_reservation_failure(
                                    crate::layout::kernel::ReservationStage::Final,
                                );
                            }
                            match payload {
                                "prepared layer name" => {
                                    backend.begin_layer("retained name")?;
                                    backend.end_layer()
                                }
                                "prepared scatter points" => backend.draw_scatter(
                                    &[Point2D::new(1.0, 2.0)],
                                    &PointStyle::plain((0, 0, 0), 1),
                                ),
                                _ => backend.draw_linear_gradient(
                                    PanelBox::new(0.0, 10.0, 0.0, 10.0),
                                    Point2D::new(0.0, 0.0),
                                    Point2D::new(10.0, 10.0),
                                    &[
                                        GradientStop::new(0.0, (0, 0, 0)),
                                        GradientStop::new(1.0, (255, 255, 255)),
                                    ],
                                ),
                            }
                        },
                    )
                },
            );
            let Err(error) = result else {
                panic!("capture payload reservation must refuse");
            };
            assert!(error.to_string().contains(payload), "{error}");
            NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }

    #[test]
    fn frozen_scene_replay_consumes_exactly_the_captured_calls() {
        let theme = crate::Theme::Publication.resolve();
        let style = replay_text_style(&theme);
        let position = Point2D::new(40.0, 60.0);
        let plan = capture_test_plan(&theme, |backend| {
            backend.draw_text(position, "first", &style)?;
            backend.draw_text(position, "second", &style)
        });
        let plan = plan.plan();

        assert_eq!(plan.call_count(), 2);
        validate_text_replay_plan(100, 100, plan, "test Null replay").unwrap();
        SVG_RAW_TEXT_CALLS.with(|calls| calls.set(0));
        SVG_PLANNED_TEXT_CALLS.with(|calls| calls.set(0));
        let mut svg = SvgBackend::in_memory(100, 100);
        plan.replay(100, 100, &mut svg, "test SVG replay").unwrap();
        SVG_RAW_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        SVG_PLANNED_TEXT_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }

    #[test]
    fn planned_significance_capture_preserves_the_exact_deferred_plan() {
        let theme = crate::Theme::Publication.resolve();
        let resolved = theme
            .measure_text(
                "**",
                &theme.typography.significance_label,
                PhysicalTextAlignment::Center,
                "test significance label",
                true,
            )
            .unwrap();
        let deferred = DeferredAnnotationTextPlan::from_resolved(resolved);
        let bracket = SignificanceBracket {
            x_left: 10.0,
            x_right: 20.0,
            y: 30.0,
            label: "**".into(),
            orientation: BracketOrientation::Vertical,
        };
        let style = BracketStyle {
            color: Rgba::rgb(0, 0, 0),
            label_color: deferred.plan.style.paint,
            width: 1.0,
            font_family: deferred.plan.style.stack_source.clone(),
            font_size: deferred.plan.style.font_size,
            line_height: deferred.plan.line_height(),
            weight: deferred.plan.style.weight,
            label_gap: 4.0,
        };
        let plan = capture_test_plan(&theme, |backend| {
            backend.draw_planned_bracket(PlannedSignificanceCall {
                bracket: &bracket,
                style: &style,
                label: &deferred,
            })
        });
        let plan = plan.plan();

        let [PlannedCall::PlannedBracket(captured)] = plan.calls.as_slice() else {
            panic!("expected one planned significance call");
        };
        assert_eq!(captured.label, deferred);

        let mut null = NullBackend::default();
        let mut replay = TextReplayBackend::new(&mut null, plan);
        replay.begin(100, 100).unwrap();
        replay
            .draw_planned_bracket(PlannedSignificanceCall {
                bracket: &bracket,
                style: &style,
                label: &deferred,
            })
            .unwrap();
        replay.end().unwrap();
    }

    #[test]
    fn raw_bracket_capture_freezes_and_replays_one_label_plan() {
        let theme = crate::Theme::Publication.resolve();
        let bracket = SignificanceBracket {
            x_left: 10.0,
            x_right: 20.0,
            y: 30.0,
            label: "***".into(),
            orientation: BracketOrientation::Vertical,
        };
        let style = BracketStyle {
            color: Rgba::rgb(0, 0, 0),
            label_color: Rgba::rgb(1, 2, 3),
            width: 1.0,
            font_family: theme.font_family.clone(),
            font_size: 12,
            line_height: 14.4,
            weight: crate::TextWeight::Semibold,
            label_gap: 4.0,
        };
        let plan = capture_test_plan(&theme, |backend| backend.draw_bracket(&bracket, &style));
        let plan = plan.plan();

        let [PlannedCall::RawBracket(captured)] = plan.calls.as_slice() else {
            panic!("expected one raw bracket call");
        };
        assert_eq!(captured.label.plan.source, bracket.label);
        assert_eq!(captured.label.plan.style.paint, style.label_color);
        assert!(!captured.label.exact_override);

        let mut null = NullBackend::default();
        let mut replay = TextReplayBackend::new(&mut null, plan);
        replay.begin(100, 100).unwrap();
        replay.draw_bracket(&bracket, &style).unwrap();
        replay.end().unwrap();
    }

    #[test]
    fn null_backend_preserves_degenerate_line_and_polygon_noops() {
        let invalid_line = LineStyle::solid((0, 0, 0), f32::NAN);
        let invalid_fill = FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width: f32::NAN,
            opacity: f64::NAN,
        };

        let (result, svg) = paired_draw_result(|backend| {
            backend.draw_line(&[Point2D::new(f64::NAN, f64::NAN)], &invalid_line)?;
            backend.draw_polygon(
                &[
                    Point2D::new(f64::NAN, f64::NAN),
                    Point2D::new(f64::NAN, f64::NAN),
                ],
                &invalid_fill,
            )
        });

        assert_eq!(result, Ok(()));
        assert!(!svg.contains("<polyline"));
        assert!(!svg.contains("<polygon"));
    }

    #[test]
    fn null_and_svg_share_non_degenerate_line_and_polygon_failures() {
        let invalid_fill = FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width: f32::NAN,
            opacity: f64::NAN,
        };

        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)],
                &LineStyle::solid((0, 0, 0), 1.0),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)],
                &LineStyle::solid((0, 0, 0), f32::NAN),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_polygon(
                &[
                    Point2D::new(0.0, 0.0),
                    Point2D::new(1.0, 0.0),
                    Point2D::new(0.0, 1.0),
                ],
                &invalid_fill,
            )
        });
    }

    #[test]
    fn null_and_svg_share_gradient_geometry_and_stop_failures() {
        let bounds = PanelBox::new(0.0, 10.0, 0.0, 10.0);
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(10.0, 10.0);
        let stops = [
            GradientStop::new(0.0, (0, 0, 0)),
            GradientStop::new(1.0, (255, 255, 255)),
        ];

        assert_paired_rejection(|backend| {
            backend.draw_linear_gradient(PanelBox::new(0.0, 0.0, 0.0, 10.0), start, end, &stops)
        });
        assert_paired_rejection(|backend| {
            backend.draw_linear_gradient(bounds, start, start, &stops)
        });
        assert_paired_rejection(|backend| {
            backend.draw_linear_gradient(bounds, start, end, &[GradientStop::new(0.0, (0, 0, 0))])
        });
        assert_paired_rejection(|backend| {
            backend.draw_linear_gradient(
                bounds,
                start,
                end,
                &[
                    GradientStop::new(0.0, (0, 0, 0)),
                    GradientStop::new(0.5, (127, 127, 127)),
                    GradientStop::new(0.5, (128, 128, 128)),
                    GradientStop::new(1.0, (255, 255, 255)),
                ],
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_linear_gradient(
                bounds,
                start,
                end,
                &[
                    GradientStop::new(0.0, (0, 0, 0)),
                    GradientStop::new(0.000_000_4, (127, 127, 127)),
                    GradientStop::new(1.0, (255, 255, 255)),
                ],
            )
        });
    }

    #[test]
    fn null_and_svg_share_surface_geometry_validation() {
        assert_paired_success(|backend| {
            backend.draw_surface(
                PanelBox::new(1.0, 99.0, 2.0, 98.0),
                Rgba::new(1, 2, 3, 0),
                SurfaceRole::PlotPanel,
            )
        });
        for bounds in [
            PanelBox::new(f64::NAN, 10.0, 0.0, 10.0),
            PanelBox::new(10.0, 10.0, 0.0, 10.0),
            PanelBox::new(0.0, 0.0004, 0.0, 10.0),
            PanelBox::new(-f64::MAX, f64::MAX, 0.0, 10.0),
        ] {
            assert_paired_rejection(|backend| {
                backend.draw_surface(bounds, Rgba::rgb(1, 2, 3), SurfaceRole::CanvasRoot)
            });
        }
    }

    #[test]
    fn null_and_svg_share_rounded_rect_geometry_style_and_order() {
        let valid_fill = FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width: 1.0,
            opacity: 1.0,
        };
        let invalid_fill = FillStyle {
            color: (0, 0, 0),
            stroke: (0, 0, 0),
            stroke_width: f32::NAN,
            opacity: f64::NAN,
        };

        assert_paired_success(|backend| {
            backend.draw_rounded_rect(1.0, 2.0, 0.0, 0.0, 0.0, &valid_fill)
        });
        assert_paired_rejection(|backend| {
            backend.draw_rounded_rect(1.0, 2.0, -1.0, 4.0, 1.0, &valid_fill)
        });
        assert_paired_rejection(|backend| {
            backend.draw_rounded_rect(1.0, 2.0, 4.0, 4.0, 3.0, &valid_fill)
        });
        assert_paired_rejection(|backend| {
            backend.draw_rounded_rect(f64::MAX, 2.0, f64::MAX, 4.0, 1.0, &valid_fill)
        });
        assert_paired_rejection(|backend| {
            backend.draw_rounded_rect(1.0, 2.0, 4.0, 4.0, 1.0, &invalid_fill)
        });
        let (result, _) = paired_draw_result(|backend| {
            backend.draw_rounded_rect(1.0, 2.0, -1.0, 4.0, 1.0, &invalid_fill)
        });
        assert!(result
            .unwrap_err()
            .contains("dimensions and radius must be non-negative"));
    }

    #[test]
    fn null_and_svg_share_bracket_geometry_and_style_failures() {
        let bracket = SignificanceBracket {
            x_left: 10.0,
            x_right: 20.0,
            y: 20.0,
            label: "*".into(),
            orientation: BracketOrientation::Vertical,
        };
        let bracket_style = BracketStyle {
            color: (0, 0, 0).into(),
            width: 1.0,
            label_color: (0, 0, 0).into(),
            font_family: "sans-serif".into(),
            font_size: 12,
            line_height: 14.4,
            weight: crate::TextWeight::Normal,
            label_gap: 4.0,
        };

        let mut invalid_geometry = bracket.clone();
        invalid_geometry.y = f64::NAN;
        assert_paired_rejection(|backend| backend.draw_bracket(&invalid_geometry, &bracket_style));
        let mut collapsed = bracket.clone();
        collapsed.x_left = 0.0;
        collapsed.x_right = 0.0004;
        assert_paired_rejection(|backend| backend.draw_bracket(&collapsed, &bracket_style));
        let mut invalid_style = bracket_style.clone();
        invalid_style.width = f32::NAN;
        assert_paired_rejection(|backend| backend.draw_bracket(&bracket, &invalid_style));
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
