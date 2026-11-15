//! Paired synthetic assay observations and their univariate marginal densities.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{Labels, NumericValues, Plot, TickGeneration};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;

    // ANCHOR: joint-values
    let mut activities = Vec::new();
    let mut titers = Vec::new();
    for aliquot in 0..96 {
        let activity = 18.0 + 5.0 * synthetic::normal(aliquot);
        let titer = 42.0 + 2.2 * activity + 8.0 * synthetic::normal(aliquot + 200);
        activities.push(activity);
        titers.push(titer);
    }
    // ANCHOR_END: joint-values

    // ANCHOR: joint-plot
    // Marginal owns its central scatter and top/right KDE strips; no generic
    // faceting or separate histogram/scatter reconstruction is needed.
    Plot::marginal(
        NumericValues::new("Enzyme activity (U/mL)", activities),
        NumericValues::new("Product titer (mg/L)", titers),
    )
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(1000, 900)
    .x_tick_generation(TickGeneration::target(5)?)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Enzyme activity (U/mL)")
    .y_label("Product titer (mg/L)")
    .title("Read the pair and both marginal distributions")
    .subtitle("96 paired synthetic measurements of enzyme activity and product titer")
    .caption(
        "Each point is one synthetic aliquot. Top: activity density (mL/U); right: titer density (L/mg). \
         Both KDEs use the same paired rows. No regression, statistical test or causal claim is made.",
    )
    .svg(directory.join("joint_measurements.svg"))?;
    // ANCHOR_END: joint-plot
    Ok(())
}
// ANCHOR_END: example
