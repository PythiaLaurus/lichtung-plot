//! One scientific numeric policy across axes, values, guides and total shelves.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    AxisScale, Canvas, CategoryValues, CoefficientInput, HeatmapAnnotations, NumericFactor,
    NumericFormat, NumericLocale, NumericNotation, NumericPrecision, NumericPrefixes, NumericRate,
    NumericRounding, NumericSymbol, NumericUnit, NumericUnitPlacement, NumericValues, Plot,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let percent = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(1))?
        .rate(NumericRate::Percent)?
        .rounding(NumericRounding::NearestAway)?
        .locale(NumericLocale::invariant())?;
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
    .y_tick_format(percent.clone().value_function(|context| {
        let label = context.format_builtin()?;
        Ok(if context.index() == 0 {
            format!("start: {label}")
        } else {
            label
        })
    }))
    .title("Custom labels on automatic positions");
    let cells = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"])?,
        CategoryValues::new("row", ["one", "one"])?,
        NumericValues::new("fraction", [0.125, 0.875]),
    )
    .color_label_format(percent)
    .annotations(HeatmapAnnotations::Values(adaptive.clone().unit(Some(
        NumericUnit::new("fraction", NumericUnitPlacement::Shared),
    ))?))
    .title("Raw values and percentage guide");
    let estimates = Plot::coefficient(
        CategoryValues::new("term", ["a", "b"])?,
        CoefficientInput::estimates(NumericValues::new("estimate", [1.01, 1.02])),
    )
    .estimate_labels(
        adaptive
            .unit(Some(NumericUnit::new(
                "units",
                NumericUnitPlacement::Shared,
            )))?
            .set_function(|context| Ok(context.format_builtin()?.explanation("Adjusted values"))),
    )
    .title("One complete estimate set");
    let totals = Plot::waterfall(
        CategoryValues::new("step", ["gain", "loss"])?,
        NumericValues::new("delta", [12.5, -2.0]),
        100.0,
    )
    .total_format(
        NumericFormat::default()
            .locale(NumericLocale::named("en-GB")?)?
            .symbol(NumericSymbol::Currency)?
            .unit(Some(NumericUnit::new("GBP", NumericUnitPlacement::Shared)))?,
    )
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
        .panel(2, 1, prefix_plot()?)?
        .svg_preserve_text(directory.join("numeric_formats_text.svg"))?
        .svg(directory.join("numeric_formats.svg"))?;
    println!(
        "numeric_formats.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}

fn prefix_plot() -> lichtung_plot::Result<Plot<lichtung_plot::kind::Scatter>> {
    Ok(Plot::scatter(
        NumericValues::new("distance", [1000.0, 2500.0]),
        NumericValues::new("mass", [1_000_000.0, 2_000_000.0]),
    )
    .x_tick_format(
        NumericFormat::from_specifier(".3s")?
            .unit(Some(NumericUnit::new("m", NumericUnitPlacement::PerLabel)))?,
    )
    .y_tick_format(
        NumericFormat::from_specifier(",.0f")?
            .prefixes(NumericPrefixes::factor(NumericFactor::PowerOfTen(
                "6".into(),
            ))?)?
            .unit(Some(NumericUnit::new("", NumericUnitPlacement::Shared)))?,
    )
    .title("Repeated SI units and one common factor"))
}
// ANCHOR_END: example
