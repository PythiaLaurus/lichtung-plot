# T5b-T5e Execution Plan

**Status: IN PROGRESS — T5b complete; T5c in progress; T5d–T5e pending.**

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
| T5c1 | IN PROGRESS | T5c0 | Umbrella; completes after T5c1a-b |
| T5c1a | COMPLETE | T5c0 | `feat(theme): separate interval and summary strokes` |
| T5c1b | PENDING | T5c1a | `feat(theme): resolve remaining private mark and annotation roles` |
| T5c2a | PENDING | T5c1 | `refactor(marks): measure point path and reference requirements` |
| T5c2b | PENDING | T5c2a | `refactor(marks): measure categorical estimate and bar requirements` |
| T5c2c | PENDING | T5c2b | `refactor(marks): measure distribution and interaction requirements` |
| T5c2d | PENDING | T5c2c | `refactor(marks): measure specialized mark requirements` |
| T5c3 | PENDING | T5c2d | `refactor(annotations): measure native overlay consequences` |
| T5c4 | PENDING | T5c3 | `refactor(layout): replace deferred mark and annotation minima` |
| T5c5 | PENDING | T5c4 | `test(theme): close native mark and annotation ownership` |

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

The common standard preference is the exact `05` density/`66ch`/`4:3` rule;
categorical hard/preferred pitch is derived from actual occupied slots. Aspect
and density are preferences, not a new hard minimum or root-size percentage.
Data-window tests include off-domain marks, crossing lines/intervals, exact-edge
strokes, mixed exact/automatic axes, genuine zeros, and malformed/collapsed data.

### T5c3 — Native Annotation And Overlay Consequences

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
| T5d0 | PENDING | T5c | `docs(scales): map semantic domains and migration consumers` |
| T5d1 | PENDING | T5d0 | `feat(color): implement deterministic scale arithmetic` |
| T5d2 | PENDING | T5d1 | `feat(scales): resolve categorical maps palettes and accents` |
| T5d3 | PENDING | T5d2 | `feat(scales): resolve sequential and diverging mappings` |
| T5d4 | PENDING | T5d3 | `refactor(scales): coordinate final domains guides and contrast` |
| T5d5 | PENDING | T5d4 | `test(scales): close mapping and guide consistency` |

### T5d0 — Domain And Consumer Census

Map every categorical field, positioned series, unkeyed primary/accent consumer,
continuous field, missing-value section, guide contributor, and significance
target to its prepared identity and current override API. Specify the private
replacement and deletion owner for every existing palette/gradient helper.
Pin fixtures for domain growth, sparse facets, hidden legends with visible marks,
same-color exact mappings, mixed themes, and unequal exact/automatic domains.

### T5d1 — Pinned Color Arithmetic

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
