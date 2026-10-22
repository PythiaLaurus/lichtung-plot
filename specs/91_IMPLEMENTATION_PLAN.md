# Maximum-Width Root Sizing

## Scope And Target Contract

Implement `Plot::max_width(u32)` and `Canvas::max_width(u32)` as a fourth root
sizing mode: a maximum outer CSS-pixel width with content-derived height. The
owning Plot request also controls its facets. This document is a plan, not evidence
that the API exists. Current contracts remain in
[rendering](02_RENDERING_CONTRACT.md) and [layout](05_LAYOUT_SYSTEM.md).

| Request | Width selection | Height |
| --- | --- | --- |
| Omitted | Natural preferred outer width, chosen once | Measured at the selected width |
| `width(w)` | Exactly `w` | Measured at `w` |
| `max_width(m)` | Natural preferred outer width capped by `m` | Measured at the selected width |
| `size(w,h)` | Exactly `w` | Exactly `h`, fit or fail |

For an ordinary representable natural width `P`, the new policy is
`W = min(P,m)`. Apply the cap to the existing checked natural outer-width
requirement before conversion to integral public dimensions. Round the selected
requirement upward using the existing CSS-dimension rule; an integral cap must
never be exceeded by rounding. A soft natural preference that exceeds `u32` must
not itself reject a representable capped result. Genuine checked arithmetic,
allocation and hard-content failures remain errors.

The cap covers the complete root: padding, prose, axes, guides, scientific body,
Canvas cells/headers and gaps. It does not name a data-panel width or a PNG target.
It is not a request to fill the cap when natural content is narrower.

Required setter and composition semantics:

- The latest `width`, `max_width` or `size` call replaces the entire root request.
  `max_width` clears an earlier exact height and exact width; bounds do not stack.
- Zero rejects during root preflight before font discovery or backend begin;
  setters do not panic. A superseded zero request has no remaining effect.
- Canvas alone assigns embedded Plot geometry. Embedded standalone requests,
  including `max_width(0)`, are ignored just like existing child width/size requests.
- The root chooses width once from prepared intrinsic preferences. Later gutter,
  alignment, ink, domain or content-height feedback cannot reselect it or raise it
  beyond the cap. Height remains content-driven under the existing eight-round
  coordinator.
- Complete required text, scientific sources, annotations and guide sections must
  fit. Preserve the selected typography, mark metrics and owner-local spacing;
  no fit-driven padding compression, font shrink, omission or root-width search.
- Output reports the actual selected canonical width/height. SVG modes and PNG
  options replay that frozen scene; PNG scaling does not rerun root layout.

This scope does not admit height-only sizing, minimum dimensions, maximum height,
public generic constraints, geometry-responsive padding or new Canvas topology.
Residual root-bound proposals remain in [deferred features](99_DEFERRED_FEATURES.md).

## Existing Owners And Required Changes

| Owner | Responsibility |
| --- | --- |
| `src/layout/kernel.rs`, `src/layout.rs` | One complete root request, mode validation, selected-width locking, final bounds validation and the existing prepare/measure/freeze lifecycle |
| `src/builder.rs`, `src/builder/configuration.rs`, `src/builder/accessors.rs`, `src/builder/facets.rs` | Plot storage, public setter, exact assigned geometry, erasure/copying and composed-input equality |
| `src/orchestrator/lifecycle.rs` | Standalone/faceted dispatch and automatic-height selection |
| `src/layout/interior/sizing.rs` | Unfaceted natural outer-width composition and selected-width height proposal, including native scientific families |
| `src/orchestrator/facets/sizing.rs`, `src/orchestrator/facets/tracks.rs`, `src/orchestrator/facets/planning.rs` | Owning-root preference, sparse declared facet columns, shared axis titles and width-dependent cell/strip heights |
| `src/canvas.rs`, `src/canvas/sizing.rs`, `src/canvas/tracks.rs` | Canvas setter, mixed/all-text preferences, headers/spans, Shared/PerPlot guides and content-height measurements |
| `src/layout/kernel/feedback.rs`, `src/canvas/dependencies.rs`, `src/decorations/plan/children.rs` | Audit actual request/constraint consumers and operation-local reuse identity |

### One Mode-Aware Root Request

Replace the current optional-width/optional-height request encoding with a closed
representation of the supported modes. The final mode set is Natural, ExactWidth,
MaxWidth and ExactSize. Keep measured/frozen dimensions concrete and checked; a
maximum is never an assigned width merely because it is present in a request.

Replace `plan_exact_root(width,height,...)` with one mode-aware root entry point,
named `plan_root(request,...)`, and migrate every repository caller. Remove the
old signature and pair-based request decoding rather than keeping an adapter or
parallel coordinator. Exact assigned-box helpers must still reject unresolved
requests. Preserve the parent's ability to install concrete child geometry.

Use one shared width-selection rule across Plot, owning facets and Canvas. Retain
the original request/cap for validation while subsequent rounds receive the
already selected width. Check the first measured result against its mode before
accepting feedback; reject later width changes and final out-of-bound results
before capture. Preserve exact-size validation and fatal/fit error distinctions.

Carry complete request identity through storage, erasure, source attachment and
the equality/feedback dependencies that consume it. Equal output dimensions do
not make a capped request interchangeable with an exact-size request. Keep the
existing one-operation cache boundary and complete assigned-geometry keys.

### Width Selection Before Height Measurement

Reuse the existing scientific, text and guide preferences. Do not add a second
definition of natural width for the capped mode. Root padding, axis estimates,
guide gaps and prose widths participate exactly once under their current owners.

The existing Canvas/facet natural-width probes call Grid allocation that continues
into width-dependent height measurement. Separate root width selection from final
selected-width height work, reusing column requirements, canonical span handling
and allocation rules. Preserve sparse/trailing tracks and the current initial
preference's exclusion of later feedback. Do not require a successful complete
uncapped root layout or conversion of uncapped width/height to public dimensions.

Preserve existing local height dependencies in the preference calculation. In
particular, `shared_axis_requirements` measures a vertical facet axis title against
the natural Grid height, and its perpendicular thickness contributes to natural
outer width. Retain the necessary scoped Grid-height/title measurement in the
preference pass; do not silently substitute a one-line title, zero height or a new
width formula. Propagate its genuine typed errors. After selecting width, measure
actual content and shared-title gutters at that width under the existing feedback
rules. This local dependency does not authorize iteration over root widths.

For an unfaceted Plot, select width after composing body/prose/guide preferences
and before `natural_body_height` and width-constrained prose measurement. Preserve
Heatmap, Beeswarm, Waterfall, Matrix and Marginal native sizing routes. After width
selection, use the current exact-width/automatic-height machinery and named
feedback rather than a second fit algorithm or an extra root operation.

## Delivery Order

### 1. Make Existing Root Modes Explicit

Replace the optional-pair request representation and coordinator signature for the
three existing modes first. Migrate all production and test callers, embedded
assignment helpers and equality consumers in the same logical change. Do not add
an unused future mode or a compatibility wrapper.

Validate exact-pair enforcement, automatic-height behavior, latest-setter
replacement, zero preflight, width locking and one prepare/capture lifecycle with
the focused checks below. Commit the completed representation change after those
checks and strict Clippy pass.

### 2. Separate Natural Width Composition From Height Work

Factor the necessary width-first paths in Plot, facets and Canvas while retaining
their current three-mode behavior. Use the same column summaries/allocator for
natural and assigned dimensions, including headers, row-header bands, shared axis
gutters, local Plot prose and guide preferences. Avoid a generic new layout DSL.

Validate current-mode semantic equivalence, sparse facet/Canvas topology,
unbreakable text requirements, native Matrix sizing and mixed/all-text Canvas
height measurement. Commit this refactoring after focused tests and strict Clippy.

### 3. Add And Document Maximum Width End To End

Add MaxWidth to the single request representation and expose both consuming
`#[must_use]` setters, following the existing `pub const fn` builder style. Apply
the common cap policy in all three root routes and enforce bounds throughout the
coordinator. Update actual request/height/cache dependencies and preflight handling.

Replace lower-value/obsolete tests with the contract coverage below and keep the
library-test budget. Update current contracts in `specs/02_RENDERING_CONTRACT.md`
and `specs/05_LAYOUT_SYSTEM.md`, API examples, `docs/getting-started.md` and any
affected README sizing guidance together with the implemented behavior. Remove
the admitted maximum-width subset from the deferred list when this plan is
approved; preserve the remaining root-bound discussion. Commit the completed
feature after focused verification and strict Clippy.

## Verification And Acceptance

### Behavioral Relationships

Use a bounded selection of meaningful contracts, not a new all-family x root x
output matrix:

1. **Nonbinding cap:** compare the current natural route and a larger cap on the
   same source/theme. Retain natural dimensions and complete semantic content;
   do not pin a renderer revision's automatic size.
2. **Binding cap:** use reflowable content whose natural width exceeds the chosen
   explicit cap. Actual width equals that cap, content stays complete and measured
   height is positive. Compare dimensions/content with the current `width(cap)`
   route rather than a copied layout algorithm.
3. **Request replacement:** cover replacement in both directions with width and
   size, including superseded zero requests and clearing an earlier exact height.
4. **Validation and refusal:** root `max_width(0)` fails before font/backend work;
   a positive cap below a real unbreakable requirement fails before publication
   without truncation, partial output or modification of an existing destination.
5. **Width-first numerics:** independently exercise an integral cap at a fractional
   quantum boundary and a checked soft preference above the public dimension range.
   The cap is applied before integral conversion; real arithmetic failures remain
   fatal. Test width-selection behavior through the root boundary, not helper
   setter/getter echoes.
6. **Root coverage:** representative standalone coordinate, native Matrix and one
   width-sensitive native guide route; owning sparse/wrapped facets with multiline
   shared axis titles; mixed Canvas with spanning headers and Shared/PerPlot guide
   consumption; all-text Canvas. Verify root bounds and complete sources without
   changing declared topology.
7. **Embedded requests:** Canvas allocation ignores child standalone maximum-width
   requests, including zero; compare with the equivalent child without that request.
8. **Lifecycle and outputs:** retain one scientific/font preparation, selected
   capture and final validation; freeze a width that stays fixed across feedback.
   Repeated same-Output SVG replay is equal, source-sized PNG follows actual root
   dimensions and explicit PNG scaling changes only raster dimensions.

Keep tests in their existing logical owners:

- `src/layout/kernel/tests.rs`: modes, invalid requests, bounds, quantization,
  selected-width locking, convergence and capture boundaries.
- `src/layout/interior/sizing.rs` and `src/canvas/sizing/tests.rs`: width-first
  preference/height dependencies where private behavior needs direct evidence.
- `src/orchestrator/tests/lifecycle.rs`: early zero refusal and one frozen lifecycle.
- `tests/layout/root_sizing.rs`: externally compiled sizing, complete sources,
  representative root owners and SVG/PNG publication relationships.

Record same-platform discovered library counts before and after each test-changing
step. Current Linux ceiling is 671 including 17 ignored; each new unit-test function
must replace an existing one in the same logical change. Name replacements in the
change review and remove obsolete helpers. Do not evade the limit by merging
functions, packaging cases or moving unit responsibilities into integration tests.

### Focused Commands

Use the narrowest applicable selections at each code-changing step and confirm
nonzero execution. If a named test is deliberately replaced, use its replacement
filter. Do not claim discovery as behavioral execution.

```sh
cargo test --locked --offline --lib layout::kernel::tests::
cargo test --locked --offline --lib layout::interior::sizing::tests::
cargo test --locked --offline --lib canvas::sizing::tests::
cargo test --locked --offline --lib zero_sizing_requests_refuse_before_font_discovery_or_backend_begin
cargo test --locked --offline --lib standalone_terminal_resolves_and_plans_once_with_one_final_lifecycle
cargo test --locked --offline --lib chained_terminals_reuse_one_canonical_scene_and_plan_each_png
cargo test --locked --offline --test layout root_sizing::standalone_sizing_calls_replace_the_complete_request_before_frozen_publication
cargo test --locked --offline --test layout root_sizing::automatic_roots_expand_for_complete_unbreakable_text_and_exact_width_refuses_it
cargo test --locked --offline --test layout root_sizing::owning_facets_keep_sparse_wrap_and_complete_prose_in_all_public_root_modes
cargo test --locked --offline --test layout root_sizing::matrix_roots_keep_native_cells_and_their_hue_guide_in_all_public_root_modes
cargo test --locked --offline --test layout root_sizing::mixed_canvas_owns_all_modes_and_ignores_standalone_child_requests
cargo test --locked --offline --test layout root_sizing::all_text_canvas_reflows_complete_content_and_replaces_root_requests
cargo test --locked --offline --test composition facets::faceted_shared_axis_titles_preserve_explicit_multiline_text
cargo test --locked --offline --test composition facets::shared_facet_axis_titles_clear_outside_legends
just fmt-check
just lint
```

Run the newly written/replacement maximum-width witnesses as well as affected
existing callers; the commands above are not substitutes for new-contract coverage.
Select existing guide/cache neighbors only when their dependency paths change or
a failure exposes an interaction. Execute affected public doctests and `just docs`
when adding API documentation/examples. Do not run the heavyweight large-SVG/PNG
boundary workloads merely because they share the root-sizing test file.

No automatic full-suite gate is required for this bounded feature. Expand beyond
the focused set only for a demonstrated uncovered invariant or an explicit request,
following [development](90_DEVELOPMENT.md). Documentation-only planning requires
source/reference consistency and `git diff --check`, not compiler/test execution.
