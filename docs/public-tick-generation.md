# Public Numeric Tick Generation

The interval-only public API is replaced by the single opaque crate-root
`TickGeneration`. This receipt follows the source/API design at `19a0744` and the
private bounded source implementation at `f5bdaba`; it closes the public cutover,
not parent/root/output closure or integrated release verification.

## One Current Value And Attachment

`TickGeneration` is `Clone + Copy + Debug + Default + PartialEq`. Its private
selection is Automatic, Additive, Log10Decades, Target or Maximum:

- Default and `automatic()` select Automatic and provide an explicit reset.
- `additive(f64)` requires finite strictly positive spacing.
- `log10_decades(u32)` requires a positive exponent step.
- `target(u32)` and `maximum(u32)` require `1..=10_000`.
- `is_automatic`, `additive_step`, `decade_step`, `target_count` and
  `maximum_count` inspect the selection without exposing unvalidated variants.

`Plot::x_tick_generation` and `Plot::y_tick_generation` replace their axis's whole
choice under the unchanged NumericX/Y gates. No old type, setter, alias,
deprecation, optional interval/count chain or alternative planner remains in
compiled repository code. Source programs retain the previously verified nice
grammar, checked cardinality, candidate order, first-fitting complete labels and
original-domain midpoint.

Exact ticks remain separate and authoritative in either setter order, including
intentionally unlabeled positions. Generated count instructions do not truncate
an exact list. Actual categorical axes reject non-Automatic selection before font
discovery; interval scale-kind validation does not misclassify Target/Maximum.

## Retained Inputs And Consumers

`AxisSpec.generation` and `AxisSemanticInputs.generation` hold the concrete value,
with Automatic as default. This same value reaches `Sources`,
`numeric_tick_plan`, `NumericTickPlan::measure_selected_source`, ordinary layout
and the Forest exponent-space route. Matrix's private numeric cells explicitly
select Automatic without acquiring a public axis attachment.

Explicit copies in `PlotRenderContext::layout_with_ticks`, shared-axis preparation
and composed-input retention preserve the complete generation selection alongside
independent exact ticks, format, limits and range provenance. Both-axis equality
keeps Target and Maximum distinct even with equal counts/output extent. Feedback
is Automatic tag 0, Additive 1 plus exact f64, Log10Decades 2 plus u32, Target 3
plus count, Maximum 4 plus count. The old optional interval feedback path and the
private behavioral leaf's temporary dead-code expectations are removed.

## Public Semantic Witnesses

The existing constructor/context inventory now attaches the actual methods to
all 32 admitted family/axis combinations: 17 NumericX and 15 NumericY. Neither
Heatmap, Matrix nor Any gains a method. Compile-fail docs exercise both axes for
these excluded families, categorical Bar x, Beeswarm y, and private raw variants.

`public_tick_generation_counts_replace_complete_choices_and_automatic_resets`
renders a complete public Scatter with explicit `[0,1]` and `[10,14]` domains:

- additive 0.2 followed by target four yields x labels `0.00,0.50,1.00`, proving
  whole-choice replacement and permitted nice-step deviation;
- a later target five yields `0.00,0.25,0.50,0.75,1.00`;
- maximum three yields the complete three-position source, without changing
  the independent y target's five labels;
- Automatic/default reset clears an otherwise invalid logarithmic instruction
  and a singleton maximum, reproducing the unconfigured complete SVG.

The existing public exact-order test now includes two-position exact lists with
empty labels, exceeding generated target/maximum one. Both setter orders and
Automatic produce the same exact output without applying the generated bound.
The existing actual-categorical refusal test covers additive, target and maximum
in standalone and Canvas under an unavailable font and observes no discovery.

## Verification

Actual focused results:

| Command/filter | Passed |
| --- | ---: |
| `cargo test --locked --offline --lib layout::panel::sources::tests::` | 21 |
| `cargo test --locked --offline --lib builder::facets::generation_tests::` | 6 |
| `cargo test --locked --offline --lib layout::panel::tests::exact_tick_configuration` | 3 |
| `cargo test --locked --offline --lib layout::panel::tests::frozen_axes_and_grid` | 1 |
| `cargo test --locked --offline --lib automatic_selection_and_frozen_axis_agree_at_quantum_fit_boundary` | 1 |
| `cargo test --locked --offline --lib adjacent_ordinary_and_large_ticks_keep_both_distinguishable_endpoints` | 1 |
| `cargo test --locked --offline --test lines public_tick_generation` | 2 |
| `cargo test --locked --offline --test rendering constructors_and_contextual_builders_preserve_exact_plot_markers` | 1 |
| `cargo test --locked --offline --test estimates forest_log_scale_projects_exact_ticks_from_raw_data_space` | 1 |

This is 33 distinct focused library tests and four external contracts. The
strengthened categorical test was rerun after extending its coverage. No unit
test function is added, deleted, merged or moved into integration; the six
composed tests' module is renamed from `interval_tests` to `generation_tests`.
Actual library discovery remains **671, including 17 ignored**. The new external
test exercises the public attachment/rendering boundary rather than relocating
private source tests.

Strict all-target `just lint`, formatting/diff checks, **119 doctests**, warning-denied
`just docs` and offline `just package` pass. Logs use
`/tmp/opencode/tick-generation-public-*.log`. Current architecture/rendering/theme/
layout contracts, construction handoff, property ownership and README describe
only the replacement. Historical interval receipts retain their recorded context.

Full-suite, parent/root breadth, cache-disabled multi-format closure and release
time/RSS measurements are not claimed by this cutover; spec97 owns those gates.
