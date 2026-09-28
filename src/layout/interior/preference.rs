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

fn panel<'a, T>(
    panels: &'a [(Option<String>, T)],
    scope: Option<&str>,
    owner: &str,
    resources: &OperationGenerationResources,
) -> Result<&'a T, Error> {
    resources
        .claim_visits(panels.len() as u128)
        .map_err(|error| {
            Error::Render(format!("{owner} primary-mark scope admission: {error:?}"))
        })?;
    panels
        .iter()
        .find(|(key, _)| key.as_deref() == scope)
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render(format!("{owner} primary-mark scope is not prepared")))
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
        let mut theme = crate::Theme::Publication.resolve();
        let original = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        let preference = original.native_preference.as_ref().unwrap();
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
        let preference = styled.native_preference.as_ref().unwrap();
        assert_eq!(preference.primary_marks, 25);
        assert!(preference.mark_pitch > 160.0);
        assert!(preference.density_side > 800.0);
        assert!(preference.body_quanta.0 > width);
        assert_eq!(styled.panel_box, original.panel_box);

        theme.marks.primary_point.fill_alpha = 0;
        theme.marks.primary_point.outline.color =
            crate::render::backend::Rgba::new(255, 255, 255, 0);
        let invisible = super::super::coordinate::canvas_layout(&chart, &theme).unwrap();
        assert_eq!(
            invisible.native_preference.as_ref().unwrap().primary_marks,
            25
        );
        assert_eq!(
            invisible.native_preference.unwrap().mark_pitch.to_bits(),
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
        assert_eq!(exact.native_preference.unwrap().primary_marks, 25);
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
                &crate::Theme::Publication.resolve(),
            )
            .unwrap();
            assert_eq!(root.native_preference.unwrap().primary_marks, expected);
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
        let root =
            super::super::coordinate::canvas_layout(&chart, &crate::Theme::Publication.resolve())
                .unwrap();
        assert_eq!(root.native_preference.unwrap().primary_marks, 4);
    }
}
