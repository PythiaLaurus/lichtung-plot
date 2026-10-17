# T5e — Private Scientific Layout Closure

T5e is COMPLETE. The five specialized migrations, private three-mode root
engine, all-producer resource/feedback audit and final deletion review are closed.
At this historical checkpoint, public exact defaults remained Plot 640×480 and
Canvas 960×640. The later [public root-sizing cutover](content-derived-root-sizing.md)
installs automatic defaults, consuming width and frozen Output dimensions through
the same private engine; this record describes the earlier implementation gate.

## Delivered Architecture

| Stage | Current measured owner |
| --- | --- |
| Heatmap | Column-local hard requirements, assigned columns before constrained annotations and variable rows; selected cells, categorical ticks and grid share track edges. Common continuous guide; no parity floors or equal-cell adapter. |
| Matrix | Complete declared variable square, variable hard tracks and `12em` soft preference, shared orthogonal axis spans; no fixed cell floor or local stabilization loop. |
| Marginal | Native two-by-two Grid, three linked subpanels and empty corner; top/right preferences derive once from central preference. Retained KDE/source identities and complete density ink; no fixed strips. |
| Beeswarm | Source-ordered deterministic checked tangent packing at assigned numeric width, conservative stroked-glyph disks and occupied category tracks. No bin jitter, capped offsets or height-driven repacking. |
| Waterfall | Original signed deltas/endpoints/row IDs and named base/terminal targets; complete greedy text shelves with independent ascents, descents and exact baselines. No mark-panel floor or clamped label repair. |
| Root | One existing at-most-eight-round driver for automatic, exact-width/content-height and exact-pair requests. Automatic width is chosen once; keyed content height may replace or contract. Exact roots do not resize. |
| Parents | Canvas and owning facets compose native tracks, measured prose/guides and scoped domains before assigned child measurements. Hidden Canvas guides suppress continuous as well as discrete guide content while retaining scientific scales. |
| Final output | One selected immutable scene and retained text/geometry operands, shared by Null, SVG and PNG replay. PNG admission stays terminal-local. |

The current owner census reconciles **25 scientific kinds / 16 shared mark
payloads / two composition parents / 24 Canvas-eligible kinds / 23 generic-facet
kinds**. It distinguishes actual declared topology from selected paint, records,
examined work and measurement calls. Composition exclusions retain their
pre-font validation.

## Closure Audit Corrections

These were concrete gaps exposed while connecting every producer to the complete
measurement contract:

- Content height now uses complete owner/domain/width/gutter/scope/visibility/
  guide inputs and replaces prior measurements, including contraction. Canvas
  installs shared/scoped domains before both height composition and final child
  measurement. Mark-domain requests retain the handles that can resolve actual
  Waterfall root overflow through the same driver.
- Complete legend appearance participates in theme equality and child keys.
  Theme-owned strings are admitted before copying. Local continuous guides retain
  the common complete draw plan, including frame and missing-key outline.
- Discrete key requirements include actual point, line-cap and outlined-fill ink.
  Continuous label clearance uses actual scientific fractions; singleton domains
  retain the prescribed `2*max(Lc,Rc)+2*gap` minimum. One pre-font numeric-label
  producer preserves distinct labels for distinct values.
- Specialized topology and source identities are admitted before materialization.
  Matrix and Marginal inspect complete unclipped automatic mark/density ink;
  explicit windows continue to clip only the permitted data dimensions.
- Address storage is admitted and shared. Feedback/cache/collision/history scans
  price actual examined entries. Fresh payload and retained-reference admission
  remain separate; failures are fatal and rejected work is not refunded.
- Unknown feedback streams grow geometrically, admitting each complete new
  buffer before allocation. This fixes actual quadratic cumulative storage,
  without changing a resource cap or reducing an input fixture.

The final independent read-only review found no remaining concrete implementation
or contract blocker. It explicitly rechecked the singleton guide correction and
geometric feedback growth after the earlier root/source/cache/guide review.

## Verification Record

The required broad gate was invoked as:

```sh
just verify > /tmp/opencode/t5e-verify.log 2>&1
```

Formatting and strict locked/offline all-target Clippy passed. The library run
passed **1,310 active tests, 17 ignored**. Annotation, categorical, composition,
distribution, estimate and Heatmap targets passed. The run stopped at two stale
Interaction expectations caused by complete guide ink.

Per the repository's no-repeat rule, successful suites were not repeated for
localized corrections. Subsequent targets and focused follow-ups complete the
gate coverage as follows:

| Integration target | Tests | Closure disposition |
| --- | ---: | --- |
| annotations | 27 | Broad run passed |
| categorical | 44 | Broad run passed |
| composition | 68 | Broad run passed |
| distributions | 67 | Broad run passed |
| estimates | 38 | Broad run passed |
| heatmap | 42 | Broad run passed |
| interactions | 23 | 21 broad passes; both failures corrected after actual SVG inspection and focused checks with relevant neighbors |
| layout | 20 | Remaining-target run passed |
| lines | 41 | Remaining-target run passed |
| marginal-matrix | 19 | 18 broad passes; grouped guide fingerprints inspected and corrected; all eight parity cases and relevant neighbors pass |
| output | 28 | Remaining-target run passed |
| rendering | 43 | 42 broad passes; original text-child fit address expectation migrated from CanvasGrid to TextPanelContent; focused failure and eight contract neighbors pass |
| statistics | 10 | Remaining-target run passed |
| visual-regression | 13 | Seven broad passes; six failing tests closed through individual snapshot inspection and focused reruns, including the corrected Interaction clip assertion |
| **Total** | **483** | Every integration test covered by successful broad or bounded follow-up execution |

The resource-growth correction subsequently added **three focused passing units**:
two independent feedback-buffer admission tests and the unchanged real Matrix
publication fixture. Its complete 26-test feedback module, affected child-cache,
Canvas admission and root-refusal neighbors also pass.

Final integrated `just fmt-check`, `just lint`, and `git diff --check` pass.
`just docs` passes with rustdoc warnings denied; `just package` passes unpacked
package verification. There was no second successful monolithic `just verify`
invocation; closure is the broad attempt plus the recorded bounded follow-ups.

Logs for the remaining targets are `/tmp/opencode/t5e-remaining-integrations.log`,
`t5e-final-integrations.log` and `t5e-last-integrations.log`. Initial localized
failures remain in those logs; the individual audit documents record the
successful corrections rather than erasing their provenance.

## Release Workloads And Scaling

```sh
cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1
cargo test --locked --offline --release --lib orchestrator::tests::benchmarks::publication_efficiency_measurements -- --ignored --nocapture --test-threads=1
```

The first command passed **nine groups**: significance, three font groups,
inside/outside/shared guides, paragraph search and specialized text search.
Publication failed at the unchanged `matrix4` fixture on the 536,870,912-byte
operation limit; the publication-only retry passed all nine original cases with
their original **three samples**. Thus all **ten release groups** are covered.
Logs: `/tmp/opencode/t5e-release-workloads.log` and
`/tmp/opencode/t5e-publication-efficiency-retry.log`.

The failure was repeated exact one-entry feedback-vector growth. At 4,869
references the next 38,960-byte buffer exceeded 536,869,159 already charged bytes.
For independently tested 16,384 reference slots, the former buffer sequence
requires 1,073,807,360 cumulative bytes; geometric buffers require **262,136**.
Fresh payloads remain additionally admitted. Earlier buffers and rejected work
remain charged.

| Publication case | Records | Examined visits | Cumulative admitted bytes | Calls | Cache entries | Topology | End-to-end min/max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `caption200_1para` | 28,841 | 191,126 | 29,619,639 | 390 | 114 | 114 | 88.798 / 91.270 |
| `caption200_3para` | 28,542 | 190,184 | 29,870,012 | 390 | 118 | 114 | 89.943 / 90.627 |
| `matrix4` | 71,744 | 172,224 | 31,573,771 | 1,061 | 106 | 1,230 | 76.099 / 76.603 |

Counters and SVG/PNG/decoded-pixel hashes are stable across publication samples.
Long captions and legal-break fixtures remain complete. The separate paragraph
fixture retains 22 lines / 707 glyphs and 44 exact shapes; cached repeats perform
zero shaping. Specialized long Wrap/Elide, shared-prefix typed refusal and
adequate/tight Matrix fixtures pass without source reduction.

These are optimized, instrumented, single-thread observations with no warmup and
uncontrolled OS caches/scheduling. Cumulative admitted bytes are not peak RSS;
the storage reduction is not itself a claim of end-to-end CPU improvement.
Beeswarm's independent increasing collision-work/refusal evidence remains in
`docs/t5e4-tangent-packing.md`.

## Supporting Evidence And Next Boundary

The complete phase diff from the T5d closure (`96fa559`) touches **125 files**:
88 under `src/` (including Rust unit-test modules and inline tests), 12 integration/
benchmark source files, 14 SVG snapshots, and 11 specification/evidence documents.
This is the cumulative T5e0–T5e8 migration, not 125 independent new features.
The final root/admission/guide corrections share private API migrations and are
integrated with their callers and intentional output expectations in one closure
commit; the five specialized migrations retain their earlier logical commits.

- `docs/t5e0-specialized-root-census.md`: historical starting inventory and current
  owner handoff.
- `docs/t5e7-producer-census.md`: actual family/parent/topology/admission/cache/
  feedback/replay evidence and coverage boundaries.
- `docs/t5e-root-audit.md`: root coordination, contraction, Hidden visibility,
  Waterfall overflow and actual measurement-versus-capture traces.
- `docs/t5e7-guide-cache-closure.md`: guide geometry, numeric grammar, complete
  theme keys and historical fingerprint attribution.
- `docs/t5e-marginal-audit.md`: independently checked points/KDEs/linked spans.
- `docs/t5e-snapshot-closure.md`: individually inspected SVG changes and successful
  bounded follow-ups.

T6 entry still requires the recorded DF1/DF2/remaining DF5/DF7 decisions. This
closure supplies the private measured implementation and evidence; it does not
start the public API migration or admit a deferred feature.
