//! Example 01: a minimal scatter using the `Chart<'a>` Builder.
//!
//! Run with:  cargo run --example 01_quick_start -p lichtung-plot
//!
//! Writes a Nature-themed SVG to the system temporary directory.

use lichtung_plot::theme::Theme;
use lichtung_plot::Chart;
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 30 points on a noisy sinusoid.
    let xs: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.5).collect();
    let ys: Vec<f64> = (0..30)
        .map(|i| (f64::from(i) * 0.4).sin() * 5.0 + 10.0)
        .collect();
    let sx = Series::new("x".into(), xs);
    let sy = Series::new("y".into(), ys);
    let df = DataFrame::new(sx.len(), vec![sx.into(), sy.into()])?;

    let path = std::env::temp_dir().join("quick_start.svg");
    let mut chart = Chart::new(&df);
    chart
        .x("x")
        .y("y")
        .theme(Theme::Nature)
        .size(640, 480)
        .scatter();
    chart.render_to_svg(&path)?;

    println!("wrote {}", path.display());
    Ok(())
}
