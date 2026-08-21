# Getting Started

`lichtung-plot` renders Polars dataframes directly. SVG support is always
available; enable `png` only when raster output is needed.

```toml
[dependencies]
lichtung-plot = { git = "<repository-url>", rev = "<commit>" }
polars = "0.54"
```

For a neighboring local checkout:

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot" }
polars = "0.54"
```

```rust
use lichtung_plot::{Chart, Theme};
use polars::df;

let frame = df!(
    "dose" => [1.0, 2.0, 3.0],
    "response" => [1.5, 2.5, 2.0],
)?;

let mut chart = Chart::new(&frame);
chart
    .x("dose")
    .y("response")
    .theme(Theme::Publication)
    .scatter();

chart.render_to_svg(std::path::Path::new("response.svg"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

No runtime initialization or asset provisioning is required. The destination
parent must already exist.

To attach caller-computed pairwise results, construct plot-owned display data:

```rust
use lichtung_plot::ComparisonAnnotation;

let comparisons = [
    ComparisonAnnotation::new("control", "treatment", 0.01)
        .adjusted(0.02, "Holm"),
];

chart.add_comparisons(comparisons);
```

The renderer validates and displays these values but does not perform or choose
a statistical test.
