# Getting Started

`lichtung-plot` renders owned immutable display values. SVG support is always
available; enable `png` only when raster output is needed. Callers convert
external data containers into the renderer's native values at their boundary.

```toml
[dependencies]
lichtung-plot = { git = "<repository-url>", rev = "<commit>" }
```

For a neighboring local checkout:

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot" }
```

```rust
use lichtung_plot::{NumericValues, Plot, Theme};

Plot::scatter(
    NumericValues::new("dose", [1.0, 2.0, 3.0]),
    NumericValues::new("response", [1.5, 2.5, 2.0]),
)
    .theme(Theme::Publication)
    .svg("response.svg")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

No runtime initialization or asset provisioning is required. The destination
parent must already exist.

To attach caller-computed pairwise results, construct plot-owned display data:

```rust
use lichtung_plot::{CategoryValues, ComparisonAnnotation, NumericValues, Plot};

let comparisons = [
    ComparisonAnnotation::new("control", "treatment", 0.01)
        .adjusted(0.02, "Holm"),
];

let plot = Plot::boxplot(
    CategoryValues::new("group", ["control", "control", "treatment", "treatment"] )?,
    NumericValues::new("response", [1.0, 1.5, 2.0, 2.5]),
)
.add_comparisons(comparisons);
# let _ = plot;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The renderer validates and displays these values but does not perform or choose
a statistical test.
