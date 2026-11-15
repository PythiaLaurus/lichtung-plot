//! Four descriptive views of the same synthetic grouped assay measurements.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{
    Canvas, CategoryValues, Labels, LegendPolicy, LegendPosition, NumericValues, Plot, QqLine,
    RawOverlay, Text, TickGeneration,
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

    // ANCHOR: diagnostic-values
    let mut schedules = Vec::new();
    let mut delays = Vec::new();
    let mut warm_hold = Vec::new();
    for (schedule, center, spread, skew, offset) in [
        ("Fresh", 24.0, 3.0, 0.0, 0),
        ("Cold hold", 29.0, 4.5, 0.3, 80),
        ("Warm hold", 34.0, 6.0, 1.4, 160),
    ] {
        for sample in 0..72 {
            let deviation = synthetic::normal(sample + offset);
            let delay = center + spread * deviation + skew * (deviation * deviation - 1.0);
            schedules.push(schedule);
            delays.push(delay);
            if schedule == "Warm hold" {
                warm_hold.push(delay);
            }
        }
    }
    let schedules = CategoryValues::new("Holding schedule", schedules)?;
    let delays = NumericValues::new("Completion delay (min)", delays);
    // ANCHOR_END: diagnostic-values

    // ANCHOR: diagnostic-panels
    let density = Plot::density(delays.clone())
        .hue(schedules.clone())
        .labels(Labels::raw())
        .x_label("Completion delay (min)")
        .y_label("Density (1/min)")
        .x_tick_generation(TickGeneration::target(5)?)
        .y_tick_generation(TickGeneration::target(4)?)
        .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
        .title(Text::new("A - Density by schedule").style(panel_title.clone()))
        .subtitle("One density curve per holding schedule");

    // Q-Q uses an explicitly selected group, not a pooled mixture of schedules.
    let normal_departure = Plot::qq(NumericValues::new("Warm-hold delay (min)", warm_hold))
        .reference_line(QqLine::Quartiles)
        .labels(Labels::raw())
        .x_label("Theoretical standard-normal quantile")
        .y_label("Warm-hold delay (min)")
        .x_tick_generation(TickGeneration::target(5)?)
        .y_tick_generation(TickGeneration::target(5)?)
        .title(Text::new("B - Warm-hold normal Q-Q").style(panel_title.clone()))
        .subtitle("Quartile reference line; descriptive, not a test");

    let quartiles = Plot::boxplot(schedules.clone(), delays.clone())
        .raw_overlay(RawOverlay::Jittered)
        .labels(Labels::raw())
        .x_label("Holding schedule")
        .y_label("Completion delay (min)")
        .y_tick_generation(TickGeneration::target(5)?)
        .title(Text::new("C - Medians and quartiles").style(panel_title.clone()))
        .subtitle("Boxes, whiskers and all 216 observations");

    let shape = Plot::violin(schedules, delays)
        .raw_overlay(RawOverlay::Jittered)
        .labels(Labels::raw())
        .x_label("Holding schedule")
        .y_label("Completion delay (min)")
        .y_tick_generation(TickGeneration::target(5)?)
        .title(Text::new("D - Shape and every sample").style(panel_title))
        .subtitle("The same grouped measurements as A and C");
    // ANCHOR_END: diagnostic-panels

    // ANCHOR: diagnostic-canvas
    Canvas::grid(2, 2)?
        .theme(theme)
        .size(1360, 1120)
        .title("Does holding change the shape of an assay delay?")
        .subtitle("Compare overlap, normal departure, group medians and distribution shape")
        .caption(
            "Synthetic assay delays; 72 observations per schedule. Q-Q uses Warm hold only. \
             These are descriptive diagnostics, not evidence from a study or a normality test.",
        )
        .panel(0, 0, density)?
        .panel(0, 1, normal_departure)?
        .panel(1, 0, quartiles)?
        .panel(1, 1, shape)?
        .svg(directory.join("distribution_diagnostics.svg"))?;
    // ANCHOR_END: diagnostic-canvas
    Ok(())
}
// ANCHOR_END: example
