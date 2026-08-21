//! Example 03: injecting owned pairwise comparisons into a plot.
//!
//! The standalone plot crate performs no hypothesis test. It consumes its own
//! `ComparisonAnnotation` display values supplied by the caller.
//!
//! This example supplies raw and adjusted p-values for multiple named pairs;
//! the adjusted values determine the stars shown on the boxplot.
//!
//! Run with:  cargo run --example 03_inject_stats -p lichtung-plot
//!
//! Writes a boxplot SVG to the system temporary directory.

use lichtung_plot::theme::Theme;
use lichtung_plot::Chart;
use lichtung_plot::ComparisonAnnotation;
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Three groups with clearly different means.
    let group_a: Vec<f64> = (0..30).map(|i| 5.0 + f64::from(i) * 0.1).collect();
    let group_b: Vec<f64> = (0..30).map(|i| 8.0 + f64::from(i) * 0.1).collect();
    let group_c: Vec<f64> = (0..30).map(|i| 12.0 + f64::from(i) * 0.1).collect();

    // Build a polars DataFrame with a categorical `group`
    // column and a continuous `value` column. The boxplot
    // groups by `group` and plots `value`.
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
    let s_group = Series::new("group".into(), group_col);
    let s_value = Series::new("value".into(), value_col);
    let df = DataFrame::new(s_group.len(), vec![s_group.into(), s_value.into()])?;

    let comparisons = vec![
        ComparisonAnnotation::new("A", "B", 0.002).adjusted(0.006, "Holm"),
        ComparisonAnnotation::new("A", "C", 0.000_01).adjusted(0.000_03, "Holm"),
    ];

    let path = std::env::temp_dir().join("inject_stats.svg");
    let mut chart = Chart::new(&df);
    chart
        .x("group")
        .y("value")
        .theme(Theme::Nature)
        .size(640, 480)
        .add_comparisons(comparisons)
        .boxplot();
    chart.render_to_svg(&path)?;

    println!("wrote {}", path.display());
    Ok(())
}
