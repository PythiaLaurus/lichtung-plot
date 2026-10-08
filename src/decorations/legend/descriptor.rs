//! Font-independent shared continuous guide semantics. The containing frame
//! measures one assigned guide only after semantic compatibility is established.

use crate::decorations::legend::LegendEntry;
use crate::render::backend::GradientStop;
use crate::theme::{Color, ColorStop, ContinuousColorScale};
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
    pub fill: Color,
    pub outline: crate::theme::OutlineTheme,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousSemantics {
    pub domain: (f64, f64),
    pub selection: ContinuousColorScale,
    pub missing: Option<MissingKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousContributor {
    pub field: String,
    pub title: String,
    pub selection: ContinuousColorScale,
    pub samples: Vec<f64>,
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
    pub selection: ContinuousColorScale,
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
    ) -> Result<Option<Self>, Error> {
        let Some(field) = plot.color_dimension_key() else {
            return Ok(None);
        };
        let field = crate::data::source::copy_source_string(field)?;
        let (title, semantics, samples) = match plot.kind() {
            crate::PlotKind::Heatmap => {
                let marks = input
                    .heatmap
                    .as_ref()
                    .and_then(|inputs| inputs.panels.first())
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| {
                        Error::Render("shared Heatmap guide has no prepared input".into())
                    })?;
                let semantics = marks.guide_semantics(theme)?;
                (
                    crate::data::source::copy_source_string(&marks.color_label)?,
                    semantics,
                    marks.guide_samples()?,
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
                let semantics = marks.continuous_semantics();
                (
                    crate::data::source::copy_source_string(&marks.color_label)?,
                    semantics,
                    marks.guide_samples()?,
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
        let exact = semantics.selection.domain().is_some();
        Ok(Some(Self {
            field,
            title,
            selection: semantics.selection,
            samples,
            missing: semantics.missing,
            domain: semantics.domain,
            exact,
        }))
    }

    pub(crate) fn merge(
        contributors: impl IntoIterator<Item = Self>,
    ) -> Result<Option<ContinuousDescriptor>, Error> {
        let mut combined: Option<ContinuousDescriptor> = None;
        let mut automatic = None;
        let mut samples = Vec::new();
        for contributor in contributors {
            if let Some(existing) = &mut combined {
                let same_center = match (existing.center, contributor.selection.center()) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a.to_bits() == b.to_bits(),
                    _ => false,
                };
                if !same_text(&existing.field, &contributor.field)
                    || !same_text(&existing.title, &contributor.title)
                    || existing.selection.kind() != contributor.selection.kind()
                    || !same_stops(existing.selection.stops()?, contributor.selection.stops()?)
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
                    center: contributor.selection.center(),
                    selection: contributor.selection,
                    stops: Vec::new(),
                    missing: contributor.missing,
                    domain: contributor.domain,
                    exact: contributor.exact,
                    labels: Vec::new(),
                });
            }
            crate::data::source::reserve_source(&mut samples, contributor.samples.len())?;
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
            existing.selection = existing.selection.clone().with_domain(
                existing
                    .exact
                    .then(|| crate::theme::ColorDomain::exact(existing.domain.0, existing.domain.1))
                    .transpose()?,
            )?;
            existing.stops = final_stops(
                existing.domain,
                existing.center,
                existing.selection.stops()?,
                &samples,
            )?;
            existing.labels = generated_labels(existing)?;
        }
        Ok(combined)
    }
}

fn same_stops(left: &[ColorStop], right: &[ColorStop]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (left, right) in left.iter().zip(right) {
        if left.offset().to_bits() != right.offset().to_bits() || left.color() != right.color() {
            return false;
        }
    }
    true
}

fn final_stops(
    domain: (f64, f64),
    center: Option<f64>,
    semantic: &[ColorStop],
    samples: &[f64],
) -> Result<Vec<GradientStop>, Error> {
    if let Some(center) = center {
        // Validate the supplied semantic gradient before projecting its center
        // into the selected domain; never repair malformed endpoint offsets.
        let _ = crate::theme::ContinuousColors::resolve((0.0, 1.0), semantic)?;
        if !center.is_finite() || center < domain.0 || center > domain.1 {
            return Err(Error::Data(
                "shared diverging color domain excludes its center".into(),
            ));
        }
        if semantic.len() != 3 || semantic[1].offset().to_bits() != 0.5_f64.to_bits() {
            return Err(Error::Data(
                "shared diverging gradient requires one neutral center stop".into(),
            ));
        }
        let neutral = semantic[1].color();
        let selected = if same_value(domain.0, domain.1) {
            [
                ColorStop::new(0.0, neutral)?,
                ColorStop::new(1.0, neutral)?,
                ColorStop::new(1.0, neutral)?,
            ]
        } else if same_value(center, domain.0) {
            [
                ColorStop::new(0.0, neutral)?,
                ColorStop::new(1.0, semantic[2].color())?,
                ColorStop::new(1.0, semantic[2].color())?,
            ]
        } else if same_value(center, domain.1) {
            [
                ColorStop::new(0.0, semantic[0].color())?,
                ColorStop::new(1.0, neutral)?,
                ColorStop::new(1.0, neutral)?,
            ]
        } else {
            let offset = crate::theme::ContinuousColors::normalized_fraction(domain, center)?;
            if !(0.0..1.0).contains(&offset) || offset.to_bits() == 0 {
                return Err(Error::Data(
                    "shared diverging center has no representable gradient stop".into(),
                ));
            }
            [
                ColorStop::new(0.0, semantic[0].color())?,
                ColorStop::new(offset, neutral)?,
                ColorStop::new(1.0, semantic[2].color())?,
            ]
        };
        let count = if center > domain.0 && center < domain.1 {
            3
        } else {
            2
        };
        return crate::theme::ContinuousColors::resolve(domain, &selected[..count])?
            .guide_stops(samples);
    }
    if same_value(domain.0, domain.1) {
        let scale = crate::theme::ContinuousColors::resolve((0.0, 1.0), semantic)?;
        let midpoint = scale.color(0.5)?;
        let stops = [
            ColorStop::new(0.0, midpoint)?,
            ColorStop::new(1.0, midpoint)?,
        ];
        return crate::theme::ContinuousColors::resolve(domain, &stops)?.guide_stops(samples);
    }
    crate::theme::ContinuousColors::resolve(domain, semantic)?.guide_stops(samples)
}

pub(crate) fn same_value(left: f64, right: f64) -> bool {
    left.total_cmp(&right).is_eq() || (left.abs().to_bits() == 0 && right.abs().to_bits() == 0)
}

fn same_text(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (left, right) in left.bytes().zip(right.bytes()) {
        if left != right {
            return false;
        }
    }
    true
}

fn generated_labels(descriptor: &ContinuousDescriptor) -> Result<Vec<String>, Error> {
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
    numeric_labels(&values[..count])
}

/// One pre-font numeric grammar and precision search for every continuous guide.
/// Equal scientific values (including signed zero) may repeat in a family's
/// retained census; distinct scientific values must have distinct labels.
pub(crate) fn numeric_labels(values: &[f64]) -> Result<Vec<String>, Error> {
    if !matches!(values.len(), 1..=3) || values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Data(
            "continuous guide needs one to three finite numeric values".into(),
        ));
    }
    for precision in std::iter::once(0).chain(2..=17) {
        let mut labels: Vec<String> = Vec::new();
        crate::data::source::reserve_source(&mut labels, values.len())?;
        let mut unique = true;
        for (index, value) in values.iter().copied().enumerate() {
            let label = crate::layout::format_guide_label(value, precision)?;
            for (previous, existing) in labels.iter().enumerate() {
                if !same_value(values[previous], values[index]) && same_text(existing, &label) {
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

    #[test]
    fn numeric_guide_precision_is_unique_in_every_order_and_normalizes_signed_zero() {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        for values in [
            [0.1, 0.1001],
            [0.1, 0.100_000_1],
            [1.0, f64::from_bits(1.0_f64.to_bits() + 1)],
        ] {
            let labels = numeric_labels(&values).unwrap();
            assert_ne!(labels[0], labels[1]);
            assert_eq!(
                numeric_labels(&[values[1], values[0]]).unwrap(),
                [labels[1].clone(), labels[0].clone()]
            );
        }
        assert_eq!(numeric_labels(&[-0.0, 0.0]).unwrap(), ["0", "0"]);
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    fn contributor(domain: (f64, f64)) -> ContinuousContributor {
        ContinuousContributor {
            field: "measurement".into(),
            title: "Measurement".into(),
            selection: ContinuousColorScale::diverging(
                0.0,
                Some(
                    crate::theme::DivergingGradient::new([
                        ColorStop::new(0.0, crate::theme::OpaqueColor::rgb(0, 0, 0)).unwrap(),
                        ColorStop::new(0.5, crate::theme::OpaqueColor::rgb(127, 127, 127)).unwrap(),
                        ColorStop::new(1.0, crate::theme::OpaqueColor::rgb(255, 255, 255)).unwrap(),
                    ])
                    .unwrap(),
                ),
                None,
            )
            .unwrap(),
            samples: Vec::new(),
            missing: None,
            domain,
            exact: false,
        }
    }

    #[test]
    fn merged_domain_precedes_unique_numeric_labels_and_rejects_missing_conflicts() {
        let merged =
            ContinuousContributor::merge([contributor((-2.0, 0.0)), contributor((0.0, 3.0))])
                .unwrap()
                .unwrap();
        assert_eq!(merged.domain, (-2.0, 3.0));
        assert_eq!(merged.labels, ["-2", "0", "3"]);
        let mut constant = contributor((1.0, 1.0));
        constant.selection = constant.selection.with_center(1.0).unwrap();
        let constant = ContinuousContributor::merge([constant]).unwrap().unwrap();
        assert_eq!(constant.labels, ["1"]);
        let mut missing = contributor((0.0, 1.0));
        missing.missing = Some(MissingKey {
            fill: Color::rgb(200, 200, 200),
            outline: crate::theme::OutlineTheme {
                paint: Color::rgb(50, 50, 50),
                width: 1.0,
            },
            label: "Missing".into(),
        });
        assert!(matches!(
            ContinuousContributor::merge([contributor((0.0, 1.0)), missing.clone()]),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        let mut different_alpha = missing.clone();
        different_alpha.missing.as_mut().unwrap().fill = Color::rgba(200, 200, 200, 127);
        let mut different_width = missing.clone();
        different_width.missing.as_mut().unwrap().outline.width = 3.0;
        assert!(matches!(
            ContinuousContributor::merge([missing.clone(), different_width]),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            ContinuousContributor::merge([missing, different_alpha]),
            Err(Error::Data(message)) if message.contains("missing block")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(crate::theme::DivergingGradient::new([
            ColorStop::new(0.1, crate::theme::OpaqueColor::rgb(0, 0, 0)).unwrap(),
            ColorStop::new(0.5, crate::theme::OpaqueColor::rgb(127, 127, 127)).unwrap(),
            ColorStop::new(1.0, crate::theme::OpaqueColor::rgb(255, 255, 255)).unwrap(),
        ])
        .is_err());
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn ordered_sequential_stops_are_retained_and_mismatched_colors_reject_before_fonts() {
        let mut first = contributor((0.0, 2.0));
        first.selection = ContinuousColorScale::sequential(
            Some(
                crate::theme::SequentialGradient::new([
                    ColorStop::new(0.0, crate::theme::OpaqueColor::rgb(0, 0, 0)).unwrap(),
                    ColorStop::new(0.3, crate::theme::OpaqueColor::rgb(30, 40, 50)).unwrap(),
                    ColorStop::new(1.0, crate::theme::OpaqueColor::rgb(255, 255, 255)).unwrap(),
                ])
                .unwrap(),
            ),
            None,
        )
        .unwrap();
        let mut second = first.clone();
        second.domain = (1.0, 3.0);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let merged = ContinuousContributor::merge([first.clone(), second.clone()])
            .unwrap()
            .unwrap();
        assert_eq!(merged.domain, (0.0, 3.0));
        assert_eq!(
            merged.stops,
            first
                .selection
                .stops()
                .unwrap()
                .iter()
                .map(ColorStop::backend_stop)
                .collect::<Vec<_>>()
        );
        second.selection = second
            .selection
            .with_sequential_gradient(Some(
                crate::theme::SequentialGradient::new([
                    ColorStop::new(0.0, crate::theme::OpaqueColor::rgb(0, 0, 0)).unwrap(),
                    ColorStop::new(0.3, crate::theme::OpaqueColor::rgb(31, 40, 50)).unwrap(),
                    ColorStop::new(1.0, crate::theme::OpaqueColor::rgb(255, 255, 255)).unwrap(),
                ])
                .unwrap(),
            ))
            .unwrap();
        assert!(matches!(
            ContinuousContributor::merge([first, second]),
            Err(Error::Data(message)) if message.contains("incompatible identity")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn close_endpoints_retry_a_complete_unique_format() {
        let close = (1.0, f64::from_bits(1.0_f64.to_bits() + 1));
        let run = || {
            let mut candidate = contributor(close);
            candidate.selection = ContinuousColorScale::sequential(
                Some(
                    crate::theme::SequentialGradient::new([
                        ColorStop::new(0.0, crate::theme::OpaqueColor::rgb(0, 0, 0)).unwrap(),
                        ColorStop::new(1.0, crate::theme::OpaqueColor::rgb(255, 255, 255)).unwrap(),
                    ])
                    .unwrap(),
                ),
                None,
            )
            .unwrap();
            ContinuousContributor::merge([candidate])
        };
        let result = run().unwrap().unwrap();
        assert_eq!(result.labels.len(), 2);
        assert_ne!(result.labels[0], result.labels[1]);
    }
}
