# Output Contracts

## SVG

SVG is always available and is rendered from one private canonical scene. Output contains a stable
system-font fallback declaration, vector geometry, deterministic clip and
gradient identifiers, and no embedded font data. A consuming `Output::svg` call
can publish the same scene again without rerunning layout.

## Outlined SVG

The `text-as-path` feature enables `.svg_text(SvgText::Paths)` before terminal
output. It converts the canonical scene's text to vector glyph paths using
installed system fonts and verifies that no `<text>` elements remain. This
output is not selectable or searchable and can differ across font environments.

## PNG

PNG requires the `png` feature, which includes text outlining:

```toml
lichtung-plot = { path = "../lichtung-plot", features = ["png"] }
```

PNG is rasterized from the canonical SVG scene. `.png(path)` publishes at 300
DPI with balanced compression, scaling canonical CSS pixels from 96 DPI and
writing matching physical-density metadata. `.png_preview(path)` preserves the
canonical SVG pixel dimensions, uses balanced compression, and writes no
physical-density metadata. `.png_with_options(path, options)` applies the exact
caller's `PngSizing` and `PngCompression` values.

PNG options are call-local. `Plot`, `Canvas`, and the returned `Output` retain no
PNG configuration, so one scene can independently publish preview, publication,
and custom PNGs:

```rust
use lichtung_plot::{NumericValues, Plot, PngCompression, PngOptions, PngSizing};

fn publish() -> Result<(), lichtung_plot::Error> {
    Plot::scatter(
        NumericValues::new("x", [1.0]),
        NumericValues::new("y", [2.0]),
    )
    .png_preview("preview.png")?
    .png_with_options(
        "custom.png",
        PngOptions {
            sizing: PngSizing::Width(1600),
            compression: PngCompression::High,
        },
    )?
    .png("publication.png")?;
    Ok(())
}
```

Callers that need SVG and PNG output chain independent same-scene publication,
for example `.svg("plot.svg")?.png_preview("plot.png")`.

PNG text requires at least one installed system font. The preferred order is
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, then Arial. If none
is present, any discovered system family is used; an empty font database is a
render error.

## Determinism

For identical validated data, configuration, dependency versions, and font
environment, rendering is deterministic. Do not assume byte stability across
crate revisions, dependency changes, operating systems, font installations, or
rendering-engine changes.

## Atomicity

SVG and PNG files are completed at sibling staging paths before publication.
Each consuming `Plot`, `Canvas`, or `Output` call publishes one file
independently. Preflight and conversion failures occur before destination
replacement and staging residue is removed.

## Unsupported Output

The crate does not produce PDF. PDF conversion and report generation are
application responsibilities outside the renderer.
