# lichtung-plot

`lichtung-plot` is a strongly typed scientific renderer for owned immutable
display values. It publishes outlined SVG and PNG artifacts and can also publish
canonical SVG with preserved text from the same opaque rendered scene.

This first standalone-boundary release intentionally has no dependency on the
Lichtung analysis, pipeline, asset, or reporting crates. Callers supply
precomputed inferential annotations as ordinary plotting data.

The renderer is maintained in its own repository. Its architecture and output
contracts live in [`specs/`](specs/); user guides live in [`docs/`](docs/).

```rust,no_run
use lichtung_plot::{NumericValues, Plot};

Plot::scatter(
    NumericValues::new("x", [1.0, 2.0, 3.0]),
    NumericValues::new("y", [1.5, 2.5, 2.0]),
)
.svg("scatter")?; // Publishes scatter.svg.
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Themes And Composition

Omitting a theme selects Publication for a standalone Plot and for the Canvas
root. `Canvas::theme` replaces that root; child Plots without their own theme
inherit it. `Plot::theme` completely replaces the inherited theme for only that
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

Built-in constructors return complete immutable values. Customize any component
through its validated `with_*` edits, then build a new theme:

```rust
use lichtung_plot::{Color, Theme};

let base = Theme::publication();
let surfaces = base.surfaces().clone()
    .with_canvas_root(Color::hex("#FAFAFA")?)?;
let custom = base.customize().with_surfaces(surfaces).build()?;
# let _ = custom;
# Ok::<(), lichtung_plot::Error>(())
```

Categorical mappings use `CategoricalColorScale` with an ordered
`QualitativePalette` or exact raw-key `CategoricalColorMap`. Heatmap and Beeswarm
both accept one `ContinuousColorScale`, containing kind, optional gradient and
optional exact `ColorDomain`. Data-scale colors use opaque `OpaqueColor`; mark
roles supply alpha independently. See [complete theme and scale
construction](docs/complete-theme-foundations.md).

## Exact Element Paint

Exact styles accept a complete `Color`, including its byte alpha. An independent
point-layer alpha replaces that color's alpha in either edit order; omitted
color, alpha and radius instructions use the owning theme's defaults.

```rust
use lichtung_plot::{Color, GuideSpec, GuideStyle, LinePattern, NumericValues, Plot, PointLayer};

let points = PointLayer::new(
    NumericValues::new("selected x", [1.0]),
    NumericValues::new("selected y", [2.0]),
)
.with_color(Color::hex("#14508C80")?)
.with_alpha(192)
.with_radius(5)?;
let guide = GuideStyle::new(Color::rgba(180, 60, 40, 128), 1.5, LinePattern::Dashed)?;
let plot = Plot::line(
    NumericValues::new("x", [0.0, 1.0, 2.0]),
    NumericValues::new("y", [1.0, 2.0, 3.0]),
)
.point_layer(points)
.guide(GuideSpec::horizontal(2.0, guide));
# let _ = plot;
# Ok::<(), lichtung_plot::Error>(())
```

## Fonts

Outlined SVG and PNG generation resolve installed system fonts from this
default ordered stack and never download or embed font files:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

Each of the eighteen complete text roles owns a typed `FontStack`. Inspect a
built-in profile through
`Theme::publication().typography().root().title().stack()`, or construct ordered
`FontFamily::named` and `FontFamily::generic` entries. A named value is one literal
family, including any punctuation; it is not a CSS stack expression.
`ThemeBuilder::with_font_stack` edits all roles once during construction. A later
single-role replacement takes precedence normally. Font coverage is checked only
when a render operation consumes the selected roles.

The normal `.svg(path)` publication method converts all text to vector glyph
paths. Recipients therefore do not need the generation fonts, but the exact
paths depend on the fonts installed in the generation environment and the text
is no longer selectable or searchable. `.svg_preserve_text(path)` instead
publishes the canonical SVG with accessible, selectable, searchable, and
editable `<text>` elements; its displayed text depends on fonts available to the
viewer.

## Output Formats

Publication SVG with outlined text, preserve-text SVG, and PNG are unconditional
crate capabilities. Terminal paths are base destinations: SVG methods append
`.svg` and PNG methods append `.png` unless the final filename already has the
complete expected suffix under ASCII case-insensitive comparison. Existing
filename text is never replaced, so `report.v1` becomes `report.v1.svg` and
`report.png` passed to `.svg` becomes `report.png.svg`. PDF output is deliberately
outside this crate.

## Development

Rendering retains operation/source ownership, operation-local reuse, checked
dimensions and allocation, and bounded layout coordination. Current correctness
boundaries and the historical output/time/RSS comparisons are recorded in
[the resource-accounting cleanup evidence](docs/resource-accounting-cleanup.md)
and [the complete caption measurements](docs/caption-cleanup-performance.md).

The repository pins its Rust toolchain and tracks `Cargo.lock`. Use focused
tests and `just lint` for scoped changes. The complete read-only closure gate is:

```bash
just verify
```

`just test` runs library, integration and example targets; `just test-doc`
executes public examples and compile-fail API boundaries. `just verify` includes
both, strict Clippy, formatting, documentation and package verification. See
[`specs/90_DEVELOPMENT.md`](specs/90_DEVELOPMENT.md) for test ownership and stable
semantic expectations.
