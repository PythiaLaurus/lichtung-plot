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
    .theme(Theme::Nature)
    .panel(0, 0, Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("inherited", [1.0, 2.0]),
    ))?
    .panel(0, 1, Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("replacement", [2.0, 1.0]),
    ).theme(Theme::Science))?;
# let _ = canvas;
# Ok::<(), lichtung_plot::Error>(())
```

## Fonts

Outlined SVG and PNG generation resolve installed system fonts from this
ordered stack and never download or embed font files:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

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
