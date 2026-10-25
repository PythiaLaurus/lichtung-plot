//! An ordered synthetic response matrix with an explicit diverging color scale.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    CategoryValues, ColorDomain, ColorStop, ContinuousColorScale, DivergingGradient, Labels,
    NumericValues, OpaqueColor, Plot,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    // ANCHOR: expression-values
    let times = ["0", "1", "2", "3", "4", "6", "8", "12"];
    let programs = [
        ("IFN response", [-0.1, 0.4, 1.3, 1.9, 1.7, 1.0, 0.4, 0.0]),
        ("Inflammation", [0.0, 1.2, 1.8, 1.4, 0.6, 0.0, -0.4, -0.2]),
        (
            "Stress response",
            [0.1, 0.7, 1.1, 0.8, 0.4, -0.1, -0.4, -0.5],
        ),
        ("Cell cycle", [0.2, -0.1, -0.8, -1.6, -1.9, -1.2, -0.6, 0.3]),
        ("DNA repair", [0.0, -0.4, -0.5, 0.0, 0.6, 1.3, 1.7, 1.1]),
        ("Translation", [0.4, 0.0, -0.8, -1.3, -0.8, -0.2, 0.3, 0.6]),
        ("Respiration", [-0.3, -0.5, -0.9, -0.6, 0.2, 0.9, 1.4, 1.7]),
        (
            "Lipid metabolism",
            [-0.2, -0.7, -1.2, -1.0, -0.3, 0.5, 1.0, 1.3],
        ),
    ];
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut values = Vec::new();
    for (program, row) in programs {
        for (time, value) in times.into_iter().zip(row) {
            x.push(time);
            y.push(program);
            values.push(value);
        }
    }
    // ANCHOR_END: expression-values
    // ANCHOR: expression-plot
    let gradient = DivergingGradient::new([
        ColorStop::new(0.0, OpaqueColor::hex("#286A84")?)?,
        ColorStop::new(0.5, OpaqueColor::hex("#F7F5EF")?)?,
        ColorStop::new(1.0, OpaqueColor::hex("#C96451")?)?,
    ])?;
    Plot::heatmap(
        CategoryValues::new("Time after stimulus (h)", x)?,
        CategoryValues::new("Response program", y)?,
        NumericValues::new("log2 fold change", values),
    )
    .continuous_color_scale(ContinuousColorScale::diverging(
        0.0,
        Some(gradient),
        Some(ColorDomain::exact(-2.0, 2.0)?),
    )?)
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(840, 560)
    .title("Coordinated response programs")
    .subtitle("An explicit neutral center separates induction from suppression")
    .caption("Synthetic expression patterns · caller-supplied row order · no inferred clustering")
    .svg(directory.join("expression_heatmap.svg"))?;
    // ANCHOR_END: expression-plot
    Ok(())
}
// ANCHOR_END: example
