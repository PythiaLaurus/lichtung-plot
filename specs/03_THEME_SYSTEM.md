# Theme System

**Status**: Complete immutable foundations and narrow heading instructions current;
numeric/sectioned-guide presentation remains a staged target

## Purpose

This specification defines the complete theme, color, and
data-color-scale model for `lichtung-plot`. `05_LAYOUT_SYSTEM.md` defines the
approved coordinated replacement layout model.
`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative for the
implemented public API. Phase T4 in `91_IMPLEMENTATION_PLAN.md` made the
Canvas/Plot ownership and complete built-in inheritance subset current using the
closed `Theme` selector. The complete immutable Theme/component/font/scale
construction cutover replaces that selector and its callers. Narrow heading
instructions are current; later presentation policies become current only in
their owning phases. No alias or dormant placeholder is a second API.

T5a0s in `91_IMPLEMENTATION_PLAN.md` records accepted layout-related behavior:
strict Unicode-legal wrapping, the three-mode sizing surface, format-local PNG
limits, compatible base-and-mark font fallback, and independent caption alignment.
The public three-mode sizing cutover is current. Complete theme caption and
heading alignment defaults and the narrow Plot/Canvas instruction setters below
are current. Remaining broader text controls
remain deferred with explicit review gates.

The completed T6 ENTRY review additionally admits one locale-free scientific
numeric presentation core and selectable combined/separate discrete legend
composition. Their execution and detailed contracts are in
`93_T6_EXECUTION_PLAN.md`. Both are semantic presentation policies, not theme
appearance. Remaining DF2/DF5/DF7 scope is reviewed at T7 CLOSURE.
Their refined architectural contracts and consumer/proof designs are closed in
`06_NUMERIC_PRESENTATION.md`, `07_LOG10_DISPLAY.md` and `08_DISCRETE_GUIDES.md`;
implementation and executable acceptance evidence remain pending.

T6A2's authoritative validated construction, fields, metrics and migration
signatures are in `94_T6_FOUNDATION_CONSTRUCTION.md`. They specify the complete
current foundation taxonomy. The user chose theme-only Shared
legend spacing and one shared `#F7FBFF -> #08306B` sequential default; the latter
intentionally replaces Beeswarm's different built-in sequence at T6B2.

This model governs standalone plots, facets, composed
canvases, shared legends, text panels, SVG, and PNG. It does not authorize
mutable global configuration, dynamic style plugins, backend-specific
properties, or a second theme representation.

## Concepts

- A **theme** is one complete immutable set of visual defaults. It owns
  appearance, not scientific data, statistical behavior, semantic labels, or
  output routing.
- A **preset** is a constructor for a complete theme. Publication, Nature,
  Science, and CHI are presets, not enum identities or runtime modes.
- A **component theme** is one role-based part of a complete theme, such as axes,
  legends, marks, or annotations. Components are not independently inherited at
  render time.
- A **data-color scale** maps categorical or numeric data values to opaque
  colors. Scale resolution is distinct from non-data theme appearance.
- An **exact element style** configures one caller-owned guide, layer,
  annotation, or mark. It has narrower scope and higher precedence than a theme
  or data scale.
- A **policy** controls semantic presence or placement, such as legend
  visibility, marker visibility, category-label sharing, or content supplied by
  the caller. A theme does not replace a policy.
- A **renderer invariant** is fixed behavior such as clipping, draw order,
  geometry validity, semantic orientation, or backend protocol. Text anchoring
  is invariant except for the root title, subtitle, and caption alignment
  defaults and narrow Plot/Canvas overrides defined below.
- A **resolved theme** is the private complete value consumed by preflight,
  measurement, collision planning, legend construction, and rendering. It is
  not a public inspection or extension surface.
- A **relative spacing value** is a validated dimensionless visual relationship
  resolved against the local text, glyph, mark, or composition metrics it
  separates. It is not a root-canvas scale factor or output-size policy.

## Public Foundation Model

The public foundations consist of these roles:

| Type | Responsibility |
| --- | --- |
| `Theme` | One complete immutable renderable theme |
| `ThemeBuilder` | Construction-only editor initialized from one complete theme |
| `SurfaceTheme` | Canvas and panel paints |
| `TypographyTheme` | Every admitted role-based text style |
| `TextTheme` | One complete text role: font stack, color, size, weight, and finite positive line-height multiplier |
| `AxisTheme` | Frame sides, axis and tick strokes, grid, lengths, and role-local spacing relationships |
| `LegendTheme` | Discrete legend and continuous color-guide chrome, keys, padding, and role-local spacing relationships |
| `CanvasLegendPolicy` | Per-plot, hidden, or one shared Canvas legend behavior |
| `LegendComposition` | Closed Combined/Separate semantic composition of one discrete guide; not a component theme |
| `TitleAlignment` | Closed `Start`, `Center`, or `End` horizontal alignment for Plot or Canvas root titles, subtitles, and captions |
| `CompositionTheme` | Plot and Canvas root title, optional subtitle, and independent caption alignment defaults and spacing relationships, facet strips, Canvas headers, row titles, text-panel chrome, and role-local composition spacing relationships |
| `MarkTheme` | Common point, line, fill, interval, ribbon, reference, outline, missing-mark, and supplemental-accent defaults |
| `AnnotationTheme` | Panel-text, significance, value-label, and highlight geometry plus contrast-backdrop policy |
| `ColorScales` | Default qualitative, sequential, and diverging data-color scales |
| `FontStack` | Ordered validated generation-host font-family preferences |
| `FontFamily` | One validated named family or closed generic family |
| `OpaqueColor` | RGB-only data-scale color |
| `Color` | Complete RGBA paint color |
| `QualitativePalette` | Ordered categorical colors with explicit capacity behavior |
| `AccentPalette` | Ordered non-data colors for primary and supplemental mark roles |
| `CategoricalColorMap` | Exact raw categorical identity-to-color mapping |
| `SequentialGradient` | Ordered low-to-high numeric color mapping |
| `DivergingGradient` | Ordered low-to-neutral-to-high numeric color mapping |
| `CategoricalColorScale` | Plot-local ordered palette or exact keyed mapping |
| `ContinuousColorScale` | Plot-local sequential or diverging domain and optional gradient replacement |

The component types have private fields, typed accessors, and validated
construction. They do not expose string-keyed property maps or backend style
objects. The visual-property inventory in Phase T2 assigns each admitted field
to exactly one of these components before the public API is implemented.

`Theme::publication()`, `Theme::nature()`, `Theme::science()`, and
`Theme::chi()` each return a complete validated value. There is no public preset
enum. `Theme::customize()` returns a `ThemeBuilder` initialized with the complete
source theme. The builder may replace components or use typed convenience
setters, but `build()` returns one complete validated `Theme`. A builder is never
stored by `Plot`, `Canvas`, or the renderer and is not a sparse runtime override.

`TitleAlignment` is the closed `Start`, `Center`, and `End` discriminant.
`CompositionTheme` owns concrete `root_title_alignment: TitleAlignment`, optional
`root_subtitle_alignment: Option<TitleAlignment>`, and independent concrete
`root_caption_alignment: TitleAlignment`. No separate caption alignment type is
introduced. `Start` and `End` mean the physical left and right edges of the root's
post-inset local content span; they are not bidi-logical directions or backend
anchor spellings. Both block placement and per-line alignment consume each role's
effective value. All four built-in themes use `Center` for root title, `None` for
root subtitle, and `Start` for caption. A subtitle with no applicable instruction
uses the final effective title alignment, as resolved below.

The public cutovers replace the closed `Theme` enum, map-only categorical control,
Heatmap-only color selector, RGB tuple inputs and floating opacity routes. Every
repository caller uses the complete values that own those responsibilities;
replaced representations and spellings are deleted, without compatibility wrappers.

Spec94 assigns the current exact-style, root font-export, palette/map/gradient
attachment and consuming-edit cutovers explicitly. `TitleAlignment` is public
with its consumed complete CompositionTheme at B4; the four instruction setters
follow at B5. `Canvas::shared_legend_gap` and its override field are removed at B4;
custom Shared spacing is supplied only by the complete LegendTheme component.

## Component Boundaries

The component taxonomy is role-based rather than plot-family-based:

- `SurfaceTheme` owns the root canvas and plot-panel paints.
- `TypographyTheme` owns title, subtitle, caption, axis-title, tick-label,
  legend-title, legend-label, facet/header, body, detail, annotation, and
  automatic light/dark contrast text roles. Each role is complete. A ThemeBuilder
  bulk font operation may update all roles, and an individual role may use a
  different stack.
- `AxisTheme` owns nonsemantic axis chrome. Frame sides, grid lines, and tick
  marks may be absent. It does not decide whether caller-provided titles, exact
  ticks, tick labels, or axis labels exist.
  Ordinary panel `frame` and variable-grid `cell_frame` are independent complete
  roles in the same component. Built-ins preserve their existing four-side and
  bottom/left defaults respectively; custom cell sides are not masked by Matrix.
- `LegendTheme` owns appearance and measured geometry of discrete legends and
  continuous color guides. Legend policies continue to own visibility,
  placement and combined/separate semantic composition. Numeric presentation
  policies own generated label spelling separately from typography.
- `CompositionTheme` owns independent title, optional subtitle, and caption alignment defaults,
  spatial relationships for Plot and Canvas root title blocks, and Canvas/facet chrome.
  `TypographyTheme` still owns each title, subtitle, and caption text role.
  `CompositionTheme` does not own title content, panel data, grid occupancy,
  shared-range policy, or text-panel content.
- `MarkTheme` owns common visual defaults by semantic mark role, including one
  supplemental accent palette for unkeyed primary and appended mark roles. A
  specialized component is justified only when a mark has genuinely distinct
  semantics, not merely because one renderer currently contains a hardcoded
  constant.
- `AnnotationTheme` owns non-text geometry and spacing for caller-provided and
  renderer-generated annotations, together with the opaque planning backdrop
  used for automatic contrast on transparent output. Annotation text paint and
  metrics remain `TypographyTheme` roles. `AnnotationTheme` does not decide
  whether caller-provided annotation content is present.
- `ColorScales` owns default data-color mappings. Foreground, surfaces, grid,
  text, neutral references, and missing marks are not qualitative palette slots.

Visual spacing is local. Text leading comes from the owning `TextTheme`; axis
gaps resolve against axis text and stroke metrics; legend padding resolves
against legend text and exact key glyphs; annotation clearance resolves against
annotation text, stroke, and mark extents; Plot and Canvas root title-block
relationships resolve from `CompositionTheme` against adjacent title metrics;
and other composition gaps resolve against the owning Canvas or facet
typography. A theme never scales spacing from the root canvas width or height.
`05_LAYOUT_SYSTEM.md` owns parent constraints, intrinsic requirements,
deterministic component measurement, parent allocation/arrangement, bounded
feedback, and output dimensions. A component returns one immutable measured
plan or typed fit miss; it does not export complete alternatives for root search.

Except for the admitted root horizontal alignment defaults and narrow overrides,
anchor, rotation, orientation, clipping, z-order, statistical
geometry, collision rules, local layout decision order, feedback budgets, and
minimum representable geometry remain renderer invariants. Root title,
subtitle, and caption rotation remains fixed.
Bar spacing, legend policy, marker and connector visibility, category-label
sharing, title and label content, and annotation presence remain explicit
Plot/Canvas configuration or input.

## Ownership And Inheritance

`Canvas::theme(Theme)` and `Plot::theme(Theme)` are the sole owner attachments for
complete immutable values. The Canvas call is optional because Canvas constructs
`Theme::publication()` as its default root. Omission of the Plot call represents
inheritance when composed and owner-local Publication when standalone.

`Plot` stores an optional complete explicit theme. Absence has two meanings
selected only by the render owner:

- A standalone Plot without an explicit theme resolves to
  `Theme::publication()`.
- A Plot inside a Canvas without an explicit theme inherits the Canvas root
  theme.

`Canvas` stores one complete root theme and defaults to `Theme::publication()`.
An explicit Plot theme completely replaces the inherited theme for that plot;
components are not merged. Facets inherit their owning Plot's one resolved
theme and introduce no additional theme scope.

Canvas-owned title blocks, shared headers, row titles,
text panels, surfaces, and shared-legend chrome use the Canvas root theme. Plot
titles, plot subtitles, plot captions, axes, marks, annotations, and panel chrome
use the resolved Plot theme, subject only to the narrow title/subtitle alignment
overrides below. A composed Plot's panel surface comes from its Plot theme;
it never repaints the Canvas root surface.

A Canvas's own title uses its theme's `root_title_alignment`; its own subtitle
uses that theme's `root_subtitle_alignment` when present, otherwise its own final
title alignment. Its caption uses that theme's `root_caption_alignment`.
Canvas panel alignment settings are separate and do not affect Canvas-owned text.
A standalone, embedded, faceted outer-root, or Matrix-root Plot retains Plot text
semantics and the alignment resolution below. Its caption independently uses its
resolved Plot theme's `root_caption_alignment`. Independence changes neither
complete theme replacement nor content ownership.

Theme state is immutable and owner-local. There is no process-global,
thread-local, environment-selected, backend-selected, or asynchronously scoped
theme.

## Resolution And Precedence

Owner precedence is explicit complete Plot value, inherited Canvas value, then
owner-local Publication. Component, scale and exact-element precedence below
governs the current retained foundations.

For a property admitted at multiple scopes, precedence from highest to lowest is:

1. Exact element style on one guide, layer, annotation, or mark.
2. Plot-local categorical or continuous data-color scale configuration.
3. Complete explicit Plot theme.
4. Complete inherited Canvas theme.
5. Complete Publication library default.

Absence at one scope selects the next applicable owner; it does not copy or
mutate that owner. A Plot scale replaces only the relevant scale and leaves all
other theme components unchanged. A complete explicit Plot theme does not merge
with its Canvas theme.

Title/subtitle alignment has a narrow, separately resolved public override surface:
`Plot::title_alignment`, `Plot::subtitle_alignment`,
`Canvas::panel_title_alignment`, and `Canvas::panel_subtitle_alignment`. Each
sets its own optional `TitleAlignment` override input; absence is `None`.
The public word `panel` identifies the composed Plot semantic scope, not a data
panel anchor or a public layout-tree abstraction; private trees may use `child`.

After resolving the one complete Plot theme, choose the first present instruction
in each chain, from highest to lowest:

1. Title: explicit Plot title override > Canvas panel title default (when composed)
   > resolved Plot theme's concrete `root_title_alignment`.
2. Subtitle: explicit Plot subtitle override > Canvas panel subtitle default
   (when composed) > resolved Plot theme's optional `root_subtitle_alignment`
   > final effective Plot title alignment.

`None` means no instruction at that scope, not an instruction to force following
the title or to bypass lower-precedence subtitle settings. These optional inputs
do not make themes sparse or introduce general component merging. Theme setters
and alignment setters are call-order independent; repeated setters replace only
their own stored input. Resolve concrete title, subtitle, and caption alignments
before measurement and retain them in immutable plans; drawing does not resolve
optional inputs again.

The renderer resolves theme and scale inputs before canonical plot validation,
layout measurement, or backend begin. One private resolved theme and the
resolved scales derived from it are reused unchanged through preflight,
measurement, collision planning, legend construction, scene creation, and final
rendering. Renderers and backends do not reconstruct presets or look up theme
state late.

## Color And Alpha

`OpaqueColor` stores three `u8` channels and is `Copy` and `Eq`.
`OpaqueColor::rgb(r, g, b)` is infallible and
`OpaqueColor::hex("#RRGGBB")` accepts exactly six hexadecimal digits encoding
three channels after the leading `#`.

`Color` stores red, green, blue, and alpha as four `u8` channels and is `Copy`
and `Eq`. `Color::rgb(r, g, b)` sets alpha to 255;
`Color::rgba(r, g, b, a)` sets all channels;
`Color::hex("#RRGGBB")` and `Color::hex("#RRGGBBAA")` accept the two exact
forms; and a consuming alpha setter replaces the alpha channel. Conversion from
`OpaqueColor` to `Color` sets alpha to 255.

Data-color palettes, exact categorical maps, and continuous gradients consume
`OpaqueColor`, not `Color`. They map data to chromatic identity and do not
silently introduce a data-alpha aesthetic. Mark and typography roles combine a
resolved opaque data color with their one alpha value to produce a final private
RGBA paint. Exact element paints may supply `Color` directly.

Every final fill, stroke, text paint, bracket paint, outline, and rendered
gradient stop has exactly one RGBA color. Theme mark alpha combines only with an
opaque scale color. An exact element `Color` replaces that complete RGBA paint;
therefore `Color::rgb` intentionally makes an exact element opaque. If the same
exact style also exposes a separate alpha setter, that alpha replaces the
`Color` alpha after all color resolution, regardless of builder call order.
Alpha values are never multiplied. Fill and stroke are independent paints. SVG
emits the one alpha through the property appropriate to the paint and does not
combine RGBA alpha with element-wide opacity.

During the private T5c migration, a retained exact `f64` point/fill opacity
input passes its existing finite/range validation, then resolves once to a
`u8` alpha by nearest-even rounding of `opacity * 255`. That alpha replaces
the complete fill-paint alpha, including for an exact RGB color; it never
multiplies the theme alpha or affects an independent outline/stroke. This
quantization intentionally changes intermediate SVG/PNG bytes for values that
cannot be expressed as 8-bit alpha. T6 replaces the public legacy input
with the complete `Color`/alpha controls; it does not re-quantize retained
marks or introduce a second paint route.

Canvas and panel surfaces accept RGBA `Color`. The SVG backend does not insert a
hardcoded white root. Transparent SVG and PNG output preserves the resolved
surface alpha, and composed panel colors composite over the Canvas surface in
document order.

## Qualitative Scales

A qualitative palette assigns colors to unordered categorical identities by
resolved domain order. It is not a numeric gradient and does not imply
magnitude.

Caller-created palettes select capacity behavior explicitly:

- A fixed palette rejects during preflight when the resolved domain requires
  more colors than supplied.
- An extendable palette preserves every supplied color as a stable prefix and
  deterministically generates additional colors. It never cycles.

Every extendable palette requires a nonempty seed prefix. Extension operates in
OKLab/OKLCH, preserves that exact prefix, derives its lightness and chroma
character from the supplied seeds, reduces chroma deterministically for sRGB
gamut fit, and chooses each next candidate by maximum minimum perceptual
distance from colors already selected. Built-in presets use their curated colors
as seeds; caller-created extendable palettes use their caller-supplied colors.
The first-party candidate order, gamut reduction, distance calculation, and
tie-breaking are one fixed implementation pinned by exact tests and are not
caller-selectable strategies. The generated prefix is independent of the final
requested count, so adding a category does not recolor existing categories.
Very large domains necessarily become less distinguishable; the renderer does
not claim unlimited color-only perceptual separation.

`CategoricalColorMap` binds exact raw categorical identities to
`OpaqueColor`. Keys are the normalized semantic strings carried by
`CategoryValues`, before display aliases or humanization. Keys are nonempty,
trimmed, control-free, and unique. Every used identity must have a mapping.
Unused mappings are allowed so one map can cover related plots, sparse facets,
and Canvas panels. Repeated caller-selected colors are allowed and never
silently replaced.

Exact maps and Plot-local palettes apply to every plot family that consumes the
same hue or positioned-series color semantics. A family-specific exception
requires a distinct semantic owner; implementation history is not sufficient.

Exact categorical maps have no anonymous slots. `AccentPalette` is a separate
nonempty, always-extendable sequence using the same deterministic extension
algorithm. Unkeyed primary colors and automatic supplemental `PointLayer` colors
resolve from the Plot theme's `MarkTheme` accent palette, not from a categorical
map. That accent sequence is stable and independent of categorical domain size;
an exact local point-layer color remains its final override.

An unkeyed primary mark consumes accent slot zero. Its first appended
`PointLayer` consumes slot one. A primary colored by a categorical scale consumes
no accent slot, so its first appended `PointLayer` consumes slot zero. Later
layers consume consecutive slots in append order. A layer with an exact color
still consumes its accent position so adding or removing that override cannot
recolor later automatic layers.

Categorical domain order is deterministic first-observed order unless an
explicit current ordering contract says otherwise. Facets resolve one domain
from the complete parent Plot before panel slicing. A Canvas shared legend
resolves one row-major domain for each raw semantic color field across every
plot with visible marks in that field, including a plot whose own legend entries
are hidden. Per-plot legends retain plot-local domains.

## Continuous Scales

Sequential scales map low-to-high numeric values through one
`SequentialGradient`. Diverging scales map values below and above one explicit
semantic center through a `DivergingGradient`; the center maps exactly to its
neutral color. Automatic diverging domains are symmetric around the center when
representable. Explicit limits remain exact and must contain the center.

Gradients contain validated ordered opaque color stops. Interpolation uses the
fixed first-party OKLab model, with deterministic conversion and sRGB gamut
handling. There is no backend-selected interpolation or callback. Constant
sequential domains use the gradient midpoint. Constant diverging domains at the
center use the neutral color.

A Plot-local continuous scale may replace the theme's sequential or diverging
gradient without replacing non-data appearance. Heatmaps, beeswarms, their
marks, gradients, and color guides consume the same resolved scale. Missing
values use the missing-mark role from `MarkTheme`, not a palette endpoint.
Color-guide text and chrome use `LegendTheme` and `TypographyTheme`; automatic
cell-label contrast uses the light/dark `TypographyTheme` roles and the
`AnnotationTheme` planning backdrop.

## Shared Legends

A shared discrete Canvas legend uses Canvas `LegendTheme`, Canvas typography,
and Canvas surface roles for its container. Combined entries retain the exact
final glyphs resolved by their owning plots, including shape, line pattern, fill,
outline, and RGBA paint. Separate sections retain the complete producer-owned
channel projections defined in spec05/spec93, including explicitly neutralized
non-target mapping properties. Canvas does not invent those projections.

Mixed explicit Plot themes are allowed. Entries with distinct semantic
identities coexist. Entries with the same typed semantic identity merge only
when their resolved labels and complete glyphs are equal. Conflicting glyphs,
including colors or alpha, reject during Canvas preflight rather than allowing
one theme or panel to win. The shared legend never reconstructs generic swatches
from the Canvas palette.

Canvas legend policy owns composition behavior:

- `PerPlot` preserves each Plot legend policy and position.
- `Hidden` suppresses all Canvas legend entries.
- `Shared` suppresses per-panel legend placement and owns the one shared
  position. A Plot with hidden legend policy contributes no entries but still
  contributes visible mark identities to global color-domain coordination. A
  Plot with automatic policy contributes its renderer-owned entries. A Plot with
  an explicit per-plot legend position rejects under `Shared` because that
  position cannot be honored.

One shared Canvas guide is either discrete or continuous; mixing the two kinds
rejects during Canvas preflight. The T6 discrete target supports typed sections
under one measured guide; continuous guides are not sectioned. A continuous
contributor carries its typed field identity, scale kind, resolved
title, gradient stops, diverging center when present, automatic-domain data
envelope or exact caller domain, and missing-value block. Contributors merge
only when field identity, scale kind, title, complete gradient, center,
missing-value block and normalized numeric presentation policy are exactly equal.
With only automatic domains, the shared domain is their finite lower/upper union.
If any contributor has an exact domain,
all exact domains must be equal and that domain must contain every automatic
envelope; otherwise preflight rejects. The shared guide generates its labels once
from that final domain and resolved Canvas typography. It never merges separately
generated per-Plot guide labels or lets one Plot's gradient/formatter win.

Combined shared composition retains one exact typed primary semantic title
identity and resolved display title across contributing primary entries.
Separate shared composition unions typed sections with exact title, label and
complete projected-glyph compatibility. Distinct sections with equal nonempty
display titles reject; sections are never identified by display strings alone.
Same-field group/hue collapses and unsupported pair factorization rejects before
fonts. See `93_T6_EXECUTION_PLAN.md` T6L for ordering and actual encoding rules.

`LegendComposition::{Combined,Separate}` defaults to Combined. Standalone/facets
use the owning Plot selection; Canvas PerPlot preserves each Plot selection;
Canvas Shared owns its selection for all contributors. Hidden entries contribute
no section/font work while visible marks retain color-domain coordination.
Shared composition does not consult child selections or turn them into conflicts;
switching back to PerPlot restores their unchanged meaning. Placement and theme
attachments do not mutate composition.

A labeled `PointLayer` owns a typed layer-entry identity. When its Plot already
has a categorical legend title, the layer entry uses that title for display but
does not become a categorical identity or affect its domain. Otherwise it uses
the standard literal `series` title identity. Separate composition places labeled
layers in a final typed `series` section. Shared compatibility follows the chosen
composition's exact section/entry rule; layer labels never become hue values.

## Scientific Numeric Presentation

The public `NumericFormat` controls numeric labels independently of Theme
appearance. Fixed fractional/significant precision, scientific/engineering notation,
decimal-power display scaling including percentage, literal affixes, explicit
negative-zero/trailing-zero handling and bounded adaptive precision are in scope.
`06_NUMERIC_PRESENTATION.md` owns construction/attachment/default/set/identity
details, and `07_LOG10_DISPLAY.md` proves the logarithmic display design. Spec05
owns their layout consequences; spec93 owns implementation timing and evidence.

Formats apply to generated numeric axes, continuous guides, existing Heatmap/
Coefficient value labels and Waterfall totals. They do not change scientific
values, projections, colors, exact tick strings, categories or annotation text.
Configured Forest Log10 ticks format the mathematical value of their selected
exponent through certified decimal rounding, not exponent-operand precision.
Shared continuous guides require identical normalized effective policies and
format one final-domain label set. Themes do not infer or override units.

## Visibility And Content

A theme may omit nonsemantic visual chrome such as grid lines, frame sides,
tick marks, panel outlines, or a legend box. It cannot remove caller-provided or
semantic content. Titles, subtitles, captions, axis labels, exact ticks, tick
labels, annotations, and requested legends remain present unless an explicit
current Plot or Canvas policy owns their absence.

Theme changes therefore cannot make caller-provided text or annotations
silently disappear. Renderer-owned layout reserves and measures every present
element using the same resolved style that final rendering uses.

## Fonts

`FontFamily` is either one trimmed, control-free named family or one closed
generic family: serif, sans-serif, or monospace. `FontStack` owns a nonempty
ordered list of those values. It is not an opaque caller-authored CSS expression.
Each `TextTheme` contains a complete stack. ThemeBuilder may replace all
typography stacks together or one text role independently.

Before text measurement and scene creation, the generation host builds one
canonical candidate list for each used `(stack,weight)` pair, in the zero-based
canonical first-use pair order defined by `05_LAYOUT_SYSTEM.md`. Text style and
stretch are fixed renderer invariants `normal`; they are not missing theme
fields. The renderer performs the bounded canonical system-font source
enumeration and reservation required by `05_LAYOUT_SYSTEM.md`, then loads each
admitted file into `fontdb 0.23.0` explicitly; it does not call the library's
unbounded `Database::load_system_fonts`. Font metadata and host generic-family
names are exactly those exposed by that resulting database before any project-
owned generic remap. A face's family occurrences are every OpenType typographic-family
name record (name ID 16) and legacy-family record (name ID 1) that locked
`ttf-parser 0.25.1` decodes to a string. Named-family matching compares the
Unicode-16.0 NFKC-casefolded caller name with every such occurrence. A generic
entry resolves exactly through `Database::family_name` and contributes every face
having a decoded ID-16 or ID-1 occurrence that NFKC-casefolds equal to that
resolved host name; an unmatched generic contributes none. No project-owned
global generic remap occurs.

Each font file identity is exactly
`(sha256_complete_file_bytes,face_index_u32)`; paths, discovery order, mtimes, and
database ids are excluded. Duplicate identities deduplicate. Candidates sort by
`(stack_entry_index,family_match_rank,style_distance,
stretch_distance,weight_distance,sha256_bytes,face_index)`. The exact metadata
domains are `style_distance = 0,1,2` for face style Normal, Oblique, Italic;
`stretch_distance = abs(face_stretch.to_number()-5)` over the OpenType width
classes `1..=9`; and
`weight_distance = abs(face_weight_u16-requested_weight_u16)`, where the closed
requested Normal, Semibold, and Bold weights are `400`, `600`, and `700`.
`family_match_rank` is zero for a named ID-16 match, one for a named ID-1 match,
and two for a generic-family contribution.
Generate every occurrence tuple first; for duplicate file identities retain the
lexicographically least complete occurrence tuple, then sort surviving identities
by that tuple. The first candidate is the primary face. Glyph fallback scans this
same list. For a base-and-combining-mark unit defined by `05_LAYOUT_SYSTEM.md`,
prefer the first face covering every visible scalar in the unit. If no one face
covers the unit, select the first covering face separately for each visible
scalar. Standalone scalars retain first-covering-face selection. No required
visible scalar may become a replacement box or disappear. Retain unit source
spans and face decisions: a character-only union cannot describe context-dependent
base-and-mark selection. Candidate ordering and the primary face do not change.
Nonpainting Unicode controls follow the explicit text semantics in
`05_LAYOUT_SYSTEM.md`; absence of their standalone cmap glyph is not a missing
visible character. This exception does not change primary-face ordering or the
base-and-mark fallback rule. General emoji-sequence or whole-word font selection
is not implied by the narrow base-and-mark preference.
The operation records the complete ordered identity list as font-environment
provenance and fails if the
list is empty or any required glyph has no face. Measurement and drawing use the
same selected face per glyph. No font is downloaded, provisioned, embedded, or
read from a network or project cache.

Font candidates are preflight font-resolution inputs, not alternative layouts.
Layout does not choose another font because it prefers that font's dimensions.
File enumeration, inspected/hashed bytes, decoded metadata/occurrences, identity work,
and glyph probes consume the separate font-preflight limits and checked
first-party admission in `05_LAYOUT_SYSTEM.md` before text measurement or backend
creation. Known counts reserve fallibly after admission; unknown-length payloads
use checked bounded growth, not an unbounded draft followed by a second reserved
copy. Pinned dependencies may supply Unicode algorithms/data; generated tables
require reproducible provenance and conformance evidence, not bespoke ownership
for its own sake.

Preserve-text SVG emits the validated ordered stack and selected line structure
while preserving paragraph bidi context. Recipient fonts and shaping can differ;
editable text does not promise exact generation-host glyph geometry. Outlined
SVG and PNG use retained generation-host glyph identities and positioning.
Text wrapping, text-derived layout geometry, outlined glyph paths, and PNG bytes
require the same installed-font environment. The remaining scene structure is
deterministic for identical inputs and options.

`TextTheme::line_height` is a finite positive binary64 multiplier of that role's
font size, not an absolute length. Layout resolves it from the multiplier's exact
binary rational value times normalized font-size quanta, rounding the resulting
nonnegative hard occupancy toward positive infinity. No percentage, CSS unit,
root-scale factor, or backend interpretation remains after this resolution.

## Contrast

Built-in presets have regression evidence for their admitted text/background
and chrome/background contrast pairs. Custom-theme contrast remains caller
owned; general theme construction does not reject a complete theme solely for
failing a universal contrast threshold.

For renderer-selected text over variable fills, `TypographyTheme` supplies
complete light and dark text roles. The renderer composites the fill over the
resolved panel surface and then the Canvas surface. If that stack remains
translucent, contrast planning composites it over the opaque backdrop stored in
`AnnotationTheme`; that backdrop affects selection only and is not painted. The
light and dark candidate text colors must be opaque. The renderer computes
relative luminance deterministically and uses the candidate with the greater
contrast ratio. Convert one `u8` sRGB channel `c` to linear binary64 through
one checked-in 256-entry table of exact binary64 bit patterns. Entries 0–10
are the nearest-even result of `c/255 / 12.92`; entries 11–255 are the
nearest-even result of `((c/255 + 0.055)/1.055)^2.4`, with each operation before
the power evaluated in the stated binary64 order. The table's normative
bits are pinned by exhaustive tests against independently computed
high-precision reference values; the renderer performs no platform-dependent
runtime `pow`. This finite-domain approach replaces the earlier prescribed
vendored CORE-MATH `cr_pow` implementation, which is unnecessary for 256
possible channel inputs. Composite source-over in linear
light by starting with the opaque planning backdrop, then applying Canvas,
panel, and variable fill from back to front. For each layer and each channel in
`r,g,b` order, calculate `one_minus_a = 1.0 - a`, `source_term = source*a`,
`destination_term = destination*one_minus_a`, then
`destination = source_term + destination_term`; every named binary64 operation
rounds to nearest-even and fused substitution is forbidden. The destination
remains opaque, so no alternate straight/premultiplied-alpha recurrence is used.
Calculate luminance as `0.2126*r + 0.7152*g + 0.0722*b` in that operation
order without fused substitution and contrast as
`(max(l1,l2)+0.05)/(min(l1,l2)+0.05)`. Greater binary64 contrast wins; exact-bit
equality selects the dark role. Selection occurs before font resolution/layout,
so the selected complete role alone supplies metrics. Transparent root output cannot guarantee contrast against the
recipient's actual unknown background and remains caller-owned.

## Validation

`ThemeBuilder::build` and component constructors validate all data-independent
invariants, including nonempty font stacks and palettes, finite positive sizes
and widths, finite nonnegative role-local lengths and dimensionless spacing
relationships, valid line heights, ordered gradient stops, opaque automatic
contrast candidates, and structurally complete components.

Plot or Canvas preflight validates data-dependent invariants after inheritance
and scale selection, including fixed-palette capacity, exact-map completeness,
continuous domains and centers, font availability, shared-legend compatibility,
layout fit, and representable geometry. Validation errors identify the owner,
required and available capacity where relevant, and missing semantic identities.
There is no cycling, substitution, generated fallback for fixed palettes, font
fallback outside the supplied stack, or alternate renderer after an error.

## Public Foundations Limit

The first public customization milestone includes the component and scale roles
named in this specification. It does not include CSS selectors, string-keyed
properties, callbacks, dynamic theme extensions, process-global defaults,
backend dash strings, font loading, data-mapped alpha, arbitrary blend modes,
animation, or one theme component per plot family. Such features require a new
canonical contract and are not reserved by placeholder API.
