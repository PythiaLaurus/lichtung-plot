# Scientific Foundation Closure

T6 is COMPLETE. The verified code baseline is `ddf0fbd`, after the focused guide
and lifecycle corrections in `4d058e2` and `ddf0fbd`. This receipt and the final
status reconciliation are documentation-only changes to that baseline.

## Current Contract And Audit

The renderer has one current public construction model: complete immutable Theme
and its eight components, eighteen independent text roles, opaque data-color
scales, exact element paints, independent heading instructions, automatic/width/
exact roots, NumericFormat, TickGeneration, LegendComposition and explicit data
alpha. `scientific-contract-reconciliation.md` and spec04's current-owner overlay
attribute the admitted extensions without changing historical property counts.

The actual census remains **25 scientific families, 24 Canvas-eligible families
and 23 generic-facet families**. Matrix retains its native parent and Marginal
retains its composition restrictions. Alpha has exactly **17 point, 17 line and
15 fill attachments**. All 25 exact families have Legend capability; erasure
retains the configured inputs without granting exact-family setters to Any.

The current external inventory, all-family lifecycle, numeric, section, tick and
alpha contracts cover those owners. In particular, the full gate executes the
147 mapped family/root publications and the 60 alpha owner/root/cache
publications described in `alpha-output-closure.md`. The latter contract compares
preserved text, outlined SVG and PNG. Source-drop/capture contracts retain actual
scientific members, original source identities and independently owned outlines/
chrome. One selected capture and final Null validation consume retained operands;
publication does not prepare scientific inputs, fonts, text or layout.

Source/export review confirms that only `kind` and display `stats` are public
modules. Replaced Theme selectors, raw font constants, old color-scale carriers,
tuple/floating-opacity attachments, NumericLabelFormat, TickInterval, exclusive
guide descriptors/kinds and duplicate continuous-guide frames are absent from
current APIs and runtime routes. Old names in negative doctests demonstrate
rejection. Private `discrete` field/module names carry the sole PreparedGuide;
they do not select a legacy renderer. Alpha declarations share its frame, finite
measurement, cache, font census and frozen draw plan with symbols and color.

Hidden/Undeclared alpha retains complete scientific identity independently of
explanatory visibility. Native Shared contributors retain prepared final color
presentation while suppressing local placement; actual absence yields no
descriptor. The final corrections are recorded in
`prepared-guide-regression-closure.md`, including the initial eight library
failures and subsequent integration failures. Their focused checks passed before
the successful complete gate; no root bound, cycle policy or complete-scene
golden was relaxed or repinned.

## Complete Verification

On Linux x86_64 / WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Rust
`1.96.0 (ac68faa20 2026-05-25)` and Cargo `1.96.0`:

```sh
just verify
```

The complete command passed at `ddf0fbd`. Log:
`/tmp/opencode/scientific-foundation-verify-ddf0fbd.log`.

| Gate | Actual result |
| --- | --- |
| Formatting | Passed |
| Locked/offline all-target Clippy, warnings denied | Passed |
| Library discovery/execution | 671 discovered: 654 passed, 17 ignored; 105.49 s |
| Integration targets | All 481 passed, breakdown below |
| Example targets | All eight compiled and passed their test targets |
| Public/compile-fail doctests | All 146 passed |
| Warning-denied API documentation | Passed |
| Offline unpacked package verification | Passed; 428 files, 9.5 MiB / 2.3 MiB compressed at the verified baseline |

| Integration target | Passed |
| --- | ---: |
| annotations | 27 |
| categorical | 43 |
| composition | 72 |
| distributions | 67 |
| estimates | 38 |
| heatmap | 43 |
| interactions | 24 |
| layout | 30 |
| lines | 43 |
| marginal-matrix | 20 |
| output | 21 |
| rendering | 43 |
| statistics | 10 |
| **Total** | **481** |

The unit budget is unchanged at **671 including 17 ignored**. Replacements are
documented in their owning leaves; the closure adds no test or test-only carrier.

After adding this closure documentation, `just package` also passed unpacked
offline verification with all 429 files included (9.5 MiB / 2.3 MiB compressed).

Examples05/06/07/08 also ran through their actual public terminals with
`cargo run --locked --offline --example <name>`:

- Numeric formats: 1200-pixel exact width and measured height (1469 in this run).
- Separate/local and Shared guide examples: each exactly 1600x1000.
- Independent tick targets/maxima: exactly 1200x600.
- Observation and keyed-series alpha: exactly 1600x800, preserved SVG, outlined
  SVG and PNG from one Output.

These are observed successful publications, not new automatic-size goldens.

## Actual Release Execution And Peak RSS

Release compilation completed first (24.92 s), outside the measurement:

```sh
cargo test --release --locked --offline --lib orchestrator::tests::benchmarks:: --no-run
/usr/bin/time -v -o /tmp/opencode/scientific-foundation-release-ddf0fbd.time.log \
  cargo test --release --locked --offline --lib orchestrator::tests::benchmarks:: \
  -- --ignored --nocapture --test-threads=1 \
  > /tmp/opencode/scientific-foundation-release-ddf0fbd.log 2>&1
```

All **ten groups passed**, with 661 unrelated tests filtered out: significance,
font observer overhead, synthetic font coverage, system font preparation,
inside guides, outside guides, Shared guides, publication efficiency, paragraph
search and specialized text search. The seven ignored Unicode conformance/
regeneration tests are outside this release filter.

The complete measured command exited 0: **10.86 s wall, 9.23 s user, 1.73 s
system, maximum RSS 69,200 KiB (67.58 MiB)**. The test harness reports 10.79 s.
This is the command-level peak reported by GNU time, not an allocation estimate
or a per-case peak. Samples are optimized and instrumented, single-test-threaded,
with no warmup and uncontrolled OS caches/scheduling. Font observer controls
retain test hooks in both modes; their difference is not a production zero-hook
overhead claim.

### Retained Work And Outputs

The original publication fixtures retain 100 observations, complete 200-word
captions, four-variable Matrix, mixed Canvas and repeated publication. All rows
below have three successful samples. Timings are end-to-end minimum/median ms;
byte sizes are observations of the same-operation equality controls, not goldens.

| Publication case | Min / median ms | Root rounds / captures / Null / producer invocations | Text proposal visits / exact trials | Outlined SVG / PNG bytes |
| --- | ---: | --- | ---: | ---: |
| `caption200_1para` | 87.909 / 89.267 | 2 / 1 / 1 / 1 | 219 / 257 | 668562 / 135864 |
| `caption200_3para` | 87.772 / 89.443 | 2 / 1 / 1 / 1 | 218 / 259 | 667862 / 134825 |
| `matrix4` | 74.988 / 76.672 | 2 / 1 / 1 / 1 | 0 / 19 | 310006 / 173764 |
| `separate10` | 407.163 / 407.700 | 20 / 10 / 10 / 10 | 0 / 130 | Same final artifacts as `reuse10` |
| `reuse10` | 171.721 / 175.842 | 2 / 1 / 1 / 1 | 0 / 13 | Same final artifacts as `separate10` |

Matrix retains all four variables / 100 observations, with three native layout
passes. The separate/reuse controls each perform 20 publications, ten SVG
conversions and ten PNG conversions; reuse performs one retained SVG replay
serialization rather than ten fresh producer operations. The raw SHA256 identities
for separate/reused SVG, PNG and decoded pixels are equal. The raw log records all
32 actual/capture counters, root/guide/gutter/intrinsic/child events, SVG and PNG
bytes, and SVG/PNG/decoded-pixel SHA256 identities. Repeated publication samples
assert unchanged identities; no historical renderer hash is an acceptance target.

The paragraph legal-break fixture retains **22 lines / 707 glyphs**, 99 proposal
visits and 44 exact shapes on first layout, with zero proposals/exact trials/
shaping on cached repeats. Multiline first-layout min/median/max is
2.208/2.222/3.876 ms. Long Heatmap Wrap/Elide, shared-prefix ambiguity refusal,
adequate/tight Matrix and outside-guide starvation retain their complete sources.
Expected refusal samples remain separate from successful samples and assert no
published artifacts. Dense inside-guide work retains all 400 marks.

### Admitted Numeric, Section, Tick And Alpha Workloads

- Numeric sets retain every original member. Decimal 32/32, 1024/32 and
  1024/1024 total/distinct members select precision 4; Scientific 1024/1024
  selects significant precision 5. Three-sample elapsed ranges are respectively
  0.038–0.051, 0.085–0.104, 1.611–1.638 and 1.750–1.796 ms. Label bytes are
  192, 6144, 6144 and 8192. Saved membership and repeat equality are asserted.
- Certified mathematical powers retain independent exact strings for exponents
  0.5, -0.5, 39.5 and the least positive subnormal. The 39.5 case actually refines
  through `[128, 256]` bits, with a 40-byte label and 0.095–0.100 ms conversion;
  working-constant hits/misses are logged separately.
- Dense section sources retain **40 source rows / 20 observed group-hue pairs**.
  Combined is one section/20 keys/20 occurrences; Separate is two sections/nine
  keys/40 occurrences. Local, Facets, PerPlot and Shared all execute three
  interleaved runs at 1200x800, with repeat/cache-disabled publication equality.
  Shared Combined/Separate median end-to-end times are 47.577/42.880 ms and
  median measurement times 4.546/3.494 ms. Each has two root rounds/two guide
  trials/four body checks and 12 child misses, zero hits/replays in this workload.
  Separate's smaller key set is a semantic difference, not an inferred speedup
  for equal logical work.
- Target128 versus Maximum12 retains six observations and the exact 1200x800
  root. Median end-to-end times are 61.842/38.056 ms; measurement medians are
  24.306/1.140 ms. Actual source visits/distinct-list work/metric lookups are
  376/350/381 versus 50/14/45, with 129 versus 17 exact text trials. These are
  examined source counts across planning, not final painted tick counts. Exact
  caller ticks, the hard maximum and source/domain preservation retain their
  independent semantic assertions.
- Alpha Shared/Hidden retains **64 observations per panel, two panels and three
  independent explicit targets**, including zero-valued members, series strokes
  and bands, at exactly 1600x1100. Median end-to-end times are 85.050/83.513 ms;
  measurement medians are 4.574/2.451 ms. Both use two root rounds, one capture,
  one Null validation and one producer invocation. Shared/Hidden perform two/zero
  guide trials, four/zero body checks and 12/eight child misses; both have zero
  hits/replays. Exact text trials are 94/36 and tick-metric lookups 460/272.
  Every capture-stage counter is zero. Each configuration has byte-equal
  preserved SVG, outlined SVG, PNG and decoded pixels across repeats and the
  cache-disabled control. Shared and Hidden intentionally have different guide
  content and are not compared as equal scenes.

These timings describe this revision's actual workloads; no historical speedup,
fee/quota counter or reduced-input baseline is inferred. Existing exact arithmetic,
checked storage, root feedback and guide-trial limits remain the governing bounds.

## Next Boundary

After this closure the user cancelled T7 as a separate technical audit, including
the requirement to backfill a per-key historical property review. Existing T5/T6
evidence remains authoritative; cancellation does not claim that the unperformed
audit or a new full external Unicode run occurred. DF2/DF4/DF5/DF6/DF7/DF8/DF9/DF10
are now discussed directly in the active post-foundation deferred review in
spec91/99, before successor planning. Fully caller-decided legends remain in DF11
after the workspace refactor, polars and ndarray adapters, Python interface and
first release. No additional renderer API is admitted by cancelling the audit.
