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
    PlannedSignificanceCall, PlannedTextPlacement, Point2D, PointStyle, SignificanceBracket,
    SurfaceRole, TextAnchor, TextStyle,
};
use crate::render::coordinates::CanonicalPoint;
use crate::render::coordinates::{CanonicalBox, CanonicalRoundedBox};
use crate::render::primitives::{
    self, CanonicalFillStyle, CanonicalGradientFill, CanonicalLineStyle, CanonicalPlannedBracket,
    CanonicalPlannedText, CanonicalScatter, PathKind,
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
        paint: crate::theme::Color,
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
        #[cfg(test)]
        let _timer = super::measurements::Timer::new(7);
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
            PlannedCall::BeginClip(region) => backend.begin_canonical_clip(*region)?,
            PlannedCall::EndClip => backend.end_clip()?,
            PlannedCall::Surface {
                bounds,
                paint,
                role,
            } => backend.draw_canonical_surface(*bounds, *paint, *role)?,
            PlannedCall::Scatter(scatter) => backend.draw_canonical_scatter(scatter)?,
            PlannedCall::Line { points, style } => backend.draw_canonical_line(points, style)?,
            PlannedCall::Polygon { points, style } => {
                backend.draw_canonical_polygon(points, style)?;
            }
            PlannedCall::LinearGradient(gradient) => backend.draw_canonical_gradient(gradient)?,
            PlannedCall::PlannedText(call) => {
                backend.draw_canonical_text(&call.canonical)?;
            }
            PlannedCall::RoundedRect(rounded) => backend.draw_canonical_rounded_rect(*rounded)?,
            PlannedCall::PlannedBracket(call) => {
                backend.draw_canonical_bracket(&call.canonical)?;
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
    BeginClip(CanonicalBox),
    EndClip,
    Surface {
        bounds: CanonicalBox,
        paint: crate::theme::Color,
        role: SurfaceRole,
    },
    Scatter(CanonicalScatter),
    Line {
        points: Vec<CanonicalPoint>,
        style: CanonicalLineStyle,
    },
    Polygon {
        points: Vec<CanonicalPoint>,
        style: CanonicalFillStyle,
    },
    LinearGradient(CanonicalGradientFill),
    PlannedText(PlannedTextCall),
    RoundedRect(CanonicalRoundedBox),
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
            Self::BeginClip(region) => serializer.begin_canonical_clip(*region)?,
            Self::EndClip => serializer.end_clip()?,
            Self::Surface {
                bounds,
                paint,
                role,
            } => serializer.draw_canonical_surface(*bounds, *paint, *role)?,
            Self::Scatter(scatter) => serializer.draw_canonical_scatter(scatter)?,
            Self::Line { points, style } => serializer.draw_canonical_line(points, style)?,
            Self::Polygon { points, style } => serializer.draw_canonical_polygon(points, style)?,
            Self::LinearGradient(gradient) => serializer.draw_canonical_gradient(gradient)?,
            Self::PlannedText(call) => {
                serializer.draw_canonical_text(&call.canonical)?;
            }
            Self::RoundedRect(rounded) => serializer.draw_canonical_rounded_rect(*rounded)?,
            Self::PlannedBracket(call) => serializer.draw_canonical_bracket(&call.canonical)?,
        }
        serializer.check_buffers()
    }
}

#[cfg(test)]
impl PlannedCall {
    const fn kind(&self) -> &'static str {
        match self {
            Self::PlannedText(_) => "planned text",
            Self::PlannedBracket(_) => "planned bracket",
            _ => "non-text draw call",
        }
    }
}

#[derive(Debug)]
struct PlannedTextCall {
    canonical: CanonicalPlannedText,
    #[cfg(test)]
    placement: PlannedTextPlacement,
    #[cfg(test)]
    plan: ResolvedTextPlan,
}

#[derive(Debug)]
struct PlannedBracketCall {
    canonical: CanonicalPlannedBracket,
    #[cfg(test)]
    bracket: SignificanceBracket,
    #[cfg(test)]
    style: BracketStyle,
    #[cfg(test)]
    label: DeferredAnnotationTextPlan,
}

pub(super) struct TextPlanningBackend<'a, 'b> {
    height: u32,
    font_environment: PreparedFontEnvironment,
    calls: Vec<PlannedCall>,
    final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
}

impl<'a, 'b> TextPlanningBackend<'a, 'b> {
    fn new(
        theme: &crate::theme::ResolvedTheme,
        final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
        height: u32,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(calls.get() + 1));
        Ok(Self {
            height,
            font_environment: theme.prepared_font_environment()?.clone(),
            calls: Vec::new(),
            final_stage,
        })
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
                stack: &style.font_family,
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
        let plan = self
            .font_environment
            .plan_prepared_text(TextPlanInput {
                source: &bracket.label,
                stack: &style.font_family,
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
        for call in &calls {
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
            crate::render::svg_backend::SvgStorageRequirements { scopes },
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
            .reserve_capture(&mut values, source.len(), payload)?;
        values.extend_from_slice(source);
        Ok(values)
    }

    fn copy_string(&mut self, source: &str, payload: &str) -> Result<String, Error> {
        String::from_utf8(self.copy_slice(source.as_bytes(), payload)?)
            .map_err(|error| Error::Render(format!("{payload}: {error}")))
    }

    fn claim_call(&mut self) -> Result<(), Error> {
        // Admit and reserve the command before copying payloads or shaping raw text.
        self.final_stage.claim_capture_call()?;
        self.final_stage
            .reserve_capture(&mut self.calls, 1, "prepared scene calls")
    }

    #[cfg(test)]
    fn copy_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(SignificanceBracket, BracketStyle), Error> {
        let label = self.copy_string(&bracket.label, "prepared bracket label")?;
        let font_family = style
            .font_family
            .copy_with_stage(
                &self
                    .font_environment
                    .font_resources()
                    .map_err(|error| Error::Render(error.to_string()))?,
                None,
            )
            .map_err(|error| Error::Render(error.to_string()))?;
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

pub(crate) fn capture_text_replay_plan(
    width: u32,
    height: u32,
    _scope: &str,
    theme: &crate::theme::ResolvedTheme,
    final_stage: &mut crate::layout::kernel::FinalStage<'_>,
    capture: impl FnOnce(&mut dyn Backend) -> Result<(), Error>,
) -> Result<TextReplayPlan, Error> {
    #[cfg(test)]
    let _timer = super::measurements::Timer::new(2);
    if width == 0 || height == 0 {
        return Err(Error::Render(
            "prepared scene dimensions must be positive".into(),
        ));
    }
    let mut backend = TextPlanningBackend::new(theme, final_stage, height)?;
    #[cfg(test)]
    super::measurements::producer();
    capture(&mut backend)?;
    backend.into_plan(width, height)
}

impl Backend for TextPlanningBackend<'_, '_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "prepared scene dimensions must be positive".into(),
            ));
        }
        Ok(())
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.claim_call()?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayer(name));
        Ok(())
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.claim_call()?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayerWithRole(name, role));
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.claim_call()?;
        self.calls.push(PlannedCall::EndLayer);
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.claim_call()?;
        let region =
            CanonicalBox::from_bottom_up(backend::validate_clip_region(region)?, self.height)?;
        self.calls.push(PlannedCall::BeginClip(region));
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.claim_call()?;
        self.calls.push(PlannedCall::EndClip);
        Ok(())
    }

    fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: crate::theme::Color,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.claim_call()?;
        self.calls.push(PlannedCall::Surface {
            bounds: CanonicalBox::from_bottom_up(
                backend::validate_surface_bounds(bounds)?,
                self.height,
            )?,
            paint,
            role,
        });
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.claim_call()?;
        let scatter = CanonicalScatter::new(points, style, self.height, Some(self.final_stage))?;
        self.calls.push(PlannedCall::Scatter(scatter));
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.claim_call()?;
        if points.len() < 2 {
            return Ok(());
        }
        let points = primitives::path(points, PathKind::Line, self.height, Some(self.final_stage))?;
        self.calls.push(PlannedCall::Line {
            points,
            style: CanonicalLineStyle::new(style)?,
        });
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.claim_call()?;
        if points.len() < 3 {
            return Ok(());
        }
        let points = primitives::path(
            points,
            PathKind::Polygon,
            self.height,
            Some(self.final_stage),
        )?;
        self.calls.push(PlannedCall::Polygon {
            points,
            style: CanonicalFillStyle::new(style)?,
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
        self.claim_call()?;
        let gradient = CanonicalGradientFill::new(
            bounds,
            start,
            end,
            stops,
            self.height,
            Some(self.final_stage),
        )?;
        self.calls.push(PlannedCall::LinearGradient(gradient));
        Ok(())
    }

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.claim_call()?;
        let canonical = crate::render::coordinates::canonical_text(
            position,
            style.rotation,
            crate::render::coordinates::CoordinatePrecision::CssQuantum,
        )?;
        let placement = PlannedTextPlacement::middle(position, style.anchor, style.rotation);
        let plan = self.plan_text(
            text,
            style,
            TextLineHeight::Multiplier(1.2),
            alignment(style.anchor),
            "raw text capture",
        )?;
        backend::validate_text_plan(placement, &plan)?;
        let canonical = CanonicalPlannedText::from_validated(
            canonical,
            placement,
            &plan,
            self.height,
            Some(self.final_stage),
        )?;
        self.calls.push(PlannedCall::PlannedText(PlannedTextCall {
            canonical,
            #[cfg(test)]
            placement,
            #[cfg(test)]
            plan,
        }));
        Ok(())
    }

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.claim_call()?;
        let canonical =
            CanonicalPlannedText::new(placement, plan, self.height, Some(self.final_stage))?;
        self.calls.push(PlannedCall::PlannedText(PlannedTextCall {
            canonical,
            #[cfg(test)]
            placement,
            #[cfg(test)]
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
        self.claim_call()?;
        let rounded = backend::validate_rounded_rect(x, y, width, height, radius)?;
        backend::validate_fill_style(style)?;
        let rounded = CanonicalRoundedBox::from_bottom_up(rounded, style, self.height)?;
        self.calls.push(PlannedCall::RoundedRect(rounded));
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error> {
        self.claim_call()?;
        let geometry = backend::validate_bracket(bracket, style)?;
        let label = self.plan_raw_bracket(bracket, style)?;
        let canonical = CanonicalPlannedBracket::from_geometry(
            geometry,
            PlannedSignificanceCall {
                bracket,
                style,
                label: &label,
            },
            self.height,
            Some(self.final_stage),
        )?;
        #[cfg(test)]
        let (bracket, style) = self.copy_bracket(bracket, style)?;
        self.calls
            .push(PlannedCall::PlannedBracket(PlannedBracketCall {
                canonical,
                #[cfg(test)]
                bracket,
                #[cfg(test)]
                style,
                #[cfg(test)]
                label,
            }));
        Ok(())
    }

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.claim_call()?;
        let canonical = CanonicalPlannedBracket::new(call, self.height, Some(self.final_stage))?;
        #[cfg(test)]
        let (bracket, style) = self.copy_bracket(call.bracket, call.style)?;
        self.calls
            .push(PlannedCall::PlannedBracket(PlannedBracketCall {
                canonical,
                #[cfg(test)]
                bracket,
                #[cfg(test)]
                style,
                #[cfg(test)]
                label: call.label.clone(),
            }));
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
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
        paint: crate::theme::Color,
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
        let PlannedCall::PlannedText(expected) = expected else {
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
        self.backend.draw_canonical_text(&expected.canonical)
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
        self.backend.draw_canonical_text(&expected.canonical)
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
        let PlannedCall::PlannedBracket(expected) = expected else {
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
        self.backend.draw_canonical_bracket(&expected.canonical)
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
        self.backend.draw_canonical_bracket(&expected.canonical)
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
    height: u32,
    state: NullCanvasState,
    scopes: Vec<NullScope>,
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

pub(crate) fn validate_text_replay_plan(
    width: u32,
    height: u32,
    plan: &TextReplayPlan,
    scope: &str,
) -> Result<(), Error> {
    #[cfg(test)]
    let _timer = super::measurements::Timer::new(3);
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
    fn draw_canonical_text(&mut self, _text: &CanonicalPlannedText) -> Result<(), Error> {
        self.ensure_open("draw_planned_text")
    }
    fn draw_canonical_bracket(&mut self, _bracket: &CanonicalPlannedBracket) -> Result<(), Error> {
        self.ensure_open("draw_planned_bracket")
    }
    fn draw_canonical_scatter(&mut self, _scatter: &CanonicalScatter) -> Result<(), Error> {
        self.ensure_open("draw_scatter")
    }
    fn draw_canonical_line(
        &mut self,
        _points: &[CanonicalPoint],
        _style: &CanonicalLineStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_line")
    }
    fn draw_canonical_polygon(
        &mut self,
        _points: &[CanonicalPoint],
        _style: &CanonicalFillStyle,
    ) -> Result<(), Error> {
        self.ensure_open("draw_polygon")
    }
    fn draw_canonical_gradient(&mut self, _gradient: &CanonicalGradientFill) -> Result<(), Error> {
        self.ensure_open("draw_linear_gradient")
    }
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
        self.height = height;
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
        let region =
            CanonicalBox::from_bottom_up(backend::validate_clip_region(region)?, self.height)?;
        self.begin_canonical_clip(region)
    }

    fn begin_canonical_clip(&mut self, _region: CanonicalBox) -> Result<(), Error> {
        self.ensure_open("begin_clip")?;
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
        _paint: crate::theme::Color,
        _role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_open("draw_surface")?;
        CanonicalBox::from_bottom_up(backend::validate_surface_bounds(bounds)?, self.height)?;
        Ok(())
    }

    fn draw_canonical_surface(
        &mut self,
        _bounds: CanonicalBox,
        _paint: crate::theme::Color,
        _role: SurfaceRole,
    ) -> Result<(), Error> {
        self.ensure_open("draw_surface")
    }

    fn draw_canonical_rounded_rect(&mut self, _rounded: CanonicalRoundedBox) -> Result<(), Error> {
        self.ensure_open("draw_rounded_rect")
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.ensure_open("draw_scatter")?;
        CanonicalScatter::new(points, style, self.height, None)?;
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.ensure_open("draw_line")?;
        if points.len() < 2 {
            return Ok(());
        }
        primitives::path(points, PathKind::Line, self.height, None)?;
        CanonicalLineStyle::new(style)?;
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.ensure_open("draw_polygon")?;
        if points.len() < 3 {
            return Ok(());
        }
        primitives::path(points, PathKind::Polygon, self.height, None)?;
        CanonicalFillStyle::new(style)?;
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
        CanonicalGradientFill::new(bounds, start, end, stops, self.height, None)?;
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
        CanonicalPlannedText::new(placement, plan, self.height, None)?;
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
        CanonicalRoundedBox::from_bottom_up(rounded, style, self.height)?;
        Ok(())
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
        CanonicalPlannedBracket::new(call, self.height, None)?;
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
    fn draw_canonical_text(text: &CanonicalPlannedText);
    fn draw_canonical_bracket(bracket: &CanonicalPlannedBracket);
    fn draw_canonical_scatter(scatter: &CanonicalScatter);
    fn draw_canonical_line(points: &[CanonicalPoint], style: &CanonicalLineStyle);
    fn draw_canonical_polygon(points: &[CanonicalPoint], style: &CanonicalFillStyle);
    fn draw_canonical_gradient(gradient: &CanonicalGradientFill);
    fn begin_canonical_clip(region: CanonicalBox);
    fn draw_canonical_surface(bounds: CanonicalBox, paint: crate::theme::Color, role: SurfaceRole);
    fn draw_canonical_rounded_rect(rounded: CanonicalRoundedBox);
    fn begin_layer(name: &str);
    fn begin_layer_with_role(name: &str, role: LayerRole);
    fn end_layer();
    fn begin_clip(region: PanelBox);
    fn end_clip();
    fn draw_surface(bounds: PanelBox, paint: crate::theme::Color, role: SurfaceRole);
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
    use crate::render::backend::{BracketOrientation, SurfaceRole};
    use crate::render::svg_backend::{SvgBackend, SVG_PLANNED_TEXT_CALLS, SVG_RAW_TEXT_CALLS};
    use crate::theme::Color;

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

    #[test]
    fn canonical_primitive_collapse_and_y_overflow_reject_equally() {
        let points = [
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        assert_paired_rejection(|backend| {
            backend.draw_line(
                &points,
                &LineStyle::solid(Color::rgb(0, 0, 0), 1.0 / 4096.0),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_polygon(
                &points,
                &FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), 1.0 / 4096.0),
            )
        });
        let mut outlined = PointStyle::new((0, 0, 0), 1);
        outlined.outline = Some(crate::render::backend::PointOutline {
            color: Color::rgb(0, 0, 0),
            width_ratio: 1.0 / 4096.0,
        });
        assert_paired_rejection(|backend| backend.draw_scatter(&points, &outlined));
        assert_paired_rejection(|backend| {
            backend.draw_scatter(
                &[Point2D::new(0.0, -9_007_199_254_740_992.0)],
                &PointStyle::plain((0, 0, 0), 1),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[
                    Point2D::new(0.0, -9_007_199_254_740_992.0),
                    Point2D::new(1.0, 0.0),
                ],
                &LineStyle::solid(Color::rgb(0, 0, 0), 1.0),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[points[0], points[1], Point2D::new(1.0 / 4096.0, 0.0)],
                &LineStyle::solid(Color::rgb(0, 0, 0), 1.0),
            )
        });
        assert_paired_success(|backend| {
            backend.draw_line(
                &[points[0], points[1], points[0]],
                &LineStyle::solid(Color::rgb(0, 0, 0), 1.0),
            )
        });
    }

    fn replay_text_style(theme: &crate::theme::ResolvedTheme) -> TextStyle {
        let title = &theme.typography.title;
        TextStyle {
            color: title.paint,
            font_family: title.stack.clone(),
            font_size: title.font_size,
            weight: title.weight,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        }
    }

    fn capture_test_plan(
        theme: &crate::theme::ResolvedTheme,
        capture: impl FnOnce(&mut dyn Backend) -> Result<(), Error>,
    ) -> crate::layout::FrozenRoot<TextReplayPlan> {
        crate::layout::plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), assigned, (), final_stage| {
                let (width, height) = assigned.dimensions()?;
                capture_text_replay_plan(width, height, "test capture", theme, final_stage, capture)
            },
        )
        .unwrap()
    }

    #[test]
    fn frozen_marks_and_gradients_convert_once_before_count_and_replay() {
        use crate::render::coordinates::{
            COORDINATE_CONVERSIONS, SPATIAL_CONVERSIONS, Y_CONVERSIONS,
        };
        let counters = |expected, spatial, y| {
            COORDINATE_CONVERSIONS.with(|count| {
                assert_eq!(count.get(), expected);
                count.set(0);
            });
            SPATIAL_CONVERSIONS.with(|count| {
                assert_eq!(count.get(), spatial);
                count.set(0);
            });
            Y_CONVERSIONS.with(|count| {
                assert_eq!(count.get(), y);
                count.set(0);
            });
        };
        let theme = crate::Theme::Publication.resolve();
        let root = capture_test_plan(&theme, |backend| {
            COORDINATE_CONVERSIONS.with(|count| count.set(0));
            SPATIAL_CONVERSIONS.with(|count| count.set(0));
            Y_CONVERSIONS.with(|count| count.set(0));
            let points = [
                Point2D::new(-0.5 / 1024.0, 0.75),
                Point2D::new(1.25, 2.5),
                Point2D::new(3.0, -2.25),
            ];
            backend.draw_scatter(&points[..2], &PointStyle::new((1, 2, 3), 2))?;
            backend.draw_scatter(
                &points[1..2],
                &PointStyle::new((4, 5, 6), 2).with_shape(crate::PointShape::TriangleUp),
            )?;
            backend.draw_line(&points, &LineStyle::short_dashed(Color::rgb(7, 8, 9), 0.8))?;
            backend.draw_polygon(
                &[points[0], points[1], points[2], points[0]],
                &FillStyle::new(Color::rgba(10, 11, 12, 191), Color::rgb(13, 14, 15), 0.8),
            )?;
            backend.draw_linear_gradient(
                PanelBox::new(-0.25, 4.5, -3.0, 5.75),
                points[0],
                points[1],
                &[
                    GradientStop::new(0.0, (1, 2, 3)),
                    GradientStop::new(0.25, (4, 5, 6)),
                    GradientStop::new(1.0, (7, 8, 9)),
                ],
            )?;
            counters(40, 37, 16);
            Ok(())
        });
        counters(0, 0, 0);
        let plan = root.plan();
        assert_eq!(plan.call_count(), 5);
        validate_text_replay_plan(100, 100, plan, "canonical primitive Null").unwrap();
        counters(0, 0, 0);
        let mut writer = plan.take_svg_backend().unwrap();
        plan.replay(100, 100, &mut writer, "canonical primitive SVG")
            .unwrap();
        let svg = String::from_utf8(writer.into_bytes().unwrap()).unwrap();
        counters(0, 0, 0);
        assert!(svg.contains("points=\"0,99.25 1.25,97.5 3,102.25\""));
        assert!(svg.contains("stroke-width=\"0.7998046875\""));
        assert!(svg.contains("x1=\"0\" y1=\"99.25\" x2=\"1.25\" y2=\"97.5\""));
        assert_eq!(svg.matches("<circle ").count(), 2);
        assert_eq!(svg.matches("<polygon ").count(), 2);
        assert_eq!(svg.matches("<stop ").count(), 3);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one lifecycle checks every text origin, anchor, rotation and both bracket orientations"
    )]
    fn frozen_text_and_brackets_never_relayout_or_convert_during_count_replay_and_outline() {
        use crate::font::canonical::FONT_SHAPE_CALLS;
        use crate::render::coordinates::{COORDINATE_CONVERSIONS, Y_CONVERSIONS};
        use crate::render::primitives::TEXT_PLACEMENTS;
        let reset = || {
            COORDINATE_CONVERSIONS.with(|count| count.set(0));
            Y_CONVERSIONS.with(|count| count.set(0));
            TEXT_PLACEMENTS.with(|count| count.set(0));
            FONT_SHAPE_CALLS.with(|count| count.set(0));
        };
        let unchanged = || {
            COORDINATE_CONVERSIONS.with(|count| assert_eq!(count.get(), 0));
            Y_CONVERSIONS.with(|count| assert_eq!(count.get(), 0));
            TEXT_PLACEMENTS.with(|count| assert_eq!(count.get(), 0));
            FONT_SHAPE_CALLS.with(|count| assert_eq!(count.get(), 0));
        };
        let theme = crate::Theme::Publication.resolve();
        let source = " A & B\n\nC \u{5d0}\u{5d1}  D";
        let root = capture_test_plan(&theme, |backend| {
            for (anchor, alignment) in [
                (TextAnchor::Start, PhysicalTextAlignment::Start),
                (TextAnchor::Middle, PhysicalTextAlignment::Center),
                (TextAnchor::End, PhysicalTextAlignment::End),
            ] {
                let plan = theme.measure_text(
                    source,
                    &theme.typography.title,
                    alignment,
                    "canonical text",
                    false,
                )?;
                for angle in [-0.0, -0.04, 0.06, -89.96] {
                    let position = Point2D::new(-0.5 / 1024.0, 31.25 + 1.5 / 1024.0);
                    backend.draw_planned_text(
                        PlannedTextPlacement::middle(position, anchor, angle),
                        &plan,
                    )?;
                    backend.draw_planned_text(
                        PlannedTextPlacement::baseline(position, anchor, angle),
                        &plan,
                    )?;
                }
            }
            backend.draw_text(
                Point2D::new(20.125, 30.25),
                source,
                &replay_text_style(&theme),
            )?;
            for orientation in [BracketOrientation::Vertical, BracketOrientation::Horizontal] {
                backend.draw_bracket(
                    &SignificanceBracket {
                        x_left: -0.0,
                        x_right: 21.125,
                        y: 31.25,
                        label: "**".into(),
                        orientation,
                    },
                    &BracketStyle {
                        color: Color::rgb(1, 2, 3),
                        label_color: Color::rgb(4, 5, 6),
                        width: 0.8,
                        font_family: theme.typography.title.stack.clone(),
                        font_size: 12,
                        line_height: 14.4,
                        weight: crate::TextWeight::Normal,
                        label_gap: 4.125,
                    },
                )?;
            }
            reset();
            Ok(())
        });
        unchanged();
        let plan = root.plan();
        let PlannedCall::PlannedText(first) = &plan.calls[0] else {
            panic!("canonical text")
        };
        assert_eq!(first.canonical.0.plan.source, source);
        assert_eq!(first.canonical.0.center.x.units(), 0);
        assert_eq!(first.canonical.0.center.y.units(), 70_398);
        assert_eq!(
            first.canonical.0.rotation.value().to_bits(),
            0.0_f64.to_bits()
        );
        validate_text_replay_plan(100, 100, plan, "text Null").unwrap();
        unchanged();
        let mut svg = plan.take_svg_backend().unwrap();
        let addresses = svg.reserved_storage_addresses();
        plan.replay(100, 100, &mut svg, "text SVG").unwrap();
        let output = svg.into_output().unwrap();
        assert_eq!(addresses, output.storage_addresses());
        let (_, _, bytes, outline) = output.into_parts();
        let preserve = std::str::from_utf8(&bytes).unwrap();
        assert!(preserve.contains("A &amp; B"));
        assert!(preserve.contains("<tspan "));
        assert!(preserve.contains("rotate(0.1,"));
        assert!(!preserve.contains("rotate(-0.0,"));
        assert!(preserve.contains("stroke-width=\"0.7998046875\""));
        unchanged();
        let count = outline.count_bytes(100, 100).unwrap();
        let first = outline.derive_counted(100, 100, count).unwrap();
        assert_eq!(first.len(), count);
        assert_eq!(first, outline.derive_counted(100, 100, count).unwrap());
        unchanged();
    }

    #[test]
    fn canonical_text_checks_absolute_origins_and_bottom_up_overflow_before_freeze() {
        let theme = crate::Theme::Publication.resolve();
        let plan = theme
            .measure_text(
                "A",
                &theme.typography.title,
                PhysicalTextAlignment::Start,
                "absolute text",
                false,
            )
            .unwrap();
        let placement =
            PlannedTextPlacement::baseline(Point2D::new(1.25, -2.5), TextAnchor::Start, -0.0);
        crate::render::coordinates::COORDINATE_CONVERSIONS.with(|count| count.set(0));
        crate::render::coordinates::Y_CONVERSIONS.with(|count| count.set(0));
        let canonical = CanonicalPlannedText::new(placement, &plan, 100, None).unwrap();
        crate::render::coordinates::COORDINATE_CONVERSIONS.with(|count| assert_eq!(count.get(), 3));
        crate::render::coordinates::Y_CONVERSIONS.with(|count| assert_eq!(count.get(), 4));
        let line = &canonical.0.lines[0];
        assert_eq!(line.origin.y.units(), 104_960);
        assert_eq!(
            line.origin.x.units(),
            1280 + plan.lines[0].translation_x_quanta
        );
        let glyph = &plan.lines[0].runs[0].glyphs[0];
        assert_eq!(
            line.runs[0].glyphs[0].x.units(),
            line.origin.x.units() + glyph.glyph_origin_x_quanta
        );
        assert_eq!(
            line.runs[0].glyphs[0].y.units(),
            104_960 - glyph.glyph_origin_y_quanta
        );
        let mut overflow = plan.clone();
        overflow.edit_for_test().lines[0].runs[0].glyphs[0].glyph_origin_x_quanta = i64::MAX;
        assert!(CanonicalPlannedText::new(placement, &overflow, 100, None).is_err());
        let mut inverted = placement;
        inverted.position.y = -9_007_199_254_740_992.0;
        assert!(CanonicalPlannedText::new(inverted, &plan, 100, None).is_err());
        for (placement, plan) in [(placement, &overflow), (inverted, &plan)] {
            let mut null = NullBackend::default();
            let mut svg = SvgBackend::in_memory(100, 100);
            null.begin(100, 100).unwrap();
            svg.begin(100, 100).unwrap();
            let null_error = null.draw_planned_text(placement, plan).unwrap_err();
            let svg_error = svg.draw_planned_text(placement, plan).unwrap_err();
            assert_eq!(null_error.to_string(), svg_error.to_string());
        }
        for angle in [f64::NAN, f64::INFINITY, f64::MAX] {
            let mut invalid = placement;
            invalid.rotation = angle;
            assert!(CanonicalPlannedText::new(invalid, &plan, 100, None).is_err());
        }
    }

    #[test]
    fn canonical_text_storage_is_admitted_before_each_allocation() {
        use crate::render::primitives::{CanonicalTextLine, CanonicalTextRun};
        let theme = crate::Theme::Publication.resolve();
        let plan = theme
            .measure_text(
                "A",
                &theme.typography.title,
                PhysicalTextAlignment::Start,
                "text admission",
                false,
            )
            .unwrap();
        assert_eq!(plan.lines.len(), 1);
        assert_eq!(plan.lines[0].runs.len(), 1);
        assert_eq!(plan.lines[0].runs[0].glyphs.len(), 1);
        let mut admitted = 0_u64;
        for (payload, bytes) in [
            (
                "prepared text lines",
                std::mem::size_of::<CanonicalTextLine>(),
            ),
            (
                "prepared text runs",
                std::mem::size_of::<CanonicalTextRun>(),
            ),
            ("prepared text run order", std::mem::size_of::<usize>()),
            (
                "prepared text glyph origins",
                std::mem::size_of::<CanonicalPoint>(),
            ),
            ("prepared text shared geometry", 0),
        ] {
            let result = crate::layout::plan_exact_root(
                100,
                100,
                |stage| Ok(stage.font_resources()),
                |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |resources, _, (), stage| {
                    let used = u64::try_from(resources.usage().unwrap().font_bytes).unwrap();
                    resources
                        .claim_bytes(536_870_912 - used - admitted)
                        .unwrap();
                    CanonicalPlannedText::new(
                        PlannedTextPlacement::middle(
                            Point2D::new(1.0, 2.0),
                            TextAnchor::Start,
                            0.0,
                        ),
                        &plan,
                        100,
                        Some(stage),
                    )
                },
            );
            let error = result
                .err()
                .expect("text storage must refuse at the shared limit");
            assert!(error.to_string().contains(payload), "{payload}: {error}");
            admitted += u64::try_from(bytes).unwrap();
        }
    }

    #[test]
    fn final_line_indices_are_admitted_before_any_point_conversion() {
        use crate::render::coordinates::COORDINATE_CONVERSIONS;
        let result = crate::layout::plan_exact_root(
            100,
            100,
            |stage| Ok(stage.font_resources()),
            |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |resources, _, (), stage| {
                let used = u64::try_from(resources.usage().unwrap().font_bytes).unwrap();
                let point_bytes = u64::try_from(2 * std::mem::size_of::<CanonicalPoint>()).unwrap();
                resources
                    .claim_bytes(536_870_912 - used - point_bytes)
                    .unwrap();
                COORDINATE_CONVERSIONS.with(|count| count.set(0));
                primitives::path(
                    &[Point2D::new(f64::NAN, 0.0); 2],
                    PathKind::Line,
                    100,
                    Some(stage),
                )
            },
        );
        let error = result
            .err()
            .expect("line scratch must refuse before geometry");
        assert!(
            error
                .to_string()
                .contains("prepared line validation indices"),
            "{error}"
        );
        COORDINATE_CONVERSIONS.with(|count| assert_eq!(count.get(), 0));
    }

    #[test]
    fn frozen_rectangles_replay_without_spatial_conversion() {
        use crate::render::coordinates::SPATIAL_CONVERSIONS;
        let theme = crate::Theme::Publication.resolve();
        let root = capture_test_plan(&theme, |backend| {
            SPATIAL_CONVERSIONS.with(|count| count.set(0));
            let bounds = PanelBox::new(-0.25, 50.125, -2.5, 80.25);
            backend.draw_surface(bounds, Color::rgb(1, 2, 3), SurfaceRole::PlotPanel)?;
            backend.begin_clip(bounds)?;
            backend.draw_rounded_rect(
                -0.25,
                -2.5,
                20.125,
                10.25,
                1.125,
                &FillStyle::new(Color::rgba(4, 5, 6, 191), Color::rgb(7, 8, 9), 0.8),
            )?;
            backend.end_clip()?;
            SPATIAL_CONVERSIONS.with(|count| {
                assert_eq!(count.get(), 14);
                count.set(0);
            });
            Ok(())
        });
        SPATIAL_CONVERSIONS.with(|count| assert_eq!(count.get(), 0));
        let plan = root.plan();
        let [PlannedCall::Surface { bounds, .. }, PlannedCall::BeginClip(clip), PlannedCall::RoundedRect(rounded), PlannedCall::EndClip] =
            plan.calls.as_slice()
        else {
            panic!("expected only canonical rectangle calls");
        };
        assert_eq!(bounds, clip);
        assert_eq!(bounds.y.to_string(), "19.75");
        assert_eq!(rounded.stroke_width.to_string(), "0.7998046875");
        validate_text_replay_plan(100, 100, plan, "canonical Null").unwrap();
        let mut svg = plan.take_svg_backend().unwrap();
        plan.replay(100, 100, &mut svg, "canonical SVG").unwrap();
        let output = String::from_utf8(svg.into_bytes().unwrap()).unwrap();
        assert!(output.contains("x=\"-0.25\" y=\"92.25\" width=\"20.125\" height=\"10.25\""));
        assert!(output.contains("stroke-width=\"0.7998046875\""));
        SPATIAL_CONVERSIONS.with(|count| assert_eq!(count.get(), 0));
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
        let line = LineStyle::short_dashed(Color::rgb(1, 2, 3), 0.8);
        let fill = FillStyle::new(Color::rgba(10, 20, 30, 191), Color::rgb(1, 2, 3), 0.5);
        backend.draw_surface(bounds, Color::rgb(255, 255, 255), SurfaceRole::CanvasRoot)?;
        backend.draw_surface(bounds, Color::rgba(1, 2, 3, 128), SurfaceRole::PlotPanel)?;
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
                color: Color::rgb(1, 2, 3),
                label_color: Color::rgb(3, 2, 1),
                width: 0.8,
                font_family: theme.typography.title.stack.clone(),
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
    fn prepared_storage_uses_quantum_primitive_svg_and_matching_png_bytes() {
        use sha2::{Digest as _, Sha256};
        let theme = crate::Theme::Publication.resolve();
        let root = capture_test_plan(&theme, |backend| {
            backend.draw_surface(
                PanelBox::new(0.0, 100.0, 0.0, 100.0),
                Color::rgb(255, 255, 255),
                SurfaceRole::CanvasRoot,
            )?;
            backend.draw_scatter(
                &[Point2D::new(10.0, 20.0)],
                &PointStyle::plain((1, 2, 3), 2).with_opacity(1.0).unwrap(),
            )
        });
        let plan = root.plan();
        validate_text_replay_plan(100, 100, plan, "complete paint fixture Null").unwrap();
        let mut writer = plan.take_svg_backend().unwrap();
        plan.replay(100, 100, &mut writer, "complete paint fixture SVG")
            .unwrap();
        let scene = crate::scene::RenderedScene::new(100, 100, writer.into_output()).unwrap();
        let expected = concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\" viewBox=\"0 0 100 100\" role=\"img\">\n",
            "<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>\n",
            "<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n",
            "<circle cx=\"10\" cy=\"80\" r=\"2\" fill=\"#010203\"/>\n",
            "</svg>\n",
        ).as_bytes();
        assert_eq!(scene.svg_bytes(), expected);
        assert_eq!(scene.svg_with_text_as_paths().unwrap(), expected);
        let options = crate::PngOptions {
            sizing: crate::PngSizing::Source,
            compression: crate::PngCompression::Balanced,
        };
        let target = crate::render::svg_backend::PngTargetPlan::new(
            resvg::tiny_skia::IntSize::from_wh(100, 100).unwrap(),
            &options,
        )
        .unwrap();
        let expected_png =
            crate::render::svg_backend::rasterize_svg_bytes_to_png(expected, target, &options)
                .unwrap();
        let png = scene.png_bytes(&options).unwrap();
        assert_eq!(png, expected_png);
        println!(
            "complete RGBA SVG SHA-256: {:x}",
            Sha256::digest(scene.svg_bytes())
        );
        println!("complete RGBA PNG SHA-256: {:x}", Sha256::digest(&png));
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
                                        color: Color::rgb(0, 0, 0),
                                        label_color: Color::rgb(0, 0, 0),
                                        width: 1.0,
                                        font_family: crate::font::canonical::named_stack(&[
                                            "Unavailable capture family",
                                        ]),
                                        font_size: 12,
                                        line_height: 14.4,
                                        weight: crate::TextWeight::Normal,
                                        label_gap: 4.0,
                                    },
                                )
                            } else {
                                let mut style = replay_text_style(&theme);
                                style.font_family = crate::font::canonical::named_stack(&[
                                    "Unavailable capture family",
                                ]);
                                backend.draw_text(Point2D::new(50.0, 50.0), "raw text", &style)
                            }
                        },
                    )
                },
            );
            let Err(error) = result else {
                panic!("an unprepared raw font stack must be rejected");
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
            "canonical SVG replacements",
            "canonical SVG document",
            "canonical SVG body",
            "canonical SVG root surfaces",
            "canonical SVG panel surfaces",
            "canonical SVG grid",
            "canonical SVG clips",
            "canonical SVG gradients",
            "final Null scopes",
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
                "final Null scopes",
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
    fn native_families_preflight_gate_resources_before_the_only_null_lifecycle() {
        use crate::layout::interior::distributions::{sources, STAGES};
        use crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS;
        for (family, source) in sources()
            .into_iter()
            .chain(crate::layout::interior::estimates::sources())
            .chain(crate::layout::interior::bars::sources())
            .chain(crate::layout::interior::categorical_distributions::sources())
            .chain([crate::plots::categorical_distribution::extraction::supplied()])
            .chain([crate::plots::interaction::extraction::source().erase()])
            .enumerate()
        {
            for payload in [
                "canonical SVG body",
                "canonical SVG document",
                "canonical SVG replacements",
                "canonical SVG scopes",
                "final Null scopes",
            ] {
                NULL_BEGIN_CALLS.with(|calls| calls.set(0));
                GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
                crate::render::svg_backend::REFUSE_STORAGE.with(|refuse| refuse.set(Some(payload)));
                let error = source.preflight().unwrap_err();
                assert!(error.to_string().contains(payload), "{error}");
                NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
                GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| assert_eq!(calls.get(), 0));
            }
            for small in [true, false] {
                STAGES.with(|counts| counts.set([[0; 4]; 4]));
                crate::plots::interval_estimate::STAGES.with(|counts| counts.set([[0; 4]; 3]));
                crate::plots::dumbbell::STAGES.with(|counts| counts.set([0; 4]));
                crate::plots::bar_data::STAGES.with(|counts| counts.set([[0; 4]; 4]));
                crate::plots::categorical_distribution::STAGES
                    .with(|counts| counts.set([[0; 4]; 3]));
                crate::plots::interaction::STAGES.with(|counts| counts.set([0; 3]));
                NULL_BEGIN_CALLS.with(|calls| calls.set(0));
                let plot = if small {
                    source.clone().size(100, 100)
                } else {
                    source.clone()
                };
                let result = plot.preflight();
                assert_eq!(result.is_ok(), !small, "{result:?}");
                NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), usize::from(!small)));
                let counts = match family {
                    0..=3 => STAGES.with(std::cell::Cell::get)[family],
                    4..=6 => crate::plots::interval_estimate::STAGES.with(std::cell::Cell::get)
                        [family - 4],
                    7 => crate::plots::dumbbell::STAGES.with(std::cell::Cell::get),
                    8..=11 => crate::plots::bar_data::STAGES.with(std::cell::Cell::get)[family - 8],
                    12..=15 => crate::plots::categorical_distribution::STAGES
                        .with(std::cell::Cell::get)[(family - 12).min(2)],
                    16 => {
                        let [prepared, measured, drawn] =
                            crate::plots::interaction::STAGES.with(std::cell::Cell::get);
                        [prepared, measured, measured, drawn]
                    }
                    _ => unreachable!(),
                };
                assert_eq!(counts[0], 1);
                assert_eq!(counts[3], usize::from(!small));
            }
        }
    }

    #[test]
    fn raw_text_capture_is_immutable_and_svg_replay_uses_only_planned_calls() {
        let theme = crate::Theme::Publication.resolve();
        let style = replay_text_style(&theme);
        let position = Point2D::new(40.5, 61.5);
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let source = String::from("captured text");
        let mut captures = 0;
        let plan = capture_test_plan(&theme, |backend| {
            captures += 1;
            let result = backend.draw_text(position, &source, &style);
            drop(source); // The producer is genuinely FnOnce; replay owns its source.
            result
        });
        let plan = plan.plan();
        assert_eq!(captures, 1);
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        assert!(shapes > 0);
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        validate_text_replay_plan(100, 100, plan, "single capture").unwrap();

        assert_eq!(plan.call_count(), 1);
        assert_eq!(plan.calls.capacity(), 1);
        let PlannedCall::PlannedText(call) = &plan.calls[0] else {
            panic!("captured raw text");
        };
        assert_eq!(call.placement.position, position);
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
            crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), shapes));
        }
    }

    #[test]
    fn capture_call_limit_uses_shared_root_admission_before_shaping() {
        let theme = crate::Theme::Publication.resolve();
        let mut captures = 0;
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            100,
            100,
            |stage| Ok(stage.generation_resources()),
            |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |resources, _, (), final_stage| {
                capture_text_replay_plan(100, 100, "call limit", &theme, final_stage, |backend| {
                    captures += 1;
                    let (used, _) = resources.usage().unwrap();
                    resources.claim_slots(1_000_000 - used).unwrap();
                    let mut invalid = replay_text_style(&theme);
                    invalid.font_size = 0;
                    backend.draw_text(Point2D::new(0.0, 0.0), "not shaped", &invalid)
                })
            },
        );
        let Err(error) = result else {
            panic!("capture must respect prior root resource claims");
        };
        assert!(
            error.to_string().contains("prepared scene call admission"),
            "{error}"
        );
        assert_eq!(captures, 1);
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn capture_append_refusal_precedes_payload_shaping() {
        let theme = crate::Theme::Publication.resolve();
        let mut pass = 0;
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), final_stage| {
                capture_text_replay_plan(
                    100,
                    100,
                    "append refusal test",
                    &theme,
                    final_stage,
                    |backend| {
                        pass += 1;
                        crate::layout::kernel::force_reservation_failure(
                            crate::layout::kernel::ReservationStage::Final,
                        );
                        let mut invalid = replay_text_style(&theme);
                        invalid.font_size = 0;
                        backend.draw_text(Point2D::new(0.0, 0.0), "not shaped", &invalid)
                    },
                )
            },
        );
        let Err(error) = result else {
            panic!("capture must refuse append before shaping");
        };
        assert!(
            error.to_string().contains("prepared scene calls"),
            "{error}"
        );
        assert_eq!(pass, 1);
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn capture_payload_bytes_use_the_shared_root_limit() {
        let theme = crate::Theme::Publication.resolve();
        let mut captures = 0;
        NULL_BEGIN_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            100,
            100,
            |stage| Ok(stage.font_resources()),
            |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |resources, _, (), final_stage| {
                capture_text_replay_plan(100, 100, "byte limit", &theme, final_stage, |backend| {
                    captures += 1;
                    for _ in 0..3 {
                        backend.draw_surface(
                            PanelBox::new(0.0, 10.0, 0.0, 10.0),
                            Color::rgb(0, 0, 0),
                            SurfaceRole::CanvasRoot,
                        )?;
                    }
                    let used = u64::try_from(resources.usage().unwrap().font_bytes).unwrap();
                    resources.claim_bytes(536_870_912 - used).unwrap();
                    backend
                        .draw_scatter(&[Point2D::new(1.0, 2.0)], &PointStyle::plain((0, 0, 0), 1))
                })
            },
        );
        let Err(error) = result else {
            panic!("capture must admit payload bytes before copying");
        };
        assert!(
            error.to_string().contains("prepared scatter points"),
            "{error}"
        );
        assert!(error.to_string().contains("system-font bytes"), "{error}");
        assert_eq!(captures, 1);
        NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn retained_capture_payload_refusal_precedes_null() {
        let theme = crate::Theme::Publication.resolve();
        for payload in [
            "prepared layer name",
            "prepared scatter points",
            "prepared scatter triangles",
            "prepared line points",
            "prepared polygon points",
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
                            // Leave one admitted command slot available, so refusal
                            // exercises the payload allocation rather than call growth.
                            for _ in 0..3 {
                                backend.draw_surface(
                                    PanelBox::new(0.0, 10.0, 0.0, 10.0),
                                    Color::rgb(0, 0, 0),
                                    SurfaceRole::CanvasRoot,
                                )?;
                            }
                            crate::layout::kernel::force_reservation_failure(
                                crate::layout::kernel::ReservationStage::Final,
                            );
                            crate::render::coordinates::COORDINATE_CONVERSIONS
                                .with(|count| count.set(0));
                            match payload {
                                "prepared layer name" => {
                                    backend.begin_layer("retained name")?;
                                    backend.end_layer()
                                }
                                "prepared scatter points" => backend.draw_scatter(
                                    &[Point2D::new(1.0, 2.0)],
                                    &PointStyle::plain((0, 0, 0), 1),
                                ),
                                "prepared scatter triangles" => backend.draw_scatter(
                                    &[Point2D::new(1.0, 2.0)],
                                    &PointStyle::plain((0, 0, 0), 1)
                                        .with_shape(crate::PointShape::TriangleUp),
                                ),
                                "prepared line points" => backend.draw_line(
                                    &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)],
                                    &LineStyle::solid(Color::rgb(0, 0, 0), 1.0),
                                ),
                                "prepared polygon points" => backend.draw_polygon(
                                    &[
                                        Point2D::new(0.0, 0.0),
                                        Point2D::new(1.0, 1.0),
                                        Point2D::new(2.0, 0.0),
                                    ],
                                    &FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), 1.0),
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
            assert_eq!(pass, 1);
            NULL_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            crate::render::coordinates::COORDINATE_CONVERSIONS
                .with(|count| assert_eq!(count.get(), 0));
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
            color: Color::rgb(0, 0, 0),
            label_color: deferred.plan.style.paint,
            width: 1.0,
            font_family: deferred.plan.style.stack.clone(),
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
    fn direct_planned_bracket_null_and_svg_reject_absolute_and_y_overflow() {
        let theme = crate::Theme::Publication.resolve();
        let resolved = theme
            .measure_text(
                "A",
                &theme.typography.significance_label,
                PhysicalTextAlignment::Center,
                "bracket overflow",
                true,
            )
            .unwrap();
        let mut deferred = DeferredAnnotationTextPlan::from_resolved(resolved);
        let mut bracket = SignificanceBracket {
            x_left: 10.0,
            x_right: 20.0,
            y: -9_007_199_254_740_992.0,
            label: "A".into(),
            orientation: BracketOrientation::Vertical,
        };
        let style = BracketStyle {
            color: Color::rgb(0, 0, 0),
            label_color: deferred.plan.style.paint,
            width: 0.8,
            font_family: deferred.plan.style.stack.clone(),
            font_size: deferred.plan.style.font_size,
            line_height: deferred.plan.line_height(),
            weight: deferred.plan.style.weight,
            label_gap: 4.0,
        };
        for absolute in [false, true] {
            if absolute {
                bracket.y = 30.0;
                deferred.plan.edit_for_test().lines[0].runs[0].glyphs[0].glyph_origin_x_quanta =
                    i64::MAX;
            }
            let mut null = NullBackend::default();
            let mut svg = SvgBackend::in_memory(100, 100);
            null.begin(100, 100).unwrap();
            svg.begin(100, 100).unwrap();
            let call = || PlannedSignificanceCall {
                bracket: &bracket,
                style: &style,
                label: &deferred,
            };
            let null_error = null.draw_planned_bracket(call()).unwrap_err();
            let svg_error = svg.draw_planned_bracket(call()).unwrap_err();
            assert_eq!(null_error.to_string(), svg_error.to_string());
        }
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
            color: Color::rgb(0, 0, 0),
            label_color: Color::rgb(1, 2, 3),
            width: 1.0,
            font_family: theme.typography.title.stack.clone(),
            font_size: 12,
            line_height: 14.4,
            weight: crate::TextWeight::Semibold,
            label_gap: 4.0,
        };
        let plan = capture_test_plan(&theme, |backend| backend.draw_bracket(&bracket, &style));
        let plan = plan.plan();

        let [PlannedCall::PlannedBracket(captured)] = plan.calls.as_slice() else {
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
        let invalid_line = LineStyle::solid(Color::rgb(0, 0, 0), f32::NAN);
        let invalid_fill = FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), f32::NAN);

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
        let invalid_fill = FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), f32::NAN);

        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)],
                &LineStyle::solid(Color::rgb(0, 0, 0), 1.0),
            )
        });
        assert_paired_rejection(|backend| {
            backend.draw_line(
                &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)],
                &LineStyle::solid(Color::rgb(0, 0, 0), f32::NAN),
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
                Color::rgba(1, 2, 3, 0),
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
                backend.draw_surface(bounds, Color::rgb(1, 2, 3), SurfaceRole::CanvasRoot)
            });
        }
    }

    #[test]
    fn null_and_svg_share_rounded_rect_geometry_style_and_order() {
        let valid_fill = FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), 1.0);
        let invalid_fill = FillStyle::new(Color::rgb(0, 0, 0), Color::rgb(0, 0, 0), f32::NAN);

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
            color: Color::rgb(0, 0, 0),
            width: 1.0,
            label_color: Color::rgb(0, 0, 0),
            font_family: crate::font::canonical::FontStack::new([
                crate::font::canonical::FontFamily::generic(
                    crate::font::canonical::GenericFontFamily::SansSerif,
                ),
            ])
            .unwrap(),
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
        let style = LineStyle::solid(Color::rgb(0, 0, 0), 1.0);

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
