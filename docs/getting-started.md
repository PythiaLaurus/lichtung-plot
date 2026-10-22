# Getting Started

`lichtung-plot` renders owned immutable display values. Publication SVG,
preserve-text SVG, and PNG output are unconditional crate capabilities. Callers
convert external data containers into the renderer's native values at their
boundary.

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
use lichtung_plot::{NumericValues, Plot};

Plot::scatter(
    NumericValues::new("dose", [1.0, 2.0, 3.0]),
    NumericValues::new("response", [1.5, 2.5, 2.0]),
)
    .svg("response")?; // Publishes response.svg.
# Ok::<(), Box<dyn std::error::Error>>(())
```

Omitting `Plot::theme` selects the Publication theme for a standalone Plot.

`.svg(path)` is the normal publication method and converts all text to vector
paths. When an accessible, selectable, searchable, and editable SVG is also
needed, chain the call-local `.svg_preserve_text(path)` method on the returned
`Output`; both files reuse one rendered scene:

```rust
use lichtung_plot::{NumericValues, Plot};

Plot::scatter(
    NumericValues::new("x", [1.0, 2.0, 3.0]),
    NumericValues::new("y", [1.5, 2.5, 2.0]),
)
    .svg("response")?
    .svg_preserve_text("response-editable")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Terminal paths are base file destinations. SVG methods append `.svg` and PNG
methods append `.png` unless the final filename already ends in that complete
suffix under ASCII case-insensitive comparison. Existing filename text is kept,
so `response.v1` becomes `response.v1.svg`. No runtime initialization or asset
provisioning is required. The destination parent must already exist. Outlined
SVG and PNG generation use installed system fonts.

## Content-Derived Dimensions

Omitting sizing calls derives both root dimensions from content. On Plot and
Canvas, `.width(600)` allocates exactly 600 CSS pixels and derives height after
reflow; `.size(600, 400)` allocates the exact pair and refuses content that cannot
fit. Every call replaces the full request, so `.size(600, 1).width(600)` clears the
earlier height. Zero dimensions reject during preflight, without setter panics.

The returned `Output` exposes `.width()` and `.height()` for the frozen canonical
scene. PNG options can choose different raster dimensions without changing those
values or redoing layout. Embedded Plot requests do not control Canvas cells;
only the Canvas root request assigns the composed dimensions.

```rust
use lichtung_plot::{Canvas, NumericValues, Plot};

let plot = || Plot::scatter(
    NumericValues::new("dose", [0.0, 1.0, 2.0]),
    NumericValues::new("response", [1.0, 3.0, 2.0]),
);
let output = plot().size(600, 400).width(300).svg("width-controlled")?;
assert_eq!(output.width(), 300);
assert!(output.height() > 0);
let output = Canvas::grid(1, 1)?
    .panel(0, 0, plot())?
    .width(300)
    .size(600, 400)
    .svg("exact-canvas")?;
assert_eq!((output.width(), output.height()), (600, 400));
# Ok::<(), lichtung_plot::Error>(())
```

## Themes And Canvases

Canvas owns one root theme and defaults it to Publication, so callers do not
need to make an explicit theme call. An unthemed child Plot inherits that root.
An explicit Plot theme completely replaces the inherited theme for only that
Plot.

```rust
use lichtung_plot::{Canvas, NumericValues, Plot, Theme};

let canvas = Canvas::grid(1, 2)?
    .theme(Theme::nature())
    .panel(0, 0, Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("inherited", [1.0, 2.0]),
    ))?
    .panel(0, 1, Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("replacement", [2.0, 1.0]),
    ).theme(Theme::science()))?;
# let _ = canvas;
# Ok::<(), lichtung_plot::Error>(())
```

The Canvas call remains optional: omitting it selects Publication for the root.
Theme state is owner-local and never global or sticky across render calls.

Presets are complete immutable `Theme` values, with eight fully typed components.
Use `theme.customize()`, validated component `with_*` edits, and `build()` to
construct a replacement. Each text role has its own typed `FontStack`; named
families are literal identities, not comma-separated CSS input. Plot-local
`categorical_color_scale` and `continuous_color_scale` attach complete typed
mappings independently of the theme. See [complete theme and scale
construction](complete-theme-foundations.md) for the current boundaries.

Use `Plot::title_alignment(TitleAlignment::Start)` and
`Plot::subtitle_alignment(TitleAlignment::End)` for independent heading
instructions. Canvas's `panel_title_alignment` and `panel_subtitle_alignment`
provide defaults for embedded Plots; explicit Plot instructions take precedence.
Start and End are physical left/right after content insets. A subtitle with no
instruction uses its optional theme default, then the effective title alignment;
captions always use their independent complete theme field.

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

## Numeric Tick Generation

`TickGeneration::additive(2.0)?` requests first-tried linear spacing;
`TickGeneration::log10_decades(2)?` requests integral Forest Log10 exponent spacing.
Attach through `x_tick_generation` or `y_tick_generation` on numeric-axis families.
Dense labels can coarsen locally. `target(5)?` requests approximately five positions;
`maximum(4)?` bounds the complete generated list, including endpoints.
`automatic()` resets the complete choice. Exact `AxisTick` lists win in either
setter order and are not truncated by generated counts. An actually categorical
Trajectory x axis rejects non-Automatic instructions before fonts. See
[numeric tick generation](public-tick-generation.md).

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
supported layouts. Wrap construction rejects a zero column count. Facets add no
theme scope: every facet panel uses its owning Plot's selected theme, and an
unthemed standalone faceted Plot uses Publication. Generic faceted Plots remain
non-nestable in Canvas.
