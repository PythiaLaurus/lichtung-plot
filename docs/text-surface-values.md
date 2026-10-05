# Validated text and surface values

## Current private representation

`src/theme/resolved.rs` defines the live `TextTheme` and `SurfaceTheme` values.
`src/theme.rs` re-exports them within the crate. They implement
`Clone + Debug + PartialEq` and are the types retained by `ResolvedTheme`, its
eighteen typography roles, text-query profiles, standard-panel inputs and Heatmap
intrinsic profiles. The former `TextAppearance` and `SurfaceAppearance` types are
removed, and every Rust caller uses the current names.

The types and their fields are externally private. Existing internal geometry
and negative fixtures can still access `pub(crate)` fields. The retained text
field names `paint` and `line_height_multiplier` describe the existing operands;
their construction/inspection vocabulary is `color` and `line_height` as specified
in spec94 §§2 and 4. Public foundation exposure belongs to B4.

## Construction, inspection and edits

The private construction signatures are:

```rust,ignore
TextTheme::new(stack: FontStack, color: Color, font_size: u32,
               weight: TextWeight, line_height: f64) -> Result<TextTheme>
SurfaceTheme::new(canvas_root: Color, plot_panel: Color) -> Result<SurfaceTheme>
```

Text construction requires positive integral **actual CSS-pixel** font size and
a finite positive binary64 leading multiplier. It retains the supplied typed
stack, paint, size, weight and exact multiplier bits. Positive subnormal and
maximum finite multipliers, and `u32::MAX` font size, are valid construction
inputs; representability and fit remain planning responsibilities. There is no
construction-time shrinking or font discovery.

`stack()` borrows `&FontStack`; `color()`, `font_size()`, `weight()` and
`line_height()` return their retained scalar inputs. The consuming edits
`with_stack`, `with_color`, `with_font_size`, `with_weight` and
`with_line_height` each return `Result<Self>`, replace only their named input,
and revalidate the complete result. An invalid internal sibling field therefore
cannot pass through an unrelated edit. Repeated edits replace their own input;
editing a clone preserves the source value.

Surface construction retains two complete `Color` paints. Both colors are already
validated values, including transparent and opaque endpoints, so this uniform
`Result` constructor is infallible for its typed inputs. `canvas_root()` and
`plot_panel()` return the paints; `with_canvas_root` and `with_plot_panel` consume
the value and reconstruct it with only that field replaced.

General text paint can be translucent. The existing automatic contrast selector
still requires both complete candidate text paints and the planning backdrop to
be opaque. The contrast fixture constructs a legal translucent text edit and
then verifies rejection at that cross-role selection boundary.

## Live construction and metric consumers

All built-in surfaces and all eighteen built-in text roles use the constructors.
The old `resolved.rs::text_appearance` factory is removed. Text-query retention,
generation-owned role copies, standard-panel inputs, Heatmap intrinsic copies,
Coefficient detail/significance roles and supplemental Line labels also construct
the current complete text value through `TextTheme::new`.

`line_height()` is exclusively the retained **multiplier** getter. The existing
computed CSS-pixel product has the explicit internal name `line_pitch_pixels()`:
`f64::from(font_size) * line_height_multiplier`, in the original operation order.
Its callers in Canvas tracks/TextPanel, root layout, natural plot sizing, tick
intrinsics, legend metrics and private geometry fixtures are migrated. Frozen
`ResolvedTextPlan::line_height()` retains its separate measured-plan meaning.

Text planning receives the raw multiplier through
`TextLineHeight::Multiplier(text.line_height())`. Publication title leading keeps
the binary64 `1.3` bits `0x3ff4cccccccccccd`; its exact local product still rounds
upward to **26625 quanta** at 20 CSS pixels. The helper rename does not replace
that measured occupancy with a 26px estimate.

Construction moves the supplied `FontStack`. Preset assembly performs the same
eighteen immutable stack-handle clones as its prior factory. Operation copies
retain their existing `copy_font_stack`, `copy_with_generation` or `share`
admission path and supplied operation ledger, then pass the resulting stack to
the constructor. Constructor/edit validation adds only scalar checks. The exact
stack-copy, shared-profile, cache-hit, hashing and complete-theme clone work/byte
boundary fixtures pass with their existing expectations.

The four construction edits awaiting complete customization (`with_stack`,
`with_line_height` and the two surface edits) have method-local, non-test-build
`expect(dead_code)` annotations with their specific cutover reason. Their actual
constructor/edit fixtures exercise every method; there is no module-wide
dead-code allowance for these values.

## Changed source paths

| Paths | Live responsibility |
| --- | --- |
| `src/theme.rs`, `src/theme/{resolved,contrast}.rs` | Sole value definitions, crate-private reachability, checked presets/copies, multiplier inspection, contrast and constructor fixtures |
| `src/layout/{fit,canvas}.rs`, `src/layout/text/requirements.rs` | Typed text planning/census inputs, computed-pitch consumers and existing fixtures |
| `src/layout/panel/semantics.rs`, `src/layout/interior/sizing.rs`, `src/layout/interior/coordinate/tests.rs` | Checked retained panel roles, natural sizing and metric fixture migration |
| `src/canvas.rs`, `src/canvas/{text_panel,tracks}.rs` | Actual Canvas role census, TextPanel intrinsics and track preferences |
| `src/decorations/legend.rs` | Existing guide label/title pitch operands |
| `src/plots/{bar_data,heatmap,interval_estimate,line}.rs` | Typed significance inputs and checked cached/annotation/label role copies |
| `src/significance.rs`, `src/significance/tests.rs` | Current role type and exact planned-label metric evidence |

## Verification

All commands use `--locked --offline`. The following bounded filters passed:

| Target | Filter | Tests |
| --- | --- | ---: |
| library | `theme::resolved::tests` | 39 |
| library | `theme::contrast::tests` | 2 |
| library | `layout::text::requirements::tests` | 23 |
| library | `leading_` | 4 |
| library | `canvas::admission_tests::canvas_preset_construction` | 1 |
| library | `canvas::admission_tests::post_font_canvas_intrinsics` | 1 |
| library | `plots::heatmap::tests::heatmap_intrinsic_cache_rechecks_contrast_role_metrics_after_theme_change` | 1 |
| library | `plots::heatmap::tests::missing_fill_and_backdrop_invalidate_cached_intrinsics_with_unequal_contrast_roles` | 1 |
| library | `significance::tests::planner_retains_the_exact_effective_significance_label` | 1 |
| composition integration | `themes::` | 6 |
| composition integration | `text_panel_content_stays_top_left_when_its_fill_cell_grows` | 1 |
| lines integration | `point_labels_and_whole_layer_legend_identity_remain_distinct` | 1 |
| estimates integration | `coefficient_fixed_value_labels_are_point_only_and_default_is_inert` | 1 |
| visual-regression integration | `root_title_block_` | 2 |
| visual-regression integration | `scatter_smoke_snapshot` | 1 |
| visual-regression integration | `native_path_snapshots` | 1 |

These are **85 distinct tests** (86 executions: `leading_` also selects one
constructor test already covered by the resolved-theme filter). The three new
fixtures cover complete independent edits, invalid font/leading inputs and valid
extremes, and independent translucent surface edits. Existing fixtures cover
opacity selection, literal font identity, admitted copies, source counts,
root/explicit-child ownership, frozen role metrics and cache reuse/refusal.

Seven selected SVG snapshots match their existing expected output: standalone
and Canvas odd-size title blocks, Scatter, Line, Trajectory, LinePath and
categorical Trajectory. Production output expectations and snapshot files have
no changes.

`just fmt-check` and strict `just lint` passed; the latter runs
`cargo clippy --locked --offline --all-targets -- -D warnings`. Final whitespace
validation uses `git diff --check` and the new document's no-index diff.

This evidence closes the bounded text/surface value leaf. Complete role-group,
compound/component and scale construction are the subsequent B1d leaves;
complete theme ownership and metric resolution remain B2.
