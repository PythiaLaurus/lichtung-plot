# Data Alpha And Numeric Tick Selection

**Status**: User-approved scope; numeric tick selection COMPLETE, including design,
bounded sources, public cutover and focused parent/root/frozen-output closure.
Data-alpha source-backed design, private owned mapping values and explicit
binding/source-slot preparation are COMPLETE. Mark consumers are IN PROGRESS;
guides, public cutover and integrated release verification remain PENDING.

## Authority And Recorded Decision

At the T6 closure review, the user requested comparison with established plotting
libraries, discussed custom alpha functions and continuous versus discrete guide
semantics, then authorized proceeding with DF3/DF4 according to that discussion.
The user separately directed fully caller-decided legends to the existing DF11
AI-authored-workflow review. That review retains its post-first-release checkpoint
and all five prerequisites in `91_IMPLEMENTATION_PLAN.md` and
`99_DEFERRED_FEATURES.md`.

The approved dispositions are:

| Item | Disposition | Owning work |
| --- | --- | --- |
| DF3 data-mapped alpha | ADMIT caller-computed alpha, built-in linear/log mappings, and automatic explanation only from declared mapping semantics | T6D below; remove DF3 from the deferred list now |
| DF4 numeric tick counts | ADMIT target and maximum generated major-tick counts, alongside existing preferred intervals and exact caller ticks | T6T below; retain only the residual excluded controls under DF4 |
| Residual DF4 | Keep exact generated counts, density scalars, arbitrary ratios, categorical stride/drop and generation callbacks outside the admission | Review at T7 CLOSURE, before successor planning |
| Fully caller-decided legends | DEFER to the existing AI-workflow review, including data-independent entries and the broader custom-drawing question | DF11; not a new T6/T7 closure requirement |

This closes the DF3/DF4 disposition requirement, not feature implementation or
integrated T6 verification. `93_T6_EXECUTION_PLAN.md` owns execution status. The
current APIs and rendering contracts remain in specs01/02/03/05/08 until the
corresponding implementation cutover; this plan exports no placeholder type,
callback, legend-entry carrier, or alternate renderer.

## Research Basis

Official references establish the relevant distinctions:

- [ggplot2 alpha scales](https://ggplot2.tidyverse.org/reference/scale_alpha.html)
  provide continuous, binned, manual and identity mappings. Their existence does
  not establish actual usage frequency or justify guessing an input's semantics.
- [Observable Plot mark channels](https://observablehq.com/plot/features/marks#mark-options)
  accept fields, accessor functions and arrays; its
  [scale options](https://observablehq.com/plot/features/scales#scale-options)
  distinguish channel values from linear/log mappings and domain/range choices.
- [Vega-Lite mark-property channels](https://vega.github.io/vega-lite/docs/encoding.html#mark-prop)
  distinguish values, fields, scales and explanatory legends.
- [ggplot2 continuous scales](https://ggplot2.tidyverse.org/reference/scale_continuous.html),
  [D3 ticks](https://d3js.org/d3-scale/linear#linear_ticks), and
  [Vega-Lite tickCount](https://vega.github.io/vega-lite/docs/axis.html#ticks)
  treat a requested count as a target that may change for nice values.
- [Plotly nticks](https://plotly.com/python/reference/layout/xaxis/#layout-xaxis-nticks)
  specifies an automatic-mode maximum. In contrast,
  [Matplotlib MaxNLocator](https://matplotlib.org/stable/api/ticker_api.html#matplotlib.ticker.MaxNLocator)
  primarily counts intervals, admits up to `nbins + 1` in-range ticks, and can
  relax constraints for its minimum-tick requirement. Do not conflate these APIs.

These references motivate the approved product scope. They do not authorize
importing another renderer's defaults, callback lifecycle, opacity multiplication,
category omission, layout search, or dependencies.

## Numeric Tick Selection

### Admitted Semantics

- A target requests approximately that many generated major-tick positions.
  Nice-step selection and measured fit can produce a different count.
- A maximum is a hard upper bound on generated in-domain major-tick positions,
  including any in-domain endpoints. It is not an interval count or a cap only
  on nonempty painted labels. Coarsening considers complete candidate sources;
  it does not hide labels from an otherwise selected source.
- Existing exact `AxisTick` lists remain indivisible, verbatim and authoritative
  in either setter order. Generated-count instructions do not constrain an exact
  caller list or change scientific limits, coordinate transforms or formatting.
- Target, maximum and preferred interval belong to one per-axis generated-source
  selection. Define complete replacement semantics and one retained value before
  implementation; do not accumulate competing optional fields or precedence
  fallbacks. Any replaced project-owned API and its callers migrate atomically.
- Keep the existing NumericX/Y capability gates and actual-axis validation.
  Categorical axes do not acquire count/stride/drop behavior. Cover linear axes
  and Forest Log10, original-domain midpoint handling, absent-selection behavior,
  scientific range provenance and bounded real source cardinality.
- Facets and Canvas retain the complete instruction in prepared copies, intrinsic
  and assigned measurement, child/height identities and feedback. Shared ranges
  alone do not turn every child into an identical tick-presentation owner.

### Design And Acceptance Gate

Before changing production representation, close constructor validation, exact
public signatures, replacement/deletion inventory, deterministic candidate order,
nice-step ties, hard-maximum filtering, terminal behavior and the complete consumer
matrix against current source. Reuse the existing numeric source/format/fit route.
Prove target deviation and strict maximum independently, including endpoint counts,
narrow panels, extreme/adjacent finite values, Log10, exact precedence, actual
categorical refusal before fonts, setter replacement, parent transport and actual
cached/uncached frozen output. No new root search or resource quota follows.

## Data Alpha

### Admitted Inputs And Execution Boundary

Accept immutable caller-computed alpha values and validated built-in linear/log
mapping factories. Custom calculations execute in caller-side or input-construction
work and are materialized before Plot/Canvas rendering. Retained plots, preparation,
layout, capture and Output replay do not store or invoke application closures.
Resolve once to the existing byte-alpha paint model; do not create a parallel
floating-opacity draw route.

Close domain/range, constant input, missing/nonfinite input, log-invalid values,
out-of-domain behavior, quantization and exact-paint precedence in the source-backed
design. Each final fill or stroke has one alpha resolved by explicit replacement,
never multiplication; independent outlines and text remain independently owned.
Alpha zero does not itself delete a scientific observation or remove it from a
scientific domain.

The user subsequently selected **explicit semantic-unit inputs** after reviewing
Scatter, Line, Bar and Violin examples: individual observations receive individual
alpha values; series and aggregate marks receive explicitly keyed per-series or
per-group values. These inputs are independent when a family displays both raw
observations and collective geometry. Do not require repeated equal row values as
the aggregate input, and do not infer mean/first/last alpha from contributing rows.

### Mapping Semantics And Automatic Explanation

Mapping rules, source semantics and guide presentation are distinct:

- Declared continuous mappings carry the source identity, domain, transformation
  and the information needed for a truthful numeric explanation.
- Declared categorical or threshold mappings carry the actual categories or
  intervals, labels/order and their alpha association.
- A custom function or final-value array alone does not declare either semantic
  kind. Without explanatory semantics, do not automatically invent an alpha guide.
- Never infer continuity from a numeric return type, input/output unique counts,
  a small sample or alpha quantization. A continuous mapping can be illustrated
  by numeric representative symbols without becoming a categorical mapping.
- If custom construction supplies explanatory semantics, materialize the required
  guide operands there too. Do not infer thresholds, an inverse or a smooth ramp
  from an opaque function, and do not interpolate a misleading continuous rule
  merely because the observed samples permit it.

Automatic guide entries remain consequences of the declared source mapping.
Independent caller-selected legend contents, unrelated explanatory items, arbitrary
symbols/graphics and a manual append/replacement API belong to DF11. This admission
does not introduce that manual legend surface through alpha metadata.

### Design And Acceptance Gate

Inventory all actual mark consumers before adding public setters. Specify which
semantic unit is mapped for raw observations, series and aggregate marks; preserve
source-row alignment through finite filtering, facets and prepared/supplied inputs.
Do not invent a mean, first-row representative or per-series reduction merely to
make a row-valued channel fit an aggregate producer. Pin the precise admitted
family/component capability matrix and explicit unsupported cases before coding.

Close mapping/guide ownership for Plot, owning facets, Canvas PerPlot/Shared/Hidden,
source-domain coordination, key projections when hue and alpha coexist, continuous
versus discrete explanation, compatibility, canonical ordering and font census.
Use complete immutable values in cache/dependency/feedback equality. Preparation,
measured hard bounds and selected output consume the same mapping/descriptor;
freeze final RGBA and guide/text operands through the existing lifecycle.

Acceptance covers independent endpoint/interior mapping expectations, byte rounding,
invalid and missing input, exact RGBA/alpha precedence and setter order, complete
source membership, actual component paint parity, declared versus undeclared guide
semantics, parent compatibility/suppression, same-extent cache invalidation and
source-drop/frozen SVG/PNG equality. A source-backed design, not this broad scope
record, fixes public constructors, numerical rules and guide representation.

The completed exact design is `../docs/data-alpha-design.md`, inspected at
`f97bd8c`. It fixes the two opaque input values, explicit semantic-unit binding,
three independent paint targets and 49 family/component attachments, numerical
rules, immutable custom construction and declared explanations. It also owns the
unified guide-envelope replacement, native color-plus-alpha composition boundary,
actual producer/row-slot census, parent/key/font/feedback transport and public
migration/deletion obligations. It is a design gate, not a current alpha API.

## Execution And Verification

Finish the current section-guide acceptance leaf before the extensions. Implement
numeric tick selection first, then data alpha. Each umbrella has an independently
committed design/consumer handoff, behavioral implementation, atomic public cutover
and focused closure; split behavioral leaves further when needed.

| Stage | Status | Depends on | Deliverable |
| --- | --- | --- | --- |
| T6T0 | COMPLETE | T6L | Exact generation value/API, source order/cardinality, consumer/deletion and acceptance design in `../docs/numeric-tick-selection-design.md` |
| T6T1 | COMPLETE | T6T0 | Private target/maximum sources, pre-allocation cardinality and exact identity/feedback; evidence in `../docs/bounded-tick-count-sources.md` |
| T6T2 | COMPLETE | T6T1 | Atomic TickGeneration API/caller/current-contract migration; evidence in `../docs/public-tick-generation.md` |
| T6T3 | COMPLETE | T6T2 | Complete parent height identities, all-family/root and 60 cached/uncached multi-format publications; `../docs/tick-generation-parent-closure.md` |
| T6D0 | COMPLETE | T6T | User-selected explicit units, exact inputs/numerics, 49 component gates, unified guide/parent and migration design in `../docs/data-alpha-design.md` |
| T6D1 | IN PROGRESS | T6D0 | Immutable mapping preparation and actual mark consumers |
| T6D1a | COMPLETE | T6D0 | Owned AlphaValues, common scalar/label owners, eager custom construction and complete selected recipes; `../docs/owned-alpha-values.md` |
| T6D1b | COMPLETE | T6D1a | Explicit encodings, exact keyed/bin/facet resolution, original slots, retained three-target input and complete child/height identity; `../docs/explicit-alpha-bindings.md` |
| T6D1c | IN PROGRESS | T6D1b | Actual observation/estimate/strip point consumers and paint-aware intrinsic/ink requirements |
| T6D1d | PENDING | T6D1c | Actual series, trace and uncertainty stroke consumers with explicit units |
| T6D1e | PENDING | T6D1d | Actual aggregate, cell, bin, band, native and supplied-fill consumers |
| T6D2 | PENDING | T6D1 | Declared automatic guides, parent transport, census and identities |
| T6D2a | PENDING | T6D1 | Sole typed guide envelope, actual alpha descriptors/projections and exclusive-kind retirement |
| T6D2b | PENDING | T6D2a | Common section measurement/freeze, scoped parent ownership, cache/height/feedback and font census |
| T6D3 | PENDING | T6D2 | Atomic public API/caller/current-contract migration |
| T6D4 | PENDING | T6D3 | Full admitted-family/parent/root/frozen-output evidence |

The observation-consumer leaf closes Scatter, Qq and Regression in
`../docs/alpha-coordinate-points.md`, and Line/Trajectory/LinePath, Interaction and
PointInterval/Coefficient/Forest in `../docs/alpha-path-estimate-points.md`:
original slots through filtering/sorting/order/status, final-paint batches and
paint-aware intrinsic/ink. Ten of seventeen admitted point families are wired.
Remaining Dumbbell, categorical and native/supplied consumers keep T6D1c IN PROGRESS.

Do not start a code leaf while its exact owning design is still pending. Use modern
Rust module layout, the standalone offline renderer and existing pinned dependencies.
Every new library unit test replaces an existing one in the same logical change;
the discovered Linux baseline remains 671 including 17 ignored, without banked
deletions or integration-suite migration. Run the narrowest affected tests and
representative caller contracts, `just fmt-check`, and strict `just lint`
(`cargo clippy --locked --offline --all-targets -- -D warnings`), never `cargo check`.
Public cutovers include compile-pass/fail, doctests, warning-denied docs and package
checks. Documentation-only leaves need source/spec review and `git diff --check`.

Reconcile specs01/02/03/05/08, examples, public exports and property/consumer
attribution when each implementation lands. T6Z1/T6Z3 include both extensions in
the existing final integrated gate and bounded release/output/time/RSS evidence;
successful scope admission does not mark either feature, T6, or T7 complete.
