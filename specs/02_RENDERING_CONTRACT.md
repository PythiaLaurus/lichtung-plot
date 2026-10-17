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
The renderer assigns a deterministic theme-coordinated color and selects the
theme's supplemental radius and byte alpha (built-ins use 4 and 204). An explicit
`Color` replaces the complete mark RGBA; a separate byte-alpha
instruction replaces its alpha without multiplication or floating conversion. Radius replaces only
geometry. Point-label text retains its complete theme text role: automatic mark
color/alpha do not recolor it, explicit color replaces its paint, and explicit
alpha replaces only its alpha. The
primary line occupies one palette slot without hue, or one slot per level in its
resolved hue domain with hue. A Canvas shared legend therefore reserves every
Canvas-coordinated hue level, including a level absent from that local plot. An
automatic point-layer color uses the palette slot after those primary slots plus
that layer's zero-based append index. Every layer consumes its append position
even when it has an explicit color. Palette cycling and repeated explicit colors
are allowed. Optional point labels annotate individual points, while one optional
legend label identifies the complete layer.

Heatmap and Beeswarm use the owning theme's common sequential gradient. All four
built-ins select `#F7FBFF -> #08306B`; the former Beeswarm-only default is removed.
Local and Shared continuous guides consume the final prepared domain and the
same chromatic mapping as marks, with independent mark alpha. Constant domains
have one required numeric label; an interior diverging center contributes its
required label without making every gradient stop a numeric label.

`Plot::trajectory` requires an explicit graphical group, sorts points within
each group, and always draws an observation marker. `Plot::line_path` requires
numeric x values and an explicit group, preserves source-row order and repeated
x values, and alone accepts `LinePathStyle`. Marker visibility is configurable
for ordinary lines and line paths, but not trajectories.

Dumbbell construction atomically consumes both numeric endpoints and their
explicit labels and complete `Color` paints. Each endpoint's exact RGBA also
supplies its literal guide key; theme radius/shape and independent outline apply
without multiplying the exact fill alpha by a theme alpha. The neutral connector
is independent. Waterfall construction consumes a finite base;
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
the same point shape and connector pattern. A `CategoricalColorMap` is keyed by
hue when present and otherwise by the actual color-owning raw group. A group-only
styled interaction still emits the complete configured group
glyph. Plain grouping without hue or group-specific visible style partitions
geometry but creates no legend identity.
The style constructor is fallible and requires a nonempty, already trimmed,
control-free raw group selector, retaining interior whitespace and Unicode bytes
for exact group lookup. Prepared resolution rejects unknown, duplicate or
incomplete supplied mappings before fonts. `with_point_shape` and
`with_line_pattern` replace only their own glyph fields.

Trajectory and neutral line-path grouping remain different: group owns the
connector while hue may vary point by point inside that group. Their legends
identify visible point hues only. If neutral line-path markers are hidden, the
inert hue legend is suppressed.

If group and hue have the same semantic name, their aligned categorical values
must be identical and they collapse to one semantic dimension. Same-named but
different values reject. Distinct semantic fields remain distinct even when one
observed pair contains equal display values.

## Exact Element Styles

`Color` is an immutable crate-root RGBA byte value with `rgb`, `rgba` and strict
`hex` constructors, readonly channel getters and replacement `with_alpha`.
Hex accepts exactly ASCII `#RRGGBB` or `#RRGGBBAA`, with no trimming, shorthand or
CSS expression syntax. All alpha values, including 0 and 255, are valid.

`GuideStyle` consumes a complete Color, positive finite width and closed
`LinePattern`; readonly properties and fallible `with_color`, `with_width` and
`with_pattern` edits retain one complete style. Guide geometry is unchanged.
`PointLayer` exposes optional color/alpha/radius instructions through getters and
`with_color`, `with_alpha`, and positive-radius `with_radius` edits. Independent
alpha wins in either edit order. `BarPointLayer` instead requires complete Color
and positive radius in its constructor, retains plain Circle/no-outline geometry
and prepared bar centers, and creates no layer legend.

`SignificanceMarkStyle` consumes positive CSS font size, closed weight and finite
nonnegative CSS gap, normalizing signed zero. Getters inspect the exact value;
fallible `with_font_size`, `with_weight` and `with_gap` replace their own fields.
An explicit default is 16px/Normal/8px; absence reads the complete theme role and
theme-owned gap. Exact style never replaces role stack/color/leading or bracket
geometry. Old tuple-paint, floating-opacity and consuming bare-field style routes
are removed rather than retained as aliases.

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
colorbars, waterfall totals, point-layer labels, or legends. Broader scientific,
engineering, compact, percentage and cross-surface controls are not current API.
The admitted locale-free scientific replacement is planned in
`93_T6_EXECUTION_PLAN.md`; residual scope remains in `99_DEFERRED_FEATURES.md`.
Planning does not expose a notation enum, callback or format-string surface.

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

## Complete Theme And Scale Ownership

`Theme` is a complete immutable value with eight independently constructed
components. Publication, Nature, Science and CHI are ordinary validated
constructors, not selector identities. `ThemeBuilder` starts complete and returns
a complete value; it is never a runtime sparse override. All component fields are
private and exposed through readonly getters and validated `with_*` edits.
`Canvas::theme(Theme)` and `Plot::theme(Theme)` are the only theme-owner
attachments. A Canvas always owns one complete root theme initialized with
`Theme::publication()`, so callers are not required to call `Canvas::theme`.

A Plot records whether the caller made an explicit selection. An unthemed
standalone Plot selects Publication at its lifecycle boundary. An unthemed Plot
inside a Canvas inherits that Canvas's root value. An explicit Plot theme
completely replaces the inherited root for that Plot; values are not merged.
Facets use their owning Plot's one selected theme and introduce no additional
scope.

Theme state is immutable and owner-local. There is no global, thread-local,
environment-selected, backend-selected, or sticky theme state. The render owner
resolves the selected complete value before canonical validation and measurement. The
same private resolved value is reused through preflight, planning, null-backend
validation, and final rendering; renderers do not reconstruct a preset late.

Canvas surfaces, title blocks, shared headers, row titles, text panels and
shared-guide chrome use the Canvas root value. Plot and facet marks, axes,
annotations, panel surfaces and prose use their selected Plot value. All eighteen
text roles retain their own stack, color, CSS size, weight and leading.
Construction does not discover fonts. A bulk `ThemeBuilder::with_font_stack`
replaces the stacks once; later role replacement takes precedence normally.
`FontFamily::named` preserves one trimmed, control-free literal identity, including
punctuation, while closed generics have explicit typed semantics. `FontStack`
preserves order and duplicates. Default-font constants and a raw CSS parser are
not public.

`Plot::categorical_color_scale` replaces one complete palette-or-map selection on
the fifteen `kind::CategoricalScale` families. Fixed palettes must cover the active
prepared raw domain before fonts, even with hidden legends; extendable palettes
use deterministic first-party extension without cycling. Exact maps require every
used raw key before fonts, permit unused entries and never fall back to a palette.
Display aliases are not keys. Qualitative mappings and MarkTheme accents are
independent.

Heatmap and Beeswarm share `Plot::continuous_color_scale`, one typed kind, optional
gradient and automatic/exact `ColorDomain`. Exact domains may be constant; a
diverging center is finite and inclusively contained in an exact domain. Wrong-kind
gradient/center edits reject immediately. Ordered opaque gradients have exact zero
and one endpoints; diverging gradients also have one explicit neutral at 0.5.
Both marks and guides consume the same final-domain mapping, including Shared
installation. All built-ins use `#F7FBFF -> #08306B` sequentially. Color scales use
`OpaqueColor`; final marks install their own role alpha. The old categorical map
setter, Heatmap selector and separate color-limit routes are removed.

Shared Canvas legend gap is exclusively `LegendTheme::shared_gap`, resolved
against its selected legend-label em. There is no Canvas override or policy
compatibility exception for a theme with custom spacing.

The crate computes display statistics needed to draw a requested plot, such as
histogram bins, KDE curves, quantiles, confidence geometry for explicitly
requested descriptive summaries, and ordinary least-squares display fits. It
does not select scientific tests or infer caller intent.

## Size And Layout Boundary

The current implemented Plot stores one exact output width and height, defaults
to `640x480`, and exposes those values through `width()` and `height()`. The
current Canvas stores one exact composed width and height and defaults to
`960x640`. `size(width, height)` replaces the complete pair. Canvas owns the
composed dimensions and replaces child Plot dimensions for its render context.

`05_LAYOUT_SYSTEM.md` defines the approved replacement contract. After the
atomic Phase T6 migration, absence of a sizing call selects automatic layout,
consuming `width(u32)` requests exact width with automatic height, and
`size(width, height)` remains exact. Sizing calls replace the complete request:
`width` after `size` discards the prior exact height; `size` after `width` replaces
both dimensions. Zero dimensions fail at preflight. No public maximum-width or
general bounds object is admitted. Final canonical dimensions then belong
to the immutable root plan and rendered `Output`, not infallible pre-render Plot
getters. None of those target behaviors or names is current before T6.

The target protocol keeps scientific layout private. Parents use intrinsic
requirements to allocate child constraints; each child returns one immutable
measured local plan or a typed fit miss. Parents assign final positions. Named
bounded local choices and root-coordinated feedback replace exhaustive
cross-component search. This deterministic policy does not promise to find every
feasible arrangement. Semantic content does not silently clip, elide, or disappear
to satisfy a bound. Exact constraints either
fit the complete requested content or fail before backend creation.
Every root mode passes shared planning-work, representability, and fallible
allocation checks. PNG-specific dimensions, pixels, and parser-input byte limits
apply only at PNG publication; neither hypothetical default-PNG viability nor an
automatic-only 64 MiB SVG byte gate can reject SVG. A valid SVG in any sizing mode
may therefore be followed by a PNG refusal without relayout or scene mutation.

T5a0s records user approval of these target behaviors. Automatic placement must
validate the affected plot-plus-guide layout, not accept guide-only fit. Bounded
supported edge/reflow attempts may remeasure affected children, but explicit
placements, typography, and content remain fixed. Current sizing and publication
methods are unchanged until their scheduled implementation and public migration.

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

Planning retains complete text/glyph plans and producer placements before final
capture. Capture and serializer counting do not open Null lifecycles. Canonical
SVG storage and validation scratch are reserved before the final Null validation;
concrete replay uses the same frozen calls without remeasurement or reshaping.
Automatic guide placement validates the affected body, axes, labels, and guide
together and uses bounded supported recovery. Explicit placements remain fixed.

`Canvas` renders all panels through one backend begin/end lifecycle. Marks are
clipped to their panel; titles, axes, legends, captions, and other decorations
remain outside mark clipping where required.

## Output Formats

Publication SVG, preserve-text SVG, and PNG are unconditional capabilities.
Plain `svg` derives vector glyph paths from the scene's retained font faces and
planned glyph IDs and verifies that no `<text>` elements remain. The derived
SVG requires no recipient-side fonts, but its text is no longer selectable,
searchable, editable, or available to assistive text extraction.
`svg_preserve_text` publishes the canonical SVG with accessible, selectable,
searchable, and editable `<text>` elements intact. PNG rasterizes the derived
outlined SVG through `resvg` and `tiny-skia`. PNG sizing validates positive,
finite DPI or scale, positive dimensions, and bounded pixel work before allocation.

The PNG conversion limits are source and target axes through `32_768` inclusive,
source and target products through `100_000_000` pixels inclusive, and actual
SVG parser input through `67_108_864` bytes inclusive. Validate call-local options
before expensive conversion, then source geometry, rounded-positive target,
target axes/work and DPI metadata, actual outlined input bytes, parse/dimension
agreement, pixmap allocation, render, and encode. Existing canonical source-byte
admission is not a substitute for checking the expanded parser input.
Preserve-text and outlined SVG routes do not apply those PNG-only limits, either
before or after the Phase T6 sizing switch. Shared representability, resource
admission, and fallible allocation still apply.

The completed T5a3 repair validates requested PNG options before outline work,
counts actual outlined parser input before allocating that derived payload, and
checks the parser bytes again before parsing. These PNG-only refusals do not
poison the retained SVG source or outline cache. Public automatic sizing remains
scheduled for T6; its target preserves the same format-local separation.

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

Scene preparation discovers installed fonts and resolves faces before text
measurement. `fontdb` supplies face metadata and host generic-family names;
project-owned ordering and glyph coverage choose the preferred available face
for each required visible unit or scalar. Font candidates are not layout alternatives:
layout does not switch to a less-preferred font to make content fit.
The scene retains font identities, bytes, and planned glyphs for later outlined
SVG and PNG conversion without rediscovery. Rendering performs no download,
provisioning, or project-cache lookup. T5a3 implements the canonical selection,
Unicode, geometry, and admission contract detailed in
`03_THEME_SYSTEM.md` and `05_LAYOUT_SYSTEM.md`; public custom font types remain
target-only until T6.

Font selection prefers one compatible face for
a base letter and its combining marks when a face in the ordered stack covers
the unit; otherwise it retains first coverage per visible scalar. Missing visible
coverage is an error, not replacement squares. The precise unit and retained
selection rules are in `03`/`05`. The original source remains unchanged, including
nonpainting Unicode controls and discretionary soft hyphens. Selected soft-hyphen
breaks paint the prepared literal hyphen with retained original source mapping.

Publication SVG fails if any text remains after conversion. Its outlined text
requires no recipient-side font, but exact paths and text-derived layout depend
on the generation font environment. Preserve-text SVG remains accessible and
editable, while its displayed glyphs depend on viewer fonts and shaping. For
identical inputs/options and the same generation font environment, scene geometry
and retained output decisions are deterministic; no cross-viewer pixel parity
is promised for editable text.

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
Project-owned layout or serialization arithmetic overflow and first-party
capacity reservation refusal are render errors with no reduced-quality fallback.
The contract does not promise recovery from process-global allocation failure
inside an external dependency.

## Artifact Publication

Terminal path arguments are base file destinations. After deriving the complete
output bytes successfully, `svg` and `svg_preserve_text` complete the final
filename with `.svg`; `png`, `png_preview`, and `png_with_options` complete it
with `.png`. Plot and Canvas terminals delegate this operation to the same
`Output` boundary and do not apply a separate suffix rule.

If the final native filename already ends in the complete expected ASCII suffix
under ASCII case-insensitive comparison, the complete caller path is preserved
exactly. Otherwise the lowercase suffix is appended to that filename only.
This rule is literal: it does not interpret an extension or infer a format,
and it never removes, replaces, normalizes, or rejects contradictory filename
text. Thus `chart.v1` becomes `chart.v1.svg`, `chart.png` passed to `svg`
becomes `chart.png.svg`, `chart.SVG` remains `chart.SVG`, `.chart` becomes
`.chart.svg`, and `.svg` remains `.svg`.

Suffix completion uses native filename units without requiring UTF-8. It first
rejects a caller path that currently identifies a directory, following symlinks
according to `Path::is_dir`, and then requires a file-name component. These path
errors are `Error::Io` values with `InvalidInput`. Format conversion and byte
derivation precede all caller-path access. The completed path then undergoes the
ordinary parent canonicalization, Windows leaf validation, sibling staging,
replacement, and cleanup sequence. Parent directories are not created. An
existing unsuffixed non-directory base is neither read nor replaced when its
suffixed sibling is the completed destination.

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

Renderer behavior is covered by focused semantic unit/integration tests: source
completeness, scientific transformations, styles, containment, lifecycle, requested
output dimensions and format validation. Whole-scene SVG sizes/hashes, stored
scene snapshots and incidental automatic geometry are not acceptance gates.
Equivalent current routes and frozen replay may compare their actual outputs for
determinism; numerical rounding and serialization syntax use independent oracles.
`90_DEVELOPMENT.md` defines the current test-expectation policy.
