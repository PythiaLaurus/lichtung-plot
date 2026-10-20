# Scoped guide dependency keys

Child and width-dependent height reuse now retain an explicit guide-consumption
key. This extends the effective policy and complete prepared contribution with
the local placement slot and the exact declared ancestor operands. This records
the typed-key checkpoint; actual Canvas/facet call routing is completed in
`routed-guide-dependencies.md`.

## Exact consumption

`GuideConsumption` is a borrowed call input. `ScopedGuideInputs` retains its
effective composition, Hidden/SharedSuppressed/Local use, authoritative source
preparation presence and complete scoped contribution, optional local
`DiscreteGuideSlot`, and immutable ordered dependency records. A source
contribution transported to Shared is distinct from the descriptor actually
drawn by a routed owner. Suppressed use cannot retain a local placement slot.

Each `ConsumedGuideDependency` names its original component owner, exact
canonical allocation and relevant feedback snapshot. Allocation-only use has no
descriptor operand. Descriptor-and-allocation use additionally retains the full
prepared descriptor by value, orientation and normal/compact selection. Width
and box allocations are distinct; widths use capacity Floor and box edges use
canonical nearest-even quanta. Duplicate or noncanonical owner order refuses.
Vectors use checked fallible reservation and immutable shared storage.

Complete descriptor equality includes section boundaries/order, raw identities,
strings, full glyph/explanation geometry and paints, source occurrences and
selected neutral RGB. Equal text, extent or Arc identity is insufficient. Source
matching also distinguishes unprepared input from authoritative prepared absence;
there is no semantic reconstruction on a miss.

## Reuse and stability

`ComposedInputs` owns this guide key instead of separate presence, descriptor and
composition fields. `ChildPlans` measurement and panel keys compare actual
consumption alongside their existing geometry, complete Theme, computed domains,
numeric policy and own scoped feedback. The duplicate panel placement-slot field
is removed. A hit requires both matching declared ancestor snapshots and stable
declared owners. End-of-round retention uses the same selective stability rule.
Unrelated root siblings are not added to the dependency set.

`HeightInputs` retains the same source/local/dependency identity. A recorded
consequence requires exact assignment equality and stable declared ancestors,
even when child width and numerical endpoints agree. This preserves replacement
height feedback, including contraction, rather than accumulating guide ink.

The existing placement-attempt accounting and child feedback replay remain
unchanged. A section does not become a placement owner or gain attempts. No
resource fee, quota, global cache or fallback flow is introduced.

## Replaced and strengthened tests

`actual_child_hits_require_exact_consumed_guides_and_stable_declared_ancestors`
replaces the narrower changed-tick-stack key test. It drives the real child
measurement cache with actual prepared Scatter guide descriptors. Different raw
fields have equal aliased titles, labels, glyphs and child bounds but invalidate
descriptor-consuming hits. Orientation, compact selection, dependency use and
width-versus-box allocations invalidate. Equal allocation-only operands hit.
Unrelated sibling feedback preserves the hit; an unstable declared guide owner
invalidates it at unchanged bounds. The literal changed tick-stack input remains
covered in the replacement contract.

The existing content-height numeric-key test is strengthened in place as
`recorded_heights_require_complete_numeric_and_guide_inputs_at_equal_width_and_domain`.
An exact declared ancestor width hits; undeclared dependencies and an ancestor
width smaller by one quantum miss while child width/domain remain equal. Complete
numeric-policy changes continue to miss, record and hit independently.

No unit or integration test count is added. Discovery remains **671 library
tests, including 17 ignored**. The new test file belongs to the existing private
child-cache module; no tests move to integrations.

## Verification

**41 distinct focused library tests** pass: six child-cache tests, six composed
interval/provenance tests, the height-key test, Shared effective-policy authority,
five facet-parent tests, sixteen native coordinate/feedback/reuse tests, five
section measurement/cache tests and frozen legend replay. **18 existing external
tests** pass: sixteen Shared composition callers, complete explicit guide
placement and inside-legend/significance collision.

Strict all-target Clippy, `just fmt-check` and `git diff --check` pass. Commands
use the named locked/offline Cargo filters; evidence is in
`/tmp/opencode/scoped-guides-*.log`. No broad-suite or release gate is claimed.

The subsequent routing checkpoint enumerates actual containing allocations and
guide owners for Canvas/facet child calls and the height allocation pass, and
removes this checkpoint's temporary non-test dead-code expectation. Public
composition controls and the full end-to-end Separate matrix remain later work.
