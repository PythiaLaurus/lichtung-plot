//! Observed pair keys versus marginal channel sections, with one guide per owner.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    Bounds, Canvas, CanvasLegendPolicy, CategoryValues, InteractionStyle, Labels,
    LegendComposition, LinePattern, NumericValues, Plot, PointShape,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let mut conditions = Vec::new();
    let mut groups = Vec::new();
    let mut hues = Vec::new();
    let mut estimates = Vec::new();
    for group in 0..5 {
        for hue in 0..4 {
            for condition in 0..2 {
                conditions.push(if condition == 0 { "Before" } else { "After" });
                groups.push(format!("G{}", group + 1));
                hues.push(format!("H{}", hue + 1));
                estimates.push(f64::from(group * 3 + hue) + f64::from(condition) * 0.5);
            }
        }
    }
    let lower = estimates
        .iter()
        .map(|value| value - 0.2)
        .collect::<Vec<_>>();
    let upper = estimates
        .iter()
        .map(|value| value + 0.2)
        .collect::<Vec<_>>();
    let styles = (0..5)
        .map(|group| {
            InteractionStyle::new(format!("G{}", group + 1))?
                .point_shape(if group % 2 == 0 {
                    PointShape::Circle
                } else {
                    PointShape::TriangleUp
                })?
                .line_pattern(if group % 2 == 0 {
                    LinePattern::Solid
                } else {
                    LinePattern::Dashed
                })
        })
        .collect::<lichtung_plot::Result<Vec<_>>>()?;
    let plot = Plot::interaction(
        CategoryValues::new("condition", conditions)?,
        NumericValues::new("estimate", estimates),
        CategoryValues::new("group", groups)?,
        Bounds::new(
            NumericValues::new("lower", lower),
            NumericValues::new("upper", upper),
        ),
    )
    .hue(CategoryValues::new("hue", hues)?)
    .styles(styles)
    .labels(Labels::raw());
    let combined = plot.clone().legend_composition(LegendComposition::Combined);
    let separate = plot.legend_composition(LegendComposition::Separate);
    let output = Canvas::grid(1, 2)?
        .size(1600, 1000)
        .panel(0, 0, combined.clone().title("20 observed pair keys"))?
        .panel(0, 1, separate.clone().title("5 group + 4 hue keys"))?
        .svg_preserve_text(directory.join("legend_sections.svg"))?;
    let shared = Canvas::grid(1, 2)?
        .size(1600, 1000)
        .legend_composition(LegendComposition::Separate)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel(0, 0, combined)?
        .panel(0, 1, separate)?
        .svg_preserve_text(directory.join("shared_legend_sections.svg"))?;
    println!(
        "legend_sections.svg: {}×{}; shared_legend_sections.svg: {}×{} CSS pixels",
        output.width(),
        output.height(),
        shared.width(),
        shared.height(),
    );
    Ok(())
}
// ANCHOR_END: example
