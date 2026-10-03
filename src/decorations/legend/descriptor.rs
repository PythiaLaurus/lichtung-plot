//! Font-independent shared continuous guide semantics. The containing frame
//! measures one assigned guide only after semantic compatibility is established.

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::render::backend::{GradientStop, Rgba};
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MissingKey {
    pub fill: Rgba,
    pub outline: crate::theme::FillOutlineAppearance,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousSemantics {
    pub domain: (f64, f64),
    pub center: Option<f64>,
    /// Semantic stop positions precede final-domain center projection.
    pub stops: Vec<GradientStop>,
    pub missing: Option<MissingKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousContributor {
    pub field: String,
    pub title: String,
    pub stops: Vec<GradientStop>,
    pub samples: Vec<f64>,
    pub center: Option<f64>,
    pub missing: Option<MissingKey>,
    pub domain: (f64, f64),
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousDescriptor {
    pub field: String,
    pub title: String,
    /// Complete final-domain gradient, retained before guide measurement.
    pub stops: Vec<GradientStop>,
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
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Option<Self>, Error> {
        let Some(field) = plot.color_dimension_key() else {
            return Ok(None);
        };
        let field = data.copy_source_string(field)?;
        let (title, semantics, samples, exact) = match plot.kind() {
            crate::PlotKind::Heatmap => {
                let marks = input
                    .heatmap
                    .as_ref()
                    .and_then(|inputs| inputs.panels.first())
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| {
                        Error::Render("shared Heatmap guide has no prepared input".into())
                    })?;
                let semantics = marks.guide_semantics(theme, data)?;
                (
                    data.copy_source_string(&marks.color_label)?,
                    semantics,
                    marks.guide_samples(data)?,
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
                let semantics = marks.guide_semantics(data)?;
                (
                    data.copy_source_string(&marks.color_label)?,
                    semantics,
                    marks.guide_samples(data)?,
                    plot.beeswarm_color_limits_spec().is_some(),
                )
            }
            _ => return Ok(None),
        };
        if ![semantics.domain.0, semantics.domain.1]
            .into_iter()
            .all(f64::is_finite)
            || semantics.domain.0 > semantics.domain.1
        {
            return Err(Error::Data(
                "continuous guide domain is not finite and ordered".into(),
            ));
        }
        Ok(Some(Self {
            field,
            title,
            stops: semantics.stops,
            samples,
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
        let mut samples = Vec::new();
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
                    || !same_stops(&existing.stops, &contributor.stops, data)?
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
                    stops: contributor.stops,
                    center: contributor.center,
                    missing: contributor.missing,
                    domain: contributor.domain,
                    exact: contributor.exact,
                    labels: Vec::new(),
                });
            }
            data.reserve_source(&mut samples, contributor.samples.len())?;
            samples.extend_from_slice(&contributor.samples);
        }
        if let (Some(existing), Some((low, high))) = (&combined, automatic) {
            if existing.exact && (low < existing.domain.0 || high > existing.domain.1) {
                return Err(Error::Data(
                    "shared continuous guide exact domain excludes an automatic contributor".into(),
                ));
            }
        }
        if let Some(existing) = &mut combined {
            existing.stops = final_stops(
                existing.domain,
                existing.center,
                &existing.stops,
                &samples,
                data,
            )?;
            existing.labels = generated_labels(existing, data)?;
        }
        Ok(combined)
    }
}

fn same_stops(
    left: &[GradientStop],
    right: &[GradientStop],
    data: PlotData<'_>,
) -> Result<bool, Error> {
    data.claim_source_visits(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        data.claim_source_visits(1)?;
        if left.offset.to_bits() != right.offset.to_bits() || left.color != right.color {
            return Ok(false);
        }
    }
    Ok(true)
}

fn final_stops(
    domain: (f64, f64),
    center: Option<f64>,
    semantic: &[GradientStop],
    samples: &[f64],
    data: PlotData<'_>,
) -> Result<Vec<GradientStop>, Error> {
    let resources = data.source_resources()?;
    if let Some(center) = center {
        // Validate the supplied semantic gradient before projecting its center
        // into the selected domain; never repair malformed endpoint offsets.
        let _ = crate::theme::ContinuousColors::resolve((0.0, 1.0), semantic, resources)?;
        if !center.is_finite() || center < domain.0 || center > domain.1 {
            return Err(Error::Data(
                "shared diverging color domain excludes its center".into(),
            ));
        }
        if semantic.len() != 3 || semantic[1].offset.to_bits() != 0.5_f64.to_bits() {
            return Err(Error::Data(
                "shared diverging gradient requires one neutral center stop".into(),
            ));
        }
        let neutral = semantic[1].color;
        let selected = if same_value(domain.0, domain.1) {
            [
                GradientStop::new(0.0, neutral),
                GradientStop::new(1.0, neutral),
                GradientStop::new(1.0, neutral),
            ]
        } else if same_value(center, domain.0) {
            [
                GradientStop::new(0.0, neutral),
                GradientStop::new(1.0, semantic[2].color),
                GradientStop::new(1.0, semantic[2].color),
            ]
        } else if same_value(center, domain.1) {
            [
                GradientStop::new(0.0, semantic[0].color),
                GradientStop::new(1.0, neutral),
                GradientStop::new(1.0, neutral),
            ]
        } else {
            let offset = crate::theme::ContinuousColors::normalized_fraction(domain, center)?;
            if !(0.0..1.0).contains(&offset) || offset.to_bits() == 0 {
                return Err(Error::Data(
                    "shared diverging center has no representable gradient stop".into(),
                ));
            }
            [
                GradientStop::new(0.0, semantic[0].color),
                GradientStop::new(offset, neutral),
                GradientStop::new(1.0, semantic[2].color),
            ]
        };
        let count = if center > domain.0 && center < domain.1 {
            3
        } else {
            2
        };
        return crate::theme::ContinuousColors::resolve(domain, &selected[..count], resources)?
            .guide_stops(samples, data);
    }
    if same_value(domain.0, domain.1) {
        let scale = crate::theme::ContinuousColors::resolve((0.0, 1.0), semantic, resources)?;
        let midpoint = scale.color(0.5, resources)?;
        let stops = [
            GradientStop::new(0.0, midpoint),
            GradientStop::new(1.0, midpoint),
        ];
        return crate::theme::ContinuousColors::resolve(domain, &stops, resources)?
            .guide_stops(samples, data);
    }
    crate::theme::ContinuousColors::resolve(domain, semantic, resources)?.guide_stops(samples, data)
}

pub(crate) fn same_value(left: f64, right: f64) -> bool {
    left.total_cmp(&right).is_eq() || (left.abs().to_bits() == 0 && right.abs().to_bits() == 0)
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
    let count = if same_value(descriptor.domain.0, descriptor.domain.1) {
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
    numeric_labels(&values[..count], data)
}

/// One pre-font numeric grammar and precision search for every continuous guide.
/// Equal scientific values (including signed zero) may repeat in a family's
/// retained census; distinct scientific values must have distinct labels.
pub(crate) fn numeric_labels(values: &[f64], data: PlotData<'_>) -> Result<Vec<String>, Error> {
    data.claim_source_visits(values.len())?;
    if !matches!(values.len(), 1..=3) || values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Data(
            "continuous guide needs one to three finite numeric values".into(),
        ));
    }
    for precision in std::iter::once(0).chain(2..=17) {
        let mut labels: Vec<String> = Vec::new();
        data.reserve_source(&mut labels, values.len())?;
        let mut unique = true;
        for (index, value) in values.iter().copied().enumerate() {
            data.claim_source_visits(1)?;
            let label = crate::layout::format_guide_label(value, precision, data)?;
            for (previous, existing) in labels.iter().enumerate() {
                if !same_value(values[previous], values[index])
                    && same_text(existing, &label, data)?
                {
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
        "continuous guide labels are not unique at supported numeric precisions".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;

    #[test]
    fn numeric_guide_precision_is_unique_in_every_order_and_normalizes_signed_zero() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let data = source.data().with_axis_resources(&resources);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        for values in [
            [0.1, 0.1001],
            [0.1, 0.100_000_1],
            [1.0, f64::from_bits(1.0_f64.to_bits() + 1)],
        ] {
            let labels = numeric_labels(&values, data).unwrap();
            assert_ne!(labels[0], labels[1]);
            assert_eq!(
                numeric_labels(&[values[1], values[0]], data).unwrap(),
                [labels[1].clone(), labels[0].clone()]
            );
        }
        assert_eq!(numeric_labels(&[-0.0, 0.0], data).unwrap(), ["0", "0"]);
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    fn contributor(domain: (f64, f64)) -> ContinuousContributor {
        ContinuousContributor {
            field: "measurement".into(),
            title: "Measurement".into(),
            stops: vec![
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(0.5, (127, 127, 127)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
            samples: Vec::new(),
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
        let mut constant = contributor((1.0, 1.0));
        constant.center = Some(1.0);
        let constant = ContinuousContributor::merge([constant], data)
            .unwrap()
            .unwrap();
        assert_eq!(constant.labels, ["1"]);
        let mut missing = contributor((0.0, 1.0));
        missing.missing = Some(MissingKey {
            fill: (200, 200, 200).into(),
            outline: crate::theme::FillOutlineAppearance {
                paint: (50, 50, 50).into(),
                width: 1.0,
            },
            label: "Missing".into(),
        });
        assert!(matches!(
            ContinuousContributor::merge([contributor((0.0, 1.0)), missing.clone()], data),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        let mut different_alpha = missing.clone();
        different_alpha.missing.as_mut().unwrap().fill = Rgba::new(200, 200, 200, 127);
        let mut different_width = missing.clone();
        different_width.missing.as_mut().unwrap().outline.width = 3.0;
        assert!(matches!(
            ContinuousContributor::merge([missing.clone(), different_width], data),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            ContinuousContributor::merge([missing, different_alpha], data),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        let mut malformed = contributor((-1.0, 1.0));
        malformed.stops[0].offset = 0.1;
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            ContinuousContributor::merge([malformed], data),
            Err(Error::Data(message)) if message.contains("span exactly 0..1")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn ordered_sequential_stops_are_retained_and_mismatched_colors_reject_before_fonts() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        let mut first = contributor((0.0, 2.0));
        first.center = None;
        first.stops = vec![
            GradientStop::new(0.0, (0, 0, 0)),
            GradientStop::new(0.3, (30, 40, 50)),
            GradientStop::new(1.0, (255, 255, 255)),
        ];
        let mut second = first.clone();
        second.domain = (1.0, 3.0);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let merged = ContinuousContributor::merge([first.clone(), second.clone()], data)
            .unwrap()
            .unwrap();
        assert_eq!(merged.domain, (0.0, 3.0));
        assert_eq!(merged.stops, first.stops);
        second.stops[1].color = (31, 40, 50);
        assert!(matches!(
            ContinuousContributor::merge([first, second], data),
            Err(Error::Data(message)) if message.contains("incompatible identity")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
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
            let mut candidate = contributor(close);
            candidate.center = None;
            candidate.stops.remove(1);
            let outcome = ContinuousContributor::merge(
                [candidate],
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
