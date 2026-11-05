//! Chunked nullable synthetic observations with explicit original sample IDs.

// ANCHOR: example
use lichtung_plot_polars::{kind, Labels, Plot, Scatter, Theme};
use polars_core::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = match arguments.next() {
        Some(path) if !path.is_empty() => std::path::PathBuf::from(path),
        Some(_) => return Err("output directory must not be empty".into()),
        None => std::env::current_dir()?,
    };
    if arguments.next().is_some() {
        return Err("usage: grouped_observations [output-directory]".into());
    }
    std::fs::create_dir_all(&directory)?;
    // ANCHOR: chunked-values
    let mut response = Series::new(
        "Response (a.u.)".into(),
        [Some(0.2), Some(0.8), None, Some(1.6), Some(2.1), Some(2.4)],
    );
    response.append(&Series::new(
        "Response (a.u.)".into(),
        [
            Some(0.3),
            Some(1.2),
            Some(1.8),
            Some(2.5),
            Some(3.0),
            Some(3.4),
        ],
    ))?;
    let frame = DataFrame::new(
        response.len(),
        vec![
            Column::new(
                "Concentration (µM)".into(),
                [0.2, 0.7, 1.2, 1.8, 2.5, 3.2, 0.2, 0.7, 1.2, 1.8, 2.5, 3.2],
            ),
            response.into_column(),
            Column::new(
                "Condition".into(),
                [
                    "Baseline",
                    "Baseline",
                    "Baseline",
                    "Baseline",
                    "Baseline",
                    "Baseline",
                    "Stimulated",
                    "Stimulated",
                    "Stimulated",
                    "Stimulated",
                    "Stimulated",
                    "Stimulated",
                ],
            ),
            Column::new(
                "Sample ID".into(),
                [
                    101_u64, 102, 103, 104, 105, 106, 201, 202, 203, 204, 205, 206,
                ],
            ),
        ],
    )?;
    // ANCHOR_END: chunked-values
    // ANCHOR: observation-plot
    let base = Theme::publication();
    let radii = base.marks().get_points().get_radii().clone().scatter(4)?;
    let points = base.marks().get_points().clone().radii(radii)?;
    let theme = base
        .customize()
        .marks(base.marks().clone().points(points)?)
        .build()?;
    let plot = Scatter::from_data(&frame, "Concentration (µM)", "Response (a.u.)")?
        .hue("Condition")?
        .source_rows("Sample ID")?
        .theme(theme)
        .labels(Labels::raw())
        .size(800, 480)
        .title("Observational response by condition")
        .caption("Synthetic illustration · one response missing · no inferred fit or significance");
    let plot: Plot<kind::Scatter> = plot.into();
    drop(frame); // Native handoff ends the adapter's field-lookup borrow.
    plot.svg(directory.join("grouped_observations.svg"))?
        .svg_preserve_text(directory.join("grouped_observations_text.svg"))?;
    // ANCHOR_END: observation-plot
    Ok(())
}
// ANCHOR_END: example
