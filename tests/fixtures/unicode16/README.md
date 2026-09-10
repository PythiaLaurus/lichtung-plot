# Offline Unicode 16 Profile

## Source And Provenance

The source is the official Unicode 16.0.0 UCD archive, obtained externally from
<https://www.unicode.org/Public/zipped/16.0.0/UCD.zip>. No project code downloads
it or provisions assets. The retained tables use the pinned official data,
not host Unicode tables.

Archive SHA-256:
`c86dd81f2b14a43b0cc064aa5f89aa7241386801e35c59c7984e579832634eb2`.

`sources.json` records the SHA-256 of every selected original archive
member, before comment stripping or other transformation. These checksums were
computed from the supplied official archive, not a separately signed digest.
The ignored Rust test in `src/font/unicode16/regeneration.rs` regenerates the
runtime tables from an explicitly supplied, locally extracted UCD directory.
It validates all 11 original members against this manifest before generating or
writing any output. The manifest reader accepts only the checked-in canonical
JSON format, allowlisted member names, unique entries, and lowercase SHA-256
values; it rejects missing members, unexpected paths, escapes, and malformed data.
No additional dependency, executable target, script, or network access is needed.

## Offline Regeneration

Extract the already acquired, hash-verified archive locally before running the
test. Extraction is not performed by project code. Use the repository's Rust
toolchain, including rustfmt, which the test invokes to format the three tables.
The default mode compares all output bytes without modifying repository files:

```sh
LICHTUNG_UNICODE16_SOURCE_DIR=/absolute/local/UCD-16.0.0 \
LICHTUNG_UNICODE16_ARCHIVE=/absolute/local/UCD-16.0.0.zip \
cargo test --locked --offline --lib font::unicode16::regeneration::regenerate_unicode16 -- --ignored --exact --nocapture
```

`LICHTUNG_UNICODE16_SOURCE_DIR` is mandatory; there is no discovery or download
fallback. `LICHTUNG_UNICODE16_ARCHIVE` is optional and, when supplied, verifies
the original archive's pinned SHA-256 in addition to the mandatory member checks.
Directory-only input validates the extracted members, not an archive container.

Leave `LICHTUNG_UNICODE16_WRITE` unset for checking. To intentionally overwrite
the generated outputs, set it to exactly `1` for that invocation:

```sh
LICHTUNG_UNICODE16_SOURCE_DIR=/absolute/local/UCD-16.0.0 \
LICHTUNG_UNICODE16_WRITE=1 \
cargo test --locked --offline --lib font::unicode16::regeneration::regenerate_unicode16 -- --ignored --exact --nocapture
```

Any other value is rejected. Both modes validate every source before processing,
then construct all outputs in memory. The output set is exactly five files: three
Rust tables, the original ReadMe, and `SHA256SUMS`. Neither mode emits corpora,
compressed or otherwise. Output hashes use deterministic path order. The test
does not overwrite this README, `sources.json`, or the retained Unicode license.
Normal test runs leave regeneration and full-corpus conformance ignored.

## Generated Data

`src/font/unicode16/casefold_data.rs` comes from `NFKC_CF` in
`DerivedNormalizationProps.txt`. `nfc_data.rs` uses canonical decomposition and
combining classes from `UnicodeData.txt`, and `Full_Composition_Exclusion` from
`DerivedNormalizationProps.txt`. Hangul decomposition/composition is algorithmic.

`src/layout/unicode16/data.rs` uses `LineBreak.txt`, `EastAsianWidth.txt` (F, W,
H), UnicodeData categories Mn/Mc for LB1, Pi/Pf for quotes, and the intersection
of Extended_Pictographic with unassigned code points. Ranges are sorted and
adjacent equal-valued entries are coalesced without changing lookup semantics.

The six complete conformance/property corpora are not retained in the checkout.
The original archive ReadMe is retained verbatim. `SHA256SUMS` covers only the
three generated tables and ReadMe; an active Rust test checks those bytes from
project-relative paths. Default tests are self-contained: literal, independently
expected NFC/casefold, bidi, grapheme, line-break, and ASCII-contraction cases
complement the existing rule regressions, admission tests, profile version checks,
and semantic table checksums. They do not claim full official coverage.

Full-corpus tests are explicitly ignored and require
`LICHTUNG_UNICODE16_SOURCE_DIR` pointing to original official raw UCD members,
including the `auxiliary/` subdirectory. Each consumer verifies the source's raw
SHA-256 against `sources.json` before parsing. Comments, blank lines, and surrounding
whitespace are stripped only in memory; source order, expected values, and
`@` section directives are preserved. Explicit invocation with an unset variable,
missing member, or mismatched hash fails rather than skipping. There is no source
discovery, dependency-cache lookup, network access, or compressed corpus fallback.

Run the five Unicode corpus tests and the separate contraction corpus test with:

```sh
LICHTUNG_UNICODE16_SOURCE_DIR=/absolute/local/UCD-16.0.0 \
cargo test --locked --offline --lib ::conformance::official_ -- --ignored
LICHTUNG_UNICODE16_SOURCE_DIR=/absolute/local/UCD-16.0.0 \
cargo test --locked --offline --lib contracted_line_break_analysis_matches_every_official_case_and_expanded_ascii_runs -- --ignored
```

The Unicode data and all derived tables are covered by the accompanying
`tests/fixtures/unicode16/LICENSE` (Unicode License V3). The original source
notice is Copyright 2024 Unicode, Inc.; see the retained ReadMe and
<https://www.unicode.org/terms_of_use.html>.

## Runtime Inventory

| Consumer | Package/Data | Actual Unicode |
| --- | --- | --- |
| Extended grapheme clusters | unicode-segmentation 1.12.0 | 16.0.0 |
| Script and script extensions | unicode-script 0.5.7 | 16.0.0 |
| Bidi classes and UAX #9 | unicode-bidi 0.3.18 | 16.0.0 |
| General category and emoji, including shaping | unicode-properties 0.1.3 | 16.0.0 |
| Shaping combining classes | unicode-ccc 0.4.0 | 16.0.0 |
| Shaping bidi mirroring | unicode-bidi-mirroring 0.4.0 | 16.0.0 |
| Shaping normalization | rustybuzz 0.20.1 internal unicode_norm.rs | 16.0.0 |
| Family-name NFC and NFKC_CF | Generated project tables | 16.0.0 |
| Default UAX #14 | Generated project tables | 16.0.0 |

Existing Rust version tests assert every exported Unicode version in this table,
including the tables shared with rustybuzz's transitive dependencies. The exact
package constraints and Cargo.lock retain the audited dependency profile;
no separate graph-audit script ships. rustybuzz's private normalization
version was inspected at `src/hb/unicode_norm.rs`; it is not exported, so its
exact 0.20.1 package pin and Cargo.lock checksum protect that audited source.
rustybuzz also has upstream assertions for all its Unicode tables. In particular,
unicode-properties 0.1.4 contains Unicode 17 despite the compatible package
version; the direct =0.1.3 constraint prevents rustybuzz resolving it again.
The ccc and mirroring dev dependencies let tests assert the same resolved tables,
not substitute test implementations.

unicode-ident belongs to host proc-macro parsing, not rendered text, and is
excluded from the target-runtime closure. The renderer does not use Rust's host
Unicode normalization or category predicates for these generated properties.

fontdb 0.23.0 retains its existing filesystem/system-font/memmap capabilities for
canonical discovery; ttf-parser 0.25.1 remains the metadata/coverage parser;
rustybuzz remains the shaping engine. resvg/usvg 0.47.0 no longer enable text,
system-fonts, or memmap-fonts: canonical text is already outlined before SVG/PNG
rendering. Removing those features compiles the library and its unit tests and
removes unicode-vo 0.1.0 and the usvg font engine edges from Cargo.lock. This
change did not remove fontdb capabilities or change canonical discovery.

## Conformance Coverage

| Official Input | Executed Coverage |
| --- | --- |
| GraphemeBreakTest | 1,093 extended-grapheme cases |
| LineBreakTest | 16,672 cases, directly against default linebreaks |
| NormalizationTest | 19,965 rows, all five required NFC comparisons (99,825), plus the Part1 unlisted-scalar identity invariant |
| DerivedNormalizationProps NFKC_CF | 10,554 explicitly mapped scalars plus every other scalar's default identity mapping |
| BidiCharacterTest | 91,707 cases, paragraph levels, resolved levels, and visual scalar order |
| BidiTest | 490,846 class rows, all 770,241 selected paragraph-direction runs, levels and visual order |
| LineBreakTest through ASCII contraction | 16,672 rows at each of two ASCII expansion lengths (33,344 comparisons) |

These counts apply to the opt-in full-corpus tests, not default tests. Once invoked,
all corpus cases are scalar-representable; none are skipped. Bidi
`x` entries are the official X9 exclusions, not suppressed failures. L1 is applied
before comparing levels and L2 visual ordering, with original scalar indices
preserved after X9 removal. Class fixtures use non-bracket representatives;
character fixtures supply the actual bracket characters. Normalization tests do
not mislabel NFKC columns as NFKC-casefold: only NFC invariants and the official
NFKC_CF property are checked. Family-name composition across mapping boundaries
also retains the `A + COMBINING RING -> small a with ring` regression.

Full official NFC and UAX #14 fixtures passed the initial algorithms without
weakening expectations. Subsequent review found two UAX #14 defects outside
those fixtures' coverage; the rule-derived regression tests below reproduce
both failures before their fixes. Conformance is not a proof for every input.
Line-break tests certify the direct UAX #14 function, not text wrapping's choice
among legal breaks. Normalization remains family-name-only; rendered text is
not normalized.

### Additional Rule Regressions

The normative source is Unicode 16.0.0 UAX #14 revision 53:
<https://www.unicode.org/reports/tr14/tr14-53.html#LB25> and
<https://www.unicode.org/reports/tr14/tr14-53.html#LB8a>.
These expectations are derived from the published rules, not generated by the
implementation or misrepresented as original official fixture cases.

- `1}2`: LB13 protects `1 x }`; LB25's `NU (SY | IS)* x NU`
  does not include an optional CL/CP. LB31 allows `} / 2`, at UTF-8 byte 2.
  The numeric-prefix helper previously stripped a closing mark even when the
  right-hand class was NU; it now does so only for the PO/PR suffix rules.
  `1}2%` also breaks at byte 2, while `1}%` and `1,2` remain unbroken.
- `U+4E00 U+200D U+0308 U+4E01`: LB8a protects `ZWJ x CM`.
  LB9 then ignores the attached ZWJ and CM for subsequent rules, so LB31 allows
  the effective `ID / ID`, at UTF-8 byte 8. The implementation incorrectly
  remembered any ZWJ inside the collapsed sequence. It now tracks only its
  final raw scalar. The reversed mark/ZWJ order and `ID ZWJ ID` remain unbroken.

Both regression tests were run and failed against the previous implementation.
Neither correction changes official fixture inputs or expected outputs.

## Runtime Admission

PF4's accepted coarse contract below replaces the prior synthetic prices. Code
is IN PROGRESS, not yet primary-verified; final comparison review and integrated
verification remain pending. Prior test observations are retained as historical
evidence, not proof that the current PF4 work has closed its gates.

Normalization and line breaking expose one fallible implementation each, using
borrowed root-budget callbacks and existing project-owned errors:

```rust
fn nfkc_casefold(
    value: &str,
    claim: &mut impl FnMut(u64, u64) -> Result<(), FontError>,
) -> Result<String, FontError>;

fn linebreaks(
    value: &str,
    claim: &mut impl FnMut(u64, u64) -> Result<(), TextPlanError>,
) -> Result<Vec<(usize, BreakOpportunity)>, TextPlanError>;
```

Callback arguments are newly requested storage bytes and actual processing visits.
Canonical family-name validation, UTF-16 decoding, normalization, text line breaking,
and grapheme filtering share ONE cumulative operation/root font+text ledger with
an inclusive 1,000,000 processing-visit limit. Font resources, text resources, and
clones retain the same consumption; no callback resets counters, creates a local
quota, swallows a refusal, or retries without limits.

Independent admissions remain 1,000,000 each for retained planning records,
measurement/intrinsic calls, cache entries, prepared topology slots, and discovery
entries, plus 536,870,912 (512 MiB) cumulative font/source/scratch bytes before
read or retention. Unicode scratch buffers consume bytes, not candidate slots.
Processing, records, calls, and cache remain the four planning-counter classes;
sharing font/text processing does not add a fifth class. Counters do not bound
CPU instructions, wall time, RSS, allocator overhead, or third-party allocations.

Prior pair-resolution evidence charged 20 resolution units, 570 normalization
units for `Test Sans`, and 40 cache-key/retention/copy units with one normalization
shared across weights. Those figures belong to the superseded two-work-ledger
policy, not current charges or measured CPU operations.

Input lengths are checked against the u32 representation before scanning.
Each actual decoding/segmentation pass is admitted before it starts. Text's
outer pass counts were reduced when those charges moved into the fallible
helpers, rather than double-counting the same pass. Borrowed source text is not
charged as a new allocation and is never normalized for rendering.

All first-party Unicode buffers use checked size/capacity arithmetic and
`try_reserve_exact` before writes. Incremental geometric growth charges the
complete new allocation (conservatively accounting for coexistence with the
old buffer) before reserve. Copying and hashing have no extra processing price;
actual storage still requires admission. Allocator failure returns `Allocation`; arithmetic overflow
returns `Arithmetic`; callback errors retain their original typed classification.
A refusal is terminal, so successful earlier claims are not rolled back.

### Normalization Work

Fixed Unicode-table binary searches carry no per-comparison processing price.
Normalization admits actual scalar-pass inputs and decomposition nodes, including
algorithmic Hangul expansion; output storage is admitted separately. Variable
lookups count actual examined elements, not unused list tails. NFC ordering guards
each actual preceding comparison in its variable/quadratic scan, not a fixed
allowance or extra swap price. Recomposition stays in-place in admitted storage;
the final UTF-8 string grows with checked byte admission and fallible reservation.
Canonical UTF-16 validation and bounded dependency decoding admit their actual
input before processing. These guards are not a third-party CPU/OOM guarantee.

### Line-Break Work

UAX #14 builds only the collapsed scalar buffer, not an additional raw-scalar
vector. It caches the previous non-space index, numeric-prefix state, and RI
parity in one linear pass. All later backward-context queries are O(1), replacing
the earlier repeated prefix scans. Actual scalar inputs to variable passes and
actual boundary visits consume the shared ledger; fixed table searches and rule
cascades have no separate price. The prior `64` units per input scalar, `8` per
collapsed scalar, and `256` per boundary were synthetic allowances, not current
policy. Prior property-depth tests supported those old allowances only. Break
offsets, mandatory opportunities, and LB25/LB8a semantics remain unchanged.
Grapheme filtering uses an admitted linear intersection of sorted boundaries;
storage remains independently admitted before retention.

### Refusal Evidence

Prior admission tests exercised the same fallible implementations as conformance.
The following records remain historical evidence; PF4 must revalidate refusal
boundaries in actual visits/bytes rather than preserve synthetic scores:

- Input refusal returns on the first callback, before decoding or allocation.
- Injected storage refusal leaves destination strings/vectors empty with zero
  capacity, before reserve or push.
- U+FDFA expansion and cross-scalar NFC exercise all intermediate buffers at
  exact byte/work limits, and reject either limit minus one.
- A 1,200-mark descending-class input exhausts root ordering work rather than
  completing quadratic processing.
- Long spaces, numeric separators, and RI sequences have linear admitted work.
- Overflow rejects before reservation; callback error variants are preserved.
- Cloned root resources share consumed work, and text helper tests exhaust the
  actual shared work and byte ledgers to demonstrate caller error propagation.

Official conformance cases run with fresh, bounded production-limit root
resources per independent normalization or line-break operation. Infallible
convenience functions and unlimited accounting probes exist only under
`cfg(test)`; there is no infallible production wrapper or alternate runtime path.

### Operation-Local Reuse

The font operation retains normalized stack names once per parsed stack, sharing
them across requested weights. `CanonicalFontCore` also retains candidate
metadata by structural stack and weight for incremental requests outside the
sealed census. Those requests probe only previously unseen scalars; successful
selections accumulate in first-observation order. Repeated labels therefore do
not repeat NFKC_CF, candidate discovery/sorting, or existing glyph probes.
Returned provenance still uses the requesting ordinal. The sealed census path
retains its existing complete pair results and rejects late resolution.

`PreparedFontEnvironment::parse_family_stack` supplies a raw-source parser cache
for text callers. It returns admitted copies of the retained parsed stack; the
free parser remains the uncached primitive used during initial census parsing.
Variable cache lookups charge actual examined entries in the shared processing
ledger. Retained keys/payloads and returned copies consume record/byte admission,
without extra hash/copy/materialization prices. Environment clones share the core
and its caches; nothing is global, and warm hits do not reset or bypass limits.

Prior regression coverage included 1,000 repeated requests after pre-consuming
900,000 synthetic root work units, with independent parser/normalization/candidate/
probe counters. That score is historical, not a current processing-visit claim.
Adding a space probes just that new painting scalar; adding a mandatory newline
does not probe a glyph. Changing weight builds its candidate metadata once but
reuses normalized names. A separate sealed-census regression verifies two pair
resolutions, one stack normalization, and exactly the expected scalar probes.
