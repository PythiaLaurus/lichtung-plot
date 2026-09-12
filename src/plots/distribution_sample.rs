//! Shared strict observation sample for categorical distribution summaries.

use crate::Error;

use crate::data::PlotData;
use crate::render::backend::PointStyle;
use crate::{Plot, RawOverlay};

#[derive(Debug)]
pub(crate) struct DistributionObservation {
    pub(crate) source_row: usize,
    pub(crate) value: f64,
}

#[derive(Debug)]
pub(crate) struct DistributionGroup {
    pub(crate) label: String,
    pub(crate) group_label: Option<String>,
    pub(crate) hue_label: Option<String>,
    pub(crate) x_index: usize,
    pub(crate) group_index: usize,
    pub(crate) hue_index: Option<usize>,
    pub(crate) observations: Vec<DistributionObservation>,
}

#[derive(Debug)]
pub(crate) struct DistributionSample {
    pub(crate) x_levels: Vec<String>,
    pub(crate) group_levels: Vec<String>,
    pub(crate) hue_levels: Vec<String>,
    pub(crate) groups: Vec<DistributionGroup>,
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
    let group_levels = data.collect_source(
        facet
            .and_then(|scope| scope.groups)
            .or_else(|| chart.global_group_levels())
            .unwrap_or(&channels.group_levels)
            .iter()
            .map(|level| data.copy_source_string(level)),
    )?;
    let hue_levels = data.collect_source(
        facet
            .and_then(|scope| scope.hues)
            .or_else(|| chart.global_hue_levels())
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| data.copy_source_string(level)),
    )?;
    let mut groups: Vec<DistributionGroup> = Vec::new();
    for (row, (label, value)) in x_values.into_iter().zip(y_values).enumerate() {
        data.claim_source_visits(x_levels.len())?;
        if !x_levels.contains(&label) {
            data.reserve_source(&mut x_levels, 1)?;
            x_levels.push(data.copy_source_string(&label)?);
        }
        let group_label = channels
            .groups
            .as_ref()
            .map(|values| data.copy_source_string(&values[row]))
            .transpose()?;
        let hue_label = channels
            .hues
            .as_ref()
            .map(|values| data.copy_source_string(&values[row]))
            .transpose()?;
        data.claim_source_visits(groups.len())?;
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.label == label && group.group_label == group_label)
        {
            data.reserve_source(&mut group.observations, 1)?;
            group.observations.push(DistributionObservation {
                source_row: data.source_row(row),
                value,
            });
            continue;
        }
        data.claim_source_visits(x_levels.len())?;
        data.claim_source_visits(group_levels.len())?;
        data.claim_source_visits(hue_levels.len())?;
        let x_index = x_levels
            .iter()
            .position(|level| level == &label)
            .unwrap_or(0);
        let group_index = group_label
            .as_ref()
            .and_then(|group| group_levels.iter().position(|level| level == group))
            .unwrap_or(0);
        let hue_index = hue_label
            .as_ref()
            .and_then(|hue| hue_levels.iter().position(|level| level == hue));
        let mut observations = Vec::new();
        data.reserve_source(&mut observations, 1)?;
        observations.push(DistributionObservation {
            source_row: data.source_row(row),
            value,
        });
        data.reserve_source(&mut groups, 1)?;
        groups.push(DistributionGroup {
            label,
            group_label,
            hue_label,
            x_index,
            group_index,
            hue_index,
            observations,
        });
    }
    if groups.is_empty() {
        return Err(Error::Data(format!("{family}: no groups")));
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

pub(crate) fn overlay_style(color: (u8, u8, u8)) -> PointStyle {
    PointStyle::new(color, 2)
}

#[cfg(test)]
mod tests {
    use super::{overlay_x, splitmix64_source_row};
    use crate::RawOverlay;

    #[test]
    fn jitter_is_stable_and_bounded_by_twenty_percent_of_effective_width() {
        let values = (0..10_000)
            .map(|row| overlay_x(RawOverlay::Jittered, 3.0, 0.5, row))
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            (0..10_000)
                .map(|row| overlay_x(RawOverlay::Jittered, 3.0, 0.5, row))
                .collect::<Vec<_>>()
        );
        assert!(values.iter().all(|value| (value - 3.0).abs() <= 0.1));
    }

    #[test]
    fn points_are_exactly_centered() {
        assert_eq!(
            overlay_x(RawOverlay::Points, 2.5, 0.8, 99).to_bits(),
            2.5_f64.to_bits()
        );
    }

    #[test]
    fn splitmix64_source_row_vectors_are_pinned_across_pointer_widths() {
        let vectors = [
            (0, 0xe220_a839_7b1d_cdaf),
            (1, 0x910a_2dec_8902_5cc1),
            (42, 0xbdd7_3226_2feb_6e95),
            (123_456_789, 0x223c_74d9_3deb_7679),
        ];
        assert_eq!(
            vectors.map(|(row, _)| splitmix64_source_row(row)),
            vectors.map(|(_, expected)| expected)
        );

        let expected_max = match usize::BITS {
            32 => 0x73b1_3ba2_aff1_81c0,
            64 => 0xe4d9_7177_1b65_2c20,
            width => panic!("unsupported usize width {width}"),
        };
        assert_eq!(splitmix64_source_row(usize::MAX), expected_max);
    }

    #[test]
    fn jitter_projection_vectors_are_pinned() {
        let vectors = [
            (0, 0x3fc3_a021_a518_9f8f),
            (1, 0x3f9b_437c_ada8_03c0),
            (42, 0x3fb8_bc7a_75ac_c490),
            (123_456_789, 0xbfc2_c0b5_6e26_d0ea),
        ];
        assert_eq!(
            vectors.map(|(row, _)| overlay_x(RawOverlay::Jittered, 0.0, 1.0, row).to_bits()),
            vectors.map(|(_, expected)| expected)
        );

        let expected_max = match usize::BITS {
            32 => 0xbf93_b13a_2ee6_7d9a,
            64 => 0x3fc4_2b7d_17d2_476f,
            width => panic!("unsupported usize width {width}"),
        };
        assert_eq!(
            overlay_x(RawOverlay::Jittered, 0.0, 1.0, usize::MAX).to_bits(),
            expected_max
        );
    }
}
