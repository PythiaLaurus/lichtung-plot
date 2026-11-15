//! Absolute carbon fractions, a mass balance and individual aliquot measurements.

// ANCHOR: example
#[path = "support/gallery.rs"]
mod gallery;
#[path = "support/output.rs"]
mod output;
#[path = "support/synthetic.rs"]
mod synthetic;

use lichtung_plot::{
    Canvas, CategoryValues, Labels, LegendPolicy, LegendPosition, NumericValues, Plot, Text,
    TickGeneration,
};

#[expect(
    clippy::too_many_lines,
    reason = "keep the composition tables, aliquot observations and three related views together"
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

    // ANCHOR: composition-values
    let fractions = ["Sugars", "Amino acids", "Organic acids"];
    let mut stages = Vec::new();
    let mut stage_fractions = Vec::new();
    let mut concentrations = Vec::new();
    for (stage, amounts) in [
        ("Inlet", [26.0, 14.0, 10.0]),
        ("Midpoint", [15.0, 13.0, 9.0]),
        ("Outlet", [6.0, 10.0, 8.0]),
    ] {
        for (fraction, amount) in fractions.into_iter().zip(amounts) {
            stages.push(stage);
            stage_fractions.push(fraction);
            concentrations.push(amount);
        }
    }

    // A separate supplied balance for one 1 L inlet aliquot:
    // 26 mg C retained + 24 mg C in effluent = 50 mg C initially present.
    let mut fates = Vec::new();
    let mut fate_fractions = Vec::new();
    let mut masses = Vec::new();
    for (fate, amounts) in [
        ("Retained", [20.0, 4.0, 2.0]),
        ("Effluent", [6.0, 10.0, 8.0]),
    ] {
        for (fraction, amount) in fractions.into_iter().zip(amounts) {
            fates.push(fate);
            fate_fractions.push(fraction);
            masses.push(amount);
        }
    }

    let mut aliquot_stages = Vec::new();
    let mut aliquot_carbon = Vec::new();
    let mut absorbance = Vec::new();
    for (stage, center, uv_center, offset) in [
        ("Inlet", 50.0, 0.72, 0),
        ("Midpoint", 37.0, 0.50, 80),
        ("Outlet", 24.0, 0.29, 160),
    ] {
        for aliquot in 0..32 {
            aliquot_stages.push(stage);
            aliquot_carbon.push(center + 4.0 * synthetic::normal(aliquot + offset));
            // This is a separate continuous measurement, not a categorical hue
            // or a second encoding of each point's carbon concentration.
            absorbance.push(uv_center + 0.07 * synthetic::normal(aliquot + offset + 500));
        }
    }
    // ANCHOR_END: composition-values

    // ANCHOR: composition-panels
    let stages = Plot::stacked_bar(
        CategoryValues::new("Treatment stage", stages)?,
        NumericValues::new("Dissolved carbon (mg C/L)", concentrations),
        CategoryValues::new("Carbon fraction", stage_fractions)?,
    )
    .labels(Labels::raw())
    .y_limits(0.0, 55.0)
    .y_tick_generation(TickGeneration::target(5)?)
    .x_label("Treatment stage")
    .y_label("Dissolved carbon (mg C/L)")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("A - Fractions along treatment").style(panel_title.clone()))
    .subtitle("Absolute concentrations; totals are 50, 37 and 24 mg C/L");

    let balance = Plot::stacked_horizontal_bar(
        CategoryValues::new("Carbon fate", fates)?,
        NumericValues::new("Carbon amount (mg C)", masses),
        CategoryValues::new("Carbon fraction", fate_fractions)?,
    )
    .labels(Labels::raw())
    .x_limits(0.0, 30.0)
    .x_tick_generation(TickGeneration::target(4)?)
    .x_label("Carbon amount (mg C)")
    .y_label("Carbon fate")
    .legend(LegendPolicy::Visible(LegendPosition::OutsideBottomLeft))
    .title(Text::new("B - Follow one inlet aliquot").style(panel_title.clone()))
    .subtitle("1 L balance: 26 mg C retained + 24 mg C in effluent");

    let aliquots = Plot::beeswarm(
        CategoryValues::new("Treatment stage", aliquot_stages)?,
        NumericValues::new("Dissolved carbon (mg C/L)", aliquot_carbon),
        NumericValues::new("Absorbance at 254 nm (AU)", absorbance),
    )
    .labels(Labels::raw())
    .x_limits(0.0, 70.0)
    .x_tick_generation(TickGeneration::target(6)?)
    .x_label("Aliquot dissolved carbon (mg C/L)")
    .y_label("Treatment stage")
    .color_label("Absorbance at 254 nm (AU)")
    .title(Text::new("C - See the individual aliquots").style(panel_title))
    .subtitle("32 aliquots per stage; point color is a separate UV measurement");
    // ANCHOR_END: composition-panels

    // ANCHOR: composition-canvas
    Canvas::grid(2, 2)?
        .theme(theme)
        .size(1440, 1120)
        .title("Separate composition, mass balance and sample variation")
        .subtitle("Dissolved-carbon fractions through a synthetic treatment process")
        .caption(
            "Synthetic supplied composition tables and illustrative aliquot observations. \
             Stacks are absolute quantities, not percentages; UV absorbance is continuous color.",
        )
        .panel(0, 0, stages)?
        .panel(0, 1, balance)?
        .panel_span(1, 0, 1, 2, aliquots)?
        .svg(directory.join("composition_profiles.svg"))?;
    // ANCHOR_END: composition-canvas
    Ok(())
}
// ANCHOR_END: example
