//! Observation and explicit-series alpha share one immutable guide lifecycle.

use lichtung_plot::{
    AlphaEncoding, AlphaValues, Canvas, CategoryValues, LegendPolicy, LegendPosition,
    NumericValues, Plot,
};

fn main() -> lichtung_plot::Result<()> {
    let points = Plot::scatter(
        NumericValues::new("time", [0.0, 1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("response", [0.0, 0.8, 0.4, 1.3, 1.0]),
    )
    .point_alpha(AlphaEncoding::observations(AlphaValues::linear(
        NumericValues::new("confidence", [0.0, 0.25, 0.5, 0.75, 1.0]),
        (0.0, 1.0),
        (0, 255),
    )?))
    .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
    .title("Original observation alpha");
    let paths = Plot::line(
        NumericValues::new("time", [0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        NumericValues::new("response", [0.0, 0.8, 0.4, 0.5, 0.2, 1.0]),
    )
    .hue(CategoryValues::new(
        "series",
        [
            "control", "control", "control", "treated", "treated", "treated",
        ],
    )?)
    .line_alpha(AlphaEncoding::keyed(
        [CategoryValues::new("series", ["control", "treated"])?],
        AlphaValues::categorical(
            CategoryValues::new("reliability", ["limited", "strong"])?,
            |value| if value == "limited" { 96 } else { 230 },
        )?,
    )?)
    .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
    .title("Explicit series alpha");
    let destination = std::env::temp_dir().join("data_alpha");
    let output = Canvas::grid(1, 2)?
        .panel(0, 0, points)?
        .panel(0, 1, paths)?
        .size(1600, 800)
        .svg_preserve_text(&destination)?
        .svg(destination.with_file_name("data_alpha_outlined"))?
        .png(destination.with_file_name("data_alpha_raster"))?;
    println!(
        "data_alpha: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
