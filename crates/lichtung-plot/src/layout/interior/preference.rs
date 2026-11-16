//! Bottom-up standard-panel soft sizes; exact root assignment remains separate.

use crate::layout::PhysicalTextAlignment;
use crate::orchestrator::registration::PreparedFamily as F;
use crate::{Error, PlotKind};

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
    registered: &crate::orchestrator::registration::PlotRegistration<'_>,
    _domains: crate::builder::RootFrameDomainInput<'_>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    hard: (f64, f64),
) -> Result<CategoricalSizePreference, Error> {
    let plot = registered.source;
    macro_rules! family {
        ($missing:literal, $owner:literal, $pattern:pat => $marks:ident) => {{
            let $pattern = panel(Some(registered), $owner)? else {
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
                registered
                    .axes
                    .categories(1)
                    .expect("prepared Estimate categories")
                    .raw
                    .len(),
                marks.primary_mark_count(),
            )
        }
        PlotKind::Dumbbell => {
            let marks = family!("Dumbbell preference lacks preparation", "Dumbbell", F::Dumbbell(marks) => marks);
            (
                crate::layout::AxisSide::Left,
                registered
                    .axes
                    .categories(1)
                    .expect("prepared Dumbbell categories")
                    .raw
                    .len(),
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
            (
                axis,
                registered
                    .axes
                    .categories(usize::from(axis == crate::layout::AxisSide::Left))
                    .expect("prepared Bar categories")
                    .raw
                    .len(),
                marks.primary_mark_count(),
            )
        }
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
            let marks = family!("distribution preference lacks preparation", "Distribution", F::Distribution { marks, .. } => marks);
            (
                crate::layout::AxisSide::Bottom,
                registered
                    .axes
                    .categories(0)
                    .expect("prepared distribution categories")
                    .raw
                    .len(),
                marks.primary_mark_count(),
            )
        }
        PlotKind::Interaction => {
            let marks = family!("Interaction preference lacks preparation", "Interaction", F::Interaction(marks) => marks);
            (
                crate::layout::AxisSide::Bottom,
                registered
                    .axes
                    .categories(0)
                    .expect("prepared Interaction categories")
                    .raw
                    .len(),
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
    owner: &str,
) -> Result<&'a crate::orchestrator::registration::PreparedFamily, Error> {
    if let Some(registered) = registered {
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
    occurrence: &crate::orchestrator::registration::PlotRegistration<'_>,
    _domains: crate::builder::RootFrameDomainInput<'_>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    hard: (f64, f64),
) -> Result<StandardSizePreference, Error> {
    let plot = occurrence.source;
    let registered = Some(occurrence);
    macro_rules! standard {
        ($owner:literal, $pattern:pat => $marks:ident) => {{
            if registered.is_none() {
                return Err(Error::Render(format!(
                    "{} mark preference lacks preparation",
                    $owner
                )));
            }
            let $pattern = panel(registered, $owner)? else {
                return Err(Error::Render(format!(
                    "{} mark preference lacks preparation",
                    $owner
                )));
            };
            $marks
        }};
    }
    let point_ink = |radius| -> Result<f64, Error> {
        let point = plot.point_style(theme, (0, 0, 0), radius);
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
            let marker = if plot.kind() == PlotKind::Trajectory || plot.markers_visible() {
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
