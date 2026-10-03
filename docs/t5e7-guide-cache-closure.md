# T5e7/8 Guide And Child-Cache Audit Fixes

Final integrated verification and phase status are in `docs/t5e-closure.md`.
The partition-local wording below preserves the audit and correction history.

This evidence covers the guide/cache audit partition. The phase owner retains
the T5e6 composition-sizing and broad T5e closure gates.

`ResolvedTheme` equality includes complete legend appearance. Child keys admit
the palette, legacy stack and all fourteen typography stacks before copying the
theme. Focused tests exercise real hits, legend-only paint/geometry invalidation,
and independent exact byte/record boundaries with distinct stack lengths.

Heatmap and Beeswarm retain the common `LegendDrawPlan` directly. The complete
selected frame, gradient, title, labels and missing block reach both capture and
unclipped geometry. Heatmap supplies its owner-local missing fill and full key
outline before measurement; shared compatibility includes outline width. A
shared guide uses its containing owner's frame appearance while preserving its
contributor-owned missing paint/outline.

Continuous measurement reserves broad frame/missing strokes and frame radius,
rounds required extents outward, and validates the complete unclipped painted
union inside its allocation. Frame paint/stroke/radius and missing outline
operands participate in feedback. Tests cover both orientations, custom 30px
frame/40px missing strokes, a one-quantum refusal at compact minimum, and actual
40pt bold title ink overhang.

Containing sizing owners can obtain the same effective content chrome and frame
minimum from `MeasuredContinuousGuide::allocation_metrics`. Their width proposal
must use the returned padding/key gap and be at least the returned frame extent;
the T5e6 parent integration owns that caller migration.

## Intentional Rendering Changes And Historical Attribution

Local colorbars now emit their already-selected rounded frame and use the
common call order (frame, gradient, title, values, missing block). The inspected
Beeswarm clip snapshot changes only title/value emission order; its coordinates
and mark clip are preserved. Full SVG fingerprints also include the newly
retained frame and outward-rounded guide extents.

The following are the pre-fix **T5e4 tangent-packing** goldens from
`tests/categorical/beeswarm.rs`, recorded here as historical evidence, not current
expectations. Case order is standalone, exact limits, constant, extreme domain,
four facet-scale cases, shared-x Canvas, and mixed Heatmap Canvas. Earlier bin
fingerprints remain attributed in `docs/t5e4-tangent-packing.md` and
`docs/local-search-benchmarks.md`.

| Case | Publication bytes / FNV-1a | Science bytes / FNV-1a |
| ---: | ---: | ---: |
| 0 | 10842 / 1054007616624498645 | 10443 / 15370886491911308377 |
| 1 | 10168 / 12505122077286044374 | 9769 / 7362132271872610060 |
| 2 | 10540 / 11709302911898653818 | 10141 / 15015258662383779716 |
| 3 | 10809 / 6269937400834678507 | 10410 / 14749205861666310563 |
| 4 | 20309 / 5824250669689777772 | 19896 / 9084235565820573999 |
| 5 | 20321 / 16942717934935890684 | 19903 / 13364500898271719653 |
| 6 | 20309 / 5824250669689777772 | 19896 / 9084235565820573999 |
| 7 | 20321 / 16942717934935890684 | 19903 / 13364500898271719653 |
| 8 | 23093 / 9006924917855375421 | 21502 / 10442436034119911723 |
| 9 | 16146 / 1002323509216004973 | 15064 / 17433636076107263233 |

## Final Guide Re-Audit

Discrete entry requirements now measure the actual origin-relative union of
point, line and rectangular-fill ink, including Circle/TriangleUp outlines,
polygon miters and broad strokes. Both natural and assigned flows consume the
same key width and asymmetric ascent/descent. The frame reserves its complete
stroke and corner diameter; selected unclipped calls must fit the assigned box.
The local/shared fixture obtains real Scatter guide entries and also exercises
a composite point/line/fill key in both flows, comparing selection geometry to
actual drawing. The original guide call-admission guards remain in place.
The small-metrics fixture also accepts 2pt title/label roles and a 1px key
without installing a new font, key, or guide-size floor.

The four inspected native Bar/HorizontalBar/stacked-family snapshots now price
the 1px key outline: guide width grows by 1px plus outward quantum rounding,
and the two-row guide height grows by 1px. The normal-flow body is reprojected
into that correspondingly narrower allocation. Labels retain their measured
fractional baselines rather than whole-pixel snapping. Scientific values,
uncertainty endpoints, signs, colors, annotations and caption content remain
present. Those fixtures also verify the actual PNG dimensions. Their historical
pre-fix SVG snapshots remain attributed to commit
`82455c5ebdc1d0cdc75842a8cedf3a59165c713b` under
`tests/visual_regression/snapshots/visual_regression__native_{bar,horizontal_bar,stacked_bar,stacked_horizontal_bar}.snap`.

`descriptor::numeric_labels` is the sole pre-font precision-search producer for
local Heatmap, local Beeswarm, and shared guides. It uses the existing modern
axis/guide grammar and re-generates every number at the same precision until
distinct scientific values have distinct text. Signed zero retains the existing
normalization to `0`. Family storage order, final scientific domains, stops and
missing-fill ownership are preserved; the old Beeswarm formatter and Heatmap
`ColorbarValue` implementation are removed.

Continuous label requirements use each scientific label's actual normalized
fraction, never a sampled stop's index or an assumed one-half center. Adjacent
projected line-box/ink half-extents plus the traced gap determine required length,
with a two-quantum allowance for canonical position/ink rounding. The theme's
gradient minimum is only a lower bound. Selection validates actual adjacent ink
clearance. Near-endpoint centers return typed fit when the assignment is too
short; they never move to one half or omit a label. The asymmetric numeric and
PNG fixture keeps zero at fraction `0.05`, samples the neutral color there, and
checks that the sampled one-half color remains nonneutral.

### Constant-Domain Singleton Clearance

The final singleton audit restores `05`'s separate constant-domain requirement:
gradient length is at least `2*max(Lc,Rc)+2*gap` and the traced minimum. A singleton
has no adjacent pair, so the pair-window calculation alone cannot enforce it.
The independent half-extent 30px, gap 5px, traced-minimum 40px example requires
70px. Actual-fraction pair calculation and its canonical rounding allowance
remain current.

The constant-domain fixture uses one generated `12.3k` label for the exact domain
`(12345,12345)`, with a private 96pt bold label role. In horizontal and vertical
flows, preferred and compact plans retain the complete midpoint label and
constant-color stops; actual selected text ink and complete unclipped guide ink
fit the allocation. Both variants accept the compact minimum allocation, and
both refuse an allocation one quantum below it along the gradient direction
with typed `LegendOrGuide` fit. No new size floor or fitting policy is introduced.

The intentional output effect is a longer singleton gradient when the text-plus-
gap requirement exceeds its traced minimum, particularly for compact or
constrained guides with large private typography. Built-in constant-domain
regression fixtures retain their current midpoint colors and label semantics;
this correction does not update any snapshot or fingerprint expectation.

The following intermediate fingerprints are attributed to the **first T5e7/8
guide-frame retention audit checkpoint**, before the final numeric-grammar and
fractional-clearance corrections. They superseded the historical T5e4 goldens
above, but are not current expectations after the final re-audit.

| Case | Publication bytes / FNV-1a | Science bytes / FNV-1a |
| ---: | ---: | ---: |
| 0 | 11019 / 1825385880297321021 | 10620 / 2960003661562927583 |
| 1 | 10311 / 713644927056565513 | 9912 / 1536757255564266869 |
| 2 | 10713 / 4522804833360596967 | 10314 / 15681363286863238177 |
| 3 | 10975 / 10357223378859205191 | 10576 / 13303339861000591691 |
| 4 | 20597 / 580262077683286106 | 20172 / 6779047544499001665 |
| 5 | 20609 / 7147670587444672895 | 20179 / 6167574795368840791 |
| 6 | 20597 / 580262077683286106 | 20172 / 6779047544499001665 |
| 7 | 20609 / 7147670587444672895 | 20179 / 6167574795368840791 |
| 8 | 23488 / 13718270375003789200 | 21768 / 5136673241507865077 |
| 9 | 16409 / 9163405914067832987 | 15351 / 3241744297710238993 |

## Focused Verification

### Interaction Composite-Key Integration Followup

The two Interaction closure failures were stale expectations after complete
guide ink allocation. All four 800x600 Publication/Science × plain/styled SVGs
and both 480x320 connector-visible/hidden SVGs were inspected. The renderer
retains the source marker/interval order, interval endpoints, palette colors,
point outline/alpha, and styled Circle/TriangleUp and dash semantics.

In the connector fixture, the visible guide reserves the round caps of its
1.7998046875px connector in addition to the nominal 11px handle. Each half-cap
rounds outward to 0.900390625px, requiring 1.80078125px extra key width. Hiding
connectors removes that painted component: the data clip width grows from
271.234375px to 273.03515625px, and the guide width shrinks by the same amount
while retaining its outer edge. Reprojecting the same conditions into that
larger body is intentional. Equality of the full SVG after deleting only
connector strings was therefore no longer a valid contract.

The replacement assertions independently project marker and interval values
from observed numeric-axis ticks, check categorical centers in each actual
allocation, retain exact trace/source order and all paints/outline sizes, and
require visible connectors to join the corresponding markers. The point-only
guide retains its labels and glyphs without connector calls; composite guide
caps and markers fit its unclipped frame. The explicit/default connector byte
contract and hidden-connector range-planning contract remain separately tested.

The inspected historical **pre-T5e complete-guide** root fingerprints from
`tests/interactions/styles.rs` remain attributed below. Earlier T5a evidence is
retained in `docs/local-search-benchmarks.md`; the current test pins every full
SVG case, rather than normalizing away guide or allocation geometry.

| Theme / style | Historical bytes / FNV-1a | Current bytes / FNV-1a |
| --- | --- | --- |
| Publication / plain | 11009 / `12aaf18097b9c5cc` | 11007 / 13027462214619244865 |
| Publication / styled | 11353 / `89b9c58e98ee340a` | 11352 / 15258619364319190699 |
| Science / plain | 10616 / `05b4da364c197d7e` | 10614 / 15450700033276445789 |
| Science / styled | 10960 / `ea68c6dbaa4dc974` | 10959 / 11883049564942211587 |

The discrete guide units, continuous-guide units, shared descriptor units,
Heatmap/Beeswarm extraction and exact-admission units, modern numeric grammar,
and guide-title failure provenance pass. Representative actual Shared discrete
and collective-pair integrations pass. Close numeric labels agree between local
and Shared output; asymmetric-center SVG/PNG checks and typed near-endpoint
refusal pass. The 20-case Beeswarm fingerprint matrix, its structural snapshot,
native path snapshots and all four native bar-family SVG/PNG fixtures pass.
Strict offline locked all-target Clippy, formatting and `git diff --check` pass.
This partition did not run a full test suite or publish a commit.
