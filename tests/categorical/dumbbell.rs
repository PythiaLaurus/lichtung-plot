use lichtung_plot::{DumbbellEndpoints, Plot};

use super::{data, df, unique_dir, DataFrame};

fn endpoints() -> DumbbellEndpoints {
    DumbbellEndpoints::new(
        "Lower is preferred",
        (20, 90, 150),
        "Higher is preferred",
        (190, 65, 45),
    )
    .unwrap()
}

fn replicated_data() -> DataFrame {
    df!(
        "category" => ["second", "first", "second", "first"],
        "fold" => ["fold-2", "fold-1", "fold-1", "fold-2"],
        "left" => [1.0_f64, 2.0, 1.5, 2.5],
        "right" => [2.0_f64, 1.0, 2.5, 1.5]
    )
    .unwrap()
}

fn chart(data: &DataFrame) -> Plot<lichtung_plot::kind::Dumbbell> {
    Plot::dumbbell(
        data::category(data, "category"),
        data::numeric(data, "left"),
        data::numeric(data, "right"),
        endpoints(),
    )
    .group(data::category(data, "fold"))
}

fn svg(chart: &Plot<lichtung_plot::kind::Dumbbell>) -> String {
    let directory = unique_dir("dumbbell-svg");
    let output = directory.join("plot.svg");
    chart.clone().svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

#[test]
fn dumbbell_renders_connectors_endpoint_legend_and_all_replicates() {
    let data = replicated_data();
    let rendered = svg(&chart(&data));
    let marks = clipped_marks(&rendered);

    assert_eq!(marks.matches("<polyline ").count(), 4);
    assert_eq!(marks.matches("<circle ").count(), 8);
    assert!(rendered.contains(">Lower is preferred</text>"));
    assert!(rendered.contains(">Higher is preferred</text>"));
    assert!(rendered.contains("fill=\"#145A96\"") && rendered.contains("fill=\"#BE412D\""));
}

#[test]
fn dumbbell_preserves_first_observed_category_order_top_first() {
    let data = replicated_data();
    let rendered = svg(&chart(&data));

    assert!(text_y(&rendered, "Second") < text_y(&rendered, "First"));
}

#[test]
fn dumbbell_replicates_have_distinct_offsets_without_aggregation() {
    let data = replicated_data();
    let rendered = svg(&chart(&data));
    let marks = clipped_marks(&rendered);
    let mut y_values = marks
        .match_indices("<circle ")
        .map(|(start, _)| attribute(&marks[start..], "cy"))
        .collect::<Vec<_>>();
    y_values.sort_by(f64::total_cmp);
    y_values.dedup_by(|left, right| left.to_bits() == right.to_bits());

    assert_eq!(y_values.len(), 4);
}

#[test]
fn dumbbell_rejects_duplicate_category_group_keys() {
    let data = df!(
        "category" => ["same", " same "],
        "fold" => ["fold-1", "fold-1"],
        "left" => [1.0_f64, 2.0],
        "right" => [2.0_f64, 3.0]
    )
    .unwrap();

    let error = chart(&data).preflight().unwrap_err().to_string();

    assert!(error.contains("duplicate normalized (category, group) key"));
}

#[test]
fn dumbbell_without_group_rejects_duplicate_categories() {
    let data = df!(
        "category" => ["same", "same"],
        "left" => [1.0_f64, 2.0],
        "right" => [2.0_f64, 3.0]
    )
    .unwrap();
    let chart = point_only_chart(&data);

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate normalized (category, group) key"));
}

#[test]
fn dumbbell_rejects_null_and_nonfinite_inputs() {
    let null_data = df!(
        "category" => ["a", "b"],
        "left" => [Some(1.0_f64), None],
        "right" => [2.0_f64, 3.0]
    )
    .unwrap();
    let infinite_data = df!(
        "category" => ["a", "b"],
        "left" => [1.0_f64, 2.0],
        "right" => [2.0_f64, f64::INFINITY]
    )
    .unwrap();
    let errors = [
        point_only_chart(&null_data)
            .preflight()
            .unwrap_err()
            .to_string(),
        point_only_chart(&infinite_data)
            .preflight()
            .unwrap_err()
            .to_string(),
    ];

    assert!(errors.iter().all(|error| error.contains("dumbbell")));
    assert!(errors.iter().any(|error| error.contains("contains null")));
    assert!(errors.iter().any(|error| error.contains("non-finite")));
}

#[test]
fn dumbbell_output_is_byte_deterministic_and_registered() {
    let data = replicated_data();

    assert_eq!(svg(&chart(&data)), svg(&chart(&data)));
}

#[test]
fn dumbbell_endpoint_labels_are_caller_owned_not_inferred_direction() {
    let data = replicated_data();
    let rendered = svg(&chart(&data));

    assert!(rendered.contains("Lower is preferred") && rendered.contains("Higher is preferred"));
    assert!(!rendered.contains(">lower</text>") && !rendered.contains(">higher</text>"));
}

#[test]
fn dumbbell_endpoint_type_rejects_empty_or_duplicate_labels() {
    let errors = [
        DumbbellEndpoints::new(" ", (0, 0, 0), "After", (1, 1, 1))
            .unwrap_err()
            .to_string(),
        DumbbellEndpoints::new("Same", (0, 0, 0), "Same", (1, 1, 1))
            .unwrap_err()
            .to_string(),
        DumbbellEndpoints::new("Same", (0, 0, 0), " Same ", (1, 1, 1))
            .unwrap_err()
            .to_string(),
        DumbbellEndpoints::new("Before\n", (0, 0, 0), "After", (1, 1, 1))
            .unwrap_err()
            .to_string(),
    ];

    assert!(errors
        .iter()
        .all(|error| error.contains("dumbbell endpoint labels")));
}

fn point_only_chart(data: &DataFrame) -> Plot<lichtung_plot::kind::Dumbbell> {
    Plot::dumbbell(
        data::category(data, "category"),
        data::numeric(data, "left"),
        data::numeric(data, "right"),
        endpoints(),
    )
}

fn text_y(svg: &str, text: &str) -> f64 {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    attribute(&svg[start..end], "y")
}

fn attribute(tag: &str, name: &str) -> f64 {
    tag.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}
