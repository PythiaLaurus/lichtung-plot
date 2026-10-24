//! One scientific numeric policy across axes, values, guides and total shelves.

use lichtung_plot::{
    AxisScale, Canvas, CategoryValues, CoefficientInput, HeatmapAnnotations, NumericFormat,
    NumericNotation, NumericPrecision, NumericValues, Plot,
};

fn main() -> lichtung_plot::Result<()> {
    let percent = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(1))?
        .decimal_scale(2)?
        .suffix("%")?;
    let adaptive = NumericFormat::new(
        NumericNotation::Decimal,
        NumericPrecision::AdaptiveFractional { min: 0, max: 20 },
    )?;
    let scientific = NumericFormat::new(
        NumericNotation::Scientific,
        NumericPrecision::Significant(3),
    )?;
    let axes = Plot::scatter(
        NumericValues::new("time", [1.0, 2.0]),
        NumericValues::new("fraction", [0.125, 0.875]),
    )
    .y_tick_format(percent.clone())
    .title("Explicit percentage scaling");
    let cells = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"])?,
        CategoryValues::new("row", ["one", "one"])?,
        NumericValues::new("fraction", [0.125, 0.875]),
    )
    .color_label_format(percent)
    .annotations(HeatmapAnnotations::Values(adaptive.clone()))
    .title("Raw values and percentage guide");
    let estimates = Plot::coefficient(
        CategoryValues::new("term", ["a", "b"])?,
        CoefficientInput::estimates(NumericValues::new("estimate", [1.01, 1.02])),
    )
    .estimate_labels(adaptive.suffix(" units")?)
    .title("One complete estimate set");
    let totals = Plot::waterfall(
        CategoryValues::new("step", ["gain", "loss"])?,
        NumericValues::new("delta", [12.5, -2.0]),
        100.0,
    )
    .total_format(NumericFormat::default().prefix("$")?)
    .title("Verbatim names and numeric affixes");
    let forest = Plot::forest(
        CategoryValues::new("term", ["ratio"])?,
        NumericValues::new("estimate", [3.0]),
        lichtung_plot::Bounds::new(
            NumericValues::new("lower", [2.0]),
            NumericValues::new("upper", [5.0]),
        ),
        1.0,
    )
    .scale(AxisScale::Log10)
    .x_tick_format(scientific)
    .title("Certified mathematical powers");
    let output = Canvas::grid(3, 2)?
        .width(1200)
        .panel(0, 0, axes)?
        .panel(0, 1, cells)?
        .panel(1, 0, estimates)?
        .panel(1, 1, totals)?
        .panel(2, 0, forest)?
        .svg_preserve_text(std::env::temp_dir().join("numeric_formats"))?;
    println!(
        "numeric_formats.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
