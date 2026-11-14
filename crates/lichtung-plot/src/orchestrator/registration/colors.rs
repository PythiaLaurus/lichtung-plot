//! Occurrence-local color inputs are independent of scientific and guide order.

use std::sync::Arc;

use crate::builder::PlotDescription;
use crate::data::category::{CategoryDomain, RawField};
use crate::{CategoricalColorScale, Error, PlotKind};

#[derive(Debug)]
pub(crate) enum CategoricalDomainSource {
    Prepared(Arc<CategoryDomain>),
    Axis(super::AxisSlot),
    DistributionFacetCategories,
    Observed,
}

#[derive(Debug)]
enum CategoricalColorSource {
    Explicit(CategoricalColorScale),
    Theme,
}

#[derive(Debug)]
pub(crate) struct CategoricalBinding {
    pub(crate) local: CategoricalDomainSource,
    pub(crate) parent: Option<Arc<CategoryDomain>>,
    field: Option<RawField>,
    selection: CategoricalColorSource,
}

impl CategoricalBinding {
    pub(crate) fn for_source(source: &PlotDescription) -> Option<Self> {
        if !matches!(
            source.kind(),
            PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Line
                | PlotKind::Trajectory
                | PlotKind::LinePath
                | PlotKind::Scatter
                | PlotKind::Density
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Raincloud
                | PlotKind::Interaction
                | PlotKind::Marginal
                | PlotKind::Matrix
        ) {
            return None;
        }
        let local = match source.kind() {
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar
                if source.hue_dimension_key().is_none() =>
            {
                CategoricalDomainSource::Axis(super::AxisSlot::Pair(
                    if matches!(
                        source.kind(),
                        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
                    ) {
                        crate::layout::AxisSide::Left
                    } else {
                        crate::layout::AxisSide::Bottom
                    },
                ))
            }
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
                if source.hue_dimension_key().is_none() =>
            {
                if source.has_facets() && source.raincloud_geometry_input().is_none() {
                    CategoricalDomainSource::DistributionFacetCategories
                } else {
                    CategoricalDomainSource::Axis(super::AxisSlot::Pair(
                        crate::layout::AxisSide::Bottom,
                    ))
                }
            }
            _ => CategoricalDomainSource::Observed,
        };
        let selection = source
            .categorical_color_scale_spec()
            .filter(|_| !source.colors_by_sign())
            .cloned()
            .map_or(
                CategoricalColorSource::Theme,
                CategoricalColorSource::Explicit,
            );
        Some(Self {
            local,
            parent: None,
            field: source.categorical_field(),
            selection,
        })
    }

    pub(crate) fn set_local_hues(&mut self, hues: Vec<String>) -> Result<(), Error> {
        if !hues.is_empty() || matches!(self.local, CategoricalDomainSource::Observed) {
            self.local = CategoricalDomainSource::Prepared(crate::data::source::source_arc(
                CategoryDomain::from_values(self.field.as_ref(), hues)?,
            )?);
        }
        Ok(())
    }

    pub(crate) fn set_local_keys(
        &mut self,
        ordered: Vec<crate::data::category::CategoryKey>,
    ) -> Result<(), Error> {
        self.local =
            CategoricalDomainSource::Prepared(crate::data::source::source_arc(CategoryDomain {
                ordered,
            })?);
        Ok(())
    }

    pub(crate) fn local_values(&self) -> impl Iterator<Item = &str> + Clone {
        let domain = match &self.local {
            CategoricalDomainSource::Prepared(domain) => Some(domain.as_ref()),
            _ => None,
        };
        domain.into_iter().flat_map(CategoryDomain::values)
    }

    pub(crate) fn axis_categories<'a>(
        &self,
        axes: &'a super::PreparedAxesOwner,
    ) -> Option<&'a [String]> {
        match self.local {
            CategoricalDomainSource::Axis(super::AxisSlot::Pair(side)) => Some(
                &axes
                    .categories(usize::from(side == crate::layout::AxisSide::Left))
                    .expect("prepared categorical color axis")
                    .raw,
            ),
            _ => None,
        }
    }

    pub(crate) fn local_len(&self) -> usize {
        match &self.local {
            CategoricalDomainSource::Prepared(domain) => domain.ordered.len(),
            _ => 0,
        }
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &str> + Clone {
        self.parent
            .as_deref()
            .into_iter()
            .flat_map(CategoryDomain::values)
            .chain(self.local_values().filter(|_| self.parent.is_none()))
    }

    pub(crate) fn resolve<'a>(
        &self,
        values: impl Iterator<Item = &'a str> + Clone,
        theme: &crate::theme::ResolvedTheme,
        operation: &crate::font::canonical::OperationContext,
    ) -> Result<crate::theme::CategoryColors, Error> {
        crate::theme::CategoryColors::resolve(
            self.field.as_ref(),
            values,
            theme.color_scales.get_qualitative(),
            match &self.selection {
                CategoricalColorSource::Explicit(selection) => Some(selection),
                CategoricalColorSource::Theme => None,
            },
            operation,
        )
    }
}
