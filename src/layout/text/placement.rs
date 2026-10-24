//! One measured block/reference transform shared by text owners and final geometry.

use super::ResolvedTextPlan;
use crate::layout::PanelBox;
use crate::render::backend::{
    PlannedTextOrigin, PlannedTextPlacement, Point2D, TextAnchor as BackendAnchor,
};
use crate::text::layout::VerticalAnchor;
use crate::{Error, TextAnchor, TextOffsetFrame};

#[cfg(test)]
mod tests;

pub(crate) fn resolve(
    default: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
    region: Option<PanelBox>,
) -> Result<PlannedTextPlacement, Error> {
    let Some(request) = &plan.request else {
        return Ok(default);
    };
    let request = request.instructions();
    let aligned = matches!(
        (default.anchor, plan.style.alignment),
        (BackendAnchor::Start, super::PhysicalTextAlignment::Start)
            | (BackendAnchor::Middle, super::PhysicalTextAlignment::Center)
            | (BackendAnchor::End, super::PhysicalTextAlignment::End)
    );
    if request.anchor.is_none()
        && request.offset.is_none()
        && request.orientation.is_none()
        && request.rotation_mode.is_none()
        && aligned
    {
        return Ok(default);
    }
    let anchor = request.anchor.unwrap_or_else(|| from_default(default));
    let reference = if let (Some(_), Some(region)) = (request.anchor, region) {
        let vertical = match anchor.vertical {
            VerticalAnchor::Fraction(value) => value.get(),
            _ => 0.0,
        };
        Point2D::new(
            region.x0 + anchor.horizontal.get() * region.width(),
            region.y1 - vertical * region.height(),
        )
    } else {
        default.position
    };
    at(
        plan,
        reference,
        anchor,
        default.rotation,
        default.reverse_lines && request.orientation.is_none(),
        request.offset,
    )
}

fn from_default(placement: PlannedTextPlacement) -> TextAnchor {
    let horizontal = match placement.anchor {
        BackendAnchor::Start => 0.0,
        BackendAnchor::Middle => 0.5,
        BackendAnchor::End => 1.0,
    };
    match placement.origin {
        PlannedTextOrigin::Middle => TextAnchor::fractions(horizontal, 0.5).expect("finite preset"),
        PlannedTextOrigin::Baseline => {
            TextAnchor::first_baseline(horizontal).expect("finite preset")
        }
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "measured block coordinates are bounded canonical quanta"
)]
pub(crate) fn at(
    plan: &ResolvedTextPlan,
    reference: Point2D,
    anchor: TextAnchor,
    rotation: f64,
    reverse_lines: bool,
    offset: Option<crate::TextOffset>,
) -> Result<PlannedTextPlacement, Error> {
    let instructions = plan.request.as_ref().map(crate::TextLayout::instructions);
    let rotation = plan.selected_rotation.map_or_else(
        || {
            instructions
                .and_then(|request| request.orientation.as_ref())
                .map_or(Ok(rotation), |request| match &request.0 {
                    crate::text::orientation::Orientation::Fixed(angle) => Ok(angle.0.get()),
                    crate::text::orientation::Orientation::Data(..) => Err(Error::Render(
                        "data text orientation requires its actual projection before measurement"
                            .into(),
                    )),
                    crate::text::orientation::Orientation::Automatic(..) => Err(Error::Data(
                        "automatic orientation requires an actual tick group".into(),
                    )),
                })
        },
        |angle| Ok(angle.0.get()),
    )?;
    let width = plan.width();
    let height = plan.height();
    let baseline = |value: Option<i64>| value.map_or(0.0, |value| value as f64 / 1024.0);
    let anchor_y = match anchor.vertical {
        VerticalAnchor::Fraction(value) => height * value.get(),
        VerticalAnchor::FirstBaseline => baseline(plan.first_baseline_quanta),
        VerticalAnchor::CenterBaseline => f64::midpoint(
            baseline(plan.first_baseline_quanta),
            baseline(plan.last_baseline_quanta),
        ),
    };
    let (alignment, horizontal) = match plan.style.alignment {
        super::PhysicalTextAlignment::Start => (BackendAnchor::Start, 0.0),
        super::PhysicalTextAlignment::Center => (BackendAnchor::Middle, width / 2.0),
        super::PhysicalTextAlignment::End => (BackendAnchor::End, width),
    };
    let mut local = (
        horizontal - width * anchor.horizontal.get(),
        height / 2.0 - anchor_y,
    );
    let mut screen = (0.0, 0.0);
    if let Some(offset) = offset {
        match offset.frame {
            TextOffsetFrame::Screen => screen = (offset.dx.get(), offset.dy.get()),
            TextOffsetFrame::TextLocal => {
                local.0 += offset.dx.get();
                local.1 += offset.dy.get();
            }
        }
    }
    let (sine, cosine) = match rotation {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        -90.0 => (-1.0, 0.0),
        _ => rotation.to_radians().sin_cos(),
    };
    let position = Point2D::new(
        reference.x + screen.0 + local.0 * cosine - local.1 * sine,
        reference.y - screen.1 - local.0 * sine - local.1 * cosine,
    );
    let canonical = crate::render::coordinates::canonical_text(
        position,
        rotation,
        crate::render::coordinates::CoordinatePrecision::CssQuantum,
    )?;
    let mut placement =
        PlannedTextPlacement::middle(canonical.position.value(), alignment, rotation);
    placement.reverse_lines = reverse_lines;
    if instructions.is_some_and(|request| {
        request.rotation_mode == Some(crate::TextRotationMode::AlignAfterRotation)
    }) {
        let bounds = logical_bounds(placement, plan)?;
        let x = bounds.x0 + anchor.horizontal.get() * bounds.width();
        let y = match anchor.vertical {
            VerticalAnchor::Fraction(fraction) => bounds.y1 - fraction.get() * bounds.height(),
            _ => reference.y - screen.1,
        };
        placement.position.x += reference.x + screen.0 - x;
        placement.position.y += reference.y - screen.1 - y;
    }
    Ok(placement)
}

pub(crate) fn logical_bounds(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> Result<PanelBox, Error> {
    rectangle_bounds(placement, plan, plan.width_quanta, plan.height_quanta)
}

/// The actual logical/paint footprint, after text-local clipping or hiding.
pub(crate) fn footprint(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> Result<PanelBox, Error> {
    let mut bounds = logical_bounds(placement, plan)?;
    if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
        plan,
        placement.position,
        placement.anchor,
        placement.rotation,
        placement.origin,
        placement.reverse_lines,
    ) {
        bounds = PanelBox::new(
            bounds.x0.min(ink.left),
            bounds.x1.max(ink.right),
            bounds.y0.min(ink.bottom),
            bounds.y1.max(ink.top),
        );
    }
    if let Some(clip) = paint_clip(placement, plan)? {
        let x0 = bounds.x0.max(clip.x0);
        let y0 = bounds.y0.max(clip.y0);
        bounds = PanelBox::new(
            x0,
            bounds.x1.min(clip.x1).max(x0),
            y0,
            bounds.y1.min(clip.y1).max(y0),
        );
    }
    Ok(bounds)
}

/// Apply only an actual spatial fit miss. Font/projection/arithmetic errors remain fatal.
pub(crate) fn contain(
    plan: ResolvedTextPlan,
    placement: PlannedTextPlacement,
    region: PanelBox,
    scope: &crate::layout::fit::FitScope,
) -> Result<ResolvedTextPlan, crate::layout::fit::PlanError> {
    let bounds = footprint(placement, &plan)?;
    if plan.lines.is_empty()
        || (bounds.x0 >= region.x0
            && bounds.x1 <= region.x1
            && bounds.y0 >= region.y0
            && bounds.y1 <= region.y1)
    {
        return Ok(plan);
    }
    let failure = plan
        .request
        .as_ref()
        .and_then(|request| request.instructions().overflow.as_ref())
        .map_or(crate::TextFailure::Error, |overflow| overflow.failure);
    match failure {
        crate::TextFailure::Error => Err(scope
            .failure(
                crate::layout::fit::FitKind::Text,
                format!("transformed text exceeds its assigned region: footprint {bounds:?}, region {region:?}"),
            )
            .into()),
        crate::TextFailure::Hide => Ok(plan.hide()?),
        crate::TextFailure::Clip => Ok(plan.clip_region(region)),
        crate::TextFailure::Visible => Ok(plan),
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "logical extents use the same bounded quanta as frozen glyph origins"
)]
fn rectangle_bounds(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
    width: i64,
    height: i64,
) -> Result<PanelBox, Error> {
    let half = |value| {
        crate::render::backend::nearest_even_text_half(value)
            .ok_or_else(|| Error::Render("negative text extent".into()))
    };
    let left = match placement.anchor {
        BackendAnchor::Start => 0,
        BackendAnchor::Middle => -half(plan.width_quanta)?,
        BackendAnchor::End => -plan.width_quanta,
    };
    let top = match placement.origin {
        PlannedTextOrigin::Middle => half(plan.height_quanta)?,
        PlannedTextOrigin::Baseline => plan.first_baseline_quanta.unwrap_or(0),
    };
    let (sine, cosine) = match placement.rotation {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        -90.0 => (-1.0, 0.0),
        _ => placement.rotation.to_radians().sin_cos(),
    };
    let corners = [
        (left, top),
        (left + width, top),
        (left, top - height),
        (left + width, top - height),
    ]
    .map(|(x, y)| {
        let x = x as f64 / 1024.0;
        let y = y as f64 / 1024.0;
        Point2D::new(
            placement.position.x + x * cosine + y * sine,
            placement.position.y - x * sine + y * cosine,
        )
    });
    let lower = |axis: fn(&Point2D) -> f64| corners.iter().map(axis).fold(f64::INFINITY, f64::min);
    let upper =
        |axis: fn(&Point2D) -> f64| corners.iter().map(axis).fold(f64::NEG_INFINITY, f64::max);
    Ok(PanelBox::new(
        lower(|p| p.x),
        upper(|p| p.x),
        lower(|p| p.y),
        upper(|p| p.y),
    ))
}

pub(crate) fn paint_clip(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> Result<Option<PanelBox>, Error> {
    let local = plan
        .clip_size
        .map(|(width, height)| {
            rectangle_bounds(
                placement,
                plan,
                width.min(plan.width_quanta),
                height.min(plan.height_quanta),
            )
        })
        .transpose()?;
    let region = plan.paint_region.map(|values| {
        let [x0, x1, y0, y1] = values.map(f64::from_bits);
        PanelBox::new(x0, x1, y0, y1)
    });
    Ok(match (local, region) {
        (Some(a), Some(b)) => {
            let x0 = a.x0.max(b.x0);
            let y0 = a.y0.max(b.y0);
            Some(PanelBox::new(
                x0,
                a.x1.min(b.x1).max(x0),
                y0,
                a.y1.min(b.y1).max(y0),
            ))
        }
        (local, region) => local.or(region),
    })
}
