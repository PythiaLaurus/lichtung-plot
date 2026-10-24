//! Admitted final primitive payloads. All positions are retained top-down quanta.

use super::backend::{
    self, FillStyle, GradientStop, LineStyle, Point2D, PointStyle, StrokePattern,
};
use super::backend::{
    PlannedSignificanceCall, PlannedTextOrigin, PlannedTextPlacement, TextAnchor,
};
use super::coordinates::{
    self, CanonicalBox, CanonicalCoordinate, CanonicalGradientStop, CanonicalPoint,
    CoordinatePrecision,
};
use crate::layout::ResolvedTextPlan;
use crate::layout::{kernel::FinalStage, PanelBox};
use crate::theme::Color;
use crate::{Error, PointShape};
use std::sync::Arc;

#[cfg(test)]
thread_local! {
    pub(crate) static TEXT_PLACEMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug)]
pub(crate) struct CanonicalTextRun {
    pub(crate) origin: Option<CanonicalPoint>,
    pub(crate) glyphs: Vec<CanonicalPoint>,
}

#[derive(Debug)]
pub(crate) struct CanonicalTextLine {
    pub(crate) origin: CanonicalPoint,
    pub(crate) runs: Vec<CanonicalTextRun>,
    pub(crate) order: Vec<usize>,
    pub(crate) isolate_runs: bool,
}

#[derive(Debug)]
pub(crate) struct CanonicalTextPayload {
    pub(crate) plan: ResolvedTextPlan,
    pub(crate) center: CanonicalPoint,
    pub(crate) rotation: coordinates::CanonicalAngle,
    pub(crate) rotation_sin_cos: (f64, f64),
    pub(crate) lines: Vec<CanonicalTextLine>,
    pub(crate) clip: Option<CanonicalBox>,
    pub(crate) suppressed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CanonicalPlannedText(pub(crate) Arc<CanonicalTextPayload>);

impl CanonicalPlannedText {
    pub(crate) fn new(
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
        height: u32,
        stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        let canonical = backend::validate_planned_text(placement, plan)?;
        Self::from_validated(canonical, placement, plan, height, stage)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one admitted pass freezes linked line, run, and glyph placement"
    )]
    pub(crate) fn from_validated(
        canonical: coordinates::CanonicalText,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
        height: u32,
        mut stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        TEXT_PLACEMENTS.with(|count| count.set(count.get() + 1));
        let half = |value| {
            backend::nearest_even_text_half(value)
                .ok_or_else(|| Error::Render("planned text extent cannot be negative".into()))
        };
        let overflow = || Error::Render("planned text absolute placement overflowed".into());
        let anchor = match placement.anchor {
            TextAnchor::Start => 0,
            TextAnchor::Middle => half(plan.width_quanta)?,
            TextAnchor::End => plan.width_quanta,
        };
        let left = canonical
            .position
            .x
            .units()
            .checked_sub(anchor)
            .ok_or_else(overflow)?;
        let reference = match placement.origin {
            PlannedTextOrigin::Middle => half(plan.height_quanta)?,
            PlannedTextOrigin::Baseline => plan.first_baseline_quanta.unwrap_or(0),
        };
        let mut lines = storage(
            plan.lines.len(),
            stage.as_deref_mut(),
            "prepared text lines",
        )?;
        for line in &plan.lines {
            let x = left
                .checked_add(line.translation_x_quanta)
                .ok_or_else(overflow)?;
            let baseline = canonical
                .position
                .y
                .units()
                .checked_add(reference)
                .and_then(|y| {
                    y.checked_sub(placement.line_y(plan, line.translation_y_quanta).ok()?)
                })
                .ok_or_else(overflow)?;
            let origin = CanonicalPoint {
                x: CanonicalCoordinate::from_quanta(x),
                y: CanonicalCoordinate::from_quanta(baseline),
            }
            .inverted(height)?;
            let mut runs = storage(line.runs.len(), stage.as_deref_mut(), "prepared text runs")?;
            let mut order = storage(
                line.runs.len(),
                stage.as_deref_mut(),
                "prepared text run order",
            )?;
            order.extend(0..line.runs.len());
            order.sort_unstable_by_key(|index| line.runs[*index].source_span.start_byte);
            for run in &line.runs {
                let run_origin = run
                    .glyphs
                    .first()
                    .map(|first| {
                        CanonicalPoint {
                            x: CanonicalCoordinate::from_quanta(
                                first
                                    .glyph_origin_x_quanta
                                    .checked_sub(first.x_offset_quanta)
                                    .and_then(|v| x.checked_add(v))
                                    .ok_or_else(overflow)?,
                            ),
                            y: CanonicalCoordinate::from_quanta(
                                first
                                    .glyph_origin_y_quanta
                                    .checked_sub(first.y_offset_quanta)
                                    .and_then(|v| baseline.checked_add(v))
                                    .ok_or_else(overflow)?,
                            ),
                        }
                        .inverted(height)
                    })
                    .transpose()?;
                let mut glyphs = storage(
                    run.glyphs.len(),
                    stage.as_deref_mut(),
                    "prepared text glyph origins",
                )?;
                for glyph in &run.glyphs {
                    glyphs.push(
                        CanonicalPoint {
                            x: CanonicalCoordinate::from_quanta(
                                x.checked_add(glyph.glyph_origin_x_quanta)
                                    .ok_or_else(overflow)?,
                            ),
                            y: CanonicalCoordinate::from_quanta(
                                baseline
                                    .checked_add(glyph.glyph_origin_y_quanta)
                                    .ok_or_else(overflow)?,
                            ),
                        }
                        .inverted(height)?,
                    );
                }
                runs.push(CanonicalTextRun {
                    origin: run_origin,
                    glyphs,
                });
            }
            lines.push(CanonicalTextLine {
                origin,
                runs,
                order,
                isolate_runs: line.selected_soft_hyphen.is_some()
                    || line.runs.len() > 1
                    || line
                        .runs
                        .iter()
                        .any(|run| run.bidi_level > 1 || run.direction != line.base_direction),
            });
        }

        let clip = crate::layout::text::placement::paint_clip(placement, plan)?;
        let suppressed = clip.is_some_and(|region| region.width() <= 0.0 || region.height() <= 0.0);
        let clip = clip
            .filter(|_| !suppressed)
            .map(|region| {
                coordinates::CanonicalBox::from_bottom_up(
                    backend::validate_clip_region(region)?,
                    height,
                )
            })
            .transpose()?;
        Ok(Self(Arc::new(CanonicalTextPayload {
            plan: plan.clone(),
            center: canonical.position.inverted(height)?,
            rotation: canonical.rotation,
            rotation_sin_cos: canonical.rotation.value().to_radians().sin_cos(),
            lines,
            clip,
            suppressed,
        })))
    }
}

#[derive(Debug)]
pub(crate) struct CanonicalPlannedBracket {
    pub(crate) points: [CanonicalPoint; 4],
    pub(crate) orientation: backend::BracketOrientation,
    pub(crate) width: CanonicalCoordinate,
    pub(crate) color: Color,
    pub(crate) text: CanonicalPlannedText,
}

impl CanonicalPlannedBracket {
    pub(crate) fn new(
        call: PlannedSignificanceCall<'_>,
        height: u32,
        stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        let canonical = backend::validate_bracket(call.bracket, call.style)?;
        Self::from_geometry(canonical, call, height, stage)
    }

    pub(crate) fn from_geometry(
        canonical: coordinates::CanonicalBracket,
        call: PlannedSignificanceCall<'_>,
        height: u32,
        stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        backend::validate_bracket_label(canonical, call)?;
        let width = CanonicalCoordinate::nonnegative_length(
            f64::from(call.style.width),
            "bracket stroke width",
        )?;
        let points = [
            canonical.line_start.inverted(height)?,
            canonical.line_end.inverted(height)?,
            canonical.first_tick_end.inverted(height)?,
            canonical.second_tick_end.inverted(height)?,
        ];
        let text = CanonicalPlannedText::from_validated(
            coordinates::CanonicalText {
                position: canonical.label_position,
                rotation: coordinates::CanonicalAngle::ZERO,
            },
            backend::planned_bracket_placement(canonical, &call.label.plan)?,
            &call.label.plan,
            height,
            stage,
        )?;
        Ok(Self {
            points,
            orientation: canonical.orientation,
            width,
            color: call.style.color,
            text,
        })
    }
}

fn storage<T>(
    count: usize,
    stage: Option<&mut FinalStage<'_>>,
    payload: &str,
) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    if let Some(stage) = stage {
        stage.reserve_capture(&mut values, count, payload)?;
    } else {
        values
            .try_reserve_exact(count)
            .map_err(|error| Error::Render(format!("{payload} reservation: {error}")))?;
    }
    Ok(values)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CanonicalLineStyle {
    pub(crate) color: Color,
    pub(crate) width: CanonicalCoordinate,
    pub(crate) pattern: StrokePattern,
}

impl CanonicalLineStyle {
    pub(crate) fn new(style: &LineStyle) -> Result<Self, Error> {
        backend::validate_line_style(style)?;
        Ok(Self {
            color: style.color,
            width: CanonicalCoordinate::nonnegative_length(
                f64::from(style.width),
                "line stroke width",
            )?,
            pattern: style.pattern,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CanonicalFillStyle {
    pub(crate) color: Color,
    pub(crate) stroke: Color,
    pub(crate) stroke_width: CanonicalCoordinate,
}

impl CanonicalFillStyle {
    pub(crate) fn new(style: &FillStyle) -> Result<Self, Error> {
        backend::validate_fill_style(style)?;
        Ok(Self {
            color: style.color,
            stroke: style.stroke,
            stroke_width: CanonicalCoordinate::nonnegative_length(
                f64::from(style.stroke_width),
                "polygon stroke width",
            )?,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PathKind {
    Line,
    Polygon,
}

pub(crate) fn path(
    raw: &[Point2D],
    kind: PathKind,
    height: u32,
    mut stage: Option<&mut FinalStage<'_>>,
) -> Result<Vec<CanonicalPoint>, Error> {
    let (name, payload) = match kind {
        PathKind::Line => ("line", "prepared line points"),
        PathKind::Polygon => ("polygon", "prepared polygon points"),
    };
    let mut points = storage(raw.len(), stage.as_deref_mut(), payload)?;
    let mut indices = match kind {
        PathKind::Line => storage(raw.len(), stage, "prepared line validation indices")?,
        PathKind::Polygon => Vec::new(),
    };
    for point in coordinates::canonical_points_iter(raw, name) {
        points.push(point?);
    }
    match kind {
        PathKind::Line => {
            indices.extend(0..raw.len());
            coordinates::validate_line_geometry(raw, &points, &mut indices)?;
        }
        PathKind::Polygon => coordinates::validate_polygon_geometry(raw, &points)?,
    }
    for point in &mut points {
        *point = point.inverted(height)?;
    }
    Ok(points)
}

#[derive(Debug)]
pub(crate) enum CanonicalMarkers {
    Circles {
        points: Vec<CanonicalPoint>,
        radius: CanonicalCoordinate,
    },
    Triangles(Vec<[CanonicalPoint; 3]>),
}

#[derive(Debug)]
pub(crate) struct CanonicalScatter {
    pub(crate) markers: CanonicalMarkers,
    pub(crate) color: Color,
    pub(crate) outline: Option<(Color, CanonicalCoordinate)>,
}

impl CanonicalScatter {
    pub(crate) fn new(
        raw: &[Point2D],
        style: &PointStyle,
        height: u32,

        stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        let markers = match style.shape {
            PointShape::Circle => {
                let mut points = storage(raw.len(), stage, "prepared scatter points")?;
                for point in coordinates::canonical_points_iter(raw, "scatter") {
                    points.push(point?.inverted(height)?);
                }
                CanonicalMarkers::Circles {
                    points,
                    radius: CanonicalCoordinate::from_f64(
                        f64::from(style.radius),
                        CoordinatePrecision::CssQuantum,
                        "scatter radius",
                    )?,
                }
            }
            PointShape::TriangleUp => {
                let mut triangles = storage(raw.len(), stage, "prepared scatter triangles")?;
                for (index, point) in raw.iter().copied().enumerate() {
                    let mut triangle = coordinates::canonical_triangle(point, style.radius, index)?;
                    for vertex in &mut triangle {
                        *vertex = vertex.inverted(height)?;
                    }
                    triangles.push(triangle);
                }
                CanonicalMarkers::Triangles(triangles)
            }
        };
        backend::validate_point_style(style)?;
        let outline = style
            .outline
            .as_ref()
            .map(|outline| Ok::<_, Error>((outline.color, outline.width_for_radius(style.radius)?)))
            .transpose()?;
        Ok(Self {
            markers,
            color: style.color,
            outline,
        })
    }
}

#[derive(Debug)]
pub(crate) struct CanonicalGradientFill {
    pub(crate) bounds: CanonicalBox,
    pub(crate) start: CanonicalPoint,
    pub(crate) end: CanonicalPoint,
    pub(crate) stops: Vec<CanonicalGradientStop>,
}

impl CanonicalGradientFill {
    pub(crate) fn new(
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        raw: &[GradientStop],
        height: u32,
        stage: Option<&mut FinalStage<'_>>,
    ) -> Result<Self, Error> {
        let mut stops = storage(raw.len(), stage, "prepared gradient stops")?;
        let (bounds, start, end) =
            coordinates::visit_canonical_gradient(bounds, start, end, raw, |stop| {
                stops.push(stop);
            })?;
        Ok(Self {
            bounds: CanonicalBox::from_bottom_up(bounds, height)?,
            start: start.inverted(height)?,
            end: end.inverted(height)?,
            stops,
        })
    }
}
