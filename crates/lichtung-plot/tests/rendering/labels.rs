use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{BarInput, CategoryValues, Labels, NumericValues, Plot};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn explicit_tilt_keeps_complete_category_labels_and_non_tenth_annotation_rotation() {
    use lichtung_plot::{
        PanelCorner, PanelTextAnnotation, Text, TextAnchor, TextLayout, TextOverflow, TextRotation,
    };
    let plot = Plot::bar(
        CategoryValues::new("model", ["LightGBM", "CatBoost", "RandomForest"]).unwrap(),
        BarInput::estimates(NumericValues::new("score", [1.0, 2.0, 3.0])),
    )
    .size(360, 380)
    .labels(Labels::raw())
    .x_tick_text_layout(TextLayout::default().rotation(TextRotation::degrees(-60.0).unwrap()))
    .panel_text_annotation(
        PanelTextAnnotation::new(
            PanelCorner::TopLeft,
            Text::new("Projected note").layout(
                TextLayout::default()
                    .anchor(TextAnchor::top_left())
                    .rotation(TextRotation::degrees(30.05).unwrap())
                    .overflow(TextOverflow::visible()),
            ),
        )
        .unwrap(),
    );
    let svg = render(plot);
    for label in ["LightGBM", "CatBoost", "RandomForest", "Projected note"] {
        assert!(svg.contains(&format!(">{label}</text>")));
    }
    assert!(svg.contains("rotate(30.05"));
    assert!(svg.contains("rotate(-60"));
}

#[test]
fn scatter_legend_omits_hues_from_discarded_nonfinite_rows() {
    Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, f64::NAN]),
        NumericValues::new("y", [0.0, 1.0, 2.0]),
    )
    .hue(CategoryValues::new("cohort", ["kept", "kept", "\u{10FFFF}"]).unwrap())
    .preflight()
    .unwrap();
}

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-labels-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn horizontal_bar(
    treatment_group: &CategoryValues,
    mean_score: &NumericValues,
) -> Plot<lichtung_plot::kind::HorizontalBar> {
    Plot::horizontal_bar(
        treatment_group.clone(),
        BarInput::estimates(mean_score.clone()),
    )
}

fn text_layer<'a>(svg: &'a str, name: &str) -> &'a str {
    svg.split_once(&format!("<g id=\"{name}\">"))
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn horizontal_category_text(svg: &str, count: u32) -> Vec<String> {
    let panel = super::parse_first_clip_rect(svg);
    // Exclude the rotated title and other layers before joining wrapped text
    // within each bottom-first category slot. Keep XML entities and spaces exact.
    let axes = text_layer(svg, "axes")
        .lines()
        .filter(|line| !line.contains("transform="))
        .collect::<Vec<_>>()
        .join("\n");
    let text = super::parse_text_elements(&axes);
    (0..count)
        .map(|index| {
            let top = panel.y + panel.height * (1.0 - f64::from(index + 1) / f64::from(count));
            let bottom = panel.y + panel.height * (1.0 - f64::from(index) / f64::from(count));
            text.iter()
                .filter(|entry| entry.x < panel.x && entry.y >= top && entry.y < bottom)
                .map(|entry| entry.content.as_str())
                .collect()
        })
        .collect()
}

#[test]
fn direct_plot_humanizes_axes_categories_legend_title_and_entries() {
    let treatment_group =
        CategoryValues::new("treatment_group", ["control_group", "active_group"]).unwrap();
    let mean_score = NumericValues::new("mean_score", [1.0, 2.0]);
    let study_arm = CategoryValues::new("study_arm", ["control_arm", "active_arm"]).unwrap();
    let svg = render(horizontal_bar(&treatment_group, &mean_score).series(study_arm));

    let resolved =
        ["control_group", "active_group"].map(|raw| Labels::default().resolve(raw).unwrap());
    assert_eq!(resolved, ["Control group", "Active group"]);
    assert_eq!(horizontal_category_text(&svg, 2), resolved);
    let axes = super::parse_text_elements(text_layer(&svg, "axes"));
    let words = axes
        .iter()
        .flat_map(|entry| entry.content.split_whitespace())
        .collect::<Vec<_>>();
    for label in ["Mean score", "Treatment group"] {
        let required = label.split_whitespace().collect::<Vec<_>>();
        assert!(
            words.windows(required.len()).any(|words| words == required),
            "missing {label:?}"
        );
    }
    assert_eq!(
        super::parse_text_elements(text_layer(&svg, "legend"))
            .into_iter()
            .flat_map(|entry| entry
                .content
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        ["Study", "arm", "Control", "arm", "Active", "arm"]
    );
}

#[test]
fn svg_preserves_repeated_spaces_in_vertical_axis_labels() {
    let category = CategoryValues::new("category", ["first", "first", "second", "second"]).unwrap();
    let value = NumericValues::new("value", [1.0, 1.5, 2.0, 2.5]);
    let svg = render(
        Plot::bar(category, BarInput::samples(value))
            .y_label("Response  score")
            .size(640, 480),
    );
    let label = svg
        .lines()
        .find(|line| line.contains(">Response  score</text>"))
        .unwrap();

    assert!(label.contains("xml:space=\"preserve\""));
    let angle = label
        .split_once("rotate(")
        .unwrap()
        .1
        .split_once(',')
        .unwrap()
        .0
        .parse::<f64>()
        .unwrap();
    assert_eq!(angle.to_bits(), (-90.0_f64).to_bits());
}

#[test]
fn aliases_override_every_automatic_horizontal_bar_surface_and_escape_svg() {
    let feature_name = CategoryValues::new("feature_name", ["raw_feature"]).unwrap();
    let mean_score = NumericValues::new("mean_score", [2.0]);
    let labels = Labels::humanized()
        .alias("mean_score", "Score\n(points)")
        .alias("feature_name", "Feature")
        .alias("raw_feature", "A & B <feature>");
    let resolved = labels.resolve("mean_score").unwrap();
    assert_eq!(resolved, "Score\n(points)");
    let svg =
        render(Plot::horizontal_bar(feature_name, BarInput::estimates(mean_score)).labels(labels));

    let axes = super::parse_text_elements(text_layer(&svg, "axes"));
    for label in resolved.lines().chain(std::iter::once("Feature")) {
        assert!(
            axes.iter().any(|entry| entry.content == label),
            "missing {label:?}"
        );
    }
    assert_eq!(
        horizontal_category_text(&svg, 1),
        ["A &amp; B &lt;feature&gt;"]
    );
    assert!(!svg.contains("Score\n(points)"));
}

#[test]
fn ambiguous_display_labels_fail_during_preflight() {
    let group = CategoryValues::new("group", ["control_group", "control-group"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let plot = Plot::bar(group, BarInput::estimates(value));

    let error = plot.clone().preflight().unwrap_err().to_string();

    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("Labels::raw()"));
    let error = plot
        .labels(Labels::raw().aliases([("control_group", ""), ("control-group", "")]))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("colliding aliases"));
}
