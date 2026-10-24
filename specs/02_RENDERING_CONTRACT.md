# Rendering Contract

## Scientific Inputs And Validation

Plot consumes owned immutable native values. Semantic constructors return exact
`Plot<kind::Family>` values; sealed capabilities reject categorically unsupported
configuration at compile time. Data-dependent checks validate aligned lengths,
finite values, categorical identities, source provenance, bounds, facets, actual
axis kinds and representable geometry before backend begin. Standalone and Canvas
children use the same canonical validator in `src/orchestrator/validation.rs`;
composition adds restrictions and never weakens Plot invariants.

Required constructor channels have one owner. Trajectory, LinePath, Interaction
and stacked-series groups, and Heatmap/Beeswarm continuous color values, cannot
be replaced by a later setter. Explicit `Plot::erase` retains common/output
operations without regaining exact-family configuration. Canvas accepts exact
children and erases only at storage.

### Categories, Groups And Provenance

First-observed categorical order is deterministic by default. Explicit order
names every rendered category exactly once. Value-derived order is valid only
when the family prepares one unambiguous scalar rendered value per category.
Physical input row order does not substitute for category order.

`Plot::source_rows` owns one aligned diagnostic provenance map for all plot-aligned
values, facets, composed views and raw-observation overlays. Individual values
and layers do not own competing provenance. Original slot indices remain distinct
from arbitrary SourceRows IDs through filtering, sorting, status handling and
scientific preparation.

Bars, HorizontalBars, Boxplots, sampled Violins and sampled Rainclouds optionally
use `Plot::series` for positioning, color, guide identity and supported significance
targets. Stacked bars require series atomically. Supplied Raincloud geometry
shares its marker but rejects sample-only series during preflight.

Density, ordinary Line, Interaction, Marginal, Matrix and Hue LinePath keep group
and hue independent. Each observed `(group,hue)` pair owns one collective mark;
hue without group partitions by hue. Do not create unobserved Cartesian pairs.
Ordering is first-observed group, then first-observed hue within group. Density,
Marginal and Matrix reject a pair unable to produce its required geometry.

Equal group/hue field names require identical aligned raw values and collapse
to one semantic dimension. Same name with different values rejects. Distinct
fields stay distinct even if display values/aliases or paints are equal.

Interaction cells are unique by `(condition,group,hue)`. Explicit InteractionStyle
maps raw group to point shape/connector pattern, applying to every hue-specific
trace of that group. Selectors are nonempty already-trimmed control-free raw
identities; preserve interior spaces/Unicode bytes. Reject unknown, duplicate or
incomplete configured group mappings before fonts. Color maps key hue when
present, otherwise the actual color-owning group. Plain unstyled grouping
partitions geometry without creating a guide identity.

InteractionStyle consumes `point_shape(value)` / `line_pattern(value)` edits and
retains `get_point_shape()` / `get_line_pattern()` readers. Its read-only raw
selector keeps `group()`. Each edit validates the complete style.

### Summaries And Closed Sources

Ordinary bars consume one BarInput: samples, one estimate per cell, or estimates
with drawable Bounds. Samples render arithmetic cell means; the default error
bar is deterministic two-sided 95% percentile bootstrap with 1,000 resamples and
seed zero. Sampling units select multilevel resampling within category/group/facet
cells. A singleton observation/unit renders its mean without error bar. Stacked
bars retain separate sum semantics.

Bounds is the sole caller lower/upper representation; it carries no inferred
classification, level, scale or statistical provenance. PointInterval and
Interaction require bounds atomically, Interaction also its graphical group.
Forest requires bounds and scientific reference. CoefficientInput selects
estimates or estimates_with_intervals. Validate aligned finite drawable endpoints
and `lower<=estimate<=upper`; unavailable rows retain explicit status semantics.
Ordinary Line alone accepts optional band geometry via `Plot::band`.

HistogramInput and RaincloudInput select samples or complete supplied geometry
at construction. Supplied geometry is self-contained and rejects sample-only
facets/source-row channels at preflight. Supplied KDE over sample-owned Histogram
bins/rugs remains an explicitly selected composite, not a second whole input.
Scientific zero, missing, finite-filtering and availability policies belong to
each closed family input; appearance cannot substitute another source policy.

### Paths, Layers And Specialized Inputs

Ordinary Line requires numeric x and sorts numeric series. Trajectory requires
graphical group, sorts within it and always draws observation markers; its x
conditions may be numeric or categorical. LinePath requires numeric x/group,
preserves original row order/repeated x and alone accepts LinePathStyle. Ordinary
Line and LinePath marker visibility remains configurable, not Trajectory's.
Trajectory/Neutral LinePath groups own connectivity while hue may vary by row;
their hue keys explain points, not neutral connectors. Hiding Neutral LinePath
markers suppresses that inert hue contribution.

Only ordinary Line admits PointLayer. Each appended layer owns a nonempty
independent aligned numeric x/y dataset, not necessarily aligned with primary
data. Layers retain append order over primary bands/lines/markers and reject
generic faceting. Optional per-point label strings and one layer guide label are
different content. BarPointLayer uses exact prepared category/series centers.
Exact paint, alpha, radius, accent and label-position ownership are specified in
[visual properties](04_VISUAL_PROPERTY_OWNERSHIP.md).

Dumbbell atomically consumes both endpoints and explicit distinct validated labels/
complete Colors. Exact endpoint paint also supplies its literal key, independently
of neutral connector and point outline.

Waterfall consumes finite base/deltas and derives every span and terminal by one
ordered cumulative pass. Every intermediate endpoint must be finite. The final
cumulative endpoint is the only terminal used by ranges/guides/labels; never
substitute a differently associated detached delta sum. Preserve scientific zero
spans, source order and adjacent connector cardinality.

Heatmap and Beeswarm share complete continuous-color scale selection and one
final-domain mapping for marks/guides. Their point/cell alpha is independent of
the opaque color gradient. Required labels are endpoints and an interior diverging
center, or one constant, not every gradient stop. Heatmap missing blocks and
fully-null-row annotations retain their distinct semantic owners.

### Facets And Composition

FacetSpec owns one complete row, column, grid or wrap layout plus axis sharing.
Wrap columns are positive; grid variable names differ. Facet Fixed numeric axes
share one domain. FreeX/FreeY/Free resolve selected dimensions per panel. A free
estimate axis with only NotEstimable rows requires explicit limits; it cannot
borrow a sibling range. Categorical shared axes require matching raw identity/order.

Matrix/Marginal do not support generic facets. Supplied Histogram/Raincloud
geometry rejects them at runtime. Already faceted plots cannot be Canvas children;
Matrix remains standalone native. Canvas retains explicit row-major topology,
empty/trailing cells and spans, and does not compact scientific absence.

Canvas shared-x groups require configured members with continuous automatic x
domains. Categorical axes, explicit x limits and unconfigured members reject.
Install the finite strictly ascending domain union before child measurement.
Shared range alone does not make formats/tick instructions identical.

### Scientific Windows And Required Content

`x_limits`/`y_limits` are explicit data windows. Retain finite off-window marks and
crossing endpoints, project crossings and apply panel clipping; do not mutate
values, broaden exact domains or reject merely because scientific paint is clipped.
A marker stroke centered at an explicit boundary may clip there.

Automatic axes instead contain actual painted mark requirements through the
named intrinsic/reserve/domain rules. Mixed exact/automatic axes apply this
distinction per dimension. Nonfinite/unrepresentable geometry and nonzero source
geometry collapsed by transformation/quantization still reject, even when hidden
by clipping. True scientific zeros retain their defined family behavior.

Data windows do not permit clipping required titles, axes/ticks, guides or
annotations outside assigned boxes. An annotation's explicit visibility/window
policy may suppress it; otherwise it fits complete uncut ink or fails. Exact image
size cannot succeed by clipping layout content. Theme chrome absence does not
remove semantic content. Heatmap's explicit Wrap/Elide category-label policy is
not a general text-omission control.

## Presentation Owners

Complete immutable appearance, inheritance, eighteen text roles, local spacing,
scales and contrast are defined in [Theme system](03_THEME_SYSTEM.md). Exact
styles/data-alpha use the [property contract](04_VISUAL_PROPERTY_OWNERSHIP.md).
Generated tick choice and numeric spelling are separate policies in
[numeric presentation](06_NUMERIC_PRESENTATION.md). Exact AxisTick lists stay
indivisible and verbatim. Neither numeric formatting nor alpha changes scientific
statistics, domains, source membership or categorical identity.

### Display Text

Free-form display text shares one source-syntax validator in `src/text.rs`.
Plot/Canvas prose, x/y titles, TextPanel title/body, Canvas headers/row titles,
panel notes, point labels, guide display text and aliases preserve caller bytes,
whitespace and mandatory breaks. LF, CRLF and the engine's mandatory separators
are accepted; CRLF is one break. Tabs and unsupported controls reject at a
fallible constructor or preflight, before shaping. There is no trimming or
single-line restriction on these display slots.

Present empty prose, titles and annotations retain one empty line's font metrics;
structural absence is `None` or an explicit owner visibility policy. Exact tick
empty labels deliberately suppress text/gap while keeping their positions.
PanelTextAnnotation fits its complete block at its selected corner and retains
the existing duplicate-corner, collision and facet rules.

Automatically sourced names still default to `Labels::humanized()`. Public
`Labels::resolve(&self, raw: &str) -> Result<String>` uses the same alias-first,
humanized/raw resolution as rendering; aliases preserve their explicit bytes.
One-name resolution does not replace complete-domain collision checks. Raw
category/series/field keys, endpoint identities, font-family names, significance
selectors, paths and NumericFormat grammar retain their own semantic validation.

Plot/Canvas title, subtitle and caption, Plot x/y and color-guide titles, TextPanel
title/body and Canvas column/row headers and row titles accept `impl Into<Text>`
through one source representation. String, &str and &String
carry no local instructions. Text optionally retains
one complete exact TextTheme and one whole TextLayout request. Its alignment edit
uses shared TextAlignment. TextLayout defaults leave alignment unspecified, while
an explicitly supplied default request remains distinguishable from omission.
TextPanel's existing title/body readers borrow the source as `&str`.

Prose, x/y-title and color-guide-title setters replace source in call order. Omitted incoming
style/layout preserves the slot's earlier instructions; a present style replaces the complete appearance,
and a present layout replaces the complete request. An explicit default layout
resets alignment to owner defaults. Plot title/subtitle convenience alignment
edits write that same request, including before any content is supplied. Presence
remains separate from an empty source and from retained instructions.
X/y title styles resolve independently over the shared Theme axis-title role.
Their physical line alignment defaults to Center within each centered title block,
before its horizontal/vertical transform, including parent-owned facet titles.
Canvas headers retain declared span and overlap validation at insertion. Each
local style replaces the complete Canvas header role; line alignment defaults to
Center inside the centered block. Band padding uses each selected title size.

PanelTextAnnotation likewise accepts `impl Into<Text>` and preserves complete Eq;
its existing text reader returns `&str`. Local style replaces the Plot's panel-text
role before the census. Local line alignment is independent of corner ownership,
and the selected size determines the annotation's font-relative padding and fit.

### Guides

LegendPolicy is one automatic/hidden/explicit-position choice; placement and
visibility are not independent switches. LegendComposition selects truthful
Combined/Separate explanations under the [guide contract](08_GUIDE_SYSTEM.md).
All 25 exact families are Legend-capable; Any is not. Source-owned key geometry,
complete final paints and mapping provenance feed one PreparedGuide. Native
Heatmap/Beeswarm right tracks and Matrix trailing track place their guides once;
transport for census/Shared identity must not create an outer duplicate.
`color_label(impl Into<Text>)` retains complete local style and line alignment over
the consuming Plot or Shared Canvas guide-title role. Shared contributors require
equal complete Text requests before fonts, even when different requests could
produce identical text or geometry. Guide titles default to Start inside the
left-positioned block; selected title size owns its title gap.

## Significance

Significance<Attached> and Significance<Pairwise> are caller-owned display data,
constructed with `at` and `between`. Raw strings identify full categories;
SignificanceTarget::category_series selects a positioned series. Sealed placement
capabilities gate attachment. Bars/HorizontalBars/Coefficient admit attached
marks; bar variants/Boxplot/Violin/sampled Raincloud admit pairwise brackets.
Stacked series do not supply separate bracket anchors. Coefficient targets terms
and does not require drawable bounds merely for significance. Supplied Raincloud
geometry rejects significance during preflight.

SignificanceMark accepts ns, one-through-four stars or one caller-selected display
p-value. Values >=0.05 display ns; smaller values use conventional one/two/three/
four-star thresholds. Every supplied annotation renders. Tests, adjustment,
familywise provenance and inferential interpretation remain caller-owned.

Facet annotations identify every configured dimension exactly once via
FacetCell::single/grid, without repeating variable names or silently replicating
annotations. Resolve original category/series/facet identities before fonts.
Complete selected style, label, bracket and shelf fit remain mandatory.

## Root Sizing And Scene Lifecycle

Plot/Canvas each retain one complete sizing request: omitted is natural width
chosen once followed by measured height; `width(u32)` is exact width/automatic
height; `size(u32,u32)` is an exact pair. Each call replaces the complete request,
so width clears earlier height. Zero rejects at preflight without setter panic.
Canvas controls embedded dimensions and ignores child standalone requests.
Output alone exposes frozen canonical dimensions; no pre-render Plot dimensions,
public maximum width or generic constraints are provided.

[Layout](05_LAYOUT_SYSTEM.md) defines constraints-down, one immutable result-up,
parent positioning, bounded local choices and at most eight root feedback rounds.
Automatic placement must fit the entire affected body, axes, labels, annotations
and guide, not just guide chrome. Explicit choices stay fixed. A policy fit error
can occur despite an unimplemented feasible alternative; no shrink/elision or
exhaustive layout solver is implied.

An operation resolves/validates immutable inputs, prepares scientific sources and
parent domains/guide compatibility, establishes complete text requirements,
discovers/resolves installed fonts, obtains intrinsics, and measures the selected
root. Capture each selected producer once into immutable commands. Count canonical
serialization and reserve canonical scene/SVG/scratch storage before one balanced
final Null validation. Count/capture are not extra Null lifecycles. Preflight uses
the same preparation/planning/validation path without publishing an artifact.

Terminal Plot/Canvas calls consume the owner, render one canonical scene through
one balanced concrete lifecycle, publish one file and return Output. Canvas uses
one begin/end for all panels. Scientific marks use their panel clip; required
layout ink remains separately contained. Frozen replay does not measure, shape,
format, prepare statistics, reread sources, change domains or choose layout again.

Output publications consume/reuse the exact retained scene. No SVG text mode or
PNG option is stored. Chained calls are independent same-scene publications, not
one multi-file transaction. A failed later publication leaves earlier successes.

## Fonts And Determinism

Only installed system fonts are used, through typed role stacks. There is no
download, bundled-font recovery, project asset cache or embedded font data.
Operation-local canonical discovery/matching, same immutable bytes, used-role
census, Unicode16 text behavior and base-plus-mark fallback are specified by
layout. Missing visible coverage, invalid selected metrics or selected-face
shaping failure is fatal; layout cannot select a different font to fit.

Identical validated inputs/options/dependency versions and generation-font
environment give deterministic scene decisions. Do not promise bytes across
renderer revisions, hosts or font environments. Outlined SVG/PNG consume retained
generation glyphs; preserve-text SVG remains viewer-font/shaping-dependent.

## Output Conversion

Outlined SVG, preserve-text SVG and PNG are unconditional first-party capabilities.
`svg` converts every text element to vector paths using retained glyph identities
and same-byte face sources, then verifies no text remains. It needs no viewer
fonts but loses selectable/searchable/editable text. `svg_preserve_text` keeps
canonical accessible text, selected line structure and validated CSS stack,
without embedding fonts. Conversion never relayouts or changes the scene.

PNG rasterizes the outlined scene through resvg/tiny-skia. `png` uses 300 DPI and
Balanced compression; `png_preview` uses canonical CSS dimensions, Balanced and
no physical-density metadata; `png_with_options` uses its call's exact sizing/
compression. Output width/height remain canonical, not the raster target.
Canonical SVG contains no raster image elements; unrelated JPEG/GIF/WebP decoders
are not part of the output contract. PDF conversion belongs outside the crate.

### PNG-Local Limits And Ordering

Source and target axes are at most 32,768, source and target area at most
100,000,000 pixels, actual outlined parser input at most 67,108,864 bytes, all inclusive.
Validate positive finite DPI/scale or positive requested dimensions before
expensive conversion; then source geometry, rounded-positive target, target
axes/area and DPI metadata, actual outlined payload bytes, parser/dimension
agreement, pixmap allocation, rendering and encoding. Check known limits before
their corresponding allocations/work. Canonical text-byte count is not the
expanded outlined parser count; check the actual payload before parsing too.

These are PNG-only limits in every root sizing mode. SVG has no hypothetical
default-PNG viability test, automatic-only 64 MiB gate or replacement scene-byte
quota. Checked representation/lengths and fallible first-party buffers still
apply. Valid SVG can fail a later PNG conversion without invalidating SVG or
mutating retained scene/font/path sources. No retry at another layout/width.

## Errors And Storage

Public operations return the crate Result: Error::Io for filesystem failure,
Error::Data for invalid data/data-bound options, Error::Render for layout/font/
backend/rasterization/publication failures. No alternate renderer or output path
is attempted after failure. Fit misses are typed layout outcomes; real arithmetic,
allocation, I/O, font and invariant errors are immediately fatal.

Project-owned lengths/products/indices/capacities/spatial conversions are checked;
known first-party extents reserve fallibly, unknown input grows fallibly from
actual demand. Real serializer counts and parser lengths are retained. There is
no cumulative visits/records/cache/topology/discovery/byte-price ledger or
production budget switch. This does not promise recoverable process-global OOM
inside external dependencies. Existing mathematical/source/input limits and
bounded layout algorithms are independent of resource pricing.

## Artifact Publication

After complete byte derivation succeeds, SVG methods complete the native filename
with `.svg`, PNG methods with `.png`. If it already ends in the entire expected
ASCII suffix under ASCII case-insensitive comparison, preserve the path exactly;
otherwise append lowercase suffix to the leaf. Do not infer format, replace
extension text, normalize or reject contradictory names. Thus chart.v1 becomes
chart.v1.svg, chart.png passed to svg becomes chart.png.svg, chart.SVG stays,
.chart becomes .chart.svg and .svg stays.

Native suffix completion does not require UTF-8. Caller-path access follows byte
derivation. Reject a directory destination (following Path::is_dir symlinks) or
missing filename as Io/InvalidInput. Parent directories must already exist. Then
use parent canonicalization, Windows leaf validation, unique sibling staging,
replacement and cleanup. An unsuffixed nondirectory base is not read/replaced
when its suffixed sibling is the destination.

Retain the staging handle for every write. Publish by one same-directory atomic
replacement on supported POSIX/Windows hosts only after bytes succeed. Success
installs complete staged bytes; failure preserves the previous destination with
no backup/rollback window and removes owned staging residue. Windows publication/
cleanup uses the exclusive staging handle. POSIX rename/unlink remain pathname
syscalls: their exact-source claim assumes no external namespace mutation between
final identity check and syscall. Callers exclude independent concurrent output
namespace mutation. `src/orchestrator/artifacts.rs` and `src/output.rs` own these
publication/conversion boundaries.
