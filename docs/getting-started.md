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

To attach caller-computed pairwise results, construct plot-owned
`Significance<Pairwise>` display data. The placement marker is inferred from
`Significance::between`:

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

## Bars

Ordinary bars make the value source explicit. Raw samples render cell means and
default to deterministic 95% percentile-bootstrap confidence intervals:

```rust
use lichtung_plot::{BarInput, CategoryValues, NumericValues, Plot};

let plot = Plot::bar(
    CategoryValues::new("group", ["control", "control", "active", "active"] )?,
    BarInput::samples(NumericValues::new("response", [1.0, 1.5, 2.0, 2.5])),
);
# let _ = plot;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Generated error bars use variant-specific configuration. For example, select a
90% Student-t interval or configure the bootstrap seed and resample count:

```rust
use lichtung_plot::{BarInput, Bootstrap, ErrorBar, NumericValues};

let student_t = BarInput::samples(NumericValues::new("response", [1.0, 2.0, 3.0]))
    .error_bar(ErrorBar::confidence_interval().level(0.9)?.student_t());
let bootstrap = BarInput::samples(NumericValues::new("response", [1.0, 2.0, 3.0]))
    .error_bar(ErrorBar::confidence_interval().bootstrap(
        Bootstrap::new().resamples(2_000)?.seed(42),
    ));
# let _ = (student_t, bootstrap);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Precomputed estimates never imply confidence or credible semantics. Supply
drawable bounds directly when needed:

```rust
use lichtung_plot::{BarInput, Bounds, NumericValues};

let input = BarInput::estimates_with_error_bars(
    NumericValues::new("estimate", [1.2, 2.4]),
    Bounds::new(
        NumericValues::new("lower", [0.9, 2.0]),
        NumericValues::new("upper", [1.6, 2.9]),
    ),
);
# let _ = input;
```

## Closed Sources

Histograms and rainclouds select their complete data source at construction:

```rust
use lichtung_plot::{CategoryValues, HistogramInput, NumericValues, Plot, RaincloudInput};

let values = NumericValues::new("response", [1.0, 1.5, 2.0, 2.5]);
let histogram = Plot::histogram(HistogramInput::samples(values.clone()));
let raincloud = Plot::raincloud(RaincloudInput::samples(
    CategoryValues::new("group", ["control", "control", "active", "active"] )?,
    values,
));
# let _ = (histogram, raincloud);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `HistogramInput::geometry` or `RaincloudInput::geometry` instead when the
caller owns complete display geometry. Geometry and samples cannot be combined
on one plot.

A caller-computed KDE can instead remain a layer over sample-owned histogram
bins and rugs. Its coordinates use the same named point type as complete
histogram geometry:

```rust
use lichtung_plot::{
    HistogramComposite, HistogramInput, HistogramKde, HistogramKdePoint, HistogramStat,
    NumericValues, Plot,
};

let histogram = Plot::histogram(HistogramInput::samples(NumericValues::new(
    "response",
    [0.0, 0.5, 1.5, 2.0],
)))
.stat(HistogramStat::Density)
.composite(HistogramComposite::KdeAndRug(HistogramKde::Precomputed(
    vec![
        HistogramKdePoint { x: 0.0, y: 0.1 },
        HistogramKdePoint { x: 1.0, y: 0.3 },
        HistogramKdePoint { x: 2.0, y: 0.1 },
    ],
)));
# let _ = histogram;
```

## Paths And Facets

Ordinary lines, grouped trajectories, and source-ordered numeric paths are
different typed families. Generic facets are supplied as one complete layout:

```rust
use lichtung_plot::{CategoryValues, FacetSpec, NumericValues, Plot};

let plot = Plot::trajectory(
    NumericValues::new("time", [1.0, 2.0, 1.0, 2.0]),
    NumericValues::new("response", [1.0, 1.5, 2.0, 2.5]),
    CategoryValues::new("subject", ["a", "a", "b", "b"] )?,
)
.facet(FacetSpec::column(
    CategoryValues::new("site", ["north", "north", "south", "south"] )?,
));
# let _ = plot;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `FacetSpec::row`, `FacetSpec::grid`, or `FacetSpec::wrap` for the other
supported layouts. Wrap construction rejects a zero column count.
