# Explicit Data Alpha

AlphaValues, AlphaEncoding and closed AlphaSemantics are crate-root immutable
inputs. Plot::point_alpha, line_alpha and fill_alpha select independent complete
instructions on supported families: point fill, scientific stroke or body/cell
fill. Every exact family has Legend controls; Any has neither alpha nor Legend
configuration. Parent admission remains unchanged.

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

## Explicit Units

Observations align with all original slots, including filtered/unavailable rows;
SourceRows IDs are diagnostic, not indices. Supplied Raincloud points follow
original group/point order. Single requires exactly one value and explicitly
broadcasts it. Collective geometry instead takes keyed actual series/aggregates:

```rust
use lichtung_plot::{AlphaEncoding, AlphaValues, CategoryValues};

let strokes = AlphaEncoding::keyed(
    [CategoryValues::new("series", ["control", "treated"])?],
    AlphaValues::new("reliability", [128, 230])?,
)?;
# let _ = strokes;
# Ok::<(), lichtung_plot::Error>(())
```

Keys use required actual group/hue or category/positioned-series fields, optionally
owning facets. Complete nonmissing raw tuples are unique and lengths align;
canonical field order preserves associations. Unused pooled tuples create no
marks/fonts. Every actual unit resolves before fonts including 0; absent optional
components still validate kind/schema. There is no mean/first/last alpha or
repeated-row agreement in place of explicit collective input.

AlphaEncoding::bins uses exact finite lower<upper Histogram intervals and optional
facet tuples, aligned unique keys and normalized zero. Alpha does not move bin
edges or reduce sample opacity into bins. Single can broadcast without a bin map.
The complete 25-family/49-component binding table is in
[property ownership](../specs/04_VISUAL_PROPERTY_OWNERSHIP.md).

## Mappings

- new(name,bytes) eagerly owns arbitrary u8 values, Undeclared, without explanation.
- linear/log10 require explicit finite strictly increasing domain; constant source
  and ascending/descending/constant byte ranges are valid. Log needs positive
  source/domain and distinguishable transformed endpoints.
- Finite outside values clamp; null/nonfinite or nonpositive Log values reject
  with source index. `a0+t*(a1-a0)` quantizes once nearest/ties-to-even; quarters
  from 0..1 to 0..255 give `[0,64,128,191,255]`.
- continuous(source,domain,f) explicitly declares continuity and evaluates each
  distinct clamped source/required example once, in numeric order. categorical
  evaluates raw categories once in first-observed order. No callback is retained.
- threshold has increasing finite boundaries and one more byte, equality on the
  right. Empty boundaries/one byte means all values.
- select permits repeated valid indices and preserves the entire declaration.
  Construct a pooled map once, then select per Plot for matching Shared recipes.

Names are nonempty already-trimmed control-free identities. Empty value arrays
are constructible; attachment checks actual alignment/completeness. Inspect name,
len, is_empty, get and semantics without exposing tables or renderer types.

## Paint And Explanations

Normal RGB/role/exact paint resolves first, then the target replaces alpha, never
multiplies it. Outlines, text, unlisted references/chrome and independent layers
keep their paints. Zero retains scientific rows/domains/statistics/order and
coincident marks. Repeated setters replace the complete target; theme() resets it
independently of Theme/scale setter order.

Declared alpha sections follow symbols and continuous color in Point/Line/Fill
order under either composition. Continuous retains up to 5 original-domain numeric
examples even with equal/zero bytes: symbols, not an inferred ramp/inverse.
Category/threshold explains actual consumed members in declaration order. Geometry
comes from actual component roles. Target RGB uses source Plot legend-label RGB
without its text alpha; fixed geometry/independent outlines stay actual, mapped
group shape/pattern neutralizes to Circle/Solid. Existing context keys neutralize
data-bound alpha to 255, including Undeclared inputs.

Shared uses Canvas chrome/composition without remapping alpha; full recipes and
projected glyphs/titles must agree while actual members may differ. Equal bytes/
examples alone are insufficient; conflicting aliases reject before fonts. Hidden
omits explanatory text/fonts, not binding validation or complete input identity.
Native Heatmap/Beeswarm right tracks and Matrix trailing track place one guide,
suppressed by Hidden. Color+alpha shares one frame and frozen plan.

See [`08_data_alpha.rs`](../crates/lichtung-plot/examples/08_data_alpha.rs) and the
[guide specification](../specs/08_GUIDE_SYSTEM.md).
