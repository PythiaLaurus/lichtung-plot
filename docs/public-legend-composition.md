# Legend Composition

LegendComposition is a closed policy: Combined(default) or Separate. Attach it
through Plot::legend_composition on all 25 exact families or Canvas for Shared.
Any has no setter. It is independent of Theme, placement, visibility and sizing.

Combined keeps complete observed symbol keys. Separate explains actual mapped
channels in ordered group-style, joint, hue-color, literal and supplemental-series
sections. Continuous color and declared alpha content follow inside the same
frame. There is one placement/collision owner, with no per-section positions.

An Interaction with explicitly configured group styles and independent hue can
produce two sections. A dense five-group/four-hue crossing has 20 Combined keys
versus 5+4 Separate keys; sparse three-pair data can have 3 versus 4. Marginal
explanations do not assert unobserved combinations or change marks/statistics.
Repeated configured shapes/colors are valid.

Same raw-field group/hue stays joint. Unsupported independent pairs without a
mapped group-style channel reject before fonts, even for one pair/equal RGB.
Hue-only/positioned-series keys preserve complete actual geometry. Row-hue paths
explain point colors, not neutral connectors. Dumbbell/Regression/sign domains
remain literal. Labeled Line PointLayers form the final series section.

Group projection neutralizes hue-mapped RGB to the source Plot's legend-label RGB,
without borrowing text alpha. Hue projection neutralizes only mapped Interaction
shape/pattern to Circle/Solid. Independent paints/outlines/geometry remain actual.
Both policies neutralize data-bound context alpha to 255; declared alpha has its
own explanation and Undeclared does not invent one.

```rust
use lichtung_plot::{Canvas, CanvasLegendPolicy, LegendComposition};

let canvas = Canvas::grid(1, 2)?
    .legend_composition(LegendComposition::Separate)
    .legend_policy(CanvasLegendPolicy::Shared);
# let _ = canvas;
# Ok::<(), lichtung_plot::Error>(())
```

Standalone/facets use owning Plot selection, PerPlot each child, Shared Canvas.
Shared ignores stored child selections without mutating them; returning to PerPlot
uses those selections again. Shared chrome/text comes from Canvas while keys
retain source Plot paint/neutral RGB. Repeated raw identities require equal
labels/full keys; conflicting aliases/styles or ambiguous nonempty section titles
reject before fonts. Explicit child guide positions remain incompatible with Shared.

Hidden suppresses explanatory semantics/fonts but still validates scientific
bindings/scales, and visible marks still coordinate raw color domains. Matrix
remains native standalone, Marginal nongeneric-facetable. Native Heatmap/Beeswarm
right tracks and Matrix trailing track place their guides only once.

See [data alpha](public-data-alpha.md), the
[guide specification](../specs/08_GUIDE_SYSTEM.md) and
`examples/06_legend_sections.rs` for the corresponding API use.
