use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::{fail, Result, LANGUAGES};

pub struct Case {
    pub id: String,
    pub example: String,
    pub artifacts: Vec<String>,
    pub target: String,
    pub api: String,
    pub featured: bool,
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

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
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
        let [id, example, artifacts, target, api, placement, title_en, title_zh, purpose_en, purpose_zh] =
            fields.as_slice()
        else {
            return Err(fail(
                "gallery.tsv requires ten nonempty tab-separated columns",
            ));
        };
        let featured = match *placement {
            "showcase" => true,
            "reference" => false,
            _ => return Err(fail(format!("unknown gallery placement: {placement}"))),
        };
        if fields.iter().any(|field| field.trim().is_empty())
            || !identifier(id)
            || !identifier(target)
            || !filename(example)
            || !filename(api)
            || !ids.insert(*id)
            || !examples.insert(*example)
        {
            return Err(fail(format!("invalid or duplicate gallery case: {id}")));
        }
        let artifacts = artifacts.split(',').map(str::to_owned).collect::<Vec<_>>();
        if artifacts.iter().any(|name| {
            !filename(name) || Path::new(name).extension().is_none_or(|ext| ext != "svg")
        }) || artifacts.iter().collect::<BTreeSet<_>>().len() != artifacts.len()
        {
            return Err(fail(format!("invalid SVG artifact paths: {id}")));
        }
        cases.push(Case {
            id: (*id).into(),
            example: (*example).into(),
            artifacts,
            target: (*target).into(),
            api: (*api).into(),
            featured,
            titles: [(*title_en).into(), (*title_zh).into()],
            purposes: [(*purpose_en).into(), (*purpose_zh).into()],
        });
    }
    if !cases.iter().any(|case| case.featured) {
        return Err(fail("the gallery must have executable showcase cases"));
    }
    Ok(cases)
}

fn links(text: &str) -> Vec<&str> {
    let markdown = text
        .split("](")
        .skip(1)
        .filter_map(|tail| tail.split_once(')').map(|(link, _)| link));
    let html = ["href=\"", "src=\""].into_iter().flat_map(|prefix| {
        text.split(prefix)
            .skip(1)
            .filter_map(|tail| tail.split_once('"').map(|(link, _)| link))
    });
    markdown.chain(html).collect()
}

fn includes(text: &str) -> Result<Vec<&str>> {
    text.split("{{#include ")
        .skip(1)
        .map(|tail| {
            tail.split_once("}}")
                .map(|(reference, _)| reference.trim())
                .ok_or_else(|| fail("unclosed source include"))
        })
        .collect()
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

pub fn navigation(text: &str, language: usize, target: &str) -> Result<()> {
    let peer = LANGUAGES[1 - language];
    if !links(text).contains(&format!("../{peer}/index.html#{target}").as_str()) {
        return Err(fail(format!(
            "{target} must link to its peer {peer} tutorial anchor"
        )));
    }
    Ok(())
}

pub fn rendered_anchors(html: &str) -> Result<BTreeSet<&str>> {
    let mut ids = BTreeSet::new();
    for tail in html.split(" id=\"").skip(1) {
        let (id, _) = tail
            .split_once('"')
            .ok_or_else(|| fail("unclosed HTML id"))?;
        if !ids.insert(id) {
            return Err(fail(format!("ambiguous rendered tutorial anchor: {id}")));
        }
    }
    if html.contains("{{#include ") {
        return Err(fail(
            "mdBook left an unexpanded source include in the guide",
        ));
    }
    Ok(ids)
}

#[derive(Default, PartialEq, Eq)]
struct Outline {
    order: Vec<String>,
    sources: BTreeMap<String, BTreeSet<String>>,
    assets: BTreeMap<String, BTreeSet<String>>,
}

impl Outline {
    fn topic(&mut self, id: &str) -> Result<()> {
        if !identifier(id) || self.sources.contains_key(id) {
            return Err(fail(format!("invalid or duplicate tutorial anchor: {id}")));
        }
        self.order.push(id.into());
        self.sources.insert(id.into(), BTreeSet::new());
        self.assets.insert(id.into(), BTreeSet::new());
        Ok(())
    }
}

fn source(root: &Path, directory: &Path, reference: &str) -> Result<String> {
    let (file, name) = reference
        .rsplit_once(':')
        .ok_or_else(|| fail("example includes require semantic source anchors"))?;
    if !identifier(name) {
        return Err(fail(format!("invalid semantic anchor: {reference}")));
    }
    let examples = root.join("crates/lichtung-plot/examples").canonicalize()?;
    let file = directory.join(file).canonicalize()?;
    if !file.starts_with(&examples) || file.extension().is_none_or(|ext| ext != "rs") {
        return Err(fail(format!(
            "include must refer to an owned example: {reference}"
        )));
    }
    anchor(&fs::read_to_string(&file)?, name)?;
    Ok(format!("{}:{name}", file.strip_prefix(examples)?.display()))
}

fn scan(root: &Path, file: &Path, text: &str, outline: &mut Outline) -> Result<()> {
    let mut current = None;
    for line in text.lines() {
        if let Some(id) = line
            .trim()
            .strip_prefix("<a id=\"")
            .and_then(|tail| tail.strip_suffix("\"></a>"))
        {
            outline.topic(id)?;
            current = Some(id.to_owned());
        }
        for reference in includes(line)? {
            let topic = current
                .as_ref()
                .ok_or_else(|| fail("example source needs a tutorial anchor"))?;
            let identity = source(
                root,
                file.parent().ok_or_else(|| fail("fragment parent"))?,
                reference,
            )?;
            outline
                .sources
                .get_mut(topic)
                .expect("registered topic")
                .insert(identity);
        }
        for asset in links(line)
            .into_iter()
            .filter(|link| link.starts_with("gallery/"))
        {
            let topic = current
                .as_ref()
                .ok_or_else(|| fail("figure needs a tutorial anchor"))?;
            outline
                .assets
                .get_mut(topic)
                .expect("registered topic")
                .insert(asset.into());
        }
    }
    Ok(())
}

fn local_links(root: &Path, text: &str, outline: &Outline, cases: &[Case]) -> Result<()> {
    for link in links(text) {
        if link.starts_with("https://") {
            continue;
        }
        let fragment = link.strip_prefix('#').or_else(|| {
            LANGUAGES
                .iter()
                .find_map(|language| link.strip_prefix(&format!("../{language}/index.html#")))
        });
        if let Some(fragment) = fragment {
            if outline.sources.contains_key(fragment) {
                continue;
            }
        } else if let Some(file) = link.strip_prefix("examples/") {
            if !file.contains("..")
                && root
                    .join("crates/lichtung-plot/examples")
                    .join(file)
                    .is_file()
            {
                continue;
            }
        } else if cases.iter().any(|case| {
            case.artifacts
                .iter()
                .any(|file| link == format!("gallery/{}/{file}", case.id))
        }) {
            continue;
        }
        return Err(fail(format!("missing or out-of-guide link: {link}")));
    }
    Ok(())
}

fn edition(root: &Path, language: usize, cases: &[Case]) -> Result<Outline> {
    let directory = root.join("docs").join(LANGUAGES[language]).join("src");
    if links(&fs::read_to_string(directory.join("SUMMARY.md"))?) != ["index.md"] {
        return Err(fail(
            "the main guide must be one page, not separate tutorial chapters",
        ));
    }
    let main = fs::read_to_string(directory.join("index.md"))?;
    navigation(&main, language, "gallery")?;
    let references = includes(&main)?;
    if references.first() != Some(&"generated/gallery.md") {
        return Err(fail(
            "the guide must show its generated gallery before tutorial sections",
        ));
    }
    let mut outline = Outline::default();
    for line in main.lines() {
        if let Some(id) = line
            .trim()
            .strip_prefix("<a id=\"")
            .and_then(|tail| tail.strip_suffix("\"></a>"))
        {
            outline.topic(id)?;
        }
    }
    if outline.order.first().map(String::as_str) != Some("gallery") {
        return Err(fail(
            "the guide needs a stable gallery anchor before its tutorial",
        ));
    }
    let mut fragments = Vec::new();
    for reference in references.into_iter().skip(1) {
        let name = reference
            .strip_prefix("sections/")
            .ok_or_else(|| fail("tutorial fragments belong in sections/"))?;
        if !filename(name) || Path::new(name).extension().is_none_or(|ext| ext != "md") {
            return Err(fail(format!("invalid tutorial fragment: {reference}")));
        }
        let file = directory.join(reference);
        let text = fs::read_to_string(&file)?;
        let before = outline.order.len();
        scan(root, &file, &text, &mut outline)?;
        let first = outline
            .order
            .get(before)
            .ok_or_else(|| fail("fragment needs an explicit tutorial anchor"))?;
        navigation(&text, language, first)?;
        fragments.push(text);
    }
    local_links(root, &main, &outline, cases)?;
    for text in &fragments {
        local_links(root, text, &outline, cases)?;
    }
    for case in cases {
        let uses_example = outline.sources.get(&case.target).is_some_and(|sources| {
            sources
                .iter()
                .any(|source| source.starts_with(&format!("{}.rs:", case.example)))
        });
        let shows_figure = outline.assets.get(&case.target).is_some_and(|assets| {
            assets.contains(&format!("gallery/{}/{}", case.id, case.artifacts[0]))
        });
        if !uses_example || !shows_figure {
            return Err(fail(format!(
                "{} must link directly to a walkthrough using its own figure and example source",
                case.id
            )));
        }
    }
    Ok(outline)
}

pub fn check(root: &Path) -> Result<Vec<Case>> {
    let cases = catalog(&fs::read_to_string(root.join("docs/gallery.tsv"))?)?;
    let english = edition(root, 0, &cases)?;
    let chinese = edition(root, 1, &cases)?;
    if english != chinese {
        return Err(fail(
            "peer guides must cover the same tutorial anchors, figures and source excerpts",
        ));
    }
    println!("Checked peer single-page guides: {} shared anchors, {} showcase walkthroughs and {} reference workflows. Prose equivalence still requires review.", english.order.len(), cases.iter().filter(|case| case.featured).count(), cases.iter().filter(|case| !case.featured).count());
    Ok(cases)
}

#[cfg(test)]
mod tests;
