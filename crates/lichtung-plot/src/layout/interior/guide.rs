//! Typed guide sections from one selected prepared semantic owner, without a panel
//! assignment, font environment, or backend measurement.

use crate::decorations::legend::discrete::PreparedGuide;
use crate::decorations::legend::GuideSourceKey;
use crate::orchestrator::registration::{
    CategoricalBinding, FamilyRef as F, GuideContribution, ParentOrders, PlotRegistrationBuild,
    PlotScopePolicy, PreparedAxesOwner, Scope,
};
use crate::{Error, PlotKind};

/// One measured mandatory right guide beside the flexible scientific body.
#[expect(
    clippy::too_many_lines,
    reason = "one ordered transaction measures the complete legend before track allocation and optional content-height callbacks"
)]
pub(crate) fn native_right(
    descriptor: std::sync::Arc<PreparedGuide>,
    panel: crate::layout::PixelRect,
    minimum: f64,
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &crate::layout::fit::FitScope,

    content_plot: Option<&crate::spec::PlotMeasureContext<'_, '_>>,
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
    let top = if let Some(context) = content_plot {
        let plot = context.registered.source;
        let registered = Some(context.registered);
        let intrinsic = context.intrinsic;
        let operation = context.operation;
        let hard = plot.intrinsic_body_minimum(
            registered,
            intrinsic,
            operation,
            theme,
            context.assignment.local.ranges(),
        )?;
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
                    context.assignment.local.ranges(),
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

fn family_marks<'a, T>(
    family: F<'a>,
    scope: &Scope,
    kind: PlotKind,
    select: impl FnOnce(F<'a>) -> Option<&'a T>,
) -> Result<&'a T, Error> {
    select(family).ok_or_else(|| {
        let scope = scope.render_name.as_deref();
        Error::Render(format!(
            "{kind:?} guide is missing prepared semantics for {scope:?}"
        ))
    })
}

/// Scientific color membership is independent of guide visibility, display
/// aliases and eventual section projection. No body probe participates.
pub(crate) fn observed_color_levels(
    chart: &crate::builder::PlotDescription,
    panels: &[PlotRegistrationBuild<'_>],
) -> Result<Vec<String>, Error> {
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
    for registered in panels {
        match registered
            .family
            .as_ref()
            .expect("prepared family")
            .as_ref()
        {
            F::Scatter(marks) => {
                for value in marks.hue_levels() {
                    include(value)?;
                }
            }
            F::Line { .. } | F::Matrix(_) => {
                for value in registered
                    .categorical
                    .as_ref()
                    .expect("prepared color binding")
                    .local_values()
                {
                    include(value)?;
                }
            }
            F::Bars { marks, .. } => {
                for value in marks.observed_hue_levels(
                    registered
                        .categorical
                        .as_ref()
                        .expect("prepared color binding")
                        .local_values(),
                ) {
                    include(value)?;
                }
            }
            F::Distribution { marks, .. } => {
                for value in marks.observed_hue_levels(
                    registered
                        .categorical
                        .as_ref()
                        .expect("prepared color binding")
                        .local_values(),
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
    chart: &crate::builder::PlotDescription,
    panels: &[PlotRegistrationBuild<'_>],
    appearance: &crate::theme::ResolvedTheme,
    global_hues: Option<&crate::data::category::CategoryDomain>,
) -> Result<(), Error> {
    let global_hues = global_hues.or_else(|| {
        panels
            .first()
            .and_then(|panel| panel.parent_orders.hues.as_deref())
    });
    let palette = if let Some(scale) = chart.categorical_color_scale_spec() {
        let Some(palette) = scale.palette() else {
            if !chart.colors_by_sign() {
                for key in global_hues
                    .into_iter()
                    .flat_map(crate::data::category::CategoryDomain::values)
                {
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
        appearance.color_scales().get_qualitative()
    };
    if palette.capacity() != crate::theme::PaletteCapacity::Fixed || chart.colors_by_sign() {
        return Ok(());
    }
    let mut keys = Vec::new();
    if let Some(global) = global_hues {
        include_keys(&mut keys, global.values())?;
    }
    for registered in panels {
        let axes = registered.axes.as_ref().expect("prepared axes");
        match registered
            .family
            .as_ref()
            .expect("prepared family")
            .as_ref()
        {
            F::Scatter(marks) => include_keys(&mut keys, marks.hue_levels())?,
            F::Line { .. } | F::Matrix(_) => include_keys(
                &mut keys,
                registered
                    .categorical
                    .as_ref()
                    .expect("prepared color binding")
                    .local_values(),
            )?,
            F::Density(marks) => include_keys(&mut keys, marks.hue_levels())?,
            F::Bars { .. } => {
                let categories = &axes
                    .categories(usize::from(matches!(
                        chart.kind(),
                        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                    )))
                    .expect("prepared Bar categories")
                    .raw;
                let binding = registered
                    .categorical
                    .as_ref()
                    .expect("prepared color binding");
                if binding.local_len() == 0 {
                    include_keys(&mut keys, categories.iter().map(String::as_str))?;
                } else {
                    include_keys(&mut keys, binding.local_values())?;
                }
            }
            F::Distribution { .. } => {
                let binding = registered
                    .categorical
                    .as_ref()
                    .expect("prepared color binding");
                if binding.local_len() == 0 {
                    let domain = &axes
                        .categories(0)
                        .expect("prepared distribution categories")
                        .raw;
                    include_keys(&mut keys, domain.iter().map(String::as_str))?;
                } else {
                    include_keys(&mut keys, binding.local_values())?;
                }
            }
            F::Interaction(marks) => include_keys(&mut keys, marks.color_keys())?,
            F::Marginal(marks) => include_keys(&mut keys, marks.hue_levels())?,
            _ => {}
        }
    }
    palette.validate_capacity(keys.len())
}

#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "the closed guide dispatch borrows each family's actual scientific and axis owners"
)]
pub(crate) fn guide_sources(
    chart: &crate::builder::PlotDescription,
    scope: &Scope,
    family: F<'_>,
    axes: &PreparedAxesOwner,
    categorical: Option<&CategoricalBinding>,
    parent_orders: &ParentOrders,
    theme: &crate::theme::ResolvedTheme,
    operation: &crate::font::canonical::OperationContext,
) -> Result<Vec<GuideSourceKey>, Error> {
    let kind = chart.kind();
    match kind {
        PlotKind::Scatter => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Scatter(marks) => Some(marks),
                _ => None,
            })?;
            let binding = categorical.expect("prepared color binding");
            let order = marks.group_order(binding)?;
            marks.guide_entries_in_order(chart, binding, theme, operation, &order)
        }
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Line { marks, .. } => Some(marks),
                _ => None,
            })?;
            let order = marks.group_order(
                chart,
                parent_orders.collectives.as_deref().map(Vec::as_slice),
                parent_orders.groups.as_deref().map(Vec::as_slice),
            )?;
            marks.guide_entries_in_order(
                chart,
                categorical.expect("prepared color binding"),
                theme,
                operation,
                &order,
            )
        }
        PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Bars { marks, .. } => Some(marks),
                _ => None,
            })?;
            let binding = categorical.expect("prepared color binding");
            let (colors, accents) =
                crate::plots::bar_data::resolved_colors(chart, axes, binding, theme, operation)?;
            crate::plots::bar_data::guide_sources(marks, chart, binding, theme, &colors, &accents)
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Distribution { marks, .. } => Some(marks),
                _ => None,
            })?;
            marks.guide_entries(
                chart,
                axes,
                categorical.expect("prepared color binding"),
                theme,
                operation,
            )
        }
        PlotKind::Density => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Density(marks) => Some(marks),
                _ => None,
            })?;
            let order =
                marks.group_order(parent_orders.collectives.as_deref().map(Vec::as_slice))?;
            marks.guide_entries_in_order(
                chart,
                categorical.expect("prepared color binding"),
                theme,
                operation,
                &order,
            )
        }
        PlotKind::Interaction => {
            let marks = family_marks(family, scope, kind, |family| match family {
                F::Interaction(marks) => Some(marks),
                _ => None,
            })?;
            marks.guide_entries(
                chart,
                theme,
                operation,
                categorical.expect("prepared color binding"),
            )
        }
        PlotKind::Marginal => match family {
            F::Marginal(marks) => Some(marks),
            _ => None,
        }
        .ok_or_else(|| Error::Render("Marginal guide is missing prepared semantics".into()))?
        .guide_entries(
            chart,
            categorical.expect("prepared color binding"),
            theme,
            operation,
        ),
        PlotKind::Regression => {
            family_marks(family, scope, kind, |family| match family {
                F::Regression(marks) => Some(marks),
                _ => None,
            })?;
            let color =
                crate::theme::AccentColors::resolve(theme.marks.get_accents(), 1)?.color(0)?;
            crate::plots::regression::PreparedRegression::guide_entries(chart, theme, color)
        }
        PlotKind::Dumbbell => {
            family_marks(family, scope, kind, |family| match family {
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

#[expect(
    clippy::too_many_arguments,
    reason = "one guide contribution borrows the occurrence's actual owners before publication"
)]
pub(crate) fn prepare_guide_contribution(
    source: &crate::builder::PlotDescription,
    scope: &Scope,
    policy: &PlotScopePolicy,
    family: F<'_>,
    axes: &PreparedAxesOwner,
    categorical: Option<&CategoricalBinding>,
    parent_orders: &ParentOrders,
    appearance: &crate::theme::ResolvedTheme,
    operation: &crate::font::canonical::OperationContext,
) -> Result<GuideContribution, Error> {
    let alpha = alpha_sources(source, scope, family, axes, appearance)?;
    let descriptor = if policy.discrete_guide_use
        == crate::decorations::legend::discrete::scoped::GuideUse::Hidden
    {
        None
    } else {
        let keys = if source.kind() == PlotKind::Matrix {
            match family {
                F::Matrix(marks) => Some(marks),
                _ => None,
            }
            .ok_or_else(|| Error::Render("Matrix guide preparation missing".into()))?
            .guide_sources(
                source,
                categorical.expect("prepared color binding"),
                appearance,
                operation,
            )?
        } else {
            guide_sources(
                source,
                scope,
                family,
                axes,
                categorical,
                parent_orders,
                appearance,
                operation,
            )?
        };
        let continuous = continuous_source(source, scope, family, appearance)?;
        PreparedGuide::prepare(
            source,
            policy,
            appearance,
            &scope.address,
            keys,
            continuous,
            alpha.sections(&policy.labels)?,
        )?
    };
    Ok(GuideContribution { descriptor, alpha })
}

pub(crate) fn prepare_discrete_guides(
    chart: &crate::builder::PlotDescription,
    registered: &mut [PlotRegistrationBuild<'_>],
    resources: &crate::font::canonical::OperationContext,
    facet_cells: bool,
) -> Result<Option<std::sync::Arc<PreparedGuide>>, Error> {
    let mut panels = Vec::new();
    crate::data::source::reserve_source(&mut panels, registered.len())?;
    for panel in registered.iter() {
        panel.data(resources)?;
        panels.push(prepare_guide_contribution(
            chart,
            &panel.scope,
            &panel.policy,
            panel.family.as_ref().expect("prepared family").as_ref(),
            panel.axes.as_ref().expect("prepared axes"),
            panel.categorical.as_ref(),
            &panel.parent_orders,
            &panel.appearance,
            resources,
        )?);
    }
    let order = parent_guide_order(
        chart,
        registered.first().map_or_else(
            || chart.legend_composition_spec(),
            |registered| registered.policy.guide_composition,
        ),
        registered
            .first()
            .and_then(|registered| registered.parent_orders.collectives.as_deref())
            .map(Vec::as_slice),
        registered.first().and_then(|registered| {
            registered
                .parent_orders
                .groups
                .as_deref()
                .map(Vec::as_slice)
        }),
        registered
            .first()
            .and_then(|registered| registered.parent_orders.hues.as_deref()),
        &panels,
        facet_cells,
    )?;
    let root = if facet_cells && matches!(chart.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
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
    for (registered, guide) in registered.iter_mut().zip(panels) {
        registered.guide = Some(guide);
    }
    Ok(root)
}

pub(crate) fn continuous_source(
    chart: &crate::builder::PlotDescription,
    scope: &Scope,
    family: F<'_>,
    theme: &crate::theme::ResolvedTheme,
) -> Result<
    Option<std::sync::Arc<crate::decorations::legend::descriptor::ContinuousDescriptor>>,
    Error,
> {
    let field = chart.color_dimension_key().unwrap_or("");
    let descriptor = match chart.kind() {
        PlotKind::Heatmap => family_marks(family, scope, chart.kind(), |family| match family {
            F::Heatmap(marks) => Some(marks),
            _ => None,
        })?
        .guide_descriptor(field, theme)?,
        PlotKind::Beeswarm => family_marks(family, scope, chart.kind(), |family| match family {
            F::Beeswarm(marks) => Some(marks),
            _ => None,
        })?
        .guide_descriptor(field)?,
        _ => return Ok(None),
    };
    descriptor.map(crate::data::source::source_arc).transpose()
}

pub(crate) fn alpha_sources(
    chart: &crate::builder::PlotDescription,
    scope: &Scope,
    family: F<'_>,
    axes: &PreparedAxesOwner,
    theme: &crate::theme::ResolvedTheme,
) -> Result<std::sync::Arc<crate::decorations::legend::alpha::PreparedAlphaInputs>, Error> {
    let mut guide = crate::decorations::legend::alpha::AlphaGuideBuilder::new(
        chart,
        scope.render_name.as_deref(),
        theme,
        &scope.address,
    );
    if !guide.any_enabled() {
        return guide.finish();
    }
    macro_rules! standard {
        ($pattern:pat => $marks:ident) => {
            family_marks(family, scope, chart.kind(), |family| {
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
            let marks = family_marks(family, scope, chart.kind(), |family| {
                if let F::Bars { marks, .. } = family {
                    Some(marks)
                } else {
                    None
                }
            })?;
            let categories = &axes
                .categories(usize::from(matches!(
                    chart.kind(),
                    PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                )))
                .expect("prepared Bar categories")
                .raw;
            marks.alpha_guide(chart, categories, &marks.series, theme, &mut guide)?;
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            standard!(F::Distribution { marks, .. } => marks);
        }
        PlotKind::Beeswarm => standard!(F::Beeswarm(marks) => marks),
        PlotKind::Heatmap => standard!(F::Heatmap(marks) => marks),
        PlotKind::Waterfall => standard!(F::Waterfall(marks) => marks),
        PlotKind::Marginal => match family {
            F::Marginal(marks) => Some(marks),
            _ => None,
        }
        .ok_or_else(|| Error::Render("Marginal alpha guide is missing actual groups".into()))?
        .alpha_guide(chart, theme, &mut guide)?,
        PlotKind::Matrix => match family {
            F::Matrix(marks) => Some(marks),
            _ => None,
        }
        .ok_or_else(|| Error::Render("Matrix alpha guide is missing actual groups".into()))?
        .alpha_guide(chart, axes, theme, &mut guide)?,
    }
    guide.finish()
}
fn parent_guide_order(
    chart: &crate::builder::PlotDescription,
    composition: crate::LegendComposition,
    collectives: Option<&[crate::data::CollectivePair]>,
    groups: Option<&[String]>,
    hues: Option<&crate::data::category::CategoryDomain>,
    panels: &[GuideContribution],
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
                (
                    chart.group_name(),
                    groups.map(crate::data::category::CategoryOrderRef::Raw),
                ),
                (
                    chart.hue_dimension_key(),
                    hues.map(crate::data::category::CategoryOrderRef::Keys),
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
                    for value in levels.values() {
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
            order.push(chart.collective_legend_identity(pair)?);
        }
    } else if let Some(hues) = hues {
        crate::data::source::reserve_source(&mut order, hues.ordered.len())?;
        for hue in hues.values() {
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
