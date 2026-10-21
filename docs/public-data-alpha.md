# Explicit Data Alpha

`AlphaValues`, `AlphaEncoding` and `AlphaSemantics` are opaque/closed crate-root
inputs. Consuming `Plot::point_alpha`, `line_alpha` and `fill_alpha` select three
independent complete instructions. The sealed sets contain **17 point, 17 stroke
and 15 fill attachments**. Every exact family has Legend controls; erased Any has
neither alpha nor Legend configuration. Native parent admission stays unchanged.
Heatmap/Beeswarm retain their native right guide track and Matrix its native
trailing grid track; Hidden suppresses their guides. Standard panels consume the
selected inside/outside placement. Each guide has one placement owner.

## Observation And Collective Units

```rust
use lichtung_plot::{AlphaEncoding, AlphaValues, NumericValues, Plot};

let plot = Plot::scatter(
    NumericValues::new("time", [0.0, 1.0, 2.0]),
    NumericValues::new("response", [1.0, 0.5, 1.5]),
).point_alpha(AlphaEncoding::observations(AlphaValues::linear(
    NumericValues::new("confidence", [0.0, 0.5, 1.0]),
    (0.0, 1.0), (0, 255),
)?));
# let _ = plot;
# Ok::<(), lichtung_plot::Error>(())
```

Observations align with all original slots, including scientifically filtered or
unavailable rows. SourceRows IDs are diagnostic identities, not alpha indices.
Supplied Raincloud observations follow original group/point order. Single requires
exactly one value and explicitly broadcasts to actual units. Collective geometry
instead takes explicitly keyed values:

```rust
use lichtung_plot::{AlphaEncoding, AlphaValues, CategoryValues};

let strokes = AlphaEncoding::keyed(
    [CategoryValues::new("series", ["control", "treated"])?],
    AlphaValues::new("reliability", [128, 230])?,
)?;
# let _ = strokes;
# Ok::<(), lichtung_plot::Error>(())
```

Required keys are actual group/hue series fields or category/positioned-series
aggregate fields. Additional selectors may be owning facets. Complete raw tuples
must be unique and nonmissing; field order is canonicalized without changing alpha
association. Unused pooled tuples create no marks/text/font requirements. Every
actual unit resolves before fonts, including zero; absent optional components
still validate binding kind/schema. There is no mean, first/last value or repeated
row agreement instead of an explicit collective input.

`AlphaEncoding::bins(bounds, facets, values)` binds exact finite lower < upper
Histogram intervals and optional facet tuples. Lengths agree, complete keys are
unique, and signed zero has one identity. Prepared edges match exactly; alpha does
not adjust bin selection or reduce samples into bin opacity. Single can explicitly
broadcast when a per-bin map is unnecessary.

## Mappings And Quantization

- `AlphaValues::new(name, bytes)` eagerly owns arbitrary u8 values and is
  **Undeclared**. Name/cardinality/equal bytes never invent guide semantics.
- `linear(source, domain, range)` and `log10(source, domain, range)` declare an
  explicit finite strictly increasing domain. Constant sources and ascending,
  descending or constant byte ranges are valid. Log requires positive sources
  and endpoints whose binary64 log10 coordinates remain distinguishable.
- Finite out-of-domain values clamp; missing/nonfinite mapped sources reject
  immediately with index. Log nonpositive values reject rather than clamping.
  Calculate `a0 + t * (a1-a0)` and quantize once nearest/ties-to-even: quarters on
  0..1 to 0..255 give **0, 64, 128, 191, 255**.
- `continuous(source, domain, f)` explicitly declares a continuous mapping. Invoke
  `f` once per distinct clamped source/required representative coordinate in numeric
  order during construction. Retain its complete evaluated table, never a function.
- `categorical(source, f)` invokes `f` once per first-observed raw category and
  retains that complete declared order/table. Null sources reject.
- `threshold(source, boundaries, alphas)` requires increasing finite boundaries
  and one more byte: `< b0`, `[b0,b1)`, ..., `>= blast`, equality on the right.
  Empty boundaries and one byte declare `all values`.
- `select(indices)` permits repeated valid indices and preserves the complete
  declaration. Construct a pooled custom/category map once, then select per Plot
  to preserve common Shared recipes.

Names are nonempty, already trimmed and control-free. Empty construction values
are valid; actual attachment checks alignment/completeness before fonts. Inspect
`name`, `len`, `is_empty`, `get` and `semantics` without exposing tables, callbacks
or renderer types. Complete values compare by value, independently of allocation.

## Paint And Automatic Guides

Normal data RGB, role and exact paints resolve first. The selected target then
**replaces** alpha, never multiplies it. Outlines, text, unlisted references/chrome
and PointLayer/BarPointLayer keep their own paints. Zero retains rows, domains,
statistics, ordering and coincident marks. Intrinsic ink, geometry, selected paint
and frozen output use the same styles. Repeated setters replace the complete
target; `AlphaEncoding::theme()` resets it independently of Theme/scale call order.

Declared sections follow existing symbols and optional continuous-color content
in Point/Line/Fill order, under both Combined and Separate. Continuous maps retain
up to five original-domain numeric examples, even with equal/zero output bytes;
these are symbols, not an inferred inverse or smooth ramp. Categorical/threshold
maps explain actual consumed members in declaration order, including zero. Labels
preserve the independently disambiguated complete numeric set.

Identity includes raw source field, semantics, target and actual unit; titles
qualify the resolved source with unit/target. All actual role glyphs/members remain
retained. Deterministic role order selects actual component geometry for the
one-key examples. Target RGB is source Plot legend-label RGB without text alpha.
Fixed geometry/independent outlines stay actual; mapped group shape/pattern becomes
Circle/Solid. Existing group/hue/literal context keys neutralize data-bound target
alpha to 255, including Undeclared inputs.

Plot/facets own their mappings; Canvas PerPlot uses each child; Shared unions visible
prepared contributions under Canvas chrome/composition without remapping alpha or
source-neutral RGB. Shared requires complete equal recipes and compatible titles/
projected glyphs while permitting different actual members. Equal bytes/examples
alone do not prove recipe compatibility. Conflicting aliases reject before fonts.
Color-plus-alpha uses one frame/cache/obstacle/frozen plan. Categorical-RGB versus
continuous-color Shared incompatibility remains. Hidden omits guide-only text/fonts
while actual scientific validation and complete child/height/feedback identity remain.

## Complete Component Census

Every admitted target accepts Theme and Single. R = original records, K = explicitly
keyed actual series/aggregates, B = exact bins. Optional components keep their actual
presence rules. Em dashes have no setter capability.

| Family | Point | Line | Fill |
| --- | --- | --- | --- |
| Scatter | R | — | — |
| Histogram | — | Single optional KDE | B |
| Density | — | K curve | K area |
| Qq | R | Single optional reference | — |
| PointInterval / Coefficient / Forest | R available | R present uncertainty | — |
| Line | R visible markers | K path | K complete band |
| Trajectory / LinePath | R visible points | K path | — |
| Interaction | R markers | K trace intervals/connector | — |
| Dumbbell | R paired endpoints | R connector | — |
| Heatmap | — | — | R finite eligible cells |
| Bar / HorizontalBar | — | K present uncertainty | K body |
| StackedBar / StackedHorizontalBar | — | — | K segment |
| Boxplot | R actual fliers/raw | K summary | K box |
| Violin | R configured raw | K inner summary | K cloud |
| Raincloud sampled/supplied | R mandatory strip | K median | K cloud |
| Regression | R | Single fit | Single configured band |
| Marginal | R central | — | K linked densities |
| Matrix | R off-diagonal | — | K diagonal densities |
| Beeswarm | R packed points | — | — |
| Waterfall | — | — | R input delta bodies |

`examples/08_data_alpha.rs` publishes observation/series mappings through preserved
SVG, outlined SVG and PNG. Independent numerical witnesses and source/parent proofs
are in `owned-alpha-values.md`, `declared-alpha-sections.md` and
`alpha-parent-identity.md`. Spec97 owns the separate integrated output/release gates.

## Public Cutover Evidence

The existing external constructor inventory invokes all **49** admitted setters
and Legend/composition controls for all 25 exact markers, including the eight newly
admitted Legend families. The existing erasure/output contract retains actual
observation alpha. **All 146 doctests pass**, including independent byte examples,
opaque representation/module boundaries, all 26 unsupported family/target pairs,
three Any/alpha negatives and the retained Any/composition negative. The eight
former concrete Legend negatives are removed. Private staging dead-code
expectations are removed; the common internal assignment helper serves the three
sealed public setters.

The new example ran and published preserved SVG, outlined SVG and PNG from one
1600x800 output. Strict all-target Clippy, formatting, warning-denied API docs and
offline unpacked package verification pass. Current specs01/02/03/05/08, README
and guide documentation describe the same public contract.

Native publication review found the transported Heatmap/Beeswarm descriptor could
also be placed by the outer standard-panel path. The common path now leaves native
placement with the native track. The existing finite-cell test first reproduced
two gradients and now proves exactly one; the all-25-family lifecycle checks one
native gradient for both families with all alpha targets. Both scoped native-facet
natural-size tests and the public mixed-family Shared gradient contract pass.
No scene-length/hash golden is repinned and no test is added. Library discovery
remains **671, including 17 ignored**.
