//! Bottom-up standard-panel soft sizes; exact root assignment remains separate.

use crate::layout::PhysicalTextAlignment;
use crate::orchestrator::registration::PreparedFamily as F;
use crate::{Error, Plot, PlotKind};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StandardSizePreference {
    pub(crate) primary_marks: u128,
    pub(crate) mark_pitch: f64,
    pub(crate) density_side: f64,
    pub(crate) body_quanta: (i64, i64),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CoordinateSizePreference {
    Standard(StandardSizePreference),
    Categorical(CategoricalSizePreference),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CategoricalSizePreference {
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
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    hard: (f64, f64),
) -> Result<CategoricalSizePreference, Error> {
    let scope = plot.axis_scope();
    macro_rules! family {
        ($missing:literal, $owner:literal, $pattern:pat => $marks:ident) => {{
            if registered.is_none() {
                return Err(Error::Render($missing.into()));
            }
            let $pattern = panel(registered, scope, $owner)? else {
                return Err(Error::Render($missing.into()));
            };
            $marks
        }};
    }
    let (axis, slots, primary_marks) = match plot.kind() {
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let marks = family!("estimate preference lacks preparation", "Estimate", F::Estimate { marks, .. } => marks);
            (
                crate::layout::AxisSide::Left,
                marks.labels.len(),
                marks.primary_mark_count(),
            )
        }
        PlotKind::Dumbbell => {
            let marks = family!("Dumbbell preference lacks preparation", "Dumbbell", F::Dumbbell(marks) => marks);
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
            let marks =
                family!("Bar preference lacks preparation", "Bars", F::Bars { marks, .. } => marks);
            let axis = if matches!(plot.kind(), PlotKind::Bar | PlotKind::StackedBar) {
                crate::layout::AxisSide::Bottom
            } else {
                crate::layout::AxisSide::Left
            };
            (axis, marks.categories.len(), marks.primary_mark_count())
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            let marks = family!("distribution preference lacks preparation", "Distribution", F::Distribution { marks, .. } => marks);
            (
                crate::layout::AxisSide::Bottom,
                marks.categories().len(),
                marks.primary_mark_count(),
            )
        }
        PlotKind::Interaction => {
            let marks = family!("Interaction preference lacks preparation", "Interaction", F::Interaction(marks) => marks);
            (
                crate::layout::AxisSide::Bottom,
                marks.conditions.len(),
                marks.checked_primary_mark_count()?,
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
        2.0 * theme.typography.axes.tick_label.font_size,
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
    Ok(CategoricalSizePreference {
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

pub(crate) fn panel<'a>(
    registered: Option<&'a crate::orchestrator::registration::PlotRegistration<'_>>,
    scope: Option<&str>,
    owner: &str,
) -> Result<&'a crate::orchestrator::registration::PreparedFamily, Error> {
    if let Some(registered) =
        registered.filter(|registered| registered.scope.render_name.as_deref() == scope)
    {
        return Ok(&registered.family);
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
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    hard: (f64, f64),
) -> Result<StandardSizePreference, Error> {
    let scope = plot.axis_scope();
    macro_rules! standard {
        ($owner:literal, $pattern:pat => $marks:ident) => {{
            if registered.is_none() {
                return Err(Error::Render(format!(
                    "{} mark preference lacks preparation",
                    $owner
                )));
            }
            let $pattern = panel(registered, scope, $owner)? else {
                return Err(Error::Render(format!(
                    "{} mark preference lacks preparation",
                    $owner
                )));
            };
            $marks
        }};
    }
    let point_ink = |radius| -> Result<f64, Error> {
        let point = plot.description().point_style(theme, (0, 0, 0), radius);
        let (x, y) = crate::render::geometry::point_ink_dimensions(&point)?;
        Ok(x.max(y))
    };
    let (count, primary_ink) = match plot.kind() {
        PlotKind::Scatter => {
            let marks = standard!("Scatter", F::Scatter(marks) => marks);
            (
                marks.checked_primary_mark_count()?,
                point_ink(theme.marks.get_points().get_radii().get_scatter())?,
            )
        }
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            let marks = standard!("Line", F::Line { marks, .. } => marks);
            let stroke = if plot.kind() == PlotKind::Line {
                theme.marks.get_data_lines().get_series_line()
            } else {
                theme.marks.get_data_lines().get_path_line()
            };
            let line = crate::render::geometry::line_ink_width(&stroke.backend_style((0, 0, 0)))?;
            let marker =
                if plot.kind() == PlotKind::Trajectory || plot.description().markers_visible() {
                    point_ink(if plot.kind() == PlotKind::Line {
                        theme.marks.get_points().get_radii().get_path()
                    } else {
                        theme.marks.get_points().get_radii().get_trajectory()
                    })?
                } else {
                    0.0
                };
            (marks.checked_primary_mark_count()?, line.max(marker))
        }
        PlotKind::Histogram => {
            let marks = standard!("Histogram", F::Histogram(marks) => marks);
            let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(
                &theme
                    .marks
                    .get_fills()
                    .get_bodies()
                    .get_histogram_bin()
                    .backend_style((0, 0, 0)),
            )?;
            (marks.primary_mark_count(), x.max(y))
        }
        PlotKind::Density => {
            let marks = standard!("Density", F::Density(marks) => marks);
            let line = crate::render::geometry::line_ink_width(
                &theme
                    .marks
                    .get_data_lines()
                    .get_density_line()
                    .backend_style((0, 0, 0)),
            )?;
            let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(
                &theme
                    .marks
                    .get_fills()
                    .get_densities()
                    .get_density_area()
                    .backend_style((0, 0, 0)),
            )?;
            (marks.primary_mark_count(), line.max(x).max(y))
        }
        PlotKind::Qq => {
            let marks = standard!("Qq", F::Qq(marks) => marks);
            (
                marks.primary_mark_count(),
                point_ink(theme.marks.get_points().get_radii().get_qq())?,
            )
        }
        PlotKind::Regression => {
            let marks = standard!("Regression", F::Regression(marks) => marks);
            (
                marks.primary_mark_count(),
                point_ink(theme.marks.get_points().get_radii().get_regression())?,
            )
        }
        PlotKind::Marginal => {
            let marks = registered
                .map(|registered| registered.family.as_ref())
                .and_then(|family| match family {
                    crate::orchestrator::registration::FamilyRef::Marginal(marks) => Some(marks),
                    _ => None,
                })
                .ok_or_else(|| Error::Render("Marginal preference lacks preparation".into()))?;
            (
                marks.checked_primary_mark_count()?,
                point_ink(theme.marks.get_points().get_radii().get_marginal())?,
            )
        }
        _ => {
            return Err(Error::Render(
                "standard mark preference requires a standard coordinate family".into(),
            ))
        }
    };
    let em = theme.typography.axes.tick_label.font_size;
    let mark_pitch = em.max(primary_ink);
    let density_side = f64::from(ceil_sqrt(count)?) * mark_pitch;
    let ch = theme
        .measure_text(
            "0",
            &theme.typography.axes.tick_label,
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
    Ok(StandardSizePreference {
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

    fn frame(
        plot: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
    ) -> crate::layout::RootFramePlan {
        crate::orchestrator::with_prepared_plot_for_test(plot, theme, |plot, inputs, request| {
            super::super::coordinate::canvas_layout(plot, inputs.plots.first(), request).unwrap()
        })
        .unwrap()
    }

    fn standard(root: &crate::layout::RootFramePlan) -> &StandardSizePreference {
        let Some(CoordinateSizePreference::Standard(preference)) = root.native_preference.as_ref()
        else {
            panic!("standard native root preference")
        };
        preference
    }

    fn categorical(root: &crate::layout::RootFramePlan) -> &CategoricalSizePreference {
        let Some(CoordinateSizePreference::Categorical(preference)) =
            root.native_preference.as_ref()
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
            crate::DumbbellEndpoints::new(
                "Before",
                crate::Color::rgb(1, 2, 3),
                "After",
                crate::Color::rgb(4, 5, 6),
            )
            .unwrap(),
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
            let chart = source;
            let theme = crate::Theme::publication().resolve().unwrap();
            let text = crate::text::TextService::default();
            let theme = crate::text::TextRequestContext::new(&theme, &text);
            let root = frame(&chart, &theme);
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
            category_dimension: "category".into(),
            value_dimension: "value".into(),
            hue_dimension: None,
            category: CategoryValues::new("category", ["a", "b"]).unwrap(),
            hue: None,
            q1: NumericValues::new("q1", [1.0; 2]),
            median: NumericValues::new("median", [3.0; 2]),
            q3: NumericValues::new("q3", [5.0; 2]),
            density_category: CategoryValues::new("category", ["a", "a", "a", "b", "b", "b"])
                .unwrap(),
            density_hue: None,
            density_y: NumericValues::new("y", [0.0, 3.0, 6.0, 0.0, 3.0, 6.0]),
            normalized_width: NumericValues::new("width", [0.0, 1.0, 0.0, 0.0, 1.0, 0.0]),
            point_category: CategoryValues::new("category", ["a", "b"]).unwrap(),
            point_hue: None,
            point_y: NumericValues::new("y", [3.0; 2]),
            normalized_strip: NumericValues::new("strip", [0.5; 2]),
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
            let chart = source;
            let theme = crate::Theme::publication().resolve().unwrap();
            let text = crate::text::TextService::default();
            let theme = crate::text::TextRequestContext::new(&theme, &text);
            let root = frame(&chart, &theme);
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
        let chart = source;
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let original = frame(&chart, &crate::text::TextRequestContext::new(&theme, &text));
        let preference = standard(&original);
        assert_eq!(preference.primary_marks, 25);
        assert_eq!(preference.mark_pitch.to_bits(), 16.0_f64.to_bits());
        assert_eq!(preference.density_side.to_bits(), 80.0_f64.to_bits());
        let width = preference.body_quanta.0;
        assert!(
            width >= crate::layout::kernel::ceil_spatial_quanta(preference.density_side).unwrap()
        );
        assert!(preference.body_quanta.1 >= (3 * width + 3) / 4);

        let radii = theme
            .marks
            .get_points()
            .get_radii()
            .clone()
            .scatter(80)
            .unwrap();
        let points = theme.marks.get_points().clone().radii(radii).unwrap();
        theme.marks = theme.marks.clone().points(points).unwrap();
        let styled = frame(&chart, &crate::text::TextRequestContext::new(&theme, &text));
        let preference = standard(&styled);
        assert_eq!(preference.primary_marks, 25);
        assert!(preference.mark_pitch > 160.0);
        assert!(preference.density_side > 800.0);
        assert!(preference.body_quanta.0 > width);
        assert_eq!(styled.panel_box, original.panel_box);

        let mut primary = theme.marks.get_points().get_primary().clone();
        primary.fill_alpha = 0;
        primary.outline.as_mut().unwrap().color = crate::theme::Color::rgba(255, 255, 255, 0);
        let points = theme.marks.get_points().clone().primary(primary).unwrap();
        theme.marks = theme.marks.clone().points(points).unwrap();
        let invisible = frame(&chart, &crate::text::TextRequestContext::new(&theme, &text));
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
        let clipped = source;
        let exact = frame(
            &clipped,
            &crate::text::TextRequestContext::new(&theme, &text),
        );
        assert_eq!(standard(&exact).primary_marks, 25);
    }
}
