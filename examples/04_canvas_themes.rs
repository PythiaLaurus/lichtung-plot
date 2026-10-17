//! Example 04: Canvas theme inheritance.
//!
//! Renders an automatic Publication Canvas and an exact-width Nature Canvas
//! with inherited and explicit Science children, reporting frozen dimensions.
//! Run with:
//!   cargo run --example 04_canvas_themes -p lichtung-plot

use lichtung_plot::{Canvas, NumericValues, Plot, Theme};

fn scatter(name: &'static str, values: [f64; 3]) -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new(name, values),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir();
    let default_path = directory.join("canvas_theme_default.svg");
    let output = Canvas::grid(1, 1)?
        .panel(0, 0, scatter("Publication", [1.0, 3.0, 2.0]))?
        .svg(&default_path)?;
    println!(
        "wrote {} ({}×{} CSS pixels)",
        default_path.display(),
        output.width(),
        output.height()
    );

    let inherited_path = directory.join("canvas_theme_inheritance.svg");
    let output = Canvas::grid(1, 2)?
        .theme(Theme::nature())
        .width(1000)
        .panel(0, 0, scatter("Inherited Nature", [1.0, 3.0, 2.0]))?
        .panel(
            0,
            1,
            scatter("Explicit Science", [2.0, 1.0, 3.0]).theme(Theme::science()),
        )?
        .svg(&inherited_path)?;
    println!(
        "wrote {} ({}×{} CSS pixels)",
        inherited_path.display(),
        output.width(),
        output.height()
    );

    Ok(())
}
