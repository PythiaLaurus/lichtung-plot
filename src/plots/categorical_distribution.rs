//! Root-owned categorical distribution semantics and selected display operands.

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::render::backend::{Backend, FillStyle, LineStyle, Point2D, PointStyle};
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
    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<LegendEntry>, Error> {
        match self {
            Self::Boxplot(prepared) => {
                data.collect_source(prepared.sample.hue_levels.iter().enumerate().map(
                    |(index, label)| {
                        let color = theme.colors[index % theme.colors.len()];
                        Ok(LegendEntry::line_fill(
                            chart.point_legend_identity(label, data)?,
                            theme.marks.summary_stroke.backend_style(),
                            theme.marks.box_body.backend_style(color),
                        ))
                    },
                ))
            }
            Self::Violin(prepared) => {
                let family = if chart.kind() == PlotKind::Violin {
                    "violin"
                } else {
                    "raincloud"
                };
                data.collect_source(prepared.sample.hue_levels.iter().enumerate().map(
                    |(index, label)| {
                        let color = chart.categorical_color(family, label, index, theme)?;
                        let identity = chart.point_legend_identity(label, data)?;
                        let fill = theme.marks.violin_body.backend_style(color);
                        Ok(if chart.kind() == PlotKind::Violin {
                            LegendEntry::fill(identity, fill)
                        } else {
                            LegendEntry::point_fill(
                                identity,
                                crate::plots::distribution_sample::overlay_style(color),
                                fill,
                            )
                        })
                    },
                ))
            }
            Self::Supplied(prepared) => prepared.guide_entries(chart, theme, data),
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
        data.claim_mark_rows()?;
        #[cfg(test)]
        count(chart.kind(), 0);
        match chart.kind() {
            PlotKind::Boxplot => Ok(Self::Boxplot(crate::plots::boxplot::prepare(
                chart, data, facet,
            )?)),
            PlotKind::Raincloud if chart.raincloud_geometry_input().is_some() => Ok(
                Self::Supplied(crate::plots::violin::precomputed::prepare(chart, data)?),
            ),
            PlotKind::Violin | PlotKind::Raincloud => Ok(Self::Violin(
                crate::plots::violin::prepare(chart, data, facet)?,
            )),
            _ => Err(Error::Render(
                "categorical distribution preparation requires its family".into(),
            )),
        }
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
enum Mark {
    Polygon(Vec<Point2D>, FillStyle),
    Line(Vec<Point2D>, LineStyle),
    Scatter(Vec<Point2D>, PointStyle),
}

#[derive(Debug)]
pub(crate) struct DistributionPayload {
    clip: crate::layout::PanelBox,
    marks: Vec<Mark>,
    #[cfg(test)]
    kind: PlotKind,
}

impl DistributionPayload {
    pub(crate) fn new(clip: crate::layout::PanelBox, kind: PlotKind) -> Self {
        let _ = kind;
        Self {
            clip,
            marks: Vec::new(),
            #[cfg(test)]
            kind,
        }
    }

    pub(crate) fn polygon(
        &mut self,
        points: Vec<Point2D>,
        style: FillStyle,
        data: PlotData<'_>,
    ) -> Result<(), Error> {
        data.reserve_source(&mut self.marks, 1)?;
        self.marks.push(Mark::Polygon(points, style));
        Ok(())
    }

    pub(crate) fn line(
        &mut self,
        points: &[Point2D],
        style: &LineStyle,
        data: PlotData<'_>,
    ) -> Result<(), Error> {
        let points = data.collect_source(points.iter().copied().map(Ok))?;
        data.reserve_source(&mut self.marks, 1)?;
        self.marks.push(Mark::Line(points, style.clone()));
        Ok(())
    }

    pub(crate) fn scatter(
        &mut self,
        points: Vec<Point2D>,
        style: PointStyle,
        data: PlotData<'_>,
    ) -> Result<(), Error> {
        data.reserve_source(&mut self.marks, 1)?;
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
        self,
        mut result: crate::spec::PlotRenderResult,
        context: crate::spec::PlotRenderContext<'_>,
    ) -> Result<crate::spec::PlotRenderResult, Error> {
        let resources = context
            .theme
            .prepared_font_environment()?
            .generation_resources();
        let data = context.data.with_axis_resources(&resources);
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(context.theme)?;
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        for mark in &self.marks {
            match mark {
                Mark::Polygon(points, style) => geometry.polygon(points, style, Some(clip))?,
                Mark::Line(points, style) => geometry.line(points, style, Some(clip))?,
                Mark::Scatter(points, style) => geometry.scatter(points, style, clip)?,
            }
        }
        #[cfg(test)]
        count(self.kind, 2);
        result.x_domain = crate::spec::AxisDomain::Categorical;
        result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
            payload: crate::layout::interior::MarkPayload::Distribution(self),
            geometry,
        })?);
        Ok(result)
    }
}

#[cfg(test)]
mod appearance_tests {
    use super::Mark;
    use crate::{CategoryValues, NumericValues, Plot};

    #[test]
    fn custom_violin_body_preserves_selected_geometry_and_changes_exact_guide() {
        let source = Plot::violin(
            CategoryValues::new("category", ["A"; 12]).unwrap(),
            NumericValues::new(
                "value",
                [1.0, 1.3, 1.6, 2.0, 2.3, 2.6, 3.0, 3.3, 3.6, 4.0, 4.3, 4.6],
            ),
        )
        .series(
            CategoryValues::new(
                "series",
                ["a", "a", "a", "a", "a", "a", "b", "b", "b", "b", "b", "b"],
            )
            .unwrap(),
        )
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let panel = crate::layout::PanelBox::new(10.0, 620.0, 20.0, 420.0);
        let mut theme = crate::Theme::Publication.resolve();
        let old = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                panel,
                &theme,
            ))
            .unwrap();
        theme.marks.violin_body.fill_alpha = 79;
        theme.marks.violin_body.outline_width = 2.0;
        let changed = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                panel,
                &theme,
            ))
            .unwrap();
        let extract = |result: &crate::spec::PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Distribution(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("selected violin")
            };
            let Some(Mark::Polygon(points, fill)) = payload.marks.first() else {
                panic!("selected violin body")
            };
            (points.clone(), fill.clone())
        };
        let (old_polygon, old_style) = extract(&old);
        let (new_polygon, new_style) = extract(&changed);
        assert_eq!(old_polygon, new_polygon);
        assert_eq!(old_style.color.alpha(), 178);
        assert_eq!(new_style.color.alpha(), 79);
        assert_eq!(new_style.stroke_width.to_bits(), 2.0_f32.to_bits());
        assert!(crate::decorations::legend::merge_exact_entries(
            [
                old.legend_entries[0].clone(),
                changed.legend_entries[0].clone()
            ],
            "custom violin body",
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
    }
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
