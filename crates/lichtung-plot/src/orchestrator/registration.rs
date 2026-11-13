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
    Heatmap(crate::plots::heatmap::HeatmapData),
    Beeswarm(crate::plots::beeswarm::PreparedBeeswarm),
    Waterfall(crate::plots::waterfall::PreparedWaterfall),
    Marginal(crate::plots::marginal::PreparedMarginal),
    Matrix(crate::plots::matrix::PreparedMatrixAxes),
}

impl PreparedFamily {
    pub(crate) const fn as_ref(&self) -> FamilyRef<'_> {
        match self {
            Self::Scatter(marks) => FamilyRef::Scatter(marks),
            Self::Line { kind, marks } => FamilyRef::Line { kind: *kind, marks },
            Self::Histogram(marks) => FamilyRef::Histogram(marks),
            Self::Density(marks) => FamilyRef::Density(marks),
            Self::Qq(marks) => FamilyRef::Qq(marks),
            Self::Regression(marks) => FamilyRef::Regression(marks),
            Self::Bars { kind, mode, marks } => FamilyRef::Bars {
                kind: *kind,
                mode: *mode,
                marks,
            },
            Self::Estimate { kind, marks } => FamilyRef::Estimate { kind: *kind, marks },
            Self::Dumbbell(marks) => FamilyRef::Dumbbell(marks),
            Self::Distribution { kind, marks } => FamilyRef::Distribution { kind: *kind, marks },
            Self::Interaction(marks) => FamilyRef::Interaction(marks),
            Self::Heatmap(marks) => FamilyRef::Heatmap(marks),
            Self::Beeswarm(marks) => FamilyRef::Beeswarm(marks),
            Self::Waterfall(marks) => FamilyRef::Waterfall(marks),
            Self::Marginal(marks) => FamilyRef::Marginal(marks),
            Self::Matrix(marks) => FamilyRef::Matrix(marks),
        }
    }

    pub(crate) const fn kind(&self) -> crate::PlotKind {
        self.as_ref().kind()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum FamilyRef<'v> {
    Scatter(&'v crate::plots::scatter::PreparedScatter),
    Line {
        kind: crate::PlotKind,
        marks: &'v crate::plots::line::PreparedLine,
    },
    Histogram(&'v crate::plots::histogram::HistogramSemantics),
    Density(&'v crate::plots::density::PreparedDensity),
    Qq(&'v crate::plots::qq::PreparedQq),
    Regression(&'v crate::plots::regression::PreparedRegression),
    Bars {
        kind: crate::PlotKind,
        mode: crate::plots::bar_data::BarMode,
        marks: &'v crate::plots::bar_data::PreparedBars,
    },
    Estimate {
        kind: crate::PlotKind,
        marks: &'v crate::plots::interval_estimate::PreparedEstimate,
    },
    Dumbbell(&'v crate::plots::dumbbell::DumbbellData),
    Distribution {
        kind: crate::PlotKind,
        marks: &'v crate::plots::categorical_distribution::PreparedDistribution,
    },
    Interaction(&'v crate::plots::interaction::PreparedInteraction),
    Heatmap(&'v crate::plots::heatmap::HeatmapData),
    Beeswarm(&'v crate::plots::beeswarm::PreparedBeeswarm),
    Waterfall(&'v crate::plots::waterfall::PreparedWaterfall),
    Marginal(&'v crate::plots::marginal::PreparedMarginal),
    Matrix(&'v crate::plots::matrix::PreparedMatrixAxes),
}

impl FamilyRef<'_> {
    pub(crate) const fn kind(self) -> crate::PlotKind {
        match self {
            Self::Scatter(_) => crate::PlotKind::Scatter,
            Self::Histogram(_) => crate::PlotKind::Histogram,
            Self::Density(_) => crate::PlotKind::Density,
            Self::Qq(_) => crate::PlotKind::Qq,
            Self::Regression(_) => crate::PlotKind::Regression,
            Self::Line { kind, .. }
            | Self::Bars { kind, .. }
            | Self::Estimate { kind, .. }
            | Self::Distribution { kind, .. } => kind,
            Self::Dumbbell(_) => crate::PlotKind::Dumbbell,
            Self::Interaction(_) => crate::PlotKind::Interaction,
            Self::Heatmap(_) => crate::PlotKind::Heatmap,
            Self::Beeswarm(_) => crate::PlotKind::Beeswarm,
            Self::Waterfall(_) => crate::PlotKind::Waterfall,
            Self::Marginal(_) => crate::PlotKind::Marginal,
            Self::Matrix(_) => crate::PlotKind::Matrix,
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
