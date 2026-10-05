//! Bottom-up standard-panel soft sizes; exact root assignment remains separate.

use crate::font::canonical::OperationGenerationResources;
use crate::layout::PhysicalTextAlignment;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StandardPanelPreference {
    pub(crate) primary_marks: u128,
    pub(crate) mark_pitch: f64,
    pub(crate) density_side: f64,
    pub(crate) body_quanta: (i64, i64),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NativePanelPreference {
    Standard(StandardPanelPreference),
    Categorical(CategoricalPanelPreference),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CategoricalPanelPreference {
    pub(crate) axis: crate::layout::AxisSide,
    pub(crate) slots: u32,
    pub(crate) primary_marks: u128,
    pub(crate) hard_pitch_quanta: i64,
    pub(crate) preferred_pitch_quanta: i64,
    pub(crate) body_quanta: (i64, i64),
}

fn ceil_ratio(numerator: i128, denominator: i128) -> Result<i64, Error> {
    if numerator < 0 || denominator <= 0 {
        return Err(Error::Render(
            "categorical preference ratio is not positive".into(),
        ));
    }
    i64::try_from((numerator + denominator - 1) / denominator)
        .map_err(|_| Error::Render("categorical preference exceeds spatial range".into()))
}

#[expect(
    clippy::too_many_lines,
    reason = "one closed categorical owner lookup and checked pitch/aspect composition"
)]
pub(crate) fn categorical_preference(
    plot: &Plot,
    theme: &ResolvedTheme,
    hard: (f64, f64),
) -> Result<CategoricalPanelPreference, Error> {
    let resources = theme.prepared_font_environment()?.generation_resources();
    let scope = plot.axis_scope();
    let (axis, slots, primary_marks) =
        match plot.kind() {
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                let inputs = plot
                    .prepared_estimate
                    .as_deref()
                    .ok_or_else(|| Error::Render("estimate preference lacks preparation".into()))?;
                let marks = panel(&inputs.panels, scope, "Estimate", &resources)?;
                (
                    crate::layout::AxisSide::Left,
                    marks.labels.len(),
                    marks.primary_mark_count(&resources)?,
                )
            }
            PlotKind::Dumbbell => {
                let inputs = plot
                    .prepared_dumbbell
                    .as_deref()
                    .ok_or_else(|| Error::Render("Dumbbell preference lacks preparation".into()))?;
                let marks = panel(&inputs.panels, scope, "Dumbbell", &resources)?;
                (
                    crate::layout::AxisSide::Left,
                    marks.categories.len(),
                    marks.primary_mark_count(),
                )
            }
            PlotKind::Bar
            | PlotKind::StackedBar
            | PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar => {
                let inputs = plot
                    .prepared_bars
                    .as_deref()
                    .ok_or_else(|| Error::Render("Bar preference lacks preparation".into()))?;
                let marks = panel(&inputs.panels, scope, "Bars", &resources)?;
                let axis = if matches!(plot.kind(), PlotKind::Bar | PlotKind::StackedBar) {
                    crate::layout::AxisSide::Bottom
                } else {
                    crate::layout::AxisSide::Left
                };
                (axis, marks.categories.len(), marks.primary_mark_count())
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                let inputs = plot.prepared_distributions.as_deref().ok_or_else(|| {
                    Error::Render("distribution preference lacks preparation".into())
                })?;
                let marks = panel(&inputs.panels, scope, "Distribution", &resources)?;
                (
                    crate::layout::AxisSide::Bottom,
                    marks.categories().len(),
                    marks.primary_mark_count(),
                )
            }
            PlotKind::Interaction => {
                let inputs = plot.prepared_interaction.as_deref().ok_or_else(|| {
                    Error::Render("Interaction preference lacks preparation".into())
                })?;
                let marks = panel(&inputs.panels, scope, "Interaction", &resources)?;
                (
                    crate::layout::AxisSide::Bottom,
                    marks.conditions.len(),
                    marks.primary_mark_count(&resources)?,
                )
            }
            _ => {
                return Err(Error::Render(
                    "categorical preference requires a prepared categorical family".into(),
                ))
            }
        };
    let slots = u32::try_from(slots).map_err(|_| {
        Error::Render("categorical preference slot count exceeds spatial range".into())
    })?;
    if slots == 0 {
        return Err(Error::Render(
            "categorical preference requires occupied category slots".into(),
        ));
    }
    let (hard_x, hard_y) = (
        crate::layout::kernel::ceil_spatial_quanta(hard.0)?,
        crate::layout::kernel::ceil_spatial_quanta(hard.1)?,
    );
    let category_hard = if axis == crate::layout::AxisSide::Bottom {
        hard_x
    } else {
        hard_y
    };
    let hard_pitch = ceil_ratio(i128::from(category_hard), i128::from(slots))?.max(1);
    let em = crate::layout::kernel::ceil_spatial_quanta(
        2.0 * f64::from(theme.typography.tick_label.font_size),
    )?;
    let preferred_pitch = hard_pitch.max(em);
    let category_span = i128::from(preferred_pitch) * i128::from(slots);
    let (width, height) = if axis == crate::layout::AxisSide::Bottom {
        let width = i128::from(hard_x).max(category_span);
        (
            width,
            i128::from(hard_y).max(i128::from(ceil_ratio(width * 3, 4)?)),
        )
    } else {
        let height = i128::from(hard_y).max(category_span);
        (
            i128::from(hard_x).max(i128::from(ceil_ratio(height * 4, 3)?)),
            height,
        )
    };
    Ok(CategoricalPanelPreference {
        axis,
        slots,
        primary_marks,
        hard_pitch_quanta: hard_pitch,
        preferred_pitch_quanta: preferred_pitch,
        body_quanta: (
            i64::try_from(width)
                .map_err(|_| Error::Render("categorical preferred width overflowed".into()))?,
            i64::try_from(height)
                .map_err(|_| Error::Render("categorical preferred height overflowed".into()))?,
        ),
    })
}

pub(crate) fn panel<'a, T>(
    panels: &'a [(Option<String>, T)],
    scope: Option<&str>,
    owner: &str,
    resources: &OperationGenerationResources,
) -> Result<&'a T, Error> {
    for (key, marks) in panels {
        resources.claim_visits(1).map_err(|error| {
            Error::Render(format!("{owner} primary-mark scope admission: {error:?}"))
        })?;
        if key.as_deref() == scope {
            return Ok(marks);
        }
    }
    Err(Error::Render(format!(
        "{owner} primary-mark scope is not prepared"
    )))
}

fn ceil_sqrt(value: u128) -> Result<u32, Error> {
    let value = value.max(1);
    let limit = u32::try_from(value).map_err(|_| {
        Error::Render("standard mark count exceeds supported resource limit".into())
    })?;
    let (mut low, mut high) = (1_u32, limit);
    while low < high {
        let middle = low + (high - low) / 2;
        if u128::from(middle) * u128::from(middle) < value {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low)
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive closed standard-family summary and its checked soft composition"
)]
pub(crate) fn mark_preference(
    plot: &Plot,
    theme: &ResolvedTheme,
    hard: (f64, f64),
) -> Result<StandardPanelPreference, Error> {
    let resources = theme.prepared_font_environment()?.generation_resources();
    let scope = plot.axis_scope();
    let point_ink = |radius| -> Result<f64, Error> {
        let point = plot.point_style(theme, (0, 0, 0), radius);
        let (x, y) = crate::render::geometry::point_ink_dimensions(theme, &point)?;
        Ok(x.max(y))
    };
    let (count, primary_ink) = match plot.kind() {
        PlotKind::Scatter => {
            let inputs = plot
                .prepared_scatter
                .as_deref()
                .ok_or_else(|| Error::Render("Scatter mark preference lacks preparation".into()))?;
            let marks = panel(&inputs.panels, scope, "Scatter", &resources)?;
            (
                marks.primary_mark_count(&resources)?,
                point_ink(theme.marks.point_sizes.scatter)?,
            )
        }
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let inputs = plot
                .prepared_line
                .as_deref()
                .ok_or_else(|| Error::Render("Line mark preference lacks preparation".into()))?;
            let marks = panel(&inputs.panels, scope, "Line", &resources)?;
            let stroke = if plot.kind() == PlotKind::Line {
                &theme.marks.series_line
            } else {
                &theme.marks.path_line
            };
            let line =
                crate::render::geometry::line_ink_width(theme, &stroke.backend_style((0, 0, 0)))?;
            let marker = if plot.kind() == PlotKind::Trajectory || plot.markers_visible() {
                point_ink(if plot.kind() == PlotKind::Line {
                    theme.marks.point_sizes.path
                } else {
                    theme.marks.point_sizes.trajectory
                })?
            } else {
                0.0
            };
            (marks.primary_mark_count(&resources)?, line.max(marker))
        }
        PlotKind::Histogram => {
            let inputs = plot.prepared_histogram.as_deref().ok_or_else(|| {
                Error::Render("Histogram mark preference lacks preparation".into())
            })?;
            let marks = panel(&inputs.panels, scope, "Histogram", &resources)?;
            let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(
                theme,
                &theme.marks.histogram_bin.backend_style((0, 0, 0)),
            )?;
            (marks.primary_mark_count(), x.max(y))
        }
        PlotKind::Density => {
            let inputs = plot
                .prepared_density
                .as_deref()
                .ok_or_else(|| Error::Render("Density mark preference lacks preparation".into()))?;
            let marks = panel(&inputs.panels, scope, "Density", &resources)?;
            let line = crate::render::geometry::line_ink_width(
                theme,
                &theme.marks.density_line.backend_style((0, 0, 0)),
            )?;
            let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(
                theme,
                &theme.marks.density_area.backend_style((0, 0, 0)),
            )?;
            (marks.primary_mark_count(), line.max(x).max(y))
        }
        PlotKind::Qq => {
            let inputs = plot
                .prepared_qq
                .as_deref()
                .ok_or_else(|| Error::Render("Qq mark preference lacks preparation".into()))?;
            let marks = panel(&inputs.panels, scope, "Qq", &resources)?;
            (
                marks.primary_mark_count(),
                point_ink(theme.marks.point_sizes.qq)?,
            )
        }
        PlotKind::Regression => {
            let inputs = plot.prepared_regression.as_deref().ok_or_else(|| {
                Error::Render("Regression mark preference lacks preparation".into())
            })?;
            let marks = panel(&inputs.panels, scope, "Regression", &resources)?;
            (
                marks.primary_mark_count(),
                point_ink(theme.marks.point_sizes.regression)?,
            )
        }
        PlotKind::Marginal => {
            let marks = plot
                .prepared_marginal
                .as_ref()
                .ok_or_else(|| Error::Render("Marginal preference lacks preparation".into()))?;
            (
                marks.primary_mark_count(&resources)?,
                point_ink(theme.marks.point_sizes.marginal)?,
            )
        }
        _ => {
            return Err(Error::Render(
                "standard mark preference requires a standard coordinate family".into(),
            ))
        }
    };
    let em = f64::from(theme.typography.tick_label.font_size);
    let mark_pitch = em.max(primary_ink);
    let density_side = f64::from(ceil_sqrt(count)?) * mark_pitch;
    let ch = theme
        .measure_text(
            "0",
            &theme.typography.tick_label,
            PhysicalTextAlignment::Start,
            "standard panel 66ch preference",
            false,
        )?
        .width();
    let width = hard.0.max(66.0 * ch).max(density_side);
    let width_quanta = crate::layout::kernel::ceil_spatial_quanta(width)?;
    let height_quanta = ((i128::from(width_quanta) * 3 + 3) / 4)
        .max(i128::from(crate::layout::kernel::ceil_spatial_quanta(
            hard.1,
        )?))
        .max(i128::from(crate::layout::kernel::ceil_spatial_quanta(
            density_side,
        )?));
    let height_quanta = i64::try_from(height_quanta)
        .map_err(|_| Error::Render("standard preferred height exceeds spatial range".into()))?;
    Ok(StandardPanelPreference {
        primary_marks: count,
        mark_pitch,
        density_side,
        body_quanta: (width_quanta, height_quanta),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CategoryValues, NumericValues};

    #[test]
    fn retained_panel_lookup_prices_only_entries_actually_examined() {
        let panels = [
            (None, 10),
            (Some("second".into()), 20),
            (Some("unused".into()), 30),
        ];
        for (scope, visits, expected) in [(None, 1, 10), (Some("second"), 2, 20)] {
            let resources = OperationGenerationResources::with_test_limits(visits, 0);
            assert_eq!(
                *panel(&panels, scope, "test", &resources).unwrap(),
                expected
            );
            assert_eq!(resources.usage().unwrap(), (0, visits));
            let limited = OperationGenerationResources::with_test_limits(visits - 1, 0);
            assert!(panel(&panels, scope, "test", &limited).is_err());
        }
        for visits in [0, 2, 3, 4] {
            let resources = OperationGenerationResources::with_test_limits(visits, 0);
            let error = panel(&panels, Some("missing"), "test", &resources)
                .unwrap_err()
                .to_string();
            assert_eq!(resources.usage().unwrap(), (0, visits.min(3)));
            if visits < 3 {
                assert!(error.contains("root processing visits"));
            } else {
                assert!(error.contains("scope is not prepared"));
            }
        }
    }

    fn standard(root: &crate::layout::RootFramePlan) -> &StandardPanelPreference {
        let Some(NativePanelPreference::Standard(preference)) = root.native_preference.as_ref()
        else {
            panic!("standard native root preference")
        };
        preference
    }

    fn categorical(root: &crate::layout::RootFramePlan) -> &CategoricalPanelPreference {
        let Some(NativePanelPreference::Categorical(preference)) = root.native_preference.as_ref()
        else {
            panic!("categorical native root preference")
        };
        preference
    }

    #[test]
    fn categorical_preferences_keep_unavailable_slots_and_occupied_grouped_bars() {
        let estimate = Plot::point_interval(
            CategoryValues::new("term", ["available", "unavailable"]).unwrap(),
            NumericValues::nullable("estimate", [Some(1.0), None]),
            crate::Bounds::new(
                NumericValues::nullable("lower", [Some(0.5), None]),
                NumericValues::nullable("upper", [Some(1.5), None]),
            ),
        )
        .estimate_status([
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::NotEstimable,
        ])
        .x_limits(0.0, 2.0)
        .size(1200, 900)
        .erase();
        let grouped = |horizontal: bool| {
            let category = CategoryValues::new("category", ["a", "b"]).unwrap();
            let input = crate::BarInput::estimates(NumericValues::new("value", [1.0, 2.0]));
            let series = CategoryValues::new("series", ["one", "two"]).unwrap();
            if horizontal {
                Plot::horizontal_bar(category, input)
                    .series(series)
                    .size(1200, 900)
                    .erase()
            } else {
                Plot::bar(category, input)
                    .series(series)
                    .size(1200, 900)
                    .erase()
            }
        };
        let dumbbell = Plot::dumbbell(
            CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap(),
            NumericValues::new("left", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("right", [4.0, 3.0, 2.0, 1.0]),
            crate::DumbbellEndpoints::new("Before", (1, 2, 3), "After", (4, 5, 6)).unwrap(),
        )
        .group(CategoryValues::new("series", ["one", "one", "two", "two"]).unwrap())
        .size(1200, 900)
        .erase();
        for (source, axis, count) in [
            (estimate, crate::layout::AxisSide::Left, 1),
            (dumbbell, crate::layout::AxisSide::Left, 4),
            (grouped(false), crate::layout::AxisSide::Bottom, 2),
            (grouped(true), crate::layout::AxisSide::Left, 2),
        ] {
            let resources = OperationGenerationResources::new();
            let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
            let chart = prepared.attach(source).unwrap();
            let theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            let root = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
            let preference = categorical(&root);
            assert_eq!(preference.axis, axis);
            assert_eq!(preference.slots, 2);
            assert_eq!(preference.primary_marks, count);
            assert!(preference.preferred_pitch_quanta >= 32 * 1024);
            assert!(preference.preferred_pitch_quanta >= preference.hard_pitch_quanta);
            let (width, height) = preference.body_quanta;
            if axis == crate::layout::AxisSide::Bottom {
                assert!(width >= 2 * preference.preferred_pitch_quanta);
                assert!(height >= (3 * width + 3) / 4);
            } else {
                assert!(height >= 2 * preference.preferred_pitch_quanta);
                assert!(width >= (4 * height + 2) / 3);
            }
        }
    }

    #[test]
    fn private_bar_outline_changes_preference_but_not_exact_root_box() {
        let source = Plot::bar(
            CategoryValues::new("category", ["a", "b"]).unwrap(),
            crate::BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
        )
        .series(CategoryValues::new("series", ["one", "two"]).unwrap())
        .size(1200, 900)
        .erase();
        let resources = OperationGenerationResources::new();
        let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = prepared.attach(source).unwrap();
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let original = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        theme.marks.bar_body.outline_width = 100.0;
        let styled = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        assert_eq!(styled.panel_box, original.panel_box);
        assert!(categorical(&styled).hard_pitch_quanta > categorical(&original).hard_pitch_quanta);
        assert!(categorical(&styled).body_quanta.0 > categorical(&original).body_quanta.0);
    }

    #[test]
    fn sparse_collective_families_share_one_categorical_preference_carrier() {
        let category = CategoryValues::new("category", ["a", "a", "a", "b", "b", "b"]).unwrap();
        let values = NumericValues::new("value", [1.0, 2.0, 3.0, 3.0, 4.0, 5.0]);
        let sampled = [
            Plot::boxplot(category.clone(), values.clone())
                .size(1600, 900)
                .erase(),
            Plot::violin(category.clone(), values.clone())
                .size(1600, 900)
                .erase(),
            Plot::raincloud(crate::RaincloudInput::samples(category, values))
                .size(1600, 900)
                .erase(),
        ];
        let supplied = Plot::raincloud(crate::RaincloudInput::geometry(crate::RaincloudGeometry {
            dimensions: crate::RaincloudDimensions {
                category: "category".into(),
                value: "value".into(),
                hue: None,
            },
            groups: ["a", "b"]
                .into_iter()
                .map(|category| crate::RaincloudGroup {
                    category: category.into(),
                    hue: None,
                    density: vec![
                        crate::RaincloudDensityPoint {
                            y: 0.0,
                            normalized_width: 0.0,
                        },
                        crate::RaincloudDensityPoint {
                            y: 3.0,
                            normalized_width: 1.0,
                        },
                        crate::RaincloudDensityPoint {
                            y: 6.0,
                            normalized_width: 0.0,
                        },
                    ],
                    q1: 1.0,
                    median: 3.0,
                    q3: 5.0,
                    points: vec![crate::RaincloudPoint {
                        y: 3.0,
                        normalized_strip: 0.5,
                    }],
                })
                .collect(),
        }))
        .size(1600, 900)
        .erase();
        let interaction = Plot::interaction(
            CategoryValues::new("condition", ["a", "b", "c", "a"]).unwrap(),
            NumericValues::new("estimate", [1.0, 2.0, 3.0, 4.0]),
            CategoryValues::new("group", ["one", "one", "one", "two"]).unwrap(),
            crate::Bounds::new(
                NumericValues::new("lower", [0.0, 1.0, 2.0, 3.0]),
                NumericValues::new("upper", [2.0, 3.0, 4.0, 5.0]),
            ),
        )
        .size(1600, 900)
        .erase();
        for (source, slots, primary) in sampled
            .into_iter()
            .map(|plot| (plot, 2, 2))
            .chain([(supplied, 2, 2), (interaction, 3, 4)])
        {
            let resources = OperationGenerationResources::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
            let chart = inputs.attach(source).unwrap();
            let theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            let root = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
            let preference = categorical(&root);
            assert_eq!(preference.axis, crate::layout::AxisSide::Bottom);
            assert_eq!(preference.slots, slots);
            assert_eq!(preference.primary_marks, primary);
            assert!(preference.preferred_pitch_quanta >= preference.hard_pitch_quanta);
            assert!(
                preference.body_quanta.0
                    >= i64::from(preference.slots) * preference.preferred_pitch_quanta
            );
        }
    }

    #[test]
    fn integer_ceil_sqrt_has_exact_square_and_neighbor_boundaries() {
        for (value, expected) in [
            (0, 1),
            (1, 1),
            (2, 2),
            (4, 2),
            (5, 3),
            (9, 3),
            (10, 4),
            (999_999, 1_000),
            (1_000_000, 1_000),
            (1_000_001, 1_001),
        ] {
            assert_eq!(ceil_sqrt(value).unwrap(), expected);
        }
    }

    #[test]
    fn coincident_scatter_marks_still_supply_soft_density_and_custom_point_pitch() {
        let source = Plot::scatter(
            NumericValues::new("x", [0.0; 25]),
            NumericValues::new("y", [0.0; 25]),
        )
        .size(800, 600)
        .erase();
        let resources = OperationGenerationResources::new();
        let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = prepared.attach(source).unwrap();
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let original = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        let preference = standard(&original);
        assert_eq!(preference.primary_marks, 25);
        assert_eq!(preference.mark_pitch.to_bits(), 16.0_f64.to_bits());
        assert_eq!(preference.density_side.to_bits(), 80.0_f64.to_bits());
        let width = preference.body_quanta.0;
        assert!(
            width >= crate::layout::kernel::ceil_spatial_quanta(preference.density_side).unwrap()
        );
        assert!(preference.body_quanta.1 >= (3 * width + 3) / 4);

        theme.marks.point_sizes.scatter = 80;
        let styled = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        let preference = standard(&styled);
        assert_eq!(preference.primary_marks, 25);
        assert!(preference.mark_pitch > 160.0);
        assert!(preference.density_side > 800.0);
        assert!(preference.body_quanta.0 > width);
        assert_eq!(styled.panel_box, original.panel_box);

        theme.marks.primary_point.fill_alpha = 0;
        theme.marks.primary_point.outline.color = crate::theme::Color::rgba(255, 255, 255, 0);
        let invisible = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        assert_eq!(standard(&invisible).primary_marks, 25);
        assert_eq!(
            standard(&invisible).mark_pitch.to_bits(),
            16.0_f64.to_bits()
        );

        let source = Plot::scatter(
            NumericValues::new("x", [0.0; 25]),
            NumericValues::new("y", [0.0; 25]),
        )
        .x_limits(10.0, 11.0)
        .size(800, 600)
        .erase();
        let resources = OperationGenerationResources::new();
        let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let clipped = prepared.attach(source).unwrap();
        let exact = super::super::coordinate::canvas_layout(&clipped, &theme).unwrap();
        assert_eq!(standard(&exact).primary_marks, 25);
    }

    #[test]
    fn standard_distribution_counts_exclude_samples_and_generated_vertices() {
        for (source, expected) in super::super::distributions::sources()
            .into_iter()
            .zip([8, 2, 6, 6])
        {
            let resources = OperationGenerationResources::new();
            let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
            let chart = prepared.attach(source).unwrap();
            let root = super::super::coordinate::canvas_layout(
                &chart,
                &crate::Theme::Publication
                    .resolve(&crate::font::canonical::OperationGenerationResources::new())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(standard(&root).primary_marks, expected);
        }
    }

    #[test]
    fn path_count_does_not_multiply_connectors_or_supplemental_ink() {
        let x = NumericValues::new("x", [0.0, 1.0, 1.0, 2.0]);
        let y = NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]);
        let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
        let source = Plot::trajectory(x, y, group).size(800, 600).erase();
        let resources = OperationGenerationResources::new();
        let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = prepared.attach(source).unwrap();
        let root = super::super::coordinate::canvas_layout(
            &chart,
            &crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(standard(&root).primary_marks, 4);
    }
}
