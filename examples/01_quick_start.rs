//! Example 01: a minimal scatter using native renderer values.
//!
//! Run with:  cargo run --example 01_quick_start -p lichtung-plot
//!
//! Writes `quick_start.svg` to the system temporary directory from an
//! extensionless base destination.

use lichtung_plot::{NumericValues, Plot, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 30 points on a noisy sinusoid.
    let xs: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.5).collect();
    let ys: Vec<f64> = (0..30)
        .map(|i| (f64::from(i) * 0.4).sin() * 5.0 + 10.0)
        .collect();
    let directory = std::env::temp_dir();
    let base_path = directory.join("quick_start");
    Plot::scatter(NumericValues::new("x", xs), NumericValues::new("y", ys))
        .theme(Theme::Nature)
        .size(640, 480)
        .svg(&base_path)?;

    println!("wrote {}", directory.join("quick_start.svg").display());
    Ok(())
}
