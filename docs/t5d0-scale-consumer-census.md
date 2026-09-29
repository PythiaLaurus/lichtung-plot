# T5d0 — Prepared Color Domains And Migration Consumers

The scientific inputs and selected mark/annotation styles are closed by T5c.
T5d resolves **opaque data color** before those styles, glyphs, font census,
guides and geometry are measured. Public replacement values/setters stay T6.
`specs/03_THEME_SYSTEM.md` supplies the normative scale and exact-window rules.

| Family/consumer | Prepared raw identity/domain | Present mapping and replacement owner |
| --- | --- | --- |
| Scatter | `PreparedScatter` finite selected group points and first-observed hue; full parent hue before facet slices | `scatter.rs` uses `hue_color_index` and `% theme.colors.len()`; resolve keyed hue once, or unkeyed primary accent slot zero. Point/guide share the selected `PointStyle`. |
| Line, Trajectory, LinePath | `PreparedLine` group/condition and independent hue identities, complete path vertices, prepared point layers | `line.rs` repeats hue/index/modulo in both guide and mark projection; resolve keyed hue across paths and markers, neutral connector as a separate non-data style, and source-order point-layer accent slots. Exact `PointLayer::color` still consumes its accent slot. |
| Histogram, Density, Qq, Regression | Prepared Histogram bins, Density `curves[*].identity`, Qq paired quantiles, Regression observations/fit | Histogram/QQ/Regression `theme.colors[0]` are unkeyed primary accents; Density hue is keyed independently of group traversal, with group-only curves sharing primary accent. Preserve source bin/KDE/OLS computations and guide glyph equality. |
| PointInterval, Coefficient, Forest, Dumbbell | Prepared estimate rows/status, categorical term keys and typed Dumbbell endpoints | Estimate markers use accent zero, unavailable rows have no color value. Dumbbell endpoint colors are exact caller RGB (independent of direction), its connector/reference paint belongs to its mark role. |
| Bar, HorizontalBar, stacked siblings | `PreparedBars.categories`, positioned `series`, `hues`, occupied segments, supplemental-point styles | Ordinary ungrouped category fills currently cycle `theme.colors`; grouped/stacked and hue use `%` with index/series identity, and sign controls use two raw slots. T5d2 makes each *actually keyed* color stable under the parent domain and maps unkeyed/sign accents separately, retaining exact `BarPointLayer` colors. |
| Boxplot, Violin, Raincloud (sampled and supplied) | Prepared distribution `x_levels`, populated group/hue pairs and source-row strips; supplied `RaincloudGroup` identities | Body/outlier/strip and originating guide colors currently mix `theme.colors[index % len()]`, `hue_color_index`, and `categorical_color`. One resolved raw-key map feeds complete independent fill/outline styles; no second sampled/supplied color route. Missing series slots remain declared positions, not invented marks. |
| Interaction | `PreparedInteraction.conditions`, ordered trace `CollectivePair` identities, sparse cells and caller `InteractionStyle` | Existing `categorical_color("interaction", raw hue-or-group, trace.color_index)` already supports an exact map; resolve the same final keyed color for trace markers, connectors, intervals and exact legend glyphs. Shape/pattern overrides remain independent. |
| Marginal, Matrix | Prepared `groups[*]` collective identities and Matrix variable square/diagonals; Marginal linked KDEs | Complete group/hue domain precedes local selection. Both `group.color_index % colors.len()` routes and their point/density guides consume the same final keyed map; diagonal Matrix KDE is not an extra color identity. |
| Beeswarm | `PreparedBeeswarm.rows` with separate continuous color values, finite/exact color range, `values` and guide stops; category levels only position points | Replace `beeswarm.rs::continuous_color`, its fixed `(49,130,189)`→`(222,45,38)` RGB interpolation and local two-stop literals with one resolved sequential scale/guide. Beeswarm categories do not become qualitative color keys. |
| Heatmap | Prepared active cells (including missing and merged rows), `ColorDomain {minimum,maximum,center}` and final shared color domain | Replace `heatmap.rs::heatmap_color`, `interpolate`, `colorbar_gradient_stops` and their fixed encoded-RGB literals with one resolved sequential/diverging scale. Missing cells/rows and highlights retain their separate neutral/private mark appearances; complete contrast typography must be chosen from **final** mapped fills before fonts. |
| Waterfall | Prepared finite signed cumulative spans, base/terminal label sources | Positive/negative `theme.colors[0/1]` are distinct unkeyed accents, not category/hue keys. A zero step retains source identity; reference/connector strokes and total-label roles do not inherit a data color. |

## Current Overrides And Precedence

- `Plot::categorical_colors(CategoricalColors)` stores exact normalized raw
  strings and opaque RGB, but `Plot::categorical_color` is currently called
  only by a subset of consumers. Migrate all keyed families to one private
  `CategoricalColorMap`; reject a missing **used** key before fonts, admit
  unused/repeated-color keys, and never fall back to a palette for a missing
  exact key. Display aliases/humanization are not keys.
- `hue_color_index` and `% theme.colors.len()` are superseded together by a
  fixed-or-extendable private palette over a complete prepared domain.
  Built-in `ResolvedTheme.colors` supplies the stable seed prefix. The exact
  public map overrides *only* its used keyed identities. Category order,
  explicit order, hue and positioned series retain their own identities.
- `PointLayer::color`, `BarPointLayer` color, `DumbbellEndpoints`,
  `InteractionStyle` shape/pattern, and exact mark/annotation paints keep
  their current local precedence. Exact layer paint does not remove its
  declared accent position. RGBA alpha stays with the T5c mark/text role.
- The family-typed `Plot::color_limits` on Heatmap/Beeswarm,
  Heatmap `Plot::scale(HeatmapScale::Sequential/Diverging { center })`, and
  existing missing-cell/highlight options are the current continuous/domain
  controls. A Plot-local private scale may replace its theme gradient, but
  does not replace non-data appearance. Exact color limits saturate finite
  out-of-domain values to end colors; they are not geometric `x_limits`/
  `y_limits` and never make missing values a gradient endpoint.

## One Descriptor And Deletion Sequence

`decorations/legend/descriptor.rs::ContinuousContributor` is already the
font-independent contributor: typed source field, title, complete palette,
center, missing key, finite domain and exactness. `ContinuousDescriptor::merge`
performs one semantic compatibility check, automatic union, exact equality/
containment and final label generation. T5d4 replaces its palette with the
resolved stop/scale identity **in place**; it neither merges per-panel labels
nor adds a second Canvas scale winner. Discrete legends compare the retained
glyph and raw semantic title identity under the existing T5b guide owner.
`src/significance.rs` now receives the original typed target and selected
anchor; the same input receives final T5d domain identities, with no
string-target translation.

The final consumer deletion owners are `src/builder/accessors.rs::categorical_color`
and `src/builder/facets.rs::hue_color_index` (T5d2), direct `% theme.colors.len()`
and local group-color helpers in the family rows (T5d2),
`src/plots/heatmap.rs::{heatmap_color,interpolate,colorbar_gradient_stops}`
and `src/plots/beeswarm.rs::continuous_color` (T5d3), and the former
`ContinuousContributor` palette-only comparison plus any leftover parallel
domain/guide decisions (T5d4–T5d5). Only one mapping is retained for each
semantic owner; no compatibility alias survives.

## Pinned Closure Fixtures

1. Incrementally extend a parent qualitative domain, including a hue hidden
   in one facet: compare every earlier RGB byte/guide glyph and check the
   newly appended color. A sparse, trailing Canvas cell cannot reorder it.
2. Under a Canvas shared legend, include a visible mark whose local legend
   is hidden, mixed Plot themes, one exact caller map with two identities of
   the same color, and a configured point layer whose exact color consumes a
   slot. Only a genuinely incompatible shared **glyph** rejects.
3. Exact-map keys use raw normalized names even when distinct aliases produce
   similar display strings; an unused key is allowed, a missing used key fails
   before fonts, and a fixed palette refuses capacity without index cycling.
4. Check Heatmap/Beeswarm marks, selected stops, continuous guide labels and
   decoded output pixels at endpoints, interior fractions, divergent center,
   constant/one-sided domains and a finite value beyond exact color limits.
   A missing/merged row remains neutral; a changed final fill reselects its
   contrasting font before the census.
5. Preserve typed significance targets, exact `x_limits`/`y_limits`, current
   source-row diagnostics, prepared primary-mark counts and Canvas output
   replay across every color-domain change.
