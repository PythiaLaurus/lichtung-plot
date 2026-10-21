# Numeric Tick Generation Through Parents And Frozen Output

Baseline `52cafa6` exposes the single current TickGeneration API. This checkpoint
closes its focused parent/root/output obligations, including an actual missing
Canvas height-key input. Integrated release verification remains separate.

## Complete Parent Identity

Source review found that `canvas::sizing::HeightInputs` retained numeric formats,
ranges and provenance, but not generated-source selection. It now retains both
concrete `TickGeneration` values independently of NumericFormat and compares
them in `matches_assignment`. The private key uses exact PartialEq over validated
finite generation values, rather than projecting away kind or count to preserve
an Eq derive. No duplicate selection encoding or optional interval/count chain is
introduced.

The existing recorded-height contract now also prepares a valid Scatter with
target five and unchanged explicit ranges. Changing x to maximum five, target
four, Automatic, or changing only y's generation must miss at the same canonical
width/domain, even when the recorded height is identical. Re-recording the changed
inputs hits them and invalidates the original inputs. Existing Heatmap numeric
format and declared guide-allocation witnesses remain.

The actual ChildPlans cache test independently checks Target/Maximum five on
each axis: changed selections invoke the producer, unchanged repeats share the
retained Arc, and normalized format inputs remain independently significant.
Existing source/provenance/composed tests remain in place. The feedback contract
now distinguishes thirteen complete states: Automatic plus two additive bit
values and Target/Maximum counts five/six on either axis, while independently
cloned equal inputs compare equal. A selected exact-axis plan retains both
otherwise suppressed generation choices beside its frozen source/projection.

## Actual Public Consumer Breadth

The existing public family/root census now attaches target five to numeric x and
maximum three to numeric y, together with the established numeric formats. It
renders all twenty-five kinds in Automatic, exact-width and exact-size modes,
plus every admitted one-cell Canvas: **147 actual canonical SVG roots**. This
includes Forest's certified exponent route, Coefficient/Waterfall value consumers,
Marginal and Matrix's unchanged native owner. Every generated y set contains
one through three labels. Required caller prefixes and requested exact dimensions
remain present; automatic sizes are not pinned.

`examples/07_tick_generation.rs` publishes independent targets/maxima on both
axes through a two-panel Canvas. The executed example reports **1200×600 CSS
pixels**. README links the example and the public construction contract.

## Frozen Multi-format And Cache-disabled Relationship

The real publication contract in `tests/benchmarks/ticks.rs` renders:

- standalone linear Scatter;
- shared owning facets;
- a Canvas x-range group with distinct child generation/format instructions;
- standalone Forest Log10;
- a Forest Canvas with independent child selection.

Each uses Target or Maximum, all three root requests, and enabled/disabled child
reuse: **60 complete outlined SVG / preserved-text SVG / PNG publications**.
Same-case cached and uncached bytes must agree in every format. Scientific point
membership is independently six, twelve, two or four according to actual topology;
faceting partitions the six rows without duplication. Every selected source is
nonempty, generated maxima bound all present numeric positions, and ample exact
requests retain the independently justified five linear/four logarithmic targets.
Shared ranges do not force identical tick presentations. Width-only/exact requests
preserve 1200px and exact height 800px; PNG reports the frozen SVG dimensions.

All sixty operations have one selected capture and one final Null validation,
zero work in all 32 capture counters, and one through eight root rounds. Root
constructors are consumed by publication; subsequent terminals retain only Output.
No new width search, selected-coordinate golden, source-row omission, guide budget,
font lifecycle or draw-time formatting is introduced.

This meaningful library publication contract replaces the lower-value
`paired_measurement_contract` helper inventory in the same logical change. No
other test is deleted, added, merged or moved into integration. The strengthened
external all-family contract is the existing public consumer census, not a unit
test migration. Discovery remains **671 library tests, including 17 ignored**.

## Bounded Release Workload

The existing ignored `publication_efficiency_measurements` now calls
`ticks::measure()`: target 128 versus maximum 12 over six actual linear
observations at 1200×800, three interleaved runs and one uncached maximum control.
It checks per-operation output, repeated output identity and cached/uncached
equality, and reports actual candidate/format/metric/root work through the existing
paired measurement machinery. Different policy outputs are not asserted equal,
and no speedup is inferred from fewer logical candidates.

This release workload is constructed and strictly compiled here, **not yet
release-executed**. Its timing/RSS evidence belongs to the integrated gate.

## Focused Verification

| Actual filter | Passed |
| --- | ---: |
| `recorded_heights_require_complete_numeric_and_guide_inputs_at_equal_width_and_domain` | 1 library |
| `child_measurement_reuses_hits_and_replaces_changed_numeric_policies` | 1 library |
| `axis_semantics_and_standard_typography_are_feedback_identity` | 1 library |
| `selected_axis_plan_retains_semantics_beside_projection_and_sources` | 1 library |
| `generated_tick_choices_survive_all_root_modes_parents_and_cache_disabled_replay` | 1 library / 60 publications |
| `builder::facets::generation_tests::` | 6 library |
| `layout::panel::sources::tests::` | 21 library |
| `--test layout all_public_families_render_content_requests_with_their_actual_axes_and_chrome` | 1 external / 147 roots |

The 32 focused library tests, one external contract, actual example, strict
all-target Clippy, formatting/diff checks and 671/17 discovery pass. Logs use
`/tmp/opencode/tick-generation-closure-*.log`. No full-suite or final release
claim follows; data alpha and integrated T6 closure remain explicit in spec97/93.
