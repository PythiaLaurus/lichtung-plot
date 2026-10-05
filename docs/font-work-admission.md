# Reusable operation-local font and text-query work

## Reproduced failure

The unchanged test
`layout::text::tests::ordinary_caption_workloads_render_in_adequate_public_boxes`
failed at the 1,000,000-visit limit on both the clean font-migration commit
`5f2c705` and the concurrent spacing worktree. The clean baseline was reproduced
in a detached temporary worktree with locked/offline Cargo, then that worktree
was removed. The fixture was not reduced: it retains the 144-repeat source, the
200-word single paragraph, the 200-word three-paragraph source, and all 2,048
`Measured` words in the long source/publication case.

The failure is a production work regression, not a reason to increase the cap.
On this host the baseline selected 3, 20, 20 and 228 direct-plan lines. The direct
long-source plan shaped 57,294 scalars/glyphs across 456 exact trials, retaining
all its source. Publication subsequently repeated full-input work for lookup,
insertion, reflow verification and endpoint queries until admission failed.

## Changes and invariants

* A font stack has one immutable, reference-counted ordered payload. Equality
  remains full typed value equality, including original named bytes, order and
  duplicates. A shared payload is an exact immutable-reference fast path, not a
  surrogate semantic identity. Independently constructed values still require
  admitted entry/name-byte comparison.
* Explicit font-buffer copies remain fallible, admitted copies. Their retained
  Arc allocation layout is included in the byte claim, in addition to the entry
  array and actual named bytes. Sharing a stack handle does not copy those
  buffers; it admits its one handle operation without inventing name-copy work.
* Query profiles are indexed by source length, font size, weight, preparation
  policy and admitted source-byte ordering. Binary lookup avoids a quadratic
  scan of unrelated endpoint profiles. Equal-source intervals still compare
  complete typed font stacks, so a literal `A, B` cannot collide with `A`, `B`.
* The first source/font prepass admits and computes its complete shaping hash.
  Its immutable SHA-256 prefix state is retained. Each later query admits exact
  profile verification, then hashes only presentation/provenance operands from
  that state. The verified profile entry is carried through lookup and insertion,
  rather than rehashing/rechecking the entire source for each cache transaction.
* Width/owner variants share an immutable retained source allocation. First
  source/owner copies and Arc layouts are admitted; later source-handle shares
  admit the actual constant operation. No copied buffer or hash pass is hidden.
* After the cache verifies a caller's complete profile, reflow receives borrowed
  references to its original immutable analysis profile. The normal reflow
  validator still runs; exact-reference equality avoids a second whole-source
  scan. No validation bypass flag or budget refund is used.
* Calls, cache entries, shaped output, glyph retention and other actual work
  retain their admission. The visit and byte caps are unchanged. Glyph shaping
  output counts/bytes are still actual engine output, not a reduced estimate.

The mutation regression test reuses a mutable caller String's address and length
while changing `A` to `X`. It must select a different retained profile. This
checks that an original caller address is never trusted as immutable ownership.

## Measured evidence

The opt-in test trace is enabled with `LICHTUNG_FONT_WORK_AUDIT=1`. It reports
actual operation-resource deltas; it does not change resource claims or caps.

| Long-source root query (18,432 bytes) | Baseline visits | Reuse/index correction visits before final source-sharing refinement |
| --- | ---: | ---: |
| Cold natural-width query, including insertion | 240,409 | 221,737 |
| Assigned-width reflow query, including insertion | 178,006 | 103,701 |
| Assigned-width payload-hit lookup | Not completed: baseline exhausted during publication | 18,517 |

For the baseline assigned-width reflow lookup alone the measured cost was
37,125 visits. The correction reduced it to 18,496: one exact source verification
plus its bounded profile/presentation work, with no second full hash.

The production evidence test uses the full public SVG publication path and
counts retained words in the resulting SVG:

| Complete publication | Visits | Retained bytes | Retained planning records | Measurement/intrinsic calls | Cache entries |
| --- | ---: | ---: | ---: | ---: | ---: |
| 200 words, three paragraphs, 800 × 900 | 200,246 | 29,614,797 | 24,042 | 184 | 66 |
| 2,048 words, 1600 × 8000 | 991,968 | 55,296,567 | 234,247 | 1,154 | 476 |

The three-paragraph publication contains all 20 `Measured` words; the long
publication contains all 2,048. The original four-workload test passes unchanged.
The evidence uses the current spacing worktree; broad release closure remains
the coordinator's separate gate, not an excuse to leave the debug failure open.

Useful focused commands:

```sh
just lint
cargo test --locked --offline --lib ordinary_caption_workloads_render_in_adequate_public_boxes
cargo test --locked --offline --lib caption_publication_work_evidence -- --nocapture
cargo test --locked --offline --lib presentation_queries_share_source_profile
cargo test --locked --offline --lib retained_profile_does_not_trust
```

## Feedback-owner handoff

The existing `admitted_text_history_equality_matches_rendered_identity_and_checks_glyphs`
assertion still expects the pre-migration 99 visits. The independent current
calculation is **90**: 32 source bytes + 13 owner bytes + 1 candidate + 1 family
entry + 9 named bytes + 1 line + 1 run + 32 glyph pairs. The removed display-only
stack string accounts for the old additional nine visits. First-glyph mismatch
is **59**, not 68, because the remaining 31 glyph pairs are not examined.

The metadata comparisons and every actual glyph comparison remain intact. Shared
rendered-payload identity remains its existing fast path; independent rendered
plans still compare all retained glyphs. The feedback file is spacing-owned and
is deliberately not edited by this font correction. Its owner should update
the independently derived boundaries (89/90 and 59), not skip comparisons.

The child-cache clone test also needs its owner's corresponding representation
update. On the current Linux build `Theme::admitted_clone` retains **24 color
bytes** and clones **18 immutable font handles**; it does not allocate new font
entry/name buffers. The old fixture's 627-byte/36-font-record expectation describes
the replaced deep-copy implementation. The font-side all-18-role test verifies
the exact color-byte boundary, 18/17 visit success/refusal, preserved values and
shared immutable entry/name buffers. No child-cache source/test file is edited
by this correction.
