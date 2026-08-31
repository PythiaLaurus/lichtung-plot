//! Example 04: Canvas theme inheritance.
//!
//! Renders one default Publication Canvas and one Nature Canvas with an
//! inherited child and an explicit Science replacement. Run with:
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
    Canvas::grid(1, 1)?
        .panel(0, 0, scatter("Publication", [1.0, 3.0, 2.0]))?
        .svg(&default_path)?;
    println!("wrote {}", default_path.display());

    let inherited_path = directory.join("canvas_theme_inheritance.svg");
    Canvas::grid(1, 2)?
        .theme(Theme::Nature)
        .panel(0, 0, scatter("Inherited Nature", [1.0, 3.0, 2.0]))?
        .panel(
            0,
            1,
            scatter("Explicit Science", [2.0, 1.0, 3.0]).theme(Theme::Science),
        )?
        .svg(&inherited_path)?;
    println!("wrote {}", inherited_path.display());

    Ok(())
}
