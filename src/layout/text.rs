//! Deterministic measured multiline text layout.

use crate::Error;
use unicode_segmentation::UnicodeSegmentation;

/// One measured block of lines resolved for a fixed width and font size.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextBlock {
    lines: Vec<String>,
    width: f64,
    line_height: f64,
}

impl TextBlock {
    pub(crate) fn layout(
        text: &str,
        font_size: u32,
        maximum_width: f64,
        role: &str,
    ) -> Result<Self, Error> {
        if !maximum_width.is_finite() || maximum_width <= 0.0 {
            return Err(Error::Render(format!(
                "{role} has no positive finite width available for text layout"
            )));
        }
        let mut lines = Vec::new();
        let mut line = String::new();
        for grapheme in text.graphemes(true) {
            if matches!(grapheme, "\n" | "\r" | "\r\n") {
                lines.push(std::mem::take(&mut line));
                continue;
            }
            if crate::layout::deterministic_text_width(grapheme, f64::from(font_size))
                > maximum_width
            {
                return Err(Error::Render(format!(
                    "{role} contains a grapheme that cannot fit within the available {maximum_width:.1}px width"
                )));
            }
            line.push_str(grapheme);
            if crate::layout::deterministic_text_width(&line, f64::from(font_size)) <= maximum_width
            {
                continue;
            }
            if let Some(split) = last_wrap_boundary_before_final_grapheme(&line) {
                let remainder = line.split_off(split);
                lines.push(std::mem::take(&mut line));
                line = remainder;
            }
        }
        if !line.is_empty() || text.ends_with('\n') || text.ends_with('\r') || lines.is_empty() {
            lines.push(line);
        }
        rebalance_orphaned_parentheticals(&mut lines, font_size, maximum_width);
        let width = lines
            .iter()
            .map(|line| crate::layout::deterministic_text_width(line, f64::from(font_size)))
            .fold(0.0_f64, f64::max);
        Ok(Self {
            lines,
            width,
            line_height: f64::from(font_size) * 1.2,
        })
    }

    pub(crate) fn lines(&self) -> &[String] {
        &self.lines
    }

    pub(crate) fn into_lines(self) -> Vec<String> {
        self.lines
    }

    pub(crate) const fn width(&self) -> f64 {
        self.width
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "allocated text lines are bounded by finite input and layout dimensions"
    )]
    pub(crate) fn height(&self) -> f64 {
        self.line_height * self.lines.len() as f64
    }

    pub(crate) const fn line_height(&self) -> f64 {
        self.line_height
    }

    pub(crate) fn unwrapped(text: &str, font_size: u32) -> Self {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines = normalized
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let width = lines
            .iter()
            .map(|line| crate::layout::deterministic_text_width(line, f64::from(font_size)))
            .fold(0.0_f64, f64::max);
        Self {
            lines,
            width,
            line_height: f64::from(font_size) * 1.2,
        }
    }
}

fn last_wrap_boundary_before_final_grapheme(line: &str) -> Option<usize> {
    let mut boundaries = line
        .grapheme_indices(true)
        .map(|(index, grapheme)| (index, grapheme, index + grapheme.len()))
        .collect::<Vec<_>>();
    boundaries.pop();
    boundaries
        .into_iter()
        .rev()
        .find(|(index, grapheme, end)| {
            grapheme.chars().all(char::is_whitespace)
                || (matches!(*grapheme, "-" | "‐" | "‑")
                    && line[..*index]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_alphabetic)
                    && line[*end..].chars().next().is_some_and(char::is_alphabetic))
        })
        .map(|(_, _, end)| end)
}

fn rebalance_orphaned_parentheticals(lines: &mut [String], font_size: u32, maximum_width: f64) {
    for index in 1..lines.len() {
        let current = lines[index].trim();
        if !current.starts_with('(')
            || !current.ends_with(')')
            || current.chars().any(char::is_whitespace)
        {
            continue;
        }
        let previous = lines[index - 1].trim_end();
        let Some((split, whitespace)) = previous
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
        else {
            continue;
        };
        let prefix = previous[..split].trim_end();
        let word = previous[split + whitespace.len_utf8()..].trim_start();
        if prefix.is_empty() || word.is_empty() {
            continue;
        }
        let candidate = format!("{word} {current}");
        if crate::layout::deterministic_text_width(&candidate, f64::from(font_size))
            <= maximum_width
        {
            let prefix = prefix.to_owned();
            lines[index - 1] = prefix;
            lines[index] = candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_keeps_an_indivisible_word_intact() {
        let family = "👨‍👩‍👧‍👦";
        let width = crate::layout::deterministic_text_width(family, 12.0);
        let block = TextBlock::layout(&format!("{family}{family}"), 12, width, "test").unwrap();

        assert_eq!(block.lines(), [format!("{family}{family}")]);
    }

    #[test]
    fn wrapping_breaks_after_a_hyphen_without_splitting_words() {
        let width = crate::layout::deterministic_text_width("Risk-", 12.0);
        let block = TextBlock::layout("Risk-warning score", 12, width, "test").unwrap();

        assert_eq!(block.lines(), ["Risk-", "warning ", "score"]);
    }

    #[test]
    fn wrapping_does_not_break_a_scientific_notation_exponent() {
        let width = crate::layout::deterministic_text_width("8.83e-", 12.0);
        let block = TextBlock::layout("8.83e-1", 12, width, "test").unwrap();

        assert_eq!(block.lines(), ["8.83e-1"]);
    }

    #[test]
    fn wrapping_keeps_a_short_parenthetical_with_its_preceding_word() {
        let width = crate::layout::deterministic_text_width("(d) Response Criterion ", 12.0);
        let block = TextBlock::layout("(d) Response Criterion (c)", 12, width, "test").unwrap();

        assert_eq!(block.lines(), ["(d) Response", "Criterion (c)"]);
    }

    #[test]
    fn explicit_blank_and_trailing_lines_are_preserved() {
        let block = TextBlock::layout("first\n\nlast\n", 12, 100.0, "test").unwrap();

        assert_eq!(block.lines(), ["first", "", "last", ""]);
    }

    #[test]
    fn windows_and_classic_line_endings_create_explicit_lines() {
        let block = TextBlock::layout("first\r\nsecond\rlast", 12, 100.0, "test").unwrap();

        assert_eq!(block.lines(), ["first", "second", "last"]);
    }

    #[test]
    fn individually_unfit_grapheme_returns_actionable_error() {
        let error = TextBlock::layout("中", 12, 1.0, "axis title").unwrap_err();

        assert!(error.to_string().contains("axis title contains a grapheme"));
    }
}
