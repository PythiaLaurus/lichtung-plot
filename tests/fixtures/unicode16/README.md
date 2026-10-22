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
version is defined by the pinned dependency's
`rustybuzz-0.20.1/src/hb/unicode_norm.rs`; it is not exported, so its
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

## Runtime Boundary

Normalization and line breaking retain one fallible implementation each, with
checked actual input lengths, capacity arithmetic and first-party buffer growth.
There are no accounting callbacks, cumulative visit/byte quotas or raw CSS
font-stack parser. Typed font preferences and operation-local normalization reuse
follow the [font/layout contract](../../../specs/05_LAYOUT_SYSTEM.md).

Rendered text preserves its original bytes; family-name matching alone uses
NFKC-casefold. Line breaking retains collapsed-scalar context for numeric prefixes,
previous nonspace positions and RI parity; grapheme filtering intersects sorted
boundaries without changing legal opportunities. Actual arithmetic, storage,
font-coverage and sealed-census failures remain errors. Resource and external-corpus
publication policies are defined in [development](../../../specs/90_DEVELOPMENT.md).
