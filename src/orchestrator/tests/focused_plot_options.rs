use super::{svg_bytes, PanelBox, Plot, PlotKind, RecordingBackend};
use crate::render::backend::{TextAnchor, TextStyle};
use crate::test_support::{DataFrame, Series};
use crate::{CategoricalColors, LinePathStyle, NumericLabelFormat, WaterfallLabels};

fn path_data() -> DataFrame {
    DataFrame::new(
        6,
        vec![
            Series::new("path".into(), ["a", "b", "a", "b", "a", "b"]).into(),
            Series::new(
                "hue".into(),
                ["first", "second", "first", "second", "first", "second"],
            )
            .into(),
            Series::new("x".into(), [2.0_f64, 0.0, 1.0, 1.0, 1.0, 2.0]).into(),
            Series::new("y".into(), [0.0_f64, 0.0, 1.0, 0.5, 2.0, 1.0]).into(),
        ],
    )
    .unwrap()
}

fn trajectory_data() -> DataFrame {
    DataFrame::new(
        6,
        vec![
            Series::new("path".into(), ["a", "b", "a", "b", "a", "b"]).into(),
            Series::new(
                "hue".into(),
                ["first", "second", "first", "second", "first", "second"],
            )
            .into(),
            Series::new("x".into(), [2.0_f64, 0.0, 1.0, 1.0, 0.0, 2.0]).into(),
            Series::new("y".into(), [0.0_f64, 0.0, 1.0, 0.5, 2.0, 1.0]).into(),
        ],
    )
    .unwrap()
}

fn path_chart(data: &DataFrame, style: LinePathStyle) -> Plot<crate::kind::LinePath> {
    Plot::line_path(
        crate::test_support::numeric(data, "x"),
        crate::test_support::numeric(data, "y"),
        crate::test_support::category(data, "path"),
    )
    .hue(crate::test_support::category(data, "hue"))
    .style(style)
}

#[test]
fn hue_path_style_colors_complete_paths_preserves_order_and_draws_line_legends() {
    let data = path_data();
    let chart = path_chart(&data, LinePathStyle::Hue);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::LinePath, &mut backend).unwrap();

    let path_lines = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 1 && (style.width - 1.4).abs() <= f32::EPSILON
        })
        .collect::<Vec<_>>();
    assert_eq!(path_lines.len(), 2);
    assert_ne!(path_lines[0].1.color, path_lines[1].1.color);
    assert!(path_lines[0].0[0].x > path_lines[0].0[1].x);
    assert!((path_lines[0].0[1].x - path_lines[0].0[2].x).abs() <= f64::EPSILON);

    let legend_lines = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 0
                && (style.width - 1.4).abs() <= f32::EPSILON
                && path_lines
                    .iter()
                    .any(|(_, path_style, _)| path_style.color == style.color)
        })
        .collect::<Vec<_>>();
    assert_eq!(legend_lines.len(), 2);
}

#[test]
fn hue_path_style_uses_caller_colors_for_paths_markers_and_legend_lines() {
    let data = path_data();
    let colors = CategoricalColors::new([
        ("first".to_owned(), (51, 51, 51)),
        ("second".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let chart = path_chart(&data, LinePathStyle::Hue).categorical_colors(colors);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::LinePath, &mut backend).unwrap();

    let path_colors = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 1 && (style.width - 1.4).abs() <= f32::EPSILON
        })
        .map(|(_, style, _)| style.color)
        .collect::<Vec<_>>();
    let marker_colors = backend
        .scatter_draws
        .iter()
        .filter(|(_, _, clip_depth)| *clip_depth == 1)
        .map(|(_, style, _)| style.color)
        .collect::<Vec<_>>();
    let legend_colors = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 0 && (style.width - 1.4).abs() <= f32::EPSILON
        })
        .map(|(_, style, _)| style.color)
        .collect::<Vec<_>>();

    assert_eq!(path_colors, [(51, 51, 51), (0, 114, 178)]);
    assert_eq!(
        marker_colors,
        [(51, 51, 51); 3]
            .into_iter()
            .chain([(0, 114, 178); 3])
            .collect::<Vec<_>>()
    );
    assert_eq!(legend_colors, [(51, 51, 51), (0, 114, 178)]);
}

#[test]
fn hue_path_style_splits_one_group_into_observed_hue_paths() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("path".into(), ["a", "a", "a", "a"]).into(),
            Series::new("hue".into(), ["first", "second", "first", "second"]).into(),
            Series::new("x".into(), [0.0_f64, 1.0, 2.0, 3.0]).into(),
            Series::new("y".into(), [0.0_f64, 1.0, 2.0, 3.0]).into(),
        ],
    )
    .unwrap();
    let chart = path_chart(&data, LinePathStyle::Hue);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::LinePath, &mut backend).unwrap();
    let path_colors = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 1 && (style.width - 1.4).abs() <= f32::EPSILON
        })
        .map(|(_, style, _)| style.color)
        .collect::<Vec<_>>();

    assert_eq!(path_colors, [(0, 114, 178), (213, 94, 0)]);
}

#[test]
fn line_only_hue_paths_keep_connectors_order_repeated_x_and_line_legends() {
    let data = path_data();
    let chart = path_chart(&data, LinePathStyle::Hue).markers(false);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::LinePath, &mut backend).unwrap();

    let path_lines = backend
        .line_draws
        .iter()
        .filter(|(_, style, clip_depth)| {
            *clip_depth == 1 && (style.width - 1.4).abs() <= f32::EPSILON
        })
        .collect::<Vec<_>>();
    assert_eq!(path_lines.len(), 2);
    assert_ne!(path_lines[0].1.color, path_lines[1].1.color);
    assert!(path_lines[0].0[0].x > path_lines[0].0[1].x);
    assert!((path_lines[0].0[1].x - path_lines[0].0[2].x).abs() <= f64::EPSILON);
    assert!(backend.scatter_draws.is_empty());
    assert_eq!(
        backend
            .line_draws
            .iter()
            .filter(|(_, style, clip_depth)| {
                *clip_depth == 0
                    && (style.width - 1.4).abs() <= f32::EPSILON
                    && path_lines
                        .iter()
                        .any(|(_, path_style, _)| path_style.color == style.color)
            })
            .count(),
        2
    );
}

#[test]
fn trajectory_always_renders_observation_markers() {
    let data = trajectory_data();
    let chart = Plot::trajectory(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
        crate::test_support::category(&data, "path"),
    )
    .hue(crate::test_support::category(&data, "hue"));
    let mut backend = RecordingBackend::default();

    chart
        .render_kind(PlotKind::Trajectory, &mut backend)
        .unwrap();

    assert_eq!(
        backend
            .scatter_draws
            .iter()
            .filter(|(_, _, clip_depth)| *clip_depth == 1)
            .count(),
        6
    );
}

#[test]
fn hue_path_rejects_constant_hue_absent_from_resolved_order() {
    let data = path_data();
    let chart =
        path_chart(&data, LinePathStyle::Hue).with_global_hue_levels(vec!["unrelated".into()]);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::LinePath, &mut backend)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("absent from the resolved hue order"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn neutral_path_default_is_byte_inert_and_hue_path_bytes_are_deterministic() {
    let data = path_data();
    let default = svg_bytes(path_chart(&data, LinePathStyle::default()));
    let explicit_neutral = svg_bytes(path_chart(&data, LinePathStyle::Neutral));
    let first_hue = svg_bytes(path_chart(&data, LinePathStyle::Hue));
    let second_hue = svg_bytes(path_chart(&data, LinePathStyle::Hue));

    assert_eq!(default, explicit_neutral);
    assert_eq!(first_hue, second_hue);
    assert_ne!(default, first_hue);
}

fn beeswarm_data(colors: &[f64]) -> DataFrame {
    let categories = vec!["feature"; colors.len()];
    let mut next_x = 0.0;
    let xs = colors
        .iter()
        .map(|_| {
            let x = next_x;
            next_x += 1.0;
            x
        })
        .collect::<Vec<_>>();
    DataFrame::new(
        colors.len(),
        vec![
            Series::new("category".into(), categories).into(),
            Series::new("x".into(), xs).into(),
            Series::new("color".into(), colors).into(),
        ],
    )
    .unwrap()
}

fn beeswarm_chart(data: &DataFrame, limits: Option<(f64, f64)>) -> Plot<crate::kind::Beeswarm> {
    let mut chart = Plot::beeswarm(
        crate::test_support::category(data, "category"),
        crate::test_support::numeric(data, "x"),
        crate::test_support::numeric(data, "color"),
    );
    if let Some((minimum, maximum)) = limits {
        chart = chart.color_limits(minimum, maximum);
    }
    chart
}

#[test]
fn explicit_beeswarm_domains_are_shared_exactly_and_saturate_outside_values() {
    let first_data = beeswarm_data(&[-1.0, 0.0, 1.0, 2.0]);
    let second_data = beeswarm_data(&[0.25, 0.75]);
    let mut first_backend = RecordingBackend::default();
    let mut second_backend = RecordingBackend::default();

    beeswarm_chart(&first_data, Some((0.0, 1.0)))
        .render_kind(PlotKind::Beeswarm, &mut first_backend)
        .unwrap();
    beeswarm_chart(&second_data, Some((0.0, 1.0)))
        .render_kind(PlotKind::Beeswarm, &mut second_backend)
        .unwrap();

    assert_eq!(first_backend.gradients, second_backend.gradients);
    let point_colors = first_backend
        .scatter_draws
        .iter()
        .filter(|(_, _, clip_depth)| *clip_depth == 1)
        .map(|(_, style, _)| style.color)
        .collect::<Vec<_>>();
    assert_eq!(point_colors[0], point_colors[1]);
    assert_eq!(point_colors[2], point_colors[3]);
    assert_ne!(point_colors[0], point_colors[2]);
    assert!(first_backend
        .texts
        .iter()
        .any(|(_, _, text)| text == "0.000"));
    assert!(first_backend
        .texts
        .iter()
        .any(|(_, _, text)| text == "1.000"));
}

#[test]
fn invalid_beeswarm_color_limits_fail_before_backend_begin() {
    let data = beeswarm_data(&[0.0, 1.0]);
    for limits in [
        (f64::NAN, 1.0),
        (0.0, f64::INFINITY),
        (1.0, 1.0),
        (2.0, 1.0),
    ] {
        let mut backend = RecordingBackend::default();
        let error = beeswarm_chart(&data, Some(limits))
            .render_kind(PlotKind::Beeswarm, &mut backend)
            .unwrap_err();

        assert!(error.to_string().contains("finite and strictly ascending"));
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn beeswarm_accepts_adjacent_and_opposite_extreme_finite_color_limits() {
    for limits in [(0.0, f64::from_bits(1)), (-f64::MAX, f64::MAX)] {
        let data = beeswarm_data(&[limits.0, limits.1]);
        let mut backend = RecordingBackend::default();

        beeswarm_chart(&data, Some(limits))
            .render_kind(PlotKind::Beeswarm, &mut backend)
            .unwrap();

        let point_colors = backend
            .scatter_draws
            .iter()
            .filter(|(_, _, clip_depth)| *clip_depth == 1)
            .map(|(_, style, _)| style.color)
            .collect::<Vec<_>>();
        assert_ne!(point_colors[0], point_colors[1]);
    }
}

#[test]
fn beeswarm_auto_domain_matches_the_same_explicit_domain_byte_for_byte() {
    let data = beeswarm_data(&[0.1, 1.0]);
    let automatic = svg_bytes(beeswarm_chart(&data, None));
    let explicit = svg_bytes(beeswarm_chart(&data, Some((0.1, 1.0))));

    assert_eq!(automatic, explicit);
}

fn waterfall_data() -> DataFrame {
    DataFrame::new(
        2,
        vec![
            Series::new("step".into(), ["first", "second"]).into(),
            Series::new("delta".into(), [2.0_f64, -1.0]).into(),
            Series::new("base".into(), [10.0_f64, 10.0]).into(),
        ],
    )
    .unwrap()
}

fn waterfall_chart(
    data: &DataFrame,
    labels: Option<WaterfallLabels>,
) -> Plot<crate::kind::Waterfall> {
    let mut chart = Plot::waterfall(
        crate::test_support::category(data, "step"),
        crate::test_support::numeric(data, "delta"),
        crate::test_support::constant_numeric(data, "base"),
    );
    if let Some(labels) = labels {
        chart = chart.total_labels(labels);
    }
    chart
}

#[test]
fn waterfall_labels_are_nonempty_verbatim_and_replace_only_configured_text() {
    for (base, terminal) in [
        ("", "Final prediction"),
        ("Expected output", "  "),
        (" Expected output", "Final prediction"),
        ("Expected output ", "Final prediction"),
        ("Expected\noutput", "Final prediction"),
        ("Expected output", "Final\tprediction"),
    ] {
        assert!(WaterfallLabels::new(base, terminal).is_err());
    }
    let data = waterfall_data();
    let defaults = String::from_utf8(svg_bytes(waterfall_chart(&data, None))).unwrap();
    let labels = WaterfallLabels::new("Expected output", "Final prediction").unwrap();
    assert_eq!(labels.base(), "Expected output");
    assert_eq!(labels.terminal(), "Final prediction");
    let custom_chart = waterfall_chart(&data, Some(labels));
    let first = svg_bytes(custom_chart.clone());
    let second = svg_bytes(custom_chart);
    let custom = String::from_utf8(first.clone()).unwrap();

    assert!(defaults.contains(">base 10.000</text>"));
    assert!(defaults.contains(">total 11.000</text>"));
    assert!(custom.contains(">Expected output 10.000</text>"));
    assert!(custom.contains(">Final prediction 11.000</text>"));
    assert!(!custom.contains(">base 10.000</text>"));
    assert!(!custom.contains(">total 11.000</text>"));
    assert_eq!(first, second);
}

#[test]
fn waterfall_close_total_annotations_are_measured_disjoint_and_in_bounds() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("step".into(), ["first", "second"]).into(),
            Series::new("delta".into(), [0.01_f64, -0.009]).into(),
            Series::new("base".into(), [10.0_f64, 10.0]).into(),
        ],
    )
    .unwrap();
    let labels = WaterfallLabels::new("Expected model output", "Final model prediction").unwrap();
    let chart = waterfall_chart(&data, Some(labels)).size(520, 280);
    let outer_panel = chart.figure_layout().panel_box;
    let mut backend = RecordingBackend::default();

    chart
        .render_kind(PlotKind::Waterfall, &mut backend)
        .unwrap();

    let mark_panel = backend.clips[0];
    let base = text_bounds(&backend, "Expected model output 10.000");
    let terminal = text_bounds(&backend, "Final model prediction 10.001");
    assert!(base.x0 >= mark_panel.x0 && base.x1 <= mark_panel.x1);
    assert!(terminal.x0 >= mark_panel.x0 && terminal.x1 <= mark_panel.x1);
    assert!(base.y0 >= mark_panel.y1 && base.y1 <= outer_panel.y1);
    assert!(terminal.y0 >= mark_panel.y1 && terminal.y1 <= outer_panel.y1);
    assert!(!rectangles_intersect(base, terminal));
    assert!((base.y0 - terminal.y0).abs() > f64::EPSILON);
    let font_sizes = [
        text_style(&backend, "Expected model output 10.000").font_size,
        text_style(&backend, "Final model prediction 10.001").font_size,
    ];
    assert!(font_sizes.into_iter().all(|size| size >= 11));
}

#[test]
fn affected_detail_text_uses_publication_detail_typography() {
    let expected = crate::Theme::Publication.palette().detail_font_size;
    assert!(expected >= 11);

    let beeswarm_data = beeswarm_data(&[0.0, 1.0]);
    let beeswarm = beeswarm_chart(&beeswarm_data, None).color_label("Beeswarm detail");
    let mut beeswarm_backend = RecordingBackend::default();
    beeswarm
        .render_kind(PlotKind::Beeswarm, &mut beeswarm_backend)
        .unwrap();

    let heatmap_data = DataFrame::new(
        2,
        vec![
            Series::new("column".into(), ["left", "right"]).into(),
            Series::new("row".into(), ["top", "bottom"]).into(),
            Series::new("value".into(), [0.0_f64, 1.0]).into(),
        ],
    )
    .unwrap();
    let heatmap = Plot::heatmap(
        crate::test_support::category(&heatmap_data, "column"),
        crate::test_support::category(&heatmap_data, "row"),
        crate::test_support::numeric(&heatmap_data, "value"),
    )
    .color_label("Heatmap detail");
    let mut heatmap_backend = RecordingBackend::default();
    heatmap
        .render_kind(PlotKind::Heatmap, &mut heatmap_backend)
        .unwrap();

    let estimate_data = DataFrame::new(
        1,
        vec![
            Series::new("term".into(), ["intercept"]).into(),
            Series::new("estimate".into(), [1.25_f64]).into(),
        ],
    )
    .unwrap();
    let coefficient = Plot::coefficient(
        crate::test_support::category(&estimate_data, "term"),
        crate::CoefficientInput::estimates(crate::test_support::numeric(
            &estimate_data,
            "estimate",
        )),
    )
    .estimate_labels(NumericLabelFormat::with_suffix(2, " detail").unwrap());
    let mut coefficient_backend = RecordingBackend::default();
    coefficient
        .render_kind(PlotKind::Coefficient, &mut coefficient_backend)
        .unwrap();

    let detail_sizes = [
        text_style(&beeswarm_backend, "Beeswarm detail").font_size,
        text_style(&beeswarm_backend, "0.000").font_size,
        text_style(&heatmap_backend, "Heatmap detail").font_size,
        text_style(&heatmap_backend, "Missing").font_size,
        text_style(&coefficient_backend, "1.25 detail").font_size,
    ];
    assert!(detail_sizes.into_iter().all(|size| size == expected));
}

fn text_style<'a>(backend: &'a RecordingBackend, text: &str) -> &'a TextStyle {
    &backend
        .text_draws
        .iter()
        .find(|(_, _, candidate, _)| candidate == text)
        .unwrap_or_else(|| panic!("text draw {text:?} should exist"))
        .3
}

fn text_bounds(backend: &RecordingBackend, text: &str) -> PanelBox {
    let (x, y, _, style) = backend
        .text_draws
        .iter()
        .find(|(_, _, candidate, _)| candidate == text)
        .unwrap_or_else(|| panic!("text draw {text:?} should exist"));
    assert_eq!(style.anchor, TextAnchor::Middle);
    let half_width =
        crate::layout::deterministic_text_width(text, f64::from(style.font_size)) / 2.0;
    let half_height = f64::from(style.font_size) / 2.0;
    PanelBox::new(
        f64::from(*x) - half_width,
        f64::from(*x) + half_width,
        f64::from(*y) - half_height,
        f64::from(*y) + half_height,
    )
}

fn rectangles_intersect(left: PanelBox, right: PanelBox) -> bool {
    left.x0 < right.x1 && left.x1 > right.x0 && left.y0 < right.y1 && left.y1 > right.y0
}

#[test]
fn line_path_style_requires_path_mode_and_hue_before_backend_begin() {
    let data = path_data();
    let cases = vec![
        path_chart(&data, LinePathStyle::Hue).erase(),
        Plot::line_path(
            crate::test_support::numeric(&data, "x"),
            crate::test_support::numeric(&data, "y"),
            crate::test_support::category(&data, "path"),
        )
        .style(LinePathStyle::Hue)
        .erase(),
    ];
    for (index, chart) in cases.into_iter().enumerate() {
        let mut backend = RecordingBackend::default();
        let kind = if index == 0 {
            PlotKind::Line
        } else {
            PlotKind::LinePath
        };
        let error = chart.render_kind(kind, &mut backend).unwrap_err();

        assert!(error.to_string().contains("LinePathStyle::Hue"));
        assert_eq!(backend.begins, 0);
    }
}
