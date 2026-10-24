//! Independent approximate targets and complete-source maxima, with frozen output.

use lichtung_plot::{
    Canvas, NumericFormat, NumericValues, Plot, TickGeneration, TickSingleton, TickSpace,
};

fn main() -> lichtung_plot::Result<()> {
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
    let output = Canvas::grid(1, 3)?
        .panel(0, 0, target)?
        .panel(0, 1, maximum)?
        .panel(0, 2, exact)?
        .size(1800, 600)
        .svg_preserve_text(std::env::temp_dir().join("tick_generation"))?;
    println!(
        "tick_generation.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
