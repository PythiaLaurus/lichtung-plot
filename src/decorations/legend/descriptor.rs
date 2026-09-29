//! Font-independent shared continuous guide semantics. The containing frame
//! measures one assigned guide only after semantic compatibility is established.

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::{Error, Plot};
use std::sync::Arc;

pub(crate) type PreparedDiscreteGuide = (String, Arc<[LegendEntry]>);

/// The sole prepared Canvas guide descriptor. Placement and geometry are owned
/// by its containing frame and never selected during semantic preparation.
#[derive(Debug)]
pub(crate) enum PreparedGuideDescriptor {
    Discrete(PreparedDiscreteGuide),
    Continuous(ContinuousDescriptor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissingKey {
    pub fill: (u8, u8, u8),
    pub outline: (u8, u8, u8),
    pub label: String,
}

/// Only semantic palette stops are compared; the location of a diverging center
/// within a painted guide depends on the final merged numeric domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Palette {
    pub low: (u8, u8, u8),
    pub center: Option<(u8, u8, u8)>,
    pub high: (u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousSemantics {
    pub domain: (f64, f64),
    pub center: Option<f64>,
    pub palette: Palette,
    pub missing: Option<MissingKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousContributor {
    pub field: String,
    pub title: String,
    pub palette: Palette,
    pub center: Option<f64>,
    pub missing: Option<MissingKey>,
    pub domain: (f64, f64),
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousDescriptor {
    pub field: String,
    pub title: String,
    pub palette: Palette,
    pub center: Option<f64>,
    pub missing: Option<MissingKey>,
    pub domain: (f64, f64),
    pub exact: bool,
    pub labels: Vec<String>,
}

impl ContinuousContributor {
    pub(crate) fn for_plot(
        plot: &Plot,
        input: &crate::layout::interior::PreparedInteriorInputs,
        data: PlotData<'_>,
    ) -> Result<Option<Self>, Error> {
        let Some(field) = plot.color_dimension_key() else {
            return Ok(None);
        };
        let field = data.copy_source_string(field)?;
        let (title, semantics, exact) = match plot.kind() {
            crate::PlotKind::Heatmap => {
                let marks = input
                    .heatmap
                    .as_ref()
                    .and_then(|inputs| inputs.panels.first())
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| {
                        Error::Render("shared Heatmap guide has no prepared input".into())
                    })?;
                let semantics = marks.guide_semantics(data)?;
                (
                    data.copy_source_string(&marks.color_label)?,
                    semantics,
                    plot.heatmap_color_limits_spec().is_some(),
                )
            }
            crate::PlotKind::Beeswarm => {
                let marks = input
                    .beeswarm
                    .as_ref()
                    .and_then(|inputs| inputs.panels.first())
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| {
                        Error::Render("shared Beeswarm guide has no prepared input".into())
                    })?;
                let semantics = marks.guide_semantics();
                (
                    data.copy_source_string(&marks.color_label)?,
                    semantics,
                    plot.beeswarm_color_limits_spec().is_some(),
                )
            }
            _ => return Ok(None),
        };
        if ![semantics.domain.0, semantics.domain.1]
            .into_iter()
            .all(f64::is_finite)
            || semantics.domain.0 > semantics.domain.1
            || exact && semantics.domain.0.total_cmp(&semantics.domain.1).is_eq()
        {
            return Err(Error::Data(
                "continuous guide domain is not finite and ordered".into(),
            ));
        }
        Ok(Some(Self {
            field,
            title,
            palette: semantics.palette,
            center: semantics.center,
            missing: semantics.missing,
            domain: semantics.domain,
            exact,
        }))
    }

    pub(crate) fn merge(
        contributors: impl IntoIterator<Item = Self>,
        data: PlotData<'_>,
    ) -> Result<Option<ContinuousDescriptor>, Error> {
        let mut combined: Option<ContinuousDescriptor> = None;
        let mut automatic = None;
        for contributor in contributors {
            data.claim_source_visits(1)?;
            if let Some(existing) = &mut combined {
                let same_center = match (existing.center, contributor.center) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a.to_bits() == b.to_bits(),
                    _ => false,
                };
                if !same_text(&existing.field, &contributor.field, data)?
                    || !same_text(&existing.title, &contributor.title, data)?
                    || existing.palette != contributor.palette
                    || !same_center
                    || existing.missing != contributor.missing
                {
                    return Err(Error::Data(
                        "shared continuous guide contributors have incompatible identity, title, scale, or missing block".into(),
                    ));
                }
                if contributor.exact {
                    if existing.exact && existing.domain != contributor.domain {
                        return Err(Error::Data(
                            "shared continuous guide exact domains disagree".into(),
                        ));
                    }
                    existing.domain = contributor.domain;
                    existing.exact = true;
                } else {
                    automatic = Some(automatic.map_or(
                        contributor.domain,
                        |(low, high): (f64, f64)| {
                            (
                                low.min(contributor.domain.0),
                                high.max(contributor.domain.1),
                            )
                        },
                    ));
                    if !existing.exact {
                        existing.domain = (
                            existing.domain.0.min(contributor.domain.0),
                            existing.domain.1.max(contributor.domain.1),
                        );
                    }
                }
            } else {
                if !contributor.exact {
                    automatic = Some(contributor.domain);
                }
                combined = Some(ContinuousDescriptor {
                    field: contributor.field,
                    title: contributor.title,
                    palette: contributor.palette,
                    center: contributor.center,
                    missing: contributor.missing,
                    domain: contributor.domain,
                    exact: contributor.exact,
                    labels: Vec::new(),
                });
            }
        }
        if let (Some(existing), Some((low, high))) = (&combined, automatic) {
            if existing.exact && (low < existing.domain.0 || high > existing.domain.1) {
                return Err(Error::Data(
                    "shared continuous guide exact domain excludes an automatic contributor".into(),
                ));
            }
        }
        if let Some(existing) = &mut combined {
            if existing.center.is_none()
                && existing.domain.0.total_cmp(&existing.domain.1).is_eq()
                && existing.palette.low != existing.palette.high
            {
                let midpoint = crate::theme::interpolate_data_color(
                    existing.palette.low,
                    existing.palette.high,
                    0.5,
                    data.source_resources()?,
                )?;
                existing.palette.low = midpoint;
                existing.palette.high = midpoint;
            }
            existing.labels = generated_labels(existing, data)?;
        }
        Ok(combined)
    }
}

fn same_text(left: &str, right: &str, data: PlotData<'_>) -> Result<bool, Error> {
    if left.len() != right.len() {
        data.claim_source_visits(1)?;
        return Ok(false);
    }
    for (left, right) in left.bytes().zip(right.bytes()) {
        data.claim_source_visits(1)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

fn generated_labels(
    descriptor: &ContinuousDescriptor,
    data: PlotData<'_>,
) -> Result<Vec<String>, Error> {
    let mut values = [0.0_f64; 3];
    let count = if descriptor.domain.0.total_cmp(&descriptor.domain.1).is_eq() {
        values[0] = descriptor.domain.0;
        1
    } else {
        values[0] = descriptor.domain.0;
        let mut count = 1;
        if let Some(center) = descriptor
            .center
            .filter(|center| *center > descriptor.domain.0 && *center < descriptor.domain.1)
        {
            values[1] = center;
            count += 1;
        }
        values[count] = descriptor.domain.1;
        count + 1
    };
    for precision in std::iter::once(0).chain(2..=17) {
        let mut labels: Vec<String> = Vec::new();
        data.reserve_source(&mut labels, count)?;
        let mut unique = true;
        for value in values[..count].iter().copied() {
            data.claim_source_visits(1)?;
            let label = crate::layout::format_guide_label(value, precision, data)?;
            for existing in &labels {
                if same_text(existing, &label, data)? {
                    unique = false;
                    break;
                }
            }
            labels.push(label);
        }
        if unique {
            return Ok(labels);
        }
    }
    Err(Error::Data(
        "shared continuous guide labels are not unique at supported numeric precisions".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;

    fn contributor(domain: (f64, f64)) -> ContinuousContributor {
        ContinuousContributor {
            field: "measurement".into(),
            title: "Measurement".into(),
            palette: Palette {
                low: (0, 0, 0),
                center: Some((127, 127, 127)),
                high: (255, 255, 255),
            },
            center: Some(0.0),
            missing: None,
            domain,
            exact: false,
        }
    }

    #[test]
    fn merged_domain_precedes_unique_numeric_labels_and_rejects_missing_conflicts() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        let merged =
            ContinuousContributor::merge([contributor((-2.0, 0.0)), contributor((0.0, 3.0))], data)
                .unwrap()
                .unwrap();
        assert_eq!(merged.domain, (-2.0, 3.0));
        assert_eq!(merged.labels, ["-2", "0", "3"]);
        let constant = ContinuousContributor::merge([contributor((1.0, 1.0))], data)
            .unwrap()
            .unwrap();
        assert_eq!(constant.labels, ["1"]);
        let mut missing = contributor((0.0, 1.0));
        missing.missing = Some(MissingKey {
            fill: (200, 200, 200),
            outline: (50, 50, 50),
            label: "Missing".into(),
        });
        assert!(matches!(
            ContinuousContributor::merge([contributor((0.0, 1.0)), missing], data),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
    }

    #[test]
    fn close_endpoints_retry_a_complete_unique_format_under_one_budget() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let close = (1.0, f64::from_bits(1.0_f64.to_bits() + 1));
        let run = |limit: Option<(u128, u64)>| {
            let resources = OperationGenerationResources::new();
            if let Some((visits, bytes)) = limit {
                resources.set_test_limits(visits, bytes);
            }
            let outcome = ContinuousContributor::merge(
                [contributor(close)],
                plot.data().with_axis_resources(&resources),
            );
            (resources, outcome)
        };
        let (resources, result) = run(None);
        let result = result.unwrap().unwrap();
        assert_eq!(result.labels.len(), 2);
        assert_ne!(result.labels[0], result.labels[1]);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        assert!(visits > 0 && bytes > 0);
        assert!(run(Some((visits - 1, bytes))).1.is_err());
        assert!(run(Some((visits, bytes - 1))).1.is_err());
        assert_eq!(run(Some((visits, bytes))).1.unwrap().unwrap(), result);
    }
}
