//! Decode literal owner policies before borrowing any mutable Python owner.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

macro_rules! choice {
    ($name:ident, $native:ty, {$($literal:literal => $variant:path),+ $(,)?}) => {
        pub(super) fn $name(value: &str) -> PyResult<$native> {
            match value {
                $($literal => Ok($variant),)+
                _ => Err(PyValueError::new_err(concat!(
                    stringify!($name), " must be one of: ", $($literal, " ",)+
                ))),
            }
        }
    };
}

choice!(alignment, core::TextAlignment, {
    "start" => core::TextAlignment::Start,
    "center" => core::TextAlignment::Center,
    "end" => core::TextAlignment::End,
});
choice!(guide_role, core::GuideTextRole, {
    "title" => core::GuideTextRole::Title,
    "entry" => core::GuideTextRole::Entry,
    "number" => core::GuideTextRole::Number,
});
choice!(composition, core::LegendComposition, {
    "combined" => core::LegendComposition::Combined,
    "separate" => core::LegendComposition::Separate,
});
choice!(canvas_legend, core::CanvasLegendPolicy, {
    "per_plot" => core::CanvasLegendPolicy::PerPlot,
    "shared" => core::CanvasLegendPolicy::Shared,
    "hidden" => core::CanvasLegendPolicy::Hidden,
});
choice!(legend_position, core::LegendPosition, {
    "auto" => core::LegendPosition::Auto,
    "inside_top_left" => core::LegendPosition::InsideTopLeft,
    "inside_top" => core::LegendPosition::InsideTop,
    "inside_top_right" => core::LegendPosition::InsideTopRight,
    "inside_right_top" => core::LegendPosition::InsideRightTop,
    "inside_right" => core::LegendPosition::InsideRight,
    "inside_right_bottom" => core::LegendPosition::InsideRightBottom,
    "inside_bottom_left" => core::LegendPosition::InsideBottomLeft,
    "inside_bottom" => core::LegendPosition::InsideBottom,
    "inside_bottom_right" => core::LegendPosition::InsideBottomRight,
    "inside_left_top" => core::LegendPosition::InsideLeftTop,
    "inside_left" => core::LegendPosition::InsideLeft,
    "inside_left_bottom" => core::LegendPosition::InsideLeftBottom,
    "outside_right_top" => core::LegendPosition::OutsideRightTop,
    "outside_right" => core::LegendPosition::OutsideRight,
    "outside_right_bottom" => core::LegendPosition::OutsideRightBottom,
    "outside_top_left" => core::LegendPosition::OutsideTopLeft,
    "outside_top" => core::LegendPosition::OutsideTop,
    "outside_top_right" => core::LegendPosition::OutsideTopRight,
    "outside_bottom_left" => core::LegendPosition::OutsideBottomLeft,
    "outside_bottom" => core::LegendPosition::OutsideBottom,
    "outside_bottom_right" => core::LegendPosition::OutsideBottomRight,
    "outside_left_top" => core::LegendPosition::OutsideLeftTop,
    "outside_left" => core::LegendPosition::OutsideLeft,
    "outside_left_bottom" => core::LegendPosition::OutsideLeftBottom,
});
choice!(line_pattern, core::LinePattern, {
    "solid" => core::LinePattern::Solid,
    "dashed" => core::LinePattern::Dashed,
});
choice!(raw_overlay, core::RawOverlay, {
    "hidden" => core::RawOverlay::Hidden,
    "points" => core::RawOverlay::Points,
    "jittered" => core::RawOverlay::Jittered,
});
choice!(spacing, core::BarSpacing, {
    "comfort" => core::BarSpacing::Comfort,
    "compact" => core::BarSpacing::Compact,
});
choice!(histogram_stat, core::HistogramStat, {
    "count" => core::HistogramStat::Count,
    "probability" => core::HistogramStat::Probability,
    "density" => core::HistogramStat::Density,
});
choice!(qq_line, core::QqLine, {
    "quartiles" => core::QqLine::Quartiles,
    "identity" => core::QqLine::Identity,
    "hidden" => core::QqLine::Hidden,
});
choice!(regression_band, core::RegressionBand, {
    "pointwise_mean_95" => core::RegressionBand::PointwiseMean95,
    "hidden" => core::RegressionBand::Hidden,
});
choice!(line_path_style, core::LinePathStyle, {
    "neutral" => core::LinePathStyle::Neutral,
    "hue" => core::LinePathStyle::Hue,
});
choice!(axis_scale, core::AxisScale, {
    "linear" => core::AxisScale::Linear,
    "log10" => core::AxisScale::Log10,
});
choice!(endpoints, core::LabelEndpoints, {
    "none" => core::LabelEndpoints::None,
    "first" => core::LabelEndpoints::First,
    "last" => core::LabelEndpoints::Last,
    "both" => core::LabelEndpoints::Both,
});
choice!(canvas_categories, core::CanvasCategoryLabels, {
    "every_plot" => core::CanvasCategoryLabels::EveryPlot,
    "first_column_per_row" => core::CanvasCategoryLabels::FirstColumnPerRow,
});

pub(super) fn legend(
    policy: &str,
    position: &Bound<'_, PyAny>,
    unset: &Bound<'_, PyAny>,
) -> PyResult<core::LegendPolicy> {
    if policy != "visible" && !position.is(unset) {
        return Err(PyTypeError::new_err(
            "only a visible legend accepts position",
        ));
    }
    match policy {
        "automatic" => Ok(core::LegendPolicy::Automatic),
        "hidden" => Ok(core::LegendPolicy::Hidden),
        "visible" => Ok(core::LegendPolicy::Visible(if position.is(unset) {
            core::LegendPosition::Auto
        } else {
            legend_position(position.extract()?)?
        })),
        _ => Err(PyValueError::new_err(
            "legend policy must be 'automatic', 'hidden', or 'visible'",
        )),
    }
}

pub(super) fn category_order(value: &Bound<'_, PyAny>) -> PyResult<core::CategoryOrder> {
    if value.is_instance_of::<PyString>() {
        match value.extract::<&str>()? {
            "observed" => Ok(core::CategoryOrder::Observed),
            "value_ascending" => Ok(core::CategoryOrder::Value(core::ValueSort::Ascending)),
            "value_descending" => Ok(core::CategoryOrder::Value(core::ValueSort::Descending)),
            _ => Err(PyValueError::new_err("invalid category ordering policy")),
        }
    } else {
        Ok(core::CategoryOrder::Explicit(value.extract()?))
    }
}

pub(super) fn category_labels(
    every: usize,
    offset: usize,
    endpoints: &str,
) -> PyResult<core::CategoryLabelSelection> {
    Ok(core::CategoryLabelSelection::every(every)
        .and_then(|value| value.offset(offset))
        .map_err(crate::native_error)?
        .endpoints(self::endpoints(endpoints)?))
}
