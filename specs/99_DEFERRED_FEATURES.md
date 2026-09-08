# Deferred Features

**Status**: Deferred and non-current

## Contract

This document records only genuinely excluded scope, not approved work awaiting
implementation. It grants no implementation authorization and reserves no API,
storage, callback, or fallback. Approved boundaries are in `03_THEME_SYSTEM.md`,
`05_LAYOUT_SYSTEM.md`, and T5a0s/D1-D6 in `91_IMPLEMENTATION_PLAN.md`.

Every item has a stable ID and an explicit review checkpoint enforced by the
entry/closure gates in `91_IMPLEMENTATION_PLAN.md`. Review is not an implementation
promise and does not automatically expand T6. At its checkpoint, present the
item's evidence and tradeoffs to the user and record one user-approved outcome:
admit with an approved new phase, reject with a reason, or re-defer with a new
explicit milestone checkpoint. A vague future review does not close a gate.

When work is admitted into an approved implementation plan, remove the admitted
feature from this file immediately, retaining a broader heading only for related
excluded scope. Do not wait for code to land. Separately, update the current
architecture/rendering contracts atomically when implementation lands; planning
or removal here never makes an API current. Future updates must preserve explicit
checkpoints and evidence/decision records for every remaining item.

After a review, no retained item may keep an expired checkpoint. Re-deferral
updates its checkpoint; rejection removes it from this active review list and
records the reason and any lasting exclusion in the plan or canonical contract.
Stable IDs are not reused. Related unadmitted scope may retain its ID only with
an explicit next checkpoint.

## DF1: Separate Legend Sections

Caller selection of separate group-style and hue-color sections, instead of
combined entries, remains excluded. Automatic placement and compact reflow are
approved layout work, not deferred legend composition modes; see `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T6 ENTRY, after T5e closure.

**Evidence and decision**: Review combined-entry examples with independent group
and hue, repeated hues, same-field collapse, sparse facets, and row-level hue.
Specify ordering, titles, label mapping, collision behavior, and Canvas shared
compatibility before recommending admission, rejection, or explicit re-deferral
under the user-approval process above.

## DF2: Caller Numeric Formatting

Broader caller scientific, engineering, compact, percentage, and locale-aware
formatting across axes, guides, values, totals, labels, and legends remains
excluded. No general format string, callback, prefix, or notation enum is reserved.

**Review checkpoint**: T6 ENTRY, after T5e closure, alongside DF1.

**Evidence and decision**: Collect cross-surface workflows beyond current
`NumericLabelFormat`; compare scaling, rounding, negative zero, extreme finite
values, measured widths, collisions, and deterministic SVG/PNG behavior. Present
a cohesive surface and its costs for the user-approved disposition, rather than
admitting formatter controls implicitly through legend work.

## DF3: Data-Mapped Alpha

Data-dependent alpha scales remain excluded; approved RGBA paint ownership is in
`03_THEME_SYSTEM.md`.

**Review checkpoint**: T6 CLOSURE, before T7 starts.

**Evidence and decision**: Review scientific workflows, numeric/categorical domain
and range, missing values, legends, accessibility, facets, Canvas coordination,
and precedence against final RGBA exact styles. Compare explicit alternatives
without introducing multiplied opacity; obtain the user-approved disposition.

## DF4: Broader Automatic Tick Controls

Target/maximum/exact generated counts, visual-density scalars, arbitrary
multiplicative ratios, categorical stride/drop policies, and generation callbacks
remain excluded. The admitted preferred-interval boundary is in `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T6 CLOSURE, before T7 starts.

**Evidence and decision**: Use implemented interval/coarsening and exact-tick
examples on small panels, facets, and shared Canvas dimensions to identify unmet
needs. Review precedence, nice-step deviation, semantic completeness, and resource
limits without a second planning route; obtain the user-approved disposition.

## DF5: Broader Text Layout Controls

Independent title versus subtitle alignment, general per-role alignment/anchoring,
arbitrary rotation, emergency identifier breaking, ellipsis, and font-shrink
controls remain excluded. Independent root captions and strict Unicode-legal
wrapping are admitted; see `03_THEME_SYSTEM.md` and `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T6 ENTRY, after T5e closure.

**Evidence and decision**: Review narrow unbroken identifiers, crowded category
labels, mixed-direction multiline text, and role-specific workflows. Compare
strict fit failures with opt-in alternatives, source preservation, physical versus
bidi-logical direction, precedence, collisions, and SVG/PNG agreement. Any proposed
omission or shrink behavior requires an explicit revised contract and user
approval; obtain the disposition without relaxing strict wrapping by default.

## DF6: Broader Root Bounds And Maximum Width

Maximum-width sizing, minimum width/height, maximum height, exact-height/automatic-
width sizing, and public Flutter-like constraint objects remain excluded. The
three admitted sizing modes are defined in `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Collect publication-column and embedding workflows using
automatic, exact-width/automatic-height, and exact-pair output. Demonstrate unmet
needs and compare interactions with aspect, wrapping, legends, facets, Canvas,
and strict fit. Let the user choose a small justified extension, rejection, or
re-deferral; do not assume a general bounds surface is the destination.

## DF7: Sparse Composition Compaction

Packing occupied cells or collapsing empty rows, columns, or trailing wrap slots
remains excluded; preserved topology is specified in `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T6 ENTRY, after T5e closure.

**Evidence and decision**: Compare sparse Canvas and Cartesian facet examples,
coordinate remapping, shared axes, header spans, row titles, range groups, empty
semantic combinations, and deterministic ordering. Show the semantic cost of
compaction and obtain the user-approved disposition rather than silently packing.

## DF8: Caller-Selected Layout Objectives

Caller objectives for aspect, density, whitespace, legend flow, text measure, or
grid balance remain excluded. No scoring callback, strategy switch, string mode,
or public layout plugin is reserved.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Review unmet scientific workflows against the completed
private local policies, including exact-constraint failures and reproducibility.
Compare a small typed objective with existing explicit controls and its bounded
costs; obtain the user-approved disposition without creating global search by
implication.

## DF9: Global Layout Optimization

Exhaustive whole-tree search, sibling Cartesian products, exhaustive text break
partitions, and root-wide dense-rank objectives remain excluded. Approved bounded
containing-frame placement recovery is not this feature; see `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T7 CLOSURE, before successor planning, alongside DF8.

**Evidence and decision**: Require demonstrated scientific workloads, measured
performance/resource comparisons with bounded local recovery, semantic
completeness, determinism, and a coherent replacement contract. Obtain the
user-approved disposition; historical search achievements do not authorize a
dormant solver or runtime strategy switch.
