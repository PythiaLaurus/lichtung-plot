use crate::data::PlotData;
use crate::layout::interior::{InteriorRoute, PreparedInteriorInputs};
use crate::layout::{AxisSpec, AxisTick};
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::{Error, Plot};

/// Scientific coordinates and raw category sources. Post-font summaries are
/// initialized by the owning root before allocating its panel, never by preparation.
#[derive(Debug)]
pub(crate) struct PreparedAxes {
    pub(crate) intrinsics: std::sync::OnceLock<crate::layout::interior::coordinate::Intrinsics>,
    pub(crate) heatmap_intrinsics:
        std::sync::OnceLock<crate::plots::heatmap::HeatmapIntrinsicCache>,
    pub(crate) ranges: [(f64, f64); 2],
    pub(crate) categories: [Option<Vec<AxisTick>>; 2],
    pub(crate) display_categories: [Option<Vec<String>>; 2],
    pub(crate) labels: [String; 2],
    pub(crate) specs: [AxisSpec; 2],
    pub(crate) logarithmic: bool,
}

/// Scientific inputs before a parent's shared range is known.
pub(crate) enum PreparedReference {
    Waterfall(crate::plots::waterfall::PreparedWaterfall),
    Beeswarm(crate::plots::beeswarm::PreparedBeeswarm),
    Marginal(crate::plots::marginal::PreparedMarginal),
    Heatmap(crate::plots::heatmap::HeatmapData),
    Interaction(crate::plots::interaction::PreparedInteraction),
    Distribution(
        crate::PlotKind,
        crate::plots::categorical_distribution::PreparedDistribution,
    ),
    Bars(crate::PlotKind, crate::plots::bar_data::PreparedBars),
    Estimate(crate::plots::interval_estimate::PreparedEstimate),
    Dumbbell(crate::plots::dumbbell::DumbbellData),
    Histogram(crate::plots::histogram::HistogramSemantics),
    Density(crate::plots::density::PreparedDensity),
    Qq(crate::plots::qq::PreparedQq),
    Regression(crate::plots::regression::PreparedRegression),
}

impl PreparedReference {
    pub(crate) fn new(source: &Plot, data: PlotData<'_>) -> Result<Self, Error> {
        match source.kind() {
            PlotKind::Waterfall => Ok(Self::Waterfall(crate::plots::waterfall::prepare(
                source, data,
            )?)),
            PlotKind::Beeswarm => Ok(Self::Beeswarm(crate::plots::beeswarm::prepare(
                source, data,
            )?)),
            PlotKind::Marginal => Ok(Self::Marginal(crate::plots::marginal::prepare(
                source, data,
            )?)),
            PlotKind::Heatmap => Ok(Self::Heatmap(crate::plots::heatmap::prepare(source, data)?)),
            PlotKind::Interaction => Ok(Self::Interaction(crate::plots::interaction::prepare(
                source, data, None,
            )?)),
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => Ok(Self::Distribution(
                source.kind(),
                crate::plots::categorical_distribution::PreparedDistribution::new(
                    source, data, None,
                )?,
            )),
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => Ok(Self::Bars(
                source.kind(),
                crate::plots::bar_data::prepare(
                    source,
                    data,
                    crate::plots::bar_data::mode(source),
                )?,
            )),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => Ok(
                Self::Estimate(crate::plots::interval_estimate::PreparedEstimate::new(
                    source,
                    data,
                    source.significance_annotations(),
                )?),
            ),
            PlotKind::Dumbbell => Ok(Self::Dumbbell(crate::plots::dumbbell::prepare(
                source, data,
            )?)),
            PlotKind::Histogram => Ok(Self::Histogram(
                crate::plots::histogram::HistogramSemantics::new(source, data)?,
            )),
            PlotKind::Density => Ok(Self::Density(crate::plots::density::PreparedDensity::new(
                source, data,
            )?)),
            PlotKind::Qq => Ok(Self::Qq(crate::plots::qq::PreparedQq::new(source, data)?)),
            PlotKind::Regression => Ok(Self::Regression(
                crate::plots::regression::PreparedRegression::new(source, data)?,
            )),
            _ => Err(Error::Render(
                "shared semantics require an extracted reference or paired-endpoint family".into(),
            )),
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "category counts are admitted source rows"
    )]
    pub(crate) fn x_range(&self, source: &Plot, data: PlotData<'_>) -> Result<(f64, f64), Error> {
        Ok(match self {
            Self::Waterfall(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source.x_axis_spec(),
                data.source_resources()?,
            )?,
            Self::Beeswarm(marks) => crate::layout::continuous_auto_range(
                marks.x_data,
                source.x_axis_spec(),
                data.source_resources()?,
            )?,
            Self::Marginal(marks) => marks.ranges[0],
            Self::Heatmap(marks) => (0.0, marks.x_levels.len() as f64),
            Self::Interaction(marks) => (0.0, marks.conditions.len() as f64),
            Self::Distribution(_, marks) => (0.0, marks.categories().len() as f64),
            Self::Bars(kind, marks) => {
                if matches!(
                    kind,
                    PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                ) {
                    crate::layout::anchored_auto_range(
                        marks.value_range,
                        source.x_axis_spec(),
                        0.0,
                        data.source_resources()?,
                    )?
                } else {
                    (0.0, marks.categories.len() as f64)
                }
            }
            Self::Estimate(marks) => marks.x_range()?,
            Self::Dumbbell(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source.x_axis_spec(),
                data.source_resources()?,
            )?,
            Self::Histogram(marks) => marks.ranges[0],
            Self::Density(marks) => marks.ranges(source, data)?[0],
            Self::Qq(marks) => marks.ranges(source, data)?[0],
            Self::Regression(marks) => marks.ranges[0],
        })
    }
}

#[derive(Default)]
struct ReferenceAxisConfig<'a> {
    waterfall: Option<&'a crate::plots::waterfall::PreparedWaterfall>,
    beeswarm: Option<&'a crate::plots::beeswarm::PreparedBeeswarm>,
    marginal: Option<&'a crate::plots::marginal::PreparedMarginal>,
    heatmap: Option<&'a crate::plots::heatmap::HeatmapData>,
    interaction: Option<&'a crate::plots::interaction::PreparedInteraction>,
    distribution: Option<&'a crate::plots::categorical_distribution::PreparedDistribution>,
    bars: Option<&'a crate::plots::bar_data::PreparedBars>,
    estimate: Option<&'a crate::plots::interval_estimate::PreparedEstimate>,
    estimate_fallback: Option<(f64, f64)>,
    dumbbell: Option<&'a crate::plots::dumbbell::DumbbellData>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&'a crate::Labels>,
}

#[expect(
    clippy::too_many_lines,
    reason = "borrowed root configuration attaches each admitted family payload without cloning source-owned plots"
)]
pub(crate) fn prepare_reference_plot(
    source: &Plot,
    resources: &crate::font::canonical::OperationGenerationResources,
    marks: PreparedReference,
    scope: Option<&str>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&crate::Labels>,
) -> Result<PreparedInteriorInputs, Error> {
    let data = source
        .configured_source_rows()?
        .map_or_else(|| source.data(), |rows| source.data().map_source_rows(rows))
        .with_axis_resources(resources);
    let mut bins = None;
    let mut beeswarm = None;
    let mut waterfall = None;
    let mut heatmap = None;
    let mut marginal = None;
    let mut dumbbell = None;
    let mut estimate = None;
    let mut bars = None;
    let mut distribution = None;
    let mut interaction = None;
    let mut curves = None;
    let mut quantiles = None;
    let mut fit = None;
    let kind = match marks {
        PreparedReference::Waterfall(marks) => {
            waterfall = Some(marks);
            PlotKind::Waterfall
        }
        PreparedReference::Beeswarm(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.prepare_color_label(data, labels)?;
            }
            beeswarm = Some(marks);
            PlotKind::Beeswarm
        }
        PreparedReference::Marginal(marks) => {
            marginal = Some(crate::plots::marginal::finish(marks, x_limits, data)?);
            PlotKind::Marginal
        }
        PreparedReference::Heatmap(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.prepare_color_label(data, labels)?;
            }
            heatmap = Some(marks);
            PlotKind::Heatmap
        }
        PreparedReference::Interaction(marks) => {
            interaction = Some(marks);
            PlotKind::Interaction
        }
        PreparedReference::Distribution(kind, marks) => {
            distribution = Some(marks);
            kind
        }
        PreparedReference::Bars(kind, marks) => {
            bars = Some(marks);
            kind
        }
        PreparedReference::Estimate(marks) => {
            let kind = marks.kind;
            estimate = Some(marks);
            kind
        }
        PreparedReference::Dumbbell(marks) => {
            dumbbell = Some(marks);
            PlotKind::Dumbbell
        }
        PreparedReference::Histogram(marks) => {
            bins = Some(marks);
            PlotKind::Histogram
        }
        PreparedReference::Density(marks) => {
            curves = Some(marks);
            PlotKind::Density
        }
        PreparedReference::Qq(marks) => {
            quantiles = Some(marks);
            PlotKind::Qq
        }
        PreparedReference::Regression(marks) => {
            fit = Some(marks);
            PlotKind::Regression
        }
    };
    let route = InteriorRoute::classify(kind);
    route.validate(source)?;
    let axes = prepare(
        source,
        data,
        None,
        None,
        None,
        bins.as_ref(),
        curves.as_ref(),
        quantiles.as_ref(),
        fit.as_ref(),
        Some(ReferenceAxisConfig {
            waterfall: waterfall.as_ref(),
            beeswarm: beeswarm.as_ref(),
            marginal: marginal.as_ref(),
            heatmap: heatmap.as_ref(),
            interaction: interaction.as_ref(),
            distribution: distribution.as_ref(),
            bars: bars.as_ref(),
            x_limits,
            labels,
            dumbbell: dumbbell.as_ref(),
            estimate: estimate.as_ref(),
            estimate_fallback: None,
        }),
    )?;
    if let Some(quantiles) = &quantiles {
        quantiles.validate_ranges(axes.ranges, data)?;
    }
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges, data)?;
    let scope = || scope.map(|s| data.copy_source_string(s)).transpose();
    let mut retained = Vec::new();
    data.reserve_source(&mut retained, 1)?;
    retained.push((scope()?, axes));
    Ok(PreparedInteriorInputs {
        waterfall: waterfall
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::waterfall::PreparedWaterfallInputs { panels })
            })
            .transpose()?,
        beeswarm: beeswarm
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::beeswarm::PreparedBeeswarmInputs { panels })
            })
            .transpose()?,
        marginal: marginal.map(|marks| data.source_arc(marks)).transpose()?,
        heatmap: heatmap
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::heatmap::PreparedHeatmapInputs { panels })
            })
            .transpose()?,
        interaction: interaction
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::interaction::PreparedInteractionInputs { panels })
            })
            .transpose()?,
        distributions: distribution
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(
                    crate::plots::categorical_distribution::PreparedDistributionInputs { panels },
                )
            })
            .transpose()?,
        bars: bars
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::bar_data::PreparedBarInputs { panels })
            })
            .transpose()?,
        estimate: estimate
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::interval_estimate::PreparedEstimateInputs { panels })
            })
            .transpose()?,
        dumbbell: dumbbell
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::dumbbell::PreparedDumbbellInputs { panels })
            })
            .transpose()?,
        route,
        axes: data.source_arc(retained)?,
        matrix: None,
        scatter: None,
        line: None,
        histogram: bins
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::histogram::PreparedHistogramInputs { panels })
            })
            .transpose()?,
        density: curves
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::density::PreparedDensityInputs { panels })
            })
            .transpose()?,
        qq: quantiles
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::qq::PreparedQqInputs { panels })
            })
            .transpose()?,
        regression: fit
            .map(|marks| {
                let mut panels = Vec::new();
                data.reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                data.source_arc(crate::plots::regression::PreparedRegressionInputs { panels })
            })
            .transpose()?,
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "root preparation retains per-component scientific axes and admitted point/path semantics before fonts"
)]
pub(crate) fn prepare_plot(
    source: &Plot,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<PreparedInteriorInputs, Error> {
    let mut matrix = None;
    let mut retained = Vec::new();
    let mut scatter = Vec::new();
    let mut line = Vec::new();
    let mut histogram = Vec::new();
    let mut density = Vec::new();
    let mut qq = Vec::new();
    let mut regression = Vec::new();
    let mut dumbbell = Vec::new();
    let mut estimate = Vec::new();
    let mut bars = Vec::new();
    let mut distributions = Vec::new();
    let mut interaction = Vec::new();
    let mut heatmap = Vec::new();
    let mut beeswarm = Vec::new();
    let mut waterfall = Vec::new();
    let source_data = source
        .configured_source_rows()?
        .map_or_else(|| source.data(), |rows| source.data().map_source_rows(rows))
        .with_axis_resources(resources);
    if matches!(
        source.kind(),
        PlotKind::Histogram
            | PlotKind::Waterfall
            | PlotKind::Beeswarm
            | PlotKind::Heatmap
            | PlotKind::Marginal
            | PlotKind::Density
            | PlotKind::Qq
            | PlotKind::Regression
            | PlotKind::Dumbbell
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Boxplot
            | PlotKind::Violin
            | PlotKind::Raincloud
            | PlotKind::Interaction
    ) && !source.has_facets()
    {
        return prepare_reference_plot(
            source,
            resources,
            PreparedReference::new(source, source_data)?,
            source.axis_scope(),
            None,
            None,
        );
    }
    if source.kind() == PlotKind::Scatter && !source.has_facets() {
        return prepare_scatter_plot(
            source,
            resources,
            crate::plots::scatter::PreparedScatter::new(source, source_data)?,
        );
    }
    if matches!(
        source.kind(),
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
    ) && !source.has_facets()
    {
        return prepare_line_plot(
            source,
            resources,
            crate::plots::line::PreparedLine::new(source, source_data, None)?,
        );
    }
    if source.has_facets() {
        let row_topology = (InteriorRoute::classify(source.kind()).is_coordinate()
            || matches!(
                source.kind(),
                PlotKind::Heatmap | PlotKind::Beeswarm | PlotKind::Waterfall
            ))
        .then(|| source_data.admit_row_topology())
        .transpose()?;
        let panels = crate::orchestrator::text_census_panels(source, source_data)?;
        let interaction_parent = (source.kind() == PlotKind::Interaction)
            .then(|| {
                crate::data::observed_collectives(
                    source_data,
                    source.group_col(),
                    source.group_name(),
                    source.hue_col(),
                    source.hue_name(),
                    None,
                    "interaction",
                )
            })
            .transpose()?;
        let mut estimate_panels = if matches!(
            source.kind(),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
        ) {
            source_data.collect_source(panels.iter().map(|panel| {
                crate::plots::interval_estimate::PreparedEstimate::new(
                    source,
                    panel
                        .data()
                        .with_axis_resources(resources)
                        .with_row_topology(row_topology.as_ref()),
                    panel.significance_annotations(source_data)?,
                )
                .map(Some)
            }))?
        } else {
            Vec::new()
        };
        let mut estimate_fallback: Option<(f64, f64)> = None;
        if matches!(
            source.facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeY
        ) {
            for marks in estimate_panels.iter().flatten() {
                source_data.claim_source_visits(1)?;
                if let Some(range) = marks.range {
                    estimate_fallback = Some(
                        estimate_fallback
                            .map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                    );
                }
            }
        }
        let groups = source
            .group_col()
            .filter(|_| {
                matches!(
                    source.kind(),
                    PlotKind::Bar
                        | PlotKind::StackedBar
                        | PlotKind::HorizontalBar
                        | PlotKind::StackedHorizontalBar
                        | PlotKind::Boxplot
                        | PlotKind::Violin
                        | PlotKind::Raincloud
                )
            })
            .map(|column| {
                source_data.source_levels(&crate::data::strict_strings(
                    source_data,
                    column,
                    "facet graphical group",
                )?)
            })
            .transpose()?;
        let conditions = if source.kind() == PlotKind::Trajectory
            && source
                .x_col()
                .map(|column| {
                    source_data
                        .column(column)
                        .map(|column| !column.is_numeric())
                })
                .transpose()?
                .unwrap_or(false)
        {
            Some(
                source_data.source_levels(&crate::data::strict_normalized_strings(
                    source_data,
                    source.x_col().expect("categorical x"),
                    "trajectory",
                    "condition",
                )?)?,
            )
        } else {
            None
        };
        let hues = source
            .hue_col()
            .filter(|_| {
                matches!(
                    source.kind(),
                    PlotKind::Bar
                        | PlotKind::HorizontalBar
                        | PlotKind::StackedBar
                        | PlotKind::StackedHorizontalBar
                        | PlotKind::Boxplot
                        | PlotKind::Violin
                        | PlotKind::Raincloud
                )
            })
            .map(|column| {
                source_data.source_levels(&crate::data::strict_strings(
                    source_data,
                    column,
                    "facet hue",
                )?)
            })
            .transpose()?;
        source_data.reserve_source(&mut retained, panels.len())?;
        for (index, panel) in panels.into_iter().enumerate() {
            let data = panel
                .data()
                .with_axis_resources(resources)
                .with_row_topology(row_topology.as_ref());
            let facet = crate::data::FacetAxisSources {
                hues: hues.as_deref(),
                groups: groups.as_deref(),
                conditions: conditions.as_deref(),
            };
            let marks = if source.kind() == PlotKind::Scatter {
                Some(crate::plots::scatter::PreparedScatter::new(source, data)?)
            } else {
                None
            };
            let paths = if matches!(
                source.kind(),
                PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
            ) {
                Some(crate::plots::line::PreparedLine::new(
                    source,
                    data,
                    facet.conditions,
                )?)
            } else {
                None
            };
            let bins = (source.kind() == PlotKind::Histogram)
                .then(|| crate::plots::histogram::HistogramSemantics::new(source, data))
                .transpose()?;
            let curves = (source.kind() == PlotKind::Density)
                .then(|| crate::plots::density::PreparedDensity::new(source, data))
                .transpose()?;
            let quantiles = (source.kind() == PlotKind::Qq)
                .then(|| crate::plots::qq::PreparedQq::new(source, data))
                .transpose()?;
            let fit = (source.kind() == PlotKind::Regression)
                .then(|| crate::plots::regression::PreparedRegression::new(source, data))
                .transpose()?;
            let pairs = (source.kind() == PlotKind::Dumbbell)
                .then(|| crate::plots::dumbbell::prepare(source, data))
                .transpose()?;
            let estimates = estimate_panels.get_mut(index).and_then(Option::take);
            let heatmap_marks = (source.kind() == PlotKind::Heatmap)
                .then(|| crate::plots::heatmap::prepare(source, data))
                .transpose()?;
            let beeswarm_marks = (source.kind() == PlotKind::Beeswarm)
                .then(|| crate::plots::beeswarm::prepare(source, data))
                .transpose()?;
            let waterfall_marks = (source.kind() == PlotKind::Waterfall)
                .then(|| crate::plots::waterfall::prepare(source, data))
                .transpose()?;
            let interaction_marks = (source.kind() == PlotKind::Interaction)
                .then(|| {
                    crate::plots::interaction::prepare(source, data, interaction_parent.as_ref())
                })
                .transpose()?;
            let distribution_marks = matches!(
                source.kind(),
                PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
            )
            .then(|| {
                crate::plots::categorical_distribution::PreparedDistribution::new(
                    source,
                    data,
                    Some(facet),
                )
            })
            .transpose()?;
            let bar_marks = matches!(
                source.kind(),
                PlotKind::Bar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedBar
                    | PlotKind::StackedHorizontalBar
            )
            .then(|| {
                crate::plots::bar_data::prepare_scoped(
                    source,
                    data,
                    crate::plots::bar_data::mode(source),
                    Some(facet),
                    panel.significance_annotations(source_data)?,
                )
            })
            .transpose()?;
            let axes = prepare(
                source,
                data,
                Some(facet),
                marks.as_ref(),
                paths.as_ref(),
                bins.as_ref(),
                curves.as_ref(),
                quantiles.as_ref(),
                fit.as_ref(),
                Some(ReferenceAxisConfig {
                    waterfall: waterfall_marks.as_ref(),
                    beeswarm: beeswarm_marks.as_ref(),
                    marginal: None,
                    heatmap: heatmap_marks.as_ref(),
                    interaction: interaction_marks.as_ref(),
                    distribution: distribution_marks.as_ref(),
                    bars: bar_marks.as_ref(),
                    dumbbell: pairs.as_ref(),
                    estimate: estimates.as_ref(),
                    estimate_fallback,
                    ..ReferenceAxisConfig::default()
                }),
            )?;
            let scope = panel.scope(source_data)?;
            if let Some(marks) = waterfall_marks {
                source_data.reserve_source(&mut waterfall, 1)?;
                waterfall.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(marks) = beeswarm_marks {
                source_data.reserve_source(&mut beeswarm, 1)?;
                beeswarm.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(marks) = heatmap_marks {
                source_data.reserve_source(&mut heatmap, 1)?;
                heatmap.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(marks) = interaction_marks {
                source_data.reserve_source(&mut interaction, 1)?;
                interaction.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(marks) = distribution_marks {
                source_data.reserve_source(&mut distributions, 1)?;
                distributions.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(marks) = bar_marks {
                source_data.reserve_source(&mut bars, 1)?;
                bars.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            if let Some(estimates) = estimates {
                source_data.reserve_source(&mut estimate, 1)?;
                estimate.push((Some(source_data.copy_source_string(&scope)?), estimates));
            }
            if let Some(pairs) = pairs {
                source_data.reserve_source(&mut dumbbell, 1)?;
                dumbbell.push((Some(source_data.copy_source_string(&scope)?), pairs));
            }
            if let Some(quantiles) = quantiles {
                source_data.reserve_source(&mut qq, 1)?;
                qq.push((Some(source_data.copy_source_string(&scope)?), quantiles));
            }
            if let Some(fit) = fit {
                source_data.reserve_source(&mut regression, 1)?;
                regression.push((Some(source_data.copy_source_string(&scope)?), fit));
            }
            if let Some(bins) = bins {
                source_data.reserve_source(&mut histogram, 1)?;
                histogram.push((Some(source_data.copy_source_string(&scope)?), bins));
            }
            if let Some(curves) = curves {
                source_data.reserve_source(&mut density, 1)?;
                density.push((Some(source_data.copy_source_string(&scope)?), curves));
            }
            if let Some(paths) = paths {
                source_data.reserve_source(&mut line, 1)?;
                line.push((Some(source_data.copy_source_string(&scope)?), paths));
            }
            if let Some(marks) = marks {
                source_data.reserve_source(&mut scatter, 1)?;
                scatter.push((Some(source_data.copy_source_string(&scope)?), marks));
            }
            retained.push((Some(scope), axes));
        }
        for (side, fixed) in [
            matches!(
                source.facet_scales_spec(),
                FacetScales::Fixed | FacetScales::FreeY
            ),
            matches!(
                source.facet_scales_spec(),
                FacetScales::Fixed | FacetScales::FreeX
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if !fixed {
                continue;
            }
            reconcile_facet_axis(&mut retained, side, source_data)?;
        }
        for ((_, quantiles), (_, axes)) in qq.iter().zip(&retained) {
            quantiles.validate_ranges(axes.ranges, source_data)?;
        }
        for (_, axes) in &retained {
            if matches!(
                source.kind(),
                PlotKind::Bar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedBar
                    | PlotKind::StackedHorizontalBar
            ) {
                for side in 0..2 {
                    if let Some(ticks) = &axes.specs[side].exact_ticks {
                        source_data.claim_source_visits(ticks.len())?;
                        crate::layout::Axis::validate_ticks(
                            ticks,
                            axes.ranges[side],
                            if side == 0 { "x-axis" } else { "y-axis" },
                        )?;
                    }
                }
            }
            crate::orchestrator::layers::GuideSnapshot::validate_sources(
                source,
                axes.ranges,
                source_data,
            )?;
        }
    } else if source.kind() == PlotKind::Matrix {
        matrix =
            Some(source_data.source_arc(crate::plots::matrix::prepare_axes(source, source_data)?)?);
    } else {
        let axes = prepare(
            source,
            source_data,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )?;
        source_data.reserve_source(&mut retained, 1)?;
        retained.push((
            source
                .axis_scope()
                .map(|scope| source_data.copy_source_string(scope))
                .transpose()?,
            axes,
        ));
    }
    Ok(PreparedInteriorInputs {
        marginal: None,
        waterfall: if source.kind() == PlotKind::Waterfall {
            Some(
                source_data.source_arc(crate::plots::waterfall::PreparedWaterfallInputs {
                    panels: waterfall,
                })?,
            )
        } else {
            None
        },
        beeswarm: if source.kind() == PlotKind::Beeswarm {
            Some(
                source_data.source_arc(crate::plots::beeswarm::PreparedBeeswarmInputs {
                    panels: beeswarm,
                })?,
            )
        } else {
            None
        },
        heatmap: if source.kind() == PlotKind::Heatmap {
            Some(
                source_data
                    .source_arc(crate::plots::heatmap::PreparedHeatmapInputs { panels: heatmap })?,
            )
        } else {
            None
        },
        interaction: if source.kind() == PlotKind::Interaction {
            Some(
                source_data.source_arc(crate::plots::interaction::PreparedInteractionInputs {
                    panels: interaction,
                })?,
            )
        } else {
            None
        },
        distributions: if matches!(
            source.kind(),
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
        ) {
            Some(source_data.source_arc(
                crate::plots::categorical_distribution::PreparedDistributionInputs {
                    panels: distributions,
                },
            )?)
        } else {
            None
        },
        bars: if matches!(
            source.kind(),
            PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
        ) {
            Some(
                source_data
                    .source_arc(crate::plots::bar_data::PreparedBarInputs { panels: bars })?,
            )
        } else {
            None
        },
        estimate: if matches!(
            source.kind(),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
        ) {
            Some(source_data.source_arc(
                crate::plots::interval_estimate::PreparedEstimateInputs { panels: estimate },
            )?)
        } else {
            None
        },
        dumbbell: if source.kind() == PlotKind::Dumbbell {
            Some(
                source_data.source_arc(crate::plots::dumbbell::PreparedDumbbellInputs {
                    panels: dumbbell,
                })?,
            )
        } else {
            None
        },
        qq: if source.kind() == PlotKind::Qq {
            Some(source_data.source_arc(crate::plots::qq::PreparedQqInputs { panels: qq })?)
        } else {
            None
        },
        regression: if source.kind() == PlotKind::Regression {
            Some(
                source_data.source_arc(crate::plots::regression::PreparedRegressionInputs {
                    panels: regression,
                })?,
            )
        } else {
            None
        },
        histogram: if source.kind() == PlotKind::Histogram {
            Some(
                source_data.source_arc(crate::plots::histogram::PreparedHistogramInputs {
                    panels: histogram,
                })?,
            )
        } else {
            None
        },
        density: if source.kind() == PlotKind::Density {
            Some(
                source_data
                    .source_arc(crate::plots::density::PreparedDensityInputs { panels: density })?,
            )
        } else {
            None
        },
        route: InteriorRoute::classify(source.kind()),
        axes: source_data.source_arc(retained)?,
        matrix,
        line: if matches!(
            source.kind(),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
        ) {
            Some(source_data.source_arc(crate::plots::line::PreparedLineInputs { panels: line })?)
        } else {
            None
        },
        scatter: if source.kind() == PlotKind::Scatter {
            Some(
                source_data
                    .source_arc(crate::plots::scatter::PreparedScatterInputs { panels: scatter })?,
            )
        } else {
            None
        },
    })
}

/// Complete one already prepared Scatter component after root range coordination.
pub(crate) fn prepare_scatter_plot(
    source: &Plot,
    resources: &crate::font::canonical::OperationGenerationResources,
    marks: crate::plots::scatter::PreparedScatter,
) -> Result<PreparedInteriorInputs, Error> {
    InteriorRoute::classify(PlotKind::Scatter).validate(source)?;
    let data = source
        .configured_source_rows()?
        .map_or_else(|| source.data(), |rows| source.data().map_source_rows(rows))
        .with_axis_resources(resources);
    let axes = prepare(
        source,
        data,
        None,
        Some(&marks),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges, data)?;
    let scope = || {
        source
            .axis_scope()
            .map(|scope| data.copy_source_string(scope))
            .transpose()
    };
    let mut retained_axes = Vec::new();
    data.reserve_source(&mut retained_axes, 1)?;
    retained_axes.push((scope()?, axes));
    let mut panels = Vec::new();
    data.reserve_source(&mut panels, 1)?;
    panels.push((scope()?, marks));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(PlotKind::Scatter),
        beeswarm: None,
        waterfall: None,
        marginal: None,
        heatmap: None,
        bars: None,
        interaction: None,
        distributions: None,
        estimate: None,
        dumbbell: None,
        qq: None,
        regression: None,
        histogram: None,
        density: None,
        axes: data.source_arc(retained_axes)?,
        matrix: None,
        line: None,
        scatter: Some(data.source_arc(crate::plots::scatter::PreparedScatterInputs { panels })?),
    })
}

/// Complete one retained line component after the parent's range coordination.
pub(crate) fn prepare_line_plot(
    source: &Plot,
    resources: &crate::font::canonical::OperationGenerationResources,
    marks: crate::plots::line::PreparedLine,
) -> Result<PreparedInteriorInputs, Error> {
    let data = source
        .configured_source_rows()?
        .map_or_else(|| source.data(), |rows| source.data().map_source_rows(rows))
        .with_axis_resources(resources);
    let axes = prepare(
        source,
        data,
        None,
        None,
        Some(&marks),
        None,
        None,
        None,
        None,
        None,
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges, data)?;
    let scope = || {
        source
            .axis_scope()
            .map(|scope| data.copy_source_string(scope))
            .transpose()
    };
    let mut retained_axes = Vec::new();
    data.reserve_source(&mut retained_axes, 1)?;
    retained_axes.push((scope()?, axes));
    let mut panels = Vec::new();
    data.reserve_source(&mut panels, 1)?;
    panels.push((scope()?, marks));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(source.kind()),
        beeswarm: None,
        waterfall: None,
        marginal: None,
        heatmap: None,
        bars: None,
        interaction: None,
        distributions: None,
        estimate: None,
        dumbbell: None,
        axes: data.source_arc(retained_axes)?,
        histogram: None,
        qq: None,
        regression: None,
        density: None,
        matrix: None,
        scatter: None,
        line: Some(data.source_arc(crate::plots::line::PreparedLineInputs { panels })?),
    })
}

fn reconcile_facet_axis(
    retained: &mut [(Option<String>, PreparedAxes)],
    side: usize,
    data: PlotData<'_>,
) -> Result<(), Error> {
    let Some((_, first)) = retained.first() else {
        return Ok(());
    };
    for (_, axes) in retained.iter() {
        data.claim_source_visits(1)?;
        if let (Some(a), Some(b)) = (&axes.categories[side], &first.categories[side]) {
            for (a, b) in a.iter().zip(b) {
                data.claim_source_visits(1)?;
                data.claim_source_visits(a.label.len().min(b.label.len()))?;
            }
        }
        if axes.categories[side] != first.categories[side] {
            return Err(Error::Render(format!("fixed facet {} axis requires identical categorical levels and ordering in every panel; use a free scale or provide complete panel data", if side == 0 { "x" } else { "y" })));
        }
    }
    if first.categories[side].is_none() {
        data.claim_source_visits(retained.len())?;
        let union = retained
            .iter()
            .map(|(_, axes)| axes.ranges[side])
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, b| {
                (a.0.min(b.0), a.1.max(b.1))
            });
        validate_range(union)?;
        data.claim_source_visits(retained.len())?;
        for (_, axes) in retained {
            axes.ranges[side] = union;
        }
    }
    Ok(())
}

fn validate_range(range: (f64, f64)) -> Result<(), Error> {
    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        return Err(Error::Data(
            "axis domain must be finite and strictly ascending".into(),
        ));
    }
    Ok(())
}

#[expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "closed scientific axis dispatch; category lengths are bounded source rows"
)]
fn prepare(
    chart: &Plot,
    data: PlotData<'_>,
    facet: Option<crate::data::FacetAxisSources<'_>>,
    scatter: Option<&crate::plots::scatter::PreparedScatter>,
    line: Option<&crate::plots::line::PreparedLine>,
    histogram: Option<&crate::plots::histogram::HistogramSemantics>,
    density: Option<&crate::plots::density::PreparedDensity>,
    qq: Option<&crate::plots::qq::PreparedQq>,
    regression: Option<&crate::plots::regression::PreparedRegression>,
    reference_config: Option<ReferenceAxisConfig<'_>>,
) -> Result<PreparedAxes, Error> {
    // Two scalar domain results and their fixed five-sample nice-range seeds.
    data.claim_source_visits(12)?;
    #[cfg(test)]
    super::AXIS_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let mut categories: [Option<Vec<String>>; 2] = [None, None];
    let mut ranges = match chart.kind() {
        PlotKind::Regression => {
            let ranges = regression
                .ok_or_else(|| Error::Render("Regression axes require prepared semantics".into()))?
                .ranges;
            [
                chart.x_axis_spec().limits.unwrap_or(ranges[0]),
                chart.y_axis_spec().limits.unwrap_or(ranges[1]),
            ]
        }
        PlotKind::Scatter => {
            let ranges = scatter
                .ok_or_else(|| {
                    Error::Render("Scatter axis inputs require prepared semantics".into())
                })?
                .ranges;
            [
                crate::layout::continuous_auto_range(
                    ranges[0],
                    chart.x_axis_spec(),
                    data.source_resources()?,
                )?,
                crate::layout::continuous_auto_range(
                    ranges[1],
                    chart.y_axis_spec(),
                    data.source_resources()?,
                )?,
            ]
        }
        PlotKind::Histogram => {
            let ranges = histogram
                .ok_or_else(|| Error::Render("Histogram axes require prepared semantics".into()))?
                .ranges;
            [
                chart.x_axis_spec().limits.unwrap_or(ranges[0]),
                chart.y_axis_spec().limits.unwrap_or(ranges[1]),
            ]
        }
        PlotKind::Density => density
            .ok_or_else(|| Error::Render("Density axes require prepared semantics".into()))?
            .ranges(chart, data)?,
        PlotKind::Qq => qq
            .ok_or_else(|| Error::Render("Qq axes require prepared semantics".into()))?
            .ranges(chart, data)?,
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let line = line.ok_or_else(|| {
                Error::Render("line axis inputs require prepared semantics".into())
            })?;
            categories[0] = line
                .conditions
                .as_ref()
                .map(|levels| {
                    data.collect_source(levels.iter().map(|label| data.copy_source_string(label)))
                })
                .transpose()?;
            [
                chart.x_axis_spec().limits.unwrap_or(line.ranges[0]),
                chart.y_axis_spec().limits.unwrap_or(line.ranges[1]),
            ]
        }
        PlotKind::Marginal => {
            reference_config
                .as_ref()
                .and_then(|config| config.marginal)
                .ok_or_else(|| Error::Render("marginal axes require retained semantics".into()))?
                .ranges
        }
        PlotKind::Bar
        | PlotKind::StackedBar
        | PlotKind::HorizontalBar
        | PlotKind::StackedHorizontalBar => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.bars)
                .ok_or_else(|| Error::Render("bar axes require prepared semantics".into()))?;
            let levels = data.collect_source(
                marks
                    .categories
                    .iter()
                    .map(|label| data.copy_source_string(label)),
            )?;
            let range = marks.value_range;
            let horizontal = matches!(
                chart.kind(),
                PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
            );
            let slots = (0.0, levels.len() as f64);
            categories[usize::from(horizontal)] = Some(levels);
            let values = crate::layout::anchored_auto_range(
                range,
                if horizontal {
                    chart.x_axis_spec()
                } else {
                    chart.y_axis_spec()
                },
                0.0,
                data.source_resources()?,
            )?;
            if horizontal {
                [values, slots]
            } else {
                [slots, values]
            }
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.distribution)
                .ok_or_else(|| {
                    Error::Render("categorical distribution axes require retained semantics".into())
                })?;
            let levels = data.collect_source(
                marks
                    .categories()
                    .iter()
                    .map(|label| data.copy_source_string(label)),
            )?;
            let values = marks.range();
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(
                    values,
                    chart.y_axis_spec(),
                    data.source_resources()?,
                )?,
            ]
        }
        PlotKind::Dumbbell => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.dumbbell)
                .ok_or_else(|| Error::Render("Dumbbell axes require prepared semantics".into()))?;
            categories[1] = Some(
                data.collect_source(
                    marks
                        .categories
                        .iter()
                        .map(|label| data.copy_source_string(label)),
                )?,
            );
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.x_axis_spec(),
                    data.source_resources()?,
                )?,
                (0.0, marks.categories.len() as f64),
            ]
        }
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let config = reference_config.as_ref().ok_or_else(|| {
                Error::Render("estimate axes require root-prepared semantics".into())
            })?;
            let marks = config
                .estimate
                .ok_or_else(|| Error::Render("estimate axes require retained rows".into()))?;
            categories[1] = Some(
                data.collect_source(
                    marks
                        .labels
                        .iter()
                        .map(|label| data.copy_source_string(label)),
                )?,
            );
            let range = marks
                .range
                .or(config.estimate_fallback)
                .map_or_else(|| marks.x_range(), Ok)?;
            [range, (0.0, marks.labels.len() as f64)]
        }
        PlotKind::Interaction => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.interaction)
                .ok_or_else(|| {
                    Error::Render("interaction axes require retained semantics".into())
                })?;
            let levels = data.collect_source(
                marks
                    .conditions
                    .iter()
                    .map(|level| data.copy_source_string(level)),
            )?;
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.y_axis_spec(),
                    data.source_resources()?,
                )?,
            ]
        }
        PlotKind::Beeswarm => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.beeswarm)
                .ok_or_else(|| Error::Render("beeswarm axes require retained semantics".into()))?;
            let levels = data.collect_source(
                marks
                    .levels
                    .iter()
                    .map(|level| data.copy_source_string(level)),
            )?;
            let slots = chart
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.x_data,
                    chart.x_axis_spec(),
                    data.source_resources()?,
                )?,
                slots,
            ]
        }
        PlotKind::Waterfall => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.waterfall)
                .ok_or_else(|| Error::Render("waterfall axes require retained semantics".into()))?;
            let levels = data.collect_source(
                marks
                    .labels
                    .iter()
                    .map(|label| data.copy_source_string(label)),
            )?;
            let slots = chart
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.x_axis_spec(),
                    data.source_resources()?,
                )?,
                slots,
            ]
        }
        PlotKind::Heatmap => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.heatmap)
                .ok_or_else(|| Error::Render("heatmap axes require retained semantics".into()))?;
            let x =
                data.collect_source(marks.x_levels.iter().map(|s| data.copy_source_string(s)))?;
            let y =
                data.collect_source(marks.y_levels.iter().map(|s| data.copy_source_string(s)))?;
            let ranges = [(0.0, x.len() as f64), (0.0, y.len() as f64)];
            categories = [Some(x), Some(y)];
            ranges
        }
        PlotKind::Matrix => {
            return Err(Error::Render(
                "Matrix has no single panel axis owner".into(),
            ))
        }
    };
    if let Some(shared) = chart.resolved_facet_x_range() {
        ranges[0] = shared;
    }
    let ReferenceAxisConfig {
        x_limits, labels, ..
    } = reference_config.unwrap_or_default();
    if let Some(limits) = x_limits {
        ranges[0] = limits;
    }
    let logarithmic =
        chart.kind() == PlotKind::Forest && chart.axis_scale_spec() == AxisScale::Log10;
    let mut specs = [AxisSpec::default(), AxisSpec::default()];
    for (side, source) in [chart.x_axis_spec(), chart.y_axis_spec()]
        .into_iter()
        .enumerate()
    {
        specs[side] = AxisSpec {
            provenance: source.provenance,
            limits: if side == 0 {
                x_limits.or(source.limits)
            } else {
                source.limits
            },
            interval: source.interval,
            exact_ticks: if logarithmic && side == 0 {
                chart.projected_explicit_x_ticks(data)?
            } else {
                source
                    .exact_ticks
                    .as_ref()
                    .map(|ticks| {
                        data.collect_source(ticks.iter().map(|tick| {
                            Ok(AxisTick::new(
                                tick.position,
                                data.copy_source_string(&tick.label)?,
                            ))
                        }))
                    })
                    .transpose()?
            },
        };
    }
    if logarithmic {
        // Both explicit and parent-coordinated domains are already transformed.
        specs[0].limits = specs[0].limits.map(|_| ranges[0]);
    }
    let mut ticks = [None, None];
    let mut display_categories = [None, None];
    for side in 0..2 {
        validate_range(ranges[side])?;
        if let Some(levels) = categories[side].take() {
            let domain = if side == 0 {
                "x-axis categories"
            } else {
                "y-axis categories"
            };
            let raw_labels = levels.iter().map(String::as_str);
            display_categories[side] = Some(if let Some(policy) = labels {
                policy.resolve_domain_source(domain, raw_labels, Some(data))?
            } else {
                chart.resolve_axis_label_domain(domain, raw_labels, data)?
            });
            let reverse = side == 1
                && matches!(
                    chart.kind(),
                    PlotKind::PointInterval
                        | PlotKind::Coefficient
                        | PlotKind::Forest
                        | PlotKind::Dumbbell
                        | PlotKind::Heatmap
                );
            let count = levels.len() as f64;
            let mut retained = Vec::new();
            data.reserve_source(&mut retained, levels.len())?;
            for (index, label) in levels.into_iter().enumerate() {
                retained.push(AxisTick::new(
                    if reverse {
                        count - index as f64 - 0.5
                    } else {
                        index as f64 + 0.5
                    },
                    label,
                ));
            }
            ticks[side] = Some(retained);
        }
        let spec = &specs[side];
        if let Some(exact) = &spec.exact_ticks {
            if facet.is_some()
                && matches!(
                    chart.kind(),
                    PlotKind::Bar
                        | PlotKind::HorizontalBar
                        | PlotKind::StackedBar
                        | PlotKind::StackedHorizontalBar
                )
            {
                continue;
            }
            data.claim_source_visits(exact.len())?;
            crate::layout::Axis::validate_ticks(
                exact,
                ranges[side],
                if side == 0 { "x-axis" } else { "y-axis" },
            )?;
        }
    }
    let labels = if facet.is_some() {
        [String::new(), String::new()]
    } else {
        chart.prepare_axis_labels(data, labels)?
    };
    Ok(PreparedAxes {
        intrinsics: std::sync::OnceLock::new(),
        heatmap_intrinsics: std::sync::OnceLock::new(),
        ranges,
        categories: ticks,
        display_categories,
        labels,
        specs,
        logarithmic,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BarInput, CategoryValues, FacetSpec, NumericValues};

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "all extracted facet families validate final shared ranges before an unavailable font census"
    )]
    fn facet_guides_validate_reconciled_ranges_before_unavailable_fonts() {
        fn plots(scales: FacetScales, guide: crate::GuideSpec) -> [Plot; 4] {
            let x = NumericValues::new("x", [0.0, 1.0, 10.0, 11.0]);
            let y = NumericValues::new("y", [0.0, 1.0, 10.0, 11.0]);
            let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
            let facet = FacetSpec::column(
                CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            )
            .scales(scales);
            [
                Plot::scatter(x.clone(), y.clone())
                    .guide(guide.clone())
                    .facet(facet.clone())
                    .erase(),
                Plot::line(x.clone(), y.clone())
                    .guide(guide.clone())
                    .facet(facet.clone())
                    .erase(),
                Plot::trajectory(x.clone(), y.clone(), group.clone())
                    .guide(guide.clone())
                    .facet(facet.clone())
                    .erase(),
                Plot::line_path(x, y, group)
                    .guide(guide)
                    .facet(facet)
                    .erase(),
            ]
        }
        fn prepare_with_unavailable_fonts(plot: &Plot) -> Result<(), Error> {
            let mut theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            theme.typography.tick_label.stack =
                crate::font::canonical::named_stack(&["Lichtung T5a6a3a Unavailable Font 8c42e1"]);
            crate::layout::plan_exact_root(
                640,
                480,
                |stage| {
                    // Exercise the root boundary, independently of the current public
                    // restriction on combining supplemental guides with faceting.
                    prepare_plot(plot, &stage.generation_resources())?;
                    let requirements = crate::layout::foundational_plot_requirements(
                        None,
                        plot,
                        plot.kind(),
                        &theme,
                        &crate::layout::ComponentAddress::root(),
                        true,
                        true,
                        true,
                        Some(stage),
                    )?;
                    let mut presence = stage.reserve_scalar_presence_map()?;
                    let mut census = crate::layout::PreparedTextCensus::materialize(
                        requirements,
                        Some(stage),
                        Some(&mut presence),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                    theme.prepare_font_environment_from_census(&mut census, presence, stage)
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            )
            .map(|_| ())
        }
        let style = crate::GuideStyle::new((0, 0, 0), 1.0, crate::LinePattern::Solid).unwrap();
        for vertical in [false, true] {
            let guide = |value| {
                if vertical {
                    crate::GuideSpec::vertical(value, style.clone())
                } else {
                    crate::GuideSpec::horizontal(value, style.clone())
                }
            };
            for scales in [
                FacetScales::Fixed,
                FacetScales::FreeX,
                FacetScales::FreeY,
                FacetScales::Free,
            ] {
                for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 100.0] {
                    for plot in plots(scales, guide(value)) {
                        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
                        let error = prepare_with_unavailable_fonts(&plot).unwrap_err();
                        assert!(matches!(error, Error::Data(_)), "{error}");
                        assert!(error.to_string().contains("guide"), "{error}");
                        assert_eq!(
                            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                            0
                        );
                    }
                }
                for plot in plots(scales, guide(5.0)) {
                    let resources = crate::font::canonical::OperationGenerationResources::new();
                    let shared = scales == FacetScales::Fixed
                        || if vertical {
                            scales == FacetScales::FreeY
                        } else {
                            scales == FacetScales::FreeX
                        };
                    let prepared = prepare_plot(&plot, &resources);
                    if shared {
                        let prepared = prepared.unwrap();
                        for (_, axes) in prepared.axes.iter() {
                            let range = axes.ranges[usize::from(!vertical)];
                            assert!(range.0 <= 0.0 && range.1 >= 11.0);
                        }
                    } else {
                        assert!(matches!(prepared, Err(Error::Data(_))));
                    }
                }
            }
            // A valid shared guide reaches the unavailable-font error, proving
            // the earlier assertions are not simply bypassing font preparation.
            let [plot, ..] = plots(FacetScales::Fixed, guide(5.0));
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let error = prepare_with_unavailable_fonts(&plot).unwrap_err();
            assert!(matches!(error, Error::Render(_)), "{error}");
            assert!(error.to_string().contains("font"), "{error}");
            assert!(crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get) > 0);
        }
    }

    #[test]
    fn density_explicit_endpoint_ticks_use_display_limits_not_kde_support() {
        let natural = Plot::density(NumericValues::new("sample", [1.0, 2.0, 3.0]));
        let plot = natural
            .clone()
            .x_limits(-100.0, 100.0)
            .x_ticks(vec![
                AxisTick::new(-100.0, "-100"),
                AxisTick::new(100.0, "100"),
            ])
            .erase();
        let natural = natural.erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let prepared = prepare_plot(&plot, &resources).unwrap();
        assert_eq!(prepared.axes[0].1.ranges[0], (-100.0, 100.0));
        assert_eq!(
            crate::plots::density::axis_ranges(
                &natural,
                natural.data().with_axis_resources(&resources)
            )
            .unwrap()[1],
            prepared.axes[0].1.ranges[1]
        );
        assert_ne!(
            crate::plots::density::axis_ranges(
                &natural,
                natural.data().with_axis_resources(&resources)
            )
            .unwrap()[0],
            prepared.axes[0].1.ranges[0]
        );
        plot.preflight().unwrap();
    }

    #[test]
    fn retention_boundaries_share_the_operation_without_retaining_a_plot_clone() {
        use crate::font::canonical::OperationGenerationResources;
        let scatter = Plot::scatter(
            NumericValues::new("x", [1.0, 2.0, 3.0]),
            NumericValues::new("y", [2.0, 1.0, 4.0]),
        )
        .erase();
        let facets = Plot::scatter(
            NumericValues::new("x", [1.0, 2.0, 30.0]),
            NumericValues::new("y", [2.0, 1.0, 40.0]),
        )
        .facet(FacetSpec::column(
            CategoryValues::new("panel", ["a", "a", "b"]).unwrap(),
        ))
        .erase();
        let matrix = Plot::matrix(vec![
            NumericValues::new("first", [1.0, 2.0, 3.0]),
            NumericValues::new("second", [2.0, 1.0, 4.0]),
        ])
        .erase();
        for source in [scatter, facets, matrix] {
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let count = std::sync::Arc::strong_count(&source.prepared_axes);
            let resources = OperationGenerationResources::new();
            let retained = prepare_plot(&source, &resources).unwrap();
            assert_eq!(std::sync::Arc::strong_count(&source.prepared_axes), count);
            let bytes = resources.byte_usage().unwrap();
            let visits = resources.usage().unwrap().1;
            for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
                let resources = OperationGenerationResources::with_test_limits(visits, bytes);
                assert!(matches!(
                    prepare_plot(&source, &resources),
                    Err(Error::Render(_))
                ));
            }
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            prepare_plot(&source, &resources).unwrap();
            // Unused text and policy values stay borrowed, not copied and charged.
            let large = "unused".repeat(10_000);
            let decorated = source
                .clone()
                .title(large.clone())
                .caption(large.clone())
                .labels(crate::Labels::humanized().alias("unreferenced", large));
            let resources = OperationGenerationResources::with_test_limits(u128::MAX, bytes);
            prepare_plot(&decorated, &resources).unwrap();
            let attached = retained.attach(source).unwrap();
            assert!(std::sync::Arc::ptr_eq(
                &attached.prepared_axes,
                &retained.axes
            ));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn fixed_facet_comparisons_and_union_refuse_before_their_work_or_mutation() {
        use crate::font::canonical::OperationGenerationResources;
        let source = Plot::scatter(
            NumericValues::new("x", [1.0]),
            NumericValues::new("y", [1.0]),
        )
        .erase();
        let axes = |range, label: Option<&str>| PreparedAxes {
            intrinsics: std::sync::OnceLock::new(),
            heatmap_intrinsics: std::sync::OnceLock::new(),
            ranges: [range, (0.0, 1.0)],
            categories: [label.map(|label| vec![AxisTick::new(0.5, label)]), None],
            display_categories: [None, None],
            labels: [String::new(), String::new()],
            specs: [AxisSpec::default(), AxisSpec::default()],
            logarithmic: false,
        };
        for visits in [0, 2, 4, 5] {
            let mut retained = [
                (None, axes((0.0, 1.0), None)),
                (None, axes((2.0, 3.0), None)),
            ];
            let resources = OperationGenerationResources::with_test_limits(visits, 0);
            assert!(reconcile_facet_axis(
                &mut retained,
                0,
                source.data().with_axis_resources(&resources)
            )
            .is_err());
            assert_eq!(
                retained.map(|(_, axes)| axes.ranges[0]),
                [(0.0, 1.0), (2.0, 3.0)]
            );
        }
        let mut retained = [
            (None, axes((0.0, 1.0), None)),
            (None, axes((2.0, 3.0), None)),
        ];
        let resources = OperationGenerationResources::with_test_limits(6, 0);
        reconcile_facet_axis(
            &mut retained,
            0,
            source.data().with_axis_resources(&resources),
        )
        .unwrap();
        assert_eq!(retained.map(|(_, axes)| axes.ranges[0]), [(0.0, 3.0); 2]);
        for (visits, expected) in [
            (0, "work admission"),
            (9, "work admission"),
            (10, "identical categorical"),
        ] {
            let mut retained = [
                (None, axes((0.0, 1.0), Some("one"))),
                (None, axes((0.0, 1.0), Some("two"))),
            ];
            let resources = OperationGenerationResources::with_test_limits(visits, 0);
            let error = reconcile_facet_axis(
                &mut retained,
                0,
                source.data().with_axis_resources(&resources),
            )
            .unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn p2_facets_admit_original_mark_rows_before_selectors_and_preserve_source_ids() {
        let facet = crate::FacetSpec::column(
            CategoryValues::new("panel", ["left", "right", "left", "right"]).unwrap(),
        )
        .scales(FacetScales::Free);
        let rows = crate::SourceRows::new([100, 200, 300, 400]).unwrap();
        let sources = [
            Plot::scatter(
                NumericValues::nullable("x", [Some(1.0), None, Some(3.0), Some(4.0)]),
                NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
            )
            .facet(facet.clone())
            .source_rows(rows.clone())
            .erase(),
            Plot::point_interval(
                CategoryValues::new("term", ["a", "b", "c", "d"]).unwrap(),
                NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
                crate::Bounds::new(
                    NumericValues::new("lower", [0.0, 1.0, 2.0, 3.0]),
                    NumericValues::new("upper", [2.0, 3.0, 4.0, 5.0]),
                ),
            )
            .facet(facet)
            .source_rows(rows)
            .erase(),
        ];
        for source in sources {
            for available in [3, 4, 5] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                resources
                    .claim_topology_slots(1_000_000 - available)
                    .unwrap();
                crate::data::SOURCE_COLUMN_READS.with(|count| count.set(0));
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
                let result = prepare_plot(&source, &resources);
                if available < 4 {
                    let error = result
                        .err()
                        .expect("topology refusal before selector preparation");
                    assert!(error.to_string().contains("prepared topology slots"));
                    assert_eq!(resources.usage().unwrap(), (0, 0));
                    assert_eq!(resources.byte_usage().unwrap(), 0);
                    assert_eq!(resources.topology_usage(), 999_997);
                    crate::data::SOURCE_COLUMN_READS.with(|count| assert_eq!(count.get(), 0));
                } else {
                    let prepared = result.unwrap();
                    assert_eq!(prepared.axes.len(), 2);
                    assert_eq!(resources.topology_usage(), 1_000_000 - available + 4);
                    let panels = crate::orchestrator::text_census_panels(
                        &source,
                        source.data().with_axis_resources(&resources),
                    )
                    .unwrap();
                    assert_eq!(
                        panels
                            .iter()
                            .map(|panel| (0..panel.data().height())
                                .map(|row| panel.data().source_row(row))
                                .collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        [vec![100, 300], vec![200, 400]]
                    );
                    assert_eq!(resources.topology_usage(), 1_000_000 - available + 4);
                }
                crate::font::canonical::FONT_DISCOVERY_CALLS
                    .with(|count| assert_eq!(count.get(), 0));
            }
        }
    }

    #[test]
    fn borrowed_facet_panels_and_scope_sources_refuse_before_materialization() {
        use crate::font::canonical::OperationGenerationResources;
        let source = Plot::bar(
            CategoryValues::new("category", ["a", "b", "a"]).unwrap(),
            BarInput::estimates(NumericValues::new("value", [1.0, 2.0, 3.0])),
        )
        .facet(
            FacetSpec::column(CategoryValues::new("panel", ["left", "right", "left"]).unwrap())
                .scales(FacetScales::Free),
        )
        .erase();
        for (visits, bytes) in [(0, u64::MAX), (u128::MAX, 0)] {
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(crate::orchestrator::text_census_panels(
                &source,
                source.data().with_axis_resources(&resources)
            )
            .is_err());
        }
        let resources = OperationGenerationResources::new();
        let panels = crate::orchestrator::text_census_panels(
            &source,
            source.data().with_axis_resources(&resources),
        )
        .unwrap();
        assert_eq!(panels.len(), 2);
        for (panel, (scope, rows)) in panels.iter().zip([
            ("facet panel=left", vec![0, 2]),
            ("facet panel=right", vec![1]),
        ]) {
            assert!(std::ptr::eq(panel.plot(), std::ptr::from_ref(&source)));
            assert_eq!(
                (0..panel.data().height())
                    .map(|row| panel.data().source_row(row))
                    .collect::<Vec<_>>(),
                rows
            );
            assert_eq!(panel.scope(source.data()).unwrap(), scope);
            let copied = OperationGenerationResources::with_test_limits(0, u64::MAX);
            assert_eq!(
                panel
                    .scope(source.data().with_axis_resources(&copied))
                    .unwrap(),
                scope
            );
            assert_eq!(copied.usage().unwrap(), (1, 0));
            for (records, bytes) in [(0, u64::MAX), (1, 0)] {
                let resources = OperationGenerationResources::with_test_limits(0, bytes);
                resources.claim_slots(1_000_000 - records).unwrap();
                assert!(panel
                    .scope(source.data().with_axis_resources(&resources))
                    .is_err());
            }
        }
    }

    #[test]
    fn borrowed_facet_axis_views_preserve_bootstrap_and_global_conditions() {
        let source = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a", "a", "a", "b", "b"]).unwrap(),
            BarInput::samples(NumericValues::new(
                "value",
                [1.0, 4.0, 2.0, 8.0, 3.0, 7.0, 5.0, 9.0],
            )),
        )
        .series(
            CategoryValues::new("series", ["g1", "g1", "g2", "g2", "g2", "g2", "g2", "g2"])
                .unwrap(),
        )
        .facet(
            FacetSpec::column(
                CategoryValues::new(
                    "panel",
                    [
                        "left", "left", "left", "left", "right", "right", "right", "right",
                    ],
                )
                .unwrap(),
            )
            .scales(FacetScales::Free),
        )
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let retained = prepare_plot(&source, &resources).unwrap();
        let panels = crate::orchestrator::text_census_panels(&source, source.data()).unwrap();
        for (panel, (scope, axes)) in panels.iter().zip(retained.axes.iter()) {
            let child = source
                .clone()
                .without_facets()
                .with_global_group_levels(vec!["g1".to_owned(), "g2".to_owned()]);
            let expected = crate::plots::bar_data::prepare(
                &child,
                panel.data(),
                crate::plots::bar_data::BarMode::Grouped,
            )
            .unwrap();
            assert_eq!(
                scope.as_deref(),
                Some(panel.scope(source.data()).unwrap().as_str())
            );
            assert_eq!(
                axes.ranges[1],
                crate::layout::anchored_auto_range(
                    expected.value_range,
                    child.y_axis_spec(),
                    0.0,
                    &crate::font::canonical::OperationGenerationResources::new()
                )
                .unwrap()
            );
            assert_eq!(
                axes.categories[0]
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                expected.categories
            );
        }
        let trajectory = Plot::trajectory(
            CategoryValues::new("condition", ["before", "after", "before", "later"]).unwrap(),
            NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
            CategoryValues::new("subject", ["one", "one", "two", "two"]).unwrap(),
        )
        .facet(
            FacetSpec::column(
                CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            )
            .scales(FacetScales::Free),
        )
        .erase();
        let retained = prepare_plot(&trajectory, &resources).unwrap();
        for (_, axes) in retained.axes.iter() {
            assert_eq!(axes.ranges[0], (0.0, 3.0));
            assert_eq!(
                axes.categories[0]
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                ["before", "after", "later"]
            );
            assert_eq!(axes.labels, ["", ""]);
        }
    }

    #[test]
    fn invalid_category_and_log_sources_reject_before_discovery() {
        let duplicate = CategoryValues::new("term", ["a", "a"]).unwrap();
        let values = NumericValues::new("estimate", [1.0, 2.0]);
        let bounds = || {
            crate::Bounds::new(
                NumericValues::new("lower", [0.5, 1.5]),
                NumericValues::new("upper", [1.5, 2.5]),
            )
        };
        let plots = [
            Plot::matrix(vec![
                NumericValues::new("constant", [1.0, 1.0]),
                NumericValues::new("variable", [1.0, 2.0]),
            ])
            .erase(),
            Plot::coefficient(
                duplicate.clone(),
                crate::CoefficientInput::estimates(values.clone()),
            )
            .erase(),
            Plot::forest(duplicate.clone(), values.clone(), bounds(), 1.0).erase(),
            Plot::point_interval(duplicate, values, bounds()).erase(),
            Plot::forest(
                CategoryValues::new("term", ["a", "b"]).unwrap(),
                NumericValues::new("estimate", [1.0, 2.0]),
                bounds(),
                1.0,
            )
            .scale(AxisScale::Log10)
            .x_ticks(vec![AxisTick::new(1000.0, "outside")])
            .erase(),
            Plot::forest(
                CategoryValues::new("term", ["a", "b"]).unwrap(),
                NumericValues::new("estimate", [0.0, 2.0]),
                crate::Bounds::new(
                    NumericValues::new("lower", [-1.0, 1.0]),
                    NumericValues::new("upper", [1.0, 3.0]),
                ),
                1.0,
            )
            .scale(AxisScale::Log10)
            .x_limits(0.1, 10.0)
            .erase(),
        ];
        for plot in plots {
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(plot.preflight().is_err());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn mixed_facet_policies_retain_actual_numeric_unions_and_local_categories() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let categories = CategoryValues::new("category", ["a", "b", "a", "c"]).unwrap();
        let facets = CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap();
        let plot = Plot::bar(
            categories,
            BarInput::estimates(NumericValues::new("value", [1.0, 2.0, 100.0, 200.0])),
        );
        let free = plot
            .clone()
            .facet(FacetSpec::column(facets.clone()).scales(FacetScales::Free))
            .erase();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let free = prepare_plot(&free, &resources)
            .unwrap()
            .attach(free)
            .unwrap();
        let union = (
            free.prepared_axes[0].1.ranges[1]
                .0
                .min(free.prepared_axes[1].1.ranges[1].0),
            free.prepared_axes[0].1.ranges[1]
                .1
                .max(free.prepared_axes[1].1.ranges[1].1),
        );
        let mixed = plot
            .clone()
            .facet(FacetSpec::column(facets.clone()).scales(FacetScales::FreeX))
            .erase();
        let mixed = prepare_plot(&mixed, &resources)
            .unwrap()
            .attach(mixed)
            .unwrap();
        for (index, (_, axes)) in mixed.prepared_axes.iter().enumerate() {
            assert_eq!(axes.ranges[1], union);
            assert_eq!(
                axes.categories[0],
                free.prepared_axes[index].1.categories[0]
            );
        }
        for scales in [FacetScales::Fixed, FacetScales::FreeY] {
            let invalid = plot
                .clone()
                .facet(FacetSpec::column(facets.clone()).scales(scales));
            let error = invalid.preflight().unwrap_err();
            assert!(
                error.to_string().contains("identical categorical"),
                "{error}"
            );
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        mixed.preflight().unwrap();
    }

    #[test]
    fn free_y_uses_histogram_edge_union_and_keeps_actual_panel_heights() {
        let source = Plot::histogram(crate::HistogramInput::samples(NumericValues::new(
            "value",
            [0.0, 1.0, 10.0, 20.0, 20.0],
        )))
        .facet(
            FacetSpec::column(CategoryValues::new("panel", ["a", "a", "b", "b", "b"]).unwrap())
                .scales(FacetScales::FreeY),
        )
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let prepared = prepare_plot(&source, &resources)
            .unwrap()
            .attach(source.clone())
            .unwrap();
        let panels = crate::orchestrator::text_census_panels(&source, source.data()).unwrap();
        let local = panels
            .iter()
            .map(|panel| {
                crate::plots::histogram::axis_ranges(
                    panel.plot(),
                    panel.data().with_axis_resources(&resources),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let x = (
            local[0][0].0.min(local[1][0].0),
            local[0][0].1.max(local[1][0].1),
        );
        for (index, (_, axes)) in prepared.prepared_axes.iter().enumerate() {
            assert_eq!(axes.ranges, [x, local[index][1]]);
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
