# Themes, fonts and scales

[简体中文](../zh-CN/themes.html)

Theme is a complete immutable value with eight components: SurfaceTheme,
TypographyTheme, AxisTheme, LegendTheme, CompositionTheme, MarkTheme,
AnnotationTheme and ColorScales. `publication`, `nature`, `science` and `chi` are
ordinary complete constructors, not global state. Standalone omission selects
Publication. An unthemed Canvas child inherits the root; explicit Plot Theme
replaces that complete value. Facets add no appearance scope.

`Theme::new` accepts all components. `customize` makes a working copy;
ThemeBuilder::new accepts a complete base. Component `property(value)` edits
validate complete values; paired readers are `get_property()`. Read-only Theme
components retain names such as `typography()`. `build()` validates without font discovery.

```rust
{{#include ../../../crates/lichtung-plot/examples/02_themes.rs:example}}
```

All eighteen text roles own stack, RGBA color, positive fractional CSS font size,
weight and leading. FontFamily::named is one literal identity, not a CSS stack
expression; GenericFontFamily explicitly chooses Serif/SansSerif/Monospace.
FontStack retains nonempty ordered entries, including duplicates. Availability is
checked when a render consumes the role, not during theme construction.
ThemeBuilder::font_stack edits all roles once; later local replacements win normally.

RelativeSpacing::new retains the exact binary rational; `ratio(u32,u32)` retains
the reduced mathematical ratio. Spacing resolves against the owning text role,
not root dimensions or approximate `as_f64` inspection. A Shared guide's chrome
uses Canvas, while source mark appearance remains Plot-owned.

## Color and exact paint

Data mappings use opaque OpaqueColor; exact/role paint uses RGBA Color with
independent alpha. A fixed QualitativePalette must cover the raw domain; an
extendable palette preserves its prefix and extends deterministically without
cycling. CategoricalColorMap uses unique trimmed control-free raw keys, not aliases.
Unused entries and repeated colors are valid; missing used keys fail even with Hidden guides.

CategoricalColorScale selects a complete palette/map. Heatmap and Beeswarm share
ContinuousColorScale::sequential/diverging, optional matching gradient and exact
ColorDomain. Missing gradient uses Theme; missing domain uses prepared values.
Constant domains are valid; diverging domains contain their center inclusively.
Stops are strictly ordered with exact endpoints 0/1; diverging also needs a 0.5
neutral. Built-in sequential gradients use `#F7FBFF → #08306B`.

Exact element styles override only their own fields. PointLayer color, radius and
alpha remain independent; a separately set alpha replaces the complete color's
alpha byte in either setter order. Supplemental layers, outlines and text do not
inherit a scientific data-alpha instruction.

## Data alpha and explanations

`point_alpha`, `line_alpha` and `fill_alpha` attach complete AlphaEncoding values
on supported exact families. Original observation slots remain aligned, including
unavailable/filtered rows. SourceRows IDs are diagnostic, not indices. Supplied
Raincloud points follow original group/point order. `single` explicitly broadcasts
one value. Collective series/aggregates require complete unique raw keyed tuples;
Histogram bins use exact finite lower<upper intervals, optionally owning facet keys.
No mean/first/last reduction or repeated-row agreement is inferred.

AlphaValues::new owns arbitrary bytes without an explanation. Linear/Log10 declare
finite increasing domains; constant data and ascending/descending/constant byte
ranges are valid. Outside finite values clamp, invalid/null/nonfinite inputs fail
with source index; Log10 additionally needs positive distinguishable transformed
endpoints. Quantization is nearest/ties-to-even once: quarters over 0..255 give
[0,64,128,191,255]. Continuous functions evaluate distinct required source/examples
once in numeric order; categorical functions use first-observed keys. No callback
is retained. Threshold equality belongs to the right interval; select preserves a
complete pooled declaration for compatible Shared guides.

Zero alpha retains scientific membership, geometry and statistics. Data alpha
replaces, rather than multiplies, the resolved byte. Repeated setters replace the
complete instruction; `AlphaEncoding::theme()` resets it. Declared Point/Line/Fill
sections follow existing guide content. Continuous alpha uses at most five explicit
domain examples, not an inferred inverse ramp. Context keys neutralize data-bound
alpha to 255; undeclared bytes do not invent an explanation. Shared compatibility
requires complete recipes, not merely matching bytes; Hidden does not skip binding validation.

```rust
{{#include ../../../crates/lichtung-plot/examples/08_data_alpha.rs:example}}
```

LegendComposition::Combined keeps complete observed keys. Separate explains
group-style, joint, hue-color, literal and supplemental sections in order, then
continuous color and declared alpha, within one frame/placement owner. A dense
five-group/four-hue Interaction can explain 20 observed keys or 5+4 marginal keys;
sparse input need not make Separate smaller. Neither invents unobserved combinations.

Independent Interaction group-style/hue mappings can separate; same-field mappings
stay joint, and unsupported independent pairs fail before fonts. Shared uses Canvas
composition/chrome and source Plot mark geometry; PerPlot retains child choices.
Repeated identities must agree on labels/styles/complete keys, and explicit child
guide positions remain incompatible with Shared. Hidden skips explanatory semantics,
not scientific scales. Heatmap/Beeswarm and Matrix retain one native guide track.

```rust
{{#include ../../../crates/lichtung-plot/examples/06_legend_sections.rs:example}}
```

See [composition](composition.md), [text](text.md) and the
[appearance ownership specification](https://github.com/PythiaLaurus/lichtung-plot/blob/main/specs/04_VISUAL_PROPERTY_OWNERSHIP.md).
