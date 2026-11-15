//! Four named synthetic measurements on the same aliquots, in one native Matrix.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{Labels, NumericValues, Plot};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;

    // ANCHOR: assay-values
    let mut activities = Vec::new();
    let mut titers = Vec::new();
    let mut residual_glucose = Vec::new();
    let mut acidity = Vec::new();
    for aliquot in 0..96 {
        // Fixed synthetic variation, not a fitted model or a mechanistic law.
        let shared_variation = synthetic::normal(aliquot);
        let assay_variation = synthetic::normal(aliquot + 200);
        activities.push(18.0 + 4.5 * shared_variation);
        titers.push(75.0 + 10.0 * shared_variation + 4.0 * assay_variation);
        residual_glucose.push(6.5 - shared_variation + 0.5 * synthetic::normal(aliquot + 400));
        acidity.push(6.8 - 0.15 * shared_variation + 0.07 * assay_variation);
    }
    // ANCHOR_END: assay-values

    // ANCHOR: assay-matrix
    // Matrix owns its full 4 x 4 layout, ticks and named-variable labels.
    // It is neither a generic facet nor an admissible Canvas child.
    Plot::matrix(vec![
        NumericValues::new("Activity (U/mL)", activities),
        NumericValues::new("Titer (mg/L)", titers),
        NumericValues::new("Residual glucose (g/L)", residual_glucose),
        NumericValues::new("pH (dimensionless)", acidity),
    ])
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(1320, 1320)
    .title("Which assay variables move together?")
    .subtitle("Four named measurements on the same 96 synthetic aliquots")
    .caption(
        "Off-diagonal cells retain paired rows; diagonal curves show each variable's density. \
         These synthetic associations are descriptive, not fitted effects or statistical tests.",
    )
    .svg(directory.join("multivariate_assay.svg"))?;
    // ANCHOR_END: assay-matrix
    Ok(())
}
// ANCHOR_END: example
