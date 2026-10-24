//! Resolve raw references once from final scientific axes and owning root geometry.

use super::{Coordinate, DataCoordinate, DataValue, Position, TextPosition};
use crate::layout::{Layout, PanelBox};
use crate::render::backend::Point2D;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, Result};

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
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
) -> Result<[ReferenceAxis; 2]> {
    let Position::Coordinates(x, y) = &position.0 else {
        return Err(Error::Render(
            "corner references require their semantic inset".into(),
        ));
    };
    let panel = layout.projection.panel_box();
    let axis = |coordinate: &super::TextCoordinate, side: usize| -> Result<ReferenceAxis> {
        match &coordinate.0 {
            Coordinate::Panel(value) => Ok(ReferenceAxis::Panel(value.get())),
            Coordinate::Figure(value) => Ok(ReferenceAxis::Figure(
                value.get() * {
                    let (width, height) = plot.figure_dimensions()?;
                    f64::from(if side == 0 { width } else { height })
                },
            )),
            Coordinate::Data(value) => {
                if let DataValue::Category(raw) = &value.0 {
                    let axes = plot.prepared_axis_inputs().ok_or_else(|| {
                        Error::Render("data text reference requires actual axes".into())
                    })?;
                    let index = axes.categories[side]
                        .as_ref()
                        .and_then(|ticks| ticks.iter().position(|tick| &tick.label == raw))
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
                let value = data(value, side, plot, theme)?;
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

pub(crate) fn point(reference: [ReferenceAxis; 2], panel: PanelBox) -> Point2D {
    Point2D::new(
        reference[0].resolve(panel.x0, panel.x1),
        reference[1].resolve(panel.y0, panel.y1),
    )
}

pub(crate) fn data(
    coordinate: &DataCoordinate,
    side: usize,
    plot: &Plot,
    theme: &ResolvedTheme,
) -> Result<f64> {
    let axes = plot
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("data text reference requires prepared actual axes".into()))?;
    let categories = axes.categories[side].as_deref();
    match &coordinate.0 {
        DataValue::Numeric(value) => {
            if categories.is_some() {
                return Err(Error::Data(
                    "numeric text reference requires an actual numeric axis".into(),
                ));
            }
            let value = value.get();
            if side == 0 && axes.logarithmic {
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
            .and_then(|ticks| ticks.iter().find(|tick| &tick.label == raw))
            .map(|tick| tick.position)
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
            if let Some(distributions) = &plot.prepared_distributions {
                let marks = distributions
                    .panels
                    .iter()
                    .find(|(scope, _)| scope.as_deref() == plot.axis_scope())
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| Error::Data("annotation distribution scope is absent".into()))?;
                return marks.series_center(category, series);
            }
            let bars=plot.prepared_bars.as_ref().and_then(|input|input.panels.iter().find(|(scope,_)|scope.as_deref()==plot.axis_scope())).map(|(_,bars)|bars).ok_or_else(||Error::Data("category-series text reference requires positioned series on the actual axis".into()))?;
            let category = bars
                .categories
                .iter()
                .position(|raw| raw == category)
                .ok_or_else(|| Error::Data("annotation category is absent".into()))?;
            let series = bars
                .series
                .iter()
                .position(|raw| raw == series)
                .ok_or_else(|| Error::Data("annotation series is absent".into()))?;
            Ok(crate::plots::bar_data::category_center(
                category,
                Some(series),
                bars,
                crate::plots::bar_data::mode(plot),
                theme
                    .marks
                    .get_extents()
                    .get_categories()
                    .group_span(side == 1),
            ))
        }
    }
}
