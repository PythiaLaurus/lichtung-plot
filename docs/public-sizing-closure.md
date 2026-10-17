# Public Root-Sizing Closure

This closes the public three-mode contract installed in
[content-derived root sizing](content-derived-root-sizing.md). All tests use the
current consuming builders and inspect final Output dimensions. Automatic roots
are checked through content, containment and replay relationships, not pinned
natural dimensions or renderer-revision fingerprints.

## Actual Allocation Repairs

The public caller refresh exposed two real small-categorical-root failures:

1. Canvas selected natural width from a scientific body preference without its
   horizontal axis/chrome preference. A two-row Forest then had insufficient hard
   body width when the measured axis gutters arrived. The shared private
   `layout::interior::sizing::horizontal_gutter_preference` now supplies the initial
   leading label/title/chrome and trailing edge operands to both standalone width
   composition and Canvas's initial natural columns. It reads actual displayed
   categories or exact labels before numeric probes, skips hidden y labels, and
   aggregates the complete local relationships before hard spatial conversion.
   Selected-source gutter feedback still replaces the initial estimate. Matrix
   retains its native variable Grid's trailing-edge owner rather than reserving a
   second ordinary outer edge.
2. A content-height Bar could measure changed axis gutters, then report a terminal
   residual-height miss before those same gutters reached its parent. The existing
   named gutter/ink dependency now keeps that content-height miss provisional.
   The next bounded root round derives height with the consumed measurements at
   the already chosen width. Exact-height requests keep their actual fit refusal;
   no width reselection, previous-best plan or additional retry loop is introduced.

The all-family public fixture renders both automatic and exact-width roots for
all **25 standalone families** and all **24 admitted Canvas children**, yielding
98 successful real root operations. Matrix's established Canvas boundary remains
separate. The earlier explicitly requested 640×480 all-family lifecycle fixture
continues to own exact assignment, complete capture and source-drop replay.

## Public Content And Output Contracts

`tests/layout/root_sizing.rs` adds ten independent public contracts:

| Contract | Observable relationship |
| --- | --- |
| All actual family constructors | Both content requests reach complete root output and canonical SVG dimensions, with their actual axes/chrome. |
| Standalone setter replacement | Size→width clears height; repeated width replaces width; width→size and repeated size replace both dimensions, including an earlier invalid request. Complete long-caption words occur exactly once and increase measured height at the same requested width. |
| Unbreakable automatic content | A complete unbreakable heading expands natural width relative to a short heading. The same source under insufficient exact width refuses without an artifact. |
| Owning facets | Automatic/width/exact roots retain three sparse-wrap strips, independent outer title/subtitle and the complete caption; three occupied panels remain. |
| Native Matrix | All modes retain four native cells, variable labels, the complete hue guide and caption. Ordinary and cell-frame ownership remain distinct. |
| Mixed sparse Canvas | All modes preserve two Plot title/subtitle/local-caption sets, separate Canvas prose, a spanning header, Shared guide and TextPanel. Replacing standalone child requests with zero width gives the same composed output. |
| All-text Canvas | All modes reflow one complete paragraph, keep the root footnote separate and honor full request replacement. |
| PNG source limits | Valid preserve-text and outlined SVG remain publishable beyond the source-axis and source-area PNG limits in all three modes. PNG refuses without an artifact or canonical dimension change. |
| Default PNG target limits | The independent 300-DPI axis and area boundaries are exercised with actual successful rasterization at their admitted sides. |
| Actual large parser payload | All modes preserve every one of 900,000 coincident points in canonical SVG and successfully outline it beyond 64 MiB. Source dimensions/area remain inside PNG geometry limits; PNG specifically refuses its actual outlined parser payload. |

The prose fixture is a 200-plus-word **single paragraph**. Complete word sequences
are parsed from the actual wrapped SVG text and required exactly once. Ordinary
successful scenes publish preserve-text SVG → outlined SVG → source-sized PNG →
half-scale PNG → preserve-text SVG through one Output handle. Both SVG root and
viewBox dimensions match that handle; decoded PNG dimensions agree with the source
scene; half-scale rounding is independently checked. The final canonical bytes
equal the same operation's first bytes, and Output dimensions remain unchanged.

At default 300 DPI, `10_485×240` produces `32_766×750`; `10_486×240` rejects the
32,768-axis gate. `3_200×3_200` produces the admitted `10_000×10_000` target;
`3_200×3_201` rejects target area. These are requested dimensions and independently
specified conversion boundaries, not automatic geometry baselines.

The large public test uses a complete explicitly selected 8px tick role to keep
natural source geometry inside PNG's independent geometry limits. It checks actual
serialized sizes against the 67,108,864-byte parser gate, complete circle count and
same-operation replay. It does not rasterize a known invalid payload. Existing
scene tests separately prove refusal before derived-buffer reservation, actual
outlined-byte counting and reusable retained paths after a PNG refusal.

## Existing Caller And Example Migration

The former omitted-size/default-640×480 output equality is retired. Its external
fixture now relates automatic SVG dimensions to frozen Output and independently
checks the explicit 640×480 request. Histogram/Density/Q-Q preview dimensions now
match each returned canonical Output rather than an omitted historical default.

The interval/cache fixture uses public exact width with content height. The
automatic Forest/cache fixture checks raw exact precedence and even-decade
coarsening without assuming an incidental automatic panel admits all three first
preference ticks. Cached and uncached bytes remain equal. Existing exact-root cache
fixtures retain their separate fixed-assignment contract.

The quick-start executable now renders automatically and prints frozen dimensions.
The Canvas executable demonstrates an automatic root and a 1000px exact-width
root, preserving inheritance and explicit child replacement. Both executables ran
successfully using `/tmp/opencode` destinations. The getting-started guide includes
actual consuming Plot/Canvas replacement and Output inspection examples.

## Focused Verification

All logs below are under `/tmp/opencode/`:

| Focus | Distinct passing tests | Evidence |
| --- | ---: | --- |
| Existing natural roots, complete prose, guides, sparse facets and selected chrome | 15 | `public-sizing-natural.log`; the three affected Matrix neighbors also pass in `public-sizing-matrix-neighbors.log` |
| Existing independent scene/PNG limits and retained-path lifecycle | 7 | `public-sizing-scene-boundaries.log` |
| Automatic, exact-width and exact-root cached/uncached callers | 3 | `public-sizing-automatic-cache.log`, `public-sizing-width-cache.log`, `public-sizing-exact-cache.log` |
| Seven ordinary public parent/mode/content contracts | 7 | Six pass in `public-sizing-contracts.log`; the Matrix ownership correction passes its focused rerun in `public-sizing-matrix.log` |
| Every family and admitted composed child | 1 | `public-sizing-families.log`, optimized profile |
| Actual default PNG target boundaries | 1 | `public-sizing-png-targets.log`, optimized profile |
| Actual >64 MiB canonical and outlined payloads in all modes | 1 | `public-sizing-large-parser.log`, optimized profile |
| Existing multi-format, distribution-preview and unused/active-font callers | 6 | `public-sizing-multiformat.log`, `public-sizing-distribution-output.log`, `public-sizing-font-roles.log` |

This is **25 focused library tests and 16 distinct external contracts**. No unit
tests are introduced, deleted or moved. Current discovery remains **671 library
tests, including 17 ignored** (`public-sizing-census.log`,
`public-sizing-ignored.log`). Strict all-target Clippy passes
(`public-sizing-lint.log`). All 95 doctests pass (`public-sizing-doctests.log`),
as do warning-denied documentation (`public-sizing-docs.log`), offline package
staging compilation (`public-sizing-package.log`), formatting and whitespace
checks. Executable results are in `public-sizing-quick-example.log` and
`public-sizing-canvas-example.log`.

Numeric presentation and sectioned discrete guides remain the following staged
work. Their source refresh must include the new shared horizontal-gutter probe
owner in addition to the existing final axis/guide/annotation consumers.
