//! Synthetic assay response from alternate columns of a noncontiguous view.

// ANCHOR: example
use lichtung_plot::{CategoryOrder, ColorDomain, ContinuousColorScale, Labels, Plot};
use lichtung_plot_ndarray as nd;
use ndarray::{array, s};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = match arguments.next() {
        Some(path) if !path.is_empty() => std::path::PathBuf::from(path),
        Some(_) => return Err("output directory must not be empty".into()),
        None => std::env::current_dir()?,
    };
    if arguments.next().is_some() {
        return Err("usage: strided_heatmap [output-directory]".into());
    }
    std::fs::create_dir_all(&directory)?;
    // ANCHOR: strided-values
    // Interleaved quality-control columns are deliberately excluded by the view.
    let measurements = array![
        [0.2, 99.0, 0.8, 99.0, 1.4, 99.0, 1.7, 99.0],
        [0.1, 99.0, 0.6, 99.0, 1.1, 99.0, 1.3, 99.0],
        [-0.1, 99.0, -0.5, 99.0, -0.9, 99.0, -1.4, 99.0],
        [0.0, 99.0, 0.3, 99.0, 0.9, 99.0, 1.2, 99.0],
    ];
    let times = ["0 h", "2 h", "6 h", "12 h"];
    let programs = [
        "Inflammation",
        "Stress response",
        "Cell cycle",
        "DNA repair",
    ];
    let cells = nd::heatmap_values(
        "Response (log2 fold change)",
        measurements.slice(s![.., ..;2]),
        ("Time after stimulus", &times),
        ("Response program", &programs),
    )?;
    // ANCHOR_END: strided-values
    // ANCHOR: strided-plot
    Plot::heatmap(cells.x, cells.y, cells.fill)
        .continuous_color_scale(ContinuousColorScale::diverging(
            0.0,
            None,
            Some(ColorDomain::exact(-2.0, 2.0)?),
        )?)
        .x_category_order(CategoryOrder::explicit(times))
        .y_category_order(CategoryOrder::explicit(programs))
        .labels(Labels::raw())
        .size(780, 480)
        .title("Response programs from a strided assay matrix")
        .caption("Synthetic illustration · alternate source columns · no inferred clustering")
        .svg(directory.join("strided_heatmap.svg"))?
        .svg_preserve_text(directory.join("strided_heatmap_text.svg"))?;
    // ANCHOR_END: strided-plot
    Ok(())
}
// ANCHOR_END: example
