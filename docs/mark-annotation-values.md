# Complete Mark And Annotation Values

## Retained Construction And Identity

`src/theme/mark.rs` replaces the flat `MarkAppearance` and `PointSizeAppearance`
records with the eight complete `MarkTheme` inputs from spec94: accents, points,
fills, mapped lines, neutral paints, strokes, cells and extents. Nested groups
retain all twelve radii, independent optional-observation and mandatory-strip
roles, supplemental defaults, category geometry and independent Waterfall breadth.

`src/theme/annotation.rs` replaces `AnnotationAppearance` with the complete
significance, cell, estimate, point-label, panel-text, value-shelf and opaque
planning-backdrop inputs. Heatmap highlight outline has one annotation owner.
The old generic `ResolvedTheme.foreground` carrier is removed.

Fields of these groups are private. Typed constructors and consuming `with_*`
edits validate the complete result, including nested siblings; readonly accessors
do not expose mutable storage. Primitive paint constructors and group validation
share the same validators. Positive widths/radii, structural optional outlines,
exact fraction bounds, and type-enforced nonnegative spacing/opaque backdrop are
checked before any font or backend work. Breadth one is constructible; its
data-dependent containing-ink checks belong to the consumer migration.

Preset construction retains the specified byte alphas, binary32 stroke values,
integer radii and binary64-derived nonspatial fractions. Relative annotation
spacing retains the exact ratios from spec94. Complete component equality includes
every retained input, with no separate point/non-point field inventory. Runtime
font environments and query caches remain outside appearance equality.

## Current Consumer Migration

| Owner | Implemented route |
| --- | --- |
| All existing mark-record consumers | Selected point/radius, body/band/density fill, mapped-line, reference/connector/summary/curve stroke and accent groups, including intrinsic, preference, mark and originating-guide readers |
| Neutral paths and Boxplot hidden-overlay fliers | Independent opaque neutral RGB; the selected path-line or primary-point role supplies alpha and geometry once |
| Boxplot/Violin optional observations | Independent `points.raw_overlay`; RawOverlay presence remains plot-owned |
| Sampled/supplied Raincloud strips | Independent `points.mandatory_strip` for intrinsic pitch, occupied ink, final scatter and point-bearing guide; outline visibility is applied once before these consumers |
| Heatmap cells, merged missing rows and missing keys | Optional outline storage carried through geometry, descriptor measurement and final fill adaptation; highlight reads only the annotation component |
| Heatmap intrinsic cache | Complete retained cell-mark and cell-annotation groups plus the selected opaque backdrop, text, surfaces and scale |
| Bracket stroke | Nested significance stroke used by planning, geometry and replay-style materialization |
| Supplemental point-label gap | Exact product against the selected point-label font size, retaining the existing radius-plus-gap and whole-pixel placement policy |
| Waterfall value shelf | Exact selected-role padding/gap/row-gap products in intrinsic and assigned planning; complete width/height reserves aggregate before upward rounding, while frozen baselines use nearest-even placement |

An absent missing-key outline retains its filled key, label and gradient rather
than removing semantic missing content. The existing continuous-guide absence
fixture now exercises this structural absence together with the absent frame.

## Remaining Consumer Resolution

This closes private value construction. T6B2 still owns complete immutable Theme
construction/customization and the remaining literal-to-component routing in
spec94. In particular, the newly retained inputs for category/group/cap geometry,
Waterfall body breadth, interval caps, sampled rug and local density fractions,
supplemental defaults, mapped-cell alpha/cross inset, Heatmap text padding,
estimate clearances and panel-text padding still require their owning consumer
cutovers. The significance absent/exact-style selection and resolved tick-height
handoff through canonical backend validation also remain there.

Those routes must use the retained exact relationships and complete dependency
keys; their existing literal calculations are not a second custom-value fallback.
General continuous-scale consumption, the approved common Beeswarm sequential
default, and selective PointLayer text paint/alpha resolution retain their B2/B3
owners. The public complete-Theme/component cutover remains B4.

## Test Replacement And Verification

Three introduced library tests replace these three removed fixtures in the same
logical change:

- `fixed_fill_edits_and_absence_retain_only_their_selected_paints`
- `data_line_edits_replace_one_input_and_never_requantize_alpha`
- `canonical_absent_outline_metadata_preserves_png_pixels`

The replacements check complete sibling validation/fraction bounds, actual sampled
and supplied strip styles/fit/source counts/guide identity under unrelated raw-role
edits, and the independent half-quantum Waterfall shelf reserve. The last uses two
quarter-quantum padding edges: their complete hard reserve is one quantum.
Existing scientific, independent alpha, width refusal, source completeness,
canonical absence, decoded PNG and frozen-replay owners continue supplying the
shared contracts. Tests are not moved into external targets or newly ignored.

| Actual check | Result |
| --- | --- |
| `cargo test --locked --offline --lib theme::` | 41 passed |
| Focused mark/style, preference, continuous-guide/intrinsic, missing-row census, frozen-replay and canonical lifecycle filters | 35 passed, including the one-lifecycle fixture for all 25 canonical families |
| Representative external annotations, categorical, composition themes, Raincloud, estimate, Heatmap SVG/decoded PNG and Line contracts | 44 passed across seven existing targets |
| Focused final structural-absence recheck | 1 passed |
| Focused final compound/group validation recheck | 8 passed |
| Strict `just lint` | Passed, all targets, locked/offline, warnings denied |
| Formatting and whitespace checks | Passed |
| Library discovery, including ignored tests | 671 total; 17 ignored; no growth from the pre-change count |

The focused sets contain 76 distinct library tests and 44 external tests. The
last two rechecks repeat affected tests within those sets after the final local
edits. This leaf used bounded checks of its changed call paths.
