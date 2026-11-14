//! Root-owned categorical distribution semantics and selected display operands.

use crate::data::PlotData;
use crate::decorations::legend::source::{FillRole, LineRole, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::orchestrator::registration::{CategoricalBinding, CategoricalDomainSource};
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
    pub(crate) fn positioned_groups(&self) -> Option<&[String]> {
        match self {
            Self::Boxplot(marks) => Some(&marks.sample.group_levels),
            Self::Violin(marks) => Some(&marks.sample.group_levels),
            Self::Supplied(_) => None,
        }
    }
    pub(crate) fn series_center(&self, category: &str, series: &str) -> Result<f64, Error> {
        let sample = match self {
            Self::Boxplot(marks) => &marks.sample,
            Self::Violin(marks) => &marks.sample,
            Self::Supplied(_) => {
                return Err(Error::Data(
                    "supplied Raincloud has no positioned series domain".into(),
                ))
            }
        };
        let group = sample
            .groups
            .iter()
            .find(|group| group.label == category && group.group_label.as_deref() == Some(series))
            .ok_or_else(|| {
                Error::Data(
                    "annotation category-series pair is absent from the actual distribution".into(),
                )
            })?;
        Ok(crate::plots::distribution_sample::center(
            group,
            sample.group_levels.len(),
        ))
    }
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        match self {
            Self::Boxplot(marks) => marks.alpha_guide(chart, theme, guide),
            Self::Violin(marks) => marks.alpha_guide(chart, theme, guide),
            Self::Supplied(marks) => marks.alpha_guide(chart, theme, guide),
        }
    }

    pub(crate) fn resolved_colors(
        &self,
        chart: &Plot,
        axes: &crate::orchestrator::registration::PreparedAxesOwner,
        binding: &CategoricalBinding,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: PlotData<'_, '_>,
    ) -> Result<CategoryColors, Error> {
        let resources = data.source_resources();
        if binding.local_len() != 0 {
            return binding.resolve(binding.values(), theme, resources);
        }
        if matches!(
            binding.local,
            CategoricalDomainSource::DistributionFacetCategories
        ) && chart.description().has_facets()
            && !matches!(self, Self::Supplied(_))
        {
            // A category keyed by its position must not be recolored when an
            // earlier observed category is absent from a particular facet.
            let column = chart.description().x_col().ok_or_else(|| {
                Error::Render("distribution category color source is missing".into())
            })?;
            let source = chart
                .description()
                .data_column(column)
                .ok_or_else(|| Error::Data(format!("unknown semantic value {column:?}")))?;
            let crate::data::DataColumn::Category(values) = source else {
                return Err(Error::Data(format!(
                    "distribution color domain: value {:?} must be categorical",
                    source.name()
                )));
            };
            let raw =
                crate::data::strict_source_strings(values, None, "distribution color domain")?;
            let parent = crate::data::source::source_levels(&raw)?;
            return binding.resolve(parent.iter().map(String::as_str), theme, resources);
        }
        let keys = binding.axis_categories(axes).unwrap_or_else(|| {
            &axes
                .categories(0)
                .expect("prepared distribution categories")
                .raw
        });
        binding.resolve(keys.iter().map(String::as_str), theme, resources)
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
        axes: &crate::orchestrator::registration::PreparedAxesOwner,
        binding: &CategoricalBinding,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: PlotData<'_, '_>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let colors = self.resolved_colors(chart, axes, binding, theme, data)?;
        self.guide_entries_with_colors(chart, binding, theme, &colors)
    }

    pub(crate) fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        binding: &CategoricalBinding,
        theme: &crate::text::TextRequestContext<'_, '_>,

        colors: &CategoryColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (sample, boxplot) = match self {
            Self::Boxplot(prepared) => (&prepared.sample, true),
            Self::Violin(prepared) => (&prepared.sample, false),
            Self::Supplied(_) => {
                return crate::plots::violin::precomputed::guide_entries(
                    chart, binding, theme, colors,
                )
            }
        };
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, binding.local_len())?;
        for label in sample.observed_hue_levels(binding.local_values()) {
            let color = colors.color(label)?;
            let key = GuideSourceKey::hue_only(chart.description().point_legend_identity(label)?)?;
            let key = if boxplot {
                key.with_role_line(
                    theme
                        .marks
                        .get_strokes()
                        .get_summaries()
                        .get_summary()
                        .backend_style(),
                    LineRole::SummaryStroke,
                )
                .with_fill(
                    theme.marks.get_fills().get_bodies().get_box_body(),
                    color,
                    FillRole::BoxBody,
                )
            } else {
                let key = key.with_fill(
                    theme.marks.get_fills().get_bodies().get_violin_body(),
                    color,
                    FillRole::ViolinBody,
                );
                if chart.kind() == PlotKind::Violin {
                    key
                } else {
                    key.with_point(
                        chart.description().distribution_point_style(
                            theme.marks.get_points().get_mandatory_strip(),
                            color,
                        ),
                        PointRole::MandatoryStrip,
                    )
                }
            };
            entries.push(key);
        }
        Ok(entries)
    }

    pub(crate) fn observed_hue_levels<'a>(
        &'a self,
        hues: impl Iterator<Item = &'a str> + 'a,
    ) -> impl Iterator<Item = &'a str> {
        hues.filter(|hue| match self {
            Self::Boxplot(prepared) => prepared.sample.observes_hue(hue),
            Self::Violin(prepared) => prepared.sample.observes_hue(hue),
            Self::Supplied(_) => true,
        })
    }
    pub(crate) fn new(
        chart: &crate::builder::PlotDescription,
        data: PlotData<'_, '_>,
        facet: Option<crate::data::FacetAxisSources<'_>>,
    ) -> Result<(Self, Vec<String>, Vec<String>), Error> {
        data.validate_row_source()?;
        let (prepared, categories, hues) = match chart.kind() {
            PlotKind::Boxplot => {
                let (marks, categories, hues) = crate::plots::boxplot::prepare(chart, data, facet)?;
                Ok((Self::Boxplot(marks), categories, hues))
            }
            PlotKind::Raincloud if chart.raincloud_geometry_input().is_some() => {
                let (marks, categories, hues) = crate::plots::violin::precomputed::prepare(chart)?;
                Ok((Self::Supplied(marks), categories, hues))
            }
            PlotKind::Violin | PlotKind::Raincloud => {
                let (marks, categories, hues) = crate::plots::violin::prepare(chart, data, facet)?;
                Ok((Self::Violin(marks), categories, hues))
            }
            _ => Err(Error::Render(
                "categorical distribution preparation requires its family".into(),
            )),
        }?;
        if let Some(exact) = chart.categorical_map_spec() {
            let keys = if hues.is_empty() { &categories } else { &hues };
            for key in keys {
                if exact.color_in(key).is_none() {
                    return Err(Error::Data(format!(
                        "distribution: category {key:?} has no caller-owned color"
                    )));
                }
            }
        }
        Ok((prepared, categories, hues))
    }

    pub(crate) const fn range(&self) -> (f64, f64) {
        match self {
            Self::Boxplot(marks) => marks.range,
            Self::Violin(marks) => marks.support_range,
            Self::Supplied(marks) => marks.range,
        }
    }
}

pub(crate) fn retained(
    family: Option<crate::orchestrator::registration::FamilyRef<'_>>,
) -> Result<&PreparedDistribution, Error> {
    family
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Distribution { marks, .. } => Some(marks),
            _ => None,
        })
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
    clip: crate::layout::PixelRect,
    pub(crate) marks: Vec<Mark>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl DistributionPayload {
    pub(crate) fn new(clip: crate::layout::PixelRect) -> Self {
        Self {
            clip,
            marks: Vec::new(),
            domain_expansions: [None; 2],
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

    pub(crate) fn scatter(
        &mut self,
        points: Vec<Point2D>,
        style: &PointStyle,
        alphas: impl ExactSizeIterator<Item = crate::alpha::marks::ObservationAlpha>,
    ) -> Result<(), Error> {
        crate::alpha::marks::require_aligned_points(points.len(), alphas.len())?;
        let mut batches = Vec::new();
        for (point, alpha) in points.into_iter().zip(alphas) {
            crate::alpha::marks::push_point(&mut batches, point, alpha.point(style.clone()))?;
        }
        crate::data::source::reserve_source(&mut self.marks, batches.len())?;
        self.marks.extend(
            batches
                .into_iter()
                .map(|(points, style)| Mark::Scatter(points, style)),
        );
        Ok(())
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
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
        context: crate::spec::PlotRenderContext<'_, '_>,
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
                let categories = categorical_distribution_count(
                    context.prepared_family(),
                    &context
                        .registered
                        .expect("prepared distribution occurrence")
                        .axes
                        .categories(0)
                        .expect("prepared distribution categories")
                        .raw,
                )?;
                self.domain_expansions =
                    crate::layout::interior::automatic_absolute_ink_expansions(
                        [
                            (0.0, f64::from(categories)),
                            retained(context.prepared_family())?.range(),
                        ],
                        &result.layout.projection,
                        bounds,
                        [false, true],
                        context.fit_scope(),
                    )?;
            }
        }
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

fn categorical_distribution_count(
    family: Option<crate::orchestrator::registration::FamilyRef<'_>>,
    categories: &[String],
) -> Result<u32, Error> {
    retained(family)?;
    u32::try_from(categories.len()).map_err(|_| {
        Error::Render("categorical distribution count exceeds coordinate range".into())
    })
}
