# Content-Derived Root Sizing

## Public Request And Single Representation

Plot and Canvas now store the existing private `layout::RootSize` directly.
Omission means automatic width and height. Consuming `width(u32)` requests exact
CSS-pixel width and content-derived height; `size(u32,u32)` requests an exact pair.
Each setter replaces that whole value, including clearing an earlier height when
width follows size. Zero dimensions refuse at preflight before font discovery or
backend begin, without setter panics.

The old nominal Plot/Canvas dimension fields and pre-render Plot dimension getters
are removed. Exact-family erasure, parent input retention and complete child-key
comparison carry the same request. A root or parent allocation installs the exact
assigned pair before private geometry consumers read `canvas_dimensions`; an
unassigned request cannot masquerade as measured geometry. Canvas continues to
own embedded dimensions, independently of each Plot's standalone request.

Both public roots use the existing prepared three-mode engine, natural preference,
fixed-width reflow, bounded feedback and frozen capture. There is no hidden
640×480/960×640 trial, alternate renderer, public request type or compatibility
getter. The test-only `PlotPlan::new_with_sizing` route and Canvas sizing-wrapper
entry point are removed; their callers use the actual public builders.

The universal Canvas 160×120 check is deleted. Positive exact roots still require
complete real content to fit. The external TextPanel fixture successfully renders
at explicitly requested 159×240, just below the retired width floor, with both
complete text sources present. Existing insufficient-content-height and occupied
cell refusals remain meaningful checks. Requests below actual TextPanel track
requirements still refuse rather than changing the selected typography.

## Frozen Output Inspection

`Output::width()` and `height()` borrow the canonical `RenderedScene` dimensions.
They do not inspect the original request, prepare data, measure, draw or plan a
PNG. Canonical scene construction still checks positive integral dimensions and
exact agreement with finalized SVG output.

The existing chained Plot/Canvas publication contract now inspects 320×240 before
and after publication. Source preview remains 320×240, a 192-DPI PNG is 640×480,
and default 300-DPI PNG is 1000×750, while Output stays 320×240. The same-operation
preserve-text bytes remain equal; the recorded one-scene lifecycle and no later
font discovery/shaping invariants remain intact. Automatic/width root capture
tests also preserve complete text and retained SVG after a PNG-local refusal.

## Focused Verification

Thirty-five distinct library tests pass:

| Focus | Tests | Log under `/tmp/opencode/` |
| --- | ---: | --- |
| Natural roots, complete captions, Matrix/facets, selected chrome and guide recovery | 15 | `root-sizing-natural.log` |
| Native coordinate requirements, exact assignment, retained inputs, feedback and cached/uncached replay | 16 | `root-sizing-coordinate.log` |
| Zero Plot/Canvas requests before fonts/backend begin | 1 | `root-sizing-zero.log` |
| Every family on an explicitly requested 640×480 root plus representative composed parent | 1 | `root-sizing-families.log` |
| Same-scene Plot/Canvas SVG and PNG publication with frozen Output dimensions | 1 | `root-sizing-chained.log` |
| Exact offset panel projection through one backend lifecycle | 1 | `root-sizing-offset.log` |

Four external contracts pass: exact-family/erased output parity and actual frozen
dimensions (`root-sizing-erasure.log`); automatic mixed/all-text Canvas and exact
TextPanel fit/occupancy (`root-sizing-text-canvas.log`); and the sub-floor positive
Canvas (`root-sizing-small-canvas.log`). Ninety-five doctests pass, including
compile-fail retirement of both Plot dimension getters and the consuming width
example (`root-sizing-doctests.log`).

The zero-size behavioral unit test replaces
`ordinary_test_sources_cover_every_native_route_exactly_once`; no unit coverage
is moved to integration tests. Library discovery remains **671 tests, including
17 ignored** (`root-sizing-census.log`, `root-sizing-ignored.log`). The existing
fixed-root lifecycle/coordinate fixtures explicitly request their dimensions;
automatic fixtures do not assert an incidental panel or root size.

Strict Clippy, formatting/whitespace checks, warning-denied docs and offline
package staging compilation pass (`root-sizing-lint.log`, `root-sizing-docs.log`,
`root-sizing-package.log`; the formatting and whitespace commands report success).
The following public sizing closure leaf owns the expanded all-mode parent,
setter-order and format-local evidence; scientific numeric and sectioned-guide
implementation remain subsequent stages.
