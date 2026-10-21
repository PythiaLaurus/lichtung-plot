# Observation Alpha In Categorical Distributions And Paired Endpoints

Source baseline: `361f108`. This leaf connects private point-alpha inputs to
Boxplot, Violin, sampled/supplied Raincloud and Dumbbell. Fourteen of the seventeen
admitted point families now consume their prepared observation operands.

## Retained Membership And Paint

- `DistributionObservation` retains `ObservationAlpha` alongside its scientific
  value. Original input slots and independent diagnostic/jitter source IDs survive
  category/series grouping and selected-row preparation. Alpha does not enter
  quantiles, KDE samples, source-row duplicate validation or significance maxima.
- Boxplot fliers retain actual observations, replacing the former value-only
  representation. Their existing numeric paint order is retained with distinct
  original occurrences for duplicate values. Raw overlays continue to replace the
  separate flier display without dropping any scientific observation.
- The common distribution scatter attachment validates membership alignment and
  freezes contiguous equal-style batches through the existing `Mark::Scatter`
  renderer. It neither sorts by alpha nor removes transparent/coincident points.
- Sample Raincloud uses source IDs for strip jitter. Supplied Raincloud resolves
  slots in original group/point order, across group boundaries, against the actual
  point count rather than its zero-row synthetic input. Both freeze final point
  fill alpha while preserving independently owned outlines and summary geometry.
- Intrinsic point ink and Raincloud strip clearance use resolved observation paint.
  The canonical point-ink helper handles a data override making a theme-zero fill
  visible. Distribution automatic clearance already consumes the same final marks.
- Dumbbell stores final left/right endpoint styles per record. A record's alpha
  replaces both fill bytes after exact endpoint RGB/RGBA resolution. Outline paint,
  connector paint and scientific pair/category order retain their own ownership.
  Its automatic clearance now measures the actual full painted pairs.

## Focused Evidence

Five library contracts passed:

1. `plots::boxplot::alpha_tests::duplicate_outliers_keep_sorted_original_occurrences_and_replacement_paint`:
   sorted outlier slots `[7,0,5]`, distinct source IDs `[914,900,910]`, final bytes
   `[128,230,0]`, retained coincident outliers and 80px actual point ink despite a
   theme-zero fill without outlines.
2. `plots::violin::alpha_tests::overlay_alpha_follows_original_slots_and_source_jitter_through_positioned_groups`:
   selected slots `[3,0,2]` remain distinct from source IDs, actual grouped batches
   retain `[0,230,64,64]`, and jitter still uses the independent source identity.
3. `plots::dumbbell::tests::exact_endpoint_rgba_reaches_marks_literal_keys_and_canonical_capture_without_outline_changes`:
   existing exact-paint/canonical-capture checks plus record bytes 64 and 230 on
   both endpoint RGB values, retaining the independent outline alpha 61.
4. `theme::mark::tests::sampled_and_supplied_strips_keep_their_own_style_and_guide_when_raw_roles_change`:
   mandatory strips retain alternating bytes 0/230 and their own geometry/outline
   policy while the unused raw-overlay role changes.
5. `geometry::validation_tests::supplied_geometry_preserves_support_and_rejects_nonfinite_values`:
   supplied zero-row input freezes bytes 0/230 across two actual geometry groups;
   support and invalid nonfinite geometry checks remain.

Six existing public caller contracts passed:

- `categorical::dumbbell::dumbbell_renders_connectors_endpoint_legend_and_all_replicates`
- `categorical::plots::boxplot_renders_fliers_by_default`
- `categorical::plots::violin_draws_inner_whisker_mark`
- `distributions::raw_overlays::plot_source_rows_preserve_jitter_after_compaction_and_reordering`
- `distributions::raw_overlays::facets_keep_original_source_rows_for_jitter`
- `distributions::composites::supplied_and_sample_rainclouds_preserve_labels_colors_legend_and_recorded_marks`

Each command used `cargo test --locked --offline`, with its explicit library/module
or integration-target filter; every filter ran a nonzero number of tests. Strict
`just lint`, `just fmt-check` and `git diff --check` passed.

The two new unit tests replace
`render::geometry::tests::collision_pairs_keep_their_geometric_overlap_on_repeated_queries`
and `plots::qq::tests::qq_quartile_line_is_horizontal_for_constant_sample` in this
same change. No deletion is banked. Discovery remains **671 library tests including
17 ignored**, recorded in `/tmp/opencode/alpha-distribution-{count,ignored}.log`.

The remaining Beeswarm, Marginal and Matrix point consumers keep the observation
leaf open. Series strokes, aggregate fills, unified alpha guides, public exports
and integrated full/release verification remain subsequent work.
