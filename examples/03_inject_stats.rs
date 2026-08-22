//! Example 03: injecting owned pairwise comparisons into a plot.
//!
//! The standalone plot crate performs no hypothesis test. It consumes its own
//! [`Significance`] display values supplied by the caller.
//!
//! This example supplies the adjusted display p-value for each named pair.
//!
//! Run with:  cargo run --example 03_inject_stats -p lichtung-plot
//!
//! Writes a boxplot SVG to the system temporary directory.

use lichtung_plot::{CategoryValues, NumericValues, Plot, Significance, SignificanceMark, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Three groups with clearly different means.
    let group_a: Vec<f64> = (0..30).map(|i| 5.0 + f64::from(i) * 0.1).collect();
    let group_b: Vec<f64> = (0..30).map(|i| 8.0 + f64::from(i) * 0.1).collect();
    let group_c: Vec<f64> = (0..30).map(|i| 12.0 + f64::from(i) * 0.1).collect();

    // Build owned categorical and continuous display values.
    let mut group_col: Vec<&str> = Vec::new();
    let mut value_col: Vec<f64> = Vec::new();
    for v in &group_a {
        group_col.push("A");
        value_col.push(*v);
    }
    for v in &group_b {
        group_col.push("B");
        value_col.push(*v);
    }
    for v in &group_c {
        group_col.push("C");
        value_col.push(*v);
    }
    let comparisons = vec![
        Significance::between("A", "B", SignificanceMark::PValue(0.006)),
        Significance::between("A", "C", SignificanceMark::PValue(0.000_03)),
    ];

    let path = std::env::temp_dir().join("inject_stats.svg");
    Plot::boxplot(
        CategoryValues::new("group", group_col)?,
        NumericValues::new("value", value_col),
    )
    .theme(Theme::Nature)
    .size(640, 480)
    .significance(comparisons)
    .svg(&path)?;

    println!("wrote {}", path.display());
    Ok(())
}
