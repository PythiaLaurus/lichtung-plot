# Discrete legend composition

`LegendComposition` is a crate-root closed policy with `Combined` (default) and
`Separate`. Attach it through consuming `Plot::legend_composition` on all 25 exact
families, or `Canvas::legend_composition` for Shared.
Each setter replaces only that selection, independently of theme, labels,
visibility, placement and sizing call order. All families can now explain declared
data alpha; see `public-data-alpha.md` for the 49 actual component attachments.

## Meaning and ownership

Combined retains one section, observed pair identities and complete mark keys.
Separate explains actual mapped channels in semantic section order: group style,
joint, hue color, literal/reference domains, then supplemental point series.
Sections share one outer frame, placement, collision obstacle and frozen draw
plan. There is no per-section position or new placement budget.

An Interaction with explicitly configured group styles and an independent hue
field can produce group-style and hue-color sections. A five-group/four-hue full
crossing has twenty Combined keys versus five plus four Separate keys. Sparse
three-pair support can have three Combined versus four marginal keys. Marks,
source rows and intervals are unchanged; marginal sections do not claim a
Cartesian set of observed combinations. Repeated configured styles/colors are
permitted without promising perceptual uniqueness.

Same raw-field group/hue channels retain one complete joint section. Families
without an independent mapped group-style channel refuse unsupported independent
pair factorization before fonts, including a single observed pair or equal RGB.
Hue-only and positioned categorical guides retain complete truthful color keys.
Row-hue paths explain point colors without inventing hue-colored connectors.
Literal Dumbbell/Regression/sign keys remain literal; labeled Line PointLayers
occupy the final `series` section. Raw identities remain distinct from aliases.

Group-style projection neutralizes only hue-mapped RGB to the originating Plot's
legend-label text RGB, preserving independent role/exact alpha, presence, fixed/exact paints,
outlines and geometry. The text role's alpha is not applied to the key. Hue
projection neutralizes only mapped Interaction group shape/pattern to Circle/Solid.
Shared Canvas text/chrome uses the Canvas theme; complete keys retain their source
Plot paints and neutral RGB.

Both compositions neutralize data-bound target alpha to 255 in existing context
keys. Declared alpha explanations follow symbols and optional continuous-color
content inside the same frame; Undeclared bytes do not invent an explanation.

| Owner | Effective selection |
| --- | --- |
| Standalone Plot or generic facets | Owning Plot |
| Canvas PerPlot | Each Plot independently |
| Canvas Shared | Canvas selection for every contribution |
| Canvas Hidden or Plot Hidden contributor | No discrete guide semantics/fonts |

Shared ignores stored child selections without mutating them or treating them as
conflicts. Returning to PerPlot uses those selections again. Hidden marks still
participate in scientific color-domain coordination. Canvas composition is
inactive under PerPlot/Hidden and for all-continuous guides. Explicit child guide
edges remain incompatible with Shared. Matrix remains native Plot-local and
excluded from Canvas/generic facets; Marginal remains excluded from generic
facets. PointLayer attachment remains Line-only.

## Public construction

```rust
use lichtung_plot::{Canvas, CanvasLegendPolicy, LegendComposition};

let canvas = Canvas::grid(1, 2)?
    .legend_composition(LegendComposition::Separate)
    .legend_policy(CanvasLegendPolicy::Shared);
# let _ = canvas;
# Ok::<(), lichtung_plot::Error>(())
```

All 25 exact families expose the setter, including Histogram, Qq, PointInterval,
Coefficient, Forest, Heatmap, Beeswarm and Waterfall after declared-alpha admission.
Erased Any has no setter. Continuous content and symbols share one PreparedGuide.
The enum moved from private semantic preparation into the existing public policy
owner; the former private enum implementation/path is removed.

`examples/06_legend_sections.rs` publishes side-by-side Combined/Separate PerPlot
guides and a Separate Shared guide, with the same twenty observed pairs and forty
source observations per panel. Repeated shapes/patterns are deliberately explicit
existing styles.

## Lifecycle

Effective policy, scientific domains, actual producer bindings, typed sections,
labels and complete source occurrences are prepared before the final font
census. Titles and entry text use stable section descendants of the original
guide slot. Natural size, assigned section flow, exact owner-local reuse,
consumed-ancestor dependency keys and selected output use the same descriptor.
No first-section-only rendering, flat compatibility carrier, late body-probe
fallback, generic palette reconstruction or draw-time source access remains.

Measurement stacks ordered sections with one chrome reserve, bounded global
normal/compact variants and deterministic finite-height capacities. Complete
text/glyph ink participates in fit. Frozen SVG/PNG reuse the selected
`LegendDrawPlan` with no new fonts, shaping or layout. See
`../specs/08_DISCRETE_GUIDES.md` and the preceding semantic, stacked measurement,
exact-reuse, effective-owner and routed-dependency evidence documents.

## Historical Composition Cutover Verification

The existing all-family constructor/capability inventory now attaches Separate to
all seventeen admitted markers while preserving exact kind/erasure checks. Nine
new compile-fail documentation examples reject all eight other concrete families
and erased Any through the actual public setter, not an obsolete import. Public
Plot and Canvas construction examples compile; **all 114 doctests pass**.

The existing Shared authority test now uses public setters, checks replacement and
both Canvas policy/selection orders, retains actual omitted Combined behavior,
and returns to mixed PerPlot selections without mutating child values. The
continuous numeric-guide test stores Separate under Shared and Hidden and retains
the same final numeric sources and pre-font policy refusals. Five facet-parent
tests, two actual producer-binding tests, five measured section/cache tests and
six child-cache tests also pass: **20 focused library tests** in total.

**18 existing external tests** pass: the all-family capability inventory, sixteen
Shared composition callers and actual styled Interaction key/mark parity. No unit
or integration tests are added or moved; library discovery remains **671,
including 17 ignored**. Strict all-target Clippy, formatting/whitespace checking,
warning-free API docs and unpacked package verification pass. The example runs
successfully and publishes both explicit **1600×1000** scenes; selected SVG text
has all twenty pair labels, the five group/four hue marginal labels, and their
owning titles in semantic order. Logs are `/tmp/opencode/public-guides-*.log`.

The full all-owner Separate acceptance/release matrix and integrated phase closure
remain pending. This public cutover does not claim a full-suite or release gate.

The eight former concrete-family negatives above are historical evidence. The
alpha public cutover migrates them to the all-25-family positive inventory and
retains Any as negative; `public-data-alpha.md` owns the current boundary.
