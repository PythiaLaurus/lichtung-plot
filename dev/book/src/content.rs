use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use crate::{fail, Result, LANGUAGES};

pub struct Case {
    pub id: String,
    pub example: String,
    pub artifacts: Vec<String>,
    pub chapter: String,
    pub api: String,
    pub titles: [String; 2],
    pub purposes: [String; 2],
}

fn filename(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
}

pub fn catalog(text: &str) -> Result<Vec<Case>> {
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    let mut examples = BTreeSet::new();
    for line in text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [id, example, artifacts, chapter, api, title_en, title_zh, purpose_en, purpose_zh] =
            fields.as_slice()
        else {
            return Err(fail(
                "gallery.tsv requires nine nonempty tab-separated columns",
            ));
        };
        if fields.iter().any(|field| field.trim().is_empty())
            || ![id, example, chapter, api]
                .into_iter()
                .all(|field| filename(field))
            || Path::new(chapter)
                .extension()
                .is_none_or(|extension| extension != "md")
            || !ids.insert(*id)
            || !examples.insert(*example)
        {
            return Err(fail(format!("invalid or duplicate gallery case: {id}")));
        }
        let artifacts = artifacts.split(',').map(str::to_owned).collect::<Vec<_>>();
        if artifacts.iter().any(|name| {
            !filename(name)
                || Path::new(name)
                    .extension()
                    .is_none_or(|extension| extension != "svg")
        }) || artifacts.iter().collect::<BTreeSet<_>>().len() != artifacts.len()
        {
            return Err(fail(format!("invalid SVG artifact paths: {id}")));
        }
        cases.push(Case {
            id: (*id).into(),
            example: (*example).into(),
            artifacts,
            chapter: (*chapter).into(),
            api: (*api).into(),
            titles: [(*title_en).into(), (*title_zh).into()],
            purposes: [(*purpose_en).into(), (*purpose_zh).into()],
        });
    }
    if cases.is_empty() {
        return Err(fail("the gallery must have executable cases"));
    }
    Ok(cases)
}

fn links(text: &str) -> Vec<&str> {
    text.split("](")
        .skip(1)
        .filter_map(|tail| tail.split_once(')').map(|(link, _)| link))
        .collect()
}

fn chapters(summary: &str) -> Result<Vec<String>> {
    let chapters = links(summary)
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if chapters.is_empty()
        || chapters.iter().any(|name| {
            !filename(name)
                || Path::new(name)
                    .extension()
                    .is_none_or(|extension| extension != "md")
        })
        || chapters.iter().collect::<BTreeSet<_>>().len() != chapters.len()
    {
        return Err(fail(
            "SUMMARY.md must contain unique, named Markdown chapters",
        ));
    }
    Ok(chapters)
}

pub fn anchor(source: &str, name: &str) -> Result<()> {
    let begin = format!("// ANCHOR: {name}");
    let end = format!("// ANCHOR_END: {name}");
    let starts = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == begin)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let ends = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == end)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if !matches!((starts.as_slice(), ends.as_slice()), ([start], [end]) if start + 1 < *end) {
        return Err(fail(format!(
            "missing, duplicate or empty source anchor: {name}"
        )));
    }
    Ok(())
}

fn includes(root: &Path, directory: &Path, text: &str) -> Result<BTreeSet<String>> {
    let mut includes = BTreeSet::new();
    let examples = root.join("crates/lichtung-plot/examples").canonicalize()?;
    for tail in text.split("{{#include ").skip(1) {
        let (reference, _) = tail
            .split_once("}}")
            .ok_or_else(|| fail("unclosed source include"))?;
        let (file, name) = reference
            .trim()
            .rsplit_once(':')
            .ok_or_else(|| fail("source includes require semantic anchors, not line numbers"))?;
        if name.chars().any(|c| !c.is_ascii_alphabetic() && c != '-') || name.is_empty() {
            return Err(fail(format!("invalid semantic anchor: {reference}")));
        }
        let source = directory.join(file).canonicalize()?;
        if !source.starts_with(&examples)
            || source.extension().is_none_or(|extension| extension != "rs")
        {
            return Err(fail(format!(
                "include must refer to an owned runnable example: {reference}"
            )));
        }
        anchor(&fs::read_to_string(source)?, name)?;
        includes.insert(reference.trim().to_owned());
    }
    Ok(includes)
}

pub fn navigation(text: &str, language: usize, chapter: &str) -> Result<()> {
    let peer = LANGUAGES[1 - language];
    let stem = chapter
        .strip_suffix(".md")
        .ok_or_else(|| fail("chapter extension"))?;
    if !links(text).contains(&format!("../{peer}/{stem}.html").as_str()) {
        return Err(fail(format!(
            "{chapter} must link to its peer {peer} chapter"
        )));
    }
    Ok(())
}

fn local_links(directory: &Path, text: &str, chapters: &[String], cases: &[Case]) -> Result<()> {
    for link in links(text) {
        if link.starts_with("https://") || link.starts_with('#') {
            continue;
        }
        if let Some(peer) = LANGUAGES
            .iter()
            .find(|language| link.starts_with(&format!("../{language}/")))
        {
            let stem = link
                .strip_prefix(&format!("../{peer}/"))
                .and_then(|p| p.strip_suffix(".html"));
            if stem.is_some_and(|stem| chapters.contains(&format!("{stem}.md"))) {
                continue;
            }
        }
        if cases
            .iter()
            .any(|case| link == format!("gallery-{}.md", case.id))
        {
            continue;
        }
        let destination = link.split('#').next().unwrap_or_default();
        if !Path::new(destination)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
            || !directory.join(destination).is_file()
        {
            return Err(fail(format!("missing or out-of-book link: {link}")));
        }
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<Vec<Case>> {
    let cases = catalog(&fs::read_to_string(root.join("docs/gallery.tsv"))?)?;
    let mut edition_chapters = Vec::new();
    let mut edition_includes = Vec::new();
    for (index, language) in LANGUAGES.iter().enumerate() {
        let directory = root.join("docs").join(language).join("src");
        let chapters = chapters(&fs::read_to_string(directory.join("SUMMARY.md"))?)?;
        let mut sources = Vec::new();
        for chapter in &chapters {
            let text = fs::read_to_string(directory.join(chapter))?;
            navigation(&text, index, chapter)?;
            local_links(&directory, &text, &chapters, &cases)?;
            sources.push(includes(root, &directory, &text)?);
        }
        for case in &cases {
            if !chapters.contains(&case.chapter)
                || !root
                    .join(format!("crates/lichtung-plot/examples/{}.rs", case.example))
                    .is_file()
            {
                return Err(fail(format!(
                    "missing chapter or executable source for {}",
                    case.id
                )));
            }
        }
        edition_chapters.push(chapters);
        edition_includes.push(sources);
    }
    if edition_chapters[0] != edition_chapters[1] || edition_includes[0] != edition_includes[1] {
        return Err(fail(
            "peer editions must have identical chapter identities/order and source includes",
        ));
    }
    println!("Checked both editions: {} peer chapters and {} executable gallery cases. Prose equivalence still requires review.", edition_chapters[0].len(), cases.len());
    Ok(cases)
}

#[cfg(test)]
mod tests;
