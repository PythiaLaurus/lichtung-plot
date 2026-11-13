//! Root-prepared ordered series, trajectories, and source-ordered paths.

use crate::alpha::{
    marks::{fill_style, line_style, point_ink, push_point, ObservationAlpha},
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::data::{CollectivePair, PlotData};
use crate::decorations::legend::source::{FillRole, LayerLabelContext, LineRole, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, PlacedText, Point2D, PointStyle, TextAnchor,
};
use crate::spec::{AxisDomain, LinePathStyle, PlotRenderContext, PlotRenderResult};
use crate::theme::{AccentColors, CategoryColors};
use crate::{Error, Plot, PlotKind};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct PreparedLine {
    mark_ranges: [(f64, f64); 2],
    groups: Vec<LineGroup>,
    hues: Vec<String>,
    layers: Vec<PointLayer>,
    primary_accent_slots: usize,
}

#[derive(Debug)]
struct LineGroup {
    identity: Option<CollectivePair>,
    hue_index: Option<usize>,
    points: Vec<LinePoint>,
    line_alpha: Option<ResolvedAlpha>,
    fill_alpha: Option<ResolvedAlpha>,
}

#[derive(Debug)]
struct LinePoint {
    x: f64,
    y: f64,
    interval: Option<(f64, f64)>,
    hue_index: Option<usize>,
    alpha: ObservationAlpha,
}

#[derive(Debug)]
struct PointLayer {
    points: Vec<(f64, f64)>,
    source_index: usize,
}

#[derive(Debug)]
pub(crate) struct LinePayload {
    clip: crate::layout::PixelRect,
    pub(crate) marks: Vec<LineMark>,
    layers: Vec<(Vec<Point2D>, PointStyle, Vec<PlacedText>)>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

/// Resolve one prepared path's painted hard span before parent allocation.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Line { marks, .. } => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Line intrinsic requires root-prepared paths".into()))?;

    let count = prepared.groups.iter().try_fold(0_u128, |total, group| {
        total
            .checked_add(group.points.len() as u128)
            .ok_or_else(|| Error::Render("Line primary vertex cardinality overflowed".into()))
    })?;
    if count == 0 {
        return Err(Error::Render(
            "Line has no prepared primary vertices".into(),
        ));
    }
    let path_style = if chart.kind() == PlotKind::Line {
        theme.marks.get_data_lines().get_series_line()
    } else {
        theme.marks.get_data_lines().get_path_line()
    }
    .backend_style((0, 0, 0));
    let mut path = 0.0_f64;
    for group in &prepared.groups {
        path = path.max(crate::render::geometry::line_ink_width(&line_style(
            group.line_alpha,
            path_style.clone(),
        ))?);
    }
    let points_visible =
        chart.kind() == PlotKind::Trajectory || chart.description().markers_visible();
    let mark = if points_visible {
        let radius = if chart.kind() == PlotKind::Line {
            theme.marks.get_points().get_radii().get_path()
        } else {
            theme.marks.get_points().get_radii().get_trajectory()
        };
        let point_style = chart.description().point_style(theme, (0, 0, 0), radius);
        point_ink(
            &point_style,
            prepared.groups.iter().flat_map(|group| {
                group
                    .points
                    .iter()
                    .map(|point| (point.x, point.y, point.alpha))
            }),
        )?
        .map_or((0.0, 0.0), crate::alpha::marks::PointInk::dimensions)
    } else {
        (0.0, 0.0)
    };
    let mut ribbon = (0.0_f64, 0.0_f64);
    if chart.kind() == PlotKind::Line && chart.description().line_band_configured() {
        for group in &prepared.groups {
            if !group.points.iter().all(|point| point.interval.is_some()) {
                continue;
            }
            let ink = crate::render::geometry::rectangular_fill_ink_outset(&fill_style(
                group.fill_alpha,
                theme
                    .marks
                    .get_fills()
                    .get_bands()
                    .get_line_band()
                    .backend_style((0, 0, 0)),
            ))?;
            ribbon = (ribbon.0.max(ink.0), ribbon.1.max(ink.1));
        }
    }
    Ok((
        if chart.user_x_limits().is_none() {
            mark.0.max(path).max(ribbon.0)
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            mark.1.max(path).max(ribbon.1)
        } else {
            0.0
        },
    ))
}

#[derive(Debug)]
pub(crate) enum LineMark {
    Ribbon(Vec<Point2D>, FillStyle),
    Line(Vec<Point2D>, LineStyle),
    Points(Vec<Point2D>, PointStyle),
}

impl PreparedLine {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let line = if chart.kind() == PlotKind::Line {
            theme.marks.get_data_lines().get_series_line()
        } else {
            theme.marks.get_data_lines().get_path_line()
        };
        let radius = if chart.kind() == PlotKind::Line {
            theme.marks.get_points().get_radii().get_path()
        } else {
            theme.marks.get_points().get_radii().get_trajectory()
        };
        let point = chart.description().point_style(theme, (0, 0, 0), radius);
        let band = theme.marks.get_fills().get_bands().get_line_band();
        for group in &self.groups {
            let unit = AlphaUnit::series(chart, group.identity.as_ref())?;
            guide.line(
                AlphaGlyphRole::Line,
                AlphaUnitKind::Series,
                unit.clone(),
                group.line_alpha,
                line.backend_style((0, 0, 0)),
                false,
            )?;
            if chart.kind() == PlotKind::Trajectory || chart.description().markers_visible() {
                for vertex in &group.points {
                    guide.point(AlphaGlyphRole::Primary, vertex.alpha, point.clone(), false)?;
                }
            }
            if chart.kind() == PlotKind::Line
                && chart.description().line_band_configured()
                && group.points.iter().all(|point| point.interval.is_some())
            {
                guide.fill(
                    AlphaGlyphRole::Fill,
                    AlphaUnitKind::Series,
                    unit,
                    group.fill_alpha,
                    band.backend_style((0, 0, 0)),
                    band.get_outline()
                        .is_some_and(|outline| outline.mapped_width().is_some()),
                )?;
            }
        }
        Ok(())
    }

    fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: PlotData<'_, '_>,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let resources = data.source_resources();
        let map = CategoryColors::resolve(
            chart.global_hue_levels().unwrap_or(&self.hues),
            theme.color_scales.get_qualitative(),
            chart.description().categorical_color_scale_spec(),
            resources,
        )?;
        let slots = self
            .primary_accent_slots
            .checked_add(self.layers.len())
            .ok_or_else(|| Error::Render("Line accent slot count overflowed".into()))?;
        let accent = AccentColors::resolve(theme.marks.get_accents(), slots)?;
        Ok((map, accent))
    }

    fn group_color(
        &self,
        chart: &Plot,
        group: &LineGroup,
        theme: &crate::text::TextRequestContext<'_, '_>,
        map: &CategoryColors,
        accent: &AccentColors,
    ) -> Result<(u8, u8, u8), Error> {
        let hue_paths = chart.kind() == PlotKind::LinePath
            && chart.description().line_path_style_spec() == LinePathStyle::Hue;
        if chart.kind() == PlotKind::Line {
            if let Some(hue) = group
                .identity
                .as_ref()
                .and_then(|identity| identity.hue.as_deref())
            {
                map.color(hue)
            } else {
                accent.color(0)
            }
        } else if hue_paths {
            let hue = group.hue_index.ok_or_else(|| {
                Error::Render("line path: hue-styled path has no resolved hue index".into())
            })?;
            map.color(&self.hues[hue])
        } else {
            Ok(theme.marks.get_neutral_paints().get_path().as_rgb())
        }
    }
    pub(crate) fn checked_primary_mark_count(&self) -> Result<u128, Error> {
        self.groups.iter().try_fold(0_u128, |count, group| {
            count
                .checked_add(group.points.len() as u128)
                .ok_or_else(|| Error::Render("Line primary vertex count overflowed".into()))
        })
    }

    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.hues.iter().map(String::as_str)
    }

    pub(crate) fn group_order(&self, chart: &Plot) -> Result<Vec<usize>, Error> {
        let hue_paths = chart.kind() == PlotKind::LinePath
            && chart.description().line_path_style_spec() == LinePathStyle::Hue;
        let mut order = crate::data::source::collect_source((0..self.groups.len()).map(Ok))?;
        crate::data::source::sort_source_by(&mut order, |a, b| {
            let key = |index: usize| {
                let group = &self.groups[index];
                group
                    .identity
                    .as_ref()
                    .and_then(|identity| {
                        if chart.kind() == PlotKind::Line || hue_paths {
                            chart
                                .global_collective_pairs()
                                .and_then(|pairs| pairs.iter().position(|pair| pair == identity))
                        } else {
                            chart.global_group_levels().and_then(|levels| {
                                levels
                                    .iter()
                                    .position(|level| Some(level) == identity.group.as_ref())
                            })
                        }
                    })
                    .unwrap_or(index)
            };
            (key(*a), *a).cmp(&(key(*b), *b))
        });
        Ok(order)
    }

    /// Exact legend glyphs from prepared semantic groups; no assigned panel,
    /// source reprojection, or font environment is needed.
    pub(crate) fn guide_entries_in_order(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: PlotData<'_, '_>,
        order: &[usize],
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (map, accent) = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, order, &map, &accent)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,

        order: &[usize],
        map: &CategoryColors,
        accent: &AccentColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let mut entries = Vec::new();
        let hue_paths = chart.kind() == PlotKind::LinePath
            && chart.description().line_path_style_spec() == LinePathStyle::Hue;
        for &index in order {
            let group = &self.groups[index];
            let color = self.group_color(chart, group, theme, map, accent)?;
            let Some(identity) = group
                .identity
                .as_ref()
                .filter(|identity| identity.hue.is_some())
            else {
                continue;
            };

            let has_fill = chart.kind() == PlotKind::Line
                && group.points.iter().all(|point| point.interval.is_some());
            let (line, line_role) = if chart.kind() == PlotKind::Line {
                (
                    theme.marks.get_data_lines().get_series_line(),
                    LineRole::SeriesLine,
                )
            } else {
                (
                    theme.marks.get_data_lines().get_path_line(),
                    LineRole::PathLine,
                )
            };
            crate::data::source::reserve_source(&mut entries, 1)?;
            let identity = chart.description().collective_legend_identity(identity)?;
            let mut key = GuideSourceKey::collective(identity, chart, false)?
                .with_line(line.backend_style(color), line_role);
            if chart.description().markers_visible() {
                key = key.with_point(
                    chart.description().point_style(
                        theme,
                        color,
                        if chart.kind() == PlotKind::Line {
                            theme.marks.get_points().get_radii().get_path()
                        } else {
                            theme.marks.get_points().get_radii().get_trajectory()
                        },
                    ),
                    PointRole::Primary,
                );
            }
            if has_fill {
                key = key.with_fill(
                    theme.marks.get_fills().get_bands().get_line_band(),
                    color,
                    FillRole::LineBand,
                );
            }
            entries.push(key);
        }
        if chart.kind() != PlotKind::Line
            && (chart.kind() == PlotKind::Trajectory || chart.description().markers_visible())
            && !hue_paths
        {
            let levels = chart.global_hue_levels().unwrap_or(&self.hues);
            crate::data::source::reserve_source(&mut entries, levels.len())?;
            for label in levels.iter().filter(|label| self.hues.contains(label)) {
                entries.push(
                    GuideSourceKey::row_hue(chart.description().point_legend_identity(label)?)?
                        .with_point(
                            chart.description().point_style(
                                theme,
                                map.color(label)?,
                                theme.marks.get_points().get_radii().get_trajectory(),
                            ),
                            PointRole::Primary,
                        ),
                );
            }
        }
        for layer in &self.layers {
            let index = layer.source_index;
            let source = &chart.description().point_layers()[index];
            if let Some(label) = source.legend_content() {
                crate::data::source::reserve_source(&mut entries, 1)?;
                let style = source.point_style(
                    accent.color(self.primary_accent_slots + index)?,
                    theme.marks.get_points().get_supplemental(),
                );
                let key = GuideSourceKey::supplemental(label, LayerLabelContext::for_plot(chart))?;
                entries.push(if source.get_color().is_some() {
                    key.with_exact_point(style, PointRole::Supplemental)
                } else {
                    key.with_point(style, PointRole::Supplemental)
                });
            }
        }
        Ok(entries)
    }

    #[expect(
        clippy::type_complexity,
        reason = "the original seeds and optional category source move directly to the registration without a second retained record"
    )]
    pub(crate) fn new(
        chart: &Plot,
        data: PlotData<'_, '_>,
        conditions: Option<&[String]>,
    ) -> Result<(Self, [(f64, f64); 2], Option<Vec<String>>), Error> {
        let (mut prepared, ranges, conditions) = Self::prepare_scientific(chart, data, conditions)?;
        let include_hue = chart.kind() == PlotKind::Line
            || (chart.kind() == PlotKind::LinePath
                && chart.description().line_path_style_spec() == LinePathStyle::Hue);
        let alpha = chart.description().prepare_collective_alpha(
            AlphaTarget::Line,
            AlphaUnitKind::Series,
            include_hue,
            data.facet_selector()?,
        )?;
        let fill_alpha = (chart.kind() == PlotKind::Line)
            .then(|| {
                chart.description().prepare_collective_alpha(
                    AlphaTarget::Fill,
                    AlphaUnitKind::Series,
                    true,
                    data.facet_selector()?,
                )
            })
            .transpose()?;
        for group in &mut prepared.groups {
            let identity = group.identity.as_ref().ok_or_else(|| {
                Error::Render("prepared line series has no scientific identity".into())
            })?;
            group.line_alpha = alpha.collective(chart.description(), identity)?;
            if let Some(alpha) = fill_alpha
                .as_ref()
                .filter(|_| group.points.iter().all(|point| point.interval.is_some()))
            {
                group.fill_alpha = alpha.collective(chart.description(), identity)?;
            }
        }
        Ok((prepared, ranges, conditions))
    }

    #[expect(
        clippy::too_many_lines,
        clippy::float_cmp,
        clippy::type_complexity,
        reason = "one normalization preserves family order; duplicate scientific coordinates compare exactly including signed zero"
    )]
    fn prepare_scientific(
        chart: &Plot,
        data: PlotData<'_, '_>,
        conditions: Option<&[String]>,
    ) -> Result<(Self, [(f64, f64); 2], Option<Vec<String>>), Error> {
        data.validate_row_source()?;
        let alpha = chart.description().prepare_alpha(
            AlphaTarget::Point,
            AlphaUnitKind::Observation,
            &[],
        )?;
        let kind = chart.kind();
        let family = match kind {
            PlotKind::Line => "line",
            PlotKind::Trajectory => "trajectory",
            PlotKind::LinePath => "line path",
            _ => {
                return Err(Error::Data(
                    "line preparation requires a line family".into(),
                ))
            }
        };
        let x_col = chart
            .description()
            .x_col()
            .ok_or_else(|| Error::Data(format!("{family}: x column not set")))?;
        let y_col = chart
            .description()
            .y_col()
            .ok_or_else(|| Error::Data(format!("{family}: y column not set")))?;
        if kind != PlotKind::Line && chart.description().group_col().is_none() {
            return Err(Error::Data(format!(
                "{family}: explicit graphical group column not set"
            )));
        }
        let channels = crate::data::observed_collectives(
            data,
            chart.description().group_col(),
            chart.description().group_name(),
            chart.description().hue_col(),
            chart.description().hue_name(),
            None,
            family,
        )?;
        // Indices and membership belong to observed scientific rows. The
        // owning chart supplies a separate coordinated palette order later.
        let hues = crate::data::source::collect_source(
            channels
                .hue_levels
                .iter()
                .map(|label| crate::data::source::copy_source_string(label)),
        )?;
        if let Some(exact) = chart.description().categorical_map_spec() {
            for hue in &hues {
                if exact.color_in(hue).is_none() {
                    return Err(Error::Data(format!(
                        "line: category {hue:?} has no caller-owned color"
                    )));
                }
            }
        }
        let y = crate::data::strict_finite_f64(data, y_col, family)?;
        let (x, conditions) = if data.column(x_col)?.is_numeric() {
            (crate::data::strict_finite_f64(data, x_col, family)?, None)
        } else if kind == PlotKind::Trajectory {
            let labels = crate::data::strict_normalized_strings(data, x_col, family, "condition")?;
            let levels =
                if let Some(levels) = conditions.or_else(|| chart.global_line_condition_levels()) {
                    crate::data::source::collect_source(
                        levels
                            .iter()
                            .map(|label| crate::data::source::copy_source_string(label)),
                    )?
                } else {
                    crate::data::source::source_levels(&labels)?
                };
            let positions = crate::data::source::collect_source(labels.iter().map(|label| {
                let index = crate::data::source::source_position(&levels, |level| level == label)
                    .ok_or_else(|| {
                    Error::Data(format!(
                        "trajectory: condition {label:?} is absent from global order"
                    ))
                })?;
                Ok(categorical_index(index)? + 0.5)
            }))?;
            (positions, Some(levels))
        } else {
            return Err(Error::Data(format!("{family}: x must be numeric")));
        };
        if x.is_empty() || x.len() != y.len() || channels.rows.len() != x.len() {
            return Err(Error::Data(format!(
                "{family}: aligned columns must have equal non-zero lengths"
            )));
        }
        let bounds = if kind == PlotKind::Line {
            chart
                .description()
                .interval_bound_cols()
                .map(|(lower, upper)| {
                    Ok::<_, Error>((
                        crate::data::strict_finite_f64(data, lower, family)?,
                        crate::data::strict_finite_f64(data, upper, family)?,
                    ))
                })
                .transpose()?
        } else {
            None
        };
        let hue_paths = kind == PlotKind::LinePath
            && chart.description().line_path_style_spec() == LinePathStyle::Hue;
        if hue_paths && chart.description().hue_col().is_none() {
            return Err(Error::Data(
                "line path: LinePathStyle::Hue requires a categorical hue column".into(),
            ));
        }
        let identities = chart
            .global_collective_pairs()
            .unwrap_or(&channels.identities);
        let group_levels = chart
            .global_group_levels()
            .unwrap_or(&channels.group_levels);
        let collective = kind == PlotKind::Line || hue_paths;
        if kind == PlotKind::Trajectory {
            let mut rows = crate::data::source::collect_source((0..x.len()).map(Ok))?;
            crate::data::source::sort_source_by(&mut rows, |a, b| {
                let normalized = |row: usize| if x[row] == 0.0 { 0.0 } else { x[row] };
                channels.rows[*a]
                    .group
                    .cmp(&channels.rows[*b].group)
                    .then_with(|| normalized(*a).total_cmp(&normalized(*b)))
                    .then_with(|| a.cmp(b))
            });

            if let Some(row) = rows
                .windows(2)
                .filter(|pair| {
                    channels.rows[pair[0]].group == channels.rows[pair[1]].group
                        && x[pair[0]] == x[pair[1]]
                })
                .map(|pair| pair[1])
                .min()
            {
                return Err(Error::Data(format!(
                    "trajectory: duplicate normalized (group, condition) at source row {}",
                    data.source_row(row)
                )));
            }
        }
        let count = if collective {
            identities.len()
        } else {
            group_levels.len()
        };
        let mut groups = Vec::new();
        crate::data::source::reserve_source(&mut groups, count)?;
        for index in 0..count {
            let mut rows = Vec::new();
            let belongs = |row: usize| {
                if collective {
                    channels.rows[row] == identities[index]
                } else {
                    channels.rows[row].group.as_ref() == Some(&group_levels[index])
                }
            };

            let count = (0..x.len()).filter(|row| belongs(*row)).count();
            crate::data::source::reserve_source(&mut rows, count)?;
            for row in 0..x.len() {
                if belongs(row) {
                    rows.push(row);
                }
            }
            if rows.is_empty() {
                continue;
            }
            if kind != PlotKind::Line && rows.len() < 2 {
                return Err(Error::Data(if hue_paths {
                    format!(
                        "line path: {} requires at least two retained points",
                        identities[index].description(channels.collapsed)
                    )
                } else {
                    format!("trajectory: graphical group {:?} requires at least two retained conditions", group_levels[index])
                }));
            }
            let identity = Some(if collective {
                crate::data::source::copy_source_pair(&identities[index])?
            } else {
                CollectivePair {
                    group: Some(crate::data::source::copy_source_string(
                        &group_levels[index],
                    )?),
                    hue: None,
                }
            });
            let hue_index = identity
                .as_ref()
                .and_then(|identity| identity.hue.as_ref())
                .and_then(|hue| crate::data::source::source_position(&hues, |level| level == hue));
            if hue_paths && hue_index.is_none() {
                return Err(Error::Data(
                    "line path: hue is absent from the resolved hue order".into(),
                ));
            }
            let mut points = crate::data::source::collect_source(rows.iter().map(|row| {
                let row = *row;
                let interval = if let Some((lower, upper)) = &bounds {
                    let (lower, upper) = (lower[row], upper[row]);
                    if lower > y[row] || y[row] > upper {
                        return Err(Error::Data(format!(
                            "line: source row {} must satisfy lower <= y <= upper",
                            data.source_row(row)
                        )));
                    }
                    Some((lower, upper))
                } else {
                    None
                };
                Ok(LinePoint {
                    x: x[row],
                    y: y[row],
                    alpha: ObservationAlpha::resolve(data, &alpha, row)?,
                    interval,
                    hue_index: channels.rows[row].hue.as_ref().and_then(|hue| {
                        crate::data::source::source_position(&hues, |level| level == hue)
                    }),
                })
            }))?;
            if kind != PlotKind::LinePath {
                crate::data::source::sort_source_by(&mut points, |a, b| a.x.total_cmp(&b.x));

                if kind == PlotKind::Line && points.windows(2).any(|pair| pair[0].x == pair[1].x) {
                    return Err(Error::Data(format!(
                        "line: duplicate x value within series {}",
                        identities[index].description(channels.collapsed)
                    )));
                }
            }
            groups.push(LineGroup {
                identity,
                hue_index,
                points,
                line_alpha: None,
                fill_alpha: None,
            });
        }
        let mut layers = Vec::new();
        crate::data::source::reserve_source(&mut layers, chart.description().point_layers().len())?;
        for (index, layer) in chart.description().point_layers().iter().enumerate() {
            let mut buffer = [0_u8; 64];
            let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
            std::io::Write::write_fmt(&mut cursor, format_args!("line point layer {index}"))
                .expect("bounded layer index");
            let length = usize::try_from(cursor.position()).expect("bounded layer index");
            let context = std::str::from_utf8(&buffer[..length]).expect("ASCII layer index");
            let x = layer.x_values();
            let y = layer.y_values();
            if x.is_empty() || x.len() != y.len() {
                return Err(Error::Data(
                    "line point layer: x and y columns must have equal non-zero lengths".into(),
                ));
            }
            let points = crate::data::source::collect_source((0..x.len()).map(|row| {
                Ok((
                    crate::data::strict_numeric_value(x, row, context)?,
                    crate::data::strict_numeric_value(y, row, context)?,
                ))
            }))?;
            let labels = layer.label_values();
            if let Some(values) = labels {
                for row in 0..values.len() {
                    if values.get(row).flatten().is_none() {
                        return Err(Error::Data(format!(
                            "{context}: label contains null at source row {row}"
                        )));
                    }
                }
            }
            if labels.is_some_and(|labels| labels.len() != points.len()) {
                return Err(Error::Data(
                    "line point layer: labels must align one-to-one with x and y".into(),
                ));
            }
            if let Some(text) = layer.legend_content() {
                text.required_source("point layer legend label")?;
            }
            layers.push(PointLayer {
                points,
                source_index: index,
            });
        }
        let mut ranges = [(f64::INFINITY, f64::NEG_INFINITY); 2];
        for group in &groups {
            for point in &group.points {
                ranges[0] = (ranges[0].0.min(point.x), ranges[0].1.max(point.x));
                let (lower, upper) = point.interval.unwrap_or((point.y, point.y));
                ranges[1] = (ranges[1].0.min(lower), ranges[1].1.max(upper));
            }
        }
        for layer in &layers {
            for &(x, y) in &layer.points {
                ranges[0] = (ranges[0].0.min(x), ranges[0].1.max(x));
                ranges[1] = (ranges[1].0.min(y), ranges[1].1.max(y));
            }
        }
        let mark_ranges = ranges;
        ranges[0] = if let Some(levels) = &conditions {
            (0.0, categorical_index(levels.len())?)
        } else {
            crate::layout::continuous_auto_range(
                ranges[0],
                chart
                    .description()
                    .x_axis_spec()
                    .window
                    .map(crate::layout::RawWindow::pair),
            )?
        };
        ranges[1] = crate::layout::continuous_auto_range(
            ranges[1],
            chart
                .description()
                .y_axis_spec()
                .window
                .map(crate::layout::RawWindow::pair),
        )?;
        let primary_accent_slots = usize::from(chart.description().hue_col().is_none());
        Ok((
            Self {
                mark_ranges,
                groups,
                hues,
                layers,
                primary_accent_slots,
            },
            ranges,
            conditions,
        ))
    }
}

fn categorical_index(index: usize) -> Result<f64, Error> {
    u32::try_from(index).map(f64::from).map_err(|_| {
        Error::Data(
            "trajectory: categorical condition count exceeds the supported coordinate range".into(),
        )
    })
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_, '_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let prepared = context
        .prepared_family()
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Line { marks, .. } => Some(marks),
            _ => None,
        })
        .ok_or_else(|| {
            Error::Render(
                "line measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let theme = context.theme;
    theme.text.prepared_font_environment()?;
    let data = context.data;
    project(
        chart,
        prepared,
        context
            .registered
            .expect("prepared line occurrence")
            .axes
            .categories(0)
            .is_some(),
        layout,
        theme,
        data,
        context.fit_scope(),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "selected projection retains series, ribbon, marker and label order without a backend"
)]
fn project(
    chart: &Plot,
    prepared: &PreparedLine,
    categorical_x: bool,
    layout: crate::layout::Layout,
    theme: &crate::text::TextRequestContext<'_, '_>,
    data: PlotData<'_, '_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let mut payload = LinePayload {
        clip: layout.projection.panel_box(),
        marks: Vec::new(),
        layers: Vec::new(),
        domain_expansions: [None; 2],
    };
    let project = |x, y| {
        let (x, y) = layout.projection.project(x, y);
        Point2D::new(x, y)
    };
    let hue_paths = chart.kind() == PlotKind::LinePath
        && chart.description().line_path_style_spec() == LinePathStyle::Hue;
    let order = prepared.group_order(chart)?;
    let (map, accent) = prepared.resolved_colors(chart, theme, data)?;

    for &index in &order {
        let group = &prepared.groups[index];
        let color = prepared.group_color(chart, group, theme, &map, &accent)?;
        let fill = fill_style(
            group.fill_alpha,
            theme
                .marks
                .get_fills()
                .get_bands()
                .get_line_band()
                .backend_style(color),
        );

        let has_fill = chart.kind() == PlotKind::Line
            && group.points.iter().all(|point| point.interval.is_some());
        if has_fill {
            let mut polygon = Vec::new();
            let count = group
                .points
                .len()
                .checked_mul(2)
                .ok_or_else(|| Error::Render("line ribbon point count overflow".into()))?;
            crate::data::source::reserve_source(&mut polygon, count)?;
            let mut has_area = false;
            for point in &group.points {
                let (lower, upper) = point.interval.expect("complete ribbon");
                has_area |= lower < upper;
                polygon.push(project(point.x, upper));
            }
            if let Some((first, last)) = group.points.first().zip(group.points.last()) {
                layout.projection.validate_span(
                    crate::layout::AxisSide::Bottom,
                    (first.x, last.x),
                    (polygon[0].x, polygon[polygon.len() - 1].x),
                    has_area && first.x < last.x,
                    "line ribbon polygon width",
                )?;
            }
            for (index, point) in group.points.iter().enumerate().rev() {
                let (lower, upper) = point.interval.expect("complete ribbon");
                let projected = project(point.x, lower);
                layout.projection.validate_span(
                    crate::layout::AxisSide::Left,
                    (lower, upper),
                    (projected.y, polygon[index].y),
                    lower < upper,
                    "line ribbon polygon",
                )?;
                polygon.push(projected);
            }
            crate::data::source::reserve_source(&mut payload.marks, 1)?;
            payload.marks.push(LineMark::Ribbon(polygon, fill.clone()));
        }
        let points = crate::data::source::collect_source(
            group
                .points
                .iter()
                .map(|point| Ok(project(point.x, point.y))),
        )?;
        if points.len() >= 2 {
            // Match canonical_line_points' global distinction rule, not just
            // adjacent segments. Keep source order and normalize signed zeros.
            let mut indices = crate::data::source::collect_source((0..points.len()).map(Ok))?;
            crate::data::source::sort_source_by(&mut indices, |a, b| {
                let key = |index: usize| {
                    let point = points[index];
                    let bits = |value: f64| if value == 0.0 { 0 } else { value.to_bits() };
                    (bits(point.x), bits(point.y), index)
                };
                key(*a).cmp(&key(*b))
            });

            for pair in indices.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if points[a] != points[b] {
                    continue;
                }
                let (first, second) = (&group.points[a], &group.points[b]);
                for (axis, source, projected) in [
                    (
                        crate::layout::AxisSide::Bottom,
                        (first.x, second.x),
                        (points[a].x, points[b].x),
                    ),
                    (
                        crate::layout::AxisSide::Left,
                        (first.y, second.y),
                        (points[a].y, points[b].y),
                    ),
                ] {
                    layout.projection.validate_span(
                        axis,
                        source,
                        projected,
                        source.0.min(source.1) < source.0.max(source.1),
                        "line distinct source points",
                    )?;
                }
            }
        }
        let line = if chart.kind() == PlotKind::Line {
            theme.marks.get_data_lines().get_series_line()
        } else {
            theme.marks.get_data_lines().get_path_line()
        }
        .backend_style(color);
        let line = line_style(group.line_alpha, line);
        let point_style = chart.description().point_style(
            theme,
            color,
            if chart.kind() == PlotKind::Line {
                theme.marks.get_points().get_radii().get_path()
            } else {
                theme.marks.get_points().get_radii().get_trajectory()
            },
        );
        if chart.kind() == PlotKind::Line && chart.description().markers_visible() {
            crate::data::source::reserve_source(&mut payload.marks, 1)?;
            let mut markers = Vec::new();
            for (point, source) in points.iter().zip(&group.points) {
                push_point(
                    &mut markers,
                    *point,
                    source.alpha.point(point_style.clone()),
                )?;
            }
            payload.marks.push(LineMark::Line(points, line.clone()));
            crate::data::source::reserve_source(&mut payload.marks, markers.len())?;
            payload.marks.extend(
                markers
                    .into_iter()
                    .map(|(points, style)| LineMark::Points(points, style)),
            );
        } else {
            crate::data::source::reserve_source(&mut payload.marks, 1)?;
            payload.marks.push(LineMark::Line(points, line.clone()));
        }
    }
    // Trajectories and paths paint every connector before any point.
    if chart.kind() != PlotKind::Line
        && (chart.kind() == PlotKind::Trajectory || chart.description().markers_visible())
    {
        for &index in &order {
            let group = &prepared.groups[index];
            for point in &group.points {
                let color = if hue_paths {
                    let index = group
                        .hue_index
                        .ok_or_else(|| Error::Render("line path: missing hue index".into()))?;
                    map.color(&prepared.hues[index])?
                } else if let Some(index) = point.hue_index {
                    map.color(&prepared.hues[index])?
                } else {
                    accent.color(0)?
                };
                crate::data::source::reserve_source(&mut payload.marks, 1)?;
                payload.marks.push(LineMark::Points(
                    crate::data::source::collect_source(std::iter::once(Ok(project(
                        point.x, point.y,
                    ))))?,
                    point.alpha.point(chart.description().point_style(
                        theme,
                        color,
                        theme.marks.get_points().get_radii().get_trajectory(),
                    )),
                ));
            }
        }
    }
    crate::data::source::reserve_source(&mut payload.layers, prepared.layers.len())?;
    let mut label_source: Option<[(f64, f64); 2]> = None;
    for layer in &prepared.layers {
        let index = layer.source_index;
        let source = &chart.description().point_layers()[index];
        let style = source.point_style(
            accent.color(prepared.primary_accent_slots + index)?,
            theme.marks.get_points().get_supplemental(),
        );
        let points = crate::data::source::collect_source(
            layer.points.iter().map(|&(x, y)| Ok(project(x, y))),
        )?;
        let mut text = Vec::new();
        let mut label_indices = Vec::new();
        if let Some(labels) = source.label_values() {
            let appearance =
                source.label_appearance(&theme.typography.annotations.point_layer_label)?;
            let label_layout = source.label_layout.as_ref();
            crate::data::source::reserve_source(&mut text, labels.len())?;
            crate::data::source::reserve_source(&mut label_indices, labels.len())?;
            let x_range = layout.projection.x_range();
            let y_range = layout.projection.y_range();
            let eligible = crate::data::source::collect_source(
                layer
                    .points
                    .iter()
                    .enumerate()
                    .filter(|(_, point)| {
                        point.0 >= x_range.0
                            && point.0 <= x_range.1
                            && point.1 >= y_range.0
                            && point.1 <= y_range.1
                    })
                    .map(|(index, _)| Ok(index)),
            )?;
            let offset = f64::from(style.radius)
                + theme
                    .annotations
                    .point_label_gap_length(
                        theme
                            .typography
                            .annotations
                            .point_layer_label
                            .get_font_size(),
                    )?
                    .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?;
            let labels = (0..labels.len()).map(|row| {
                labels
                    .get(row)
                    .flatten()
                    .expect("prepared point layer label")
            });
            for (source_index, ((&(x, y), point), label)) in
                layer.points.iter().zip(&points).zip(labels).enumerate()
            {
                if x < x_range.0 || x > x_range.1 || y < y_range.0 || y > y_range.1 {
                    continue;
                }

                let source = label_source.get_or_insert([(x, x), (y, y)]);
                source[0] = (source[0].0.min(x), source[0].1.max(x));
                source[1] = (source[1].0.min(y), source[1].1.max(y));
                let mut buffer = [0_u8; 80];
                let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
                std::io::Write::write_fmt(
                    &mut cursor,
                    format_args!("line point layer {index} label"),
                )
                .expect("bounded layer index");
                let length = usize::try_from(cursor.position()).expect("bounded layer index");
                let owner = std::str::from_utf8(&buffer[..length]).expect("ASCII layer index");
                let selected = crate::text::TextAppearance::from(&appearance).layout(label_layout);
                let plan = if label_layout
                    .and_then(|request| request.instructions().visibility.as_ref())
                    .is_some_and(|policy| !policy.includes(source_index, &eligible))
                {
                    crate::layout::ResolvedTextPlan::hidden_source(
                        label,
                        selected,
                        crate::layout::PhysicalTextAlignment::Start,
                        owner,
                    )?
                } else {
                    theme.measure_appearance_text(
                        label,
                        selected,
                        crate::layout::PhysicalTextAlignment::Start,
                        owner,
                    )?
                };
                let position = if label_layout
                    .is_some_and(|request| request.instructions().offset.is_some())
                {
                    *point
                } else {
                    Point2D::new((point.x + offset).round(), (point.y + offset).round())
                };
                text.push(PlacedText::middle(plan, position, TextAnchor::Start, 0.0)?);
                label_indices.push(source_index);
            }
            crate::layout::text::font_fit::uniform_placed(theme, &mut text, scope)?;
            crate::layout::text::visibility::apply(
                &mut text,
                &label_indices,
                layout.projection.panel_box(),
                false,
                scope,
            )?;
        }
        payload.layers.push((points, style, text));
    }
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let mut label_ink = crate::render::geometry::GeometrySnapshot::new();
    for mark in &payload.marks {
        match mark {
            LineMark::Ribbon(points, style) => geometry.polygon(points, style, Some(clip))?,
            LineMark::Line(points, style) => geometry.line(points, style, Some(clip))?,
            LineMark::Points(points, style) => geometry.scatter(points, style, clip)?,
        }
    }
    for (points, style, labels) in &payload.layers {
        geometry.scatter(points, style, clip)?;
        for text in labels {
            label_ink.text(text.placement, &text.plan, None)?;
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
    }
    if let (Some(source), Some(bounds)) = (
        label_source,
        label_ink
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union),
    ) {
        let automatic = [
            chart.user_x_limits().is_none() && !categorical_x,
            chart.user_y_limits().is_none(),
        ];
        let additional = crate::layout::interior::automatic_absolute_ink_expansions(
            source,
            &layout.projection,
            bounds,
            automatic,
            scope,
        )?;
        for (axis, (painted, containing)) in [
            ((bounds.left, bounds.right), (clip.x0, clip.x1)),
            ((bounds.bottom, bounds.top), (clip.y0, clip.y1)),
        ]
        .into_iter()
        .enumerate()
        {
            if (painted.0 < containing.0 || painted.1 > containing.1)
                && (!automatic[axis] || additional[axis].is_none())
            {
                return Err(scope
                    .failure(
                        crate::layout::fit::FitKind::OverlayCollision,
                        "required point-layer label ink exceeds its assigned plot panel",
                    )
                    .into());
            }
        }
        payload.domain_expansions = crate::layout::interior::merge_mark_domain_expansions(
            payload.domain_expansions,
            additional,
        );
    }
    let mut mark_ink = crate::render::geometry::GeometrySnapshot::new();

    for mark in &payload.marks {
        match mark {
            LineMark::Ribbon(points, style) => mark_ink.polygon(points, style, None)?,
            LineMark::Line(points, style) => mark_ink.line(points, style, None)?,
            LineMark::Points(points, style) => mark_ink.scatter_with_clip(points, style, None)?,
        }
    }
    for (points, style, _) in &payload.layers {
        mark_ink.scatter_with_clip(points, style, None)?;
    }
    if let Some(bounds) = mark_ink
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        let additional = crate::layout::interior::automatic_absolute_ink_expansions(
            prepared.mark_ranges,
            &layout.projection,
            bounds,
            [
                chart.user_x_limits().is_none() && !categorical_x,
                chart.user_y_limits().is_none(),
            ],
            scope,
        )?;
        payload.domain_expansions = crate::layout::interior::merge_mark_domain_expansions(
            payload.domain_expansions,
            additional,
        );
    }
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?)
        .with_axis_domains(
            if categorical_x {
                AxisDomain::Categorical
            } else {
                AxisDomain::Continuous
            },
            AxisDomain::Continuous,
        );
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Line(payload),
            geometry,
        },
    )?);
    Ok(result)
}

impl LinePayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            for mark in &self.marks {
                match mark {
                    LineMark::Ribbon(points, style) => backend.draw_polygon(points, style)?,
                    LineMark::Line(points, style) => backend.draw_line(points, style)?,
                    LineMark::Points(points, style) => backend.draw_scatter(points, style)?,
                }
            }
            Ok(())
        })?;
        if !self.layers.is_empty() {
            with_clip(backend, self.clip, |backend| {
                for (points, style, labels) in &self.layers {
                    backend.draw_scatter(points, style)?;
                    for text in labels {
                        text.draw(backend)?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}
