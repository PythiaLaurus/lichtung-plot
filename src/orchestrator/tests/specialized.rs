use super::{
    configured_chart, fixture, test_directory, DataFrame, Plot, PlotKind, RecordingBackend, Series,
};

#[test]
fn specialized_renderers_clip_resolved_marks_without_clipping_decorations() {
    let long_group = "category label long enough to expand the panel";
    let group = Series::new("group".into(), [long_group, long_group]);
    let x = Series::new("x".into(), [1.0_f64, 2.0]);
    let y = Series::new("y".into(), [2.0_f64, 3.0]);
    let color = Series::new("color".into(), [0.1_f64, 1.0]);
    let data = DataFrame::new(
        group.len(),
        vec![group.into(), x.into(), y.into(), color.into()],
    )
    .unwrap();

    let horizontal = Plot::horizontal_bar(
        crate::test_support::category(&data, "group"),
        crate::BarInput::samples(crate::test_support::numeric(&data, "y")),
    );
    let outer_panel = horizontal.canvas_layout().panel_box;
    let mut horizontal_backend = RecordingBackend::default();
    horizontal
        .render_kind(PlotKind::HorizontalBar, &mut horizontal_backend)
        .unwrap();
    assert!(horizontal_backend.clips[0].x0 > outer_panel.x0);

    let beeswarm = Plot::beeswarm(
        crate::test_support::category(&data, "group"),
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "color"),
    )
    .size(800, 600);
    let mut beeswarm_backend = RecordingBackend::default();
    beeswarm
        .render_kind(PlotKind::Beeswarm, &mut beeswarm_backend)
        .unwrap();
    let beeswarm_panel = beeswarm_backend.clips[0];
    assert!(beeswarm_panel.x1 < beeswarm.canvas_layout().panel_box.x1);
    let height = beeswarm_panel.height().min(150.0);
    let bottom = beeswarm_panel.y0 + (beeswarm_panel.height() - height) / 2.0;
    let high_label = beeswarm_backend
        .texts
        .iter()
        .find(|(_, _, text)| text == "1.000")
        .unwrap();
    assert!((f64::from(high_label.0) - (beeswarm_panel.x1 + 34.0).trunc()).abs() < f64::EPSILON);
    assert!((f64::from(high_label.1) - (bottom + height - 4.0).trunc()).abs() < f64::EPSILON);
    let low_label = beeswarm_backend
        .texts
        .iter()
        .find(|(_, _, text)| text == "0.100")
        .unwrap();
    assert!((f64::from(low_label.0) - (beeswarm_panel.x1 + 34.0).trunc()).abs() < f64::EPSILON);
    assert!((f64::from(low_label.1) - (bottom - 13.0).max(0.0).trunc()).abs() < f64::EPSILON);

    let marginal = Plot::marginal(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    );
    let mut marginal_backend = RecordingBackend::default();
    marginal
        .render_kind(PlotKind::Marginal, &mut marginal_backend)
        .unwrap();
    assert_eq!(marginal_backend.clips.len(), 3);

    let matrix = Plot::matrix(vec![
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    ])
    .title("Matrix title");
    let mut matrix_backend = RecordingBackend::default();
    matrix
        .render_kind(PlotKind::Matrix, &mut matrix_backend)
        .unwrap();
    assert_eq!(matrix_backend.clips.len(), 4);
}

#[test]
fn specialized_svg_clips_only_marks_and_keeps_titles_and_colorbars_outside() {
    let data = fixture();
    let cases = [
        (PlotKind::HorizontalBar, "horizontal", 1_usize),
        (PlotKind::Beeswarm, "beeswarm", 1),
        (PlotKind::Matrix, "matrix", 4),
    ];
    for (kind, name, expected_clips) in cases {
        let chart = configured_chart(&data, kind).title(format!("{name} title"));
        let dir = test_directory(name);
        let output = dir.join("canvas.svg");
        chart.svg(&output).unwrap();
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
    let data = fixture();
    let layer = crate::PointLayer::new(
        crate::NumericValues::new("peak x", [2.0]),
        crate::NumericValues::new("peak y", [3.0]),
    )
    .color((180, 45, 40))
    .opacity(0.9)
    .unwrap()
    .labels(crate::CategoryValues::new("peak label", ["2.0"]).unwrap());
    let chart = Plot::line(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
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
    assert!((f64::from(label.0) - (peak.x + 6.0).round()).abs() < f64::EPSILON);
    assert!((f64::from(label.1) - (peak.y + 6.0).round()).abs() < f64::EPSILON);
}

#[test]
fn explicit_limits_omit_out_of_range_point_layer_labels_from_svg() {
    let data = fixture();
    let layer = crate::PointLayer::new(
        crate::NumericValues::new("peak x", [-0.1, 2.0, 3.5]),
        crate::NumericValues::new("peak y", [2.0, 3.0, 4.0]),
    )
    .color((180, 45, 40))
    .opacity(0.9)
    .unwrap()
    .labels(crate::CategoryValues::new("peak label", ["-0.1", "2.0", "3.5"]).unwrap());
    let chart = Plot::line(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .point_layer(layer)
    .x_limits(0.0, 3.0)
    .y_limits(0.0, 6.0);
    let dir = test_directory("mass-label-explicit-limits");
    let output = dir.join("canvas.svg");

    chart.svg(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();

    assert!(!svg.contains(">-0.1</text>"));
    assert!(svg.contains(">2.0</text>"));
    assert!(!svg.contains(">3.5</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn explicit_limits_omit_out_of_range_waterfall_annotations_from_svg() {
    let category = Series::new("category".into(), ["a", "b"]);
    let delta = Series::new("delta".into(), [2.0_f64, -1.0]);
    let base = Series::new("base".into(), [10.0_f64, 10.0]);
    let data = DataFrame::new(
        category.len(),
        vec![category.into(), delta.into(), base.into()],
    )
    .unwrap();
    let chart = Plot::waterfall(
        crate::test_support::category(&data, "category"),
        crate::test_support::numeric(&data, "delta"),
        10.0,
    )
    .x_limits(10.5, 11.5);
    let dir = test_directory("waterfall-label-explicit-limits");
    let output = dir.join("canvas.svg");

    chart.svg(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();

    assert!(!svg.contains(">base 10.000</text>"));
    assert!(svg.contains(">total 11.000</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}
