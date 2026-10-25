<a id="families"></a>
## Choosing a plot family

[简体中文](../zh-CN/index.html#families)

Constructors return exact `Plot<kind::Family>` values. Sealed capabilities expose
only applicable edits; an unavailable setter is not a runtime option to ignore.
The [Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html)
provides signatures and family-specific options.

| Constructor | Use and input distinction |
| --- | --- |
| `scatter` | Paired numeric observations |
| `histogram` | Distribution from samples or caller-supplied geometry |
| `density` | Smoothed sample distribution |
| `qq` | Sample versus normal quantiles |
| `point_interval` | Positioned estimates and intervals |
| `line` | Ordinary numeric line series |
| `trajectory` | Repeated trajectories with constructor-owned group identity |
| `line_path` | Numeric paths preserving source order rather than ordinary line ordering |
| `interaction` | Categorical condition profiles, grouped estimates and bounds |
| `coefficient` | Named model estimates, optional status/detail/value labels |
| `forest` | Estimates and bounds around a reference, including Log10 display |
| `dumbbell` | Paired positions and connectors |
| `heatmap` | Categorical cell coordinates and constructor-owned continuous color |
| `bar` | Vertical samples or supplied estimates |
| `horizontal_bar` | Horizontal samples or supplied estimates |
| `stacked_bar` | Vertical stacks with required series identity |
| `stacked_horizontal_bar` | Horizontal stacks with required series identity |
| `boxplot` | Grouped sample distribution summaries |
| `violin` | Grouped mirrored sample densities |
| `raincloud` | Samples or complete supplied distribution geometry |
| `regression` | Numeric relationship with display regression |
| `marginal` | Numeric relationship with native marginal displays |
| `matrix` | Scatter matrix of named numeric variables, not an arbitrary cell matrix |
| `beeswarm` | Packed points with categorical rows and continuous color |
| `waterfall` | Ordered deltas and explicit starting total |

Required constructor channels cannot be replaced by optional hue/group/color
setters. Supplied geometry is not permission to rerun analysis. Choose the family
whose scientific grammar matches the intended figure, not merely its mark shape.

`Plot::erase()` explicitly produces `Plot<kind::Any>` for heterogeneous storage.
It preserves common configuration, preflight and publication but does not regain
family-specific setters. Configure exact plots before inserting them into Canvas.
Matrix owns its grid and cannot be nested in Canvas or generic facets. Marginal
can enter Canvas but not generic facets; other families retain their documented
data-dependent restrictions. See [composition](#composition).

This catalog covers all 25 families. The initial [gallery](#gallery) deliberately
contains a few cross-cutting workflows, not a claimed image example for every family.
