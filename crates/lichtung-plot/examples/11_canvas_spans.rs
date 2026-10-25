//! A capped root with one tall scientific owner beside two independent owners.

use lichtung_plot::{Canvas, NumericValues, Plot};

fn main() -> lichtung_plot::Result<()> {
    let plot = || {
        Plot::scatter(
            NumericValues::new("x", [0.0, 1.0, 2.0]),
            NumericValues::new("y", [1.0, 0.0, 2.0]),
        )
    };
    let output = Canvas::grid(2, 2)?
        .panel(0, 0, plot().title("Upper left"))?
        .panel(1, 0, plot().title("Lower left"))?
        .panel_span(0, 1, 2, 1, plot().title("One continuous tall owner"))?
        .column_header(0, 2, "Three independent scientific sources")?
        .row_header(0, 1, "Upper row")?
        .row_header(1, 1, "Lower row")?
        .max_width(1200)
        .svg_preserve_text(std::env::temp_dir().join("canvas_spans"))?;
    assert!(output.width() <= 1200);
    println!(
        "canvas_spans.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
