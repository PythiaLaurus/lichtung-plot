# Complete Themes, Fonts And Scales

Theme is one complete immutable appearance value with eight components:
SurfaceTheme, TypographyTheme, AxisTheme, LegendTheme, CompositionTheme, MarkTheme,
AnnotationTheme and ColorScales. Fields are private, with readonly getters and
validated consuming `with_*` edits.

`Theme::new` consumes all eight components. The built-ins are ordinary
`publication()`, `nature()`, `science()` and `chi()` constructors. `customize()`
clones a complete working copy; ThemeBuilder::new consumes one. Builder getters
borrow working components, component edits replace complete values and `build()`
validates without discovering fonts.

```rust
use lichtung_plot::{Color, RelativeSpacing, Theme};

let base = Theme::publication();
let surfaces = base.surfaces().clone()
    .with_canvas_root(Color::hex("#FAFAFA")?)?;
let legend = base.legend().clone()
    .with_shared_gap(RelativeSpacing::ratio(2, 11)?)?;
let custom = base.customize()
    .with_surfaces(surfaces)
    .with_legend(legend)
    .build()?;
# let _ = custom;
# Ok::<(), lichtung_plot::Error>(())
```

Standalone omission selects Publication; an unthemed Canvas child inherits its
Canvas root. Explicit Plot Theme replaces the inherited value completely; facets
add no scope. Canvas surfaces/headers/TextPanel/Shared-guide chrome use Canvas,
while Plot marks/axes/prose use Plot. The example Shared gap is exactly 2 CSS
pixels with Publication's 11px legend-label role. LegendTheme alone owns it and
the value remains valid under PerPlot/Hidden policies.

Plot title/subtitle alignment instructions override Canvas panel defaults, then
Theme defaults. Captions use independent Theme alignment. Panel instructions do
not affect Canvas root prose, headers or TextPanel.

## Fonts And Local Spacing

All eighteen text roles independently own stack, RGBA color, positive CSS-pixel
size, closed weight and positive leading. Obtain a built-in stack through, for
example, `Theme::publication().typography().root().title().stack().clone()`.
FontFamily::named is one nonempty trimmed control-free literal identity, not CSS;
GenericFontFamily explicitly selects Serif/SansSerif/Monospace. FontStack retains
nonempty ordered entries including duplicates. Availability is checked only when
an operation consumes the role. No downloads or provisioning occur.

ThemeBuilder::with_font_stack replaces all role stacks once during construction;
later role/component replacements have ordinary latest-edit precedence.
RelativeSpacing::new(f64) retains that exact binary rational; ratio(u32,u32)
retains the reduced mathematical ratio. Local role metrics resolve spacing, never
root dimensions. Approximate as_f64 inspection does not drive cosmetic geometry.

## Color Mappings

Mapped colors use OpaqueColor; complete role/exact paints use RGBA Color with
independent alpha. QualitativePalette::fixed requires enough colors for the raw
domain; extendable preserves its exact prefix and deterministically extends,
without cycling. CategoricalColorMap consumes nonempty unique trimmed control-free
raw keys; aliases are not keys. Unused entries and repeated colors are valid,
missing used keys reject before fonts even with Hidden guides.

CategoricalColorScale::from_palette/from_map selects a complete Plot mapping on
supported families. Accent colors for unkeyed/supplemental marks are separate.
Heatmap and Beeswarm share ContinuousColorScale::sequential/diverging, containing
optional matching gradient, optional exact ColorDomain and diverging center.
Absent gradient uses Theme, absent domain uses prepared values. Constants are
valid; exact diverging domain contains center inclusively. Wrong-kind edits reject.

Gradients retain strictly ordered opaque stops with exact 0/1 endpoints;
diverging also requires one 0.5 neutral. All built-ins use sequential
`#F7FBFF -> #08306B`. Actual marks, contrast and guides consume the same final
mapping, including Shared-domain installation before fonts.

See [Theme specification](../specs/03_THEME_SYSTEM.md) for component/local-metric
requirements and [property ownership](../specs/04_VISUAL_PROPERTY_OWNERSHIP.md)
for exact paint and data-alpha precedence.
