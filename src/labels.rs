//! Presentation policy for automatically sourced plot labels.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::data::{source::SourceAdmission, PlotData};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelMode {
    Humanized,
    Raw,
}

/// Immutable presentation rules for labels inferred from data identities.
///
/// Explicit chart text such as axis-label overrides, titles, and captions is
/// always rendered verbatim. Aliases take precedence over the selected
/// humanized or raw fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labels {
    mode: LabelMode,
    aliases: Option<Arc<BTreeMap<String, String>>>,
}

impl Default for Labels {
    fn default() -> Self {
        Self::humanized()
    }
}

impl Labels {
    /// Humanize automatically sourced column names and categorical values.
    #[must_use]
    pub const fn humanized() -> Self {
        Self {
            mode: LabelMode::Humanized,
            aliases: None,
        }
    }

    /// Render automatically sourced labels exactly as represented by the plot data.
    #[must_use]
    pub const fn raw() -> Self {
        Self {
            mode: LabelMode::Raw,
            aliases: None,
        }
    }

    /// Add one exact raw-identity to display-label mapping.
    #[must_use]
    pub fn alias(mut self, raw: impl Into<String>, display: impl Into<String>) -> Self {
        Arc::make_mut(
            self.aliases
                .get_or_insert_with(|| Arc::new(BTreeMap::new())),
        )
        .insert(raw.into(), display.into());
        self
    }

    /// Add exact raw-identity to display-label mappings.
    #[must_use]
    pub fn aliases<I, R, D>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = (R, D)>,
        R: Into<String>,
        D: Into<String>,
    {
        for (raw, display) in aliases {
            self = self.alias(raw, display);
        }
        self
    }

    pub(crate) fn resolve(&self, raw: &str) -> String {
        self.resolve_source(raw, None).expect("label allocation")
    }

    pub(crate) fn resolve_source(
        &self,
        raw: &str,
        data: Option<PlotData<'_>>,
    ) -> Result<String, Error> {
        data.source_visits(raw.len())?;
        // BTreeMap does not expose its comparisons. Admit a conservative bound
        // before lookup, including the constant work of empty-key comparisons.
        data.source_visits(
            raw.len()
                .checked_add(1)
                .and_then(|length| {
                    self.aliases
                        .as_ref()
                        .map_or(0, |aliases| aliases.len())
                        .checked_mul(length)
                })
                .ok_or_else(|| Error::Render("label alias lookup work overflow".into()))?,
        )?;
        if let Some(alias) = self.aliases.as_ref().and_then(|aliases| aliases.get(raw)) {
            copy_label(alias, data)
        } else if self.mode == LabelMode::Humanized {
            humanize(raw, data)
        } else {
            copy_label(raw, data)
        }
    }

    pub(crate) fn resolve_domain<'a>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<String>, Error> {
        self.resolve_domain_source(domain, raw_labels, None)
    }

    pub(crate) fn resolve_domain_source<'a>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'a str>,
        data: Option<PlotData<'_>>,
    ) -> Result<Vec<String>, Error> {
        let mut identities = Vec::new();
        let mut resolved = Vec::new();
        let raw_labels = raw_labels.into_iter();
        let minimum = raw_labels.size_hint().0;
        data.source_reserve(&mut identities, minimum)?;
        data.source_reserve(&mut resolved, minimum)?;
        for raw in raw_labels {
            data.source_reserve(&mut identities, 1)?;
            data.source_reserve(&mut resolved, 1)?;
            let display = self.resolve_source(raw, data)?;
            let mut collision = None;
            for (&previous, prior) in identities.iter().zip(&resolved) {
                data.source_visits(1)?;
                data.source_visits(display.len())?;
                data.source_visits(raw.len())?;
                if prior == &display && previous != raw {
                    collision = Some(previous);
                    break;
                }
            }
            if let Some(previous) = collision {
                let alias_guidance = if self.aliases.as_ref().is_some_and(|aliases| {
                    aliases.contains_key(previous) || aliases.contains_key(raw)
                }) {
                    "change or remove the colliding aliases; Labels::raw() does not override explicit aliases"
                } else {
                    "configure an explicit alias or use Labels::raw()"
                };
                return Err(Error::Render(format!(
                    "label collision in {domain}: raw identities {previous:?} and {raw:?} both resolve to {display:?}; {alias_guidance}"
                )));
            }
            identities.push(raw);
            resolved.push(display);
        }
        Ok(resolved)
    }
}

fn copy_label(raw: &str, data: Option<PlotData<'_>>) -> Result<String, Error> {
    let mut output = String::new();
    reserve_label(&mut output, raw.len(), data)?;
    output.push_str(raw);
    Ok(output)
}

fn reserve_label(
    output: &mut String,
    count: usize,
    data: Option<PlotData<'_>>,
) -> Result<(), Error> {
    if let Some(data) = data {
        data.reserve_source_string(output, count)
    } else {
        output
            .try_reserve_exact(count)
            .map_err(|error| Error::Render(error.to_string()))
    }
}

fn humanize(raw: &str, data: Option<PlotData<'_>>) -> Result<String, Error> {
    let mut split = String::new();
    reserve_label(
        &mut split,
        raw.len()
            .checked_mul(2)
            .ok_or_else(|| Error::Render("humanized label length overflow".into()))?,
        data,
    )?;
    let mut chars = raw.chars().peekable();
    let mut previous = None;
    let mut last_was_space = true;
    while let Some(character) = chars.next() {
        let next = chars.peek().copied();
        let prior = previous.replace(character);
        if matches!(character, '_' | '-' | '.' | ' ' | '\t' | '\n' | '\r') {
            if !last_was_space {
                split.push(' ');
                last_was_space = true;
            }
            continue;
        }
        if should_split_before(prior, character, next) && !last_was_space {
            split.push(' ');
        }
        split.push(character);
        last_was_space = false;
    }
    let mut output = String::new();
    data.source_visits(split.len())?;
    for (index, word) in split.split_whitespace().enumerate() {
        data.source_visits(
            word.len()
                .checked_mul(22)
                .ok_or_else(|| Error::Render("humanized word work overflow".into()))?,
        )?;
        let acronym = acronym(word);
        reserve_label(
            &mut output,
            word.len()
                .checked_add(12)
                .ok_or_else(|| Error::Render("humanized word length overflow".into()))?,
            data,
        )?;
        if index != 0 {
            output.push(' ');
        }
        if let Some(acronym) = acronym {
            output.push_str(acronym);
        } else if word.len() > 1 && word.bytes().all(|c| c.is_ascii_uppercase()) {
            output.push_str(word);
        } else {
            for (offset, character) in word.chars().enumerate() {
                let character = character.to_ascii_lowercase();
                if index == 0 && offset == 0 {
                    output.extend(character.to_uppercase());
                } else {
                    output.push(character);
                }
            }
        }
    }
    if output.is_empty() {
        copy_label(raw, data)
    } else {
        Ok(output)
    }
}

fn should_split_before(previous: Option<char>, current: char, next: Option<char>) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    (previous.is_ascii_lowercase() && current.is_ascii_uppercase())
        || (previous.is_ascii_digit() && current.is_ascii_alphabetic())
        || (previous.is_ascii_alphabetic() && current.is_ascii_digit())
        || (previous.is_ascii_uppercase()
            && current.is_ascii_uppercase()
            && next.is_some_and(|character| character.is_ascii_lowercase()))
}

fn acronym(word: &str) -> Option<&'static str> {
    [
        ("ci", "CI"),
        ("dna", "DNA"),
        ("hwe", "HWE"),
        ("id", "ID"),
        ("ms", "MS"),
        ("mz", "m/z"),
        ("m/z", "m/z"),
        ("pca", "PCA"),
        ("qc", "QC"),
        ("rna", "RNA"),
        ("tsne", "t-SNE"),
        ("t-sne", "t-SNE"),
        ("umap", "UMAP"),
        ("x", "X"),
        ("y", "Y"),
        ("z", "Z"),
    ]
    .into_iter()
    .find_map(|(raw, display)| word.eq_ignore_ascii_case(raw).then_some(display))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_labels_admit_aliases_humanization_and_collision_storage() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let cases = [
            ("HTTPResponse_code", "HTTP response code"),
            ("ci95_pca", "CI 95 PCA"),
            ("---", "---"),
            ("a\u{2003}b", "A b"),
            ("\u{df}eta_code", "SSeta code"),
            ("e\u{301}Value", "E\u{301}value"),
            ("m/zValue", "m/z value"),
        ];
        for (raw, expected) in cases {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let data = plot.data().with_axis_resources(&resources);
            assert_eq!(
                Labels::humanized().resolve_source(raw, Some(data)).unwrap(),
                expected
            );
            assert_eq!(Labels::humanized().resolve(raw), expected);
        }
        for labels in [
            Labels::raw(),
            Labels::humanized(),
            Labels::raw().alias("raw", "display"),
        ] {
            for (visits, bytes) in [(0, u64::MAX), (u128::MAX, 0)] {
                let resources =
                    crate::font::canonical::OperationGenerationResources::with_test_limits(
                        visits, bytes,
                    );
                let data = plot.data().with_axis_resources(&resources);
                assert!(matches!(
                    labels.resolve_source("raw", Some(data)),
                    Err(Error::Render(_))
                ));
                assert!(labels
                    .resolve_domain_source("axis", ["raw"], Some(data))
                    .is_err());
            }
        }
        let labels = Labels::raw().alias("a", "same").alias("b", "same");
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        assert_eq!(
            labels
                .resolve_domain_source("axis", ["a", "a"], Some(data))
                .unwrap(),
            ["same", "same"]
        );
        let expected = labels
            .resolve_domain("axis", ["a", "b"])
            .unwrap_err()
            .to_string();
        assert_eq!(
            labels
                .resolve_domain_source("axis", ["a", "b"], Some(data))
                .unwrap_err()
                .to_string(),
            expected
        );
        let calls = std::cell::Cell::new(0);
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(u128::MAX, 0);
        let data = plot.data().with_axis_resources(&resources);
        let result = labels.resolve_domain_source(
            "axis",
            ["a", "b"]
                .into_iter()
                .inspect(|_| calls.set(calls.get() + 1)),
            Some(data),
        );
        assert!(result.is_err());
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn humanized_labels_split_common_identifier_styles() {
        let labels = Labels::humanized();

        assert_eq!(labels.resolve("HTTPResponse_code"), "HTTP response code");
    }

    #[test]
    fn aliases_take_precedence_over_humanization() {
        let labels = Labels::humanized().alias("petal_length", "Petal length (cm)");

        assert_eq!(labels.resolve("petal_length"), "Petal length (cm)");
    }

    #[test]
    fn distinct_raw_labels_cannot_share_one_display_in_a_domain() {
        let error = Labels::humanized()
            .resolve_domain("x-axis categories", ["control_group", "control-group"])
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("label collision in x-axis categories"));
    }

    #[test]
    fn alias_collisions_explain_that_raw_mode_does_not_override_aliases() {
        let error = Labels::raw()
            .alias("a", "same")
            .alias("b", "same")
            .resolve_domain("legend", ["a", "b"])
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("Labels::raw() does not override explicit aliases"));
    }
}
