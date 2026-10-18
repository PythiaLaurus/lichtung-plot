# Prepared Discrete Sections

Combined guides now use one immutable `PreparedDiscreteGuide`, containing a
nonempty typed section, resolved title, typed entries, complete point/line/fill
keys, actual producer/address/ordinal source occurrences and checked integer
counts. Entries no longer own duplicate titles. Transient `GuideSourceKey` values
are producer operands without display strings, not another retained flat guide.
Exact duplicate entries retain all actual source occurrences; entry deduplication
does not erase the source census. Ordinary one-scope roots share their descriptor
directly when no parent reordering is needed.

## Preparation And Ownership

Standalone, facet, Canvas PerPlot/Shared/Hidden and native Matrix preparation now
establish contribution or explicit absence before the font census. A missing
prepared owner/scope is an invariant error. Shared-suppressed children retain
their prepared source contribution while receiving no local guide placement.
Hidden sources do not resolve guide text or materialize guide keys.

Facet color membership comes from prepared scientific inputs rather than legend
entries. Canvas coordinates actual categorical color fields, including Interaction
group colors without hue, independently of guide presence. Parent palette order
remains separate from entry membership; Line retains observed local hue indices
even when preparation receives a global palette domain, and row-hue keys filter
that domain by local membership. The exhaustive producer dispatch retains the
intentional Matrix parent refusal; Matrix has its own pre-font source adapter
and uses the same descriptor and measured guide. Its native preparation explicitly
retains its scope even though it has no ordinary axis-panel record.

Only final descriptor titles and labels enter discrete font census, in section
preorder. `LegendSection[s]/LegendTitle[0]` and
`LegendSection[s]/LegendEntry[i]/LegendLabel[0]` remain below the existing guide
slot. A `DiscreteGuideSlot` retains its original owner when a facet routes its
root guide into a selected cell. Canvas local slots stay below CanvasCell; Shared
slots remain at the Canvas root. Empty strings add no painted text requirement.
Continuous descriptors and their numeric/missing text retain their separate path.

Natural preference, assigned Combined measurement and selected drawing consume
the descriptor. Complete descriptor equality participates in composed child keys,
consumed guide-slot equality and relevant local-guide height keys; selected
feedback retains the complete immutable descriptor. Existing geometry, numeric
policies, resolved Theme and selective feedback inputs remain. The selected
`LegendDrawPlan` still owns all frozen drawing operands.

## Retired Storage And Reconstruction

The flat tuple, `LegendEntry`, per-entry titles, flat result/cache overrides,
`resolve_entries`, `merge_exact_entries` and late `resolved_title` are removed.
`PlotRenderResult.discrete_guide` attaches the already prepared source descriptor;
selected marks no longer rebuild entries. Canvas's body-probe fallback and
legend-derived hue fallback are deleted. Facet title/entry duplicates and inferred
discrete title/value/layer font strings are deleted, including their obsolete
finite-pair census helpers. Guide-kind compatibility uses actual prepared presence,
so signed HorizontalBar keys do not disappear merely because no hue list exists.

Direct producer/root tests now establish the same explicit guide preparation
boundary. Small Rust-only test helpers perform that boundary without a production
fallback. Fresh theme operands get fresh prepared guides; poisoned selected
metadata still cannot alter frozen publication.
Selected descriptor validation applies to native self-managed owners and retained-
axis callers as well as ordinary panels; their axis handling does not suppress
guide identity validation.

## Verification

**73 distinct focused library tests pass**, covering:

* 22 text-census contracts, complete Line/Dumbbell/Regression/Density key/mark
  parity, and sampled/supplied mandatory-strip roles;
* the actual Shared primary/layer authority contract, now also checking exact
  section/key/source counts and final pre-font section addresses;
* all 25 native kinds through one lifecycle, 16 native coordinate/source/feedback
  callers, six child-cache contracts, the numeric height-key contract, native
  Matrix content roots, selected-guide metadata poisoning and specialized prose;
* actual Trajectory point/key parity with an unobserved parent hue, Waterfall
  geometry, and selected Heatmap/Beeswarm/continuous-scale owner callers;
* five facet parent and five facet feedback/capture contracts, plus the final
  Shared numeric-guide boundary.

**22 existing external tests pass**: public all-family/all-root census (147 actual
SVG roots), native Matrix modes, exact Shared/sparse facet pairs, Interaction
glyphs, layer collisions/duplicates, independent pair aliases/same-field collapse,
Hidden font/color behavior, PerPlot membership, Shared completeness, continuous
Heatmap/Beeswarm boundaries, neutral row-hue points/connectors, sparse faceted
Line/Trajectory domains, and signed HorizontalBar local/Shared output. The sign
contract now requires one real `Change` title and each actual sign label exactly
once in both owners.
The Hidden color contract now coordinates group-only Interaction with a visible
Scatter peer: the hidden group's actual marks retain palette slot zero and the
visible hue retains slot one despite the hidden source's absent guide.

Strict all-target Clippy, formatting, whitespace and retirement searches pass.
Discovery remains **671 library tests, including 17 ignored**; no test functions
are added, deleted or moved in this leaf. A zero-match exploratory facet filter
is excluded; the real `feedback_tests` module runs five tests. Initial direct-test
fixtures missing the new preparation boundary were migrated, not given a fallback;
the native Matrix missing-scope failure was repaired in its owning adapter.
Focused repaired callers pass; no full-suite gate or golden repin was used.

This completes Combined semantic storage/transport and flat retirement. Producer
paint/style binding and projection provenance, Separate construction, multiple-
section flow, the complete consumed-ancestor dependency contract and public
composition controls remain the following independently verified leaves. This
record does not claim those target capabilities or integrated T6 closure.
