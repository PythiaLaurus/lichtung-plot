//! Typed guide sections from one selected prepared semantic owner, without a panel
//! assignment, font environment, or backend measurement.

use super::PreparedInteriorInputs;
use crate::data::PlotData;
use crate::decorations::legend::discrete::{PreparedGuide, PreparedGuides, ScopedGuide};
use crate::decorations::legend::GuideSourceKey;
use crate::orchestrator::registration::PreparedFamily as F;
use crate::{Error, Plot, PlotKind};

/// One measured mandatory right guide beside the flexible scientific body.
pub(crate) fn native_right(
    descriptor: std::sync::Arc<PreparedGuide>,
    panel: crate::layout::PixelRect,
    minimum: f64,
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &crate::layout::fit::FitScope,

    content_plot: Option<(
        &Plot,
        Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        &crate::orchestrator::root::PlotIntrinsics,
        &crate::font::canonical::OperationContext,
    )>,
) -> Result<
    (crate::layout::PixelRect, crate::decorations::legend::Legend),
    crate::layout::fit::PlanError,
> {
    use crate::layout::fit::{self, FitKind};
    use crate::layout::kernel::{allocate_grid_axis, GridRequirement, TrackSpec};
    let widths = crate::decorations::legend::Legend::vertical_widths(&descriptor, theme)?;
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
    let guide = fit::measured(crate::decorations::legend::Legend::measure_assigned(
        descriptor,
        (width, content_plot.is_none().then_some(panel.height())),
        false,
        false,
        theme,
        &mut false,
        scope,
    ))?;
    let hard = crate::layout::kernel::ceil_spatial_quanta(minimum)?;
    let width = crate::layout::kernel::ceil_spatial_quanta(guide.compute_box_width())?;
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
    let top = if let Some((plot, registered, intrinsic, operation)) = content_plot {
        let hard = plot.intrinsic_body_minimum(registered, intrinsic, operation, theme)?;
        let preference = crate::layout::interior::sizing::body_preference(
            plot, registered, intrinsic, operation, theme, hard,
        )?;
        panel.y0
            + plot
                .natural_body_height(
                    registered,
                    intrinsic,
                    operation,
                    theme,
                    body_right - panel.x0,
                    preference.1,
                )?
                .max(guide.compute_box_height())
    } else {
        panel.y1
    };
    Ok((
        crate::layout::PixelRect::new(panel.x0, body_right, panel.y0, top),
        guide,
    ))
}

impl PreparedInteriorInputs<'_> {
    fn family<'a, T>(
        registered: &'a crate::orchestrator::registration::PlotRegistration<'_>,
        kind: PlotKind,
        select: impl FnOnce(&'a crate::orchestrator::registration::PreparedFamily) -> Option<&'a T>,
    ) -> Result<&'a T, Error> {
        select(&registered.family).ok_or_else(|| {
            let scope = registered.scope.render_name.as_deref();
            Error::Render(format!(
                "{kind:?} guide is missing prepared semantics for {scope:?}"
            ))
        })
    }

    /// Scientific color membership is independent of guide visibility, display
    /// aliases and eventual section projection. No body probe participates.
    pub(crate) fn observed_color_levels(&self, chart: &Plot) -> Result<Vec<String>, Error> {
        self.route.validate(chart.description())?;
        if chart.description().categorical_color_field().is_none() {
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
        for registered in &self.plots {
            match &registered.family {
                F::Scatter(marks) => {
                    for value in marks.hue_levels() {
                        include(value)?;
                    }
                }
                F::Line { marks, .. } => {
                    for value in marks.hue_levels() {
                        include(value)?;
                    }
                }
                F::Bars { marks, .. } => {
                    for value in marks.observed_hue_levels(
                        registered
                            .policy
                            .coordinated
                            .colors
                            .as_deref()
                            .expect("prepared Bar color domain"),
                    ) {
                        include(value)?;
                    }
                }
                F::Distribution { marks, .. } => {
                    for value in marks.observed_hue_levels(
                        registered
                            .policy
                            .coordinated
                            .colors
                            .as_deref()
                            .expect("prepared distribution color domain"),
                    ) {
                        include(value)?;
                    }
                }
                F::Density(marks) => {
                    for value in marks.hue_levels() {
                        include(value)?;
                    }
                }
                F::Interaction(marks) => {
                    for value in marks.color_keys() {
                        include(value)?;
                    }
                }
                F::Marginal(marks) => {
                    for value in marks.hue_levels() {
                        include(value)?;
                    }
                }
                F::Matrix(_) => {
                    for value in registered
                        .policy
                        .coordinated
                        .colors
                        .as_deref()
                        .expect("prepared Matrix color domain")
                    {
                        include(value)?;
                    }
                }
                _ => {}
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
        let palette = if let Some(scale) = chart.description().categorical_color_scale_spec() {
            let Some(palette) = scale.palette() else {
                if !chart.description().colors_by_sign() {
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
            theme.color_scales().get_qualitative()
        };
        if palette.capacity() != crate::theme::PaletteCapacity::Fixed
            || chart.description().colors_by_sign()
        {
            return Ok(());
        }
        let mut keys = Vec::new();
        if let Some(global) = global_hues {
            include_keys(&mut keys, global.iter().map(String::as_str))?;
        }
        for registered in &self.plots {
            match &registered.family {
                F::Scatter(marks) => include_keys(&mut keys, marks.hue_levels())?,
                F::Line { marks, .. } => include_keys(&mut keys, marks.hue_levels())?,
                F::Density(marks) => include_keys(&mut keys, marks.hue_levels())?,
                F::Bars { .. } => {
                    let categories = &registered
                        .axes
                        .categories(usize::from(matches!(
                            chart.kind(),
                            PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                        )))
                        .expect("prepared Bar categories")
                        .raw;
                    include_keys(
                        &mut keys,
                        crate::plots::bar_data::color_keys(
                            categories,
                            registered
                                .policy
                                .coordinated
                                .colors
                                .as_deref()
                                .expect("prepared Bar color domain"),
                        )
                        .iter()
                        .map(String::as_str),
                    )?;
                }
                F::Distribution { .. } => {
                    let hues = registered
                        .policy
                        .coordinated
                        .colors
                        .as_deref()
                        .expect("prepared distribution color domain");
                    let domain = if hues.is_empty() {
                        &registered
                            .axes
                            .categories(0)
                            .expect("prepared distribution categories")
                            .raw
                    } else {
                        hues
                    };
                    include_keys(&mut keys, domain.iter().map(String::as_str))?;
                }
                F::Interaction(marks) => include_keys(&mut keys, marks.color_keys())?,
                F::Marginal(marks) => include_keys(&mut keys, marks.hue_levels())?,
                F::Matrix(_) => {
                    include_keys(
                        &mut keys,
                        registered
                            .policy
                            .coordinated
                            .colors
                            .as_deref()
                            .expect("prepared Matrix color domain")
                            .iter()
                            .map(String::as_str),
                    )?;
                }
                _ => {}
            }
        }
        palette.validate_capacity(keys.len())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the closed guide dispatch borrows each family's actual scientific and axis owners"
    )]
    pub(crate) fn guide_sources(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: PlotData<'_, '_>,
        registered: &crate::orchestrator::registration::PlotRegistration<'_>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        self.route.validate(chart.description())?;
        let kind = chart.kind();
        match kind {
            PlotKind::Scatter => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Scatter(marks) => Some(marks),
                    _ => None,
                })?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Line { marks, .. } => Some(marks),
                    _ => None,
                })?;
                let order = marks.group_order(
                    chart,
                    crate::orchestrator::registration::collective_pairs(Some(registered)),
                    crate::orchestrator::registration::composed_group_levels(Some(registered)),
                )?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Bars { marks, .. } => Some(marks),
                    _ => None,
                })?;
                let categories = &registered
                    .axes
                    .categories(usize::from(matches!(
                        kind,
                        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                    )))
                    .expect("prepared Bar categories")
                    .raw;
                let hues = registered
                    .policy
                    .coordinated
                    .colors
                    .as_deref()
                    .expect("prepared Bar color domain");
                let (colors, accents) =
                    crate::plots::bar_data::resolved_colors(chart, categories, hues, theme, data)?;
                crate::plots::bar_data::guide_sources(marks, chart, hues, theme, &colors, &accents)
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Distribution { marks, .. } => Some(marks),
                    _ => None,
                })?;
                marks.guide_entries(
                    chart,
                    &registered
                        .axes
                        .categories(0)
                        .expect("prepared distribution categories")
                        .raw,
                    registered
                        .policy
                        .coordinated
                        .colors
                        .as_deref()
                        .expect("prepared distribution color domain"),
                    theme,
                    data,
                )
            }
            PlotKind::Density => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Density(marks) => Some(marks),
                    _ => None,
                })?;
                let order = marks.group_order(
                    crate::orchestrator::registration::collective_pairs(Some(registered)),
                )?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Interaction => {
                let marks = Self::family(registered, kind, |family| match family {
                    F::Interaction(marks) => Some(marks),
                    _ => None,
                })?;
                marks.guide_entries(
                    chart,
                    theme,
                    data,
                    crate::orchestrator::registration::composed_group_levels(Some(registered)),
                )
            }
            PlotKind::Marginal => Some(&registered.family)
                .and_then(|family| match family {
                    F::Marginal(marks) => Some(marks),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::Render("Marginal guide is missing prepared semantics".into())
                })?
                .guide_entries(
                    chart,
                    registered
                        .policy
                        .coordinated
                        .colors
                        .as_deref()
                        .expect("prepared Marginal color domain"),
                    theme,
                    data,
                ),
            PlotKind::Regression => {
                Self::family(registered, kind, |family| match family {
                    F::Regression(marks) => Some(marks),
                    _ => None,
                })?;
                let color =
                    crate::theme::AccentColors::resolve(theme.marks.get_accents(), 1)?.color(0)?;
                crate::plots::regression::PreparedRegression::guide_entries(chart, theme, color)
            }
            PlotKind::Dumbbell => {
                Self::family(registered, kind, |family| match family {
                    F::Dumbbell(marks) => Some(marks),
                    _ => None,
                })?;
                crate::plots::dumbbell::DumbbellData::guide_entries(chart, theme)
            }
            PlotKind::Histogram
            | PlotKind::Qq
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Waterfall
            | PlotKind::Heatmap
            | PlotKind::Beeswarm => Ok(Vec::new()),
            PlotKind::Matrix => Err(Error::Render(
                "self-managed Matrix has no parent-owned guide entry".into(),
            )),
        }
    }

    pub(crate) fn prepare_discrete_guides(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        resources: &crate::font::canonical::OperationContext,
        owner: &crate::layout::ComponentAddress,
        facet_cells: bool,
    ) -> Result<(), Error> {
        if self.discrete_guides.get().is_some() {
            return Err(Error::Render(
                "discrete guides were prepared more than once".into(),
            ));
        }
        let mut panels = Vec::new();
        crate::data::source::reserve_source(&mut panels, self.plots.len())?;
        for (index, registered) in self.plots.iter().enumerate() {
            let data = registered.data(resources)?;
            let scope = registered.scope.render_name.as_deref();
            let contributor = if facet_cells {
                owner.child(
                    crate::layout::AddressRole::FacetCell,
                    u32::try_from(index)
                        .map_err(|_| Error::Render("facet guide source address overflow".into()))?,
                )?
            } else {
                owner.clone()
            };
            let alpha = self.alpha_sources(chart, theme, registered, &contributor)?;
            let descriptor = if crate::orchestrator::registration::legend_policy(
                chart.description(),
                Some(registered),
            ) == crate::LegendPolicy::Hidden
            {
                None
            } else {
                let keys = if chart.kind() == PlotKind::Matrix {
                    Some(&registered.family)
                        .and_then(|family| match family {
                            F::Matrix(marks) => Some(marks),
                            _ => None,
                        })
                        .ok_or_else(|| Error::Render("Matrix guide preparation missing".into()))?
                        .guide_sources(
                            chart,
                            registered
                                .policy
                                .coordinated
                                .colors
                                .as_deref()
                                .expect("prepared Matrix color domain"),
                            theme,
                            data,
                        )?
                } else {
                    self.guide_sources(chart, theme, data, registered)?
                };
                let continuous = Self::continuous_source(chart, theme, registered)?;
                PreparedGuide::prepare(
                    chart,
                    &registered.policy,
                    theme,
                    &contributor,
                    keys,
                    continuous,
                    alpha.sections(&registered.policy.labels)?,
                )?
            };
            panels.push(ScopedGuide {
                scope: scope
                    .map(crate::data::source::copy_source_string)
                    .transpose()?,
                descriptor,
                alpha,
            });
        }
        let order = parent_guide_order(
            chart,
            self.plots.first().map_or_else(
                || chart.description().legend_composition_spec(),
                |registered| registered.policy.guide_composition,
            ),
            crate::orchestrator::registration::collective_pairs(self.plots.first()),
            crate::orchestrator::registration::composed_group_levels(self.plots.first()),
            &panels,
            facet_cells,
        )?;
        let root = if facet_cells && matches!(chart.kind(), PlotKind::Heatmap | PlotKind::Beeswarm)
        {
            // Native color bodies belong to each actual facet. Their domains,
            // missing blocks and alpha members are not a parent-wide union.
            None
        } else if panels.len() == 1 && order.is_empty() {
            panels[0].descriptor.clone()
        } else {
            PreparedGuide::merge(
                panels.iter().filter_map(|panel| panel.descriptor.as_ref()),
                &order,
                "prepared discrete guide",
            )?
        };
        self.discrete_guides
            .set(crate::data::source::source_arc(PreparedGuides {
                root,
                panels,
            })?)
            .map_err(|_| Error::Render("discrete guide preparation was replaced".into()))
    }

    pub(crate) fn continuous_source(
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        registered: &crate::orchestrator::registration::PlotRegistration<'_>,
    ) -> Result<
        Option<std::sync::Arc<crate::decorations::legend::descriptor::ContinuousDescriptor>>,
        Error,
    > {
        let field = chart.description().color_dimension_key().unwrap_or("");
        let descriptor = match chart.kind() {
            PlotKind::Heatmap => Self::family(registered, chart.kind(), |family| match family {
                F::Heatmap(marks) => Some(marks),
                _ => None,
            })?
            .guide_descriptor(field, theme)?,
            PlotKind::Beeswarm => Self::family(registered, chart.kind(), |family| match family {
                F::Beeswarm(marks) => Some(marks),
                _ => None,
            })?
            .guide_descriptor(field)?,
            _ => return Ok(None),
        };
        descriptor.map(crate::data::source::source_arc).transpose()
    }

    pub(crate) fn alpha_sources(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        registered: &crate::orchestrator::registration::PlotRegistration<'_>,
        owner: &crate::layout::ComponentAddress,
    ) -> Result<std::sync::Arc<crate::decorations::legend::alpha::PreparedAlphaInputs>, Error> {
        self.route.validate(chart.description())?;
        let mut guide = crate::decorations::legend::alpha::AlphaGuideBuilder::new(
            chart,
            registered.scope.render_name.as_deref(),
            theme,
            owner,
        );
        if !guide.any_enabled() {
            return guide.finish();
        }
        macro_rules! standard {
            ($pattern:pat => $marks:ident) => {
                Self::family(registered, chart.kind(), |family| {
                    if let $pattern = family {
                        Some($marks)
                    } else {
                        None
                    }
                })?
                .alpha_guide(chart, theme, &mut guide)?
            };
        }
        match chart.kind() {
            PlotKind::Scatter => standard!(F::Scatter(marks) => marks),
            PlotKind::Qq => standard!(F::Qq(marks) => marks),
            PlotKind::Regression => standard!(F::Regression(marks) => marks),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                standard!(F::Line { marks, .. } => marks);
            }
            PlotKind::Density => standard!(F::Density(marks) => marks),
            PlotKind::Interaction => standard!(F::Interaction(marks) => marks),
            PlotKind::Dumbbell => standard!(F::Dumbbell(marks) => marks),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                standard!(F::Estimate { marks, .. } => marks);
            }
            PlotKind::Histogram => standard!(F::Histogram(marks) => marks),
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let marks = Self::family(registered, chart.kind(), |family| {
                    if let F::Bars { marks, .. } = family {
                        Some(marks)
                    } else {
                        None
                    }
                })?;
                let categories = &registered
                    .axes
                    .categories(usize::from(matches!(
                        chart.kind(),
                        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                    )))
                    .expect("prepared Bar categories")
                    .raw;
                marks.alpha_guide(
                    chart,
                    categories,
                    registered
                        .policy
                        .coordinated
                        .groups
                        .as_deref()
                        .expect("prepared Bar group domain"),
                    theme,
                    &mut guide,
                )?;
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                standard!(F::Distribution { marks, .. } => marks);
            }
            PlotKind::Beeswarm => standard!(F::Beeswarm(marks) => marks),
            PlotKind::Heatmap => standard!(F::Heatmap(marks) => marks),
            PlotKind::Waterfall => standard!(F::Waterfall(marks) => marks),
            PlotKind::Marginal => Some(&registered.family)
                .and_then(|family| match family {
                    F::Marginal(marks) => Some(marks),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::Render("Marginal alpha guide is missing actual groups".into())
                })?
                .alpha_guide(chart, theme, &mut guide)?,
            PlotKind::Matrix => Some(&registered.family)
                .and_then(|family| match family {
                    F::Matrix(marks) => Some(marks),
                    _ => None,
                })
                .ok_or_else(|| Error::Render("Matrix alpha guide is missing actual groups".into()))?
                .alpha_guide(chart, &registered.axes, theme, &mut guide)?,
        }
        guide.finish()
    }
}

fn parent_guide_order(
    chart: &Plot,
    composition: crate::LegendComposition,
    collectives: Option<&[crate::data::CollectivePair]>,
    groups: Option<&[String]>,
    panels: &[ScopedGuide],
    facet_cells: bool,
) -> Result<Vec<crate::decorations::legend::GuideEntryIdentity>, Error> {
    use crate::decorations::legend::discrete::GuideSectionIdentity;
    use crate::decorations::legend::GuideEntryIdentity;
    use crate::LegendComposition;
    let mut order = Vec::new();
    if composition == LegendComposition::Separate {
        // Facet marginals follow their own full parent domains, not the first
        // pair or occupied facet containing them. Canvas color domains also
        // include Hidden marks and cannot order visible guide contributions.
        if facet_cells {
            for (field, levels) in [
                (chart.description().group_name(), groups),
                (
                    chart.description().hue_dimension_key(),
                    chart.global_hue_levels(),
                ),
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
    } else if let Some(pairs) = collectives {
        crate::data::source::reserve_source(&mut order, pairs.len())?;
        for pair in pairs
            .iter()
            .filter(|pair| pair.group.is_some() || pair.hue.is_some())
        {
            order.push(chart.description().collective_legend_identity(pair)?);
        }
    } else if let Some(hues) = chart.global_hue_levels() {
        crate::data::source::reserve_source(&mut order, hues.len())?;
        for hue in hues {
            order.push(chart.description().point_legend_identity(hue)?);
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
