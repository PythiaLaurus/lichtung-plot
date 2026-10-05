# Complete private typography values

## Implemented boundary

`src/theme/typography.rs` implements the complete typography constructor schema
in §4 of `specs/94_T6_FOUNDATION_CONSTRUCTION.md`. `ResolvedTheme.typography` now
owns one `TypographyTheme`; the flat `FoundationalTypography` representation and
its implementation are removed. The current public `Theme` remains the preset
selector, and its existing `resolve(resources)` factory constructs these groups
for Publication, Nature, Science, and CHI.

This is the complete typography-only assembly unit. The other component
assemblies and the local-relative-metric consumer cutover remain separately
implementable work, listed below. No replacement component is drafted without
becoming the actual owning source.

## Complete construction, inspection, and edits

Each group implements `Clone + Debug + PartialEq`. Every constructor requires all
of its typed inputs and returns `Result<Self, Error>`. Every owned field has a
borrowed field-named getter and a consuming `with_field` edit returning the same
result type. There are no defaults, missing-role fallbacks, mutable getters, or
extra font-stack construction paths in these values.

| Complete value | Retained constructor inputs |
| --- | --- |
| `RootTextRoles` | `title`, `subtitle`, `caption`: `TextTheme` |
| `AxisTextRoles` | `axis_title`, `tick_label`: `TextTheme` |
| `GuideTextRoles` | `legend_title`, `legend_label`: `TextTheme` |
| `CompositionTextRoles` | `facet_header`, `composition_header`, `text_panel_title`, `body`: `TextTheme` |
| `ContrastTextRoles` | `light`, `dark`: `TextTheme` |
| `AnnotationTextRoles` | `significance_label`, `estimate_detail`, `waterfall_total`, `point_layer_label`, `panel_text`: `TextTheme`; `contrast`: `ContrastTextRoles` |
| `TypographyTheme` | `root`: `RootTextRoles`; `axes`: `AxisTextRoles`; `guides`: `GuideTextRoles`; `composition`: `CompositionTextRoles`; `annotations`: `AnnotationTextRoles` |

The small `text_group!` helper generates this uniform construction contract.
Inputs are moved into inline groups once. Construction and edits recursively
validate the complete retained result, including unchanged siblings. They do not
allocate another font payload or create an operation ledger.

`TextTheme::validate` supplies the existing positive CSS font-size and finite,
positive binary64 line-height checks. `ContrastTextRoles` additionally requires
both complete candidate paints to have alpha 255. That condition is checked at
construction and every edit, including through annotation and typography parent
replacement. Transparent general text remains valid. Neither candidate's font,
size, weight, paint, or leading is inferred from the other candidate.

## Exhaustive live role mapping

Paths below start at `ResolvedTheme.typography`. Each row is retained once and is
consumed through the nested path by its existing planner, census, or renderer.

| Former flat field | Current retained path | Actual consumers |
| --- | --- | --- |
| `title` | `root.title` | Plot/Canvas title blocks, embedded Plot headings, root intrinsic widths |
| `subtitle` | `root.subtitle` | Independent Plot/Canvas subtitles and title-block measurement |
| `caption` | `root.caption` | Plot/Canvas captions, text-query and shaping fixtures |
| `axis_title` | `axes.axis_title` | Standard/shared axes, Matrix labels, Heatmap axes, standard-panel inputs |
| `tick_label` | `axes.tick_label` | Numeric/categorical ticks, specialized intrinsics, standard-panel and Heatmap caches |
| `legend_title` | `guides.legend_title` | Discrete/continuous guide headings, Shared Canvas census |
| `legend_label` | `guides.legend_label` | Guide entries/numbers/missing keys and their intrinsic measurement |
| `facet_header` | `composition.facet_header` | Facet strip census, text measurement, retained strip plans |
| `composition_header` | `composition.composition_header` | Canvas column/row headers and row titles |
| `text_panel_title` | `composition.text_panel_title` | Canvas TextPanel title census and measured text |
| `body` | `composition.body` | Canvas TextPanel body census and measured text |
| `significance_label` | `annotations.significance_label` | Attached/pairwise significance, exact size/weight precedence |
| `estimate_detail` | `annotations.estimate_detail` | Estimate values, required unavailable marker, pre-font census |
| `waterfall_total` | `annotations.waterfall_total` | Waterfall total shelves and their measured reserve |
| `point_layer_label` | `annotations.point_layer_label` | Supplemental point-label census and final text materialization |
| `panel_text` | `annotations.panel_text` | Caller panel annotations, their census, placement, and collision bounds |
| `contrast_light` | `annotations.contrast.light` | Complete light candidate selection before fonts and Heatmap intrinsic retention |
| `contrast_dark` | `annotations.contrast.dark` | Complete dark candidate selection before fonts and Heatmap intrinsic retention |

The existing deferred-census semantic selection and raw/public font helper
boundaries are preserved. This unit changes their owning paths, not their source
strings, ordering, selected profiles, or preparation policy.

The existing generic scalar carriers (`detail_font_size`, `label_font_size`,
`composition_title_font_size`, `facet_header_font_size`) retain their existing
factory, equality, fixture, and remaining metric consumers for B2. In particular,
Canvas/facet declared-track em and facet's historical `size + 6` reserve stay at
their existing handoff. They are not inputs or fallback sources in the new
typography constructors. B2 owns their removal and the complete-role/local-metric
consumer reconciliation, including the intentional facet-strip occupancy fix.

## Defaults, exact leading, and output

All eighteen built-in role values preserve §4's existing stack, RGBA channels,
CSS size, weight, and raw multiplier bits. Root title/subtitle/caption leading
keeps the operation order `(f64::from(size) + 6.0) / f64::from(size)`; the remaining
roles retain `1.2`.

The independent Publication title fixture pins size 20 and line-height bits
`0x3ff4cccccccccccd`. Its exact represented multiplier times `20 * 1024` quanta
rounds upward to **26625**, not 26624. No absolute pitch or decimal-rational
replacement is stored. Existing whole-pixel placement rounding and facet-strip
geometry remain at their current consumer boundaries.

Root/Canvas title, Matrix, Marginal, Bar/Waterfall, estimate-display, and Beeswarm
SVG snapshots pass without changes. The eight-case Marginal full-SVG fingerprint
fixture also passes unchanged. Heatmap signed-zero center output remains equal
between the automatic and exact-domain selections.

## Copies, comparisons, and cache admission

`TypographyTheme::roles()` traverses each of the eighteen complete text roles
once, in the existing retained-profile order. Nested groups are inline values;
they add no Arc, variable-size buffer, extra handle visit, or separate payload
record. Existing object-storage charges continue to use their actual owning
object sizes.

`ResolvedTheme::admitted_clone` still admits **22 handle operations**: eighteen
font handles, qualitative/sequential/diverging scale handles, and one accent
handle. It then clones the same immutable payloads with zero new payload records
or buffer bytes. Exact-work and minus-one refusal fixtures verify this at both
the theme and child-cache boundaries. The child fixture fills all one-million
record slots and gives the copy zero remaining bytes, proving that sharing does
not charge the old copied RGB array or font strings. Each font and scale/accent
payload address is retained.

`ResolvedTheme::same` prices the four scale/accent witnesses and then each actually
examined complete text/font profile. A shared equal theme costs 22 visits. The
new exhaustive mismatch fixture changes the retained leading in each of the
eighteen roles: comparison costs `4 + preceding_role_count`, returns unequal at
the differing scalar, and refuses with one fewer available visit. A child tick
font-content mismatch costs **20 visits**: four scale/accent witnesses, four
preceding shared font witnesses, and twelve terminating-name examination visits.
No nested-group visit is added, no content refusal becomes a cache miss, and
later roles remain unexamined.

Effective significance construction keeps its existing admitted font-copy path
and exact caller size/weight precedence. Standard-panel and Heatmap intrinsic
copies/comparisons retain their selected complete roles and their own existing
operation resources. Text-query hashing still operates on exact selected text,
font, presentation, and provenance operands.

## Verification

**72 distinct focused tests passed**, covering constructor, preset,
copy/comparison, cache, census, guide, Plot/Canvas,
facet, annotation, and output tests exercise the actual changed paths. The new
seven typography fixtures cover all eighteen inputs/edits, all seven complete
group constructors, immediate opaque-candidate boundaries, sibling isolation,
every nested role's admitted identity, and exact root-leading occupancy.

Strict compiler validation uses `just lint`
(`cargo clippy --locked --offline --all-targets -- -D warnings`), with formatting
and diff whitespace checks. No full-suite test command is required for this
bounded representation migration. Snapshot expectations are unchanged.

Representative reproducible filters (all run with `cargo test --locked --offline`):

```text
--lib theme::typography::tests
--lib theme::resolved::tests::built_in_
--lib admitted_theme_clone_covers_all_eighteen_roles
--lib retained_theme_matching_prices_all_eighteen_stack_profiles
--lib effective_significance_copy_uses_actual_role_storage
--lib decorations::plan::children::tests
--lib preset_construction_
--lib plots::heatmap::cache_tests
--lib heatmap_intrinsic_cache_rechecks
--lib canvas::tracks::tests
--test composition themes::
--test visual-regression root_title_block
--test visual-regression native_bar_family_snapshots
--test visual-regression native_estimate_display_snapshots
--test visual-regression matrix_clip_structure_snapshot
--test visual-regression marginal_clip_structure_snapshot
--test visual-regression beeswarm_clip_structure_snapshot
--test marginal-matrix marginal_full_svg_fingerprints
--test annotations exact_per_bar_marks_render_with_independent_weight
--test heatmap legacy_signed_zero_diverging_centers_render
```

Additional focused filters cover TextTheme validation, exact query hashing,
standard-panel matching, selected contrast/estimate census, large/overhanging
guide text, TextPanel ownership, facet strips/shared titles, and poisoned replay.

## Concrete remaining assembly partitions

These are the remaining schema implementation units, not unresolved design
questions. Each must replace its actual aggregate storage, migrate affected
field consumers and admitted copy/equality paths, and finish with focused evidence.

| Remaining unit | Construction/storage replacements | Concrete consuming handoffs |
| --- | --- | --- |
| Axis chrome | `FrameSides`, `FrameTheme`, `TickTheme`, relative `AxisSpacing`, `AxisTheme` replace `FrameAppearance`, `TickAppearance`, pixel `AxisSpacing`, and `AxisAppearance` in `theme/resolved.rs` | `layout/panel/axis_frame_grid.rs::{measure_axes_with_inputs, measure_axes}`, `plots/matrix.rs`, `orchestrator/facets/rendering.rs::plan_shared_facet_axis_labels`; delete dead top/right preset metadata, retain four-side defaults and structural absence |
| Guide and composition chrome | `LegendSpacing`, `GradientLengths`, `LegendFrameTheme`, `LegendTheme`; `RootTitleSpacing`, `RootPadding`, `TextPanelSpacing`, `HeaderSpacing`, `FacetStripSpacing`, `CompositionTheme` replace the remaining legend/composition aggregates | `decorations/legend.rs::LegendMetrics`, `decorations/legend/continuous.rs::MeasuredContinuousGuide`, `decorations/title_block/plan.rs`, `layout/canvas.rs::PanelRequirements`, `canvas/{tracks,text_panel,sizing}.rs`, `orchestrator/facets/{tracks,rendering}.rs`; resolve supplied-ledger local products, preserve declared source literal bits and existing placement policy |
| Mark role assembly | `PointRadii`, `SizedPointTheme`, `SupplementalPointTheme`, `PointRoles`; body/band/density/fill groups, mapped-line groups, neutral paint groups, reference/connector/summary/curve/stroke groups; cell/category/extent groups and `MarkTheme` replace `PointSizeAppearance` and `MarkAppearance` | `plots/distribution_sample.rs::overlay_style`, `plots/violin.rs` and `plots/violin/precomputed.rs` split optional observations from mandatory strips; `plots/line.rs::PreparedLine::group_color` and Boxplot flier materialization use independent neutral paints; `plots/{bar_data,waterfall,histogram,marginal,matrix,heatmap}.rs` route admitted extents and missing-cell geometry to actual owners |
| Annotation geometry | `SignificanceGeometry`, `CellAnnotationGeometry`, `EstimateAnnotationGeometry`, `ValueShelfSpacing`, `AnnotationTheme` replace `AnnotationAppearance`; Heatmap highlight moves from Marks to annotation cells | `significance.rs` bracket planning, `plots/heatmap.rs` value/highlight intrinsics and drawing, `plots/interval_estimate.rs` detail/value reserve, `plots/waterfall.rs` shelves, `plots/line.rs` label clearance; validate opaque planning backdrop and field ranges in the complete constructor |

The subsequent B2 consumer work also removes the generic scalar carriers, derives
exact local metric consequences after selecting complete roles, reconciles facet
strip measured occupancy, and switches the built-in Beeswarm sequential default
to the approved common Heatmap sequence. This typography unit does not change
that sequence or add component exports.
