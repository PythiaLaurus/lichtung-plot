# T6B1a — Internal Paint Value Evidence

Implementation base: A2 commit `c7f3a77`. Authority: spec94 §3,
`Color`/`OpaqueColor`, and spec93's B1a leaf. Execution-plan status and commit
ownership remain with the main reviewer.

## Representation And Construction

`src/theme/paint.rs` owns the sole complete production paint value:

* `OpaqueColor` retains private red, green and blue `u8` channels.
* `Color` retains one `OpaqueColor` and one `u8` alpha. Its RGB storage is an
  actual production owner of `OpaqueColor`, including nonopaque paints.
* Both values derive `Debug + Clone + Copy + PartialEq + Eq`. Inspection borrows
  `&self`; `with_alpha` consumes and replaces only alpha.
* `rgb`/`rgba` are exact, infallible `const` byte-channel constructors.
  `From<OpaqueColor> for Color` preserves RGB and installs alpha 255.
* Hex construction rejects invalid inputs as the existing crate's `Error::Data`.
  Opaque input accepts exactly `#RRGGBB`; complete input accepts exactly
  `#RRGGBB` or `#RRGGBBAA`. Only ASCII hex digits are decoded, case-insensitively.
  Validation precedes byte decoding: no trimming, shorthand, names, CSS
  expressions, Unicode-digit admission or UTF-8 string slicing occurs.

The private `theme` module exposes these values only within the crate. There are
no new root exports. `render::backend::Rgba`, its RGB tuple conversion trait and
its tuple equality implementation are deleted. Direct consumers use
`theme::Color`; the backend retains neither a color alias nor a color re-export.
The only remaining Rust `Rgba` identifiers belong to `png::ColorType::Rgba`.

`theme::color_from_rgb` is a crate-private boundary factory for current tuple
inputs. It constructs an `OpaqueColor` and converts it into `Color`. There is no
`From<(u8,u8,u8)> for Color`, tuple `PartialEq`, or conversion from `Color` to
`OpaqueColor` that could discard alpha. Assertions that previously compared a
paint with a tuple now compare complete typed expected paints.

## Legacy Opacity Boundary

The existing floating-opacity input remains supported by
`render::backend::quantize_legacy_opacity`, outside the new paint foundation.
Its validation/error message and arithmetic are preserved: finite inclusive
`[0,1]`, multiplication by 255, then nearest/ties-to-even rounding once.
`PointStyle::with_opacity` installs that result with `Color::with_alpha`.
Geometry fixtures use the same helper. The new `Color` has no floating-opacity
method or carrier.

The retained quantizer test covers both endpoints, negative zero, even/odd
half-byte ties, current custom/default opacities, NaN, both infinities, and the
immediately out-of-range binary64 values below zero and above one. Independent
fill and stroke alpha remain separate.

## Changed Rust Files

All 45 Rust files are the new foundation, direct paint carriers/materializers,
or their existing callers/fixtures:

| Owner | Paths |
| --- | --- |
| Foundation/resolution | `src/theme.rs`, `src/theme/paint.rs` (new), `src/theme/resolved.rs`, `src/theme/contrast.rs` |
| Backend/retained primitives | `src/render/backend.rs`, `src/render/coordinates.rs`, `src/render/primitives.rs`, `src/render/svg_backend.rs`, `src/render/geometry.rs`, `src/scene.rs` |
| Layout/text/identity | `src/layout/text.rs`, `src/layout/text/requirements.rs`, `src/layout/kernel/feedback.rs`, `src/layout/panel/tests.rs`, `src/layout/interior/preference.rs`, `src/layout/interior/sizing.rs` |
| Guides/decorations | `src/decorations/legend.rs`, `src/decorations/legend/continuous.rs`, `src/decorations/legend/descriptor.rs`, `src/decorations/panel_text.rs`, `src/decorations/plan/children.rs`, `src/decorations/plan/tests.rs` |
| Orchestration/capture/fixtures | `src/orchestrator/backend.rs`, `src/orchestrator/tests.rs`, `src/orchestrator/tests/data_contracts.rs`, `src/orchestrator/tests/facets.rs`, `src/orchestrator/tests/focused_plot_options.rs`, `src/orchestrator/tests/lifecycle.rs` |
| Mark materializers/fixtures | `src/plots/bar_data.rs`, `src/plots/categorical_distribution/extraction.rs`, `src/plots/density.rs`, `src/plots/heatmap.rs`, `src/plots/heatmap/tests.rs`, `src/plots/histogram.rs`, `src/plots/interaction/native.rs`, `src/plots/interval_estimate.rs`, `src/plots/line/tests.rs`, `src/plots/marginal.rs`, `src/plots/marginal/extraction.rs`, `src/plots/matrix/axis_tests.rs`, `src/plots/qq.rs`, `src/plots/regression.rs`, `src/plots/scatter.rs`, `src/plots/waterfall.rs` |
| Current tuple exact-style boundary | `src/spec/estimate.rs` (`LinePattern::line_style` paint materialization only) |

## Focused Verification

All commands ran locked/offline on Linux. **35 tests passed, zero failed**:
29 unit tests and six integration tests, including three snapshot tests.
No full suite, `verify`, package build or `cargo check` was used.

### Paint, Backend, Contrast And Representative Guides — 22 Passed

```sh
cargo test --locked --offline --lib -- \
  theme::paint::tests:: \
  render::backend::tests::rgb_boundary_conversion_preserves_complete_paint_channels \
  render::backend::tests::legacy_opacity_rounds_nearest_even_once_before_complete_paint \
  theme::contrast::tests:: \
  render::svg_backend::tests::all_byte_channels_round_trip_through_independently_parsed_svg_paints \
  render::svg_backend::tests::filled_paths_emit_independent_fill_and_stroke_alphas \
  render::svg_backend::tests::outlined_points_emit_independent_fill_and_outline_alphas \
  render::svg_backend::tests::opaque_and_transparent_surfaces_have_exact_role_geometry_and_paint \
  render::svg_backend::tests::rgba_surface_alpha_is_emitted_once_on_the_fill_property \
  render::svg_backend::tests::migrated_rgba_paints_share_property_specific_alpha_serialization \
  render::svg_backend::tests::png_preserves_surface_alpha_and_panel_over_root_compositing \
  render::geometry::tests::polygon_bounds_union_only_current_painted_fill_and_stroke \
  render::geometry::tests::transparent_line_has_no_painted_ink_but_still_validates_geometry \
  theme::resolved::tests::built_in_foundational_paints_are_complete_rgba \
  theme::resolved::tests::composed_plot_contrast_uses_the_canvas_owner_surface \
  decorations::legend::tests::local_and_shared_discrete_guides_allocate_complete_custom_glyph_and_frame_ink \
  decorations::legend::continuous::tests::custom_frame_and_owner_missing_strokes_are_allocated_and_retained_unclipped \
  plots::scatter::tests::custom_primary_point_paints_match_prepared_guides_and_hidden_outline_stays_hidden
```

Four new foundation tests use explicit independently encoded expected vectors,
invalid byte lengths and same-length non-ASCII inputs. They cover every value
`0..=255` in each RGBA channel, opaque conversion with alpha 255, mixed-case
decoding, RGB/RGBA constructor agreement, and all replacement alphas while
retaining the source value.

The new SVG round-trip test emits **1,024 rectangles**, parses the resulting XML
with the existing `roxmltree` dependency, and independently decodes RGB with
`u32::from_str_radix` and alpha from the published opacity attribute. It recovers
every byte in every fill channel and independently reordered stroke channels,
including alpha 0, 1 and 255. It also checks opaque-opacity omission and the
absence of element-wide opacity. It does not decode output with `Color::hex`.
This XML test uses the existing dependency's Unix/non-macOS/non-Android target
gate; the foundation tests have no platform gate.

Existing exact SVG strings for independent fill/stroke/point-outline paints are
retained. The PNG test decodes composited translucent red root/blue panel pixels
and still requires `[85,0,170,192]`. Contrast checks retain the ordered binary64
sRGB table digest
`74731e067f75a8f96dfc099a4353f64cd42ef1fccc2e372f509c6c53c55d0cbb`
and complete-role selection over translucent surfaces.

### Defaults, Capture, Paint-Sensitive Caches And Current Exact Inputs — 7 Passed

```sh
cargo test --locked --offline --lib -- \
  theme::resolved::tests::built_in_themes_preserve_every_resolved_value \
  orchestrator::tests::deferred_backend_lazily_begins_and_forwards_complete_surface_calls \
  orchestrator::tests::lifecycle::standalone_lifecycle_preserves_distinct_transparent_root_and_panel_surfaces \
  orchestrator::tests::focused_plot_options::hue_path_style_uses_caller_colors_for_paths_markers_and_legend_lines \
  plots::heatmap::tests::custom_heatmap_outlines_and_cross_leave_selected_cells_and_missing_key_in_place \
  plots::heatmap::tests::missing_fill_and_backdrop_invalidate_cached_intrinsics_with_unequal_contrast_roles \
  layout::text::requirements::tests::transparent_missing_row_census_uses_the_final_fill_and_planning_backdrop
```

### Published Local/Shared Guides And Default Scale/Contrast — 3 Passed

```sh
cargo test --locked --offline --test heatmap -- \
  heatmap_sequential_and_diverging_scales_use_pinned_endpoint_colors \
  local_and_shared_guides_publish_the_same_distinct_close_numeric_labels \
  heatmap_annotations_use_default_and_chained_formats_with_contrasting_text
```

### Existing Expected SVG Snapshots — 3 Passed

```sh
cargo test --locked --offline --test visual-regression -- \
  scatter_smoke_snapshot native_path_snapshots beeswarm_clip_structure_snapshot
```

The expected snapshot files were not changed or repinned, and no `.snap.new`
files were produced.

### Static Checks

* `just lint` passed: `cargo clippy --locked --offline --all-targets -- -D warnings`.
* `just fmt-check` passed.
* `git diff --check` passed.

The only staged dead-code expectations are on the two hex constructors, with
specific B3 exact-paint/B4 scale-input reasons. Each color impl narrowly expects
Clippy's `trivially_copy_pass_by_ref` because spec94 requires borrowed getters.
There is no module-wide warning allowance.

## Default Output Effect And Remaining Staging

Expected default output effect: **none**. The migrated materializers preserve
every RGB/alpha operand and the serializer's property-specific opacity spelling.
The selected default SVG snapshots and exact paint-output tests pass unchanged.
Figure source flags, labels, source identity, scientific calculations, palettes,
gradient stops and mark defaults receive no behavioral edits.

The current public Theme selector and tuple exact styles remain the current
public contract. Complete opaque palette/gradient/data-scale migration remains
B1d/B2/B4 work; the approved shared Heatmap/Beeswarm sequential-gradient change
belongs to B2. The private tuple boundary factory and legacy floating-opacity
quantizer remain necessary until their owning cutovers; Color/OpaqueColor public
exposure and typed exact-style inputs remain B3/B4 work. No functional blocker
was found in these focused checks. Other-platform execution was not performed.

The implementation partition edited Rust files and this evidence document only;
the phase owner records the execution status. After commit-message cleanup, the
owner verified that the restored paint worktree was byte-identical, then repeated
the eight focused foundation/backend/independent-SVG/PNG tests successfully.
Strict all-target Clippy, formatting and whitespace checks also pass. No
dependency or snapshot change belongs to this implementation.
