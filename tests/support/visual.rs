#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::{BarInput, BarSpacing, CategoryValues, NumericValues, Plot};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn fixture_values() -> (NumericValues, NumericValues) {
    let xs: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.5).collect();
    let ys: Vec<f64> = (0..30)
        .map(|i| (f64::from(i) * 0.3).sin() * 5.0 + 10.0)
        .collect();
    (NumericValues::new("x", xs), NumericValues::new("y", ys))
}

pub(super) fn normalize_embedded_font(svg: &str) -> String {
    let prefix = "data:font/woff2;base64,";
    let Some(start) = svg.find(prefix) else {
        return svg.to_owned();
    };
    let data_start = start + prefix.len();
    let Some(length) = svg[data_start..].find('\'') else {
        return svg.to_owned();
    };
    let mut normalized = svg.to_owned();
    normalized.replace_range(data_start..data_start + length, "<subset>");
    normalized
}

pub(super) fn specialized_svg_structure(svg: &str, selected_text: &[&str]) -> String {
    let mut output = Vec::new();
    let mut mark_counts: Option<[usize; 5]> = None;
    for line in svg.lines().map(str::trim) {
        if let Some(counts) = mark_counts.as_mut() {
            if line == "</g>" {
                output.push(format!(
                    "MARKS circle={} line={} polygon={} polyline={} text={}",
                    counts[0], counts[1], counts[2], counts[3], counts[4]
                ));
                output.push("END MARKS".to_string());
                mark_counts = None;
            } else if line.starts_with("<circle ") {
                counts[0] += 1;
            } else if line.starts_with("<line ") {
                counts[1] += 1;
            } else if line.starts_with("<polygon ") {
                counts[2] += 1;
            } else if line.starts_with("<polyline ") {
                counts[3] += 1;
            } else if line.starts_with("<text ") {
                counts[4] += 1;
            }
            continue;
        }

        if line.starts_with("<clipPath id=") {
            output.push(line.to_string());
        } else if line.starts_with("<g clip-path=") {
            output.push(line.to_string());
            mark_counts = Some([0; 5]);
        } else if line.starts_with("<g id=\"grid\"")
            || line.starts_with("<g id=\"axes\"")
            || line.starts_with("<g id=\"legend\"")
            || selected_text
                .iter()
                .any(|text| line.contains(&format!(">{text}</text>")))
        {
            output.push(line.to_string());
        }
    }
    output.join("\n")
}

pub(super) fn tempdir() -> std::path::PathBuf {
    create_unique_dir("lichtung-plot-test")
}

pub(super) fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]));
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (width, height)
}

pub(super) fn png_phys(bytes: &[u8]) -> Option<(u32, u32, u8)> {
    let mut index = 8;
    while index + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[index..index + 4].try_into().unwrap()) as usize;
        let chunk_type = &bytes[index + 4..index + 8];
        let data_start = index + 8;
        let data_end = data_start + length;
        if data_end + 4 > bytes.len() {
            return None;
        }
        if chunk_type == b"pHYs" && length == 9 {
            let xppu = u32::from_be_bytes(bytes[data_start..data_start + 4].try_into().unwrap());
            let yppu =
                u32::from_be_bytes(bytes[data_start + 4..data_start + 8].try_into().unwrap());
            return Some((xppu, yppu, bytes[data_start + 8]));
        }
        index = data_end + 4;
    }
    None
}

/// Helper: create a uniquely-named temp directory
/// for the test's output artifacts.
pub(super) fn unique_dir(prefix: &str) -> std::path::PathBuf {
    create_unique_dir(prefix)
}

fn create_unique_dir(prefix: &str) -> std::path::PathBuf {
    loop {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("{prefix}-{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&directory) {
            Ok(()) => return directory,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("tempdir: {error}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgRect {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

#[derive(Debug)]
pub(super) struct SvgTextElement {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) font_size: f64,
    pub(super) content: String,
}

impl SvgRect {
    pub(super) fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub(super) fn contains(self, other: Self) -> bool {
        other.x >= self.x
            && other.x + other.width <= self.x + self.width
            && other.y >= self.y
            && other.y + other.height <= self.y + self.height
    }
}

pub(super) fn parse_legend_rect(svg: &str) -> (f64, f64, f64, f64) {
    let (rect, _) = parse_legend_rect_and_start(svg);
    (rect.x, rect.y, rect.width, rect.height)
}

pub(super) fn parse_first_clip_rect(svg: &str) -> SvgRect {
    let clip_start = svg.find("<clipPath id=").expect("clip path should exist");
    let relative_rect_start = svg[clip_start..]
        .find("<rect ")
        .expect("clip path should contain a rect");
    let rect_start = clip_start + relative_rect_start;
    let rect_end = rect_start
        + svg[rect_start..]
            .find("/>")
            .expect("clip rect tag should be closed")
        + 2;
    let tag = &svg[rect_start..rect_end];
    SvgRect {
        x: parse_svg_attr(tag, "x"),
        y: parse_svg_attr(tag, "y"),
        width: parse_svg_attr(tag, "width"),
        height: parse_svg_attr(tag, "height"),
    }
}

pub(super) fn parse_legend_svg_rect(svg: &str) -> SvgRect {
    parse_legend_rect_and_start(svg).0
}

pub(super) fn has_legend(svg: &str) -> bool {
    svg.contains("<g id=\"legend\">")
}

pub(super) fn parse_legend_rect_and_start(svg: &str) -> (SvgRect, usize) {
    let mut search_offset = 0;
    while let Some(relative_start) = svg[search_offset..].find("<rect x=\"") {
        let start = search_offset + relative_start;
        let tag_end = start
            + svg[start..]
                .find("/>")
                .expect("legend rect tag should be closed")
            + 2;
        let tag = &svg[start..tag_end];
        if tag.contains("fill-opacity=\"0.80\"") && tag.contains("stroke=\"#C8C8C8\"") {
            return (
                SvgRect {
                    x: parse_svg_attr(tag, "x"),
                    y: parse_svg_attr(tag, "y"),
                    width: parse_svg_attr(tag, "width"),
                    height: parse_svg_attr(tag, "height"),
                },
                start,
            );
        }
        search_offset = tag_end;
    }
    panic!("legend background rect");
}

pub(super) fn parse_svg_attr(tag: &str, attr: &str) -> f64 {
    let needle = format!("{attr}=\"");
    let (_, rest) = tag.split_once(&needle).expect("svg attr exists");
    let (value, _) = rest.split_once('"').expect("svg attr is quoted");
    value.parse().expect("svg attr is numeric")
}

pub(super) fn parse_text_elements(svg: &str) -> Vec<SvgTextElement> {
    svg.split("<text ")
        .skip(1)
        .filter_map(|segment| {
            let (attrs, rest) = segment.split_once('>')?;
            let (content, _) = rest.split_once("</text>")?;
            Some(SvgTextElement {
                x: parse_svg_attr(attrs, "x"),
                y: parse_svg_attr(attrs, "y"),
                font_size: parse_svg_attr(attrs, "font-size"),
                content: content.to_string(),
            })
        })
        .collect()
}

pub(super) fn numeric_text_values<'a>(text: impl Iterator<Item = &'a SvgTextElement>) -> Vec<f64> {
    text.filter_map(|entry| entry.content.parse::<f64>().ok())
        .collect()
}

pub(super) fn text_element<'a>(text: &'a [SvgTextElement], content: &str) -> &'a SvgTextElement {
    text.iter()
        .find(|entry| entry.content == content)
        .unwrap_or_else(|| panic!("text element {content:?} should exist"))
}

pub(super) fn text_y(text: &[SvgTextElement], content: &str) -> f64 {
    text_element(text, content).y
}

pub(super) fn grouped_bar_internal_gap(spacing: BarSpacing) -> f64 {
    let group = CategoryValues::new("group", ["A", "A"]).unwrap();
    let series = CategoryValues::new("series", ["control", "treatment"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let dir = unique_dir("grouped-bar-spacing");
    let out = dir.join("bar.svg");
    let chart = Plot::bar(group, BarInput::estimates(value))
        .series(series)
        .spacing(spacing)
        .size(360, 260);
    chart
        .svg_preserve_text(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let mut bars = parse_polygon_bounds(&svg);
    bars.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(
        bars.len(),
        2,
        "single category × 2 series levels should draw 2 bars"
    );
    bars[1].x - (bars[0].x + bars[0].width)
}

pub(super) fn assert_legend_does_not_overlap_graphics(svg: &str, plot_type: &str) {
    let (legend_rect, legend_start) = parse_legend_rect_and_start(svg);
    let graphic_layer = remove_named_layer(&svg[..legend_start], "grid");
    let graphic_bounds = parse_graphic_bounds(&graphic_layer);
    assert!(
        !graphic_bounds.is_empty(),
        "{plot_type} overlap test should detect rendered graphic marks before the legend"
    );
    let overlaps: Vec<SvgRect> = graphic_bounds
        .into_iter()
        .filter(|bounds| bounds.intersects(legend_rect))
        .collect();
    assert!(
        overlaps.is_empty(),
        "{plot_type} legend should not cover graphic marks; legend={legend_rect:?}, overlaps={overlaps:?}"
    );
}

pub(super) fn legend_gap_from_graphics(svg: &str) -> f64 {
    let (legend_rect, legend_start) = parse_legend_rect_and_start(svg);
    let graphic_layer = remove_named_layer(&svg[..legend_start], "grid");
    let graphic_bounds = parse_graphic_bounds(&graphic_layer);
    let graphics_right = graphic_bounds
        .iter()
        .map(|bounds| bounds.x + bounds.width)
        .fold(f64::NEG_INFINITY, f64::max);
    legend_rect.x - graphics_right
}

pub(super) fn assert_inside_legend_clear(svg: &str, panel: SvgRect, plot_type: &str) {
    let (legend_rect, _) = parse_legend_rect_and_start(svg);
    assert!(
        panel.contains(legend_rect),
        "{plot_type} inside legend should stay within target axes; panel={panel:?}, legend={legend_rect:?}"
    );
    assert_legend_does_not_overlap_graphics(svg, plot_type);
}

pub(super) fn assert_outside_legend_clear(svg: &str, panel: SvgRect, plot_type: &str) {
    let (legend, _) = parse_legend_rect_and_start(svg);
    let root = SvgRect {
        x: 0.0,
        y: 0.0,
        width: parse_svg_attr(svg, "width"),
        height: parse_svg_attr(svg, "height"),
    };
    assert!(
        root.contains(legend),
        "{plot_type} guide must fit the root: {legend:?}"
    );
    assert!(
        !panel.intersects(legend),
        "{plot_type} automatic guide must be outside the data panel: {legend:?}"
    );
    assert_legend_does_not_overlap_graphics(svg, plot_type);
}

pub(super) fn remove_named_layer(svg: &str, layer: &str) -> String {
    let open = format!("<g id=\"{layer}\">\n");
    let Some(start) = svg.find(&open) else {
        return svg.to_string();
    };
    let content_start = start + open.len();
    let Some(relative_end) = svg[content_start..].find("</g>\n") else {
        return svg.to_string();
    };
    let end = content_start + relative_end + "</g>\n".len();
    let mut cleaned = String::with_capacity(svg.len() - (end - start));
    cleaned.push_str(&svg[..start]);
    cleaned.push_str(&svg[end..]);
    cleaned
}

pub(super) fn remove_legend_layers(svg: &str) -> String {
    let mut cleaned = svg.to_owned();
    while let Some((start, content_start)) =
        cleaned
            .match_indices("<g id=\"")
            .find_map(|(start, prefix)| {
                let id_start = start + prefix.len();
                let id_end = cleaned[id_start..].find('"')? + id_start;
                let content_start = cleaned[id_end..].find('>')? + id_end + 1;
                cleaned[id_start..id_end]
                    .contains("legend")
                    .then_some((start, content_start))
            })
    {
        let Some(relative_end) = cleaned[content_start..].find("</g>") else {
            break;
        };
        let end = content_start + relative_end + "</g>".len();
        cleaned.replace_range(start..end, "");
    }
    cleaned
}

pub(super) fn parse_graphic_bounds(svg: &str) -> Vec<SvgRect> {
    let mut bounds = parse_circle_bounds(svg);
    bounds.extend(parse_polyline_bounds(svg));
    bounds.extend(parse_polygon_bounds(svg));
    bounds
}

pub(super) fn parse_circle_bounds(svg: &str) -> Vec<SvgRect> {
    svg.split("<circle cx=\"")
        .skip(1)
        .filter_map(|segment| {
            let (cx, circle_rest) = segment.split_once("\" cy=\"")?;
            let (cy, circle_rest) = circle_rest.split_once("\" r=\"")?;
            let (r, _) = circle_rest.split_once('"')?;
            let cx = cx.parse::<f64>().ok()?;
            let cy = cy.parse::<f64>().ok()?;
            let r = r.parse::<f64>().ok()?;
            Some(SvgRect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            })
        })
        .collect()
}

pub(super) fn parse_polyline_bounds(svg: &str) -> Vec<SvgRect> {
    svg.split("<polyline points=\"")
        .skip(1)
        .filter_map(|segment| {
            let (points, rest) = segment.split_once('"')?;
            let stroke_width = parse_optional_svg_attr(rest, "stroke-width").unwrap_or(1.0);
            bounds_from_points(points, stroke_width / 2.0)
        })
        .collect()
}

pub(super) fn parse_polygon_bounds(svg: &str) -> Vec<SvgRect> {
    let svg = remove_legend_layers(svg);
    svg.split("<polygon points=\"")
        .skip(1)
        .filter_map(|segment| {
            let (points, rest) = segment.split_once('"')?;
            let stroke_width = parse_optional_svg_attr(rest, "stroke-width").unwrap_or(1.0);
            bounds_from_points(points, stroke_width / 2.0)
        })
        .collect()
}

pub(super) fn parse_optional_svg_attr(tag_tail: &str, attr: &str) -> Option<f64> {
    let needle = format!("{attr}=\"");
    let (_, rest) = tag_tail.split_once(&needle)?;
    let (value, _) = rest.split_once('"')?;
    value.parse().ok()
}

pub(super) fn bounds_from_points(points: &str, pad: f64) -> Option<SvgRect> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut count = 0;
    for point in points.split_whitespace() {
        let (x, y) = point.split_once(',')?;
        let x = x.parse::<f64>().ok()?;
        let y = y.parse::<f64>().ok()?;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        count += 1;
    }
    (count > 0).then_some(SvgRect {
        x: min_x - pad,
        y: min_y - pad,
        width: max_x - min_x + pad * 2.0,
        height: max_y - min_y + pad * 2.0,
    })
}

pub(super) fn parse_circles(svg: &str) -> Vec<(f64, f64)> {
    svg.split("<circle cx=\"")
        .skip(1)
        .filter_map(|segment| {
            let (cx, circle_rest) = segment.split_once("\" cy=\"")?;
            let (cy, _) = circle_rest.split_once('"')?;
            Some((cx.parse::<f64>().ok()?, cy.parse::<f64>().ok()?))
        })
        .collect()
}

pub(super) fn circle_bounds(circles: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    circles.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    )
}
