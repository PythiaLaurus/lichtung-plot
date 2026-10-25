//! Independent approximate targets and complete-source maxima, with frozen output.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    Canvas, NumericFormat, NumericValues, Plot, TickGeneration, TickRequirement, TickSingleton,
    TickSpace,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let observations = Plot::scatter(
        NumericValues::new("fraction", [0.0, 0.25, 0.5, 0.75, 1.0]),
        NumericValues::new("response", [0.0, 0.2, 0.1, 0.8, 0.5]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0)
    .x_tick_format(NumericFormat::default())
    .y_tick_format(NumericFormat::default());
    let target = observations
        .clone()
        .x_tick_generation(TickGeneration::target(5)?)
        .y_tick_generation(TickGeneration::maximum(3)?)
        .title("Target five x positions; maximum three y positions");
    let maximum = observations
        .x_tick_generation(TickGeneration::maximum(3)?)
        .y_tick_generation(TickGeneration::target(5)?)
        .title("Maximum three x positions; target five y positions");
    let exact = Plot::scatter(
        NumericValues::new("x", [0.0, 10.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(0.0, 10.0)
    .y_limits(0.0, 1.0)
    .x_tick_generation(TickGeneration::exact_count(4, TickSpace::Data).maximum_count(Some(4))?)
    .y_tick_generation(
        TickGeneration::exact_count(1, TickSpace::Data).singleton(TickSingleton::Upper)?,
    )
    .x_tick_format(NumericFormat::from_specifier(".2~f")?)
    .title("Exactly four positions; one upper-endpoint y tick");
    let interval = Plot::scatter(
        NumericValues::new("x", [5.0, 95.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(5.0, 95.0)
    .y_limits(0.0, 1.0)
    .x_tick_generation(TickGeneration::interval(
        20.0,
        5.0,
        TickSpace::Data,
        TickRequirement::Fixed,
    )?)
    .x_tick_format(NumericFormat::from_specifier(".0f")?)
    .title("Fixed data interval: 5, 25, 45, 65, 85");
    let output = Canvas::grid(3, 2)?
        .panel(0, 0, target)?
        .panel(0, 1, maximum)?
        .panel(1, 0, exact)?
        .panel(1, 1, interval)?
        .panel(2, 0, geometric(2.0)?)?
        .panel(2, 1, geometric(1.5)?)?
        .size(1400, 1500)
        .svg_preserve_text(directory.join("tick_generation.svg"))?;
    println!(
        "tick_generation.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}

fn geometric(ratio: f64) -> lichtung_plot::Result<Plot<lichtung_plot::kind::Forest>> {
    Ok(Plot::forest(
        lichtung_plot::CategoryValues::new("term", ["ratio"])?,
        NumericValues::new("estimate", [8.0]),
        lichtung_plot::Bounds::new(
            NumericValues::new("lower", [4.0]),
            NumericValues::new("upper", [16.0]),
        ),
        1.0,
    )
    .scale(lichtung_plot::AxisScale::Log10)
    .x_limits(1.0, 64.0)
    .x_tick_generation(TickGeneration::multiplicative(
        1.0,
        ratio,
        TickRequirement::Fixed,
    )?)
    .x_tick_format(NumericFormat::from_specifier(".3~g")?)
    .title(format!("Anchored geometric lattice; ratio {ratio}")))
}
// ANCHOR_END: example
