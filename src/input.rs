//! Closed semantic inputs for supported plot families.

use crate::{
    BarInput, Bounds, CategoryValues, CoefficientInput, HistogramCompositeGeometryInput,
    NumericValues, RaincloudGeometryInput,
};

/// Closed owned source for a histogram.
#[derive(Debug, Clone)]
pub struct HistogramInput {
    source: HistogramSource,
}

#[derive(Debug, Clone)]
enum HistogramSource {
    Samples(NumericValues),
    Geometry(HistogramCompositeGeometryInput),
}

impl HistogramInput {
    /// Compute histogram geometry from one numeric sample.
    #[must_use]
    pub fn samples(values: NumericValues) -> Self {
        Self {
            source: HistogramSource::Samples(values),
        }
    }

    /// Render complete caller-computed density-axis geometry.
    #[must_use]
    pub fn geometry(input: HistogramCompositeGeometryInput) -> Self {
        Self {
            source: HistogramSource::Geometry(input),
        }
    }

    pub(crate) const fn samples_input(&self) -> Option<&NumericValues> {
        match &self.source {
            HistogramSource::Samples(values) => Some(values),
            HistogramSource::Geometry(_) => None,
        }
    }

    pub(crate) const fn geometry_input(&self) -> Option<&HistogramCompositeGeometryInput> {
        match &self.source {
            HistogramSource::Samples(_) => None,
            HistogramSource::Geometry(input) => Some(input),
        }
    }
}

/// Closed owned source for a raincloud plot.
#[derive(Debug, Clone)]
pub struct RaincloudInput {
    source: RaincloudSource,
}

#[derive(Debug, Clone)]
enum RaincloudSource {
    Samples {
        category: CategoryValues,
        values: NumericValues,
    },
    Geometry(RaincloudGeometryInput),
}

impl RaincloudInput {
    /// Compute raincloud geometry from aligned categories and numeric samples.
    #[must_use]
    pub fn samples(category: CategoryValues, values: NumericValues) -> Self {
        Self {
            source: RaincloudSource::Samples { category, values },
        }
    }

    /// Render complete caller-computed raincloud geometry.
    #[must_use]
    pub fn geometry(input: RaincloudGeometryInput) -> Self {
        Self {
            source: RaincloudSource::Geometry(input),
        }
    }

    pub(crate) const fn samples_input(&self) -> Option<(&CategoryValues, &NumericValues)> {
        match &self.source {
            RaincloudSource::Samples { category, values } => Some((category, values)),
            RaincloudSource::Geometry(_) => None,
        }
    }

    pub(crate) const fn geometry_input(&self) -> Option<&RaincloudGeometryInput> {
        match &self.source {
            RaincloudSource::Samples { .. } => None,
            RaincloudSource::Geometry(input) => Some(input),
        }
    }
}

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
        input: HistogramInput,
    },
    Density {
        values: NumericValues,
    },
    Qq {
        values: NumericValues,
    },
    PointInterval {
        estimate: NumericValues,
        category: CategoryValues,
        bounds: Bounds,
    },
    Line {
        x: AxisValues,
        y: NumericValues,
    },
    Interaction {
        condition: CategoryValues,
        estimate: NumericValues,
        bounds: Bounds,
    },
    Coefficient {
        input: CoefficientInput,
        term: CategoryValues,
    },
    Forest {
        estimate: NumericValues,
        term: CategoryValues,
        bounds: Bounds,
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
        input: BarInput,
    },
    HorizontalBar {
        category: CategoryValues,
        input: BarInput,
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
    Raincloud {
        input: RaincloudInput,
    },
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
            Self::Histogram { input } => input.samples_input().map_or(0, NumericValues::len),
            Self::Density { values } | Self::Qq { values } => values.len(),
            Self::PointInterval { estimate, .. }
            | Self::Interaction { estimate, .. }
            | Self::Forest { estimate, .. } => estimate.len(),
            Self::Coefficient { input, .. } => input.values().len(),
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
            Self::Raincloud { input } => input
                .samples_input()
                .map_or(0, |(category, _)| category.len()),
            Self::Matrix { variables } => variables.first().map_or(0, NumericValues::len),
            Self::Waterfall { labels, .. } => labels.len(),
        }
    }

    pub(crate) fn x_name(&self) -> Option<&str> {
        match self {
            Self::Scatter { x, .. } | Self::Regression { x, .. } | Self::Marginal { x, .. } => {
                Some(x.name())
            }
            Self::Histogram { input } => input.samples_input().map(NumericValues::name),
            Self::Density { values } | Self::Qq { values } => Some(values.name()),
            Self::PointInterval { estimate, .. } | Self::Forest { estimate, .. } => {
                Some(estimate.name())
            }
            Self::Coefficient { input, .. } => Some(input.values().name()),
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
            Self::Raincloud { input } => input.samples_input().map(|(category, _)| category.name()),
            Self::Beeswarm { value, .. } => Some(value.name()),
            Self::Matrix { .. } => None,
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
            Self::Raincloud { input } => input.samples_input().map(|(_, values)| values.name()),
            Self::Bar { input, .. } | Self::HorizontalBar { input, .. } => {
                Some(input.values().name())
            }
            Self::Heatmap { y, .. } => Some(y.name()),
            Self::Histogram { .. }
            | Self::Density { .. }
            | Self::Qq { .. }
            | Self::Matrix { .. } => None,
        }
    }
}
