//! Example 02: theme tour.
//!
//! Renders the same data under each of the four themes
//! (Publication, Nature, Science, CHI) to demonstrate the
//! theme system. Run with:
//!   cargo run --example 02_themes -p lichtung-plot

use lichtung_plot::{NumericValues, Plot, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xs: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.7).collect();
    let ys: Vec<f64> = (0..20)
        .map(|i| (f64::from(i) * 0.5).cos() * 3.0 + 7.0)
        .collect();
    for (name, theme) in [
        ("publication", Theme::publication()),
        ("nature", Theme::nature()),
        ("science", Theme::science()),
        ("chi", Theme::chi()),
    ] {
        let path = std::env::temp_dir().join(format!("theme_{name}.svg"));
        Plot::scatter(
            NumericValues::new("x", xs.clone()),
            NumericValues::new("y", ys.clone()),
        )
        .theme(theme)
        .size(480, 360)
        .svg(&path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
