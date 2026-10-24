//! The four semantic total-eligibility sets, resolved before fonts. Feedback
//! selects an already prepared result or recorded precision refusal.

use super::PreparedWaterfall;
use crate::numeric::set::{FormattedSet, NumericPresentation};
use crate::{Error, Result};

#[derive(Debug)]
pub(super) struct PreparedTotals {
    pub(super) totals: [(f64, Option<crate::Text>); 2],
    pub(super) numeric: std::sync::Arc<NumericPresentation>,
}

#[derive(Debug)]
pub(super) enum TotalVariant {
    Ready(PreparedTotals),
    Collision,
}

impl PreparedWaterfall {
    pub(crate) fn prepare_totals(
        &mut self,
        axes: &crate::plots::shared_x::PreparedAxes,
        can_expand: bool,
    ) -> Result<()> {
        let x = axes.specs[0].limits.unwrap_or(axes.ranges[0]);
        let y = axes.specs[1].limits.unwrap_or(axes.ranges[1]);
        let projection = crate::layout::Projection::new(x, y, (0.0, 1.0, 0.0, 1.0));
        let initial = self.eligibility(&projection)?;
        let row_position = self.source_y()?;
        let y_visible = row_position >= y.0 && row_position <= y.1;
        for mask in 0..4_u8 {
            if mask != initial && !(can_expand && y_visible && (mask | initial) == mask) {
                continue;
            }
            if self.total_variants[usize::from(mask)].is_none() {
                self.total_variants[usize::from(mask)] = Some(self.prepare_variant(mask)?);
            }
        }
        self.selected_totals(&projection)?;
        Ok(())
    }

    fn source_y(&self) -> Result<f64> {
        Ok(f64::from(u32::try_from(self.labels.len()).map_err(
            |_| Error::Render("Waterfall category count exceeds spatial range".into()),
        )?))
    }

    fn eligibility(&self, projection: &crate::layout::Projection) -> Result<u8> {
        let row_position = self.source_y()?;
        Ok(self
            .total_candidates
            .iter()
            .enumerate()
            .fold(0, |mask, (index, (value, _))| {
                mask | if projection.contains_data(*value, row_position) {
                    1 << index
                } else {
                    0
                }
            }))
    }

    pub(super) fn selected_totals(
        &self,
        projection: &crate::layout::Projection,
    ) -> Result<&PreparedTotals> {
        match self.total_variants[usize::from(self.eligibility(projection)?)].as_ref() {
            Some(TotalVariant::Ready(prepared)) => Ok(prepared),
            Some(TotalVariant::Collision) => Err(Error::Data(
                "Waterfall total labels are not unique at configured precision".into(),
            )),
            None => Err(Error::Render(
                "Waterfall total eligibility was not prepared before fonts".into(),
            )),
        }
    }

    fn prepare_variant(&self, mask: u8) -> Result<TotalVariant> {
        let mut members = [0.0; 2];
        let mut count = 0;
        for (index, (value, _)) in self.total_candidates.iter().enumerate() {
            if mask & (1 << index) != 0 {
                members[count] = *value;
                count += 1;
            }
        }
        let members = &members[..count];
        let set = if let Some(format) = &self.total_format {
            let Some(set) = format.format_set(members, false, false)? else {
                return Ok(TotalVariant::Collision);
            };
            set
        } else {
            FormattedSet {
                members: crate::data::source::collect_source(
                    members.iter().map(|value| Ok(value.to_bits())),
                )?,
                labels: crate::data::source::collect_source(members.iter().map(|value| {
                    crate::numeric::profiles::BuiltinProfile::WaterfallTotal.format(*value)
                }))?,
                precision: 3,
            }
        };
        let mut labels = set.labels.iter();
        let mut totals = self
            .total_candidates
            .each_ref()
            .map(|(value, _)| (*value, None));
        for (index, ((_, name), (_, source))) in
            self.total_candidates.iter().zip(&mut totals).enumerate()
        {
            if mask & (1 << index) == 0 {
                continue;
            }
            let number = labels
                .next()
                .ok_or_else(|| Error::Render("Waterfall numeric occurrence missing".into()))?;
            let mut label = String::new();
            let length = name
                .as_str()
                .len()
                .checked_add(1)
                .and_then(|length| length.checked_add(number.len()))
                .ok_or_else(|| Error::Render("Waterfall total source length overflow".into()))?;
            crate::data::source::reserve_source_string(&mut label, length)?;
            label.push_str(name.as_str());
            label.push(' ');
            label.push_str(number);
            *source = Some(name.try_copy_with_source(label)?);
        }
        Ok(TotalVariant::Ready(PreparedTotals {
            totals,
            numeric: crate::data::source::source_arc(NumericPresentation {
                format: self.total_format.clone(),
                set,
            })?,
        }))
    }

    /// Every actual successful source reachable under admitted window feedback.
    pub(crate) fn total_font_sources(&self) -> impl Iterator<Item = &crate::Text> {
        self.total_variants
            .iter()
            .flatten()
            .filter_map(|variant| match variant {
                TotalVariant::Ready(prepared) => Some(prepared),
                TotalVariant::Collision => None,
            })
            .flat_map(|prepared| prepared.totals.iter())
            .filter_map(|(_, source)| source.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::{
        decimal::NumericNotation,
        format::{NumericFormat, NumericPrecision},
    };

    #[test]
    fn eligible_total_masks_resolve_before_fonts_and_select_only_retained_results() {
        let format = NumericFormat::new(
            NumericNotation::Decimal,
            NumericPrecision::AdaptiveFractional { min: 0, max: 2 },
        )
        .unwrap();
        let source = || {
            crate::Plot::waterfall(
                crate::CategoryValues::new("step", ["one"]).unwrap(),
                crate::NumericValues::new("delta", [0.01]),
                1.01,
            )
            .total_format(format.clone())
        };
        let operation = crate::font::canonical::OperationContext::new();
        for (limits, y_limits, expected) in [
            ((1.0, 1.03), (0.0, 1.0), vec!["base 1.01", "total 1.02"]),
            ((1.0, 1.015), (0.0, 1.0), vec!["base 1"]),
            ((1.015, 1.03), (0.0, 1.0), vec!["total 1"]),
            ((1.0, 1.03), (0.0, 0.75), vec![]),
        ] {
            let plot = source()
                .x_limits(limits.0, limits.1)
                .erase()
                .with_planned_y_limits(y_limits);
            let inputs = crate::plots::shared_x::prepare_plot(&plot, &operation, None).unwrap();
            let marks = &inputs.waterfall.as_ref().unwrap().panels[0].1;
            let projection =
                crate::layout::Projection::new(limits, y_limits, (0.0, 800.0, 0.0, 500.0));
            let selected = marks.selected_totals(&projection).unwrap();
            let labels: Vec<_> = selected
                .totals
                .iter()
                .filter_map(|(_, text)| text.as_ref().map(crate::Text::as_str))
                .collect();
            assert_eq!(labels, expected);
            assert_eq!(
                marks
                    .total_font_sources()
                    .map(crate::Text::as_str)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                selected.numeric.set.precision,
                if expected.len() == 2 { 2 } else { 0 }
            );
            assert_eq!(selected.numeric.set.members.len(), expected.len());
            let closed = crate::layout::Projection::new(
                (marks.total_candidates[0].0, marks.total_candidates[1].0),
                y_limits,
                (0.0, 1.0, 0.0, 1.0),
            );
            if expected.len() == 2 {
                assert_eq!(marks.eligibility(&closed).unwrap(), 3);
            }
        }
        crate::Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .panel(0, 0, source())
            .unwrap()
            .panel(
                0,
                1,
                crate::Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 2.0]),
                    crate::NumericValues::new("y", [0.0, 1.0]),
                ),
            )
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .preflight()
            .unwrap();
        let limited_format = format
            .clone()
            .precision(NumericPrecision::AdaptiveFractional { min: 0, max: 1 })
            .unwrap();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = source()
            .total_format(limited_format.clone())
            .preflight()
            .unwrap_err();
        assert!(error.to_string().contains("not unique"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        let limited = source()
            .total_format(limited_format)
            .x_limits(1.0, 1.015)
            .erase();
        let mut inputs = crate::plots::shared_x::prepare_plot(&limited, &operation, None).unwrap();
        let marks = &mut std::sync::Arc::get_mut(inputs.waterfall.as_mut().unwrap())
            .unwrap()
            .panels[0]
            .1;
        marks.prepare_totals(&inputs.axes[0].1, true).unwrap();
        assert!(matches!(
            marks.total_variants[3],
            Some(TotalVariant::Collision)
        ));
        let expanded =
            crate::layout::Projection::new((1.0, 1.03), (0.0, 1.0), (0.0, 800.0, 0.0, 500.0));
        assert!(matches!(
            marks.selected_totals(&expanded),
            Err(Error::Data(_))
        ));
    }
}
