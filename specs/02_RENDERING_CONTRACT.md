# Rendering Contract

**Status**: Canonical

## Inputs

`Chart` consumes a borrowed Polars `DataFrame`. Plot families validate required
columns, finite numeric values, categorical identities, bounds, channel
compatibility, and representable geometry before opening a backend canvas.

The crate computes display statistics needed to draw a requested plot, such as
histogram bins, KDE curves, quantiles, confidence geometry for explicitly
requested descriptive summaries, and ordinary least-squares display fits. It
does not select scientific tests or infer caller intent.

## Scene Lifecycle

`Chart::render_scene` and `Figure::render_scene` perform one complete preflight
and render one immutable `RenderedSvgScene`. SVG bytes are the canonical scene.
File methods and in-memory methods must preserve those bytes exactly.

`Figure` renders all panels through one backend begin/end lifecycle. Marks are
clipped to their panel; titles, axes, legends, captions, and other decorations
remain outside mark clipping where required.

## Output Formats

SVG is always compiled. The `text-as-path` feature enables conversion of every
text element in the canonical scene to vector glyph paths through `resvg`. The
derived SVG is self-contained but its text is no longer selectable, searchable,
or available to assistive text extraction. The `png` feature includes this
capability and enables rasterization of the canonical SVG through `resvg` and
`tiny-skia`. PNG sizing validates positive, finite DPI or scale, positive
dimensions, and bounded pixel work before allocation.

There is no plot-level PDF output. Applications that need PDF must own that
conversion or report workflow outside this crate.

## Font Policy

SVG emits this ordered CSS system-font stack and embeds no font bytes:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

PNG scans platform font directories and selects the first installed preferred
family. Rendering performs no download, provisioning, or cache lookup. The
optional text-as-path conversion resolves this same stack from installed fonts
and fails if no font is available or any text remains. SVG structure and
geometry remain deterministic for identical inputs and options; exact glyph
paths, glyph appearance, and PNG bytes require the same installed-font
environment.

## Comparison Annotations

`ComparisonAnnotation` is plot-owned display data. It carries two explicit
category endpoints, optional group and hue qualifiers, optional facet identity,
a raw p-value, and an optional adjusted or intrinsically familywise p-value.
The displayed p-value determines the conventional significance mark.

The renderer validates annotations against rendered categories and graphical
anchors. It never computes, adjusts, or substitutes p-values.

## Errors

Public operations return `lichtung_plot::Result<T>` with these domains:

- `Error::Io` for filesystem failures.
- `Error::Data` for invalid input or data-bound options.
- `Error::Render` for layout, backend, font, rasterization, or publication
  failures.

No fallback renderer or alternate output path is attempted after an error.

## Artifact Publication

File output is staged at unique sibling paths and renamed only after rendering
or conversion succeeds. Combined SVG/PNG output validates distinct destination
identity before touching either destination. Failures preserve existing output
where the underlying filesystem supports ordinary same-directory rename
semantics.

## Regression Evidence

Renderer behavior is covered by unit and integration tests, deterministic SVG
snapshots, exact byte/hash assertions where appropriate, PNG tests under the
optional feature, and focused rendering invariants. A snapshot change is
evidence to inspect, not an automatic approval of visual quality.
