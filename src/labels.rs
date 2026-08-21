//! Presentation policy for automatically sourced plot labels.

use std::collections::BTreeMap;

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
    aliases: BTreeMap<String, String>,
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
            aliases: BTreeMap::new(),
        }
    }

    /// Render automatically sourced labels exactly as represented by the plot data.
    #[must_use]
    pub const fn raw() -> Self {
        Self {
            mode: LabelMode::Raw,
            aliases: BTreeMap::new(),
        }
    }

    /// Add one exact raw-identity to display-label mapping.
    #[must_use]
    pub fn alias(mut self, raw: impl Into<String>, display: impl Into<String>) -> Self {
        self.aliases.insert(raw.into(), display.into());
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
        self.aliases.extend(
            aliases
                .into_iter()
                .map(|(raw, display)| (raw.into(), display.into())),
        );
        self
    }

    pub(crate) fn resolve(&self, raw: &str) -> String {
        self.aliases.get(raw).cloned().unwrap_or_else(|| {
            if self.mode == LabelMode::Humanized {
                humanize(raw)
            } else {
                raw.to_owned()
            }
        })
    }

    pub(crate) fn resolve_domain<'a>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<String>, Error> {
        let mut displays = BTreeMap::<String, String>::new();
        let mut resolved = Vec::new();
        for raw in raw_labels {
            let display = self.resolve(raw);
            if let Some(previous) = displays.get(&display).filter(|previous| *previous != raw) {
                let alias_guidance = if self.aliases.contains_key(previous)
                    || self.aliases.contains_key(raw)
                {
                    "change or remove the colliding aliases; Labels::raw() does not override explicit aliases"
                } else {
                    "configure an explicit alias or use Labels::raw()"
                };
                return Err(Error::Render(format!(
                    "label collision in {domain}: raw identities {previous:?} and {raw:?} both resolve to {display:?}; {alias_guidance}"
                )));
            }
            displays.insert(display.clone(), raw.to_owned());
            resolved.push(display);
        }
        Ok(resolved)
    }
}

fn humanize(raw: &str) -> String {
    let split = split_words(raw);
    let words = split.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        return raw.to_owned();
    }
    words
        .iter()
        .enumerate()
        .map(|(index, word)| humanize_word(word, index == 0))
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_words(raw: &str) -> String {
    let chars = raw.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(raw.len());
    let mut last_was_space = true;
    for (index, &character) in chars.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(index + 1).copied();
        if matches!(character, '_' | '-' | '.' | ' ' | '\t' | '\n' | '\r') {
            if !last_was_space {
                output.push(' ');
                last_was_space = true;
            }
            continue;
        }
        if should_split_before(previous, character, next) && !last_was_space {
            output.push(' ');
        }
        output.push(character);
        last_was_space = false;
    }
    output
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

fn humanize_word(word: &str, first: bool) -> String {
    let lower = word.to_ascii_lowercase();
    if let Some(acronym) = acronym(&lower) {
        return acronym.to_owned();
    }
    if word.len() > 1 && word.chars().all(|character| character.is_ascii_uppercase()) {
        return word.to_owned();
    }
    if word.chars().all(|character| character.is_ascii_digit()) {
        return word.to_owned();
    }
    let mut characters = lower.chars();
    let Some(initial) = characters.next() else {
        return String::new();
    };
    if first {
        let mut output = initial.to_uppercase().collect::<String>();
        output.push_str(characters.as_str());
        output
    } else {
        lower
    }
}

fn acronym(lower: &str) -> Option<&'static str> {
    match lower {
        "ci" => Some("CI"),
        "dna" => Some("DNA"),
        "hwe" => Some("HWE"),
        "id" => Some("ID"),
        "ms" => Some("MS"),
        "mz" | "m/z" => Some("m/z"),
        "pca" => Some("PCA"),
        "qc" => Some("QC"),
        "rna" => Some("RNA"),
        "tsne" | "t-sne" => Some("t-SNE"),
        "umap" => Some("UMAP"),
        "x" => Some("X"),
        "y" => Some("Y"),
        "z" => Some("Z"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
