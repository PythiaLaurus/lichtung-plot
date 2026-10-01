# T5e4 — Beeswarm Tangent Packing

The prepared owner retains categorical/source-row order, original numeric and
color values, source identities, final OKLab colors and sampled guide stops.
It performs no font-dependent or assigned-width packing before fonts. The old
48-bin, x/color sorting, alternating 0.09 offsets and 0.36 cap are deleted.

At each assigned numeric projection the producer rounds x to spatial quanta,
uses a conservative disk enclosing the complete stroked glyph, generates zero
and prior-point tangencies, orders by absolute offset/negative first/source row,
deduplicates equal offsets and selects the first collision-free candidate.
Integer square roots round tangencies outward; squared distances and spans use
checked wide arithmetic. Prior placements never change during that call.
The occupied symmetric category extent becomes the common categorical pitch;
native Grid bands retain semantic categories and `2em` as a soft preference.
Changed root inputs cause another admitted call, not an internal retry/search.
Insufficient assigned height returns a typed track miss with all input rows
retained. Exact numeric windows may clip points; required axes/guides still fit.

Independent expected tests include coincident offsets `[0,-4,4,-8]` for radius
two quanta, acceptance of exact touching at four-quanta x separation, outward
rounding of `sqrt(15)`, both admitted Circle/TriangleUp glyphs with broad strokes,
and complete collision checks over retained points. Selected plans preserve
category/source-row pairs and frozen replay performs no packing or preparation.

## Actual Core Work

The focused increasing-coincident workload uses a ten-quanta conservative
radius. Figures exclude font/semantic preparation and are actual operation
ledger usage, not estimates of an SVG shape count:

| Points | Processing visits | Cumulative bytes | Occupied hard pitch (quanta) |
| ---: | ---: | ---: | ---: |
| 8 | 333 | 1,344 | 180 |
| 16 | 1,689 | 4,736 | 340 |
| 32 | 9,521 | 17,664 | 660 |
| 64 | 60,001 | 68,096 | 1,300 |

The 512-coincident source refuses fatally at the existing visit budget; there is
no secondary packing cap, fallback, hidden point omission or prior-point
backtracking. Candidate generation, sorting comparisons, dedup visits and every
tested pair are admitted. Exact visit/byte boundaries and selected-plan storage
are separately tested. The large-marker numerical-edge fixture uses sufficient
height for real tangencies rather than relying on the former capped overlap.

## Rendering Evidence

The Beeswarm clip snapshot keeps every mark and required guide label while
showing the native frame/axes and common measured right continuous guide.
All 20 Publication/Science standalone, four-scale facet, shared-range Canvas
and mixed Heatmap Canvas fingerprints intentionally change. Historical bin
fingerprints remain available in commit `3800626` and earlier attribution in
`docs/local-search-benchmarks.md`; the current expected fingerprints are in
`tests/categorical/beeswarm.rs`. Heatmap and Beeswarm now share the same measured
guide-track assignment helper, with their plot-owned paint payloads retained.
Public exact-size defaults and supported composition routes remain current.
