//! Example 01: a minimal scatter using native renderer values.
//!
//! Run with: cargo run --example 01_quick_start -p lichtung-plot
//!
//! Writes a content-sized `quick_start.svg` to the working directory by default
//! from an extensionless base destination, then reports its frozen dimensions.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{NumericValues, Plot, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ANCHOR: scatter-values
    // 30 points on a noisy sinusoid.
    let xs: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.5).collect();
    let ys: Vec<f64> = (0..30)
        .map(|i| (f64::from(i) * 0.4).sin() * 5.0 + 10.0)
        .collect();
    // ANCHOR_END: scatter-values
    let directory = output::directory()?;
    let base_path = directory.join("quick_start");
    // ANCHOR: scatter-publication
    let output = Plot::scatter(NumericValues::new("x", xs), NumericValues::new("y", ys))
        .theme(Theme::nature())
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
