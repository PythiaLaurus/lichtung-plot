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
| T2 | Complete visual-property ownership inventory | COMPLETE |
| T2a | Ledger protocol and renderer foundations | COMPLETE |
| T2a1 | Current theme and font foundations | COMPLETE |
| T2a2 | Private style carriers and backend protocol | COMPLETE |
| T2a3 | Serialization, geometry recording, and scene conversion | COMPLETE |
| T2a4 | Lifecycle, output, and artifact foundations | COMPLETE |
| T2b | Public controls and policies | COMPLETE |
| T2c | Layout, typography, axes, and grid | COMPLETE |
| T2d | Legends, Canvas composition, and decorations | COMPLETE |
| T2e | Ordinary plot marks and local styles | COMPLETE |
| T2f | Specialized marks and annotations | COMPLETE |
| T2g | Data scales and semantic domains | COMPLETE |
| T2h | Cross-source reconciliation and migration map | COMPLETE |
| T3 | One private resolved-theme pipeline | IN PROGRESS |
| T3a | T3 implementation partition | COMPLETE |
| T3b | Private resolved-theme foundation | COMPLETE |
| T3c | Backend lifecycle and protocol parity | IN PROGRESS |
| T3c1 | Closed primitive protocol and validation | COMPLETE |
| T3c2 | Backend lifecycle state machines | COMPLETE |
| T3c3 | Canonical coordinate conversion | PENDING |
| T3c4 | Concrete SVG and Null protocol parity | PENDING |
| T3d | Geometry discovery and painted bounds | PENDING |
| T3e | One resolved Plot theme and immutable plan | PENDING |
| T3f | Canvas resolved-theme handoff | PENDING |
| T3g | Canonical Scene and terminal conversion routing | PENDING |
| T3h | Failure-atomic artifact publication | PENDING |
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

**Status**: COMPLETE

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

**Status**: COMPLETE

Decision record: source reconciliation admits exactly 177 canonical T2c records
attributed as 12 text-layout records, 4 panel representations, 27 standard
axis/frame/grid responsibilities, 17 panel-geometry responsibilities, 20 range
and automatic-tick algorithms, 3 projection invariants, 11 exact-tick planning
branches, 34 Canvas-layout responsibilities, 5 composed-panel typography
values, 17 shared title-block responsibilities, 4 canonical axis-text preflight
branches, 1 ordinary panel-selection route, 4 primitive shared-facet-axis
responsibilities, and 18 ordinary multiline-axis adjustment responsibilities.
Module wiring contributes no record. These are source-based counts, not a
completion estimate. Together with the 483 completed T2a and T2b records, the
ledger inventories 660 records through completed T2c. T2d through T2h remain
source-driven and pending.

Verification: independent source-coverage, semantic-boundary, ownership,
schema, count, class, and table-order review corrected omitted line-height,
stroke-pattern, ordinary-layout, and route-specific responsibilities. Final
rereviews found no remaining issue. Tracked-diff checks and `just verify` passed
on 2026-08-26. No production code, test, snapshot, generated output, or other
specification changed.

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

**Status**: COMPLETE

Decision record: source reconciliation admits exactly 292 canonical T2d records:
106 Canvas and composed-layout responsibilities, 139 decoration and legend
responsibilities, and 47 facet-composition responsibilities. Source attribution
is 3/10/10/10/8/9/5/10/15 `canvas.rs` records, 2/9/9/6
`layout/canvas.rs` records, 0 module-wiring, 3 decoration-application, 62 legend,
2 panel-text collision, 72 panel-planning, 0 title-block, and 5/17/5/20 facet
route/planning/legend/rendering records. T2h reconciles the closed classes to
exactly 57 Theme component, 4 Explicit policy, and 231 Renderer invariant
records after four directional outside-spacing sources become consumers.
Together with the 660 completed T2a through T2c records, the ledger inventories
952 records through completed T2d. T2e through T2h remain source-driven and
pending.

Verification: independent source-partition, ownership, behavior, schema, count,
class, and table-order review corrected over-grouped Canvas and facet sources,
legend placement ownership, and exact composition traces. Final rereviews found
no remaining issue. Tracked-diff checks and `just verify` passed on 2026-08-26.
No production code, test, snapshot, generated output, or other specification
changed.

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

**Status**: COMPLETE

Decision record: source reconciliation admits exactly 203 canonical T2e records:
19 vertical-bar, 13 shared-bar, 22 horizontal-bar, 13 boxplot, 19 density, 8
distribution-sample, 38 sampled violin/raincloud, 20 precomputed raincloud, 26
line, 9 regression, 4 scatter, 8 guide-rendering, 3 exact-style-conversion, and 1
closed-line-pattern-conversion responsibilities. Closed classes are exactly 59
Theme component, 6 Explicit policy, and 138 Renderer invariant records. Together
with the 952 completed T2a through T2d records, the ledger inventories 1155
records through completed T2e. T2f through T2h remain source-driven and pending.

Verification: independent source-partition, ownership, behavior, schema, count,
class, relationship, and table-order review corrected significance and data-scale
leakage, over-grouped styles and geometry, incomplete source ranges, and exact
point/legend measurement traces. Final rereviews found no remaining issue.
Tracked-diff checks and `just verify` passed on 2026-08-27. No production code,
public API, tests, snapshots, generated output, intended rendering, or other
specification changed.

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

**Status**: COMPLETE

Decision record: T2h source reconciliation corrects T2f to exactly 509 canonical
records: 31 beeswarm, 3 coefficient, 9 dumbbell, 2 forest, 73 heatmap, 45
histogram, 23 interaction, 46 shared interval-estimate, 38 marginal, 62 Matrix,
18 point-interval, 18 Q-Q, 35 Waterfall, 55 significance, 13 panel-text, 1
numeric-label conversion, 1 text-weight conversion, 16 vertical-bar
significance, 15 horizontal-bar significance, 2 Boxplot significance, 2 sampled
Violin/Raincloud significance, and 1 attached-bar edge responsibility. Closed
classes are exactly 115 Theme component, 26 Explicit policy, and 368 Renderer
invariant records. Together with the 1155 completed T2a through T2e records, the
ledger inventories 1664 records through reconciled T2f. The net increase splits
sampled Histogram rug relative height from its shared stroke role; it adds no
production source or implementation change.

Verification: independent source-partition, ownership, behavior, schema, count,
class, relationship, migration-phase, and table-order review corrected missing
guide/range/title sources, data-domain leakage, overclaimed fallbacks, incomplete
annotation migration metadata, and duplicate-cluster phase order. Final
rereviews found no remaining issue. Tracked-diff checks and `just verify` passed
on 2026-08-27. No production code, public API, tests, snapshots, generated
output, intended rendering, or other specification changed.

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

**Status**: COMPLETE

Decision record: source reconciliation admits exactly 173 canonical T2g
records: 32 core semantic-value/accessor, 10 facet/Canvas/validation, 58 ordinary
plot-consumer, 71 specialized plot-consumer, and 2 focused residual
responsibilities. Closed classes are exactly 167 Renderer invariant, 3 Theme
data scale, 2 Plot-local data scale, and 1 Theme component records. Together
with the 1664 reconciled T2a through T2f records, the ledger inventories 1837
records through completed T2g after the T2h granularity correction.

Verification: independent source-partition, current-behavior, target-ownership,
public-name, schema, count, class, relationship, migration-phase, and table-order
review corrected over-grouped facet/Canvas sources, missing Beeswarm and Dumbbell
semantics, exact-map and hue-projection attribution, constant-domain handling,
accent naming, and ungated facet hue contribution. Final rereviews found no
remaining issue. Tracked-diff checks and `just verify` passed on 2026-08-27. No
production code, public API, tests, snapshots, generated output, intended
rendering, or specification outside the ownership ledger and this status changed.

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

**Status**: COMPLETE

Decision record: repository-wide residual and duplicate-cluster reconciliation
adds 23 T2h records: 5 Canvas terminal policies, 10 horizontal categorical-axis
helper sources, 3 significance selector policies, 3 significance resolver
invariants, 1 Canvas shared-x policy, and 1 KDE-support invariant. Together with
the one-record T2f rug split, the final ledger contains exactly 1860 records.
The corrected T2a-T2g manifests and T2h manifest are disjoint at the owned
expression/semantic-aspect boundary; their union plus explicit scientific,
transport, test, example, and supplemental-guide exclusions covers production
`src/`. Duplicate current sources retain separate evidence records while the
relationship matrix fixes one target owner or explicitly proves independent
semantic roles. No production code, public API, snapshot, generated output, or
intended rendering changed.

Verification: independent source-partition, target-owner, class, relationship,
public-name, schema-width, count, table-order, and T3-T7 scope review found no
remaining issue. All three keyed table sets contain the same 1860 unique keys in
identical order; `git diff --check` and `just verify` passed on 2026-08-27.

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

**Status**: IN PROGRESS

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
- Apply the T3-owned lifecycle, geometry-recording, Scene, output, and artifact
  corrections as explicit private changes: one immutable preflight plan, one
  canonical reusable Scene, paint-aware current-carrier geometry, exact terminal
  conversion routing, and failure-atomic publication. Retain the five Canvas
  terminal choices as policies rather than folding them into the pipeline.
- Keep backend traits theme-agnostic and continue passing concrete styles at draw
  boundaries.
- Preserve current intended built-in visuals and snapshots. Handle known visual
  defects only in separately identified changes with explicit evidence.

Acceptance:

- One resolved theme instance governs both preflight and final rendering.
- No renderer independently reconstructs a built-in palette or hidden default.
- Geometry discovery, canonical Scene storage, Output conversion, and artifact
  publication consume the same immutable plan and preserve terminal-specific
  policy/error identity.
- Backends remain independent of public theme types.
- Focused lifecycle, layout, theme, and visual-regression tests pass without
  unintended snapshot changes.
- `just verify` passes.

### T3a: Implementation Partition

**Status**: COMPLETE

Split T3 before production implementation so private representation changes,
intentional geometry corrections, lifecycle enforcement, planning reuse, output
routing, and filesystem publication do not land as one unreviewable change.

Scope:

- Assign every T3-owned concern to exactly one T3b-through-T3h implementation
  subphase and record the dependency order below.
- Keep the public `Theme`, Plot, Canvas, `Output`, terminal, options, and backend
  extension surfaces unchanged throughout T3.
- Preserve built-in rendered output except for the geometry and lifecycle defects
  explicitly assigned to T3c and T3d with focused evidence.
- Leave optional Plot themes, Canvas root-theme replacement, and inheritance to
  T4. T3f carries only the current selected Plot themes and hidden Publication
  Canvas appearance through one private resolved path.

Acceptance:

- T3b through T3h each define one reviewable objective, explicit dependencies,
  focused evidence, and a public-boundary constraint.
- The union of the subphases covers the parent T3 scope without authorizing T4
  inheritance or T5/T6 appearance and API migrations.
- No production code, public API, snapshot, or rendered output changes in T3a.

### T3b: Private Resolved-Theme Foundation

**Status**: COMPLETE

Replace the private palette identity with the one resolved appearance value that
later T3 subphases will carry through rendering.

Dependencies: T3a.

Scope:

- Replace private `ThemePalette` with `ResolvedTheme` without an alias or second
  representation.
- Preserve the current flat fields and exact four-preset values until T5 moves
  them into the approved component taxonomy.
- Make each built-in preset construct one structurally complete resolved value
  and validate all built-ins with exhaustive private tests.
- Keep the current public `Theme` enum and variants unchanged and expose no T6
  component, color, builder, inspection, or scale API.

Acceptance:

- All four built-ins resolve to complete values with nonempty colors and font
  family, positive finite widths, positive font sizes, and exact current values.
- No `ThemePalette` identity remains in production or test code.
- Existing theme tests and snapshots pass without changes.

### T3c: Backend Lifecycle And Protocol Parity

**Status**: IN PROGRESS

Establish the lifecycle and concrete-protocol guarantees required before one
plan and one canonical Scene can be reused safely.

Dependencies: T3a.

Scope:

- Enforce fresh begin/open/end state, balanced scopes, post-end rejection, and
  cleanup-error precedence consistently across SVG, Deferred, and Null backends.
- Invalidate stale SVG bytes whenever a later session begins or fails, and allow
  extraction only from one successfully finalized session.
- Replace lossy or saturating coordinate conversion with one checked canonical
  post-conversion geometry path, including signed off-canvas text and
  subprecision representability.
- Remove implicit rounded-rectangle fallback behavior and keep backend traits
  independent of themes.
- Replace arbitrary line dash strings with one closed private pattern carrier and
  retain exact degenerate primitive, gradient, and bracket validation parity.

Acceptance:

- Focused tests cover exactly-one begin/end, reuse, failed reuse, balanced clips
  and layers, primary-plus-cleanup failures, and draw-outside-canvas rejection.
- Geometry and SVG consume identical checked converted values; malformed or
  collapsed values fail rather than silently changing geometry.
- Existing successful built-in snapshots remain unchanged.

#### T3c1: Closed Primitive Protocol And Validation

**Status**: COMPLETE

Replace open or fallback primitive protocol states before tightening concrete
backend lifecycle and coordinate conversion.

Dependencies: T3a.

Ledger assignments:

- `backend.line.dash-string-carrier`,
  `backend.line.pattern-constructor-branches`,
  `backend.protocol.gradient-contract`, and
  `backend.protocol.rounded-rectangle-fallback`.
- `backend.validation.primitive-points`, `backend.validation.line-style`,
  `backend.validation.linear-gradient`,
  `backend.validation.rounded-rectangle`, and
  `backend.validation.significance-bracket`.

Scope:

- Replace the private arbitrary dash string and empty-string sentinel with one
  closed line-pattern carrier that serializes only admitted fixed patterns.
- Remove the `Backend::draw_rounded_rect` polygon fallback so every admitted
  backend must implement rounded rectangles explicitly.
- Reject rounded radii larger than half either extent and retain the exact
  current arity, gradient-stop, and significance-bracket protocols.
- Add focused shared validation evidence without changing successful built-in
  rendering.

Acceptance:

- Malformed line patterns are unrepresentable and solid/dashed construction is
  exhaustive.
- Deferred, Geometry, Null, SVG, and the test backend each compile only with an
  explicit rounded-rectangle implementation.
- Focused tests cover degenerate line/polygon bypass, invalid line widths, every
  gradient branch, rounded finite/nonnegative/far-edge/oversized-radius cases,
  and invalid bracket anchors/styles.

#### T3c2: Backend Lifecycle State Machines

**Status**: COMPLETE

Give SVG, Deferred, and Null explicit balanced canvas/scope state before one plan
or Scene is reused.

Dependencies: T3c1.

Ledger assignments:

- `backend.layer.semantic-role`, `backend.protocol.begin-end-lifecycle`,
  `backend.protocol.ordinary-layer-fallbacks`,
  `backend.protocol.grid-role-delegation`,
  `backend.protocol.clip-scope-contract`, `backend.validation.clip-region`, and
  `backend.protocol.scope-cleanup-precedence`.
- `backend.svg.concrete-canvas-lifecycle`,
  `backend.svg.layer-buffering-and-grid-order`,
  `backend.svg.clip-scope-serialization`,
  `backend.deferred.lazy-canvas-state-machine`,
  `backend.null.canvas-lifecycle-parity`, and
  `backend.null.clip-validation-parity`.

Scope:

- Invalidate SVG output before every attempted new session and expose bytes only
  from the most recently completed session.
- Make Deferred dimensions authoritative, reject mismatched explicit begin,
  start at most once, end the underlying backend exactly once, and reject every
  post-end operation.
- Make Null validate dimensions, open/closed state, nested clip balance, and
  draw-within-canvas parity.
- Replace panel Null calls that currently draw without begin/end with one balanced
  private validation session. This resolves the T2 wording conflict in favor of
  the target `backend.null.canvas-lifecycle-parity` contract; "without opening a
  canvas" means no output-producing canvas, not no backend lifecycle.

Acceptance:

- Focused tests cover begin/end reuse and failure paths, stale-byte invalidation,
  stored-dimension mismatch, empty success, post-end rejection, exactly-once
  finalization, invalid dimensions, unmatched/unclosed clips, and draws outside
  an open canvas.
- Scope helpers preserve cleanup-only and primary-plus-cleanup errors.
- Existing grid-before-marks and balanced-layer snapshots remain unchanged.

#### T3c3: Canonical Coordinate Conversion

**Status**: PENDING

Use one checked signed post-conversion geometry representation in measurement,
recording, and SVG serialization.

Dependencies: T3c2.

Ledger assignments:

- `backend.coordinates.projected-point-carrier`,
  `backend.protocol.coordinate-convention`, `backend.text.anchor-default`,
  `backend.text.placement-carrier`, and
  `backend.svg.coordinate-conversion-and-precision`.

Scope:

- Replace unsigned text placement at the private backend boundary with finite
  signed coordinates while retaining current ordinary integer placement bytes.
- Centralize every fixed-precision geometry conversion used by SVG and make
  Geometry consume the identical converted values.
- Reject nonfinite, saturated, or precision-collapsed distinct points, extents,
  bracket anchors, gradient endpoints, and stops without clamping or fallback.

Acceptance:

- Exact tests cover signed off-canvas text, y orientation, representable
  subprecision geometry, positive subprecision extents, and rejection when a
  serialization step would collapse a required distinction.
- Geometry and SVG tests observe identical post-conversion values.
- Ordinary built-in snapshots retain their current bytes.

#### T3c4: Concrete SVG And Null Protocol Parity

**Status**: PENDING

Close the retained primitive serializer and validating-backend evidence after the
shared protocol, lifecycle, and coordinates are canonical.

Dependencies: T3c1, T3c2, and T3c3.

Ledger assignments:

- `backend.point.geometry-carrier`, `backend.line.stroke-width-carrier`,
  `backend.bracket.geometry-carrier`, `backend.svg.xml-escaping`,
  `backend.svg.scatter-glyph-serialization`,
  `backend.svg.line-degenerate-noop`,
  `backend.svg.line-structure-and-pattern-serialization`,
  `backend.svg.polygon-degenerate-noop`,
  `backend.svg.polygon-structure-serialization`,
  `backend.svg.linear-gradient-serialization`,
  `backend.svg.rounded-rectangle-serialization`, and
  `backend.svg.document-framing`.
- `backend.null.line-degenerate-parity`,
  `backend.null.gradient-validation-parity`, the T3 portion of
  `backend.null.rounded-rectangle-validation-parity`, and the T3 portion of
  `backend.null.bracket-validation-parity`.

Scope:

- Emit exact closed line patterns, explicit cap/join rules, fixed triangle and
  polygon miter join/limit, deterministic gradients, native rounded rectangles,
  escaped XML, and canonical document framing.
- Keep short lines and polygons explicit no-ops that consume neither coordinates
  nor style.
- Prove Null rejects every current line, gradient, rounded-rectangle, and bracket
  input that final SVG rejects while deferring later RGBA and typography work to
  its assigned phases.

Acceptance:

- Exact SVG tests cover all admitted point shapes, solid/dashed lines, polygon
  joins, gradient order, rounded geometry, escaping, and document order.
- Degenerate invalid-carrier bypass and nondegenerate Null/SVG failure parity are
  explicit.
- Representative visual snapshots remain unchanged and `just verify` passes.

### T3d: Geometry Discovery And Painted Bounds

**Status**: PENDING

Make decoration discovery obey backend lifecycle and record the occupied bounds
of what current paint carriers actually draw.

Dependencies: T3c.

Scope:

- Require direct decoration geometry passes to begin and end one fresh Geometry
  session and reject unbalanced clips or drawing outside that session.
- Intersect every active clip and prevent bounds from leaking across sessions.
- Omit degenerate, zero-width, transparent, or otherwise unpainted current
  geometry as assigned by the T2 ledger.
- Record line caps and joins, polygon miter-limited stroke extent, and the painted
  fill/stroke union for rounded rectangles using rules shared with SVG.
- Defer complete independent RGBA handling to T6 and other explicitly assigned
  scatter, text, bracket, and mark-style work to T5.

Acceptance:

- Focused tests cover lifecycle balance, fresh reuse, nested noncontained clips,
  zero-width lines, transparent shapes, no-painted-area shapes, acute miters, and
  rounded-rectangle painted unions.
- Automatic-legend collision evidence reflects the corrected local bounds.
- Every intentional visual correction has separately reviewed snapshot or exact
  structural evidence.

### T3e: One Resolved Plot Theme And Immutable Plan

**Status**: PENDING

Resolve and plan a standalone Plot once, then reuse those immutable inputs for
preflight and the single final render.

Dependencies: T3b, T3c, and T3d.

Scope:

- Resolve the selected built-in once before canonical Plot validation and backend
  begin.
- Pass the same borrowed resolved value through layout, Null preflight, geometry
  discovery, facet planning, legend planning, planned-chart substitutions,
  post-panel validation, decorations, and final rendering.
- Remove `Plot::palette()` and every renderer call that reconstructs appearance.
- Produce one immutable private preflight plan that the final render consumes;
  public preflight reports from that same planning path without exposing the
  plan.
- Keep backend traits theme-agnostic and preserve all public Plot methods and
  terminal signatures.

Acceptance:

- Focused identity and call-count tests prove one resolution, one plan, and one
  final canvas lifecycle per Plot terminal.
- Facets and planned range/data substitutions cannot reconstruct appearance.
- Existing standalone, facet, lifecycle, and representative visual snapshots
  remain unchanged except for T3d corrections.

### T3f: Canvas Resolved-Theme Handoff

**Status**: PENDING

Carry current Canvas and child-Plot appearance through preflight and rendering
without adding T4 inheritance behavior.

Dependencies: T3b, T3c, T3d, and T3e.

Scope:

- Resolve one private Publication value for current Canvas-owned chrome and one
  selected value per child Plot during Canvas preflight.
- Store those values in the private Canvas plan and reuse them for text, header,
  row-title, layout, legend, panel, and final drawing paths.
- Remove draw-time hidden Publication and child-Plot reconstruction while
  preserving current first-Plot header and row-title behavior until T4.
- Retain all five Canvas terminal choices as policies and do not add
  `Canvas::theme`, optional Plot themes, or inheritance.

Acceptance:

- Focused tests prove preflight/render identity for text-only, homogeneous, and
  mixed-preset canvases.
- Canvas-owned chrome has no draw-time `Theme::Publication` reconstruction.
- Existing composition snapshots and public Canvas behavior remain unchanged.

### T3g: Canonical Scene And Terminal Conversion Routing

**Status**: PENDING

Construct one checked immutable Scene and make every terminal conversion consume
that canonical result with call-local policy.

Dependencies: T3c, T3e, and T3f.

Scope:

- Construct a Scene only from successfully finalized SVG bytes and canonical
  positive dimensions; remove duplicate source-dimension validation from PNG.
- Reuse exact canonical bytes for preserve-text SVG and as the source for
  outlined SVG and PNG conversion without rerendering.
- Compute PNG target dimensions and work limits once per call while preserving
  SVG bypasses, terminal-specific errors, and non-sticky options.
- Preserve Plot, Canvas, and reusable `Output` terminal signatures and choices;
  do not expose private plans, scenes, or resolved themes.

Acceptance:

- Tests reject incomplete output, zero dimensions, and dimension mismatches at
  canonical Scene construction.
- Chained outputs prove one render and exact Scene-byte reuse across SVG and PNG
  routes.
- Source and target limits, sizing modes, conversion failures, and call-local
  options retain exact route-specific behavior.

### T3h: Failure-Atomic Artifact Publication

**Status**: PENDING

Publish fully derived terminal bytes atomically without exposing partial files or
destroying an existing destination on failure.

Dependencies: T3g.

Scope:

- Retain derive-before-stage timing, exclusive same-directory sibling
  reservation, deterministic cleanup, and POSIX rename publication.
- Replace Windows backup-and-rollback publication with one true atomic
  same-directory replacement that preserves the previous destination on every
  failure.
- Aggregate cleanup failures deterministically and retain Drop cleanup as a final
  retry without masking the primary conversion, write, or publication error.

Acceptance:

- Focused tests cover reservation collisions, sibling placement, conversion and
  write failures, publication failure, cleanup aggregation, and no staging
  residue.
- Supported-host Windows evidence proves successful replacement, old-destination
  preservation on failure, and absence of backup artifacts.
- `just verify` passes and T3 can be marked complete.

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
- Carry exact Canvas/Plot theme provenance into shared-legend planning. T4 does
  not claim final compatibility: T5b owns legend/chrome comparison, T5c owns
  final mark-glyph style identity, T5d owns resolved mappings, and T6 closes the
  public replacement surface.
- Preserve plot erasure and ownership boundaries without global or thread-local
  theme state.

Acceptance:

- Standalone and composed defaults are visually identical when no explicit theme
  is supplied.
- Canvas-owned chrome never falls back to hidden `Theme::Publication` calls.
- Unthemed plots inherit; explicitly themed plots replace; neither behavior is
  sticky outside its owner.
- Measurement and rendering agree for headers, titles, text panels, and the
  inherited inputs supplied to facets/shared legends.
- Focused standalone, Canvas, facet, and inheritance-input tests pass; the final
  mixed-theme shared-legend acceptance gate remains in T5b/T5c/T5d/T6.
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
- Wire or remove dormant frame, label, and background properties according to
  the approved inventory. The dormant general line-width field belongs to T5c,
  where its mark-role consumers can be migrated together.
- Replace the horizontal categorical-axis helper duplicates with canonical
  raw/display resolution, logical-box measurement, line height, reserves, and
  final panel-fit checks; replace Canvas shared-x replay with a validated
  automatic continuous-range union that rejects exact-limit conflicts.
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
- Resolve outside spacing once from `LegendTheme` and reuse it for reservation
  and all four directional placements; do not reconstruct helper-local gaps.
- Complete Canvas shared-x integration after T5a range validation without
  overriding explicit limits or admitting categorical domains.
- Remove implicit Publication styling and duplicated fixed whites and grays from
  composition code.

Acceptance:

- Legend and composition measurement consume the exact final styles.
- Shared and per-plot legends use one component contract while retaining exact
  mark glyphs.
- Mixed explicit Plot themes have deterministic documented behavior.
- Shared-legend compatibility is complete only after T5c supplies final glyph
  styles, T5d supplies final mappings, and T6 supplies the current public theme
  values; T5b owns the final comparison and failure path across those inputs.
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
- Replace the dormant general line-width field with coherent semantic mark roles;
  deduplicate grouped-bar point/significance spans, sampled/precomputed Histogram
  and Raincloud styles, interval/PointInterval styles, mark/legend style
  reconstruction, and local automatic-range retries exactly as the T2h
  relationship matrix specifies.
- Connect exact significance rendered-target/facet-panel resolution from T5d to
  annotation geometry, and make shared KDE-support widening reject any result
  that is still nonfinite or constant.
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

- Define typed qualitative, sequential, and diverging palette or scale values.
  Missing-mark appearance is a non-data `MarkTheme` role owned by T5c/T6, not a
  scale endpoint; T5d owns only actual missing-value mapping semantics.
- Make built-in themes supply default data scales without conflating those
  scales with foreground, surface, grid, and annotation colors.
- Define exact keyed categorical mappings, ordered palette mappings, continuous
  interpolation, domain ownership, capacity validation, and missing mappings.
- Replace hardcoded heatmap and beeswarm gradients with resolved scale values.
- Generalize exact categorical color control to every plot family that consumes
  the same semantic mapping, or document why a family has a distinct owner.
- Coordinate facets and Canvas global hue order without silently changing exact
  caller mappings.
- Migrate every non-data accent-slot consumer from the current mixed palette into
  its resolved `MarkTheme` role while preserving T5d ownership of categorical
  identity/domain mapping and handing one final opaque color to T5c style
  assembly.

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
- Add or migrate explicit Canvas-root and Plot-replacement theme ownership using
  the exact public attachment names approved during T6; this plan does not
  pre-authorize `Canvas::theme(...)`, `Plot::theme(...)`, aliases, or a second
  attachment surface.
- Provide a typed way to derive a modified complete theme from a built-in base.
- Migrate current local style and categorical color callers to the one approved
  precedence model; delete replaced public and private representations.
- Keep backend styles private and avoid exposing renderer implementation details.
- Close the complete RGBA carrier migration through the sole per-paint SVG alpha
  serializer, gradient stops, text/bracket paints, paint-aware geometry, and PNG
  conversion. RGB token spelling remains a serialization dependency, not an
  alpha owner.
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
- Reconcile all 1860 final ledger keys against implemented owners, current
  public names, focused evidence, and deletion of every superseded source. A
  historical 1836-record pre-T2h count is not an acceptance threshold.
- Audit all specifications, README, guides, examples, and rustdoc for residual
  drift from the already-current implemented contract.

Acceptance:

- Phases T1 through T7 and all admitted T5 subphases are complete with commit and
  verification evidence.
- Every visual property has one implemented owner matching the approved
  1860-key inventory, with every source addition, split, consumer relationship,
  and explained exclusion re-audited.
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
