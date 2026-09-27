//! Discrete entries from one selected prepared semantic owner, without a panel
//! assignment, font environment, or backend measurement.

use super::PreparedInteriorInputs;
use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

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
                let order = marks.group_order(chart, data)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                let marks = panel(self.line.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                let order = marks.group_order(chart, data)?;
                marks.guide_entries_in_order(chart, theme, data, &order)
            }
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => {
                let marks = panel(self.bars.as_ref().map(|p| p.panels.as_slice()), scope, kind)?;
                crate::plots::bar_data::legend_entries(marks, chart, theme, data)
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
                let order = marks.group_order(chart, data)?;
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
                crate::plots::regression::PreparedRegression::guide_entries(chart, theme, data)
            }
            PlotKind::Dumbbell => {
                panel(
                    self.dumbbell.as_ref().map(|p| p.panels.as_slice()),
                    scope,
                    kind,
                )?;
                crate::plots::dumbbell::DumbbellData::guide_entries(chart, theme, data)
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
