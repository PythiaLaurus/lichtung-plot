# Observation Alpha In Swarms And Native Linked Panels

Source baseline: `1592e12`. Beeswarm, Marginal and Matrix now retain and consume
their original observation alpha. Together with the coordinate, path/estimate and
distribution receipts, this completes all **17 admitted point-family consumers**.
Public alpha APIs and the other component/guide leaves remain subsequent work.

## Actual Consumers

- Beeswarm retains `ObservationAlpha` in each source-ordered prepared row. The
  scientific packing iterator still contains the same category, numeric value,
  continuous-color sample and independent source ID. Color-domain preparation,
  Shared color remapping and guide samples remain independent of point alpha.
- A data-alpha-bound Beeswarm packs each observation's complete glyph, independent
  of transparent byte values. This supports a visible override of a theme-zero
  fill and prevents alpha changes from removing or repacking coincident records.
  Theme-only packing preserves its existing role-paint policy. Intrinsic numeric
  ink, containment, selected geometry and automatic clearance use actual final
  point paint; capture retains final styles rather than reading alpha inputs.
- Marginal groups replace their source-ID-only observation storage with complete
  alpha operands. Joint-finite filtering retains original slots, while densities,
  bandwidths, linked scientific ranges and source-ID feedback keep their existing
  inputs. Central point batches retain paint order. Their actual painted extrema
  and canonical glyph bounds supply intrinsic and automatic-ink requirements.
- Matrix groups likewise retain original operands. Each off-diagonal cell freezes
  contiguous final-style batches with the matching original observation members.
  The same member's byte is reused in each actual off-diagonal occurrence. Its
  point dimensions are shared by native hard tracks, natural preferences and
  assigned tracks. Diagonal KDE values, weights, supports and independent density
  outlines are unaffected; the native owner and Matrix Canvas boundary remain.

## Focused Evidence

Seven library tests passed:

- All five `plots::beeswarm::` tests: three retained exact-packing contracts,
  existing Shared diverging-color remapping, and the new actual point-alpha
  source/packing contract.
- `plots::marginal::alpha_tests::central_alpha_retains_filtered_original_members_and_does_not_change_densities`.
- `plots::matrix::alpha_tests::every_off_diagonal_replays_original_observation_alpha_without_changing_diagonal_sources`.

Independent witnesses include Beeswarm source IDs `[10,20,70,80,90]` paired with
original-slot bytes `[64,230,0,128,0]`, identical coincident point positions under
varying/all-zero/all-255 alpha and empty painted point geometry for all-zero fills
without outlines. Marginal keeps original slots `[0,2,3]`, actual batches
`[0,128,0]` and unchanged x/y KDE sources. Matrix keeps selected slots `[2,0,1]`
separate from IDs `[300,100,200]`, reuses `[0,128,0]` in both off-diagonal cells
and retains identical diagonal positions/densities/maxima after an alpha change.
All three actual selection contracts include a nonzero override of theme-zero
point fill.

Six public integration tests passed, using the following filters and targets:

- `distributions`: `beeswarm_is_deterministic_and_renders_continuous_colorbar` (1).
- `marginal-matrix`: `marginal_preserves_scientific_values_and_linked_axes` (1),
  `matrix_preserves_observations_and_variable_labels_for_each_theme` (1), and
  `hue_only_marginal_and_matrix_split_multiple_hues` (1).
- `composition`: `shared_beeswarm` (2 actual Shared-domain publications).

All commands used `cargo test --locked --offline` with explicit filters. Strict
`just lint`, `just fmt-check` and `git diff --check` passed. The three new library
tests replace, within this same change:

- `plots::beeswarm::packing::tests::coincident_packing_retains_every_input_point`;
- `plots::qq::tests::qq_opposite_extreme_auto_range_retains_endpoints_with_scaled_ticks`;
- `plots::qq::tests::reference_clipping_omits_nonhorizontal_line_outside_rectangle`.

The obsolete test-only root `nice_ticks` re-export is removed; panel-local numeric
source coverage remains. No deletion is banked. Discovery remains **671 library
tests including 17 ignored**, in `/tmp/opencode/alpha-native-{count,ignored}.log`.

This is focused observation-consumer closure. Explicit series/uncertainty strokes,
aggregate/cell/bin/band fills, the unified automatic guide envelope, public cutover
and integrated full/release/time/RSS gates are not supplied by these checks.
