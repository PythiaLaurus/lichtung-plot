use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{CategoryValues, ColorDomain, ContinuousColorScale, NumericValues, Plot};

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct HeatmapData {
    column: CategoryValues,
    row: CategoryValues,
    value: NumericValues,
}

impl HeatmapData {
    fn new(column: CategoryValues, row: CategoryValues, value: NumericValues) -> Self {
        Self { column, row, value }
    }
}

fn chart(data: &HeatmapData) -> Plot<lichtung_plot::kind::Heatmap> {
    Plot::heatmap(data.column.clone(), data.row.clone(), data.value.clone()).size(560, 360)
}

fn render(
    data: &HeatmapData,
    configure: impl FnOnce(Plot<lichtung_plot::kind::Heatmap>) -> Plot<lichtung_plot::kind::Heatmap>,
) -> Result<String, String> {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    let plot = configure(chart(data));
    let result = plot
        .svg_preserve_text(&output)
        .map(|_| std::fs::read_to_string(&output).unwrap())
        .map_err(|error| error.to_string());
    std::fs::remove_dir_all(directory).unwrap();
    result
}

#[test]
fn asymmetric_diverging_guide_preserves_numeric_center_clearance_and_raster_colors() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["low", "center", "half", "high"]).unwrap(),
        CategoryValues::new("row", ["row"; 4]).unwrap(),
        NumericValues::new("value", [-1.0, 0.0, 9.0, 19.0]),
    );
    let directory = std::env::temp_dir().join(format!(
        "lichtung-asymmetric-guide-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let svg_path = directory.join("plot.svg");
    let png_path = directory.join("plot.png");
    chart(&data)
        .size(560, 600)
        .continuous_color_scale(
            ContinuousColorScale::diverging(
                0.0,
                None,
                Some(ColorDomain::exact(-1.0, 19.0).unwrap()),
            )
            .unwrap(),
        )
        .svg_preserve_text(&svg_path)
        .unwrap()
        .png_preview(&png_path)
        .unwrap();
    let svg = std::fs::read_to_string(svg_path).unwrap();
    let png = std::fs::read(png_path).unwrap();
    let gradient = svg
        .lines()
        .find(|line| line.starts_with("<linearGradient id=\"linear-gradient-0\""))
        .unwrap();
    let (x, low, high) = (
        attr(gradient, "x1"),
        attr(gradient, "y1"),
        attr(gradient, "y2"),
    );
    assert!((high - low).abs() >= 250.0);
    let fraction = (text_attr(&svg, "-1", "y") - text_attr(&svg, "0", "y"))
        / (text_attr(&svg, "-1", "y") - text_attr(&svg, "19", "y"));
    assert!((fraction - 0.05).abs() < 1.0 / 1024.0);
    assert!(svg.contains("<stop offset=\"0.050000\" stop-color=\"#F7F7F7\"/>"));
    assert_eq!(decoded_cell_pixel(&svg, &png, "#F7F7F7"), [247, 247, 247]);
    let center = decoded_png_pixel(&png, x, low + (high - low) * 0.05);
    assert!(center
        .into_iter()
        .all(|channel| channel.abs_diff(247) <= 10));
    let half = decoded_png_pixel(&png, x, f64::midpoint(low, high));
    assert!(half.into_iter().any(|channel| channel.abs_diff(247) > 20));
    std::fs::remove_dir_all(directory).unwrap();

    let close = HeatmapData::new(
        CategoryValues::new("column", ["low", "center", "high"]).unwrap(),
        CategoryValues::new("row", ["row"; 3]).unwrap(),
        NumericValues::new("value", [-0.01, 0.0, 100.0]),
    );
    let failure = render(&close, |chart| {
        chart.continuous_color_scale(
            ContinuousColorScale::diverging(
                0.0,
                None,
                Some(ColorDomain::exact(-0.01, 100.0).unwrap()),
            )
            .unwrap(),
        )
    })
    .unwrap_err();
    assert!(failure.contains("LegendOrGuide"), "{failure}");
}

fn decoded_cell_pixel(svg: &str, png: &[u8], fill: &str) -> [u8; 3] {
    let polygon = clipped_marks(svg)
        .lines()
        .find(|line| line.starts_with("<polygon") && line.contains(&format!("fill=\"{fill}\"")))
        .unwrap();
    let points = polygon
        .split_once("points=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0;
    let corners = points
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
        })
        .collect::<Vec<_>>();
    let x = f64::midpoint(corners[0].0, corners[1].0);
    let y = f64::midpoint(corners[0].1, corners[2].1);
    decoded_png_pixel(png, x, y)
}

fn decoded_png_pixel(png: &[u8], x: f64, y: f64) -> [u8; 3] {
    let mut reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert!(x >= 0.0 && x < f64::from(info.width));
    assert!(y >= 0.0 && y < f64::from(info.height));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "sample the interior of a checked finite positive pixel rectangle"
    )]
    let (x, y) = (x as usize, y as usize);
    let stride = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => panic!("unexpected Heatmap PNG color type: {other:?}"),
    };
    let index = (y * info.width as usize + x) * stride;
    pixels[index..index + 3].try_into().unwrap()
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn text_tag<'a>(svg: &'a str, text: &str) -> &'a str {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    &svg[start..end]
}

fn text_attr(svg: &str, text: &str, name: &str) -> f64 {
    attr(text_tag(svg, text), name)
}

fn attr(tag: &str, name: &str) -> f64 {
    tag.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}
