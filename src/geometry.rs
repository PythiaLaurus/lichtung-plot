//! Caller-computed geometry accepted by distribution renderers.

use crate::Error;

const MAX_RAINCLOUD_GROUPS: usize = 512;
const MAX_RAINCLOUD_DENSITY_POINTS_PER_GROUP: usize = 2_048;
const MAX_RAINCLOUD_DENSITY_POINTS: usize = 1_000_000;
const MAX_RAINCLOUD_RAW_POINTS: usize = 1_000_000;
const MAX_HISTOGRAM_BINS: usize = 1_000_000;
const MAX_HISTOGRAM_KDE_POINTS: usize = 2_048;
const MAX_HISTOGRAM_RUGS: usize = 1_000_000;

/// Complete ordered geometry for a raincloud canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct RaincloudGeometry {
    /// Semantic dimensions used for inferred axis and legend titles.
    pub dimensions: RaincloudDimensions,
    /// Raincloud groups in exact left-to-right display order.
    pub groups: Vec<RaincloudGroup>,
}

/// Semantic dimensions carried by self-contained raincloud geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaincloudDimensions {
    /// Raw semantic identity used for the categorical x-axis title.
    pub category: String,
    /// Raw semantic identity used for the continuous y-axis title.
    pub value: String,
    /// Optional raw semantic identity used for the categorical color legend title.
    pub hue: Option<String>,
}

/// Caller-computed geometry and categorical identity for one raincloud group.
#[derive(Debug, Clone, PartialEq)]
pub struct RaincloudGroup {
    /// Raw categorical x-axis identity, resolved through the chart's [`crate::Labels`].
    pub category: String,
    /// Optional raw categorical color and legend identity.
    pub hue: Option<String>,
    /// Density-body coordinates in strictly increasing y order.
    pub density: Vec<RaincloudDensityPoint>,
    /// First quartile y coordinate.
    pub q1: f64,
    /// Median y coordinate.
    pub median: f64,
    /// Third quartile y coordinate.
    pub q3: f64,
    /// Complete raw-point geometry in caller-supplied display order.
    pub points: Vec<RaincloudPoint>,
}

/// One point on a precomputed raincloud density boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaincloudDensityPoint {
    /// Data-space y coordinate.
    pub y: f64,
    /// Body width normalized to the inclusive range `[0, 1]`.
    pub normalized_width: f64,
}

/// One raw observation in a precomputed raincloud strip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaincloudPoint {
    /// Data-space y coordinate.
    pub y: f64,
    /// Horizontal strip coordinate normalized to the inclusive range `[0, 1]`.
    pub normalized_strip: f64,
}

/// Complete caller-computed density-axis histogram composite geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramGeometry {
    /// Semantic dimensions used for inferred axis titles.
    pub dimensions: HistogramDimensions,
    /// Contiguous bins in strictly increasing x order.
    pub bins: Vec<HistogramBin>,
    /// KDE polyline points in strictly increasing x order.
    pub kde: Vec<HistogramKdePoint>,
    /// Rug segments in caller-supplied display order.
    pub rugs: Vec<HistogramRug>,
}

/// Semantic dimensions carried by self-contained histogram geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramDimensions {
    /// Raw semantic identity used for the numeric x-axis title.
    pub value: String,
    /// Raw semantic identity used for the density y-axis title.
    pub density: String,
}

/// One precomputed histogram bin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramBin {
    /// Inclusive lower x edge.
    pub lower: f64,
    /// Upper x edge.
    pub upper: f64,
    /// Nonnegative density-axis height.
    pub density: f64,
}

/// One named point on a caller-computed histogram KDE polyline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramKdePoint {
    /// Data-space x coordinate.
    pub x: f64,
    /// Nonnegative density-axis y coordinate.
    pub y: f64,
}

/// One explicit rug segment in a precomputed histogram composite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramRug {
    /// Data-space x coordinate.
    pub x: f64,
    /// Density-axis segment baseline.
    pub baseline: f64,
    /// Density-axis segment top, strictly above `baseline`.
    pub top: f64,
}

impl RaincloudGeometry {
    pub(crate) fn validate(&self) -> Result<(f64, f64), Error> {
        validate_dimension(&self.dimensions.category, "raincloud category dimension")?;
        validate_dimension(&self.dimensions.value, "raincloud value dimension")?;
        if let Some(hue) = &self.dimensions.hue {
            validate_dimension(hue, "raincloud hue dimension")?;
        }
        if self.groups.is_empty() {
            return Err(data_error("raincloud geometry requires at least one group"));
        }
        if self.groups.len() > MAX_RAINCLOUD_GROUPS {
            return Err(data_error(format!(
                "raincloud geometry has {} groups, exceeding the safety limit of {MAX_RAINCLOUD_GROUPS}",
                self.groups.len()
            )));
        }
        let has_hue = self.groups[0].hue.is_some();
        if self.dimensions.hue.is_some() != has_hue {
            return Err(data_error(
                "raincloud hue dimension and group hue identities must either both be present or both be absent",
            ));
        }
        let mut density_point_count = 0_usize;
        let mut raw_point_count = 0_usize;
        let mut y_range = (f64::INFINITY, f64::NEG_INFINITY);
        for (group_index, group) in self.groups.iter().enumerate() {
            if group.category.trim().is_empty() {
                return Err(data_error(format!(
                    "raincloud geometry group {group_index} has an empty category"
                )));
            }
            if group.hue.is_some() != has_hue {
                return Err(data_error(
                    "raincloud geometry hue identities must be present for every group or absent for every group",
                ));
            }

            if group.hue.as_ref().is_some_and(|hue| hue.trim().is_empty()) {
                return Err(data_error(format!(
                    "raincloud geometry group {group_index} has an empty hue"
                )));
            }
            for previous in &self.groups[..group_index] {
                if previous.category == group.category && previous.hue == group.hue {
                    return Err(data_error(format!(
                        "raincloud geometry contains duplicate category-hue identity ({:?}, {:?})",
                        group.category, group.hue
                    )));
                }
            }
            density_point_count = density_point_count
                .checked_add(group.density.len())
                .ok_or_else(|| data_error("raincloud density-point count overflow"))?;
            raw_point_count = raw_point_count
                .checked_add(group.points.len())
                .ok_or_else(|| data_error("raincloud raw-point count overflow"))?;
            if density_point_count > MAX_RAINCLOUD_DENSITY_POINTS {
                return Err(data_error(format!(
                    "raincloud geometry has more than {MAX_RAINCLOUD_DENSITY_POINTS} density points, exceeding the safety limit"
                )));
            }
            if raw_point_count > MAX_RAINCLOUD_RAW_POINTS {
                return Err(data_error(format!(
                    "raincloud geometry has more than {MAX_RAINCLOUD_RAW_POINTS} raw points, exceeding the safety limit"
                )));
            }
            let group_range = validate_raincloud_group(group, group_index)?;
            y_range.0 = y_range.0.min(group_range.0);
            y_range.1 = y_range.1.max(group_range.1);
        }
        Ok(y_range)
    }
}

fn validate_raincloud_group(
    group: &RaincloudGroup,
    group_index: usize,
) -> Result<(f64, f64), Error> {
    if group.density.len() < 2 {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} requires at least two density points"
        )));
    }
    if group.density.len() > MAX_RAINCLOUD_DENSITY_POINTS_PER_GROUP {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} has {} density points, exceeding the safety limit of {MAX_RAINCLOUD_DENSITY_POINTS_PER_GROUP}",
            group.density.len()
        )));
    }
    let mut positive_width = false;
    for (point_index, point) in group.density.iter().enumerate() {
        if !point.y.is_finite()
            || !point.normalized_width.is_finite()
            || !(0.0..=1.0).contains(&point.normalized_width)
        {
            return Err(data_error(format!(
                "raincloud geometry group {group_index} density point {point_index} requires finite y and normalized width in [0, 1]"
            )));
        }
        if point_index > 0 && point.y <= group.density[point_index - 1].y {
            return Err(data_error(format!(
                "raincloud geometry group {group_index} density y coordinates must be strictly increasing"
            )));
        }
        positive_width |= point.normalized_width > 0.0;
    }
    if !positive_width {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} requires a positive density width"
        )));
    }
    if !group.q1.is_finite()
        || !group.median.is_finite()
        || !group.q3.is_finite()
        || group.q1 > group.median
        || group.median > group.q3
    {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} requires finite q1 <= median <= q3"
        )));
    }
    let lower = group.density[0].y;
    let upper = group.density[group.density.len() - 1].y;
    if group.q1 < lower || group.q3 > upper {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} summary lies outside density support {lower}..{upper}"
        )));
    }
    if group.points.is_empty() {
        return Err(data_error(format!(
            "raincloud geometry group {group_index} requires at least one raw point"
        )));
    }
    for (point_index, point) in group.points.iter().enumerate() {
        if !point.y.is_finite()
            || !point.normalized_strip.is_finite()
            || !(0.0..=1.0).contains(&point.normalized_strip)
        {
            return Err(data_error(format!(
                "raincloud geometry group {group_index} raw point {point_index} requires finite y and normalized strip coordinate in [0, 1]"
            )));
        }
        if !(lower..=upper).contains(&point.y) {
            return Err(data_error(format!(
                "raincloud geometry group {group_index} raw point {point_index} lies outside density support {lower}..{upper}"
            )));
        }
    }
    Ok((lower, upper))
}

impl HistogramGeometry {
    pub(crate) fn validate(&self) -> Result<(f64, f64), Error> {
        validate_dimension(&self.dimensions.value, "histogram value dimension")?;
        validate_dimension(&self.dimensions.density, "histogram density dimension")?;
        if self.bins.is_empty() {
            return Err(data_error(
                "histogram composite geometry requires at least one bin",
            ));
        }
        if self.bins.len() > MAX_HISTOGRAM_BINS {
            return Err(data_error(format!(
                "histogram composite geometry has {} bins, exceeding the safety limit of {MAX_HISTOGRAM_BINS}",
                self.bins.len()
            )));
        }
        if self.kde.len() > MAX_HISTOGRAM_KDE_POINTS {
            return Err(data_error(format!(
                "histogram composite geometry has {} KDE points, exceeding the safety limit of {MAX_HISTOGRAM_KDE_POINTS}",
                self.kde.len()
            )));
        }
        if self.rugs.len() > MAX_HISTOGRAM_RUGS {
            return Err(data_error(format!(
                "histogram composite geometry has {} rugs, exceeding the safety limit of {MAX_HISTOGRAM_RUGS}",
                self.rugs.len()
            )));
        }
        let mut positive_density = false;
        for (index, bin) in self.bins.iter().enumerate() {
            if !bin.lower.is_finite()
                || !bin.upper.is_finite()
                || !bin.density.is_finite()
                || bin.lower >= bin.upper
                || bin.density < 0.0
            {
                return Err(data_error(format!(
                    "histogram composite bin {index} requires finite lower < upper and nonnegative finite density"
                )));
            }
            if index > 0
                && bin.lower.partial_cmp(&self.bins[index - 1].upper)
                    != Some(std::cmp::Ordering::Equal)
            {
                return Err(data_error(format!(
                    "histogram composite bins must be ordered and contiguous; bin {index} starts at {} after {}",
                    bin.lower,
                    self.bins[index - 1].upper
                )));
            }
            positive_density |= bin.density > 0.0;
        }
        if !positive_density {
            return Err(data_error(
                "histogram composite geometry requires a positive bin density",
            ));
        }
        let range = (self.bins[0].lower, self.bins[self.bins.len() - 1].upper);
        validate_histogram_kde(&self.kde, range)?;
        validate_histogram_rugs(&self.rugs, range)?;
        Ok(range)
    }
}

fn validate_histogram_kde(points: &[HistogramKdePoint], range: (f64, f64)) -> Result<(), Error> {
    if points.len() < 2 {
        return Err(data_error(
            "histogram composite geometry requires at least two KDE points",
        ));
    }
    let mut positive_density = false;
    for (index, point) in points.iter().enumerate() {
        if !point.x.is_finite() || !point.y.is_finite() || point.y < 0.0 {
            return Err(data_error(format!(
                "histogram composite KDE point {index} requires finite x and nonnegative finite y"
            )));
        }
        if !(range.0..=range.1).contains(&point.x) {
            return Err(data_error(format!(
                "histogram composite KDE point {index} lies outside bin range {}..{}",
                range.0, range.1
            )));
        }
        if index > 0 && point.x <= points[index - 1].x {
            return Err(data_error(
                "histogram composite KDE x coordinates must be strictly increasing",
            ));
        }
        positive_density |= point.y > 0.0;
    }
    if !positive_density {
        return Err(data_error(
            "histogram composite geometry requires a positive KDE y coordinate",
        ));
    }
    Ok(())
}

fn validate_histogram_rugs(rugs: &[HistogramRug], range: (f64, f64)) -> Result<(), Error> {
    if rugs.is_empty() {
        return Err(data_error(
            "histogram composite geometry requires at least one rug segment",
        ));
    }
    for (index, rug) in rugs.iter().enumerate() {
        if !rug.x.is_finite()
            || !rug.baseline.is_finite()
            || !rug.top.is_finite()
            || rug.baseline < 0.0
            || rug.baseline >= rug.top
        {
            return Err(data_error(format!(
                "histogram composite rug {index} requires finite x and 0 <= baseline < top"
            )));
        }
        if !(range.0..=range.1).contains(&rug.x) {
            return Err(data_error(format!(
                "histogram composite rug {index} lies outside bin range {}..{}",
                range.0, range.1
            )));
        }
    }
    Ok(())
}

fn data_error(message: impl Into<String>) -> Error {
    Error::Data(message.into())
}

fn validate_dimension(value: &str, name: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        Err(data_error(format!("{name} must be nonempty")))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one supplied-input contract retains support and independently keyed summary/observation paints while refusing invalid geometry"
    )]
    fn supplied_geometry_preserves_support_and_rejects_nonfinite_values() {
        let histogram = HistogramGeometry {
            dimensions: HistogramDimensions {
                value: "x".into(),
                density: "y".into(),
            },
            bins: vec![HistogramBin {
                lower: -2.0,
                upper: 2.0,
                density: 1.0,
            }],
            kde: vec![
                HistogramKdePoint { x: -2.0, y: 0.5 },
                HistogramKdePoint { x: 2.0, y: 0.5 },
            ],
            rugs: vec![HistogramRug {
                x: 0.0,
                baseline: 0.0,
                top: 0.1,
            }],
        };
        let raincloud = RaincloudGeometry {
            dimensions: RaincloudDimensions {
                category: "x".into(),
                value: "y".into(),
                hue: Some("cohort".into()),
            },
            groups: ["a", "b"]
                .into_iter()
                .map(|category| RaincloudGroup {
                    category: category.into(),
                    hue: Some("same".into()),
                    density: vec![
                        RaincloudDensityPoint {
                            y: -2.0,
                            normalized_width: 0.5,
                        },
                        RaincloudDensityPoint {
                            y: 2.0,
                            normalized_width: 0.5,
                        },
                    ],
                    q1: -1.0,
                    median: 0.0,
                    q3: 1.0,
                    points: vec![RaincloudPoint {
                        y: 0.0,
                        normalized_strip: 0.5,
                    }],
                })
                .collect(),
        };
        assert_eq!(histogram.validate().unwrap(), (-2.0, 2.0));
        assert_eq!(raincloud.validate().unwrap(), (-2.0, 2.0));
        let supplied = crate::Plot::raincloud(crate::RaincloudInput::geometry(raincloud.clone()))
            .with_alpha(
                crate::alpha::AlphaTarget::Point,
                crate::alpha::AlphaEncoding::observations(
                    crate::alpha::AlphaValues::linear(
                        crate::NumericValues::new("confidence", [0.0, 230.0]),
                        (0.0, 255.0),
                        (0, 255),
                    )
                    .unwrap(),
                ),
            )
            .with_alpha(
                crate::alpha::AlphaTarget::Line,
                crate::alpha::AlphaEncoding::keyed(
                    [
                        crate::CategoryValues::new("x", ["b", "a"]).unwrap(),
                        crate::CategoryValues::new("cohort", ["same", "same"]).unwrap(),
                    ],
                    crate::alpha::AlphaValues::linear(
                        crate::NumericValues::new("summary confidence", [64.0, 128.0]),
                        (0.0, 255.0),
                        (0, 255),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .with_alpha(
                crate::alpha::AlphaTarget::Fill,
                crate::alpha::AlphaEncoding::keyed(
                    [
                        crate::CategoryValues::new("x", ["b", "a"]).unwrap(),
                        crate::CategoryValues::new("cohort", ["same", "same"]).unwrap(),
                    ],
                    crate::alpha::AlphaValues::linear(
                        crate::NumericValues::new("body confidence", [0.0, 191.0]),
                        (0.0, 255.0),
                        (0, 255),
                    )
                    .unwrap(),
                )
                .unwrap(),
            );
        assert_eq!(supplied.row_count(), 0);
        let theme = crate::Theme::publication().resolve().unwrap();
        let chart =
            crate::plots::shared_x::prepare_panel_for_test(&supplied.erase(), &theme).unwrap();
        let selected = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                crate::layout::PanelBox::new(20.0, 600.0, 20.0, 420.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Distribution(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Raincloud payload")
        };
        let alphas = payload
            .marks
            .iter()
            .filter_map(|mark| match mark {
                crate::plots::categorical_distribution::Mark::Scatter(points, style) => {
                    Some((points.len(), style.color.alpha()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(alphas, [(1, 0), (1, 230)]);
        let inner_alpha = theme
            .marks
            .fills()
            .bodies()
            .inner_summary()
            .backend_style()
            .color
            .alpha();
        assert_eq!(
            payload
                .marks
                .iter()
                .filter_map(|mark| match mark {
                    crate::plots::categorical_distribution::Mark::Polygon(_, style) =>
                        Some(style.color.alpha()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [191, inner_alpha, 0, inner_alpha]
        );
        assert_eq!(
            payload
                .marks
                .iter()
                .filter_map(|mark| match mark {
                    crate::plots::categorical_distribution::Mark::Line(_, style) =>
                        Some(style.color.alpha()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [128, 64]
        );
        let descriptor = selected.discrete_guide.as_ref().unwrap();
        let sections = descriptor
            .sections
            .iter()
            .filter_map(|section| match &section.body {
                crate::decorations::legend::discrete::GuideSectionBody::Alpha(alpha) => Some(alpha),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(sections.len(), 3);
        assert!(sections
            .iter()
            .all(|section| section.entries.len() == 5
                && section.sources[0].glyphs[0].members.len() == 2));
        let mut invalid_histogram = histogram;
        invalid_histogram.bins[0].lower = f64::INFINITY;
        let mut invalid_raincloud = raincloud;
        invalid_raincloud.groups[0].density[0].y = f64::INFINITY;
        for histogram in [true, false] {
            let error = if histogram {
                invalid_histogram.validate()
            } else {
                invalid_raincloud.validate()
            }
            .unwrap_err();
            assert!(matches!(error, Error::Data(_)), "{error}");
        }
    }
}
