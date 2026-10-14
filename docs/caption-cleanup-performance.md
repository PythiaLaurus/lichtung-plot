# Complete 2,048-word caption after accounting removal

## Direct result

The original two-observation figure, with all 2,048 occurrences of `Measured `
and an explicitly requested 1600×8000 root, was measured again. The successful
baseline is `8c6e011` (quota refusal disabled, accounting retained); the current
renderer is `6fd2d0a` (accounting removed, feedback comparison cleanup completed).

| Fresh-root in-memory route | Baseline median (ms) | Current median (ms) | Difference of medians |
| --- | ---: | ---: | ---: |
| Preserve-text SVG | 103.890180 | 97.812765 | 6.077415 ms / 5.849845% lower |
| Outlined SVG | 231.388126 | 223.968381 | 7.419745 ms / 3.206623% lower |

Each median pools **22 measured renders per version/format**, not paired
per-render differences. Per-process medians also agree in direction: preserve
103.86/103.92 ms versus 97.44/97.87 ms; outlined 232.34/230.56 ms versus
224.09/223.85 ms. These are results for this input and host, not a universal
percentage or the old October 7 experimental result.

All measured renders report **2,403 shaping calls**. Preserve-text outputs verify
the complete ordered 2,048-word vector, not a shortened caption. Baseline/current
artifacts were compared byte-for-byte for both formats and match; all four
sessions report identical output digests:

| Format | Observed bytes | SHA-256 |
| --- | ---: | --- |
| Preserve | 113,880 | `2de3a51ba96dc055f3d409aed790fb1f4e85747eb10f8526b708d6548e13d644` |
| Outline | 7,115,390 | `f7f5ae6d469e7f779ea74ee4c16c65a0037f5bc23dc35da18abd8c526e53f760` |

These sizes/hashes are observations in this experiment, not permanent tests of
future renderer revisions. [Raw samples](caption-cleanup-measurements.csv)
include every measured render; warmups are excluded.

## What actually takes time

A separate current-only diagnostic build ran five complete preserve-text roots,
using existing stage observations and five scoped wall timers. Its medians are:

| Scope | Median accumulated wall time (ms) | Calls per root |
| --- | ---: | ---: |
| Font discovery / retained database preparation | 18.816586 | 1 |
| Text intrinsic-width queries | 31.206606 | 114 |
| Text reflow | 25.178770 | 10 |
| Complete `prepare_line` | 68.251885 | 2,403 |
| The `rustybuzz::shape` call itself | 14.268929 | 2,403 |

**These scopes overlap and must not be added.** Intrinsic/reflow queries call
line preparation, which includes the shaper call. Existing coarse observations
report preparation including fonts at about 65.80 ms and root measurement at
29.70 ms (medians); capture/replay are each below 1 ms in this diagnostic.

The evidence identifies line preparation and intrinsic/reflow work as major
contributors. Raw Rustybuzz execution is only part of line preparation; the rest
includes bidi/run selection, font metrics, exact advances/ink and retained glyph
payload construction. Font preparation also remains material. Source inspection
confirms that uncached hard-width computation prepares each of the 2,048 legal
word segments. It would be incorrect to assign all remaining time to locks, the
shaper or allocation alone.

This is scoped **wall-time attribution**, not a CPU sampling or allocator
profile. `perf` is not available on this host. No new cache, shortened source,
font substitution, line-selection change or rendering optimization was made.

## Peak process RSS

GNU `/usr/bin/time` measured three additional fresh, single-render processes per
version/format, with zero warmups. The complete process includes the test harness,
fonts and output buffers:

| Route | Baseline min / median / max (KiB) | Current min / median / max (KiB) |
| --- | ---: | ---: |
| Preserve | 48,756 / 49,004 / 49,220 | 47,780 / 47,832 / 47,924 |
| Outline | 65,612 / 65,672 / 65,856 | 64,408 / 64,620 / 64,900 |

The current processes are roughly 1 MiB lower here. Three samples do not establish
a general memory-saving percentage, and these are not live-owned-payload or
cumulative-allocation measurements. The earlier nine-case publication study's
overlapping RSS ranges remain a separate valid observation.

## Method and reproducibility

Both isolated worktrees use the same temporary driver
`/tmp/opencode/caption-study.rs`, attached as a test-only child of
`src/orchestrator/output.rs`. They declare a temporary diagnostic-only
`caption-profile` feature; no experimental module, feature or benchmark test was
added to the main checkout. Driver SHA-256 is
`42d664550a540a5b18f125722ff66bc09a043155f3d4ff829742b59019aea41a`.

Fixture: `Plot::scatter` with x/y `[0.0, 1.0]`, inferred x/y labels, caption
`"Measured ".repeat(2048)`, `.size(1600, 8000)`, unchanged built-in defaults.
Construction is outside timing. Timing covers `render_scene`, selected output
byte retrieval/copy and destruction of the local scene, matching the old
in-memory study's boundary. File publication, source-word checks, hashes and
artifact comparison are outside timing. Outlined samples also start from fresh
roots. No PNG timing is claimed here.

Primary optimized test builds have existing test counters but no new scoped
timers and no active stage observation session. Each process executes two
warmups then eleven measured renders per format. Four sequential processes run
baseline-A, current-A, current-B, baseline-B; compilation is completed beforehand.
OS file caches, allocator state and scheduling remain uncontrolled. The separate
five-run diagnostic includes new timers/active observations and is not pooled
into primary timings. It includes its first run rather than discarding it.

Host: Linux/WSL2 `6.18.33.2-microsoft-standard-WSL2`, x86_64; Rust 1.96.0
(`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`), LLVM 22.1.2, the same locked
dependencies and local fonts. Both primary worktrees passed strict all-target
Clippy; the scoped build passed strict all-target Clippy with its temporary
feature enabled. No network, asset provisioning or dependency installation was
used.

In each worktree:

```sh
cargo test --locked --offline --release --lib caption_timing --no-run
```

Run the reported executable with:

```text
--exact orchestrator::output::caption_measurement::caption_timing --ignored --nocapture --test-threads=1
```

The temporary driver defaults to the stated warmup/sample counts and both formats.
RSS uses `CAPTION_ROUNDS=1 CAPTION_WARMUPS=0 CAPTION_FORMATS=preserve` (or `outline`)
under `/usr/bin/time -f 'peak_rss_kib,%M'`. For scoped observations, current-only:

```sh
cargo clippy --locked --offline --all-targets --features caption-profile -- -D warnings
cargo test --locked --offline --release --lib --features caption-profile caption_diagnostics -- --ignored --nocapture --test-threads=1
```

Temporary patches remain in `/tmp/opencode/accounting-cleanup-{baseline,current}`;
raw timing, RSS and diagnostic logs are `/tmp/opencode/caption-*.log` and
`caption-rss-*.csv`. The source/CSV summary helper also passed strict Clippy.
Further feature implementation remains paused for the requested test-code audit.
