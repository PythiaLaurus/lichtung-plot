# T5e Final Visual Snapshot Inspection

## Result And Integrated Verification

Eight snapshot changes were individually inspected against their actual SVG
diffs, source producers, and `specs/05_LAYOUT_SYSTEM.md`. Each is intentional
measured allocation/geometry. No unexpected production defect was identified.
All six originally failing tests pass after the phase owner corrected the stale
Interaction fixture's expected clip width from `420.698_242_187_5` to
`418.896_484_375`. Its other components remain `(85, 51, 175)`. The integrated
focused verification passed:

```sh
cargo test --locked --offline --test visual-regression native_interaction_snapshot -- --exact
```

The original partition left that source assertion for integration; this section
records its completed correction, rather than attributing the initial failure to
an unexplained rendering change.

## Inspection Basis

The initial failures and diffs are in `/tmp/opencode/t5e-last-integrations.log`.
Grouped snapshot tests stop at their first failed assertion: accepting Boxplot
after inspection exposed Violin, then Raincloud. Both additional diffs were
inspected separately before updating them.

The relevant spec05 rules are canonical outward ink/required-length rounding
(lines 172–183), columns before rows and measured gutters, independent root and
embedded-Plot text alignment (352–366), first-complete-fitting automatic tick
sources (653–658, 681–707), shared mark/tick/grid projection (734–750), and complete
guide title/key/baseline/padding/border fit (798–855).

Source inspection covered:

- `src/decorations/legend.rs`: `LegendEntry::glyph_ink`,
  `discrete_guide_intrinsics`, `Legend::allocation_metrics`, `measure_assigned`,
  and `visit_calls`. The selected union includes point outlines, rectangular
  miters and line caps. Width includes the title's fit envelope; height uses
  measured asymmetric key ascent/descent. Selected complete ink is validated,
  required extents round outward, and text baselines are no longer snapped to
  whole pixels.
- `src/plots/categorical_distribution.rs`: Boxplot line/fill, Violin fill and
  supplied Raincloud point/fill semantics; `src/plots/density.rs`: retained KDE
  sources and complete curve stroke requirements.
- `src/plots/matrix.rs`: `matrix_guide_entries`, `matrix_legend`, guide reservation
  beside the native Grid, and selected retained guide calls. Matrix uses a
  point/fill key with the density body's 1px outline.
- `src/canvas.rs`, `src/canvas/tracks.rs`, `src/canvas/plot_text.rs`,
  `src/layout/canvas.rs` and `src/decorations/title_block/plan.rs`: actual child
  gutter feedback, assigned cell geometry and immutable local/root text plans.
- `src/plots/interaction/native.rs` and the neighboring Interaction connector/style
  contracts: retained source conditions, uncertainty endpoints, composite guide
  semantics and actual assigned projection.

For the six native full-SVG snapshots, XML comparison against their committed
pre-closure versions verified element count/order, text and all nongeometry
attributes, including colors, opacity, shapes, radii, strokes and dash patterns.
Every scientific mark Y coordinate and every polygon/polyline vertex count was
checked. These checks inspect the long Density/Violin coordinate records without
relying on truncated terminal diff lines. Guide geometry, scientific reprojection,
axes, annotations and root prose were then inspected per case below.

All snapshot names below have the path prefix
`tests/visual_regression/snapshots/visual_regression__` and suffix `.snap`.

## Per-Snapshot Classification

### `matrix_clip_structure_snapshot` — intentional complete guide/Grid geometry

This supplements the already-reviewed native density-outline reserve; it keeps
that earlier change. Against the **entry working-tree** snapshot, the first
column shrinks from `91.033203125` to `90.369140625`, the second column starts at
`187.369140625` rather than `188.033203125`, and its width shrinks from
`90.36328125` to `89.6962890625`. The Grid's right edge moves left by
`1.3349609375px` as the complete title/key/frame requirement is reserved.
Both row heights and the 12px intercolumn/interrow gaps are retained.

All four clip identities remain; diagonal cells retain two density polygons each,
off-diagonal cells four circles each. The actual SVG also retains the earlier
`1.001953125px` projection inset within each cell for complete density ink.
Outer X/Y titles and numeric ticks follow their assigned column geometry; all
their Y coordinates are retained. The guide title becomes
`(293.9912109375, 74.1171875)` and the actual rounded frame is
`(286.365234375, 54.5, 53.134765625, 68.8544921875)`. Its two point/fill keys
retain their colors and strokes. The root title remains unchanged. The structural
snapshot intentionally omits guide key/frame calls, so those were inspected in
the emitted full SVG as well.

### `canvas_root_title_block_odd_size_snapshot` — intentional measured gutter/alignment

The exact root remains `433 × 501`. The scientific left edge changes from
`94.33984375` to `109.26953125`, reducing width from `318.259765625` to
`303.330078125`; right edge `412.599609375`, top `194.4013671875` and height
`183.7978515625` are retained. Clip, background, grid, frame and all 30 scatter
marks consistently use the new assignment. Scientific Y coordinates and styles
remain unchanged.

At the narrower assigned span, automatic X ticks select the first fitting complete
source `0, 5, 10, 15`, replacing `0, 2.5, 5, 7.5, 10, 12.5, 15`. This is permitted
automatic numeric-source selection, not categorical/exact-list omission. Y ticks
remain `5, 10, 15`. Local Plot title/subtitle and X title move right by
`7.46484375px`, half the changed left gutter, staying centered in their owning
assigned cell. Their baselines remain unchanged.

Canvas root title/subtitle coordinates, explicit empty subtitle paragraph and
both start-aligned caption lines are byte-identical. Root text keeps its own
post-inset alignment rather than inheriting the scientific projection. The
standalone odd-root snapshot also passes unchanged. Root-policy independence and
actual assigned-gutter replay neighbors pass.

### `native_boxplot` — intentional outlined composite key reserve

Clip `(95, 51, 438.31640625, 194)` becomes
`(95, 51, 437.3154296875, 194)`. The 1px outlined fill key plus outward enclosure
increases guide width by `1.0009765625px` and height by `1px`; its frame becomes
`(546.615234375, 113.572265625, 77.8837890625, 68.8544921875)`, with the same
outer right edge. All three boxes, fifteen summary segments and six jittered raw
points remain. All 48 scientific Y coordinates preserve quartiles, medians,
whiskers and raw values. Their X positions/widths reproject into the narrower
category tracks.

The comparison bracket stays attached to the two category centers, moving from
`204.5791015625/423.7373046875` to `204.3291015625/422.986328125`; its vertical
geometry and `*` label remain. Humanized/aliased category labels, three axis-title
lines, root title and independent caption remain complete. Guide labels use
measured baselines `152.23046875/167.9072265625` instead of `153/168`.

### `native_violin` — intentional outlined fill key reserve

Clip width `431.31640625 → 430.3154296875` at `(102, 51)`, height `194`.
The frame and guide baselines have the same measured dimensions as Boxplot,
although this key is fill-only and retains its `0.6980392156862745` opacity.
Six polygons (three density bodies and three white summary boxes), twelve summary
segments and six raw points remain, with all 426 scientific Y coordinates and
sample vertex counts preserved. Density width, summary width, raw point centers,
category ticks and comparison bracket follow the new category allocation.
The bracket retains Y `77`, drops to `84`, and the annotation retains Y `69`.
All categories, guide labels, axis-title lines and root prose remain present.

### `native_raincloud` — intentional supplied-cloud composite key reserve

Clip width `438.31640625 → 437.3154296875` at `(95, 51)`, height `194`.
Its outlined fill/point guide uses the same larger frame as Boxplot/Violin.
Both supplied density bodies and summary boxes, two median segments and four raw
points remain: four polygons, two polylines, four circles and all 26 scientific
Y coordinates are preserved. The caller's density, quantile and strip geometry
is reprojected horizontally without changing supplied numeric values. Both
semantic categories and both guide entries remain in order. Root title, three
axis-title lines and independent caption remain complete and outside the mark
clip. This supplied fixture has no significance annotation to remove or change.

### `native_density` — intentional complete capped line/fill guide allocation

Clip width `298.07421875 → 296.6953125` at `(100, 51)`, height `194`.
The measured guide frame changes from width `52.125` to `53.50390625`, retaining
right edge `464.4990234375`; height rounds outward by one quantum to
`67.85546875`. The capped `1.599609375px` line and translucent fill participate in
the union with title/label fit requirements; a cap diameter alone is not the
net width delta of that maximum.

Both area polygons, all 128 samples per curve and all 254 outline segments remain.
All 768 scientific vertex Y coordinates and every paint attribute are identical.
The full support X coordinates transform affinely between the retained support
endpoints with maximum error `0.7993` quantum, consistent with canonical rounding.
Using the raw clip-width ratio instead of the stroke-cleared support span would
incorrectly flag the fixed curve-edge reserve as a source change. The focused KDE
support/root-feedback regression passes without re-preparing KDE.

X ticks remain `0, 5, 10`; Y ticks remain `0, 0.1, 0.2`. Mark/grid/tick positions
share the new projection. The two guide entries retain A/B order, colors, fill
opacity and capped line style. Their labels use baselines
`152.7294921875/167.9072265625`, and the guide title uses `133.689453125`.
Root title, axis-title lines and captions remain complete.

### `native_dumbbell` — intentional complete point ink/fractional enclosure

Clip width changes by one quantum, `349.833984375 → 349.8330078125`, at
`(188, 51)`, height `194`. The guide frame grows from
`72.365234375 × 68.814453125` to `72.3662109375 × 69.2158203125`.
Its 4px point radius already fits the nominal horizontal handle, but the
`0.400390625px` outline contributes to vertical key ink and the complete
enclosure rounds outward. Six colored endpoints and three connectors remain;
all twelve scientific Y coordinates preserve the three rows. Numeric endpoint
values and grid/ticks retain shared projection, with only quantum-level X changes.

Before/After labels remain with their caller colors `#14508C/#B43C28`. The first
guide baseline becomes `152.0498046875`; the second remains `167.9072265625`.
The long category still wraps into two lines, all three categories remain, and
root/axis/caption prose is retained. PointInterval, Coefficient and Forest pass
their unchanged snapshots in the same grouped estimate test.

### `native_interaction` — intentional capped composite guide

Clip width `420.6982421875 → 418.896484375` at `(85, 51)`, height `175`.
Guide width `104.5009765625 → 106.302734375` accounts for both connector round
caps, each outward half-cap `0.900390625px`, plus complete enclosure rounding.
Measured asymmetric Circle/TriangleUp entry ink changes frame height from
`68.814453125` to `68.53515625`, centered at the same panel-parallel position.
The frame is now `(518.1962890625, 104.232421875, 106.302734375, 68.53515625)`.

The source order retains five uncertainty segments, two connectors, three circle
markers and two triangle markers. All 24 scientific vertex Y coordinates,
uncertainty endpoints, paints, marker outlines/alpha and dashed second connector
remain identical. Category centers become
`154.81640625, 294.4482421875, 434.080078125`; ticks, markers, intervals and
connectors agree. Missing second-series middle condition stays missing; no
synthetic marker is introduced. The aliased first category retains its two lines.
Both composite guide labels and three axis-title lines remain complete, and
root title/caption coordinates remain unchanged.

The updated snapshot SVG exactly equals the independently emitted initial
closure-run SVG in `/tmp/lichtung-plot-test-495318-0/native-interaction.svg`.
Its SHA-256 is
`268a8c79b05be4268c417f1e7a257763b71b39075eeb1c4cc318b6fc73e51f59`.
That run's PNG header is exactly `641 × 401`. The integrated focused test passes
both the corrected width assertion and the reviewed snapshot, including its PNG
dimensions and title/axis/caption containment checks.

## Historical Attribution And Existing Work

The committed pre-closure expectations remain recoverable at
`82455c5ebdc1d0cdc75842a8cedf3a59165c713b` under the same snapshot paths.
The latest modifying commits for those historical versions are:

| Snapshot | Last committed geometry/paint checkpoint |
| --- | --- |
| Matrix structural | `ecb0fdb` — native variable Grid tracks |
| Canvas odd-root | `316d5a4` — independent point/outline paints; local text ownership introduced in `acbb4f1` |
| Boxplot | `316d5a4` — independent point/outline paints |
| Violin | `316d5a4` — independent point/outline paints |
| Raincloud | `de37531` — sampled/supplied cloud category pitch |
| Density | `f3f1045` — curve support/root-feedback clearance |
| Dumbbell | `316d5a4` — independent point/outline paints |
| Interaction | `316d5a4` — independent point/outline paints |

Matrix additionally had an already-reviewed uncommitted T5e7 density-outline
reserve when this partition began; the geometry deltas above explicitly compare
against that entry checkpoint from the failure log. That reserve is preserved.
The other six previously modified snapshots (Beeswarm, Marginal and four native
bar-family snapshots) were not edited by this partition. Their attribution stays
in the root/guide audit documents, including
`docs/t5e7-guide-cache-closure.md` and `docs/t5e-marginal-audit.md`.

All eight updates used `apply_patch`. Reviewed `.snap.new` contents were moved
into their snapshot paths with transient assertion-line metadata removed;
Interaction used the inspected emitted SVG because the stale fixture prevented
Insta from creating a `.snap.new`. No `.snap.new` files remain.

## Exact Checks

Passing focused snapshot commands:

```sh
cargo test --locked --offline --test visual-regression matrix_clip_structure_snapshot -- --exact
cargo test --locked --offline --test visual-regression root_title_block_ -- --nocapture
cargo test --locked --offline --test visual-regression native_ -- --skip native_bar_family_snapshots --skip native_path_snapshots --skip native_interaction_snapshot
cargo test --locked --offline --test visual-regression clip_structure_snapshot -- --skip matrix_clip_structure_snapshot
```

Counts are respectively **1, 2, 3, 3** passing tests. The native groups exercise
Boxplot/Violin/Raincloud, Histogram/Density/QQ/Regression and
PointInterval/Coefficient/Forest/Dumbbell, including their actual PNG dimensions.
Focused categorical discovery initially failed at Violin and then Raincloud;
the final native-group run passes all three groups after individual inspection.

Passing overlapping source/Integration contracts:

```sh
cargo test --locked --offline --lib odd_canvas_root_policy_is_independent_of_local_panel_alignments
cargo test --locked --offline --lib canvas_gutter_remeasurement_uses_the_assigned_projection
cargo test --locked --offline --lib local_and_shared_discrete_guides_allocate_complete_custom_glyph_and_frame_ink
cargo test --locked --offline --lib density_curve_support_clears_automatic_edges_without_repreparing_kde
cargo test --locked --offline --test interactions connector
cargo test --locked --offline --test interactions styles::native_interaction_preserves_exact_root_svg_bytes -- --exact
cargo test --locked --offline --test interactions styles::interaction_style_legend_matches_each_trace_glyph -- --exact
```

Counts are **1, 1, 1, 1, 5, 1, 1**. An initial abbreviated unit filter combined
with `--exact` selected zero tests and was rerun with the correct substring filter.
An exploratory nonexistent Interaction fingerprint filter likewise selected zero;
the actual `styles::native_interaction_preserves_exact_root_svg_bytes` test above
was then run. Neither zero-test run is counted as verification. The Cargo target
is `visual-regression`; an initial underscore-spelled target was rejected before
running tests and corrected.

The partition's initial `native_interaction_snapshot -- --exact` rerun exposed
the stale width assertion. The phase owner corrected it and reran the same
command successfully: **1 passed**, with no snapshot update needed afterward.

`just lint` passed strict locked/offline all-target Clippy with `-D warnings`.
`just fmt-check` and `git diff --check` passed. No broad test suite or commit was
performed in this partition.
