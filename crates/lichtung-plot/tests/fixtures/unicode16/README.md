# Offline Unicode 16 Profile

Paths beginning with `src/` or `tests/` below are relative to this package's root:
`crates/lichtung-plot/` in the workspace, or the independently unpacked package.

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
The runtime tables were generated from all 11 manifest members. Original bytes
were verified before generation; the retained canonical JSON identifies
allowlisted member names and lowercase SHA-256 values. This package retains the
tables and provenance, not a public regeneration or full-corpus test command.

## Offline Regeneration

Independent offline tooling must receive an explicit local UCD directory and
verify all selected raw member bytes against `sources.json` before parsing.
If it also receives an archive, verify the archive digest above; directory-only
input does not certify the archive container. There is no discovery, download,
dependency-cache lookup or asset provisioning in the renderer.

Regeneration must preserve the single runtime implementation and Unicode version.
The derived output set is three Rust tables, the original ReadMe and
`SHA256SUMS`; use deterministic path order and the pinned Rust formatter.
Compare all outputs without writing by default. Asset writes require explicit
authorization and must not recreate full corpora, overwrite `sources.json` or
change the retained license. These requirements describe provenance maintenance,
not an available public executable or a successful current conformance run.

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
three generated tables and ReadMe; a public regression checks those bytes from
package-relative paths. The bounded public normalization/font guards do not
claim full official coverage. Full-corpus activities are not public test targets.
Independent consumers must verify raw hashes before stripping comments or
whitespace, preserve source order, expected values and `@` directives, and fail
on missing or mismatched local input rather than silently skipping.

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

The exact package constraints and Cargo.lock retain the audited dependency profile,
including the tables shared with rustybuzz's transitive dependencies;
no separate graph-audit script ships. rustybuzz's private normalization
version is defined by the pinned dependency's
`rustybuzz-0.20.1/src/hb/unicode_norm.rs`; it is not exported, so its
exact 0.20.1 package pin and Cargo.lock checksum protect that audited source.
rustybuzz also has upstream assertions for all its Unicode tables. In particular,
unicode-properties 0.1.4 contains Unicode 17 despite the compatible package
version; the direct =0.1.3 constraint prevents rustybuzz resolving it again.
The ccc and mirroring constraints pin those same resolved tables, not substitute
test implementations.

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

## Historical Conformance Evidence

| Official Input | Executed Coverage |
| --- | --- |
| GraphemeBreakTest | 1,093 extended-grapheme cases |
| LineBreakTest | 16,672 cases, directly against default linebreaks |
| NormalizationTest | 19,965 rows, all five required NFC comparisons (99,825), plus the Part1 unlisted-scalar identity invariant |
| DerivedNormalizationProps NFKC_CF | 10,554 explicitly mapped scalars plus every other scalar's default identity mapping |
| BidiCharacterTest | 91,707 cases, paragraph levels, resolved levels, and visual scalar order |
| BidiTest | 490,846 class rows, all 770,241 selected paragraph-direction runs, levels and visual order |
| LineBreakTest through ASCII contraction | 16,672 rows at each of two ASCII expansion lengths (33,344 comparisons) |

These are historical opt-in conformance receipts, not current public test
definitions or a new execution claim. All executed corpus cases were
scalar-representable; none were skipped. Bidi
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
follow the [font/layout contract](https://github.com/PythiaLaurus/lichtung-plot/blob/main/specs/05_LAYOUT_SYSTEM.md).

Rendered text preserves its original bytes; family-name matching alone uses
NFKC-casefold. Line breaking retains collapsed-scalar context for numeric prefixes,
previous nonspace positions and RI parity; grapheme filtering intersects sorted
boundaries without changing legal opportunities. Actual arithmetic, storage,
font-coverage and sealed-census failures remain errors under the
[font/layout contract](https://github.com/PythiaLaurus/lichtung-plot/blob/main/specs/05_LAYOUT_SYSTEM.md).
External-corpus provenance and distribution limits are described above.
