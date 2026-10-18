//! Typed scientific fields/categories, literal domains and supplemental layers.
//! Display aliases never participate in raw identity or domain coordination.

use std::sync::Arc;

use crate::{Error, Plot, RegressionBand, Result};

pub(super) fn shared_text(source: &str) -> Result<Arc<str>> {
    let bytes = std::alloc::Layout::array::<u8>(source.len())
        .map_err(|_| Error::Render("guide identity text layout overflow".into()))?;
    std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(bytes)
        .map_err(|_| Error::Render("guide identity Arc layout overflow".into()))?;
    Ok(Arc::from(crate::data::source::copy_source_string(source)?))
}

macro_rules! raw_identity {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) struct $name(Arc<str>);

        impl $name {
            fn new(value: &str) -> Result<Self> {
                shared_text(value).map(Self)
            }

            pub(crate) fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

raw_identity!(RawField);
raw_identity!(RawCategory);
raw_identity!(RawLayerLabel);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PrimaryTitleIdentity {
    Field(RawField),
    Pair { group: RawField, hue: RawField },
    Series,
    BarSign,
}

impl PrimaryTitleIdentity {
    pub(crate) fn resolved_text(&self, plot: &Plot) -> Result<String> {
        match self {
            Self::Field(field) => plot.resolve_label(field.as_str()),
            Self::Pair { group, hue } => join(
                plot.resolve_label(group.as_str())?,
                " × ",
                &plot.resolve_label(hue.as_str())?,
            ),
            Self::Series => crate::data::source::copy_source_string("series"),
            Self::BarSign => crate::data::source::copy_source_string("Change"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideEntryIdentity {
    Category {
        field: RawField,
        value: RawCategory,
    },
    Pair {
        group_field: RawField,
        group_value: RawCategory,
        hue_field: RawField,
        hue_value: RawCategory,
    },
    BarSign {
        decreasing: bool,
    },
    DumbbellEndpoint {
        label: Arc<str>,
    },
    RegressionObservation,
    RegressionFit {
        band: RegressionBand,
    },
    SupplementalLayer {
        label: RawLayerLabel,
    },
}

impl GuideEntryIdentity {
    pub(crate) fn category(field: &str, value: &str) -> Result<Self> {
        Ok(Self::Category {
            field: RawField::new(field)?,
            value: RawCategory::new(value)?,
        })
    }

    pub(crate) fn pair(
        group_field: &str,
        group_value: &str,
        hue_field: &str,
        hue_value: &str,
    ) -> Result<Self> {
        Ok(Self::Pair {
            group_field: RawField::new(group_field)?,
            group_value: RawCategory::new(group_value)?,
            hue_field: RawField::new(hue_field)?,
            hue_value: RawCategory::new(hue_value)?,
        })
    }

    pub(crate) const fn bar_sign(decreasing: bool) -> Self {
        Self::BarSign { decreasing }
    }

    pub(crate) fn endpoint(label: &str) -> Result<Self> {
        Ok(Self::DumbbellEndpoint {
            label: shared_text(label)?,
        })
    }

    pub(crate) fn supplemental(label: &str) -> Result<Self> {
        Ok(Self::SupplementalLayer {
            label: RawLayerLabel::new(label)?,
        })
    }

    pub(crate) fn primary_title(&self) -> Option<PrimaryTitleIdentity> {
        Some(match self {
            Self::Category { field, .. } => PrimaryTitleIdentity::Field(field.clone()),
            Self::Pair {
                group_field,
                hue_field,
                ..
            } => PrimaryTitleIdentity::Pair {
                group: group_field.clone(),
                hue: hue_field.clone(),
            },
            Self::BarSign { .. } => PrimaryTitleIdentity::BarSign,
            Self::DumbbellEndpoint { .. }
            | Self::RegressionObservation
            | Self::RegressionFit { .. } => PrimaryTitleIdentity::Series,
            Self::SupplementalLayer { .. } => return None,
        })
    }

    pub(crate) fn resolved_label(
        &self,
        plot: &Plot,
        layer_context: Option<super::source::LayerLabelContext>,
    ) -> Result<String> {
        let literal = crate::data::source::copy_source_string;
        match self {
            Self::Category { value, .. } => plot.resolve_label(value.as_str()),
            Self::Pair {
                group_value,
                hue_value,
                ..
            } => join(
                plot.resolve_label(group_value.as_str())?,
                ", ",
                &plot.resolve_label(hue_value.as_str())?,
            ),
            Self::BarSign { decreasing } => literal(if *decreasing {
                "Decreasing"
            } else {
                "Nondecreasing"
            }),
            Self::DumbbellEndpoint { label } => literal(label),
            Self::RegressionObservation => literal("Observations"),
            Self::RegressionFit { band } => literal(band.legend_label()),
            Self::SupplementalLayer { label } => match layer_context {
                Some(super::source::LayerLabelContext::Categorical) => {
                    plot.resolve_label(label.as_str())
                }
                Some(super::source::LayerLabelContext::LiteralSeries) => literal(label.as_str()),
                None => Err(Error::Render(
                    "supplemental guide requires its source label context".into(),
                )),
            },
        }
    }
}

fn join(mut first: String, separator: &str, second: &str) -> Result<String> {
    let additional = separator
        .len()
        .checked_add(second.len())
        .ok_or_else(|| Error::Render("guide label length overflow".into()))?;
    crate::data::source::reserve_source_string(&mut first, additional)?;
    first.push_str(separator);
    first.push_str(second);
    Ok(first)
}
