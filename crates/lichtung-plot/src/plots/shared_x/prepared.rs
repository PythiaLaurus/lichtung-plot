use crate::data::PlotData;
use crate::layout::interior::{InteriorRoute, PreparedInteriorInputs};
use crate::layout::{AxisSpec, AxisTick};
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::{Error, Plot};

fn continuous_selection(
    source: &Plot,
    owner: Option<&crate::theme::Theme>,
) -> Result<crate::theme::ContinuousColorScale, Error> {
    source.continuous_selection_in(
        owner
            .ok_or_else(|| {
                Error::Render("continuous preparation requires its owning color scales".into())
            })?
            .color_scales(),
    )
}

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
    pub(crate) labels: [Option<crate::Text>; 2],
    pub(crate) specs: [AxisSpec; 2],
    pub(crate) logarithmic: bool,
}

impl PreparedAxes {
    pub(crate) fn empty_numeric_source(&self, side: usize) -> Result<bool, Error> {
        let generation = &self.specs[side].generation;
        if generation
            .exact_parameters()
            .is_some_and(|(count, _, _)| count == 0)
        {
            return Ok(true);
        }
        let range = self.specs[side].limits.unwrap_or(self.ranges[side]);
        let log = self.logarithmic && side == 0;
        let maximum = generation.get_maximum_count().unwrap_or(usize::MAX);
        if let Some((anchor, ratio, requirement)) = generation.multiplicative_parameters() {
            let count =
                crate::numeric::lattice::generation::LatticeRange::new(range, log, anchor, ratio)?
                    .count();
            if requirement == crate::TickRequirement::Fixed
                && count > num_bigint::BigInt::from(maximum)
            {
                return Err(Error::Render("fixed multiplicative count exceeds its configured maximum or platform representation".into()));
            }
            return Ok(count == num_bigint::BigInt::ZERO);
        }
        if let Some(parameters) = generation.interval_parameters() {
            let raw = self.specs[side]
                .numeric_source
                .as_deref()
                .and_then(|source| {
                    source.raw_limits(if side == 0 {
                        crate::layout::AxisSide::Bottom
                    } else {
                        crate::layout::AxisSide::Left
                    })
                });
            return crate::layout::fixed_interval_is_empty(range, raw, log, parameters, maximum);
        }
        Ok(false)
    }

    pub(crate) fn numeric_probe(
        &self,
        side: usize,
    ) -> Result<Option<crate::numeric::set::FormattedSet>, Error> {
        if self.specs[side].exact_ticks.is_some()
            || self.categories[side].is_some()
            || self.empty_numeric_source(side)?
        {
            return Ok(None);
        }
        let Some(request) = self.specs[side].numeric_request()? else {
            return Ok(None);
        };
        let range = self.specs[side].limits.unwrap_or(self.ranges[side]);
        request.materialize(
            &[range.0, f64::midpoint(range.0, range.1), range.1].map(|value| {
                crate::numeric::operand::Operand::number(value, self.logarithmic && side == 0)
            }),
            self.logarithmic && side == 0,
            if side == 0 {
                crate::layout::AxisSide::Bottom
            } else {
                crate::layout::AxisSide::Left
            },
            range,
            crate::NumericPurpose::IntrinsicProbe,
        )
    }
    fn bind_numeric(
        &mut self,
        plot: &Plot,
        resources: &crate::font::canonical::OperationContext,
        facet: Option<&crate::FacetCell>,
        scope: Option<&str>,
    ) -> Result<(), Error> {
        let mut source = crate::numeric::source::NumericSource::for_plot(plot, resources, facet)?;
        source.owner.scope = scope
            .map(crate::data::source::copy_source_string)
            .transpose()?;
        let source = crate::data::source::source_arc(source)?;
        for spec in &mut self.specs {
            spec.numeric_source = Some(source.clone());
        }
        Ok(())
    }
    pub(crate) fn copy_label(&self, side: usize) -> Result<Option<crate::Text>, Error> {
        self.labels[side]
            .as_ref()
            .map(crate::Text::try_copy)
            .transpose()
    }
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
    pub(crate) fn new(
        source: &Plot,
        data: PlotData<'_>,
        owner: Option<&crate::theme::Theme>,
    ) -> Result<Self, Error> {
        match source.kind() {
            PlotKind::Waterfall => Ok(Self::Waterfall(crate::plots::waterfall::prepare(
                source, data,
            )?)),
            PlotKind::Beeswarm => Ok(Self::Beeswarm(crate::plots::beeswarm::prepare(
                source,
                data,
                continuous_selection(source, owner)?,
            )?)),
            PlotKind::Marginal => Ok(Self::Marginal(crate::plots::marginal::prepare(
                source, data,
            )?)),
            PlotKind::Heatmap => Ok(Self::Heatmap(crate::plots::heatmap::prepare(
                source,
                data,
                continuous_selection(source, owner)?,
            )?)),
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
                crate::plots::histogram::HistogramSemantics::new(source, data, owner)?,
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
    pub(crate) fn x_range(&self, source: &Plot) -> Result<(f64, f64), Error> {
        Ok(match self {
            Self::Waterfall(marks) => {
                crate::layout::continuous_auto_range(marks.range, source.x_axis_spec())?
            }
            Self::Beeswarm(marks) => {
                crate::layout::continuous_auto_range(marks.x_data, source.x_axis_spec())?
            }
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
                    )?
                } else {
                    (0.0, marks.categories.len() as f64)
                }
            }
            Self::Estimate(marks) => marks.x_range()?,
            Self::Dumbbell(marks) => {
                crate::layout::continuous_auto_range(marks.range, source.x_axis_spec())?
            }
            Self::Histogram(marks) => marks.ranges[0],
            Self::Density(marks) => marks.ranges(source)?[0],
            Self::Qq(marks) => marks.ranges(source)?[0],
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

    marks: PreparedReference,
    scope: Option<&str>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&crate::Labels>,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
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
                marks.color_label = source.prepare_color_label(labels)?;
            }
            beeswarm = Some(marks);
            PlotKind::Beeswarm
        }
        PreparedReference::Marginal(marks) => {
            marginal = Some(crate::plots::marginal::finish(marks, x_limits)?);
            PlotKind::Marginal
        }
        PreparedReference::Heatmap(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.prepare_color_label(labels)?;
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
    let mut axes = prepare(
        source,
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
    axes.bind_numeric(source, resources, None, scope)?;
    if let Some(marks) = waterfall.as_mut() {
        marks.prepare_totals(&axes, false)?;
    }
    if let Some(quantiles) = &quantiles {
        quantiles.validate_ranges(axes.ranges)?;
    }
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges)?;
    let scope = || {
        scope
            .map(crate::data::source::copy_source_string)
            .transpose()
    };
    let mut retained = Vec::new();
    crate::data::source::reserve_source(&mut retained, 1)?;
    retained.push((scope()?, axes));
    Ok(PreparedInteriorInputs {
        discrete_guides: std::sync::OnceLock::new(),
        waterfall: waterfall
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::waterfall::PreparedWaterfallInputs {
                    panels,
                })
            })
            .transpose()?,
        beeswarm: beeswarm
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::beeswarm::PreparedBeeswarmInputs {
                    panels,
                })
            })
            .transpose()?,
        marginal: marginal.map(crate::data::source::source_arc).transpose()?,
        heatmap: heatmap
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::heatmap::PreparedHeatmapInputs {
                    panels,
                })
            })
            .transpose()?,
        interaction: interaction
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(
                    crate::plots::interaction::PreparedInteractionInputs { panels },
                )
            })
            .transpose()?,
        distributions: distribution
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(
                    crate::plots::categorical_distribution::PreparedDistributionInputs { panels },
                )
            })
            .transpose()?,
        bars: bars
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::bar_data::PreparedBarInputs {
                    panels,
                })
            })
            .transpose()?,
        estimate: estimate
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(
                    crate::plots::interval_estimate::PreparedEstimateInputs { panels },
                )
            })
            .transpose()?,
        dumbbell: dumbbell
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::dumbbell::PreparedDumbbellInputs {
                    panels,
                })
            })
            .transpose()?,
        route,
        axes: crate::data::source::source_arc(retained)?,
        matrix: None,
        scatter: None,
        line: None,
        histogram: bins
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::histogram::PreparedHistogramInputs {
                    panels,
                })
            })
            .transpose()?,
        density: curves
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::density::PreparedDensityInputs {
                    panels,
                })
            })
            .transpose()?,
        qq: quantiles
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(crate::plots::qq::PreparedQqInputs { panels })
            })
            .transpose()?,
        regression: fit
            .map(|marks| {
                let mut panels = Vec::new();
                crate::data::source::reserve_source(&mut panels, 1)?;
                panels.push((scope()?, marks));
                crate::data::source::source_arc(
                    crate::plots::regression::PreparedRegressionInputs { panels },
                )
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
    resources: &crate::font::canonical::OperationContext,
    owner: Option<&crate::theme::Theme>,
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
        .with_operation(resources);
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
            PreparedReference::new(source, source_data, owner)?,
            source.axis_scope(),
            None,
            None,
            resources,
        );
    }
    if source.kind() == PlotKind::Scatter && !source.has_facets() {
        return prepare_scatter_plot(
            source,
            crate::plots::scatter::PreparedScatter::new(source, source_data)?,
            resources,
        );
    }
    if matches!(
        source.kind(),
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
    ) && !source.has_facets()
    {
        return prepare_line_plot(
            source,
            crate::plots::line::PreparedLine::new(source, source_data, None)?,
            resources,
        );
    }
    if source.has_facets() {
        let row_topology = (InteriorRoute::classify(source.kind()).is_coordinate()
            || matches!(
                source.kind(),
                PlotKind::Heatmap | PlotKind::Beeswarm | PlotKind::Waterfall
            ))
        .then(|| source_data.bind_row_source())
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
            crate::data::source::collect_source(panels.iter().map(|panel| {
                crate::plots::interval_estimate::PreparedEstimate::new(
                    source,
                    panel
                        .data()
                        .with_operation(resources)
                        .with_row_source(row_topology.as_ref()),
                    panel.significance_annotations(),
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
                crate::data::source::source_levels(&crate::data::strict_strings(
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
            Some(crate::data::source::source_levels(
                &crate::data::strict_normalized_strings(
                    source_data,
                    source.x_col().expect("categorical x"),
                    "trajectory",
                    "condition",
                )?,
            )?)
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
                crate::data::source::source_levels(&crate::data::strict_strings(
                    source_data,
                    column,
                    "facet hue",
                )?)
            })
            .transpose()?;
        crate::data::source::reserve_source(&mut retained, panels.len())?;
        for (index, panel) in panels.into_iter().enumerate() {
            let data = panel
                .data()
                .with_operation(resources)
                .with_row_source(row_topology.as_ref());
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
                .then(|| crate::plots::histogram::HistogramSemantics::new(source, data, owner))
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
                .then(|| {
                    crate::plots::heatmap::prepare(
                        source,
                        data,
                        continuous_selection(source, owner)?,
                    )
                })
                .transpose()?;
            let beeswarm_marks = (source.kind() == PlotKind::Beeswarm)
                .then(|| {
                    crate::plots::beeswarm::prepare(
                        source,
                        data,
                        continuous_selection(source, owner)?,
                    )
                })
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
                    panel.significance_annotations(),
                )
            })
            .transpose()?;
            let mut axes = prepare(
                source,
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
            let scope = panel.scope()?;
            axes.bind_numeric(source, resources, panel.numeric_cell(), Some(&scope))?;
            if let Some(marks) = waterfall_marks {
                crate::data::source::reserve_source(&mut waterfall, 1)?;
                waterfall.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(marks) = beeswarm_marks {
                crate::data::source::reserve_source(&mut beeswarm, 1)?;
                beeswarm.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(marks) = heatmap_marks {
                crate::data::source::reserve_source(&mut heatmap, 1)?;
                heatmap.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(marks) = interaction_marks {
                crate::data::source::reserve_source(&mut interaction, 1)?;
                interaction.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(marks) = distribution_marks {
                crate::data::source::reserve_source(&mut distributions, 1)?;
                distributions.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(marks) = bar_marks {
                crate::data::source::reserve_source(&mut bars, 1)?;
                bars.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
            }
            if let Some(estimates) = estimates {
                crate::data::source::reserve_source(&mut estimate, 1)?;
                estimate.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    estimates,
                ));
            }
            if let Some(pairs) = pairs {
                crate::data::source::reserve_source(&mut dumbbell, 1)?;
                dumbbell.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    pairs,
                ));
            }
            if let Some(quantiles) = quantiles {
                crate::data::source::reserve_source(&mut qq, 1)?;
                qq.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    quantiles,
                ));
            }
            if let Some(fit) = fit {
                crate::data::source::reserve_source(&mut regression, 1)?;
                regression.push((Some(crate::data::source::copy_source_string(&scope)?), fit));
            }
            if let Some(bins) = bins {
                crate::data::source::reserve_source(&mut histogram, 1)?;
                histogram.push((Some(crate::data::source::copy_source_string(&scope)?), bins));
            }
            if let Some(curves) = curves {
                crate::data::source::reserve_source(&mut density, 1)?;
                density.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    curves,
                ));
            }
            if let Some(paths) = paths {
                crate::data::source::reserve_source(&mut line, 1)?;
                line.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    paths,
                ));
            }
            if let Some(marks) = marks {
                crate::data::source::reserve_source(&mut scatter, 1)?;
                scatter.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    marks,
                ));
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
            reconcile_facet_axis(&mut retained, side)?;
        }
        for ((_, marks), (_, axes)) in waterfall.iter_mut().zip(&retained) {
            marks.prepare_totals(axes, false)?;
        }
        for ((_, quantiles), (_, axes)) in qq.iter().zip(&retained) {
            quantiles.validate_ranges(axes.ranges)?;
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
                        crate::layout::Axis::validate_ticks(
                            ticks,
                            axes.ranges[side],
                            if side == 0 { "x-axis" } else { "y-axis" },
                        )?;
                    }
                }
            }
            crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges)?;
        }
    } else if source.kind() == PlotKind::Matrix {
        matrix = Some(crate::data::source::source_arc(
            crate::plots::matrix::prepare_axes(source, source_data)?,
        )?);
    } else {
        let mut axes = prepare(source, None, None, None, None, None, None, None, None)?;
        axes.bind_numeric(source, resources, None, source.axis_scope())?;
        crate::data::source::reserve_source(&mut retained, 1)?;
        retained.push((
            source
                .axis_scope()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
            axes,
        ));
    }
    Ok(PreparedInteriorInputs {
        marginal: None,
        discrete_guides: std::sync::OnceLock::new(),
        waterfall: if source.kind() == PlotKind::Waterfall {
            Some(crate::data::source::source_arc(
                crate::plots::waterfall::PreparedWaterfallInputs { panels: waterfall },
            )?)
        } else {
            None
        },
        beeswarm: if source.kind() == PlotKind::Beeswarm {
            Some(crate::data::source::source_arc(
                crate::plots::beeswarm::PreparedBeeswarmInputs { panels: beeswarm },
            )?)
        } else {
            None
        },
        heatmap: if source.kind() == PlotKind::Heatmap {
            Some(crate::data::source::source_arc(
                crate::plots::heatmap::PreparedHeatmapInputs { panels: heatmap },
            )?)
        } else {
            None
        },
        interaction: if source.kind() == PlotKind::Interaction {
            Some(crate::data::source::source_arc(
                crate::plots::interaction::PreparedInteractionInputs {
                    panels: interaction,
                },
            )?)
        } else {
            None
        },
        distributions: if matches!(
            source.kind(),
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
        ) {
            Some(crate::data::source::source_arc(
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
            Some(crate::data::source::source_arc(
                crate::plots::bar_data::PreparedBarInputs { panels: bars },
            )?)
        } else {
            None
        },
        estimate: if matches!(
            source.kind(),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
        ) {
            Some(crate::data::source::source_arc(
                crate::plots::interval_estimate::PreparedEstimateInputs { panels: estimate },
            )?)
        } else {
            None
        },
        dumbbell: if source.kind() == PlotKind::Dumbbell {
            Some(crate::data::source::source_arc(
                crate::plots::dumbbell::PreparedDumbbellInputs { panels: dumbbell },
            )?)
        } else {
            None
        },
        qq: if source.kind() == PlotKind::Qq {
            Some(crate::data::source::source_arc(
                crate::plots::qq::PreparedQqInputs { panels: qq },
            )?)
        } else {
            None
        },
        regression: if source.kind() == PlotKind::Regression {
            Some(crate::data::source::source_arc(
                crate::plots::regression::PreparedRegressionInputs { panels: regression },
            )?)
        } else {
            None
        },
        histogram: if source.kind() == PlotKind::Histogram {
            Some(crate::data::source::source_arc(
                crate::plots::histogram::PreparedHistogramInputs { panels: histogram },
            )?)
        } else {
            None
        },
        density: if source.kind() == PlotKind::Density {
            Some(crate::data::source::source_arc(
                crate::plots::density::PreparedDensityInputs { panels: density },
            )?)
        } else {
            None
        },
        route: InteriorRoute::classify(source.kind()),
        axes: crate::data::source::source_arc(retained)?,
        matrix,
        line: if matches!(
            source.kind(),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
        ) {
            Some(crate::data::source::source_arc(
                crate::plots::line::PreparedLineInputs { panels: line },
            )?)
        } else {
            None
        },
        scatter: if source.kind() == PlotKind::Scatter {
            Some(crate::data::source::source_arc(
                crate::plots::scatter::PreparedScatterInputs { panels: scatter },
            )?)
        } else {
            None
        },
    })
}

/// Complete one already prepared Scatter component after root range coordination.
pub(crate) fn prepare_scatter_plot(
    source: &Plot,

    marks: crate::plots::scatter::PreparedScatter,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
    InteriorRoute::classify(PlotKind::Scatter).validate(source)?;

    let mut axes = prepare(
        source,
        None,
        Some(&marks),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    axes.bind_numeric(source, resources, None, source.axis_scope())?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges)?;
    let scope = || {
        source
            .axis_scope()
            .map(crate::data::source::copy_source_string)
            .transpose()
    };
    let mut retained_axes = Vec::new();
    crate::data::source::reserve_source(&mut retained_axes, 1)?;
    retained_axes.push((scope()?, axes));
    let mut panels = Vec::new();
    crate::data::source::reserve_source(&mut panels, 1)?;
    panels.push((scope()?, marks));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(PlotKind::Scatter),
        discrete_guides: std::sync::OnceLock::new(),
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
        axes: crate::data::source::source_arc(retained_axes)?,
        matrix: None,
        line: None,
        scatter: Some(crate::data::source::source_arc(
            crate::plots::scatter::PreparedScatterInputs { panels },
        )?),
    })
}

/// Complete one retained line component after the parent's range coordination.
pub(crate) fn prepare_line_plot(
    source: &Plot,

    marks: crate::plots::line::PreparedLine,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
    let mut axes = prepare(
        source,
        None,
        None,
        Some(&marks),
        None,
        None,
        None,
        None,
        None,
    )?;
    axes.bind_numeric(source, resources, None, source.axis_scope())?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges)?;
    let scope = || {
        source
            .axis_scope()
            .map(crate::data::source::copy_source_string)
            .transpose()
    };
    let mut retained_axes = Vec::new();
    crate::data::source::reserve_source(&mut retained_axes, 1)?;
    retained_axes.push((scope()?, axes));
    let mut panels = Vec::new();
    crate::data::source::reserve_source(&mut panels, 1)?;
    panels.push((scope()?, marks));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(source.kind()),
        discrete_guides: std::sync::OnceLock::new(),
        beeswarm: None,
        waterfall: None,
        marginal: None,
        heatmap: None,
        bars: None,
        interaction: None,
        distributions: None,
        estimate: None,
        dumbbell: None,
        axes: crate::data::source::source_arc(retained_axes)?,
        histogram: None,
        qq: None,
        regression: None,
        density: None,
        matrix: None,
        scatter: None,
        line: Some(crate::data::source::source_arc(
            crate::plots::line::PreparedLineInputs { panels },
        )?),
    })
}

fn reconcile_facet_axis(
    retained: &mut [(Option<String>, PreparedAxes)],
    side: usize,
) -> Result<(), Error> {
    let Some((_, first)) = retained.first() else {
        return Ok(());
    };
    for (_, axes) in retained.iter() {
        if axes.categories[side] != first.categories[side] {
            return Err(Error::Render(format!("fixed facet {} axis requires identical categorical levels and ordering in every panel; use a free scale or provide complete panel data", if side == 0 { "x" } else { "y" })));
        }
    }
    if first.categories[side].is_none() {
        fn presentation(
            axes: &PreparedAxes,
            side: usize,
        ) -> (bool, Option<&crate::numeric::format::NumericFormat>) {
            (
                axes.logarithmic && side == 0,
                if axes.specs[side].exact_ticks.is_some() {
                    None
                } else {
                    axes.specs[side].format.as_deref()
                },
            )
        }
        for (_, axes) in &*retained {
            if presentation(axes, side) != presentation(first, side) {
                return Err(Error::Data(
                    "fixed facet numeric axes require identical numeric formats and operand contexts"
                        .into(),
                ));
            }
        }
        let union = retained
            .iter()
            .map(|(_, axes)| axes.ranges[side])
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, b| {
                (a.0.min(b.0), a.1.max(b.1))
            });
        validate_range(union)?;

        for (_, axes) in retained {
            axes.ranges[side] = union;
            if axes.specs[side].provenance != crate::layout::RangeProvenance::Configured {
                axes.specs[side].provenance = crate::layout::RangeProvenance::Shared;
            }
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
                crate::layout::continuous_auto_range(ranges[0], chart.x_axis_spec())?,
                crate::layout::continuous_auto_range(ranges[1], chart.y_axis_spec())?,
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
            .ranges(chart)?,
        PlotKind::Qq => qq
            .ok_or_else(|| Error::Render("Qq axes require prepared semantics".into()))?
            .ranges(chart)?,
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let line = line.ok_or_else(|| {
                Error::Render("line axis inputs require prepared semantics".into())
            })?;
            categories[0] = line
                .conditions
                .as_ref()
                .map(|levels| {
                    crate::data::source::collect_source(
                        levels
                            .iter()
                            .map(|label| crate::data::source::copy_source_string(label)),
                    )
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
            let levels = crate::data::source::collect_source(
                marks
                    .categories
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
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
            let levels = crate::data::source::collect_source(
                marks
                    .categories()
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?;
            let values = marks.range();
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(values, chart.y_axis_spec())?,
            ]
        }
        PlotKind::Dumbbell => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.dumbbell)
                .ok_or_else(|| Error::Render("Dumbbell axes require prepared semantics".into()))?;
            categories[1] = Some(crate::data::source::collect_source(
                marks
                    .categories
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?);
            [
                crate::layout::continuous_auto_range(marks.range, chart.x_axis_spec())?,
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
            categories[1] = Some(crate::data::source::collect_source(
                marks
                    .labels
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?);
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
            let levels = crate::data::source::collect_source(
                marks
                    .conditions
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?;
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(marks.range, chart.y_axis_spec())?,
            ]
        }
        PlotKind::Beeswarm => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.beeswarm)
                .ok_or_else(|| Error::Render("beeswarm axes require retained semantics".into()))?;
            let levels = crate::data::source::collect_source(
                marks
                    .levels
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?;
            let slots = chart
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(marks.x_data, chart.x_axis_spec())?,
                slots,
            ]
        }
        PlotKind::Waterfall => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.waterfall)
                .ok_or_else(|| Error::Render("waterfall axes require retained semantics".into()))?;
            let levels = crate::data::source::collect_source(
                marks
                    .labels
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?;
            let slots = chart
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(marks.range, chart.x_axis_spec())?,
                slots,
            ]
        }
        PlotKind::Heatmap => {
            let marks = reference_config
                .as_ref()
                .and_then(|config| config.heatmap)
                .ok_or_else(|| Error::Render("heatmap axes require retained semantics".into()))?;
            let x = crate::data::source::collect_source(
                marks
                    .x_levels
                    .iter()
                    .map(|s| crate::data::source::copy_source_string(s)),
            )?;
            let y = crate::data::source::collect_source(
                marks
                    .y_levels
                    .iter()
                    .map(|s| crate::data::source::copy_source_string(s)),
            )?;
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
            generation: source.generation.clone(),
            category_labels: source.category_labels.clone(),
            format: source.format.clone(),
            numeric_source: None,
            exact_ticks: if logarithmic && side == 0 {
                chart.projected_explicit_x_ticks()?
            } else {
                source
                    .exact_ticks
                    .as_ref()
                    .map(|ticks| {
                        crate::data::source::collect_source(ticks.iter().map(|tick| {
                            Ok(AxisTick::new(
                                tick.position,
                                crate::data::source::copy_source_string(&tick.label)?,
                            ))
                        }))
                    })
                    .transpose()?
            },
        };
    }
    if x_limits.is_some() && specs[0].provenance != crate::layout::RangeProvenance::Configured {
        specs[0].provenance = crate::layout::RangeProvenance::Shared;
    }
    if logarithmic {
        // Both explicit and parent-coordinated domains are already transformed.
        specs[0].limits = specs[0].limits.map(|_| ranges[0]);
    }
    let mut ticks = [None, None];
    let mut display_categories = [None, None];
    for side in 0..2 {
        validate_range(ranges[side])?;
        if categories[side].is_none() && !specs[side].category_labels.is_all() {
            return Err(Error::Data(
                "category label selection requires an actual categorical axis".into(),
            ));
        }
        if let Some(levels) = categories[side].take() {
            if specs[side].format.is_some() {
                return Err(Error::Data(format!(
                    "{}-axis categories do not support a numeric tick format",
                    if side == 0 { "x" } else { "y" },
                )));
            }
            if !specs[side].generation.is_automatic() {
                return Err(Error::Data(format!(
                    "{}-axis categories do not support numeric tick generation",
                    if side == 0 { "x" } else { "y" },
                )));
            }
            let domain = if side == 0 {
                "x-axis categories"
            } else {
                "y-axis categories"
            };
            let raw_labels = levels.iter().map(String::as_str);
            display_categories[side] = Some(if let Some(policy) = labels {
                policy.resolve_domain(domain, raw_labels)?
            } else {
                chart.resolve_axis_label_domain(domain, raw_labels)?
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
            crate::data::source::reserve_source(&mut retained, levels.len())?;
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

            crate::layout::Axis::validate_ticks(
                exact,
                ranges[side],
                if side == 0 { "x-axis" } else { "y-axis" },
            )?;
        }
    }
    let labels = if facet.is_some() {
        [None, None]
    } else {
        chart.prepare_axis_labels(labels)?
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
