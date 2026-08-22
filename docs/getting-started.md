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
use lichtung_plot::{
    CategoryValues, NumericValues, Plot, Significance, SignificanceMark,
};

let significance = [
    Significance::between(
        "control",
        "treatment",
        SignificanceMark::PValue(0.02),
    ),
];

let plot = Plot::boxplot(
    CategoryValues::new("group", ["control", "control", "treatment", "treatment"] )?,
    NumericValues::new("response", [1.0, 1.5, 2.0, 2.5]),
)
.significance(significance);
# let _ = plot;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The caller supplies the exact display p-value after any desired adjustment. The
renderer validates and displays it but does not perform or choose a statistical
test.
