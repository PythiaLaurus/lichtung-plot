<a id="families"></a>
## Choosing a plot family

[简体中文](../zh-CN/index.html#families)

Constructors return exact `Plot<kind::Family>` values. Sealed capabilities expose
only applicable edits; an unavailable setter is not a runtime option to ignore.
The [Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html)
provides signatures and family-specific options.

Every row links to a same-page walkthrough with an actual figure and its owned
source. A multi-panel example demonstrates each named family's distinct input
grammar; sharing an executable does not make its panels one scientific scenario.

| Constructor | Use and input distinction | Figure and source walkthrough |
| --- | --- | --- |
| `scatter` | Paired numeric observations | [Enzyme response](#first-plot) |
| `histogram` | Samples or caller-supplied geometry | [Calibration residuals](#calibration) |
| `density` | Smoothed sample distribution | [Distribution diagnostics](#distribution-diagnostics) |
| `qq` | Sample versus normal quantiles | [Distribution diagnostics](#distribution-diagnostics) |
| `point_interval` | Positioned estimates and intervals | [Estimate comparisons](#estimate-comparisons) |
| `line` | Ordinary numeric line series | [Response over time](#time-response) |
| `trajectory` | Repeated trajectories with constructor-owned group identity | [Ordered processes](#ordered-processes) |
| `line_path` | Numeric paths preserving source order | [Ordered processes](#ordered-processes) |
| `interaction` | Categorical condition profiles, grouped estimates and bounds | [Ordered processes](#ordered-processes) |
| `coefficient` | Named estimates, optional status/detail/value labels | [Estimate comparisons](#estimate-comparisons) |
| `forest` | Estimates and bounds around a reference, including Log10 display | [Effects and intervals](#effect-intervals) |
| `dumbbell` | Paired positions and connectors | [Estimate comparisons](#estimate-comparisons) |
| `heatmap` | Categorical cell coordinates and constructor-owned continuous color | [Response heatmap](#expression-heatmap) |
| `bar` | Vertical samples or supplied estimates | [Estimate comparisons](#estimate-comparisons) |
| `horizontal_bar` | Horizontal samples or supplied estimates | [Estimate comparisons](#estimate-comparisons) |
| `stacked_bar` | Vertical stacks with required series identity | [Composition profiles](#composition-profiles) |
| `stacked_horizontal_bar` | Horizontal stacks with required series identity | [Composition profiles](#composition-profiles) |
| `boxplot` | Grouped sample distribution summaries | [Distribution diagnostics](#distribution-diagnostics) |
| `violin` | Grouped mirrored sample densities | [Distribution diagnostics](#distribution-diagnostics) |
| `raincloud` | Samples or complete supplied distribution geometry | [Beyond the average](#distributions) |
| `regression` | Numeric relationship with display regression | [Calibration](#calibration) |
| `marginal` | Numeric relationship with native marginal displays | [Joint measurements](#joint-measurements) |
| `matrix` | Scatter matrix of named numeric variables, not arbitrary cells | [Multivariate assay](#multivariate-assay) |
| `beeswarm` | Packed points with categorical rows and continuous color | [Composition profiles](#composition-profiles) |
| `waterfall` | Ordered deltas and explicit starting total | [Ordered processes](#ordered-processes) |

Required constructor channels cannot be replaced by optional hue/group/color
setters. Supplied geometry is not permission to rerun analysis. Choose the family
whose scientific grammar matches the intended figure, not merely its mark shape.

`Plot::erase()` explicitly produces `Plot<kind::Any>` for heterogeneous storage.
It preserves common configuration, preflight and publication but does not regain
family-specific setters. Configure exact plots before inserting them into Canvas.
Matrix owns its grid and cannot be nested in Canvas or generic facets. Marginal
can enter Canvas but not generic facets; other families retain their documented
data-dependent restrictions. See [composition](#composition).

The [Python interface](#python) exposes the same 25 families as exact classes,
not one universal owner with every possible method. The [gallery](#gallery)
organizes their figures by scientific question rather than by setter inventory.
