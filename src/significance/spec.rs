//! Public significance display values.

use std::marker::PhantomData;

use crate::Error;

/// Exact significance text or the p-value used to derive it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignificanceMark {
    /// Derive `ns` or one through four stars from this exact display p-value.
    PValue(f64),
    /// Render `ns`.
    NotSignificant,
    /// Render `*`.
    OneStar,
    /// Render `**`.
    TwoStars,
    /// Render `***`.
    ThreeStars,
    /// Render `****`.
    FourStars,
}

impl SignificanceMark {
    pub(crate) fn glyph(self) -> Result<&'static str, Error> {
        match self {
            Self::PValue(value) if !value.is_finite() || !(0.0..=1.0).contains(&value) => {
                Err(Error::Data(format!(
                    "significance p-value must be finite and in [0, 1]; got {value}"
                )))
            }
            Self::PValue(value) if value >= 0.05 => Ok("ns"),
            Self::PValue(value) if value < 0.0001 => Ok("****"),
            Self::PValue(value) if value < 0.001 => Ok("***"),
            Self::PValue(value) if value < 0.01 => Ok("**"),
            Self::PValue(_) | Self::OneStar => Ok("*"),
            Self::NotSignificant => Ok("ns"),
            Self::TwoStars => Ok("**"),
            Self::ThreeStars => Ok("***"),
            Self::FourStars => Ok("****"),
        }
    }
}

/// One rendered categorical target for a significance annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignificanceTarget {
    /// A complete categorical axis level or category cluster.
    Category(String),
    /// One positioned series inside a category.
    CategorySeries {
        /// Categorical axis level.
        category: String,
        /// Series level configured through [`crate::Plot::group`] or
        /// [`crate::Plot::hue`].
        series: String,
    },
}

impl SignificanceTarget {
    /// Target a complete categorical axis level or category cluster.
    #[must_use]
    pub fn category(value: impl Into<String>) -> Self {
        Self::Category(value.into())
    }

    /// Target one positioned series inside a category.
    #[must_use]
    pub fn category_series(category: impl Into<String>, series: impl Into<String>) -> Self {
        Self::CategorySeries {
            category: category.into(),
            series: series.into(),
        }
    }

    pub(crate) fn category_name(&self) -> &str {
        match self {
            Self::Category(category) | Self::CategorySeries { category, .. } => category,
        }
    }

    pub(crate) fn series_name(&self) -> Option<&str> {
        match self {
            Self::Category(_) => None,
            Self::CategorySeries { series, .. } => Some(series),
        }
    }
}

impl From<String> for SignificanceTarget {
    fn from(value: String) -> Self {
        Self::Category(value)
    }
}

impl From<&str> for SignificanceTarget {
    fn from(value: &str) -> Self {
        Self::Category(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SignificanceFacet {
    variable: String,
    level: String,
}

/// Marker for a significance mark attached to one rendered target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attached;

/// Marker for a pairwise significance bracket between two rendered targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pairwise;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SignificanceAnnotation {
    Attached {
        target: SignificanceTarget,
        mark: SignificanceMark,
        facets: Vec<SignificanceFacet>,
    },
    Pairwise {
        endpoint_a: SignificanceTarget,
        endpoint_b: SignificanceTarget,
        mark: SignificanceMark,
        facets: Vec<SignificanceFacet>,
    },
}

/// One caller-owned significance annotation with a compile-time placement.
#[derive(Debug, Clone, PartialEq)]
pub struct Significance<P> {
    annotation: SignificanceAnnotation,
    placement: PhantomData<P>,
}

impl Significance<Attached> {
    /// Place one significance mark at a categorical target.
    #[must_use]
    pub fn at(target: impl Into<SignificanceTarget>, mark: SignificanceMark) -> Self {
        Self {
            annotation: SignificanceAnnotation::Attached {
                target: target.into(),
                mark,
                facets: Vec::new(),
            },
            placement: PhantomData,
        }
    }
}

impl Significance<Pairwise> {
    /// Draw one pairwise bracket between two categorical targets.
    #[must_use]
    pub fn between(
        endpoint_a: impl Into<SignificanceTarget>,
        endpoint_b: impl Into<SignificanceTarget>,
        mark: SignificanceMark,
    ) -> Self {
        Self {
            annotation: SignificanceAnnotation::Pairwise {
                endpoint_a: endpoint_a.into(),
                endpoint_b: endpoint_b.into(),
                mark,
                facets: Vec::new(),
            },
            placement: PhantomData,
        }
    }
}

impl<P> Significance<P> {
    /// Restrict this annotation to one level of a configured facet dimension.
    #[must_use]
    pub fn facet(mut self, variable: impl Into<String>, level: impl Into<String>) -> Self {
        let facet = SignificanceFacet {
            variable: variable.into(),
            level: level.into(),
        };
        let facets = self.annotation.facets_mut();
        if !facets.contains(&facet) {
            facets.push(facet);
        }
        self
    }

    pub(crate) fn into_annotation(self) -> SignificanceAnnotation {
        self.annotation
    }
}

impl SignificanceAnnotation {
    pub(crate) fn pairwise(
        endpoint_a: impl Into<SignificanceTarget>,
        endpoint_b: impl Into<SignificanceTarget>,
        mark: SignificanceMark,
    ) -> Self {
        Self::Pairwise {
            endpoint_a: endpoint_a.into(),
            endpoint_b: endpoint_b.into(),
            mark,
            facets: Vec::new(),
        }
    }

    pub(crate) const fn mark(&self) -> SignificanceMark {
        match self {
            Self::Attached { mark, .. } | Self::Pairwise { mark, .. } => *mark,
        }
    }

    pub(crate) fn at_target(&self) -> Option<&SignificanceTarget> {
        match self {
            Self::Attached { target, .. } => Some(target),
            Self::Pairwise { .. } => None,
        }
    }

    pub(crate) fn endpoints(&self) -> Option<(&SignificanceTarget, &SignificanceTarget)> {
        match self {
            Self::Attached { .. } => None,
            Self::Pairwise {
                endpoint_a,
                endpoint_b,
                ..
            } => Some((endpoint_a, endpoint_b)),
        }
    }

    pub(crate) fn facet_ownership(&self) -> impl Iterator<Item = (&str, &str)> {
        self.facets()
            .iter()
            .map(|facet| (facet.variable.as_str(), facet.level.as_str()))
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        self.mark().glyph()?;
        match self {
            Self::Attached { target, .. } => validate_target(target)?,
            Self::Pairwise {
                endpoint_a,
                endpoint_b,
                ..
            } => {
                validate_target(endpoint_a)?;
                validate_target(endpoint_b)?;
                if endpoint_a == endpoint_b {
                    return Err(Error::Data(
                        "significance comparison endpoints must be different".into(),
                    ));
                }
            }
        }
        let facets = self.facets();
        for (index, facet) in facets.iter().enumerate() {
            validate_text("facet variable", &facet.variable)?;
            validate_text("facet level", &facet.level)?;
            if facets[..index]
                .iter()
                .any(|previous| previous.variable == facet.variable)
            {
                return Err(Error::Data(format!(
                    "significance annotation names facet variable {:?} more than once",
                    facet.variable
                )));
            }
        }
        Ok(())
    }

    fn facets(&self) -> &[SignificanceFacet] {
        match self {
            Self::Attached { facets, .. } | Self::Pairwise { facets, .. } => facets,
        }
    }

    fn facets_mut(&mut self) -> &mut Vec<SignificanceFacet> {
        match self {
            Self::Attached { facets, .. } | Self::Pairwise { facets, .. } => facets,
        }
    }
}

fn validate_target(target: &SignificanceTarget) -> Result<(), Error> {
    validate_text("target category", target.category_name())?;
    if let Some(series) = target.series_name() {
        validate_text("target series", series)?;
    }
    Ok(())
}

fn validate_text(role: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "significance {role} must be nonempty, trimmed, and free of control characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_attached(_: Significance<Attached>) {}
    fn assert_pairwise(_: Significance<Pairwise>) {}

    #[test]
    fn constructors_return_exact_placement_types() {
        assert_attached(Significance::at("a", SignificanceMark::OneStar));
        assert_pairwise(Significance::between("a", "b", SignificanceMark::TwoStars));
    }

    #[test]
    fn facet_preserves_each_placement_type() {
        assert_attached(Significance::at("a", SignificanceMark::OneStar).facet("panel", "one"));
        assert_pairwise(
            Significance::between("a", "b", SignificanceMark::TwoStars).facet("panel", "one"),
        );
    }
}
