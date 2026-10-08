//! Discrete entries from one selected prepared semantic owner, without a panel
//! assignment, font environment, or backend measurement.

use super::PreparedInteriorInputs;
use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

/// One measured mandatory right guide beside the flexible scientific body.
pub(crate) fn continuous_right(
    descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    panel: crate::layout::PanelBox,
    minimum: f64,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,

    content_plot: Option<&Plot>,
) -> Result<
    (
        crate::layout::PanelBox,
        crate::decorations::legend::MeasuredContinuousGuide,
    ),
    crate::layout::fit::PlanError,
> {
    use crate::layout::fit::{self, FitKind};
    use crate::layout::kernel::{allocate_grid_axis, GridRequirement, TrackSpec};
    let widths =
        crate::decorations::legend::MeasuredContinuousGuide::vertical_widths(descriptor, theme)?;
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
    let guide = fit::measured(
        crate::decorations::legend::MeasuredContinuousGuide::measure(
            descriptor,
            (width, content_plot.is_none().then_some(panel.height())),
            false,
            false,
            theme,
            &mut false,
            scope,
        ),
    )?;
    let hard = crate::layout::kernel::ceil_spatial_quanta(minimum)?;
    let width = crate::layout::kernel::ceil_spatial_quanta(guide.width)?;
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
    let top = if let Some(plot) = content_plot {
        let hard = plot.intrinsic_body_minimum(theme)?;
        let preference = crate::layout::interior::sizing::body_preference(plot, theme, hard)?;
        panel.y0
            + plot
                .natural_body_height(theme, body_right - panel.x0, preference.1)?
                .max(guide.height)
    } else {
        panel.y1
    };
    Ok((
        crate::layout::PanelBox::new(panel.x0, body_right, panel.y0, top),
        guide,
    ))
}

fn panel<'a, T>(
    panels: Option<&'a [(Option<String>, T)]>,
    scope: Option<&str>,
    kind: PlotKind,
) -> Result<&'a T, Error> {
    panels
        .and_then(|panels| panels.iter().find(|(owner, _)| owner.as_deref() == scope))
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render(format!(
                "{kind:?} guide is missing prepared semantics for {scope:?}"
            ))
        })
}

impl PreparedInteriorInputs {
    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: PlotData<'_>,
        scope: Option<&str>,
    ) -> Result<Vec<LegendEntry>, Error> {
        self.route.validate(chart)?;
        let kind = chart.kind();
        match kind {
            PlotKind::Scatter => {
                let marks = panel(
                    self.scatter.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                let marks = panel(self.line.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let marks = panel(self.bars.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                let (colors, accents) = marks.resolved_colors(chart, theme, data)?;
                crate::plots::bar_data::legend_entries(marks, chart, theme, &colors, &accents)
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => {
                let marks = panel(
                    self.distributions.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                marks.guide_entries(chart, theme, data)
            }
            PlotKind::Density => {
                let marks = panel(
                    self.density.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let order = marks.group_order(chart)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Interaction => {
                let marks = panel(
                    self.interaction.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                marks.guide_entries(chart, theme, data)
            }
            PlotKind::Marginal => self
                .marginal
                .as_ref()
                .ok_or_else(|| {
                    Error::Render("Marginal guide is missing prepared semantics".into())
                })?
                .guide_entries(chart, theme, data),
            PlotKind::Regression => {
                panel(
                    self.regression.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                let color =
                    crate::theme::AccentColors::resolve(&theme.marks.accents, 1)?.color(0)?;
                crate::plots::regression::PreparedRegression::guide_entries(chart, theme, color)
            }
            PlotKind::Dumbbell => {
                panel(
                    self.dumbbell.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                crate::plots::dumbbell::DumbbellData::guide_entries(chart, theme)
            }
            PlotKind::Histogram
            | PlotKind::Qq
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Waterfall => Ok(Vec::new()),
            PlotKind::Heatmap | PlotKind::Beeswarm => Err(Error::Render(
                "continuous guides cannot be prepared as discrete entries".into(),
            )),
            PlotKind::Matrix => Err(Error::Render(
                "self-managed Matrix has no parent-owned guide entry".into(),
            )),
        }
    }
}
