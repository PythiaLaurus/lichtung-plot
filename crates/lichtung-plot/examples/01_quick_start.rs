//! Two synthetic enzyme-response regimes from owned native values.
//!
//! Run with: cargo run --example 01_quick_start -p lichtung-plot
//!
//! Writes `quick_start.svg` to the working directory by default
//! from an extensionless base destination, then reports its frozen dimensions.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{
    CategoryValues, Labels, LegendPolicy, LegendPosition, NumericValues, Plot, TickGeneration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ANCHOR: scatter-values
    let mut concentrations = Vec::new();
    let mut rates = Vec::new();
    let mut enzymes = Vec::new();
    for (name, maximum, half_saturation, offset) in
        [("Wild type", 82.0, 2.4, 0), ("Variant", 56.0, 3.8, 80)]
    {
        for sample in 0..72 {
            let x = 0.5 + f64::from(sample) / 6.0;
            concentrations.push(x);
            rates.push(
                maximum * x / (half_saturation + x)
                    * (0.04 * synthetic::normal(sample + offset)).exp(),
            );
            enzymes.push(name);
        }
    }
    // ANCHOR_END: scatter-values
    let directory = output::directory()?;
    let base_path = directory.join("quick_start");
    // ANCHOR: scatter-publication
    let output = Plot::scatter(
        NumericValues::new("Substrate concentration (mM)", concentrations),
        NumericValues::new("Reaction rate (a.u.)", rates),
    )
    .hue(CategoryValues::new("Enzyme", enzymes)?)
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(840, 560)
    .x_limits(0.0, 12.5)
    .y_limits(0.0, 80.0)
    .x_tick_generation(TickGeneration::target(6)?)
    .y_tick_generation(TickGeneration::target(5)?)
    .title("Two enzymes, different saturation")
    .subtitle("A shared substrate range reveals two response regimes")
    .caption("Synthetic observations · illustrative kinetics, not experimental measurements")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideRightTop))
    .svg(&base_path)?;
    // ANCHOR_END: scatter-publication

    println!(
        "wrote {} ({}×{} CSS pixels)",
        directory.join("quick_start.svg").display(),
        output.width(),
        output.height()
    );
    Ok(())
}
// ANCHOR_END: example
