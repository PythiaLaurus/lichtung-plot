use crate::data::PlotData;
use crate::layout::interior::{InteriorRoute, PreparedInteriorInputs};
use crate::layout::{AxisSpec, AxisTick};
use crate::orchestrator::registration::PreparedFamily;
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::{Error, Plot};

fn continuous_selection(
    source: &Plot,
    owner: Option<&crate::theme::Theme>,
) -> Result<crate::theme::ContinuousColorScale, Error> {
    source.description().continuous_selection_in(
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

impl PreparedFamily {
    pub(crate) fn new(
        source: &Plot,
        data: PlotData<'_, '_>,
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
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => Ok(Self::Distribution {
                kind: source.kind(),
                marks: crate::plots::categorical_distribution::PreparedDistribution::new(
                    source, data, None,
                )?,
            }),
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => Ok(Self::Bars {
                kind: source.kind(),
                mode: crate::plots::bar_data::mode(source),
                marks: crate::plots::bar_data::prepare(
                    source,
                    data,
                    crate::plots::bar_data::mode(source),
                )?,
            }),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                Ok(Self::Estimate {
                    kind: source.kind(),
                    marks: crate::plots::interval_estimate::PreparedEstimate::new(
                        source,
                        data,
                        source.description().significance_annotations(),
                    )?,
                })
            }
            PlotKind::Dumbbell => Ok(Self::Dumbbell(crate::plots::dumbbell::prepare(
                source.description(),
                data,
            )?)),
            PlotKind::Histogram => Ok(Self::Histogram(
                crate::plots::histogram::HistogramSemantics::new(
                    source.description(),
                    data,
                    owner,
                )?,
            )),
            PlotKind::Density => Ok(Self::Density(crate::plots::density::PreparedDensity::new(
                source, data,
            )?)),
            PlotKind::Qq => Ok(Self::Qq(crate::plots::qq::PreparedQq::new(
                source.description(),
                data,
            )?)),
            PlotKind::Regression => Ok(Self::Regression(
                crate::plots::regression::PreparedRegression::new(source.description(), data)?,
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
            Self::Waterfall(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source.description().x_axis_spec(),
            )?,
            Self::Beeswarm(marks) => crate::layout::continuous_auto_range(
                marks.x_data,
                source.description().x_axis_spec(),
            )?,
            Self::Marginal(marks) => marks.ranges[0],
            Self::Heatmap(marks) => (0.0, marks.x_levels.len() as f64),
            Self::Interaction(marks) => (0.0, marks.conditions.len() as f64),
            Self::Distribution { marks, .. } => (0.0, marks.categories().len() as f64),
            Self::Bars { kind, marks, .. } => {
                if matches!(
                    kind,
                    PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                ) {
                    crate::layout::anchored_auto_range(
                        marks.value_range,
                        source.description().x_axis_spec(),
                        0.0,
                    )?
                } else {
                    (0.0, marks.categories.len() as f64)
                }
            }
            Self::Estimate { kind, marks } => marks.x_range(*kind)?,
            Self::Dumbbell(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source.description().x_axis_spec(),
            )?,
            Self::Histogram(marks) => marks.ranges[0],
            Self::Density(marks) => marks.ranges(source.description())?[0],
            Self::Qq(marks) => marks.ranges(source.description())?[0],
            Self::Regression(marks) => marks.ranges[0],
            Self::Scatter(_) | Self::Line { .. } | Self::Matrix(_) => {
                return Err(Error::Render(
                    "shared semantics require an extracted reference or paired-endpoint family"
                        .into(),
                ))
            }
        })
    }
}

#[derive(Default)]
struct ReferenceAxisConfig<'a> {
    estimate_fallback: Option<(f64, f64)>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&'a crate::Labels>,
}

pub(crate) fn prepare_reference_plot(
    source: &Plot,

    family: PreparedFamily,
    scope: Option<&str>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&crate::Labels>,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
    let mut family = match family {
        PreparedFamily::Beeswarm(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.description().prepare_color_label(labels)?;
            }
            PreparedFamily::Beeswarm(marks)
        }
        PreparedFamily::Marginal(marks) => {
            PreparedFamily::Marginal(crate::plots::marginal::finish(marks, x_limits)?)
        }
        PreparedFamily::Heatmap(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.description().prepare_color_label(labels)?;
            }
            PreparedFamily::Heatmap(marks)
        }
        family => family,
    };
    let route = InteriorRoute::classify(family.kind());
    route.validate(source.description())?;
    let config = ReferenceAxisConfig {
        x_limits,
        labels,
        ..ReferenceAxisConfig::default()
    };
    let mut axes = prepare(source, None, Some(&family), Some(config))?;
    axes.bind_numeric(source, resources, None, scope)?;
    if let PreparedFamily::Waterfall(marks) = &mut family {
        marks.prepare_totals(&axes, false)?;
    }
    if let PreparedFamily::Qq(quantiles) = &family {
        quantiles.validate_ranges(axes.ranges)?;
    }
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges,
    )?;
    let scope = || {
        scope
            .map(crate::data::source::copy_source_string)
            .transpose()
    };
    let mut retained = Vec::new();
    crate::data::source::reserve_source(&mut retained, 1)?;
    retained.push((scope()?, axes));
    let mut families = Vec::new();
    crate::data::source::reserve_source(&mut families, 1)?;
    families.push((scope()?, family));
    Ok(PreparedInteriorInputs {
        discrete_guides: std::sync::OnceLock::new(),
        route,
        axes: crate::data::source::source_arc(retained)?,
        families: crate::data::source::source_arc(families)?,
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
    let mut retained = Vec::new();
    let mut families = Vec::new();
    let source_data = source
        .description()
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
    ) && !source.description().has_facets()
    {
        return prepare_reference_plot(
            source,
            PreparedFamily::new(source, source_data, owner)?,
            source.axis_scope(),
            None,
            None,
            resources,
        );
    }
    if source.kind() == PlotKind::Scatter && !source.description().has_facets() {
        return prepare_scatter_plot(
            source,
            crate::plots::scatter::PreparedScatter::new(source.description(), source_data)?,
            resources,
        );
    }
    if matches!(
        source.kind(),
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
    ) && !source.description().has_facets()
    {
        return prepare_line_plot(
            source,
            crate::plots::line::PreparedLine::new(source, source_data, None)?,
            resources,
        );
    }
    if source.description().has_facets() {
        let row_topology = (InteriorRoute::classify(source.kind()).is_coordinate()
            || matches!(
                source.kind(),
                PlotKind::Heatmap | PlotKind::Beeswarm | PlotKind::Waterfall
            ))
        .then(|| source_data.bind_row_source())
        .transpose()?;
        let panels = crate::orchestrator::text_census_panels(source.description())?;
        let interaction_parent = (source.kind() == PlotKind::Interaction)
            .then(|| {
                crate::data::observed_collectives(
                    source_data,
                    source.description().group_col(),
                    source.description().group_name(),
                    source.description().hue_col(),
                    source.description().hue_name(),
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
            source.description().facet_scales_spec(),
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
            .description()
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
                .description()
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
                    source.description().x_col().expect("categorical x"),
                    "trajectory",
                    "condition",
                )?,
            )?)
        } else {
            None
        };
        let hues = source
            .description()
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
                Some(crate::plots::scatter::PreparedScatter::new(
                    source.description(),
                    data,
                )?)
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
                .then(|| {
                    crate::plots::histogram::HistogramSemantics::new(
                        source.description(),
                        data,
                        owner,
                    )
                })
                .transpose()?;
            let curves = (source.kind() == PlotKind::Density)
                .then(|| crate::plots::density::PreparedDensity::new(source, data))
                .transpose()?;
            let quantiles = (source.kind() == PlotKind::Qq)
                .then(|| crate::plots::qq::PreparedQq::new(source.description(), data))
                .transpose()?;
            let fit = (source.kind() == PlotKind::Regression)
                .then(|| {
                    crate::plots::regression::PreparedRegression::new(source.description(), data)
                })
                .transpose()?;
            let pairs = (source.kind() == PlotKind::Dumbbell)
                .then(|| crate::plots::dumbbell::prepare(source.description(), data))
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
            let family = waterfall_marks
                .map(PreparedFamily::Waterfall)
                .or_else(|| beeswarm_marks.map(PreparedFamily::Beeswarm))
                .or_else(|| heatmap_marks.map(PreparedFamily::Heatmap))
                .or_else(|| interaction_marks.map(PreparedFamily::Interaction))
                .or_else(|| {
                    distribution_marks.map(|marks| PreparedFamily::Distribution {
                        kind: source.kind(),
                        marks,
                    })
                })
                .or_else(|| {
                    bar_marks.map(|marks| PreparedFamily::Bars {
                        kind: source.kind(),
                        mode: crate::plots::bar_data::mode(source),
                        marks,
                    })
                })
                .or_else(|| {
                    estimates.map(|marks| PreparedFamily::Estimate {
                        kind: source.kind(),
                        marks,
                    })
                })
                .or_else(|| pairs.map(PreparedFamily::Dumbbell))
                .or_else(|| quantiles.map(PreparedFamily::Qq))
                .or_else(|| fit.map(PreparedFamily::Regression))
                .or_else(|| bins.map(PreparedFamily::Histogram))
                .or_else(|| curves.map(PreparedFamily::Density))
                .or_else(|| {
                    paths.map(|marks| PreparedFamily::Line {
                        kind: source.kind(),
                        marks,
                    })
                })
                .or_else(|| marks.map(PreparedFamily::Scatter));
            let mut axes = prepare(
                source,
                Some(facet),
                family.as_ref(),
                Some(ReferenceAxisConfig {
                    estimate_fallback,
                    ..ReferenceAxisConfig::default()
                }),
            )?;
            let scope = panel.scope()?;
            axes.bind_numeric(source, resources, panel.numeric_cell(), Some(&scope))?;
            if let Some(family) = family {
                crate::data::source::reserve_source(&mut families, 1)?;
                families.push((
                    Some(crate::data::source::copy_source_string(&scope)?),
                    family,
                ));
            }
            retained.push((Some(scope), axes));
        }
        for (side, fixed) in [
            matches!(
                source.description().facet_scales_spec(),
                FacetScales::Fixed | FacetScales::FreeY
            ),
            matches!(
                source.description().facet_scales_spec(),
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
        for ((_, family), (_, axes)) in families.iter_mut().zip(&retained) {
            if let PreparedFamily::Waterfall(marks) = family {
                marks.prepare_totals(axes, false)?;
            }
        }
        for ((_, family), (_, axes)) in families.iter().zip(&retained) {
            let PreparedFamily::Qq(quantiles) = family else {
                continue;
            };
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
            crate::orchestrator::layers::GuideSnapshot::validate_sources(
                source.description(),
                axes.ranges,
            )?;
        }
    } else if source.kind() == PlotKind::Matrix {
        let marks = crate::plots::matrix::prepare_axes(source, source_data)?;
        crate::data::source::reserve_source(&mut families, 1)?;
        families.push((
            source
                .axis_scope()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
            PreparedFamily::Matrix(marks),
        ));
    } else {
        let mut axes = prepare(source, None, None, None)?;
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
        discrete_guides: std::sync::OnceLock::new(),
        families: crate::data::source::source_arc(families)?,
        route: InteriorRoute::classify(source.kind()),
        axes: crate::data::source::source_arc(retained)?,
    })
}

/// Complete one already prepared Scatter component after root range coordination.
pub(crate) fn prepare_scatter_plot(
    source: &Plot,

    marks: crate::plots::scatter::PreparedScatter,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
    InteriorRoute::classify(PlotKind::Scatter).validate(source.description())?;
    let family = PreparedFamily::Scatter(marks);
    let mut axes = prepare(source, None, Some(&family), None)?;
    axes.bind_numeric(source, resources, None, source.axis_scope())?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges,
    )?;
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
    panels.push((scope()?, family));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(PlotKind::Scatter),
        discrete_guides: std::sync::OnceLock::new(),
        axes: crate::data::source::source_arc(retained_axes)?,
        families: crate::data::source::source_arc(panels)?,
    })
}

/// Complete one retained line component after the parent's range coordination.
pub(crate) fn prepare_line_plot(
    source: &Plot,

    marks: crate::plots::line::PreparedLine,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedInteriorInputs, Error> {
    let family = PreparedFamily::Line {
        kind: source.kind(),
        marks,
    };
    let mut axes = prepare(source, None, Some(&family), None)?;
    axes.bind_numeric(source, resources, None, source.axis_scope())?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges,
    )?;
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
    panels.push((scope()?, family));
    Ok(PreparedInteriorInputs {
        route: InteriorRoute::classify(source.kind()),
        discrete_guides: std::sync::OnceLock::new(),
        axes: crate::data::source::source_arc(retained_axes)?,
        families: crate::data::source::source_arc(panels)?,
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
    reason = "closed scientific axis dispatch; category lengths are bounded source rows"
)]
fn prepare(
    chart: &Plot,

    facet: Option<crate::data::FacetAxisSources<'_>>,
    family: Option<&PreparedFamily>,
    reference_config: Option<ReferenceAxisConfig<'_>>,
) -> Result<PreparedAxes, Error> {
    // Two scalar domain results and their fixed five-sample nice-range seeds.

    #[cfg(test)]
    super::AXIS_PREPARATIONS.with(|count| count.set(count.get() + 1));
    macro_rules! marks {
        ($source:expr, $pattern:pat => $marks:ident, $error:literal) => {{
            let Some($pattern) = $source else {
                return Err(Error::Render($error.into()));
            };
            $marks
        }};
    }
    let reference = reference_config.as_ref().and(family);
    let mut categories: [Option<Vec<String>>; 2] = [None, None];
    let mut ranges = match chart.kind() {
        PlotKind::Regression => {
            let ranges = marks!(family, PreparedFamily::Regression(marks) => marks, "Regression axes require prepared semantics").ranges;
            [
                chart
                    .description()
                    .x_axis_spec()
                    .limits
                    .unwrap_or(ranges[0]),
                chart
                    .description()
                    .y_axis_spec()
                    .limits
                    .unwrap_or(ranges[1]),
            ]
        }
        PlotKind::Scatter => {
            let ranges = marks!(family, PreparedFamily::Scatter(marks) => marks, "Scatter axis inputs require prepared semantics").ranges;
            [
                crate::layout::continuous_auto_range(ranges[0], chart.description().x_axis_spec())?,
                crate::layout::continuous_auto_range(ranges[1], chart.description().y_axis_spec())?,
            ]
        }
        PlotKind::Histogram => {
            let ranges = marks!(family, PreparedFamily::Histogram(marks) => marks, "Histogram axes require prepared semantics").ranges;
            [
                chart
                    .description()
                    .x_axis_spec()
                    .limits
                    .unwrap_or(ranges[0]),
                chart
                    .description()
                    .y_axis_spec()
                    .limits
                    .unwrap_or(ranges[1]),
            ]
        }
        PlotKind::Density => marks!(family, PreparedFamily::Density(marks) => marks, "Density axes require prepared semantics").ranges(chart.description())?,
        PlotKind::Qq => marks!(family, PreparedFamily::Qq(marks) => marks, "Qq axes require prepared semantics").ranges(chart.description())?,
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let line = marks!(family, PreparedFamily::Line { marks, .. } => marks, "line axis inputs require prepared semantics");
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
                chart
                    .description()
                    .x_axis_spec()
                    .limits
                    .unwrap_or(line.ranges[0]),
                chart
                    .description()
                    .y_axis_spec()
                    .limits
                    .unwrap_or(line.ranges[1]),
            ]
        }
        PlotKind::Marginal => {
            marks!(reference, PreparedFamily::Marginal(marks) => marks, "marginal axes require retained semantics").ranges
        }
        PlotKind::Bar
        | PlotKind::StackedBar
        | PlotKind::HorizontalBar
        | PlotKind::StackedHorizontalBar => {
            let marks = marks!(reference, PreparedFamily::Bars { marks, .. } => marks, "bar axes require prepared semantics");
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
                    chart.description().x_axis_spec()
                } else {
                    chart.description().y_axis_spec()
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
            let marks = marks!(reference, PreparedFamily::Distribution { marks, .. } => marks, "categorical distribution axes require retained semantics");
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
                crate::layout::continuous_auto_range(values, chart.description().y_axis_spec())?,
            ]
        }
        PlotKind::Dumbbell => {
            let marks = marks!(reference, PreparedFamily::Dumbbell(marks) => marks, "Dumbbell axes require prepared semantics");
            categories[1] = Some(crate::data::source::collect_source(
                marks
                    .categories
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?);
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.description().x_axis_spec(),
                )?,
                (0.0, marks.categories.len() as f64),
            ]
        }
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let config = reference_config.as_ref().ok_or_else(|| {
                Error::Render("estimate axes require root-prepared semantics".into())
            })?;
            let marks = marks!(family, PreparedFamily::Estimate { marks, .. } => marks, "estimate axes require retained rows");
            categories[1] = Some(crate::data::source::collect_source(
                marks
                    .labels
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?);
            let range = marks
                .range
                .or(config.estimate_fallback)
                .map_or_else(|| marks.x_range(chart.kind()), Ok)?;
            [range, (0.0, marks.labels.len() as f64)]
        }
        PlotKind::Interaction => {
            let marks = marks!(reference, PreparedFamily::Interaction(marks) => marks, "interaction axes require retained semantics");
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
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.description().y_axis_spec(),
                )?,
            ]
        }
        PlotKind::Beeswarm => {
            let marks = marks!(reference, PreparedFamily::Beeswarm(marks) => marks, "beeswarm axes require retained semantics");
            let levels = crate::data::source::collect_source(
                marks
                    .levels
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?;
            let slots = chart
                .description()
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.x_data,
                    chart.description().x_axis_spec(),
                )?,
                slots,
            ]
        }
        PlotKind::Waterfall => {
            let marks = marks!(reference, PreparedFamily::Waterfall(marks) => marks, "waterfall axes require retained semantics");
            let levels = crate::data::source::collect_source(
                marks
                    .labels
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?;
            let slots = chart
                .description()
                .y_axis_spec()
                .limits
                .unwrap_or((0.0, levels.len() as f64));
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.description().x_axis_spec(),
                )?,
                slots,
            ]
        }
        PlotKind::Heatmap => {
            let marks = marks!(reference, PreparedFamily::Heatmap(marks) => marks, "heatmap axes require retained semantics");
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
    let logarithmic = chart.kind() == PlotKind::Forest
        && chart.description().axis_scale_spec() == AxisScale::Log10;
    let mut specs = [AxisSpec::default(), AxisSpec::default()];
    for (side, source) in [
        chart.description().x_axis_spec(),
        chart.description().y_axis_spec(),
    ]
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
                chart.description().projected_explicit_x_ticks()?
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
                chart
                    .description()
                    .resolve_axis_label_domain(domain, raw_labels)?
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
        chart.description().prepare_axis_labels(labels)?
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
