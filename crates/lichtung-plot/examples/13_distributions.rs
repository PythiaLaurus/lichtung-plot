//! Compare complete synthetic sample distributions, not only their averages.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{CategoryValues, Labels, LegendPolicy, NumericValues, Plot, RaincloudInput};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    // ANCHOR: distribution-values
    let mut conditions = Vec::new();
    let mut times = Vec::new();
    for (condition, center, spread, offset) in [
        ("Baseline", 620.0, 65.0, 0),
        ("Practice", 485.0, 42.0, 100),
        ("Transfer", 535.0, 80.0, 200),
    ] {
        for sample in 0..80 {
            let z = synthetic::normal(sample + offset);
            conditions.push(condition);
            times.push(center + spread * z + 12.0 * z * z);
        }
    }
    // ANCHOR_END: distribution-values
    // ANCHOR: distribution-plot
    Plot::raincloud(RaincloudInput::samples(
        CategoryValues::new("Condition", conditions)?,
        NumericValues::new("Completion time (ms)", times),
    ))
    .labels(Labels::raw())
    .theme(gallery::theme()?)
    .size(840, 560)
    .title("Averages do not tell the whole story")
    .subtitle("Distribution shape, quartiles and every observation")
    .caption("Synthetic samples · 80 observations per condition · no significance claim")
    .legend(LegendPolicy::Hidden)
    .svg(directory.join("distributions.svg"))?;
    // ANCHOR_END: distribution-plot
    Ok(())
}
// ANCHOR_END: example
