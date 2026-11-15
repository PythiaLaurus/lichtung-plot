//! Supplied levels, contrasts and paired changes have different comparison targets.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    BarInput, Bounds, Canvas, CategoryValues, CoefficientInput, Color, DumbbellEndpoints, Labels,
    LegendPolicy, LegendPosition, NumericValues, Plot, Text, TickGeneration,
};

#[expect(
    clippy::too_many_lines,
    reason = "keep the supplied summaries and five distinct comparison views together"
)]
fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let theme = gallery::theme()?;
    let panel_title = theme
        .typography()
        .get_root()
        .get_title()
        .clone()
        .font_size(17.0)?;

    // ANCHOR: estimate-values
    // These are supplied illustrative summaries, not estimates computed here.
    let media = CategoryValues::new("Medium", ["Low carbon", "Reference", "Enriched"])?;
    let titers = NumericValues::new("After-hold titer (mg/L)", [42.0, 56.0, 68.0]);
    let titer_bounds = Bounds::new(
        NumericValues::new("Supplied lower titer (mg/L)", [37.0, 51.0, 61.0]),
        NumericValues::new("Supplied upper titer (mg/L)", [47.0, 62.0, 75.0]),
    );
    let before = NumericValues::new("Before-hold titer (mg/L)", [38.0, 54.0, 61.0]);
    let changes = NumericValues::new("After - before titer (mg/L)", [4.0, 2.0, 7.0]);
    let change_bounds = Bounds::new(
        NumericValues::new("Supplied lower change (mg/L)", [1.0, -1.0, 3.0]),
        NumericValues::new("Supplied upper change (mg/L)", [7.0, 5.0, 11.0]),
    );
    // ANCHOR_END: estimate-values

    // ANCHOR: estimate-panels
    let intervals = Plot::point_interval(media.clone(), titers.clone(), titer_bounds.clone())
        .zero_line()
        .labels(Labels::raw())
        .x_limits(0.0, 85.0)
        .x_tick_generation(TickGeneration::target(5)?)
        .x_label("After-hold titer (mg/L)")
        .y_label("Medium")
        .title(Text::new("A - Supplied titer").style(panel_title.clone()))
        .subtitle("Absolute level; zero is the physical baseline");

    // A reference-relative additive contrast is not an absolute concentration.
    let contrasts = Plot::coefficient(
        CategoryValues::new("Contrast", ["Low carbon", "Enriched", "Long hold"])?,
        CoefficientInput::estimates_with_intervals(
            NumericValues::new("Change vs reference (mg/L)", [-14.0, 12.0, -6.0]),
            Bounds::new(
                NumericValues::new("Supplied lower contrast (mg/L)", [-21.0, 5.0, -12.0]),
                NumericValues::new("Supplied upper contrast (mg/L)", [-7.0, 19.0, 0.0]),
            ),
        ),
    )
    .reference_line(0.0)
    .labels(Labels::raw())
    .x_limits(-25.0, 25.0)
    .x_tick_generation(TickGeneration::target(5)?)
    .x_label("Change vs reference (mg/L)")
    .y_label("Contrast")
    .title(Text::new("B - Reference contrasts").style(panel_title.clone()))
    .subtitle("Supplied additive effects; null = 0 mg/L");

    let paired = Plot::dumbbell(
        media.clone(),
        before,
        titers.clone(),
        DumbbellEndpoints::new(
            "Before hold",
            Color::hex("#7772A5")?,
            "After hold",
            Color::hex("#207F83")?,
        )?,
    )
    .labels(Labels::raw())
    .x_limits(0.0, 85.0)
    .x_tick_generation(TickGeneration::target(5)?)
    .x_label("Product titer (mg/L)")
    .y_label("Medium")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("C - Before and after").style(panel_title.clone()))
    .subtitle("Two supplied endpoints per medium");

    // Explicit estimate inputs avoid sample-derived means/bootstrap intervals.
    let absolute = Plot::bar(
        media.clone(),
        BarInput::estimates_with_error_bars(titers, titer_bounds),
    )
    .labels(Labels::raw())
    .y_limits(0.0, 85.0)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Medium")
    .y_label("After-hold titer (mg/L)")
    .title(Text::new("D - Absolute titer").style(panel_title.clone()))
    .subtitle("Bar heights compare supplied amounts from a zero baseline");

    let change = Plot::horizontal_bar(
        media,
        BarInput::estimates_with_error_bars(changes, change_bounds),
    )
    .zero_line()
    .labels(Labels::raw())
    .x_limits(-3.0, 12.0)
    .x_tick_generation(TickGeneration::target(5)?)
    .x_label("After - before titer (mg/L)")
    .y_label("Medium")
    .title(Text::new("E - Change after hold").style(panel_title))
    .subtitle("Supplied paired differences; zero means no change");
    // ANCHOR_END: estimate-panels

    // ANCHOR: estimate-canvas
    Canvas::grid(2, 6)?
        .theme(theme)
        .size(1680, 1160)
        .title("Separate the level, the contrast and the change")
        .subtitle("Five views of supplied assay summaries and paired endpoints")
        .caption(
            "All values and drawable bounds are illustrative and supplied. \
             No model, confidence interval, paired test or other inference is computed here.",
        )
        .panel_span(0, 0, 1, 2, intervals)?
        .panel_span(0, 2, 1, 2, contrasts)?
        .panel_span(0, 4, 1, 2, paired)?
        .panel_span(1, 0, 1, 3, absolute)?
        .panel_span(1, 3, 1, 3, change)?
        .svg(directory.join("estimate_comparisons.svg"))?;
    // ANCHOR_END: estimate-canvas
    Ok(())
}
// ANCHOR_END: example
