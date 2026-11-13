//! Font-independent shared continuous guide semantics. The containing frame
//! measures one assigned guide only after semantic compatibility is established.

use crate::numeric::{format::NumericFormat, set::FormattedSet};
use crate::render::backend::GradientStop;
use crate::theme::{Color, ColorStop, ContinuousColorScale};
use crate::{Error, Plot};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MissingKey {
    pub fill: Color,
    pub outline: Option<crate::theme::OutlineTheme>,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousSemantics {
    pub domain: (f64, f64),
    pub selection: ContinuousColorScale,
    pub missing: Option<MissingKey>,
    pub format: Option<Arc<NumericFormat>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ContinuousContributor<'v> {
    pub field: String,
    pub title: crate::Text,
    pub selection: ContinuousColorScale,
    pub samples: Vec<f64>,
    pub missing: Option<MissingKey>,
    pub domain: (f64, f64),
    pub exact: bool,
    pub format: Option<Arc<NumericFormat>>,
    pub numeric: crate::numeric::source::NumericBinding<'v>,
    pub raw_windows: [Option<&'v crate::layout::RawWindow>; 2],
}

impl PartialEq for ContinuousContributor<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
            && self.title == other.title
            && self.selection == other.selection
            && self.samples == other.samples
            && self.missing == other.missing
            && self.domain == other.domain
            && self.exact == other.exact
            && self.format == other.format
            && self
                .numeric
                .operation
                .same_operation(other.numeric.operation)
            && self.numeric.owner == other.numeric.owner
            && self
                .raw_windows
                .map(|window| window.map(|window| (window.lower.to_bits(), window.upper.to_bits())))
                == other.raw_windows.map(|window| {
                    window.map(|window| (window.lower.to_bits(), window.upper.to_bits()))
                })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousDescriptor {
    pub field: String,
    pub title: Arc<crate::Text>,
    /// Complete final-domain gradient, retained before guide measurement.
    pub stops: Vec<GradientStop>,
    pub selection: ContinuousColorScale,
    pub center: Option<f64>,
    pub missing: Option<MissingKey>,
    pub domain: (f64, f64),
    pub exact: bool,
    pub format: Option<Arc<NumericFormat>>,
    pub numeric: FormattedSet,
}

// Prepared coordinates, stops and formats are finite, validated immutable values.
// Equality therefore has the same reflexive contract as resolved symbol glyphs.
impl Eq for ContinuousDescriptor {}

impl<'v> ContinuousContributor<'v> {
    pub(crate) fn for_plot(
        plot: &'v Plot,
        input: &crate::orchestrator::registration::PlotRegistrationBuild<'_>,
        theme: &crate::text::TextRequestContext<'_, '_>,
        numeric: crate::numeric::source::NumericBinding<'v>,
    ) -> Result<Option<Self>, Error> {
        let Some(field) = plot.description().color_dimension_key() else {
            return Ok(None);
        };
        let field = crate::data::source::copy_source_string(field)?;
        let (title, semantics, samples) = match plot.kind() {
            crate::PlotKind::Heatmap => {
                let marks = input
                    .family
                    .as_ref()
                    .and_then(|family| match family {
                        crate::orchestrator::registration::PreparedFamily::Heatmap(marks) => {
                            Some(marks)
                        }
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::Render("shared Heatmap guide has no prepared input".into())
                    })?;
                let semantics = marks.guide_semantics(theme)?;
                (
                    marks.color_label.try_copy()?,
                    semantics,
                    marks.guide_samples()?,
                )
            }
            crate::PlotKind::Beeswarm => {
                let marks = input
                    .family
                    .as_ref()
                    .and_then(|family| match family {
                        crate::orchestrator::registration::PreparedFamily::Beeswarm(marks) => {
                            Some(marks)
                        }
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::Render("shared Beeswarm guide has no prepared input".into())
                    })?;
                let semantics = marks.continuous_semantics();
                (
                    marks.color_label.try_copy()?,
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
        let exact = semantics.selection.get_domain().is_some();
        Ok(Some(Self {
            field,
            title,
            selection: semantics.selection,
            samples,
            missing: semantics.missing,
            domain: semantics.domain,
            exact,
            format: semantics.format,
            numeric,
            raw_windows: [
                plot.user_x_limits()
                    .and(plot.description().x_axis_spec().window.as_ref()),
                plot.user_y_limits()
                    .and(plot.description().y_axis_spec().window.as_ref()),
            ],
        }))
    }
}

pub(crate) fn merge_continuous<'v>(
    root: &crate::orchestrator::registration::RootIdentity,
    operation: &crate::font::canonical::OperationContext,
    contributors: impl IntoIterator<Item = ContinuousContributor<'v>>,
) -> Result<Option<ContinuousDescriptor>, Error> {
    let mut combined: Option<ContinuousDescriptor> = None;
    let mut automatic = None;
    let mut samples = Vec::new();
    let mut numeric_owner: Option<crate::numeric::context::OwnerInfo> = None;
    for contributor in contributors {
        if let Some(owner) = &mut numeric_owner {
            if owner.kind != contributor.numeric.owner.kind {
                owner.kind = None;
            }
        } else {
            numeric_owner = Some(contributor.numeric.owner.try_copy()?);
        }
        if let Some(existing) = &mut combined {
            let same_center = match (existing.center, contributor.selection.get_center()) {
                (None, None) => true,
                (Some(a), Some(b)) => a.to_bits() == b.to_bits(),
                _ => false,
            };
            if !same_text(&existing.field, &contributor.field)
                || existing.title.as_ref() != &contributor.title
                || existing.selection.kind() != contributor.selection.kind()
                || !same_stops(existing.selection.stops()?, contributor.selection.stops()?)
                || !same_center
                || existing.missing != contributor.missing
                || existing.format != contributor.format
            {
                return Err(Error::Data(
                    "shared continuous guide contributors have incompatible identity, title, scale, numeric format, or missing block".into(),
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
                title: crate::data::source::source_arc(contributor.title)?,
                center: contributor.selection.get_center(),
                selection: contributor.selection,
                stops: Vec::new(),
                missing: contributor.missing,
                domain: contributor.domain,
                exact: contributor.exact,
                format: contributor.format,
                numeric: FormattedSet {
                    members: Vec::new(),
                    labels: Vec::new(),
                    precision: 0,
                    ..FormattedSet::default()
                },
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
        let owner = numeric_owner
            .as_mut()
            .ok_or_else(|| Error::Render("shared guide source identity missing".into()))?;
        finish_shared(root, operation, existing, &samples, owner)?;
    }
    Ok(combined)
}

fn finish_shared(
    root: &crate::orchestrator::registration::RootIdentity,
    operation: &crate::font::canonical::OperationContext,
    existing: &mut ContinuousDescriptor,
    samples: &[f64],
    owner: &mut crate::numeric::context::OwnerInfo,
) -> crate::Result<()> {
    existing.selection = existing.selection.clone().domain(
        existing
            .exact
            .then(|| crate::theme::ColorDomain::exact(existing.domain.0, existing.domain.1))
            .transpose()?,
    )?;
    existing.stops =
        crate::theme::ContinuousColors::selected(existing.domain, &existing.selection)?
            .guide_stops(samples)?;
    owner.shared = true;
    owner.canvas = None;
    owner.facet = None;
    let mut scope = String::new();
    let prefix = "shared continuous ";
    let length = prefix
        .len()
        .checked_add(existing.field.len())
        .ok_or_else(|| Error::Render("shared numeric owner length overflow".into()))?;
    crate::data::source::reserve_source_string(&mut scope, length)?;
    scope.push_str(prefix);
    scope.push_str(&existing.field);
    owner.scope = Some(scope);
    let numeric = crate::numeric::source::NumericBinding::shared(root, operation, owner)?;
    existing.numeric = domain_presentation(
        existing.domain,
        existing.center,
        existing
            .format
            .as_deref()
            .map(|format| (format, numeric, crate::NumericRangeOrigin::ParentResolved)),
    )?;
    Ok(())
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

pub(crate) fn domain_presentation(
    domain: (f64, f64),
    center: Option<f64>,
    format: Option<(
        &NumericFormat,
        crate::numeric::source::NumericBinding<'_>,
        crate::NumericRangeOrigin,
    )>,
) -> Result<FormattedSet, Error> {
    let mut values = [0.0_f64; 3];
    let count = if same_value(domain.0, domain.1) {
        values[0] = domain.0;
        1
    } else {
        values[0] = domain.0;
        let mut count = 1;
        if let Some(center) = center.filter(|center| *center > domain.0 && *center < domain.1) {
            values[1] = center;
            count += 1;
        }
        values[count] = domain.1;
        count + 1
    };
    crate::numeric::set::guide_presentation(&values[..count], format)
}
