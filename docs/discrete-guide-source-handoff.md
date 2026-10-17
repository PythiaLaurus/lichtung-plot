# Discrete Guide Source Handoff

This source refresh is against `3de25e0`, after public numeric presentation and
its scoped/all-root closure. Spec08's producer, typed section, projection,
authority and bounded flow design remains closed. This record maps it to the
current sources; it does not claim section implementation or acceptance tests.

## Current Representation And Measurement

`decorations/legend/descriptor.rs::PreparedDiscreteGuide` remains the tuple
`(String, Arc<[LegendEntry]>)`. Continuous descriptors are already immutable Arcs
with complete numeric policies/sets and must retain that independent boundary.
The flat `LegendIdentity::{Single,Pair,Literal,BarSign}` and `LegendEntry` still
own per-entry title/label, complete point/line/fill glyph and a guessed
`contributes_hue_level` flag. `resolve_entries`, `merge_exact_entries` and
`resolved_title` enforce one flat title. `Legend` retains flat entries, one title
block and entry blocks/slots; its exact metrics, full key ink, normal/compact
flow, draw calls and feedback are the measured owners to extend.

`PlotRenderResult.legend_entries`, the child cache's entry override, and
`decorations/plan.rs` discovered-entry replacement still carry the flat semantic
route. Canvas also retains a late `None -> collect result.legend_entries` fallback
in `preflight_shared_legend`. These must retire with authoritative pre-font typed
descriptors, including explicit noncontribution.

## Actual Producers

The sealed Legend set remains seventeen kinds:

| Families | Current producer and complete key |
| --- | --- |
| Scatter | `PreparedScatter::guide_entries_in_order/with_colors`; primary point |
| Line / Trajectory / LinePath | `PreparedLine::guide_entries_in_order/with_colors`; actual family/marker/ribbon grammar, row-hue point and labeled layer keys |
| Density | `PreparedDensity::guide_entries_in_order/with_colors`; curve plus area |
| Bar / HorizontalBar / StackedBar / StackedHorizontalBar | `bar_data::legend_entries`; body fill/outline, including observed sign domain |
| Boxplot / Violin / Raincloud | `PreparedDistribution::guide_entries`; summary/box, violin, or mandatory-strip point plus violin |
| Interaction | `PreparedInteraction::guide_entries/with_colors`; actual resolved trace point shape and connector pattern/presence |
| Dumbbell | `DumbbellData::guide_entries`; exact caller endpoint points |
| Regression | `PreparedRegression::guide_entries`; observations and fit/optional band, explicit Visible only |
| Marginal | `PreparedMarginal::guide_entries/with_colors`; point plus marginal density |
| Matrix | `matrix_guide_entries`, `matrix_legend`, `natural_guide_preference`; local point plus matrix density |

The remaining six ordinary/shelf kinds contribute no discrete key; Heatmap and
Beeswarm are continuous only. TextPanel is caller prose. Matrix remains excluded
from Canvas/facets; Marginal remains excluded from generic facets.

The complete public Theme migration is already done. Use current named roles:
`theme.typography.guides.legend_label.paint` for projection-neutral source RGB,
the producer's selected DataOutline/PointOutline policy and complete backend
Color/geometry. Raincloud sample/supplied keys already use `mandatory_strip` with
the actual outline policy; do not restore the historical raw-overlay substitution.
Do not infer a binding from equal RGB. Only caller-configured Interaction styles
are a mapped independent group-style channel.

## Source Drift And Parent Boundaries

* `PreparedLine` row-hue keys still iterate coordinated global hues, and labeled
  layers still use `point_legend_identity(...).without_hue_level()`. Retain observed
  membership independently of color order, then introduce true typed layer keys
  and their original categorical/literal label context.
* Interaction already selects group colors without hue and can read
  `global_group_levels`; Canvas preparation currently coordinates primarily via
  `hue_dimension_key`. Carry the actual prepared color field/domain independently
  of section visibility, including hidden-entry marks.
* `GuideKind::contributing_prepared` has family-specific presence guesses (bar
  hue lists in particular). The typed descriptor must become authoritative;
  signed keys and explicit noncontribution cannot depend on a later body probe.
* `PreparedInteriorInputs::guide_entries` is the exhaustive parent-capable
  producer dispatch. Matrix's local three functions are a separate owner to
  migrate to the same descriptor and measured guide, not a bypass.
* `orchestrator/facets/legend.rs::{prepare_discrete_guide,order_entries}` retains
  parent domain ordering; its categorical axis signatures remain unrelated.
* `layout/text/requirements.rs` still infers `legend_title_census`, discrete
  values and layer strings; Canvas walks one flat title/label list. Replace only
  those paths with final section preorder before canonical font resolution.
* N5's `ComposedInputs` and `HeightInputs` now include complete `NumericInputs`.
  Preserve those keys while adding effective section policy/descriptor and the
  explicitly consumed allocation/feedback dependencies. Child cache also keeps
  exact origin, content/fill flags, complete Theme and relevant scoped feedback.

Spec95 continues to supersede historical accounting prose: preserve real checked
counts/lengths, fallible storage, exact comparisons and one owner budget; no fees,
quotas or comparison tickets return. Section construction, producer factorization,
bounded measurement, parent coordination and public selection remain separately
verified implementation leaves in spec93. This documentation-only refresh was
checked against those owners and with `git diff --check`; no tests were required.
