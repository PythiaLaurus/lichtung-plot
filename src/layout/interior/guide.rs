//! Discrete entries from one selected prepared semantic owner, without a panel
//! assignment, font environment, or backend measurement.

use super::PreparedInteriorInputs;
use crate::data::PlotData;
use crate::decorations::legend::discrete::{
    PreparedDiscreteGuide, PreparedDiscreteGuides, ScopedDiscreteGuide,
};
use crate::decorations::legend::GuideSourceKey;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

/// One measured mandatory right guide beside the flexible scientific body.
pub(crate) fn continuous_right(
    descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    panel: crate::layout::PanelBox,
    minimum: f64,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,

    content_plot: Option<&Plot>,
) -> Result<
    (
        crate::layout::PanelBox,
        crate::decorations::legend::MeasuredContinuousGuide,
    ),
    crate::layout::fit::PlanError,
> {
    use crate::layout::fit::{self, FitKind};
    use crate::layout::kernel::{allocate_grid_axis, GridRequirement, TrackSpec};
    let widths =
        crate::decorations::legend::MeasuredContinuousGuide::vertical_widths(descriptor, theme)?;
    let available = panel.width() - minimum - widths.outside_gap;
    if widths.title_minimum > available {
        return Err(scope
            .child(crate::layout::AddressRole::GuideTitle, 0)?
            .failure(
                FitKind::Text,
                "continuous guide title cannot preserve its text and the hard data tracks",
            )
            .into());
    }
    let width = widths.preferred.min(available);
    if width < widths.minimum {
        return Err(scope
            .failure(
                FitKind::LegendOrGuide,
                "continuous guide cannot preserve its values and the hard data tracks",
            )
            .into());
    }
    let guide = fit::measured(
        crate::decorations::legend::MeasuredContinuousGuide::measure(
            descriptor,
            (width, content_plot.is_none().then_some(panel.height())),
            false,
            false,
            theme,
            &mut false,
            scope,
        ),
    )?;
    let hard = crate::layout::kernel::ceil_spatial_quanta(minimum)?;
    let width = crate::layout::kernel::ceil_spatial_quanta(guide.width)?;
    let tracks = [
        TrackSpec {
            min: hard,
            preferred: hard,
            max: None,
            flexible: true,
        },
        TrackSpec {
            min: width,
            preferred: width,
            max: Some(width),
            flexible: false,
        },
    ];
    let children: [GridRequirement<'_>; 0] = [];
    let start = crate::layout::kernel::nearest_spatial_quanta(panel.x0)?;
    let end = crate::layout::kernel::nearest_spatial_quanta(panel.x1)?;
    let span = end
        .checked_sub(start)
        .ok_or_else(|| Error::Render("continuous guide containing span overflow".into()))?;
    let assigned = fit::measured(allocate_grid_axis(
        &tracks,
        &children,
        span,
        [0, 0],
        crate::layout::kernel::ceil_spatial_quanta(widths.outside_gap)?,
        None,
        true,
        scope,
    ))?;
    let edge = start
        .checked_add(assigned.sizes[0])
        .ok_or_else(|| Error::Render("continuous guide body edge overflow".into()))?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "root-bounded spatial quanta convert exactly to CSS pixels"
    )]
    let body_right = edge as f64 / 1024.0;
    let top = if let Some(plot) = content_plot {
        let hard = plot.intrinsic_body_minimum(theme)?;
        let preference = crate::layout::interior::sizing::body_preference(plot, theme, hard)?;
        panel.y0
            + plot
                .natural_body_height(theme, body_right - panel.x0, preference.1)?
                .max(guide.height)
    } else {
        panel.y1
    };
    Ok((
        crate::layout::PanelBox::new(panel.x0, body_right, panel.y0, top),
        guide,
    ))
}

fn panel<'a, T>(
    panels: Option<&'a [(Option<String>, T)]>,
    scope: Option<&str>,
    kind: PlotKind,
) -> Result<&'a T, Error> {
    panels
        .and_then(|panels| panels.iter().find(|(owner, _)| owner.as_deref() == scope))
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render(format!(
                "{kind:?} guide is missing prepared semantics for {scope:?}"
            ))
        })
}

impl PreparedInteriorInputs {
    /// Scientific color membership is independent of guide visibility, display
    /// aliases and eventual section projection. No body probe participates.
    pub(crate) fn observed_color_levels(&self, chart: &Plot) -> Result<Vec<String>, Error> {
        self.route.validate(chart)?;
        if chart.categorical_color_field().is_none() {
            return Ok(Vec::new());
        }
        let mut levels = Vec::new();
        let mut include = |value: &str| -> Result<(), Error> {
            if !levels.iter().any(|existing| existing == value) {
                crate::data::source::reserve_source(&mut levels, 1)?;
                levels.push(crate::data::source::copy_source_string(value)?);
            }
            Ok(())
        };
        if let Some(inputs) = &self.scatter {
            for (_, marks) in &inputs.panels {
                for value in marks.hue_levels() {
                    include(value)?;
                }
            }
        }
        if let Some(inputs) = &self.line {
            for (_, marks) in &inputs.panels {
                for value in marks.hue_levels() {
                    include(value)?;
                }
            }
        }
        if let Some(inputs) = &self.bars {
            for (_, marks) in &inputs.panels {
                for value in marks.observed_hue_levels() {
                    include(value)?;
                }
            }
        }
        if let Some(inputs) = &self.distributions {
            for (_, marks) in &inputs.panels {
                for value in marks.observed_hue_levels() {
                    include(value)?;
                }
            }
        }
        if let Some(inputs) = &self.density {
            for (_, marks) in &inputs.panels {
                for value in marks.hue_levels() {
                    include(value)?;
                }
            }
        }
        if let Some(inputs) = &self.interaction {
            for (_, marks) in &inputs.panels {
                for value in marks.color_keys() {
                    include(value)?;
                }
            }
        }
        if let Some(marks) = &self.marginal {
            for value in marks.hue_levels() {
                include(value)?;
            }
        }
        if let Some(marks) = &self.matrix {
            for value in marks.hue_levels() {
                include(value)?;
            }
        }
        if levels.is_empty() {
            return Err(Error::Render(
                "categorical color domain has no prepared observed source".into(),
            ));
        }
        Ok(levels)
    }

    /// Capacity is a semantic requirement even when no legend will be shown.
    /// Facets share the union of their prepared raw identities, not a per-cell
    /// supply that would silently recolor or fail after fonts have been prepared.
    pub(crate) fn validate_categorical_scale(
        &self,
        chart: &Plot,
        theme: &crate::theme::Theme,
        global_hues: Option<&[String]>,
    ) -> Result<(), Error> {
        let global_hues = global_hues.or_else(|| chart.global_hue_levels());
        let palette = if let Some(scale) = chart.categorical_color_scale_spec() {
            let Some(palette) = scale.palette() else {
                if !chart.colors_by_sign() {
                    for key in global_hues.into_iter().flatten() {
                        if scale
                            .map()
                            .expect("validated map selection")
                            .color(key)
                            .is_none()
                        {
                            return Err(Error::Data(format!(
                                "categorical color map has no color for used raw key {key:?}"
                            )));
                        }
                    }
                }
                return Ok(());
            };
            palette
        } else {
            theme.color_scales().qualitative()
        };
        if palette.capacity() != crate::theme::PaletteCapacity::Fixed || chart.colors_by_sign() {
            return Ok(());
        }
        let mut keys = Vec::new();
        if let Some(global) = global_hues {
            include_keys(&mut keys, global.iter().map(String::as_str))?;
        }
        if let Some(inputs) = &self.scatter {
            for (_, marks) in &inputs.panels {
                include_keys(&mut keys, marks.hue_levels())?;
            }
        }
        if let Some(inputs) = &self.line {
            for (_, marks) in &inputs.panels {
                include_keys(&mut keys, marks.hue_levels())?;
            }
        }
        if let Some(inputs) = &self.density {
            for (_, marks) in &inputs.panels {
                include_keys(&mut keys, marks.hue_levels())?;
            }
        }
        if let Some(inputs) = &self.bars {
            for (_, marks) in &inputs.panels {
                include_keys(&mut keys, marks.color_keys().iter().map(String::as_str))?;
            }
        }
        if let Some(inputs) = &self.distributions {
            for (_, marks) in &inputs.panels {
                let domain = if marks.hue_levels().is_empty() {
                    marks.categories()
                } else {
                    marks.hue_levels()
                };
                include_keys(&mut keys, domain.iter().map(String::as_str))?;
            }
        }
        if let Some(inputs) = &self.interaction {
            for (_, marks) in &inputs.panels {
                include_keys(&mut keys, marks.color_keys())?;
            }
        }
        if let Some(marks) = &self.marginal {
            include_keys(&mut keys, marks.hue_levels())?;
        }
        if let Some(marks) = &self.matrix {
            include_keys(&mut keys, marks.hue_levels().iter().map(String::as_str))?;
        }
        palette.validate_capacity(keys.len())
    }

    pub(crate) fn guide_sources(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: PlotData<'_>,
        scope: Option<&str>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        self.route.validate(chart)?;
        let kind = chart.kind();
        match kind {
            PlotKind::Scatter => {
                let marks = panel(
                    self.scatter.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                let marks = panel(self.line.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let marks = panel(self.bars.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                let (colors, accents) = marks.resolved_colors(chart, theme, data)?;
                crate::plots::bar_data::guide_sources(marks, chart, theme, &colors, &accents)
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                let marks = panel(
                    self.distributions.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                marks.guide_entries(chart, theme, data)
            }
            PlotKind::Density => {
                let marks = panel(
                    self.density.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Interaction => {
                let marks = panel(
                    self.interaction.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                marks.guide_entries(chart, theme, data)
            }
            PlotKind::Marginal => self
                .marginal
                .as_ref()
                .ok_or_else(|| {
                    Error::Render("Marginal guide is missing prepared semantics".into())
                })?
                .guide_entries(chart, theme, data),
            PlotKind::Regression => {
                panel(
                    self.regression.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let color =
                    crate::theme::AccentColors::resolve(theme.marks.accents(), 1)?.color(0)?;
                crate::plots::regression::PreparedRegression::guide_entries(chart, theme, color)
            }
            PlotKind::Dumbbell => {
                panel(
                    self.dumbbell.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                crate::plots::dumbbell::DumbbellData::guide_entries(chart, theme)
            }
            PlotKind::Histogram
            | PlotKind::Qq
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Waterfall => Ok(Vec::new()),
            PlotKind::Heatmap | PlotKind::Beeswarm => Err(Error::Render(
                "continuous guides cannot be prepared as discrete entries".into(),
            )),
            PlotKind::Matrix => Err(Error::Render(
                "self-managed Matrix has no parent-owned guide entry".into(),
            )),
        }
    }

    pub(crate) fn prepare_discrete_guides(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        resources: &crate::font::canonical::OperationContext,
        owner: &crate::layout::ComponentAddress,
        facet_cells: bool,
    ) -> Result<(), Error> {
        if self.discrete_guides.get().is_some() {
            return Err(Error::Render(
                "discrete guides were prepared more than once".into(),
            ));
        }
        let data = chart.data().with_operation(resources);
        let mut components = crate::data::source::collect_source(
            self.axes.iter().map(|(scope, _)| Ok(scope.as_deref())),
        )?;
        if self.matrix.is_some() && components.is_empty() {
            crate::data::source::reserve_source(&mut components, 1)?;
            components.push(chart.axis_scope());
        }
        let mut panels = Vec::new();
        crate::data::source::reserve_source(&mut panels, components.len())?;
        for (index, scope) in components.into_iter().enumerate() {
            let scoped = if let Some(scope) = scope {
                chart
                    .clone()
                    .with_render_scope(crate::data::source::copy_source_string(scope)?)
            } else {
                chart.clone()
            };
            let contributor = if facet_cells {
                owner.child(
                    crate::layout::AddressRole::FacetCell,
                    u32::try_from(index)
                        .map_err(|_| Error::Render("facet guide source address overflow".into()))?,
                )?
            } else {
                owner.clone()
            };
            let descriptor = if chart.legend_policy_spec() == crate::LegendPolicy::Hidden
                || matches!(chart.kind(), PlotKind::Heatmap | PlotKind::Beeswarm)
            {
                None
            } else {
                let keys = if chart.kind() == PlotKind::Matrix {
                    self.matrix
                        .as_ref()
                        .ok_or_else(|| Error::Render("Matrix guide preparation missing".into()))?
                        .guide_sources(&scoped, theme, data)?
                } else {
                    self.guide_sources(&scoped, theme, data, scope)?
                };
                PreparedDiscreteGuide::prepare(
                    &scoped,
                    theme,
                    &contributor,
                    keys,
                    scoped.effective_legend_composition(),
                )?
            };
            panels.push(ScopedDiscreteGuide {
                scope: scope
                    .map(crate::data::source::copy_source_string)
                    .transpose()?,
                descriptor,
            });
        }
        let order = parent_guide_order(chart, &panels, facet_cells)?;
        let root = if panels.len() == 1 && order.is_empty() {
            panels[0].descriptor.clone()
        } else {
            PreparedDiscreteGuide::merge(
                panels.iter().filter_map(|panel| panel.descriptor.as_ref()),
                &order,
                "prepared discrete guide",
            )?
        };
        self.discrete_guides
            .set(crate::data::source::source_arc(PreparedDiscreteGuides {
                root,
                panels,
            })?)
            .map_err(|_| Error::Render("discrete guide preparation was replaced".into()))
    }
}

fn parent_guide_order(
    chart: &Plot,
    panels: &[ScopedDiscreteGuide],
    facet_cells: bool,
) -> Result<Vec<crate::decorations::legend::GuideEntryIdentity>, Error> {
    use crate::decorations::legend::discrete::{GuideSectionIdentity, LegendComposition};
    use crate::decorations::legend::GuideEntryIdentity;
    let mut order = Vec::new();
    if chart.effective_legend_composition() == LegendComposition::Separate {
        // Facet marginals follow their own full parent domains, not the first
        // pair or occupied facet containing them. Canvas color domains also
        // include Hidden marks and cannot order visible guide contributions.
        if facet_cells {
            for (field, levels) in [
                (chart.group_name(), chart.global_group_levels()),
                (chart.hue_dimension_key(), chart.global_hue_levels()),
            ] {
                if let (Some(field), Some(levels)) = (field, levels) {
                    let consumed = panels
                        .iter()
                        .filter_map(|panel| panel.descriptor.as_ref())
                        .flat_map(|guide| guide.sections.iter())
                        .any(|section| match &section.identity {
                            GuideSectionIdentity::GroupStyle(raw)
                            | GuideSectionIdentity::Joint(raw)
                            | GuideSectionIdentity::HueColor(raw) => raw.as_str() == field,
                            _ => false,
                        });
                    if !consumed {
                        continue;
                    }
                    crate::data::source::reserve_source(&mut order, levels.len())?;
                    for value in levels {
                        let identity = GuideEntryIdentity::category(field, value)?;
                        if !order.contains(&identity) {
                            order.push(identity);
                        }
                    }
                }
            }
        }
    } else if let Some(pairs) = chart.global_collective_pairs() {
        crate::data::source::reserve_source(&mut order, pairs.len())?;
        for pair in pairs
            .iter()
            .filter(|pair| pair.group.is_some() || pair.hue.is_some())
        {
            order.push(chart.collective_legend_identity(pair)?);
        }
    } else if let Some(hues) = chart.global_hue_levels() {
        crate::data::source::reserve_source(&mut order, hues.len())?;
        for hue in hues {
            order.push(chart.point_legend_identity(hue)?);
        }
    }
    Ok(order)
}

fn include_keys<'a>(
    keys: &mut Vec<&'a str>,
    domain: impl IntoIterator<Item = &'a str>,
) -> Result<(), Error> {
    for key in domain {
        if !keys.contains(&key) {
            crate::data::source::reserve_source(keys, 1)?;
            keys.push(key);
        }
    }
    Ok(())
}
