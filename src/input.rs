//! Closed semantic inputs for supported plot families.

use crate::{CategoryValues, NumericValues};

/// Native values accepted by a line plot's horizontal axis.
#[derive(Debug, Clone)]
pub enum AxisValues {
    /// Continuous numeric axis values.
    Numeric(NumericValues),
    /// Ordered categorical axis values.
    Category(CategoryValues),
}

impl From<NumericValues> for AxisValues {
    fn from(values: NumericValues) -> Self {
        Self::Numeric(values)
    }
}

impl From<CategoryValues> for AxisValues {
    fn from(values: CategoryValues) -> Self {
        Self::Category(values)
    }
}

impl AxisValues {
    /// Semantic value name used for inferred labels.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Numeric(values) => values.name(),
            Self::Category(values) => values.name(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Numeric(values) => values.len(),
            Self::Category(values) => values.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PlotInput {
    Scatter {
        x: NumericValues,
        y: NumericValues,
    },
    Histogram {
        values: NumericValues,
    },
    HistogramGeometry,
    Density {
        values: NumericValues,
    },
    Qq {
        values: NumericValues,
    },
    PointInterval {
        estimate: NumericValues,
        category: CategoryValues,
    },
    Line {
        x: AxisValues,
        y: NumericValues,
    },
    Interaction {
        condition: CategoryValues,
        estimate: NumericValues,
    },
    Coefficient {
        estimate: NumericValues,
        term: CategoryValues,
    },
    Forest {
        estimate: NumericValues,
        term: CategoryValues,
    },
    Dumbbell {
        category: CategoryValues,
        left: NumericValues,
    },
    Heatmap {
        x: CategoryValues,
        y: CategoryValues,
    },
    Bar {
        category: CategoryValues,
        value: NumericValues,
    },
    HorizontalBar {
        category: CategoryValues,
        value: NumericValues,
    },
    StackedBar {
        category: CategoryValues,
        value: NumericValues,
    },
    StackedHorizontalBar {
        category: CategoryValues,
        value: NumericValues,
    },
    Boxplot {
        category: CategoryValues,
        value: NumericValues,
    },
    Violin {
        category: CategoryValues,
        value: NumericValues,
    },
    RaincloudGeometry,
    Regression {
        x: NumericValues,
        y: NumericValues,
    },
    Marginal {
        x: NumericValues,
        y: NumericValues,
    },
    Matrix {
        variables: Vec<NumericValues>,
    },
    Beeswarm {
        value: NumericValues,
        category: CategoryValues,
    },
    Waterfall {
        labels: CategoryValues,
        deltas: NumericValues,
    },
}

impl PlotInput {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Scatter { x, .. } | Self::Regression { x, .. } | Self::Marginal { x, .. } => {
                x.len()
            }
            Self::Histogram { values } | Self::Density { values } | Self::Qq { values } => {
                values.len()
            }
            Self::PointInterval { estimate, .. }
            | Self::Interaction { estimate, .. }
            | Self::Coefficient { estimate, .. }
            | Self::Forest { estimate, .. } => estimate.len(),
            Self::Line { x, .. } => x.len(),
            Self::Dumbbell { left, .. } => left.len(),
            Self::Heatmap { x, .. }
            | Self::Bar { category: x, .. }
            | Self::HorizontalBar { category: x, .. }
            | Self::StackedBar { category: x, .. }
            | Self::StackedHorizontalBar { category: x, .. }
            | Self::Boxplot { category: x, .. }
            | Self::Violin { category: x, .. }
            | Self::Beeswarm { category: x, .. } => x.len(),
            Self::Matrix { variables } => variables.first().map_or(0, NumericValues::len),
            Self::Waterfall { labels, .. } => labels.len(),
            Self::HistogramGeometry | Self::RaincloudGeometry => 0,
        }
    }

    pub(crate) fn x_name(&self) -> Option<&str> {
        match self {
            Self::Scatter { x, .. } | Self::Regression { x, .. } | Self::Marginal { x, .. } => {
                Some(x.name())
            }
            Self::Histogram { values } | Self::Density { values } | Self::Qq { values } => {
                Some(values.name())
            }
            Self::PointInterval { estimate, .. }
            | Self::Coefficient { estimate, .. }
            | Self::Forest { estimate, .. } => Some(estimate.name()),
            Self::Line { x, .. } => Some(x.name()),
            Self::Interaction { condition, .. }
            | Self::Dumbbell {
                category: condition,
                ..
            }
            | Self::Heatmap { x: condition, .. }
            | Self::Bar {
                category: condition,
                ..
            }
            | Self::HorizontalBar {
                category: condition,
                ..
            }
            | Self::StackedBar {
                category: condition,
                ..
            }
            | Self::StackedHorizontalBar {
                category: condition,
                ..
            }
            | Self::Boxplot {
                category: condition,
                ..
            }
            | Self::Violin {
                category: condition,
                ..
            }
            | Self::Waterfall {
                labels: condition, ..
            } => Some(condition.name()),
            Self::Beeswarm { value, .. } => Some(value.name()),
            Self::HistogramGeometry | Self::RaincloudGeometry | Self::Matrix { .. } => None,
        }
    }

    pub(crate) fn y_name(&self) -> Option<&str> {
        match self {
            Self::Scatter { y, .. }
            | Self::Line { y, .. }
            | Self::Regression { y, .. }
            | Self::Marginal { y, .. } => Some(y.name()),
            Self::PointInterval { category, .. }
            | Self::Coefficient { term: category, .. }
            | Self::Forest { term: category, .. }
            | Self::Beeswarm { category, .. } => Some(category.name()),
            Self::Interaction { estimate, .. }
            | Self::Dumbbell { left: estimate, .. }
            | Self::Bar {
                value: estimate, ..
            }
            | Self::HorizontalBar {
                value: estimate, ..
            }
            | Self::StackedBar {
                value: estimate, ..
            }
            | Self::StackedHorizontalBar {
                value: estimate, ..
            }
            | Self::Boxplot {
                value: estimate, ..
            }
            | Self::Violin {
                value: estimate, ..
            }
            | Self::Waterfall {
                deltas: estimate, ..
            } => Some(estimate.name()),
            Self::Heatmap { y, .. } => Some(y.name()),
            Self::Histogram { .. }
            | Self::HistogramGeometry
            | Self::Density { .. }
            | Self::Qq { .. }
            | Self::RaincloudGeometry
            | Self::Matrix { .. } => None,
        }
    }
}
