//! Resolve raw references once from final scientific axes and owning root geometry.

use super::{Coordinate, DataCoordinate, DataValue, Position, TextPosition};
use crate::layout::PixelRect;
use crate::render::backend::Point2D;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReferenceAxis {
    Panel(f64),
    Figure(f64),
}
impl ReferenceAxis {
    pub(crate) fn resolve(self, lower: f64, upper: f64) -> f64 {
        match self {
            Self::Panel(value) => lower + value * (upper - lower),
            Self::Figure(value) => value,
        }
    }
}

pub(crate) fn resolve(
    position: &TextPosition,
    reference: &crate::text::orientation::TextReferenceResolver<'_, '_>,
    theme: &crate::theme::ResolvedTheme,
) -> Result<[ReferenceAxis; 2]> {
    let Position::Coordinates(x, y) = &position.0 else {
        return Err(Error::Render(
            "corner references require their semantic inset".into(),
        ));
    };
    let layout = reference.layout;
    let registered = reference.registered;
    let panel = layout.projection.panel_box();
    let axis = |coordinate: &super::TextCoordinate, side: usize| -> Result<ReferenceAxis> {
        match &coordinate.0 {
            Coordinate::Panel(value) => Ok(ReferenceAxis::Panel(value.get())),
            Coordinate::Figure(value) => Ok(ReferenceAxis::Figure(
                value.get() * {
                    let (width, height) = reference.dimensions.figure_size().dimensions()?;
                    f64::from(if side == 0 { width } else { height })
                },
            )),
            Coordinate::Data(value) => {
                if let DataValue::Category(raw) = &value.0 {
                    let axes = registered.axes.coordinate().ok_or_else(|| {
                        Error::Render("data text reference requires actual axes".into())
                    })?;
                    let index = axes
                        .categories(side)
                        .and_then(|categories| categories.raw.iter().position(|label| label == raw))
                        .ok_or_else(|| {
                            Error::Data(format!(
                                "annotation category {raw:?} is absent from the actual axis"
                            ))
                        })?;
                    let selected = if side == 0 {
                        &layout.x_axis
                    } else {
                        &layout.y_axis
                    };
                    if let Some(cells) = &selected.assigned_cells {
                        let (lower, upper) = cells[index];
                        let coordinate = f64::midpoint(lower, upper);
                        return Ok(ReferenceAxis::Panel(if side == 0 {
                            (coordinate - panel.x0) / panel.width()
                        } else {
                            (coordinate - panel.y0) / panel.height()
                        }));
                    }
                }
                let value = data(value, side, registered, theme)?;
                let (x, y) = if side == 0 {
                    layout
                        .projection
                        .project(value, layout.projection.y_range().0)
                } else {
                    layout
                        .projection
                        .project(layout.projection.x_range().0, value)
                };
                let fraction = if side == 0 {
                    (x - panel.x0) / panel.width()
                } else {
                    (y - panel.y0) / panel.height()
                };
                if !fraction.is_finite() {
                    return Err(Error::Render(
                        "annotation data reference cannot be projected".into(),
                    ));
                }
                Ok(ReferenceAxis::Panel(fraction))
            }
        }
    };
    Ok([axis(x, 0)?, axis(y, 1)?])
}

pub(crate) fn point(reference: [ReferenceAxis; 2], panel: PixelRect) -> Point2D {
    Point2D::new(
        reference[0].resolve(panel.x0, panel.x1),
        reference[1].resolve(panel.y0, panel.y1),
    )
}

pub(crate) fn data(
    coordinate: &DataCoordinate,
    side: usize,
    registered: &crate::orchestrator::registration::PlotRegistration<'_>,
    theme: &crate::theme::ResolvedTheme,
) -> Result<f64> {
    let axes = registered
        .axes
        .coordinate()
        .ok_or_else(|| Error::Render("data text reference requires prepared actual axes".into()))?;
    let categories = axes.categories(side);
    match &coordinate.0 {
        DataValue::Numeric(value) => {
            if categories.is_some() {
                return Err(Error::Data(
                    "numeric text reference requires an actual numeric axis".into(),
                ));
            }
            let value = value.get();
            if side == 0 && axes.logarithmic() {
                if value <= 0.0 {
                    return Err(Error::Render(
                        "nonpositive annotation reference cannot be projected on Log10".into(),
                    ));
                }
                Ok(value.log10())
            } else {
                Ok(value)
            }
        }
        DataValue::Category(raw) => categories
            .and_then(|categories| {
                categories
                    .raw
                    .iter()
                    .position(|label| label == raw)
                    .map(|index| categories.ticks[index].position)
            })
            .ok_or_else(|| {
                Error::Data(format!(
                    "annotation category {raw:?} is absent from the actual axis"
                ))
            }),
        DataValue::Series(category, series) => {
            if categories.is_none() {
                return Err(Error::Data(
                    "category-series reference requires the categorical axis".into(),
                ));
            }
            if matches!(
                &registered.family,
                crate::orchestrator::registration::PreparedFamily::Distribution { .. }
            ) {
                let marks = Some(registered.family.as_ref())
                    .and_then(|family| match family {
                        crate::orchestrator::registration::FamilyRef::Distribution {
                            marks,
                            ..
                        } => Some(marks),
                        _ => None,
                    })
                    .ok_or_else(|| Error::Data("annotation distribution scope is absent".into()))?;
                return marks.series_center(category, series);
            }
            let mode = Some(registered.family.as_ref()).and_then(|family| match family { crate::orchestrator::registration::FamilyRef::Bars { mode, .. } => Some(mode), _ => None })
                .ok_or_else(||Error::Data("category-series text reference requires positioned series on the actual axis".into()))?;
            let category = categories
                .expect("positioned categorical axis")
                .raw
                .iter()
                .position(|raw| raw == category)
                .ok_or_else(|| Error::Data("annotation category is absent".into()))?;
            let groups = registered
                .family
                .as_ref()
                .positioned_groups()
                .expect("prepared Bar group domain");
            let series = groups
                .iter()
                .position(|raw| raw == series)
                .ok_or_else(|| Error::Data("annotation series is absent".into()))?;
            Ok(crate::plots::bar_data::category_center(
                category,
                Some(series),
                groups.len(),
                mode,
                theme
                    .marks
                    .get_extents()
                    .get_categories()
                    .group_span(side == 1),
            ))
        }
    }
}
