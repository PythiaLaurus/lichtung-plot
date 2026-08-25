# Output Contracts

## Publication SVG

`.svg(path)` is the normal publication method. It converts every text element in
the private canonical scene to vector glyph paths using installed system fonts
and verifies that no `<text>` elements remain. The result requires no fonts on
the recipient's system, but it is not selectable, searchable, or available to
assistive text extraction, and its exact paths depend on the generation font
environment.

## Preserve-Text SVG

`.svg_preserve_text(path)` publishes the canonical SVG with its `<text>`
elements and ordered system-font fallback declaration intact. It embeds no font
data and remains accessible, selectable, searchable, and editable; its displayed
text depends on fonts available to the viewer.

Both methods are call-local consuming terminal operations on `Plot`, `Canvas`,
and `Output`. No SVG text mode is retained, so one rendered scene can publish
both forms without rerunning layout:

```rust
use lichtung_plot::{NumericValues, Plot};

fn publish_svg() -> Result<(), lichtung_plot::Error> {
    Plot::scatter(
        NumericValues::new("x", [1.0]),
        NumericValues::new("y", [2.0]),
    )
    .svg("publication.svg")?
    .svg_preserve_text("editable.svg")?;
    Ok(())
}
```

## PNG

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

Callers chain any needed formats as independent same-scene publications, for
example:

```rust,ignore
.svg("plot.svg")?
.svg_preserve_text("plot-editable.svg")?
.png_preview("plot.png")
```

Outlined SVG and PNG generation require at least one installed system font. The
preferred order is Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans,
then Arial. If none is present, any discovered system family is used; an empty
font database is a render error.

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
