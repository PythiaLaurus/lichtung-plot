# T5b-T5e Execution Plan

**Status: IN PROGRESS — T5b–T5c complete; T5d in progress; T5e pending.**

This is the execution partition of the four T5b–T5e umbrellas in
`91_IMPLEMENTATION_PLAN.md`, based on the verified T5a closure at `1e8e3ee`.
The user requested this planning work and approved the explicit-data-window
decision below. Planning is not evidence of implementation. `03_THEME_SYSTEM.md`
and `05_LAYOUT_SYSTEM.md` retain authority over target behavior; `01` and `02`
describe the implemented public contract until its scheduled migration.

`91` records umbrella progress. The tables here are the authoritative substage
status inventory; update them as work proceeds rather than maintaining another
copy of their statuses. All substages are initially PENDING. Complete, review,
verify, and commit each logical substage before its successor. An umbrella has
no separate implementation commit. Further splitting is allowed before a broad
substage starts, with explicit dependencies and unchanged behavior boundaries.

## Starting Point And Obligations

Reuse, do not reimplement, these T5a foundations:

- Twenty native ordinary interiors, five retained specialized interiors, and
  retained Facet/Canvas parent adapters; one selected mark/annotation payload.
- `RootFramePlan`, `RootTitleBlockPlan`, prepared text/font data, `PanelAxisPlan`,
  common projection and canonical quantum operands, and `LegendDrawPlan` replay.
- The one at-most-eight-round root driver, typed fit versus fatal failures,
  independently admitted records, processing visits, calls, cache entries,
  topology, discovery, and bytes; no alternate solver or fresh per-child ledger.
- Operation-local font freshness and immutable font bytes; one final capture,
  one Null validation, and concrete replay without post-freeze layout work.

The remaining work changes owners and policies deliberately left deferred. It
is not another family-wide extraction of scientific data or a second scene
representation. The source baseline and detailed evidence are in
`docs/t5a-closure.md`. A symbol already removed there is a regression-search
assertion, not a new implementation deliverable.

### Cross-Stage Handoff

| Boundary | Current state | Replacement owner | Required handoff |
| --- | --- | --- | --- |
| Discrete/continuous guide input | Current entries, retained calls, family continuous-guide inputs | T5b shared descriptor/measurement; T5d final scale values | One compatibility path and one final measured guide; no second descriptor introduced by T5d |
| Facet/Canvas parent allocation | Named adapters with legacy equal tracks | T5b | Delete parent adapters as native composition lands; preserve child producer/capture contracts |
| Embedded Plot headings/captions | Private policies carried; caption/heading-track integration deferred | T5b | Consume the common root-text producer locally, exactly once, with independent Canvas heading ownership |
| Mark/annotation requirements | Retained payloads and consequences with named `80x60` parity floor | T5c | Native hard/preferred/ink/domain inputs for every consumer before deleting the floor and deferred consequences |
| Mark paints and semantic color identity | Current resolved styles and mixed palette consumers | T5c styles; T5d identity/mapping | T5c consumes opaque data color plus owned alpha; T5d supplies final color/domain without taking style ownership |
| Special-purpose packing | Five `SpecializedLegacy` interiors with frozen payloads | T5e | Replace packing separately for each family; do not redo prepared statistics or permit backend discovery |
| Automatic/width-only root requests | Target only; public exact defaults remain | T5e private implementation, T6 public switch | One root engine with three requests; public exact defaults/getters remain until T6 |

### Required Evidence In Every Implementation Substage

Record the active ledger keys, input owner, validation stage, intrinsic supplier,
assigned constraints, feedback, cache identity, admitted allocations/work, frozen
payload, and old implementation deleted. No completion claim based solely on a
new type name or passing route-count test.

- Add tests with the implementation: semantic failures before fonts where
  font-independent; genuine geometry misses recover only through their named
  owner; arithmetic/font/resource failures are fatal and publish no artifact.
- Admit before materialization. Copying/reservation is records/bytes, not a new
  processing price. Charge actual searches/iterations, calls including cache
  hits, insertions, and topology at their approved independent boundaries.
- For new constraint-sensitive fields, update the immutable measurement input
  and cache identity together. Hits replay retained effects; rejected trials
  discard effects but never refund work. Use cache-on/off differentials on real
  parent layouts, plus a controlled fixture that actually demonstrates a hit.
- Keep raw identities distinct from display aliases, source-row identity from
  canonical address, and source nonzero intent from post-transform geometry.
- Prove zero source preparation, projection, packing, wrapping/shaping, style
  assembly, and candidate selection during selected capture and final replay.
- Inspect intentional SVG/snapshot changes with the source change. Preserve
  historical fingerprints as historical evidence, not current byte-parity claims.
  Record complete text, relevant geometry, and output-equivalent comparisons.
- Run focused tests while iterating, strict locked/offline all-target Clippy,
  then `just verify` before completion. Never substitute `cargo check`. No
  Python, `tools/`, runtime network/provisioning, or cross-`lichtung-*` dispatch.
- Rerun affected ignored release groups when a measured path changes. Preserve
  full workloads, rejected cases, min/median/max, actual trials and resource
  classes. No timing threshold or universal speedup claim is a CI requirement.

## Approved Data-Window Decision

The existing `x_limits`/`y_limits` APIs remain data-window policies, not root
dimensions. The user approved preserving their conventional clipping behavior.
The target clarification is in `05_LAYOUT_SYSTEM.md`; implementation tests land
in T5c1 and the relevant geometry substages.

| Case | Required target behavior |
| --- | --- |
| Finite mark outside an explicitly limited data axis | Keep its semantic/prepared identity; the data-panel clip may hide its paint. Do not extend the exact domain or reject merely because that paint lies outside the explicit window |
| Line/path or interval crosses an explicit boundary | Preserve the crossing geometry and clipping. Do not prefilter away endpoints needed to draw the visible crossing or move an endpoint to the boundary as data mutation |
| Stroked marker centered at an explicit boundary | Data-mark stroke may be clipped under that explicit window, even if the full marker would fit at another range |
| Automatic data axis | Include actual mark ink/clearance through the approved intrinsic, reserve, and automatic-domain rules; unresolved required ink is a fit miss, not silent cropping |
| One exact axis, one automatic axis | Apply the explicit window only along its constrained data dimension; automatic-axis geometry still follows its complete-ink rules |
| Title, subtitle, caption, axis/exact-tick label, legend | Preserve required content within its assigned layout allocation. Data clipping cannot hide layout overflow |
| Required annotation or label attached to data | The data-window policy alone does not authorize dropping its text/bracket. Use its existing explicit visibility/domain policy if one exists; otherwise an incompatible required annotation fails |
| Nonfinite geometry or nonzero source geometry collapsed by transformation/quantization | Preserve the existing canonical rejection, even if a clip would hide the malformed geometry. True scientific zeros keep their documented semantics |

This adds no clipping switch, annotation-omission policy, emergency wrapping, or
new public control. Audit existing Heatmap Elide and other explicit caller
policies against their ledger owners; do not extend them to other text roles.
If an existing default is inconsistent with the approved semantic-preservation
rules and has no explicit policy owner, stop that affected implementation and
ask the user rather than silently changing the default or legitimizing omission.

## T5b: Native Guides And Composition

**Entry:** T5a complete. Exact public root sizing remains unchanged. The existing
Plot/Canvas theme replacement and narrow title/subtitle precedence are settled.
No broader nesting, public alignment builders, or new guide positions are added.

| Step | Status | Depends on | Logical commit |
| --- | --- | --- | --- |
| T5b0 | COMPLETE | T5a | `docs(layout): pin native composition migration contracts` |
| T5b1 | COMPLETE | T5b0 | Umbrella; completed after T5b1a-c |
| T5b1a | COMPLETE | T5b0 | `feat(legend): classify shared guide kinds before fonts` |
| T5b1b | COMPLETE | T5b1a | Umbrella; completed after T5b1b1-b2 |
| T5b1b1 | COMPLETE | T5b1a | `refactor(legend): admit compatibility before fonts` |
| T5b1b2 | COMPLETE | T5b1b1 | Umbrella; completed after T5b1b2a-b |
| T5b1b2a | COMPLETE | T5b1b1 | `refactor(legend): resolve display labels before fonts` |
| T5b1b2b | COMPLETE | T5b1b2a | Umbrella; completed after T5b1b2b1-b3 |
| T5b1b2b1 | COMPLETE | T5b1b2a | `refactor(legend): prepare Scatter shared guides` |
| T5b1b2b2 | COMPLETE | T5b1b2b1 | Umbrella; completed after T5b1b2b2a-b |
| T5b1b2b2a | COMPLETE | T5b1b2b1 | `refactor(legend): prepare point-path shared guides` |
| T5b1b2b2b | COMPLETE | T5b1b2b2a | `refactor(legend): prepare remaining ordinary guide entries` |
| T5b1b2b3 | COMPLETE | T5b1b2b2 | Umbrella; completed after T5b1b2b3a-b |
| T5b1b2b3a | COMPLETE | T5b1b2b2 | `refactor(legend): classify prepared Canvas contributors` |
| T5b1b2b3b | COMPLETE | T5b1b2b3a | `refactor(legend): prepare faceted shared guides` |
| T5b1c | COMPLETE | T5b1b | `feat(legend): merge continuous guide contributors` |
| T5b2 | COMPLETE | T5b1 | Umbrella; completed with T5b2c2 |
| T5b2a | COMPLETE | T5b1 | `refactor(legend): resolve complete private legend roles` |
| T5b2b | COMPLETE | T5b2a | Umbrella; completed with T5b2b3b |
| T5b2b1 | COMPLETE | T5b2a | `refactor(legend): measure assigned discrete outside flow` |
| T5b2b2 | COMPLETE | T5b2b1 | `refactor(legend): replace legacy inside packing` |
| T5b2b3 | COMPLETE | T5b2b2 | Umbrella; completed with T5b2b3b |
| T5b2b3a | COMPLETE | T5b2b2 | `feat(canvas): install merged continuous domains` |
| T5b2b3b | COMPLETE | T5b2b3a | `feat(legend): measure and draw shared continuous guides` |
| T5b2c | COMPLETE | T5b2b | Umbrella; completed with T5b2c2 |
| T5b2c1 | COMPLETE | T5b2b | `refactor(layout): resolve guide frame gaps from themes` |
| T5b2c2 | COMPLETE | T5b2c1 | `refactor(layout): unify assigned outside guide frames` |
| T5b3 | COMPLETE | T5b2 | Umbrella; completed with T5b3b |
| T5b3a | COMPLETE | T5b2 | `feat(layout): allocate TextPanel sequence tracks` |
| T5b3b | COMPLETE | T5b3a | `feat(layout): allocate checked grid spans` |
| T5b4 | COMPLETE | T5b3 | Umbrella; completed with T5b4b |
| T5b4a | COMPLETE | T5b3 | `feat(layout): allocate measured facet tracks` |
| T5b4b | COMPLETE | T5b4a | `refactor(layout): retire facet parent adapter` |
| T5b5 | COMPLETE | T5b4 | Native measured Canvas tracks and complete local text |
| T5b5a | COMPLETE | T5b4 | `feat(canvas): allocate measured Canvas cell tracks` |
| T5b5b | COMPLETE | T5b5a | `feat(canvas): measure complete Plot-local text` |
| T5b5c | COMPLETE | T5b5b | `refactor(canvas): retire duplicate panel probe` |
| T5b5d | COMPLETE | T5b5c | Header and row-title spans owned by the Canvas Grid |
| T5b5d1a | COMPLETE | T5b5c | `feat(canvas): constrain spanning header columns` |
| T5b5d1b | COMPLETE | T5b5d1a | `feat(canvas): measure header bands` |
| T5b5d2 | COMPLETE | T5b5d1b | `feat(canvas): allocate measured row titles` |
| T5b6 | COMPLETE | T5b5 | Shared-domain and guide-frame ownership closed by T5b6c |
| T5b6a | COMPLETE | T5b5 | `refactor(canvas): skip absent shared guide trials` |
| T5b6b | COMPLETE | T5b6a | `refactor(canvas): install shared ranges before guide probes` |
| T5b6c | COMPLETE | T5b6b | `test(canvas): trace multi-owner guide recovery` |
| T5b7 | COMPLETE | T5b6 | `test(layout): close native composition ownership` |

### T5b0 — Source Map And Behavioral Fixtures

Inventory current `decorations/legend.rs`, `canvas.rs`, facet planning, shared
range preparation, and the relevant `04` rows. Distinguish already-frozen calls
from remaining legacy flow/track policies. Record a supported/rejected matrix
for each PlotKind in Canvas and generic facets, and reject unsupported shapes
before fonts; private adapter replacement must not silently expand support.

Pin representative fixture definitions, old output evidence, and expected
intentional differences: asymmetric gutters, sparse/spanning cells, long panel
titles, panel plus Canvas captions, shared/per-Plot/hidden guides, mixed explicit
themes, categorical/fixed/free facets, TextPanel-only and mixed canvases. Preserve
the current row-title versus panel-heading conflict unless separately approved.
This documentation step runs no test suite; execution evidence starts below.

#### T5b0 Current-Source Inventory And Baseline

The current-source comparison is against `1e8e3ee`, not the older ledger's
historical line-number table. This inventory is a deletion checklist for T5b1-7;
it does not declare those substitutions implemented.

| Current owner | Already retained | T5b replacement/deletion |
| --- | --- | --- |
| `decorations/legend.rs`: `LegendEntry`, `LegendIdentity`, `LegendDrawPlan` | Exact typed entry identity and glyph, resource-backed entry merge, measured operands retained before capture | T5b1 introduces one discrete/continuous descriptor. T5b2 replaces root-scaled `LegendMetrics::from_canvas`, 11/18 font clamps, six-entry column heuristic, hardcoded paint/gaps, and legacy column packing with assigned-width flow. Keep the selected draw-plan storage, not an alternate route |
| `decorations/plan.rs`, `orchestrator/facets.rs`, `canvas.rs` | Bounded automatic outside-guide trial/feedback and final containing-frame checks | T5b2/T5b6 route every guide through the one descriptor, measured producer, and exact enclosing-frame fit; replace duplicate helper-local gap/reserve logic. Keep first-fit order and existing trial budgets |
| `canvas.rs::preflight_shared_legend` | Renderer-selected `LegendEntry` merge and final draw calls; cached body probes | T5b1 resolves compatible guide contributors *before* fonts and without measuring children for compatibility. T5b6 consumes the descriptor, removing compatibility dependence on body probes and repeated title/label choice |
| `plots/shared_x/prepared.rs`, Canvas `prepare_shared_x` | Font-independent scientific domains and checked strict union retained | T5b6 installs union in native measured children and removes only adapter-only reapplication; the renderer-probe alternative was already removed by T5a |
| `layout/canvas.rs::allocate_composed_grid_fit`, facet and Canvas planning | Parent-assigned equal legacy cells with unequal decoration gutters and cached child results | T5b3 installs the active allocator and TextPanel consumer; T5b4/T5b5 replace equal-grid parent adapters separately. Keep explicit sparse coordinates, header spans, and one root driver |
| `layout/canvas.rs::PanelRequirements`, `canvas.rs::DeferredT5bPanelHeadingInputs` | Frozen legacy panel headers/row titles and private alignment-policy inputs; outer roots already use `RootTitleBlockPlan` | T5b5 integrates complete Plot-local title/subtitle/caption within its assigned cell and top-left TextPanel; remove legacy panel header storage, not the common outer-root title producer |
| `decorations/plan/children.rs` | Constraint/input keyed reuse and feedback replay | T5b4-6 update identities as tracks/guides change; hits charge calls, rejected trials retain work, no independent parent budget |

The closed *current API* composition matrix, to be tested before and after adapter
deletion, is:

| Shape | Current admission and responsible boundary |
| --- | --- |
| Ordinary Plot inside Canvas | Admitted when individually valid; `Canvas::panel` stores a Plot, with inherited Canvas or complete explicit Plot theme |
| Specialized Heatmap, Marginal, Beeswarm, Waterfall inside Canvas | Admitted as valid Plot cells under current route classification. Matrix is rejected before fonts as self-managed. Existing family-specific input restrictions still apply |
| Generic-faceted Plot inside Canvas | Rejected before fonts (`ParentCompositionAdapter::Canvas`), independently of child kind |
| Generic faceting of ordinary Plot kinds | Admitted subject to that kind's existing facet/input/channel validation. Panel-text annotation, value-derived category order, and some supplied geometries remain rejected by their specific policies |
| Generic faceting of Heatmap, Beeswarm, Waterfall | Admitted under existing kind/channel validation, with current parent/cell policies; no new topology inferred from the private selected-payload representation |
| Generic faceting of Matrix or Marginal | Rejected before fonts as self-managed panels (`ParentCompositionAdapter::Facet`) |
| Canvas inside Canvas | No public insertion type; `Canvas::panel` accepts a Plot, so there is no nested-Canvas planning route |

`src/layout/interior.rs::ParentCompositionAdapter::validate_child` is the
pre-font unsupported-shape gate. `canvas.rs::validate_structure` rejects
faceted-in-Canvas and Matrix-in-Canvas; `orchestrator/validation.rs` rejects
Matrix/Marginal faceting and other family-specific channel combinations.
Current late `SelfManaged` checks in the parent planners are redundant guards,
not permission to add a supported shape. Verify the semantic timing and remove
obsolete late alternatives when native parents replace the adapters. A current
row title is incompatible with any present Plot title/subtitle in its row; retain
this rule even after Plot-local alignment becomes effective.

| Regression fixture / source | Current contract to pin; intended T5b change |
| --- | --- |
| `tests/benchmarks/legends.rs`, `docs/local-search-benchmarks.md` | Nonempty right-fit, body-starvation→top, genuine compact-right, many-entry reflow, fixed edge failure, sparse/dense inside, shared facet/Canvas, controlled cache replay. Three interleaved release samples, actual trial/round/call counters and Auto/selected-edge SVG/PNG/pixel identity. T5b changes flow/track geometry deliberately; older timings and hashes stay attributed to their T5a checkout |
| `src/canvas/admission_tests.rs`, `tests/composition`, seven parent/title snapshots in `tests/visual_regression` | Exact defaults and varied sizes, mixed/all-TextPanel, spans and sparse coordinates, current row-title conflict, Canvas root heading independent of private panel policy. T5b adds simultaneous local and Canvas captions plus varied mixed themes; snapshot changes must be inspected against source and geometry |
| `src/orchestrator/facets/parent_tests.rs`, facet/scales integration | Fixed/free x/y, strip text, exact shared domains and empty/unavailable estimate panels; T5b changes track widths while retaining data-panel alignment, source rows, and one semantic owner per caption |
| `src/decorations/plan/tests.rs`, `src/decorations/plan/children.rs` | Complete affected-frame fit, rejected/provisional/final choices, cached feedback replay, and fatal-versus-fit classification. Native composition must prove several guide owners and cache-on/off identical outputs; it must not infer reuse from multiple root rounds |

The initial implementation report for T5b1 must distinguish the current lack
of a publicly reachable populated automatic inside-corner selection from its
explicit fixed-corner controls. Do not manufacture automatic corner counts or
broaden public nesting to create a benchmark. Baseline snapshots and T5a release
measurements already exist; T5b0 pins their fixture sources and conditions
without rerunning tests in a documentation-only stage.

T5b0 closure: current owners, supported/rejected composition shapes, existing
fixture sources, and deliberate T5b replacements are pinned above. This adds no
runtime code or public API; `git diff --check` is the documentation gate. The
next step is T5b1. Completion here does not change the T5b umbrella's PENDING
implementation status.

### T5b1 — Prepared Guide Semantics And Compatibility

This migration is split into three dependency-ordered logical commits: T5b1a
classifies continuous-family guides and configured visible discrete channels
before fonts, rejecting known mixed kinds; T5b1b replaces that channel-level
classification with the actual prepared discrete entry set, including explicit
layer entries and empty contributors, and moves identity/glyph/title compatibility into the
same prepared owner instead of probing a measured body; T5b1c supplies and
merges the complete continuous descriptor under that owner. They evolve one
active descriptor, not parallel render paths. T5b1 as an umbrella has no commit
and remains incomplete until all three are verified. T5b1a's early tag is a
validation input, not a selected guide or proof that every configured channel
produces entries. A continuous guide's
native measurement belongs to T5b2, but its semantics and compatibility must
already be final in T5b1c. A genuine empty contributor does not manufacture a
kind, and Hidden does not override data-color domain coordination.

Create one kind-discriminated private descriptor for discrete and continuous
contributors using current prepared semantic inputs. Resolve identities,
display titles/labels, complete glyphs, exact/automatic envelopes, missing blocks,
and ownership before the font census. Feed the existing guide path immediately;
do not add unused final-scale placeholders or a second current descriptor.

Discrete equality is typed identity plus resolved label and complete glyph;
continuous compatibility and domain merging follow `03` exactly. Shared guides
format the final merged domain once under Canvas typography. Preserve hidden
contributors' visible-mark domain participation and the rejection of explicit
per-Plot placement under Shared. Replace duplicate family/Canvas compatibility
branches; T5d later supplies its final resolved values to this same path.

Evidence: compatible/incompatible kinds, titles, aliases, shape/stroke/alpha,
fixed/automatic continuous domains, source ordering, hidden contributors, and
failure before font discovery or any artifact. No separate legend sections.

T5b1a evidence: the active Canvas preflight now rejects a Shared guide with a
configured discrete channel alongside Heatmap/Beeswarm continuous color before
font discovery. An unconfigured Scatter or a hidden discrete legend contributes
no competing kind; non-Shared Canvas behavior is unchanged. T5b1b must replace
this conservative channel tag with actual prepared entries (including labeled
layers, filtered/empty data, and full glyph equality); T5b1c must supply the
complete continuous descriptor. This result is not a claim that all Shared
guides already render under the final policy. Focused fixture and strict Clippy
pass. `just verify` passed after formatting: 1,025 library tests (17 ignored),
467 integration tests including 27 snapshots, API docs, and package verification
(271 files). Log: `/tmp/opencode/t5b1a-verify.log`.

T5b1b is split because the current `merge_exact_entries` and title compatibility
use the post-font theme/measurement allocator. T5b1b1 moves that one comparison
and reservation contract onto the already-prepared operation ledger, with no
parallel compatibility route and identical glyph/identity rules. T5b1b2 then
moves actual family entry/style preparation and merged descriptor ownership ahead
of the font census, replacing the temporary channel-level inference from T5b1a.
Neither substep alone establishes the full T5b1 acceptance contract. T5b1b2a
transfers current Plot/facet/Canvas display-title and alias resolution to the
same operation ledger without requiring an already-prepared font environment;
actual entry generation still follows selected mark measurement. T5b1b2b moves
the originating family entries/glyphs into prepared ownership and eliminates
post-font compatibility/body-probe dependencies. Splitting these contracts keeps
the same current `LegendEntry` representation; no public selector is added.

T5b1b1 completion: `merge_exact_entries` now takes the current operation ledger
instead of requiring a prepared font environment. Its identity/title/label/glyph
rules are unchanged; comparisons charge actual examined bytes and merging admits
retained entries before allocation. Plot, facet, and Canvas callers consume the
same routine with their existing root ledger. The pre-font test covers duplicate
merging, conflicting glyph refusal, exact work/storage limits, and zero font
discovery. Focused legend and composition tests passed; `just verify` passed
1,026 library tests (17 ignored), 467 integration tests, docs and package checks
(271 files). Log: `/tmp/opencode/t5b1b1-verify.log`. T5b1b2 is in progress:
family entries are not yet all prepared before the font census.

T5b1b2a completion: `resolve_entries` and `resolved_title` use the owning
generation resources directly. Identity display aliases, titles, duplicate
merging, and source bytes can be processed before font discovery; the title
copy is charged as records/bytes, not synthetic processing visits. Plot, facet,
Canvas, and Matrix consumers have migrated to these signatures. Focused legend
and composition tests passed; `just verify` passed 1,027 library tests (17
ignored), 467 integration tests, docs and package verification (271 files).
Log: `/tmp/opencode/t5b1b2a-verify.log`. Actual family entry generation and
glyph compatibility still await T5b1b2b.

T5b1b2b in-progress evidence: Scatter now has one guide-glyph producer over
root-prepared semantic groups and current theme; the same order and glyphs feed
selected marks. For an all-Scatter Shared Canvas, canonical hue groups and
actual merged entries/title are retained before font discovery, including visible
marks whose local legend is hidden. Compatibility does not read probe entries;
the existing body probes remain for the T5b2/T5b6 *geometry* fit contract.
Mixed explicit-theme conflicting glyphs refuse before fonts, while label and
output equivalence checks for the existing shared Scatter fixtures pass. This
does **not** close T5b1b2b: other families still generate their entries during
constrained measurement, and the temporary channel-kind inference must be
replaced by actual prepared contributors before claiming T5b1 complete.

T5b1b2b1 is an independently reviewable source migration: it covers only the
all-Scatter Shared Canvas owner, including a hidden contributor to the hue domain.
The non-Scatter producer remains explicitly classified for later migration,
not a fallback when a prepared Scatter descriptor fails. T5b1b2b2a extends
the same owner to Line, Trajectory, and LinePath, including labeled point-layer
entries; T5b1b2b2b covers the remaining ordinary entries;
T5b1b2b3 connects eligible specialized discrete contributors and faceted owners,
removes the channel-level kind inference, and proves one pre-font descriptor
across all admitted routes. None changes the existing supported nesting matrix.

T5b1b2b3a replaces the Canvas channel-level kind inference with actual prepared
entry presence and a closed shared family dispatch, including group-only lines,
labeled layers, hidden guides, and Marginal. T5b1b2b3b moves the facet parent's
shared-guide compatibility to globally coordinated pre-font prepared entries;
it retains panel body probes only for containing-frame geometry. Both must close
before T5b1b2b3 or T5b1b can be marked complete. T5b1c separately supplies
complete continuous descriptors under the same owner.

T5b1b2b3a completion: `GuideKind` now consumes prepared semantic input and
actual contributing entry presence, before the font census. It distinguishes
group-only lines and genuinely labeled point layers, honors Hidden policy, and
identifies Marginal's populated discrete guide independently of `PlotKind`.
One closed `PreparedInteriorInputs::guide_entries` dispatcher supplies all
20 ordinary kinds and Marginal to the Canvas's pre-font shared descriptor;
Waterfall has no discrete entry. Matrix remains an unsupported Canvas child.
Mixed-kind rejection and explicit per-Plot-edge refusal precede fonts. A shared
Heatmap with no-guide Scatter or hidden guide remains kind-compatible, whereas
Heatmap with a labeled Line layer or Marginal hue fails as mixed kinds. Focused
Canvas, 58 composition, and five normal legend-contract tests passed.
`just verify` passed 1,038 library tests (17 ignored), 468 integrations including
27 snapshots, docs, and package verification (272 files); log
`/tmp/opencode/t5b1b2b3a-verify.log`. Facet preparation was still post-font at
that checkpoint; T5b1b2b3b closes it below. T5b1 itself still awaits T5b1c.

T5b1b2b3b completion: faceted roots now merge real prepared discrete glyphs,
aliases, titles, and observed global hue identities before the font census.
`FacetPlan` consumes the retained descriptor; it no longer derives compatibility
from child body probes or performs a first probe solely to discover hues. One
child measurement under the prepared global order still supplies scientific
ranges, gutters, and enclosing-frame fit. Heatmap and Beeswarm continue to
own continuous colorbars pending T5b1c; a faceted discrete display-label
collision fails before font discovery. Source-row and theme ownership, all
supported facet scale modes, exact-size output, and Auto/selected-edge SVG/PNG
equivalence remain covered. Facet child cache misses decrease from 16/24 to
12/18 (adequate/pressured); that is removed discovery work, not an assumed
cache hit. `just verify` passed 1,039 library tests (17 ignored), 468
integrations, docs and package verification (272 files), and all three ignored
release legend groups passed. Logs: `/tmp/opencode/t5b1b2b3b-verify.log` and
`/tmp/opencode/t5b1b2b3b-legends-release.log`. T5b1 remains IN PROGRESS for
the complete continuous descriptor; guide flow and native tracks are T5b2+.

T5b1c completion: Shared Canvas uses one kind-discriminated prepared descriptor.
Heatmap and Beeswarm contribute raw color-field identity, resolved title,
complete semantic low/center/high paint, diverging center, missing-key paint/text,
and either exact domain or automatic data envelope before font discovery. One
owner validates title/scale/missing compatibility, unions automatic domains,
requires identical exact domains containing every automatic envelope, and
generates a single unique numeric-label vector only from the final domain.
Constant domains retain one label. Its title, final labels, and optional missing
key enter Canvas-owned font census under the shared-guide address; the merged
semantic identity enters root feedback. Tests cover two unequal automatic
Heatmaps, contained versus conflicting exact domains, Heatmap/Beeswarm palette
conflict, close endpoints requiring precision retries, independent work/storage
refusal, and no font discovery before semantic rejection. Focused Canvas,
numeric-source, 58 composition, and 37 Heatmap integration tests pass.
`just verify` passed 1,044 library tests (17 ignored), 468 integration tests
including 27 snapshots, docs and package verification (273 files); log
`/tmp/opencode/t5b1c-verify.log`. T5b2 will measure and publish the one shared
continuous guide from this descriptor; existing specialized colorbar geometry
is not falsely called native guide measurement here. The public surface and
supported nesting matrix are unchanged.

T5b1b2b1 completion: `PreparedScatter` now produces guide entries, order, and
complete point glyphs from the same semantic groups/style source used by its
selected mark payload. All-Scatter Shared Canvas roots retain their canonical
hue domain and merged guide entries/title before font discovery; hidden local
legends still contribute visible mark identities to that domain. Conflicting
glyphs from mixed explicit themes reject before font discovery. The prepared
guide owns compatibility; the existing body probes are retained only for
complete-frame geometry until T5b2/T5b6. Real composition and pre-font tests
pass; all three release legend samples preserve successful Auto/selected-edge
output equality. Canvas child-cache misses change from 16 to 12 (adequate) and
24 to 18 (pressured) because the Hue-only body probes are gone. The selected
guide flow and root-round counts are unchanged; adequate Canvas text work
falls from 24 to 12 proposals and 52 to 40 exact trials, while pressured text
counts remain unchanged. Primary `just verify` passed 1,030 library tests (17
ignored), 468 integration tests, docs, and package verification (271 files).
Logs: `/tmp/opencode/t5b1b2b1-verify.log` and
`/tmp/opencode/t5b1b2b1-legends-release.log`. Remaining ordinary, labeled-layer,
specialized discrete, and facet contributors belong to T5b1b2b2/b3; no claim
of complete T5b1 or continuous shared measurement is made.

T5b1b2b2a completion: the same source of `LegendEntry` glyphs is used before
fonts and during selected mark measurement for Line, Trajectory, and LinePath,
including ribbon/marker combinations and labeled supplemental point layers.
The prepared Shared Canvas hue domain now covers Scatter and these three path
families without Hue-only child body probes. All-point/path Shared canvases
perform compatibility before fonts; conflicting mixed mark glyphs fail there,
and hidden discrete guides still contribute visible-mark color identities.
The retained enclosing-frame body probes remain for geometric fit, not guide
compatibility. Tests check all path-family pre-font glyphs against selected
payloads, mixed Scatter/Line source order and rejection timing, unchanged
composition output, and theme-owner provenance once in preparation rather
than once per root round. `just verify` passed 1,033 library tests (17 ignored),
468 integration tests including 27 snapshots, docs and package checks (271
files); log `/tmp/opencode/t5b1b2b2a-verify.log`. Remaining ordinary families,
specialized discrete contributors, and Facet ownership still await b2b/b3.

T5b1b2b2b completion: all 20 ordinary routes now have one prepared discrete
entry producer (or an explicitly empty entry set). Existing selected payloads
consume the same producers for point/path, bars, all categorical distributions
including supplied Raincloud, Density, Regression, Dumbbell, and Interaction.
Histogram, Qq, PointInterval, Coefficient, and Forest produce no detached
entries. An all-ordinary Shared Canvas prepares actual merged glyphs, aliases,
title, and row-major color identities before font discovery; geometry body
probes remain only for containing-frame fit. Explicit child edges under Shared
reject before fonts instead of being silently overridden. Tests include mixed
Nature/Science glyph rejection for Bar, distribution, Density, point/path,
empty-entry Histogram beside a populated guide, and hidden-but-visible hue
identities. Primary `just verify` passed 1,038 library tests (17 ignored), 468
integration tests, docs and package verification (271 files); log
`/tmp/opencode/t5b1b2b2b-verify.log`. Specialized discrete owners, faceted
prepared guide ownership, and removal of the channel-inferred guide-kind gate
are still T5b1b2b3 obligations; continuous contributors remain T5b1c.

### T5b2 — Native Discrete And Continuous Measurement

The private typography/chrome migration (T5b2a) must be in active measurement,
font census, and frozen draw before replacing flow; no root-size-scaled fallback.
T5b2b replaces discrete/continuous assigned-width measurement using T5b1's
descriptor. T5b2c connects the same result to Plot, facet, and Canvas enclosing
frames and removes superseded helper-local gap/reservation logic. These steps
share one active guide representation and are committed in order; this umbrella
has no separate commit.

T5b2b is split into three dependency-ordered commits: replace assigned-width
outside discrete flow and its premeasurements (b1); replace the remaining
legacy `Legend::new` packing for explicit inside/Marginal/Matrix consumers with
the same intrinsic and assigned-width semantics (b2); then measure and draw
continuous Canvas guides from their prepared descriptor (b3). The outside and
inside placements are distinct current consumers during the migration, not
alternative fallbacks for one selected guide. Close the umbrella only when all
three share the theme-owned measured representation and old packing is gone.

T5b2a closure: active discrete Plot/facet/Canvas guides now measure and freeze
title, label, frame, key, and spacing roles from the owning theme rather than
deriving typography from root dimensions. Specialized Heatmap/Beeswarm guides
use the same title and label text roles. Prepared font census admits these
roles, including Canvas's continuous descriptor and specialized guide values;
cell annotations remain body text. T5b2b adds continuous descriptor drawing
and replaces the existing edge/flow algorithm; T5e replaces specialized
colorbar packing. Updated affected SVG fingerprints and inspected the
Beeswarm clip-structure snapshot,
while retaining genuine right-guide rejection in the native cache-equivalence
fixture. `just lint` and `just test` pass: 1,045 library tests and 468
integration tests, including 13 visual regressions. Logs:
`/tmp/opencode/t5b2a-lint.log`,
`/tmp/opencode/t5b2a-test.log`. T5b2b is next.

T5b2b1 closure: outside discrete guides derive checked key-plus-label minimum
and preferred widths before entry text layout. Assigned horizontal rows keep
fixed preferred entry widths; assigned vertical columns share one fixed width.
Compact width uses the single quantized half-width trial, and row baselines
include the union of ascent and descent. Title gaps are active only for present
titles; a zero-entry guide has no assigned geometry. Normative worked examples,
hard-unbroken label failure, actual body-starvation recovery, 1,048 library
tests, affected composition/geometry/visual integration tests, refreshed
exact-work traces, and strict Clippy pass. Logs:
`/tmp/opencode/t5b2b1-lint.log`, `/tmp/opencode/t5b2b1-lib.log`,
`/tmp/opencode/t5b2b1-legend-bench.log`,
`/tmp/opencode/t5b2b1-integration.log`, and
`/tmp/opencode/t5b2b1-geometry-integration.log`. T5b2b2 removes legacy
inside-guide packing.

T5b2b2 closure: ordinary Plot inside overlays and the Matrix-owned guide now
use the same assigned-width `Legend` measurement as outside guides. The root
width/height percentages, six-entry column heuristic, multi-column search,
row-center reconstruction, and old `Legend::new` constructor have been removed;
an inside fit miss retains its typed guide address and rejects before capture.
Exact entry-Arc admission remains part of this path. The inspected Matrix SVG
snapshot changes only the title baseline placement. Focused Legend, panel,
Matrix, explicit-inside, annotation, composition, visual, and measured-work
tests pass, along with strict Clippy. Logs: `/tmp/opencode/t5b2b2-lint.log`,
`/tmp/opencode/t5b2b2-legend-unit.log`,
`/tmp/opencode/t5b2b2-plan-unit.log`,
`/tmp/opencode/t5b2b2-matrix-unit.log`, and
`/tmp/opencode/t5b2b2-composition-integration.log`. T5b2b3 adds the
continuous guide producer.

T5b2b3 has two ordered commits. First install the prepared merged domain into
Heatmap and Beeswarm children before font discovery and frozen color selection,
updating any still-visible local guide labels and selected colors together.
Then measure and draw the sole Canvas continuous guide from the same descriptor,
suppressing its children's local colorbars and reserves in that same change.
This prevents a displayed shared scale from disagreeing with child mark colors.

T5b2b3a closure: Canvas installs the pre-font merged continuous domain in each
prepared Heatmap/Beeswarm child before its text census or selected geometry.
Heatmap cell colors and local value labels use the merged domain; Beeswarm
prepared points, gradient stops, and local labels do likewise. A selected
numeric value common to two Beeswarm children now has exactly the same RGB.
Existing local guides still display that same domain until T5b2b3b replaces
them with one Canvas guide. Focused Canvas, Heatmap, Beeswarm, and shared-domain
tests, plus strict Clippy, pass. Logs: `/tmp/opencode/t5b2b3a-lint.log`,
`/tmp/opencode/t5b2b3a-canvas-unit.log`,
`/tmp/opencode/t5b2b3a-heatmap-unit.log`, and
`/tmp/opencode/t5b2b3a-beeswarm-unit.log`.

T5b2b3b closure: Canvas measures the prepared continuous descriptor at each
assigned outside edge and freezes one title, projected numeric labels, ordered
gradient stops, and optional missing-key block before capture. A Shared Canvas
now suppresses child Heatmap/Beeswarm colorbar reserves and paints, while
retaining the merged data-color domain. The gradient, labels, and theme-owned
chrome participate in guide fit and root feedback. Ordinary, constant, and
diverging supported compositions render one gradient with final-domain labels;
the diverging missing key remains present once. Projected length, compact
selection, horizontal/vertical placement, Plot-local unaffected fingerprints,
focused Canvas/producer/Heatmap/Beeswarm tests, and strict Clippy pass. Logs:
`/tmp/opencode/t5b2b3b-lint.log`,
`/tmp/opencode/t5b2b3b-continuous-unit.log`,
`/tmp/opencode/t5b2b3b-canvas-unit.log`, and
`/tmp/opencode/t5b2b3b-heatmap-integration.log`. T5b2c connects the unified
guide measurement to native enclosing frames.

T5b2c proceeds in two dependency-ordered commits: first make Plot/facet and
Canvas shared-track gaps explicit owner-local theme roles (preserving the
current effective defaults and the exact Canvas override); then route the
measured guide rectangles and residual panel through common assigned-frame
requirements instead of parent-specific subtraction and reconstructed gaps.
The second step must keep complete body and guide fit inside the one bounded
attempt and preserve exact edge/compact budgets.

T5b2c1 closure: private Plot/facet outside-guide and Canvas shared-guide
track gaps now resolve from distinct owner-local theme roles, preserving their
existing effective 8.8px and 17.6px defaults. The explicit Canvas gap remains
an exact override. Both assigned-frame reservation and placement consume the
same measured outside gap; a changed private theme role moves the Plot guide
exactly once. Focused frame, facet, Canvas gap, complete-guide benchmark, and
strict Clippy checks pass. Logs: `/tmp/opencode/t5b2c1-lint.log`,
`/tmp/opencode/t5b2c1-frame-gap.log`,
`/tmp/opencode/t5b2c1-legend-bench.log`, and
`/tmp/opencode/t5b2c1-canvas-gap.log`.

T5b2c2 closure: Plot, facet, and Canvas outside guides use one measured
`AssignedGuideFrame` for available guide constraints, quantized residual-body
reservation, and suffix-aware placement. Canvas no longer subtracts guide
width/height directly, and Plot no longer reconstructs separate side-specific
gaps or retains the old frame-requirements helper. The final frame still checks
every affected child under the same bounded edge/compact attempt. Four-edge
quantized enclosure, Plot and facet body feedback, Canvas shared continuous
output, unchanged Plot/Marginal SVG fingerprints, exact guide-work contracts,
and strict Clippy pass. Logs: `/tmp/opencode/t5b2c2-lint.log`,
`/tmp/opencode/t5b2c2-assigned-frame.log`,
`/tmp/opencode/t5b2c2-legend-bench.log`,
`/tmp/opencode/t5b2c2-plot-frame.log`,
`/tmp/opencode/t5b2c2-facet-frame.log`, and
`/tmp/opencode/t5b2c2-canvas-frame.log`. T5b3 is next.

Use separate guide intrinsics and assigned-width measurement, exact label plans,
fixed entry widths before greedy flow, one preferred and one compact flow per
edge, and the exact continuous-guide minimum-length formulas from `05`.
Retain title/entry/key/label or gradient/missing-block geometry in the existing
frozen draw representation. Zero entries mean absent guide; keys reflect final
mark glyphs. Supersede legacy measurement/spacing/style reconstruction in place.

Evidence: the normative horizontal/vertical worked examples, minimum-width
failure before wrapping, duplicate compact allocation reuse, all explicit edges,
constant/diverging guides, endpoint overhang, nonempty many-entry reflow, and
actual body-starvation recovery. Glyph/style/resource errors never become another
edge trial. Do not make unreachable automatic inside-corner selection public.

### T5b3 — One Track Allocator, Used By Active Parents

T5b3 has two dependency-ordered commits: install one quantized sequence-track
allocator in the active Canvas TextPanel content producer (a), replacing its
centered fixed-gap placement with intrinsic top-left measured content; then
extend the same allocator with checked Grid topology and spans and consume its
one-column/two-row Grid in TextPanel (b). Canvas/facet parent track migration
remains T5b4/T5b5. Neither step leaves a dormant competing TextPanel producer.

T5b3a closure: quantized tracks now perform one checked proportional shrink,
largest-remainder tie by canonical index, capped equal surplus redistribution,
and explicit trailing slack. The active Canvas TextPanel measures title/body
intrinsics, allocates its fill width and nonfill two-row sequence, then freezes
complete text at the post-inset top-left instead of centering it in spare cell
space. Private composition roles retain the former 16px insets and 12px gap.
Hand-computed allocation, one-quantum height refusal, exact resource caps,
small/large TextPanel position invariance, mixed-theme ownership, Canvas
admission/capture, and strict Clippy pass. Logs: `/tmp/opencode/t5b3a-lint.log`,
`/tmp/opencode/t5b3a-track-unit.log`,
`/tmp/opencode/t5b3a-text-panel-integration.log`, and
`/tmp/opencode/t5b3a-canvas-admission.log`. T5b3b adds Grid and spans to this
same producer.

T5b3b closure: the active TextPanel producer now assigns its one-column,
two-row Grid in column-before-row order, measuring both title and body only
after the column receives its width. The same quantized track allocator admits
Grid topology before track arrays, orders spans by length and canonical child
address, raises covered flexible tracks by capped equal surplus, and rechecks
all hard spans after the single allocation. Tests cover a spanning header,
overlapping feasible/infeasible spans, capped redistribution, duplicate and
invalid coordinates, empty children, sparse noncollapsing trailing tracks,
TextPanel placement, Canvas admission/capture, and strict Clippy. Logs:
`/tmp/opencode/t5b3b-lint.log`,
`/tmp/opencode/t5b3b-grid-unit.log`,
`/tmp/opencode/t5b3b-text-panel-integration.log`, and
`/tmp/opencode/t5b3b-canvas-admission.log`. T5b4 migrates the facet parent.

Implement the `05` Grid/sequence rules in quanta: columns before rows, canonical
span order, flexible/inflexible tracks, proportional shrink, largest-remainder
ties, capped surplus, trailing slack, and post-allocation span validation.
Use the allocator and sequence primitives in the active TextPanel content
producer under its currently assigned Canvas cell: measure intrinsic title/body
requirements, then place complete content top-left after chrome insets. This is
the first production consumer; T5b5 supplies its final native parent tracks. Do
not leave a dormant general-purpose layout framework. Keep data-panel alignment
separate from unequal decoration gutters. Admit complete sparse topology before arrays.

Evidence: hand-calculated vectors, exact one-quantum ties, overflow/refusal,
infeasible overlapping spans, empty/trailing noncollapsing rows and columns,
zero-size empty children, header spans, and no fallback allocation search.
Intrinsic and measured-result interfaces must support later specialized parents
without implementing their packing in this step.

### T5b4 — Native Facet Parent

Migrate in two commits: T5b4a installs the T5b3 Grid allocation for actual
facet cells, keeping the existing measured-child and guide trial owners while
the panel minimum/gutter inputs are transferred. T5b4b removes the remaining
legacy parent adapter, its equal-grid path, and duplicate per-child probes in
favor of one native containing-frame plan with unchanged bounded feedback.

T5b4a closure: root-prepared per-panel body intrinsics and column/row-specific
decoration gutters feed the active quantized Grid axis allocator, including
declared sparse and trailing tracks. The former equal facet allocator and its
exact-budget wrapper are gone; child probe/feedback routing remains under the
current parent owner until T5b4b. An asymmetric fixture proves unequal data
column widths and aligned row positions. Focused facet-parent, sparse/header,
fixed/free estimate, four-scale Beeswarm/Waterfall fingerprint, resource-limit,
and strict Clippy checks pass. Logs: `/tmp/opencode/t5b4a-lint.log`,
`/tmp/opencode/t5b4a-tracks-unit.log`,
`/tmp/opencode/t5b4a-parent-unit.log`,
`/tmp/opencode/t5b4a-beeswarm-fingerprint.log`, and
`/tmp/opencode/t5b4a-waterfall-fingerprint.log`. T5b4b retires the remaining
parent adapter.

T5b4b closure: one `native_parent` now allocates the measured facet Grid,
checks containing guide/body/strip constraints, plans each affected cell once,
submits the measured axis gutters, and freezes the selected panel results.
The old layout-only child probe, second panel-planning pass, and late
self-managed alternative were removed; the early unsupported-shape gate stays
in force. Exact shared-domain and legend identities, cell addresses, feedback,
and child cache keys remain under the single root. Focused parent tests,
cache-on/off complete-guide benchmarks (including no intermediate `layout`
phase), sparse and shared-axis composition, unavailable fixed/free estimates,
unchanged Beeswarm/Waterfall SVG fingerprints, and strict Clippy pass. Logs:
`/tmp/opencode/t5b4b-lint.log`, `/tmp/opencode/t5b4b-parent-unit.log`,
`/tmp/opencode/t5b4b-legend-bench.log`,
`/tmp/opencode/t5b4b-beeswarm-fingerprint.log`, and
`/tmp/opencode/t5b4b-waterfall-fingerprint.log`. T5b5 is next.

Replace the facet parent adapter and equal-panel allocation with measured tracks.
Keep canonical facet cells, fixed/free domain rules, fixed-axis outer ownership,
free-axis panel ownership, strip placement, and exact Plot theme ownership.
Use prepared child intrinsics, remeasure only changed assignments, and retain
all shared axes/strips/guides/child plans before capture. Delete the facet adapter
and obsolete facet-specific stabilization, not the common bounded root driver.

Evidence: all four scale modes, sparse Cartesian cells and wrap trailing slots,
matching categorical domains, empty/unavailable estimate panels, Forest Log10,
asymmetric labels with aligned data panels, and native/deferred specialized child
combinations allowed by the support matrix. Cache-on/off bytes and actual-hit
feedback replay must agree through rejected guide attempts and later rounds.

### T5b5 — Native Canvas Cells And Complete Local Text

Four ordered commits: T5b5a transfers the active Canvas Grid to T5b3 tracks
using prepared Plot/TextPanel intrinsics, declared sparse slots, and measured
gutters. T5b5b replaces the old header in place with complete
title/subtitle/caption within each Plot cell and final resolved alignments.
T5b5c removes the remaining duplicate layout-only child adapter and retains
one selected parent/child plan. T5b5d1a admits column-header span widths into
the active Grid; T5b5d1b replaces fixed column/row header bands with measured
text plans. T5b5d2 moves row titles onto those Grid tracks and
deletes the old OwnedInk workaround. No stage introduces a new public
nesting shape or second renderer lifecycle.

T5b5a closure: Canvas provisional and final cell allocations now use the T5b3
Grid tracks with Plot-scoped intrinsic body/title widths, TextPanel intrinsic
requirements, declared empty/trailing coordinates, and feedback-owned column
and row gutters. Root and per-Plot title summaries are each prepared once
before any assigned child probes; the old equal-grid allocator was removed.
Mixed TextPanel/Plot and sparse gaps, asymmetric TextPanel columns, header and
row-title contracts, long Plot title pressure, once-per-owner intrinsic census,
exact Canvas admission, representative shared-guide work, unchanged Beeswarm
parent SVG fingerprints, and strict Clippy pass. Logs:
`/tmp/opencode/t5b5a-lint.log`, `/tmp/opencode/t5b5a-tracks-unit.log`,
`/tmp/opencode/t5b5a-admission.log`,
`/tmp/opencode/t5b5a-canvas-unit.log`, and
`/tmp/opencode/t5b5a-legend-bench.log`. T5b5b replaces local panel headers
with full Plot-local title/subtitle/caption flow.

T5b5b closure: the common `RootTitleBlockPlan` now measures and freezes every
present embedded Plot title, subtitle, and caption inside its assigned Canvas
cell. Plot alignment overrides Canvas panel policy, which overrides the Plot's
resolved theme; a missing subtitle policy follows the final title. The Canvas
root title/caption remains separate. The old panel-header text producer was
deleted, and the Canvas font census and prepared Plot widths include local
captions before font discovery. The measured local body retains the rest of
the cell. Title/subtitle and simultaneous local/root caption integration,
row-title conflict, theme ownership, cached/uncached replay, guide pressure,
and strict Clippy pass. The inspected Canvas root/title SVG snapshot changes
the embedded title order/alignment and keeps all axis/mark content. Logs:
`/tmp/opencode/t5b5b-lint.log`, `/tmp/opencode/t5b5b-captions.log`,
`/tmp/opencode/t5b5b-canvas-unit.log`,
`/tmp/opencode/t5b5b-legend-bench.log`, and
`/tmp/opencode/t5b5b-root-snapshot.log`. T5b5c removes the remaining
layout-only child probe.

T5b5c closure: Canvas now plans each occupied Plot cell's complete selected
`PanelPlan` once under its final local-text body constraint, using its axes
and immutable mark plans for that same parent's gutter feedback. The old
`Layout`-phase child probe and its duplicate axis observation are gone; the
pre-font compatibility/body probes retain their separate purpose. Cached and
uncached output and feedback replay, shared guide attempt counts, root/local
captions, retained-text capture, and the inspected title-block snapshot remain
equal under focused tests. Strict Clippy passes. Logs:
`/tmp/opencode/t5b5c-lint.log`, `/tmp/opencode/t5b5c-child-reuse.log`,
`/tmp/opencode/t5b5c-legend-bench.log`,
`/tmp/opencode/t5b5c-admission.log`, and
`/tmp/opencode/t5b5c-snapshot.log`. Header and row-title span allocation is
T5b5d.

T5b5d1a closure: each configured Canvas column header now contributes its
pre-font-source, post-font hard/preferred width as one canonical Grid span over
the declared columns. The already-active Grid allocator raises and rechecks
its covered tracks without repacking sparse or trailing slots; header paint
and height still follow the prior band until T5b5d1b. A populated two-column
span beside an empty third slot, shared horizontal-bar headers/ranges, row
titles, root capture, Canvas admission, and strict Clippy pass. Logs:
`/tmp/opencode/t5b5d1a-lint.log`,
`/tmp/opencode/t5b5d1a-tracks-unit.log`,
`/tmp/opencode/t5b5d1a-headers-integration.log`, and
`/tmp/opencode/t5b5d1a-producer.log`.

T5b5d1b closure: measured column-header text now determines one top header
band after column spans receive their widths and before Canvas rows allocate.
Long headers wrap completely instead of being rejected by a fixed single-line
band. Row-header text determines its left nonfill track and contributes its
actual height over the declared row span. Both are frozen once in the active
Grid result and drawn from that result; the separate fixed-height header
placement path was removed. Focused Canvas tracks, multi-line header SVG,
row-header spans, shared headers/ranges, row-title compatibility, exact Canvas
admission, and strict Clippy pass. Logs: `/tmp/opencode/t5b5d1b-lint.log`,
`/tmp/opencode/t5b5d1b-tracks-unit.log`,
`/tmp/opencode/t5b5d1b-wrapped-header.log`, and
`/tmp/opencode/t5b5d1b-admission.log`. T5b5d2 moves row titles into the same
Grid allocation.

T5b5d2 closure: each row title contributes its measured hard/preferred span
across all declared columns, wraps at its assigned width, and reserves an
additional measured top track on its own row above the axis gutters. The
Canvas Grid freezes the text and its placement alongside cells and headers;
the old separate row-title planning and `OwnedInk` feedback workaround were
removed. Wrapped text over a TextPanel, two-Plot spanning titles, a separate
axis-gutter reserve, root theme ownership, row-title census/lifecycle, Canvas
admission, Canvas unit consumers, and strict Clippy pass. This closes T5b5;
T5b6 integrates the remaining shared axes and guides.

Integrate measured tracks with Plot/TextPanel/empty cells, spanning headers and
row titles. Replace the Canvas parent adapter and `DeferredT5bPanelHeadingInputs`
with native inputs. Embedded Plot title/subtitle/caption use the common producer
within the panel's local post-inset span. Local captions contribute to row
requirements and remain independent of the Canvas caption. Reuse T5b3's TextPanel
producer: a fill surface with top-left nonfill content; extra size does not center
that content. Do not rebuild its intrinsic or assigned-width measurement route.

Resolve title as Plot instruction > Canvas panel instruction > resolved theme;
subtitle as Plot instruction > Canvas panel instruction > optional theme subtitle
> final title. Canvas root headings, row titles, strips, TextPanels, and captions
do not inherit the panel instruction. Builders remain private until T6.

Evidence: mixed Nature/CHI/Publication/Science panels, explicit exceptions,
subtitle-only content, all alignments, simultaneous local/Canvas captions,
mandatory blank lines, RTL physical alignment, narrow wrapping, exact-height
refusal, and exactly-once content/layers. Existing exact Plot dimensions do not
become panel preferences. Preserve current validation of invalid child settings;
do not use this layout migration to change validation timing or admit nesting.

### T5b6 — Shared Axes, Guide Frames, And Multi-Owner Recovery

T5b6a closure: a shared policy with no measured discrete or continuous guide
now allocates its cells directly, without opening eight fictitious guide
choices or consuming the shared guide trial budget. Genuine shared guides keep
the same first-fit order. The containing frame's minimum cell footprint uses
the Canvas Grid's resolved gap rather than duplicating a literal. No-guide
trial observations, a real shared-guide recovery, and strict Clippy pass.
T5b6b integrates the retained shared domains and guide frames; T5b6c records
multi-owner recovery before closing T5b6.

T5b6b closure: the pre-font strict shared-x union now installs on each scoped,
semantically attached Canvas child once per root round, before any shared-guide
body probe or native Grid allocation. Later guide trials reuse that prepared
range; the grouped-child reapplication occurs only when root feedback expands
the union. The existing single `AssignedGuideFrame` still owns shared discrete
and continuous guide placement. Focused shared-x and Forest log-domain unit
tests, mixed horizontal-bar shared ranges, per-Plot/hidden/shared discrete
guides, shared continuous Heatmap output, Canvas admission, and strict Clippy
pass. Multi-owner trial/replay evidence is T5b6c.

T5b6c closure: a real two-Plot `PerPlot` Canvas at 480 px records canonical
owners 0 then 1, initial OutsideRight choices, and two changed Grid-cell
gutters that trigger a root retry. On the second round, each owner rejects its
own OutsideRight trial and selects OutsideTop without revisiting its sibling's
choice. A separate three-round two-guide fixture compares cached/uncached
exact rendered output: a changed owner takes a new panel measurement while an
unchanged neighbor reuses its selected plan and replays feedback. A kernel
two-owner trace confirms eight rejected attempts per owner are independent,
exhaustion is a planning-budget error rather than a geometric-fit verdict,
and the new root round starts fresh. Strict Clippy and these focused tests pass.
This closes T5b6; T5b7 records the native-composition closure matrix.

Complete shared-x installation and shared/per-Plot/hidden guide integration over
the native tracks. Use the prepared strict union, never override explicit limits
or admit categorical members. Reuse the T5b1 descriptor, T5b2 guide producer, and
one frame-fit path. Consume T5a's retained union directly; remove any remaining
adapter-only range reapplication and duplicated outside-gap reservation without
recreating the already-removed renderer-probe preparation. All native parent
chrome is frozen before capture.

Require a concrete multi-owner trace on supported shapes: canonical owner order,
initial current choices, ancestor constraint changes, provisional gutter miss,
root-round retry, repeated cache hit versus new trial, and budget exhaustion.
No selected-sibling backtracking or combinations of guide choices. A budget stop
is not proof of geometric nonfit. Shared policy suppresses panel guides; do not
invent a fixture with simultaneous shared and per-panel guides forbidden by that
policy. Use PerPlot for genuinely multiple panel-guide owners.

### T5b7 — Composition Closure

Native-parent route census (25 Plot kinds, one fixed root driver per operation):

| Parent / selected child | Track and text owner | Guide owner and fitted frame |
| --- | --- | --- |
| Plot root with an ordinary child (20 kinds) | Native interior intrinsics, `RootFramePlan`, and selected `PanelPlan` | Plot-scoped prepared entries, `Legend::measure_assigned`, `AssignedGuideFrame` |
| Plot root with a specialized child (Heatmap, Marginal, Matrix, Beeswarm, Waterfall) | Root frame and retained specialized intrinsics/selected payloads; final specialized mark packing remains T5e | Kind-owned guides, with prepared continuous Heatmap/Beeswarm descriptors under Canvas sharing |
| Generic facets of an admitted Plot | `orchestrator/facets/tracks.rs` measured row/column Grid and strip/axis plans; no equal-cell parent allocator or later layout-only child probe | One prepared facet guide descriptor, one enclosing `AssignedGuideFrame`, suppressed child guides |
| Canvas Plot cell | `canvas/tracks.rs` measured Grid: cell minima, actual gutters, spanning column/row headers and measured row-title tracks; complete Plot-local text/body | `PerPlot` owns each separate child frame, `Shared` owns one Canvas guide frame, `Hidden` has no guide; empty shared guide has no fictitious trials |
| Canvas TextPanel or empty declared cell | Same measured Canvas tracks, with top-left assigned-width TextPanel flow; empty coordinates and gaps remain declared | No cell guide |

The current support boundary remains closed: ordinary Plot kinds and Heatmap,
Beeswarm, and Waterfall may be Canvas children or generic facet sources subject
to each kind's pre-existing validation; Marginal may be a Canvas child but not
generically faceted; Matrix is self-managed and may be neither. A faceted Plot
cannot be nested in Canvas, and Canvas cannot be inserted into a Plot cell.
`CompositionChildPolicy::validate_child` is solely this pre-font support gate,
not a retained parent layout adapter. The old equal-cell parent functions and
deferred panel-heading representation are absent; the specialized *interior*
parity minimum remains named separately until T5c/T5e.

Closure evidence: the 25-family lifecycle fixture renders all 24 admitted
Canvas child kinds in bounded four-cell groups, rejects Matrix before fonts,
and compares cached/uncached SVG bytes. Focused mixed TextPanel, sparse and
spanning headers, root/local captions, wrapped text, exact dimensions,
unsupported nesting, resource refusal, immutable replay, and separate
shared/hidden/per-Plot guide tests pass. The 66 composition integrations and
13 visual-regression snapshots pass. The T5b7 broad gate ran `just verify`:
1,070 library tests passed; two stale phase-probe-count assertions failed
after deletion of duplicate layout-only child passes. Those two assertions
were updated to the measured native path and pass individually, along with
their neighboring facet/cache, 25-family, composition, and snapshot tests;
strict Clippy passes. Documentation and package gates were completed
separately rather than repeating the full test suite for these localized
assertions. The release populated-guide before/after table and retained
historical baseline appear in `docs/local-search-benchmarks.md`.

Publish the native-parent route census, owner/track/guide matrix, support matrix,
cache evidence, and before/after populated-guide measurements. Both parent
adapters and deferred panel-heading storage must be gone. Shared compatibility
has one implementation; T5c/T5d provide new final styles/scales through it.
T5b does not require final mark density or specialized packing. Tests cover every
admitted child family, mixed TextPanels, exact roots, sparse/spanning topology,
required local captions, resource refusal, and immutable output reuse.

## T5c: Native Mark And Annotation Requirements

**Entry:** T5b complete. The data-window table above is approved. This stage must
not become an excuse to change statistical definitions, condition order, or exact
caller styles while centralizing their ownership.

| Step | Status | Depends on | Logical commit |
| --- | --- | --- | --- |
| T5c0 | COMPLETE | T5b | `docs(marks): classify appearance and content policies` |
| T5c1 | COMPLETE | T5c0 | Umbrella; complete after T5c1a-b |
| T5c1a | COMPLETE | T5c0 | `feat(theme): separate interval and summary strokes` |
| T5c1b | COMPLETE | T5c1a | Umbrella; complete after private roles and contrast |
| T5c1b1 | COMPLETE | T5c1a | `refactor(theme): retire unused line width` |
| T5c1b2 | COMPLETE | T5c1b1 | Umbrella; complete after bracket and remaining roles |
| T5c1b2a | COMPLETE | T5c1b1 | `feat(theme): resolve significance bracket stroke` |
| T5c1b2b | COMPLETE | T5c1b2a | Umbrella; complete after point-label spacing and remaining roles |
| T5c1b2b1 | COMPLETE | T5c1b2a | `feat(theme): measure point-label gap from annotation role` |
| T5c1b2b2 | COMPLETE | T5c1b2b1 | Umbrella; complete after path and remaining style roles |
| T5c1b2b2a | COMPLETE | T5c1b2b1 | `feat(theme): resolve data-colored path strokes` |
| T5c1b2b2b | COMPLETE | T5c1b2b2a | Umbrella; complete after Histogram strokes and remaining roles |
| T5c1b2b2b1 | COMPLETE | T5c1b2b2a | `feat(theme): resolve Histogram overlay strokes` |
| T5c1b2b2b2 | COMPLETE | T5c1b2b2b1 | Umbrella; complete after Density and remaining roles |
| T5c1b2b2b2a | COMPLETE | T5c1b2b2b1 | `feat(theme): resolve Density curve strokes` |
| T5c1b2b2b2b | COMPLETE | T5c1b2b2b2a | Umbrella; complete after Regression and remaining roles |
| T5c1b2b2b2b1 | COMPLETE | T5c1b2b2b2a | `feat(theme): resolve regression fit strokes` |
| T5c1b2b2b2b2 | COMPLETE | T5c1b2b2b2b1 | Umbrella; complete after reference strokes and remaining roles |
| T5c1b2b2b2b2a | COMPLETE | T5c1b2b2b2b1 | `feat(theme): resolve estimate and reference strokes` |
| T5c1b2b2b2b2b | COMPLETE | T5c1b2b2b2b2a | Umbrella; complete after RGBA conversion and remaining roles |
| T5c1b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2a | `docs(theme): pin legacy alpha resolution` |
| T5c1b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b1 | Umbrella; complete after complete-fill transport and remaining roles |
| T5c1b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b1 | `feat(render): resolve independent fill and stroke paints` |
| T5c1b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2a | Umbrella; complete after point paint and remaining roles |
| T5c1b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2a | `feat(render): resolve independent point and outline paints` |
| T5c1b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b1 | Umbrella; complete after primary-point appearance and remaining roles |
| T5c1b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b1 | `feat(theme): resolve primary point appearance` |
| T5c1b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2a | Umbrella; complete after composited contrast and remaining roles |
| T5c1b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2a | `feat(theme): select complete Heatmap contrast roles` |
| T5c1b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b1 | Umbrella; complete after bar fill and remaining roles |
| T5c1b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b1 | `feat(theme): resolve ordinary bar fill appearance` |
| T5c1b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2a | Umbrella for remaining private mark and annotation roles |
| T5c1b2b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2b2a | `feat(theme): resolve area and uncertainty-band fill appearance` |
| T5c1b2b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b2b1 | Umbrella for remaining mark and annotation owners |
| T5c1b2b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b2b1 | `feat(theme): resolve distribution and histogram body fills` |
| T5c1b2b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2b2a | Umbrella for remaining point, Heatmap and annotation roles |
| T5c1b2b2b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2b2b2a | `feat(theme): resolve raw distribution overlay points` |
| T5c1b2b2b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b1 | Umbrella for remaining mark and annotation roles |
| T5c1b2b2b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b2b2b1 | `feat(theme): resolve Heatmap cell and missing-key outlines` |
| T5c1b2b2b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2a | Umbrella for remaining mark/annotation appearance |
| T5c1b2b2b2b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2a | `feat(theme): resolve remaining structural mark strokes` |
| T5c1b2b2b2b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b1 | Umbrella for remaining style roles |
| T5c1b2b2b2b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b1 | `feat(theme): resolve primary point sizes by mark family` |
| T5c1b2b2b2b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2a | Umbrella for remaining private style roles |
| T5c1b2b2b2b2b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2a | `feat(theme): resolve Interaction connector appearance` |
| T5c1b2b2b2b2b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b1 | Umbrella for remaining annotation roles |
| T5c1b2b2b2b2b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b1 | `feat(theme): resolve required estimate detail text` |
| T5c1b2b2b2b2b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b2a | Umbrella for remaining annotation styles |
| T5c1b2b2b2b2b2b2b2b2b2b2b2b1 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b2a | `feat(theme): resolve Waterfall total annotation text` |
| T5c1b2b2b2b2b2b2b2b2b2b2b2b2 | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b1 | Umbrella for remaining annotation style ownership |
| T5c1b2b2b2b2b2b2b2b2b2b2b2b2a | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b2b1 | `feat(theme): resolve supplemental point-label typography` |
| T5c1b2b2b2b2b2b2b2b2b2b2b2b2b | COMPLETE | T5c1b2b2b2b2b2b2b2b2b2b2b2b2a | `feat(theme): close private mark and annotation roles` |
| T5c2a | COMPLETE | T5c1 | Standard-coordinate mark input, hard/soft ink and feedback closure |
| T5c2a0 | COMPLETE | T5c1 | `docs(marks): map standard prepared mark inputs` |
| T5c2a1 | COMPLETE | T5c2a0 | Umbrella for standard mark hard/preferred/ink requirements |
| T5c2a1a | COMPLETE | T5c2a0 | `feat(marks): measure Scatter primary-point hard ink` |
| T5c2a1b | COMPLETE | T5c2a1a | Umbrella for remaining point/path/reference requirements |
| T5c2a1b1 | COMPLETE | T5c2a1a | `feat(marks): measure Qq and Regression observation hard ink` |
| T5c2a1b2 | COMPLETE | T5c2a1b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2a | COMPLETE | T5c2a1b1 | `feat(marks): measure Line and Trajectory path hard ink` |
| T5c2a1b2b | COMPLETE | T5c2a1b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b1 | COMPLETE | T5c2a1b2a | `feat(marks): measure Qq and Regression reference hard ink` |
| T5c2a1b2b2 | COMPLETE | T5c2a1b2b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2a | COMPLETE | T5c2a1b2b1 | `feat(marks): measure Density group-curve hard ink` |
| T5c2a1b2b2b | COMPLETE | T5c2a1b2b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b1 | COMPLETE | T5c2a1b2b2a | `feat(marks): measure Histogram bin and overlay hard ink` |
| T5c2a1b2b2b2 | COMPLETE | T5c2a1b2b2b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2a | COMPLETE | T5c2a1b2b2b1 | `feat(marks): clear automatic Scatter point ink through root feedback` |
| T5c2a1b2b2b2b | COMPLETE | T5c2a1b2b2b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b1 | COMPLETE | T5c2a1b2b2b2a | `feat(marks): clear path and Qq ink through root feedback` |
| T5c2a1b2b2b2b2 | COMPLETE | T5c2a1b2b2b2b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2a | COMPLETE | T5c2a1b2b2b2b1 | `feat(marks): clear Regression observations through root feedback` |
| T5c2a1b2b2b2b2b | COMPLETE | T5c2a1b2b2b2b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2b1 | COMPLETE | T5c2a1b2b2b2b2a | `feat(marks): clear Density curves through root feedback` |
| T5c2a1b2b2b2b2b2 | COMPLETE | T5c2a1b2b2b2b2b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2b2a | COMPLETE | T5c2a1b2b2b2b2b1 | `feat(marks): clear Histogram bins and overlays through root feedback` |
| T5c2a1b2b2b2b2b2b | COMPLETE | T5c2a1b2b2b2b2b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2b2b1 | COMPLETE | T5c2a1b2b2b2b2b2a | `feat(marks): measure Regression band and fit edge ink` |
| T5c2a1b2b2b2b2b2b2 | COMPLETE | T5c2a1b2b2b2b2b2b1 | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2b2b2a | COMPLETE | T5c2a1b2b2b2b2b2b1 | `feat(marks): retain complete Density area and Line ribbon ink` |
| T5c2a1b2b2b2b2b2b2b | COMPLETE | T5c2a1b2b2b2b2b2b2a | Umbrella for remaining standard mark requirements |
| T5c2a1b2b2b2b2b2b2b1 | COMPLETE | T5c2a1b2b2b2b2b2b2a | `feat(marks): constrain pre-selection area and band hard ink` |
| T5c2a1b2b2b2b2b2b2b2 | COMPLETE | T5c2a1b2b2b2b2b2b2b1 | `feat(marks): retain standard-panel density preferences` |
| T5c2b | COMPLETE | T5c2a | Categorical estimate, Dumbbell and Bar hard/soft requirements |
| T5c2b0 | COMPLETE | T5c2a | `docs(marks): map categorical geometry and slot owners` |
| T5c2b1 | COMPLETE | T5c2b0 | Umbrella for categorical estimate and bar requirements |
| T5c2b1a | COMPLETE | T5c2b0 | `feat(marks): measure categorical estimate hard pitch` |
| T5c2b1b | COMPLETE | T5c2b1a | Umbrella for remaining categorical requirements |
| T5c2b1b1 | COMPLETE | T5c2b1a | `feat(marks): measure grouped Dumbbell slot ink` |
| T5c2b1b2 | COMPLETE | T5c2b1b1 | `feat(marks): measure Bar category and interval requirements` |
| T5c2b1b2a | COMPLETE | T5c2b1b2 | `feat(marks): measure attached estimate label row height` |
| T5c2b1b3 | COMPLETE | T5c2b1b2a | `feat(marks): retain categorical natural preferences` |
| T5c2c | COMPLETE | T5c2b | Categorical distribution/Interaction mark requirements closed |
| T5c2c0 | COMPLETE | T5c2b | `docs(marks): map collective distribution slot and ink inputs` |
| T5c2c1 | COMPLETE | T5c2c0 | Umbrella for categorical distribution and Interaction requirements |
| T5c2c1a | COMPLETE | T5c2c0 | `feat(marks): measure sparse Boxplot body and raw-point pitch` |
| T5c2c1b | COMPLETE | T5c2c1a | Umbrella for remaining collective requirements |
| T5c2c1b1 | COMPLETE | T5c2c1a | `feat(marks): retain complete distribution paint under root feedback` |
| T5c2c1b2 | COMPLETE | T5c2c1b1 | Umbrella for remaining distribution slots and sparse Interaction |
| T5c2c1b2a | COMPLETE | T5c2c1b1 | `feat(marks): measure sampled and supplied cloud category pitch` |
| T5c2c1b2b | COMPLETE | T5c2c1b2a | `feat(marks): measure sparse Interaction and collective preference` |
| T5c2d | COMPLETE | T5c2c | Umbrella for specialized mark requirements |
| T5c2d0 | COMPLETE | T5c2c | `docs(marks): map specialized mark requirement owners` |
| T5c2d1 | COMPLETE | T5c2d0 | Umbrella for specialized mark requirements |
| T5c2d1a | COMPLETE | T5c2d0 | `feat(heatmap): measure styled cell and missing-cross hard ink` |
| T5c2d1b | COMPLETE | T5c2d1a | Umbrella for remaining specialized mark requirements |
| T5c2d1b1 | COMPLETE | T5c2d1a | `feat(beeswarm): measure occupied marker category ink` |
| T5c2d1b2 | COMPLETE | T5c2d1b1 | Umbrella for remaining specialized mark requirements |
| T5c2d1b2a | COMPLETE | T5c2d1b1 | `feat(waterfall): measure signed step category ink` |
| T5c2d1b2b | COMPLETE | T5c2d1b2a | `refactor(marks): measure Marginal and Matrix mark requirements` |
| T5c3 | COMPLETE | T5c2d | Native annotation and overlay consequences |
| T5c3a | COMPLETE | T5c2d | Umbrella for typed significance targets and native shelves |
| T5c3a1 | COMPLETE | T5c2d | `refactor(annotations): resolve grouped significance by typed targets` |
| T5c3a2 | COMPLETE | T5c3a1 | Umbrella for significance shelves and painted containment |
| T5c3a2a | COMPLETE | T5c3a1 | `feat(annotations): pack significance labels by occupied ink` |
| T5c3a2b | COMPLETE | T5c3a2a | `refactor(annotations): fit complete significance ink and exact windows` |
| T5c3b | COMPLETE | T5c3a | Umbrella for remaining native overlays and text |
| T5c3b1 | COMPLETE | T5c3a | `fix(annotations): fit visible supplemental point labels` |
| T5c3b2 | COMPLETE | T5c3b1 | Umbrella for panel text and inside overlays |
| T5c3b2a | COMPLETE | T5c3b1 | `refactor(annotations): retain canonical panel text ink` |
| T5c3b2b | COMPLETE | T5c3b2a | `refactor(annotations): reuse frozen text for inside overlay obstacles` |
| T5c3b3 | COMPLETE | T5c3b2 | Umbrella for required value and cell text |
| T5c3b3a | COMPLETE | T5c3b2 | `feat(annotations): contain Heatmap cell text ink` |
| T5c3b3b | COMPLETE | T5c3b3a | Umbrella for attached and specialized value text |
| T5c3b3b1 | COMPLETE | T5c3b3a | `feat(annotations): keep Waterfall totals at exact source anchors` |
| T5c3b3b2 | COMPLETE | T5c3b3b1 | Umbrella for attached Bar and Estimate value text |
| T5c3b3b2a | COMPLETE | T5c3b3b1 | `feat(annotations): fit attached Bar glyph ink` |
| T5c3b3b2b | COMPLETE | T5c3b3b2a | `refactor(annotations): fit Estimate value text` |
| T5c4 | COMPLETE | T5c3 | Atomic deferred-consequence retirement |
| T5c4a | COMPLETE | T5c3 | `refactor(layout): remove deferred panel floors and consequences` |
| T5c5 | COMPLETE | T5c4 | `test(theme): close native mark and annotation ownership` |

### T5c0 — Property And Content Disposition Table

Map each active family constant and override from the T2h relationships in `04`
to a semantic theme role, specialized component default, exact caller style, or
renderer invariant. Include sampled/supplied Histogram and Raincloud, point-layer
accent slots, grouped-bar significance spans, uncertainty, missing rows, and
raw supplemental overlays. Distinguish already-retained geometry from remaining
legacy requirements. Attach the data-window case table to the concrete consumers.

Define the prepared-primary-mark count for every family before using the standard
density preference: distinguish semantic marks from raw input rows, curve sample
vertices, supplemental marks, and final visible primitives. Coincidence, alpha,
and explicit-window clipping must not silently reduce that semantic count. This
count is not a replacement for the independently owned structural-address census.

List every clipping/elision/omission branch, its caller-policy key if any, and the
target disposition. Preserve admitted explicit policies rather than inventing a
universal ban or a new omission feature. Any actual unresolved behavior change
requires user approval before the affected implementation starts.

T5c0 closure: `docs/t5c0-property-disposition.md` assigns the reconciled T2h
source clusters to `MarkTheme`, `AnnotationTheme`, `TypographyTheme`, T5d
data-color mapping, exact local style, or a fixed renderer invariant. It defines
the selected source-backed primary-mark count for all 25 families (including
sampled/supplied Histogram and Raincloud, sparse and missing slots) separately
from raw rows, tessellation, overlays and structural addresses. Its concrete
data-window, caller-elision, renderer-generated visibility, and required-text
table preserves the approved clipping policy and names the intended correction
branches without making a new omission API. This documentation-only stage
requires diff review, not tests; T5c1 is the first implementation consumer.

### T5c1 — Complete Private Style Inputs

Legacy exact opacity resolution decision (user delegated implementation timing):
T5c converts each validated `f64` point/fill opacity once using nearest-even
rounding of `opacity * 255` to an RGBA `u8` alpha. The old positive/finite
caller validation remains at its existing boundary. Exact alpha replaces its
paint's complete alpha and never multiplies a theme default; fill and stroke
remain independent. This is an intentional T5c SVG/PNG change for fractions
not exactly representable in 8-bit alpha. T6 exposes the replacement
public `Color` controls without introducing another internal representation.
Inspect all affected snapshots and update them with their source changes.

T5c1a closure: the legacy `ResolvedTheme::interval_width` field is gone. One
complete private uncertainty stroke now supplies vertical/horizontal bar stems
and caps, while an independent complete summary stroke supplies Boxplot and
Violin whiskers and Boxplot's originating legend line. Both presets preserve
their existing exact paint/width/pattern; neither borrows `AxisTheme` at the
consumer. Resolved-theme equality includes both roles for child cache identity.
A private custom-value test varies both independently of frame stroke; focused
bar/facet/Canvas, sampled/supplied distribution, SVG snapshot, and strict Clippy
checks pass. No alternate style assembly or old interval-width alias remains.
T5c1b completes the remaining mark/annotation styles and contrast policy.

T5c1b1 closure: the unused `ResolvedTheme::line_width` and its unused preset
constructor input are removed, not repurposed into a second line-style default.
Resolved-theme equality, all four preset fixtures, strict Clippy, and focused
built-in theme checks remain valid. T5c1b2 resolves the actually consumed
line, point, fill, specialized mark, and annotation roles.

T5c1b2a closure: `ResolvedTheme::bracket_width` is removed. One complete
`AnnotationAppearance::bracket_stroke` now supplies the same independent RGBA
paint and finite width to final significance drawing and both bar/distribution
geometry observations; label typography and exact caller gap remain separate.
The new role enters resolved-theme equality. A custom private stroke changes
the annotation identity without changing axis, mark, or label roles; focused
significance, mixed bar/facet/Canvas, Boxplot bracket, SVG snapshot, and strict
Clippy checks pass. T5c3 will use this already-resolved paint to close full
painted stroke clearance before freezing annotation geometry.

T5c1b2b1 closure: the supplemental line point-label offset now consumes the
private `AnnotationAppearance::point_label_gap`, never a renderer-local `2.0`.
It is identity-bearing in the resolved theme, with independent private values.
A selected Line payload test varies that gap and observes text move by exactly
the requested difference while its points/style stay fixed; focused line-label,
strict Clippy, and rendered composition neighbors pass. T5c1b2b2 handles the
remaining text appearance, mark paints and automatic contrast roles.

T5c1b2b2a closure: one complete private `DataLineAppearance` consumes an opaque
mapped color and its own alpha/width/pattern for a Line series (1.8 px built-in)
or Trajectory/LinePath (1.4 px built-in). Both originating legend entries and
selected path payloads use the same resolved role; the duplicate local width
switch is removed. A custom-width/alpha Line fixture changes its retained mark
and exact guide glyph identity without moving its points; point/path, faceted
guide, visual snapshot, and strict Clippy checks pass.

T5c1b2b2b1 closure: sampled and supplied Histogram routes now consume the
same resolved, independently customizable KDE-curve and rug-tick stroke roles.
The old curve/rug literals are gone; bin fill and scientific bin/KDE sources
remain separately owned. Preset paint/width/pattern match the old literals;
focused sample/supplied integration tests, custom-role independence and strict
Clippy pass. T5c1b2b2b2 continues remaining mark/annotation appearance.

T5c1b2b2b2a closure: Density curves and their guide glyphs now consume the
same resolved data-colored stroke (opaque mapped color plus role alpha, width,
and pattern) rather than two literal `1.6` assemblies. A custom alpha/width
fixture retains scientific curve segments while changing final painted stroke
and rejecting a conflicting guide glyph; focused Density, facet/Canvas
preparation and explicit-window tests plus strict Clippy pass.

T5c1b2b2b2b1 closure: Regression fit paths and exact originating guide glyphs
now consume one complete resolved neutral stroke rather than reconstructing
foreground/`1.5` twice. A custom paint/width fixture retains scientific fit
endpoints, changes the frozen mark and mismatches the original guide glyph;
focused Regression data/facet/Canvas and layout tests plus strict Clippy pass.
The distinct observation and optional uncertainty-band roles remain in
T5c1b2b2b2b2.

T5c1b2b2b2b2a closure: estimate interval, solid PointInterval reference,
dashed Coefficient/Forest reference, Q-Q reference, and Dumbbell connector now
consume five independent complete mark strokes. This removes their renderer-
local paint/width/pattern assemblies without sharing unrelated semantic roles.
Custom private roles remain separate from axes/bar uncertainty. Focused Log10
shared estimates, projected Dumbbell geometry, Q-Q integration, estimate SVG
snapshots, and strict Clippy pass.

T5c1b2b2b2b2b2a closure: the private polygon/rounded-container `FillStyle`
now carries independent complete `Rgba` fill and stroke paints plus one finite
stroke width. The old RGB+float-opacity carrier, legend opacity round trip and
duplicate SVG alpha writer were removed; canonical primitive storage, feedback
identity and GeometryBackend use the same final paints, including invisible
fill/stroke exclusion. Preset fractional fills resolve to 8-bit alpha once by
the approved nearest-even rule; `0.18/0.22/0.25/0.40/0.45/0.70/0.82` become
`46/56/64/102/115/178/209` respectively. Twelve inspected visual-regression
snapshots change only paint-alpha attributes; Waterfall, Beeswarm and Marginal
source fingerprints retain their prior values as historical evidence. Focused
backend/feedback, all composition, Heatmap, representative SVG/PNG and
sample/supplied distribution callers, current snapshots and strict Clippy pass.
T5c1b2b2b2b2b2b resolves the family-specific fill roles before T5c2 measures
their complete ink.

T5c1b2b2b2b2b2b1 closure: the sole private `PointStyle`/`PointOutline` now
stores complete independent `Rgba` paints. Exact legacy point-layer opacity
is validated and quantized once, replacing the point fill alpha without
altering the outline alpha; the float-opacity carrier and duplicated SVG
writer are removed. Canonical mark storage, feedback glyph identity,
GeometryBackend's invisible-ink rules, legend compatibility and SVG/PNG
consume that same final paint. Seventeen inspected SVG snapshots change only
point `fill-opacity` from 0.82 to 209/255 or the equivalent exact layer
value. Focused point/path/geometry/backend tests, 66 composition, 67
distribution, 37 Heatmap, 41 line tests after localized old-literal assertion
updates, output SVG/PNG and strict Clippy pass. T5c1b2b2b2b2b2b2 resolves
the remaining semantic point-role defaults and specialized fills.

T5c1b2b2b2b2b2b2a closure: all Plot primary points now resolve fill alpha,
shape and independent outline paint/width from one private `MarkAppearance`
role; their existing family-specific radii remain consumer inputs pending
separate occupancy/ink measurement. The sole caller outline instruction is
stored as visibility, so hiding the outline cannot be undone by theme
resolution. Repository guide and selected-mark callers were migrated together
and the obsolete `PointStyle::with_outline` construction path was deleted.
Custom-theme Scatter evidence confirms prepared guide equality, changed
point/outline paints and persistent hidden visibility; focused point/path,
Canvas shared guides, composition, current snapshots and strict Clippy pass.

Contrast primitive correction: CORE-MATH is a source project, not a Cargo
package; its public site links the binary64 `pow.c` implementation. Since
`TypographyTheme` only linearizes `u8` channels, the approved implementation
uses one first-party 256-entry binary64 table instead of porting a general
power routine into the standalone safe renderer. All table bits and the
binary64 preprocessing order are pinned and tested independently. This
changes the *method*, not the contrast selection or fallback policy; no
runtime network, native FFI, or platform `pow` enters the renderer.

T5c1b2b2b2b2b2b2b1 closure: one checked-in 256-channel binary64 table now
selects complete `TypographyTheme` light/dark candidates after linear-light
composition of cell fill, panel, owning Canvas root and the opaque
`AnnotationTheme` planning backdrop. Exact-bit contrast ties choose dark.
Heatmap finite-cell and fully-null-row requirements census *only* the selected
stack before font preparation; intrinsic widths/heights and selected payloads
use that same complete role, replacing renderer-local luminance threshold and
paint-only substitution. Explicitly themed Canvas Plot children consume their
actual Canvas root surface. Table SHA-256 and all 256 ordered values, three
high-precision reference evaluations (120/240/480 decimal digits), translucent
surfaces, invalid candidates, differing candidate stacks/sizes, selected-only
font discovery, Heatmap/facet/Canvas annotation paths, and strict Clippy pass.
T5d4 reuses this owner when it supplies final variable fills from resolved
continuous scales. The remaining MarkTheme fill/point sizes and annotation
styles stay in T5c1b2b2b2b2b2b2b2.

T5c1b2b2b2b2b2b2b2a closure: all four ordinary bar families and their
originating fill guide glyphs consume one private `DataFillAppearance` with
independent fill alpha, outline RGBA and width. Current presets retain the
same opaque data color and foreground outline. Custom private values change
the exact retained mark and glyph without changing scientific polygons;
focused standalone/facet/Canvas bar preparation, populated guides, current
SVG snapshots and strict Clippy pass. Remaining fill roles are separate
semantic owners in T5c1b2b2b2b2b2b2b2b.

T5c1b2b2b2b2b2b2b2b1 closure: line and regression uncertainty bands, density
areas, Marginal KDE strips, and Matrix diagonal KDE cells resolve separate
private fill-alpha and outline roles before selecting the marks and their
originating guide glyphs. Outlines can follow the data color or carry an
independent theme RGBA paint. A custom regression band changes both selected
mark and exact glyph while preserving the prepared polygon; five affected
SVG snapshot paths and strict Clippy pass with default preset bytes intact.

T5c1b2b2b2b2b2b2b2b2a closure: boxplot, sampled Violin/Raincloud and supplied
Raincloud, Histogram, and Waterfall bodies use distinct fill/outline roles.
The inner Raincloud box has a fixed white fill role and an independent outline;
current presets retain exactly the previous paint bytes. A custom Violin role
changes selected polygons and their originating legend glyph without changing
scientific positions. Focused supplied/sample parity, Histogram rendering,
Waterfall fingerprints, three SVG snapshot paths, and strict Clippy pass.

T5c1b2b2b2b2b2b2b2b2b1 closure: Boxplot and Violin raw points, sampled and
supplied Raincloud strips, and Raincloud guide points share one private point
role and radius. The obsolete production backend point constructor is gone;
backend-only fixture support remains test-only. A custom Raincloud role
changes both the selected point and originating glyph, retaining the same
source-point positions. Focused raw-overlay/guide render and strict Clippy pass.

T5c1b2b2b2b2b2b2b2b2b2a closure: ordinary, highlighted and missing-row
Heatmap cell outlines, the missing-key swatch outline, and missing-cell cross
are independently private appearance roles. Custom RGBA strokes and widths
change selected paints without moving cells or changing their fill colors.
Focused missing-cell, merged-row, deterministic SVG and strict Clippy checks
pass. Variable cell/missing fill palette ownership remains with the T5d color
scale cut because continuous guide descriptors are root-prepared before theme
selection; keep prepared value/domain semantics independent of appearance.

T5c1b2b2b2b2b2b2b2b2b2b1 closure: HorizontalBar zero, Waterfall connector
and reference, sampled/supplied Raincloud median, Beeswarm reference and
Interaction interval strokes consume separate complete private roles. Current
presets preserve paint, width and dash grammar. A custom HorizontalBar zero
stroke retains its exact geometry; Interaction unit/capture and SVG snapshots,
Waterfall fingerprints and strict Clippy pass. Interaction's two marker-visible
fingerprints were stale after the earlier RGBA point-alpha change: old
`66849c3c47bac743/178ec0a9046a6eb6`, current
`44fc64272cd500e1/6e44598a586b59bc`. The marker-hidden hash
`bae40bd86df8a981` remains identical, confirming no interval/connector
stroke change under current presets.

T5c1b2b2b2b2b2b2b2b2b2b2a closure: the existing family-specific primary-point
radii for Scatter, Line, Trajectory/LinePath, Regression, Q-Q, Marginal, Matrix,
estimate displays, Dumbbell, Interaction, Beeswarm and Boxplot fliers now enter
from private `MarkAppearance::point_sizes`; selected marks and originating guide
glyphs share one owner per family. A custom Scatter size changes the selected
point and its exact guide while retaining positions. Strict Clippy, a pinned
Interaction marker fingerprint and seven representative SVG snapshot paths pass
with existing preset sizes. T5c2 consumes these actual sizes for native ink and
occupancy constraints rather than reverting to fixed renderer assumptions.

T5c1b2b2b2b2b2b2b2b2b2b2b1 closure: Interaction connectors and originating
guide glyphs now share theme-owned data-color width and alpha; exact per-group
Solid/Dashed caller grammar overrides only the theme pattern. Custom appearance
changes retained marks and guide identity without moving data points or
overriding exact group patterns. Focused style/preflight/capture tests, current
Interaction SVG snapshot, pinned projection hashes and strict Clippy pass.

T5c1b2b2b2b2b2b2b2b2b2b2b2a closure: required estimate detail text, including
`Not estimable`, has one complete theme-owned typography role. Prepared census
selects that stack before font discovery; selected PointInterval/Coefficient/
Forest text plans use the same paint, size, weight and line height while an
exact caller significance-label override stays independent. A private size/
paint change alters the selected text but not the retained interval; focused
status/facet tests, estimate SVG snapshots and strict Clippy pass.

T5c1b2b2b2b2b2b2b2b2b2b2b2b1 closure: Waterfall's start/end total labels
use a complete private typography role in census and selected annotation
measurement. Changing private text size/paint changes the final measured
annotation shelf and hence the projected y positions, preserving scientific
spans and x positions. Focused custom-role, annotation/capture/resource tests,
current Waterfall SVG fingerprints and strict Clippy pass.

T5c1b2b2b2b2b2b2b2b2b2b2b2b2a closure: supplemental Line point-label font
stack, size, weight and line pitch now come from a complete private annotation
role in census and selected text planning. The exact layer paint overrides
only the color after role selection, preserving caller precedence. A custom
role changes the label font without moving the underlying point or replacing
its exact color; focused gap/label tests, current path SVG snapshots and strict
Clippy pass.

T5c1 closure: private mark and annotation appearance is resolved before final
paint/ink recording, including independent fill/stroke RGBA, per-family point
size, exact caller precedence, selected-only contrast typography and text
roles for required estimate/Waterfall/point annotations. Mark and originating
guide glyphs consume the same styles; intentional point alpha byte changes
were recorded with their historical fingerprints. Thirty focused theme tests,
all thirteen affected SVG regression snapshots, representative custom-role
mark/guide, sampled/supplied, facet/Canvas and annotation tests, and strict
locked/offline all-target Clippy pass. Native mark/annotation constraints and
deferred-minimum deletion follow in T5c2–4; categorical/continuous value
selection and Heatmap variable fills remain the single T5d scale owner.

Implement complete private mark/annotation roles and exact local-style precedence.
Keep data identity/domain in the scale owner: styles receive one opaque data color
and the owned alpha or one exact RGBA paint. Implement the approved independent
fill/stroke alpha replacement rules internally; no multiplicative fallback.
T5d owns final palette/gradient algorithms. Until then current resolved data colors
feed the same style assembly, without creating a second rendering route.

Resolve actual style before measuring glyph/mark ink. Automatic light/dark text
selection uses the approved complete roles and backdrop before font resolution,
not a late color-only substitution. Preserve the pinned numerical requirements;
validate their evidence/admission before integrating a new mathematical primitive.

Evidence: exact overrides/setter orders, independent fill/stroke alpha, transparent
surfaces/backdrop, candidates with deliberately different fonts/sizes, missing
coverage only in an unselected contrast role, and retained mark/legend glyph
equality. Public theme/style replacement remains T6; private custom-value tests
must exercise more than built-in defaults.

### T5c2a-d — Native Mark Requirements By Cluster

All four steps use the existing prepared scientific results and selected payload
types. Add native hard/preferred requirements and complete painted ink; do not
re-extract statistics or add a second projection. Preserve source nonzero intent,
canonical degeneracy rejection, exact data windows, and explicit caller styles.

| Step | Consumers | Required additional evidence |
| --- | --- | --- |
| T5c2a | Scatter, Line, Trajectory, LinePath, Histogram, Density, Qq, Regression | Full stroked point/path/band ink, prepared-primary-mark count including coincident/transparent marks, representability, reference/uncertainty geometry; no count from only visible clipped primitives |
| T5c2b | PointInterval, Coefficient, Forest, Dumbbell, four bar kinds | Orientation-aware category pitch, unavailable slots, interval/cap clearance, sign/stack extents, adjacent value labels, exact local widths, Forest raw/log source distinctions |
| T5c2c | Boxplot, Violin, Raincloud, Interaction | Sample/supplied parity, summaries/outliers/clouds/raw points, sparse collective slots/connectors, shared group/hue styles, category occupancy without dropping missing combinations |
| T5c2d | Heatmap, Beeswarm, Waterfall, Marginal, Matrix mark consumers | Exact cell/glyph/label and mark requirements delivered to their still-deferred packing owners; no new tangent packing or track algorithm yet |

T5c2a0 input handoff (the exact semantic cardinalities are fixed in
`docs/t5c0-property-disposition.md`):

| Family | Existing retained owner; count before clipping | Additional hard-ink input |
| --- | --- | --- |
| Scatter | `PreparedScatter.groups[*].points.len()`; hue regrouping does not add marks | Resolved point radius, shape, independent fill/outline RGBA, stroked outward ink at auto numeric edges |
| Line/Trajectory/LinePath | `PreparedLine.groups[*].points.len()`; supplemental `layers` are separate | Styled markers only when present; complete path/ribbon stroke and interval boundary across an explicit window; supplemental labels belong to T5c3 |
| Histogram | `HistogramSemantics.bins.len()` for both source kinds | Positive/nonzero source bin geometry, fill outline, optional KDE curve and rug strokes, anchored zero |
| Density | `PreparedDensity.curves.len()`, not grid vertices | KDE support/area outline and stroked curve, including segments clipped into an exact window |
| Qq | `PreparedQq.theoretical.len()` paired with `observed` | Point ink and optional reference line, with visible clipping independent of source count |
| Regression | `PreparedRegression.points.len()`, not `BAND_POINTS` samples | Point ink, complete fit stroke and optional confidence polygon edge |

T5c2a0 boundary: the current axis-only `PreparedAxes.intrinsics` `OnceLock`
checks typography through `StandardPanelInputs::matches`; mark intrinsics
depend additionally on resolved mark style and exact layer visibility. Do not
cache them under that typography-only identity. Admit the semantic count and
any iteration at the source owner; a cache hit still charges its measurement
call. Selected `GeometrySnapshot` supplies final painted bounds but cannot
retroactively determine pre-selection cardinality, natural preference or a
parent's hard assignment. Preserve the single root feedback driver and the
`80x60` compatibility floor until all four native mark clusters and annotation
consequences can replace it atomically in T5c4. Automatic-axis painted edge
clearance may request bounded owner feedback; exact data windows may clip marks
but must not drop a required annotation or mask malformed/collapsed geometry.

T5c2a1a closure: the retained finite Scatter groups supply semantic primary
cardinality before visibility/window filtering, with admitted group-count work.
One origin-relative point, resolved through the actual private mark style,
supplies canonical complete-painted bounds to both parent hard allocation and
selected native-coordinate fit. The same `GeometrySnapshot` code that records
final painted marks computes circle/triangle and independent outline alpha;
its scratch and one record are admitted. Style changes do not enter the
typography-only axis cache and are recomputed on each mark-intrinsic call.
Explicit x/y data windows independently waive only the corresponding point
hard extent. Focused custom-theme, transparent, triangle, exact-window,
resource-cap, child-reuse, Scatter snapshot and strict Clippy checks pass.
Automatic-edge domain clearance, soft density preference and all other
standard-mark families remain in T5c2a1b; the `80x60` floor is not retired.

T5c2a1b1 closure: Qq retains its paired theoretical/observed point count,
Regression its fitted-source observation count. The same admitted
origin-relative canonical point-ink measurement used by Scatter supplies
their independent actual-point hard width/height on automatic axes; exact
windows may clip that axis. Resolved point radius, paint transparency, outline
and shape are read on each call rather than cached under axis typography.
This does not yet substitute for either family's independent fit, reference
or band ink, or for automatic-domain edge clearance. Focused custom-role,
sampled/faceted/Canvas, exact resource-cap and SVG snapshot tests plus strict
Clippy pass. T5c2a1b2 completes the remaining standard-mark responsibilities.

T5c2a1b2a closure: Line, Trajectory and LinePath derive their primary-vertex
cardinality from retained groups before window clipping and charge group
processing. Origin-relative, admitted canonical line and point observations
supply painted width and height according to each family's actual marker
visibility and resolved stroke/point roles. Fully transparent lines contribute
no ink after source geometry and stroke-style validation; an independent
geometry test pins that distinction. Exact data windows waive only the
constrained-axis hard span. Focused Line/Trajectory tests, facet/Canvas
preparation, actual root resource-cap/reuse tests, current path SVG snapshots
and strict Clippy pass. Ribbon boundary ink, supplemental layer consequences,
automatic-edge clearance and soft density remain in T5c2a1b2b/T5c3.

T5c2a1b2b1 closure: Qq's optional reference and Regression's required fit
contribute their independently resolved, canonically quantized line stroke
ink to the same pre-selection hard input as their observation points. A hidden
Qq line is not inspected. Transparent but valid strokes contribute zero ink;
the malformed-source and invalid-width validation remains fatal before that
decision. Custom 140px stroke/transparent roles preserve exact-window
clipping and default SVG bytes. Focused Qq/Regression, representative
distribution snapshots, exact root resource-cap test and strict Clippy pass.

T5c2a1b2b2a closure: each retained Density group curve, not its generated KDE
vertices or input samples, supplies one semantic primary instance. Its
theme-resolved data-colored curve stroke uses the admitted canonical line ink
path for automatic-axis hard requirements; exact axes preserve mark clipping.
Independent custom wide/transparent stroke values recompute without changing
the cached axis typography. Focused grouped/source/limit tests, distribution
SVG snapshots, root exact resource-cap test and strict Clippy pass. Density
area polygon ink and the other standard families remain pending.

T5c2a1b2b2b1 closure: Histogram's retained bins, sampled or supplied, supply
primary mark cardinality without counting KDE grid vertices or rugs. An
admitted unit rectangle runs through its actual canonical mitered polygon
grammar to resolve fill/outline paint's outward hard ink; optional KDE and
rug strokes contribute only when those prepared overlays exist. Explicit
data windows waive the respective dimension. Focused custom transparent/wide
outline and optional-overlay tests, sampled/supplied parity, all focused
Histogram units, current distribution snapshots, exact root resource-cap
checks and strict Clippy pass. Painted-edge domain clearance and natural
density preference remain separately pending in this T5c2a umbrella.

T5c2a1b2b2b2a closure: Scatter measures its actual canonical origin-relative
point bounds, compares projected scientific extrema to the assigned panel and
requests at most one monotonic numeric-domain enlargement per automatic axis.
One root-owned feedback round can carry both X and Y; a subsequent round
remeasures the mark at its new panel rather than accepting a stale painted
extent. The shared axis-span calculation handles facet-local extrema against
an inherited wider numeric range. Only user-declared exact data windows waive
ink clearance: a root feedback limit is a temporary planned range, not a
clipping policy. Non-finite/unrepresentable expansions give a typed fit miss;
resource refusal remains fatal. Source preparation, select, and replay remain
separate. Custom 18px Scatter points, two-axis root convergence, fixed facets,
exact windows, default guide/snapshot bytes, exact resource-cap tests and
strict Clippy pass. Extend this owner to the other standard families in
T5c2a1b2b2b2b.

T5c2a1b2b2b2b1 closure: Line, numeric Trajectory and LinePath contribute
resolved marker and stroked-path origin-relative ink; Qq contributes its
retained paired-observation point ink. Categorical Trajectory does not grow
its condition axis. Only a user-exact numeric data window clips that
dimension; a planned range from the preceding root round remains automatic.
Both axes can grow within one root round; pending child-domain requests
invalidate provisional cache reuse until the next stable assignment. Canonical
source/mark geometry validates before a proposed domain fit miss. Four
inspected path SVG snapshots now move vertices, ticks and grid consistently
inward without altering paint, text or scientific order. Focused Qq exact
window/complete-ink tests, 41 Line integration tests, native facet/Canvas and
cache on/off tests, exact root resource-cap tests and strict Clippy pass.
Band/interval/reference edge geometry, remaining mark families and the soft
density preference remain in T5c2a1b2b2b2b2.

T5c2a1b2b2b2b2b2b2a closure: Line now retains source mark ranges apart from
nice display ranges. Its interval ribbons and Density's prepared KDE areas
submit exact admitted, *unclipped* canonical polygon ink through the same
owner as their previously measured path/curve strokes. Merge requests before
root feedback; neither later mark role can overwrite the earlier one's
clearance. Malformed scientific polygons reject before recoverable ink misses.
Custom 100px Line ribbon and 18px Density area outlines converge with full
painted bounds inside their panels; user-exact X windows still clip marks.
Correcting Line's source-versus-display range restores the four default path
SVG snapshots to their earlier bytes; their intermediate inward projection
recorded in T5c2a1b2b2b2b1 is historical, not current expected output.
Focused Line/Density, exact-window, facet/Canvas, snapshot, exact resource-cap
and child-cache tests plus strict Clippy pass. Pre-selection polygon hard
requirements and soft density preference remain in T5c2a1b2b2b2b2b2b2b.

T5c2a1b2b2b2b2b2b2b1 closure: a configured Line interval ribbon, optional
Regression band and Density area now contribute their complete resolved,
canonically measured fill-outline outsets to the pre-selection hard width and
height of an automatic data axis. When absent/fully transparent they add no
phantom hard ink; exact data windows waive only their own dimension. The
retained polygon's actual mitered shape is still validated and, if needed,
recovered through the existing selected root feedback rather than estimated
from an unrelated rectangle. Focused 100/140px custom outline, facet/Canvas,
source representability, default SVG snapshots, exact operation resource-cap
tests and strict Clippy pass. Soft primary-mark density/standard preferred
dimensions are next; the generic floor retires only in T5c4.

T5c2a1b2b2b2b2b2b2b2 closure: the eight ordinary standard-coordinate
families retain one bottom-up `StandardPanelPreference` on their exact root
frame. The semantic primary count is sourced from prepared groups/vertices,
bins, curves, ordered Qq pairs or Regression observations before clipping;
coincident, transparent and exact-window-hidden marks still count. Renderer-
generated KDE/band vertices, connectors, rugs, reference lines and raw
overlays do not multiply `m`. Actual resolved primary ink and tick-label
typography supply `max(1em, painted mark pitch)`, an integer checked
`ceil_sqrt(max(1,m))` gives density side, and the sealed `0` glyph supplies
`66ch`. Preferred width is `max(hard,66ch,density_side)`; checked spatial
quanta retain preferred height `max(hard_height,ceil_div(3*width,4),density_side)`.
Current exact root dimensions and child allocation remain unchanged; the one
retained value is the T5e private automatic/width-only root input, not a
second sizing route. Four focused cardinality/math/cache-style tests, all
twenty native exact-root defaults, facet/Canvas reuse, current SVG snapshots,
exact operation caps and strict Clippy pass. No generic hard floor is deleted
before T5c4.

T5c2a1b2b2b2b2a closure: Regression retains its admitted observation source
min/max independently of its fitted/band-inclusive display range. Only those
prepared observation extrema and resolved primary-point ink request bounded
automatic X/Y clearance; fitting the band does not masquerade as extra
observations or force point-sized margins at every band extremum. User-exact
axes still clip marks. A custom 54px observation test proves converged root
point containment and bit-exact OLS endpoints after reprojection. Focused
sample/facet/Canvas, exact root resource-cap/reuse, current Regression SVG
snapshots and strict Clippy pass. Fitted-line/band edge ink, other standard
families and soft density remain pending.

T5c2a1b2b2b2b2b1 closure: the full-sample Density support and its admitted
prepared maximum are reused as the sole scientific extrema source for curve
ink; repeated candidate selection no longer rescans KDE vertices for a second
maximum. The resolved stroked curve requests automatic X/Y clearance through
the same root feedback owner; explicit windows still clip data. Current
Density SVG snapshots intentionally reproject the full curve and numeric
ticks by less than one CSS pixel at the prior support/zero edges; paint,
source samples and group order remain unchanged. Focused custom wide stroke,
grouped Density, exact-window, facet/Canvas, exact resource-cap/cache and
strict Clippy checks pass. Area-polygon outline ink and remaining standard
families follow in T5c2a1b2b2b2b2b2.

T5c2a1b2b2b2b2b2a closure: sampled and supplied Histogram retain bin-edge
and complete bar/KDE/rug maximum semantics apart from their initial padded
display ranges. The canonical unit rectangle measures mitered fill/stroke
outsets and optional curve/rug strokes contribute only when present; the
single root driver admits two-axis domain feedback for the exact selected
paint. Custom wide outlines retain every bin inside its final panel. Exact
windows preserve clipping. The inspected native Histogram SVG snapshot moves
bar/KDE/rug geometry and axis ticks by less than one CSS pixel; source bins,
colors and labels remain identical. Focused sample/supplied parity,
precomputed KDE, 25 Histogram units, exact root resource-cap/reuse tests and
strict Clippy pass. Other area/band shapes and soft density remain pending.

T5c2a1b2b2b2b2b2b1 closure: Regression retains a separate full scientific
mark range covering the OLS fit, its observations and the optional prepared
confidence band. After canonical source/selected geometry validation, an
admitted, *unclipped* band polygon plus exact fit stroke supply absolute
painted edges; these are reconciled with existing observation-point domain
requests before the one root feedback driver. Unpainted shapes contribute no
edge ink. A custom 32px mitered band and 54px observations converge with the
whole polygon and every point inside the final panel, while the retained OLS
line projects to exactly the same scientific fit endpoints. Focused custom
styles, facet/Canvas, distribution snapshots, exact resource-cap/cache tests
and strict Clippy pass. Line ribbons, Density areas and the soft density
preference remain in T5c2a1b2b2b2b2b2b2.

T5c2a closure: all eight standard-coordinate families retain their prepared
semantic mark cardinality, complete painted hard/selected ink, exact-window
clipping and the one root feedback path; an admitted checked natural preference
is attached to the native frame for T5e root sizing. `DeferredLegacyPanelMinimum`
is still owned by T5c4, not silently displaced by a soft density target.
Automatic Qq axes at `f64::MAX`/`-f64::MAX` cannot represent a further finite
domain endpoint; the approved `05` automatic-ink rule now returns a scoped
`SpecializedGeometry` fit miss before output publication instead of emitting
the previously clipped glyph. Zero, subnormal and representable `1e300`
constants still render, and an exact caller data window still clips. Focused
Qq numeric/default tests, mark/facet/Canvas/capture and feedback tests, 41
Line integration tests, 13 SVG snapshots, exact root resource caps and strict
Clippy pass. T5c2b proceeds to categorical estimate and bar requirements.

T5c2b0 input handoff: categorical slot pitch is measured on the category axis
only, before assigning the selected numeric projection. Do not infer semantic
marks from visible clipped primitives, or treat the source row count as the
occupied-series count:

| Family/axis | Prepared semantic owner | Hard/pref and status input |
| --- | --- | --- |
| PointInterval/Coefficient/Forest, Y | `PreparedEstimate.labels` holds every row; `estimates[*]` distinguishes available tuples from `Not estimable` slots | Point paint/radius, interval/cap stroke, reference, required unavailable and adjacent label ink. Forest retains raw-vs-Log10 source identity |
| Dumbbell, Y | `DumbbellData.categories`, `group_count` and populated `rows` with category/group indices | Two endpoint paints and connector are one primary pair; `replicate_y` distributes populated series slots within each category; empty combinations do not become marks |
| Four Bar kinds, X or Y | `PreparedBars.categories`, `series`, `bars`, `points` and `significance` | Single/grouped/stacked occupancy includes scientific zero segments and signed stacks. Outline, interval/cap, supplemental point and attached label/annotation ink are separate constraints; missing series positions are not invented |

One category quantum is the minimum pitch. Actual occupied ink produces the
hard slot width/height; `2em` contributes only a preferred pitch. Category
identity and the source-value domain are already root-prepared; full painted
slot ink must join the existing native axis intrinsic and bounded parent
feedback without a local search, another mark projection, or an alternate
`80x60` floor. Retain `DeferredLegacyPanelMinimum` until T5c4 removes it with
the deferred mark and annotation consequences together.

T5c2b1a closure: PointInterval, Coefficient and Forest keep every prepared
category row, including `Not estimable` slots without fabricated numeric
marks. Pre-selection Y pitch combines complete resolved point ink, optional
interval stroke/PointInterval cap height and required unavailable text height;
the X hard stroke/glyph span applies only on an automatic numeric axis. Exact
data windows retain the categorical slot and clip only their numeric paint.
Prepared-row inspection charges work before iteration; sampled nullable,
all-unavailable, Log10 Forest, facet/Canvas, default estimate SVG and exact
resource-cap tests plus strict Clippy pass. Adjacent-label consequences,
Dumbbell occupancy and both Bar orientations follow in T5c2b1b.

T5c2b1b1 closure: Dumbbell's prepared categories, declared group count and
populated `(category,group)` rows supply a per-category hard Y pitch.
`replicate_y`-exact occupied offsets and independently RGBA-painted endpoints
and connector contribute complete slot half-ink; absent group combinations
are not fabricated. The numeric X axis alone requests bounded root domain
clearance when automatic, after selected source/projection validation. User-
exact X windows clip that paint but do not compact category tracks. Grouped
custom 54px endpoints, facet/Canvas preparation, immutable replay, exact
resource caps, default estimate snapshots and strict Clippy pass. The four
Bar orientations and attached label consequences remain in T5c2b1b2/T5c3.

T5c2b1b2 closure: vertical/horizontal and ordinary/stacked bars now share
one categorical center/thickness source for selected marks, exact attached
points and intrinsic occupancy. Each populated grouped/stack segment,
including a scientific zero, contributes its actual slot fraction and
canonically measured independent outline; unused combinations do not create
painted segments. Pre-selection hard pitch also admits nonzero polygon width,
interval cap thickness and exact supplemental point radii. The numeric-axis
painted bodies/caps/points join existing significance requests at their one
root feedback owner, using source extents that include points beyond bar
values; a caller-exact window alone clips that dimension. Focused 100px
grouped/single/stacked/orientation ink, 120px caps, 200px supplemental points,
root zero-edge and large-point clearance, 6 native Bar resource/feedback
tests, populated guides, current bar SVG snapshots and strict Clippy pass.
Natural categorical pitch and cross-axis aspect follow in T5c2b1b3; attached
significance text and collision shelves remain T5c3-owned.

T5c2b1b2a closure: attached Coefficient/Forest estimate-value and significance
labels now contribute the height of their actual prepared text to the same
category-row hard pitch. The complete estimate or effective exact caller
significance typography is resolved before the font metric query, with each
retained source inspected under admitted visits. Numeric-side placement,
collision and source-owned annotation shelves remain T5c3. Focused custom
48pt point-only labels, significance/Log10 Forest, nullable status, default
estimate SVG and exact resource-cap tests plus strict Clippy pass.

T5c2b1b3 closure: the eight categorical Estimate, Dumbbell and Bar families
retain one kind-tagged native panel preference beside the standard summary.
Every declared category slot is preserved; the semantic primary count excludes
unavailable estimate tuples, empty grouped-series combinations, duplicated
endpoint glyphs and connector/cap primitives. Checked spatial quanta retain
`hard_pitch=max(1 quantum, actual occupied slot pitch)`,
`preferred_pitch=max(hard_pitch,2em)`, the full slot span and the soft `4:3`
cross-axis target. Internal theme outline changes update the retained pitch
without altering an exact root panel. Six focused preference/math tests, all
twenty native exact-root cases, facet/Canvas reuse, current Bar/estimate SVG
snapshots, exact resource caps and strict Clippy pass. T5c2c will fill the
remaining four categorical distribution/Interaction cases on this same carrier.

T5c2b closure: pre-selection category pitch, available/unavailable tuple
identity, signed/zero bars, sparse group slots, interval/cap clearance and
complete numeric-axis mark ink all use retained semantics and the one bounded
root feedback path. Explicit numeric windows clip paint without removing the
categorical slot. Required attached-label height is already reflected in
the hard row; final label placement and collision shelves remain T5c3.

T5c2c0 input handoff: one shared category axis and the established private
categorical preference carrier serve four remaining ordinary kinds without a
new statistics or projection route:

| Consumer | Retained owner/primary count | Additional actual painted slot input |
| --- | --- | --- |
| Boxplot | `PreparedBoxplot.sample.groups`/`stats`, one summary per populated group | Quartile box, median/whiskers/caps, fliers or exact raw overlay; sparse category/group combinations do not create summaries |
| Violin | `PreparedViolin.sample.groups`/KDE, one body per group | Mirrored KDE half-width, inner summary/caps and optional exact raw points, not the 64 generated KDE vertices |
| Raincloud sampled/supplied | `PreparedViolin.sample.groups` or `PreparedRaincloud.groups`, one cloud/summary per actual group | Half KDE cloud, mandatory raw strip, inner box/median; caller-supplied strip coordinates keep their exact normalized identity |
| Interaction | `PreparedInteraction.conditions`/`traces[*].cells`, one populated factor×trace slot | Per-group exact point shape/pattern, optional connectors, interval strokes and sparse condition occupancy; holes remain empty |

For each group retain its actual `distribution_sample::center` or equivalent
source series slot, validate the declared category boundary and measure
complete painted X mark/overlay extents at the assigned categorical pitch.
Numeric Y ranges inherit already prepared scientific statistics and may grow
only through the one typed root feedback owner; exact windows clip that paint.
`2em` categorical pitch and soft `4:3` aspect are preferences, not an extra
hard floor. Raw-row/source identity and prepared guide entries stay unchanged.

T5c2c1a closure: Boxplot measures each populated category/group summary at
its resolved `distribution_sample::center` and actual box width, preserving
missing combinations without fabricated quartiles. Fill outline, median/
whisker cap stroke, fliers or the exact raw-overlay point style contribute
independent categorical hard pitch; jitter uses each retained source-row
identity. The numeric-axis hard input reads the same complete paint, while a
caller-exact Y window may clip it. A private 100px body outline differs
between sparse grouped and single boxes; a separate 80px flier demonstrates
independent point ink. Focused Boxplot summary and overlay tests, all seven
native categorical-distribution integration paths, current SVG snapshots,
exact resource-cap checks and strict Clippy pass. Selected automatic Y ink,
sampled/supplied Violin/Raincloud and sparse Interaction remain in T5c2c1b.

T5c2c1b1 closure: Boxplot, sampled Violin/Raincloud and supplied Raincloud
share the existing `DistributionPayload` owner for one additional admitted
unclipped canonical pass over final polygon, line and raw-point paints. Their
prepared scientific support determines optional automatic Y clearance; exact
Y windows still clip. The single root round reconciles that request with
already owned significance shelves rather than discarding either. Custom
220px Boxplot whisker/body strokes and supplied 120px Raincloud clouds/50px
strips show complete selected numeric ink and resolved no-extra-work replay;
the supplied exact Y window keeps its fixed domain. Seven categorical-
distribution native paths, sampled/supplied parity, raw overlays, exact root
resource-cap/cache checks, current SVG snapshots and strict Clippy pass.
Category-X pitch, sample/supplied parity under custom geometry, and sparse
Interaction follow in T5c2c1b2.

T5c2c1b2a closure: sampled Violin/Raincloud group occupancy and supplied
Raincloud declared groups now measure separate categorical hard pitches from
their actual mirrored/half-cloud, inner box/median/whisker, raw overlay and
strip grammar. Supplying the same resolved point and independent box paints
to strip placement removes the legacy 2px geometry hint: painted point widths
and box outlines must fit between the box and declared strip. Too-narrow
supplied slots give a typed recoverable category-slot or pre-selection fit
miss; an adequate 5000px custom example keeps complete cloud, strip and inner
ink inside the panel. Selected collective payloads reject uncontained X ink
instead of relying on backend clip; caller-exact Y windows still clip data.
The inspected native Raincloud SVG snapshot moves only four strip-point X
coordinates by about 0.05px, retaining all labels and paints. Sample/supplied
parity, 15 Raincloud integration cases, categorical cache/facet/Canvas, exact
candidate resource-cap tests and strict Clippy pass. Interaction and the
shared categorical preference handoff remain in T5c2c1b2b.

T5c2c1b2b closure: Interaction's declared conditions and actually populated
trace cells supply X hard occupancy, complete per-group exact marker shape,
interval/connector stroke and semantic `m` without fabricating sparse holes.
Selected, validated interval/connector/point ink is observed before freezing;
numeric Y uses bounded root feedback only when automatic. A custom 160px
marker test confirms full Y containment and all five sparse cells, without
altering the pinned current Interaction projection fingerprint. The existing
kind-tagged categorical preference now covers Boxplot, sampled/supplied
Violin/Raincloud and Interaction: prepared group/trace counts, category slot
count, `2em` soft pitch and `4:3` aspect join the eight earlier categorical
families. Seven focused preference tests, fourteen Interaction units,
categorical distribution/Canvas/facet and exact resource-cap/reuse tests,
current Interaction and collective SVG snapshots, and strict Clippy pass.

T5c2c closure: sample and supplied group bodies, inner summaries, raw source
strips/fliers, sparse collective intervals and connectors retain their one
scientific-preparation and root-feedback route. Full categorical painted ink
must fit an assigned containing panel; exact Y windows clip numeric data but
do not erase the semantic group or condition slot. Later overlay collision,
significance shelves and source-owned text remain with T5c3, not a second
collective packing pass.

T5c2d0 input handoff: the five remaining specialized interiors keep their
existing prepared semantics and selected payload, while T5e replaces each
family's packing separately. T5c2d supplies *mark requirements* to that one
owner now; it does not introduce local tangent searches or cell-track
alternatives:

| Family | Prepared count/source | T5c2d mark input; later packing owner |
| --- | --- | --- |
| Heatmap | `HeatmapData.x_levels/y_levels`, active declared cells including missing and fully-null merged rows | Cell paint, highlight/cross and selected annotation glyph hard width/height; T5e1 allocates columns then rows |
| Beeswarm | `PreparedBeeswarm.rows` source identity, `levels` and numeric/color extents | Complete point radius/outline and categorical occupancy independent of post-placement bins; T5e4 chooses tangent offsets once |
| Waterfall | `PreparedWaterfall.spans/labels/totals` retain signed source delta | Bar/connector/reference ink and per-category pitch, required total-label sources; T5e5 allocates typed shelves |
| Marginal | `PreparedMarginal.groups` retain central source pairs and linked X/Y KDEs | Central point ink plus top/right density-area strokes and support; T5e3 links native subpanel tracks |
| Matrix | `PreparedMatrixAxes.labels/ranges/groups/diagonals` retain declared variable square | Off-diagonal source points, diagonal per-group density bodies and exact cell-label ink; T5e2 allocates native grid tracks |

Record one immutable hard/soft mark summary per owner using the same resolved
theme, pre-font prepared identities and admitted work ledgers as ordinary
families. A declared data window clips numeric marks only in its dimension;
category/cell topology and required annotations remain. T5c4 can remove the
generic floor only when each specialized consumer has these real bounds.

T5c2d1a closure: Heatmap's root-prepared `x_levels/y_levels`, active layout,
merged fully-null rows, highlighted cells and explicit missing cells now
supply separate source-owned cell stroke and cross minimum pitches before
specialized panel assignment. The missing diagonal's pixel inset expands for
its actual painted stroke instead of retaining the old 4px maximum and
cropping a private custom 24px cross. Only roles actually selected by the
prepared cell topology are assembled; candidate font discovery remains
selected-only. The sole axis intrinsic cache checks complete light/dark
typography, owner-local translucent surfaces/backdrop, selected cell stroke
and final continuous color domain before reuse; mismatched inputs recompute
without stale style reuse. Focused cache/domain/custom 100px cell/24px cross,
preparation/admission, 38 Heatmap integration and font-census tests, plus
strict Clippy pass. T5e1 owns column-first native cell track assignment;
Beeswarm/Waterfall/Marginal/Matrix mark inputs follow in T5c2d1b.

T5c2d1b1 closure: Beeswarm retains all source-backed `rows` and category
levels independently of the current signed-bin visual offsets. Its actual
prepared per-row offset and fully stroked point geometry supply pre-selection
category hard pitch, without running a second swarm pack; X point/reference
width is hard only on an automatic numeric axis. After selected source and
canonical geometry validation, one unclipped mark observation requests
numeric X clearance from the existing root feedback driver. Structural zero
reference endpoints may meet the category frame, while actual swarm points
must fit its assigned category panel. Custom clustered 54/300px source-row
fixtures, exact-X windows and Y occupancy, colorbar routing, candidate resource
refusal, frozen SVG/PNG replay and unchanged Beeswarm snapshot pass with
strict Clippy. T5e4 replaces the bin offsets with the approved deterministic
tangent pass; T5c2d1b2 proceeds to Waterfall/Marginal/Matrix mark inputs.

T5c2d1b2a closure: Waterfall retains every finite signed cumulative span,
including a true zero, and derives category hard pitch from complete positive/
negative bar outlines and connector ink. Prepared base/terminal labels give a
minimal required source-owned annotation row height before the still-deferred
T5e5 shelf pack; caller-exact numeric windows may clip marks, not collapse
their category count. Selected bars and connectors are observed unclipped
before one X-domain feedback request; reference strokes deliberately terminate
at the category frame and contribute numeric, not categorical, clearance.
Custom 80px outlined signed bars and 16px connectors, deterministic replay,
typed annotation failure, eleven focused Waterfall units, current full SVG
fingerprints and strict Clippy pass. T5e5 will replace the shelf algorithm;
Marginal/Matrix mark inputs follow in T5c2d1b2b.

T5c2d1b2b closure: Marginal's root-prepared groups and linked KDE support
retain every central observation; complete primary-point ink supplies central
X/Y hard span, and selected density-area outlines supply independent top/right
strip inputs. The currently fixed 12px gaps and 82px/96px strips remain the
T5e3 packing owner's inputs, not recomputed curves. Selected central point
ink proposes X/Y feedback only for automatic domains, while exact windows
clip that data paint; the root driver reaches a frozen result. Matrix derives
its square variable topology from prepared labels, every off-diagonal group's
point style and diagonal density outline from prepared groups, and actual
unbreakable column/rotated-row glyph widths. Its existing 8px gaps remain
the T5e2 grid owner's inputs; a locally insufficient 40px cell now yields a
scoped geometry fit miss rather than a fatal render error. Focused Marginal
and Matrix units, custom linked-point/density/cell-label ink, exact-window
feedback, Matrix contracts, Marginal SVG fingerprints, and strict Clippy pass.
T5c3 now owns annotation and overlay consequences.

The common standard preference is the exact `05` density/`66ch`/`4:3` rule;
categorical hard/preferred pitch is derived from actual occupied slots. Aspect
and density are preferences, not a new hard minimum or root-size percentage.
Data-window tests include off-domain marks, crossing lines/intervals, exact-edge
strokes, mixed exact/automatic axes, genuine zeros, and malformed/collapsed data.

### T5c3 — Native Annotation And Overlay Consequences

T5c3 ownership census: `docs/t5c3-annotation-owners.md` maps pairwise/attached
significance, Waterfall and cell/value labels, inside guides, panel text and
supplemental layers to their retained source and selected consumer. T5c3a1
removes grouped significance's synthetic category/series strings and transformed
annotation copies: sorted category-cluster/series anchors are selected by the
original `SignificanceTarget` identity, with explicit rendered-series validation
and no fallback to a string key. Both orientations and existing explicit-domain
semantics use the same typed resolution. Focused significance units, Bar and
distribution feedback/facet neighbors, 19 significance integration contracts,
custom within-category series anchors, and strict Clippy pass. T5c3a2 owns the
complete painted-ink shelf update; T5c3b owns remaining overlays.

T5c3a2a closure: bracket shelves now test each prior occupied interval against
the union of the span, centered complete label width (rotated height for
horizontal brackets) and configured bracket-stroke clearance. The first
collision-free level is retained, with each level/interval comparison admitted
under the root work ledger. Step height clears the label, gap and bracket
tick, not an unrelated panel inset. A large-label example whose disjoint
category spans nevertheless overlap receives separate shelves; smaller labels
share one. Sixteen focused significance units, 19 integration significance
contracts, final-domain distribution feedback, unchanged native Bar snapshots,
and strict Clippy pass. T5c3a2b completes absolute selected-ink containment
and exact-domain refusal before T5c3a closes.

T5c3a2b closure: the prepared bracket and retained label now pass canonical
unclipped geometry measurement in the actual containing panel, transposed for
horizontal brackets. The complete stroke and glyph union must fit; failure is
an annotation-scoped overlay-collision miss with the caller's original slot.
The single shelf/domain producer reserves its actual bracket half-stroke
before automatic domain growth, while an insufficient caller-exact numeric
window remains an overlay miss. No later drawing-time clipping or second
layout search repairs the bracket. Focused custom 24px automatic/exact domain
and 200px categorical-stroke refusals, eighteen significance units, nineteen
annotation contracts, unchanged Bar snapshots and strict Clippy pass.
T5c3b follows for remaining overlays and required text.

T5c3b1 closure: the prepared supplemental Line point layer retains its exact
point-visibility gate. For each visible required label, the final resolved text
keeps its projected anchor without clamping it to the canvas origin; its
unclipped canonical glyph ink is measured before the selected data clip.
One source-backed combined label-ink proposal extends only automatic numeric
axes through the root feedback driver. Any unresolved exact-axis overflow is
a typed overlay-collision miss, not a clipped label. Custom automatic-root
and exact-edge cases, off-domain omission, 16 focused Line units, significance
neighbors and strict Clippy pass. T5c3b2 continues inside overlays and panel
text; T5c3b3 closes other required labels.

T5c3b2a closure: panel text now has one complete private font/paint/line role
consumed by both the pre-font text census and the selected producer. The old
size-only measurement helper is removed; the default 14px appearance and
SVG output are unchanged. Its unclipped canonical glyph bounds determine
containing-panel fit and sibling/inside-guide collisions, including when the
same retained text is translated under a parent assignment. Normal-flow
width/height checks remain prior to recording; an oversized required glyph
misses as annotation-scoped text instead of becoming successful by clipping.
Custom resolved-role/glyph and translation tests, eight panel-text integration
contracts, 37 root-parent feedback tests, seven Heatmap text-admission units,
and strict Clippy pass. T5c3b2b audits the remaining inside-overlay obstacle
order; T5c3b3 completes value/cell annotations.

T5c3b2b closure: candidate, explicit-inside and post-allocation guide
collision tests now consume the same selected `PlotRenderResult.panel_text`
placements and canonical ink from T5c3b2a. The root no longer remeasures
caller text under another theme or rebuilds a separate per-candidate list;
translated canonical bounds are used during assigned-frame recovery. The
existing parent-supplied mark/annotation geometry remains the static obstacle
context, and only its root feedback driver can move a resolvable inside guide.
All 37 root-plan units and eight panel-text integration contracts, including
four occupied corners and the outside recovery, pass with strict Clippy.
T5c3b3 completes other value and cell annotation owners.

T5c3b3a closure: Heatmap's selected per-cell value and fully-null merged-row
plans now measure their canonical glyph ink without the data-panel clip before
freezing the same cell payload. Every required glyph must fit its own prepared
cell/merged-row rectangle; otherwise it returns a cell-addressed text fit
miss. The existing explicit Heatmap category elision policy still shapes the
selected text once, and its data cells, missing crosses and colorbar keep
their current paint order. A controlled too-narrow cell and the identical
glyph in a larger cell, 35 focused Heatmap units, 38 integration contracts
and strict Clippy pass. T5c3b3b proceeds to attached Bar/Estimate and
Waterfall value text.

T5c3b3b1 closure: visible Waterfall base/terminal totals remain at the
projected scientific source anchor instead of being horizontally clamped into
the data frame. The still-local shelf producer preserves its selected rows,
and each glyph's uncut ink must fit vertically within its annotation band.
An exact numeric window rejects required text extending past the assigned
panel; an automatic X window proposes combined total-label ink via the
existing root domain driver, merged with complete bar/connector ink. Custom
exact/automatic label and final-source-position tests, twelve Waterfall units,
and updated Publication/Science full SVG fingerprints pass with strict Clippy.
The changed hashes are intentional: label positions and facet/shared-domain
projections respond to the removed clamp. T5e5 later replaces shelf packing,
not the exact-anchor rule.

T5c3b3b2a closure: attached Bar/HorizontalBar/stacked glyphs now use their
retained shaped, placed text to check canonical uncut ink against the
categorical frame before the mark clip. An incompatible numeric exact window
is an annotation-addressed overlay fit miss; on an automatic numeric axis the
same selected glyphs submit an admitted painted-ink domain proposal, merged
with bar/interval/supplemental-point clearance under the sole root driver.
Deliberately oversized attached glyphs in both orientations, 31 Bar units,
native Bar facet/Canvas and exact-window neighbors, 19 annotation contracts,
unchanged Bar family snapshots and strict Clippy pass. T5c3b3b2b follows for
Estimate attached and value text.

T5c3b3b2b closure: PointInterval/Coefficient/Forest retained available and
not-estimable labels now contribute their own uncut canonical glyph ink to
the assigned categorical row and numeric panel before the mark clip. A
caller-exact X window or an unavailable label outside the numeric frame is
a typed value-label/overlay fit miss. If neither attached start nor end fits
an initial automatic window, keep the first declared anchor and propose the
painted X extent through the existing root feedback driver, then reselect
under the final domain. A 42-character estimate suffix demonstrates automatic
recovery and exact-X refusal; 18 focused Estimate units, four native
facet/Canvas/axis neighbors, unchanged estimate SVG snapshots and strict
Clippy pass. T5c3's selected native annotation requirements now precede
T5c4's atomic deferred-boundary retirement.

Separate normal-flow requirements from inside guides, significance, value labels,
panel text, and supplemental layers. Implement the closed anchor order, source-
owned shelves, canonical obstacle order, and painted-ink containment from `05`.
Use retained final targets and glyphs; no post-draw reserve or global optimization.
Declared data clipping does not become permission to discard required text.

Establish one typed resolved-target input for significance/facet ownership. It
consumes current target identities now; T5d supplies final scale/domain identities
through the same input. No string-target fallback or later second resolver.
Evidence: exact-domain conflicts, automatic monotonic domain requests, horizontal
brackets, unavailable targets, all accepted anchors, collision clearance/tangency,
required annotation fit failures, and no retry after resource refusal.

### T5c4 — Atomic Deferred-Consequence Retirement

T5c4 closure: `SelectedMarkPlan`/`SelectedAnnotationPlan` replace the two
deferred ownership boundaries in all twenty-five families and their one
immutable selected scene. The `80x60` parity floor is gone from standalone,
facet, Canvas and specialized child paths: coordinate/text/mark intrinsics
and declared occupied grid tracks now determine hard minima, with one spatial
quantum only when a positive empty track needs representation. Shared guides
reserve derived column/row minima; an under-width facet strip returns a
scoped text fit miss instead of attempting an impossible zero-width font
calculation. A valid `70x58` exact Scatter mark panel now renders, while
oversized point, Heatmap cell and Matrix glyph intrinsics still reject before
paint. Focused nineteen-route coordinate/resource/capture units, five Canvas
track, five root-frame, twelve facet, 37 root-panel, and eighteen Matrix
units, native Plot family snapshots, selected composition contracts, exact
Heatmap refusal, specialized fingerprints and strict Clippy pass. Historical
T5a documents retain their original deferred-state descriptions; T5c5 now
audits all mark/annotation consequences before scale work.

Before this step, verify every row in T5c2a-d and T5c3 provides native requirements.
Then remove `DeferredMarkPlan`/`DeferredAnnotationPlan` as deferred ownership
boundaries and `DeferredLegacyPanelMinimum` together. Retained drawing storage
may remain under its single native owner; do not duplicate or rebuild it merely
to rename a type. All parents and specialized adapters consume actual child hard
and preferred requirements. No hidden `80x60`, sentinel minimum, or fallback path.

Evidence: small valid exact panels previously excluded only by the generic floor,
larger invalid panels rejected by genuine geometry, each family's hard boundary,
all public exact defaults, cache identity changes, and refusal before geometry
construction. T5e may replace packing, not repair an omitted native mark input.

### T5c5 — Mark/Annotation Closure

T5c5 closure: `docs/t5c0-property-disposition.md` now records the final
twenty-five-family count/style/annotation handoff and T5d's unchanged source
identities. Private shape, paint and stroke roles are shared with originating
guides, independent fill/outline byte alpha is asserted in the affected
family contracts, all source-backed hard inputs and attached labels have
automatic/exact-axis outcomes, and the sole root driver owns domain feedback.
The Marginal linked KDE scales by its actual finite positive peak rather than
a `1e-12` density substitute; unrepresentable strips refuse before capture.
Numeric midpoint glyphs used by automatic ordinary axes and the 66ch
preference are admitted in the same pre-font census as exact mark/label text,
so retiring the floor cannot allocate an unrenderably narrow Canvas cell.
Historically floor-dependent trace counts, RGBA expectations and intentional
Raincloud/Interaction/Beeswarm/collective fingerprints were inspected and
repinned with their source; no additional geometry owner was added.

Closure verification: the strict all-target Clippy, rustdoc and distributable
package checks pass. The broad `just verify` gate's library run passed 1,163
units; its one remaining Beeswarm mixed-Canvas fingerprint was repinned and
rechecked. The annotations, categorical, composition, distributions,
estimates, Heatmap, interactions, layout, lines, Marginal/Matrix, output,
rendering, statistics and visual-regression integration targets were then run
individually and passed without repeating the already-passing library suite.
T5d0's mapping/consumer census is next.

Reconcile every appearance constant and local override in the T5c ledger portion.
Check all 25 families and every annotation/layer route, private custom styles,
complete-ink geometry, exact clipping policy, mark/legend equality, and resource
scaling. Keep only the five specialized layout adapters; no deferred generic
mark/annotation requirement remains. Record the final values T5d must consume.

## T5d: Resolved Data Scales And Final Guide Semantics

**Entry:** T5c complete. Reuse private RGBA/style ownership and T5b's one descriptor
and compatibility path. This stage changes data mapping, not non-data style
ownership, statistical estimates, public extension points, or layout strategy.

| Step | Status | Depends on | Logical commit |
| --- | --- | --- | --- |
| T5d0 | COMPLETE | T5c | `docs(scales): map semantic domains and migration consumers` |
| T5d1 | COMPLETE | T5d0 | Deterministic scale arithmetic |
| T5d1a | COMPLETE | T5d0 | `feat(color): pin first-party OKLab conversion and channel quantization` |
| T5d1b | COMPLETE | T5d1a | `feat(color): pin gamut interpolation and palette candidates` |
| T5d2 | COMPLETE | T5d1 | Umbrella for categorical maps, palettes and accents |
| T5d2a | COMPLETE | T5d1 | `feat(scales): prepare raw-key maps and bounded accents` |
| T5d2b | COMPLETE | T5d2a | `refactor(scales): migrate ordinary qualitative consumers` |
| T5d2c | COMPLETE | T5d2b | Umbrella for collective and specialized color owners |
| T5d2c1 | COMPLETE | T5d2b | `refactor(scales): resolve Interaction trace colors by raw identity` |
| T5d2c2 | COMPLETE | T5d2c1 | `refactor(scales): resolve distribution group colors` |
| T5d2c3 | COMPLETE | T5d2c2 | Umbrella for Marginal and Matrix group colors |
| T5d2c3a | COMPLETE | T5d2c2 | `refactor(scales): resolve Marginal group colors` |
| T5d2c3b | COMPLETE | T5d2c3a | `refactor(scales): resolve Matrix cell colors` |
| T5d2d | COMPLETE | T5d2c | `refactor(scales): close palettes, accents and guide equality` |
| T5d3 | COMPLETE | T5d2 | Umbrella for active sequential and diverging owners |
| T5d3a | COMPLETE | T5d2 | `feat(scales): resolve Beeswarm sequential colors and guide` |
| T5d3b | COMPLETE | T5d3a | Umbrella for Heatmap scales and shared guide stops |
| T5d3b1 | COMPLETE | T5d3a | `feat(scales): resolve Heatmap cell and local-guide colors` |
| T5d3b2 | COMPLETE | T5d3b1 | `refactor(scales): share resolved continuous stops across compositions` |
| T5d3b3 | COMPLETE | T5d3b2 | `feat(scales): align continuous guide raster with OKLab marks` |
| T5d4 | COMPLETE | T5d3 | Umbrella for final missing/contrast/target owners |
| T5d4a | COMPLETE | T5d3 | `refactor(scales): resolve missing Heatmap paint from owner theme` |
| T5d4b | COMPLETE | T5d4a | `test(theme): verify final mapped contrast and guide cache inputs` |
| T5d4c | COMPLETE | T5d4b | `test(scales): verify final typed annotation targets` |
| T5d5 | IN PROGRESS | T5d4 | Umbrella for private custom gradients and scale closure |
| T5d5a | IN PROGRESS | T5d4 | Umbrella for active private gradient replacements |
| T5d5a1 | COMPLETE | T5d4 | `feat(scales): resolve private Heatmap gradient inputs` |
| T5d5a2 | PENDING | T5d5a1 | `feat(scales): coordinate Beeswarm and Canvas gradient owners` |
| T5d5b | PENDING | T5d5a | `test(scales): close mapping and guide consistency` |

### T5d0 — Domain And Consumer Census

T5d0 closure: `docs/t5d0-scale-consumer-census.md` enumerates the prepared
raw keys, family marks/guides, unkeyed accents, continuous domains, missing
sections, exact caller controls, current modulo/encoded-RGB helpers, their
single deletion owners and the T5b shared descriptor/typed target handoff.
Fixtures pin domain growth, sparse facets, hidden local guides, mixed themes,
same-color exact mappings and unequal exact/automatic color domains. T5d1
pins the first-party arithmetic before any consumer migrates.

Map every categorical field, positioned series, unkeyed primary/accent consumer,
continuous field, missing-value section, guide contributor, and significance
target to its prepared identity and current override API. Specify the private
replacement and deletion owner for every existing palette/gradient helper.
Pin fixtures for domain growth, sparse facets, hidden legends with visible marks,
same-color exact mappings, mixed themes, and unequal exact/automatic domains.

### T5d1 — Pinned Color Arithmetic

T5d1a closure: `src/theme/color.rs` converts opaque byte channels using the
already-audited 256-channel T5c1 sRGB-linear table and canonical OKLab
coefficients with explicit left-to-right binary64 order. Its cube root
binary-searches the ordered positive binary64 domain and resolves the final
neighbor/tie by exact `BigUint` rational cubes, with visits and bounded scratch
admitted to the operation ledger before work; neither platform libm nor an
approximate fallback is used. The inverse matrix emits linear channels only
after finite/gamut validation. Encoded-sRGB byte rounding uses 255 checked-in
half-channel linear thresholds, independently calculated with 120/300/480
decimal digits and SHA-256
`ace388c2947682af22573df8b12dde52e54c10fa706965d3011b159a5a7e340f`.
The finite coefficient-roundtrip boundary slack is a fixed `2^-22`, and
genuinely out-of-gamut results refuse until T5d1b chroma reduction is ready.
All 256 greys and three 256-channel primaries round-trip, published primary
OKLab vectors and independent cube-root hard cases agree, byte-boundary ties
choose nearest-even, exact resource refusal remains fatal, and strict Clippy
passes. T5d1b fixes gamut, interpolation, candidate order and termination
before active consumers migrate.

T5d1b closure: OKLab interpolation preserves exact caller endpoint bytes,
uses one left-associative binary64 Cartesian interpolation for interior
fractions and one fixed 24-step chroma bisection only when inverse linear RGB
lies outside its pinned coefficient slack. A rational square-ring hue sequence,
fixed ordered lightness/chroma bands, finite 8,192-candidate cap, and exact
maximin distance/tie by candidate ordinal yield deterministic opaque palette
extension without a host trig/power route or silent index cycling. Generated
bytes are checked against previously selected colors; exhausted candidates
reject instead of silently repeating a seed. Per-candidate/neighbor visits,
BigUint scratch and retained palette records are admitted to one root ledger.
Independently high-precision OKLab midpoint examples pin red/blue, red/green
and black/white results; custom out-of-gamut reduction, exact tie order,
prefix stability under increasing requests, finite-fraction refusal, all
256 sRGB channel roundtrips and strict Clippy pass. T5d2 now wires keyed
categorical and unkeyed accent consumers to this single arithmetic engine.

Implement the approved first-party OKLab/OKLCH conversion, gamut reduction,
interpolation, and deterministic candidate/tie order for prefix-stable extension.
Freeze the finite candidate construction, operation order, termination and resource
admission before wiring consumers. Verify reference vectors independently of the
implementation under test, including gamut boundaries, exact ties, alpha-free
data color, and all supported opaque channel values used by contrast.

No platform-libm substitute or approximate fallback may silently replace a pinned
numerical contract. If the prescribed primitive cannot be implemented within the
standalone/offline/safe and budgeted boundary, present the concrete alternatives
and obtain approval for a specification change before using one. Do not ship a
dead optional numerical route or claim correctness from a few visual snapshots.

### T5d2 — Categorical And Accent Mapping

T5d2a closure: `src/theme/scale.rs` prepares one immutable ordered raw-key
map or accent-slot sequence from the T5d1 engine. An exact caller map checks
each *used* normalized identity, admits unused and same-color keys, never
falls back for a missing key, and does not demand capacity from an overridden
fixed palette. Fixed capacity rejects before drawing; extendable seeds retain
their prefix, yield deterministic noncycling additional bytes, and a layer's
exact paint does not eliminate its declared accent slot. All source-key
comparisons, palette records and storage are admitted to the operation
ledger. The existing private `CategoricalColors::color` lookup also charges
its actual preceding comparisons while its remaining callers migrate.
Focused mapping/fixed/extended/exact tests, Interaction and sampled Raincloud
caller-color contracts, and strict Clippy pass. T5d2b–d install this one
representation into all prepared family/guide owners and remove the old
modulo and one-off helper routes together.

T5d2b closure: Scatter, Line/Trajectory/LinePath, Density and all four Bar
orientations now select exact raw hue/series/category keys from one resolved
private `CategoryColors` per selected producer, reusing it for complete mark
and originating guide styles. Prepared parent hue order is used for facets
and Canvas even when a local panel lacks an identity; unkeyed primary marks
use accent slot zero. A keyed path consumes no accent, so its first
supplemental `PointLayer` now consumes accent slot zero independent of hue
cardinality. Exact local layer colors retain their occupied slots, while an
exact missing used-key map fails before fonts for Scatter, Line, Density and
positioned Bars. The first built-in seed colors preserve existing output;
above-seed domains extend without `% theme.colors.len()` cycling. Fourteen
Scatter, seventeen Line, thirteen Density, eleven Bar extraction and six
native Bar feedback units, visible/hidden Canvas hue coordination, the
intentional point-layer accent change, native path/Bar snapshots and strict
Clippy pass. Collective and specialized owners follow in T5d2c.

T5d2c1 closure: Interaction's prepared trace hue (or group when there is no
hue) now indexes one exact raw-key mapping shared by marker, connector,
interval and originating legend glyph. The complete parent hue/group domain
precedes sparse facet selection, while a missing used caller key refuses
before fonts. The former `Trace.color_index` field and render-time
`categorical_color` lookup are removed; caller line pattern and point shape
remain independent. Fourteen focused Interaction units, twenty-three
integration contracts including crossed group/hue, exact caller colors,
mixed-theme Canvas compatibility, unchanged ordinary SVG fingerprints and
strict Clippy pass. T5d2c2 migrates collective distributions.

T5d2c2 closure: Boxplot, sampled Violin/Raincloud and supplied Raincloud
now resolve hue/positioned-series color by one raw prepared key for body,
raw observation/mandatory strip and originating guide. Without series, the
category is the color identity; faceted categories are resolved from their
complete unsliced source order before applying a local panel's marks.
An exact caller map with an unused/repeated-color key is admitted, and a
missing *used* series fails before font discovery, including for Boxplot.
The old render-time categorical fallback and two obsolete precomputed
color-index fields are gone; scientific KDE/box/strip geometry and source-row
identity are retained. Focused eleven distribution extraction and three
supplied-cloud units, sampled/supplied Raincloud contracts, faceted sparse
series, unchanged native distribution snapshots, and strict Clippy pass.
Marginal/Matrix complete this umbrella in T5d2c3.

T5d2c3a closure: Marginal's prepared hue domain now has one raw-key color
resolution for central points, both linked density polygons and the complete
originating guide. A group-only Marginal consumes unkeyed accent zero rather
than fabricating hue keys. Parent hue order is consulted before local
selection, while unpainted intrinsic point-ink measurement is independent of
which opaque RGB is mapped. The old per-group palette index/modulo and
candidate-time hue-index fallback are gone; a missing used exact hue refuses
before fonts. Thirteen Marginal units, full SVG fingerprints, exact/automatic
edge and resource contracts, and strict Clippy pass. Matrix's coordinated
variable square follows in T5d2c3b.

T5d2c3b closure: Matrix's root-prepared normalized hue domain now resolves
one raw-key mapping and unkeyed accent zero for all N×N cells. The same
selected group colors feed off-diagonal point styles, diagonal density
polygons and the self-managed legend; variable labels and separate group
identities never invent hue slots. The obsolete per-group palette index and
modulo are removed, and a missing exact *used* hue refuses during semantic
preparation before fonts. Nineteen focused Matrix units, eight Matrix
integration contracts, self-managed clip snapshots, exact resource refusal
and strict Clippy pass. T5d2d reconciles the remaining unkeyed accent
consumers and verifies guide/map equality before continuous mapping.

T5d2d closure: Histogram, Qq, Regression observations/band and guide,
Estimate markers, and Waterfall sign roles now use admitted unkeyed accent
slots; Waterfall extends a one-color seed without cycling. Its unpainted
intrinsic uses a color-independent fill/outline ink role. Density's unused
per-curve hue index and the last `Plot::hue_color_index` fallback are gone;
Scatter guide order uses admitted prepared-parent raw-key positions. A signed
HorizontalBar replaces its misleading series-color guide with typed
nondecreasing/decreasing entries drawn from the very same resolved accent
slots and body style, suppressing hue-color entries and treating zero as
nondecreasing. The exact unused series-color map does not override sign
paint. The sign-title and present-label census runs before fonts, and the
shared Canvas merges sign identities rather than series/display aliases.
The inspected `native_horizontal_bar` snapshot records the two sign swatches
and the narrower panel/fewer automatic numeric ticks required by its wider
guide; the other selected Bar snapshots retain their outputs. Focused
Scatter, Density, Bar, Regression, Histogram, Qq, Estimate and Waterfall
units; categorical grouping and Waterfall fingerprints; representative
path, estimate, signed-Bar, Scatter and clipping snapshots; and strict
locked/offline all-target Clippy pass. T5d2 now has one prepared
categorical/accent representation; T5d3 is the continuous mapping owner.

Implement fixed/extendable palettes, exact raw-key maps, capacity/coverage checks,
and nonempty extendable accent sequences under one deterministic color engine.
Facets resolve the parent domain before slicing; Canvas shared domains use
row-major visible semantic contributors. Display aliases do not recolor data.
An exact local point-layer paint still consumes its documented accent position.

Replace mixed categorical/accent-slot use across all families. Preserve supplied
prefixes and exact caller colors, allow unused keys/repeated colors, reject fixed
capacity and missing used-key errors before fonts, and never cycle silently.
Evidence includes increasing domain prefixes, absent identities, hidden legend
contributors, independent group/hue identities, and all family-specific exceptions
justified by a named semantic owner rather than implementation history.

### T5d3 — Continuous Mapping

T5d3 has two dependency-ordered substages. T5d3a installs a validated private
ordered-stop OKLab sequential scale directly in prepared Beeswarm marks and
its local/shared guide inputs; its existing RGB interpolation and separate
endpoint selection are deleted in that commit. T5d3b1 extends that same
resolved scale to Heatmap's sequential and diverging branches, including the
exact center, symmetric/constant/explicit domains, value-label contrast and
local guide stops; remove its RGB helper and duplicate domain/stop derivation
in place. T5d3b2 passes complete resolved stops through the existing shared
descriptor and selected guide, preserving missing style ownership and
exact mark/stop agreement at endpoints and the semantic center. T5d3b3
closes the remaining interpolation boundary: SVG/PNG gradients currently
interpolate in encoded RGB *between* stops, while data marks use OKLab. Pin
bounded, resource-admitted guide sampling at observed representable values,
including the local specialized colorbars and the final shared-domain guide;
verify decoded pixel and retained-stop agreement without an unbudgeted
per-mark SVG repair or silently conflating distinct colors. These commits keep the
current public scale controls, native guide descriptor, and T5e packing boundary;
T5d3 closes only after the final shared-domain mark/guide agreement passes.

T5d3a closure: the prepared continuous color scale (now `ContinuousColors`)
retains admitted ordered opaque stops and a
finite numeric domain; exact endpoints, saturation, constant midpoint,
opposite-extreme normalization, and the checked multi-stop position all use
the pinned T5d1 OKLab conversion. Beeswarm prepares one scale before fonts,
uses it for every prepared mark, local gradient and shared-domain recoloring,
and no longer has a separate RGB `continuous_color` route. A constant local
or all-constant Shared Canvas guide resolves both endpoint swatches to the
same exact mark midpoint; contributor compatibility continues to compare
its original semantic gradient before final-domain guide selection. Invalid
stops/domains and exhausted resource work/storage refuse; exact source-work
limits and zero work in selected capture still pass. Ten Beeswarm extraction/
fit units, scale boundary/resource tests, shared-domain mark-color and
constant-guide tests, all ten Publication/Science parent fingerprints, the
unchanged clip snapshot and strict all-target Clippy pass. Fingerprint
lengths remain unchanged; interiors and constant gradient paint account for
the inspected byte changes. T5d3b now migrates Heatmap and closes the shared
continuous mapping handoff.

T5d3b1 closure: Heatmap prepares one ordered-stop scale before fonts from its
existing finite/symmetric numeric domain and semantic center; each cell's
opaque RGB is selected and admitted there. The same retained fills supply
selected polygons and value-label contrast during census, intrinsics, and
assigned measurement, including a changed shared domain. Local guide stops
derive from that scale, with exact neutral on either center boundary, an
interior center offset computed by the *same* normalization as cell colors,
and a two-stop midpoint for a constant domain. Explicit color limits now
allow a nondecreasing constant domain: Sequential selects its midpoint,
Diverging requires the center and selects neutral, and every finite source
value saturates to that single color. Missing cells/merged rows stay on
their separate mark style. The old `heatmap_color`, RGB `interpolate`,
`colorbar_gradient_stops`, and second normalization helper are deleted.
Thirty-seven focused Heatmap units, 39 Heatmap integrations, mixed Canvas
guide and Beeswarm fingerprints, the regular specialized-text contract,
three interleaved ignored release specialized-text samples (including
rejected Elide), and strict all-target Clippy pass. The selected source
work/capture and intrinsic-cache revalidation tests remain green. T5d3b2
transfers the full resolved stop sequence to the one shared descriptor.

T5d3b2 closure: the existing Canvas descriptor retains an admitted ordered
stop sequence instead of a `Palette` triple. Heatmap and Beeswarm provide
complete semantic stops before fonts; exact stop/color mismatch rejects, while
Heatmap's diverging neutral uses the one final merged domain to determine its
offset. Constant Sequential/Diverging descriptors freeze two midpoint/neutral
stops, including exact constant limits; endpoint centers collapse to the
relevant two-stop branch. The assigned shared guide copies these already
selected stops, uses the same center offset for its label, and Canvas feedback
observes every offset and RGB channel. Tests cover multi-stop comparison,
unequal exact/automatic diverging domains, constant Heatmap and Beeswarm,
mixed-guide incompatibility before fonts, checked resource refusal, real
cache-on/off Canvas SVG equality, local/Canvas Heatmap integration, unchanged
Beeswarm fingerprints/snapshot, and strict Clippy. An exact constant Heatmap
also matches its retained SVG stops and decoded PNG cell pixel. Interior
between-stop raster agreement is specifically T5d3b3; T5d3 remains open.

T5d3b3 closure: `ContinuousColors::guide_stops` now admits and sorts every
observed finite numeric mark value against its resolved semantic stops under
the backend's canonical six-decimal offset spelling. At each representable
observed interior value, the frozen SVG gradient has the exact first-party
OKLab mark RGB; duplicate positions with equal paint coalesce, whereas
distinct paint at one serialized position refuses fatally before fonts or
capture. Constant scales retain only their two midpoint/neutral stops.
Heatmap and Beeswarm root preparation freezes their local sampled colorbars
before fonts, refreshing them when a shared domain is installed; assigned
measurement copies those stops without color work. The existing shared Canvas
descriptor unions admitted visible child values and retains the same
final-domain stops before guide measurement. No alternate gradient producer
remains. Unit tests pin sampled midpoint, exact resource
limits and conflicting collapsed colors; a real shared Beeswarm verifies
identical observed RGB against its guide stop. An interior Heatmap cell,
sampled SVG stop and decoded PNG cell/gradient-center pixels agree exactly;
the constant-domain pixel contract remains green. Updated and inspected
Beeswarm parent fingerprints grow only by the new interior `<stop>` elements;
its clip snapshot is unchanged. Focused Heatmap/Beeswarm/Canvas/descriptor,
composition and regular specialized-text contracts, three interleaved
ignored release specialized-text samples (including rejected Elide), and
strict all-target Clippy pass. T5d3's prepared marks, local guides and
shared guide now agree at their finite observed value anchors; T5d4 owns
final typography/contrast and annotation-target coordination.

Implement ordered opaque stops, deterministic sequential/diverging interpolation,
exact semantic centers, symmetric automatic diverging domains where representable,
exact caller domains, and specified constant-domain behavior. Missing data maps
through the non-data missing-mark style from T5c, not a gradient endpoint.
Preserve the existing explicit color-domain saturation policy identified by
`04`: finite values beyond a configured color domain use its endpoint colors.
This is color mapping, not geometric data-window clipping or an invented fit
clamp. Shared-domain containment still follows the separate exact/automatic
compatibility rule in `03`.
Heatmap and Beeswarm marks/guides consume one resolved scale; remove hardcoded
gradient logic as its last consumer migrates.

Evidence: endpoints and center exactness, one-sided/constant/extreme domains,
invalid centers/stops, missing values, resource refusal, and exact color agreement
between prepared marks, retained gradient stops, and decoded output pixels.

### T5d4 — Final Shared Descriptors, Contrast, And Targets

T5d4 is split before implementation. T5d4a makes the existing Plot-owned
`MarkAppearance` supply the non-data Heatmap missing fill to cells, fully-null
rows and local/shared missing keys; shared descriptors compare the complete
RGBA fill/outline from each contributor under its actual resolved Plot theme.
T5d4b pins final mapped-cell contrast selection under transparent surfaces
and unequal light/dark font metrics before the census, including cache
identity and exact override precedence. T5d4c verifies typed final targets
across the prepared scale/facet/Canvas owners, deleting a duplicate route if
the focused evidence finds one. All three reuse T5b's descriptor and T5c's
mark/annotation contracts; there is no new scale or annotation fallback.

T5d4a closure: one private `MarkAppearance::heatmap_missing_fill` supplies
complete RGBA to ordinary missing cells, fully-null merged rows and local
missing-key swatches; missing-row text census/intrinsics use that same fill
for T5c composited contrast, and the Heatmap intrinsic cache includes it.
The shared continuous descriptor receives the actual resolved owning Plot
theme for each contributor before fonts. Its missing-key equality now
compares complete RGBA fill and owning outline paints rather than ignoring
alpha; distinct alpha with equal RGB rejects pre-font. Custom translucent
missing paint changes the selected cell, row and guide together without
recoloring finite data stops. The former hardcoded missing fill/stroke and
`heatmap::missing_fill` helper are removed. Focused Heatmap/descriptor/
Canvas units, 40 Heatmap integrations, shared Canvas composition, regular
and ignored release specialized-text contracts (three interleaved samples,
including the rejected Elide workload), and strict Clippy pass with unchanged
built-in SVG/output fingerprints. T5d4b now pins the variable-fill contrast
and cache owner under unequal font and transparent surface inputs.

T5d4b closure: the retained Heatmap fills supply the *only* value-label
contrast candidates to the pre-font census. Replacing an exact local color
domain with a broader final shared domain reselects the complete light/dark
text stack before fonts; an unavailable unselected face remains absent from
font preparation. A fully translucent missing fill over translucent Plot and
Canvas surfaces chooses the light or dark role from the opaque planning
backdrop, with unequal 39px/9px font sizes; no backdrop paint is added. A
changed missing fill or backdrop invalidates the Heatmap intrinsic cache and
remeasures the long merged-row text at its newly selected width rather than
replaying the prior role's metrics. Existing exact local mark-style precedence,
shared diverging descriptor/cache-on/off equality, two focused census units,
the actual unselected-font operation, two neighboring intrinsic-cache tests
and strict Clippy pass. T5d4c verifies final typed annotation targets.

T5d4c closure: Bar significance still binds the original typed
category/series source target after the T5d raw-key color map and separate
display aliases are resolved. Two used series may intentionally share exact
RGB without merging their semantic anchors. In a faceted Bar only the
specified right cell owns the pairwise bracket; a selected display alias is
not accepted as a raw attached target and rejects before font discovery.
The same-color pair contributes two complete shared Canvas guide entries
while its bracket remains owned by the originating Bar cell. No late target
lookup or new annotation path is needed. The sixteen Significance planner
units, thirteen Bar extraction, thirteen categorical distribution units,
twenty-five facet, affected Canvas, nineteen annotation and nineteen
categorical grouping integration contracts, plus strict Clippy pass. T5d4
now hands one final guide/contrast/target representation to T5d5's closure
matrix and required broad-phase verification.

Feed final categorical identities, complete paints/glyphs, continuous domains,
gradient/title/missing descriptors, and generated guide text into T5b's existing
compatibility and measurement owners. Automatic contributors union; exact domains
must agree and contain automatic envelopes. No arbitrary winner or second merge.

Choose final variable-fill text roles before the font census using the T5c contrast
mechanism and final T5d fills. Revalidate changed metrics, intrinsics, cells,
guides, and cache identities with light/dark candidates of different sizes/fonts.
Supply final rendered-target/facet identities to the T5c annotation input.
Evidence spans transparent surfaces/backdrops, exact override precedence,
same-field conflicting glyphs, data-dependent guide cardinality, and no late
recoloring, reshaping, or target resolution in capture.

### T5d5 — Scale Closure

T5d5a finishes the private customization boundary before the broad gate.
T5d5a1 moves built-in sequential/diverging defaults into the private theme
scale owner and routes one validated Plot-local replacement into the prepared
Heatmap scale and its pre-font contrast/guide producer. Explicit complete
Plot themes leave the narrower local replacement in force. T5d5a2 connects
the same owner to
Beeswarm and mixed-theme Shared Canvas contributors without an intermediate
default gradient or a late selected-mark recoloring path. Keep the public
builder/configuration surface deferred to T6. T5d5b publishes the complete
consumer/equality/error-precedence matrix and runs the required T5d phase
`just verify` closure gate after focused affected-family checks and strict
Clippy. Do not mark T5d complete from default-only gradient evidence.

T5d5a1 closure: Heatmap's built-in semantic stops now live in `theme/scale.rs`.
Its one prepared `ContinuousColors` validates either that default or the
private Plot-local ordered replacement before fonts, retaining the exact
semantic sequence for shared guides, final-domain reinstallation, faceted
marks, selected local stops and value-label contrast. Invalid duplicate
offsets or a diverging gradient without the one neutral midpoint refuse
without silently selecting default paint. An explicit Science Plot theme
does not override a narrower custom gradient. The Heatmap intrinsic cache
now compares the complete selected continuous scale, so a color change at
the same numeric domain reselects the contrasting text role/width. Plot
clones admit custom stop records/bytes. Focused 42 Heatmap units including
faceted custom mark/guide and poisoned capture, 40 public Heatmap contracts,
unchanged Beeswarm fingerprints and shared Canvas composition, builder
configuration admission and strict Clippy pass. The public configuration
surface remains T6; T5d5a2 connects Beeswarm and shared-guide equality.

All marks, legends, gradients, colorbars, facets, and Canvas use one final mapping
per semantic owner. Remove old mixed-palette, hardcoded-gradient, and duplicate
domain/target resolution paths. Publish the full consumer/equality/error-precedence
matrix. Public replacement types/setters remain T6, but complete private custom
values must already exercise the final behavior and layout dependencies.

## T5e: Specialized Layout And Automatic-Root Closure

**Entry:** T5d complete. Native parents, exact mark/annotation requirements, and
final data scales are inputs, not unfinished tasks inherited by this umbrella.
The sole root driver and numerical/resource policies remain unchanged.

| Step | Status | Depends on | Logical commit |
| --- | --- | --- | --- |
| T5e0 | PENDING | T5d | `docs(layout): pin specialized and root sizing closure inputs` |
| T5e1 | PENDING | T5e0 | `feat(heatmap): allocate native measured cell tracks` |
| T5e2 | PENDING | T5e1 | `feat(matrix): allocate native variable grid tracks` |
| T5e3 | PENDING | T5e2 | `feat(marginal): measure native linked subpanels` |
| T5e4 | PENDING | T5e3 | `feat(beeswarm): implement bounded deterministic tangent packing` |
| T5e5 | PENDING | T5e4 | `feat(waterfall): measure native annotation shelves` |
| T5e6 | PENDING | T5e5 | `feat(layout): resolve private automatic root requests` |
| T5e7 | PENDING | T5e6 | `test(layout): close all producer resource and feedback contracts` |
| T5e8 | PENDING | T5e7 | `refactor(layout): close private scientific layout` |

### T5e0 — Rebase The Specialized Census

Map current specialized hard requirements, packing, topology and output to the
native T5b tracks, T5c requirements, and T5d scales. T5a already removed dummy root
plans, backend discovery, and final text/axis reconstruction; do not implement
those deletions again. Record the exact legacy packing symbols still active and
their replacement stage. Confirm all private automatic-root inputs have an owner
and no new preference, hard floor, or algorithm choice is being invented locally.

### T5e1 — Heatmap

Replace legacy pitches/reserves with complete measured column/row tracks. Retain
every semantic cell, merged row, highlight, value label, missing cell, and trailing
slot. Columns allocate before constrained text derives row requirements; preserve
the approved square preference as soft. Use final scales/contrast and explicit
current overflow policy. Evidence includes merged spans, long unbreakable content,
missing values, exact-size failure, and no silent elision beyond its owned policy.
Delete the Heatmap specialized adapter when this native producer lands.

### T5e2 — Matrix

Use native variable grid requirements and the specified `12em` preferred cell
side, including all declared diagonal and off-diagonal cells. Reuse retained
scientific pairs/statistics and common `PanelAxisPlan`; do not recompute either.
Replace local label/cell stabilization with common parent allocation and root
feedback. Evidence includes unequal names, null pair patterns, shared axis spans,
all cell modes, variable-count growth, tight exact roots, and admission before
quadratic topology/storage. Delete the Matrix specialized adapter.

### T5e3 — Marginal

Build the specified two-by-two Grid, with top/right preferences derived once
from the central natural preference. Central/top share width; central/right
share height; preserve the empty corner. Reuse existing KDE/source preparation
and selected primitives. No alternating subpanel-size search. Check linked ranges,
exact widths/heights, guides and labels, and typed fit refusal. Delete the fixed-
band owner and Marginal specialized adapter.

### T5e4 — Beeswarm

Replace deterministic bins with the already-approved greedy tangent producer:
source-row order within category, zero then ordered/deduplicated tangent offsets,
first complete collision-free placement, no backtracking over prior points.
Use a conservative radius containing the complete stroked glyph; validate all
admitted shapes, not only circles. Admit each offset and pair comparison before
work. Occupied height supplies native category tracks; tight height fails rather
than repacking or hiding points. This intentionally changes geometry: retain bin
baselines as historical evidence, inspect new snapshots, and measure increasing
coincident/dense inputs and refusal scaling. Delete the Beeswarm adapter.

### T5e5 — Waterfall

Use native category requirements and the approved base/terminal greedy shelves,
exact targets, baseline ascent/descent, and reserved band. Preserve cumulative
scientific endpoints, signed zero rules, source-row errors, crossing connectors,
data-window semantics, and source-aware collapse rejection. No row-index search
or clipping of required shelf labels. Evidence includes overlapping labels,
multiple shelf rows, mixed exact/automatic domains, and finite cumulative
endpoints despite detached-sum overflow. Delete the Waterfall adapter.

### T5e6 — One Private Three-Mode Root Engine

Extend the existing root request boundary to automatic, exact width/automatic
height, and exact pair. Automatic width comes once from bottom-up preferred
composition; Auto guides contribute initial preferred-right intrinsics without
committing placement. Later edge recovery never restarts width choice. Measure
height at that width, converge under the same driver, and round dimensions upward
once. Exact pairs still fail instead of resizing.

Public constructors continue supplying the existing exact defaults. Exercise the
other requests through private tests, not a second renderer, public selector,
measurement API, `max_width`, or early default switch. Canvas owns assigned panel
dimensions regardless of a valid stored standalone Plot size. Keep current
invalid-input validation and record it explicitly; no implicit validation change.

Evidence: short/long multiline captions, automatic-right-to-top recovery with
unchanged chosen width, 300px exact width with measured height, all 25 families,
sparse/spanning mixed Canvas, changing guide/track constraints, eighth-round fit,
cycles, extreme representability, and no hidden fallback to old exact defaults.

### T5e7 — All-Producer Feedback, Cache, And Resource Census

Close every native producer under all three requests with the common evidence
matrix: prepared/intrinsic/measured/arranged/frozen data, canonical addresses,
consumed/emitted feedback, cache identity, actual replay, and admission classes.
Independently exhaust records/visits/calls/cache/topology/bytes, including hits,
rejected attempts, and later rounds. Preserve complete scientific inputs and
actual work accounting; no aggregate shape-count substitute for layout trials.

Verify PNG gates remain terminal-local in every mode: valid SVG can outlive PNG
refusal, including outlined parser input and requested DPI. T5a already removed
shared hypothetical PNG rejection; extend its evidence to new modes instead of
reintroducing and removing another gate. No third-party OOM-recovery promise.

### T5e8 — Private Layout Closure And T6 Entry

Audit all ledger portions and delete remaining specialized adapters, fixed-band/
generic-minimum/global-search remnants, post-freeze reconstruction, and unused
legacy root helpers. Reconcile source counts and exclusions, not merely type
names. Publish ordinary/specialized/composition fit/failure, byte/pixel geometry,
resource and scaling evidence; include long captions and legal-break workloads.

All geometry-affecting private theme, RGBA, contrast, and scale inputs must already
work here, including private custom values, before describing T6 as public API
exposure. T6 performs the atomic public replacement and repeats integration tests;
it is not the first phase to discover that a custom contrast role changes metrics.

T5e completion does not start T6. At T6 ENTRY present DF1/DF2/remaining DF5/DF7
evidence and record user-approved admission with a new phase, rejection, or a
new explicit re-deferral checkpoint. DF3/DF4 remain T6 CLOSURE; DF6/DF8/DF9/DF10
remain T7 CLOSURE. No review gate silently admits those features.

## Completion And Decision Discipline

This revision has one newly confirmed behavior decision: preserve explicit data
windows as specified above. All other behavior comes from existing approved
contracts. No unresolved product question is currently blocking the planning
partition. This is not a guarantee that implementation cannot uncover one.

Ask the user before expanding composition support, changing an existing explicit
policy, replacing a prescribed numerical contract without equivalent approved
evidence, adding new public controls, changing resource caps, or admitting a
deferred feature. Engineering choices within the contracts are implementation
work, not reasons to stop at a partial checkpoint. Each implementation owner
reports actual remaining gaps; passing tests alone cannot close a missing owner,
admission boundary, or semantic preservation requirement.
