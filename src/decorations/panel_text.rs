//! Caller-owned text anchored inside one standard chart panel.

use crate::Error;

use crate::layout::{Layout, PanelBox};
use crate::render::backend::{with_clip, with_layer, Backend, Point2D, TextAnchor, TextStyle};
use crate::theme::ResolvedTheme;
use crate::{PanelCorner, Plot};

const PANEL_TEXT_PADDING: f64 = 8.0;

struct PanelTextPlan<'a> {
    text: &'a str,
    position: Point2D,
    anchor: TextAnchor,
    bounds: PanelBox,
    corner: PanelCorner,
}

pub(crate) fn draw(
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if plot.panel_text_annotations().is_empty() {
        return Ok(());
    }
    let plans = plans(plot, layout, theme)?;
    with_layer(backend, &plot.layer_name("panel-text"), |backend| {
        with_clip(backend, layout.projection.panel_box(), |backend| {
            for plan in plans {
                backend.draw_text(
                    plan.position,
                    plan.text,
                    &TextStyle {
                        color: theme.foreground,
                        font_family: theme.font_family.clone(),
                        font_size: theme.detail_font_size,
                        weight: crate::TextWeight::Normal,
                        anchor: plan.anchor,
                        rotation: 0.0,
                    },
                )?;
            }
            Ok(())
        })
    })
}

pub(crate) fn conflicting_legend_corner(
    plot: &Plot,
    layout: &Layout,
    legend: crate::decorations::legend::LegendBox,
    theme: &ResolvedTheme,
) -> Result<Option<PanelCorner>, Error> {
    Ok(plans(plot, layout, theme)?.into_iter().find_map(|plan| {
        rectangles_overlap(
            plan.bounds,
            PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
        )
        .then_some(plan.corner)
    }))
}

fn plans<'a>(
    plot: &'a Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
) -> Result<Vec<PanelTextPlan<'a>>, Error> {
    let panel = layout.projection.panel_box();
    let available_width = panel.width() - PANEL_TEXT_PADDING * 2.0;
    let text_height = f64::from(theme.detail_font_size) * 1.2;
    if available_width <= 0.0 || text_height + PANEL_TEXT_PADDING * 2.0 > panel.height() {
        return Err(Error::Render(
            "panel text has no usable space inside the resolved chart panel; increase the canvas"
                .into(),
        ));
    }

    let plans = plot
        .panel_text_annotations()
        .iter()
        .map(|annotation| {
            let width =
                crate::layout::estimated_text_width(annotation.text(), theme.detail_font_size);
            if width > available_width {
                return Err(Error::Render(format!(
                    "panel text {:?} requires {width:.1}px but only {available_width:.1}px is available; increase chart width or shorten the annotation",
                    annotation.text()
                )));
            }
            let (x, anchor) = match annotation.corner() {
                PanelCorner::TopLeft | PanelCorner::BottomLeft => {
                    let x = panel.x0 + PANEL_TEXT_PADDING;
                    (x, TextAnchor::Start)
                }
                PanelCorner::TopRight | PanelCorner::BottomRight => {
                    let x = panel.x1 - PANEL_TEXT_PADDING;
                    (x, TextAnchor::End)
                }
            };
            let y = match annotation.corner() {
                PanelCorner::TopLeft | PanelCorner::TopRight => {
                    panel.y1 - PANEL_TEXT_PADDING - text_height / 2.0
                }
                PanelCorner::BottomLeft | PanelCorner::BottomRight => {
                    panel.y0 + PANEL_TEXT_PADDING + text_height / 2.0
                }
            };
            let x = x.round();
            let y = y.round();
            let (left, right) = match anchor {
                TextAnchor::Start => (x, x + width),
                TextAnchor::End => (x - width, x),
                TextAnchor::Middle => (x - width / 2.0, x + width / 2.0),
            };
            Ok(PanelTextPlan {
                text: annotation.text(),
                position: Point2D::new(x, y),
                anchor,
                bounds: PanelBox::new(
                    left,
                    right,
                    y - text_height / 2.0,
                    y + text_height / 2.0,
                ),
                corner: annotation.corner(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (index, left) in plans.iter().enumerate() {
        if let Some(right) = plans[index + 1..]
            .iter()
            .find(|right| rectangles_overlap(left.bounds, right.bounds))
        {
            return Err(Error::Render(format!(
                "panel text annotations at {:?} and {:?} overlap; shorten or remove one annotation",
                left.corner, right.corner
            )));
        }
    }
    Ok(plans)
}

fn rectangles_overlap(left: PanelBox, right: PanelBox) -> bool {
    left.x0 < right.x1 && left.x1 > right.x0 && left.y0 < right.y1 && left.y1 > right.y0
}
