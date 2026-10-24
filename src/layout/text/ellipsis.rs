//! Source-bounded grapheme views with retained original/synthetic provenance.

use super::{DisplayFragment, PhysicalTextAlignment, ResolvedTextPlan};
use crate::layout::fit::{FitOutcome, FitScope};
use crate::text::TextAppearance;
use crate::{Error, TextDirection, TextElision, TextEllipsis, TextOverflow};
use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
mod tests;

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one explicit ellipsis transaction preserves complete source admission, paragraph context, source-bounded views and terminal handling"
)]
pub(crate) fn measure(
    theme: &crate::theme::ResolvedTheme,
    source: &str,
    appearance: TextAppearance<'_>,
    width: i64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
    policy: &TextEllipsis,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let original = appearance.layout.expect("ellipsis request");
    let mut request = original.clone().overflow(TextOverflow::complete());
    if let Some(lines) = policy.lines {
        let bounds = request
            .instructions()
            .bounds
            .unwrap_or(crate::TextBounds::new(None, None, None)?);
        request.edit_instructions().bounds = Some(crate::TextBounds {
            lines: Some(bounds.lines.map_or(lines, |bound| bound.min(lines))),
            ..bounds
        });
    }
    let selected = TextAppearance {
        layout: Some(&request),
        ..appearance
    };
    if let FitOutcome::Measured(plan) =
        super::request::measure_current(theme, source, selected, width, alignment, owner, scope)?
    {
        return Ok(FitOutcome::Measured(plan.with_request(Some(original))));
    }
    // Analyze the complete original before consuming any marker. A missing
    // original glyph is never a recoverable abbreviation failure.
    let mut analysis = request.clone();
    analysis.edit_instructions().bounds = None;
    analysis.edit_instructions().wrapping = Some(crate::TextWrapping::NoSoftWrap);
    let basis = match super::request::measure_current(
        theme,
        source,
        TextAppearance {
            layout: Some(&analysis),
            ..appearance
        },
        i64::MAX,
        alignment,
        owner,
        scope,
    )? {
        FitOutcome::Measured(plan) => plan,
        FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
    };
    crate::text::validate_for(&policy.marker, "ellipsis marker")?;
    let mut boundaries = crate::data::source::collect_source(
        source.grapheme_indices(true).map(|(index, _)| Ok(index)),
    )?;
    crate::data::source::reserve_source(&mut boundaries, 1)?;
    boundaries.push(source.len());
    let count = boundaries.len() - 1;
    let rtl = match original.instructions().direction.unwrap_or_default() {
        TextDirection::RightToLeft => true,
        TextDirection::LeftToRight => false,
        TextDirection::Auto => basis.lines.first().is_some_and(|line| {
            line.base_direction == crate::font::canonical::ShapingDirection::RightToLeft
        }),
    };
    request.edit_instructions().direction = Some(if rtl {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    });
    let position = match policy.position {
        TextElision::Left => {
            if rtl {
                TextElision::End
            } else {
                TextElision::Start
            }
        }
        TextElision::Right => {
            if rtl {
                TextElision::Start
            } else {
                TextElision::End
            }
        }
        position => position,
    };
    let weights = grapheme_weights(&basis, &boundaries)?;
    for kept in (0..count).rev() {
        let left = match position {
            TextElision::Start => 0,
            TextElision::End => kept,
            TextElision::Middle => balanced_split(kept, count, &weights),
            _ => unreachable!("physical direction resolved"),
        };
        let right = count - (kept - left);
        let (view, map) = view(source, boundaries[left], boundaries[right], &policy.marker)?;
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::EllipsisViewTrial,
        );
        match super::request::measure_current(
            theme,
            &view,
            TextAppearance {
                layout: Some(&request),
                ..appearance
            },
            width,
            alignment,
            owner,
            scope,
        )? {
            FitOutcome::Measured(plan) => {
                return Ok(FitOutcome::Measured(
                    plan.with_display(source, map)?.with_request(Some(original)),
                ))
            }
            FitOutcome::Miss(_) => {}
        }
    }
    // No marker/view fits. Only the caller's terminal spatial action may recover.
    let mut terminal = original.clone();
    terminal
        .edit_instructions()
        .overflow
        .as_mut()
        .expect("ellipsis overflow")
        .ellipsis = None;
    Ok(
        match super::request::measure_current(
            theme,
            source,
            TextAppearance {
                layout: Some(&terminal),
                ..appearance
            },
            width,
            alignment,
            owner,
            scope,
        )? {
            FitOutcome::Measured(plan) => FitOutcome::Measured(plan.with_request(Some(original))),
            FitOutcome::Miss(miss) => FitOutcome::Miss(miss),
        },
    )
}

fn balanced_split(kept: usize, count: usize, prefix: &[u128]) -> usize {
    let start = usize::from(kept > 1);
    let end = kept.saturating_sub(start);
    (start..=end)
        .min_by_key(|&left| prefix[left].abs_diff(prefix[count] - prefix[count - (kept - left)]))
        .unwrap_or(0)
}

fn grapheme_weights(plan: &ResolvedTextPlan, boundaries: &[usize]) -> Result<Vec<u128>, Error> {
    let mut weights = crate::data::source::collect_source(boundaries.iter().map(|_| Ok(0_u128)))?;
    for glyph in plan
        .lines
        .iter()
        .flat_map(|line| &line.runs)
        .flat_map(|run| &run.glyphs)
    {
        let index = boundaries
            .partition_point(|&byte| byte <= glyph.cluster as usize)
            .saturating_sub(1);
        weights[index] = weights[index]
            .checked_add(u128::from(glyph.x_advance_quanta.unsigned_abs()))
            .ok_or_else(|| Error::Render("ellipsis measured endpoint weight overflow".into()))?;
    }
    let mut sum = 0_u128;
    for value in &mut weights {
        let current = *value;
        *value = sum;
        sum = sum
            .checked_add(current)
            .ok_or_else(|| Error::Render("ellipsis measured prefix overflow".into()))?;
    }
    Ok(weights)
}

fn view(
    source: &str,
    left: usize,
    right: usize,
    marker: &str,
) -> Result<(String, Vec<DisplayFragment>), Error> {
    let capacity = source
        .len()
        .checked_add(marker.len())
        .ok_or_else(|| Error::Render("ellipsis display length overflow".into()))?;
    let mut display = String::new();
    crate::data::source::reserve_source_string(&mut display, capacity)?;
    let mut map = Vec::new();
    crate::data::source::reserve_source(&mut map, 3)?;
    display.push_str(&source[..left]);
    map.push(DisplayFragment {
        display: 0..display.len(),
        original: Some(0..left),
    });
    let start = display.len();
    display.push_str(marker);
    map.push(DisplayFragment {
        display: start..display.len(),
        original: None,
    });
    // Retain directional controls in the omitted interval, preserving the
    // paragraph's embedding/isolate context without consuming omitted glyphs.
    for (offset, ch) in source[left..right].char_indices() {
        if bidi_control(ch) {
            let start = display.len();
            display.push(ch);
            crate::data::source::reserve_source(&mut map, 1)?;
            map.push(DisplayFragment {
                display: start..display.len(),
                original: Some(left + offset..left + offset + ch.len_utf8()),
            });
        }
    }
    let start = display.len();
    display.push_str(&source[right..]);
    crate::data::source::reserve_source(&mut map, 1)?;
    map.push(DisplayFragment {
        display: start..display.len(),
        original: Some(right..source.len()),
    });
    Ok((display, map))
}

pub(crate) fn bidi_control(ch: char) -> bool {
    matches!(ch,'\u{061c}'|'\u{200e}'|'\u{200f}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
}
