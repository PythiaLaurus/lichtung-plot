//! A composed calibration relationship and its generating-model residuals.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{
    Canvas, HistogramBins, HistogramComposite, HistogramInput, HistogramKde, HistogramStat, Labels,
    LegendPolicy, NumericValues, Plot, RegressionBand, Text, TickGeneration,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let theme = gallery::theme()?;
    let panel_title = theme
        .typography()
        .get_root()
        .get_title()
        .clone()
        .font_size(17.0)?;
    // ANCHOR: calibration-values
    let concentrations = (0..96).map(|i| f64::from(i) / 5.0).collect::<Vec<_>>();
    let residuals = (0..96)
        .map(|i| 4.5 * synthetic::normal(i))
        .collect::<Vec<_>>();
    let signals = concentrations
        .iter()
        .zip(&residuals)
        .map(|(x, error)| 2.0 + 3.2 * x + error)
        .collect::<Vec<_>>();
    // ANCHOR_END: calibration-values
    // ANCHOR: calibration-panels
    let relationship = Plot::regression(
        NumericValues::new("Concentration (ug/mL)", concentrations),
        NumericValues::new("Detector signal (a.u.)", signals),
    )
    .band(RegressionBand::Hidden)
    .legend(LegendPolicy::Hidden)
    .labels(Labels::raw())
    .x_tick_generation(TickGeneration::target(5)?)
    .y_tick_generation(TickGeneration::target(5)?)
    .title(Text::new("A · Calibration").style(panel_title.clone()));
    let errors = Plot::histogram(HistogramInput::samples(NumericValues::new(
        "Generating-model residual (a.u.)",
        residuals,
    )))
    .bins(HistogramBins::Count(14))
    .stat(HistogramStat::Density)
    .composite(HistogramComposite::KdeAndRug(HistogramKde::Internal))
    .labels(Labels::raw())
    .x_tick_generation(TickGeneration::target(5)?)
    .y_tick_generation(TickGeneration::target(4)?)
    .title(Text::new("B · Residual distribution").style(panel_title));
    // ANCHOR_END: calibration-panels
    // ANCHOR: calibration-canvas
    Canvas::grid(1, 2)?
        .theme(theme)
        .size(1000, 620)
        .title("A calibration is more than a fitted line")
        .subtitle("Relationship and residual structure in one composed figure")
        .caption("Synthetic calibration · residuals use the known generating line, not the fitted OLS line")
        .panel(0, 0, relationship)?
        .panel(0, 1, errors)?
        .svg(directory.join("calibration.svg"))?;
    // ANCHOR_END: calibration-canvas
    Ok(())
}
// ANCHOR_END: example
