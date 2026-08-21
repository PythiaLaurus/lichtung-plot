//! Example 02: theme tour.
//!
//! Renders the same data under each of the four themes
//! (Publication, Nature, Science, CHI) to demonstrate the
//! theme system. Run with:
//!   cargo run --example 02_themes -p lichtung-plot

use lichtung_plot::theme::Theme;
use lichtung_plot::Chart;
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xs: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.7).collect();
    let ys: Vec<f64> = (0..20)
        .map(|i| (f64::from(i) * 0.5).cos() * 3.0 + 7.0)
        .collect();
    let sx = Series::new("x".into(), xs);
    let sy = Series::new("y".into(), ys);
    let df = DataFrame::new(sx.len(), vec![sx.into(), sy.into()])?;

    for theme in [
        Theme::Publication,
        Theme::Nature,
        Theme::Science,
        Theme::CHI,
    ] {
        let path = std::env::temp_dir().join(format!("theme_{theme:?}.svg"));
        let mut chart = Chart::new(&df);
        chart.x("x").y("y").theme(theme).size(480, 360).scatter();
        chart.render_to_svg(&path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
