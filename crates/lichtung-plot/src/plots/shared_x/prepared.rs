use crate::layout::interior::{InteriorRoute, PreparedInteriorInputs};
use crate::layout::AxisTick;
use crate::orchestrator::registration::{
    AxisSlot, AxisSourceWitness, AxisSpace, DisplayExtent, PreparedAxes, PreparedAxesOwner,
    PreparedAxisSource, PreparedCategorySource,
};
use crate::orchestrator::registration::{
    PlotRegistrationBuild, PlotRegistryBuilder, PreparedFamily, RowSelection, Scope,
};
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::{Error, Plot};

/// Build-only seeds are consumed by the original policy and parent passes
/// before registration publication; no range helper or formatter runs here.
pub(crate) fn prepare_pair_axes(
    build: &PlotRegistrationBuild<'_>,
    ranges: [(f64, f64); 2],
    categories: [Option<Vec<String>>; 2],
) -> Result<PreparedAxesOwner, Error> {
    let policies = crate::layout::range_policies(build.source.kind());
    let [x, y] = ranges.map(|(lower, upper)| Some(DisplayExtent { lower, upper }));
    let [x_categories, y_categories] = categories.map(|raw| {
        raw.map(|raw| PreparedCategorySource {
            raw,
            ticks: Vec::new(),
        })
    });
    Ok(PreparedAxesOwner::Pair(PreparedAxes {
        sources: [
            crate::orchestrator::registration::prepare_axis_source(
                build,
                AxisSlot::Pair(crate::layout::AxisSide::Bottom),
                x,
                if x_categories.is_some() {
                    crate::layout::RangePolicy::CategorySlots
                } else {
                    policies[0]
                },
                x_categories,
                None,
            )?,
            crate::orchestrator::registration::prepare_axis_source(
                build,
                AxisSlot::Pair(crate::layout::AxisSide::Left),
                y,
                if y_categories.is_some() {
                    crate::layout::RangePolicy::CategorySlots
                } else {
                    policies[1]
                },
                y_categories,
                None,
            )?,
        ],
    }))
}

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

impl crate::orchestrator::registration::BaseAxisRef<'_> {
    pub(crate) fn empty_numeric_source(self, raw: Option<(f64, f64)>) -> Result<bool, Error> {
        let generation = &self
            .authored()
            .expect("coordinate authored intent")
            .generation;
        if generation
            .exact_parameters()
            .is_some_and(|(count, _, _)| count == 0)
        {
            return Ok(true);
        }
        let range = self.extent().pair();
        let log = matches!(
            self.source.witness.space,
            crate::orchestrator::registration::AxisSpace::Continuous(AxisScale::Log10)
        );
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
            return crate::layout::fixed_interval_is_empty(range, raw, log, parameters, maximum);
        }
        Ok(false)
    }

    pub(crate) fn numeric_probe(
        self,
        numeric: crate::numeric::source::NumericBinding<'_>,
        windows: [Option<&(f64, f64)>; 2],
    ) -> Result<Option<crate::numeric::set::FormattedSet>, Error> {
        let (AxisSlot::Pair(axis_side) | AxisSlot::MarginalCentral(axis_side)) =
            self.source.witness.slot
        else {
            unreachable!("numeric coordinate source")
        };
        let side = usize::from(axis_side == crate::layout::AxisSide::Left);
        if self.exact_ticks().is_some()
            || self.source.categories.is_some()
            || self.empty_numeric_source(windows[side].copied())?
        {
            return Ok(None);
        }
        let Some(request) = self.numeric_request(numeric, windows)? else {
            return Ok(None);
        };
        let range = self.extent().pair();
        let logarithmic = matches!(
            self.source.witness.space,
            crate::orchestrator::registration::AxisSpace::Continuous(AxisScale::Log10)
        );
        request.materialize(
            &[range.0, f64::midpoint(range.0, range.1), range.1]
                .map(|value| crate::numeric::operand::Operand::number(value, logarithmic)),
            logarithmic,
            if side == 0 {
                crate::layout::AxisSide::Bottom
            } else {
                crate::layout::AxisSide::Left
            },
            range,
            crate::NumericPurpose::IntrinsicProbe,
        )
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "closed scientific dispatch moves each original result into its occurrence before parent completion"
)]
pub(crate) fn prepare_plot_registration(
    panel: &mut PlotRegistrationBuild<'_>,
    source: &Plot,
    owner: Option<&crate::theme::Theme>,
    operation: &crate::font::canonical::OperationContext,
) -> Result<(), Error> {
    let data = panel.data(operation)?;
    let numeric = panel
        .scope
        .numeric_binding(panel.source, &panel.row_source, operation)?;
    let family = match source.kind() {
        PlotKind::Waterfall => Ok(PreparedFamily::Waterfall(crate::plots::waterfall::prepare(
            source, data,
        )?)),
        PlotKind::Beeswarm => {
            let (marks, levels) = crate::plots::beeswarm::prepare(
                source,
                data,
                continuous_selection(source, owner)?,
                numeric,
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, levels.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [marks.x_data, slots],
                [None, Some(levels)],
            )?);
            Ok(PreparedFamily::Beeswarm(marks))
        }
        PlotKind::Marginal => {
            let (marks, axes) = crate::plots::marginal::prepare(source, data, &panel.scope.id)?;
            panel.axes = Some(axes);
            Ok(PreparedFamily::Marginal(marks))
        }
        PlotKind::Heatmap => {
            let (marks, levels) = crate::plots::heatmap::prepare(
                source,
                data,
                continuous_selection(source, owner)?,
                numeric,
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let ranges = levels.each_ref().map(|levels| (0.0, levels.len() as f64));
            panel.axes = Some(prepare_pair_axes(panel, ranges, levels.map(Some))?);
            Ok(PreparedFamily::Heatmap(marks))
        }
        PlotKind::Interaction => Ok(PreparedFamily::Interaction(
            crate::plots::interaction::prepare(source, data, None)?,
        )),
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            Ok(PreparedFamily::Distribution {
                kind: source.kind(),
                marks: crate::plots::categorical_distribution::PreparedDistribution::new(
                    source, data, None,
                )?,
            })
        }
        PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar => Ok(PreparedFamily::Bars {
            kind: source.kind(),
            mode: crate::plots::bar_data::mode(source),
            marks: crate::plots::bar_data::prepare(
                source,
                data,
                crate::plots::bar_data::mode(source),
            )?,
        }),
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            Ok(PreparedFamily::Estimate {
                kind: source.kind(),
                marks: crate::plots::interval_estimate::PreparedEstimate::new(
                    source,
                    data,
                    source.description().significance_annotations(),
                    numeric,
                )?,
            })
        }
        PlotKind::Dumbbell => Ok(PreparedFamily::Dumbbell(crate::plots::dumbbell::prepare(
            source.description(),
            data,
        )?)),
        PlotKind::Histogram => {
            let (marks, ranges) = crate::plots::histogram::HistogramSemantics::new(
                source.description(),
                data,
                owner,
            )?;
            panel.axes = Some(prepare_pair_axes(panel, ranges, [None, None])?);
            Ok(PreparedFamily::Histogram(marks))
        }
        PlotKind::Density => Ok(PreparedFamily::Density(
            crate::plots::density::PreparedDensity::new(source, data)?,
        )),
        PlotKind::Qq => Ok(PreparedFamily::Qq(crate::plots::qq::PreparedQq::new(
            source.description(),
            data,
        )?)),
        PlotKind::Regression => {
            let (marks, ranges) =
                crate::plots::regression::PreparedRegression::new(source.description(), data)?;
            panel.axes = Some(prepare_pair_axes(panel, ranges, [None, None])?);
            Ok(PreparedFamily::Regression(marks))
        }
        _ => Err(Error::Render(
            "shared semantics require an extracted reference or paired-endpoint family".into(),
        )),
    }?;
    panel.family = Some(family);
    Ok(())
}

impl PreparedFamily {
    #[expect(
        clippy::cast_precision_loss,
        reason = "category counts are admitted source rows"
    )]
    pub(crate) fn x_range(
        &self,
        source: &Plot,
        axes: Option<&PreparedAxesOwner>,
    ) -> Result<(f64, f64), Error> {
        Ok(match self {
            Self::Waterfall(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source.description().x_axis_spec(),
            )?,
            Self::Beeswarm(marks) => crate::layout::continuous_auto_range(
                marks.x_data,
                source.description().x_axis_spec(),
            )?,
            Self::Marginal(_) => axes.expect("prepared Marginal axes").ranges()[0],
            Self::Heatmap(_) => (
                0.0,
                axes.expect("prepared Heatmap axes")
                    .categories(0)
                    .expect("prepared Heatmap categories")
                    .raw
                    .len() as f64,
            ),
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
            Self::Histogram(_) => axes.expect("prepared Histogram axes").ranges()[0],
            Self::Density(marks) => marks.ranges(source.description())?[0],
            Self::Qq(marks) => marks.ranges(source.description())?[0],
            Self::Regression(_) => axes.expect("prepared Regression axes").ranges()[0],
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
    panel: &mut PlotRegistrationBuild<'_>,
    x_limits: Option<(f64, f64)>,
    labels: Option<&crate::Labels>,
    resources: &crate::font::canonical::OperationContext,
) -> Result<(), Error> {
    let mut family = match panel.family.take().expect("prepared reference family") {
        PreparedFamily::Beeswarm(mut marks) => {
            if labels.is_some() {
                marks.color_label = source.description().prepare_color_label(labels)?;
            }
            PreparedFamily::Beeswarm(marks)
        }
        PreparedFamily::Marginal(marks) => {
            PreparedFamily::Marginal(crate::plots::marginal::finish(
                marks,
                panel.axes.as_mut().expect("prepared Marginal axes"),
                x_limits,
            )?)
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
    let axes = prepare(
        source,
        &panel.scope.id,
        None,
        Some(&family),
        Some(config),
        panel.axes.take(),
    )?;
    if let PreparedFamily::Waterfall(marks) = &mut family {
        marks.prepare_totals(
            crate::plots::waterfall::WaterfallTotalsPrepare {
                numeric: panel
                    .scope
                    .numeric_binding(panel.source, &panel.row_source, resources)?,
                axes: [axes.axis(panel.source, 0), axes.axis(panel.source, 1)],
                format: source.description().total_format_spec(),
            },
            false,
        )?;
    }
    if let PreparedFamily::Qq(quantiles) = &family {
        quantiles.validate_ranges(axes.ranges())?;
    }
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges(),
    )?;
    panel.axes = Some(axes);
    panel.family = Some(family);
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "root preparation retains per-component scientific axes and admitted point/path semantics before fonts"
)]
pub(crate) fn prepare_plot<'src>(
    source: &'src Plot,
    resources: &crate::font::canonical::OperationContext,
    owner: Option<&crate::theme::Theme>,
) -> Result<PreparedInteriorInputs<'src>, Error> {
    let mut registry = PlotRegistryBuilder::new(resources.root_identity());
    let source_rows = source.description().configured_source_rows()?;
    let source_id = (!source.description().has_facets())
        .then(|| {
            let scope = Scope::new(
                source,
                resources,
                0,
                RowSelection::Full,
                None,
                source
                    .axis_scope()
                    .map(crate::data::source::copy_source_string)
                    .transpose()?,
            )?;
            registry.reserve(source.description(), scope, resources)
        })
        .transpose()?;
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
        prepare_plot_registration(&mut registry.panels[0], source, owner, resources)?;
        prepare_reference_plot(source, &mut registry.panels[0], None, None, resources)?;
        return PreparedInteriorInputs::from_registry(
            registry,
            InteriorRoute::classify(source.kind()),
        );
    }
    let source_data = source_id
        .as_ref()
        .map(|id| registry.data(id, resources))
        .transpose()?;
    if source.kind() == PlotKind::Scatter && !source.description().has_facets() {
        let (marks, ranges) = crate::plots::scatter::PreparedScatter::new(
            source.description(),
            source_data.expect("single plot data"),
        )?;
        registry.panels[0].axes = Some(prepare_pair_axes(
            &registry.panels[0],
            ranges,
            [None, None],
        )?);
        registry.panels[0].family = Some(PreparedFamily::Scatter(marks));
        prepare_scatter_plot(source, &mut registry.panels[0])?;
        return PreparedInteriorInputs::from_registry(
            registry,
            InteriorRoute::classify(source.kind()),
        );
    }
    if matches!(
        source.kind(),
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
    ) && !source.description().has_facets()
    {
        let (marks, ranges, conditions) = crate::plots::line::PreparedLine::new(
            source,
            source_data.expect("single plot data"),
            None,
        )?;
        registry.panels[0].axes = Some(prepare_pair_axes(
            &registry.panels[0],
            ranges,
            [conditions, None],
        )?);
        registry.panels[0].family = Some(PreparedFamily::Line {
            kind: source.kind(),
            marks,
        });
        prepare_line_plot(source, &mut registry.panels[0])?;
        return PreparedInteriorInputs::from_registry(
            registry,
            InteriorRoute::classify(source.kind()),
        );
    }
    if source.description().has_facets() {
        crate::orchestrator::prepare_facet_scopes(source, &mut registry, resources)?;
        let interaction_parent = (source.kind() == PlotKind::Interaction)
            .then(|| crate::data::observed_source_collectives(source.description(), "interaction"))
            .transpose()?;
        if matches!(
            source.kind(),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
        ) {
            for panel in &mut registry.panels {
                let marks = crate::plots::interval_estimate::PreparedEstimate::new(
                    source,
                    panel.data(resources)?,
                    panel.significance_annotations(),
                    panel.scope.numeric_binding(
                        source.description(),
                        &panel.row_source,
                        resources,
                    )?,
                )?;
                panel.family = Some(PreparedFamily::Estimate {
                    kind: source.kind(),
                    marks,
                });
            }
        }
        let mut estimate_fallback: Option<(f64, f64)> = None;
        if matches!(
            source.description().facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeY
        ) {
            for panel in &registry.panels {
                let Some(PreparedFamily::Estimate { marks, .. }) = &panel.family else {
                    continue;
                };
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
                crate::data::source::source_levels(&facet_source_strings(
                    source.description(),
                    column,
                    source_rows,
                    "facet graphical group",
                    None,
                )?)
            })
            .transpose()?;
        let conditions = if source.kind() == PlotKind::Trajectory
            && source
                .description()
                .x_col()
                .map(|column| {
                    source
                        .description()
                        .data_column(column)
                        .ok_or_else(|| Error::Data(format!("unknown semantic value {column:?}")))
                        .map(|column| !column.is_numeric())
                })
                .transpose()?
                .unwrap_or(false)
        {
            Some(crate::data::source::source_levels(&facet_source_strings(
                source.description(),
                source.description().x_col().expect("categorical x"),
                source_rows,
                "trajectory",
                Some("condition"),
            )?)?)
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
                crate::data::source::source_levels(&facet_source_strings(
                    source.description(),
                    column,
                    source_rows,
                    "facet hue",
                    None,
                )?)
            })
            .transpose()?;
        for panel in &mut registry.panels {
            let prepared = panel.family.take();
            let data = panel.data(resources)?;
            let facet = crate::data::FacetAxisSources {
                hues: hues.as_deref(),
                groups: groups.as_deref(),
                conditions: conditions.as_deref(),
            };
            let family = match source.kind() {
                PlotKind::Scatter => {
                    let (marks, ranges) =
                        crate::plots::scatter::PreparedScatter::new(source.description(), data)?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges, [None, None])?);
                    PreparedFamily::Scatter(marks)
                }
                PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                    let (marks, ranges, conditions) =
                        crate::plots::line::PreparedLine::new(source, data, facet.conditions)?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges, [conditions, None])?);
                    PreparedFamily::Line {
                        kind: source.kind(),
                        marks,
                    }
                }
                PlotKind::Histogram => {
                    let (marks, ranges) = crate::plots::histogram::HistogramSemantics::new(
                        source.description(),
                        data,
                        owner,
                    )?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges, [None, None])?);
                    PreparedFamily::Histogram(marks)
                }
                PlotKind::Density => PreparedFamily::Density(
                    crate::plots::density::PreparedDensity::new(source, data)?,
                ),
                PlotKind::Qq => PreparedFamily::Qq(crate::plots::qq::PreparedQq::new(
                    source.description(),
                    data,
                )?),
                PlotKind::Regression => {
                    let (marks, ranges) = crate::plots::regression::PreparedRegression::new(
                        source.description(),
                        data,
                    )?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges, [None, None])?);
                    PreparedFamily::Regression(marks)
                }
                PlotKind::Dumbbell => PreparedFamily::Dumbbell(crate::plots::dumbbell::prepare(
                    source.description(),
                    data,
                )?),
                PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                    prepared.expect("prepared facet estimates")
                }
                PlotKind::Heatmap => {
                    let (marks, levels) = crate::plots::heatmap::prepare(
                        source,
                        data,
                        continuous_selection(source, owner)?,
                        panel.scope.numeric_binding(
                            source.description(),
                            &panel.row_source,
                            resources,
                        )?,
                    )?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let ranges = levels.each_ref().map(|levels| (0.0, levels.len() as f64));
                    panel.axes = Some(prepare_pair_axes(panel, ranges, levels.map(Some))?);
                    PreparedFamily::Heatmap(marks)
                }
                PlotKind::Beeswarm => {
                    let (marks, levels) = crate::plots::beeswarm::prepare(
                        source,
                        data,
                        continuous_selection(source, owner)?,
                        panel.scope.numeric_binding(
                            source.description(),
                            &panel.row_source,
                            resources,
                        )?,
                    )?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, levels.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [marks.x_data, slots],
                        [None, Some(levels)],
                    )?);
                    PreparedFamily::Beeswarm(marks)
                }
                PlotKind::Waterfall => {
                    PreparedFamily::Waterfall(crate::plots::waterfall::prepare(source, data)?)
                }
                PlotKind::Interaction => PreparedFamily::Interaction(
                    crate::plots::interaction::prepare(source, data, interaction_parent.as_ref())?,
                ),
                PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                    PreparedFamily::Distribution {
                        kind: source.kind(),
                        marks: crate::plots::categorical_distribution::PreparedDistribution::new(
                            source,
                            data,
                            Some(facet),
                        )?,
                    }
                }
                PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar => PreparedFamily::Bars {
                    kind: source.kind(),
                    mode: crate::plots::bar_data::mode(source),
                    marks: crate::plots::bar_data::prepare_scoped(
                        source,
                        data,
                        crate::plots::bar_data::mode(source),
                        Some(facet),
                        panel.significance_annotations(),
                    )?,
                },
                PlotKind::Matrix | PlotKind::Marginal => {
                    panel.axes = Some(prepare(
                        source,
                        &panel.scope.id,
                        Some(facet),
                        None,
                        Some(ReferenceAxisConfig {
                            estimate_fallback,
                            ..ReferenceAxisConfig::default()
                        }),
                        panel.axes.take(),
                    )?);
                    continue;
                }
            };
            let axes = prepare(
                source,
                &panel.scope.id,
                Some(facet),
                Some(&family),
                Some(ReferenceAxisConfig {
                    estimate_fallback,
                    ..ReferenceAxisConfig::default()
                }),
                panel.axes.take(),
            )?;
            panel.family = Some(family);
            panel.axes = Some(axes);
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
            reconcile_facet_axis(&mut registry.panels, side)?;
        }
        for panel in &mut registry.panels {
            if let Some(PreparedFamily::Waterfall(marks)) = &mut panel.family {
                marks.prepare_totals(
                    crate::plots::waterfall::WaterfallTotalsPrepare {
                        numeric: panel.scope.numeric_binding(
                            panel.source,
                            &panel.row_source,
                            resources,
                        )?,
                        axes: [
                            panel
                                .axes
                                .as_ref()
                                .expect("prepared facet axes")
                                .axis(panel.source, 0),
                            panel
                                .axes
                                .as_ref()
                                .expect("prepared facet axes")
                                .axis(panel.source, 1),
                        ],
                        format: source.description().total_format_spec(),
                    },
                    false,
                )?;
            }
        }
        for panel in &registry.panels {
            let Some(PreparedFamily::Qq(quantiles)) = &panel.family else {
                continue;
            };
            quantiles
                .validate_ranges(panel.axes.as_ref().expect("prepared facet axes").ranges())?;
        }
        for panel in &registry.panels {
            let axes = panel.axes.as_ref().expect("prepared facet axes");
            if matches!(
                source.kind(),
                PlotKind::Bar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedBar
                    | PlotKind::StackedHorizontalBar
            ) {
                for side in 0..2 {
                    if let Some(ticks) = axes.axis(panel.source, side).exact_ticks() {
                        crate::layout::Axis::validate_exact_ticks(
                            &ticks,
                            axes.ranges()[side],
                            if side == 0 { "x-axis" } else { "y-axis" },
                        )?;
                    }
                }
            }
            crate::orchestrator::layers::GuideSnapshot::validate_sources(
                source.description(),
                axes.ranges(),
            )?;
        }
    } else if source.kind() == PlotKind::Matrix {
        let (marks, axes) = crate::plots::matrix::prepare_axes(
            source,
            source_data.expect("single plot data"),
            &registry.panels[0].scope.id,
        )?;
        registry.panels[0].family = Some(PreparedFamily::Matrix(marks));
        registry.panels[0].axes = Some(axes);
    } else {
        let axes = prepare(source, &registry.panels[0].scope.id, None, None, None, None)?;
        registry.panels[0].axes = Some(axes);
    }
    PreparedInteriorInputs::from_registry(registry, InteriorRoute::classify(source.kind()))
}

fn facet_source_strings(
    source: &crate::builder::PlotDescription,
    key: &str,
    rows: Option<&crate::SourceRows>,
    context: &str,
    normalized_role: Option<&str>,
) -> Result<Vec<String>, Error> {
    let column = source
        .data_column(key)
        .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))?;
    let crate::data::DataColumn::Category(values) = column else {
        return Err(Error::Data(format!(
            "{context}: value {:?} must be categorical",
            column.name()
        )));
    };
    normalized_role.map_or_else(
        || crate::data::strict_source_strings(values, rows, context),
        |role| crate::data::strict_source_normalized_strings(values, rows, context, role),
    )
}

/// Complete one already prepared Scatter component after root range coordination.
pub(crate) fn prepare_scatter_plot(
    source: &Plot,
    panel: &mut PlotRegistrationBuild<'_>,
) -> Result<(), Error> {
    InteriorRoute::classify(PlotKind::Scatter).validate(source.description())?;
    let axes = prepare(
        source,
        &panel.scope.id,
        None,
        panel.family.as_ref(),
        None,
        panel.axes.take(),
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges(),
    )?;
    panel.axes = Some(axes);
    Ok(())
}

/// Complete one retained line component after the parent's range coordination.
pub(crate) fn prepare_line_plot(
    source: &Plot,
    panel: &mut PlotRegistrationBuild<'_>,
) -> Result<(), Error> {
    let axes = prepare(
        source,
        &panel.scope.id,
        None,
        panel.family.as_ref(),
        None,
        panel.axes.take(),
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(
        source.description(),
        axes.ranges(),
    )?;
    panel.axes = Some(axes);
    Ok(())
}

#[expect(
    clippy::float_cmp,
    reason = "fixed categories preserve the original derived raw AxisTick equality, including signed zero"
)]
fn reconcile_facet_axis(
    retained: &mut [PlotRegistrationBuild<'_>],
    side: usize,
) -> Result<(), Error> {
    let Some(first) = retained.first() else {
        return Ok(());
    };
    let first_source = first.source;
    let first = first.axes.as_ref().expect("prepared facet axes");
    for panel in retained.iter() {
        let axes = panel.axes.as_ref().expect("prepared facet axes");
        let same = match (axes.categories(side), first.categories(side)) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.raw == b.raw
                    && a.ticks.len() == b.ticks.len()
                    && a.ticks
                        .iter()
                        .zip(&b.ticks)
                        .all(|(a, b)| a.position == b.position)
            }
            _ => false,
        };
        if !same {
            return Err(Error::Render(format!("fixed facet {} axis requires identical categorical levels and ordering in every panel; use a free scale or provide complete panel data", if side == 0 { "x" } else { "y" })));
        }
    }
    if first.categories(side).is_none() {
        fn presentation<'a>(
            axes: &'a PreparedAxesOwner,
            source: &'a crate::builder::PlotDescription,
            side: usize,
        ) -> (bool, Option<&'a crate::numeric::format::NumericFormat>) {
            (
                axes.logarithmic() && side == 0,
                if axes.axis(source, side).exact_ticks().is_some() {
                    None
                } else {
                    axes.axis(source, side)
                        .authored()
                        .expect("coordinate authored intent")
                        .format
                        .as_deref()
                },
            )
        }
        for panel in &*retained {
            let axes = panel.axes.as_ref().expect("prepared facet axes");
            if presentation(axes, panel.source, side) != presentation(first, first_source, side) {
                return Err(Error::Data(
                    "fixed facet numeric axes require identical numeric formats and operand contexts"
                        .into(),
                ));
            }
        }
        let union = retained
            .iter()
            .map(|panel| panel.axes.as_ref().expect("prepared facet axes").ranges()[side])
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, b| {
                (a.0.min(b.0), a.1.max(b.1))
            });
        validate_range(union)?;

        for panel in retained {
            let axes = panel.axes.as_mut().expect("prepared facet axes");
            axes.sources_mut()[side].base_seed =
                Some(crate::orchestrator::registration::DisplayExtent {
                    lower: union.0,
                    upper: union.1,
                });
            if axes.sources()[side].base_provenance != crate::layout::RangeProvenance::Configured {
                axes.sources_mut()[side].base_provenance = crate::layout::RangeProvenance::Shared;
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
    plot_id: &crate::orchestrator::registration::PlotOccurrenceId,

    facet: Option<crate::data::FacetAxisSources<'_>>,
    family: Option<&PreparedFamily>,
    reference_config: Option<ReferenceAxisConfig<'_>>,
    mut prepared_axes: Option<PreparedAxesOwner>,
) -> Result<PreparedAxesOwner, Error> {
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
            marks!(family, PreparedFamily::Regression(marks) => marks, "Regression axes require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Regression axes").ranges();
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
            marks!(family, PreparedFamily::Scatter(marks) => marks, "Scatter axis inputs require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Scatter axes").ranges();
            [
                crate::layout::continuous_auto_range(ranges[0], chart.description().x_axis_spec())?,
                crate::layout::continuous_auto_range(ranges[1], chart.description().y_axis_spec())?,
            ]
        }
        PlotKind::Histogram => {
            marks!(family, PreparedFamily::Histogram(marks) => marks, "Histogram axes require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Histogram axes").ranges();
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
            marks!(family, PreparedFamily::Line { marks, .. } => marks, "line axis inputs require prepared semantics");
            let axes = prepared_axes.as_mut().expect("prepared line axes");
            categories[0] = axes.sources_mut()[0].categories.take().map(|categories| categories.raw);
            let ranges = axes.ranges();
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
        PlotKind::Marginal => {
            marks!(reference, PreparedFamily::Marginal(marks) => marks, "marginal axes require retained semantics");
            prepared_axes.as_ref().expect("prepared Marginal axes").ranges()
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
            let levels = prepared_axes.as_mut().expect("prepared Beeswarm axes").sources_mut()[1]
                .categories.take().expect("prepared Beeswarm categories").raw;
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
            marks!(reference, PreparedFamily::Heatmap(marks) => marks, "heatmap axes require retained semantics");
            let [x, y] = prepared_axes.as_mut().expect("prepared Heatmap axes").sources_mut().each_mut()
                .map(|source| source.categories.take().expect("prepared Heatmap categories").raw);
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
    let specs = [
        chart.description().x_axis_spec(),
        chart.description().y_axis_spec(),
    ];
    let mut provenance = specs.map(|spec| spec.provenance);
    if x_limits.is_some() && provenance[0] != crate::layout::RangeProvenance::Configured {
        provenance[0] = crate::layout::RangeProvenance::Shared;
    }
    let mut projected_exact_positions = [
        if logarithmic {
            chart.description().projected_explicit_x_ticks()?
        } else {
            None
        },
        None,
    ];
    let mut prepared_categories = [None, None];
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
            let display = if let Some(policy) = labels {
                policy.resolve_domain(domain, raw_labels)?
            } else {
                chart
                    .description()
                    .resolve_axis_label_domain(domain, raw_labels)?
            };
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
            for (index, label) in display.into_iter().enumerate() {
                retained.push(AxisTick::new(
                    if reverse {
                        count - index as f64 - 0.5
                    } else {
                        index as f64 + 0.5
                    },
                    label,
                ));
            }
            prepared_categories[side] = Some(PreparedCategorySource {
                raw: levels,
                ticks: retained,
            });
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

            crate::layout::Axis::validate_exact_ticks(
                &crate::orchestrator::registration::ExactTicksRef {
                    raw: exact,
                    projected: projected_exact_positions[side].as_deref(),
                },
                ranges[side],
                if side == 0 { "x-axis" } else { "y-axis" },
            )?;
        }
    }
    let mut labels = if facet.is_some() {
        [None, None]
    } else {
        chart.description().prepare_axis_labels(labels)?
    };
    let policies = crate::layout::range_policies(chart.kind());
    let mut axes = prepared_axes.unwrap_or_else(|| {
        PreparedAxesOwner::Pair(PreparedAxes {
            sources: std::array::from_fn(|side| PreparedAxisSource {
                witness: AxisSourceWitness {
                    plot_id: plot_id.clone(),
                    slot: AxisSlot::Pair(if side == 0 {
                        crate::layout::AxisSide::Bottom
                    } else {
                        crate::layout::AxisSide::Left
                    }),
                    space: AxisSpace::Continuous(AxisScale::Linear),
                },
                range_policy: policies[side],
                base_seed: None,
                base_provenance: provenance[side],
                categories: None,
                projected_exact_positions: None,
                label: std::sync::OnceLock::new(),
            }),
        })
    });
    for (side, source) in axes.sources_mut().iter_mut().enumerate() {
        let categories = prepared_categories[side].take();
        source.witness.space = if categories.is_some() {
            AxisSpace::Categorical
        } else {
            AxisSpace::Continuous(if logarithmic && side == 0 {
                AxisScale::Log10
            } else {
                AxisScale::Linear
            })
        };
        source.range_policy = if categories.is_some() {
            crate::layout::RangePolicy::CategorySlots
        } else {
            policies[side]
        };
        source.base_seed = Some(DisplayExtent {
            lower: ranges[side].0,
            upper: ranges[side].1,
        });
        source.base_provenance = provenance[side];
        source.categories = categories;
        source.projected_exact_positions = projected_exact_positions[side].take();
        if facet.is_none() {
            let _ = source
                .label
                .set(labels[side].take().unwrap_or(crate::Text::UNSET));
        }
    }
    Ok(axes)
}
