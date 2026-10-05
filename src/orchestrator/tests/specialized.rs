use super::{test_directory, Plot, PlotKind, RecordingBackend};
use crate::{CategoryValues, NumericValues};

#[test]
fn specialized_renderers_clip_resolved_marks_without_clipping_decorations() {
    let long_group = "category label long enough to expand the panel";
    let group = CategoryValues::new("group", [long_group, long_group]).unwrap();
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 3.0]);
    let color = NumericValues::new("color", [0.1, 1.0]);

    let horizontal = Plot::horizontal_bar(group.clone(), crate::BarInput::samples(y.clone()));
    let short_label_panel = Plot::horizontal_bar(
        CategoryValues::new("group", ["a", "a"]).unwrap(),
        crate::BarInput::samples(y.clone()),
    )
    .canvas_layout()
    .panel_box;
    let mut horizontal_backend = RecordingBackend::default();
    horizontal
        .render_kind(PlotKind::HorizontalBar, &mut horizontal_backend)
        .unwrap();
    assert!(horizontal_backend.clips[0].x0 > short_label_panel.x0);

    let beeswarm = Plot::beeswarm(group, x.clone(), color).size(800, 600);
    let mut beeswarm_backend = RecordingBackend::default();
    beeswarm
        .render_kind(PlotKind::Beeswarm, &mut beeswarm_backend)
        .unwrap();
    let beeswarm_panel = beeswarm_backend.clips[0];
    assert!(beeswarm_panel.x1 < beeswarm.canvas_layout().panel_box.x1);
    assert_eq!(beeswarm_backend.rounded_rects.len(), 1);
    let guide = beeswarm_backend.rounded_rects[0];
    assert!(guide.x0 > beeswarm_panel.x1);
    assert!(guide.y0 >= beeswarm_panel.y0 && guide.y1 <= beeswarm_panel.y1);
    let high_label = beeswarm_backend
        .texts
        .iter()
        .find(|(x, y, text)| {
            text == "1" && *x >= guide.x0 && *x <= guide.x1 && *y >= guide.y0 && *y <= guide.y1
        })
        .unwrap();
    let low_label = beeswarm_backend
        .texts
        .iter()
        .find(|(x, y, text)| {
            text == "0.1" && *x >= guide.x0 && *x <= guide.x1 && *y >= guide.y0 && *y <= guide.y1
        })
        .unwrap();
    let title = beeswarm_backend
        .texts
        .iter()
        .find(|(_, _, text)| text == "Color")
        .unwrap();
    assert!(title.1 > high_label.1 && high_label.1 > low_label.1);
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    for (label, appearance) in [
        ("Color", &theme.typography.guides.legend_title),
        ("0.1", &theme.typography.guides.legend_label),
        ("1", &theme.typography.guides.legend_label),
    ] {
        let draws = beeswarm_backend
            .text_draws
            .iter()
            .filter(|(x, y, text, _)| {
                text == label
                    && *x >= guide.x0
                    && *x <= guide.x1
                    && *y >= guide.y0
                    && *y <= guide.y1
            })
            .collect::<Vec<_>>();
        assert_eq!(draws.len(), 1, "complete guide text {label}");
        let (x, y, _, style) = draws[0];
        assert!(*x > beeswarm_panel.x1 && *x >= guide.x0 && *x <= guide.x1);
        assert!(*y >= guide.y0 && *y <= guide.y1);
        assert_eq!(style.rotation.to_bits(), 0.0_f64.to_bits());
        assert_eq!(style.font_size, appearance.font_size);
        assert_eq!(style.weight, appearance.weight);
        assert_eq!(style.font_family, appearance.stack);
        assert_eq!(style.color, appearance.paint);
    }
    assert!(beeswarm_backend
        .scatter_draws
        .iter()
        .all(|(_, _, depth)| *depth == 1));
    assert_eq!(beeswarm_backend.clip_ends, 1);

    let marginal = Plot::marginal(x.clone(), y.clone());
    let mut marginal_backend = RecordingBackend::default();
    marginal
        .render_kind(PlotKind::Marginal, &mut marginal_backend)
        .unwrap();
    assert_eq!(marginal_backend.clips.len(), 3);

    let matrix = Plot::matrix(vec![x, y]).title("Matrix title");
    let mut matrix_backend = RecordingBackend::default();
    matrix
        .render_kind(PlotKind::Matrix, &mut matrix_backend)
        .unwrap();
    assert_eq!(matrix_backend.clips.len(), 4);
}

#[test]
fn specialized_svg_clips_only_marks_and_keeps_titles_and_colorbars_outside() {
    let cases = [
        (
            PlotKind::HorizontalBar,
            "horizontal",
            1_usize,
            Plot::horizontal_bar(
                CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
                crate::BarInput::samples(NumericValues::new("y", [1.0, 3.0, 2.0, 5.0])),
            )
            .erase(),
        ),
        (
            PlotKind::Beeswarm,
            "beeswarm",
            1,
            Plot::beeswarm(
                CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
                NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
                NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
            )
            .erase(),
        ),
        (
            PlotKind::Matrix,
            "matrix",
            4,
            Plot::matrix(vec![
                NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
                NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
            ])
            .erase(),
        ),
    ];
    for (kind, name, expected_clips, chart) in cases {
        let chart = chart.title(format!("{name} title"));
        let dir = test_directory(name);
        let output = dir.join("canvas.svg");
        chart.svg_preserve_text(&output).unwrap();
        let svg = std::fs::read_to_string(&output).unwrap();
        assert_eq!(
            svg.matches("<clipPath id=").count(),
            expected_clips,
            "{kind:?}"
        );
        let first_marks = svg.find("<g clip-path=").unwrap();
        let first_close = first_marks + svg[first_marks..].find("</g>").unwrap();
        let title = svg.find(&format!(">{name} title</text>")).unwrap();
        assert!(title < first_marks || title > first_close, "{kind:?}");
        if kind == PlotKind::Beeswarm {
            let colorbar_label = svg.find(">Color</text>").unwrap();
            assert!(colorbar_label > first_close);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn point_layer_label_uses_math_space_y() {
    let layer = crate::PointLayer::new(
        crate::NumericValues::new("peak x", [2.0]),
        crate::NumericValues::new("peak y", [3.0]),
    )
    .color((180, 45, 40))
    .opacity(0.9)
    .unwrap()
    .labels(crate::CategoryValues::new("peak label", ["2.0"]).unwrap());
    let chart = Plot::line(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .point_layer(layer);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Line, &mut backend).unwrap();

    let peak = *backend.scatter_points.last().unwrap();
    let label = backend
        .texts
        .iter()
        .find(|(_, _, text)| text == "2.0")
        .unwrap();
    assert!((label.0 - (peak.x + 6.0).round()).abs() < f64::EPSILON);
    assert!((label.1 - (peak.y + 6.0).round()).abs() < f64::EPSILON);
}

#[test]
fn explicit_limits_omit_out_of_range_point_layer_labels_from_svg() {
    let layer = crate::PointLayer::new(
        crate::NumericValues::new("peak x", [-0.1, 2.0, 3.5]),
        crate::NumericValues::new("peak y", [2.0, 3.0, 4.0]),
    )
    .color((180, 45, 40))
    .opacity(0.9)
    .unwrap()
    .labels(crate::CategoryValues::new("peak label", ["-0.1", "2.0", "3.5"]).unwrap());
    let chart = Plot::line(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .point_layer(layer)
    .x_limits(0.0, 3.0)
    .y_limits(0.0, 6.0);
    let dir = test_directory("mass-label-explicit-limits");
    let output = dir.join("canvas.svg");

    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();

    assert!(!svg.contains(">-0.1</text>"));
    assert!(svg.contains(">2.0</text>"));
    assert!(!svg.contains(">3.5</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn explicit_limits_omit_out_of_range_waterfall_annotations_from_svg() {
    let chart = Plot::waterfall(
        CategoryValues::new("category", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [2.0, -1.0]),
        10.0,
    )
    .x_limits(10.5, 11.5);
    let dir = test_directory("waterfall-label-explicit-limits");
    let output = dir.join("canvas.svg");

    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();

    assert!(!svg.contains(">base 10.000</text>"));
    assert!(svg.contains(">total 11.000</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}
