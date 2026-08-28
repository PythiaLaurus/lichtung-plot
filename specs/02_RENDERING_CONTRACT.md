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

Required constructor channels have one owner. Trajectory, line-path,
interaction, and stacked-series groups and heatmap and beeswarm continuous color
values are stored in their closed inputs and have no replacement setter.

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
family that accepts bands or supplemental point layers. Each `PointLayer` owns
an independent nonempty aligned numeric x/y dataset; it may represent raw
observations or any other caller-selected supplemental coordinates and need not
align with the primary line input. Multiple layers append in call order and
render over primary ribbons, lines, and markers. Generic faceting rejects them.
The renderer assigns a deterministic theme-coordinated color and defaults to
radius 4 and opacity 0.8; callers may selectively override those values. The
primary line occupies one palette slot without hue, or one slot per level in its
resolved hue domain with hue. A Canvas shared legend therefore reserves every
Canvas-coordinated hue level, including a level absent from that local plot. An
automatic point-layer color uses the palette slot after those primary slots plus
that layer's zero-based append index. Every layer consumes its append position
even when it has an explicit color. Palette cycling and repeated explicit colors
are allowed. Optional point labels annotate individual points, while one optional
legend label identifies the complete layer.

`Plot::trajectory` requires an explicit graphical group, sorts points within
each group, and always draws an observation marker. `Plot::line_path` requires
numeric x values and an explicit group, preserves source-row order and repeated
x values, and alone accepts `LinePathStyle`. Marker visibility is configurable
for ordinary lines and line paths, but not trajectories.

Dumbbell construction atomically consumes both numeric endpoints and their
explicit labels and colors. Waterfall construction consumes a finite base;
rendering derives every span and the terminal through one ordered cumulative
pass. Every intermediate endpoint must be finite. The final cumulative endpoint
is the sole terminal used by ranges, guides, and labels; a separately associated
sum of deltas is never substituted for rendered geometry.

`Plot::source_rows` is the sole source-row identity authority for all
plot-aligned values, faceting, composition, and raw-observation overlays.
Individual value objects and overlay options do not own provenance.

Generic faceting consumes one complete `FacetSpec`: row, column, grid, or a wrap
with a positive column count, together with its numeric-axis sharing policy. A
grid requires distinct row and column variable names. Marginal and matrix plots
do not implement the generic facet capability. Faceted significance annotations
identify one `FacetCell::single(level)` or `FacetCell::grid(row, column)`; facet
variable names come from the `FacetSpec` and are not repeated by annotations.
Precomputed histogram and raincloud geometry reject generic faceting before a
backend canvas is opened.

Fixed numeric facet axes share one resolved domain. `FreeX`, `FreeY`, and `Free`
axes resolve the selected dimensions independently in every panel. A free
interval-estimate axis containing only `NotEstimable` rows has no implicit
numeric domain and requires explicit limits; it never borrows an estimated
sibling's range.

For ordinary bars, horizontal bars, boxplots, sample-derived violins, and
sample-derived rainclouds, one optional `Plot::series` channel owns positioning,
color, legend identity, and significance targeting where supported. Stacked
bars require that series in their constructor and expose no replacement setter.
Precomputed raincloud geometry shares the family marker, so `.series` type-checks
without source typestate but rejects during preflight as sample-only configuration.

For density, ordinary line, interaction, marginal, matrix, and
`LinePathStyle::Hue`, graphical group and hue remain independent semantic
dimensions. Every observed `(group, hue)` pair owns one collective mark. Hue
without group partitions marks by hue; no unobserved Cartesian combinations are
created. Different groups may reuse one hue, and one group may own multiple
hue-specific marks. Pair order is stable first-observed group order followed by
first-observed hue order within that group. Density, marginal, and matrix reject
an observed pair that cannot independently produce its required geometry.

Interaction cells are unique by `(condition, group, hue)`. `InteractionStyle`
is keyed only by group and supplies every hue-specific mark for that group with
the same point shape and connector pattern. `CategoricalColors` is keyed only by
hue. A group-only styled interaction still emits the complete configured group
glyph. Plain grouping without hue or group-specific visible style partitions
geometry but creates no legend identity.

Trajectory and neutral line-path grouping remain different: group owns the
connector while hue may vary point by point inside that group. Their legends
identify visible point hues only. If neutral line-path markers are hidden, the
inert hue legend is suppressed.

If group and hue have the same semantic name, their aligned categorical values
must be identical and they collapse to one semantic dimension. Same-named but
different values reject. Distinct semantic fields remain distinct even when one
observed pair contains equal display values.

## Numeric Ticks

Automatic numeric tick positions and density are deterministic renderer-owned
layout decisions based on resolved range, available panel space, and measured
labels. The current API exposes no caller-selected automatic tick count.

`Plot::x_ticks` and `Plot::y_ticks` accept complete exact `AxisTick` lists. An
exact list replaces all generated ticks on that axis; generated and exact ticks
are never merged. Positions are finite, strictly ascending data-space values
inside the resolved range. Preflight projects the requested positions, measures
their labels, and rejects collisions rather than omitting, moving, or rotating
caller-owned ticks.

## Numeric Value Labels

`NumericLabelFormat` is fixed-decimal formatting with validated precision and
an optional literal suffix. It applies only to opt-in heatmap cell values and
point-only coefficient estimate labels. A suffix does not scale or otherwise
transform the numeric value.

It does not configure automatic or exact axis ticks, heatmap or beeswarm
colorbars, waterfall totals, point-layer labels, or legends. General scientific,
engineering, compact, percentage, and cross-surface formatting remain deferred;
the current API reserves no notation enum, callback, or format-string surface.

## Legends

Plot legend state is one `LegendPolicy`: automatic plot-family behavior,
hidden, or explicitly visible at one `LegendPosition`. Visibility and placement
are not independently configurable. Same-corner inside positions with different
edge names intentionally select different data-clearance edges.

Legend entries preserve the visible family mark. Point presence and outline,
line presence and pattern, and fill presence and opacity match the geometry that
the entry explains. Density entries contain both curve line and translucent fill
representations. Facet and Canvas shared legends stable-merge exact entries by
typed raw semantic identity and reject conflicting labels or glyphs; they never
reconstruct generic palette points in place of renderer-owned entries.

When independent group and hue fields are both present, the legend title is the
independently resolved `group × hue`, and each entry label is the independently
resolved `group, hue`. When the two channels are the same semantic field with
identical values, title and entry labels collapse to the one resolved identity.
Distinct fields always display both values, including an entry whose resolved
values happen to be equal. The multiplication sign and comma are display
separators only; raw pair identity remains typed and is never encoded by joining
strings.

The crate computes display statistics needed to draw a requested plot, such as
histogram bins, KDE curves, quantiles, confidence geometry for explicitly
requested descriptive summaries, and ordinary least-squares display fits. It
does not select scientific tests or infer caller intent.

## Scene Lifecycle

`Plot::svg`, `Plot::svg_preserve_text`, `Plot::png`, `Plot::png_preview`, and
`Plot::png_with_options`, together with their Canvas equivalents, perform one
complete preflight and render one private immutable scene. The terminal call
consumes the plot or canvas, publishes its requested file immediately, and
returns an opaque `Output`. The equivalent consuming `Output` publication calls
reuse that exact scene.

On `Plot`, `Canvas`, and `Output`, both `svg` and `svg_preserve_text` are
consuming, call-local, non-sticky terminal methods. Plain `svg` is the normal
publication operation and converts all text to vector paths.
`svg_preserve_text` publishes the canonical SVG with `<text>` elements intact. A
chained call such as `plot.svg(a)?.svg_preserve_text(b)?` therefore publishes
both forms from one scene.

PNG options are call-local and non-sticky. Plain `png` publishes at 300 DPI with
balanced compression. `png_preview` publishes at the canonical SVG source pixel
dimensions with balanced compression and no physical-resolution metadata.
`png_with_options` uses the exact caller-provided sizing and compression. A
chained call such as `plot.png_preview(a)?.png(b)?` therefore publishes two
independent rasterizations from one scene. There is no combined multi-format
terminal operation; callers chain the required SVG and PNG operations.

Every Canvas plot runs the same canonical plot-level validation as its
standalone form before Canvas-only composition validation. Moving a plot into a
Canvas cannot make aligned values, family-owned options, facets, intervals,
ticks, guides, or supplemental layers more permissive.

`Canvas` renders all panels through one backend begin/end lifecycle. Marks are
clipped to their panel; titles, axes, legends, captions, and other decorations
remain outside mark clipping where required.

## Output Formats

Publication SVG, preserve-text SVG, and PNG are unconditional capabilities.
Plain `svg` converts every text element in the canonical scene to vector glyph
paths through `resvg` and verifies that no `<text>` elements remain. The derived
SVG requires no recipient-side fonts, but its text is no longer selectable,
searchable, editable, or available to assistive text extraction.
`svg_preserve_text` publishes the canonical SVG with accessible, selectable,
searchable, and editable `<text>` elements intact. PNG rasterizes the canonical
SVG through `resvg` and `tiny-skia`. PNG sizing validates positive, finite DPI
or scale, positive dimensions, and bounded pixel work before allocation.

Lichtung's canonical SVG backend emits no raster image elements, so the `resvg`
JPEG, GIF, and WebP decoders are excluded. The dedicated PNG encoder remains
responsible for compression and physical-resolution metadata.

There is no plot-level PDF output. Applications that need PDF must own that
conversion or report workflow outside this crate.

## Font Policy

The canonical SVG emits this ordered CSS system-font stack and embeds no font
bytes:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

Publication SVG outlining and PNG generation scan platform font directories and
select the first installed preferred family. Generic `serif`, `sans-serif`, and
`monospace` entries resolve through the platform font database. If the primary
face lacks a required glyph, resolution continues through later stack entries
for that glyph and fails if none can render it. Rendering performs no download,
provisioning, or cache lookup. Publication SVG also fails if any text remains
after conversion. Its outlined text requires no font on the recipient's system,
but exact glyph paths depend on the generation font environment. Preserve-text
SVG remains accessible and editable, while its displayed text depends on fonts
available to the viewer. SVG structure and geometry remain deterministic for
identical inputs and options; exact glyph paths, glyph appearance, and PNG bytes
require the same installed-font environment.

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
rendering or conversion succeeds. Consuming output calls publish independently;
if a later chained publication fails, an earlier successful publication remains
in place. Supported POSIX and Windows hosts publish with one atomic
same-directory replacement: success installs the complete staged bytes and every
failure preserves the prior destination without a backup or rollback window.
The transaction retains the staging handle for every write. On POSIX, the final
rename and pre-commit unlink remain pathname syscalls; their exact-source claim
assumes no external namespace mutation between the final identity check and that
single syscall because portable POSIX exposes no rename or unlink operation by
open file handle. Windows publication and cleanup operate through the exclusive
staging handle. Callers remain responsible for excluding independent concurrent
mutation of the output namespace during a publication call.

## Regression Evidence

Renderer behavior is covered by unit and integration tests, deterministic SVG
snapshots, exact byte/hash assertions where appropriate, PNG tests, and focused
rendering invariants. A snapshot change is evidence to inspect, not an automatic
approval of visual quality.
