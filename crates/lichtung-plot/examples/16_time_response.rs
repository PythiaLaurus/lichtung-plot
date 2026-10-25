//! Synthetic response curves with caller-supplied uncertainty envelopes.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    Bounds, CategoryValues, Labels, LegendPolicy, LegendPosition, NumericValues, Plot,
    TickGeneration,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    // ANCHOR: response-values
    let mut times = Vec::new();
    let mut responses = Vec::new();
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    let mut compounds = Vec::new();
    for (name, amplitude, rate) in [
        ("Control", 0.10, 0.08),
        ("Compound A", 0.68, 0.20),
        ("Compound B", 0.45, 0.11),
    ] {
        for hour in 0..=24 {
            let time = f64::from(hour);
            let response = 1.0 - amplitude * (1.0 - (-rate * time).exp());
            let envelope = 0.025 + 0.025 * (1.0 - (-0.1 * time).exp());
            times.push(time);
            responses.push(response);
            lower.push(response - envelope);
            upper.push(response + envelope);
            compounds.push(name);
        }
    }
    // ANCHOR_END: response-values
    // ANCHOR: response-plot
    Plot::line(
        NumericValues::new("Time after treatment (h)", times),
        NumericValues::new("Relative response", responses),
    )
    .hue(CategoryValues::new("Treatment", compounds)?)
    .band(Bounds::new(
        NumericValues::new("Lower", lower),
        NumericValues::new("Upper", upper),
    ))
    .markers(false)
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(840, 560)
    .x_limits(0.0, 24.0)
    .y_limits(0.2, 1.1)
    .x_tick_generation(TickGeneration::target(6)?)
    .title("The response unfolds over time")
    .subtitle("Distinct dynamics, with the uncertainty kept visible")
    .caption("Synthetic curves · shaded envelopes are supplied, not fitted confidence intervals")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideRightTop))
    .svg(directory.join("time_response.svg"))?;
    // ANCHOR_END: response-plot
    Ok(())
}
// ANCHOR_END: example
