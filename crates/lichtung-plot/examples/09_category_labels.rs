//! Label eligibility is independent from scientific categories, cells and marks.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    BarInput, Canvas, CategoryLabelSelection, CategoryValues, Labels, NumericValues, Plot,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let bars = Plot::bar(
        CategoryValues::new("month", months)?,
        BarInput::estimates(NumericValues::new("value", [1.0; 12])),
    )
    .labels(Labels::raw())
    .x_category_labels(CategoryLabelSelection::every(2)?)
    .title("Twelve bars, six month labels");
    let cells = Plot::heatmap(
        CategoryValues::new("column", ["A", "B", "C", "D"].repeat(4))?,
        CategoryValues::new(
            "row",
            ["One", "Two", "Three", "Four"]
                .into_iter()
                .flat_map(|row| std::iter::repeat_n(row, 4)),
        )?,
        NumericValues::new("value", (1..=16).map(f64::from)),
    )
    .labels(Labels::raw())
    .x_category_labels(CategoryLabelSelection::every(2)?)
    .y_category_labels(CategoryLabelSelection::every(2)?.offset(1)?)
    .title("Full grid, independent x/y labels");
    let output = Canvas::grid(2, 1)?
        .panel(0, 0, bars)?
        .panel(1, 0, cells)?
        .size(1400, 1000)
        .svg_preserve_text(directory.join("category_labels.svg"))?;
    println!(
        "category_labels.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
// ANCHOR_END: example
