//! Project explicit source bindings, preserving component alpha and geometry.

use crate::decorations::legend::source::{
    GuideExplanation, GuideExplanationMask, Projection, RgbBinding, RgbExplanation, SourceMapping,
    StyleBinding, StyleExplanation,
};
use crate::decorations::legend::ResolvedGuideKey;
use crate::theme::{Color, OpaqueColor};
use crate::{Error, Result};

pub(super) fn project(
    complete: &ResolvedGuideKey,
    source: &SourceMapping,
    projection: Projection,
    neutral: Option<OpaqueColor>,
) -> Result<ResolvedGuideKey> {
    if !matches!(complete.explanation, GuideExplanation::Complete(_)) {
        return Err(Error::Render(
            "guide projection requires a complete source key".into(),
        ));
    }
    if neutral.is_some() != (projection == Projection::GroupStyle) {
        return Err(Error::Render(
            "guide projection has inconsistent neutral-paint inputs".into(),
        ));
    }
    if projection == Projection::Complete {
        return Ok(complete.clone());
    }
    let mut key = complete.clone();
    let paints = &source.paint_bindings;
    let styles = &source.style_bindings;
    let point_fill = paint(
        paints.point_fill.as_ref(),
        key.point.as_mut().map(|point| &mut point.color),
        projection,
        neutral,
    )?;
    let point_outline = paint(
        paints.point_outline.as_ref(),
        key.point
            .as_mut()
            .and_then(|point| point.outline.as_mut())
            .map(|outline| &mut outline.color),
        projection,
        neutral,
    )?;
    let line_stroke = paint(
        paints.line_stroke.as_ref(),
        key.line.as_mut().map(|line| &mut line.color),
        projection,
        neutral,
    )?;
    let fill_paint = paint(
        paints.fill_paint.as_ref(),
        key.fill.as_mut().map(|fill| &mut fill.color),
        projection,
        neutral,
    )?;
    let fill_outline = paint(
        paints.fill_outline.as_ref(),
        key.fill
            .as_mut()
            .filter(|fill| fill.stroke_width > 0.0)
            .map(|fill| &mut fill.stroke),
        projection,
        neutral,
    )?;
    let point_shape = style(styles.point_shape.as_ref(), projection);
    let line_pattern = style(styles.line_pattern.as_ref(), projection);
    if point_shape == Some(StyleExplanation::NeutralizedGroup) {
        key.point
            .as_mut()
            .ok_or_else(|| Error::Render("projected point style has no point".into()))?
            .shape = crate::PointShape::Circle;
    }
    if line_pattern == Some(StyleExplanation::NeutralizedGroup) {
        key.line
            .as_mut()
            .ok_or_else(|| Error::Render("projected line style has no line".into()))?
            .pattern = crate::StrokePattern::Solid;
    }
    key.explanation = GuideExplanation::Projected(GuideExplanationMask {
        alpha: source.alpha_bindings.explanation(),
        point_fill,
        point_outline,
        line_stroke,
        fill_paint,
        fill_outline,
        point_shape,
        line_pattern,
    });
    Ok(key)
}

fn paint(
    binding: Option<&RgbBinding>,
    paint: Option<&mut Color>,
    projection: Projection,
    neutral: Option<OpaqueColor>,
) -> Result<Option<RgbExplanation>> {
    match (binding, paint) {
        (None, None) => Ok(None),
        (Some(RgbBinding::Mapped { .. }), Some(paint)) => {
            if projection == Projection::GroupStyle {
                let neutral = neutral.ok_or_else(|| {
                    Error::Render("group projection requires its owning label RGB".into())
                })?;
                *paint = crate::theme::color_from_rgb(neutral.as_rgb()).alpha(paint.get_alpha());
                Ok(Some(RgbExplanation::NeutralizedColor))
            } else {
                Ok(Some(RgbExplanation::ExplainedColor))
            }
        }
        (Some(_), Some(_)) => Ok(Some(RgbExplanation::Constant)),
        _ => Err(Error::Render(
            "projected RGB binding disagrees with component presence".into(),
        )),
    }
}

fn style(binding: Option<&StyleBinding>, projection: Projection) -> Option<StyleExplanation> {
    binding.map(|binding| match binding {
        StyleBinding::Role(_) => StyleExplanation::Constant,
        StyleBinding::InteractionGroup { .. } if projection == Projection::GroupStyle => {
            StyleExplanation::ExplainedGroup
        }
        StyleBinding::InteractionGroup { .. } => StyleExplanation::NeutralizedGroup,
    })
}
