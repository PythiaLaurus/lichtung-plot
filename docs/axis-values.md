# Private axis component values

## Implemented representation

`src/theme/axis.rs` implements the axis constructor schema in §7 of
`specs/94_T6_FOUNDATION_CONSTRUCTION.md`. `ResolvedTheme.axes` owns one
`AxisTheme`. The old `FrameAppearance`, `TickAppearance`, `AxisAppearance`, pixel
`AxisSpacing`, and dead `source_show_top_right_spines` factory/fixture metadata
are removed.

The existing public `Theme` remains the preset selector. Its operation-owned
factory constructs all six exact relationships on the supplied generation
resources. These data-only component constructors introduce no operation ledger,
font discovery, asset access, output lifecycle, or public export.

## Complete construction and validation

| Value | Complete inputs | Validation |
| --- | --- | --- |
| `FrameSides` | `top`, `right`, `bottom`, `left`: `bool` | At least one side; four private bits retain the selections |
| `FrameTheme` | `sides: FrameSides`, `stroke: StrokeTheme` | Complete sides and finite positive present CSS stroke |
| `TickTheme` | `stroke: StrokeTheme`, `length: u32` | Complete stroke and positive integral CSS length |
| `AxisSpacing` | Six named `RelativeSpacing` values below | Already validated nonnegative exact relationships; zero is valid |
| `AxisTheme` | Optional complete `frame`, `cell_frame`, ticks, grid stroke; complete spacing | Every present child is revalidated; absence is structural `None` |

Every field has its field-named readonly getter and consuming `with_field` edit
returning `Result`. Edits retain unaffected fields and validate complete siblings
immediately. No all-absent `FrameSides` value substitutes for either absent frame.
Widths of zero are not absence sentinels. Positive finite f32 extremes and
`u32::MAX` lengths remain valid construction inputs; representability and
nonzero painted-width collapse are checked at preflight rather than by inventing
a constructor minimum.

The helper macros generate only the uniform field construction/edit contract.
They do not create alternate constructors, old-name aliases, mutable getters,
hidden presets, or a second appearance representation.

The two frames are independent visual roles in this one AxisTheme. `frame`
owns ordinary coordinate panels; `cell_frame` owns the existing ledger role
`axes.matrix.cell-frame-final-style`. Every built-in explicitly constructs an
ordinary four-sided frame and a bottom/left-only cell frame, each with its own
complete F/preset-width/Solid stroke. Neither role inherits from or falls back to
the other. Their independent getters and consuming edits validate all present
siblings, including a malformed unchanged frame in either role.

## Exact source and metric mapping

Let `A` be the preset axis-title size. Factory inputs use
`RelativeSpacing::ratio_in(numerator, denominator, resources)` directly:

| Retained field | Exact preset relationship | Selected local metric | Preserved default pixels |
| --- | --- | --- | ---: |
| `x_tick_label_gap` | 8/16 | Complete selected tick-label CSS size | 8 |
| `y_tick_label_gap` | 6/16 | Complete selected tick-label CSS size | 6 |
| `x_title_tick_label_gap` | 8/A | Complete selected axis-title CSS size | 8 |
| `x_title_without_tick_labels_gap` | 16/A | Complete selected axis-title CSS size | 16 |
| `y_title_tick_label_gap` | 16/A | Complete selected axis-title CSS size | 16 |
| `y_title_edge_padding` | 12/A | Complete selected axis-title CSS size | 12 |

`AxisSpacing::lengths` returns only post-selection exact products. Those products
are never stored as another theme configuration or reconstructed through
`RelativeSpacing::as_f64`. Changing a selected role's CSS size changes its own
relationships; tick-label and axis-title metrics are independent.

Actual consumers now include:

- `layout/panel/axis_frame_grid.rs`: frozen ticks, label/title placement,
  complete gutters, selected frame sides, grid and geometry capture.
- `orchestrator/facets/{planning,rendering}.rs`: shared title hard reserves and
  final selected-title placement.
- `layout/interior/sizing.rs`: natural proposal chrome selected through the same
  borrowed coordinate/cell-axis context as its renderer, including independent
  bottom/left occupied extents.
- `plots/matrix.rs`: cell-frame capture, common axis spacing, tick selection and
  retained complete cache inputs.

## Spatial boundaries and absence

Spacing, exact represented stroke half-widths, integral tick lengths, and actual
logical/glyph extents are combined before the hard boundary. Gutter sums round
upward once. They do not add separately rounded hard-gap estimates. A live
no-chrome fixture gives tick/title gaps of one third of a quantum each: its
complete hard sum reserves one quantum, rather than the two quanta that separate
ceilings would produce. Both required text plans remain present.

Positions round nearest-even with their complete anchor and offset. The exact
`ExactLength::before` adapter preserves the final coordinate's parity at an
odd-parent half-quantum tie; it does not round a half-width/offset first and then
subtract it from a differently paritied parent. All integer expressions,
conversion bounds, and reservations use checked fallible paths on the active
operation resources.

Standard frame segments retain their old bottom/right/top/left order and are
frozen as four optional segments. Only selected segments are replayed, exactly
once. Endpoint stroke overhang remains occupied on the edges it actually reaches.
An inactive opposite edge gets no fictitious frame reserve. Tick offsets include
the frame half-width only when the consuming bottom/left side is present. Removing
frame/ticks/grid removes their geometry and ink, while required labels and titles
remain independently selected and measured.

Matrix cells independently consume `cell_frame` and preserve their old
bottom/left ordering before any selected top/right sides. Common cell-axis plans
remove `frame`, `cell_frame`, and `grid` from their operands to avoid painting
duplicate cell chrome; the original selected cell stroke/sides are already frozen
in the Matrix payload. Actual cell-frame ink participates in measured axis
geometry. Standard panels do not paint or reserve `cell_frame`.

### Natural Matrix sizing role isolation

`AxisTheme::coordinate_chrome` and `cell_axis_chrome` select borrowed operational
views of the actual common-axis operands. Matrix common axes have ticks/text but
no ordinary frame or grid. Both native Matrix planning and `plot_size` use the
same cell-axis selection. Thus natural width/height do not resolve or add an
unused ordinary frame half-width. The view retains no second appearance value,
clones no theme/payload, and creates no operation resources. Selected `cell_frame`
ink remains in the measured native composite's axis geometry and containing-root
requirements, separately from this common-axis proposal.

Private automatic and width-only (800px) root regressions compare both exact
dimensions and full SVG vectors while changing only the unused ordinary frame:
64px, `f32::MAX`, absence, left-only, and top/right-only sides all produce
identical Matrix output. The corresponding Scatter roots consume the ordinary
frame: 64px changes dimensions and SVG, and `f32::MAX` refuses. A separate Matrix
regression changes the selected cell stroke to 64px and confirms changed root
dimensions and actual painted SVG; selected `f32::MAX` cell ink still refuses.

## Copy, identity, feedback, and cache accounting

The six `RelativeSpacing` payloads are immutable Arc shares; component nesting
adds no separately allocated node. `AxisTheme::share` admits six handles and zero
payload records/bytes. Standard-panel inputs retain that complete component next
to their existing admitted complete text copies. Their matching path compares
the selected chrome, every exact relationship, and both complete text roles.

`AxisTheme::same` admits one fixed descriptor witness covering both optional
frames (including their sides and complete strokes), ticks and grid, then each examined
relationship. Shared equality costs **7 visits**. Independent equal one-limb
relationships cost **31 visits**: one descriptor plus six comparisons of one
handle, two length witnesses, and two integer digits. Later-field mismatches stop
at their actual terminating digit; resource refusal propagates as an error.

`RelativeSpacing::same` prices each length/digit before examining it. It compares
normalized numerator and denominator, never approximate inspection values or
constructor provenance. A minimum-subnormal independent equality or final-bit
mismatch costs **21 visits**: handle, numerator length/digit, denominator length,
and seventeen denominator digits. The minus-one fixture refuses before the final
digit with zero new storage. Feedback records the same normalized integer
identity, with admitted digit reads and publication hashing.

Whole-theme sharing now prices **30 appearance handles**: eighteen fonts, three
scale payloads, one accent payload, six axis relationships, and the two present
point-outline relationships. Those existing point ratios are included rather
than silently skipped by the broader clone. Outline absence reduces that count
by its actual missing handle. Shared full-theme equality costs **31 visits**,
including the axis descriptor. Point-ratio matching is admitted once; the
remaining mark comparison exhaustively excludes those two already-compared
point values so it cannot rescan their arbitrary-size integers unpriced.

The child-copy fixture exhausts all one-million record slots and permits zero
new bytes. Exactly thirty visits still share all font, palette, gradient, accent,
axis-ratio and point-ratio payload addresses. Minus-one work refuses atomically
before the clone. A child tick-font mismatch costs **29 visits**: four scale
witnesses, seven axis witnesses, two point-ratio witnesses, four preceding font
witnesses, and twelve terminating-name examination visits.

Matrix tick caches now retain operation identity, selected range/span/side, the
complete `AxisTheme`, and complete selected text inputs. A different operation,
metric, font, spacing, or chrome cannot reuse the old selection. Matching refusal
does not trigger reanalysis or discard the retained cache entry. A default
warm hit costs **144 visits**: one key witness, seven axis witnesses, and two
complete font comparisons of 68 visits each. The last admitted call still succeeds
with no cache-entry capacity or new payload storage available; the next call
refuses before lookup.

### Preset construction oracle

Each exact ratio retains a header and two bounded integer operands. On this
target, its charge is
`size_of::<RelativeSpacing>() + 2*size_of::<BigUint>() + size_of::<f64>() +
2*size_of::<usize>() + 2*(16 + size_of::<BigUint>())` bytes: **160 bytes**.
Six relationships add **960 bytes**, without new retained record slots.

The six constructor work charges are header/integer work plus their Euclidean
steps: **31 visits for A=16**, **38 for A=17**. The previous independent preset
oracle `31 + 2*n` therefore becomes **`31 + 2*n + axis_work`**. Publication costs
85 visits, Nature 72, Science/CHI 79. Plot and Canvas root/explicit-child
minus/equal work/byte tests verify these figures before font discovery or backend
begin. Inherited Canvas children still share the root instead of constructing
another preset.

## Independent full Matrix byte parity

The corrected schema preserves the existing cell-frame role explicitly. A
two-variable default Matrix still has **8 frame segments** (bottom/left in four
cells). An explicit four-sided `cell_frame` has 16, left-only has 4, and absent
has 0. Live private fixtures use a distinct cell paint to count these actual SVG
segments independently of ticks, and preserve eight scientific observation points
and axis text. Removing the ordinary frame leaves the selected cell frame active;
changing a cell frame does not alter the ordinary frame.

`tests/marginal_matrix/axis_parity.rs` adds whole-document regression evidence,
beyond the existing structural snapshot that excludes out-of-clip frame lines.
An independent temporary Git worktree at **633c8e0** rendered the same public
800×600 Matrix with variables `x=[1,2,3,4]`, `y=[4,3,2,1]`, for all four presets.
Both worktree and current compiler validation used strict offline Clippy. Full
current SVG byte vectors were compared directly with those baseline files, with
no differences. The regression also pins the original lengths and SHA-256:

| Preset | Full bytes | Original SHA-256 |
| --- | ---: | --- |
| Publication | 19413 | `400421d31a904b9bdefe4f4ed5d3dc0a9d12fa3e7893e2c70955e3a593c3e654` |
| Nature | 19332 | `ddb322a2205c03adbed763785ea5c00bf6d3b49b3c9c798272839bd7b8abad5e` |
| Science | 19028 | `353bce392eaa79fc19aac3d1574d6e38450452e7a884eb20e4829dd50b59b632` |
| CHI | 19413 | `79a8aa3e47ccd976c46c2ea48b5cfc2e3cfa34aa06ecd48dafa551c4d5e96d7a` |

Every default scene retains eight frame segments, eight observation circles, and
24 complete text elements. The expected digests are taken from the committed
baseline, not updated to a changed output. There is no built-in/custom selector,
renderer-enforced two-side mask, family theme, or public mode. Snapshot
expectations are unchanged and the default-output acceptance issue is closed.

Baseline generation command, with a temporary Rust fixture in the detached
worktree's existing `tests/marginal_matrix/` target:

```text
git worktree add --detach /tmp/opencode/axis-cell-baseline-633c8e0 633c8e0
just fmt && just lint
LICHTUNG_MATRIX_BASELINE_OUTPUT=/tmp/opencode/axis-cell-baseline-svg cargo test --locked --offline --test marginal-matrix default_matrix_full_svg_baseline -- --nocapture
```

The final two commands run in that worktree. The current direct-byte comparison
and standalone golden regression use:

```text
LICHTUNG_MATRIX_BASELINE_DIR=/tmp/opencode/axis-cell-baseline-svg LICHTUNG_MATRIX_PARITY_OUTPUT=/tmp/opencode/axis-cell-current-svg cargo test --locked --offline --test marginal-matrix default_matrix_full_svg_bytes_match_the_independent_committed_baseline
cargo test --locked --offline --test marginal-matrix default_matrix_full_svg_bytes_match_the_independent_committed_baseline
```

The environment variables retain/compare test artifacts only; ordinary test runs
always enforce the committed full-document digest without needing a worktree.
The full-byte/digest test remains platform-independent. Its supplementary XML
topology helper and call are gated with
`all(unix, not(any(target_os = "macos", target_os = "android")))`, exactly matching
the existing target-specific `roxmltree` dependency. No dependency was added.

## Focused verification

**82 distinct focused tests passed**, including the affected Matrix selection
caller, source-refusal neighbor, semantic feedback and exact selected-storage
boundaries and representative natural-root sizing callers. These results come
from bounded filters, not a full-suite run. The original 78 checks were rerun with
the completed five-input schema, alongside cell-role isolation, sibling validation,
cell-descriptor cache refusal and the independent full-SVG regression.

The subsequent independent-review fix is verified with the three natural-role
regressions above plus bounded Matrix natural-sizing, selected-frame/cache,
shared-axis, and baseline-parity neighbors. Representative commands are:

```text
cargo test --locked --offline --lib matrix_natural_roots_
cargo test --locked --offline --lib scatter_natural_roots_consume_the_selected_ordinary_frame
cargo test --locked --offline --lib matrix_content_roots_compose_the_native_hue_guide
cargo test --locked --offline --lib private_width_and_automatic_roots_use_content
cargo test --locked --offline --lib plots::matrix::axis_tests
cargo test --locked --offline --lib shared_multiline_titles_use_measured_clearance
cargo test --locked --offline --test composition faceted_shared_axis_titles_preserve_explicit_multiline_text
cargo test --locked --offline --test marginal-matrix default_matrix_full_svg_bytes_match_the_independent_committed_baseline
just lint
just fmt-check
git diff --check
```

Constructor/edit and exact arithmetic tests cover all fifteen valid side choices,
all fields, complete sibling validation, optional absence, zero relationships,
selected font metrics, composite rounding, odd-anchor ties, and width/length
extremes. Frozen-axis, standard-panel, preset, whole-theme and child-cache tests
cover admission and geometry. Matrix tests cover actual side replay, scientific
source retention, complete cache matching, fatal refusal and the last call.

Scatter and Bar-family snapshots pass unchanged. Shared-facet multiline titles,
independent frame/tick paint, no-chrome required text, Plot/Canvas theme ownership,
and public tick-edge clearance tests pass. Strict compiler validation is
`just lint` (`cargo clippy --locked --offline --all-targets -- -D warnings`),
with formatting and diff whitespace checks. The targeted Matrix structural
snapshot is supplemented by exact full-document parity; no full-suite command
or snapshot repinning is used.
