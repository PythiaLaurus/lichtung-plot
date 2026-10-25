# lichtung-plot

A standalone, strongly typed scientific renderer for owned immutable display
values. The crate provides 25 scientific plot families, themes, facets and Canvas
composition, with outlined SVG, preserve-text SVG and PNG output.

It has no dependency on another `lichtung-*` crate. Callers supply native values
and precomputed inferential annotations; the renderer does not run application
analysis, access the network or provision assets.

## First Plot

```rust,no_run
use lichtung_plot::{NumericValues, Plot};

Plot::scatter(
    NumericValues::new("x", [1.0, 2.0, 3.0]),
    NumericValues::new("y", [1.5, 2.5, 2.0]),
)
.svg("scatter")?; // Publishes scatter.svg.
# Ok::<(), Box<dyn std::error::Error>>(())
```

The public API is documented on [docs.rs](https://docs.rs/lichtung-plot).
[English](https://github.com/PythiaLaurus/lichtung-plot/tree/main/docs/en/src) and
[Chinese](https://github.com/PythiaLaurus/lichtung-plot/tree/main/docs/zh-CN/src) user guides and
[architectural specifications](https://github.com/PythiaLaurus/lichtung-plot/tree/main/specs)
are maintained at the workspace root, not copied into this package.

## Output And Fonts

`Plot` and `Canvas` derive dimensions from their content by default. Explicit
`width`, `max_width` and `size` requests control the canonical figure dimensions.
Terminal methods return an opaque frozen `Output`; further SVG or PNG publication
uses that same scene without repeating scientific preparation, font discovery or
layout. SVG methods append `.svg` and PNG methods append `.png` unless the final
filename already has the matching suffix.

The renderer resolves installed system fonts and never downloads or embeds font
files. Its default ordered font stack is:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

Outlined SVG and PNG do not require those fonts at viewing time. Preserve-text SVG
keeps selectable text and depends on the viewer's installed fonts. Generation
requires installed fonts covering the requested text in either case.

## Examples And Development

The package includes runnable examples and self-contained default test fixtures.
For example, from the workspace root or this package directory:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start -- target/examples/first-plot
```

Every example accepts exactly one caller-selected output directory, creating it
if needed. That example writes `quick_start.svg` in the selected directory. The
workspace pins Rust 1.96.0 and maintains the shared lockfile. Root `Justfile`
recipes validate the whole workspace; use `-p lichtung-plot` for focused Cargo
commands. Complete Unicode conformance checks are opt-in and require explicit
external sources; they are not part of default test execution.

Licensed under MIT OR Apache-2.0; both license texts are included in this package.
