//! Presentation policy for automatically sourced plot labels.

use std::collections::BTreeMap;
use std::sync::Arc;

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

    /// Resolve one raw name with alias precedence and the selected fallback.
    ///
    /// The result uses the shared display-text syntax, preserving explicit alias
    /// whitespace and mandatory breaks. Collisions between different identities
    /// are checked when the renderer resolves their complete semantic domain.
    ///
    /// ```
    /// use lichtung_plot::Labels;
    /// let labels = Labels::humanized().alias("mean_score", " Score\r\n(points) ");
    /// assert_eq!(labels.resolve("mean_score")?, " Score\r\n(points) ");
    /// assert_eq!(labels.resolve("control_group")?, "Control group");
    /// assert_eq!(Labels::raw().resolve("control_group")?, "control_group");
    /// assert!(labels.alias("bad", "a\tb").resolve("bad").is_err());
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    pub fn resolve(&self, raw: &str) -> Result<String, Error> {
        let display =
            if let Some(alias) = self.aliases.as_ref().and_then(|aliases| aliases.get(raw)) {
                copy_label(alias)
            } else if self.mode == LabelMode::Humanized {
                humanize(raw)
            } else {
                copy_label(raw)
            }?;
        crate::text::validate_for(&display, "label")?;
        Ok(display)
    }

    pub(crate) fn resolve_domain<'a>(
        &self,
        domain: &str,
        raw_labels: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<String>, Error> {
        let mut identities = Vec::new();
        let mut resolved = Vec::new();
        let raw_labels = raw_labels.into_iter();
        let minimum = raw_labels.size_hint().0;
        crate::data::source::reserve_source(&mut identities, minimum)?;
        crate::data::source::reserve_source(&mut resolved, minimum)?;
        for raw in raw_labels {
            crate::data::source::reserve_source(&mut identities, 1)?;
            crate::data::source::reserve_source(&mut resolved, 1)?;
            let display = self.resolve(raw)?;
            let mut collision = None;
            for (&previous, prior) in identities.iter().zip(&resolved) {
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

fn copy_label(raw: &str) -> Result<String, Error> {
    let mut output = String::new();
    reserve_label(&mut output, raw.len())?;
    output.push_str(raw);
    Ok(output)
}

fn reserve_label(output: &mut String, count: usize) -> Result<(), Error> {
    crate::data::source::reserve_source_string(output, count)
}

fn humanize(raw: &str) -> Result<String, Error> {
    let mut split = String::new();
    reserve_label(
        &mut split,
        raw.len()
            .checked_mul(2)
            .ok_or_else(|| Error::Render("humanized label length overflow".into()))?,
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

    for (index, word) in split.split_whitespace().enumerate() {
        let acronym = acronym(word);
        reserve_label(
            &mut output,
            word.len()
                .checked_add(12)
                .ok_or_else(|| Error::Render("humanized word length overflow".into()))?,
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
        copy_label(raw)
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
