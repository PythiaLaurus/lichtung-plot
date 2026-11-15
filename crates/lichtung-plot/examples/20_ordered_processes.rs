//! Identity, input order, condition order and cumulative transfers are distinct.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    Bounds, Canvas, CategoryValues, InteractionStyle, Labels, LegendPolicy, LegendPosition,
    LinePathStyle, LinePattern, NumericValues, Plot, PointShape, Text, TickGeneration,
    WaterfallLabels,
};

#[expect(
    clippy::too_many_lines,
    reason = "keep four explicit process record types and their scientific context together"
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

    // ANCHOR: process-values
    let mut times = Vec::new();
    let mut oxygen = Vec::new();
    let mut reactors = Vec::new();
    for (reactor, readings) in [
        ("R1", [2.0, 3.8, 5.1, 6.2, 6.7, 6.9]),
        ("R2", [2.4, 4.0, 5.0, 6.0, 6.6, 6.8]),
        ("R3", [1.8, 3.2, 4.7, 5.8, 6.5, 6.7]),
        ("R4", [2.2, 3.5, 4.8, 5.6, 6.1, 6.3]),
    ] {
        for (time, reading) in [0.0, 5.0, 10.0, 15.0, 20.0, 30.0].into_iter().zip(readings) {
            times.push(time);
            oxygen.push(reading);
            reactors.push(reactor);
        }
    }
    let reactors = CategoryValues::new("Reactor", reactors)?;

    let mut conditions = Vec::new();
    let mut flows = Vec::new();
    let mut profiles = Vec::new();
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (aeration, estimates, lower_bounds, upper_bounds) in [
        (
            "Low flow",
            [6.6, 3.8, 5.9],
            [6.2, 3.3, 5.5],
            [7.0, 4.3, 6.3],
        ),
        (
            "High flow",
            [6.8, 4.8, 6.9],
            [6.4, 4.3, 6.5],
            [7.2, 5.3, 7.3],
        ),
    ] {
        for (((condition, estimate), lower_bound), upper_bound) in
            ["Pre-pulse", "Pulse", "Recovery"]
                .into_iter()
                .zip(estimates)
                .zip(lower_bounds)
                .zip(upper_bounds)
        {
            conditions.push(condition);
            flows.push(aeration);
            profiles.push(estimate);
            lower.push(lower_bound);
            upper.push(upper_bound);
        }
    }
    let flows = CategoryValues::new("Aeration", flows)?;
    // ANCHOR_END: process-values

    // ANCHOR: process-panels
    let repeated = Plot::trajectory(
        NumericValues::new("Elapsed time (min)", times),
        NumericValues::new("Dissolved oxygen (mg/L)", oxygen),
        reactors.clone(),
    )
    .hue(reactors)
    .labels(Labels::raw())
    .x_limits(-1.0, 31.0)
    .y_limits(0.0, 8.0)
    .x_tick_generation(TickGeneration::target(4)?)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Elapsed time (min)")
    .y_label("Dissolved oxygen (mg/L)")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("A - Keep each reactor intact").style(panel_title.clone()))
    .subtitle("Six repeated readings per reactor; not an averaged curve");

    // Temperatures return to earlier values. Sorting x would destroy this loop.
    let cycle = Plot::line_path(
        NumericValues::new(
            "Temperature (C)",
            [20.0, 23.0, 28.0, 33.0, 36.0, 33.0, 28.0, 23.0, 20.0],
        ),
        NumericValues::new(
            "Dissolved oxygen (mg/L)",
            [7.0, 6.5, 5.7, 4.7, 4.0, 4.5, 5.4, 6.6, 7.0],
        ),
        CategoryValues::new("Cycle", ["R1 heat pulse"; 9])?,
    )
    .hue(CategoryValues::new("Cycle", ["R1 heat pulse"; 9])?)
    .style(LinePathStyle::Hue)
    .markers(true)
    .labels(Labels::raw())
    .x_tick_generation(TickGeneration::target(5)?)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Temperature (C)")
    .y_label("Dissolved oxygen (mg/L)")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("B - Heating and cooling form a loop").style(panel_title.clone()))
    .subtitle("Connect in input order: 20 to 36 C, then back to 20 C")
    .caption("Start and finish coincide; repeated temperatures are retained.");

    let interaction = Plot::interaction(
        CategoryValues::new("Process condition", conditions)?,
        NumericValues::new("Dissolved oxygen (mg/L)", profiles),
        flows.clone(),
        Bounds::new(
            NumericValues::new("Supplied lower oxygen (mg/L)", lower),
            NumericValues::new("Supplied upper oxygen (mg/L)", upper),
        ),
    )
    .hue(flows)
    .styles([
        InteractionStyle::new("Low flow")?.line_pattern(LinePattern::Dashed)?,
        InteractionStyle::new("High flow")?.point_shape(PointShape::TriangleUp)?,
    ])
    .connectors(true)
    .markers(true)
    .labels(Labels::raw())
    .y_limits(0.0, 8.0)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Process condition")
    .y_label("Dissolved oxygen (mg/L)")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("C - Compare condition profiles").style(panel_title.clone()))
    .subtitle("Supplied profiles and bounds; no interaction test");

    let inventory = Plot::waterfall(
        CategoryValues::new(
            "Ordered transfer",
            ["Pulse demand", "Aeration", "Sampling", "Recovery feed"],
        )?,
        NumericValues::new("Oxygen transfer (mg)", [-38.0, 32.0, -6.0, 18.0]),
        140.0,
    )
    .total_labels(WaterfallLabels::new("Start", "Finish")?)
    .labels(Labels::raw())
    .x_tick_generation(TickGeneration::target(5)?)
    .x_label("Oxygen inventory (mg)")
    .y_label("Ordered transfer")
    .title(Text::new("D - Account for signed transfers").style(panel_title))
    .subtitle("20 L vessel; starting inventory = 140 mg");
    // ANCHOR_END: process-panels

    // ANCHOR: process-canvas
    Canvas::grid(2, 2)?
        .theme(theme)
        .size(1440, 1160)
        .title("Order is part of the measurement")
        .subtitle("Repeated reactors, a returning sensor path, condition profiles and an inventory ledger")
        .caption(
            "Synthetic process records. Connections are descriptive, not fitted dynamics. \
             The supplied interaction bounds are illustrative, not computed confidence intervals. \
             The renderer accumulates the supplied deltas, ending at 146 mg.",
        )
        .panel(0, 0, repeated)?
        .panel(0, 1, cycle)?
        .panel(1, 0, interaction)?
        .panel(1, 1, inventory)?
        .svg(directory.join("ordered_processes.svg"))?;
    // ANCHOR_END: process-canvas
    Ok(())
}
// ANCHOR_END: example
