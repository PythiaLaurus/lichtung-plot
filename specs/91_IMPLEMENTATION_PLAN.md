# Implementation Plan

**Status**: IN PROGRESS

## Purpose

This document records the active theme-system design and implementation
sequence. The completed API-boundary and output-contract remediation remains
available in Git history through commit `0dcff20`; it is not a second current
plan.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative for the
implemented renderer. `03_THEME_SYSTEM.md` is the approved replacement theme
architecture. Phase T4 established its built-in Canvas/Plot ownership and
inheritance subset using the current closed `Theme` selector and updated the
current contracts atomically when that public behavior landed. Its replacement
theme values, components, colors, and scales remain non-current until Phase T6.
`05_LAYOUT_SYSTEM.md` is the approved replacement layout architecture; its
automatic layout, bounded-width sizing, and rendered-dimension behavior remain
non-current until Phase T6. Neither target specification is a second current
API, and later implementation phases do not begin merely because they are
listed here.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each admitted phase as one logical change before starting the next
phase. Split a broad phase into smaller approved phases before implementation
rather than combining unrelated migrations in one commit.

## Confirmed Direction

The following decisions are accepted starting points for the architecture
discussion:

- Publication is the replaceable library default for a standalone Plot and a
  Canvas root. Owners select it once; renderers and child planning must not force
  or repeat it as hidden fallback state.
- Canvas supplies the root theme for composed output. A plot without an explicit
  theme inherits that Canvas theme; an explicitly themed plot may replace it.
- The first public customization milestone targets coherent full foundations,
  not a color-only API and not an exhaustive property surface.
- Theme values that affect text measurement, layout, legend size, or collision
  planning must resolve before preflight and remain identical during final
  rendering.
- Scientific layout follows one private constraint, complete-candidate
  measurement, root-selection, arrangement, convergence, accounting, and freeze
  protocol. Parents pass named width and height constraints downward, children
  return complete nonempty candidate collections upward, every parent candidate
  is provisionally arranged, and only the selected root tree is promoted to
  final positions.
- An unconstrained or bounded root selects one finite publishable candidate
  through the exact eligibility, nonexact viability, dense-rank, and total-key
  rules in `05_LAYOUT_SYSTEM.md`; it does not use a hidden fixed canvas,
  locally pruned child result, sampled continuous interval, or universal
  per-panel multiplier.
- Exact `.size(width,height)` remains exact. Automatic and bounded candidates
  pass default-PNG and canonical-SVG viability before ranking; exact roots retain
  format-local SVG behavior and apply PNG limits only on a PNG terminal.
- Plot/Canvas root title-block alignment is one `CompositionTheme` property;
  built-ins are centered and T6 exposes Start/Center/End customization.
- Automatic numeric ticks have no fixed five-tick ceiling or public count target.
  T6 exposes one optional additive or Log10-decade preferred data interval per
  axis; the renderer tries it first, then deterministic coarsenings for fit, and
  keeps one terminal midpoint source after every interval vector.

Phase T1 settled the exact replacement theme and scale public types, replacement
semantics, component boundaries, validation, and scope of local overrides in
`03_THEME_SYSTEM.md`. Phase T4 made complete built-in Canvas/Plot theme
attachment and inheritance current without introducing the replacement value
model early. Phase T3j settled the private layout terminology, public sizing
boundary, migration partition, and final ledger impact. Phase T3k closed
terminal destination naming before T4 began.

## Reference Philosophy

The target architecture combines four relevant models:

- ggplot2 supplies the plotting-specific separation between non-data theme
  appearance, data-to-aesthetic scales, and exact layer aesthetics.
- Flutter supplies immutable complete theme data, organized component themes,
  explicit parent-scope inheritance, and the constraints-down, sizes-up,
  parent-positions layout direction.
- Vega-Lite supplies typed component configuration and deterministic precedence
  from broad defaults to exact mark settings.
- CSS intrinsic sizing supplies the useful distinction between minimum content,
  natural content, and constraint-dependent fit without admitting CSS selectors
  or its open cascade.

The design should not copy Flutter's widget-context lookup, reactive rebuilding,
animation, dynamic theme extensions, silent overflow widgets, or mandatory
single-pass restriction. It should not copy Matplotlib's mutable process-global
runtime configuration or CSS's selector-driven open cascade. Static scientific
output may use explicit bounded remeasurement, but semantic content never
silently clips, elides, scrolls, or disappears to force fit.

## Target Contract Rules

The approved theme and layout targets follow these rules:

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
- Visual spacing resolves from the local text, glyph, mark, axis, annotation, or
  composition metrics it separates. Root-canvas dimensions are not a global
  spacing or typography scale.
- Final rendering consumes one immutable arranged plan. It does not wrap text,
  select ticks, choose legend flow, resize tracks, derive spacing, or alter
  domains.
- Layout quality ordering, convergence, scientific aspect, resource budgets,
  and minimum representability are private renderer policies rather than theme
  fields or public callbacks.
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
| T3 | One private resolved-theme pipeline | COMPLETE |
| T3a | T3 implementation partition | COMPLETE |
| T3b | Private resolved-theme foundation | COMPLETE |
| T3c | Backend lifecycle and protocol parity | COMPLETE |
| T3c1 | Closed primitive protocol and validation | COMPLETE |
| T3c2 | Backend lifecycle state machines | COMPLETE |
| T3c3 | Canonical coordinate conversion | COMPLETE |
| T3c4 | Concrete SVG and Null protocol parity | COMPLETE |
| T3d | Geometry discovery and painted bounds | COMPLETE |
| T3e | One resolved Plot theme and immutable plan | COMPLETE |
| T3f | Canvas resolved-theme handoff | COMPLETE |
| T3g | Canonical Scene and terminal conversion routing | COMPLETE |
| T3h | Failure-atomic artifact publication | COMPLETE |
| T3i | Post-T3 review corrections | COMPLETE |
| T3i1 | Unfaceted Plot-plan terminology | COMPLETE |
| T3i2 | Resolved-theme terminology | COMPLETE |
| T3i3 | Canvas-plan terminology | COMPLETE |
| T3i4 | Canvas theme-handoff evidence | COMPLETE |
| T3i5 | Canvas coordination identity coverage | COMPLETE |
| T3i6 | PNG metadata and reuse boundaries | COMPLETE |
| T3i7 | Owned artifact staging and commit boundary | COMPLETE |
| T3j | Scientific layout architecture adjudication | COMPLETE |
| T3j1 | Normative layout contract and private terminology | COMPLETE |
| T3j2 | Layout ownership-ledger reconciliation | COMPLETE |
| T3j3 | Migration, evidence, and resource-boundary map | COMPLETE |
| T3k | Terminal output suffix completion | COMPLETE |
| T3l | Private planning-value terminology | COMPLETE |
| T4 | Explicit Canvas-to-Plot theme inheritance | COMPLETE |
| T4a | Ownership API and implementation partition | COMPLETE |
| T4b | Owner selection and resolved provenance | COMPLETE |
| T4c | Composition theme routing | COMPLETE |
| T4d | Public attachment and current contract | COMPLETE |
| T5a | Layout foundation, surfaces, typography, titles, axes, and grid | PENDING |
| T5a0 | T5a contract adjudication and execution partition | COMPLETE |
| T5a1 | Active exact-root candidate kernel | COMPLETE |
| T5a2 | Foundational resolved appearance and surfaces | COMPLETE |
| T5a3 | Canonical fonts and immutable text plans | COMPLETE |
| T5a4 | Root title-block plans | PENDING |
| T5a5 | Axis, range, tick, frame, and grid plans | PENDING |
| T5a6 | Ordinary Plot exact-root migration and closure | PENDING |
| T5a6a | Ordinary interior carriers and deferred-family adapters | PENDING |
| T5a6b | Standard-coordinate ordinary Plot migration | PENDING |
| T5a6b1 | Standard point and path families | PENDING |
| T5a6b2 | Standard distribution and reference families | PENDING |
| T5a6c | One-categorical-axis ordinary Plot migration | PENDING |
| T5a6c1 | Estimate-display families | PENDING |
| T5a6c2 | Bar families | PENDING |
| T5a6c3 | Categorical distribution families | PENDING |
| T5a6c4 | Interaction family | PENDING |
| T5a6d | Ordinary exact-root closure audit | PENDING |
| T5b | Legends, facets, and Canvas composition chrome | PENDING |
| T5c | Mark defaults, annotations, and local styles | PENDING |
| T5d | Categorical and continuous data scales | PENDING |
| T5e | Scientific layout closure | PENDING |
| T6 | Public immutable theme, scale, tick-interval, and sizing API | PENDING |
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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

**Status**: COMPLETE

Resolve and plan a standalone Plot once, then reuse those immutable inputs for
preflight and the single final render.

Dependencies: T3b, T3c, and T3d.

Scope:

- Resolve the selected built-in once before canonical Plot validation and backend
  begin.
- Pass the same borrowed resolved value through layout, Null preflight, geometry
  discovery, facet planning, legend planning, planned-limit Plot substitutions,
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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

**Status**: COMPLETE

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

## Phase T3i: Post-T3 Review Corrections

**Status**: COMPLETE

Close private terminology and implementation issues discovered during the
checkpoint review of completed T3 work before beginning T4 ownership changes.

Dependencies: T3.

Review rule:

- Continue the checkpoint review through T3f, T3g, and T3h. Record each accepted
  post-T3 finding here with an exact correction boundary and evidence before
  implementation.
- Add independently reviewable correction units rather than expanding an
  existing unit with unrelated work.
- Do not begin T3j until every recorded T3i correction unit is complete and the
  aggregate T3i acceptance evidence passes.

### T3i1: Unfaceted Plot-Plan Terminology

**Status**: COMPLETE

The T3e migration mechanically promoted the former `render_open_canvas` route to
`OpenPlotPlan` and `PlotScenePlan::Open`. In this route, "open" means unfaceted;
it does not represent backend lifecycle state. The retained word is ambiguous
beside the concrete `Open` canvas states established in T3c.

Scope:

- Rename `OpenPlotPlan` to `UnfacetedPlotPlan` and `PlotScenePlan::Open` to
  `PlotScenePlan::Unfaceted`.
- Update focused tests and current explanatory text that describe this private
  route, without renaming genuine backend open-state terminology.
- Preserve the exact open-versus-faceted routing behavior and all public Plot
  APIs.

Acceptance:

- The private scene-plan branch is named by the actual faceting distinction and
  cannot be confused with backend lifecycle state.
- Standalone unfaceted, faceted, preflight, and terminal lifecycle tests pass.
- No SVG, snapshot, public API, or intended rendering change occurs.

### T3i2: Resolved-Theme Terminology

**Status**: COMPLETE

T3e and T3f replaced repeated complete-palette reconstruction with owned
`ResolvedTheme` values, but retained local `palette`, `header_palette`, and
`row_title_palette` names for values that also carry typography, visibility,
axis, layout, and decoration properties.

Scope:

- Rename every production parameter, local, and helper name that denotes a
  complete `ResolvedTheme` from `palette` to `theme`, including Canvas header and
  row-title owners, layout, decorations, facets, and plot renderers.
- Update current architecture prose that calls the complete resolved appearance
  a palette.
- Retain `palette` terminology for actual categorical or continuous color
  palettes, palette slots, historical T2 ledger evidence, and negative API
  evidence for the removed `Plot::palette()` method.
- Do not rename data-color concepts or pull T5d/T6 scale work into this private
  terminology correction.

Acceptance:

- No production binding typed as `ResolvedTheme` or `&ResolvedTheme` is named
  `palette`.
- Every remaining source and current-contract use of `palette` denotes an actual
  color-palette concept or deliberate historical/negative evidence.
- Focused standalone, facet, Canvas, layout, and theme tests pass.
- No SVG, snapshot, public API, or intended rendering change occurs.

### T3i3: Canvas-Plan Terminology

**Status**: COMPLETE

T3f made the value currently named `CanvasPreflight` the complete frozen handoff
from Canvas planning to final rendering. It plays the same top-level role for a
Canvas that `PlotPlan` plays for a standalone Plot, while the current
`CanvasPanelPreflight` is one subordinate planned child. The retained
`render_open_canvas` name also conflicts with T3c's explicit backend open-state
terminology because its `DeferredBackend` begins in the fresh state.

Scope:

- Rename `CanvasPreflight` to `CanvasPlan`.
- Rename `CanvasPanelPreflight` to `PlannedCanvasPanel`, keeping it distinct from
  the top-level `PlotPlan` and the contained decoration `PanelPlan`.
- Rename `render_open_canvas` to `render_planned_canvas`.
- Rename the subordinate `cell` and `plan` fields to `canvas_cell` and
  `panel_plan` so the Canvas-allocated cell remains distinct from
  `PanelPlan::panel_box`, the post-reservation Plot region.
- Update focused tests and current explanatory text without changing the
  one-shot plan ownership or rendering sequence.

Acceptance:

- Plot and Canvas top-level plans use parallel terminology, while child and
  decoration plans retain unambiguous subordinate names.
- Canvas call sites visibly distinguish the original allocated cell from the
  locally reserved Plot panel.
- No genuine backend open-state terminology is renamed.
- Focused Canvas preflight, composition, and lifecycle tests pass with no SVG,
  snapshot, public API, or intended rendering change.

### T3i4: Canvas Theme-Handoff Evidence

**Status**: COMPLETE

The T3f pointer-identity instrumentation records the first-Plot theme as
`header-render`, although that value is passed only to row-title rendering;
shared headers are drawn with the Canvas-root Publication value. It also records
header stages when no corresponding header or row-title exists, allowing tests
to appear to cover consumers they do not exercise.

Scope:

- Split the current generic header evidence into exact `shared-header-plan`,
  `shared-header-render`, `row-title-plan`, and `row-title-render` consumers.
- Record theme evidence only when the corresponding planned or rendered content
  exists.
- Add fixtures with actual shared headers and row titles rather than inferring
  those routes from an empty text-only or ordinary Plot Canvas.
- Preserve and explicitly assert the current T4-owned split in which shared
  headers are measured with the first configured Plot theme but drawn with the
  Canvas-root Publication theme.

Acceptance:

- Every Canvas theme-identity evidence label names the consumer that actually
  receives that value.
- Empty header and row-title routes produce no false-positive identity evidence.
- Tests distinguish the retained shared-header ownership divergence from the
  valid row-title measurement/draw identity.
- No production appearance, public API, SVG, or snapshot changes occur.

### T3i5: Canvas Coordination Identity Coverage

**Status**: COMPLETE

T3f proves root and child theme identity for text-only, homogeneous, and
mixed-preset ordinary Canvas paths, but its focused counters do not traverse
several coordination branches that were migrated to stored child themes.

Scope:

- Add resolution-count and pointer-identity evidence for shared hue discovery
  and shared-legend preflight.
- Cover first-column shared category-label validation and grouped automatic
  horizontal-range planning.
- Cover sparse Canvas cells so the parallel `plots` and `plot_themes` ownership
  vectors prove index alignment across empty and text-panel slots.
- Cover actual mixed-preset shared headers and row titles in coordination with
  the exact consumer evidence from T3i4.
- Reuse existing integration behavior assertions where possible rather than
  duplicating broad visual tests.

Acceptance:

- Each admitted T3f coordination branch resolves the Canvas root once and each
  configured child theme once per public operation.
- Every render or planning traversal for one child observes that child's one
  stored `ResolvedTheme` address, including sparse cell indices.
- Focused Canvas lifecycle and composition tests pass with no SVG, snapshot,
  public API, or intended rendering change.

### T3i6: PNG Metadata And Reuse Boundaries

**Status**: COMPLETE

T3g validates positive finite DPI before converting it to PNG's integer
pixels-per-meter field. A sufficiently small positive DPI can produce a nonzero
target image while rounding that metadata to zero, so a successful conversion no
longer represents the caller's positive physical-resolution request. The T3g
checkpoint also found that current chain evidence does not directly mix default,
preview, and custom PNG policies on one rendered Plot or Canvas scene.

Scope:

- Reject a positive DPI when its pixels-per-meter representation rounds outside
  the positive PNG metadata range, including the lower zero boundary as well as
  the existing upper overflow boundary. Do not clamp or silently omit metadata.
- Preserve target-size calculation, source/target work limits, compression, and
  route-specific parse or dimension errors.
- Add focused chained-publication evidence that default, preview, and custom PNG
  options remain call-local and reuse one canonical Plot or Canvas scene without
  rerendering.
- Cover valid source dimensions that fail only after scaling at a target-axis or
  target-work boundary.

Acceptance:

- Every successful DPI publication records positive representable
  pixels-per-meter metadata corresponding to the validated request.
- Lower and upper metadata boundaries reject before PNG encoding without
  producing a file.
- Mixed PNG chains prove non-sticky options, exact Scene reuse, and independent
  route-specific failures for Plot and Canvas.
- No public API, SVG, snapshot, or successful in-range PNG behavior changes.

### T3i7: Owned Artifact Staging And Commit Boundary

**Status**: COMPLETE

T3h reserves a sibling staging path by creating a file, drops that exclusive
handle, and later reopens the predictable path for writing. That gap permits
pathname or symlink substitution by another actor that can modify the directory.
The cleanup wrapper also continues to own the staging pathname after successful
replacement, so recreation of that now-vacant path can turn an already committed
publication into an error or be removed as if it were still Lichtung's file.

Scope:

- Replace path-only reservation with one private owned staging value that uses
  an opaque collision-resistant sibling name, retains its exclusive file handle,
  and receives first-party bytes through that handle without reopening the path.
- Publish that exact owned staging file through one supported-host atomic
  same-directory replacement. Preserve the Windows no-backup replacement path
  and do not add rollback or compatibility routes.
- Define publication success as the commit point. Disarm staging cleanup
  immediately after successful replacement; no later cleanup of the vacant
  staging pathname may convert success into an error or remove a new occupant.
- Before the commit point, preserve deterministic primary-plus-cleanup error
  aggregation and Drop as a final best-effort retry without masking the primary
  failure.
- Use a staging leaf independent of the full destination basename, and cover
  valid component-limit destinations plus supported-host Windows long paths with
  spaces or Unicode. Continue rejecting invalid or reserved Windows leaves.

The supported POSIX contract excludes external namespace mutation between the
final staging-identity check and the one rename or unlink syscall. Portable POSIX
has no rename or unlink operation by open file handle; this correction closes the
prior first-party drop/reopen interval rather than claiming an unavailable
directory capability. Windows retains an exclusive handle and performs commit
and cleanup through that handle.

Acceptance:

- First-party staging never drops and reopens a reserved path, and substitution
  of a pathname cannot redirect the staged write to another file.
- Successful publication returns success with the complete staged bytes at the
  destination and performs no fallible post-commit cleanup.
- Every pre-commit failure preserves an existing destination and removes only
  the still-owned staging artifact, with deterministic cleanup diagnostics.
- Focused POSIX and supported-host Windows evidence covers replacement,
  substitution attempts, long and near-limit names, cleanup failures, and no
  staging residue.
- No public API, output bytes, or successful ordinary-path behavior changes.

Aggregate acceptance:

- Every finding recorded during the T3f-through-T3h checkpoint review is either
  completed in a T3i correction unit or explicitly rejected with rationale.
- Current architecture and implementation terminology agree before layout
  adjudication and T4 ownership changes.
- `just verify` passes before T3i is marked complete and T3j begins.

## Phase T3j: Scientific Layout Architecture Adjudication

**Status**: COMPLETE

Define and reconcile the layout replacement before theme inheritance or visual
component migrations make new assumptions about fixed root dimensions.

Dependencies: T3i.

No production code changes in this phase.

### T3j1: Normative Layout Contract And Private Terminology

**Status**: COMPLETE

Dependencies: T3i.

Scope:

- Complete `05_LAYOUT_SYSTEM.md` as the sole target for normalized integer
  constraints, complete candidate collections, finite generation, root
  selection, arrangement, convergence, resource accounting, reservation, exact
  sizing, bounded width, automatic roots, and rendered dimensions.
- Define `LayoutScalar` as signed 64-bit quanta and close every outward,
  inward, nearest-even, exact-size, nonspatial-`f64`, checked-arithmetic, and
  overflow disposition.
- Define one successful measurement call as one complete nonempty
  `MeasuredCandidates<LayoutPlan>` collection. Only root selection chooses one
  candidate; parent combinations do not locally prune child alternatives.
- Define complete text, generated-tick, legend, aspect, exact-total residual,
  track, packing, preferred-size, and specialized-family generation.
- Define closed canonical addresses, role and type discriminants, PlotKind order,
  and specialized interior traversal.
- Define every total-key formula, eligibility gate, dense-rank pool, signature
  field, duplicate identity, and nonexact viability filter.
- Define the complete next-pass frontier, convergence state, cache key, cache-hit
  accounting, pass limit, five cumulative accounting identities, reservation
  stages, and failure order.
- Define exact/nonexact canonical-SVG behavior without changing the public API.
- Keep the complete kernel private and admit no public measurement, constraint,
  candidate, plan, callback, or plugin surface.

Acceptance:

- Every successful child measurement has one mechanically defined cardinality
  contract and every parent combination preserves the complete child set.
- Every finite candidate source and every preference has an exact producer,
  ordering, deduplication rule, and budget disposition.
- Every two surviving candidates compare under one total key with no undefined
  numeric conversion, rank, vector, signature, or overflow behavior.
- Convergence equality includes every next-pass alternative and breakpoint input,
  and cache reuse requires equality of the complete consumed frontier.
- Candidate counting, reservation timing, nonexact viability, and final
  reservation occur in one exact failure order before backend creation.
- Exact SVG and nonexact SVG/PNG limits have disjoint, explicit scopes.
- The target introduces no public API, hidden fixed candidate, fallback route,
  or second plan representation.

### T3j2: Layout Ownership-Ledger Reconciliation

**Status**: COMPLETE

Dependencies: T3j1.

Scope:

- Reconcile the layout target through the broad ownership interpretation: one
  property owner covers the property's source or derivation and every
  measurement, candidate-generation, quality, convergence, accounting,
  arrangement, frozen-plan, and draw consumer of that same value.
- Require a trace for every layout input from its ownership-ledger or target-only
  key through canonical address, exhaustive source category, validated or
  normalized value, consuming stages, and immutable measured- or frozen-plan
  field.
- Treat derived breakpoints, candidate descriptors, rank tuples, state fields,
  capacities, and serialization counts as traced consumers of their owning
  policy rather than unrecorded visual properties.
- Re-adjudicate every current fixed dimension, panel minimum, padding, gap,
  spacing factor, wrapping branch, post-allocation fit check, legend flow rule,
  semantic content input, resolved style metric, mark or annotation extent,
  topology source, domain identity, specialized reserve, aspect rule, and
  draw-time layout decision in `04_VISUAL_PROPERTY_OWNERSHIP.md`.
- Preserve the 1860-key T2 count as historical completed-inventory evidence.
  Record one canonical overlay for affected historical keys and target-only
  policies, then calculate one materialized final acceptance count without
  inserting future-only identities into historical Current rows.
- Distinguish theme-owned relative visual relationships from renderer-owned
  scientific geometry, data density, aspect, quality ordering, convergence, and
  resource limits.
- Reconcile Plot and Canvas root title/subtitle gaps and caption-to-content
  clearance as `CompositionTheme` relationships resolved against adjacent
  typography, rather than retaining parallel private fixed spacing owners.
- Reconcile the three current root title/subtitle/caption anchor sources into one
  target-only `CompositionTheme` root title-block alignment while retaining zero
  rotation as a renderer invariant.
- Trace each measurement input to the exact immutable plan value consumed by
  drawing. Draw-time reconstruction is a defect, not an admitted duplicate.
- Remove from the overlay every dormant or overwritten source with no reachable
  successful-output effect; retain its historical source row as T2 provenance.

Acceptance:

- The three completed T2 keyed tables remain historical evidence containing
  exactly 1860 unique keys in identical order. Future-only target policies are
  not inserted into those historical Current, Target, or Migration tables.
- One canonical post-adjudication overlay expands to exactly 1576 unique rows:
  1552 rows re-adjudicate existing historical keys and 24 rows introduce
  target-only keys. The existing-key set consists of 691 corrected `A-*` keys
  and 861 broad-trace `B-*` additions. Existing overlay keys occur in the
  historical inventory; target-only keys do not; no overlay key occurs twice.
- Exactly 62 existing keys replace their historical target disposition. The
  other 1490 existing keys retain that disposition and gain the broad immutable-
  plan relationship and focused evidence route.
- Materializing the target inventory applies existing-key overlay rows in the
  historical 1860-key order and then appends the 24 target-only keys in their
  recorded canonical order, producing exactly 1884 unique final keys.
- Every source consumed by measurement, generation, quality, convergence,
  accounting, arrangement, the frozen plan, or drawing retains its historical
  source evidence and has one final target owner, migration phase, exact
  relationship to the private layout protocol, and focused evidence route.
- The former 696-key narrow overlay is not an acceptance threshold. Its five
  no-effect removals and 861 broad additions are mechanically reproducible from
  the historical key order and Current layout-path fields.
- T7 refers only to the recalculated 1884-key final inventory. The historical
  1836-record pre-T2h count and completed 1860-record T2 count remain provenance
  and are not post-adjudication thresholds.

### T3j3: Migration, Evidence, And Resource-Boundary Map

**Status**: COMPLETE

Dependencies: T3j2.

Scope:

- Partition the private layout migration across T5a through T5e so each phase
  leaves one coherent internal representation and no temporary public API.
- Define focused preservation evidence while the new protocol runs under the
  current exact defaults, then separate intentional automatic-output snapshot
  changes for the atomic T6 switch.
- Adjudicate one finite automatic-layout resource envelope against canonical SVG,
  outlined-text conversion, default PNG source and target work, grid cardinality,
  arithmetic overflow, and allocation failure. Do not add arbitrary caps without
  source and boundary evidence.
- Define deterministic convergence state, repeated-state failure, pass budgets,
  complete next-pass frontier, cache identity, rounding, and pre-backend failure
  timing.
- Fix one spatial quantum at `1/1024` CSS pixel, one eight-complete-pass root
  budget, five independent, non-pooling cumulative `1_000_000` limits for
  candidate evaluations, candidate-generation slots, candidate-generation work,
  measurement-call attempts, and measurement-cache entries, and one
  `1_000_000`-slot prepared canonical structural-topology limit.
- Charge candidate-generation work from one canonical logical-operation census,
  including fixed per-record sort and dedup charges, never from implementation-
  specific loop or comparator counts.
- Reuse the current inclusive `32_768` PNG axis, `100_000_000` PNG pixel, and
  `67_108_864` canonical SVG byte limits. Require automatic and bounded roots to
  pass exact default-300-DPI target prediction while exact roots retain
  format-local terminal validation.
- Require checked topology and candidate arithmetic and five exact fallible
  first-party reservation stages for prepared topology and canonical font
  candidates/provenance, the fixed measurement cache, layout alternatives,
  candidates, convergence state, and the final plan/serializer/scene census.
  Project-owned capacity refusal is a render failure before backend creation;
  do not promise recovery from process-global allocation failure inside an
  external dependency.
- Apply candidate gates and nonexact viability before dense-rank construction.
  Count every cache-miss candidate before evaluation, including candidates later
  rejected or deduplicated; a complete-frontier cache hit counts zero.

Migration partition:

- T5a introduces the sole normalized constraint/complete-candidate/arrangement
  kernel, the complete total-key representation and exact-root selector, the
  reusable two-through-eight-pass convergence foundation, immutable text plans,
  ordinary Plot measurement, and exact-root preservation. Every not-yet-migrated
  family enters through one typed private measurement adapter that produces the
  same complete collection of candidate-local plans; root selection promotes one
  to the sole root-plan representation, with no second backend or root-plan
  route.
- T5a builds and reserves every canonical generation-host font list before text
  measurement, enforcing the exact matching, occurrence, identity,
  deduplication, ordering, primary-face, glyph-fallback, and provenance contract
  from `03_THEME_SYSTEM.md` without treating font faces as layout candidates.
- T5a resolves one private root title-block alignment, initializes every built-in
  to `Center`, and carries it unchanged through title/subtitle/caption text plans,
  root arrangement, signatures, Null geometry, and concrete rendering. It also
  establishes optional additive and Log10-decade automatic tick intervals and
  removes the fixed five-tick ceiling without exposing either public API early.
- T5b migrates legends, facets, Canvas tracks, shared chrome, headers, row
  titles, and text panels onto complete measured candidate collections. It
  establishes the sole kind-discriminated shared-guide representation and
  compatibility/failure path.
- T5c supplies exact mark ink, category/cell pitch, interval, density, and
  annotation requirements and removes post-allocation family-local fit logic.
- T5d supplies final scale domains, guide content, and significance targets to
  the same plans without creating another layout representation. It supplies
  final continuous contributor descriptors and performs the exact automatic-
  union/equal-exact/contained-automatic merge through T5b's shared-guide path
  before label generation or guide measurement.
- T5e removes the last adapter and completes finite generation, every remaining
  total-order supplier, root frontier/cache identity, automatic/bounded roots,
  specialized composite measurement, the five-stage resource census, and the
  pre-backend failure boundary by extending T5a's one selector and convergence
  implementation.
- T6 alone switches the public defaults and sizing/accessor surface. T7 audits
  the completed owners and does not add missing foundational behavior.

Deletion targets:

- T5a deletes current single-panel fixed-layout constructors, axis/title local
  retry helpers, draw-time text wrapping fallbacks, and generic fixed panel
  minima as their consumers migrate.
- T5b deletes equal-cell Canvas/facet allocators, `PanelRequirements`,
  provisional/first/final composition passes, draw-time shared-legend
  reconstruction, and facet draw-time text wrapping.
- T5c deletes family-local post-allocation reserve/retry branches and duplicate
  mark/legend geometry reconstruction assigned by the ledger.
- T5d deletes mixed current-palette consumers, hardcoded Heatmap/Beeswarm guide
  mappings, and duplicate guide/domain reconstruction while retaining the old
  public types until T6.
- T5e deletes every typed legacy measurement adapter, Matrix's local iteration
  loop, fixed marginal bands, hardcoded specialized reserves, remaining
  draw-time layout/domain mutation, and every fixed-size fallback.
- T6 deletes unconditional default width/height storage, pre-render Plot
  dimension getters, old public theme/color representations, and all replaced
  compatibility surfaces.

Evidence and snapshot policy:

- T5a through T5e preserve explicit-size canonical preserve-text SVG bytes
  unless one named ledger row authorizes an intentional correction. Outlined SVG
  and PNG use structural, dimension, and controlled-font hash evidence.
- T5e private automatic tests assert exact structure, dimensions, total-order
  behavior, convergence, and resource boundaries without changing public
  default snapshots early.
- Before T6 changes no-sizing-call defaults, add explicit `640x480` Plot and
  `960x640` Canvas preservation fixtures. T6 alone reviews automatic-default
  snapshot changes, each with the complete selected quality key and competing
  candidate frontier.
- Numeric evidence covers limits minus one, equal limits, and limits plus one for
  each independent candidate-evaluation, candidate-generation-slot, candidate-
  generation-work, measurement-call, measurement-cache-entry, and prepared-
  topology limit;
  default-PNG source axis `10_485` versus `10_486`; default target work at
  `3_200 x 3_200` versus `3_200 x 3_201`; pass-two/pass-eight stability; changed
  pass eight; non-adjacent repetition; candidate-order permutation; checked
  arithmetic overflow; and injected first-party reservation failure at each of
  the five reservation stages.
- Candidate evidence covers complete child Cartesian combination, no local
  quality pruning, exact dense-rank pools and formulas, specialized traversal,
  changed-frontier cache misses, unchanged-frontier cache hits, and cumulative
  accounting of rejected and deduplicated candidates. Work-unit evidence proves
  equal charges under loop fusion, vectorization, and different sorting
  implementations.
- Ownership evidence covers every historical or target-only layout input from
  owner through each consumer and immutable plan field, with no unowned
  generation, convergence, accounting, arrangement, or draw input.

Acceptance:

- Every layout unit has explicit dependencies, focused tests, snapshot policy,
  and deletion targets.
- Automatic resource failure cannot clamp, scale down, omit content, fall back
  to fixed dimensions, or select a second renderer.
- The complete source-backed resource envelope, failure order, exact finite
  quality key, convergence identity, and historical-ledger overlay are complete
  enough to implement without inventing policy in source code.
- `git diff --check`, the repository's documentation-only checks, and focused
  independent specification review pass before the adjudication closes. This
  documentation-only phase does not run the test suite.

## Phase T3k: Terminal Output Suffix Completion

**Status**: COMPLETE

Complete every successful output destination with the terminal's conventional
format suffix without interpreting or removing caller-provided filename text.

Dependencies: T3j.

Scope:

- Treat the terminal path argument as a base file destination. `svg` and
  `svg_preserve_text` own expected suffix `.svg`; `png`, `png_preview`, and
  `png_with_options` own expected suffix `.png`. Plot and Canvas terminals reach
  the same `Output` boundary and do not implement separate suffix rules.
- Require a file-name component and reject the caller path before suffix
  completion when it already identifies a directory. If the final file-name
  component ends with the complete ASCII suffix `.svg` or `.png` expected by the
  terminal under ASCII case-insensitive comparison, preserve the complete caller
  path exactly. Otherwise append the lowercase expected suffix to that component.
  This is a literal suffix rule, not `Path::extension` interpretation. Never
  replace, remove, normalize, or reject another extension.
- Apply these exact examples:

  ```text
  chart       + SVG -> chart.svg
  chart.v1    + SVG -> chart.v1.svg
  chart.png   + SVG -> chart.png.svg
  chart.SVG   + SVG -> chart.SVG
  chart       + PNG -> chart.png
  chart.v1    + PNG -> chart.v1.png
  chart.svg   + PNG -> chart.svg.png
  chart.PNG   + PNG -> chart.PNG
  .chart      + SVG -> .chart.svg
  .svg        + SVG -> .svg
  .PNG        + PNG -> .PNG
  ```

- Complete the destination before parent canonicalization, Windows leaf
  validation, sibling staging, identity checks, replacement, and cleanup. Every
  publication guarantee applies to the completed destination. After the caller-
  directory check, an existing non-directory unsuffixed base path is neither read
  nor replaced merely because its suffixed sibling becomes the destination.
- Infer no format from the caller path or existing file contents. A contradictory
  suffix remains ordinary caller filename text, and terminal selection alone
  determines the generated bytes.
- Preserve valid platform-native filename data without requiring UTF-8. Do not
  create missing parent directories, expose the private suffix helper, add a
  public path-normalization API, or return a second destination representation.
- When implementation lands, update `02_RENDERING_CONTRACT.md`, public terminal
  rustdoc, and examples atomically so the completed path behavior becomes the one
  current contract.

Acceptance:

- Focused Plot, Canvas, and reusable-`Output` tests cover extensionless,
  matching lowercase, matching mixed-case, versioned, contradictory, dotfile,
  and chained destinations for preserve-text SVG, outlined SVG, preview PNG,
  default PNG, and custom PNG.
- Native-path evidence covers matching and appended suffixes after non-UTF-8 Unix
  filename bytes, and matching and appended suffixes on valid non-ASCII Windows
  UTF-16 leaves, without lossy conversion.
- Tests prove suffix completion occurs exactly once, writes only the completed
  destination, leaves an existing unsuffixed base untouched, and preserves an
  existing completed destination on conversion, write, or publication failure.
- Missing file names, existing directories, missing parents, and invalid or
  reserved completed Windows leaves retain explicit failure behavior with no
  completed sibling or staging residue.
- The change does not alter terminal signatures, SVG or PNG bytes, Scene reuse,
  PNG option scope, layout, or the supported-host atomic replacement protocol.
- `just verify` passes before T3k is marked complete and T4 begins.

## Phase T3l: Private Planning-Value Terminology

**Status**: COMPLETE

Align private planning fields and their directly connected identifiers with the
concrete values they carry. The current use of `chart` for `Plot` values does not
represent a second type or lifecycle state and must not imply one.

Dependencies: T3j.

Naming rule:

- Name a sole value of type `Plot` or `Plot<K>` `plot`, and name a collection of
  those values `plots`. Do not use `chart` as an alternate technical identity for
  the Rust `Plot` type.
- Add a role qualifier only when one scope contains multiple `Plot` values whose
  roles must be distinguished. Name that exact role, such as `source`,
  `base_plot`, `planning_plot`, or `panel_plot`; do not call a sole value
  `render_plot` or `planned_plot` merely because a plan stores it.
- Name nested plan and geometry values by their actual role when the unqualified
  name hides the contained type. A field name need not mechanically repeat its
  Rust type when its existing semantic role is already unambiguous.
- Retain "chart" as ordinary user-facing prose where it means the rendered
  graphic. This phase changes private Rust identifiers, not the library's natural
  language or public API.

Scope:

- Rename `CanvasPlan::charts` to `plots` while retaining `CanvasPlan::panels` for
  its collection of `PlannedCanvasPanel` values and `canvas_cell` for each
  original grid allocation.
- Rename `UnfacetedPlotPlan::chart` to `plot` and
  `UnfacetedPlotPlan::panel` to `panel_plan`.
- Rename `FacetPlan::chart` to `plot` and `FacetPlan::plans` to `panel_plans` so
  the stored Plot, facet metadata, and decoration plans remain distinct.
- Rename `PanelPlan::panel` to `panel_box`. Replace
  `PanelPlan::chart_for_render` with `plot_with_planned_limits`, which states the
  exact transformation rather than inventing a render-stage Plot identity.
- Apply the same rule to directly connected Canvas, unfaceted, facet, legend,
  and decoration-planning parameters, locals, helpers, and tests. This includes
  `chart_category_domain` to `plot_category_domain` and Plot-valued
  `chart`/`charts`, `planning_chart`, `panel_chart`, and `base_chart` identifiers
  to their corresponding Plot terminology.
- Update current explanatory text that claims one of the replaced private names.
  Retain explicitly historical wording and unrelated natural-language uses of
  "chart".
- Preserve the exact planning data, ownership, ordering, cloned-Plot
  transformations, panel geometry, and final rendering sequence. Do not add a
  `Chart` type, alias, wrapper, conversion, or compatibility name.

Acceptance:

- The private plan hierarchy reads `CanvasPlan::plots`,
  `UnfacetedPlotPlan::{plot, panel_plan}`,
  `FacetPlan::{plot, panel_plans}`, and `PanelPlan::panel_box` without an
  undocumented Plot-versus-chart distinction.
- A sole Plot-valued field at these planning boundaries is named `plot`; any
  qualified Plot-valued binding is justified by another Plot value in the same
  scope and names the exact distinguishing role.
- Direct planning helpers and consumers use Plot terminology consistently, and
  no compatibility alias or duplicate route retains the replaced identifiers.
- Focused standalone, facet, Canvas, panel-planning, lifecycle, and terminology
  tests pass with no SVG, PNG, snapshot, public API, or intended rendering
  change.
- `just verify` passes before T3l is marked complete and T4 begins.

## Phase T4: Canvas Theme Inheritance

**Status**: COMPLETE

Dependencies: T3k and T3l.

Make composition ownership explicit and consistent, then expose the completed
built-in inheritance behavior through one deliberate public attachment on each
owner.

Scope:

- Store one root `Theme` selector on Canvas. Construction assigns
  `Theme::Publication`; callers are not required to make an explicit theme call.
- Retain one optional explicit `Theme` selector on Plot. Absence is meaningful
  and must not be collapsed into `Theme::Publication`, compared by value, or
  inferred after erasure.
- Approve `Canvas::theme(Theme)` as the sole Canvas-root attachment and retain
  `Plot::theme(Theme)` as the sole Plot-replacement attachment. T4d exposes the
  Canvas method only after the private owner and routing model is complete.
- Make an unthemed standalone Plot select Publication, an unthemed composed Plot
  inherit its Canvas root, and an explicitly themed Plot completely replace the
  inherited root for that Plot.
- Apply the resolved Canvas theme consistently at every existing theme-aware
  Canvas title-block, text-panel, shared-header, row-title, and spacing consumer.
  T4 does not migrate surfaces or hardcoded chrome properties assigned to T5.
- Resolve the Canvas root once per lifecycle. Represent each occupied Plot slot
  as exact inherited-root versus explicit-Plot provenance before current
  preflight and measurement consumers run. Retain that provenance for the future
  T5 measurement context so every child receives the exact owner used by its
  final plan.
- Make header and text measurement use the same resolved values as rendering.
- Carry exact Canvas/Plot theme provenance into shared-legend planning. T4 does
  not claim final compatibility: T5b establishes legend/chrome comparison, T5c
  supplies final mark-glyph style identity, T5d supplies resolved mappings, and
  T6 closes compatibility over the public replacement values.
- Preserve plot erasure and ownership boundaries without global or thread-local
  theme state.
- Preserve current exact dimensions and public sizing behavior; T4 establishes
  ownership inputs and does not implement the T5 layout protocol early.
- Keep the current closed built-in `Theme` selector throughout T4. T6 replaces
  its value model while retaining the one `.theme(Theme)` attachment on each
  owner; no alias, fallback attachment, or second theme representation is
  admitted.

Implementation partition:

1. T4a approves the owner representation, sole public attachment names,
   sequencing, and specification boundary without changing production code.
2. T4b changes the private owner selection and resolved-provenance model while
   preserving current public behavior and output.
3. T4c completes existing Canvas/Plot/facet theme-aware consumer routing and
   shared-legend provenance while the non-default Canvas root remains private.
4. T4d exposes the already-complete Canvas root attachment and atomically makes
   built-in inheritance current in the public contracts and documentation.

Acceptance:

- Standalone and composed defaults are visually identical when no explicit theme
  is supplied.
- Existing theme-aware Canvas consumers never perform hidden
  `Theme::Publication` selection.
- Unthemed plots inherit; explicitly themed plots replace; neither behavior is
  sticky outside its owner.
- Measurement and rendering agree for headers, titles, text panels, and the
  inherited inputs supplied to facets/shared legends.
- `Canvas::theme(Theme)` and `Plot::theme(Theme)` are the only public owner
  attachments; omitting the Canvas call selects Publication and omitting the
  Plot call selects inheritance or the standalone Publication default.
- Focused standalone, Canvas, facet, and inheritance-input tests pass; the final
  mixed-theme shared-legend acceptance gate remains in T5b/T5c/T5d/T6.
- `just verify` passes.

### Phase T4a: Ownership API And Implementation Partition

**Status**: COMPLETE

Dependencies: T3k and T3l.

Approve one implementation-ready ownership migration before changing runtime
behavior.

Scope:

- Record mandatory Canvas-root and optional explicit Plot selection as the sole
  source representation for T4.
- Approve `Canvas::theme(Theme)` and retain `Plot::theme(Theme)` without
  requiring an explicit Canvas call or adding clear/inherit/optional aliases.
- Reserve public exposure for T4d so callers never observe a partially routed
  root theme.
- Assign the replacement immutable `Theme` value model to T6 while fixing the
  two owner attachment names in T4.
- Partition T4b through T4d with explicit code, evidence, specification, and
  commit boundaries.
- Reconcile target/current staging language without claiming that the pending
  T4 runtime behavior is already implemented.

Acceptance:

- The T4 representation, default-selection semantics, public boundary, and
  subphase dependencies are unambiguous.
- No compatibility alias, temporary public API, production code, snapshot, or
  rendered output changes.
- Documentation review is sufficient; the code verification suite is not run
  for this documentation-only phase.

### Phase T4b: Owner Selection And Resolved Provenance

**Status**: COMPLETE

Dependencies: T4a.

Replace implicit Publication ownership with exact private selection and
provenance while preserving the current public contract.

Scope:

- Change Plot's stored theme selection from mandatory `Theme` to
  `Option<Theme>`. Constructors store `None`; `Plot::theme(Theme)` stores
  `Some(theme)`; type erasure preserves the option without defaulting it.
- Make standalone planning resolve `None` to one Publication value before
  validation or measurement. Explicit standalone selections resolve once
  through the same lifecycle boundary.
- Add one mandatory private Canvas-root `Theme` initialized to Publication, but
  do not expose `Canvas::theme(Theme)` in this phase.
- Resolve the Canvas root once. For each occupied Plot slot, retain an explicit
  inherited-root or explicit-Plot provenance variant rather than cloning the
  root, comparing presets, or storing an indistinguishable resolved value.
- Give planning consumers one immutable accessor that returns the selected
  `&ResolvedTheme`; inherited slots borrow the exact Canvas root and explicit
  slots borrow their separately resolved replacement.
- Keep resolution owner-local and lifecycle-local with no global, thread-local,
  backend-selected, or late preset lookup.
- Delete the replaced mandatory-Plot-theme and per-child Publication-selection
  routes rather than retaining late defaults outside the owning boundary.

Acceptance:

- Tests distinguish unthemed Plot selection from explicit Publication before
  resolution and prove the two standalone paths render equivalently.
- Canvas tests prove inherited children use the exact root identity, explicit
  children use a distinct resolved identity even for Publication, and erased
  plots preserve their original selection.
- Default standalone and Canvas snapshots remain unchanged.
- Focused builder, lifecycle, erasure, Canvas, and facet tests pass.
- `just verify` passes before T4b is committed.

### Phase T4c: Composition Theme Routing

**Status**: COMPLETE

Dependencies: T4b.

Complete every existing theme-aware Canvas-versus-Plot ownership route before
exposing a non-default root to callers.

Scope:

- Route existing Canvas title/subtitle/caption blocks, text panels, shared
  headers, row titles, and root-spacing consumers from the one resolved root.
- Remove first-Plot or hidden Publication ownership from shared-header and
  row-title measurement, arrangement, and rendering.
- Route existing theme-aware Plot title, subtitle, axis, panel, mark, annotation,
  and facet consumers from the selected Plot value; facets add no theme scope.
- Ensure every measurement helper receives the same selected resolved reference
  later used by arrangement and rendering.
- Carry inherited-root versus explicit-Plot provenance into shared-legend entry
  planning without routing hardcoded legend chrome through the current theme or
  claiming final T5b/T5c/T5d/T6 compatibility.
- Leave backend surfaces, new component properties, legend typography and
  chrome, and hardcoded style centralization with their assigned T5 phases. T4
  establishes ownership and provenance only where no final target property
  carrier exists yet.
- Exercise a private non-Publication Canvas root in focused module tests to prove
  existing theme-aware Canvas ownership, child inheritance, explicit
  replacement, and measurement/rendering identity before public exposure.
- Delete first-child, repeated Publication, and late-resolution routes made
  obsolete by the owner plan.

Acceptance:

- A private non-Publication root reaches every existing theme-aware Canvas owner
  consumer and every unthemed child, while an explicit child replacement affects
  only that Plot.
- Reordering explicitly themed children cannot change Canvas-owned header or row
  title appearance or measurements.
- Shared-legend planning retains exact source provenance for every contributing
  Plot without value comparison.
- Existing public default behavior and exact dimensions remain unchanged; any
  unexpected snapshot change blocks the phase.
- Focused Canvas, facet, shared-header, row-title, text-panel, and legend-input
  tests pass.
- `just verify` passes before T4c is committed.

### Phase T4d: Public Attachment And Current Contract

**Status**: COMPLETE

Dependencies: T4c.

Expose the completed built-in Canvas inheritance feature and make its current
contract atomic.

Scope:

- Add `Canvas::theme(Theme) -> Self` as the sole public Canvas-root builder. It
  replaces the stored default root; it is optional and does not introduce a
  missing-root state.
- Retain `Plot::theme(Theme) -> Self` as complete Plot replacement. Omission is
  the only inheritance request; do not add clear, inherit, optional, root, or
  compatibility attachment variants.
- Add public integration evidence for omitted Canvas selection, explicit Canvas
  selection, inherited Plot selection, explicit Plot replacement, standalone
  default selection, facets, mixed children, and owner isolation across render
  calls.
- Atomically update `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` with the
  implemented built-in owner and inheritance contract. Mark only the T4-owned
  subset of `03_THEME_SYSTEM.md` current. T5 privately migrates component and
  scale consumers; T6 exposes the replacement values, components, colors,
  scales, and custom construction.
- Update examples and user documentation with default Canvas, inherited child,
  and explicit child-replacement examples using only the current built-in
  selector.
- Inspect and commit any intentional SVG snapshot changes with their source; T4
  does not otherwise authorize visual-default or dimension changes.

Acceptance:

- A caller can omit all theme calls, replace the Canvas root once, and replace
  any individual Plot without private access or an alternate attachment API.
- Public tests prove every T4-routed theme-aware Canvas consumer is root-owned
  and unthemed Plot/facet content inherits, while explicit Plot selection is a
  complete local replacement.
- Current and target specifications have no interval in which the public T4
  behavior is implemented but undocumented.
- Examples compile and focused public API, rendering, and snapshot tests pass.
- `just verify` passes before T4d and parent T4 are marked complete.

## Phase T5a: Layout Foundation, Surfaces, Typography, Titles, Axes, And Grid

**Status**: IN PROGRESS

Dependencies: T4.

Establish the private layout kernel while moving foundational non-data
appearance into the resolved theme. T5a is complete only through T5a0-T5a6d in
dependency order; no subphase may leave a dormant replacement kernel, temporary
selector, compatibility fallback, or second root-plan representation.

Parent acceptance:

- Every T5a measurement call returns one complete nonempty candidate collection
  or a typed recoverable `Unsatisfied` outcome.
- Ordinary Plot plans contain the selected root size, title text, ticks, axis
  gutters, data panel, and final arrangement before Null or concrete rendering.
- Layout, collision, geometry, and paint use identical resolved typography,
  spacing, glyphs, ticks, and axis geometry.
- Theme appearance cannot remove caller-provided semantic text.
- No public constraint, automatic-size, bounded-width, Output-dimension, theme-
  component, title-alignment, or tick-interval API is exposed early.
- Every intentional built-in output change names its final-ledger owner and has
  focused evidence plus an inspected snapshot committed with the source change.
- Focused layout, axes, typography, output, and visual-regression tests pass.
- `just verify` passes before T5a6d, T5a6, and parent T5a are marked complete.

### T5a0: Contract Adjudication And Execution Partition

**Status**: COMPLETE

Dependencies: T4.

Scope:

- Resolve the T5a/T5e selector boundary: T5a establishes the one complete
  total-key representation, exact-root selector, and reusable two-through-eight-
  pass convergence foundation plus generic cumulative accounting and reservation
  gates. T5e completes remaining suppliers, specialized frontier/cache inputs,
  the all-producer resource census, specialized producers, and automatic/bounded
  roots by extending that implementation.
- Make an empty exact tick label intentionally unlabeled while retaining its
  tick/grid position.
- Replace the fixed five-generated-tick ceiling with private span- and
  typography-dependent source generation.
- Admit one optional per-axis preferred data interval: additive linear units or
  integral Log10 decades. It tries the requested interval first, then only
  deterministic coarsenings, with a documented terminal midpoint source after
  every interval vector; exact ticks take precedence. Public connection belongs
  to T6, and no target-count or visual-density control is admitted.
- Make one Plot/Canvas root title-block alignment a `CompositionTheme` property,
  with `Start`, `Center`, and `End`; every built-in uses `Center`. Preserve all
  non-root anchors and root text rotation as renderer invariants.
- Bound canonical system-font discovery, retained source bytes, required-scalar
  census, occurrence construction, and glyph probing before measurement. Reuse
  the adjudicated `1_000_000` record/work magnitudes and derive the byte ceiling
  as the least power of two covering the admitted maximum RGBA backing store;
  add no unrelated resource magnitude.
- Reconcile the target-only and existing-key overlay counts and partition
  T5a1-T5a6d with exact scope, exclusions, deletions, evidence, and commits.

Exclusions:

- No production code, public API, test, fixture, or snapshot changes.

Acceptance:

- `03_THEME_SYSTEM.md`, `04_VISUAL_PROPERTY_OWNERSHIP.md`,
  `05_LAYOUT_SYSTEM.md`, `91_IMPLEMENTATION_PLAN.md`, and
  `99_DEFERRED_FEATURES.md` state one consistent target.
- The materialized ledger contains 1860 historical plus 24 target-only keys,
  exactly 1884 final keys, with 1552 existing overlay keys partitioned into 62
  overrides and 1490 retained dispositions.
- Outside historical rows superseded by the canonical overlay, search audits
  confirm adaptive generated-tick density, centered built-in root titles,
  geometry-free empty exact labels, the sole selector beginning in T5a, and no
  public tick-count target.
- `git diff --check` and focused independent specification review pass. This
  documentation-only subphase does not run the test suite.

Commit: `docs(layout): adjudicate T5a implementation partition`.

### T5a1: Active Exact-Root Candidate Kernel

**Status**: COMPLETE

Dependencies: T5a0.

Objective: establish the sole private measurement, arrangement, selection, and
exact-root pass driver before component migrations.

Scope:

- Add private `LayoutScalar`, normalized `LengthConstraint`,
  `LayoutConstraints`, `LayoutSize`, measurement and assigned-geometry contexts,
  `MeasuredLayout<LayoutPlan>`, structurally nonempty
  `MeasuredCandidates<LayoutPlan>`, `MeasurementOutcome`, and typed `FitReason`.
- Add canonical component addresses, candidate-local provisional boxes, complete
  structural signatures, candidate sorting/deduplication, and final promotion.
- Add the complete `RootQualityKey` representation and one generic selector.
  Rank families not yet supplied use their specified empty normalization, never
  a temporary key or first-fit branch.
- Add the reusable convergence driver: two through eight complete passes,
  immediate-predecessor equality success, non-adjacent repetition failure, and
  changed-pass-eight exhaustion.
- Add the one root-operation resource state, checked cumulative counters, and
  reservation driver for candidate evaluations, generation work/slots,
  measurement calls/cache entries, topology slots, and the five ordered
  reservation stages. Every producer activated in T5a must count and reserve
  through these gates before materialization; later phases add producers rather
  than a second accounting route.
- Add the operation-scoped measurement cache keyed by normalized constraints,
  optional assigned context, and the complete frontier subset active in T5a;
  cache `Unsatisfied` outcomes and their fixed accounting tuple. T5b-T5e extend
  the typed frontier key as they activate new inputs rather than replacing the
  cache protocol.
- Route the current standalone Plot lifecycle through one typed exact-root
  adapter and size-owning root envelope while preserving current `640x480` or
  explicit exact dimensions. The adapter must return the same candidate and root
  plan representations used by native components.
- Make the promoted root size the sole source for Null, SVG backend, and Scene
  construction; no post-selection path rereads Plot dimensions.

Exclusions:

- Public automatic or bounded sizing; specialized and all-producer
  frontier/cache identities; native Canvas/facet measurement; complete quality
  suppliers; fonts, text, axes, and specialized producers.

Deletion targets:

- Direct root finalization outside the selector, source-dimension rereads after
  promotion, and any alternate root-plan/backend route replaced by the active
  envelope.

Evidence:

- Quantum conversion, normalized interval, overflow, `Unsatisfied`, nonempty
  collection, complete child Cartesian product, no-local-pruning, structural
  equality, deterministic selection, pass-two/pass-eight, cycle, exhaustion,
  promotion identity, cumulative limit-minus-one/exact-limit/limit-plus-one, each
  active reservation refusal, and zero-backend-call failure tests.
- Current omitted-size and explicit-size SVG dimensions and bytes remain exact.
- `just verify` passes.

Commit: `feat(layout): add exact-root planning kernel`.

### T5a2: Foundational Resolved Appearance And Surfaces

**Status**: COMPLETE

Dependencies: T5a1.

Objective: give every T5a appearance input one private resolved owner before it
participates in measurement.

Scope:

- Extend the existing `ResolvedTheme`; do not create a parallel private theme.
  Add private foundation records for Canvas/panel surfaces, title/axis text roles,
  axis/frame/tick/grid chrome, root title-block relationships and alignment, and
  role-local spacing. Include one effective significance-label text carrier that
  combines the current resolved stack/paint, the ledger-assigned T5a font-size
  and weight defaults, and the retained exact caller size/weight precedence.
  T5c later rewires annotation-theme stack/paint ownership into this same carrier
  before planning; it does not replace typography after measurement.
- Map the current four built-ins into complete private records. Every built-in
  root title-block alignment is `Center`.
- Normalize T5a-owned paints into one private RGBA carrier and install the sole
  SVG paint/alpha serializer. Connect Canvas root and Plot panel surfaces without
  exposing T6 `Color` or component types.
- Separate title, subtitle, caption, axis-title, and tick-label content presence
  from appearance. Preserve exact caller content independently of theme values.
- Preserve T4 owner selection: Canvas root appearance is Canvas-owned, composed
  panel appearance is Plot-owned, and a Plot never repaints the Canvas root.

Exclusions:

- Legend, mark, annotation gap/geometry or general annotation appearance,
  gradient, scale, and public RGBA completion; custom public themes; public
  alignment. The effective T5a significance-text carrier above is the sole
  annotation exception and does not expose a new style surface.

Deletion targets:

- Backend hardcoded white, dormant flat background routing, hardcoded title
  grays, semantic-presence theme flags, and duplicate RGB/opacity serialization
  for migrated paints.

Evidence:

- All-preset preservation; transparent Canvas and panel SVG/PNG; panel-over-
  Canvas compositing; one alpha emission; inherited/explicit owner routing;
  semantic-content presence; and private Start/Center/End resolution tests.
- `just verify` passes.

Commit: `refactor(theme): centralize foundational appearance`.

### T5a3: Canonical Fonts And Immutable Text Plans

**Status**: COMPLETE

Dependencies: T5a2.

Objective: make measurement, collision, preserve-text SVG, outlined SVG, PNG,
and drawing consume the same generation-host faces, glyphs, lines, and metrics.

Scope:

- Census every used `(stack,weight)` in canonical topology order before text
  measurement. Build, count, reserve, and materialize canonical font occurrence,
  candidate, identity, primary-face, glyph-fallback, and provenance records under
  the exact `03_THEME_SYSTEM.md` contract.
- Install the bounded project-owned system-font discovery path and enforce the
  `1_000_000` discovery-entry, `536_870_912` inspected/hashed-byte, and
  `1_000_000` font-preflight-work limits before font-sized collections
  materialize. The record/work limits reuse the T3j magnitudes; the byte limit is
  the least power of two covering the admitted `4 * 100_000_000` RGBA backing
  store. Through the T5a1 resource driver, reserve all stage-1 font metadata,
  source-byte, occurrence, candidate, selected-face, glyph-probe, and provenance
  payloads.
- Resolve every scalar face before measurement and replace estimated `TextBlock`
  geometry with immutable text plans containing exact source spans, shaped runs,
  face identities, glyph IDs, clusters, offsets, advances, logical widths, ink,
  line translations, line pitch, and first/last baselines.
- Implement the pinned Unicode segmentation, line-break, bidi, whitespace,
  mandatory-break, wrapping-alternative, and shaping rules. An intact overwide
  sequence returns a typed fit miss rather than an unwrapped fallback.
- Retain the prepared font environment needed by outlined SVG and PNG so a
  terminal never rediscovers different host faces after Scene construction.
- Convert every T5a foundational one-line or multiline text producer to the same
  plan representation; no raw foundational text bypass remains.
- Introduce a typed `DeferredAnnotationTextPlan` and freeze significance-label
  glyphs, metrics, current effective style, and exact-override provenance through
  it. T5c supplies final semantic appearance and geometry inputs before this same
  planner runs; no later phase may remeasure or replace typography after plan
  selection.

Exclusions:

- Public fonts or typography components, downloaded or provisioned assets,
  legend/facet/body roles owned by T5b, and annotation appearance or geometry
  owned by T5c.

Deletion targets:

- Fixed character-width tables, `estimated_text_width`, integer font adapters,
  arbitrary first-face fallback, process-global generic-family remapping, late
  output-only font discovery, `TextBlock::unwrapped`, draw-time wrapping, and raw
  foundational text geometry.

Evidence:

- Font name/occurrence/identity/order/deduplication, primary and per-scalar
  fallback, discovery-order invariance, discovery-entry/byte/work boundaries,
  complete prepared/generated scalar census and traversal, changing/short file
  failures, missing family/glyph failures, exact reservation, whitespace, blank
  lines, bidi, non-Latin, weight, baseline, ink, overwide sequence, significance-
  label effective/exact carrier identity, immutable replay, and
  preserve/outline/PNG parity tests.
- Host-dependent smoke evidence asserts provenance and consistency rather than
  universal glyph bytes; no network or asset provisioning is added.
- `just verify` passes.

Commit: `feat(typography): freeze resolved text plans`.

### T5a4: Root Title-Block Plans

**Status**: PENDING

Dependencies: T5a3.

Objective: establish the reusable root title-block producer and migrate the
standalone Plot root without requiring composition adapters early.

Scope:

- Build immutable title-block candidates from resolved text roles,
  `CompositionTheme` spacing, and one root `TitleBlockAlignment` value.
- Apply alignment consistently to per-line translation and horizontal child
  placement for title, subtitle, and caption. Retain zero rotation.
- Migrate the standalone Plot root. T5a6a connects the same producer to faceted
  Plot, Matrix, and Canvas outer-root adapters after those boundaries exist.
- Store exact lines, widths, baselines, paints, spacing, boxes, and serialization
  inputs before Null validation.

Exclusions:

- Faceted Plot, Matrix, and Canvas outer-root integration before T5a6a; panel
  headers, Canvas headers, row titles, text panels, facet track placement, and the
  public T6 alignment type/setter.

Deletion targets:

- Standalone-root `TitleBlock::draw` wrapping, line/baseline `zip`, hidden font
  fallback, hardcoded muted paints, duplicate root gaps/reserves, and draw-time
  title style reconstruction. T5a6d removes the corresponding deferred outer-root
  route after every adapter consumes this producer.

Evidence:

- Title-only, subtitle-only, caption-only, all-role, unequal multiline,
  Start/Center/End, all-built-ins-centered, standalone owner selection, presence,
  baseline cardinality, Null/SVG identity, and explicit-size snapshot tests.
- `just verify` passes.

Commit: `feat(layout): plan root title blocks`.

### T5a5: Axis, Range, Tick, Frame, And Grid Plans

**Status**: PENDING

Dependencies: T5a4.

Objective: replace local axis selection/retry behavior with complete immutable
alternatives consumed only by the root selector.

Scope:

- Add immutable range, projection, tick, axis-title, axis, frame, and grid plans
  using the resolved T5a appearance and text plans.
- Validate a present exact-tick list as one indivisible `1..=10_000` alternative;
  reject a present empty list rather than selecting automatic generation.
  Preserve intentionally empty labels as unlabeled tick/grid positions with no
  text geometry.
- Implement private span/typography adaptive numeric sources with no fixed five-
  tick ceiling and one always-enumerated midpoint source.
- Implement optional additive data intervals and Log10-decade intervals. Try the
  requested source first, enumerate deterministic `1,2,3,5` decimal coarsening,
  retain complete nonempty interval vectors without subsequences, order the
  terminal midpoint after every interval vector, and rank source precedence
  before all soft quality only at the root. Retain tick policy/source kind/raw
  ordinal in structural signatures and frozen axis plans so equal vectors from
  distinct sources do not deduplicate. Keep caller count/density controls absent.
- Implement exact formatting/uniqueness, projection, categorical one-label-per-
  slot behavior, Forest logarithmic sources, fit transitions, gutters, rotations,
  frame-side topology, tick reserve/draw synchronization, and grid ordering.
- Preserve automatic-range broadening's current private five-sample nice-range
  seed under its range owner; remove only the shared tick-count meaning and use
  no count target in generated-tick selection.
- Populate the standard-panel carrier with T5a typography, tick, axis,
  projection, and preferred `4:3` inputs while leaving mark/density/annotation
  inputs open for T5c.
- Count and reserve every raw tick source, text-product block, fit transition,
  axis candidate, and plan payload through the T5a1 operation resource state
  before materializing it.
- Replace horizontal categorical helper duplicates with canonical raw/display
  labels and the common text/axis path.
- Add the Canvas shared-x semantic validator: configured continuous automatic
  members only, finite strict union, categorical or exact-limit rejection. T5b
  owns child-plan/track integration.
- Replace ordinary axis/title local stabilization with the T5a1 root pass
  protocol; do not introduce another loop.

Exclusions:

- Legend clearance, Canvas/facet track measurement, mark ink/density/category
  pitch/annotations, complete Matrix/Marginal packing, full domain feedback,
  automatic roots, and public interval/alignment APIs.

Deletion targets:

- The shared `AUTOMATIC_TICK_TARGET` identity, five-only tick downsampling, and
  conditional midpoint fallback branches; retain its range-only value under a
  range-specific private owner.
- Draw-time axis text measurement, local axis/title retry loops, duplicate
  spacing/line-height estimates, measured-state booleans, horizontal categorical
  helpers, and Canvas shared-x clone/replay.

Evidence:

- Wide panels with more than five generated ticks; small panels; source-span,
  `6ch`, `10_000/10_001`, representable-index boundaries, interval
  exact/coarsened/nonempty/no-subsequence/midpoint order, exact precedence,
  source precedence over soft ranks, Log10, formatting, uniqueness, finite
  extremes, exact-list `0/1/10_000/10_001`, empty exact labels, category order,
  collision, projection endpoints, frame sides, Science duplicate removal,
  tick/grid reserve-draw parity, horizontal categorical fit, shared-x validation,
  accounting refusal, pass stabilization, and permutation tests.
- `just verify` passes.

Commit: `feat(axes): freeze axis and grid plans`.

### T5a6: Ordinary Plot Exact-Root Migration And Closure

**Status**: PENDING

Dependencies: T5a5.

Objective: complete the native ordinary standalone Plot path and close every
T5a-owned source without absorbing later component phases. This umbrella is
complete only after T5a6a-T5a6d in dependency order and has no separate commit.

Exclusions across T5a6:

- T5b legends/facets/Canvas tracks and chrome; T5c final mark and annotation
  appearance/measurement/geometry; T5d scales/guides; T5e specialized native
  producers and automatic/bounded roots; T6 public APIs/default switch.

#### T5a6a: Ordinary Interior Carriers And Deferred-Family Adapters

**Status**: PENDING

Dependencies: T5a5.

Objective: establish closed native/deferred interior boundaries before migrating
any family route.

Scope:

- Add the standard-coordinate and one-categorical-axis panel producer inputs and
  integrate the T5a standard-panel carrier into the Plot frame candidate tree.
- Add exactly one typed `DeferredMarkAdapter` producing `DeferredMarkPlan` and
  one typed `DeferredAnnotationAdapter` producing `DeferredAnnotationPlan` from
  T5a3's `DeferredAnnotationTextPlan`. Before root selection they emit candidate-
  local current hard-panel minima, automatic-domain envelopes, reserve requests,
  obstacle/outward-ink extents, and frozen semantic/style/draw payloads. Their
  consequences feed the canonical range, axis, collision, and parent candidate
  routes; they cannot select a tick/root candidate, mutate an assigned panel,
  perform work during drawing, or call a layout backend.
- Name the current generic panel floor, where still required for parity, as the
  sole `DeferredLegacyPanelMinimum` consequence. It remains only until T5c
  replaces it with exact mark density, category pitch, interval, and annotation
  requirements; no producer may consult an unnamed generic floor.
- Add closed standard-coordinate and one-categorical-axis legacy interior
  adapters covering all 20 ordinary PlotKinds. T5a6b1-T5a6c4 replace those
  exhaustive variants cluster by cluster; no ordinary route is temporarily
  unclassified.
- Add one typed interior adapter each for Heatmap, Marginal, Matrix, Beeswarm, and
  Waterfall until T5e. Add one typed parent-composition adapter each for facet and
  Canvas until T5b, regardless of child PlotKind. Connect faceted Plot, Matrix,
  and Canvas outer roots to the T5a4 title-block producer through these adapters.
  No generic fallback adapter or runtime downcast is admitted.
- Make every carrier and adapter use the T5a1 candidate, accounting, reservation,
  convergence, selected-plan, Null, and concrete lifecycle representations.

Deletion targets:

- Unclassified interior dispatch, adapter-specific root selection, outer-root
  title reconstruction, and any carrier that can invoke `GeometryBackend` for
  measurement.

Evidence:

- Compile-time exhaustiveness over native and deferred route kinds; all 20
  ordinary legacy variants; five specialized/two composition adapters; effective
  significance text reuse; legacy hard-minimum/domain/reserve/obstacle parity;
  frozen deferred payloads; accounting/refusal; selected-panel and clip identity;
  no-layout-backend-call; and pre-backend failure tests.
- `just verify` passes.

Commit: `feat(layout): add ordinary interior carriers`.

#### T5a6b: Standard-Coordinate Ordinary Plot Migration

**Status**: PENDING

Dependencies: T5a6a.

Objective: migrate the ordinary two-numeric-axis family in T5a6b1-T5a6b2. This
umbrella has no separate commit and completes only after both clusters.

Requirements shared by both clusters:

- Split semantic preparation from final replay so each renderer consumes the
  selected root, title, axes, ticks, projection, data panel, clips, and immutable
  draw payload rather than discovering foundational layout during drawing.
- Replace its closed legacy interior variant with the native standard-coordinate
  producer. Retain only the named `DeferredLegacyPanelMinimum` and typed
  mark/annotation consequences needed for current parity until T5c; no family-
  local or unnamed generic minimum remains.
- Preserve current exact default dimensions and `.size`; validate one selected
  `PlotPlan` through one Null lifecycle and replay the same plan through one
  concrete lifecycle.

##### T5a6b1: Standard Point And Path Families

**Status**: PENDING

Dependencies: T5a6a.

Scope:

- Natively migrate Scatter, Line, Trajectory, and LinePath through the standard-
  coordinate panel producer and remove exactly their four legacy variants.

Deletion targets:

- These families' fixed single-panel constructors, `GeometryBackend` layout
  discovery, duplicate panel/tick selection, and draw-time foundational
  layout/style/tick reconstruction.

Evidence:

- One route assertion and focused exact-size, range, projection, axis, deferred-
  consequence, frozen-plan, Null/concrete identity, pre-backend failure, and
  snapshot test per family.
- `just verify` passes.

Commit: `refactor(layout): migrate point and path plans`.

##### T5a6b2: Standard Distribution And Reference Families

**Status**: PENDING

Dependencies: T5a6b1.

Scope:

- Natively migrate Histogram, Density, Qq, and Regression through the standard-
  coordinate panel producer and remove exactly their four legacy variants.

Deletion targets:

- These families' fixed single-panel constructors, local range/tick retries,
  `GeometryBackend` layout discovery, duplicate panel selection, and draw-time
  foundational reconstruction.

Evidence:

- One route assertion and focused exact-size, scientific-range, projection, axis,
  deferred-consequence, frozen-plan, Null/concrete identity, pre-backend failure,
  and snapshot test per family; all eight standard routes are then native.
- `just verify` passes.

Commit: `refactor(layout): migrate distribution reference plans`.

#### T5a6c: One-Categorical-Axis Ordinary Plot Migration

**Status**: PENDING

Dependencies: T5a6b2.

Objective: migrate the ordinary categorical family through one orientation-aware
producer in T5a6c1-T5a6c4. This umbrella has no separate commit and completes only
after all four clusters.

Requirements shared by all four clusters:

- Preserve resolved category order, horizontal/vertical orientation, canonical
  raw/display labels, categorical midpoint identity, and Forest's sole family-
  owned Log10 preliminary source while using the common axis plans.
- Replace each closed legacy variant with the native one-categorical-axis
  producer. Retain only the named `DeferredLegacyPanelMinimum` and typed current
  mark/annotation range, reserve, collision, and draw consequences until T5c.
- Freeze selected projections, categorical positions, clips, and deferred mark
  and annotation plans before Null validation.

##### T5a6c1: Estimate-Display Families

**Status**: PENDING

Dependencies: T5a6b2.

Scope:

- Natively migrate PointInterval, Coefficient, Forest, and Dumbbell and remove
  exactly their four legacy variants.

Deletion targets:

- Their family-local categorical axis measurement, exact/automatic range/tick
  retries, `GeometryBackend` layout discovery, and duplicate panel selection.

Evidence:

- Per-family route, orientation, unavailable-row, exact-size, Forest Log10,
  deferred annotation/domain consequence, frozen-plan, lifecycle, and snapshot
  evidence; `just verify` passes.

Commit: `refactor(layout): migrate estimate display plans`.

##### T5a6c2: Bar Families

**Status**: PENDING

Dependencies: T5a6c1.

Scope:

- Natively migrate Bar, HorizontalBar, StackedBar, and StackedHorizontalBar and
  remove exactly their four legacy variants.

Deletion targets:

- Their orientation-local categorical axes, horizontal label helpers, local
  range/tick retries, `GeometryBackend` layout discovery, and duplicate panels.

Evidence:

- Per-family route, orientation, sign/stack, exact-size, attached-annotation
  range/reserve consequence, frozen-plan, lifecycle, and snapshot evidence;
  `just verify` passes.

Commit: `refactor(layout): migrate bar family plans`.

##### T5a6c3: Categorical Distribution Families

**Status**: PENDING

Dependencies: T5a6c2.

Scope:

- Natively migrate Boxplot, Violin, and Raincloud and remove exactly their three
  legacy variants.

Deletion targets:

- Their family-local categorical axis measurement, local range/tick retries,
  `GeometryBackend` layout discovery, and duplicate panel selection.

Evidence:

- Per-family route, orientation, group/category order, exact-size, deferred mark
  extent, frozen-plan, lifecycle, and snapshot evidence; `just verify` passes.

Commit: `refactor(layout): migrate categorical distribution plans`.

##### T5a6c4: Interaction Family

**Status**: PENDING

Dependencies: T5a6c3.

Scope:

- Natively migrate Interaction and remove its final ordinary legacy variant.

Deletion targets:

- Interaction-local categorical axis/tick measurement, local range retry,
  `GeometryBackend` layout discovery, and duplicate panel selection.

Evidence:

- Condition order, sparse connector, interval, exact-size, deferred consequence,
  frozen-plan, lifecycle, route-exhaustiveness, and snapshot evidence proves all
  12 categorical and all 20 ordinary routes are native; `just verify` passes.

Commit: `refactor(layout): migrate interaction plans`.

#### T5a6d: Ordinary Exact-Root Closure Audit

**Status**: PENDING

Dependencies: T5a6c4.

Objective: close all T5a materialization and prove that later phases can only
extend the active route.

Scope:

- Route T5a-owned Canvas root surfaces/title inputs and foundational specialized
  axis consumers through the same plans without migrating T5b tracks or T5e
  interiors.
- Audit every materialized T5a ledger key from source through measurement,
  candidate generation, accounting/reservation, arrangement, frozen plan, and
  draw; remove every superseded T5a path.
- Prove the five specialized interior adapters, two composition adapters, and
  mark/annotation deferred adapters plus their named legacy-panel-minimum
  consequence are the only remaining nonnative boundaries.
- Preserve explicit `640x480` Plot and `960x640` Canvas behavior while leaving
  public automatic and bounded sizing dormant until their atomic T6 switch.

Deletion targets:

- Remaining ordinary `CanvasLayout::measured_single_panel`, unchecked/clamping
  foundational fallback, source-dimension rereads after promotion, hidden
  Publication/background/font fallback, deferred outer-root title wrapping,
  local ordinary retry loops, and every alternate ordinary exact-root route.

Evidence:

- Exhaustive native/adapted PlotKind route census; explicit default fixtures;
  multiple exact sizes; complete T5a-content fit/failure; frozen-plan fields;
  no draw-time foundational layout; Null/concrete identity; inherited
  theme/surface/title behavior; resource/accounting boundaries; focused output,
  family, and snapshot tests.
- Search evidence proves no fixed five-tick ceiling, ordinary generic panel floor
  outside a named deferred adapter, draw-time foundational wrapping, local
  ordinary retry loop, or second root route remains.
- `just verify` passes; T5a0-T5a6d, T5a6, and parent T5a are then marked complete.

Commit: `refactor(layout): close ordinary exact-root planning`.

## Phase T5b: Legends, Facets, And Canvas Composition Chrome

**Status**: PENDING

Dependencies: T5a.

Give composed and explanatory chrome coherent theme ownership.

Scope:

- Prepare legend semantic entries independently from layout and measure finite
  complete row/column alternatives using final typography and exact glyphs;
  return all admitted candidates to parent combination rather than selecting a
  local flow.
- Replace equal-cell subtraction with measured row and column tracks that
  preserve shared data-panel alignment separately from unequal decoration
  gutters.
- Centralize legend container, title, label, key, outline, opacity, padding, and
  spacing styles while keeping visibility and placement in legend policy.
- Apply role-based typography and surfaces to facet strips, panel headers, shared
  headers, row titles, text panels, and shared legends.
- Preserve exact legend glyph reproduction from resolved mark styles.
- Keep collision planning, outside reservation, and shared-legend compatibility
  synchronized with the resolved theme.
- Establish one kind-discriminated shared-guide descriptor and failure path.
  Reject mixed kinds and incompatible discrete or continuous descriptors before
  measurement; do not select one Plot's labels, gradient, or domain.
- Resolve outside spacing once from `LegendTheme` and reuse it for reservation
  and all four directional placements; do not reconstruct helper-local gaps.
- Complete Canvas shared-x integration after T5a range validation without
  overriding explicit limits or admitting categorical domains.
- Remove implicit Publication styling and duplicated fixed whites and grays from
  composition code.
- Make Canvas `TextPanel` participate in bottom-up track measurement before
  arrangement. Keep Plot panel-text annotations in the overlay path.
- Remeasure constraint-dependent children when track constraints change; freeze
  final headers, row titles, shared legends, text lines, cells, and child plans
  before Canvas rendering.
- Preserve declared sparse coordinates, spans, groups, and facet topology. Do
  not compact empty rows or columns implicitly.

Acceptance:

- Legend and composition measurement consume the exact final styles.
- Shared and per-plot legends use one component contract while retaining exact
  mark glyphs.
- Mixed explicit Plot themes have deterministic documented behavior.
- Facet and Canvas plans have one measured root size and perform no layout
  reconstruction during final drawing.
- T5b establishes one comparison and failure path and proves it against the
  current inputs. T5c and T5d feed their final values into that same path, and T6
  closes mixed-theme compatibility over the public replacement values.
- Focused legend, facet, Canvas, collision, and visual-regression tests pass.
- `just verify` passes.

## Phase T5c: Mark Defaults, Annotations, And Local Styles

**Status**: PENDING

Dependencies: T5b.

Unify common mark roles without erasing typed local control.

Scope:

- Complete the T5a measurement carrier with exact mark ink extents, data-density
  requirements, category and cell pitch, interval clearance, annotation reserve,
  and family-specific hard or preferred panel requirements.
- Replace `DeferredMarkAdapter`, `DeferredAnnotationAdapter`, and
  `DeferredLegacyPanelMinimum` atomically with those native measured
  consequences while retaining the same T5a candidate, range, collision,
  convergence, accounting, and frozen-plan route.
- Separate normal-flow measurement from inside-legend, significance, value-label,
  panel-text, and supplemental-layer overlay planning.
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
- Establish the typed annotation-geometry input for exact significance
  rendered-target/facet-panel resolution; T5d supplies the final resolved target
  through that same input. Make shared KDE-support widening reject any result
  that is still nonfinite or constant.
- Integrate existing typed local APIs into the approved precedence model rather
  than adding parallel replacements.
- Preserve exact legend glyph agreement with the final resolved mark styles.
- Keep scale-domain expansion distinct from box overflow. Exact limits reject an
  incompatible reserve; automatic expansion remains explicit, bounded, and
  recorded in the frozen plan.
- Re-adjudicate default elision, clipping, label dropping, and post-allocation fit
  branches so automatic layout preserves semantic content unless an explicit
  current caller policy already identified in the ledger owns a narrower
  behavior. Do not create a new omission policy in implementation.

Acceptance:

- Ordinary renderers no longer duplicate unclassified widths, radii, opacities,
  outlines, fills, and neutral colors.
- Exact local styles override only their documented properties and scope.
- Renderer invariants are explicit and cannot be changed indirectly through a
  theme field.
- Mark and annotation geometry that affects fit is measured before arrangement
  or represented as an explicit overlay/domain request; it does not enlarge the
  scene during drawing.
- Focused renderer-family, layer, annotation, legend, and snapshot tests pass.
- `just verify` passes.

## Phase T5d: Categorical And Continuous Data Scales

**Status**: PENDING

Dependencies: T5c.

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
- Feed exact final categorical domains, legend cardinality, continuous-guide
  labels, missing-value sections, and heatmap/beeswarm guide content into the
  typed layout measurements established in T5a-T5c.
- Supply final continuous shared-guide contributor descriptors and perform the
  exact automatic-domain union or equal-exact/contained-automatic merge through
  T5b's sole compatibility path before labels or guide measurement.
- Supply exact significance rendered-target/facet-panel resolution to the typed
  annotation-geometry input established in T5c.

Acceptance:

- Theme appearance and data scales are distinct types and resolution steps.
- Custom mappings never silently cycle, substitute, or fall back contrary to the
  Phase T1 contract.
- Legends, marks, gradients, colorbars, facets, and Canvas share the same
  resolved mappings.
- Data-dependent guides and labels contribute one measured size before final
  arrangement and are not reconstructed from a different mapping during draw.
- Focused categorical, continuous-scale, heatmap, beeswarm, facet, Canvas, and
  legend tests pass.
- `just verify` passes.

## Phase T5e: Scientific Layout Closure

**Status**: PENDING

Dependencies: T5d.

Complete private automatic layout only after every appearance, semantic,
geometry, and scale input can participate in one measurement protocol.

Scope:

- Complete family-specific requirements for horizontal categorical plots,
  waterfall, heatmap, beeswarm, marginal composition, Matrix variable grids, and
  every other specialized self-managed layout.
- Replace fixed marginal bands, Matrix dummy generic layout, hardcoded guide
  reserves, and universal panel minima with typed complete composite
  measurements.
- Complete the T5a root quality implementation with every remaining supplier for
  semantic completeness, hard geometry, scientific aspect, readable density,
  compactness, and stable structural tie-breaking.
- Complete every closed candidate producer, dense-rank pool, and nonexact
  eligibility/viability filter on the existing Cartesian-combination, signature,
  duplicate, and selector route without local pruning or discovery-order choice.
- Complete specialized and all-producer frontier/cache identity and bounded
  remeasurement for wrapping, ticks, legend flow, grid tracks, shared alignment,
  domains, and overlays on the existing T5a convergence driver. Detect repeated
  nonfinal states and budget exhaustion explicitly.
- Resolve one private automatic root size, bounded-width root size, or exact root
  size before backend creation. Convert the finite logical result once through
  checked upward CSS-pixel rounding and store it in `PlotPlan` or `CanvasPlan`.
- Complete the T5a resource state's producer census and enforce the T3j automatic
  resource envelope over every T5b-T5e producer, checked topology arithmetic,
  and allocation limit without clamping, content omission, fixed-size fallback,
  or output-format-specific mutation of the canonical scene.
- Remove remaining final-render text layout, tick selection, legend construction,
  spacing derivation, track sizing, and domain mutation.
- Exercise automatic and bounded-width behavior through private focused tests,
  but retain the current public default and getters until T6 performs the one
  public switch.

Acceptance:

- Every admitted Plot family and Canvas child returns one complete nonempty typed
  candidate collection or rejects its unsupported composition shape before
  candidate generation; only root selection chooses one complete plan tree.
- Automatic and bounded-width private plans are finite, deterministic,
  publishable under the approved quality objective, and complete in semantic
  content.
- Exact dimensions remain exact and fail rather than clip when complete content
  cannot fit.
- Null and concrete rendering consume the same immutable root and child plans.
- Focused ordinary, categorical, specialized, facet, Canvas, convergence,
  resource, output, and snapshot tests pass.
- `just verify` passes.

## Phase T6: Public Immutable Theme, Scale, Tick-Interval, And Sizing API

**Status**: PENDING

Dependencies: T5e.

Expose the complete foundations only after every admitted consumer uses the
private resolved model.

Scope:

- Replace the closed public theme selector with the approved immutable complete
  `Theme` API and built-in constructors or presets.
- Expose only validated component theme, color, typography, and scale types
  required by the full-foundations milestone.
- Expose `TitleBlockAlignment::{Start,Center,End}` and one
  `CompositionTheme::root_title_block_alignment` property. Every built-in
  constructor returns `Center`; do not expose general `TextTheme` alignment,
  independent title/subtitle/caption alignment, backend anchors, or separate
  Plot/Canvas alignment setters.
- Retain `Canvas::theme(Theme)` and `Plot::theme(Theme)` as the sole owner
  attachments established by T4 while replacing the closed selector arguments
  with the approved complete immutable `Theme` values. Migrate repository
  callers atomically; do not add aliases or a second attachment surface.
- Provide a typed way to derive a modified complete theme from a built-in base.
- Migrate current local style and categorical color callers to the one approved
  precedence model; delete replaced public and private representations.
- Keep backend styles private and avoid exposing renderer implementation details.
- Expose opaque crate-root `TickInterval: Clone + Copy + Debug + PartialEq` with
  validated `additive(f64) -> Result<Self>` and
  `log10_decades(u32) -> Result<Self>`, plus consuming
  `Plot<K>::x_tick_interval(TickInterval) -> Self where K: kind::NumericX` and the
  corresponding `NumericY` y setter. Repeated setters replace; a present exact
  list takes precedence regardless of order and validates `1..=10_000`; absence
  retains automatic span/typography policy. Do not expose a target count, maximum
  count, visual-density scalar, categorical stride, or tick-generation callback.
- Make absence of a sizing call select automatic layout for Plot and Canvas.
- Add positive `max_width(width)` builders for bounded-width automatic height,
  retain exact `size(width, height)`, and keep the complete constraint protocol
  private.
- Remove pre-render `Plot::width()` and `Plot::height()`. Add
  `Output::width()` and `Output::height()` as exact canonical CSS-pixel scene
  dimensions that remain stable across chained publications.
- Do not expose a pre-render measurement API, size-request enum, public layout
  module, constraint object, callback, or compatibility getter.
- Close the complete RGBA carrier migration through the sole per-paint SVG alpha
  serializer, gradient stops, text/bracket paints, paint-aware geometry, and PNG
  conversion. RGB token spelling remains a serialization dependency, not an
  alpha owner.
- Atomically update `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` to the
  implemented theme and layout contracts, mark `03_THEME_SYSTEM.md` and
  `05_LAYOUT_SYSTEM.md` current, and reconcile `99_DEFERRED_FEATURES.md`; no
  target/current specification interval remains.
- Update examples and user documentation with standalone, Canvas inheritance,
  Plot replacement, palette, exact local style, automatic sizing, bounded-width
  wrapping, exact sizing, and rendered-dimension examples.

Acceptance:

- Callers can construct one complete custom theme covering the admitted full
  foundations without editing private renderer state.
- A caller can derive a complete theme with Start, Center, or End root title-
  block alignment; Canvas/Plot ownership and inheritance select the expected
  value, and every built-in remains centered.
- A caller can request an additive linear or Log10-decade preferred tick interval
  independently on x and y; focused evidence proves exact-source preference,
  deterministic coarsening, terminal midpoint order, scale mismatch, exact-tick
  precedence, setter replacement, and absent-interval automatic behavior.
- Built-in Publication, Nature, Science, and CHI behavior remains available
  through the single current API.
- The API has no ambiguous global default setter, generic property map, detached
  mutable palette, or alternate compatibility identity.
- Public constructors validate every structural invariant before rendering.
- Automatic and bounded-width output report their final dimensions only through
  `Output`; exact sizing remains exact and has no nominal pre-render getters.
- Compile-pass and compile-fail evidence enforces the intended visibility and
  ownership boundaries.
- All canonical specifications describe the implemented replacement API before
  the phase is complete.
- `just verify` passes.

## Phase T7: Contract Evidence And Final Audit

**Status**: PENDING

Dependencies: T6.

Close the phase with exact behavioral evidence and no residual old model.

Scope:

- Add precedence tests for library default, Canvas inheritance, explicit Plot
  replacement, plot scale configuration, and exact element styles.
- Prove one resolved theme is used by preflight and final rendering.
- Cover shared legends, facets, text panels, mixed explicit Plot themes, palette
  capacity, missing mappings, canonical font candidate ordering/deduplication/
  provenance, shared continuous-guide compatibility/domain merging, contrast
  behavior, SVG, and PNG.
- Cover automatic, bounded-width, and exact dimensions across ordinary,
  specialized, facet, sparse Canvas, shared-guide, SVG, and PNG routes.
- Cover root title-block alignment defaults/customization and inheritance, plus
  automatic/additive/Log10-decade/exact tick-policy precedence across standalone,
  facet, Canvas-child, SVG, and PNG routes.
- Retain reviewed deterministic snapshots for every built-in preset and selected
  custom themes.
- Search for retired theme APIs, repeated hidden Publication defaults, dormant
  theme fields, unclassified hardcoded style values, and duplicate scale logic.
- Search for fixed root defaults, universal panel minima, root-scaled spacing,
  draw-time layout decisions, silent overflow, and alternate layout routes.
- Reconcile every key in the T3j-recalculated final ledger against implemented
  owners, current public names, focused evidence, and deletion of every
  superseded source. The historical 1836-record pre-T2h count and completed
  1860-record T2 count are not post-adjudication acceptance thresholds.
- Audit all specifications, README, guides, examples, and rustdoc for residual
  drift from the already-current implemented contract.

Acceptance:

- Phases T1 through T7 and all admitted T5 subphases are complete with commit and
  verification evidence.
- Every visual property has one implemented owner matching the approved
  T3j-recalculated inventory, with every source addition, split, consumer
  relationship, and explained exclusion re-audited.
- No mutable global theme, repeated hidden Publication default outside its
  owning boundary, late theme lookup, compatibility alias, or unclassified
  renderer style constant remains.
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
- `CompositionTheme` owns one Plot/Canvas root title-block alignment applied
  jointly to title, subtitle, and caption. Built-ins use `Center`; Start and End
  are available through complete-theme customization; other anchors remain
  renderer invariants.
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
- Fonts use typed ordered stacks and one canonical generation-host candidate list
  per used `(stack,weight)`, with exact identity-based ordering, deduplication,
  glyph fallback, and recorded environment provenance.
- Themes may omit nonsemantic chrome but cannot hide caller-provided semantic
  content. Existing explicit policies own content presence and placement.
- Automatic numeric ticks have no fixed five-tick or public count target. An
  optional per-axis additive or Log10-decade data interval is the sole public
  automatic-tick preference; exact ticks remain the strict override.
- Shared Canvas legends use Canvas chrome and exact panel glyphs; conflicting
  glyphs for one semantic identity reject.
