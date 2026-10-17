# Complete Theme, Font And Scale Foundations

## Complete Construction And Ownership

Crate-root `Theme` is the single complete immutable appearance value. Its eight
components are `SurfaceTheme`, `TypographyTheme`, `AxisTheme`, `LegendTheme`,
`CompositionTheme`, `MarkTheme`, `AnnotationTheme` and `ColorScales`. Their nested
role groups, compound paints and exact `RelativeSpacing` inputs are public,
privately stored, validated values. Backend styles, local metric resolution,
font discovery, prepared sources and resolved operation state remain private.

`Theme::new` consumes all eight components. The four built-ins are ordinary
`publication()`, `nature()`, `science()` and `chi()` constructors. There is no enum
identity, selector dispatch, sparse appearance or second private ThemeValue.
`customize()` starts a complete cloned working copy; `ThemeBuilder::new` consumes
one. Builder getters borrow working components, `with_*` replaces a complete
component, and `build()` returns one validated Theme without discovering fonts.

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

Plot stores an optional explicit complete Theme and Canvas stores its complete
root Theme. Standalone omission constructs Publication; composed omission
inherits the Canvas value. An explicit Plot theme replaces it completely. Facets
add no scope. Canvas surfaces, headers, TextPanel and Shared-guide chrome remain
root-owned; Plot marks, axes, annotations and prose remain Plot-owned. The shared
gap above is exactly 2 CSS pixels with Publication's 11px legend-label role. Only
LegendTheme owns that relationship, in both natural and assigned guide layout.
Custom spacing is valid with PerPlot or Hidden policies too.

`TitleAlignment` and consumed CompositionTheme alignment defaults are public.
The four narrow Plot/Canvas heading-instruction setters belong to the following
cutover; this change does not claim their public availability.

## Typed Fonts And Exact Local Relationships

Every one of the eighteen text roles independently owns stack, RGBA paint,
positive CSS-pixel size, closed weight and finite positive leading. Obtain a
built-in font profile from a role, for example
`Theme::publication().typography().root().title().stack().clone()`.

`FontFamily::named` retains one nonempty, trimmed, control-free literal identity;
commas, CSS keywords, interior whitespace and Unicode bytes are not parsed into
other families. `GenericFontFamily` explicitly supplies Serif, SansSerif and
Monospace. `FontStack::new` retains a nonempty ordered sequence, including
duplicates. Font matching, coverage and discovery occur only in the consuming
operation; no network or asset provisioning is added.

`ThemeBuilder::with_font_stack` replaces stacks in all eighteen roles once during
construction. Later single-role or component replacement has normal latest-edit
semantics; no runtime font precedence layer survives.

RelativeSpacing exposes only exact construction and inspection. `new(f64)`
retains that binary64 value; `ratio(u32, u32)` retains the reduced mathematical
ratio. An approximate `as_f64` getter is not the spatial or identity operand.
Hard requirements still aggregate exactly before one upward boundary; final
positions retain their own prescribed rounding.

## Whole Typed Color Selections

Data mappings use `OpaqueColor`, with exact RGB or strict six-digit hex
construction. There is no conversion that silently drops Color alpha. Complete
mark roles install alpha independently; accent seeds are separate from
qualitative mappings.

- `QualitativePalette::fixed` requires sufficient supply for the active prepared
  raw domain. `extendable` uses deterministic first-party extension without
  cycling; repeated supplied colors are valid.
- `CategoricalColorMap::new` consumes ordered `(String, OpaqueColor)` entries.
  Keys are exact raw identities, already trimmed, nonempty, unique and
  control-free. Interior whitespace and Unicode bytes remain distinct; aliases
  are not keys. Unused entries are valid; missing used identities refuse.
- `CategoricalColorScale::{from_palette,from_map}` selects one complete source;
  `Plot::categorical_color_scale` replaces it on the same fifteen sealed
  `kind::CategoricalScale` families. Accent-only families and Any do not gain it.
- `ContinuousColorScale::{sequential,diverging}` owns kind, optional complete
  gradient, optional exact ColorDomain and the diverging center. Heatmap and
  Beeswarm share the same attachment. None means owner gradient or automatic
  prepared domain. Constant exact domains are valid, including Beeswarm; a
  diverging exact domain contains its finite center inclusively.
- Sequential and diverging gradients retain every strictly ordered opaque stop
  with exact 0/1 endpoints. Diverging also requires the unique 0.5 neutral.
  Unknown-length inputs use checked growth, with no arbitrary stop-count cap.
  Wrong-kind gradient/center edits reject rather than switching kind implicitly.

All four built-ins retain the common `#F7FBFF -> #08306B` sequential default.
Actual marks, contrast, prepared fonts, local guides and Shared guides consume
the same final-domain mapping and semantic-stop remapping. Facets retain their
owning Plot's domain and scoped source identities.

Capacity is checked before fonts independently of legend visibility. Standalone
and faceted roots count their real prepared identities; Shared Canvas validates
the final coordinated hue domain even when every child hides its legend. Exact
map coverage also checks coordinated keys. Sign-colored bars use their accent
owner, and unkeyed marks do not consume categorical supply.

## Retired Routes And Migrated Callers

The cutover deletes the closed Theme selector and runtime match, ThemeValue and
conversion helpers, both default-font constants, CategoricalColors and its old
capability/setter, HeatmapScale and Heatmap's `.scale`, both `.color_limits`
routes, separate family gradient/domain fields and delayed error-string storage.
Canvas's shared-gap field, setter and policy compatibility check are removed.
Forest's scientific `scale(AxisScale)` remains its existing axis control.

Source, unit/integration fixtures, examples, benchmarks, current guides and
doctests migrate together. Non-Copy Theme callers clone complete values when
needed; example filenames and independent expected palettes use explicit names,
not enum matching or complete Debug strings. Compile-fail boundaries use valid
current receiver/argument types so removed routes and wrong-family attachments
cannot pass merely because an unrelated old import is missing.

## Focused Acceptance Evidence

Two behavioral regressions replace two axis field/edit inventories in the same
change:

- `fixed_plot_palette_capacity_is_checked_before_fonts_even_with_hidden_legend`
  proves insufficient Plot supply returns a Data error without font discovery,
  despite an intentionally unavailable role font and a hidden legend.
- `hidden_shared_plots_validate_the_final_palette_domain_before_fonts` proves a
  one-color Theme supply cannot cover two distinct raw identities split across
  hidden Shared children; neither local one-key domain nor hidden glyphs mask the
  final semantic requirement.

The retired inventories were
`frame_sides_validate_all_fifteen_present_choices_and_every_consuming_edit` and
`ordinary_and_cell_frames_are_independent_complete_roles_with_cross_sibling_validation`.
Actual independent frame consumption, frozen side drawing and inactive-edge
reserve tests remain and pass. No unit cases move to integration suites.
Library discovery remains **671**, including the same **17 ignored** tests.

Validation passed:

- **47 distinct current focused library tests**: complete component/paint/spacing
  contracts; immutable construction and eighteen-role font precedence; literal
  stack/census/canonical selection; both capacity regressions; arbitrary diverging
  Beeswarm Shared mapping; independent real frame consumption; and the existing
  all-25-family lifecycle contract. The initial theme-module run also included one
  subsequently retired inventory, which is excluded from this count.
- **86 existing external contracts**: 43 Heatmap; six theme ownership; fifteen
  Shared composition and two additional hidden-guide contracts; six exact-map/
  Interaction Shared contracts; one Raincloud mapping; one four-preset Matrix
  source-completeness contract; twelve public-family/output/font contracts.
- **84 doctests**, including complete Theme/font/map/gradient examples and the
  removed selector/font/scale/gap plus opaque-color/capability boundaries.
- Strict all-target Clippy, formatting, whitespace checks, warning-denied API
  documentation and locked/offline package verification. Package compilation
  succeeds from Cargo's unpacked staging directory.

Execution logs are `/tmp/opencode/complete-theme-*.log`. Discovery-only commands
confirm the library ceiling; no complete library/integration suite or scene
golden repinning was used. The next foundation leaves expose narrow heading
instructions and close public custom-owner/contrast/scale precedence evidence.
