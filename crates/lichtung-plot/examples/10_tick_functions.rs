//! One application rule, independently formatted, consumes the final actual view.

use lichtung_plot::{Canvas, NumericFormat, NumericValues, Plot, TickGeneration};

fn main() -> lichtung_plot::Result<()> {
    let warning = TickGeneration::function(|context| {
        let (lower, upper) = context.domain().data_limits().ok_or_else(|| {
            lichtung_plot::Error::Data("warning locator requires retained raw limits".into())
        })?;
        let mut values = vec![lower];
        if lower <= 50.0 && 50.0 <= upper {
            values.push(50.0);
        }
        values.push(upper);
        Ok(values.into())
    });
    let panel = |(lower, upper)| {
        Plot::scatter(
            NumericValues::new("x", [lower, upper]),
            NumericValues::new("response", [0.0, 1.0]),
        )
        .x_limits(lower, upper)
        .x_tick_generation(warning.clone())
        .x_tick_format(NumericFormat::from_specifier(".1f").expect("literal format"))
    };
    let output = Canvas::grid(3, 1)?
        .panel(0, 0, panel((0.0, 120.0)))?
        .panel(1, 0, panel((20.0, 80.0)))?
        .panel(2, 0, panel((60.0, 120.0)))?
        .size(1200, 1200)
        .svg_preserve_text(std::env::temp_dir().join("tick_functions"))?;
    println!(
        "tick_functions.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
