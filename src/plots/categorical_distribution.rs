//! Root-owned categorical distribution semantics and selected display operands.

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::render::backend::{Backend, FillStyle, LineStyle, Point2D, PointStyle};
use crate::theme::CategoryColors;
use crate::{Error, Plot, PlotKind};

#[cfg(test)]
pub(crate) mod extraction;

#[derive(Debug)]
pub(crate) enum PreparedDistribution {
    Boxplot(crate::plots::boxplot::PreparedBoxplot),
    Violin(crate::plots::violin::PreparedViolin),
    Supplied(crate::plots::violin::precomputed::PreparedRaincloud),
}

impl PreparedDistribution {
    pub(crate) fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<CategoryColors, Error> {
        let resources = data.source_resources()?;
        let keys = if !self.hue_levels().is_empty() {
            chart.global_hue_levels().unwrap_or(self.hue_levels())
        } else if !chart.has_facets() || matches!(self, Self::Supplied(_)) {
            self.categories()
        } else {
            // A category keyed by its position must not be recolored when an
            // earlier observed category is absent from a particular facet.
            let source = chart.data().with_operation(resources);
            let column = chart.x_col().ok_or_else(|| {
                Error::Render("distribution category color source is missing".into())
            })?;
            let raw = crate::data::strict_strings(source, column, "distribution color domain")?;
            let parent = crate::data::source::source_levels(&raw)?;
            return CategoryColors::resolve(
                &parent,
                theme.color_scales.qualitative(),
                chart.categorical_color_scale_spec(),
                resources,
            );
        };
        CategoryColors::resolve(
            keys,
            theme.color_scales.qualitative(),
            chart.categorical_color_scale_spec(),
            resources,
        )
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        match self {
            Self::Boxplot(prepared) => prepared.sample.groups.len() as u128,
            Self::Violin(prepared) => prepared.sample.groups.len() as u128,
            Self::Supplied(prepared) => prepared.primary_mark_count(),
        }
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<LegendEntry>, Error> {
        let colors = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, &colors)
    }

    pub(crate) fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,

        colors: &CategoryColors,
    ) -> Result<Vec<LegendEntry>, Error> {
        match self {
            Self::Boxplot(prepared) => crate::data::source::collect_source(
                prepared.sample.hue_levels.iter().map(|label| {
                    let color = colors.color(label)?;
                    Ok(LegendEntry::line_fill(
                        chart.point_legend_identity(label)?,
                        theme.marks.strokes().summaries().summary().backend_style(),
                        theme.marks.fills().bodies().box_body().backend_style(color),
                    ))
                }),
            ),
            Self::Violin(prepared) => crate::data::source::collect_source(
                prepared.sample.hue_levels.iter().map(|label| {
                    let color = colors.color(label)?;
                    let identity = chart.point_legend_identity(label)?;
                    let fill = theme
                        .marks
                        .fills()
                        .bodies()
                        .violin_body()
                        .backend_style(color);
                    Ok(if chart.kind() == PlotKind::Violin {
                        LegendEntry::fill(identity, fill)
                    } else {
                        LegendEntry::point_fill(
                            identity,
                            chart.distribution_point_style(
                                theme.marks.points().mandatory_strip(),
                                color,
                            ),
                            fill,
                        )
                    })
                }),
            ),
            Self::Supplied(prepared) => prepared.guide_entries(chart, theme, colors),
        }
    }

    pub(crate) fn hue_levels(&self) -> &[String] {
        match self {
            Self::Boxplot(prepared) => &prepared.sample.hue_levels,
            Self::Violin(prepared) => &prepared.sample.hue_levels,
            Self::Supplied(prepared) => prepared.hue_levels(),
        }
    }
    pub(crate) fn new(
        chart: &Plot,
        data: PlotData<'_>,
        facet: Option<crate::data::FacetAxisSources<'_>>,
    ) -> Result<Self, Error> {
        data.validate_row_source()?;
        #[cfg(test)]
        count(chart.kind(), 0);
        let prepared = match chart.kind() {
            PlotKind::Boxplot => Ok(Self::Boxplot(crate::plots::boxplot::prepare(
                chart, data, facet,
            )?)),
            PlotKind::Raincloud if chart.raincloud_geometry_input().is_some() => Ok(
                Self::Supplied(crate::plots::violin::precomputed::prepare(chart)?),
            ),
            PlotKind::Violin | PlotKind::Raincloud => Ok(Self::Violin(
                crate::plots::violin::prepare(chart, data, facet)?,
            )),
            _ => Err(Error::Render(
                "categorical distribution preparation requires its family".into(),
            )),
        }?;
        if let Some(exact) = chart.categorical_map_spec() {
            let keys = if prepared.hue_levels().is_empty() {
                prepared.categories()
            } else {
                prepared.hue_levels()
            };
            for key in keys {
                if exact.color_in(key).is_none() {
                    return Err(Error::Data(format!(
                        "distribution: category {key:?} has no caller-owned color"
                    )));
                }
            }
        }
        Ok(prepared)
    }

    pub(crate) fn categories(&self) -> &[String] {
        match self {
            Self::Boxplot(marks) => &marks.sample.x_levels,
            Self::Violin(marks) => &marks.sample.x_levels,
            Self::Supplied(marks) => &marks.categories,
        }
    }

    pub(crate) const fn range(&self) -> (f64, f64) {
        match self {
            Self::Boxplot(marks) => marks.range,
            Self::Violin(marks) => marks.support_range,
            Self::Supplied(marks) => marks.range,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PreparedDistributionInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedDistribution)>,
}

pub(crate) fn retained(chart: &Plot) -> Result<&PreparedDistribution, Error> {
    chart
        .prepared_distributions
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render(
                "categorical distribution measurement requires root-prepared semantics".into(),
            )
        })
}

#[derive(Debug)]
pub(crate) enum Mark {
    Polygon(Vec<Point2D>, FillStyle),
    Line(Vec<Point2D>, LineStyle),
    Scatter(Vec<Point2D>, PointStyle),
}

#[derive(Debug)]
pub(crate) struct DistributionPayload {
    clip: crate::layout::PanelBox,
    pub(crate) marks: Vec<Mark>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
    #[cfg(test)]
    kind: PlotKind,
}

impl DistributionPayload {
    pub(crate) fn new(clip: crate::layout::PanelBox, kind: PlotKind) -> Self {
        let _ = kind;
        Self {
            clip,
            marks: Vec::new(),
            domain_expansions: [None; 2],
            #[cfg(test)]
            kind,
        }
    }

    pub(crate) fn polygon(&mut self, points: Vec<Point2D>, style: FillStyle) -> Result<(), Error> {
        crate::data::source::reserve_source(&mut self.marks, 1)?;
        self.marks.push(Mark::Polygon(points, style));
        Ok(())
    }

    pub(crate) fn line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        let points = crate::data::source::collect_source(points.iter().copied().map(Ok))?;
        crate::data::source::reserve_source(&mut self.marks, 1)?;
        self.marks.push(Mark::Line(points, style.clone()));
        Ok(())
    }

    pub(crate) fn scatter(&mut self, points: Vec<Point2D>, style: PointStyle) -> Result<(), Error> {
        crate::data::source::reserve_source(&mut self.marks, 1)?;
        self.marks.push(Mark::Scatter(points, style));
        Ok(())
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(self.kind, 3);
        crate::render::backend::with_clip(backend, self.clip, |backend| {
            for mark in &self.marks {
                match mark {
                    Mark::Polygon(points, style) => backend.draw_polygon(points, style)?,
                    Mark::Line(points, style) => backend.draw_line(points, style)?,
                    Mark::Scatter(points, style) => backend.draw_scatter(points, style)?,
                }
            }
            Ok(())
        })
    }

    pub(crate) fn retain(
        mut self,
        mut result: crate::spec::PlotRenderResult,
        context: crate::spec::PlotRenderContext<'_>,
        chart: &Plot,
    ) -> Result<crate::spec::PlotRenderResult, crate::layout::fit::PlanError> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        for mark in &self.marks {
            match mark {
                Mark::Polygon(points, style) => geometry.polygon(points, style, Some(clip))?,
                Mark::Line(points, style) => geometry.line(points, style, Some(clip))?,
                Mark::Scatter(points, style) => geometry.scatter(points, style, clip)?,
            }
        }
        let mut full = crate::render::geometry::GeometrySnapshot::new();

        for mark in &self.marks {
            match mark {
                Mark::Polygon(points, style) => full.polygon(points, style, None)?,
                Mark::Line(points, style) => full.line(points, style, None)?,
                Mark::Scatter(points, style) => full.scatter_with_clip(points, style, None)?,
            }
        }
        if let Some(bounds) = full
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union)
        {
            if bounds.left < clip.x0 || bounds.right > clip.x1 {
                return Err(context
                    .fit_scope()
                    .failure(
                        crate::layout::fit::FitKind::SpecializedGeometry,
                        "categorical distribution painted ink exceeds its assigned category panel",
                    )
                    .into());
            }
            if chart.user_y_limits().is_none() {
                let categories = categorical_distribution_count(chart)?;
                self.domain_expansions =
                    crate::layout::interior::automatic_absolute_ink_expansions(
                        [(0.0, f64::from(categories)), retained(chart)?.range()],
                        &result.layout.projection,
                        bounds,
                        [false, true],
                        context.fit_scope(),
                    )?;
            }
        }
        #[cfg(test)]
        count(self.kind, 2);
        result.x_domain = crate::spec::AxisDomain::Categorical;
        result.marks = Some(crate::data::source::source_arc(
            crate::layout::interior::SelectedMarkPlan {
                payload: crate::layout::interior::MarkPayload::Distribution(self),
                geometry,
            },
        )?);
        Ok(result)
    }
}

fn categorical_distribution_count(chart: &Plot) -> Result<u32, Error> {
    u32::try_from(retained(chart)?.categories().len()).map_err(|_| {
        Error::Render("categorical distribution count exceeds coordinate range".into())
    })
}

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 3]> = const { std::cell::Cell::new([[0; 4]; 3]) };
}

#[cfg(test)]
pub(crate) fn count(kind: PlotKind, stage: usize) {
    let family = match kind {
        PlotKind::Boxplot => 0,
        PlotKind::Violin => 1,
        PlotKind::Raincloud => 2,
        _ => unreachable!(),
    };
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}
