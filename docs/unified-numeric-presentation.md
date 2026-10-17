# Unified Numeric Presentation Owners

At this recorded private migration checkpoint, the normalized policy and default/fixed-value consumers moved onto
the exact decimal conversion and common emitter in `src/numeric/decimal.rs`.
`specs/06_NUMERIC_PRESENTATION.md` owns their semantics. The then-public
`NumericLabelFormat` stored only this private policy: accepted precision remained
`0..=12`, the default was fixed two decimals, and suffixes remained literal.
The later public replacement and wrapper retirement are recorded in
`public-numeric-formatting.md`.

## One Owned Policy And Resolved View

`src/numeric/format.rs::NumericFormat` owns all seven normalized fields: notation,
precision, exact decimal scale, optional prefix/suffix, negative-zero handling and
fractional trailing-zero handling. Complete structural equality includes fields
that can produce equal text for a particular value. Consuming edits validate the
result and preserve all other selections; cloned policies own their affixes.

`NumericPrecision` validates fractional `0..=17`, significant `1..=17`, and adaptive
`min<=max` within the same domains. Scale is `-324..=324`; affixes are nonempty,
control-free literal strings. A resolved view borrows those affixes and selects
one admitted exact precision. An adaptive policy requires its complete semantic
set; scalar formatting refuses an unresolved request instead of choosing a digit
count implicitly. Consumer-owned set selection follows in the execution plan.

The policy is private. Its module-local dead-code allowance covers the complete
construction/accessor vocabulary before configured consumer attachments use it.
The current public wrapper preserves its existing precision/suffix diagnostics.

## Preserved Profiles Of The Same Engine

`src/numeric/profiles.rs::BuiltinProfile` derives a resolved request for the common
converter, with the prescribed default operand derivation:

| Consumer | Retained grammar and conversion |
| --- | --- |
| Ordinary generated axes and guide numbers | Original-value magnitude thresholds, actual signed-zero normalization, branch-specific fractional trimming, and scientific fractional retries `2..=17` |
| Compact `k`/`M` branches | Perform the prescribed binary64 division first, then format that exact represented quotient with one fractional digit; retain the original-value branch even across a rounding carry |
| Forest integral sources | Preserve every literal from exponent `-2..=11`; other admitted integral exponents use the same decimal emitter with literal prefix `1.00e` |
| Forest adaptive/midpoint defaults | Literal `10^` followed by Automatic-formatted exponent text |
| Heatmap values and point-only Coefficient values | Current fixed precision, Preserve/Keep, scale zero and literal suffix, from the owned policy |
| Waterfall totals | Fixed three decimals with Preserve/Keep; join the verbatim name, one space and the completed numeric source using checked actual lengths |

The compact derivation intentionally differs from configured exact decimal scale.
For example, binary64 `12350/1000` formats as `12.3k`, and `1150000/1000000`
as `1.1M`. Exact scale would round the mathematical decimal ties differently.
The original threshold also permits `999999` to produce `1000k`.

Axis labels, guide labels and natural/terminal callers reach this engine through
the existing source adapters. Exact ticks, categories, caller significance and
other literal strings retain their existing source owners. Configured Forest
power presentation will use the independently certified mathematical operand;
the existing omitted-policy expression grammar remains its own built-in profile.

## Retained Text Before Font Selection

Coefficient numeric strings are converted during scientific preparation.
`PreparedEstimate::detail_sources` now supplies the actual retained numeric
strings and required unavailable text to the font census, filtered by the owning
prepared panel scope. PointInterval and Forest share that unavailable-text path;
caller significance stays on its existing significance path. Census no longer
rereads raw estimate columns or performs another numeric conversion.

Waterfall census already reads retained prepared totals. Its preparation now uses
the same converter for the number and reserves the exact joined name/space/number
length. Heatmap continues to supply retained value strings and the complete
contrast role selected from the final fill.

Removed support includes `NumericLabelDisplay`, its count-and-write fixed-decimal
body, the bounded Rust axis/guide formatter, the raw-column specialized estimate
census, its facet-only export/accessor, and the duplicate Waterfall census helper.
The general census formatter remains for actual formatted nonnumeric sources.

## Focused Evidence And Test Replacement

One new owned-policy behavior test replaces
`layout::panel::sources::tests::dense_source_returns_a_miss_without_positions`.
The removed single assertion duplicates dense-source behavior exercised through
the retained complete-source/coarsening tests. No tests were merged, ignored or
moved into an integration suite.

Existing tests now exercise the actual source adapters at both signs of all six
magnitude thresholds and their immediate binary64 neighbors. Independent Rust
scientific spelling checks every retry precision `2..=17`; literal expectations
cover compact-division rounding, carry attribution, signed zero, extreme finite
values, the first subnormal and all Forest literals. The owned-policy test checks
exact percentage scaling, literal combining-mark affixes, post-rounding zero
normalization, single-round engineering carry, full policy identity and validation
before emission. The independent rational and power/root certificates remain in
their original arithmetic tests.

The retained estimate census test checks both a complete numeric-plus-leading-mark
suffix span and unavailable text under the complete estimate-detail role. Free
Waterfall facets verify retained `base 10.000`, `total 16.000` and `total 610.000`
sources at automatic and requested width, without pinning automatic dimensions.

All test commands used `--locked --offline`. Focused library results:

| Command filter after `cargo test --locked --offline --lib` | Passed |
| --- | ---: |
| `numeric::` | 13 |
| `layout::panel::sources::tests::` | 21 |
| `layout::text::requirements::tests::` | 22 |
| `spec::tests::` | 1 |
| `free_waterfall_facets_measure_shelves_at_each_consumed_numeric_domain` | 1 |

These are **58 distinct library tests**. Representative external callers passed:

| Target and filter | Passed |
| --- | ---: |
| `estimates coefficient_fixed_value_labels_are_point_only_and_default_is_inert` | 1 |
| `estimates shared_x_facets_inherit_estimated_sibling_range_for_unavailable_panels` | 1 |
| `estimates explicit_limits_allow_unavailable_panels_with_free_x_scales` | 1 |
| `estimates model_displays::forest_` | 4 |
| `heatmap heatmap_annotations_` | 3 |
| `heatmap numeric_label_format_rejects_invalid_precision_and_suffix` | 1 |
| `heatmap local_and_shared_guides_publish_the_same_distinct_close_numeric_labels` | 1 |
| `composition shared_beeswarm` | 2 |
| `composition continuous_shared_heatmaps_draw_one_merged_guide_without_local_colorbars` | 1 |

These are **15 external tests**, including required-guide uniqueness and final
Shared domains, fixed-value validation/fit, exact Forest sources and scoped
unavailable rows. Strict all-target Clippy (`just lint`), formatting and whitespace
checks pass. Library discovery remains **671 tests, including 17 ignored**.

Spec93 schedules configured axes/guide/value sets, font boundary units, complete
composition identities and the public replacement as subsequent owners. This
checkpoint establishes the common policy/default conversion and retained text
sources. Evidence is from bounded focused checks; no full-suite run or whole-scene
golden repin is attributed to this change.
