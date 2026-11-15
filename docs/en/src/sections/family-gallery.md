<a id="family-walkthroughs"></a>
## More scientific walkthroughs

[简体中文](../zh-CN/index.html#family-walkthroughs)

These six programs add eighteen family panels to the earlier workflows. They use
synthetic measurements or caller-supplied illustrations, not research findings.
Read each panel's quantities and axis units before comparing marks. The complete
owned source is included for each figure; it is not a second plotting implementation.

<a id="distribution-diagnostics"></a>
### Distribution diagnostics: shape, reference and summary

[简体中文](../zh-CN/index.html#distribution-diagnostics)

![Density, normal Qq, Boxplot and Violin views of grouped synthetic measurements](gallery/distribution-diagnostics/distribution_diagnostics.svg)

Density, Boxplot and Violin use all three groups of synthetic delays. Density
emphasizes smoothed shape; Boxplot compresses each group into a summary, while
Violin retains its density shape. Qq instead uses only the 72 Warm hold observations
and a quartile-based normal reference line. A departure from that reference is
descriptive evidence here, not a normality test or a significance decision.

The source binds samples to these distinct families rather than turning one
family's prepared geometry into another's data:

```rust
{{#include ../../../../crates/lichtung-plot/examples/18_distribution_diagnostics.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 18_distribution_diagnostics`.
[Complete source](examples/lichtung-plot/18_distribution_diagnostics.rs) ·
[Full-size SVG](gallery/distribution-diagnostics/distribution_diagnostics.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)

<a id="estimate-comparisons"></a>
### Estimate comparisons: choose the question, not just the mark

[简体中文](../zh-CN/index.html#estimate-comparisons)

![PointInterval, Coefficient, Dumbbell, Bar and HorizontalBar with supplied values](gallery/estimate-comparisons/estimate_comparisons.svg)

PointInterval places estimates with bounds; Coefficient shows named additive
contrasts against a zero-effect reference. Dumbbell connects before/after positions,
making the pairing explicit. Bar shows absolute after-hold titer, whereas
HorizontalBar shows after-minus-before change, whose zero means no change.
The panels differ in both question and quantity, not merely mark orientation.

The estimates, interval endpoints and paired positions are supplied by the caller.
Read their labeled units; the renderer is not estimating an inferential model,
calculating these intervals or testing a difference:

```rust
{{#include ../../../../crates/lichtung-plot/examples/19_estimate_comparisons.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 19_estimate_comparisons`.
[Complete source](examples/lichtung-plot/19_estimate_comparisons.rs) ·
[Full-size SVG](gallery/estimate-comparisons/estimate_comparisons.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)

<a id="ordered-processes"></a>
### Ordered processes: identity, path, condition and accumulation

[简体中文](../zh-CN/index.html#ordered-processes)

![Repeated trajectories, an ordered path, condition profiles and a waterfall](gallery/ordered-processes/ordered_processes.svg)

Trajectory keeps repeated-unit group identity. LinePath preserves source order so
a loop is not replaced by an ordinary x-sorted line. Interaction compares profiles
over categorical conditions. Waterfall accumulates ordered deltas from an explicit
starting total. These are four different meanings of connection or order, not four
line styles.

The synthetic quantities and their units belong to the example. Keep unit identity,
path order, condition identity and the starting total in the inputs; do not ask a
theme or a display label to reconstruct them:

```rust
{{#include ../../../../crates/lichtung-plot/examples/20_ordered_processes.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 20_ordered_processes`.
[Complete source](examples/lichtung-plot/20_ordered_processes.rs) ·
[Full-size SVG](gallery/ordered-processes/ordered_processes.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)

<a id="composition-profiles"></a>
### Composition profiles: stacks are not colored point rows

[简体中文](../zh-CN/index.html#composition-profiles)

![Vertical and horizontal stacked profiles beside continuously colored Beeswarm observations](gallery/composition-profiles/composition_profiles.svg)

StackedBar and StackedHorizontalBar require both category and series identities
to describe components of each stack. Beeswarm instead uses categorical rows,
numeric observation positions and a continuous color variable. The three panels
contrast component profiles with individual observations; continuous color is not
a substitute for the required stack series channel.

All quantities are synthetic demonstrations. Use the labeled axes and color guide
to distinguish amount, row identity and the additional measured color quantity:

```rust
{{#include ../../../../crates/lichtung-plot/examples/21_composition_profiles.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 21_composition_profiles`.
[Complete source](examples/lichtung-plot/21_composition_profiles.rs) ·
[Full-size SVG](gallery/composition-profiles/composition_profiles.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)

<a id="joint-measurements"></a>
### Joint measurements: the relationship and its marginals

[简体中文](../zh-CN/index.html#joint-measurements)

![Paired synthetic assay measurements with native marginal displays](gallery/joint-measurements/joint_measurements.svg)

A joint view asks how two measurements vary together; the marginals show how each
variable is distributed on its own. Marginal keeps these views attached to the
paired synthetic assay inputs and their units. The display does not establish
independence, causation or an inferential test.

This is a standalone Marginal figure. It is not a generic faceting recipe:

```rust
{{#include ../../../../crates/lichtung-plot/examples/22_joint_measurements.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 22_joint_measurements`.
[Complete source](examples/lichtung-plot/22_joint_measurements.rs) ·
[Full-size SVG](gallery/joint-measurements/joint_measurements.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)

<a id="multivariate-assay"></a>
### Multivariate assay: named variables, not arbitrary cells

[简体中文](../zh-CN/index.html#multivariate-assay)

![A standalone scatter matrix of named numeric assay variables](gallery/multivariate-assay/multivariate_assay.svg)

Matrix compares several named numeric assay variables through their pairwise
views. Variable names and units make each coordinate pair interpretable; this is
not a Heatmap inferred from a rectangular array, nor a collection of arbitrary
Canvas cells. These synthetic associations are illustrative, not causal findings.

Matrix owns its complete grid. Publish it standalone: it cannot be inserted into
Canvas or generic facets.

```rust
{{#include ../../../../crates/lichtung-plot/examples/23_multivariate_assay.rs:example}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 23_multivariate_assay`.
[Complete source](examples/lichtung-plot/23_multivariate_assay.rs) ·
[Full-size SVG](gallery/multivariate-assay/multivariate_assay.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)
