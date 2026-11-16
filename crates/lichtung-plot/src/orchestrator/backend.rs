//! Capturing and validating backend implementations used by render orchestration.

use crate::Error;
use std::cell::RefCell;

use crate::font::canonical::SealedTextVerifier;
use crate::layout::{PixelRect, ResolvedTextPlan};
use crate::render::backend::{
    self, Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle,
    PlannedSignificanceCall, PlannedTextPlacement, Point2D, PointStyle, SignificanceBracket,
    SurfaceRole, TextStyle,
};
use crate::render::coordinates::CanonicalPoint;
use crate::render::coordinates::{CanonicalBox, CanonicalRoundedBox};
use crate::render::primitives::{
    self, CanonicalFillStyle, CanonicalGradientFill, CanonicalLineStyle, CanonicalPlannedBracket,
    CanonicalPlannedText, CanonicalScatter, PathKind,
};

#[cfg(test)]
thread_local! {
    static NULL_BEGIN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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
    pub(crate) fn retained_text(
        &self,
    ) -> Vec<(CanonicalPlannedText, &crate::layout::text::FrozenText)> {
        self.calls
            .iter()
            .filter_map(|call| match call {
                PlannedCall::PlannedText(text) => {
                    Some((text.canonical.clone(), &text.canonical.0.text))
                }
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

#[derive(Debug)]
struct PlannedTextCall {
    canonical: CanonicalPlannedText,
}

#[derive(Debug)]
struct PlannedBracketCall {
    canonical: CanonicalPlannedBracket,
}

pub(super) struct TextPlanningBackend<'v, 'env, 'a, 'b> {
    height: u32,
    verifier: &'v SealedTextVerifier<'env>,
    calls: Vec<PlannedCall>,
    final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
}

impl<'v, 'env, 'a, 'b> TextPlanningBackend<'v, 'env, 'a, 'b> {
    fn new(
        verifier: &'v SealedTextVerifier<'env>,
        final_stage: &'a mut crate::layout::kernel::FinalStage<'b>,
        height: u32,
    ) -> Self {
        Self {
            height,
            verifier,
            calls: Vec::new(),
            final_stage,
        }
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
            self.verifier.retain_fonts(),
            crate::render::svg_backend::SvgStorageRequirements { scopes },
            self.final_stage,
            |serializer| {
                for call in &calls {
                    call.emit_svg(serializer)?;
                    #[cfg(test)]
                    serializer.overflow_count_after_text_for_test();
                }
                Ok(())
            },
        )?;
        let mut null = NullBackend {
            recorder: ValidationRecorder::default(),
        };
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

    fn reserve_call(&mut self) -> Result<(), Error> {
        // Admit and reserve the command before copying payloads or shaping raw text.

        self.final_stage
            .reserve_capture(&mut self.calls, 1, "prepared scene calls")
    }
}

pub(crate) fn capture_calls(
    size: crate::layout::RootSize,
    verifier: &SealedTextVerifier<'_>,
    final_stage: &mut crate::layout::kernel::FinalStage<'_>,
    draw: impl FnOnce(&mut dyn Backend) -> Result<(), Error>,
) -> Result<TextReplayPlan, Error> {
    #[cfg(test)]
    let _call = super::measurements::StageCall::new(2);
    let (width, height) = size
        .dimensions()
        .map_err(|_| Error::Render("prepared scene dimensions must be positive".into()))?;
    let mut backend = TextPlanningBackend::new(verifier, final_stage, height);
    draw(&mut backend)?;
    backend.into_plan(width, height)
}

impl Backend for TextPlanningBackend<'_, '_, '_, '_> {
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if width == 0 || height == 0 {
            return Err(Error::Render(
                "prepared scene dimensions must be positive".into(),
            ));
        }
        Ok(())
    }

    fn begin_layer(&mut self, name: &str) -> Result<(), Error> {
        self.reserve_call()?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayer(name));
        Ok(())
    }

    fn begin_layer_with_role(&mut self, name: &str, role: LayerRole) -> Result<(), Error> {
        self.reserve_call()?;
        let name = self.copy_string(name, "prepared layer name")?;
        self.calls.push(PlannedCall::BeginLayerWithRole(name, role));
        Ok(())
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.reserve_call()?;
        self.calls.push(PlannedCall::EndLayer);
        Ok(())
    }

    fn begin_clip(&mut self, region: PixelRect) -> Result<(), Error> {
        self.reserve_call()?;
        let region =
            CanonicalBox::from_bottom_up(backend::validate_clip_region(region)?, self.height)?;
        self.calls.push(PlannedCall::BeginClip(region));
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.reserve_call()?;
        self.calls.push(PlannedCall::EndClip);
        Ok(())
    }

    fn draw_surface(
        &mut self,
        bounds: PixelRect,
        paint: crate::theme::Color,
        role: SurfaceRole,
    ) -> Result<(), Error> {
        self.reserve_call()?;
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
        self.reserve_call()?;
        let scatter = CanonicalScatter::new(points, style, self.height, Some(self.final_stage))?;
        self.calls.push(PlannedCall::Scatter(scatter));
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.reserve_call()?;
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
        self.reserve_call()?;
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
        bounds: PixelRect,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.reserve_call()?;
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
        let _ = (position, text, style);
        Err(Error::Render(
            "scene capture requires preplanned text".into(),
        ))
    }

    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.verifier
            .validate_measured(plan)
            .map_err(|error| Error::Render(format!("frozen text capture: {error}")))?;
        self.reserve_call()?;
        let canonical =
            CanonicalPlannedText::new(placement, plan, self.height, Some(self.final_stage))?;
        self.calls
            .push(PlannedCall::PlannedText(PlannedTextCall { canonical }));
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
        self.reserve_call()?;
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
        let _ = (bracket, style);
        Err(Error::Render(
            "scene capture requires a preplanned bracket label".into(),
        ))
    }

    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        let plan = &call.label.plan;
        self.verifier
            .validate_measured(plan)
            .map_err(|error| Error::Render(format!("frozen bracket capture: {error}")))?;
        self.reserve_call()?;
        let canonical = CanonicalPlannedBracket::new(call, self.height, Some(self.final_stage))?;
        self.calls
            .push(PlannedCall::PlannedBracket(PlannedBracketCall {
                canonical,
            }));
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
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
    let _call = super::measurements::StageCall::new(3);
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

    fn begin_clip(&mut self, region: PixelRect) -> Result<(), Error> {
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
        bounds: PixelRect,
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
        bounds: PixelRect,
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
    fn begin_clip(region: PixelRect);
    fn end_clip();
    fn draw_surface(bounds: PixelRect, paint: crate::theme::Color, role: SurfaceRole);
    fn draw_scatter(points: &[Point2D], style: &PointStyle);
    fn draw_line(points: &[Point2D], style: &LineStyle);
    fn draw_polygon(points: &[Point2D], style: &FillStyle);
    fn draw_linear_gradient(bounds: PixelRect, start: Point2D, end: Point2D, stops: &[GradientStop]);
    fn draw_text(position: Point2D, text: &str, style: &TextStyle);
    fn draw_planned_text(placement: PlannedTextPlacement, plan: &ResolvedTextPlan);
    fn draw_rounded_rect(x: f64, y: f64, width: f64, height: f64, radius: f64, style: &FillStyle);
    fn draw_bracket(bracket: &SignificanceBracket, style: &BracketStyle);
    fn draw_planned_bracket(call: PlannedSignificanceCall<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_preflights_reserve_scene_storage_before_the_only_null_lifecycle() {
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
            // The storage-specific refusal/release matrix is checked above.
            // Public preflight needs one reservation-order witness per root.
            let payload = "canonical SVG body";
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
}
