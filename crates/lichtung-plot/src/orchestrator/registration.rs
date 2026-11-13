//! Operation-local occurrence identity and selected input rows.

use crate::{builder::PlotDescription, Error};

#[derive(Debug, Clone)]
pub(crate) struct RootIdentity(pub(crate) std::sync::Arc<()>);

impl PartialEq for RootIdentity {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for RootIdentity {}

#[derive(Debug)]
pub(crate) enum PreparedFamily {
    Scatter(crate::plots::scatter::PreparedScatter),
    Line {
        kind: crate::PlotKind,
        marks: crate::plots::line::PreparedLine,
    },
    Histogram(crate::plots::histogram::HistogramSemantics),
    Density(crate::plots::density::PreparedDensity),
    Qq(crate::plots::qq::PreparedQq),
    Regression(crate::plots::regression::PreparedRegression),
    Bars {
        kind: crate::PlotKind,
        mode: crate::plots::bar_data::BarMode,
        marks: crate::plots::bar_data::PreparedBars,
    },
    Estimate {
        kind: crate::PlotKind,
        marks: crate::plots::interval_estimate::PreparedEstimate,
    },
    Dumbbell(crate::plots::dumbbell::DumbbellData),
    Distribution {
        kind: crate::PlotKind,
        marks: crate::plots::categorical_distribution::PreparedDistribution,
    },
    Interaction(crate::plots::interaction::PreparedInteraction),
}

impl PreparedFamily {
    pub(crate) const fn kind(&self) -> crate::PlotKind {
        match self {
            Self::Scatter(_) => crate::PlotKind::Scatter,
            Self::Histogram(_) => crate::PlotKind::Histogram,
            Self::Density(_) => crate::PlotKind::Density,
            Self::Qq(_) => crate::PlotKind::Qq,
            Self::Regression(_) => crate::PlotKind::Regression,
            Self::Line { kind, .. }
            | Self::Bars { kind, .. }
            | Self::Estimate { kind, .. }
            | Self::Distribution { kind, .. } => *kind,
            Self::Dumbbell(_) => crate::PlotKind::Dumbbell,
            Self::Interaction(_) => crate::PlotKind::Interaction,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum FacetRole {
    Row,
    Column,
    Wrap,
}

#[derive(Clone, Copy)]
pub(crate) struct FacetSelector<'src, 'op> {
    schema: &'src crate::FacetSpec,
    levels: &'op crate::FacetCell,
}

impl<'src, 'op> FacetSelector<'src, 'op> {
    pub(crate) fn checked(
        schema: &'src crate::FacetSpec,
        levels: &'op crate::FacetCell,
    ) -> Result<Self, Error> {
        let dimensions = schema.values().count();
        if dimensions != levels.dimensions() {
            return Err(Error::Render(
                "significance FacetCell shape does not match the configured facet layout".into(),
            ));
        }
        Ok(Self { schema, levels })
    }

    pub(crate) fn fields(&self) -> Vec<(FacetRole, &'src crate::CategoryValues, &'op str)> {
        if let Some((row, column)) = self.levels.grid_levels() {
            [
                (FacetRole::Row, self.schema.row_values(), row),
                (FacetRole::Column, self.schema.column_values(), column),
            ]
            .into_iter()
            .filter_map(|(role, values, level)| values.map(|values| (role, values, level)))
            .collect()
        } else {
            let level = self.levels.single_level().expect("checked single facet");
            [
                (FacetRole::Row, self.schema.row_values()),
                (FacetRole::Column, self.schema.column_values()),
                (FacetRole::Wrap, self.schema.wrap_values()),
            ]
            .into_iter()
            .filter_map(|(role, values)| values.map(|values| (role, values, level)))
            .collect()
        }
    }
}

pub(crate) enum RowSelection {
    Selected {
        input_slots: Vec<usize>,
        diagnostic_rows: Vec<usize>,
    },
}

impl RowSelection {
    pub(crate) fn selected(&self) -> (&[usize], &[usize]) {
        match self {
            Self::Selected {
                input_slots,
                diagnostic_rows,
            } => (input_slots, diagnostic_rows),
        }
    }
}

pub(crate) fn select_rows(
    source: &PlotDescription,
    input_slots: Vec<usize>,
) -> Result<RowSelection, Error> {
    let rows = source.source_row_map();
    let diagnostic_rows = crate::data::source::collect_source(
        input_slots
            .iter()
            .map(|index| Ok(rows.map_or(*index, |rows| rows.as_slice()[*index]))),
    )?;
    Ok(RowSelection::Selected {
        input_slots,
        diagnostic_rows,
    })
}
