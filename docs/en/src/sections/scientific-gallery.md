<a id="gallery-recipes"></a>
## Scientific gallery recipes

[简体中文](../zh-CN/index.html#gallery-recipes)

Each figure starts with a question, not a list of setters. All values in this
showcase are synthetic or explicitly supplied illustrations, not research results.
The examples use the same [gallery theme](examples/lichtung-plot/support/gallery.rs) and
[output helper](examples/lichtung-plot/support/output.rs); examples needing generated variation
also use the packaged [synthetic inputs](examples/lichtung-plot/support/synthetic.rs).

<a id="distributions"></a>
### Raincloud: look beyond the average

![Sample distributions, summaries and observations](gallery/distributions/distributions.svg)

Practice can change the center without removing a long tail. A raincloud retains
all three views of each sample: density, summary and individual observations.
Here each condition has 80 fixed synthetic observations, with different centers,
spreads and a small right-skew contribution. No significance test is performed.

```rust
{{#include ../../../../crates/lichtung-plot/examples/13_distributions.rs:distribution-values}}
```

Pass samples, not precomputed quartiles, to `RaincloudInput::samples`. The renderer
owns display summaries; the example makes no inferential claim. Category identity
already belongs to this input, so it does not require a second hue channel.

```rust
{{#include ../../../../crates/lichtung-plot/examples/13_distributions.rs:distribution-plot}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 13_distributions`.
[Complete source](examples/lichtung-plot/13_distributions.rs) · [Full-size SVG](gallery/distributions/distributions.svg) ·
[RaincloudInput API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.RaincloudInput.html) · [Back to gallery](#gallery)

<a id="expression-heatmap"></a>
### Heatmap: coordinated responses through time

![An ordered matrix of synthetic expression patterns](gallery/expression-heatmap/expression_heatmap.svg)

An eight-by-eight matrix exposes early induction, transient suppression and later
recovery without turning each cell into a separate chart. Rows and time labels are
supplied in their intended order; the renderer does not infer clustering or biology.
The column spacing is categorical, not proportional to elapsed time.

```rust
{{#include ../../../../crates/lichtung-plot/examples/14_expression_heatmap.rs:expression-values}}
```

An explicit diverging scale places zero at a neutral color and fixes the complete
domain to [-2, 2]. That shared reference matters more than decorating every cell
with a number. Negative and positive branches remain comparable.

```rust
{{#include ../../../../crates/lichtung-plot/examples/14_expression_heatmap.rs:expression-plot}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 14_expression_heatmap`.
[Complete source](examples/lichtung-plot/14_expression_heatmap.rs) · [Full-size SVG](gallery/expression-heatmap/expression_heatmap.svg) ·
[ContinuousColorScale API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.ContinuousColorScale.html) · [Back to gallery](#gallery)

<a id="effect-intervals"></a>
### Forest: estimates in the context of uncertainty

![Supplied effect estimates and intervals around zero](gallery/effect-intervals/effect_intervals.svg)

The point is not the entire result. Complete intervals show which illustrative
effects remain close to the no-effect reference and which are more separated.
Every endpoint below is supplied by the caller; these are not clinical results,
computed confidence intervals or automatic significance decisions.

```rust
{{#include ../../../../crates/lichtung-plot/examples/15_effect_intervals.rs:effect-plot}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 15_effect_intervals`.
[Complete source](examples/lichtung-plot/15_effect_intervals.rs) · [Full-size SVG](gallery/effect-intervals/effect_intervals.svg) ·
[Bounds API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Bounds.html) · [Back to gallery](#gallery)

<a id="time-response"></a>
### Line and band: a response unfolds over time

![Three response dynamics with supplied envelopes](gallery/time-response/time_response.svg)

These three synthetic curves differ in both amplitude and speed. Their shaded
envelopes are supplied illustrative bounds, not fitted confidence intervals.
The lower and upper values retain exactly the same row alignment as each curve.

```rust
{{#include ../../../../crates/lichtung-plot/examples/16_time_response.rs:response-values}}
```

`hue` separates the three series, while `band` consumes their aligned bounds.
Markers are disabled because the point of this figure is the complete trajectory
and its envelope, not independent sampled observations.

```rust
{{#include ../../../../crates/lichtung-plot/examples/16_time_response.rs:response-plot}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 16_time_response`.
[Complete source](examples/lichtung-plot/16_time_response.rs) · [Full-size SVG](gallery/time-response/time_response.svg) ·
[Bounds API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Bounds.html) · [Back to gallery](#gallery)

<a id="calibration"></a>
### Canvas: calibration and residual structure

![A calibration relationship beside its residual distribution](gallery/calibration/calibration.svg)

One panel asks whether signal follows concentration; the other reveals the
distribution of errors relative to the known generating line. Those residuals
are not residuals from the displayed descriptive OLS fit. Keeping that distinction
explicit avoids turning a convenient synthetic example into a misleading analysis.

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-values}}
```

The regression is descriptive, with its interval band explicitly disabled.
The histogram uses density normalization, a KDE and a rug of the original residuals.

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-panels}}
```

Both plots inherit one complete Canvas theme. Composition gives them a common
publication frame without pretending that their different x axes are shared.

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-canvas}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 17_calibration`.
[Complete source](examples/lichtung-plot/17_calibration.rs) · [Full-size SVG](gallery/calibration/calibration.svg) ·
[Canvas API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Canvas.html) · [Back to gallery](#gallery)
