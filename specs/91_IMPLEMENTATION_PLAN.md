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
automatic layout, exact-width/automatic-height sizing, and rendered-dimension behavior remain
non-current until Phase T6. Neither target specification is a second current
API, and later implementation phases do not begin merely because they are
listed here.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each admitted phase as one logical change before starting the next
phase. Split a broad phase into smaller approved phases before implementation
rather than combining unrelated migrations in one commit.

The primary-approved T5a1r2-r4/T5a3a1-f closure is one coupled interface migration
after the completed T5a1r1 commit, as recorded under T5a3f. Its original per-stage
commit titles are intended partition labels, not claims that those commits exist.
This explicit atomic bundle does not relax the rule for unrelated later phases.

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
- Scientific layout uses deterministic parent constraints and local layouts.
  Intrinsic requirements inform parent allocation separately from measurement;
  each measurement returns one immutable measured result or a typed fit miss.
  Parents assign child constraints and positions; siblings are not combined in a
  Cartesian search. Local alternatives require a declared owner, order, and cap.
- Root feedback is bounded to at most eight rounds with explicit cycle and
  exhaustion failure. No component has an unbounded local retry loop. Scientific
  requirements and local preferences do not form a global dense-rank objective.
- Exactly three sizing modes are approved: automatic preferred width followed by
  measured height, exact width with automatic height, and an exact width-height
  pair. No width search, maximum-width target, public Flutter bounds, hidden root,
  font shrink, content omission, or universal panel multiplier is admitted.
- Current defaults, `.size(width,height)`, and getters remain current until T6.
  T6 introduces consuming Plot/Canvas `width(u32) -> Self`; every sizing setter
  completely replaces the request, so width after size clears the exact height.
  Positive-dimension validation remains at preflight, matching current timing.
- All PNG limits are format-local in every sizing mode, including source/target
  dimensions, backing storage, actual parser bytes, and requested DPI. Remove
  hypothetical default-PNG checks and the automatic-only 64 MiB canonical SVG
  gate. Shared logical-work/font caps, representability, and fallible SVG
  allocation remain; PNG failure never relayouts or invalidates valid SVG.
- Automatic placement uses bounded containing-frame recovery, validating body,
  axes, labels, and guides together; explicit placements remain fixed. Wrapping
  uses only Unicode-legal breaks, with no emergency break, rotation, ellipsis,
  or shrink introduced as recovery.
- Preserve the canonical font catalog, ordering, and provenance. Prefer the first
  face covering a base plus combining marks within an extended grapheme cluster
  (EGC) as a unit; only if none covers the unit use per-scalar first coverage when
  possible. Missing visible coverage is fatal; no provisioning or whole-word,
  emoji, or generic cluster-mapping extension is admitted.
- `CompositionTheme` owns `root_title_alignment`, optional
  `root_subtitle_alignment`, and independent `root_caption_alignment`, using
  `TitleAlignment`. Built-ins use Center, None, and Start. Subtitle fallback is
  the final effective title alignment. T6 exposes independent Plot title/subtitle
  instructions and Canvas panel title/subtitle defaults; explicit Plot instruction
  wins over Canvas panel instruction, then the resolved theme default. These
  narrow policies do not merge themes or affect the Canvas's own heading.
- Automatic numeric ticks have no fixed five-tick ceiling or public count target.
  T6 exposes one optional additive or Log10-decade preferred data interval per
  axis; the renderer tries it first, then deterministic coarsenings for fit, and
  keeps one terminal midpoint source after every interval vector.

Phase T1 settled the exact replacement theme and scale public types, replacement
semantics, component boundaries, validation, and scope of local overrides in
`03_THEME_SYSTEM.md`. Phase T4 made complete built-in Canvas/Plot theme
attachment and inheritance current without introducing the replacement value
model early. Phase T3j recorded the original private layout terminology, public
sizing boundary, migration partition, and final ledger impact. Its exhaustive
search scope is historical and superseded by T5a0r; its completed audit remains
evidence of that earlier objective, not revised-target compliance. Phase T3k closed
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
- Local layout decision order, feedback, scientific aspect, resource budgets,
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
| T5a | Layout foundation, surfaces, typography, titles, axes, and grid | IN PROGRESS |
| T5a0 | T5a contract adjudication and execution partition | COMPLETE |
| T5a0r | Parent-constraint documentation adjudication | COMPLETE |
| T5a0s | User-visible behavior approval and bounded recovery | COMPLETE |
| T5a1 | Exact-root kernel (revised target) | COMPLETE |
| T5a1r | Active parent-constraint kernel repair | COMPLETE |
| T5a1r1 | Single-result exact-root and constrained-text route | COMPLETE |
| T5a1r2 | Active parent allocation and root feedback | COMPLETE |
| T5a1r3 | Typed containing-frame recovery and gutter fit | COMPLETE |
| T5a1r4 | Root admission and lifecycle closure | COMPLETE |
| T5a2 | Foundational resolved appearance and surfaces | COMPLETE |
| T5a3 | Canonical fonts and immutable text plans | COMPLETE |
| T5a3a | Unicode and font preparation, profile, and provenance | COMPLETE |
| T5a3a1 | Unicode profile, provenance, and conformance | COMPLETE |
| T5a3a2 | Bounded font discovery and handle lifecycle | COMPLETE |
| T5a3a3 | Font census, selection, and retention | COMPLETE |
| T5a3b | Deterministic assigned-width text wrapping | COMPLETE |
| T5a3c | Shaping, bidi, and geometry repair | COMPLETE |
| T5a3d | Resource and lifecycle closure | COMPLETE |
| T5a3e | Producer and SVG/PNG regressions | COMPLETE |
| T5a3f | Independent closure and performance evidence | COMPLETE |
| T5a3p | Post-T5a3 repository hygiene and publication efficiency | COMPLETE |
| T5a3p0 | Follow-up behavior review and selected-work admission | COMPLETE |
| T5a3p1 | Remove full Unicode corpora from unpublished history | COMPLETE |
| T5a3p2 | Avoid unchanged root and child measurement | COMPLETE |
| T5a3p3 | Capture producers once | COMPLETE |
| T5a3p4 | Simplify resource admission accounting | COMPLETE |
| T5a3p5 | Evaluate repeated font setup | COMPLETE |
| T5a3p6 | Avoid duplicate automatic-tick work | COMPLETE |
| T5a3p7 | Review speculative line extension | COMPLETE |
| T5a3q | Local-search benchmark coverage and placement comparisons | COMPLETE |
| T5a4 | Root title-block plans | COMPLETE |
| T5a5 | Axis, range, tick, frame, and grid plans | COMPLETE |
| T5a6 | Ordinary Plot exact-root migration and closure | PENDING |
| T5a6a | Ordinary interior carriers and deferred-family adapters | PENDING |
| T5a6a1 | Prepared interior contracts and exhaustive classification | COMPLETE |
| T5a6a2 | Retained canonical primitive geometry | COMPLETE |
| T5a6a3a | Point/path semantic and payload extraction | COMPLETE |
| T5a6a3b | Distribution/reference semantic and payload extraction | COMPLETE |
| T5a6a3c | Estimate-display semantic and payload extraction | COMPLETE |
| T5a6a3d | Bar semantic and payload extraction | COMPLETE |
| T5a6a3e | Categorical-distribution semantic and payload extraction | COMPLETE |
| T5a6a3f | Interaction semantic and payload extraction | COMPLETE |
| T5a6a4a | Heatmap deferred interior | COMPLETE |
| T5a6a4b | Marginal deferred interior | PENDING |
| T5a6a4c | Matrix deferred interior and outer title integration | PENDING |
| T5a6a4d | Beeswarm deferred interior | PENDING |
| T5a6a4e | Waterfall deferred interior | PENDING |
| T5a6a5a | Facet parent adapter and outer title integration | PENDING |
| T5a6a5b | Canvas parent adapter and outer title integration | PENDING |
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

Historical scope: this completed phase and T3j1-T3j3 below record the original
exhaustive-search design and its audit achievements. Complete child products,
global ranking, exhaustive alternatives, and exact-allocation requirements in
this historical record are superseded search scope, not active implementation
instructions. T5a0r adjudicated the parent-constraint replacement; T5a0s further
supersedes its sizing, font-selection, caption, and format-limit decisions.

**Status**: COMPLETE

Define and reconcile the layout replacement before theme inheritance or visual
component migrations make new assumptions about fixed root dimensions.

Dependencies: T3i.

No production code changes in this phase.

### T3j1: Normative Layout Contract And Private Terminology

**Status**: COMPLETE

Historical contract/evidence below: complete-candidate and global-search
requirements are superseded by the T5a0r parent-constraint adjudication.

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

Historical ledger reconciliation below: preserve its source trace and counts as
completed evidence. Its 24 target-only keys and 1884-key materialized total are
historical, not active acceptance totals. T5a0r recorded 22/1574/1882 after
search-role retirement; T5a0s expects 23 target-only keys, a 1575-row overlay, and
1883 materialized keys after adding independent caption alignment, subject to
coordinated `04` review.

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

Historical migration and evidence map below: its search producers, rank pools,
frontiers, and five exact reservation stages are not active T5 instructions.
The reopened T5 stages replace those obligations while preserving existing caps
and the current/target API boundary.

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
appearance into the resolved theme. T5a is complete only through T5a0r, T5a0s,
T5a1r, T5a3a-T5a3f, and the remaining T5a4-T5a6d stages in
dependency order; no subphase may leave a dormant replacement kernel, temporary
selector, compatibility fallback, or second root-plan representation.

Parent acceptance:

- Every T5a measurement call returns one immutable measured result or a typed
  recoverable `MeasurementOutcome::FitMiss`. Intrinsics remain separate parent-
  allocation inputs; bounded local choices never produce sibling cross-products.
- Ordinary Plot plans contain the selected root size, title text, ticks, axis
  gutters, data panel, and final arrangement before Null or concrete rendering.
- Layout, collision, geometry, and paint use identical resolved typography,
  spacing, glyphs, ticks, and axis geometry.
- Theme appearance cannot remove caller-provided semantic text.
- No public constraint, automatic-size, exact-width, Output-dimension, theme-
  component, title-alignment, or tick-interval API is exposed early.
- Every intentional built-in output change names its final-ledger owner and has
  focused evidence plus an inspected snapshot committed with the source change.
- Focused layout, axes, typography, output, and visual-regression tests pass.
- `just verify` passes before T5a6d, T5a6, and parent T5a are marked complete.

### T5a0: Contract Adjudication And Execution Partition

**Status**: COMPLETE

Historical scope: preserve this documentation achievement, including tick,
alignment, font-bound, and ledger decisions. Its complete total-key selector,
frontier, and search-specific reservation partition below describe the old
objective and are superseded by T5a0r, not instructions for the active repair.

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

### T5a0r: Parent-Constraint Documentation Adjudication

**Status**: COMPLETE

Historical approval: the scope, counts, scalar fallback, and resource gates below
record T5a0r's reviewed objective, not revised-target compliance. T5a0s supersedes
its observable sizing, font-selection, caption, and format-limit decisions.

Dependencies: completed T5a0 and user approval of deterministic local layout.

Objective: reconcile `01`/`02` current contracts, `03` theme/font policy, `04`
ownership ledger, `05` layout target, this plan, and `99` deferred features before
repair implementation.

Documentation evidence (2026-09-07): independent consistency and preserved-scope
reviews closed the resource-counter, serialization-gate, legend-width,
automatic-guide intrinsic, text-rebalance, track-flexibility, and assigned-box
containment findings. Active target-key rows/count arithmetic and current/target
boundaries were inspected; `git diff --check` passed. No code, dependencies,
fixtures, or snapshots changed and no test suite ran. Runtime repair acceptance
was pending at this documentation checkpoint; it is now discharged by the
integrated T5a1r/T5a3 closure recorded under T5a3f. The earlier documentation
completion alone did not certify the historical implementation's correctness.

Historical reopening boundary: `028e2c8` established an internally reviewed
document, not user acceptance of its fit-failure tradeoffs. T5a0s records the
subsequent accepted decisions and the document closure required before runtime repair.

Acceptance:

- Replace active universal candidate collections, root Cartesian search,
  mandatory dense ranks, and exhaustive wrapping with parent allocation,
  separate intrinsics, and one measured immutable result or typed fit miss.
- Declare local choice owners, finite orders/caps, and at most eight root
  feedback rounds. Retire global-quality ledger roles explicitly rather than
  pretending they are still mandatory suppliers. Reconcile the materialized
  inventory count and evidence links after retirement: `04` now has 22
  target-only keys, 1574 overlay rows (1552 existing plus 22 target-only), and
  1882 materialized keys (1860 historical plus 22 target-only). Historical
  T3j/T5a0 24/1576/1884 totals remain provenance, not active acceptance totals.
- Preserve current exact APIs until T6, sparse composition semantics, canonical
  ordered font stacks plus weight, and scalar fallback. Repair Common/Inherited
  script resolution without a broad cluster-fallback policy replacement.
- Preserve existing resource caps while replacing search-specific accounting
  with bounded first-party admission and checked fallible incremental growth.
  No blanket exact-allocation rule or third-party OOM guarantee is admitted.
- Map every audit concern to T5a1r or T5a3a-f, with reproduction before claims of
  confirmed defects. Preserve historical commits and full Unicode/test fixtures.
- Independent documentation review resolves contradictions and checks the diff;
  no production edits, tests, fixtures, snapshots, or verification suite here.

Commit: `docs(layout): adjudicate deterministic parent constraints`.

### T5a0s: User-Visible Behavior Approval And Bounded Recovery

**Status**: COMPLETE

Dependencies: T5a0r; the user approved D1-D6, including the delegated D3
recommendation. Target approval does not change current APIs or authorize early
T6 exposure. This documentation objective is complete; the admitted T5a1r/T5a3
runtime repairs subsequently completed through the integrated closure below.
Remaining native migrations and the public T6 API are still future work.

| Decision | Accepted behavior | Boundary and evidence owner | Status |
| --- | --- | --- | --- |
| D1 Automatic recovery | Validate the entire affected frame; bounded supported edges and preferred/compact reflow may remeasure affected children | Explicit placements stay fixed; fatal font/resource errors are not fit alternatives. T5a1r foundation, T5b guides, T5e closure | ACCEPTED |
| D2 Long labels | Option A: strict Unicode-legal wrapping and complete content preservation | Overwide unbroken text fails; no emergency break, new rotation, ellipsis, or font shrink. New controls remain DF5. T5a3b/T5a5 | ACCEPTED |
| D3 Sizing workflows | User-approved delegated best choice: exactly automatic, exact-width/automatic-height, and exact-pair modes | Remove `max_width` target; consuming Plot/Canvas `width(u32) -> Self` at T6 replaces the width getter. Setters completely replace requests; width after size clears height. Positive preflight matches current timing; Canvas owns child allocation; no public Flutter bounds. T5e/T6 | ACCEPTED |
| D4 SVG versus PNG limits | Every PNG limit is terminal-local in all sizing modes; remove hypothetical PNG viability and automatic-only 64 MiB canonical SVG gate | Shared logical-work/font caps, representability, and fallible SVG allocation remain. No relayout on PNG failure. T5a3d/T5e/T6 | ACCEPTED |
| D5 Font availability | Preserve catalog/order/provenance; prefer first face covering base plus combining marks within an EGC as a unit, otherwise allow per-scalar first-coverage fallback when possible | Missing visible coverage is fatal, including required generated alphabets; no provisioning, whole-word, emoji, or generic cluster mapping. Preparation-unit census/selection in T5a3a3; shaping in T5a3c | ACCEPTED |
| D6 Caption alignment | Independent `CompositionTheme::root_caption_alignment` uses `TitleAlignment`; built-ins use Start captions and Center titles, with subtitle following final title alignment by default | Same-owner theme inheritance; the subsequent approved independent title/subtitle policy is specified in T5a4 and T6. Panel alignment instructions never affect captions | ACCEPTED, AMENDED |

Behavioral acceptance examples, carried into the owning stages:

- D1: a right guide fits alone but starves the plot; top/bottom or compact reflow
  succeeds with all labels. Explicit right stays right or fails. Nested/multiple
  guides share trial/work budgets; distinguish exhausted fit trials from a
  planning-limit stop, without partial output in either case.
- D2: a spaced scientific caption wraps; an identifier with no legal break wider
  than its assigned box returns a typed fit miss without rotation or alteration.
- D3: `.size(600,400).width(300)` means exactly 300 wide with measured height;
  `.width(300).size(600,400)` means exactly 600 by 400. Omission uses preferred
  width; zero dimensions fail at preflight. These are T6 examples, not current API.
- D4: valid SVG may exceed PNG dimensions or 64 MiB in every sizing mode, subject
  to retained shared safety gates; subsequent PNG refusal leaves its scene intact.
- D5: if face A covers the base only and face B covers base plus mark, select B
  for that unit. If no face covers the unit but separate faces cover every scalar,
  use first coverage per scalar; if a visible mark is uncovered, fail explicitly.
- D6: all built-ins center title/subtitle and Start-align multiline captions;
  changing caption to End leaves titles unchanged, with Canvas/Plot owner isolation.

Revised ledger: 23 target-only keys, 1575 overlay rows (1552 existing
plus 23 target-only), and 1883 materialized keys (1860 historical plus 23), adding
one independent caption property. Historical T3j/T5a0/T5a0r totals remain
historical evidence, not revised target totals.

Documentation evidence (2026-09-08): independent cross-contract review checked
D1-D6, current/target staging, target-only rows/count arithmetic, and the matching
DF1-DF9 review gates. The remaining one-alignment wording and text-control review
checkpoint discrepancy were corrected. `git diff --check` passed. No code,
dependency, fixture, or snapshot changes were made; the documentation-only task
does not run the test suite. This does not certify runtime conformance.

Commit: `docs(layout): finalize behavior decisions and review gates`.

### T5a1: Exact-Root Kernel (Revised Target)

**Status**: COMPLETE

Historical achievement: `b1c1406` (`feat(layout): add active exact-root candidate kernel`)
established the original exact-root envelope, candidate kernel, accounting, and
lifecycle route. Its complete-candidate/Cartesian/total-key search scope is
historical, not evidence of compliance with the revised target. The reopened
target is now complete through T5a1r and the primary-verified integrated closure
under T5a3f; this umbrella has no separate repair commit.

### T5a1r: Active Parent-Constraint Kernel Repair

**Status**: COMPLETE

Dependencies: completed T5a0r and T5a0s. Implement the accepted contract and
preserve the current public API until its scheduled migration.

Scope and deletion targets:

- Replace the active universal candidate API with separate intrinsic inputs,
  deterministic parent allocation, normalized assigned constraints, and one
  immutable `MeasuredLayout` or typed fit miss. Preserve canonical component
  identities, checked geometry, the root envelope, and one freeze/replay route.
- Migrate all active kernel and typography consumers atomically; remove the old
  candidate collection, Cartesian combination, global quality selector, and
  dormant dual routes. Introduce the constrained text primitive needed by the
  kernel here if necessary. T5a3b removes residual exhaustive text generation
  inside that single route; it must not be needed to remove a runtime selector
  left behind by T5a1r.
- Limit this atomic migration to the current root wrapper, active adapters, and
  consumers required to replace the universal root representation. Native family
  migration remains in T5a6/T5b/T5e; this stage does not absorb those migrations.
  T5a3b repairs residual text internals behind the one active constrained route,
  not a second root representation or temporary strategy switch.
- Declare the owner/order/cap of each activated local alternative. Bound root
  feedback to eight rounds with deterministic equality, cycle, and exhaustion
  outcomes; no local unbounded retries or hidden fallback root.
- Establish D1 containing-frame recovery in activated owners, not guide-only fit:
  for example, try a supported automatic edge when right placement starves the
  body, while explicit right stays fixed. Share finite trial/work accounting;
  T5b/T5e complete native guide and composition evidence on this same route.
- Retain operation-scoped accounting, applicable existing caps, typed cache
  identities, and fallible bounded growth. Gate work and first-party storage
  before materialization, including text primitives activated here. Admission
  errors propagate as errors, not fit misses or `.ok()`-filtered absence.
- Preserve exact `640x480` defaults and explicit dimensions through the current
  API. Frozen root size alone supplies Null, SVG, and Scene construction; reserve
  the final plan and replay storage before final Null validation.

Evidence:

- Normalized constraints, intrinsic/allocation separation, fractional geometry,
  typed fit misses, deterministic local order/caps, bounded feedback, cycles,
  exhaustion, cache identity, limit boundaries, reservation refusal, and failures
  before backend calls. Prove no sibling-product or alternate active route.
- Reproduce suspected fixed 40% gutter rejection cases against actual measured
  fit; remove arbitrary percentage rejection where content fits, retaining real
  hard geometry failure. T5a5 extends evidence to native axis plans.
- Current omitted/explicit-size dimensions, immutable Null/concrete identity,
  and focused snapshots remain accounted for; `just verify` passes.

Historical inspection found that the former root executed one opaque
layout once, then wrapped it in a candidate selector and a synthetic two-pass
identity; real family/guide feedback remained below it. Replacing only those
types could not establish parent allocation, real convergence, or D1 recovery.
The following subphases preserve the accepted target and partition its migration.
All four acceptance stages are now complete. T5a1r1 landed separately; the
coupled r2-r4/a1-f acceptance is discharged by the integrated verified bundle
under T5a3f. This umbrella has no separate commit.

#### T5a1r1: Single-Result Exact-Root And Constrained-Text Route

**Status**: COMPLETE

Dependencies: T5a0s.

- Replace universal root candidate collections, Cartesian products, global
  quality selection, and synthetic convergence with one directly measured result
  under the exact assigned root dimensions. Migrate Plot, Canvas, and tests
  atomically; retain preparation once, one finalization, and one frozen replay.
- Remove the unconditional million-entry root cache and obsolete search-only
  reservations/counters. Admit actual root storage fallibly and pass assigned
  dimensions to measurement/finalization rather than rereading source size.
- Make the active text route build the requested width's result directly instead
  of retaining and selecting a collection of complete text plans. Existing
  transition preparation may remain in that same route until T5a3b; it is not
  a second renderer or strategy selector.
- Preserve exact public defaults, semantic checks, current family adapters,
  resolved fonts, and terminal replay. Retain useful numeric and refusal tests;
  replace search-only tests with exact-result, assigned-size, and lifecycle tests.
- Do not claim root feedback, D1 recovery, the full producer resource census, or
  complete pre-Null serialization reservations are repaired by this subphase.
  No fake state equality or test-only replacement solver stands in for them.
- Run focused kernel/text/lifecycle tests and `just verify`; inspect any output
  changes. This phase does not authorize shortened regression fixtures.

Historical r1-only evidence (2026-09-08; later limitations superseded by the
integrated closure under T5a3f): Plot and Canvas use one `MeasuredLayout` and
`FrozenRoot`, with exact assigned dimensions supplied to measurement/finalization.
Canvas measurement consumed the assignment; legacy Plot builders still read the
borrowed source after an explicit assignment-equality check. Their deeper
allocation migration was assigned to T5a1r2/native owners, not claimed by r1.
The universal root candidate, quality, Cartesian, fake convergence, and fixed
million-entry cache implementations are removed. The text path materializes
only the requested width's final lines/plan; all-boundary-pair transition work
and the 144-repeat limit persisted at that checkpoint and were removed in T5a3b.

Eight kernel, 24 text, and existing lifecycle/backend tests cover exact-result
identity, callback/refusal order, selected-line shaping, shared budgets, and
unchanged terminal routing. Independent code review found no introduced
regression within r1's scope. `just verify` passed formatting, locked/offline
all-target Clippy, all 520 library tests and integration/snapshot suites,
rustdoc, and package verification. Existing snapshots passed unchanged; no
fixture or snapshot was shortened or updated. T5a1r2-r4 and T5a3 were pending
at that checkpoint, not in the current status inventory.

Commit: `refactor(layout): use one exact-root measurement result`.

#### T5a1r2: Active Parent Allocation And Root Feedback

**Status**: COMPLETE

Dependencies: T5a1r1.

- Introduce separate intrinsic requirements and deterministic assigned allocation
  in an active parent/frame owner, migrating its current adapter callers.
- Connect real measured gutter/alignment/owned-ink/domain consequences to the
  sole bounded root driver. Remove migrated local stabilization loops rather
  than wrapping eight root rounds around eight local rounds.
- Compare complete relevant geometry and consumed/next feedback, with typed
  terminal/provisional fit results. Prove actual two/eight-round success,
  unchanged misses, cycles, exhaustion, and invalidation under changed inputs.
- Cache only actually requested outcomes, with operation-local identity and
  admission. Extend to other owners in their scheduled native migrations.
- Run focused production-route and resource tests and `just verify`.

Original intended title: `refactor(layout): coordinate active parent feedback`.

Historical intermediate evidence (2026-09-08; superseded as a progress report
by the integrated closure under T5a3f): the production Plot/Canvas root
coordinates up to eight measurement rounds and compares retained normalized
observations plus consumed/requested/next feedback. The active frame separates
natural guide insets from assigned body allocation; title/tick gutter and legend
domain changes return to the root rather than running local stabilization loops.
The text adapter exposes hard unbroken and natural widths separately from a
single source-ordered width preference for the assigned vertical label-row
capacity. It still reused contextual transition preparation at that checkpoint;
T5a3b subsequently removed that search-era preparation. The r2-only checkpoint
did not claim the later text/font milestones.
Canvas/facet grids measure their assigned cells once per round; Canvas range-group
alignment and row-title owned reserves participate in feedback. Shared facet
title and domain remeasurement no longer starts a nested convergence loop.
No outcome cache was installed at that checkpoint; changed assignments
remeasured. The subsequently completed font-scoped analysis and indexed query
memo retain requested results under the same operation generation budget.

That intermediate checkpoint was not a completion or full D1 claim. Its planned
per-stage commit boundary was superseded by the primary-approved atomic
interface migration; combined review and full verification are recorded below.
Typed automatic-clearance error propagation and natural vertical-label gutter
measurement are prerequisites included here: remeasurement reproduced the 40%
rejection in the original composition regressions, so measurement, validation,
tick-source fitting, and drawing no longer use that percentage. Containing-frame
edge/flow recovery and the complete typed-fit/gutter audit were assigned to r3;
producer/final-replay admission closure was assigned to r4. Both are now covered
by the integrated closure. Native scientific intrinsic minima and specialized
family adapters remain with their scheduled migrations.

Focused evidence: 16 kernel, 25 text, 25 panel, 36 lifecycle, 16 decoration-plan,
57 composition, 20 layout, and five visual-regression tests pass. The kernel
cases include actual two/eight-round success, changed assigned geometry with
unchanged feedback, cycles, exhaustion, unchanged provisional misses, fatal
precedence, and admission refusal. Production cases exercise domain expansion,
changed cell projections, shared groups, row-title reserves, and the intact
long-label facet regression. Reviewed snapshot corrections are subpixel frame
quantization in Scatter/Matrix and the measured HorizontalBar gutter; no source
label, input fixture, or mark is removed. These focused runs preceded the full
verification now recorded under T5a3f.

Independent-review corrections (2026-09-08): final parent observations now run
after facet outside-legend installation and include the shared-axis origin,
legend anchors and glyph carriers, and actual text placements and clip contexts
from geometry discovery. Canvas retains its arranged shared-legend bounds rather
than deriving them again at draw time. Regressions change final legend/origin
fields, a nonmaximal legend glyph, and a fully clipped annotation placement while
holding the former extent-only observations fixed.

Domain clearance now returns `Result<Option<DomainExpansion>>`; domain/panel
spans, expanded endpoints, and ranking ratios reject arithmetic failure instead
of treating it as a local miss. Text plans have one immutable shared payload;
feedback admits a handle before retaining it, and no longer deep-clones text,
runs, glyphs, or font provenance. Feedback string/address copies are fallible and
charged before allocation. Provisional misses retain concrete changed requests
from the current round, with stale/unrelated handles rejected; annotation and
overflow predicates link only to a resolving assignment/projection prediction.
Negative regressions retain terminal failure for screen-anchored conflicts and
unowned or unrelated-axis overflow. The next root round still performs actual
measurement; a prediction is not a frozen successful plan. Focused geometry,
SVG, annotation, composition, and snapshot suites pass without further snapshot
or fixture changes. Full admitted D1 recovery and root admission closure were
subsequently discharged with r3/r4 in the integrated bundle.

#### T5a1r3: Typed Containing-Frame Recovery And Gutter Fit

**Status**: COMPLETE

Dependencies: T5a1r2.

- Retain typed text/geometry fit misses through activated parent recovery;
  font, arithmetic, allocation, and work failures propagate without `.ok()` or
  error-as-collision conversion.
- Implement D1 recovery in the activated frame: guide-only fit is insufficient;
  bounded edge/flow changes remeasure the affected body/axes. Preserve explicit
  placement and shared nested budgets, without sibling Cartesian search.
- Reproduce the 40% gutter failures and replace unjustified percentage limits
  consistently across measurement and drawing. Preserve genuinely unfit inputs.
- Prove recovery in a real Plot route, explicit-placement failure, fatal-error
  propagation, and original long-label regressions. Native composition/legend
  completion stays in T5b/T5e. Run focused tests and `just verify`.

Original intended title: `fix(layout): recover containing-frame fit with typed failures`.

Historical intermediate evidence (2026-09-08; superseded as a progress report
by the integrated closure under T5a3f): active Plot, shared-Canvas, and shared-facet
guide owners try outside right/top/bottom/left with preferred then compact flow.
Explicit outside edges remain fixed and explicit inside placements retain their
inside policy. Guide-only fit does not accept a trial: assigned axes, annotations,
headers, tracks, and the actual body participate before acceptance. Provisional
feedback returns to the root. Rejected ancestor attempts do not reset descendant
guide trial budgets. Legend plans retain their flow slots and measured text rather
than choosing columns or wrapping while drawing. Left/bottom guide reservations
include complete measured axis ink, with pixel-envelope rounding required by the
current integer-anchor axis adapter.

Tick fitting and Matrix tick retention now propagate planner errors through the
private Layout constructors and repository callers, instead of converting them
to absent ticks or a spurious small-cell error. The offset-panel lifecycle test
compares the production quantized assignment exactly. No public API, font catalog,
text-engine implementation, theme appearance, or r4 serializer changed here.

Focused evidence includes real Plot, Canvas, and facet right-to-top containing-frame
recovery, explicit-right failure, compact-column remeasurement, left/bottom axis
clearance, nonresetting nested trial budgets, and fatal tick/Matrix planner errors.
The original long-label composition and annotation fixtures remain intact.
The typed channel now reaches panel measurement, frame assignment, and enclosing
Canvas/facet trials. Constructible plot siblings retain the least fit reason and
continue, with a production facet regression proving that a later fatal data
error wins. Diagnostic tests cover kind/address/attempt ordering and original
child-reason preservation; Render messages are never parsed to select recovery.
Typed text planning shares the existing root-local theme query memo, avoiding
repeated work that exhausted the unchanged long-label facet fixture's budget.
Parent-measured axes no longer acquire duplicate child overflow reserves. The
exact Canvas lifecycle counts consequently drop from ten to eight panel calls.
Heatmap elision reserves natural label width and does not rewrap fitted labels;
composed discovery uses local capacity rather than absolute x as its budget.
Inside-collision interaction and marginal tests now explicitly request inside
placement, retaining their data and exact geometry assertions under D1.

Follow-up (2026-09-08): both unchanged `tests/rendering/labels.rs` regressions now
pass. Standalone categorical labels request natural width when it fits the real
remaining width after axis chrome and the existing family body minimum; otherwise
they use the compact intrinsic preference. The selected width is retained on the
axis and observed by feedback, rather than independently recomputed by drawing.
Parent-measured categorical gutters retain their parent's existing preference;
residual mark width does not become a second label allocation. A narrow-width
regression proves complete source retention with genuine wrapping. Heatmap elision
uses that same selected width, and continuous guides reserve their width before
categorical allocation. No label-input shortening or percentage cap was added.

The activated fit audit now includes categorical x/y allocation and row height,
parent headers and axis-title gutters, panel-annotation conflict planning,
beeswarm/heatmap colorbar titles, heatmap value and merged-row labels, elision,
estimate/status text, attached and pairwise significance, waterfall annotations,
and raincloud strip geometry. These paths preserve typed misses through their
containing-frame trials. Fixed guides use their fixed-edge address role;
no-alternative children use local ordinal zero rather than inheriting an ancestor
guide attempt. Pairwise diagnostics retain caller slots after bracket sorting.
Constructible sibling collections retain the canonical least miss; later fatal
producer or scope-cleanup errors win. Nonfinite body and significance placement
arithmetic remains fatal, and no Render message is parsed to select recovery.

Focused evidence: rendering 31/31, composition 57/57, estimates 38/38, heatmap
35/35, layout 20/20, annotations 27/27, distributions 67/67, categorical 42/42,
and lines 41/41 pass. All five visual regressions pass after inspection of the
intentional horizontal-bar complete-label allocation and beeswarm outward-ink
reserve changes. The earlier locus-label wrapping workaround was replaced with
complete-label counts, widest-label clearance, and the existing canvas-bound
checks; the input is unchanged. Fit/fatal, canonical identity, sorted-bracket,
feedback-budget, and complete-frame guide tests also pass.

The intermediate concurrent Clippy diagnostics in kernel preparation and replay
storage are resolved and superseded by the final strict all-target Clippy and
full `just verify` pass recorded under T5a3f. The fit-only follow-up did not itself
claim census/replay closure; the integrated migration discharges that r4 scope.
Native interior representation completion stays in its existing phases; current
colorbar and annotation fit propagation is included here, not deferred to them.

#### T5a1r4: Root Admission And Lifecycle Closure

**Status**: COMPLETE

Dependencies: T5a1r3.

- Audit activated root preparation, measurement, feedback, cache, and replay
  payloads against the shared admission protocol; remove remaining search-era
  stage assumptions. Font/text-specific closure remains in its T5a3 stages.
- Reserve activated final plan/replay and canonical serialization storage before
  final Null validation. Counting is not a backend lifecycle; final replay
  neither measures nor chooses layout. Fix the active capture boundary rather
  than relying only on caller-visible concrete-backend tests.
- Independently review every T5a1r acceptance item above, including real parent
  allocation, typed recovery, dimensions, resource refusal, and actual feedback.
  Run `just verify` before completing T5a1r and T5a1. This gate and the coupled
  T5a3 acceptance are discharged by the integrated closure under T5a3f.

Original intended title: `refactor(layout): close root admission and replay lifecycle`.

### T5a2: Foundational Resolved Appearance And Surfaces

**Status**: COMPLETE

Historical achievement: `6da94ea` (`refactor(theme): centralize foundational
appearance`). Preserve its resolved appearance, surfaces, and ownership work;
kernel repair is not authorization to redesign that appearance. The centered
caption statements below are historical; T5a0s/D6 authorizes the independent
Start caption default and carrier connection in T5a4, not a rewrite of T5a2.

Dependencies at completion: historical T5a1. Revised consumers use T5a1r.

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

Dependencies: retained T5a2 and revised T5a1r; closes through T5a3a-T5a3f.

Historical achievement: `4b38de4` (`feat(typography): freeze resolved text plans`)
introduced canonical font preparation and retained text plans. That achievement
was reopened because its exhaustive wrapping scope was superseded. The revised
acceptance is now complete through T5a3a-f in the primary-verified integrated
bundle below, not by retroactively certifying the old implementation. The audit
matrix records fixes and evidence-based dispositions within this scope. The
umbrella has no separate repair commit.

Shared repair acceptance:

- One immutable text result at an assigned width, with source spans, faces,
  glyphs, clusters, offsets, advances, logical widths, ink, line translations,
  pitch, and baselines shared by measurement, collision, and replay. No omitted
  semantic text, font shrink, hidden root, emergency wrapping fallback, or
  draw-time layout. Intact overwide content produces a typed fit miss.
- Preserve canonical ordered `(stack,weight)` discovery, catalog, and provenance;
  T5a3a3 prefers the first face covering a base plus combining marks within an
  EGC as one preparation unit. Only if no face covers it, use first coverage per
  scalar when possible. Missing visible coverage is fatal. T5a3c shapes the
  selected runs; no whole-word, emoji, or generic cluster-mapping extension.
  No public API change, asset provisioning, or network access.
- Every stage gates its own input, work, and first-party storage before use and
  propagates admission errors. Fallible incremental growth is permitted within
  existing caps; T5a3d audits closure, not the first introduction of bounds.
- Preserve `1_000_000` discovery entries, `536_870_912` inspected/hashed bytes,
  `1_000_000` font-preflight work, and other applicable existing limits. Do not
  promise exact allocations everywhere or recovery from all third-party OOMs.
- Preserve all full Unicode conformance and regression fixtures. Inspect any
  changed snapshot and commit it only with its intentional source correction.
  Each acceptance stage supplies focused evidence; the coupled interface bundle
  satisfies the final `just verify` gate together, as recorded under T5a3f.

#### T5a3a: Unicode And Font Preparation, Profile, And Provenance

**Status**: COMPLETE

Dependencies: T5a1r and retained T5a2.

This umbrella is complete through T5a3a1-T5a3a3 and has no separate commit.
Each substage's admission gates and focused evidence are retained; their shared
font/census/text interfaces were migrated and fully verified in the atomic
bundle described under T5a3f. That approved coupling is not authorization for
unrelated cross-file migrations or dormant compatibility routes.

##### T5a3a1: Unicode Profile, Provenance, And Conformance

**Status**: COMPLETE

Dependencies: T5a1r and retained T5a2.

Scope and acceptance:

- Establish a coherent Unicode 16 profile across segmentation, line breaking,
  scripts, bidi, shaping dependencies, and generated data. Audit transitive
  Unicode versions, not just direct dependency labels; document any dependency
  integration required for coherence and run full applicable conformance data.
- Make generated-table provenance reproducible: upstream version, source inputs,
  checksums, generator invocation, and licensing. Do not replace full fixtures
  with reduced cases to make version mismatches disappear.
- Inventory direct/transitive dependencies and enabled font features. Prefer
  maintained pinned Unicode algorithms when they satisfy the profile rather than
  owning standard algorithms unnecessarily; migrate and remove the replaced
  implementation, not a fallback chain. Remove unused resvg/fontdb capabilities
  only with route and build evidence. Keep required metadata, glyph coverage,
  shaping, bidi, and identity dependencies; a shorter manifest is not acceptance.
- Gate runtime Unicode input, table-derived work, and first-party storage before
  materialization; retain existing caps and propagate admission errors. Evidence
  includes transitive-version reconciliation, reproducible generation, full
  applicable conformance fixtures, and boundary/refusal tests for activated work.

Original intended title: `fix(unicode): reconcile profile provenance and conformance`.

##### T5a3a2: Bounded Font Discovery And Handle Lifecycle

**Status**: COMPLETE

Dependencies: T5a3a1.

Scope and acceptance:

- Preserve canonical discovery inputs and ordered stack/weight policy; selection
  changes belong to T5a3a3, not discovery or fontconfig handling in this stage.
- Reproduce and repair suspected font file-descriptor retention/exhaustion and
  fontconfig partial-error handling. Bound discovery, open-handle lifetimes,
  reads/hashes, and discovery metadata; propagate partial discovery failures
  instead of accepting a silently incomplete environment.
- Admit fontconfig parser capacity, discovery records, and inspected bytes before
  use with checked fallible growth under existing caps. Cover malformed/partial
  configuration, handle release on success/error, changing/short files, order
  invariance, limit boundaries, and reservation refusal.

Original intended title: `fix(font): bound discovery and fontconfig handle lifecycle`.

##### T5a3a3: Font Census, Selection, And Retention

**Status**: COMPLETE

Dependencies: T5a3a2.

Scope and acceptance:

- Preserve canonical family/occurrence/identity ordering, deduplication, weight,
  primary face, catalog, and generation-host provenance. Census preparation
  units for base plus combining marks within each EGC; select the first ordered
  face covering the unit. If no face covers it, allow per-scalar first coverage
  when possible; fail on any uncovered visible scalar. This is not whole-word,
  emoji, or generic cluster mapping, and does not provision fonts.
- Count/admit preparation-unit and scalar census, occurrence/selection metadata,
  unit and scalar coverage probes, and retained
  source bytes before copying, probing, or growing storage. Keep one operation
  resource state and propagate failures rather than silently skipping faces.
- Cover missing families/glyphs, unit preference and scalar fallback, identity/deduplication/order,
  census-copy timing, limits, reservation refusal, and retained-font reuse by
  outline/PNG terminals without rediscovery.
- Example: face A covers the base, B covers base plus mark, so B wins the unit;
  if A and C separately provide the only coverage, first coverage per scalar is
  allowed. An uncovered visible mark fails, including in generated alphabets.

Original intended title: `fix(font): gate census selection and retained faces`.

#### T5a3b: Deterministic Assigned-Width Text Wrapping

**Status**: COMPLETE

Dependencies: T5a3a through T5a3a3; extends the sole constrained primitive active
in T5a1r.

Scope and acceptance:

- Replace residual exhaustive break-partition/width generation with the declared
  locally greedy algorithm in `05_LAYOUT_SYSTEM.md` at parent-assigned width.
  First retain an exactly fitting shaped paragraph. Otherwise scan paragraph-
  shaping proposal prefixes forward to the first proposal overflow, shape the
  proposed whole line exactly, and retreat legal breaks until advance and ink
  fit. Then try only successive next legal breaks, stopping at the first exact
  nonfit; never skip that failure seeking a distant fit. Reuse tested endpoints
  within a line-start sequence and release obsolete trials. No globally longest
  fit guarantee is required. Intrinsic minimum/preferred requirements are
  allocation inputs, not alternative plans.
- Preserve source text, whitespace, mandatory breaks, blank lines, grapheme
  integrity, and typed overwide failure. Local alternatives, if needed, have an
  explicit owner/order/cap and do not cause sibling products or width searches.
- Enforce D2 Option A: only Unicode-legal wrapping, with no emergency identifier
  breaks, new rotation, ellipsis, or font shrink. For example, a spaced caption
  wraps at its assigned width, while an overwide unbroken identifier fails intact.
  Broader controls remain DF5 for T6 entry review, not implementation here.
- Delete residual exhaustive text generation and its caches/ranking state in
  this change; do not retain a runtime strategy selector or dormant old route.
- Bound scans, break records, line storage, and fallible growth from admission
  onward. Propagate errors, including paths previously suspected of using
  `.ok()` to swallow resource or shaping failure as an absent choice.
- Preserve the baseline input `"A ".repeat(144)` (288 characters with legal
  spaces), an approximately 200-word scientific caption in one paragraph, and
  the three-paragraph caption;
  demonstrate fit when assigned geometry permits and typed misses otherwise.
  Measure long legal-break input scaling and work counts; record evidence rather
  than an unmeasured linear-time or latency promise. Preserve complete existing
  text fixtures.

Original intended title: `refactor(typography): wrap text at assigned widths`.

#### T5a3c: Shaping, Bidi, And Geometry Repair

**Status**: COMPLETE

Dependencies: T5a3b.

Scope and acceptance:

- Reproduce script-mark failures and repair Common/Inherited resolution with
  surrounding script context, including leading/interleaved combining marks.
  Shape using T5a3a3's frozen base-plus-mark unit selection and permitted scalar
  fallback, without reselecting faces. For example, retain one covering face for
  a base/mark unit, but preserve a necessary split-face fallback with correct
  script context and geometry; do not broaden selection to whole words or emoji.
- Resolve bidi per paragraph, then apply correct line-level handling after soft
  wrapping; test separate paragraphs, mixed directions, isolates, neutrals,
  trailing whitespace, and mandatory versus soft breaks.
- Reproduce fractional glyph-pen drift; retain fractional advances/offsets and
  consistent baselines, transforms, logical widths, and painted ink through
  measurement, collision, Null, and final outline/PNG replay.
- Gate run/glyph/cluster/geometry storage and shaping work before use, with
  checked arithmetic and propagated errors. Extend full conformance and focused
  marks, non-Latin, bidi, weight, baseline, ink, and immutable-replay evidence.

Original intended title: `fix(typography): repair script bidi and glyph geometry`.

#### T5a3d: Resource And Lifecycle Closure

**Status**: COMPLETE

Dependencies: T5a3c.

Scope and acceptance:

- Trace all first-party font/text/census/serializer/scene allocations and work
  through bounded admission, including copied census data before its gate and
  final Null execution before final-plan/replay reservation. Reproduce suspected
  ordering violations, repair them, and prove refusal occurs before side effects.
- Audit all `.ok()` and equivalent error-discard paths; distinguish recoverable
  fit misses from font, resource, parser, and shaping errors. Preserve one
  operation resource state and existing caps, using fallible incremental growth
  where exact preallocation is neither necessary nor justified.
- At PNG terminals, validate actual outlined parser-input bytes and actual
  requested target options, not only estimates or default dimensions. Cover
  scale/dimension arithmetic,
  backing-store limits, retained outline expansion, and serializer growth before
  parser/backend materialization. All PNG limits, including source dimensions,
  are terminal-local for every sizing mode. Remove hypothetical default-300-DPI
  root checks and the automatic-only 64 MiB canonical SVG gate; keep shared
  logical-work/font caps, representability, and fallible SVG allocation. A later
  PNG may fail without invalidating SVG or selecting a new layout. Example:
  representable SVG above a PNG limit succeeds under retained shared gates,
  then PNG refuses without mutating its scene. Do not claim third-party OOM immunity.
- Prove limit-minus-one/exact/plus-one, checked overflow, refusal, error identity,
  retained environment, zero-backend-call failure, and one final Null/concrete
  lifecycle over the same frozen plan. Every earlier stage must already enforce
  the bounds for its activated paths.

Original intended title: `fix(render): close typography admission and lifecycle gates`.

#### T5a3e: Producer And SVG/PNG Regressions

**Status**: COMPLETE

Dependencies: T5a3d.

Scope and acceptance:

- Census every active one-line/multiline producer and remove raw or late-layout
  bypasses. Freeze significance effective/exact style provenance through the
  retained `DeferredAnnotationTextPlan`; later owners supply inputs before
  measurement, never replace typography after freeze.
- Preserve-text SVG retains source and planned placement but remains dependent
  on viewer font/shaping behavior. Exact retained glyph outlines and PNG share
  generation-host geometry; do not assert universal preserve-text pixel parity.
- Exercise Plot/Canvas titles, axes, active legend/body/annotation consumers,
  Unicode and bidi, `"A ".repeat(144)`, the approximately 200-word
  single-paragraph and three-paragraph captions, 40% gutter fit/failure cases,
  actual parser bytes, and nondefault PNG options without moving future semantic
  ownership from T5a4/T5a5/T5b/T5c. Admit any newly activated producer first.
- Inspect snapshots and preserve full Unicode/test fixture coverage. Host smoke
  tests assert provenance and consistency, not universal host glyph bytes.

Original intended title: `test(typography): cover producers and retained output routes`.

#### T5a3f: Independent Closure And Performance Evidence

**Status**: COMPLETE

Dependencies: T5a3e.

Scope and acceptance:

- Independently review every audit concern in T5a1r and T5a3a-e, recording focused
  reproductions and fixes or evidence-based non-defect dispositions. Recheck
  Unicode transitive versions, generated provenance, resource/lifecycle order,
  script marks, paragraph/soft-wrap bidi, fractional pens, swallowed errors,
  gutter fit, font handles/partial errors, and actual PNG bytes/options.
- Benchmark `"A ".repeat(144)`, an approximately 200-word scientific
  caption in one paragraph, the three-paragraph caption, and increasing long
  legal-break inputs. Record setup,
  time, memory/work counts, and before/after evidence; do not substitute relaxed
  caps, shortened fixtures, or unmeasured performance promises for acceptance.
- Prove the universal candidate/search API, residual exhaustive wrapping, global
  ranks, strategy switches, and dormant dual routes are absent from active code.
  Close ledger/evidence links, full fixture preservation, snapshot review, and
  `just verify` before marking T5a3 and this stage COMPLETE.

##### Integrated Closure (2026-09-09)

The primary approved completion of T5a1 and T5a1r (including T5a1r1-r4), and
T5a3 (including T5a3a, T5a3a1-T5a3a3, and T5a3b-T5a3f),
after the final full verification pass. This discharges the admitted exact-root,
typed-fit, canonical-record, controlled-replay, font, and text repair acceptance;
T5a remains IN PROGRESS and T5a4 onward remain PENDING. Native T5a6/T5b/T5e
representation/composition/scientific-layout completion and the public T6 API
are not claimed. No universal proof of layout/shaping correctness, unbounded
scaling, cross-host preserve-text pixel parity, or third-party OOM recovery follows
from these tests and scoped reviews. Pinned third-party shaping/context behavior
and allocation limits remain residual boundaries, not eliminated guarantees.

Primary verification records `just verify` passing formatting, strict
`cargo clippy --locked --offline --all-targets -- -D warnings`, all-target tests,
rustdoc with warnings denied, and locked/offline package verification. Results:
666 library tests passed, zero failed, one opt-in regeneration test ignored;
all 14 integration targets passed 441 tests, including all five snapshots.
The package contains 204 files, 21.5MiB uncompressed (3.1MiB compressed), and its
unpacked verification build passed. This document update did not rerun commands.

The primary separately confirmed the ignored Rust regeneration check passed
with 11 validated original Unicode inputs and 11 byte-identical outputs. Its
owner is `src/font/unicode16/regeneration.rs::regenerate_unicode16`; source
manifest, generated-output hashes, licenses, exact commands, and full corpus
counts are in `tests/fixtures/unicode16/{README.md,sources.json,SHA256SUMS,LICENSE}`.
Full applicable conformance is covered by `src/font/unicode16/conformance.rs`
and `src/layout/unicode16/conformance.rs`, not sampled replacement fixtures.
The runtime profile is Unicode 16.0.0: unicode-segmentation 1.12.0,
unicode-script 0.5.7, unicode-bidi 0.3.18, unicode-properties 0.1.3,
unicode-ccc 0.4.0, unicode-bidi-mirroring 0.4.0, and rustybuzz 0.20.1's pinned
private `src/hb/unicode_norm.rs`. Family-name NFC/NFKC_CF uses generated
`src/font/unicode16/{nfc_data.rs,casefold_data.rs}`; rendered source is not
normalized. The original archive/member checks are distinct from output hashes.

The primary reports scoped independent r4 review with no production blocker and
closure of the five font/text findings. Their concrete dispositions remain in
the acceptance matrix: contextual/script/bidi and fractional geometry repairs,
actual output admission, retained source/control/soft-hyphen handling (including
the real sealed production title census), and font-scoped indexed reuse without
resetting the root budget. Canvas, facet, and specialized producer freeze tests
retain complete source, glyph and placement identity with no first-capture
reshaping. This is scoped acceptance evidence, not a claim that every possible
input or all future native owners have been reviewed.

**Verified bundle and commit partition.** After the completed r1 commit, the
shared font-plan, preparation-unit census, producer, and replay interfaces made
the final r2-r4/a1-f source migration coupled. Their acceptance obligations are
discharged by this same atomic verified bundle, not by invented per-stage commits.
The `Original intended title` lines retain the planned partition for provenance;
they are intended subjects, not actual commit history. Source implementation and
its inspected snapshots landed in `c97c1cb` (`fix(typography): close retained text
and root planning`); official data, source/output hashes, license, and fixture
provenance landed separately in `996f44a` (`test(unicode): retain official Unicode
16 conformance data`). Final `just verify` also passed after restoring the longer
original Beeswarm and HorizontalBar snapshot inputs. The explicit Rust-only
regeneration check passed separately against the official archive and extracted
members, with 11 byte-identical outputs. The migrated current APIs replace their
predecessors; no dormant
compatibility selector, alternate root representation, or old wrapping strategy
is retained to make a partial partition compile.

##### Recorded Working-Tree Measurements (2026-09-09)

The measurements below preceded the integrated verification and remain exact
working-tree observations. Their historical comparison is an earlier run
recorded in the same repair session, **not** a commit-to-commit benchmark and not
a measurement of historical commit `4b38de4`. No cap, source fixture, or expected
output was changed to obtain these results. Intermediate progress limits,
concurrent lint diagnostics, and full-suite counts below are superseded as
closure gates by the integrated result above, not silently rewritten benchmarks.

**Recorded reproduction commands.** The evidence runs used locked, offline Rust tests:

The indexed-cache follow-up freshly reran the performance and three-producer
selections below, plus these four cache/analysis tests and strict compiler gate:

```sh
cargo test --locked --offline --lib indexed_ -- --nocapture
cargo test --locked --offline --lib text_query_index_storage_is_admitted_before_allocation -- --nocapture
cargo test --locked --offline --lib text_analysis_is_font_scoped_not_presentation_scoped -- --nocapture
cargo clippy --locked --offline --all-targets -- -D warnings
```

The remaining commands retain the preceding evidence-pass results unless noted:

```sh
cargo test --locked --offline --lib typography_performance_evidence_and_root_cache_budgets -- --nocapture
cargo test --locked --offline --lib producer_plans_survive_first_capture -- --nocapture
cargo test --locked --offline --lib facet_producer_plans_survive_first_capture_without_shaping -- --nocapture
cargo test --locked --offline --lib reversed_rotated_block_preserves_existing_column_positions_in_png -- --nocapture
cargo test --locked --offline --lib producer_record_storage_refuses_before_allocating -- --nocapture
cargo test --locked --offline --lib production_title_census_prepares_soft_hyphen_for_unwrapped_and_wrapped_svg -- --nocapture
cargo test --locked --offline --lib reflow_reuses_analysis_through_unwrapped_wrapped_and_blank_paragraphs -- --nocapture
cargo test --locked --offline --lib contracted_line_break_analysis_matches_every_official_case_and_expanded_ascii_runs -- --nocapture
cargo test --locked --offline --lib packaged_unicode_provenance -- --nocapture
cargo test --locked --offline --test visual-regression -- --nocapture
```

At the indexed-cache checkpoint the performance test passed (1 test; 5.89s
including setup). The then-fresh three-producer selection passed 3/3 in 2.64s:
Canvas, facets, and specialized
producers all completed their full plan/placement identity and zero-first-capture-
shaping assertions. **The prior facet budget regression is closed by this rerun.**
Its historical failure at attempted work claim 1,000,040 is retained here only
as the before-index observation, not as a current open issue. The unchanged
60-repeat soft-hyphen/paragraph-bidi fixture passes with the indexed lookup.
The four cache/analysis tests also passed, giving eight targeted passes;
strict locked/offline all-target Clippy passed with warnings denied. The other
six named focused tests and all five visual snapshots have the preceding pass's
passing evidence. The then-reported 664 library passes and one ignored test are
superseded by the final 666-pass integrated verification above. These recorded
measurements and earlier focused runs remain historical evidence; the current
completion decision uses the primary's final verification and scoped reviews.

**Setup.** AMD Ryzen 9 7950X3D, 16 cores/32 logical CPUs, WSL2 Linux
`6.18.33.2-microsoft-standard-WSL2`, x86_64; rustc
`1.96.0 (ac68faa20 2026-05-25)`, LLVM 22.1.2. Cargo reported the test profile as
`unoptimized + debuginfo`. These are individual elapsed observations, not latency
thresholds, release-mode results, peak-memory measurements, or statistical
speedup estimates. A fresh Publication theme/font environment is created for
each workload; its caption uses 15px text and the stack
`Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif`.
The retained Latin face is DejaVu Sans, index 0, matching the independently hashed
`/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`:
`ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`.

Every setup reported 457 discovery entries, 26,548,922 admitted font bytes, and
75,103 font-work units before the measured queries. Setup times in microseconds:
`original144=769283`, `caption200=592575`, `paragraphs200=584656`,
`words128=593247`, `words256=568414`, `words512=566245`, `words2048=566370`.
Setup is excluded from the per-query elapsed times below.

The unchanged inputs are `"A ".repeat(144)`; the exact ten-word sentence
`"Measured responses remained stable across independent trials under controlled conditions. "`
repeated 20 times; the same sentence in 7/7/6 repetitions joined by newlines;
and `"Measured ".repeat(n)` for n=128/256/512/2048. Queries share one root ledger
in the order cold 768px, hit 768px, new_width 736px, return_width 768px.

**Exact measured counters.** The following transcribes every numeric query-log
field, factoring out repeated font identities and line/glyph totals. `slots` and
`bytes` are cumulative-admission deltas, including temporary work and retained
storage claims; they are **not live retained slots, allocator bytes, or RSS**.
`cum_slots`/`cum_work` are the root ledger after the query. `font_work` is a
separate font-resource delta. P/T/S/G denote proposal visits, exact width trials,
shaping input scalars, and shaped glyphs. B/H denote paragraph-bidi and font-shape
calls. Successful phases retain the same source and have these line/glyph totals:
original144=3/288, caption200=20/1800, paragraphs200=20/1800, words128=15/1152,
words256=29/2304, words512=57/4608. The refused 2048-word request has no final plan.

```text
workload/phase              elapsed_us slots work   cum_slots cum_work bytes   font_work P   T   S     G     B H
original144/cold             15188      2741  110879 2741      110879   312383  5782      146 6   784   784   1 6
original144/hit              26         0     827    2741      111706   0       0         0   0   0     0     0 0
original144/new_width        8550       990   5928   3731      117634   106739  139       146 5   484   484   0 5
original144/return_width     25         0     828    3731      118462   0       0         0   0   0     0     0 0
caption200/cold              113715     16851 223501 16851     223501   1534986 40614     219 40  5481  5481  1 40
caption200/hit               56         0     3851   16851     227352   0       0         0   0   0     0     0 0
caption200/new_width         75483      7452  36762  24303     264114   803570  139       219 39  3681  3681  0 39
caption200/return_width      54         0     3852   24303     267966   0       0         0   0   0     0     0 0
paragraphs200/cold           109139     16465 224128 16465     224128   1644264 41845     217 40  5283  5283  3 40
paragraphs200/hit            56         0     3855   16465     227983   0       0         0   0   0     0     0 0
paragraphs200/new_width      70955      7054  35427  23519     263410   761364  139       217 37  3483  3483  0 37
paragraphs200/return_width   56         0     3856   23519     267266   0       0         0   0   0     0     0 0
words128/cold                71706      10909 140951 10909     140951   1090279 16208     142 30  3564  3564  1 30
words128/hit                 43         0     2555   10909     143506   0       0         0   0   0     0     0 0
words128/new_width           48466      4894  24196  15803     167702   526123  139       142 29  2412  2412  0 29
words128/return_width        44         0     2556   15803     170258   0       0         0   0   0     0     0 0
words256/cold                140769     21805 281737 21805     281737   2173211 27728     284 58  7128  7128  1 58
words256/hit                 65         0     4859   21805     286596   0       0         0   0   0     0     0 0
words256/new_width           96798      9774  47729  31579     334325   1050719 139       284 57  4824  4824  0 57
words256/return_width        68         0     4860   31579     339185   0       0         0   0   0     0     0 0
words512/cold                276724     43597 563417 43597     563417   4339075 50768     568 114 14256 14256 1 114
words512/hit                 111        0     9467   43597     572884   0       0         0   0   0     0     0 0
words512/new_width           192281     19534 94914  63131     667798   2099911 139       568 113 9648  9648  0 113
words512/return_width        112        0     9468   63131     677266   0       0         0   0   0     0     0 0
words2048/cold               416930     73731 999862 73731     999862   8824736 189008    0   1   18432 18432 1 1
```

The 2048-word query returned the existing typed generation-work limit through
the Render adapter: limit 1,000,000, attempted cumulative claim 1,000,118. The
failed claim was not added to the ledger. This bounded refusal is allowed by the
test; it is not evidence of successful 2048-word layout or unbounded scalability.
The 128/256/512 successful cold and reflow work counts satisfy the test's measured
doubling allowance (`next <= 2 * previous + 4096`). Exact hits and return-width
hits share payload and analysis, allocate no new slots/bytes, and shape nothing.
With `B = source_bytes + stack_bytes + owner_bytes`, the indexed fixture charges
`2*B+61` for the immediate hit and `2*B+62` on return. This includes one source
hash pass plus 58 encoded header/scalar bytes, one/two binary-search comparisons,
one digest equality, and one full-key equality. An exact hit does not scan the
shaping bucket or compare the full key of the intervening width.
New-width queries share paragraph analysis, not the prior selected-line payload;
they make zero new paragraph-bidi calls but still shape the selected line trials.

**Historical comparison, same full inputs and widths.** Before paragraph-analysis
reuse and the optimized legal-break path, the recorded one-paragraph caption
used 673,339 cold work units, 16,847 slot claims, and 1,635,378 byte claims
(117,487us). After its 3,793-work exact hit, a second width consumed another
322,858 work units before refusing at attempted cumulative 1,000,246. The current
run uses 223,501 cold work and 36,762 successful second-width work, with the whole
four-query sequence ending at 267,966. The three-paragraph caption previously
used 674,846 cold work (115,368us), then refused its second width at attempted
1,000,220; it now uses 224,128 cold and 35,427 second-width work. The preceding
pre-index observation was slower than that historical run, while this indexed
observation is faster. Such individual elapsed variation is not a statistical
speedup result; reduced admitted work is not a universal wall-time guarantee.

The historical 512-word cold query refused at attempted 1,000,033 after admitting
999,777 work units; current cold plus changed width plus both hits succeeds at
677,266 total work. Historical original144 cold work was 109,140 versus current
110,879, while its changed-width work drops from 109,453 to 5,928. The original
fixture is retained, not replaced with a shorter proxy. Historical and current
2048-word queries both refuse under the unchanged cap.

**Indexed-lookup delta.** Keep the major pre-optimization comparison (673,339
caption cold work versus approximately 223k) distinct from this small subsequent
cache change. The preceding optimized-but-unindexed caption measured
223,384/34,777 cold/reflow work; the indexed run measures 223,501/36,762. Cold
successful queries add 117 work units, two slot claims, and 64 byte claims; the
caption reflow adds 1,985 work units, one slot, and 48 bytes. Immediate caption
hit work rises 3,793 to 3,851, but return-width hit work falls 5,690 to 3,852.
All proposal/trial/scalar/glyph/bidi/shape counts and retained line/glyph totals
in this benchmark are unchanged by the index. These differences reflect admitted
fingerprinting, binary search, key comparison, and index insertion/storage, not
new shaping or shortened inputs. `TextQueryCache` in `src/theme/resolved.rs`
maintains sorted exact and shaping fingerprint indexes and checks full keys in
matching buckets, including collisions. It removes unrelated-key scans in a
larger root workload, closing the facet fixture's prior cumulative-budget failure;
the tiny one/two-entry benchmark is not expected to show a universal lookup-time
speedup. Both the unindexed and indexed figures remain working-tree observations,
not separate commit baselines.

**Final Matrix reuse correction.** The primary's last integration failure was
Matrix-with-hue exhausting work through repeated per-cell tick selection. In
`src/plots/matrix.rs`, per-axis tick vectors are now selected once per variable
within each render, reducing selection calls from `2*n*n` to `2*n`; outer-axis
filtering still uses its actual projection and admitted copied records. The
primary recorded affected query counts falling from 3,575 to 1,416; these are
query counts, not elapsed times or work-unit totals. The log confirms all 18
`marginal-matrix` integration tests pass after this fix. Unit coverage is
`matrix_tick_selection_is_shared_only_along_the_orthogonal_axis` and
`matrix_outer_tick_copy_admits_bytes_before_copying`. This correction does not
replace the separately recorded text benchmark or claim all Matrix/native
scientific layout ownership is complete.

**Acceptance/disposition matrix.** Each row consolidates related earlier review
concerns instead of repeating superseded failures. The named nonignored tests
are included in the primary's passing full run; earlier focused observations
remain attributed to their checkpoints. Opt-in Unicode regeneration is the
separately primary-confirmed pass. The integrated closure establishes these
scoped acceptance stages, not correctness for every possible input or future owner.

| Owner / Concern | Current Disposition And Regression Evidence |
| --- | --- |
| T5a1r2: actual parent allocation, feedback convergence/cycles, stale geometry | Implemented in `src/layout/kernel.rs` and `src/layout/kernel/feedback.rs`; `src/layout/kernel/tests.rs`: `parent_feedback_reallocates_real_geometry_and_converges_on_round_eight`, `feedback_cycles_exhaustion_and_unchanged_misses_never_finalize`, `changed_owner_geometry_is_not_reused_on_equal_feedback`. Final facet origin/legend observations: `final_facet_legend_and_shared_axis_origin_invalidate_observations` in `src/orchestrator/facets.rs`. |
| T5a1r3: guide-only acceptance, nested budgets, swallowed arithmetic/fit errors | Complete-frame and typed-fatal coverage: `automatic_outside_guide_rejects_right_when_only_the_body_fails`, `clearance_arithmetic_failures_are_fatal_through_every_owner` in `src/decorations/plan/tests.rs`; `render_error_messages_never_select_recovery`, `sibling_collection_preserves_later_fatal_and_canonical_child_identity`, `inline_child_calls_do_not_inherit_the_parent_guide_attempt` in `src/layout/fit.rs`. Native future ownership is not claimed here. |
| T5a1r3: 40% gutters, offset panels, intact unfit text, tick errors | Percentage cap removed from active allocation/drawing; selected width is retained on the axis. `preferred_y_width_uses_available_room_and_retains_compact_wrapping` in `src/plots/horizontal_bar.rs`; `long_label_facets_measure_gutters_and_wrap_strips_inside_owned_cells` in `tests/composition/facets.rs`; `tick_planner_failure_is_not_reported_as_small_matrix_cells` in `src/plots/matrix.rs`. Snapshot/source inspection below preserves real body minima and full labels. |
| T5a1r4: final reservation before Null, canonical scratch, immutable replay | Implemented in `src/orchestrator/backend.rs`, `src/render/coordinates.rs`, `src/render/svg_backend.rs`: `all_root_preflights_reserve_before_the_only_null_lifecycle`, `every_nonempty_canonical_storage_refusal_precedes_null`, `capture_rejects_extra_calls_before_retaining_or_shaping_the_payload`, `frozen_scene_replay_consumes_exactly_the_captured_calls`. Historical primitive gate below is distinct from typography changes. |
| T5a3a1: actual transitive Unicode versions, provenance, fixture completeness | `unicode_16_versions_are_exact` in `src/font/unicode16.rs` and `packaged_unicode_provenance` in `src/font/unicode16/conformance.rs` pass in the final log. The runtime inventory, private rustybuzz normalization-version audit, original source hashes, license, and complete fixture counts are in `tests/fixtures/unicode16/README.md` and `sources.json`. The primary separately confirms `regenerate_unicode16` in `src/font/unicode16/regeneration.rs` passed its 11-original-member/11-output comparison. No expectations were regenerated by this documentation update. |
| T5a3a1: official conformance does not cover every line-break bug | Rule-derived `unicode_16_lb25_closing_punctuation_does_not_extend_digits` and `unicode_16_lb8a_only_protects_the_immediate_zwj_boundary` in `src/layout/unicode16/conformance.rs` cover the independently reproduced `1}2` and ZWJ/CM cases; official fixtures were not weakened. `official_breaks`, `official_bidi_characters`, `official_bidi_classes`, `official_nfc`, and `official_nfkc_casefold` retain the full packaged cases. |
| T5a3a1/d: normalization/line-break work and allocation before admission | `admission_precedes_input_work_and_first_allocation`, `expansion_and_every_intermediate_share_exact_admission`, `adversarial_mark_ordering_refuses_before_quadratic_work_finishes` in `src/font/unicode16/admission.rs`; `input_and_storage_refusals_precede_processing_and_pushes`, `linebreak_work_and_storage_boundaries_are_exact` in `src/layout/unicode16/admission.rs`. No third-party OOM guarantee. |
| T5a3a2: retained OS handles, short/changing reads | Bounded retained bytes replace long-lived file handles. `discovery_retains_one_handle_at_a_time_and_closes_on_byte_refusal`, `discovery_closes_handle_on_observed_file_changes`, `retained_font_read_rejects_short_and_never_reads_past_admitted_length` in `src/font/canonical.rs`. |
| T5a3a2: partial fontconfig and decoder errors | Not every partial parse is fatal under the pinned parser policy. `bounded_fontconfig_retains_directives_before_a_relevant_parse_failure`, `bounded_fontconfig_root_status_drives_fallback_like_locked_parser`, `bounded_fontconfig_included_failures_do_not_fail_parent` compare that policy explicitly; `bounded_fontconfig_missing_optional_paths_but_not_admitted_io_are_ignored` keeps admitted I/O failure fatal. `decoder_may_admit_zero_faces_from_a_retained_file` and `partial_collection_preserves_actual_decoder_face_index` distinguish decoder omission from lost identity. Owner: `src/font/canonical.rs`. |
| T5a3a3: source census copies, units, generated coverage, sealed re-selection | `formatted_census_admits_work_and_bytes_before_output_writes`, `retained_units_prefer_complete_faces_without_changing_scalar_or_primary_selection`, `sealed_environment_rejects_every_late_pair_resolution_route` in `src/font/canonical.rs`; `borrowed_source_work_refusal_precedes_source_retention`, `generated_metric_scalars_are_selected_and_missing_generated_coverage_is_fatal` in `src/layout/text/requirements.rs`. Scalar fallback is permitted only when no face covers the narrow complete unit. |
| T5a3b: exhaustive wrapping and failed-endpoint reuse | One greedy assigned-width implementation; no active `CandidateSet`, `TextCandidate`, `global_rank`, or `WrapStrategy` found in the source scan. `original_144_repeat_wraps_with_linear_selected_work`, `synthetic_trials_retreat_reuse_failure_and_stop_at_the_first_failed_extension`, `intact_overwide_sequence_returns_typed_fit_miss`, `mandatory_breaks_preserve_crlf_and_all_blank_paragraphs` in `src/layout/text.rs`. A longest-fitting-line or universal linear-time guarantee is not asserted. |
| T5a3c: script marks, unresolved extensions, fallback context and controls | `script_extensions_brackets_and_leading_marks_use_paragraph_context`, `review_unresolved_script_extensions_do_not_pick_an_arbitrary_script`, `review_context_survives_forced_face_boundaries`, `fallback_controls_keep_source_and_neighbor_face_without_nominal_coverage`, `control_only_text_does_not_require_a_space_or_control_glyph` in `src/layout/text/shaping_tests.rs`. Real generated GSUB/GPOS font tests distinguish contextual shaping from scalar concatenation. |
| T5a3c: paragraph bidi, fractional pen/ink, blank-line hard metrics | `wrapped_plans_keep_paragraph_embedding_context_and_line_l1_levels`, `selected_line_l1_l2_equal_paragraph_reference_with_embeddings_and_isolates`, `origins_round_exact_cumulative_positions_once_and_ink_uses_those_origins`, `fallback_runs_with_different_font_units_share_one_exact_pen_and_outward_ink`, `review_hard_metrics_round_outward_for_empty_lines` in `src/layout/text/shaping_tests.rs`. |
| T5a3c/e: selected soft hyphen, original source, sealed literal fallback | `selected_soft_hyphen_preserves_same_face_gpos_and_original_clusters`, `selected_soft_hyphen_uses_the_sealed_literal_hyphen_fallback`, `wrapped_arabic_context_stops_at_selected_line_boundaries` in `src/layout/text/shaping_tests.rs`. Fresh production pass: `production_title_census_prepares_soft_hyphen_for_unwrapped_and_wrapped_svg` in `src/orchestrator/lifecycle.rs`. Source spans remain original; painted ASCII hyphen is not a source rewrite. |
| T5a3d: actual GSUB output and retained plan storage gates | `actual_gsub_count_and_root_bytes_gate_every_glyph_copy` and `review_canonical_output_admission_precedes_retention` in `src/layout/text/shaping_tests.rs`; fresh pass `producer_record_storage_refuses_before_allocating` in `src/render/backend.rs`. Existing slot/work/byte caps remain active. |
| T5a3d: parser bytes and nondefault PNG options are terminal-local | `png_parser_gate_uses_outlined_bytes_without_poisoning_svg`, `invalid_png_options_do_not_initialize_the_outline_cache`, `actual_glyph_payload_limit_precedes_derived_buffer_reservation` in `src/scene.rs`; `png_with_options_uses_exact_scale_and_dpi` in `tests/output/png.rs`; `output_png_options_are_call_local_across_preview_custom_and_publication` in `tests/output/multiformat.rs`. Their earlier focused evidence is now joined by the primary's passing full library/output integration run. |
| T5a3e: discard/re-shape producer bypasses and significance style replacement | Retained `PlacedText`/`ProducerText` routes in `src/render/backend.rs` and `src/spec.rs`; producers in `src/canvas.rs`, `src/orchestrator/facets/rendering.rs`, `src/plots/{matrix,heatmap,beeswarm,line}.rs`, and `src/decorations/panel_text.rs`. No raw `draw_text`/line-extraction call remains in those producers. Fresh `canvas_producer_plans_survive_first_capture_without_shaping`, `facet_producer_plans_survive_first_capture_without_shaping`, and `specialized_producer_plans_survive_first_capture_without_shaping` all pass with complete payload/placement identity and zero first-capture shaping, including matrix legend and root-owned shared headers. The indexed-cache follow-up closes the prior facet budget regression without shortening its source. `planned_significance_capture_preserves_the_exact_deferred_plan` in `src/orchestrator/backend.rs` and `planner_retains_the_exact_effective_significance_label` in `src/significance/tests.rs` cover exact/effective provenance. |
| T5a3e: preserve-text versus outlines/PNG, rotated column order | Viewer shaping is not a universal pixel-parity contract. Retained outlines and PNG use host-generation glyph geometry. Fresh `reversed_rotated_block_preserves_existing_column_positions_in_png` in `src/render/svg_backend.rs` proves byte-identical PNG against independently placed columns without changing font/line-height style. `wrapped_bidi_preserve_text_isolates_frozen_visual_runs_and_keeps_logical_source` covers selectable logical source separately. |
| T5a3f: measured scaling, reflow cache correctness, return-width identity | Recorded exact performance run above; `reflow_reuses_analysis_through_unwrapped_wrapped_and_blank_paragraphs` and `contracted_line_break_analysis_matches_every_official_case_and_expanded_ascii_runs` in `src/layout/text/shaping_tests.rs` also pass in the final run. The contraction test compares every official line-break case and expanded ASCII runs; the optimization does not generate new expected answers. `indexed_text_queries_charge_search_and_matching_buckets_only`, `indexed_query_fingerprints_include_exact_presentation_and_provenance`, `text_query_index_storage_is_admitted_before_allocation`, and `text_analysis_is_font_scoped_not_presentation_scoped` in `src/theme/resolved.rs` cover indexed lookup, collision/full-key identity, storage refusal, and analysis scoping. Primary full verification and scoped review have discharged this acceptance; future native/public phases remain separate. |

**Historical primitive-phase gate, not regenerated expectations.** The earlier
logged passing `prepared_storage_preserves_legacy_primitive_svg_and_png_bytes`
run in `src/orchestrator/backend.rs` compared retained replay to this literal
pre-storage primitive source (100x100, one surface and one point), and compared
its PNG using Source sizing and Balanced compression:

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100" role="img">
<style>line,path,polyline,polygon,circle{vector-effect:non-scaling-stroke}</style>
<rect width="100%" height="100%" fill="#FFFFFF"/>
<circle cx="10.000" cy="80.000" r="2" fill="#010203" fill-opacity="1"/>
</svg>
```

That passing comparison printed SVG SHA-256
`864205bbd95079b2cf120226c538d88129a64ccb55e3bc0ec707659837127102`
and PNG SHA-256
`262de6cee2697b28de5b6bc996c77b4f55795bc568db6b183d254df4be37f2a9`.
These are transcribed historical logs, not fresh hashes installed as replacement
expectations. The test's before/after equality is gated to the primitive-storage
phase; it does not assert that subsequent intentional typography/layout changes
leave every chart byte-identical. Raw-font rejection and extra-call/payload
refusal tests separately gate capture before final Null/concrete lifecycles.

**Snapshot/source review.** All four changed snapshots under
`tests/visual_regression/snapshots/` were inspected against their complete source
diffs. Before the source commit, the original pre-4b38de4 longer Beeswarm and
HorizontalBar inputs were restored in `tests/visual_regression/suite.rs`, and
their resulting snapshots were inspected and updated. Other inputs are unchanged.
The HorizontalBar snapshot now includes both wrapped fragments so it verifies
the complete display label rather than omitting it from the structural view:

- `visual_regression__scatter_smoke_snapshot.snap`: grid/tick and five circle
  coordinate spellings shift by 0.001px; text, mark count, and source data remain.
  `FrameRequirements::assign_fit` in `src/layout/kernel/feedback.rs` quantizes
  assigned frame coordinates to 1/1024px with outward inset admission; the
  canonical formatter in `src/render/coordinates.rs` then spells those values.
- `visual_regression__matrix_clip_structure_snapshot.snap`: second-column clip
  x changes 181.198 to 181.199; dimensions, four clips, data, and mark counts
  remain. The same quantized parent assignment feeds `src/plots/matrix.rs`.
- `visual_regression__horizontal_bar_clip_structure_snapshot.snap`: clip x/width
  changes 131.508/198.492 to 172.422/157.578 with the restored
  `a deliberately long category` input. The snapshot retains `A deliberately `
  and `long category`, including the inter-word space, as two measured lines.
  `src/plots/horizontal_bar.rs` selects natural
  width when real room permits and `src/layout/panel/axis_frame_grid.rs` consumes
  that retained width. This is not removal of the long label or smaller text.
- `visual_regression__beeswarm_clip_structure_snapshot.snap`: clip x/width changes
  86.828/166.945 to 114.884/138.889 with restored `first`/`second` categories,
  four points, and title/colorbar text intact.
  `src/plots/beeswarm.rs` reserves the measured guide before the shared
  categorical allocator, whose natural width includes conservative fit width.

Related source-test deltas were also inspected: `tests/layout/layout_and_legends.rs`
now checks all `locus_0 aa`, `locus_0 Aa`, and `locus_0 AA` labels exactly once and
uses the widest label for clearance; no genotype input was shortened. Its new
outside-guide assertions use `tests/support/visual.rs` to require root containment,
panel separation, and no graphics overlap. `tests/interactions/connectors.rs`
and `tests/marginal_matrix/contracts.rs` explicitly select inside placement for
their inside-collision tests, retaining their datasets and geometry comparisons
under the new automatic outside-guide policy. Unicode fixture integrity passed
the packaged hash check. The separate primary-confirmed source regeneration and
the supplied final full verification now satisfy those closure gates; the
snapshot inspection and original primitive hashes remain attributed to their
earlier observations rather than newly generated expectations.

Original intended title: `test(layout): close typography repair and scaling evidence`.

### T5a3p: Post-T5a3 Repository Hygiene And Publication Efficiency

**Status**: COMPLETE

Dependencies: completed T5a3; T5a3p0 records the user's follow-up decisions.

T5a3's preceding completion and measurements remain evidence of the implemented
contract, not proof that its repository size, repeated work, or accounting model
are proportionate to publication-ready plotting. This follow-up addresses those
costs without reopening accurate shaping, complete content, or immutable replay.
It is not a global optimizer, a new public layout mode, or permission to change
fonts, shrink text, drop labels, or overwrite exact caller dimensions to save work.

The user accepted PF1-PF7 below, including full-corpus history removal and the
external-data test workflow. `05_LAYOUT_SYSTEM.md` records the minimal accepted
policy amendments; approval is not runtime completion. P1 is complete; PF2/PF3
are committed in `5c0acdf`, PF6 in `23db99f`. PF4 implementation, primary integrated
verification, independent recheck, and final comparison review are complete. PF5
is complete as evaluation/reporting with existing font freshness retained; PF7's
KEEP disposition is complete. All p0-p7 technical and disposition gates are
satisfied. T5a4 and later phases remain PENDING and have not started.
The primary's PF4 closure commit is not yet assigned an ID; intended subject:
`refactor(layout): use coarse shared processing admission`. This completion record
does not claim that commit has already occurred or authorize successor work/push.
This documentation revision performs no tests, compilation, history rewrite,
runtime change, benchmark, Git commit, or push.

**Publication gate:** do not push the current branch or any ref containing the
corpus ancestry. T5a3p1 must prove that corpus paths and original blobs are absent
from the intended published ancestry. A forward deletion, passing tests, or
completion of another phase does not satisfy this gate. Cleanup does not authorize
a push or force-push.

#### T5a3p0: Follow-Up Behavior Review And Selected-Work Admission

**Status**: COMPLETE (agreement record only)

| ID / stage | Baseline behavior and example | Accepted outcome | Agreement |
| --- | --- | --- | --- |
| PF1 / p1 | Prior full upstream corpora occupied 639,054 lines / 16,821,506 bytes; forward deletion alone retained ancestry | External official hash-verified data for explicitly invoked ignored full-conformance tests; focused default tests and small provenance/license records. Remove full corpora from ancestry/packages while preserving all later code | COMPLETE; per-stage cleanup evidence below |
| PF2 / p2 | Prior stable scatter required a second root measurement | Stop at unchanged effective feedback with no misses and a complete validated immutable plan; dirty dependency reuse, not a cache-purity assumption | COMPLETE; `5c0acdf` |
| PF3 / p3 | Prior producer ran once to count and again to retain commands | Single capture with admitted incremental fallible growth during planning; count frozen commands and reserve final canonical storage before Null | COMPLETE; `5c0acdf` |
| PF4 / p4 | Prior caption charged 223,501 synthetic units, including 137,504 fixed Unicode allowances | One shared actual-processing-visits ledger; exact replacement contract documented before implementation | COMPLETE; final verification and independent recheck passed |
| PF5 / p5 | Separate roots repeat font setup; about 0.6 seconds was a debug observation | Release evaluation and before/after report, no global font cache | COMPLETE; retain operation-local freshness and same-Output reuse |
| PF6 / p6 | Prior duplicate automatic lists/label queries under greatest-count selection | Deduplication ONLY, preserving the exact selected tick list and label bytes; no first-fit change | COMPLETE; `23db99f` |
| PF7 / p7 | Exact next-endpoint extension fills lines; 20 lines took 40 shape calls, not 40 paragraph layouts | KEEP current extension; revisit only later if meaningful benefit, via DF10 at T7 closure | ACCEPTED; disposition COMPLETE, no runtime change |

These are explicit user agreements, not acceptance by silence. PF1 supersedes
earlier full-corpus repository/package preservation requirements in this plan and
`05`; historical full-run and package observations remain true for their recorded
trees. Fresh-checkout defaults will not claim full official coverage. Preserve
strict wrapping, font coverage, exact dimensions, explicit placement, and all later
code during cleanup. No fewer-tick or earlier-line-break behavior is admitted.
Update implementation evidence and the ownership overlay when the respective work
lands; retain no old/new runtime selector. P0 completion certifies agreement only.

**Shared evidence gate for p2-p7:** benchmark identical representative inputs in
release mode before changing algorithms. Include a simple scatter, long-label
bar, 200-word single/multi-paragraph captions, facet grid, mixed Canvas, Matrix,
and separate-plot versus same-Output reuse. Separate font setup, planning,
producer capture, validation, serialization, and publication times. Distinguish
the first plot in a process from a new text query after font preparation; do not
claim to control cold filesystem caches. Record elapsed time and actual calls/
trials, with clearly identified byte/record claims or measured memory. Synthetic
admission scores are not CPU timings, operation counts, or peak RSS. No new
speedup percentage or replacement limit is accepted without those records.

Prioritize output-preserving p2/p3/p6 and accounting simplification p4. P5 is an
evaluation checkpoint; p7 is a completed KEEP disposition, not a redesign. Each
accepted change gets its own logical commit and verification; split further
before coding if shared interfaces prevent a coherent reviewable change.

**Release baseline (evaluation evidence, not optimized runtime):** supplied records
`/tmp/opencode/publication-efficiency-summary.csv` and
`/tmp/opencode/publication-efficiency-baseline.log`. Conditions: AMD Ryzen 9 7950X3D,
Linux, Rust 1.96, optimized release, one test thread, three runs, no warmup;
OS/filesystem cache state uncontrolled. SVG is outlined; PNG uses source-balanced
preview. Test source: `src/orchestrator/tests/publication_efficiency.rs`, ignored
test `orchestrator::tests::publication_efficiency::publication_efficiency_measurements`.
Reproduction command (not run by this docs update):

```sh
cargo test --locked --offline --release --lib orchestrator::tests::publication_efficiency::publication_efficiency_measurements -- --ignored --nocapture --test-threads=1
```

Times below are milliseconds, rounded to three decimals; all stage columns are
medians. Root preparation includes fonts and other preparation, not font-only
setup. Stage medians need not sum to the median total. Publication is distinct
from SVG replay/serialization and format conversion.

| Case | Min total | Median total | Prepare incl. fonts | Root measure | Capture | Null | Publication | SVG outline | PNG raster/encode | SVG replay/serialize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scatter100 | 36.899 | 46.408 | 28.343 | 0.678 | 0.190 | 0.041 | 0.262 | 0.307 | 16.533 | 0.117 |
| long_label_bar | 30.846 | 31.785 | 18.821 | 0.690 | 0.104 | 0.018 | 0.249 | 1.016 | 10.172 | 0.050 |
| caption200_1para | 81.361 | 81.485 | 18.783 | 4.704 | 0.189 | 0.043 | 0.664 | 10.683 | 46.183 | 0.138 |
| caption200_3para | 82.258 | 82.909 | 18.681 | 4.741 | 0.192 | 0.043 | 0.661 | 10.812 | 47.663 | 0.137 |
| facets2x2 | 45.416 | 46.238 | 19.104 | 2.540 | 0.466 | 0.081 | 0.483 | 0.989 | 22.380 | 0.239 |
| mixed_canvas | 46.197 | 47.232 | 18.933 | 2.642 | 0.276 | 0.058 | 0.513 | 1.217 | 23.213 | 0.170 |
| matrix4 | 72.374 | 72.537 | 19.190 | 3.565 | 1.457 | 0.396 | 0.574 | 0.847 | 45.314 | 1.111 |
| separate10 | 385.529 | 386.541 | 226.153 | 6.711 | 1.827 | 0.405 | 2.686 | 2.852 | 137.740 | 1.262 |
| reuse10 | 149.227 | 150.995 | 18.914 | 0.725 | 0.181 | 0.041 | 2.087 | 0.306 | 127.901 | 0.131 |

| Case | Roots | Root rounds | Producer invocations | Font shape calls |
| --- | ---: | ---: | ---: | ---: |
| scatter100 | 1 | 2 | 2 | 9 |
| long_label_bar | 1 | 2 | 2 | 11 |
| caption200_1para | 1 | 2 | 2 | 35 |
| caption200_3para | 1 | 2 | 2 | 37 |
| facets2x2 | 1 | 3 | 2 | 13 |
| mixed_canvas | 1 | 3 | 2 | 79 |
| matrix4 | 1 | 2 | 2 | 13 |
| separate10 | 10 | 20 | 20 | 90 |
| reuse10 | 1 | 2 | 2 | 9 |

Each root has one capture and one Null validation in this baseline, but two
producer invocations. The log records SVG/PNG/decoded hashes, sizes, and PNG
geometry; the recorded test passed once with 670 filtered tests. This is not a
new full-suite run, a memory measurement, or an after-optimization report.
The font probe's first sample separates fresh-environment setup **18.510 ms**,
first query **0.204 ms**, new query **0.093 ms**, and repeated-query hit
**0.000918 ms**. The earlier caption investigation's roughly **0.6 s cold setup**
was debug-only, not release root preparation or evidence of a measured speedup.
P5's baseline and final evaluation/report are complete; the final comparison and
retained-freshness disposition are recorded below and in `docs/resource-admission.md`.

#### T5a3p1: Remove Full Unicode Corpora From Unpublished History

**Status**: COMPLETE

The consumer migration and localized history rewrite are complete. Default
verification passed with 663 library tests and eight explicit opt-in tests
ignored (six external corpus consumers, regeneration, and the release benchmark),
all integration/snapshot targets, strict Clippy, rustdoc, and package verification.
The external conformance consumers were separately run against hash-verified
original sources; all six passed. Missing/mismatched inputs failed explicitly.
Regeneration check and write modes validated 11 original members and produced
only five retained outputs; neither recreated the corpora. Package size fell from
21.5 MiB to 5.5 MiB (1.2 MiB compressed, including the release measurement harness).

Before rewriting, HTTPS confirmed server `main` at
`4b38de440f4ee684662a05f97a84c43e38114f17`; SSH port 22 timed out. No remote ref
or configuration changed and no push occurred. Native Git reconstructed only
the six-commit unpublished suffix after unchanged `35fc235`, preserving original
author/committer metadata and message bytes. Every mapped tree was checked to
differ solely by removal of the six corpus paths where present:

| Original local commit | Clean replacement |
| --- | --- |
| `996f44a` | `22965ff` |
| `c97c1cb` | `8d1db56` |
| `0e70bdf` | `c8d51b6` |
| `b8fe313` | `12ec81e` |
| `77d8bf2` | `65f3cf4` |
| `b6f8321` | `e8764ae` |

The later typography, layout, snapshot, documentation, measurement, and consumer
migration work was not dropped. The final tree stayed exactly
`5a3ce6dcbd8eedadfe807ac1586cfc148d2360f8` before and after rewriting because the
consumer-migration tip already lacked the corpora. Updating `main` used an
expected-old-OID guard. All earlier commits and `origin/main` are unchanged.
Earlier hashes elsewhere in this document identify pre-rewrite history; the
table supplies their replacements, not a second code route.

Final history proof is recorded by `51d3b23`: all six original corpus blobs are
unreachable from all refs, and the mapped repair trees were preserved. The clean
source-consumer tip is `e8764ae` (historical original `b6f8321`); the clean baseline
is `65f3cf4` (historical original `77d8bf2`). Old IDs remain explicit historical
evidence via the mapping above, not current ref identities. No push occurred.

`git log --all -- <six corpus paths>` returned no history, and all six original
blob IDs were absent from `git rev-list --objects --all`. No backup refs were
created. The worktree and index remained clean. Old recovery objects may remain
locally through reflogs; they are not reachable from publishable refs and are not
sent by a normal branch push. Unrelated reflogs were not expired and no aggressive
garbage collection was run. This closes the corpus-ancestry gate, not permission
to push; later publication must still check current refs/server.

The completed procedure preserved all later code, not only the original corpus
introduction. Its acceptance criteria below remain recorded for auditing. These
six full fixtures and their historical blobs were removed from publishable history:

```text
tests/fixtures/unicode16/BidiCharacterTest.txt
tests/fixtures/unicode16/BidiTest.txt
tests/fixtures/unicode16/DerivedNormalizationProps.txt
tests/fixtures/unicode16/GraphemeBreakTest.txt
tests/fixtures/unicode16/LineBreakTest.txt
tests/fixtures/unicode16/NormalizationTest.txt
```

- Inspect current refs, worktrees, index, worktree changes, and actual remote tip
  before rewriting; guard ref updates against unexpected changes. Preserve
  unrelated work and the published base. Do not automatically stash/reset edits.
- Rewrite only the affected unpublished suffix, preserving repair code, intended
  snapshots, and small metadata. Do not create an in-Git backup branch or tag
  retaining unwanted ancestry. Native noninteractive Git reconstruction is
  sufficient; no Python files, `tools/`, or runtime dependency is needed.
- Migrate all six corpus-reading tests, retained-artifact checksums, and offline
  regeneration together. Default tests keep focused independently expected cases,
  version checks, and retained-table integrity. Ignored, explicitly invoked full
  conformance tests read an external official Unicode 16 directory and validate
  source hashes before parsing.
  Invoking it with missing/mismatched data fails, not silently succeeds.
- Stop regeneration from writing corpora into the repository. Keep necessary
  runtime tables, provenance/license, and table regeneration. Do not substitute
  a compressed repository blob or hidden dependency-cache lookup; add no automatic
  download or provisioning.
- Verify rewritten trees differ only by intended removals and consumer/doc
  migrations. Prove corpus paths and original blob IDs are absent from every ref
  intended for publication. Recheck remote state before any later user-requested
  push; no force-push is assumed.
- Reflogs/unreachable local objects are not sent by a normal branch push.
  Distinguish this from physical erasure; do not clear unrelated recovery history
  or run aggressive garbage collection before validating rewrite/rollback safety.
- Update coverage/package claims. Preserve prior full-run observations as
  historical evidence, not a claim that fresh checkouts still run the whole
  corpus. Verify default tests, strict Clippy, packaging, and explicit external
  conformance with valid data before closing this stage.

#### T5a3p2: Avoid Unchanged Root And Child Measurement

**Status**: COMPLETE

The root now accepts complete, miss-free plans at an effective feedback fixed
point without a duplicate round. Stable roots measure once; the release facet
and mixed-Canvas cases require two rounds instead of three. Concrete root-local
child reuse in `src/decorations/plan/children.rs` keys exact assigned geometry,
computed input policy, appearance, and relevant feedback. Unresolved or changing
results cannot become reusable next-round entries. Reuse replays observations,
fresh requests, and guide-trial accounting, not stale round-local handles.
Production cached/uncached Canvas and facet regressions preserve exact output,
glyph plans/placements, ticks/domains, and annotation geometry. An independently
changed cell reduces panel planning from three calls to two while keeping the
neighbor's same plan allocation; shared-domain changes invalidate both children.
Scoped independent review found no blocker. Full verification passed, and all
nine release workloads match baseline SVG, PNG-file, and decoded-pixel hashes.
Committed with PF3 in `5c0acdf`.

Current owners: `src/layout/kernel.rs` and `src/layout/kernel/feedback.rs`.
Before PF2 the driver required at least two successful rounds and had no child
reuse. No general-purpose public cache API was introduced by the correction.

Accepted fix: accept a complete validated immutable plan when effective outgoing feedback
equals consumed feedback and no misses remain. An empty request list alone is
not success. Propagate changed gutter, shared-alignment, owned-ink, and domain
dependencies; reuse unchanged results rather than remeasure unrelated siblings.
Successful completion takes one through eight rounds, not a mandatory two;
dirty reuse requires explicit invalidation, not an assumption of cache purity.
Retain containment checks, determinism, cycle detection, and the maximum bound
for genuinely changing feedback.

Example: if a title, plot, and right legend fit and return unchanged requirements,
their second identical measurement disappears. If a wider y-axis gutter changes
panel width, affected labels and body are remeasured; checks are not skipped.
Require identical successful output/semantic content on the baseline suite and a
test for each invalidation edge. Audit mutable caches for output independence;
do not assume a function's Rust trait proves purity.

#### T5a3p3: Capture Producers Once

**Status**: COMPLETE

`CallCountingBackend` is removed. Capture accepts a single `FnOnce` producer and
uses bounded fallible command/payload growth under the same root record/byte
ledgers. Canonical output counting still traverses frozen commands and reserves
real buffers/scratch before final Null validation. Capture refusal, scope/style
validation, cleanup, and output reuse tests pass. The release benchmark records
one producer invocation per root with baseline-identical SVG/PNG/pixels.
The integrated gate passed 677 library tests (eight explicit opt-ins ignored),
all integration/snapshot suites, strict Clippy, docs, and package verification.
PF2/PF3's shared root/capture interfaces and lifecycle count assertions are
committed together in `5c0acdf`; PF6 is the separate tick-only change `23db99f`. No finer-grained commit
history is claimed or manufactured with compatibility routes.

Current owners: `capture_text_replay_plan` in `src/orchestrator/backend.rs` and
producer calls in `src/orchestrator/lifecycle.rs`/Canvas.

Accepted fix: retain commands during one producer traversal with bounded fallible
growth, then count canonical output from those immutable commands. This changes
planning/capture admission, not the requirement to reserve canonical output
storage before final Null/concrete execution. Preserve glyph plans, scope balance,
invalid-input rejection, cleanup, and failure-atomic publication.

Prior example: a line-plot producer ran once for command counts and again
to retain commands. It now runs once; validation and SVG serialization
consume retained commands. Those passes are not removed merely because they are
called validation. Compare exact SVG/decoded PNG, producer calls, refusal timing,
and retained storage. Never execute the producer again just to size that storage.

#### T5a3p4: Simplify Resource Admission Accounting

**Status**: COMPLETE

Final implementation/evidence following PF2/PF3 `5c0acdf` and PF6 `23db99f` is
recorded in `docs/resource-admission.md`. Primary `just verify` passed in
`/tmp/opencode/pf4-final-verify.log`: 685 library tests passed, eight opt-ins ignored,
all 441 integration tests including five snapshots passed, strict locked/offline
all-target Clippy and rustdoc passed, and package verification passed with 202
files, 5.6 MiB uncompressed and 1.2 MiB compressed. Separately invoked external
official Unicode tests all passed; ignored defaults do not claim full conformance.
Independent review `ses_f793c53d9ffedekZJtzvGR365p` rechecked and closed both
byte-admission and variable-list-visit findings, with no new blocker.
The intended primary commit subject is
`refactor(layout): use coarse shared processing admission`; no PF4 commit hash
is invented before that commit.

- Font resources own the same generation ledger from operation creation. Removed
  the independent font logical-work counter and generation-work prices, with no
  alias counters. Rebinding a prepared environment to a different ledger refuses
  without changing consumption; ordinary clones retain the existing ledger.
- Fixed Unicode search/rule prices and hash/copy/materialization charges are
  removed. Variable ordering, candidate/list visits, and collision pairs remain
  guarded. Rustybuzz input includes only its actual five-scalar contexts, before
  buffer/shaping; actual output records/bytes precede first-party glyph copying.
- Cache, producer, and feedback storage retains record/byte admission. Child
  keys copy only concrete parent-replaceable values using the existing root
  environment, not full Plots or fresh quotas; equality behavior is preserved.
- Independent review corrections admit prefix/cluster and positioned-text byte
  capacity before reservation. Guide trial/ordinal searches and all retained-face
  shaping/metrics/outline searches charge actual examined entries before
  comparison, not one flat lookup price. Four focused regressions prove byte
  refusal before allocation and visit refusal before mutation/engine callbacks.
- Strict offline locked all-target Clippy passes. Full library: 685 passed,
  eight ignored. All four focused review regressions pass. All six external
  official Unicode tests passed using the
  existing local UCD, without fixture changes or corpus reintroduction.
- Release publication benchmark: all nine SVG/PNG/decoded-RGBA8 hash sets match
  the preserved baseline, with unchanged shaping-call counts. Full numeric
  timings and actual admission counts are in `docs/resource-admission.md`.
- The 2,048-word workload succeeds cold and on reflow, finishing at 709,818 root
  processing visits and 253,693 records. The public finite-box SVG case also
  succeeds. Fewer accounting units are not claimed as a CPU speed improvement.
- Primary integrated verification and independent recheck are complete. The docs
  closure performs no new test/build, commit, network provisioning, or history
  rewrite; it introduces no global cache or font-freshness change.

Concrete pre-code contract: `05_LAYOUT_SYSTEM.md`, Resources And Lifecycle,
now specifies one cumulative `1_000_000` actual-processing-visits ledger shared
from root creation across font/text budgets and clones. Count real scalar pass
inputs, normalization nodes and actual quadratic preceding comparisons,
candidate/unit/endpoint/list visits, and collision pairs. Remove fixed Unicode
search/rule allowances and hash/copy/materialization prices; glyph output claims
actual records/bytes before copying, not processing twice. Canonical Rustybuzz
input admission includes at most five trailing pre-context and five leading
post-context scalars consumed by locked Rustybuzz `0.20.1`, and occurs before
buffer/shaping. Sharing must preserve consumed quotas;
cache copying must not reset them, and replay record accounting stays independent.

Keep inclusive records/topology/calls/cache/discovery limits of `1_000_000`, the
`536_870_912` byte limit, `8` root rounds and `8` guide trials, optional exact-cache
stop at `256`, and PNG-only `32_768` axis, `100_000_000` pixels, `67_108_864` parser
bytes. Checked arithmetic precedes admission and fallible reservation; typed
refusals remain terminal. Reuse existing resource handles, use fewer than roughly
five helpers, and add no framework/public API or global cache. Preserve equality
fallback, algorithms, PF7 filling, Unicode fixtures, and successful output.

Acceptance evidence must replace synthetic-score assertions with actual input,
glyph/order/collision caps and a fitting 2,048-word positive case; run focused and
full library tests, strict offline locked all-target Clippy, and release benchmarks
checking all nine preserved output hash sets. Report actual visits/records apart
from timings in release evidence; reduced units alone imply no CPU improvement.
The external full Unicode fixtures remain opt-in. The primary's final verification
above satisfies the integrated gate; no new history rewrite or test run follows
from recording closure. The final report preserves all nine cases' raw counters,
before/after timings, and 27 matching SVG/PNG/decoded-RGBA8 baseline hashes.

Current owners: font generation ledgers, Unicode admission callbacks, text
planning, and cache accounting. In the prior recorded caption, fixed line-break
allowances charged `420*64 + 420*8 + 419*256 = 137504` units. These superseded
prices are historical evidence, not current policy, CPU instructions, or layouts.

Accepted fix: retain checked input sizes, glyph/record counts, font/source bytes,
fallible allocation, and bounded feedback/placement. Use coarse cumulative guards
for expensive shaping/collision work and remaining variable/quadratic normalization.
Remove routine per-rule/hash/copy prices and unnecessary per-claim synchronization.
Do not simply disable protection, raise a magic quota, reset counters per call,
or substitute timeouts for deterministic fit semantics.

Example: a physically fitting caption should not be rejected solely because fixed
Unicode-rule allowances exhaust a synthetic score. Truly excessive text, glyph
expansion, packing, or allocation still fails explicitly. Exact output for already
admitted inputs must remain unchanged; admission/refusal boundaries can change.
The concrete replacement table above and in spec 05 satisfies the pre-code
documentation gate: units, numeric limits, charge points, workload rationale, and
failure precedence are explicit. Final runtime and measured workload evidence,
primary verification, and independent recheck close this stage. Both normal
workloads and pathological refusal passed, not merely a lower reported score.

#### T5a3p5: Evaluate Repeated Font Setup

**Status**: COMPLETE (evaluation/report; no global cache or freshness change)

The final nine-case before/after table in `docs/resource-admission.md` preserves
all 27 baseline output hashes and unchanged shaping counts. Fresh font setup
median is 21.022727 ms versus baseline 18.510384 ms; root preparation includes
other work and is not a font-only measurement. First/new/repeated-query medians
are 0.286900/0.145248/0.001099 ms versus 0.203697/0.092979/0.000918 ms.
Three release runs, one thread, no warmup, and uncontrolled OS caches/scheduling
yield noisy wall timings, not CPU measurements or isolated causal speed claims.
The retained 1,312.165787 ms caption sample includes 1,231.117979 ms in publication,
a large publication/I/O outlier whose cause was not isolated; no sample was dropped.
Final disposition: retain operation-local preparation, same-Output reuse, and
existing installed-font freshness/lifetime. No cross-operation/global cache is
admitted. Source parameters, text, and PF7 line extension are unchanged.

Current setup is shared within a root and across chained Output publications,
but separate Plot operations prepare fonts separately. Recorded setup of roughly
0.6 seconds and caption querying of roughly 0.11 seconds are debug observations,
not release end-to-end promises.

Example: ten independent figures repeat font setup; ten formats from one Output
already reuse its scene. Measure that distinction in release mode. First optimize
within the existing lifetime; add no permanent process/global cache or public
context object here. Cross-operation reuse can delay observation of installed-font
changes and retain memory longer. If release evidence justifies it, present
freshness, invalidation, memory, and lifetime choices before separate admission.
This review is concluded with current freshness retained. No timing improvement
is promised for this evaluation-only stage.

#### T5a3p6: Avoid Duplicate Automatic-Tick Work

**Status**: COMPLETE

Complete candidate vectors are deduplicated before fitting; a bounded stack-local
metric cache also reuses labels shared by distinct candidates. Greatest-count
selection and earlier-source ties are unchanged. The 192-case regression compares
position bits and label bytes against the previous algorithm: identical selected
lists with 726 size-helper calls instead of 3,468. Five identical sources require
two calls instead of 20; different lists sharing labels require five instead of
30. These are lookup counts, not timing or shaping counts. All 30 panel tests,
strict Clippy, and the integrated `just verify` gate passed without snapshot changes.
Committed in `23db99f`.

Current owner: `src/layout/panel/range_ticks.rs`. The active code tries several
target-derived lists and retains the fitting list with greatest count; the prior
duplicate label-size requests are now reused. This is not the future native T5a5 source
algorithm or an unbounded search.

Accepted fix: deduplicate identical complete lists and measure each distinct
label once per relevant font/constraint context; preserve the exact selected list,
including positions, label bytes, order, and selection tie behavior.
Example: measuring `0,2,4,6,8` twice becomes one measurement with identical tick
positions. Keep uniqueness, finite-number and containment checks; exact caller
ticks remain untouched.

An optional first-fitting rule might stop at a readable four-tick list instead
of examining a later fitting five-tick list. This illustrates a possible difference,
not a measured result for a particular dataset. It requires separate user approval
and measured benefit; do not include it silently in deduplication or preempt T5a5.

#### T5a3p7: Review Speculative Line Extension

**Status**: COMPLETE (KEEP disposition only; no runtime change)

Current owner: `src/layout/text.rs`. Paragraph analysis proposes an endpoint;
exact shaping retreats if necessary, then tests following legal endpoints to fill
the line. The recorded 20-line/40-shape-call workload is not 40 paragraphs.
Exact glyphs, bidi, soft hyphens, and ink-fit checks remain useful.

Excluded alternative: freeze the first exactly fitting proposal after necessary retreat,
without extending it. Illustrative difference: a word fitting on the current line
may instead move to the next one. A caption can become taller, and tight exact
height can fail where current filling succeeds. No character, font size, direction,
or explicit break changes; additive word estimates do not replace final shaping.

Accepted disposition: KEEP current line extension. Revisit only later if measured
benefit justifies the wrapping/fit tradeoff, under new DF10 in `99_DEFERRED_FEATURES.md`
at T7 CLOSURE after p2/p3/p4/p6 evidence. This closes p7's disposition, not an
implementation or a new wrapping mode; no additional p7 review blocks this
follow-up's closure. Admission would require separate user approval and a new phase.

**Completion gate satisfied:** p0 records every accepted disposition; p1 supplies
the all-ref corpus-history proof; p2/p3/p6 have their recorded commits; p4 supplies
final focused/full verification, independent recheck, unchanged output hashes,
and before/after evidence; p5 concludes evaluation with unchanged freshness;
p7 retains line extension with DF10 review at T7 CLOSURE. No tests were weakened
to accept a faster result. The primary will record the PF4 closure commit under
the intended subject above; no ID or completed commit is claimed in advance.
T5a4 and later phases remain PENDING, not started by this closure. Historical
T5a3 evidence and old mapped IDs remain historical; the final evidence above
certifies this follow-up. No push is requested or performed.

### T5a3q: Local-Search Benchmark Coverage And Placement Comparisons

**Status**: COMPLETE

Dependencies: completed T5a3p; user approval of `tests/benchmarks/` and measurement
of the remaining local choices. This stage measures existing behavior; it does
not change placement priority, wrapping, tick selection, font policy, or limits.

The original nine end-to-end publication workloads contain no nonempty legends.
Their root/shape counts and timings therefore do not measure Auto placement cost.
The earlier low-single-digit-millisecond estimate is a hypothesis, not acceptance
evidence. Record actual trials and decisions rather than substituting worst-case
caps, synthetic visit totals, or aggregate font-shape calls for those counts.

#### Organization And Execution

- Move the existing ignored release harness to `tests/benchmarks/suite.rs`, loaded
  as a crate-private test module from `src/orchestrator/tests.rs`. Add focused
  sibling modules for legends, annotations, text/specialized layout, and fonts.
  Reuse the Rust test harness and existing helpers; no public instrumentation API,
  benchmark framework/dependency, Python, `tools/`, or separate executable target.
- Keep expensive measurements explicitly ignored in ordinary runs. Use
  `cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1`.
  Small counter/fixture-contract tests remain normal tests. No wall-time threshold
  makes ordinary CI fail. Compiler validation remains strict all-target Clippy.
- Generate small deterministic input data in Rust. Do not commit rendered images,
  full upstream corpora, or large raw result collections. Record concise results,
  reproduction conditions, and interpretation in documentation.
- Preserve the original nine workloads as a no-guide control and regression
  baseline. Separate font setup, planning, capture, validation, serialization,
  rasterization, and publication. Counters/timers are opt-in and `cfg(test)` only;
  report instrumentation overhead and uncontrolled OS/scheduling conditions.

#### Required Coverage

| Group | Cases and comparisons | Actual observations required |
| --- | --- | --- |
| Outside legends | Same data under Auto, explicit outside right, and explicit bottom: a small right-fitting legend, a narrow/body-starving right arrangement, and many entries needing reflow | Nonempty legend assertion; each attempted edge/flow, owning round, body check, rejection/provisional/acceptance, final successful placement, calls and timings |
| Shared guides | Nonempty Canvas/facet guide with adequate and pressured space | Shared-owner attempts versus root rounds; cached/repeated work and final placement |
| Inside legends | Sparse and crowded geometry with a populated inside legend | Corner/clearance choices and actual geometry/annotation overlap scans, not just four possible corners |
| Significance | Valid increasing bracket counts that exercise automatic headroom | Level checks, actual range-growth and bisection trials, repeated calculations, elapsed cost and final fit |
| Heatmap text | Wrap and existing elision policies with short/long labels and multiple categories | Prefix-fit trials, text/shape work, selected text and preserved uniqueness; distinguish policy choice from searching between modes |
| Matrix text | Short and long variable names at adequate/tight cell sizes | Actual local label/cell iterations separately from root feedback and tick work |
| Font coverage | Controlled narrow/wider candidate and character coverage sets, plus representative system-font preparation | Candidate counts, scalar/unit coverage probes, cache reuse and elapsed setup; keep font candidates distinct from layout trials |
| Text wrapping | Short and multi-line paragraphs with the existing extension rule | Proposal visits, exact line trials, shape calls, retained lines/glyphs and cache reuse; no alternative paragraph count inferred from shape calls |

Current Beeswarm bin offsets and fixed panel-annotation corners are not tangent
packing or automatic annotation-corner searches. If included as controls, label
their deterministic work accurately rather than invent candidate counts.

#### Comparison Rules And Acceptance

- Auto accepts the first complete fitting arrangement in the existing
  right/preferred, right/compact, top, bottom, left order. Do not describe it as
  selecting the aesthetically best position. Explicit right fixes the edge, not
  the absence of text reflow or compact flow.
- Pair successful Auto with the explicit placement that it actually selects;
  compare decoded PNG output and retained geometry, and report SVG/PNG hashes.
  If explicit right fails while Auto succeeds, report failure separately rather
  than calling the failed operation a faster successful render.
- Use repeated release samples with deterministic inputs and interleaved policy
  order for paired cases. Report sample count, minima/medians and outliers, build
  profile, hardware, font environment, dimensions and output modes. Do not turn
  noisy measurements into a universal speedup or maximum-latency claim.
- Counters must distinguish attempted work, cached replay, and successful final
  choices. A provisional selection is not final fit; record the root round and
  terminal outcome. Add normal tests proving counter activation/reset and that
  fixtures genuinely reach the named paths. Failed fixtures must not disappear
  from the report or silently substitute a simpler workload.
- Preserve rendering algorithms and existing snapshots. Any discovered functional
  bug or proposed optimization needs a separately scoped correction, not an
  unreported change to make a benchmark look better.
- Run all explicit benchmark groups, scoped counter/behavior tests, strict Clippy,
  and `just verify`; inspect and document results before completion. The outcome
  is a cost/coverage assessment and prioritized recommendations, not automatic
  permission for another optimizer or a changed public API.

Commit: `test(perf): cover local search and legend placement costs`.

#### Completion Evidence

The harness now lives in `tests/benchmarks/`, with the nine original no-guide
controls unchanged and focused legend, annotation, text, and font modules.
See `docs/local-search-benchmarks.md` for reproduction conditions, measured
minima/medians/maxima, real search/cache counts, output comparisons, observer
overhead, and prioritized recommendations. All 10 explicit ignored release
measurement tests passed. `just verify` passed strict Clippy, 695 library tests
(17 ignored), 441 integration tests including five unchanged snapshots, API docs,
and package verification (207 files). All 27 original control hashes match.

Coverage is bounded by the current API, not expanded to manufacture comparisons:
populated public panel guides cannot reach automatic inside-corner selection,
so inside fixtures verify fixed-corner clearance and actual geometry/annotation
comparisons; Canvas exposes no explicit shared-edge selector. End-to-end shared
fixtures record zero cache hits, with controlled identical-input child tests
separately proving cache hits and feedback replay. Font observer activation is
measured, not production zero-hook or whole-render instrumentation overhead.
The root timer now includes complete feedback rounds, excluding finalization;
historical root-stage timings are not directly comparable. These limitations
are reported rather than assigned fabricated counts or new product behavior.

The old baseline source/test paths above describe their historical location.
For an isolated current baseline process, run:

```sh
cargo test --release --locked --offline --lib benchmarks::publication_efficiency_measurements -- --ignored --nocapture --test-threads=1
```

The all-groups command does not make the control's first scatter the process's
first font environment. T5a4 remains pending; this stage changes no rendering
algorithm, font policy, public API, snapshot, or deferred-feature decision.

### T5a4: Root Title-Block Plans

**Status**: COMPLETE

Dependencies: completed revised T5a1/T5a1r and T5a3 through T5a3f. Their repair
gate is satisfied by the integrated verification, not merely historical commits.
The post-T5a3 T5a3p and T5a3q gates are complete. T5a4 passed the primary's
integrated verification after the deferred-placement scope repair below.

Objective: establish the reusable root title-block producer and migrate the
standalone Plot root without requiring composition adapters early.

Scope:

- Build one immutable title-block result at the parent-assigned width from the
  existing retained text plans, resolved text roles, and `CompositionTheme` spacing.
  Use `TitleAlignment` with theme defaults `root_title_alignment = Center`,
  `root_subtitle_alignment = None`, and `root_caption_alignment = Start`.
- Establish private optional Plot title/subtitle instructions and Canvas panel
  title/subtitle defaults. Resolve title as Plot instruction, Canvas panel
  instruction when embedded, then resolved Plot theme. Resolve subtitle as Plot
  instruction, Canvas panel instruction, optional resolved-theme subtitle, then
  final effective title. None means no instruction, not forced inheritance over
  a lower-priority subtitle instruction. Freeze concrete effective alignments.
  Canvas root headings use only their own theme; panel instructions do not affect
  captions, TextPanels, strips, row titles, or other shared headers. This narrow
  policy does not merge themes or infer intent from preset/default equality.
- Supply intrinsic requirements separately, measure height after deterministic
  wrapping, and return a typed fit miss when exact allocated geometry cannot fit.
  Admit text and plan storage before materialization; do not search title widths.
- Apply each role's independent alignment consistently to per-line translation and
  horizontal block placement over its local post-inset content span, not the data
  panel. Retain zero rotation. Unspecified subtitles follow the effective title
  even when title text is absent. Include both effective values in plan identity.
- Migrate the standalone Plot root. T5a6a connects the same producer to faceted
  Plot, Matrix, and Canvas outer-root adapters after those boundaries exist.
- Store exact lines, widths, baselines, paints, spacing, boxes, and serialization
  inputs before Null validation.

Exclusions:

- Faceted Plot, Matrix, and Canvas outer-root integration before T5a6a; panel
  headers, Canvas headers, row titles, text panels, facet track placement, and the
  public T6 alignment type/setter.

Deletion targets:

- Standalone-root all-role centering and whole-pixel placement rounding, parallel
  title-plan/baseline carriers, and duplicate root gaps/reserves. Earlier repairs
  already removed draw-time wrapping and added source/cardinality checks; reuse
  those guarantees rather than rebuilding text preparation. T5a6d removes the
  deferred outer-root placement route after every adapter consumes this producer.

Evidence:

- Title-only, subtitle-only, caption-only, all-role, unequal multiline, independent
  Start/Center/End, all four presets, owner selection, baseline cardinality,
  physical Start/End under RTL/mixed text, odd widths/asymmetric gutters, mandatory
  blank lines, whitespace presence, source-preserving soft hyphens, exact-height
  refusal, Null/SVG identity, and explicit-size snapshot tests.
- Private precedence tests: Canvas panel Start affects Nature/CHI panels; Plot
  title End overrides it and an unspecified subtitle follows End. Canvas panel
  subtitle Center instead stays Center until Plot subtitle overrides it. Explicit
  custom-theme subtitle is used only below both instruction scopes. Theme/setter
  order is independent; repeated setters replace their own input. Caption End
  changes neither title nor subtitle. No public builders land before T6.
- `just verify` passes.

Commit: `feat(layout): plan root title blocks`.

Implementation evidence (2026-09-11):

- `decorations/title_block/plan.rs` supplies independent intrinsic widths and one
  assigned-width immutable result, with retained text, role boxes, physical
  alignments, baseline placements, active spacing reserves, and typed fit misses.
- Standalone Plot uses this producer. `CanvasRootText` separates the planned
  result from the deferred composed-root carrier, so a standalone root cannot
  carry parallel text plans and baseline vectors. Matrix, facet, and Canvas
  adapters remain deferred to T5a6a; panel integration remains deferred to T5b.
- Private Plot inputs survive marker erasure; private Canvas panel inputs remain
  separate from root ownership. Built-ins resolve Center/None/Start, with the
  specified optional subtitle chain. Public `Theme` remains the closed enum and
  no public alignment builders were added.
- Retained-text canonicalization preserves subpixel anchors instead of snapping
  them to whole pixels. The P2 audit found that rounding the first-line anchor
  alone did not preserve deferred multiline placement: the subsequent half-block
  offset could still be fractional. `PlacedText::from_deferred_first_line` now
  explicitly snaps the final block anchor with the former canonical nearest-even
  rule. It replaces the old helper, with no alias or backend compatibility mode.
  Facet strips/shared axes, Canvas row titles/column headers/panel headers/
  TextPanels, Matrix ticks/labels, and beeswarm/heatmap colorbars use that policy.
  Bracket-label placement and the deferred annotation projection predictor also
  retain their final pixel snap. Raw-text capture freezes the raw API's canonical
  position before converting to a retained call; replay validates the same raw
  position policy. Null, geometry, and SVG still share subpixel validation.
- The remaining deferred-placement audit found final producer-side rounding in
  standard axes, deferred outer-root titles, translated legends, panel text,
  bar/horizontal-bar/interval-estimate labels, waterfall annotations, line-point
  labels, Canvas row headers, and heatmap cells. These needed no new placement
  route. No composed owner is migrated to the T5a4 root producer by this repair.
- P2 regression evidence compares actual multiline facet strip and horizontal/
  reversed vertical shared-axis plans and SVG bytes against the old integer
  canonical placement. Additional checks cover positive/negative half-pixel
  nearest-even offsets, both bracket orientations, and fractional raw capture.
  The standalone-root test continues to require fractional retained placement.
- Focused evidence covers all role-presence combinations, all presets, all 27
  independent alignment combinations, RTL/mixed text, mandatory blank lines,
  whitespace, source/cardinality rejection, exact-height refusal, private
  precedence/order/replacement, and production Null/SVG replay. The existing
  production soft-hyphen test exercises the migrated standalone producer.
- Inspected snapshot changes: the new 333x401 all-role root retains fractional
  centers/baselines and Start captions. Beeswarm, horizontal bar, and marginal
  snapshots reflect measured text height instead of line-count reserves. The
  horizontal bar's existing height-sensitive axis allocation consequently wraps
  its long category into four lines rather than two. Matrix and untitled scatter
  snapshots are unchanged.
- Passing implementation-owner checks: strict offline all-target Clippy;
  `cargo test --locked --offline --lib decorations::title_block` (6 tests),
  `--lib title` (14 tests),
  `--lib lifecycle` (43), `--lib render::` (95), `--lib layout::canvas` (5),
  `--test layout` (20), and read-only `--test visual-regression` (6). The final
  admission test also covers typed intrinsic-width refusal and storage exhaustion.
  These focused checks preceded primary integrated verification.
- P2 follow-up checks pass: strict offline all-target Clippy; `--lib deferred_`
  (12), `--lib render::` (97), the fractional raw-capture regression (1), the
  standalone fractional-root regression (1), and integration targets composition
  (57), heatmap (35), marginal-matrix (18), and visual-regression (6). The snapshot
  run was read-only and required no further updates. No `just verify` or commit
  was performed by the implementation owner.
- Primary `just verify` passed: formatting, strict all-target Clippy, 705 library
  tests (17 ignored), 442 integration tests including six snapshots, API docs,
  and package verification (210 files). Log: `/tmp/opencode/t5a4-verify.log`.

### T5a5: Axis, Range, Tick, Frame, And Grid Plans

**Status**: COMPLETE

Final implementation and acceptance audit (2026-09-11): the axis-owned source,
retention, measurement, and replay gates are closed. The existing final
`just verify` log, `/tmp/opencode/t5a5-final-verify.log`, records formatting,
strict locked/offline all-target Clippy, **797 library tests passed (17 ignored)**,
**442 integration tests including six snapshots**, warning-denied API docs, and
package verification (**219 files**). All ten normal benchmark contracts pass in
that run. `/tmp/opencode/t5a5-final-benchmarks.log` separately records **all ten
ignored release groups passed**, zero failed, 804 filtered out (8.41 seconds).
These are completed implementation runs, not commands rerun for this documentation
closure; no commit is claimed. Normative scope below is unchanged.

Final acceptance matrix (paths relative to `src/` unless prefixed `tests/`):

| Gate | Implemented Result And Passing Evidence |
| --- | --- |
| Numeric source policy | `layout/panel/sources.rs` and `sources/tests.rs`: assigned-span `6ch` descending seeds, complete interval sources with `1,2,3,5` coarsening, exact precedence, conditional terminal midpoint; wide/small spans, representable-index boundaries, `10_000/10_001`, no subsequences, deterministic local order and exhaustion |
| Declared caps and arithmetic | Linear adaptive `upper+1`, integral-log `min(upper,n)+1`, additive at most `4*633+1`, integer-log at most `4*20+1`; checked index products and scaled finite extremes; empty/dense/unrepresentable trials miss locally, resource refusal remains fatal |
| Exact ticks, formatting, census | Configuration and prepared-domain validation enforce indivisible `1..=10_000` lists, finite increasing positions and containment before fonts; empty/duplicate label bytes remain permitted. Source tests cover nearest-even formatting, whole-list scientific uniqueness, integral/nonintegral Log10 labels and exact projection keys; the closed source alphabet supplies the pre-font census |
| Pre-font domains and sharing | `plots/shared_x{,/prepared}.rs`, Canvas D1 and lifecycle tests: retained standalone/faceted x/y domains, raw/display categories, titles and exact specs; Fixed/FreeX/FreeY/Free reconciliation; Matrix variable domains/labels; all 25 branches, 17 shared-x eligible and eight rejected, zero discovery on owned invalid inputs, including one-step-outside automatic ticks |
| Immutable semantics and composition | `layout/panel/semantics.rs`, axis-plan, builder-facet and child-plan tests: automatic/configured/shared provenance, linear/Log10 interpretation, range/interval policy, projection endpoints and source kind/ordinal/cap; `StandardPanelInputs` retains typography and preferred `4:3`; cache identity covers provenance, intervals, title/parent-width allocations and standard typography; cached/uncached Forest/shared-domain parity |
| Assigned cells and categorical fit | `layout/panel/cells.rs`, panel/axis tests: adjacent nonempty-label midpoint cells, inward endpoints, shared 4px clearance, exact integer-ratio category bands with nearest-even quantization and reverse order. Empty labels retain tick/grid positions without text; legal wrapping and intact overwide/row-height failures preserve D2 |
| Native axis/frame/grid replay | `PanelAxisPlan` retained by `PlotRenderResult`/`PanelPlan` freezes text, cells, ticks, grid, frame sides and strokes during measurement. Drop-input/clone tests replay without layout/theme or additional shaping; empty positions and reserve/draw parity survive. Science draws four sides once |
| Measured gutters and feedback | Panel and decoration-plan tests cover complete title blocks, quarter-turn ink, stroke/tick reserve, outward quantization, fractional-origin invariance, 39%/40%/41% gutters and real residual failure. Root/composition ownership replaces booleans and local retries; pending growth and inside-domain proposals use the existing bounded root feedback protocol |
| Post-freeze SOURCE boundary | `PlotRenderContext::axis_layout`/`axis_range`, lifecycle and specialized producer tests: all 25 canonical families have zero capture range/spec/source preparation, numeric source selection, formatter-list attempts and tick-fit lookups; positive preparation observations keep instrumentation live. Matrix capture requires retained sources and matching spans; native replay adds no shaping |
| Resource admission | Family manifest and final audit below; cumulative operation visits/bytes precede source temporaries, retention and measurement copies, checked fallible Vec/String growth, refusal-before-work/mutation and numeric parity tests. `spec::native_axis_consumption_admits_every_owned_source_copy` covers categorical, exact and title copies |
| Typed local fit failure | `decorations/plan/tests.rs` and `plots/matrix.rs`: terminal numeric exhaustion remains scoped `FitKind::Tick`, not a fatal generic error or small-cell error; Marginal Auto can reject Right and accept Top, fixed Right preserves its axis address, Matrix preserves Canvas/Matrix-cell/axis ancestry; later fatal source errors still win |
| Deletion and regression closure | No shared `AUTOMATIC_TICK_TARGET`, five-only downsampling, shared-x renderer range probe, draw-time axis text planning, duplicate horizontal helpers or local axis/title stabilization remains. Range broadening retains its independent private five-sample seed; no public count/density/interval API or old-policy route was added. Full verification and ignored release groups pass |

Important fixes and intentional output changes:

- Density pre-font x domains use configured display limits when present, not KDE
  support, so exact endpoint ticks agree with measurement. The regression
  `density_explicit_endpoint_ticks_use_display_limits_not_kde_support` passes;
  full-sample KDE preparation and clipping semantics remain unchanged.
- Owned measurement-time category/exact/title copies now receive the same source
  admission as preparation. Marginal/Matrix dispatch preserves scoped Tick misses
  through local selection. Forest shared exponent-space domains are not transformed
  twice, and Waterfall pre-font errors preserve their original source row.
- Compact-guide gutter feedback remeasures changed assignments rather than rejecting
  a stale guide. The original 640x400 reflow selects compact Right in two rounds;
  pressured facets select Right in three rounds without a provisional Top detour.
  Auto/explicit output equality and cache-disabled complete-frame checks pass.
- Heatmap long Elide records 400 actual prefix candidates: 200 in unsuccessful
  root 0 and 200 in successful root 1, asserted in normal and release contracts.
  The former 500 expectation has no established historical cause and is not an
  optimization baseline. Labels, fixture dimensions, uniqueness/failure gates and
  the historical T5a3q data remain intact; see `docs/local-search-benchmarks.md`.
- Four reviewed snapshots intentionally change: `scatter_smoke`,
  `root_title_block_explicit_odd_size`, `horizontal_bar_clip_structure`, and
  `beeswarm_clip_structure` under `tests/visual_regression/snapshots/`. Adaptive
  short-axis tick/grid removal (including `7.5`/`12.5`), inward axis placement and
  measured categorical/title gutters explain the differences; root-title block
  geometry is unchanged. All six snapshots pass in the final gate. Line collision
  fixtures now constrain an interior cell; horizontal clip checks compare measured
  long/short labels rather than the deleted second inset.

Final activation-source resource audit: no known remaining admission finding in
the activated axis-source graph. This includes helper-internal work/storage and
retained/measurement copies, not merely the dispatcher resource argument.

- `shared_x/prepared.rs::prepare_plot` returns only `PreparedPlotAxes`, not an
  owned Plot. It borrows source configuration during preparation and attaches
  retained axis handles to the existing measurement-time Plot only after font
  preparation. It no longer deep-copies supplied geometry, exact ticks, label
  policy, titles, annotations, orders or other unrelated Plot configuration.
  `PreparedAxes` itself is not Clone.
- `text_census_panels` now owns admitted row/selector buffers while borrowing its
  source Plot. There is no provisional panel-box vector, base-Plot construction,
  per-panel Plot clone or significance-vector clone on this route. Shared facet
  selector construction, categorical checks, Cartesian cell storage, column
  temporaries, row selection, source-row mapping and selector retention all use
  the operation-bearing data view. Pre-font significance ownership validation
  uses these same panels instead of constructing a second provisional set.
- `FacetAxisSources` borrows only global Bar group slots and Trajectory condition
  order. Unused collective/color metadata is not prepared. Empty child axis
  titles and the existing generic-facet Bar bootstrap identity behavior are
  preserved without cloning or rewriting the source Plot. Scope strings are
  built on demand with checked, admitted String capacity. Fixed categorical
  comparisons charge before comparison; numeric unions charge read and write
  passes before applying a union. Free axes bypass those passes.
- Labels use one resolution/humanization implementation. Axis titles select
  borrowed inferred/verbatim sources before their admitted retained copy. Alias
  policy is borrowed, not cloned. Collision checking retains raw references and
  one display vector rather than a second map of owned raw/display strings.
  Alias lookups and collision comparisons have admitted work bounds, including
  empty identities. Humanization uses bounded fallible split/output Strings,
  with no character vector, word vector, per-word lowercase String, joined
  result vector or infallible output growth. Matrix variable labels and ordinary
  raw/display categories use this same admitted owner.
- Ordinary retention uses `Arc<Vec<_>>`, moving the already admitted vector
  rather than copying it into an Arc slice. Matrix and ordinary Arc allocations
  claim the checked reference-count-header plus value layout before `Arc::new`.
  Arc handle cloning does not copy axis payloads. Stable std Arc construction
  remains abort-on-allocator-exhaustion: these are pre-construction budget
  refusals, not a claim of recoverable std Arc OOM. Vec/String growth remains
  checked and fallible; error diagnostic formatting is not a general allocator
  interception or OOM-recovery facility.
- The final graph audit additionally found Regression's axis-only `unzip`.
  Its two channel buffers now reserve before filling; the allocation-free OLS
  finite/scale/mean/product/residual scans and bounded band evaluations are
  admitted before work. No regression algorithm changed. The audit also added
  the missing repeated scalar range/status scans in the closed dispatcher, the
  PointInterval value-order status scan, and a bounded stack context formatter
  for Line supplemental-axis ranges.

Activated family-source manifest (all 25 PlotKind branches; the shared-x entry
uses the same operation state for its 17 eligible families and rejects the
other eight before fonts):

| Family | Actual Source Owner And Resource Disposition |
| --- | --- |
| Scatter | `shared_x/prepared.rs`, `data.rs`: aligned finite pairs, two admitted scalar reductions, configured/automatic range seeds; no grouping or marks |
| Regression | `regression::axis_ranges`: admitted aligned/channel buffers, allocation-free authoritative OLS with preclaimed scans, bounded endpoint/band evaluations |
| Histogram | `histogram::{axis_ranges,shared_x_domain,prepare_edges,prepare_histogram}`: admitted samples, sorted quantile scratch, edges/counts/heights/bin comparisons; supplied validators receive the gate; composite KDE maximum uses admitted work and a stack grid or borrowed curve |
| Density | `density::{axis_ranges,shared_x_domain}` and their common preparation: admitted channels, reusable group samples, support/bandwidth scans, grids/densities and maximum reduction |
| Q-Q | `qq::shared_x_domain`, `shared_x/prepared.rs`: admitted observations, theoretical endpoint quantiles, observed-y scalar reduction; no sort or reference-line payload |
| Line, Trajectory, LinePath | `line::axis_inputs`: admitted numeric/condition sources and range reductions; borrowed global facet conditions; supplemental scalar extents and stack-only diagnostic context; no path grouping/sorting payload |
| Marginal | `marginal::{axis_ranges,shared_x_domain,build_groups}`: admitted nested groups/channels and support reductions; visible-support KDE uses admitted work/storage and a stack grid; resource refusal propagates rather than skipping a group |
| Bar, StackedBar, HorizontalBar, StackedHorizontalBar | `bar_data::{axis_inputs,prepare_core}`: admitted aggregation, bounds/order/permutations, signed stacks, generated intervals/quantiles/bootstrap buffers and every RNG rejection-loop draw; borrowed facet group slots/unit keys, reused scratch and scalar supplemental extents; exact limits omit unused interval work; refusal/non-mutation and bit-exact ordinary/unequal-unit bootstrap parity tests |
| Boxplot, Violin, Raincloud | `distribution_sample::prepare`, `shared_x/prepared.rs`: admitted categorical/numeric channels, nested groups/observations, raw extrema or KDE support scans and scratch; supplied Raincloud validation/labels receive the same gate |
| PointInterval | `point_interval::{axis_categories,shared_x_domain,prepare}`: admitted tuple/row/normalization/order/permutation buffers and status/range scans; fixed-x unavailable panels use admitted sibling tuples |
| Coefficient, Forest | `interval_estimate::{axis_categories,shared_x_domain,prepare_estimate_tuples}`: admitted tuples, raw labels and uniqueness work; scalar references/bounds/Log10 transformation; fixed-x sibling input stays in the same operation |
| Dumbbell | `dumbbell::{axis_categories,shared_x_domain}`: admitted normalized observed levels and endpoint channels/reductions; no connector/offset payload |
| Interaction | `shared_x/prepared.rs`: admitted observed categories and lower/upper channels/reductions; no trace/style payload |
| Beeswarm | `shared_x.rs`, `shared_x/prepared.rs`: admitted categorical levels and numeric reduction; no swarm/colorbar payload |
| Waterfall | `waterfall::shared_x_domain`, `shared_x/prepared.rs`: admitted labels/deltas and cumulative endpoint work, preserving source-row diagnostics; no annotation or primitive payload |
| Heatmap | `heatmap::{axis_categories,category_levels}`: shared admitted normalization, Cartesian/triangular ordering and explicit-order copies; no cell/color/highlight grid |
| Matrix | `matrix::{prepare_axes,build_groups,validate_group_kdes,matrix_support_range}`: admitted variable reference/label buffers, nested channels/groups/bandwidths and support scans, then pre-admitted Arc retention; no cell packing/primitive payload |

Common source gates are `PlotData::{reserve_source,collect_source,
copy_source_string,normalize_source_string,reserve_source_string,source_arc}`,
`data::explicit_category_order`, `SourceAdmission`, the fallible in-place source
sorter, and the admitted KDE/geometry validators. All calls reached from the
activation dispatcher carry its operation resources. Existing configuration
validation, ordinary mark/statistical preparation, facet strip/legend work and
post-font Plot copies remain under their existing owners; this audit does not
claim a T5a6a/T5b/T5c payload migration or generic process-wide allocation control.
They are not new blockers assigned to this axis-source unit.

Final retention refusal and parity evidence:

- `retention_boundaries_share_the_operation_without_retaining_a_plot_clone`:
  standalone/faceted/Matrix exact work-and-byte admission boundaries, one-less
  refusal, zero font discovery, no retained source Plot handle, and unchanged
  axis storage budget when large unused title/caption/alias values are present.
- `borrowed_facet_panels_and_scope_sources_refuse_before_materialization`:
  work/storage refusal, source pointer identity, selected/source row identity and
  exact scope text. `fixed_facet_comparisons_and_union_refuse_before_their_work_or_mutation`
  checks category-comparison refusal and union read/write boundaries with zero
  storage, preserving ranges until write admission.
- `borrowed_facet_axis_views_preserve_bootstrap_and_global_conditions` compares
  sampled grouped Bar facet domains with the authoritative existing child
  preparer and checks complete global Trajectory condition order on sparse panels.
- `source_labels_admit_aliases_humanization_and_collision_storage` covers raw,
  humanized and aliased refusal, refused output before source-iterator work,
  repeated raw identities, collision diagnostics, Unicode whitespace/combining
  text/case expansion and acronym parity.
- `arc_retention_admits_header_and_value_without_recopying_vector_storage`
  checks work/byte refusal, header/value budget boundaries and unchanged vector
  buffer identity. `regression_axis_channel_and_fit_work_is_admitted_before_preparation`
  checks channel-storage/fit-work refusal and numeric parity.

Limits and next-stage boundary: SOURCE here means **axis-owned** range resolution,
broadening, scale/policy projection, source/spec/category preparation, generated
tick/label materialization and axis fit work. Zero capture SOURCE counters do not
claim zero ordinary mark/statistical recomputation. Full-family KDE/fit/bar and
other interior payload extraction, canonical primitives and deferred-family/parent
adapters remain T5a6a; ordinary exact-root migration/closure remains T5a6b-T5a6d.
Matrix/Marginal packing, Canvas/facet tracks, full domain feedback and T5c
mark/density/annotation sizing retain their declared owners. No T5a6 substage or
parent T5a completion is implied.

Remaining `canvas_probe` uses are hue/category/legend probes, not shared-x range
replay. `PanelAxisPlan::draw_grid` replays retained lines; Matrix's `draw_frame`
is deferred cell geometry, not an automatic source selector. The resource audit
does not claim generic allocator interception, recoverable std Arc OOM, or
whole-family pre-font validation precedence. There is no known unresolved T5a5
acceptance item; these are unchanged scope limits, not deferred T5a5 blockers.

Dependencies: T5a4.

Objective: measure one immutable axis result under parent-assigned constraints,
using declared bounded local sources rather than root-wide alternatives.

Scope:

- Add immutable range, projection, tick, axis-title, axis, frame, and grid plans
  using the resolved T5a appearance and text plans.
- Validate a present exact-tick list as one indivisible `1..=10_000` alternative;
  reject a present empty list rather than selecting automatic generation.
  Preserve intentionally empty labels as unlabeled tick/grid positions with no
  text geometry.
- Implement private span/typography adaptive numeric sources with no fixed five-
  tick ceiling and one terminal midpoint source in the declared local order.
- Implement optional additive data intervals and Log10-decade intervals. Try the
  requested source first, try deterministic `1,2,3,5` decimal coarsening,
  retain complete nonempty interval vectors without subsequences, order the
  terminal midpoint after every interval vector, and apply source precedence
  locally under assigned span and measured fit. Declare the axis owner's finite
  source cap/order before generation; retain source provenance in frozen plans.
  Do not cross-product axes or sibling sources. Keep caller count/density
  controls absent.
- Implement exact formatting/uniqueness, projection, categorical one-label-per-
  slot behavior, Forest logarithmic sources, local fit checks, gutters, rotations,
  frame-side topology, tick reserve/draw synchronization, and grid ordering.
- Preserve automatic-range broadening's current private five-sample nice-range
  seed under its range owner; remove only the shared tick-count meaning and use
  no count target in generated-tick selection.
- Populate the standard-panel carrier with T5a typography, tick, axis,
  projection, and preferred `4:3` inputs while leaving mark/density/annotation
  inputs open for T5c.
- Admit every raw tick source, text block, local fit attempt, and plan payload
  through the T5a1r operation resource state before materialization, using checked
  bounded fallible growth. No exhaustive text-product block is required.
- Replace horizontal categorical helper duplicates with canonical raw/display
  labels and the common text/axis path.
- Add the Canvas shared-x semantic validator: configured continuous automatic
  members only, finite strict union, categorical or exact-limit rejection. T5b
  owns child-plan/track integration.
- Replace ordinary axis/title local stabilization with T5a1r's at-most-eight
  root feedback rounds; bounded local source trials are not independent
  stabilization loops. Use actual measured gutters, not a fixed 40% cutoff.
- Migrate composed-axis copying, source provenance, immutable cache inputs, and
  equality together with interval fields. Include every generated-label scalar in
  its role's font census before sealing; no late resolution during capture/replay.
  Preserve the range-only seed independently from intentional tick changes.
- Retain font-independent axis/exact-tick/interval and shared-x semantic inputs
  rather than validating by a second renderer pass. Invalid policies and domains
  owned by those components reject before font discovery. Whole-family data
  preparation and its error precedence belong to T5a6a's extraction steps, not
  this axis stage. Geometry/text-dependent fit remains in measurement.
  Admit raw source and cache payloads before allocation, not only at capture.

Exclusions:

- Legend clearance, Canvas/facet track measurement, mark ink/density/category
  pitch/annotations, complete Matrix/Marginal packing, full domain feedback,
  automatic roots, and public interval/alignment APIs. D2 admits no new label
  rotation or emergency-break recovery; an overwide exact category label fails
  intact, while existing role-specific rotation remains a renderer invariant.

Deletion targets:

- The shared `AUTOMATIC_TICK_TARGET` identity, five-only tick downsampling, and
  superseded ad hoc midpoint branches; retain the specified conditional terminal
  midpoint in the common local policy and the range-only seed under its private
  owner.
- Draw-time axis text measurement, local axis/title retry loops, duplicate
  spacing/line-height estimates, measured-state booleans, horizontal categorical
  helpers, and Canvas shared-x clone/replay.

Evidence:

- Wide panels with more than five generated ticks; small panels; source-span,
  `6ch`, `10_000/10_001`, representable-index boundaries, interval
  exact/coarsened/nonempty/no-subsequence/midpoint order, exact precedence,
  local source precedence and cap exhaustion, Log10, formatting, uniqueness, finite
  extremes, exact-list `0/1/10_000/10_001`, empty exact labels, category order,
  collision, projection endpoints, frame sides, Science duplicate removal,
  tick/grid reserve-draw parity, horizontal categorical fit, shared-x validation,
  accounting refusal, feedback stabilization, deterministic order, and measured
  gutter cases below/at/above the old 40% cutoff, including real fit misses.
- Cache-enabled/disabled equivalence for intervals, exact precedence, shared
  domains, Forest Log10, and changed title allocations; source provenance survives
  composition. Invalid nontext inputs win over unavailable-font errors with zero
  discovery calls for the axis/shared-x inputs owned here. Generated glyph census,
  retained axis preparation reuse, refusal
  before materialization, and no post-freeze tick/shape work are explicit gates.
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
  producers and automatic/exact-width roots; T6 public APIs/default switch.

Deletion of family-local retries below means duplicate/ad hoc stabilization and
source policies. It does not remove the common bounded axis source attempts or
the root-coordinated feedback protocol specified in `05_LAYOUT_SYSTEM.md`.

#### T5a6a: Ordinary Interior Carriers And Deferred-Family Adapters

**Status**: PENDING

Dependencies: T5a5.

Objective: establish closed native/deferred interior boundaries before migrating
any family route. This is an umbrella over the ordered extraction stages below,
not one adapter-scaffolding commit. Wrapping the combined family `render()` entry
point does not satisfy semantic preparation or retained-payload requirements.

Scope:

- Add the standard-coordinate and one-categorical-axis panel producer inputs and
  integrate the T5a standard-panel carrier into the parent-allocated Plot frame.
- Add exactly one typed `DeferredMarkAdapter` producing `DeferredMarkPlan` and
  one typed `DeferredAnnotationAdapter` producing `DeferredAnnotationPlan` from
  T5a3's `DeferredAnnotationTextPlan`. Before final arrangement they emit
  current hard-panel intrinsics, automatic-domain envelopes, reserve requests,
  obstacle/outward-ink extents, and frozen semantic/style/draw payloads. Their
  consequences feed the canonical range, axis, collision, and parent allocation
  routes; they cannot select a root alternative, mutate an assigned panel,
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
- Preserve the supported/rejected composition matrix independently from adapter
  representation: no new faceted-in-Canvas, Canvas-in-Canvas, or specialized
  nesting is admitted by this migration. Retain semantic rejection timing for
  unsupported shapes. Embedded Plot titles retain their local Plot semantics;
  T5b owns complete track integration for their local captions, not omission or
  promotion to the Canvas caption.
- Make every carrier and adapter use the T5a1r measured-result, accounting,
  admission, bounded feedback, frozen-plan, Null, and concrete lifecycle route.
  Intrinsics precede parent allocation; local alternatives require an owner,
  order, and cap, never sibling products or an adapter-specific search.

Deletion targets:

- Unclassified interior dispatch, adapter-specific root search, outer-root
  title reconstruction, and any carrier that can invoke `GeometryBackend` for
  measurement.

Evidence:

- Compile-time exhaustiveness over native and deferred route kinds; all 20
  ordinary legacy variants; five specialized/two composition adapters; effective
  significance text reuse; legacy hard-minimum/domain/reserve/obstacle parity;
  frozen deferred payloads; accounting/refusal; selected-panel and clip identity;
  no-layout-backend-call; and pre-backend failure tests.
- `just verify` passes.

#### T5a6a Execution Partition

T5a6a1, T5a6a2, all ordinary extraction steps T5a6a3a-f, and T5a6a4a are
complete with focused and primary integrated evidence below. Later extraction
steps are PENDING.
Each is one logical commit with focused evidence and `just verify`;
complete each before its successor. The umbrella has no commit.
The numbered extraction clusters retain the existing family behavior and feed
the later native axis/frame migrations; they must not duplicate those migrations.

| Step | Dependency | Deliverable and deletion boundary | Commit |
| --- | --- | --- | --- |
| T5a6a1 | T5a5 | Closed prepared-input, intrinsic/domain/reserve, constrained-result, and retained-payload contracts; exhaustive route classification; shared resource/feedback ownership. Existing routes remain explicitly classified, not falsely native | `feat(layout): define prepared interior contracts` |
| T5a6a2 | T5a6a1 | Sole canonical primitive carrier and shared count/emission from retained quantized operands; remove repeated raw conversion after freeze across native and deferred capture | `refactor(render): retain canonical primitive geometry` |
| T5a6a3a | T5a6a2 | Scatter, Line, Trajectory, LinePath semantic/payload extraction | `refactor(layout): prepare point and path payloads` |
| T5a6a3b | T5a6a3a | Histogram, Density, Qq, Regression semantic/payload extraction | `refactor(layout): prepare distribution reference payloads` |
| T5a6a3c | T5a6a3b | PointInterval, Coefficient, Forest, Dumbbell semantic/payload extraction | `refactor(layout): prepare estimate display payloads` |
| T5a6a3d | T5a6a3c | Bar, HorizontalBar, StackedBar, StackedHorizontalBar semantic/payload extraction | `refactor(layout): prepare bar payloads` |
| T5a6a3e | T5a6a3d | Boxplot, Violin, Raincloud semantic/payload extraction | `refactor(layout): prepare categorical distribution payloads` |
| T5a6a3f | T5a6a3e | Interaction semantic/payload extraction; all 20 ordinary adapters satisfy the same contract | `refactor(layout): prepare interaction payloads` |
| T5a6a4a | T5a6a3f | Heatmap typed deferred interior | `refactor(layout): prepare heatmap adapter` |
| T5a6a4b | T5a6a4a | Marginal typed deferred interior | `refactor(layout): prepare marginal adapter` |
| T5a6a4c | T5a6a4b | Matrix typed deferred interior and T5a4 outer title producer | `refactor(layout): prepare matrix adapter` |
| T5a6a4d | T5a6a4c | Beeswarm typed deferred interior | `refactor(layout): prepare beeswarm adapter` |
| T5a6a4e | T5a6a4d | Waterfall typed deferred interior | `refactor(layout): prepare waterfall adapter` |
| T5a6a5a | T5a6a4e | Facet parent adapter, shared-owner consequences, retained child plans, and T5a4 outer title producer | `refactor(layout): prepare facet adapter` |
| T5a6a5b | T5a6a5a | Canvas parent adapter, independent root/panel title inputs, T5a4 outer title producer; exhaustive adapter and lifecycle audit | `refactor(layout): prepare canvas adapter` |

T5a6a4a completion evidence:

- Heatmap retains admitted grid cells, category order, highlights, null-row
  annotations, color domains, numeric text identities, and color-label policy
  before fonts. Standalone, facet, and Canvas components prepare once; Canvas
  semantic preparation borrows configuration and census borrows retained inputs.
- Backend-free constrained measurement keeps the current Wrap/Elide packing,
  uniqueness policy, sparse/structural/missing cells, and mandatory colorbar.
  Typed cell, annotation, gradient, swatch, and text operands survive selection.
  GeometrySnapshot supplies mark/guide extents; the typed current heatmap minimum
  feeds the existing frame result alongside the named legacy panel floor.
  No T5e packing or public colorbar API is introduced.
- Exact-limit tests cover root semantics, cell and guide storage, and elision.
  Geometry parity, zero-source/projection/tick/shaping poisoned capture, root
  preparation counts, and unchanged rejected composition routes are covered.
- Focused strict offline all-target Clippy, the full library suite (878 passed,
  17 ignored), all 35 heatmap integrations, 57 composition integrations, six
  unchanged snapshots, and all ten unchanged release benchmark groups pass.
  Logs: `/tmp/opencode/t5a6a4a-unit.log`,
  `/tmp/opencode/t5a6a4a-integrations.log`, and
  `/tmp/opencode/t5a6a4a-benchmarks.log`.
- Twelve full-SVG fingerprints across Publication/Science, missing/merged rows,
  highlighted/value-annotated cells, Wrap/Elide, explicit domains, and category
  order match clean `b52c964`. The current audit was rebuilt after clearing the
  shared package artifacts. Logs: `/tmp/opencode/t5a6a4a-baseline-parity.log` and
  `/tmp/opencode/t5a6a4a-current-parity.log`.
- Primary review fixed Canvas category-label policy inheritance: preparation
  borrows the root policy without a deep clone, preserving raw category keys
  while applying aliases or raw display mode on both axes. Regression evidence
  covers exact admission, humanization collisions, inherited axis/color titles,
  and verbatim explicit titles.
- Final primary `just verify` in `/tmp/opencode/t5a6a4a-verify.log` passed:
  formatting, strict offline all-target Clippy, 879 library tests passed with
  17 ignored, and 445 integration tests passed, including 37 heatmap tests,
  57 composition tests, and all six still-unchanged snapshots. Four example
  targets ran zero tests; warning-denied rustdoc and offline package verification
  passed for 229 files (6.5 MiB; 1.4 MiB compressed). No tests failed.
- T5e packing remains deferred; T5a6a4b onward remain PENDING and no successor
  is started. No commit is made in this documentation-only completion update,
  as explicitly requested.

T5a6a3f completion evidence:

- Interaction retains admitted condition, collective, sparse-cell, interval, and
  configured-style semantics per root component. Axes consume retained domains;
  facets retain parent collective ordering, and Canvas uses borrowed configuration.
- Selected marks and common annotations capture without source reads or mark
  projection. The combined Interaction renderer is removed; categorical legacy
  classification remains until T5a6c4.
- Focused extraction, Interaction integration, and canonical lifecycle tests pass,
  as does strict offline all-target Clippy. Publication/Science default and styled
  full-SVG fingerprints match the original renderer at `ae5591f` at 800x600.
- Final primary verification in `/tmp/opencode/t5a6a3f-verify.log` passed:
  formatting, strict offline all-target Clippy, 873 library tests passed with
  17 ignored, 443 integration tests passed (including 23 Interaction tests),
  four example targets with zero tests, warning-denied rustdoc, and offline
  package verification of 228 files (6.5 MiB; 1.4 MiB compressed). No tests failed.
- Final primary review reported no actionable bugs. All twenty ordinary routes
  now use retained dispatch under the current legacy axis/frame policy; native
  migration remains pending in T5a6b/c, including Interaction in T5a6c4.

T5a6a1 declares the interfaces; each extraction step replaces its family's
combined preparation/drawing use with those interfaces and deletes superseded
helpers in that same step. Its selected draw payload is the source of reported
domain/reserve/obstacle consequences, never a discarded geometry-backend pass.
Constraint-dependent projection/payload construction may repeat when constraints
change; data/statistical preparation is retained. Final capture consumes the
selected payload without repeating preparation, projection, ticks, or shaping.
The later T5a6b/c clusters replace the classified legacy axis/frame policy with
the common native producer; they reuse, rather than rebuild, extracted payloads.

T5a6a1 interface integration evidence (based on clean `214db73`):

- `layout::interior` owns a closed route census and the actual combined dispatch:
  eight standard-family legacy variants, twelve one-categorical-axis legacy
  variants, five specialized legacy variants, and two independent parent
  composition classifications. Classification, reverse identity, and renderer
  selection are exhaustive matches, without wildcard dispatch, downcasts, or
  native labels. Trajectory's numeric/categorical source distinction remains in
  its retained axis inputs, not a falsely numeric-only route assumption.
- `PreparedInteriorInputs` replaces the former axis-only container, retaining
  T5a5's scientific ranges, provenance, exact/category sources, labels, and
  Matrix axis inputs. Preparation uses the caller's existing operation resources;
  attachment shares retained axis storage rather than storing a cloned Plot.
  This is axis preparation, not a claim that family statistics are extracted.
- `DeferredLegacyPanelMinimum` supplies the existing 80x60 parent/body floor.
  `DomainExpansion` and closed `InteriorReserveRequest` consequences submit to
  the existing `FeedbackRound`, preserving distinct source/target addresses,
  spatial quantization, request handles, admission, stale-effect replacement,
  and bounded root coordination. No adapter owns a fresh budget or retry loop.
- The current constrained result remains `PlotRenderResult` under
  `FitOutcome`/`PlanError`, with provisional failures owned by `FeedbackRound`.
  Capture now consumes `RetainedInteriorPayload::LegacyCombined`; that boundary
  supplies retained axes/layout, producer text, annotation text, panel text,
  specialized legends, and Matrix axes together. The independent annotation-text
  context setter and its separate capture wiring are removed.
- Facet/Canvas validation consumes the parent classifications at the existing
  semantic validation boundary. Invalid interior identities reject before fonts
  and family/backend work; existing option-error precedence remains intact.
  No supported nesting is added and no embedded title semantics change.

Deferred mark/annotation extraction interface obligations:

- `DeferredMarkAdapter` must produce one `DeferredMarkPlan` with typed
  `DeferredLegacyPanelMinimum` (while still needed), automatic-domain
  `DomainExpansion` envelopes, `InteriorReserveRequest` requests, and obstacle /
  outward-ink `PanelBox` extents derived from its selected mark payload. Its
  intrinsic result precedes parent allocation; constraint-dependent projection
  may repeat, but retained scientific/statistical preparation may not.
- `DeferredAnnotationAdapter` must consume the existing effective
  `DeferredAnnotationTextPlan` values (including significance text), producing one
  `DeferredAnnotationPlan` with the same typed domain/reserve/extent consequences
  and retained annotation placements. Neither adapter may choose a root, mutate
  the assigned panel, measure through a backend, or reconstruct during drawing.
- These are extraction obligations, not implemented full mark/annotation plans
  in T5a6a1. No unused Rust producer traits or empty future payload variants are
  introduced. The active closed legacy retained-result variant explicitly
  records that combined measurement/capture and geometry discovery still exist.
  T5a6a2 owns canonical primitives; T5a6a3a-f and T5a6a4a-e own actual family
  consequences/payload extraction; T5a6a5a-b own parent adapter/outer-title
  integration. Native composition tracks remain T5b work.

Focused verification: strict `just lint`; `cargo test --locked --offline --lib`
filtered to `interior`, `layout::kernel::`, `plots::shared_x::`,
`decorations::plan::`, `canvas::`, and `orchestrator::`. New tests cover the
20+5+2 census, invalid route rejection without fallback/font work, retained
source sharing, exact operation-limit admission/refusal, feedback source/target
identity, outward quantization, invalid consequences, and no budget refund after
a rejected trial. Existing lifecycle tests exercise every PlotKind, retained
text/axis capture, Null/concrete replay, and composition cache equivalence.
Primary `just verify` passed: 802 library tests (17 ignored), 442 integration
tests, strict Clippy, formatting, API docs, and package verification (221 files).
All six snapshots remain unchanged. Independent review found no runtime or
scientific regression. Log: `/tmp/opencode/t5a6a1-verify.log`.

T5a6a2 applies the `05` quantum spelling and checked bottom-up conversion once.
Cover points, paths, clips, surfaces, gradients, rounded rectangles, bracket and
text-placement transforms, stroke operands, and post-conversion degeneracy. Test
negative zero, negative coordinates, quantum half-ties, outward bounds,
nonintegral translations, collapsed lines/polygons, exact token counts, and
Null/SVG operand identity. Deterministic later glyph-outline extraction remains
format conversion under the existing font contract. Snapshot changes require
individual review; do not retain alternate raw and canonical frozen routes.

T5a6a2 completion evidence:

Independent review identified raw-text quantum, painted-stroke bounds, outward
rounding, and direct Null planned-placement validation gaps. All are corrected;
the focused test totals below describe the successive repair checks.

Review-gap correction evidence: raw text now shares CSS-quantum placement with
final capture; marker bounds include canonical circle outlines and triangle
miters using the serializer's miter limit; bracket bounds use canonical stroke
width. Recorded bounds round outward before clipping and collision consumption.
Direct planned Null text/bracket adapters use the same canonical constructors as
SVG, including absolute-origin and bottom-up overflow checks. Frozen replay and
capture resource admission are unchanged. Strict `just lint`, 103 render tests,
29 orchestrator backend tests, and the full all-target test run passed (816
library tests, 17 ignored; 442 integration tests). Existing snapshots passed
without further updates. These checks preceded primary integrated verification.

A subsequent blocking review found that rounding after floating absolute stroke
expansion still lost half-quanta at large coordinates. Stroke envelopes now use
wide integer half-quanta before outward rounding, signed-quantum edge checks,
and directed conversion to collision-space floats (with a second range check).
Polygon normals/miters use outward-rounded local intervals translated onto exact
integer vertices; axis normals and right-angle miters retain exact integer
half-quanta. No late record-time stroke rounding or frozen replay conversion is
used. Tests retain one-quantum strokes and clipped ink at both signs of 2^43 CSS
pixels and reject integer-edge and directed-float overflow near the signed
limits. Strict `just lint`, 106 render tests, 29 orchestrator backend tests, and
75 targeted annotation/interaction/layout/snapshot tests passed. These are the
checks for this correction, followed by the primary gate below. Independent
recheck found no remaining blocker in the conservative stroke-envelope fix.

- `render::primitives` retains admitted top-down quantum operands for every
  frozen primitive. Raw and planned text/bracket producers converge on one
  canonical payload; no frozen raw text/bracket variants remain. Text retains
  the original source and shaped plan alongside absolute line/run/glyph origins,
  logical run order, rotation center, and canonical angle/sine/cosine. Angles
  retain their separate tenth-degree unit, with zero signs normalized.
- Final capture performs checked placement and bottom-up conversion. Shared SVG
  count/emission and final Null consume those operands without conversion,
  sorting, layout, or shaping. `PlannedLineOutline` shares frozen geometry;
  later deterministic extraction uses retained font identities and absolute
  glyph origins, not new placement or a changed font source/policy.
- Capture admits each line/run/order/glyph vector and shared text allocation
  against the existing root resources before constructing its storage. Final
  SVG and Null storage admission and exact count/write agreement remain intact;
  obsolete replay run-order scratch is removed.
- Tests cover all primitive replay conversions, text anchors and origins,
  multiline/bidi source preservation, negative zero and small angles,
  nonintegral/half-tie placement, absolute glyph and Y-inversion overflow,
  per-allocation refusal, unchanged final storage addresses, and deterministic
  outlined byte counts. Strict `just lint` passed; 811 library tests passed
  (17 ignored), and all 442 integration tests passed.
- Individually reviewed the six existing snapshot changes: Beeswarm and
  HorizontalBar clip operands; Marginal's three clip operands; Matrix's four
  clip operands; and the odd-sized title/scatter snapshots' clip, surface,
  mark, axis, and stroke quantum spellings. Text/source structure is unchanged
  in those snapshots. Text/bracket freezing required no additional snapshot
  updates.
- Primary `just verify` passed: 819 library tests (17 ignored), 442 integration
  tests including six snapshots, strict Clippy, formatting, API docs, and package
  verification (222 files). All ten ignored release benchmark groups also pass.
  Logs: `/tmp/opencode/t5a6a2-verify.log` and
  `/tmp/opencode/t5a6a2-benchmarks.log`.

T5a6a3a completion evidence:

- `PreparedInteriorInputs` retains Scatter and Line/Trajectory/LinePath scientific
  semantics alongside the existing axes. Standalone, facet, and shared-Canvas
  components prepare once before font discovery; Canvas shared ranges consume
  those same inputs, and attachment shares their storage. Categorical Trajectory
  keeps complete normalized condition order and panel-local source diagnostics.
- Line series, conditional ribbons, supplemental points and labels, neutral
  trajectories, and source-ordered neutral/hue paths produce selected payloads
  during backend-free measurement. Series sorting, repeated-x path order, marker
  policy, palette slots, sparse collective omission, clipping, and legend glyph
  policy remain intact. No line-family combined renderer route remains.
- The existing `DeferredMarkPlan`, `DeferredAnnotationPlan`, guide snapshots,
  geometry snapshots, and selected-interior boundary serve all four families.
  Obstacles, clearance, reserves, and range consequences consume selected payload
  geometry. Axis/frame classification remains legacy until T5a6b1; parent chrome
  observations and the later parent/outer-title migrations are not claimed here.
- Semantic/projected vectors, strings, shared storage, and collective legend
  identities use the caller's admission gate before materialization. Tests cover
  exact-limit acceptance/refusal, payload/legacy ink parity, invalid-source
  precedence without fonts, one preparation per component, and selected capture
  against unusable sources. Actual source-column reads, coordinate projections,
  semantic preparation, and shaping are zero during capture; Null/SVG replay
  leaves the family-stage counts unchanged.
- Strict `just lint` and focused tests pass: 6 line extraction, 8 Scatter, 11
  shared-axis, 23 decoration-plan, 174 orchestrator (10 ignored benchmarks), 12
  Canvas, 32 geometry, 3 interior-contract, 41 line integration, 57 composition,
  and all 6 unchanged visual snapshots. These preceded the primary full gate.
- Review corrections validate every facet's guides against its final reconciled
  ranges before font discovery, including coordinates valid only under a fixed
  union. Scatter and Line now share admitted point-legend identity construction,
  accounting for both dimension and value strings on every projection candidate.
  Regression tests cover unavailable-font precedence, all four facet scale
  policies, non-finite/out-of-range guides, long caller-owned dimension names,
  exact-limit refusal, and cumulative candidate charges. Strict `just lint` and
  focused checks pass: 9 Scatter, 12 shared-axis, 6 line extraction, 41 line
  integration, and 57 composition tests.
- Primary `just verify` passed after the review corrections: 835 library tests
  (17 ignored), 442 integration tests, strict Clippy, formatting, API docs, and
  package verification (223 files). All six snapshots remain unchanged.
  Log: `/tmp/opencode/t5a6a3a-verify.log`.

T5a6a3b completion evidence:

- Histogram retains bins, internal/supplied KDE curves, and exact rug endpoints;
  Density retains collective KDE curves; Qq retains sorted quantiles and reference
  geometry; Regression retains observed points, fit endpoints, and confidence-band
  samples. Scientific axes consume these results rather than preparing statistics
  again. Standalone, facet, and shared-Canvas preparation precedes font discovery.
- All four families now use selected mark/annotation capture. Their combined
  routes and renderers are removed. Geometry snapshots consume the same projected
  primitives as drawing; no geometry backend is used for their layout. Density
  keeps parent collective order and palette slots without reading columns during
  measurement. Qq validates reference clipping against final scientific ranges.
- Source vectors, statistical scratch, supplied geometry copies, projected curves,
  bands, rugs, legends, and shared allocations use operation admission. Tests cover
  exact limits, cumulative candidate refusal, all four facet scale policies,
  shared Canvas preparation counts, selected geometry/draw extent parity, invalid
  statistics and exact ticks before unavailable fonts, and poisoned-source capture.
  Capture repeats no source reads, projection, statistics, axis work, or shaping;
  Null/SVG replay leaves the family-stage counts unchanged.
- Strict offline all-target Clippy passes. Focused checks pass: 8 interior tests,
  11 Density tests, 67 distribution integration tests, 57 composition tests,
  38 estimate integration tests, and all 6 unchanged snapshots. Earlier checks in
  this implementation also passed 154 plot unit tests and 174 orchestrator tests
  (10 ignored benchmarks).
- Review found a source-owning Plot clone in Canvas reference preparation. The
  corrected path borrows configuration and passes scope/shared limits/label policy
  separately. Actual Canvas tests cover supplied Histogram geometry and KDE,
  work/storage refusal, exact-budget success, and zero font discovery.
- Primary `just verify` passed after correction: 842 library tests (17 ignored),
  442 integration tests, strict Clippy, formatting, API docs, and package
  verification (224 files). All six snapshots are unchanged.
  Log: `/tmp/opencode/t5a6a3b-verify.log`.

T5a6a3c is complete. PointInterval, Coefficient,
Forest, and Dumbbell retain admitted scientific inputs at the root and consume
selected mark/annotation payloads in standalone, facet, and shared Canvas routes.
Axis/frame classification remains categorical legacy; native migration belongs
to T5a6c1.

PointInterval/Coefficient/Forest implementation evidence:

- Retained rows preserve categorical order, unavailable status, original source
  diagnostics, interval bounds, reference values, and adjacent numeric/significance
  label text. Forest transforms scientific tuples once and retains raw exact-tick
  policy separately; coordinated ranges are never log-transformed a second time.
- Fixed-x unavailable facets inherit the union of already prepared sibling
  domains without pooling or rereading tuples. Facet significance uses borrowed
  panel ownership selectors, so an estimated sibling's mark is never assigned to
  an unavailable row. Final reconciled guides validate before font discovery.
- Measurement produces immutable row-ordered intervals, fixed-pixel caps, points,
  status labels, adjacent labels, and reference geometry with GeometrySnapshot,
  not GeometryBackend. Capture consumes selected payloads. Obsolete combined
  renderers, fallback facet tuple reads, and attached-label reconstruction paths
  are removed. Canvas borrows source configuration throughout preparation.
- Row vectors, source label formatting, typography strings, selected geometry,
  annotation records, and shared storage use operation admission before allocation.
  Eight cross-family extraction tests cover exact limits and cumulative refusal,
  mixed shared Canvas and all facet scale policies, source-row identity, font-error
  precedence, facet-owned annotations, Forest exact ranges/ticks, geometry parity,
  and poisoned capture. Capture performs no source reads, semantic preparation,
  projection, axis generation, or shaping; Null/SVG replay preserves stage counts.
- Strict offline all-target Clippy and focused checks pass: 13 interval-estimate
  unit tests (including eight extraction tests), 174 orchestrator tests (10 ignored
  benchmarks), eight Canvas root tests, 38 estimate integration tests, 57 composition
  tests, and all six unchanged snapshots. These precede the primary full gate.

Dumbbell-focused evidence: four extraction tests cover exact semantic/payload
budget refusal, cumulative candidate charges, all facet scale policies, mixed
shared Canvas counts, final reconciled guides before fonts, geometry/draw extent
parity, and poisoned-source selected capture. Capture has zero column reads,
coordinate projections, or shaping; frozen Null/SVG replay leaves stage counts
unchanged. The 42 categorical, 57 composition, 38 estimate, 37 lifecycle, 13
Canvas, and 12 shared-axis tests pass, as do all six unchanged snapshots. These
are focused checks, not the primary `just verify` gate.

Independent review found no actionable scientific, admission, or capture
regression. Primary `just verify` passed: 854 library tests (17 ignored), 442
integration tests, strict Clippy, formatting, API docs, and package verification
(223 files). All six snapshots remain unchanged.
Log: `/tmp/opencode/t5a6a3c-verify.log`.

T5a6a3d completion evidence (based on clean `4c67f93`):

- Bar, HorizontalBar, StackedBar, and StackedHorizontalBar prepare scientific
  cells once per component before fonts. Axes consume those retained categories,
  intervals, supplemental point extents, and signed stacks instead of recomputing
  statistics. The superseded interval-skipping axis preparation is removed.
- Shared Canvas preparation borrows scope, label, and reconciled-range inputs.
  Facets borrow ownership selectors and global series/hue order; deterministic
  bootstrap identity remains facet-local. Absent series keep their dodge slots
  without acquiring marks. Parent-suppressed horizontal category labels retain
  raw keys and tick positions while omitting only their measured display text.
- Measurement retains orientation-specific polygons, interval stems/caps, points,
  zero lines, and attached text. GeometrySnapshot records their selected extents
  and pairwise bracket consequences without GeometryBackend. Final capture uses
  selected marks/annotations, never the removed combined renderers. The four
  categorical legacy variants remain unchanged until T5a6c2.
- Operation admission precedes source/category/point identity storage, retained
  panel allocation, and payload vectors. Seven extraction tests cover exact
  semantic budgets, cumulative refusal, standalone/facet/mixed shared-Canvas
  stage counts, global colors and absence, exact ticks, facet-owned annotations,
  bit-exact bootstrap intervals, invalid inputs before fonts, extent/draw parity,
  and poisoned-source capture. Selected capture repeats no source reads,
  preparation, projection, axis work, or shaping; Null/SVG replay preserves counts.
- Focused checks pass: 46 bar-filtered library tests, 16 shared-axis tests,
  44 lifecycle tests, 26 significance tests (one ignored benchmark), 42 categorical,
  27 annotation, and 57 composition integration tests. All six snapshots pass
  with `INSTA_UPDATE=no` and remain unchanged. Strict offline all-target Clippy
  and `git diff --check` pass.
- Review corrected an unadmitted legend dimension string by sharing the admitted
  identity constructor. Long caller dimensions have exact work/storage and
  cumulative-candidate refusal evidence. Two clearance unit fixtures now attach
  prepared bar inputs without altering their assertions or adding a fallback.
- Primary `just verify` passed: 861 library tests (17 ignored), 442 integration
  tests, strict Clippy, formatting, API docs, and package verification (224 files).
  All six snapshots remain unchanged. Log: `/tmp/opencode/t5a6a3d-verify.log`.

T5a6a3e completion evidence (based on clean `7534bc9`):

**Status**: COMPLETE

- Boxplot, Violin, and Raincloud retain component-scoped samples, quartiles,
  family-specific whiskers/outliers, KDE grid/densities, or admitted supplied
  geometry before fonts. Axis preparation consumes retained category order and
  ranges; facets retain global series/hue slots and reconcile their ranges.
  Canvas prepares each component once. Categorical shared-x rejection is unchanged.
- Backend-free measurement creates ordered polygon/line/scatter payloads and
  GeometrySnapshot consequences. Parent axis/frame policy remains categorical
  legacy until T5a6c3. The combined renderers and their fit/backend clip helper
  are removed. No native-axis or final T5c mark policy is introduced.
- Selected common annotations retain guide/panel text geometry and pairwise
  bracket styles in DeferredAnnotationPlan. Standalone, facet, and Canvas capture
  use those styles and the retained deferred text without consulting frontend
  significance appearance. Bracket ink is recorded once in annotation geometry,
  not duplicated in mark geometry; existing draw order is preserved.
- Supplied Histogram/Raincloud source geometry uses shared immutable ownership,
  eliminating nested vector/string clones when parent composition clones a Plot.
  Statistical sorts use the existing admitted in-place source sorter, and KDE
  uses the existing operation-bearing statistical helper. Retained and projected
  vectors and legend identity strings use the existing source admission gates.
- Six extraction tests cover exact prefont semantic budgets, shared attachment,
  source-geometry clone identity, invalid statistical inputs before discovery,
  standalone/all facet policies/Canvas stage counts, mark and annotation
  snapshot parity, poisoned-source capture, and Null/SVG replay. Capture repeats
  no source-column reads, statistics, projection, tick/axis work, or shaping.
- Strict offline all-target Clippy passes. Focused integration checks pass:
  42 categorical, 67 distribution, 27 annotation, and six unchanged visual
  snapshots. Full library test output is persisted in
  `/tmp/opencode/t5a6a3e-unit.log` (867 passed, 17 ignored).
- Review P2 is resolved: supplied hue indices are retained during semantic
  preparation, with zero repeated hue lookups during measurement or capture.
- Primary integrated `just verify` passed: 869 library tests (17 ignored),
  442 integration tests, strict offline all-target Clippy, formatting, API docs,
  and package verification (227 files). All six snapshots pass and remain
  unchanged. Log: `/tmp/opencode/t5a6a3e-verify.log`.

Every extraction step proves font-independent error precedence with zero font
discovery, admission before payload/cache allocation, actual consequence parity,
and separate preparation/measurement/payload/capture/replay counts. Resources
spent on rejected alternatives remain spent. Every feedback producer declares
consumed/emitted inputs, provisional versus terminal failure, monotonic quantities,
and replacement of stale requests. Cache keys consume the same immutable inputs
as measurement; feedback-producing hits replay their retained effects.

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
- Reuse T5a6a's prepared input and selected payload. Instrument semantic preparation,
  constrained measurement, payload construction, capture, Null, and concrete
  replay separately; capture/replay must not invoke the combined family renderer.
  Test cache-enabled/disabled identity, changed shared domains/constraints,
  rejected ancestor trials, and resource refusal before materialization.

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
- Apply the same preparation/payload/capture counters, no-recomputation gates,
  cache/feedback differential tests, and pre-materialization admission gates as
  T5a6b. Reuse T5a6a extraction rather than a second family preparation route.

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
  intrinsic/allocation/measurement, bounded admission, arrangement, frozen plan, and
  draw; remove every superseded T5a path.
- Split-phase ledger keys close only their T5a portion here. In particular,
  caption alignment and standalone/outer-root replay close in T5a; embedded Plot
  caption track allocation and exactly-once rendering remain a named T5b
  obligation carried by the composition adapter. Do not claim that portion done.
- Prove the five specialized interior adapters, two composition adapters, and
  mark/annotation deferred adapters plus their named legacy-panel-minimum
  consequence are the only remaining nonnative boundaries.
- Publish one closure matrix over all 20 native kinds and each remaining adapter:
  semantic validation/prepared input; intrinsic/domain/reserve supplier; consumed
  constraints and emitted feedback; cache identity/replay; retained text/canonical
  primitives; admission boundary; selected-payload capture count; no post-freeze
  preparation/layout; Null/concrete identity; exact-size fit/failure and snapshots.
  Count all measurement-backend paths, not only final Null replay. No adapter may
  hide an extra layout-backend pass behind the final lifecycle counter.
- Exercise shrinking axis gutters, monotonic annotation-domain growth, rollback
  after rejected containing frames, later-round cached feedback replay, cycles,
  and eighth-round success/exhaustion with actual native ordinary axis/title/root
  producers and the named retained deferred annotation/composition consequences.
  T5b replaces guide/containing-frame and composition adapters with native
  evidence; T5c replaces deferred annotation consequences. Merely
  observing changed geometry does not request a new round; owners emit the
  required feedback explicitly before stable-root early termination.
- Preserve explicit `640x480` Plot and `960x640` Canvas behavior while leaving
  public automatic and exact-width sizing unexposed until their atomic T6 switch.

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
- `just verify` passes; all reopened repair stages and T5a6a-T5a6d must be
  complete before T5a6 and parent T5a are marked complete.

Commit: `refactor(layout): close ordinary exact-root planning`.

## Phase T5b: Legends, Facets, And Canvas Composition Chrome

**Status**: PENDING

Dependencies: T5a.

Give composed and explanatory chrome coherent theme ownership.

Scope:

- Prepare legend semantic entries and intrinsic requirements independently from
  allocation. The legend owner chooses one deterministic local flow at assigned
  width using final typography and exact glyphs, with fixed entry/column widths
  before greedy flow as specified in `05_LAYOUT_SYSTEM.md`; return one measured
  result or typed fit miss. The owning frame validates the affected body/axes and
  guide together before accepting placement. Use only edge/compact-flow recovery
  approved through T5a0s/D1; do not invent additional row/column alternatives.
  Do not send alternative sets upward or combine sibling legend choices.
- Replace equal-cell subtraction with measured row and column tracks that
  preserve shared data-panel alignment separately from unequal decoration
  gutters.
- Centralize legend container, title, label, key, outline, opacity, padding, and
  spacing styles while keeping visibility and placement in legend policy.
- Apply role-based typography and surfaces to facet strips, panel headers, shared
  headers, row titles, text panels, and shared legends.
- Integrate embedded Plot titles/subtitles through the T5a4 producer, preserving
  Plot content and the resolved title/subtitle policies across their local
  post-inset allocation, not the data-panel span. Canvas-owned heading alignment
  remains independent. Render each panel Plot caption exactly once beneath that
  Plot, with its resolved theme and caption alignment; the Canvas caption remains
  separate. Measure caption space in tracks and reject unfit exact allocations
  rather than omit, merge, or promote captions. Preserve existing row-title
  conflict rules unless separately revised.
- TextPanel title/body use the approved top-left placement after chrome insets;
  extra exact space expands chrome, not text centering. Distinct shared headers,
  row titles, and facet strips do not inherit panel Plot alignment instructions.
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
- Make Canvas `TextPanel` contribute intrinsics to parent track allocation, then
  measure at its assigned width before arrangement. Keep Plot panel-text
  annotations in the overlay path.
- Remeasure constraint-dependent children when track constraints change; freeze
  final headers, row titles, shared legends, text lines, cells, and child plans
  before Canvas rendering.
- Use deterministic parent-owned track allocation and the sole at-most-eight
  root feedback rounds. Admit topology, local choices, text, and track storage
  before materialization; do not add grid/legend searches or local retry loops.
- Preserve declared sparse coordinates, spans, groups, and facet topology. Do
  not compact empty rows or columns implicitly.
- Delete the two T5a6a parent-composition adapters as their native routes land;
  census the remaining specialized interior adapters separately. Remove native
  legend/facet/Canvas stabilization loops here, not at T5e. T5e closes remaining
  specialized and cross-producer behavior on this same feedback driver.

Acceptance:

- Legend and composition measurement consume the exact final styles.
- Shared and per-plot legends use one component contract while retaining exact
  mark glyphs.
- Mixed explicit Plot themes have deterministic documented behavior.
- Mixed Nature/CHI panels obey Canvas panel title/subtitle instructions unless
  individually overridden. Test subtitle fallback to final title, explicit
  subtitle separation, independent Canvas headings, simultaneous Canvas/panel
  captions, narrow wrapping, asymmetric gutters, exact-height refusal, and
  cache-enabled/disabled equivalence. A roomy TextPanel fixture proves top-left
  placement. Unsupported nesting is rejected independently from native/adapted
  representation; no new composition shapes are implicitly admitted.
- D1 examples: automatic right-guide fit that starves the body recovers through a
  supported top/bottom edge or compact reflow with all entries intact; explicit
  right remains right or fails. Nested guides share bounded trial/work budgets,
  and resource refusal is distinct from exhausted supported fit trials.
- Multi-owner traces on supported composition shapes establish initialization and
  reset of current guide choices, ancestor constraint changes, provisional child
  misses, next-round recovery, fixed sibling decisions without backtracking, and
  planning-limit versus fit-error precedence. Cache hits replay feedback; measured
  cache misses are not described as reuse. No unsupported nesting is needed to
  manufacture the trace.
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
  consequences while retaining the same T5a1r parent-allocation, measured-result,
  range, collision, bounded feedback, admission, and frozen-plan route.
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
- Admit mark/annotation work and storage before materialization. Any local
  placement choices have declared owners, orders, and caps; use the same bounded
  root feedback for domain/reserve consequences, not a global quality score.
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
- Supply guide intrinsics before parent allocation and one measured guide at
  assigned constraints afterward. Admit generated labels, mapping/guide storage,
  and local source trials before work; never build cross-products of guide and
  panel alternatives or add a scale-specific root retry loop.
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
- Complete specialized intrinsic requirements, deterministic parent allocation,
  and local measured layouts. Semantic completeness and hard scientific geometry
  remain requirements; aspect, density, and spacing preferences belong to their
  declared local owners, not global quality suppliers or mandatory dense ranks.
- Retire remaining global-quality/search ledger roles and implementation state;
  no root Cartesian alternatives, sibling products, or strategy switch remains.
  Every local alternative has a declared owner, deterministic order, and cap.
- Complete all-producer cache identities and bounded feedback for wrapping,
  ticks, legend flow, grid tracks, shared alignment, domains, and overlays on
  T5a1r's sole at-most-eight-round driver. Detect cycles and exhaustion; no
  component adds an unbounded local retry loop.
- Resolve one private automatic root size, exact-width/automatic-height size, or exact root
  size before backend creation. Convert the finite logical result once through
  checked upward CSS-pixel rounding and store it in `PlotPlan` or `CanvasPlan`.
- Derive automatic preferred width from the existing preferred-width metric and
  measure height there. Exact-width mode assigns precisely the requested width
  and measures height; exact pair fixes both. No `max_width` mode or width search.
  Keep shared logical-work/font caps, representability, and fallible SVG growth,
  but remove the automatic-only 64 MiB canonical SVG gate and all hypothetical
  PNG viability checks. Every PNG source/target, backing-store, parser-byte, and
  requested-option limit is terminal-local in every mode. PNG refusal never
  rejects valid SVG, mutates the scene, or triggers another layout.
- Complete the operation resource census over every T5b-T5e producer, preserving
  existing caps with checked topology/work arithmetic and first-party admission
  before materialization. Fallible incremental growth is allowed; do not require
  universal exact allocations or claim third-party OOM recovery. No clamping,
  font shrink, content omission, hidden fixed root, or format-specific mutation.
- Remove remaining final-render text layout, tick selection, legend construction,
  spacing derivation, track sizing, and domain mutation.
- Exercise automatic and exact-width/automatic-height behavior through private focused tests,
  but retain the current public default and getters until T6 performs the one
  public switch.

Acceptance:

- Every admitted Plot family and Canvas child supplies separate intrinsics and
  returns one immutable measured result or typed fit miss under parent-assigned
  constraints. Unsupported composition shapes reject before materialization;
  one complete arranged plan tree is frozen before final Null validation.
- Automatic and exact-width/automatic-height private plans are finite, deterministic,
  viable under the approved local-layout rules, and complete in semantic
  content.
- Exact dimensions remain exact and fail rather than clip when complete content
  cannot fit.
- Examples: a private exact 300-pixel root measures caption-dependent height;
  an exact pair that cannot fit fails after bounded frame recovery. Valid SVG
  above PNG limits or 64 MiB remains possible under shared safety gates, with
  later PNG refusal and no mutation. Sparse coordinates and captions retain D6
  owner-local alignment; long unbroken labels retain D2 strict failure.
- Null and concrete rendering consume the same immutable root and child plans.
- Specialized, legend, and track regressions prove local ownership and bounded
  feedback without sibling products; scientific caption and long legal-break
  benchmarks retain full fixtures and report measured scaling evidence.
- Focused ordinary, categorical, specialized, facet, Canvas, convergence,
  resource, output, and snapshot tests pass.
- `just verify` passes.

## Phase T6: Public Immutable Theme, Scale, Tick-Interval, And Sizing API

**Status**: PENDING

Dependencies: T5e.

ENTRY gate, after T5e closure and before T6 implementation: review DF1 separate
legend sections, DF2 caller numeric formatting, DF5 broader text layout controls
(including emergency breaks/rotation/alignment), and DF7 sparse compaction using
the evidence requirements in `99_DEFERRED_FEATURES.md`. For each ID, record a
user-approved admission with a new approved phase, rejection with reason, or
re-deferral to a new explicit milestone checkpoint. Review is not an implementation
promise; none of these reviews automatically expands T6.

Expose the complete foundations only after every admitted consumer uses the
private resolved model.

Scope:

- Replace the closed public theme selector with the approved immutable complete
  `Theme` API and built-in constructors or presets.
- Expose only validated component theme, color, typography, and scale types
  required by the full-foundations milestone.
- Expose `TitleAlignment::{Start,Center,End}` for concrete theme defaults
  `CompositionTheme::root_title_alignment` and `root_caption_alignment`, and
  optional `root_subtitle_alignment`. Built-ins return Center, Start, and None
  respectively. Subtitle fallback follows the final effective title, not a copied
  preset value. Start/End are physical directions over the local content span.
- Expose consuming `Plot::title_alignment(TitleAlignment)` and
  `Plot::subtitle_alignment(TitleAlignment)`, and
  `Canvas::panel_title_alignment(TitleAlignment)` and
  `Canvas::panel_subtitle_alignment(TitleAlignment)`. Each sets its own optional
  instruction; repeated calls replace it independently of theme-setter order.
  Title resolves Plot instruction > Canvas panel instruction > resolved Plot
  theme. Subtitle resolves Plot instruction > Canvas panel instruction > optional
  resolved-theme subtitle > final effective title. None is absence, not a force-
  follow mode. Complete themes otherwise retain their existing replacement rule.
  Canvas's own heading uses its theme independently; panel settings affect no
  captions, TextPanels, strips, row titles, or other shared headers. Use `panel`
  in the public Canvas API; `child` remains generic internal tree terminology.
  No general per-role alignment, backend anchors, or sparse theme merging is added.
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
- Add consuming `Plot<K>::width(u32) -> Self` and `Canvas::width(u32) -> Self`
  for exact width with automatic height; retain exact `size(width, height)`.
  Every sizing setter completely replaces the request: width after size clears
  height, size after width fixes both, and repeated calls replace prior values.
  Validate positive dimensions at preflight, matching current timing. Remove the
  `max_width` target and expose no public Flutter bounds or constraint protocol.
- Connect only the repaired parent-constraint route: preferred width then measured
  height, exact width then measured height, or exact pair, producing one layout.
  Do not expose a search strategy, global objective, or alternate route.
- Replace pre-render `Plot::width()` with the consuming width builder and remove
  `Plot::height()`; migrate all repository callers atomically, without aliases. Add
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
  Plot replacement, palette, exact local style, automatic sizing, exact-width
  wrapping, exact sizing, and rendered-dimension examples.

Acceptance:

- Callers can construct one complete custom theme covering the admitted full
  foundations without editing private renderer state.
- A caller can independently choose title, subtitle, and caption alignment.
  Every built-in centers title, leaves subtitle unspecified to follow the final
  title, and Start-aligns captions. Mixed explicit presets obey Canvas panel
  policies; Plot instructions win individually. Test all precedence combinations,
  subtitle-only content, theme/setter-order independence, and replacement.
  Explicit Canvas panel subtitle Center remains Center when a Plot title is End;
  Plot subtitle End overrides it. Custom-theme subtitle applies only below both
  instruction scopes. End caption changes neither heading role. Canvas root
  headings and captions remain independent from panel defaults and captions.
- A caller can request an additive linear or Log10-decade preferred tick interval
  independently on x and y; focused evidence proves exact-source preference,
  deterministic coarsening, terminal midpoint order, scale mismatch, exact-tick
  precedence, setter replacement, and absent-interval automatic behavior.
- Built-in Publication, Nature, Science, and CHI behavior remains available
  through the single current API.
- The API has no ambiguous global default setter, generic property map, detached
  mutable palette, or alternate compatibility identity.
- Public constructors validate structural theme/scale invariants; positive sizing
  remains preflight validation, matching current timing.
- Automatic and exact-width output report their final dimensions only through
  `Output`; exact sizing remains exact and has no nominal pre-render getters.
- All sizing modes keep PNG limits local to PNG publication, including actual
  source/target dimensions, backing storage, outlined parser bytes, and options.
  No hypothetical PNG or automatic-only 64 MiB canonical SVG gate remains;
  shared logical-work/font caps, representability, and fallible SVG growth remain.
- Examples on Plot and Canvas: `.size(600,400).width(300)` reports exactly 300
  wide and measured height; `.width(300).size(600,400)` reports 600 by 400;
  repeated width calls replace. Zero fails at preflight. A valid large SVG may
  later fail PNG without changing the frozen scene or its reported dimensions.
- Compile-pass and compile-fail evidence enforces the intended visibility and
  ownership boundaries.
- All canonical specifications describe the implemented replacement API before
  the phase is complete.
- CLOSURE gate, before T7: review DF3 data-mapped alpha and DF4 broader count/
  density tick controls against implemented T6 evidence, following each item's
  `99` evidence requirements. Record for each ID a user-approved new-phase
  admission, reasoned rejection, or re-deferral to a new explicit milestone.
  Review does not promise implementation or retroactively expand T6.
- `just verify` passes.

## Phase T7: Contract Evidence And Final Audit

**Status**: PENDING

Dependencies: T6, including its DF3/DF4 closure gate.

Close the phase with exact behavioral evidence and no residual old model.

Scope:

- Add precedence tests for library default, Canvas inheritance, explicit Plot
  replacement, plot scale configuration, and exact element styles.
- Prove one resolved theme is used by preflight and final rendering.
- Cover shared legends, facets, text panels, mixed explicit Plot themes, palette
  capacity, missing mappings, canonical font candidate ordering/deduplication/
  provenance, shared continuous-guide compatibility/domain merging, contrast
  behavior, SVG, and PNG.
- Cover automatic, exact-width/automatic-height, and exact-pair dimensions across
  ordinary, specialized, facet, sparse Canvas, shared-guide, SVG, and PNG routes.
- Cover independent caption/title/subtitle defaults, final-title subtitle fallback,
  Plot/Canvas panel instruction precedence across mixed presets, and independent
  Canvas headings, plus automatic/additive/Log10-decade/exact tick-policy precedence
  across standalone, facet, Canvas-child, SVG, and PNG routes.
- Retain reviewed deterministic snapshots for every built-in preset and selected
  custom themes.
- Search for retired theme APIs, repeated hidden Publication defaults, dormant
  theme fields, unclassified hardcoded style values, and duplicate scale logic.
- Search for fixed root defaults, universal panel minima, root-scaled spacing,
  draw-time layout decisions, silent overflow, and alternate layout routes.
- Prove retirement of universal candidate APIs, root Cartesian search, mandatory
  dense ranks, residual exhaustive wrapping, and runtime strategy selectors.
  Audit titles, axis local sources, legends, tracks, and specialized layouts for
  separate intrinsics, one measured result, declared local owner/order/cap, and
  at most eight root feedback rounds without unbounded local retries.
- Recheck every T5a3 repair disposition, Unicode 16 dependency/data provenance and
  full conformance fixtures, base-plus-mark unit preference and allowed per-scalar
  first-coverage fallback, strict visible-miss failure, preserve-text viewer limits,
  exact retained outline/PNG geometry, actual parser bytes/PNG options, and
  first-party admission before census copies or final Null validation.
- Retain benchmark evidence for `"A ".repeat(144)`, an approximately
  200-word scientific caption in one paragraph, the three-paragraph caption, and
  increasing long legal-break inputs. Report measured
  performance and resource scaling, not universal unmeasured guarantees.
- Reconcile every key in the T5a0s-reconciled final ledger against implemented
  owners, current public names, focused evidence, and deletion of every
  superseded source. The historical 1836-record pre-T2h count and completed
  1860-record T2 count and historical T3j/T5a0 1884-key total are provenance,
  not unreviewed revised-target acceptance thresholds.
- Check expected active `04` totals explicitly: 23 target-only keys, 1575 overlay
  rows (1552 existing plus 23 target-only), and 1883 materialized keys, including
  the independent caption property. Coordinate any reviewed `04` difference;
  historical 22/1574/1882 and 24/1576/1884 totals remain labeled provenance.
- Audit all specifications, README, guides, examples, and rustdoc for residual
  drift from the already-current implemented contract.
- Cross-route examples prove right-guide starvation recovery versus fixed explicit
  placement, intact overwide-identifier failure, width-after-size height clearing,
  SVG success followed by PNG refusal, coherent base/mark coverage with permitted
  scalar fallback, and independent Start captions with Center titles/subtitles.

Acceptance:

- Phases T1 through T7 and all admitted T5 subphases are complete with commit and
  verification evidence.
- Every visual property has one implemented owner matching the approved
  T5a0s-reconciled 1883-key inventory, including its 23 target-only keys and
  1575-row overlay, with every source addition, split, retirement, consumer
  relationship, and explained exclusion re-audited.
- No mutable global theme, repeated hidden Publication default outside its
  owning boundary, late theme lookup, compatibility alias, or unclassified
  renderer style constant remains.
- Snapshot changes are intentional, reviewed, and committed with their owning
  behavior change.
- CLOSURE gate, before successor planning: review DF6 broader root bounds/
  maximum width, DF8 caller-selected objectives, and DF9 global optimization with
  each item's `99` workflow/performance evidence. Let the user choose a small
  justified bounds extension, not an assumed general constraint API. Each ID
  requires user-approved admission with a new approved phase, rejection with
  reason, or re-deferral to a new explicit milestone checkpoint. This is review,
  not an implementation promise or automatic expansion of T6/T7.
- CLOSURE gate, before successor planning: review DF10's deferred removal of line
  extension against post-p2/p3/p4/p6 release evidence. Revisit only if meaningful
  benefit justifies the wrapping/fit tradeoff; record user-approved admission with
  a new phase, rejection with reason, or re-deferral to an explicit new checkpoint.
  PF7's KEEP disposition remains in force; DF1-DF9 checkpoints are unchanged.
- Audit all DF1-DF10 entry/closure dispositions and user approvals. Remove admitted
  features from `99` on approved planning, not code completion; only implementation
  makes the corresponding current contracts change. Every still-deferred item
  retains an explicit checkpoint, including any user-approved replacement gate.
- `just verify` passes from the completed worktree.

## T1 Decisions

Current approved target summary, incorporating T5a0s amendments; completed T1 and
T3j/T5a0/T5a0r decision records above remain historical evidence.

- `Theme` is one complete immutable value. Built-ins are constructors, and a
  construction-only builder produces custom complete themes.
- Canvas owns the root theme. Unthemed plots inherit it; explicit Plot themes
  replace it completely; facets add no scope.
- Components are surfaces, typography, axes, legend, composition, marks,
  annotations, and color scales. Plot-family theme blocks are not admitted.
- `CompositionTheme` owns `root_title_alignment`, optional
  `root_subtitle_alignment`, and independent `root_caption_alignment`, using
  `TitleAlignment`. Built-ins use Center, None, and Start. Subtitle follows final
  effective title unless separately instructed. Plot title/subtitle instructions
  override Canvas panel instructions, which override resolved theme defaults;
  an absent subtitle at all scopes follows the final title. This narrow policy
  applies across explicit presets without general theme merging. Canvas headings,
  panel captions, and other text roles retain independent ownership. Public names
  are `title_alignment`, `subtitle_alignment`, `panel_title_alignment`, and
  `panel_subtitle_alignment`; public exposure remains T6.
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
  and recorded environment provenance. Prefer first-face base-plus-combining-mark
  coverage within an EGC as a unit; otherwise permit first coverage per scalar
  when possible. Visible misses are fatal, with no provisioning or whole-word,
  emoji, or generic cluster-mapping extension.
- D1 bounded containing-frame recovery keeps explicit placements fixed; D2 uses
  strict Unicode-legal wrapping without emergency breaking, rotation, ellipsis,
  or font shrink as recovery. Broader controls require DF5 review, not silent admission.
- T6 exposes exactly automatic, exact-width/automatic-height, and exact-pair
  sizing. Consuming Plot/Canvas `width(u32) -> Self` and `size` completely replace
  requests; width after size clears height and positivity is checked at preflight.
  No `max_width` target or public Flutter bounds remain. Current APIs stay unchanged
  until T6; output dimensions belong to rendered `Output`.
- All PNG limits are format-local in all modes; no hypothetical PNG viability or
  automatic-only 64 MiB canonical SVG gate remains. Shared logical-work/font
  caps, representability, and fallible SVG allocation still apply.
- Themes may omit nonsemantic chrome but cannot hide caller-provided semantic
  content. Existing explicit policies own content presence and placement.
- Automatic numeric ticks have no fixed five-tick or public count target. An
  optional per-axis additive or Log10-decade data interval is the sole public
  automatic-tick preference; exact ticks remain the strict override.
- Shared Canvas legends use Canvas chrome and exact panel glyphs; conflicting
  glyphs for one semantic identity reject.
