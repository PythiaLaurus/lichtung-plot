use lichtung_plot::{
    BarInput, CategoryValues, NumericValues, Pairwise, Plot, Significance, SignificanceMark,
};

#[derive(Clone)]
struct SignificanceBarData {
    category: CategoryValues,
    value: NumericValues,
}

impl SignificanceBarData {
    fn new(category: CategoryValues, value: NumericValues) -> Self {
        Self { category, value }
    }
}

#[test]
fn rendered_brackets_stay_inside_axes_clear_data_and_share_only_disjoint_levels() {
    let data = SignificanceBarData::new(
        CategoryValues::new("group", ["a", "b", "c", "d"]).unwrap(),
        NumericValues::new("value", [2.0, 4.0, 3.0, 5.0]),
    );
    let comparisons = vec![
        comparison("c", "d", 0.01),
        comparison("a", "c", 0.01),
        comparison("a", "b", 0.01),
    ];
    let dir = tempdir("inside-and-levels");
    let annotated_path = dir.join("annotated.svg");
    let plain_path = dir.join("plain.svg");

    let annotated = identity_bar_chart(&data)
        .size(500, 360)
        .significance(comparisons);
    annotated.svg_preserve_text(&annotated_path).unwrap();

    let plain = identity_bar_chart(&data).size(500, 360);
    plain.svg_preserve_text(&plain_path).unwrap();

    let annotated_svg = std::fs::read_to_string(annotated_path).unwrap();
    let plain_svg = std::fs::read_to_string(plain_path).unwrap();
    let annotated_panel = clip_rect(&annotated_svg);
    let plain_panel = clip_rect(&plain_svg);
    let bracket_y = bracket_line_y_values(&annotated_svg);
    let data_top = polygon_top(&annotated_svg);
    let panel_top = annotated_panel.1;

    assert_eq!(
        annotated_panel, plain_panel,
        "brackets must not shrink the panel"
    );
    assert_eq!(bracket_y.len(), 3);
    assert!((bracket_y[0] - bracket_y[1]).abs() < 0.001);
    assert!(bracket_y[2] < bracket_y[0] - 1.0);
    assert!(
        bracket_y.iter().all(|y| *y >= panel_top + 19.0),
        "bracket labels must remain inside the panel: panel_top={panel_top}, y={bracket_y:?}"
    );
    assert!(
        bracket_y[0] + 7.0 <= data_top - 5.0,
        "lowest bracket ticks must clear rendered bars: bracket_y={bracket_y:?}, data_top={data_top}"
    );
    assert!(
        data_top > polygon_top(&plain_svg) + 5.0,
        "automatic y range must add headroom rather than shrink the panel"
    );
}

fn bracket_line_y_values(svg: &str) -> Vec<f64> {
    svg.lines()
        .filter(|line| line.contains("class=\"significance-bracket\""))
        .map(|line| {
            let path = attribute(line, "d");
            path.split_whitespace()
                .nth(2)
                .unwrap()
                .parse::<f64>()
                .unwrap()
        })
        .collect()
}

fn polygon_top(svg: &str) -> f64 {
    svg.lines()
        .filter(|line| line.starts_with("<polygon points="))
        .flat_map(|line| {
            attribute(line, "points")
                .split_whitespace()
                .map(|point| point.split_once(',').unwrap().1.parse::<f64>().unwrap())
                .collect::<Vec<_>>()
        })
        .fold(f64::INFINITY, f64::min)
}

fn attribute<'a>(line: &'a str, name: &str) -> &'a str {
    let start_marker = format!("{name}=\"");
    let start = line.find(&start_marker).unwrap() + start_marker.len();
    let end = line[start..].find('"').unwrap() + start;
    &line[start..end]
}

fn clip_rect(svg: &str) -> (f64, f64, f64, f64) {
    let clip = &svg[svg.find("<clipPath id=").unwrap()..];
    let rect = clip.lines().find(|line| line.contains("<rect ")).unwrap();
    (
        attribute(rect, "x").parse().unwrap(),
        attribute(rect, "y").parse().unwrap(),
        attribute(rect, "width").parse().unwrap(),
        attribute(rect, "height").parse().unwrap(),
    )
}

fn comparison(endpoint_a: &str, endpoint_b: &str, p_value: f64) -> Significance<Pairwise> {
    Significance::between(endpoint_a, endpoint_b, SignificanceMark::PValue(p_value))
}

fn identity_bar_chart(data: &SignificanceBarData) -> Plot<lichtung_plot::kind::Bar> {
    Plot::bar(
        data.category.clone(),
        BarInput::estimates(data.value.clone()),
    )
}

static TEMP_DIRECTORY_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn tempdir(name: &str) -> std::path::PathBuf {
    loop {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lichtung-significance-{name}-{}-{}",
            std::process::id(),
            sequence
        ));
        match std::fs::create_dir(&directory) {
            Ok(()) => return directory,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("tempdir: {error}"),
        }
    }
}
