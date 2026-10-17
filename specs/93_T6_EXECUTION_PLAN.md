# T6 — Public Foundations And Scientific Presentation

**Status**: IN PROGRESS — immutable public foundations, their complete custom-value
closure, public preferred tick intervals and root-sizing closure are complete;
scientific numeric presentation is next.

**Current execution checkpoint:** the user-authorized resumption completed the
mark/annotation leaf, T6B1d4d, and the private-value umbrella T6B1. Evidence is in
`../docs/mark-annotation-values.md`. T6B2 owns complete Theme construction and the
remaining literal-to-component consumer resolution. Complete construction, mark
extents, annotations, continuous mappings and supplemental defaults are complete;
`../docs/theme-consumer-closure.md` records the final owner/identity reconciliation.
The public exact-style cutover is complete, with current RGBA/selector and caller
evidence in `../docs/exact-element-styles.md`. The complete public Theme/font/scale
cutover is complete, with typed construction, retirement and pre-font capacity
evidence in `../docs/complete-theme-foundations.md`. Narrow heading instructions
are complete, with precedence and frozen-root evidence in
`../docs/heading-alignment.md`. Public custom-value foundation closure is complete
with external construction/owner/contrast/scale evidence in
`../docs/public-foundation-closure.md`. This completes T6B.
Public preferred tick intervals are also complete, with validated construction,
actual categorical-axis refusal and source/parent evidence in
`../docs/preferred-tick-intervals.md`. Public root sizing is complete in
`../docs/content-derived-root-sizing.md`; all-mode parent/output evidence and the
actual allocation repairs are complete in `../docs/public-sizing-closure.md`.
This completes T6C; numeric source refresh follows.
The user-approved simplification in
`95_RESOURCE_ACCOUNTING_SIMPLIFICATION_PLAN.md` has removed production accounting,
not merely disabled quota refusal. Historical admission evidence below describes
its recorded revisions and is not a requirement to restore fees. Verification,
baseline failures and release parity/timing/RSS are recorded in
`../docs/resource-accounting-cleanup.md`.

The user explicitly requested test-code auditing/planning after the cleanup and
caption measurements, with no automatic return to feature implementation. The
approved execution plan is `96_TEST_CODE_SIMPLIFICATION_PLAN.md`; its source-backed
audit is `../docs/test-code-audit.md`. Test cleanup is complete, with retained E2E
owners, final census and successful complete-gate evidence in
`../docs/test-code-cleanup.md`. A further user-directed deletion removed 678 of
1,349 library tests; the complete gate passed with 654 passing and 17 ignored
library tests, 462 integration tests and 53 doctests. New unit tests must replace
existing tests in the same logical change; total discovered library-test count
must not increase (see `AGENTS.md` and spec90).

## Authority And Entry Decisions

`91_IMPLEMENTATION_PLAN.md` owns the umbrella status. This document owns T6's
substage dependencies, public cutovers, deletion targets and acceptance evidence.
`03_THEME_SYSTEM.md` and `05_LAYOUT_SYSTEM.md` own the current foundations and
coordinated layout, with remaining presentation work explicitly staged;
`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` describe implemented public
behavior and change with the relevant implementation commit, not this plan.

Baseline: T5e closed in `b803f2a`; future AI-workflow review recorded in `b8b75a7`.
The user approved the four-item T6 entry assessment and requested this plan.
The user additionally selected raw-value numeric formatting for configured
Forest Log10 ticks: an exponent `0.5` with two fractional digits displays `3.16`,
not a formatted exponent or `10^0.5`.

| Entry item | User-approved disposition | T6 work / remaining checkpoint |
| --- | --- | --- |
| DF1 separate legend sections | ADMIT the complete bounded discrete-section capability | T6L; remove DF1 from the active deferred list now |
| DF2 caller numeric formatting | ADMIT one cross-surface, locale-free scientific core | T6N; residual locale/general formatting remains DF2, reviewed at T7 CLOSURE |
| DF5 broader text controls | Existing approved typography/root alignment proceeds; RE-DEFER remaining controls | T6B exposes existing scope; remaining DF5 reviewed at T7 CLOSURE |
| DF7 sparse compaction | RE-DEFER topology transformation | Preserve coordinates; review empty-track removal and occupied-cell repacking separately at T7 CLOSURE |

TextPanel alignment, categorical label directions and identifier breaking are
identified candidates for that later DF5 review, not additional T6 implementation.
DF3/DF4 keep their T6 CLOSURE gates. DF6/DF8/DF9/DF10 keep T7 CLOSURE. DF11 stays
after workspace, both adapters, Python interface and first release are complete.

## Delivery Discipline

- Keep the renderer standalone, offline and safe. No workspace conversion,
  adapters, Python files, `tools/`, asset provisioning or new dependency is part
  of T6. Existing pinned `num-bigint` is available for certified numeric work.
- Extend T5e's one measured lifecycle, current prepared sources, scoped domains,
  source identities, complete ink, operation-local caches and immutable capture/replay.
  No second renderer, local grow-until-fit loop or new resource cap.
- Split behavioral implementation from mechanical repository-wide API migration.
  Each completed leaf gets focused tests, strict Clippy and its own scoped commit
  before the next leaf. Closure stages verify completed owners, not accumulate
  missing implementation into another omnibus patch.
- A private implementation stage may prepare a replacement behind the existing
  current API, but must migrate storage onto one representation. Public cutover
  removes the replaced API and migrates every caller. No aliases, deprecations,
  version selectors, dormant public types or old/new fallback chains.
- Verify intentional visual changes through source completeness, semantic values,
  effective styles and complete-geometry relationships. Spec90 excludes frozen
  whole-scene byte/hash/snapshot and incidental-coordinate test baselines.
- Record stages as COMPLETE only after actual checks; an approved plan is not
  evidence of an implemented or numerically certified primitive.

## Stage Map

The order is foundations, public sizing/ticks, numeric core, sectioned guides,
then integrated closure. Each umbrella below consists of separately committed
leaves. Suggested commit subjects are partition labels, not existing commits.

| Stage | Status | Depends on | Deliverable / suggested scoped commit |
| --- | --- | --- | --- |
| T6P | COMPLETE | T5e | This plan and user-approved entry decisions; `docs(planning): partition public foundations and presentation` |
| T6A | COMPLETE | T6P | Public foundation/caller/ledger census and construction design |
| T6A1 | COMPLETE | T6P | Current export/style/owner/caller/deletion census in `docs/t6a1-public-cutover-census.md`; `docs(api): map public cutovers and consumers` |
| T6A2 | COMPLETE | T6A1 | Validated construction, fields, metrics, owners and future fixtures in `94_T6_FOUNDATION_CONSTRUCTION.md`; `docs(theme): pin validated public component construction` |
| T6B | COMPLETE | T6A | Immutable theme, color, font, scale and exact-style migration |
| T6B1 | COMPLETE | T6A | Validated color/font and component values in the single private representation; `refactor(theme): retain validated foundational values` |
| T6B1a | COMPLETE | T6A | One Color/OpaqueColor paint representation and private channel/hex validation; `refactor(color): retain validated foundational paint values` |
| T6B1b | COMPLETE | T6B1a | Typed font preferences and complete role-stack inputs; `refactor(font): retain validated role font stacks` |
| T6B1c | COMPLETE | T6B1b | Exact relative spacing and bounded local metric operands; `feat(theme): retain exact relative spacing values` |
| T6B1d | COMPLETE | T6B1c | Complete component/role/scale inputs and structural validation; `refactor(theme): retain complete validated components` |
| T6B1d1 | COMPLETE | T6B1c | Replace live text/surface appearances with validated TextTheme/SurfaceTheme; `refactor(theme): validate text and surface values` |
| T6B1d2 | COMPLETE | T6B1d1 | Replace live compound stroke/point/fill inputs with validated values and structural absence; `refactor(theme): validate complete mark paints` |
| T6B1d3 | COMPLETE | T6B1d2 | Retain validated opaque palettes/maps/gradients and domain selection; `refactor(scales): retain validated complete scale values` |
| T6B1d4 | COMPLETE | T6B1d3 | Assemble validated complete role/chrome/annotation groups with actual consumer mappings; `refactor(theme): assemble complete validated components` |
| T6B1d4a | COMPLETE | T6B1d3 | Complete nested typography with all eighteen live roles; `refactor(theme): assemble complete typography roles` |
| T6B1d4b | COMPLETE | T6B1d4a | Complete axis chrome and exact role-local spacing; `refactor(axes): retain validated local chrome` |
| T6B1d4c | COMPLETE | T6B1d4b | Complete guide and composition chrome; `refactor(theme): retain validated composition chrome` |
| T6B1d4c1 | COMPLETE | T6B1d4b | Complete measured guide chrome and exact admission; `refactor(legend): retain exact validated guide chrome` |
| T6B1d4c2 | COMPLETE | T6B1d4c1 | Complete root/embedded composition chrome; `refactor(composition): retain validated local relationships` |
| T6B1d4d | COMPLETE | T6B1d4c | Complete mark groups and admitted annotation geometry; `refactor(theme): assemble mark and annotation roles` |
| T6B2 | COMPLETE | T6B1 | Complete Theme construction/customization and owner resolution using those values; separately committed leaves below |
| T6B2a | COMPLETE | T6B1 | Retain one complete appearance value and construction-only builder independently of operation state; `refactor(theme): retain complete construction values` |
| T6B2b | COMPLETE | T6B2a | Consume mark extents in category, Waterfall, interval, rug and density geometry; `refactor(marks): resolve retained local geometry` |
| T6B2c | COMPLETE | T6B2b | Resolve annotation metrics, exact significance absence and canonical bracket geometry; independently reviewed leaves below |
| T6B2c1 | COMPLETE | T6B2b | Retain one effective significance value through text, containing fit and canonical capture; `refactor(annotations): retain effective significance geometry` |
| T6B2c2 | COMPLETE | T6B2c1 | Resolve selected-role cell, estimate and panel-text annotation metrics; `refactor(annotations): resolve selected-role local spacing` |
| T6B2d | COMPLETE | T6B2c | General continuous-scale branches, unified presets and common local/shared guides; `refactor(scales): resolve complete continuous mappings` |
| T6B2e | COMPLETE | T6B2d | Complete supplemental paint/default resolution and remaining declared-track/cache owners; `refactor(theme): resolve complete owner-local defaults` |
| T6B2f | COMPLETE | T6B2e | Reconcile retained components and consumers with focused owner/identity evidence |
| T6B3 | COMPLETE | T6B2 | Exact element Color/alpha replacement and repository caller migration; `feat(styles): expose complete RGBA element paints` |
| T6B4 | COMPLETE | T6B3 | Public complete Theme/components/fonts/scales, built-in constructors and palette/map/gradient attachments; remove closed selectors and old scale representations; `feat(theme): expose immutable theme and scale foundations` |
| T6B5 | COMPLETE | T6B4 | Expose already implemented title/subtitle instructions and theme caption alignment; `feat(composition): expose independent heading alignment` |
| T6B6 | COMPLETE | T6B5 | Public custom-value inheritance, contrast, scale and style evidence; `test(theme): close public foundation precedence` |
| T6C | COMPLETE | T6B | Public root sizing and preferred tick intervals |
| T6C1 | COMPLETE | T6B | Opaque validated TickInterval and sealed numeric-axis setters; `feat(axes): expose preferred numeric tick intervals` |
| T6C2 | COMPLETE | T6C1 | Automatic defaults, consuming width, exact size and Output dimensions; migrate getters/callers atomically; `feat(layout): expose content-derived root sizing` |
| T6C3 | COMPLETE | T6C2 | Complete initial axis/chrome preferences, provisional content-height gutters and all-mode parent/output evidence; `fix(layout): compose complete content-sized root frames` |
| T6N | PENDING | T6C | Unified scientific numeric formatting; leaves below |
| T6L | PENDING | T6N | One section-aware discrete guide; leaves below |
| T6Z | PENDING | T6L | Integrated contract, deferred-decision and phase closure |
| T6Z1 | PENDING | T6L | Complete public examples/current-spec and property/consumer census reconciliation; `docs(api): reconcile scientific plotting contracts` |
| T6Z2 | PENDING | T6Z1 | User-approved DF3/DF4 closure dispositions |
| T6Z3 | PENDING | T6Z2 | Final audit, required broad gate/release evidence and closure commit; `test(api): close public scientific foundations` |

T6A2 must specify all accessors, validated constructors and construction-only
edits before T6B starts; the broad component names in spec03 are not permission
to expose backend structs. Split a leaf further if its behavioral changes are
not independently reviewable. Finish its owning API migration before claiming
the related public contract current.

T6A1 evidence: `docs/t6a1-public-cutover-census.md` inventories every current root
re-export, exact tuple/opacity/font/scale surface, producer/parent/caller family,
private owner and scheduled deletion. Independent source review checked named
tests, capability sets and fingerprint attribution. It records actual legacy
opacity endpoints, active typography/scale carrier gaps, raw font-constant
retirement, conflicting Shared-gap ledger assignments and the surviving public
Canvas 160×120 gate. Constructor/owner adjudication is explicitly T6A2 work;
this documentation census changes no implemented API or renderer behavior.

T6B1a evidence: `docs/t6b1a-paint-values.md` records the sole production paint
representation in `theme::Color`/`OpaqueColor`, strict ASCII hex construction,
opaque conversion, exact alpha replacement and complete caller migration.
Backend `Rgba` is deleted; no alias, tuple conversion trait or alpha-dropping
conversion replaces it. Legacy floating opacity remains at its existing private
boundary until B3. Thirty-five focused tests cover byte-channel/independent SVG
round trips, PNG alpha, contrast, guide/cache/lifecycle and unchanged snapshots;
the phase owner additionally confirmed the eight core parser/backend/SVG/PNG
tests and strict all-target Clippy. Public exposure and data-scale migration stay
with B3/B4 and B1d/B2. No snapshot or gradient expectation changed in this leaf.

The Canvas universal minimum is the existing ledger retirement
`validation.canvas.minimum-size`, not a new size policy. T6C2 must remove its
remaining public validation gate while preserving positive-size timing and
complete-content/resource refusal.

T6B1c evidence: `docs/relative-spacing-values.md` records reduced exact rational
values, immutable sharing, admitted local metric arithmetic, aggregate rounding
and active text-leading/point-outline consumers. Production preset construction
uses the supplied root ledger and propagates refusal before fonts. Forty resumed
focused tests pass, including complete caption fixtures and seven unchanged SVG
snapshots. `docs/font-work-admission.md` records the adjacent font regression
repair: admitted hash-prefix/query identity reuse and shared immutable inputs
keep all 2048 words inside the unchanged million-visit cap. Strict Clippy,
formatting and independent constructor/admission review pass; complete spacing
component storage and consumer resolution remain the next owning migration.

T6B1b evidence: `docs/font-stack-values.md` records typed literal/generic stacks
through all eighteen roles, canonical selection, source requirements, retained
plans, identity and CSS publication. The raw parser, parser cache and first-family
sentinel are removed; current root constants remain only until their public
cutover. Independent admission review closed hashing/equality, staged-copy,
significance/Matrix clones and redundant child/census charges. Exact boundary
tests cover actual examined bytes and single-use copy admission; fatal refusal
does not become a cache miss. Focused font, Unicode Mark, census, lifecycle/cache,
snapshot and Chinese fallback checks and strict Clippy pass with no repinning.

The numeric/guide design refinement is complete in `06_NUMERIC_PRESENTATION.md`,
`07_LOG10_DISPLAY.md` and `08_DISCRETE_GUIDES.md`. These architectural targets
close the consumer/default/key/font contracts, Forest interval/rounding/
termination/resource/oracle proofs, and every discrete producer's section/mapping
identity and measured-flow rules. Source-backed and independent mathematical
reviews are documentation evidence, not successful execution of the future code.
Implementation-time source refreshes below check their mapping to changed
foundations; they do not leave public signatures or product choices unplanned.

T6B1d1 evidence: `docs/text-surface-values.md` records the live validated
TextTheme/SurfaceTheme replacement, scalar validation, readonly inspection and
consuming edits with preserved source/default bits. All eighteen actual roles and
operation-admitted copies use the new constructor. Eighty-five distinct focused
tests and seven unchanged SVG snapshots pass, as do strict all-target Clippy,
formatting and whitespace checks. Complete compound and aggregate construction
continues in the following private leaves; no early root export was added.

T6B1d2 evidence: `docs/mark-paint-values.md` records nine live validated compound
values and removal of the old appearance/outline-choice representations. Present
widths are positive and absence is structural; mapped/fixed outlines preserve
their own independent alpha. Thirty-eight distinct focused tests, strict Clippy,
formatting and source/identity review pass. Two inspected Density/Regression
snapshots change only five zero-width stroke attribute sets; an independent
raster fixture proves identical PNG bytes and nonempty alpha-56 paint. Complete
group assembly and public exposure remain with subsequent owning leaves.

T6B1d4a evidence: `docs/typography-values.md` records seven complete validated
groups with eighteen live roles and removal of the flat typography aggregate.
Opaque contrast candidates, recursive sibling validation, original source/leading
bits and exact twenty-two-handle appearance-copy admission are preserved.
Seventy-two focused tests, unchanged snapshots, formatting and strict Clippy pass.
Axis, guide/composition and mark/annotation groups remain separately committed
units; the umbrella is not complete until every such owner is migrated.

T6B1d4b evidence: `docs/axis-values.md` records complete axis constructors,
exact selected-role spacing, structural absence, frozen side replay and admitted
identity. Ordinary frame and variable-grid cell-frame roles retain four-side and
bottom/left defaults independently. Actual full Matrix SVG bytes match `633c8e0`
for all four presets, with pinned hashes and semantic topology checks. Review
closed natural Matrix's unused ordinary-frame dependency; automatic/width-only
output is invariant under unrelated frame edits, while actual cell ink remains
measured. Focused constructor/cache/frame/facet/root tests and strict Clippy pass.

T6B1d4c1 evidence: `docs/guide-values.md` records the sole validated guide
component and replacement of raw pixel/glyph-comfort sources. Discrete and
continuous widths now use complete exact reserves; fractional capacity tests
close early double-ceiling and scalar-reconstruction gaps. Review restored
uniform compact vertical columns and rejection above the assigned cap, with
actual preceding-column advancement and zero-overlap ink evidence. Five inspected
snapshots change only documented quantum-level normalization consequences,
preserving content, paint and topology. Focused tests, replay/budget checks,
formatting and strict Clippy pass. Current public Shared-gap override stays
until its scheduled public cutover; composition chrome remains the next unit.

T6B1d4c2 evidence: `docs/composition-values.md` records eighteen exact local
relationships, complete owner-local root/embedded/TextPanel/header/facet consumers
and immutable operation-scoped frame reuse. Final review closed separately
rounded prose reserves and whole-pixel header placement; six real-owner boundary,
RTL, natural-height and cache regressions pass with strict Clippy and formatting.
Four-preset Matrix full-byte parity and selected unchanged snapshots are recorded
in that evidence. The caption's logical-budget count is not a CPU/time/memory
measurement or proof of efficiency. At the user's request, subsequent feature
implementation is paused for a comprehensive resource-accounting analysis after
this completed component commit; budget thresholds are not optimization targets.

T6B1d4d evidence: `docs/mark-annotation-values.md` records complete validated
MarkTheme/AnnotationTheme construction, private fields and complete consuming
edits. The flat mark/annotation/radius records and generic foreground carrier are
removed; current stored-field consumers, complete identity and Heatmap intrinsic
keys use the retained components. Sampled/supplied strip planning, marks and guides
use their independent sized role and the actual outline policy. Point-label and
Waterfall shelf gaps use exact local text metrics; complete shelf reserves round
once. Three semantic tests replace three retired fixtures; discovery remains 671
including 17 ignored. Seventy-six distinct focused library tests and 44 integration
tests pass, as do strict Clippy and formatting. This completes the private-value
umbrellas; newly retained literal geometry, significance/backend handoff and
complete Theme/scale behavior remain the explicit B2 consumer migration.

T6B1d3 evidence: `docs/scale-values.md` records immutable validated opaque
palettes, gradients/domains and selection values, actual prepared transport and
separate qualitative/accent ownership. Current categorical API spelling owns
one opaque map representation until public cutover. Sixty-eight focused initial
tests pass; review additionally closed signed-zero diverging-center normalization
and immutable gradient-handle cache admission with fifteen focused follow-up
tests. Strict Clippy, formatting and unchanged selected snapshots pass. Unified
Beeswarm defaults and general diverging consumption remain the B2 behavior change.

T6A2 evidence: `94_T6_FOUNDATION_CONSTRUCTION.md` pins complete fields, typed
construction/accessors/consuming edits, actual CSS units, exact local metrics,
default operand bits, validation, deletion stages and future focused fixtures.
Independent review closed missing neutral path/flier paint, independent Waterfall
breadth, continuous-guide successor and declared-track em owners. The user
explicitly chose theme-only Shared legend gap and a common Heatmap-style
`#F7FBFF -> #08306B` default sequential gradient; Beeswarm's color change belongs
to B2 and is not described as default-byte preservation. Old source inventories
remain historical; the field/ownership decisions are authoritative for T6B.

## Existing Public Foundations To Expose

T6B2a evidence: Plot/Canvas retain shared immutable complete `ThemeValue` inputs;
the current public selector is only a constructor input, not stored runtime
identity. `ResolvedTheme` keeps operation state beside one complete appearance
value. The construction-only builder revalidates components and applies one typed
font stack to all eighteen roles without a runtime precedence layer. Four unused
generic size carriers are removed. One constructor/immutability/font-precedence
test replaces the old local surface-edit fixture; discovery remains 671. Forty-one
theme tests, four axis/lifecycle tests (including all 25 canonical families), six
external theme-owner tests, formatting and strict all-target Clippy pass.

T6B2b evidence: `../docs/mark-geometry-resolution.md` records selected category
breadths/group spans/caps, independent Waterfall body and connector geometry,
PointInterval caps, owner-prepared sampled rugs and local density fractions.
Actual positive ink without containing room refuses; zero ink does not divide by
zero. Histogram preparation receives the complete owning appearance before fonts
without changing bins, KDE or supplied geometry. Two semantic tests replace two
old fixtures and remove their dead support; discovery remains 671. Sixteen focused
library tests, twenty contract integrations, formatting and strict Clippy pass.

T6B2c1 evidence: `../docs/significance-geometry-resolution.md` records one effective
significance appearance with actual absent/exact provenance and selected-role
geometry. Frozen brackets retain their complete style, including the tick height
consumed by canonical validation. The default-materializing accessor, backend
geometry constant and duplicate selected-style/fallback reconstruction are gone.
One semantic planner/canonical test replaces a typography group-edit fixture;
discovery remains 671. Ten focused library tests, twenty-four annotation/estimate/
parent integrations, formatting and strict all-target Clippy pass.
The subsequent closure review corrected comparison hard-space aggregation:
selected geometry remains exact through level occupancy, complete reserves and
range/ceiling fit, with one upward boundary and only final position rounding.
A two-level fractional-quantum boundary regression replaces the guide field/edit
inventory; discovery remains 671. Eleven focused library tests and twenty-six
annotation/facet/estimate integrations, formatting and strict Clippy pass; the
same evidence document records the correction and actual checks.

T6B2c2 evidence: `../docs/annotation-local-spacing.md` records distinct selected-role
cell/value and merged-row padding through intrinsics, assignment, wrapping and
final fit; final cell alpha selects the same role before fonts and in capture.
Panel text retains its selected padding and complete minima for reassignment.
Estimate text consumes its detail-role inset/clearance/numeric gap and actual
reference ink. Two semantic tests replace typography inventory/spacing-clone
fixtures and remove dead support; discovery remains 671. Five focused library
tests, twenty-one external contracts, formatting and strict Clippy pass.

T6B2d evidence: `../docs/continuous-mapping-resolution.md` records one final-domain
mapping for arbitrary ordered sequential/diverging stops, both families and Shared
guides. Beeswarm consumes complete owner-selected inputs, arbitrary diverging
branches, a real center and one-to-three labels. Shared installation remaps from
semantic stops rather than a local projection. The approved common sequential
default replaces the old Beeswarm-only source. One source-to-capture/Shared test
replaces a scale-component edit fixture; discovery stays 671. Twenty-seven focused
library tests (including the all-25-family lifecycle), sixteen external contracts,
formatting and strict all-target Clippy pass.

T6B2e evidence: `../docs/supplemental-owner-resolution.md` records optional exact
PointLayer instructions, theme supplemental defaults, selective complete label
paint and identical mark/guide styles. Floating opacity storage is removed; the
current caller setter quantizes once. Font census and final text select the same
role while preserving radius-relative whole-pixel placement and complete uncut
ink. Declared Canvas/facet em and complete cache dependencies are source-checked.
One selected-payload/guide contract replaces the composition edit inventory and
its helper; discovery stays 671. Four focused library tests, twenty numeric-layer
contracts, formatting and strict all-target Clippy pass.

T6B2f evidence: `../docs/theme-consumer-closure.md` reconciles all eight complete
components, eighteen text roles, local metrics, prepared/scoped/canonical owners
and immutable/cache identity. Forty-one final focused theme/child-key tests and
six explicit/inherited/default owner integrations pass; discovery remains 671.
Preceding behavioral leaves provide actual selected geometry, continuous branch,
font-census and frozen replay evidence with strict Clippy. This closes private
consumer resolution without claiming the later public API or presentation phases.

T6B3 evidence: `../docs/exact-element-styles.md` records public byte-channel Color,
complete Guide/PointLayer/BarPointLayer/Dumbbell paints, selective alpha precedence,
significance inspection/edits and fallible lexical Interaction construction.
All affected constructors, fields, mark/guide/canonical operands and repository
callers migrate together; old tuple, floating-opacity and consuming bare-field
routes are removed. Three behavioral replacements retire three inventories;
discovery remains 671, including 17 ignored. Eleven focused library tests,
sixty-one existing external contracts, sixty-six doctests, strict Clippy,
formatting, warning-denied docs and package verification pass.

T6B4 evidence: `../docs/complete-theme-foundations.md` records the single public
complete Theme, construction-only builder, all typed components/roles/fonts and
palette/map/gradient/domain attachments. Every caller migrates with deletion of
the selector, old scale fields/setters, raw font constants and Canvas shared-gap
override. Hidden standalone and Shared-final-domain capacity errors now precede
fonts; two regressions replace two axis inventories. Discovery remains 671,
including 17 ignored. Forty-seven distinct current focused library tests,
eighty-six existing external contracts, eighty-four doctests, strict Clippy,
formatting, warning-denied docs and package verification pass. Narrow heading
instructions and the public custom-owner closure remain the following leaves.

T6B5 evidence: `../docs/heading-alignment.md` records the four independent public
Plot/Canvas panel instructions, theme-owned caption/Canvas root alignment and
frozen concrete precedence. One real composed-scope/replay test replaces a route
inventory; existing standalone/Matrix/facet fixtures migrate to the public API.
Five focused library tests, eight existing external contracts, eighty-six
doctests, strict Clippy, formatting, warning-denied docs and package verification
pass. Discovery remains 671, including 17 ignored. Public custom-value foundation
closure is the remaining foundation leaf.

T6B6 evidence: `../docs/public-foundation-closure.md` records public eight-component
construction, distinct Canvas/facet/TextPanel roles, exact replacement/inheritance,
independent palette/accents, five-stop mixed-family Shared mapping and complete
custom contrast through identical same-operation SVG and decoded transparent PNG.
All fifteen categorical gates compile through actual attachments; custom outline
alpha conflicts refuse before unavailable fonts. Three existing focused library
tests, thirty-five distinct external contracts, eighty-six doctests, strict
Clippy, formatting, warning-denied docs and package verification pass. No library
tests are added or moved; discovery remains 671, including 17 ignored. This closes
the public foundation umbrella; interval and sizing cutovers follow.

T6C1 evidence: `../docs/preferred-tick-intervals.md` records opaque public
TickInterval, immediate constructor validation, NumericX/Y attachments and atomic
retirement of the raw private choice. Scientific categorical-axis refusal occurs
before fonts in standalone/Canvas, with one behavior test replacing a default
field inventory. Existing complete source ordering, checked widened logarithmic
coarsening, exact precedence, census, cache and quantum-fit fixtures use the public
values. Thirty-one focused library tests, six external contracts, ninety-two
doctests, strict Clippy, formatting, warning-denied docs and package verification
pass. Discovery remains 671, including 17 ignored. Root sizing remains next.

T6C2 evidence: `../docs/content-derived-root-sizing.md` records one retained private
request, automatic Plot/Canvas defaults, consuming exact width, full replacement
semantics and Output inspection from canonical scene dimensions. Pre-render Plot
getters, nominal fixed-default fields, test-only sizing entry points and the
Canvas universal minimum are removed. Zero requests refuse before fonts/backend
begin; actual complete-content fit remains mandatory. One behavioral unit test
replaces a source inventory; explicit-root fixtures request their actual sizes.
Thirty-five focused library tests, four external contracts, ninety-five doctests,
strict Clippy, formatting, warning-denied docs and offline package verification
pass. Discovery remains 671, including 17 ignored. Expanded public parent/mode
and PNG-local closure follows in T6C3.

T6C3 evidence: `../docs/public-sizing-closure.md` records complete public roots for
all twenty-five families and automatic/exact-width Canvas roots for all twenty-four
admitted children. Actual Forest/Bar failures close shared initial horizontal
axis/chrome preference and provisional content-height gutter consumption, retaining
Matrix's native trailing-edge owner. All modes preserve sparse facets, native
Matrix, separate local/root prose, mixed/all-text Canvas and frozen multi-format
dimensions. Actual canonical and outlined SVG exceed 64 MiB in all modes while
retaining all 900,000 coincident points; PNG independently refuses the outlined
parser payload. Source-axis/area and default target boundaries also pass.
Twenty-five focused library tests, sixteen distinct external contracts,
ninety-five doctests, both sizing examples, strict Clippy, formatting,
warning-denied docs and offline package verification pass. No unit tests are added
or moved; discovery remains 671, including 17 ignored. T6C is complete; T6N0 owns
the following actual-input numeric source refresh.

Use spec03 and spec94's complete `Theme`, `ThemeBuilder`, role components, typed font stack,
RGBA `Color`, RGB-only `OpaqueColor`, palettes/maps and gradients. Builders are
construction-only; a Plot/Canvas stores immutable complete values, never sparse
edits. Built-ins are `Theme::{publication,nature,science,chi}()` constructors.

Preserve explicit Plot replacement over Canvas inheritance, element paint over
Plot scale over complete theme, separate fill/stroke alpha and exact alpha
replacement. Migrate `CategoricalColors`, `HeatmapScale`, RGB tuple inputs and
opacity naming to their single replacement owners. Do not alter scientific
statistics or start data-mapped alpha work.

Title instructions follow spec03's current private precedence. Expose only
`Plot::title_alignment`, `Plot::subtitle_alignment`,
`Canvas::panel_title_alignment`, `Canvas::panel_subtitle_alignment` and concrete
theme root title/caption plus optional subtitle defaults. No TextPanel alignment,
arbitrary rotation or generic anchors are added. Complete per-role font sizes
are explicit typography, not automatic shrink-to-fit.

Expose `TickInterval::{additive,log10_decades}` with existing validation,
candidate ordering and exact-tick precedence. No count/density/callback controls.

Omitted sizing becomes automatic. `Plot::width(u32)` and `Canvas::width(u32)`
select exact width/content height; `size(u32,u32)` selects an exact pair. Every
sizing setter replaces the full request. Remove pre-render Plot dimension getters;
add `Output::width()`/`height()` for frozen canonical CSS pixels. Positive sizes
continue validating at preflight. Canvas still owns embedded child dimensions.

## T6N — One Scientific Numeric Presentation Core

`06_NUMERIC_PRESENTATION.md` is the detailed authoritative contract and current
consumer migration census. `07_LOG10_DISPLAY.md` supplies the complete numerical
design and document-level proof. The summary below remains the execution handoff;
their implementation, executable oracle and measured admission tests are pending.

### Scope And Ownership

Admit fixed fractional/significant precision, scientific and engineering
notation, explicit percentage/decimal display scaling, literal affixes,
negative-zero and trailing-zero handling. Formatting is semantic presentation,
not a theme component and not a transformation of scientific data.

Consumers are generated numeric axes, Heatmap/Beeswarm continuous guides,
existing Heatmap finite-value annotations, existing point-only Coefficient
labels and Waterfall base/terminal totals. Exact ticks, category/legend identities,
point-label strings, significance stars and caller annotation text stay verbatim.
Matrix has no new global numeric-axis setter. No new numeric annotation surface.

### Target API And Validation

One opaque crate-root `NumericFormat: Clone + Debug + PartialEq + Eq` replaces
`NumericLabelFormat`. Spec94 fixes the construction/accessor vocabulary; closed
input discriminants are:

```rust,ignore
enum NumericNotation { Decimal, Scientific, Engineering }
enum NumericPrecision {
    Fractional(u8),
    Significant(u8),
    AdaptiveFractional { min: u8, max: u8 },
    AdaptiveSignificant { min: u8, max: u8 },
}
enum NegativeZero { Preserve, Normalize }
enum TrailingZeros { Keep, Trim }
```

`NumericFormat::new(notation, precision) -> Result<Self>` validates fractional
`0..=17`, significant `1..=17` and adaptive `min<=max`. Consuming edits replace
only their own input using spec94's `with_field` names: `with_notation`, validated
`with_precision`, `with_decimal_scale(i16)` in `-324..=324`, nonempty/control-free
`with_prefix`/`with_suffix`, `with_negative_zero` and `with_trailing_zeros`.
Bare field names are readonly getters. Fields remain private; no backend formatter
or arbitrary formatting callback is public. Ordinary scalar getters follow the
validated-value style fixed in T6A2.

Percentage is exact scale `2` plus suffix `%`: `0.125` at one fractional digit
becomes `12.5%`; suffix `%` alone does not scale. Decimal powers are the admitted
display scaling primitive; arbitrary floating multipliers remain deferred. No
implicit affix spacing, unit inference or shared multiplier annotation.

Target attachments:

| Attachment | Applicability |
| --- | --- |
| `Plot<K>::x_tick_format(NumericFormat)` | Existing sealed `NumericX` capability |
| `Plot<K>::y_tick_format(NumericFormat)` | Existing sealed `NumericY` capability |
| `Plot<K>::color_label_format(NumericFormat)` | Existing `ContinuousColor`: Heatmap/Beeswarm |
| `HeatmapAnnotations::Values(NumericFormat)` | Existing Heatmap annotations |
| `Plot<kind::Coefficient>::estimate_labels(NumericFormat)` | Existing point-only restriction retained |
| `Plot<kind::Waterfall>::total_format(NumericFormat)` | Numeric part only; `WaterfallLabels` still owns names |

No blanket setters on erased `kind::Any`. A capability permits families that can
have numeric axes; configuring an actual categorical axis rejects before fonts.
An exact tick list takes precedence over its interval/formatter regardless of
setter order and contributes only its actual text, not suppressed format glyphs.

### Numeric Conversion And Precision

For ordinary finite binary64 inputs, format the exact represented rational times
`10^scale`, rounding once with nearest/ties-to-even. Never multiply a scientific
value by an intermediate binary64 scale or parse label text back into data.
Use exact decimal coefficient/exponent arithmetic and admitted scratch with the
existing integer dependency; derive bounds from binary64/precision/scale domains.

Scientific spelling is one integral mantissa digit, lowercase `e`, no positive
exponent sign or leading exponent zeros. Engineering exponent is divisible by
three and nonzero mantissa is in `[1,1000)`; a rounding carry renormalizes it.
Fractional precision applies to the displayed mantissa for exponent notations;
significant precision applies to the whole numeric value. Engineering always
emits its exponent, including `e0`, without automatic SI prefixes.

| Policy | Exact input | Expected spelling |
| --- | ---: | --- |
| Decimal fractional 1 | `1.25` | `1.2` |
| Scientific fractional 2 | `12345` | `1.23e4` |
| Engineering fractional 2 | `12345` | `12.34e3` (nearest-even tie) |
| Engineering significant 3 | `999.5` | `1.00e3` |
| Decimal significant 3 | `12345` | `12300` |
| Decimal fractional 1, scale 2, suffix `%` | `0.125` | `12.5%` |

Negative-zero normalization applies after rounding, including a negative value
rounded to zero. Trimming removes only fractional trailing zeros/empty decimal
point, never integral place-value zeros, exponents or affixes. Significant zero
has one integral zero and `p-1` fractional zeros before trimming.

Exact precision never increases or switches notation. Generated axes reject a
nonunique complete candidate through their existing source-miss ordering;
continuous guides reject an indistinguishable required set as a pre-font data
presentation error. Exact value labels/totals may intentionally share rounded
text. Geometric fit is separate from numeric distinguishability.

Adaptive precision formats the complete owner-local set at `min`, then increments
uniformly through `max` until distinct scientific values have distinct labels.
It never changes notation or scale. Equal scientific values, including signed
zero, may repeat. Sets are one axis candidate, final continuous guide, prepared
Heatmap panel values, prepared Coefficient panel estimates or semantically
window-eligible Waterfall totals. Value-set membership follows prepared scientific
inputs, not incidental ink or fit. Waterfall may select another pre-admitted
eligibility mask after a legitimate domain consequence; reachable masks, their
results/refusals and potential font requirements are prepared before fonts.
Exhaustion is an axis source miss or pre-font presentation error
for guides/values/totals. Resource refusal is immediately fatal. Large set
ordering and comparisons require fallible admitted work; no unpriced sort or
quadratic scan is permitted.

### Certified Forest Log10 Operand

The user-selected contract formats `10^(exact(e)+scale)`, where `e` is the selected
finite binary64 logarithmic tick position. Precision applies to that mathematical
positive value. For exponent `0.5`, fractional 2 yields `3.16`; scale 2 plus `%`
yields `316.23%`. Tick projection and source identity remain exponent-space.
Do not claim inversion of a rounded `log10` recovers every original observation.

The numerical design gate is closed by `07_LOG10_DISPLAY.md`, including exact
recurrences, accumulated-error/convergence proofs, scratch/capacity bounds and
independent root-oracle certificates. Implementation/certification remains a
separate required gate before integration. Its design summary is:

1. Decode the exact dyadic exponent, add decimal scale exactly, and reduce to
   `E=n+f`, `n=floor(E)`, `0<=f<1`. Retain a signed arbitrary-size integer `n`;
   no narrowing or exponent clamp. Integral powers use symbolic exact decimal
   formatting and admit output length before generating zeros.
2. For fractional powers, bound `ln(10)=3*ln(2)+ln(5/4)` with the existing certified
   integer-interval machinery. Bound `x=f*ln(10)/16`, so `0<=x<1/4`.
3. Compute outward fixed-point bounds for a positive exponential series. Bound
   its omitted tail by twice the first omitted term; four outward squarings
   enclose `10^f`. The design must prove each recurrence, tail inequality,
   accumulated rounding bound and enclosure convergence.
4. Quantize both interval endpoints directly to the requested decimal result.
   Accept only when they round identically; otherwise increase precision through
   checked precision growth. No midpoint approximation, `powf`, temporary
   binary64 value or best-effort label after refusal.
5. Integral cases are exact; a nonintegral rational exponent yields an irrational
   power, so it cannot equal a rational decimal tie boundary. Convergence gives
   eventual certification when the required storage is representable and available.
   Real allocation/arithmetic refusal stays fatal and
   cannot trigger axis coarsening.
6. Derive temporary-size/output-length admission bounds for dyadic numerators,
   series products, squares, decimal quantization and large exponents. Do not
   copy unrelated `log10` scratch constants or reintroduce accounting quotas.

Use an independent Rust oracle based on repeated certified integer square roots
of ten and exact interval products, with separately verified integer-square-root
bounds, plus exact power comparisons for small-denominator cases. Record exact
exponent bits, precision/scale, expected decimal results and oracle provenance.
No Python support files, network or new numeric dependency. The document-level
proof is complete; executable independent vectors and admission tests must still
certify the implementation. No mathematical outline is substituted for those tests.

All selected finite exponents are covered, including original-domain midpoint
sources beyond the integral-seed clamp. Scientific output may represent powers
outside binary64's finite value range. Unrepresentable/unadmitted Decimal output
refuses before giant allocation. Adaptive label uniqueness remains bounded by
public precision: different exponents can remain indistinguishable at precision
17, in which case the axis candidate misses normally.

### Defaults, Fonts, Identity And Composition

Absent overrides preserve existing consumer semantics through private built-in
profiles of the one engine: automatic magnitude/`k`/`M`/uniqueness grammar,
Forest power grammar, fixed value precision 2 and Waterfall precision 3. Preserve
the established default conversion operation order where it affects bytes;
custom display scaling has the exact rule above. There is no second legacy
formatter or public compatibility selector. `NumericFormat::default()` retains
the existing fixed-value default, including retained zeros and signed zero.

Prepare effective policies, applicability, guide/value adaptive sets and actual
affixes before fonts. Later axes contribute policy-derived closed alphabets;
affixes contribute real source spans for combining-mark fallback. Known numeric
text contributes its complete concatenated formatted span. For later-generated
axes, admit every possible terminal numeric scalar joined to the suffix's leading
combining-mark span, and any other applicable prefix/numeric boundary unit,
derived from the finite grammar rather than guessed sample numbers. Prepare the
requirements before font census; select and retain whole-unit or permitted
per-scalar face identities during canonical font resolution, before measurement.
Independent digit/suffix scalar coverage cannot change the whole-unit preference.
These requirements price their
actual strings/units without enumerating numeric values. Domain feedback
may change labels/precision, never introduce undeclared glyphs or rechoose root
width. Use the same operand/policy in terminal intrinsics, natural gutter
preferences, axis/guide measurement, annotation fit and frozen replay.

Include full normalized policy and operand context in prepared/scoped copies,
child/axis/intrinsic cache identities and feedback. Shared continuous contributors
must have identical effective policies; reject mismatch before fonts, then
format one set from the final merged domain. Canvas range sharing alone permits
different panel presentation; a genuinely shared displayed axis requires one
compatible effective format. No first-contributor-wins or Canvas format override.

### Numeric Leaves And Checks

| Stage | Status | Depends on | Deliverable / scoped commit |
| --- | --- | --- | --- |
| T6N0d | COMPLETE | T6A | Complete numeric consumer/default/transport/font/key design in `06_NUMERIC_PRESENTATION.md`; `docs(numbers): pin numeric presentation consumers and defaults` |
| T6N0 | PENDING | T6C, T6N0d | Refresh source names/callers and verify the closed contract against implemented foundations/sizing; implementation handoff, not new product design |
| T6N1 | PENDING | T6N0 | Exact finite-binary64 decimal conversion, scaling and admitted rounding; `feat(numbers): implement exact scientific decimal formatting` |
| T6N2a | COMPLETE | T6N0d | Mathematical interval/rounding/termination, resource and independent-oracle design in `07_LOG10_DISPLAY.md`; document-level proof only; `docs(numbers): prove logarithmic display rounding and resource bounds` |
| T6N2b | PENDING | T6N1, T6N2a | Exact exponent reduction, symbolic powers/output size; `feat(numbers): retain exact Log10 display operands` |
| T6N2c | PENDING | T6N2b | Outward mantissa bounds and decimal certification with independent vectors; `feat(numbers): certify fractional Log10 labels` |
| T6N3 | PENDING | T6N2c | One normalized private format/value implementation; migrate fixed values/default grammar onto it; `refactor(numbers): unify numeric presentation owners` |
| T6N4a | PENDING | T6N3 | Numeric/Forest axes, alphabets, intrinsics and natural width; `feat(axes): measure configured numeric formats` |
| T6N4b | PENDING | T6N4a | Continuous local/shared guides and policy compatibility; `feat(guides): coordinate scientific numeric formats` |
| T6N4c | PENDING | T6N4b | Adaptive prepared value sets and total shelves; `feat(annotations): format prepared numeric values and totals` |
| T6N5 | PENDING | T6N4c | Facet/Canvas/cache/domain-feedback identities; `test(numbers): close composed format identities` |
| T6N6 | PENDING | T6N5 | Public NumericFormat/attachments, remove NumericLabelFormat and migrate callers/current contracts; `feat(api): expose unified scientific numeric formatting` |
| T6N7 | PENDING | T6N6 | Scientific conversion/admission/all-surface/public-output evidence; `test(numbers): close scientific presentation contracts` |

Tests must independently cover rounding ties/carries, subnormals/finite extremes,
scale extremes without overflow/underflow, exact versus adaptive collisions,
actual/rounded negative zero, prefix/suffix Unicode spans,
numeric-affix boundary units where scalar and whole-unit fallback choose different
faces, Forest refinement and fatal refusal, original midpoint scope,
exact-tick precedence, actual categorical-axis rejection, same-domain policy
invalidation, Shared conflict, width-dependent
fit, cached/uncached output and frozen SVG/PNG. No extra full suite per numeric leaf.

## T6L — One Section-Aware Discrete Guide

`08_DISCRETE_GUIDES.md` closes the actual producer matrix, typed section/entry
schema, mapping masks, layer/literal routing, Shared compatibility, finite-height
flow examples and every flat-storage/census/cache/result deletion. Its design is
source-reviewed; the new representation and acceptance tests are not implemented.

### Public Authority And Representation

Expose `LegendComposition::{Combined,Separate}` (default Combined),
`Plot<K>::legend_composition` under the existing `kind::Legend` capability, and
`Canvas::legend_composition`. Policy is independent of appearance/placement.
Standalone/facets use the owning Plot selection; PerPlot uses each Plot's;
Shared uses Canvas's selection for all contributors. Child selections remain
stored but are not conflicts under Shared. Hidden entries have no factorization
or font work, while visible marks retain existing color-domain coordination.
Setters replace only their own value and are order-independent.

Replace the flat descriptor with one ordered typed section representation for
both policies. Section identity owns title compatibility; entry identity owns
deduplication. Combined preserves actual pairs, complete glyphs, same-field
collapse and current shared-primary-title compatibility. A separate guide is
still one outer frame, one collision obstacle, one measured owner and one frozen
plan. Continuous guides are not sectioned; discrete/continuous mixing remains
rejected.

### Truthful Factorization

Prepare mapping provenance and keys before fonts. Group-style projections retain
actual shape/pattern, component presence and constant geometry/paints; only
hue-mapped RGB is neutralized to the RGB of the owning Plot's resolved
`TypographyTheme` legend-label text paint, retaining the mapped paint's original
alpha rather than the text paint's alpha. This is a named complete role, not the
retired generic foreground carrier or Canvas's label paint. Hue projections
retain actual resolved RGB/alpha and geometry;
only group-mapped shape/pattern is neutralized to the producer's fixed default
(Interaction circle/solid). Canvas never reconstructs keys from its palette.

Projections of the same raw group/hue must agree across all observed occurrences.
No representative-pair guess or inference from equal colors. Repeated explicit
colors/styles remain allowed; composition does not promise perceptual injectivity.

| Actual encoding | Separate result |
| --- | --- |
| Interaction independent group styles and hue | Group-style and hue-color sections |
| Interaction group owns both style and group color, or same-field group/hue | One complete joint section |
| Independent pair identities without a mapped group-style channel | Explicit pre-font unsupported-factorization error; do not invent styles |
| Hue-only family or positioned categorical series | One truthful color section with complete producer glyph |
| Trajectory/neutral LinePath with row hue | Hue point section; no invented group section or hue-colored connector |
| Existing literal/reference/sign guide domains | Preserve typed domain and exact keys |
| No guide contribution | No empty section or detached title |

This supports every admitted guide owner while retaining explicit truthful
rejection for encodings that cannot be decomposed. It adds no general group-style
mapping to Line, Density, Marginal or Matrix.

### Ordering, Labels, Layers And Shared Compatibility

Separate section classes are group-style, joint, hue-color, literal, supplemental
series. Within each class use canonical first contributing occurrence. Plot
values follow prepared observed order; facets retain parent domain order;
Canvas unions use row-major contributor order. Sections explain marginal
mappings, not the observed pair support or a new Cartesian product.

Same-field group/hue collapses; raw identities remain separate from display
aliases. Reject conflicting labels or duplicate displayed identities within a
section. Equal entry labels in differently titled sections are permitted;
distinct section identities with equal nonempty display titles reject. No new
field-scoped alias API.

Labeled PointLayers get their existing typed layer-label identity, never a hue
value or append-slot identity. Combined retains appended layer keys under its
primary title (or literal `series` when alone). Separate gives them a final
`series` section. Preserve existing label-resolution behavior, source order and
accent-slot consumption; equal layer identity with unequal full glyph rejects.

Separate Shared unions typed section identities and exact titles. Repeated entry
identities require identical labels and complete projected keys, including
neutral group paint, alpha, outlines and geometry. Different group fields can
coexist with a common hue field if headings are unambiguous. Cross-family equality
requires matching complete keys, not only RGB. A joint section and a hue section
are different explanations and do not silently merge. Combined retains the
existing one-compatible-primary-domain restriction.

### Deterministic Measurement

Stack sections vertically in semantic order in both orientations. Every section
gets the full inner width `W`; one outer frame/padding is measured once. Section
gap is the owning theme's `title_gap + row_gap`, reusing existing admitted spacing.
No per-section position, permutations or extra local trial budgets.

For each section, measure its title at `W`; an empty title has no active title
gap. Entry key/label widths, baseline heights and hard minima use the current
complete-ink rules. Preferred horizontal entries use `min(P[i],W)`; preferred
vertical columns use `min(max(P[i]),W)`. Compact uses the same existing
`C2=floor((W-column_gap)/2)` limit and remeasures labels. Duplicate allocations
reuse the outcome. A variant is global to the guide, not independently selected
for each section.

Horizontal section rows are width-derived; compose all heights and test the
total. Vertical automatic-height sections use one column each. For finite
vertical inner height `H`, after entry measurement define:

```text
title[s] = measured title height + active title gap
hard[s] = title[s] + max(entry heights in section s)
preferred[s] = title[s] + sum(entry heights) + row gaps
available = H - section_gap * (section_count - 1)
```

If available is below summed hard heights, miss. Otherwise use one existing
proportional hard-to-preferred track allocation; capacity above summed preferred
is slack. Greedily form equal-width columns within each assigned section body
capacity, then check all columns fit `W`. Stack actual occupied section heights;
do not redistribute unused capacity or retry sibling allocations. This remains
a deterministic policy, not a guarantee to discover every feasible arrangement.
Single-section flow retains its current semantics. The outer guide still shares
the existing at-most-eight automatic edge/flow trials per root round.

### Lifecycle And Leaves

Census all section titles/entries in final semantic order. Section records have
stable ordinals beneath the existing guide address and are inline measured work,
not separate placement owners. Admit projections, lookups, keys, strings,
buffers and flow records through the same cumulative ledger. Keys include
composition, ordered typed sections, full glyphs, typography, constraints and
consumed feedback. Freeze section boxes, texts, draw operands and complete outer
ink/collision bounds. No draw-time factorization or layout reconstruction.

| Stage | Status | Depends on | Deliverable / scoped commit |
| --- | --- | --- | --- |
| T6L0d | COMPLETE | T6A | Complete producer/mapping/title/layer and measured-section design in `08_DISCRETE_GUIDES.md`; `docs(legend): pin discrete guide producer and section contracts` |
| T6L0 | PENDING | T6N, T6L0d | Refresh actual producer/descriptor/key names after preceding migrations; verify the closed design against those sources, without reopening product choices |
| T6L1 | PENDING | T6L0 | Typed section descriptor, Combined consumer migration and layer identity; `refactor(legend): retain typed discrete sections` |
| T6L2 | PENDING | T6L1 | Producer-owned factorized keys and pre-font validation; `feat(legend): prepare factorized channel sections` |
| T6L3 | PENDING | T6L2 | One bounded section flow/frame/collision plan; `feat(legend): measure bounded section flow` |
| T6L4 | PENDING | T6L3 | Facet/Canvas authority, exact merging, fonts, cache and feedback; `feat(composition): coordinate sectioned guides` |
| T6L5 | PENDING | T6L4 | Public selection and atomic current-contract/examples migration; `feat(api): expose legend composition policy` |
| T6L6 | PENDING | T6L5 | Full local/shared/fit/resource/output evidence; `test(legend): close section composition contracts` |

Retain current combined pair/same-field/sparse/row-hue/layer/conflict tests. Add
dense independent-style crossings, sparse marginal-domain semantics, complete
neutral glyph geometry, unsupported factorization, mixed-theme Shared conflicts,
custom themes whose legend-label paint differs from axis/annotation roles,
distinct sections/common hue, title ambiguity, Hidden and setter-order behavior,
normal/compact finite-height worked examples, no redistribution, one-quantum
fit boundaries, inside collision, changed section identity/cache, all root modes
and identical retained SVG/PNG with reuse disabled. One owner budget and unchanged
resource caps are mandatory; section count does not multiply placement trials.

## Verification And Closure

Documentation-only stages need `git diff --check` and source/spec consistency
review, not test execution. Code stages require the narrowest affected unit tests
and representative contract integrations plus `just fmt-check` and strict
`just lint` (`cargo clippy --locked --offline --all-targets -- -D warnings`). Never
substitute `cargo check`. Include compile-pass/fail coverage for sealed public
capabilities and private module/value boundaries.

T6Z3 is the required broad-phase closure gate: state this reason, run `just verify`
and the ten ignored release benchmark groups, preserve full long-caption,
legal-break, dense/compact/refusal and source fixtures, and report actual records,
visits, calls, cache, topology, bytes and retained output identities. Add bounded
numeric certification/format-set and sectioned-guide workload samples to the
relevant Rust benchmark modules; baseline speedups are not inferred from fewer
logical units. After a localized failure use its focused tests and relevant
neighbors rather than repeating successful full suites.

Final review must reconcile all 25 scientific kinds and valid parents, public
custom-value behavior, current exports/callers, property-owner exclusions and
actual numeric/guide policy consumers. Added semantic policies do not silently
change the historical visual-property count; record justified ledger additions
or splits with their owners. Remove every replaced selector, tuple/opacity
carrier, formatter and flat-guide representation. No public target-only aliases.

T6 completes only after DF3/DF4 decisions, public/current specifications and
examples, verification, inspected snapshots, evidence and commits. T7 remains
PENDING until then. The new residual DF2/DF5/DF7 T7 closure reviews are explicit
review obligations, not automatic successor implementation.
