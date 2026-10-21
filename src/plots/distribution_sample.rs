//! Shared strict observation sample for categorical distribution summaries.

use crate::alpha::{marks::ObservationAlpha, AlphaTarget, AlphaUnitKind, ResolvedAlpha};
use crate::Error;

use crate::data::PlotData;
use crate::{Plot, RawOverlay};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistributionObservation {
    pub(crate) alpha: ObservationAlpha,
    pub(crate) value: f64,
}

#[derive(Debug)]
pub(crate) struct DistributionGroup {
    pub(crate) label: String,
    pub(crate) group_label: Option<String>,
    pub(crate) hue_label: Option<String>,
    pub(crate) x_index: usize,
    pub(crate) group_index: usize,
    pub(crate) observations: Vec<DistributionObservation>,
    pub(crate) line_alpha: Option<ResolvedAlpha>,
}

#[derive(Debug)]
pub(crate) struct DistributionSample {
    pub(crate) x_levels: Vec<String>,
    pub(crate) group_levels: Vec<String>,
    pub(crate) hue_levels: Vec<String>,
    pub(crate) groups: Vec<DistributionGroup>,
}

impl DistributionSample {
    pub(crate) fn observes_hue(&self, hue: &str) -> bool {
        self.groups
            .iter()
            .any(|group| group.hue_label.as_deref() == Some(hue))
    }

    pub(crate) fn observed_hue_levels(&self) -> impl Iterator<Item = &str> {
        self.hue_levels
            .iter()
            .filter_map(|hue| self.observes_hue(hue).then_some(hue.as_str()))
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "ordered grouping admits nested source buffers"
)]
pub(crate) fn prepare_scoped(
    chart: &Plot,
    data: PlotData<'_>,
    family: &str,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<DistributionSample, Error> {
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
    let x_column = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{family}: x column not set")))?;
    let y_column = chart
        .y_col()
        .ok_or_else(|| Error::Data(format!("{family}: y column not set")))?;
    let x_values = crate::data::strict_strings(data, x_column, family)?;
    let y_values = crate::data::strict_finite_f64(data, y_column, family)?;
    if x_values.len() != y_values.len() {
        return Err(Error::Data(format!("{family}: x/y length mismatch")));
    }
    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&x_values),
        family,
    )?;
    let mut x_levels = Vec::new();
    let group_levels = crate::data::source::collect_source(
        facet
            .and_then(|scope| scope.groups)
            .or_else(|| chart.global_group_levels())
            .unwrap_or(&channels.group_levels)
            .iter()
            .map(|level| crate::data::source::copy_source_string(level)),
    )?;
    let hue_levels = crate::data::source::collect_source(
        facet
            .and_then(|scope| scope.hues)
            .or_else(|| chart.global_hue_levels())
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| crate::data::source::copy_source_string(level)),
    )?;
    let mut groups: Vec<DistributionGroup> = Vec::new();
    for (row, (label, value)) in x_values.into_iter().zip(y_values).enumerate() {
        if !x_levels.contains(&label) {
            crate::data::source::reserve_source(&mut x_levels, 1)?;
            x_levels.push(crate::data::source::copy_source_string(&label)?);
        }
        let group_label = channels
            .groups
            .as_ref()
            .map(|values| crate::data::source::copy_source_string(&values[row]))
            .transpose()?;
        let hue_label = channels
            .hues
            .as_ref()
            .map(|values| crate::data::source::copy_source_string(&values[row]))
            .transpose()?;

        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.label == label && group.group_label == group_label)
        {
            crate::data::source::reserve_source(&mut group.observations, 1)?;
            group.observations.push(DistributionObservation {
                alpha: ObservationAlpha::resolve(data, &alpha, row)?,
                value,
            });
            continue;
        }

        let x_index = x_levels
            .iter()
            .position(|level| level == &label)
            .unwrap_or(0);
        let group_index = group_label
            .as_ref()
            .and_then(|group| group_levels.iter().position(|level| level == group))
            .unwrap_or(0);
        if let Some(hue) = &hue_label {
            if !hue_levels.iter().any(|level| level == hue) {
                return Err(Error::Data(format!(
                    "{family}: hue {hue:?} is absent from the resolved domain"
                )));
            }
        }
        let mut observations = Vec::new();
        crate::data::source::reserve_source(&mut observations, 1)?;
        observations.push(DistributionObservation {
            alpha: ObservationAlpha::resolve(data, &alpha, row)?,
            value,
        });
        crate::data::source::reserve_source(&mut groups, 1)?;
        groups.push(DistributionGroup {
            label,
            group_label,
            hue_label,
            x_index,
            group_index,
            observations,
            line_alpha: None,
        });
    }
    if groups.is_empty() {
        return Err(Error::Data(format!("{family}: no groups")));
    }
    let category = data.column(x_column)?.name();
    let mut fields = Vec::new();
    crate::data::source::reserve_source(&mut fields, 2)?;
    fields.push(category);
    fields.extend(chart.group_name());
    let line_alpha =
        chart.prepare_scoped_alpha(AlphaTarget::Line, AlphaUnitKind::Aggregate, &fields, data)?;
    for group in &mut groups {
        group.line_alpha =
            if let Some(series) = chart.group_name().zip(group.group_label.as_deref()) {
                line_alpha.keyed(&[(category, &group.label), series])?
            } else {
                line_alpha.keyed(&[(category, &group.label)])?
            };
    }
    Ok(DistributionSample {
        x_levels,
        group_levels,
        hue_levels,
        groups,
    })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validated category counts are bounded by native input and rendering limits"
)]
pub(crate) fn center(group: &DistributionGroup, group_count: usize) -> f64 {
    if group_count == 0 {
        group.x_index as f64 + 0.5
    } else {
        group.x_index as f64
            + 0.5
            + ((group.group_index as f64 + 0.5) / group_count as f64 - 0.5) * 0.72
    }
}

pub(crate) fn overlay_x(
    overlay: RawOverlay,
    center: f64,
    effective_width: f64,
    source_row: usize,
) -> f64 {
    if overlay != RawOverlay::Jittered {
        return center;
    }
    let unit = source_row_unit(source_row);
    center + (unit * 2.0 - 1.0) * effective_width * 0.20
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the upper 53 hash bits are intentionally mapped exactly onto an f64 mantissa"
)]
pub(crate) fn source_row_unit(source_row: usize) -> f64 {
    let value = splitmix64_source_row(source_row);
    ((value >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
}

const fn splitmix64_source_row(source_row: usize) -> u64 {
    // Rust's supported 32/64-bit pointer widths widen losslessly to u64. The
    // same numeric row ID maps identically; usize::MAX is necessarily target-specific.
    let mut value = (source_row as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    value
}
