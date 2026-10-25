//! Supplied effect estimates, complete intervals and an explicit scientific null.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{Bounds, CategoryValues, Labels, NumericValues, Plot, TickGeneration};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    // ANCHOR: effect-plot
    Plot::forest(
        CategoryValues::new(
            "Brain region",
            [
                "Prefrontal",
                "Parietal",
                "Temporal",
                "Occipital",
                "Insula",
                "Cerebellum",
                "Hippocampus",
            ],
        )?,
        NumericValues::new(
            "Standardized effect",
            [0.46, 0.31, 0.62, 0.15, -0.18, 0.08, 0.39],
        ),
        Bounds::new(
            NumericValues::new("Lower", [0.22, 0.05, 0.33, -0.12, -0.42, -0.16, 0.12]),
            NumericValues::new("Upper", [0.70, 0.57, 0.91, 0.42, 0.06, 0.32, 0.66]),
        ),
        0.0,
    )
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(840, 500)
    .x_tick_generation(TickGeneration::target(6)?)
    .title("Regional effects and uncertainty")
    .subtitle("Read the interval, not only the point estimate")
    .x_label("Standardized effect (a.u.)")
    .y_label("")
    .caption("Illustrative estimates and supplied intervals · not results from a clinical study")
    .svg(directory.join("effect_intervals.svg"))?;
    // ANCHOR_END: effect-plot
    Ok(())
}
// ANCHOR_END: example
