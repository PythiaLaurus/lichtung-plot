# T5e Marginal Final-Guide Integration Audit

The remaining integration failure was confined to the two guide-bearing cases
per theme in `tests/marginal_matrix/parity.rs`. All eight actual preserved-text
SVGs were captured and inspected before replacing their fingerprints. The
partition changes only that test and this evidence document; no renderer,
root-sizing, guide, or kernel implementation was changed.

## Concrete Rendering Intent

The final discrete-guide audit prices the complete point-plus-density-fill key,
including its outline. Each Marginal key has an 11px rectangular handle with a
1px data-colored stroke: its painted width is **12px**, not the old nominal
11px. Its painted vertical extent also contributes to entry ascent/descent.
The 3px circular key retains its white 0.2998046875px outline. The complete guide
frame is outward-rounded, and label/title baselines remain fractional instead
of being snapped to whole pixels.

Those changes legitimately affect guide allocation and the parent body's
reprojection. They do not change scientific observations or KDE membership.
Case 2's longer composite title also participates in its guide width. Current
painted frame rectangles in both themes are:

| Case | Guide labels in order | Frame rectangle width × height | Complete 1px frame envelope |
| --- | --- | --- | --- |
| 1 | `Hue`, `B`, `A` | 44.9482421875 × 68.8544921875 | 45.9482421875 × 69.8544921875 |
| 2 | `Group × Hue`, `G 1, B`, `G 2, A` | 94.830078125 × 68.8544921875 | 95.830078125 × 69.8544921875 |

The guides remain to the right of the linked density strip, fully inside the
800×600 root. Title and entry roles remain 12pt and 11pt respectively; central
ticks and axis titles remain 16pt and 17pt. No typography shrinks.

## Eight-Case Scientific Inspection

Case order is ungrouped, hue-only, group-plus-hue, and group-plus-hue with explicit
`x=(0.5,4.0)`, `y=(1.0,5.0)` windows and hidden legend. Both Publication and
Science retain:

- Exactly six central circles, in original source order, representing
  `(1,2)`, `(1.2,2.1)`, `(1.4,2.3)`, `(3,4)`, `(3.2,4.2)`, `(3.4,4.3)`.
  The test independently projects these values from emitted numeric tick
  positions and verifies every circle within two CSS quanta.
- Circle radius 3px, fill alpha 209/255, and white 0.2998046875px outline.
  Publication keeps blue/orange; Science keeps red/blue. The first three
  observations retain the first hue and the last three the second hue.
- Two linked density polygons for the ungrouped case and four for each grouped
  case: one top and one right curve per collective. Every polygon retains all
  80 grid samples plus its two closure records, fill alpha 115/255, and the 1px
  data-colored outline.
- Identical data-grid coordinates across collective curves within each strip,
  monotone grid order, finite vertices, matching closure baselines, and positive
  peaks. An independent public KDE calculation uses the original sample groups
  and grid recovered from SVG scientific ticks; normalized density amplitudes
  agree within 0.03%, allowing only combined tick/grid/vertex quantization error.
  The shared peak reaches 90% of the actual ink-reserved amplitude span within
  two CSS quanta.
- Central/top x alignment, central/right y alignment, and 12px strip gaps.
  Duplicate per-group clip records refer to the same three physical subpanels.
  Every automatic data sample and each density amplitude reserve stays inside
  its strip. The explicit-window case preserves the specified per-dimension
  data-window clipping and has no legend layer.
- Both axis labels, the complete hue/composite guide labels, two point-plus-fill
  guide keys when visible, and the theme-owned grid/frame styles.

All four cases have central y=131.33984375 and height=401.66015625; the top strip
has y=17 and height=102.33984375. Their current horizontal allocations are:

| Case | Central x / width | Right strip x / width |
| --- | --- | --- |
| 0 | 86 / 540.046875 | 638.046875 / 140.953125 |
| 1 | 101 / 489.365234375 | 602.365234375 / 121.8857421875 |
| 2 | 101 / 449.7978515625 | 562.7978515625 / 111.5712890625 |
| 3 | 86 / 540.046875 | 638.046875 / 140.953125 |

## Fingerprints And Historical Attribution

The historical column records the **T5e7 terminal-fit-envelope checkpoint**
immediately before the final complete-guide-key/chrome audit. Earlier T5b2a,
T5e3, density-clearance, and terminal-envelope attribution remains in the test.
Ungrouped and explicit-window cases remain byte-identical to that checkpoint.
Hashes are decimal FNV-1a over the complete SVG bytes, without normalization.

| Theme / case | Historical bytes / hash | Current bytes / hash |
| --- | --- | --- |
| Publication / 0 | 17263 / 6140178027913777248 | 17263 / 6140178027913777248 |
| Publication / 1 | 23762 / 3009713261283530364 | 23474 / 8840269100508290452 |
| Publication / 2 | 23889 / 8255330456500844560 | 23872 / 4692647835872497094 |
| Publication / 3 | 22466 / 6314839169899191404 | 22466 / 6314839169899191404 |
| Science / 0 | 16658 / 753842713645876100 | 16658 / 753842713645876100 |
| Science / 1 | 23206 / 7367574597730145482 | 22918 / 1735915456015330580 |
| Science / 2 | 23333 / 6048523694157485380 | 23316 / 13086465725652164960 |
| Science / 3 | 21858 / 16168846407471475004 | 21858 / 16168846407471475004 |

The fingerprint test now inspects scientific SVG semantics before its byte
comparison and evaluates both themes before reporting a fingerprint mismatch.
For inspection-only capture, set `LICHTUNG_MARGINAL_AUDIT_DIR` to an existing
temporary-work parent or a child directory; the normal test still cleans up its
temporary output. The audited files were retained under
`/tmp/opencode/t5e-marginal-audit/marginal-{Publication,Science}-{0,1,2,3}.svg`.

## Focused Verification

All fourteen focused test functions passed (the fingerprint function audits all
eight SVG cases):

```text
cargo test --locked --offline --test marginal-matrix parity::marginal_full_svg_fingerprints
cargo test --locked --offline --test marginal-matrix contracts::marginal_
cargo test --locked --offline --test marginal-matrix contracts::hue_only_marginal_and_matrix_split_multiple_hues
cargo test --locked --offline --lib plots::marginal::extraction::complete_large_point_and_density_ink_survives_feedback_per_automatic_dimension
```

The commands selected 1, 11, 1, and 1 tests respectively. Strict locked/offline
all-target Clippy passed through `just lint`. No broad suite or commit was run.
