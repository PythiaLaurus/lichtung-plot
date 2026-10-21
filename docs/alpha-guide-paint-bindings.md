# Typed Alpha Provenance In Existing Guide Projections

Source baseline: `eaadfc2`. Existing category/literal/layer guide keys now retain
explicit alpha provenance independently of their RGB and group-style provenance.

## Binding And Projection

- Each actual point fill/outline, line stroke and area fill/outline carries a typed
  `AlphaBinding`: owning role, exact element, or complete data input plus target and
  semantic unit. Immutable `AlphaEncoding` clones retain the complete values and
  mapping declaration through their shared owned storage; no row is selected as a
  hue/group representative.
- Existing primary/mandatory observation paints, series/aggregate strokes and
  series/aggregate/Single fills install their data binding during the existing
  font-independent guide preparation. Independently owned outlines and supplemental
  layers retain their role/exact alpha. Source validation checks alpha binding,
  actual component presence and the complete explanation mask together.
- Both Combined and Separate existing keys neutralize data-bound target alpha to
  255, including Undeclared inputs. Their RGB, shape/pattern and independent outline
  rules keep their existing owners. Role/exact alpha without a data binding remains
  unchanged, and group-neutral RGB still ignores legend text alpha.
- Complete keys and projected masks now include typed Constant/Neutralized alpha
  masks for every actual paint. These participate in complete key equality and
  descriptor/source occurrence identity. Equal final glyph paints with different
  alpha explanation responsibilities cannot silently merge.
- The existing owner, source preparation, section order, placement attempts,
  measurement cache, structural key occupancy and draw lifecycle remain the route
  for these keys. This leaf adds no explanatory alpha section or second guide path.

## Focused Evidence

**Ten library contracts passed** with nonzero offline filters:

- Bar source provenance compares both compositions with actual aggregate body
  bytes `[0,128]`, proving the existing hue key uses alpha 255 while its independent
  outline retains 211. The full Aggregate/Fill input is retained. A role-opaque
  control has identical fill/outline/width but a distinct explanation mask; merging
  it with the data-neutralized key rejects specifically for that mask.
- Interaction source provenance compares both compositions with independently
  varying observation bytes and three explicit trace bytes. Existing point/line
  contexts use 255, outlines retain their exact RGBA, and group-neutral RGB remains
  `(40,50,60)` despite legend text alpha zero. The neighboring role-only witnesses
  still retain point alpha 17 and line alpha 211.
- All four Line source-order/supplemental paint contracts pass, including independent
  supplemental alpha/label paint and sorted observation/series/band targets.
- Exact Dumbbell endpoint/outline/canonical-capture behavior, independently mapped
  Regression bands, sample/supplied Raincloud strips and faceted Density operands
  pass through the extended guide source representation.

**Twenty-two existing public integration contracts passed**: unsupported Separate
factorization/Hidden scientific validation; same-field group/hue collapse across
all pair families; all nineteen Interaction style contracts, including actual
glyphs, exact paints, Shared conflicts, Separate sections, facets and deterministic
output; and sample/supplied Raincloud Shared hue coordination.

No library test was added or deleted; discovery remains **671 including 17 ignored**.
Strict `just lint`, `just fmt-check` and diff checks passed.

## Remaining Guide Work

The final design in `data-alpha-design.md` still owns the sole unified
`PreparedGuide`: ordered existing symbols, optional native continuous-color body
and declared alpha explanation sections. Actual alpha member/glyph collection,
declared explanations, exclusive-kind retirement, common measurement/frame,
parent compatibility/feedback and public cutover remain pending. In particular,
the current private alpha inputs are not a completed public guide contract.
