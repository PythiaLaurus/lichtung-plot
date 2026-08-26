# Implementation Plan

**Status**: IN PROGRESS

## Purpose

This document records the active theme-system design and implementation
sequence. The completed API-boundary and output-contract remediation remains
available in Git history through commit `0dcff20`; it is not a second current
plan.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative for the
implemented renderer. `03_THEME_SYSTEM.md` is the approved replacement
architecture, not a second current API. Its custom themes, colors, scales, and
inheritance behavior remain non-current until Phase T6 performs the atomic public
migration. Later implementation phases do not begin merely because they are
listed here.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each admitted phase as one logical change before starting the next
phase. Split a broad phase into smaller approved phases before implementation
rather than combining unrelated migrations in one commit.

## Confirmed Direction

The following decisions are accepted starting points for the architecture
discussion:

- Publication is the replaceable library default for a standalone plot. It must
  not be forced or repeated as hidden renderer or Canvas state.
- Canvas supplies the root theme for composed output. A plot without an explicit
  theme inherits that Canvas theme; an explicitly themed plot may replace it.
- The first public customization milestone targets coherent full foundations,
  not a color-only API and not an exhaustive property surface.
- Theme values that affect text measurement, layout, legend size, or collision
  planning must resolve before preflight and remain identical during final
  rendering.

Phase T1 settled the exact public types, replacement semantics, component
boundaries, validation, and scope of local overrides in `03_THEME_SYSTEM.md`.

## Reference Philosophy

The approved architecture combines three relevant models:

- ggplot2 supplies the plotting-specific separation between non-data theme
  appearance, data-to-aesthetic scales, and exact layer aesthetics.
- Flutter supplies immutable complete theme data, organized component themes,
  and explicit parent-scope inheritance.
- Vega-Lite supplies typed component configuration and deterministic precedence
  from broad defaults to exact mark settings.

The design should not copy Flutter's widget-context lookup, reactive rebuilding,
animation, or dynamic theme extensions. It should not copy Matplotlib's mutable
process-global runtime configuration or CSS's selector-driven open cascade.

## Target Contract Rules

Phase T1 confirmed these rules for the approved target:

- A theme owns visual defaults. It does not own scientific data, statistical
  behavior, semantic labels, or output routing.
- Data palettes and scales map values to aesthetics. Exact element styles
  configure one caller-owned guide, layer, annotation, or mark.
- Visibility and placement policies remain distinct from appearance when they
  determine whether caller-provided content exists.
- Theme resolution is explicit and immutable. There is no mutable process-global
  theme, thread-local theme, environment-selected theme, or late backend lookup.
- The backend continues to receive complete concrete point, line, fill, text,
  gradient, and bracket styles rather than a public theme object.
- One resolved internal theme is shared by preflight, measurement, collision
  planning, legend construction, and final rendering.
- Public customization remains typed and validated. Do not add arbitrary CSS,
  backend dash strings, string-keyed property maps, dynamic style plugins,
  unvalidated mutable palettes, or silent fallback chains.
- Keep one current API and representation. Replaced theme APIs and private
  representations are migrated atomically without aliases or deprecations.

## Phase Map

| Phase | Objective | Status |
| --- | --- | --- |
| T1 | Approved theme architecture contract | COMPLETE |
| T2 | Complete visual-property ownership inventory | IN PROGRESS |
| T2a | Ledger protocol and renderer foundations | COMPLETE |
| T2a1 | Current theme and font foundations | COMPLETE |
| T2a2 | Private style carriers and backend protocol | COMPLETE |
| T2a3 | Serialization, geometry recording, and scene conversion | COMPLETE |
| T2a4 | Lifecycle, output, and artifact foundations | COMPLETE |
| T2b | Public controls and policies | COMPLETE |
| T2c | Layout, typography, axes, and grid | PENDING |
| T2d | Legends, Canvas composition, and decorations | PENDING |
| T2e | Ordinary plot marks and local styles | PENDING |
| T2f | Specialized marks and annotations | PENDING |
| T2g | Data scales and semantic domains | PENDING |
| T2h | Cross-source reconciliation and migration map | PENDING |
| T3 | One private resolved-theme pipeline | PENDING |
| T4 | Explicit Canvas-to-Plot theme inheritance | PENDING |
| T5a | Surfaces, typography, titles, axes, and grid | PENDING |
| T5b | Legends, facets, and Canvas composition chrome | PENDING |
| T5c | Mark defaults, annotations, and local styles | PENDING |
| T5d | Categorical and continuous data scales | PENDING |
| T6 | Public immutable theme and scale API | PENDING |
| T7 | Contract evidence, documentation, and final audit | PENDING |

## Phase T1: Approved Theme Architecture

**Status**: COMPLETE

Decision record: `03_THEME_SYSTEM.md` defines the complete immutable Theme,
construction-only builder, role-based components, explicit Canvas inheritance,
complete Plot replacement, typed font stacks, RGBA paints, opaque data-scale
colors, deterministic extendable palettes, exact categorical maps, OKLab/OKLCH
color math, transparent surfaces, content-policy boundaries, and exact
shared-legend compatibility. No production code changed in this phase.

Verification: independent architecture review found no remaining specification
contradictions, retired spec-path searches were empty, and `just verify` passed
on 2026-08-26.

Define the model before exposing or moving implementation state.

Scope:

- Add a dedicated approved theme-system target specification, keep
  `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` unambiguous about the
  implemented API, and update `99_DEFERRED_FEATURES.md` for newly admitted and
  still-deferred boundaries.
- Define the exact meanings and boundaries of theme, preset, palette, scale,
  semantic color, component style, element style, visibility policy, renderer
  invariant, and resolved theme.
- Define standalone Plot ownership, Canvas inheritance, explicit Plot
  replacement, facets, text panels, shared headers, and shared-legend behavior.
- Define the complete precedence model across library defaults, Canvas theme,
  Plot theme, plot-specific scale or component configuration, and exact element
  styles.
- Decide whether public `Theme` is a complete immutable value, how callers derive
  a modified theme from a built-in preset, and whether any sparse override type
  has a justified consumer.
- Define which visibility decisions are appearance and which remain separate
  behavior or content policies.
- Define structural validation, palette capacity, missing mappings, fallback
  prohibition, color and alpha representation, contrast ownership, font policy,
  and deterministic SVG/PNG requirements.
- Define a role-based component taxonomy that avoids one independent theme block
  per plot family unless the component is genuinely distinct.
- Define the public-foundations milestone and explicitly defer properties that
  are not yet coherent enough to expose.

Acceptance:

- The approved target specification describes one unambiguous ownership and
  precedence model for standalone plots, Canvas, facets, and shared legends
  without presenting target types as current public API.
- Theme, data scale, exact style, policy, and renderer invariant are distinct
  concepts with no overlapping owner for the same property.
- The contract states when validation occurs and guarantees that preflight and
  final rendering use identical resolved values.
- Every remaining design choice is either decided or named as deferred without
  adding placeholder API.
- No production code changes in this phase.

## Phase T2: Visual-Property Ownership Inventory

**Status**: COMPLETE

Account for every current visual value before reorganizing implementation.

T2 is a documentation and classification phase. It changes no production code,
public API, snapshot, or intended rendering. Each admitted subphase is one
reviewable documentation commit. T3 does not begin until T2h closes the complete
inventory and reconciles the later migration scopes.

### Deliverable

T2a creates `04_VISUAL_PROPERTY_OWNERSHIP.md` as the dedicated migration ledger.
That document distinguishes current source from approved target ownership and
does not make the target API current. Keeping the row-level inventory separate
prevents this implementation plan from becoming an unreadable source listing
while retaining the ledger under the canonical `specs/` source of truth.

The ledger is append-only during T2 except for corrections and reconciliation.
T2h may merge records only when one declaration supplies one value to the same
semantic role and target owner. Equal literals with independent current sources
remain separate records and cross-reference one duplicate cluster.

Each property key is a unique descriptive semantic path with a source-context
suffix when multiple current sources serve the same role. Keys do not encode a
schema or protocol version. Each completed record contains every field below;
`unknown`, `other`, and unresolved `TBD` values are prohibited at subphase
completion.

Target fields recorded by T2a through T2g are concrete proposals, not unresolved
placeholders. T2h verifies or corrects them and marks their relationships final.
Earlier records may therefore propose one shared target owner while retaining
separate current-source records until T2h proves that the semantics agree.

| Field | Required content |
| --- | --- |
| Property | Unique descriptive key for one current declaration, fallback, literal, or behavior branch |
| Current source and value | Exact file plus symbol or expression context, current type, default, and unit or color representation |
| Current consumers | Every production measurement, planning, legend, scene, or backend path that reads or duplicates the value |
| Current behavior | Current resolution scope, validation or lack of validation, failure behavior, and fallback behavior |
| Current layout path | Current `none`, `measure`, `reserve`, `collision`, or `geometry` effects, traced through final drawing when applicable |
| Semantic role | What the value means visually, independent of its current file or plot-family placement |
| Target classification and owner | Exactly one closed class plus one approved component or scale, retained current style or policy field, unnamed replacement style or policy role, or descriptive private invariant responsibility |
| Target resolution | Canvas root, resolved Plot, facet inherited from Plot, plot-local scale, exact element, explicit policy owner, or final private draw boundary |
| Target validation | Construction, Plot preflight, Canvas preflight, geometry, or backend-protocol validation, including bounds and failure behavior |
| Public surface | T6 component field, retained local control, replaced local control, or private-only behavior |
| Relationships | Duplicate cluster, split parent or child, current-to-target counterpart, and measurement-to-draw counterpart keys, or explicit `none` |
| Audit evidence | Owning source-manifest entry and search-match disposition; exclusions name the data, test, or non-visual reason |
| Disposition | `retain`, `move`, `connect`, `replace`, `deduplicate`, or `remove`, with the owning migration phase |
| Visual evidence | No output change expected, intentional correction, or intentional redesign, plus the focused assertion or snapshot family required |

The closed classification set is:

| Class | Meaning |
| --- | --- |
| Theme component | Non-data appearance owned by `SurfaceTheme`, `TypographyTheme`, `AxisTheme`, `LegendTheme`, `CompositionTheme`, `MarkTheme`, or `AnnotationTheme` |
| Theme data scale | Default qualitative, sequential, or diverging mapping owned by `ColorScales` |
| Plot-local data scale | One Plot's replacement categorical or continuous data mapping |
| Exact element style | Final caller-owned appearance for one guide, layer, annotation, or mark |
| Explicit policy | Semantic presence, placement, sharing, or other non-appearance configuration |
| Renderer invariant | Fixed private clipping, ordering, anchoring, geometry, contrast-selection, or backend behavior |

One property cannot occupy multiple classes. When one current field combines
multiple meanings, the ledger creates one record per meaning, links them as a
split, and assigns the current field itself a `replace` disposition. A target
theme component may supply an appearance default consumed by an exact style, and
a target theme data scale may supply the default replaced by a Plot-local data
scale. Those defaults and final overrides remain independent records with one
owner each.

Target-owner names use only the component and scale types approved in
`03_THEME_SYSTEM.md`, a retained current public style or policy type plus field,
an unnamed semantic role classified as an exact element style or explicit
policy, or a descriptive private semantic responsibility. T2 does not invent
names for future public exact-style or policy replacements. When replacement is
required but T1 did not approve a public type name, the ledger records that
unnamed semantic role and leaves naming to its admitted implementation phase.

### Coverage Rules

The inventory covers production `src/` declarations, literals, defaults,
fallbacks, public setters, validation branches, measurement inputs, generated
legend styles, scene styles, and backend serialization that can change visible
output or occupied space. Tests, examples, and snapshots are evidence consumers,
not independent property sources. Scientific data values, statistical
coefficients, and geometry derived solely from caller data are excluded unless a
fixed rendering rule constrains their visible representation; that fixed rule is
then recorded as a renderer invariant.

Rows are source-based rather than estimate-based. The pre-audit planning ranges
were 70-85 foundation records, 58-68 public-control records, 105-120 layout and
chrome records, and about 106 mark and annotation records. These ranges overlap,
must not be summed, and are not completion thresholds. The current source
reconciliation supersedes that range with the corrected 222-record T2a split
below; later workstreams update their scopes the same way when source evidence
differs.

Every subphase records its complete source manifest in the ledger. A production
declaration, literal, default, or behavior branch is owned by exactly one
subphase, even when its file contains mixed responsibilities. Mixed files use
symbol or expression-context manifest entries instead of assigning the whole
file. Another subphase may trace that source as a consumer but may not create a
second source record for it.

For each manifest, review searches cover color constructors and tuples, font
values, sizes, widths, opacity and alpha, dashes, radii, padding, gaps, spacing,
visibility branches, palette indexing, gradients, contrast decisions, and hidden
preset construction. Every match appears either in one property record or in the
subphase's rejected-match table with its data, test, or non-visual reason. T2h
owns all residual production paths not explicitly allocated by T2a through T2g
and fails completion if a residual match is a visual source rather than an
explained exclusion.

### Subphase T2a: Ledger Protocol And Renderer Foundations

**Status**: COMPLETE

Create the ledger and inventory foundational private carriers and lifecycle
sources. Source reconciliation establishes 222 records: 65 theme and font
records, 52 private style-carrier and backend-protocol records, 45 serialization,
geometry-recording, and scene records, and 60 lifecycle and output records. T2a1
through T2a4 complete all 222 records. These counts supersede the earlier
overlapping range and the later 202/40 estimate for this workstream.

T2a owns only foundational carrier, preset construction, draw-boundary,
serialization, geometry-recording, scene, and lifecycle entries in mixed files.
Later subphases own facet, legend, mark, and scale values by exact symbol or
expression context.

T2a must identify every `ThemePalette` responsibility, every concrete backend
style field and default, RGB-versus-alpha representation, root-surface behavior,
repeated `Plot::palette()` or preset reconstruction, and every lifecycle point
where appearance can diverge between validation, preflight, measurement, scene
creation, and drawing. Test-only modules under these paths are evidence rather
than ledger sources.

T2a is complete when T2a1 through T2a4 are complete, the ledger schema is
instantiated, every production source in their disjoint manifests is accounted
for, current carrier fields have target owners, and hidden Publication or backend
fallback paths are explicitly linked to T3, T4, T5a, or T5d remediation.

#### Subphase T2a1: Current Theme And Font Foundations

**Status**: COMPLETE

Decision record: `04_VISUAL_PROPERTY_OWNERSHIP.md` instantiates the canonical
ledger protocol and records all 65 current theme and font foundation properties.
The keyed current-state, target-state, and migration/evidence tables preserve
every required schema field, exact preset values, split responsibilities,
dormant and miswired fields, hidden preset construction, font fallbacks, and one
closed T1 target classification and owner per record. Independent ownership
review split five multi-owner rows into 13 exact-owner rows, increasing the
initial 57-record count by eight without adding a production source. T2a and T2
remain in progress while the later admitted subphases are incomplete.

Verification: the T2a1 source manifest and rejected-match table account for the
three admitted production sources; tests are evidence only. Independent review
found no remaining ownership or schema issue. Tracked-diff checks and `just
verify` passed on 2026-08-26. No production code, test, or snapshot changed.

Create `04_VISUAL_PROPERTY_OWNERSHIP.md` with the required schema, source
manifest, rejected-match table, and every current property record from
`theme.rs`, `theme/palette.rs`, and `font.rs`. Cover preset routing, every
`ThemePalette` responsibility, dormant fields, split semantic roles, RGB
serialization, global font constants, and host-font discovery and fallback.

T2a1 is complete when all 65 reconciled records are either present or
explicitly corrected by a reviewed source-manifest reconciliation, and every
theme or font match has one record or rejection reason.

#### Subphase T2a2: Private Style Carriers And Backend Protocol

**Status**: COMPLETE

Decision record: `04_VISUAL_PROPERTY_OWNERSHIP.md` appends 52 keyed T2a2
records for `render/backend.rs`. Source reconciliation confirmed the initial
count without a net split correction: the combined `BracketStyle::color`
declaration is represented by separate private stroke-paint and text-paint
transport records linked to their T2a1 semantic owners. The existing coordinate
protocol record also owns integer text positions, so the corrected source
coverage does not add a record. T2a1 and T2a2 therefore complete exactly 117
T2a records; T2a3 brought the completed count to 162 under the now-superseded
202-record estimate, and T2a4 reconciliation corrects the current aggregate to
222. T2a is complete; T2 remains in progress while T2b is in progress and T2c
through T2h are pending.

Verification: source reconciliation and independent ownership review corrected
private carrier ownership, the dormant rounded-rectangle fallback, degenerate
primitive validation, gradient-stop replacement, and the rejected-match table.
The keyed current-state, target-state, and migration/evidence table counts
account for the admitted production source; independent re-review found no
remaining ownership or schema issue. Tracked-diff checks and `just verify`
passed on 2026-08-26. No production code, test, or snapshot changed.

Inventory the foundational declarations, defaults, mutators, validation, and
protocol fallbacks in `render/backend.rs`. Cover point, line, fill, gradient,
text, bracket, layer, clip, and draw-scope carriers without taking ownership of
the plot, layout, legend, annotation, or scale values passed through them.

T2a2 is complete when all 52 initially identified records are either present or
explicitly corrected, each combined carrier has split target responsibilities,
and every backend-protocol fallback has a migration disposition and evidence.

#### Subphase T2a3: Serialization, Geometry Recording, And Scene Conversion

**Status**: COMPLETE

Decision record: `04_VISUAL_PROPERTY_OWNERSHIP.md` appends 45 keyed T2a3
records: 20 SVG records including the PNG RGBA8 paint/alpha boundary, 15
Geometry records, and 10 Scene records. The initial 36-record plan was
corrected by disjoint source, target owner, and migration-phase rules: SVG
changed from 22 to 20 after PNG sizing/output ownership moved to T2a4,
Geometry changed from 11 to 15 after lifecycle and bracket path/label
responsibilities were separated, and Scene
changed from 3 to 10 after terminal routes, positive checks, and four independent
limit declarations were separated. The net correction is nine records without a
new production source. T2a1 through T2a3 complete 162 records. Their
then-current 202-record aggregate left an estimated 40 for T2a4; the T2a4
source adjudication supersedes those planning figures with 60 and 222.

Independent review retained that key set while correcting post-format geometry
collapse, stale SVG output reuse, explicit point/polygon joins, paint-aware
collision bounds, dormant Geometry lifecycle wiring, PNG-only Scene maximum
guards, canonical-scene positivity, exact text/bracket whitespace, and the
separation between T2a1 token spelling and T2a3 paint-property alpha routing.

Verification: source reconciliation, identical 45-key current/target/migration
table counts and order, and aggregate arithmetic checks passed. Paint-aware
bounds are phased explicitly: current geometry, opacity, and font corrections
land in T3/T5a/T5c, while complete RGBA carrier connections close in T6. Final
independent ownership review found no remaining semantic or schema issue.
Tracked-diff checks and `just verify` passed on 2026-08-26. No production code,
test, snapshot, or other specification changed.

Inventory `render.rs`, `render/svg_backend.rs`, `render/geometry.rs`, and
`scene.rs`. Cover SVG and PNG paint, text, surface, precision, clipping, layer,
gradient, and alpha behavior; geometry recording and collision bounds; immutable
scene reuse and conversion limits. Values merely supplied by semantic owners are
consumer references, not duplicate source records.

T2a3 is complete with all 45 reconciled records present in identical keyed
tables, root-surface and one private final RGBA-to-SVG serializer responsibility
assigned, concrete lifecycle and independent Scene limits recorded, PNG
sizing/output contexts deferred to T2a4, and bracket/text measurement-to-draw
discrepancies assigned to their owning migration phases.

#### Subphase T2a4: Lifecycle, Output, And Artifact Foundations

**Status**: COMPLETE

Inventory `orchestrator.rs`, `orchestrator/lifecycle.rs`,
`orchestrator/backend.rs`, `orchestrator/output.rs`,
`orchestrator/artifacts.rs`, `output.rs`, targeted PNG contexts in
`render/svg_backend.rs`, and PNG option declarations in `spec/output.rs`. Cover
appearance reconstruction, preflight-to-render reuse, deferred and null backend
parity, output dimensions, canonical scene routing, format-local font timing,
PNG options and sizing, and exact artifact publication. Exclude facet, legend,
mark, scale, Canvas, and public re-export sources owned elsewhere.

Decision record: `04_VISUAL_PROPERTY_OWNERSHIP.md` appends
60 keyed records: 13 Plot lifecycle and sequencing records, 10 Deferred/null
backend records, 15 output-routing and publication-timing records, 7 artifact
records, and 15 PNG option/sizing/encoding records. The source-based correction
from the superseded 40-record estimate adds no production scope: it separates
independent terminal selections, plan/render phases, Null terminals, conversion
timing, artifact failure paths, and the five T2a4-owned public PNG declarations.
Independent findings corrected source boundaries and current behavior without
changing the key count or order.

T2a4 is complete: final independent rereview confirmed all 60 records occur once
and in identical order in every keyed table, every `Plot::palette()`
reconstruction and migration-risk divergence point is traced, output policies
are separated from renderer invariants, and Windows publication targets true
atomic replacement. Tracked-diff checks and `just verify` passed on 2026-08-26.
No production code, test, snapshot, or other specification changed.

### Subphase T2b: Public Controls And Policies

**Status**: IN PROGRESS

Decision record: source reconciliation admits exactly 261 canonical
T2b records attributed as 32 `spec.rs` public policies, 12 bar/error policies, 5
categorical ordering/scale policies, 17 estimate/local-style policies, 2 facet
policies, 16 supplemental-layer policies, 6 inferred-label policies, 39
caller-geometry representation records, 23 independent capability matrices, 31
builder-only controls, 28 Canvas controls, 42 independent configuration
validation branches, and 8 independent supplemental-layer gates. These are
source-based counts, not a completion estimate. Together with the 222 completed
T2a records, the ledger inventories 483 records through completed T2b. T2c
through T2h remain pending.

Verification: independent source, ownership, schema, count, class, and table-order
review corrected default-versus-override ownership, missing Canvas and Matrix
invariants, capability counts, source dispositions, and exact current behavior.
Final rereview found no remaining issue. Tracked-diff checks and `just verify`
passed on 2026-08-26. No production code, test, snapshot, generated output, or
other specification changed.

The reconciled count supersedes the overlapping 133-record public-declaration,
63-record builder, 33-record Canvas, and 55-record orchestrator audit candidate
sets. Those candidate counts must not be summed: declaration, storage, default,
setter, accessor, and an exact matching validator share one record only when
they have one semantic role, one closed target owner and class, and one owning
migration phase. Independent paired fields, capability matrices, precedence
branches, and failure policies remain separate. There is no final-T2 estimate;
T2c through T2h remain source-driven.

Inventory the public configuration declarations and their builder access paths in
`spec.rs`, `spec/`, `layers.rs`, `builder.rs`, `builder/`, `canvas.rs`, `kind.rs`,
`labels.rs`, `geometry.rs`, and public re-exports in `lib.rs`, including every
family gate, default, validation rule, and conversion into private draw styles.
T2b owns public declarations, setters, and family gates in these mixed files;
layout, composition, mark, and scale implementation branches are consumer paths
owned by their semantic subphases. In `orchestrator/layers.rs`, T2b owns only the
family-support and presence-policy validation branches; T2c and T2e own the
tick-layout and guide-style branches respectively.

T2b expressly excludes the exact `src/spec/output.rs:3-52` PNG declarations,
which are T2a4-owned. T2b owns only their public type re-export surface at
`src/spec.rs:26` and `src/lib.rs:75`; `src/spec.rs:25` is crate-private transport
of the T2a4-owned `SVG_CSS_DPI` constant and is not a T2b public source.

In `orchestrator/validation.rs`, T2b owns every non-data-color family-support,
category-order policy/family gate, content-presence, annotation-compatibility,
marker, connector, and policy-validation branch. T2g owns categorical or
continuous data-to-color mapping, domain, capacity, and color-channel validation,
plus data-dependent categorical identity, ordering, and exact-order completeness
branches in that file; T2c through T2f trace those branches as consumers where
applicable.

At minimum, trace `CategoricalColors`, `GuideStyle`, `PointLayer`,
`BarPointLayer`, `InteractionStyle`, `SignificanceMarkStyle`,
`DumbbellEndpoints`, `HeatmapScale`, legend visibility and placement, point
outlines, sign coloring, connector and marker visibility, bar spacing, category
label sharing, exact ticks, title and label content, and Canvas composition
policies. Each field must be separated into appearance, data mapping, exact
element style, explicit policy, or invariant without using its current public
container as evidence of ownership.

T2b is complete when every current public visual or content control has one
target disposition, retained policies are distinguished from replaced style and
scale types, current family gates are accounted for, and no target T6 field
exists solely to preserve an accidental current grouping.

### Subphase T2c: Layout, Typography, Axes, And Grid

**Status**: PENDING

Inventory layout and drawing sources for canvas and panel surfaces, typography,
panel geometry, axes, ticks, grid, frame sides, projection, ranges, and labels in
`layout.rs`, `layout/text.rs`, `layout/panel.rs`, and `layout/panel/`. Include
related source entries in `canvas.rs`, `decorations/title_block.rs`, and
orchestrator planning only when they own one of these roles. T2c also owns
`MAX_EXPLICIT_TICKS`, tick projection, tick validation, collision calls, and tick
replacement in `orchestrator/layers.rs`; T2d owns composition and legend
branches.

Every `measure`, `reserve`, `collision`, or `geometry` row must trace the value
from its current source through the calculation and its final drawing consumer.
Paint-only properties must be marked `none` rather than omitted. Hidden
`Theme::Publication` construction, fixed whites and grays, and duplicated text or
axis metrics receive independent rows.

T2c is complete when every layout-affecting style has an end-to-end trace,
surface and typography measurement agree with their current drawing consumers,
axis content policy is separated from appearance, and each expected corrective
snapshot is named without changing it.

### Subphase T2d: Legends, Canvas Composition, And Decorations

**Status**: PENDING

Inventory composition and explanatory chrome in `canvas.rs`, `layout/canvas.rs`,
`decorations.rs`, `decorations/`, `orchestrator/facets.rs`,
`orchestrator/facets/planning.rs`, `orchestrator/facets/legend.rs`, and
`orchestrator/facets/rendering.rs`. T2d owns Canvas title and text-panel chrome,
facets, shared headers, row titles, legend construction, legend glyph generation,
reservations, collision planning, and composition-specific fallbacks. Axis,
surface, and typography sources already owned by T2c remain consumer references.

T2d is complete when Canvas-owned and Plot-owned chrome are separated, legend
policy is separated from legend appearance, every generated legend glyph traces
to its exact mark-style source, shared and per-plot measurements trace to drawing,
and each expected corrective snapshot is named without changing it.

### Subphase T2e: Ordinary Plot Marks And Local Styles

**Status**: PENDING

Inventory renderer-owned visual sources in `plots/bar.rs`, `plots/bar_data.rs`,
`plots/boxplot.rs`, `plots/density.rs`, `plots/distribution_sample.rs`,
`plots/horizontal_bar.rs`, `plots/line.rs`, `plots/regression.rs`,
`plots/scatter.rs`, `plots/violin.rs`, and `plots/violin/precomputed.rs`. Cover
ordinary point, line, fill, interval, ribbon, uncertainty, reference, outline,
raw-overlay, and supplemental-layer roles. Public exact-style declarations remain
owned by T2b; T2e owns renderer defaults, conversions, and final use. T2e also
owns guide projection, exact guide-style conversion, and guide drawing in
`orchestrator/layers.rs`.

T2e is complete when every listed renderer source is accounted for, shared mark
roles are proposed without merging independent records, local-style precedence
is traced to final draw styles and legend glyphs, and specialized or scale
responsibilities are explicitly linked to T2f or T2g.

### Subphase T2f: Specialized Marks And Annotations

**Status**: PENDING

Inventory renderer-owned visual sources in `plots/beeswarm.rs`,
`plots/coefficient.rs`, `plots/dumbbell.rs`, `plots/forest.rs`,
`plots/heatmap.rs`, `plots/histogram.rs`, `plots/interaction.rs`,
`plots/interval_estimate.rs`, `plots/marginal.rs`, `plots/matrix.rs`,
`plots/point_interval.rs`, `plots/qq.rs`, `plots/waterfall.rs`, and
`significance.rs`. Include panel-text, significance, value-label, missing-mark,
connector, endpoint, color-guide, and specialized geometry roles. Test modules
under these paths remain evidence rather than property sources.

Specialized geometry remains private only when the value is required for valid
or intelligible rendering rather than being an appearance default. Scale-domain
and mapping sources remain owned by T2g even when a listed renderer consumes
them. In particular, T2g owns each complete heatmap or beeswarm color-mapping
expression together with its embedded fixed gradient literals; T2f records those
expressions only as consumers of specialized mark and guide geometry.

T2f is complete when every listed renderer and annotation source is accounted
for, every private geometry constant has an invariant justification or a target
appearance owner, and each final annotation or specialized glyph traces to its
current style and evidence consumer.

### Subphase T2g: Data Scales And Semantic Domains

**Status**: PENDING

Inventory current categorical identity order, domain construction, palette
selection, continuous-color routing, and facet-wide coordination in the relevant
symbols of `values.rs`, `data.rs`, `input.rs`, `builder/accessors.rs`,
`builder/facets.rs`, `orchestrator/facets/data.rs`,
`orchestrator/validation.rs`, and scale-consumer branches in plot renderers. T2g
owns declarations and behavior branches that construct, validate, or coordinate
semantic data-to-color mappings. Renderer-local paint literals remain records in
T2e or T2f, except that an embedded gradient literal and its inseparable mapping
expression are one T2g-owned source.

Cover first-observed category identity, display-label separation, exact keyed
mapping, ordered palette mapping, modulo cycling, accent-slot consumption,
facet and Canvas domain coordination, heatmap and beeswarm gradients, missing
values, continuous domains and centers, interpolation, and color-guide agreement.

T2g is complete when every current data-color mapping has one theme-scale,
Plot-local-scale, exact-map, or invariant owner; all domain-order sources are
recorded; palette and gradient sources trace to marks and guides; and no
non-data foreground, surface, grid, annotation, or missing-mark color remains
misclassified as a data-scale slot.

### Subphase T2h: Cross-Source Reconciliation And Migration Map

**Status**: PENDING

Re-run repository-wide coverage searches, including `input.rs`, `data.rs`,
`values.rs`, `stats.rs`, `stats/`, `error.rs`, and every other residual production
path. Reconcile duplicate clusters and cross-workstream consumers, and close
every split or provisional cross-reference. Review public defaults against
private fallbacks, measurement against drawing, mark styles against legend
glyphs, data mappings against color guides, and SVG paint serialization against
scene alpha.

Equal constants in different plot families remain separate source records until
T2h proves one shared semantic role. T2h records that proof in relationship
fields before marking a shared target owner final; earlier subphases assign a
concrete proposed owner and relationship but do not merge the source records.

T2h must decide the disposition and migration phase for known defects: dormant
background and general line-width fields, frame-side and axis-label visibility,
backend-owned white surfaces, hardcoded title and legend grays, hidden
Publication construction, repeated palette reconstruction, fixed heatmap and
beeswarm gradients, implicit categorical cycling, and incomplete opacity
carriers. A general field is split or removed when its consumers have different
semantic owners; it is not connected merely because it already exists.

T2h updates the T3 through T7 scopes and acceptance gates when the completed
ledger reveals a different owner or required migration order. It also records a
focused evidence map for every intentional visual correction so structural
refactors cannot absorb snapshot changes without review.

T2h is complete only when every T2a-through-T2g source manifest is disjoint,
their union plus explained residual exclusions covers production `src/`, every
relationship is closed, and every property has one final target owner and
migration phase.

Scope:

- Inventory every `ThemePalette` field, hardcoded renderer color, font, size,
  width, opacity, dash, radius, spacing, visibility switch, and contrast rule.
- Inventory current public local controls including `CategoricalColors`,
  `GuideStyle`, `PointLayer`, `BarPointLayer`, `InteractionStyle`,
  `SignificanceMarkStyle`, `DumbbellEndpoints`, `HeatmapScale`, legend policies,
  point-outline policy, and sign coloring.
- Record each property's current source, consumers, semantic role, proposed
  owner, inheritance scope, validation, whether it affects layout, and intended
  public override surface.
- Classify data-aesthetic mappings separately from non-data chrome and exact
  element styling.
- Identify duplicated or inconsistent values across plot families, facets,
  legends, annotations, Canvas chrome, and output backends.
- Decide whether each dormant or incorrectly wired field is removed, connected,
  or replaced. Known examples include background, general line width, frame-side
  visibility, and axis-label visibility.
- Identify every intentional visual correction that requires separate snapshot
  evidence rather than hiding it inside structural refactoring.

Acceptance:

- Every visual constant and existing style API has exactly one proposed owner.
- Every layout-affecting style property is marked and traced into measurement.
- No hardcoded renderer value remains unclassified.
- Known wiring defects have explicit remediation phases and expected visual
  impact.
- The inventory agrees with the Phase T1 taxonomy and refines the later phase
  scopes before code migration begins.
- `04_VISUAL_PROPERTY_OWNERSHIP.md` contains no unresolved records, duplicate
  property keys, unassigned source matches, or unapproved future public names.
  Theme component and scale owners use the Phase T1 taxonomy; retained current
  styles and policies name their current type and field; unnamed replacement
  styles and policies name only their approved semantic role; private invariants
  name a descriptive semantic responsibility.
- No production code, public API, snapshots, or intended rendering changed in
  T2.
- `just verify` passes after every admitted T2 subphase.

## Phase T3: Private Resolved-Theme Pipeline

**Status**: PENDING

Create one internal path for built-in appearance without changing the public API
or intended rendered output.

Scope:

- Replace the current ad hoc `ThemePalette` flow with one complete private
  resolved-theme representation aligned with the approved component taxonomy.
- Make every built-in preset produce a complete validated resolved theme.
- Resolve the active theme once per rendering lifecycle and pass the same value
  through layout, preflight, geometry discovery, legend planning, and final
  rendering.
- Remove repeated owned palette construction and renderer calls back into
  `Plot::palette()`.
- Keep backend traits theme-agnostic and continue passing concrete styles at draw
  boundaries.
- Preserve current intended built-in visuals and snapshots. Handle known visual
  defects only in separately identified changes with explicit evidence.

Acceptance:

- One resolved theme instance governs both preflight and final rendering.
- No renderer independently reconstructs a built-in palette or hidden default.
- Backends remain independent of public theme types.
- Focused lifecycle, layout, theme, and visual-regression tests pass without
  unintended snapshot changes.
- `just verify` passes.

## Phase T4: Canvas Theme Inheritance

**Status**: PENDING

Make composition ownership explicit and consistent.

Scope:

- Give Canvas one replaceable root theme whose default is the same library
  Publication theme used by standalone plots.
- Represent the absence of an explicit Plot theme so a composed plot can inherit
  the Canvas theme without guessing whether Publication was caller-selected.
- Make an explicit Plot theme replace the inherited Canvas theme for that plot.
- Apply the resolved Canvas theme consistently to canvas title blocks, text
  panels, shared headers, row titles, spacing, and other Canvas-owned chrome.
- Make header and text measurement use the same resolved values as rendering.
- Define and enforce exact shared-legend compatibility when panels intentionally
  use different themes.
- Preserve plot erasure and ownership boundaries without global or thread-local
  theme state.

Acceptance:

- Standalone and composed defaults are visually identical when no explicit theme
  is supplied.
- Canvas-owned chrome never falls back to hidden `Theme::Publication` calls.
- Unthemed plots inherit; explicitly themed plots replace; neither behavior is
  sticky outside its owner.
- Measurement and rendering agree for headers, titles, text panels, facets, and
  shared legends.
- Focused standalone, Canvas, facet, and shared-legend tests pass.
- `just verify` passes.

## Phase T5a: Surfaces, Typography, Titles, Axes, And Grid

**Status**: PENDING

Move foundational non-data appearance into the resolved theme.

Scope:

- Centralize canvas and panel surfaces, foreground and muted text roles,
  typography roles, title/subtitle/caption styles, axis lines, ticks, labels,
  frame sides, and grid appearance.
- Remove backend-owned background color and title-block hardcoded grays.
- Separate title or label content presence from theme appearance.
- Wire or remove dormant frame, label, background, and line properties according
  to the approved inventory.
- Ensure every font and spacing value used for measurement is the value used for
  drawing.

Acceptance:

- These components contain no unclassified visual constants.
- Theme changes cannot silently remove caller-provided semantic text.
- Layout and paint use identical typography and spacing.
- Built-in preset changes, if any, have reviewed snapshot evidence.
- Focused layout, axes, typography, output, and visual-regression tests pass.
- `just verify` passes.

## Phase T5b: Legends, Facets, And Canvas Composition Chrome

**Status**: PENDING

Give composed and explanatory chrome coherent theme ownership.

Scope:

- Centralize legend container, title, label, key, outline, opacity, padding, and
  spacing styles while keeping visibility and placement in legend policy.
- Apply role-based typography and surfaces to facet strips, panel headers, shared
  headers, row titles, text panels, and shared legends.
- Preserve exact legend glyph reproduction from resolved mark styles.
- Keep collision planning, outside reservation, and shared-legend compatibility
  synchronized with the resolved theme.
- Remove implicit Publication styling and duplicated fixed whites and grays from
  composition code.

Acceptance:

- Legend and composition measurement consume the exact final styles.
- Shared and per-plot legends use one component contract while retaining exact
  mark glyphs.
- Mixed explicit Plot themes have deterministic documented behavior.
- Focused legend, facet, Canvas, collision, and visual-regression tests pass.
- `just verify` passes.

## Phase T5c: Mark Defaults, Annotations, And Local Styles

**Status**: PENDING

Unify common mark roles without erasing typed local control.

Scope:

- Define common point, line, fill, interval, ribbon, uncertainty, reference,
  outline, and missing-mark defaults by semantic role rather than plot family.
- Classify family-specific constants as shared theme defaults, justified
  specialized component defaults, exact caller styles, or renderer invariants.
- Centralize panel text, significance, value annotation, missing-value, raw
  overlay, and supplemental-layer appearance.
- Integrate existing typed local APIs into the approved precedence model rather
  than adding parallel replacements.
- Preserve exact legend glyph agreement with the final resolved mark styles.

Acceptance:

- Ordinary renderers no longer duplicate unclassified widths, radii, opacities,
  outlines, fills, and neutral colors.
- Exact local styles override only their documented properties and scope.
- Renderer invariants are explicit and cannot be changed indirectly through a
  theme field.
- Focused renderer-family, layer, annotation, legend, and snapshot tests pass.
- `just verify` passes.

## Phase T5d: Categorical And Continuous Data Scales

**Status**: PENDING

Separate data-to-color mapping from non-data theme appearance.

Scope:

- Define typed qualitative, sequential, and diverging palette or scale values,
  including neutral and missing-value behavior where applicable.
- Make built-in themes supply default data scales without conflating those
  scales with foreground, surface, grid, and annotation colors.
- Define exact keyed categorical mappings, ordered palette mappings, continuous
  interpolation, domain ownership, capacity validation, and missing mappings.
- Replace hardcoded heatmap and beeswarm gradients with resolved scale values.
- Generalize exact categorical color control to every plot family that consumes
  the same semantic mapping, or document why a family has a distinct owner.
- Coordinate facets and Canvas global hue order without silently changing exact
  caller mappings.

Acceptance:

- Theme appearance and data scales are distinct types and resolution steps.
- Custom mappings never silently cycle, substitute, or fall back contrary to the
  Phase T1 contract.
- Legends, marks, gradients, colorbars, facets, and Canvas share the same
  resolved mappings.
- Focused categorical, continuous-scale, heatmap, beeswarm, facet, Canvas, and
  legend tests pass.
- `just verify` passes.

## Phase T6: Public Immutable Theme And Scale API

**Status**: PENDING

Expose the complete foundations only after every admitted consumer uses the
private resolved model.

Scope:

- Replace the closed public theme selector with the approved immutable complete
  `Theme` API and built-in constructors or presets.
- Expose only validated component theme, color, typography, and scale types
  required by the full-foundations milestone.
- Add or migrate explicit `Canvas::theme(...)` and `Plot::theme(...)` ownership
  without global state or compatibility aliases.
- Provide a typed way to derive a modified complete theme from a built-in base.
- Migrate current local style and categorical color callers to the one approved
  precedence model; delete replaced public and private representations.
- Keep backend styles private and avoid exposing renderer implementation details.
- Atomically update `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` to the
  implemented theme contract, mark `03_THEME_SYSTEM.md` current, and reconcile
  `99_DEFERRED_FEATURES.md`; no target/current specification interval remains.
- Update examples and user documentation with standalone, Canvas inheritance,
  Plot replacement, palette, and exact local-style examples.

Acceptance:

- Callers can construct one complete custom theme covering the admitted full
  foundations without editing private renderer state.
- Built-in Publication, Nature, Science, and CHI behavior remains available
  through the single current API.
- The API has no ambiguous global default setter, generic property map, detached
  mutable palette, or alternate compatibility identity.
- Public constructors validate every structural invariant before rendering.
- Compile-pass and compile-fail evidence enforces the intended visibility and
  ownership boundaries.
- All canonical specifications describe the implemented replacement API before
  the phase is complete.
- `just verify` passes.

## Phase T7: Contract Evidence And Final Audit

**Status**: PENDING

Close the phase with exact behavioral evidence and no residual old model.

Scope:

- Add precedence tests for library default, Canvas inheritance, explicit Plot
  replacement, plot scale configuration, and exact element styles.
- Prove one resolved theme is used by preflight and final rendering.
- Cover shared legends, facets, text panels, mixed explicit Plot themes, palette
  capacity, missing mappings, fonts, contrast behavior, SVG, and PNG.
- Retain reviewed deterministic snapshots for every built-in preset and selected
  custom themes.
- Search for retired theme APIs, repeated hidden Publication defaults, dormant
  theme fields, unclassified hardcoded style values, and duplicate scale logic.
- Audit all specifications, README, guides, examples, and rustdoc for residual
  drift from the already-current implemented contract.

Acceptance:

- Phases T1 through T7 and all admitted T5 subphases are complete with commit and
  verification evidence.
- Every visual property has one implemented owner matching the approved
  inventory.
- No mutable global theme, hidden Canvas default, late theme lookup, compatibility
  alias, or unclassified renderer style constant remains.
- Snapshot changes are intentional, reviewed, and committed with their owning
  behavior change.
- `just verify` passes from the completed worktree.

## T1 Decisions

- `Theme` is one complete immutable value. Built-ins are constructors, and a
  construction-only builder produces custom complete themes.
- Canvas owns the root theme. Unthemed plots inherit it; explicit Plot themes
  replace it completely; facets add no scope.
- Components are surfaces, typography, axes, legend, composition, marks,
  annotations, and color scales. Plot-family theme blocks are not admitted.
- `OpaqueColor` owns RGB data-scale color. `Color` owns final RGBA paint with
  `u8` alpha. Alpha replacement is exact and never multiplicative.
- Built-in qualitative palettes preserve curated colors and extend them through
  deterministic perceptual generation. Custom palettes explicitly select fixed
  or extendable capacity and never cycle silently.
- `CategoricalColorMap` is a general exact mapping for raw categorical
  identities. Used keys are required; unused entries and repeated exact colors
  are allowed.
- Sequential and diverging interpolation and palette extension use fixed
  OKLab/OKLCH color math. Missing marks are not palette endpoints.
- Transparent Canvas and panel surfaces are supported in SVG and PNG.
- Built-in contrast is regression-tested; automatic variable-fill text selects
  the better theme candidate; general custom-theme contrast remains caller-owned.
- Fonts use typed ordered stacks and the first installed generation-host family.
- Themes may omit nonsemantic chrome but cannot hide caller-provided semantic
  content. Existing explicit policies own content presence and placement.
- Shared Canvas legends use Canvas chrome and exact panel glyphs; conflicting
  glyphs for one semantic identity reject.
