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
    .svg("publication")? // Publishes publication.svg.
    .svg_preserve_text("editable")?; // Publishes editable.svg.
    Ok(())
}
```

## PNG

PNG is rasterized from the outlined form of the canonical scene. `.png(path)`
publishes at 300 DPI with balanced compression, scaling canonical CSS pixels from 96 DPI and
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
    .png_preview("preview")?
    .png_with_options(
        "custom",
        PngOptions {
            sizing: PngSizing::Width(1600),
            compression: PngCompression::High,
        },
    )?
    .png("publication")?;
    Ok(())
}
```

Callers chain any needed formats as independent same-scene publications, for
example:

```rust,ignore
.svg("plot")?
.svg_preserve_text("plot-editable")?
.png_preview("plot-preview")
```

## Destination Completion

Terminal arguments are base file destinations. SVG terminals append `.svg` and
PNG terminals append `.png` to the final native filename unless it already ends
in the complete expected suffix under ASCII case-insensitive comparison. A
matching filename is preserved exactly, including suffix case. Other filename
text is not interpreted: `plot.v1` becomes `plot.v1.svg`, `plot.png` passed to
`.svg` becomes `plot.png.svg`, and `.plot` becomes `.plot.svg`.

Completion preserves native filename units and does not require UTF-8. It occurs
after format conversion succeeds and before atomic publication. A destination
that currently identifies a directory, including through a symlink, or has no
filename is rejected. The parent directory must already exist. An existing
unsuffixed base file is left untouched when publication targets its completed
sibling.

Every used text role selects installed fonts through its typed stack. The default
order is Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, then the
host's sans-serif generic. Missing visible glyph coverage or an empty matching
candidate list is a render error; the renderer does not choose an arbitrary
unrelated family. Outlined SVG and PNG retain the generation operation's font
bytes and glyphs without rediscovery.

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
