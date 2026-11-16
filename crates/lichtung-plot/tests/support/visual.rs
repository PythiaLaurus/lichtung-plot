#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::NumericValues;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn fixture_values() -> (NumericValues, NumericValues) {
    let xs: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.5).collect();
    let ys: Vec<f64> = (0..30)
        .map(|i| (f64::from(i) * 0.3).sin() * 5.0 + 10.0)
        .collect();
    (NumericValues::new("x", xs), NumericValues::new("y", ys))
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
    pub(super) content: String,
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
                content: content.to_string(),
            })
        })
        .collect()
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
