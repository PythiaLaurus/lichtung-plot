# Rendering Contract

**Status**: Canonical

## Inputs

`Plot` consumes owned immutable semantic values. Constructors return
`Plot<kind::Family>` values, and sealed capabilities reject configuration that is
categorically unsupported by that family at compile time. Validation before
opening a backend canvas remains responsible for data-dependent constraints such
as aligned lengths, finite numeric values, categorical identities, source-row
provenance, bounds, and representable geometry. Conversion from external data
containers belongs to callers.

Exact family markers are erased explicitly with `Plot::erase` when a caller needs
heterogeneous storage. `Plot<kind::Any>` does not regain family-specific
configuration methods. `Canvas::panel` performs this erasure after accepting a
typed plot.

Categorical ordering is plot-owned rather than encoded by physical input row
order. First-observed order is the deterministic default. An explicit order
must name every rendered category exactly once. Value-derived ordering is
applied only after a supported family prepares one scalar rendered value per
category; ambiguous multi-value categorical dimensions reject it.

Ordinary bars consume one closed `BarInput`: raw samples, one precomputed
estimate per cell, or one precomputed estimate with explicit drawable `Bounds`.
Raw samples render per-cell arithmetic means and default to a deterministic
two-sided 95% percentile-bootstrap confidence interval with 1,000 resamples and
seed zero. Optional sampling units select multilevel resampling within each
category, group, and facet cell. A singleton observation or sampling unit
renders its mean without an error bar. Stacked bars remain separate sum inputs.

`Bounds` is the sole caller-provided lower/upper representation and carries no
inferential classification, level, scale, or provenance. Point-interval and
interaction constructors require bounds atomically; interaction also requires
its graphical group. Forest construction requires bounds and its scientific
reference. Coefficients consume `CoefficientInput::estimates` or
`CoefficientInput::estimates_with_intervals`, and ordinary line series add an
optional drawable band with `Plot::band`. Rendering validates aligned lengths,
finite endpoints, and `lower <= estimate <= upper`.

Histograms consume exactly one `HistogramInput`: `samples` computes bins and
optional composite layers, while `geometry` renders one complete
caller-computed density-axis geometry input. Rainclouds likewise consume exactly
one `RaincloudInput`: aligned categorical samples or complete caller-computed
geometry. Precomputed geometry is self-contained and rejects sample-only
channels such as facets and source-row identities during preflight.

`Plot::line` is an ordinary sorted numeric line series and is the only line
family that accepts bands or supplemental point layers. `Plot::trajectory`
requires an explicit graphical group, sorts points within each group, and always
draws an observation marker. `Plot::line_path` requires numeric x values and an
explicit group, preserves source-row order and repeated x values, and alone
accepts `LinePathStyle`. Marker visibility is configurable for ordinary lines
and line paths, but not trajectories.

Dumbbell construction atomically consumes both numeric endpoints and their
explicit labels and colors. Waterfall construction consumes a finite base;
rendering derives its finite terminal as the base plus all supplied deltas.

Plot legend state is one `LegendPolicy`: automatic plot-family behavior,
hidden, or explicitly visible at one `LegendPosition`. Visibility and placement
are not independently configurable.

Generic faceting consumes one complete `FacetSpec`: row, column, grid, or a wrap
with a positive column count, together with its numeric-axis sharing policy. A
grid requires distinct row and column variable names. Marginal and matrix plots
do not implement the generic facet capability. Faceted significance annotations
identify one `FacetCell::single(level)` or `FacetCell::grid(row, column)`; facet
variable names come from the `FacetSpec` and are not repeated by annotations.
Precomputed histogram and raincloud geometry reject generic faceting before a
backend canvas is opened.

For bars, horizontal bars, stacked bars, stacked horizontal bars, boxplots,
sample-derived violins, and sample-derived rainclouds, `Plot::group` and
`Plot::hue` are last-write-wins aliases for one positioned, colored categorical
series. That series owns geometry, partitioning, color, legend identity, and
significance targeting where the plot family admits the requested placement.
The two channels remain independent for plot families whose geometry requires
separate grouping and color dimensions. Precomputed raincloud geometry is
self-contained and accepts neither setter.

The crate computes display statistics needed to draw a requested plot, such as
histogram bins, KDE curves, quantiles, confidence geometry for explicitly
requested descriptive summaries, and ordinary least-squares display fits. It
does not select scientific tests or infer caller intent.

## Scene Lifecycle

`Plot::svg`, `Plot::png`, `Canvas::svg`, and `Canvas::png` perform one complete
preflight and render one private immutable scene. The terminal call consumes the
plot or canvas, publishes its requested file immediately, and returns an opaque
`Output`. Consuming `Output::svg` and `Output::png` calls reuse that exact scene.

`Canvas` renders all panels through one backend begin/end lifecycle. Marks are
clipped to their panel; titles, axes, legends, captions, and other decorations
remain outside mark clipping where required.

## Output Formats

SVG is always compiled. Callers select searchable text or `SvgText::Paths`
before the first terminal output. The `text-as-path` feature enables conversion
of every text element in the canonical scene to vector glyph paths through `resvg`. The
derived SVG is self-contained but its text is no longer selectable, searchable,
or available to assistive text extraction. The `png` feature includes this
capability and enables rasterization of the canonical SVG through `resvg` and
`tiny-skia`. PNG sizing validates positive, finite DPI or scale, positive
dimensions, and bounded pixel work before allocation.

`png` is enabled by default; callers may disable default features for an
SVG-only build. Lichtung's canonical SVG backend emits no raster image elements,
so the `resvg` JPEG, GIF, and WebP decoders are excluded. The dedicated PNG
encoder remains responsible for compression and physical-resolution metadata.

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

## Significance Annotations

`Significance<Attached>` and `Significance<Pairwise>` are plot-owned display
data. `Significance::at` constructs the attached placement for one category or
positioned series, while `Significance::between` constructs a pairwise bracket
between exactly two such targets. `Plot::significance` is available only when
the exact plot-family marker supports the annotation's placement.
Raw strings identify complete categories; `SignificanceTarget::category_series`
identifies one exact positioned series.

Attached marks are supported by ordinary bars, ordinary horizontal bars, and
coefficient plots. Pairwise brackets are supported by bar variants, boxplots,
violins, and sample-derived rainclouds. Precomputed raincloud geometry rejects
significance at runtime because its source mode is not encoded in the plot
family marker. Stacked bars accept category-level bracket targets; their
partitioned series do not provide distinct bracket anchors. Coefficients accept
attached term targets only. Their significance marks are independent of whether
the coefficient input includes drawable bounds.

`SignificanceMark` accepts an explicit `ns` or one-through-four-star mark, or
one caller-selected display p-value. A p-value at or above `0.05` renders `ns`;
smaller values map through the conventional one-, two-, three-, and four-star
thresholds. Every supplied annotation renders. Statistical adjustment and
familywise provenance remain caller responsibilities.

Facet-owned annotations name every configured facet dimension exactly once
through `Significance::facet`; annotations are never silently replicated across
panels. The renderer validates all annotations against rendered categories,
series, and facet panels. It never computes, adjusts, or substitutes p-values.

## Errors

Public operations return `lichtung_plot::Result<T>` with these domains:

- `Error::Io` for filesystem failures.
- `Error::Data` for invalid input or data-bound options.
- `Error::Render` for layout, backend, font, rasterization, or publication
  failures.

No fallback renderer or alternate output path is attempted after an error.

## Artifact Publication

Each file output is staged at a unique sibling path and renamed only after
rendering or conversion succeeds. Independent consuming output calls publish
independently. Combined SVG/PNG output validates distinct destination identity
and publishes both files transactionally. Failures preserve existing output
where the underlying filesystem supports ordinary same-directory rename
semantics.

## Regression Evidence

Renderer behavior is covered by unit and integration tests, deterministic SVG
snapshots, exact byte/hash assertions where appropriate, PNG tests under the
optional feature, and focused rendering invariants. A snapshot change is
evidence to inspect, not an automatic approval of visual quality.
