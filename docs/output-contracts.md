# Output Contracts

## SVG

SVG is the canonical scene and is always available. Output contains a stable
system-font fallback declaration, vector geometry, deterministic clip and
gradient identifiers, and no embedded font data. In-memory and file rendering
preserve the same scene bytes.

## Outlined SVG

The `text-as-path` feature adds
`RenderedSvgScene::svg_with_text_as_paths`. It converts the canonical scene's
text to vector glyph paths using installed system fonts, verifies that no
`<text>` elements remain, and returns a self-contained derived SVG. This output
is not selectable or searchable and can differ across font environments.

## PNG

PNG requires the `png` feature, which includes text outlining:

```toml
lichtung-plot = { path = "../lichtung-plot", features = ["png"] }
```

PNG is rasterized from the canonical SVG scene. `PngSizing` supports source
dimensions, DPI, scale, width, and height. DPI output treats SVG dimensions as
CSS pixels at 96 DPI and writes matching physical-density metadata.

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
Combined output is not a portable multi-file transaction, but preflight and
conversion failures occur before destination replacement and staging residue is
removed.

## Unsupported Output

The crate does not produce PDF. PDF conversion and report generation are
application responsibilities outside the renderer.
