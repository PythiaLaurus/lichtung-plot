use crate::builder::PlotDescription;
use crate::layout::interior::InteriorRoute;
use crate::layout::AxisTick;
use crate::orchestrator::registration::{
    AxisSlot, AxisSourceWitness, AxisSpace, DisplayExtent, PreparedAxes, PreparedAxesOwner,
    PreparedAxisSource, PreparedCategorySource,
};
use crate::orchestrator::registration::{
    PlotRegistrationBuild, PlotRegistryBuilder, PreparedFamily, RowSelection, Scope,
};
use crate::spec::{AxisScale, FacetScales, PlotKind};
use crate::Error;

/// Build-only seeds are consumed by the original policy and parent passes
/// before registration publication; no range helper or formatter runs here.
pub(crate) fn prepare_pair_axes(
    build: &PlotRegistrationBuild<'_>,
    ranges: [Option<(f64, f64)>; 2],
    categories: [Option<Vec<String>>; 2],
) -> Result<PreparedAxesOwner, Error> {
    let policies = crate::layout::range_policies(build.source.kind());
    let [x, y] = ranges.map(|range| range.map(|(lower, upper)| DisplayExtent { lower, upper }));
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
    source: &PlotDescription,
    appearance: &crate::theme::ResolvedTheme,
) -> Result<crate::theme::ContinuousColorScale, Error> {
    source.continuous_selection_in(appearance.color_scales())
}

impl crate::orchestrator::registration::BaseAxisRef<'_> {
    pub(crate) fn empty_numeric_source(self, raw: Option<(f64, f64)>) -> Result<bool, Error> {
        let generation = &self
            .authored()
            .expect("coordinate authored intent")
            .generation;
        let log = matches!(
            self.source.witness.space,
            crate::orchestrator::registration::AxisSpace::Continuous(AxisScale::Log10)
        );
        crate::numeric::source::empty_fixed_source(generation, self.source.base_seed, log, raw)
    }

    pub(crate) fn numeric_probe(
        self,
        numeric: crate::numeric::source::NumericBinding<'_>,
    ) -> Result<Option<crate::numeric::set::FormattedSet>, Error> {
        let (AxisSlot::Pair(axis_side) | AxisSlot::MarginalCentral(axis_side)) =
            self.source.witness.slot
        else {
            unreachable!("numeric coordinate source")
        };
        if self.exact_ticks()?.is_some() || self.source.categories.is_some() {
            return Ok(None);
        }
        let Some(request) = crate::numeric::source::AxisRequest::base(numeric, self, axis_side)?
        else {
            return Ok(None);
        };
        if request.empty_fixed_source()? {
            return Ok(None);
        }
        let range = self.extent()?.pair();
        let logarithmic = matches!(
            self.source.witness.space,
            crate::orchestrator::registration::AxisSpace::Continuous(AxisScale::Log10)
        );
        let Some(format) = request.format else {
            return Ok(None);
        };
        format.materialize(
            request.binding.operation,
            request.probe_context(
                &[range.0, f64::midpoint(range.0, range.1), range.1]
                    .map(|value| crate::numeric::operand::Operand::number(value, logarithmic)),
            )?,
            false,
        )
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "closed scientific dispatch moves each original result into its occurrence before parent completion"
)]
pub(crate) fn prepare_plot_registration(
    panel: &mut PlotRegistrationBuild<'_>,
    operation: &crate::font::canonical::OperationContext,
) -> Result<(), Error> {
    let source = panel.source;
    let data = panel.data(operation)?;
    let numeric = panel
        .scope
        .numeric_binding(panel.source, &panel.row_source, operation)?;
    let family = match source.kind() {
        PlotKind::Waterfall => {
            let (marks, labels) = crate::plots::waterfall::prepare(source, data)?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, labels.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [marks.range, slots].map(Some),
                [None, Some(labels)],
            )?);
            Ok(PreparedFamily::Waterfall(marks))
        }
        PlotKind::Beeswarm => {
            let (marks, levels) = crate::plots::beeswarm::prepare(
                source,
                data,
                continuous_selection(source, &panel.appearance)?,
                numeric,
                panel.policy.continuous_materialization,
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, levels.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [marks.x_data, slots].map(Some),
                [None, Some(levels)],
            )?);
            Ok(PreparedFamily::Beeswarm(marks))
        }
        PlotKind::Marginal => {
            let (marks, axes, hues) =
                crate::plots::marginal::prepare(source, data, &panel.scope.id)?;
            panel.axes = Some(axes);
            panel
                .categorical
                .as_mut()
                .expect("prepared color binding")
                .set_local_hues(hues)?;
            Ok(PreparedFamily::Marginal(marks))
        }
        PlotKind::Heatmap => {
            let (marks, levels) = crate::plots::heatmap::prepare(
                source,
                data,
                continuous_selection(source, &panel.appearance)?,
                numeric,
                panel.policy.continuous_materialization,
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let ranges = levels.each_ref().map(|levels| (0.0, levels.len() as f64));
            panel.axes = Some(prepare_pair_axes(
                panel,
                ranges.map(Some),
                levels.map(Some),
            )?);
            Ok(PreparedFamily::Heatmap(marks))
        }
        PlotKind::Interaction => {
            let (marks, conditions) = crate::plots::interaction::prepare(
                source,
                data,
                None,
                panel
                    .parent_orders
                    .collectives
                    .as_deref()
                    .map(Vec::as_slice),
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, conditions.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [slots, marks.range].map(Some),
                [Some(conditions), None],
            )?);
            Ok(PreparedFamily::Interaction(marks))
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            let (marks, categories, domains) =
                crate::plots::categorical_distribution::PreparedDistribution::new(
                    source, data, None,
                )?;
            panel
                .categorical
                .as_mut()
                .expect("prepared color binding")
                .set_local_hues(domains)?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, categories.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [slots, marks.range()].map(Some),
                [Some(categories), None],
            )?);
            Ok(PreparedFamily::Distribution {
                kind: source.kind(),
                marks,
            })
        }
        PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar => {
            let (marks, categories, domains) = crate::plots::bar_data::prepare(
                source,
                data,
                crate::plots::bar_data::mode(source),
            )?;
            panel
                .categorical
                .as_mut()
                .expect("prepared color binding")
                .set_local_hues(domains)?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, categories.len() as f64);
            let horizontal = matches!(
                source.kind(),
                PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
            );
            let (ranges, categories) = if horizontal {
                ([marks.value_range, slots], [None, Some(categories)])
            } else {
                ([slots, marks.value_range], [Some(categories), None])
            };
            panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), categories)?);
            Ok(PreparedFamily::Bars {
                kind: source.kind(),
                mode: crate::plots::bar_data::mode(source),
                marks,
            })
        }
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let (marks, range, labels) = crate::plots::interval_estimate::PreparedEstimate::new(
                source,
                data,
                source.significance_annotations(),
                numeric,
            )?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, labels.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [range, Some(slots)],
                [None, Some(labels)],
            )?);
            Ok(PreparedFamily::Estimate {
                kind: source.kind(),
                marks,
            })
        }
        PlotKind::Dumbbell => {
            let (marks, categories) = crate::plots::dumbbell::prepare(source, data)?;
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, categories.len() as f64);
            panel.axes = Some(prepare_pair_axes(
                panel,
                [marks.range, slots].map(Some),
                [None, Some(categories)],
            )?);
            Ok(PreparedFamily::Dumbbell(marks))
        }
        PlotKind::Histogram => {
            let (marks, ranges) = crate::plots::histogram::HistogramSemantics::new(
                source,
                data,
                Some(&panel.appearance),
            )?;
            panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), [None, None])?);
            Ok(PreparedFamily::Histogram(marks))
        }
        PlotKind::Density => Ok(PreparedFamily::Density(
            crate::plots::density::PreparedDensity::new(
                source,
                data,
                panel
                    .parent_orders
                    .collectives
                    .as_deref()
                    .map(Vec::as_slice),
            )?,
        )),
        PlotKind::Qq => Ok(PreparedFamily::Qq(crate::plots::qq::PreparedQq::new(
            source, data,
        )?)),
        PlotKind::Regression => {
            let (marks, ranges) = crate::plots::regression::PreparedRegression::new(source, data)?;
            panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), [None, None])?);
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
        source: &PlotDescription,
        axes: Option<&PreparedAxesOwner>,
    ) -> Result<(f64, f64), Error> {
        Ok(match self {
            Self::Waterfall(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source
                    .x_axis_spec()
                    .window
                    .map(crate::layout::RawWindow::pair),
            )?,
            Self::Beeswarm(marks) => crate::layout::continuous_auto_range(
                marks.x_data,
                source
                    .x_axis_spec()
                    .window
                    .map(crate::layout::RawWindow::pair),
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
            Self::Interaction(_) => (
                0.0,
                axes.expect("prepared Interaction axes")
                    .categories(0)
                    .expect("prepared Interaction categories")
                    .raw
                    .len() as f64,
            ),
            Self::Distribution { .. } => (
                0.0,
                axes.expect("prepared distribution axes")
                    .categories(0)
                    .expect("prepared distribution categories")
                    .raw
                    .len() as f64,
            ),
            Self::Bars { kind, marks, .. } => {
                if matches!(
                    kind,
                    PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                ) {
                    crate::layout::anchored_auto_range(
                        marks.value_range,
                        source
                            .x_axis_spec()
                            .window
                            .map(crate::layout::RawWindow::pair),
                        0.0,
                    )?
                } else {
                    (
                        0.0,
                        axes.expect("prepared Bar axes")
                            .categories(0)
                            .expect("prepared Bar categories")
                            .raw
                            .len() as f64,
                    )
                }
            }
            Self::Estimate { kind, .. } => crate::plots::interval_estimate::x_range(
                *kind,
                axes.expect("prepared Estimate axes").sources()[0]
                    .base_seed
                    .map(DisplayExtent::pair),
            )?,
            Self::Dumbbell(marks) => crate::layout::continuous_auto_range(
                marks.range,
                source
                    .x_axis_spec()
                    .window
                    .map(crate::layout::RawWindow::pair),
            )?,
            Self::Histogram(_) => axes.expect("prepared Histogram axes").ranges()[0],
            Self::Density(marks) => marks.ranges(source)?[0],
            Self::Qq(marks) => marks.ranges(source)?[0],
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
    // Scatter/Line install the parent's window before their original range helper.
    shared_x_window: Option<(f64, f64)>,
    labels: Option<&'a crate::Labels>,
}

pub(crate) fn prepare_reference_plot(
    source: &PlotDescription,
    panel: &mut PlotRegistrationBuild<'_>,
    x_limits: Option<(f64, f64)>,
    override_labels: bool,
    resources: &crate::font::canonical::OperationContext,
) -> Result<(), Error> {
    let mut family = match panel.family.take().expect("prepared reference family") {
        PreparedFamily::Beeswarm(mut marks) => {
            if override_labels {
                marks.color_label = source.prepare_color_label(Some(&panel.policy.labels))?;
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
            if override_labels {
                marks.color_label = source.prepare_color_label(Some(&panel.policy.labels))?;
            }
            PreparedFamily::Heatmap(marks)
        }
        family => family,
    };
    let route = InteriorRoute::classify(family.kind());
    route.validate(source)?;
    let config = ReferenceAxisConfig {
        x_limits,
        labels: Some(&panel.policy.labels),
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
                format: source.total_format_spec(),
            },
            false,
        )?;
    }
    if let PreparedFamily::Qq(quantiles) = &family {
        quantiles.validate_ranges(axes.ranges())?;
    }
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges())?;
    panel.axes = Some(axes);
    panel.family = Some(family);
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "root preparation retains per-component scientific axes and admitted point/path semantics before fonts"
)]
pub(crate) fn prepare_plot<'src>(
    source: &'src PlotDescription,
    resources: &crate::font::canonical::OperationContext,
    appearance: &crate::theme::ResolvedTheme,
) -> Result<PlotRegistryBuilder<'src>, Error> {
    let mut registry = PlotRegistryBuilder::new(resources.root_identity());
    let source_rows = source.configured_source_rows()?;
    let source_id = (!source.has_facets())
        .then(|| {
            let scope = Scope::new(
                source,
                resources,
                0,
                crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)?,
                RowSelection::Full,
                None,
                None,
            )?;
            let policy =
                crate::orchestrator::registration::PlotScopePolicy::for_source(source, None);
            registry.reserve(
                source,
                scope.address,
                scope.numeric_owner,
                scope.axis_numeric_owner,
                scope.rows,
                scope.render_name,
                policy,
                appearance.clone(),
                resources,
            )
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
    ) && !source.has_facets()
    {
        prepare_plot_registration(&mut registry.panels[0], resources)?;
        prepare_reference_plot(source, &mut registry.panels[0], None, false, resources)?;
        return Ok(registry);
    }
    let source_data = source_id
        .as_ref()
        .map(|id| registry.data(id, resources))
        .transpose()?;
    if source.kind() == PlotKind::Scatter && !source.has_facets() {
        let (marks, ranges) = crate::plots::scatter::PreparedScatter::new(
            source,
            source_data.expect("single plot data"),
        )?;
        registry.panels[0].axes = Some(prepare_pair_axes(
            &registry.panels[0],
            ranges.map(Some),
            [None, None],
        )?);
        registry.panels[0].family = Some(PreparedFamily::Scatter(marks));
        prepare_scatter_plot(source, &mut registry.panels[0], None)?;
        return Ok(registry);
    }
    if matches!(
        source.kind(),
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
    ) && !source.has_facets()
    {
        let (marks, ranges, conditions, hues) = crate::plots::line::PreparedLine::new(
            source,
            source_data.expect("single plot data"),
            None,
            registry.panels[0]
                .parent_orders
                .collectives
                .as_deref()
                .map(Vec::as_slice),
        )?;
        registry.panels[0]
            .categorical
            .as_mut()
            .expect("prepared color binding")
            .set_local_keys(hues)?;
        registry.panels[0].axes = Some(prepare_pair_axes(
            &registry.panels[0],
            ranges.map(Some),
            [conditions, None],
        )?);
        registry.panels[0].family = Some(PreparedFamily::Line {
            kind: source.kind(),
            marks,
        });
        prepare_line_plot(source, &mut registry.panels[0], None)?;
        return Ok(registry);
    }
    if source.has_facets() {
        crate::orchestrator::prepare_facet_scopes(source, &mut registry, appearance, resources)?;
        let interaction_parent = (source.kind() == PlotKind::Interaction)
            .then(|| crate::data::observed_source_collectives(source, "interaction"))
            .transpose()?;
        if matches!(
            source.kind(),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest
        ) {
            for panel in &mut registry.panels {
                let (marks, range, labels) =
                    crate::plots::interval_estimate::PreparedEstimate::new(
                        source,
                        panel.data(resources)?,
                        panel.significance_annotations(),
                        panel
                            .scope
                            .numeric_binding(source, &panel.row_source, resources)?,
                    )?;
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "category lengths are admitted source rows"
                )]
                let slots = (0.0, labels.len() as f64);
                panel.axes = Some(prepare_pair_axes(
                    panel,
                    [range, Some(slots)],
                    [None, Some(labels)],
                )?);
                panel.family = Some(PreparedFamily::Estimate {
                    kind: source.kind(),
                    marks,
                });
            }
        }
        let mut estimate_fallback: Option<(f64, f64)> = None;
        if matches!(
            source.facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeY
        ) {
            for panel in &registry.panels {
                let Some(PreparedFamily::Estimate { .. }) = &panel.family else {
                    continue;
                };
                if let Some(range) = panel
                    .axes
                    .as_ref()
                    .expect("prepared Estimate axes")
                    .sources()[0]
                    .base_seed
                    .map(DisplayExtent::pair)
                {
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
                crate::data::source::source_levels(&facet_source_strings(
                    source,
                    column,
                    source_rows,
                    "facet graphical group",
                    None,
                )?)
            })
            .transpose()?;
        let conditions = if source.kind() == PlotKind::Trajectory
            && source
                .x_col()
                .map(|column| {
                    source
                        .data_column(column)
                        .ok_or_else(|| Error::Data(format!("unknown semantic value {column:?}")))
                        .map(|column| !column.is_numeric())
                })
                .transpose()?
                .unwrap_or(false)
        {
            Some(crate::data::source::source_levels(&facet_source_strings(
                source,
                source.x_col().expect("categorical x"),
                source_rows,
                "trajectory",
                Some("condition"),
            )?)?)
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
                crate::data::source::source_levels(&facet_source_strings(
                    source,
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
                        crate::plots::scatter::PreparedScatter::new(source, data)?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), [None, None])?);
                    PreparedFamily::Scatter(marks)
                }
                PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                    let (marks, ranges, conditions, hues) = crate::plots::line::PreparedLine::new(
                        source,
                        data,
                        facet.conditions,
                        panel
                            .parent_orders
                            .collectives
                            .as_deref()
                            .map(Vec::as_slice),
                    )?;
                    panel
                        .categorical
                        .as_mut()
                        .expect("prepared color binding")
                        .set_local_keys(hues)?;
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        ranges.map(Some),
                        [conditions, None],
                    )?);
                    PreparedFamily::Line {
                        kind: source.kind(),
                        marks,
                    }
                }
                PlotKind::Histogram => {
                    let (marks, ranges) = crate::plots::histogram::HistogramSemantics::new(
                        source,
                        data,
                        Some(&panel.appearance),
                    )?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), [None, None])?);
                    PreparedFamily::Histogram(marks)
                }
                PlotKind::Density => {
                    PreparedFamily::Density(crate::plots::density::PreparedDensity::new(
                        source,
                        data,
                        panel
                            .parent_orders
                            .collectives
                            .as_deref()
                            .map(Vec::as_slice),
                    )?)
                }
                PlotKind::Qq => {
                    PreparedFamily::Qq(crate::plots::qq::PreparedQq::new(source, data)?)
                }
                PlotKind::Regression => {
                    let (marks, ranges) =
                        crate::plots::regression::PreparedRegression::new(source, data)?;
                    panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), [None, None])?);
                    PreparedFamily::Regression(marks)
                }
                PlotKind::Dumbbell => {
                    let (marks, categories) = crate::plots::dumbbell::prepare(source, data)?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, categories.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [marks.range, slots].map(Some),
                        [None, Some(categories)],
                    )?);
                    PreparedFamily::Dumbbell(marks)
                }
                PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                    prepared.expect("prepared facet estimates")
                }
                PlotKind::Heatmap => {
                    let (marks, levels) = crate::plots::heatmap::prepare(
                        source,
                        data,
                        continuous_selection(source, &panel.appearance)?,
                        panel
                            .scope
                            .numeric_binding(source, &panel.row_source, resources)?,
                        panel.policy.continuous_materialization,
                    )?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let ranges = levels.each_ref().map(|levels| (0.0, levels.len() as f64));
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        ranges.map(Some),
                        levels.map(Some),
                    )?);
                    PreparedFamily::Heatmap(marks)
                }
                PlotKind::Beeswarm => {
                    let (marks, levels) = crate::plots::beeswarm::prepare(
                        source,
                        data,
                        continuous_selection(source, &panel.appearance)?,
                        panel
                            .scope
                            .numeric_binding(source, &panel.row_source, resources)?,
                        panel.policy.continuous_materialization,
                    )?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, levels.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [marks.x_data, slots].map(Some),
                        [None, Some(levels)],
                    )?);
                    PreparedFamily::Beeswarm(marks)
                }
                PlotKind::Waterfall => {
                    let (marks, labels) = crate::plots::waterfall::prepare(source, data)?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, labels.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [marks.range, slots].map(Some),
                        [None, Some(labels)],
                    )?);
                    PreparedFamily::Waterfall(marks)
                }
                PlotKind::Interaction => {
                    let (marks, conditions) = crate::plots::interaction::prepare(
                        source,
                        data,
                        interaction_parent.as_ref(),
                        panel
                            .parent_orders
                            .collectives
                            .as_deref()
                            .map(Vec::as_slice),
                    )?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, conditions.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [slots, marks.range].map(Some),
                        [Some(conditions), None],
                    )?);
                    PreparedFamily::Interaction(marks)
                }
                PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                    let (marks, categories, domains) =
                        crate::plots::categorical_distribution::PreparedDistribution::new(
                            source,
                            data,
                            Some(facet),
                        )?;
                    panel
                        .categorical
                        .as_mut()
                        .expect("prepared color binding")
                        .set_local_hues(domains)?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, categories.len() as f64);
                    panel.axes = Some(prepare_pair_axes(
                        panel,
                        [slots, marks.range()].map(Some),
                        [Some(categories), None],
                    )?);
                    PreparedFamily::Distribution {
                        kind: source.kind(),
                        marks,
                    }
                }
                PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar => {
                    let (marks, categories, domains) = crate::plots::bar_data::prepare_scoped(
                        source,
                        data,
                        crate::plots::bar_data::mode(source),
                        Some(facet),
                        panel.significance_annotations(),
                    )?;
                    panel
                        .categorical
                        .as_mut()
                        .expect("prepared color binding")
                        .set_local_hues(domains)?;
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "category lengths are admitted source rows"
                    )]
                    let slots = (0.0, categories.len() as f64);
                    let horizontal = matches!(
                        source.kind(),
                        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                    );
                    let (ranges, categories) = if horizontal {
                        ([marks.value_range, slots], [None, Some(categories)])
                    } else {
                        ([slots, marks.value_range], [Some(categories), None])
                    };
                    panel.axes = Some(prepare_pair_axes(panel, ranges.map(Some), categories)?);
                    PreparedFamily::Bars {
                        kind: source.kind(),
                        mode: crate::plots::bar_data::mode(source),
                        marks,
                    }
                }
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
                        format: source.total_format_spec(),
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
                    if let Some(ticks) = axes.axis(panel.source, side).exact_ticks()? {
                        crate::layout::Axis::validate_exact_ticks(
                            &ticks,
                            axes.ranges()[side],
                            if side == 0 { "x-axis" } else { "y-axis" },
                        )?;
                    }
                }
            }
            crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges())?;
        }
    } else if source.kind() == PlotKind::Matrix {
        let (marks, axes, hues) = crate::plots::matrix::prepare_axes(
            source,
            source_data.expect("single plot data"),
            &registry.panels[0].scope.id,
            registry.panels[0]
                .parent_orders
                .collectives
                .as_deref()
                .map(Vec::as_slice),
        )?;
        registry.panels[0].family = Some(PreparedFamily::Matrix(marks));
        registry.panels[0].axes = Some(axes);
        registry.panels[0]
            .categorical
            .as_mut()
            .expect("prepared color binding")
            .set_local_hues(hues)?;
    } else {
        let axes = prepare(source, &registry.panels[0].scope.id, None, None, None, None)?;
        registry.panels[0].axes = Some(axes);
    }
    Ok(registry)
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
    source: &PlotDescription,
    panel: &mut PlotRegistrationBuild<'_>,
    shared_x_window: Option<(f64, f64)>,
) -> Result<(), Error> {
    InteriorRoute::classify(PlotKind::Scatter).validate(source)?;
    let axes = prepare(
        source,
        &panel.scope.id,
        None,
        panel.family.as_ref(),
        Some(ReferenceAxisConfig {
            shared_x_window,
            labels: Some(&panel.policy.labels),
            ..ReferenceAxisConfig::default()
        }),
        panel.axes.take(),
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges())?;
    panel.axes = Some(axes);
    Ok(())
}

/// Complete one retained line component after the parent's range coordination.
pub(crate) fn prepare_line_plot(
    source: &PlotDescription,
    panel: &mut PlotRegistrationBuild<'_>,
    shared_x_window: Option<(f64, f64)>,
) -> Result<(), Error> {
    let axes = prepare(
        source,
        &panel.scope.id,
        None,
        panel.family.as_ref(),
        Some(ReferenceAxisConfig {
            shared_x_window,
            labels: Some(&panel.policy.labels),
            ..ReferenceAxisConfig::default()
        }),
        panel.axes.take(),
    )?;
    crate::orchestrator::layers::GuideSnapshot::validate_sources(source, axes.ranges())?;
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
        ) -> Result<(bool, Option<&'a crate::numeric::format::NumericFormat>), Error> {
            Ok((
                axes.logarithmic() && side == 0,
                if axes.axis(source, side).exact_ticks()?.is_some() {
                    None
                } else {
                    axes.axis(source, side)
                        .authored()
                        .expect("coordinate authored intent")
                        .format
                        .as_deref()
                },
            ))
        }
        for panel in &*retained {
            let axes = panel.axes.as_ref().expect("prepared facet axes");
            if presentation(axes, panel.source, side)? != presentation(first, first_source, side)? {
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
    chart: &PlotDescription,
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
    let shared_x_window = reference_config
        .as_ref()
        .and_then(|config| config.shared_x_window);
    let x_window = shared_x_window.or_else(|| {
        chart
            .x_axis_spec()
            .window
            .map(crate::layout::RawWindow::pair)
    });
    let mut categories: [Option<Vec<String>>; 2] = [None, None];
    let mut ranges = match chart.kind() {
        PlotKind::Regression => {
            marks!(family, PreparedFamily::Regression(marks) => marks, "Regression axes require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Regression axes").ranges();
            [
                chart
                    .x_axis_spec()
                    .window.map_or(ranges[0], crate::layout::RawWindow::pair),
                chart
                    .y_axis_spec()
                    .window.map_or(ranges[1], crate::layout::RawWindow::pair),
            ]
        }
        PlotKind::Scatter => {
            marks!(family, PreparedFamily::Scatter(marks) => marks, "Scatter axis inputs require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Scatter axes").ranges();
            [
                crate::layout::continuous_auto_range(ranges[0], x_window)?,
                crate::layout::continuous_auto_range(ranges[1], chart.y_axis_spec().window.map(crate::layout::RawWindow::pair))?,
            ]
        }
        PlotKind::Histogram => {
            marks!(family, PreparedFamily::Histogram(marks) => marks, "Histogram axes require prepared semantics");
            let ranges = prepared_axes.as_ref().expect("prepared Histogram axes").ranges();
            [
                chart
                    .x_axis_spec()
                    .window.map_or(ranges[0], crate::layout::RawWindow::pair),
                chart
                    .y_axis_spec()
                    .window.map_or(ranges[1], crate::layout::RawWindow::pair),
            ]
        }
        PlotKind::Density => marks!(family, PreparedFamily::Density(marks) => marks, "Density axes require prepared semantics").ranges(chart)?,
        PlotKind::Qq => marks!(family, PreparedFamily::Qq(marks) => marks, "Qq axes require prepared semantics").ranges(chart)?,
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            marks!(family, PreparedFamily::Line { marks, .. } => marks, "line axis inputs require prepared semantics");
            let axes = prepared_axes.as_mut().expect("prepared line axes");
            categories[0] = axes.sources_mut()[0].categories.take().map(|categories| categories.raw);
            let ranges = axes.ranges();
            [
                x_window.unwrap_or(ranges[0]),
                chart
                    .y_axis_spec()
                    .window.map_or(ranges[1], crate::layout::RawWindow::pair),
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
            let range = marks.value_range;
            let horizontal = matches!(
                chart.kind(),
                PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
            );
            let levels = prepared_axes.as_mut().expect("prepared Bar axes").sources_mut()[usize::from(horizontal)]
                .categories.take().expect("prepared Bar categories").raw;
            let slots = (0.0, levels.len() as f64);
            categories[usize::from(horizontal)] = Some(levels);
            let values = crate::layout::anchored_auto_range(
                range,
                (if horizontal {
                    chart.x_axis_spec()
                } else {
                    chart.y_axis_spec()
                }).window.map(crate::layout::RawWindow::pair),
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
            let levels = prepared_axes.as_mut().expect("prepared distribution axes").sources_mut()[0]
                .categories.take().expect("prepared distribution categories").raw;
            let values = marks.range();
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(values, chart.y_axis_spec().window.map(crate::layout::RawWindow::pair))?,
            ]
        }
        PlotKind::Dumbbell => {
            let marks = marks!(reference, PreparedFamily::Dumbbell(marks) => marks, "Dumbbell axes require prepared semantics");
            let levels = prepared_axes.as_mut().expect("prepared Dumbbell axes").sources_mut()[1]
                .categories.take().expect("prepared Dumbbell categories").raw;
            let slots = (0.0, levels.len() as f64);
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.x_axis_spec().window.map(crate::layout::RawWindow::pair),
                )?,
                slots,
            ]
        }
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let config = reference_config.as_ref().ok_or_else(|| {
                Error::Render("estimate axes require root-prepared semantics".into())
            })?;
            marks!(family, PreparedFamily::Estimate { marks, .. } => marks, "estimate axes require retained rows");
            let axes = prepared_axes.as_mut().expect("prepared Estimate axes");
            let levels = axes.sources_mut()[1].categories.take().expect("prepared Estimate categories").raw;
            let slots = (0.0, levels.len() as f64);
            categories[1] = Some(levels);
            let range = crate::plots::interval_estimate::x_range(chart.kind(),
                axes.sources()[0].base_seed.map(DisplayExtent::pair).or(config.estimate_fallback))?;
            [range, slots]
        }
        PlotKind::Interaction => {
            let marks = marks!(reference, PreparedFamily::Interaction(marks) => marks, "interaction axes require retained semantics");
            let levels = prepared_axes.as_mut().expect("prepared Interaction axes").sources_mut()[0]
                .categories.take().expect("prepared Interaction categories").raw;
            let slots = (0.0, levels.len() as f64);
            categories[0] = Some(levels);
            [
                slots,
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.y_axis_spec().window.map(crate::layout::RawWindow::pair),
                )?,
            ]
        }
        PlotKind::Beeswarm => {
            let marks = marks!(reference, PreparedFamily::Beeswarm(marks) => marks, "beeswarm axes require retained semantics");
            let levels = prepared_axes.as_mut().expect("prepared Beeswarm axes").sources_mut()[1]
                .categories.take().expect("prepared Beeswarm categories").raw;
            let slots = chart
                .y_axis_spec()
                .window.map_or((0.0, levels.len() as f64), crate::layout::RawWindow::pair);
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.x_data,
                    chart.x_axis_spec().window.map(crate::layout::RawWindow::pair),
                )?,
                slots,
            ]
        }
        PlotKind::Waterfall => {
            let marks = marks!(reference, PreparedFamily::Waterfall(marks) => marks, "waterfall axes require retained semantics");
            let levels = prepared_axes.as_mut().expect("prepared Waterfall axes").sources_mut()[1]
                .categories.take().expect("prepared Waterfall categories").raw;
            let slots = chart
                .y_axis_spec()
                .window.map_or((0.0, levels.len() as f64), crate::layout::RawWindow::pair);
            categories[1] = Some(levels);
            [
                crate::layout::continuous_auto_range(
                    marks.range,
                    chart.x_axis_spec().window.map(crate::layout::RawWindow::pair),
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
    let ReferenceAxisConfig {
        x_limits, labels, ..
    } = reference_config.unwrap_or_default();
    if let Some(limits) = x_limits {
        ranges[0] = limits;
    }
    let logarithmic =
        chart.kind() == PlotKind::Forest && chart.axis_scale_spec() == AxisScale::Log10;
    let specs = [chart.x_axis_spec(), chart.y_axis_spec()];
    let mut provenance = specs.map(|spec| spec.provenance);
    if (x_limits.is_some() || shared_x_window.is_some())
        && provenance[0] != crate::layout::RangeProvenance::Configured
    {
        provenance[0] = crate::layout::RangeProvenance::Shared;
    }
    let mut projected_exact_positions = [
        if logarithmic {
            chart.projected_explicit_x_ticks()?
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
                chart.resolve_axis_label_domain(domain, raw_labels)?
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
        chart.prepare_axis_labels(labels)?
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
