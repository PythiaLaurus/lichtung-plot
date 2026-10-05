# Retained font-stack values

## Construction and identity

The font owner defines crate-private `FontFamily`, `GenericFontFamily`, and
`FontStack` in `src/font/canonical.rs`. They are the current production
representation, with private fields and readonly inspection. They are not yet
crate-root exports.

`FontFamily::named(impl Into<String>)` retains one literal family identity. It
rejects empty names, leading or trailing whitespace, and control characters.
Validation does not trim, split punctuation, normalize retained bytes, discover
fonts, or contact a network. The supplied owned string is retained directly.
`name()` returns its original spelling; `generic_family()` returns `None`.

`FontFamily::generic` accepts only `GenericFontFamily::{Serif, SansSerif,
Monospace}`. These preferences use the existing host database mappings. A named
`"sans-serif"`, `"system-ui"`, or `"inherit"` is still a literal named family,
with no generic or CSS-directive semantics.

`FontStack::new` fallibly retains a nonempty ordered sequence of validated
families. It preserves order and duplicates. `families()` lends a readonly slice.
Equality compares complete typed entries and original named bytes, rather than
CSS, normalized names, face selections, or pointer identity.
The ordered payload is immutable and reference-counted. Independently constructed
values retain full semantic equality; sharing the exact immutable payload is a
valid admitted constant-work equality path, not an alternate identity.

For example, these are different stacks:

```rust,ignore
FontStack::new([FontFamily::named("A, B")?])?
FontStack::new([FontFamily::named("A")?, FontFamily::named("B")?])?
FontStack::new([FontFamily::named("sans-serif")?])?
FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)])?
```

The raw CSS parser, parsed-stack cache, parser error variants, and parsing
instrumentation have been removed. There is no raw-stack constructor or
selection-time CSS parsing route.

## Complete role inputs

All eighteen resolved text roles retain their own `FontStack`:

| Owner | Resolved roles |
| --- | --- |
| Root | `title`, `subtitle`, `caption` |
| Axes | `axis_title`, `tick_label` |
| Guides | `legend_title`, `legend_label` |
| Annotations | `significance_label`, `estimate_detail`, `waterfall_total`, `point_layer_label`, `panel_text` |
| Variable-fill contrast | `contrast_light`, `contrast_dark` |
| Composition | `facet_header`, `composition_header`, `text_panel_title`, `body` |

The four composition roles are distinct complete appearances. Facet planning,
Canvas headers and row titles, TextPanel title/body planning, and their text
censuses consume the corresponding retained role. Deferred estimate and
Waterfall censuses use their actual annotation role. The old generic resolved
font carrier and `legacy_text_appearance` reconstruction helper are removed.
Existing geometry metrics keep their own consumers.

Each built-in role starts with the same explicit typed order:

1. Named `Noto Sans`
2. Named `Noto Sans CJK SC`
3. Named `DejaVu Sans`
4. Named `Liberation Sans`
5. Named `Arial`
6. Generic `SansSerif`

The built-in factory constructs those entries directly. The temporarily retained
public `DEFAULT_FONT_FAMILY` and `DEFAULT_FONT_FAMILY_STACK` strings describe
their actual literal name/publication spelling; production does not parse or
dispatch on them. In particular, literal `Noto Sans` never substitutes a whole
operation stack.

## Planning, retention, and publication

Text appearances, `TextPlanInput`, census requirements, prepared/incremental
font pairs, normalized-stack cache identities, shaping-query keys, backend
text/bracket styles, and frozen text styles consume typed values. Cache hashing
includes entry count, named/generic discriminants, name lengths and original
bytes; full equality remains authoritative. Reflow accepts exact stack clones
and rejects a changed profile, even if case folding would select the same face.

Runtime lookup uses fallible, content-priced comparison. It admits the stack
length check, each examined entry, each named-length check, and each UTF-8 byte
before reading/comparing that byte, including the terminating mismatch. This
applies to prepared, incremental and normalized font caches, census deduplication
and ordinal assignment, shaping/presentation cache matching, reflow and bracket
validation, and retained child-theme matching. Resource refusal propagates as
an error; it is never interpreted as a cache miss or retried through a fallback.
Child-cache inputs rely on the admitted complete-theme comparison rather than
duplicating its typography check/storage in a StandardPanelInputs carrier.
The coordinate-intrinsic owner's independent StandardPanelInputs comparison is
fallible and uses the current operation's admitted appearance/stack comparison.

The first shaping-profile hash admits the source, tagged family entries and fixed
operands before computing them. The operation-local cache retains that immutable
hash prefix. Later queries admit exact profile/source verification through an
indexed source lookup, then hash only their presentation/provenance operands.
One verified key is carried through lookup and insertion. Cached profile IDs are
lookup witnesses for these verified immutable inputs, never substitutes for
semantic equality or a hash-collision escape. See
[`font-work-admission.md`](font-work-admission.md) for the unchanged caption
fixture, measured baseline/fix evidence and mutable-input-address regression.

Font matching still uses Unicode-16 NFKC casefold. Candidate order, weights
400/600/700, Normal style/stretch, host generics, base-plus-Mark units, same-byte
retained faces, and glyph/line geometry retain their existing policies.

Operation copies admit entry storage and actual named bytes before reservation
or copying, using the owning prepared-stage/font or generation resources.
Both staged and generation-owned stack copies first admit one known full copy
pass: entry count plus named UTF-8 byte count. Byte/storage admission is separate
from that work charge. Staged copying cannot reserve its entry array or name
buffers when this initial work claim fails.
Census admits that same single copy pass before source formatting or retention,
then consumes a single-use proof holding the immutable stack and its actual
operation resources when copying the font buffers. There is no parser or
synthetic font-stack scalar pass. Source scalar/unit and generated-alphabet work
are separate real consumers and retain their own admission.
Font limits remain 1,000,000 processing visits and 536,870,912 retained bytes.
Budget failure remains fatal before the concrete backend begins. Child-theme
clone admission includes the actual color array and all eighteen font-handle
operations. The immutable entry/name arrays are shared rather than deep-cloned.
Effective significance roles use fallible admitted appearance copies, including
the pre-font census route and the later annotation/mark planners. Matrix's
axis-only theme copy and retained child-theme copies use the same complete
18-role admission helper, with the actual operation resources required before
the standard-library clone retains handles and copies the color buffer. Built-in data-only
construction and test assertions are separate from these operation-time copies.

Private CSS serialization is publication syntax. Frozen plans retain
`stack_source` only as display output; no identity or selection reparses it.
CSS count and materialization are two distinct passes, admitted up front using
entry counts and known UTF-8 byte lengths. No unpriced `chars().count()` prescan
is needed to price either CSS publication or a stack copy.
Ordinary CSS identifier sequences, including the built-in names with single
spaces, preserve their current unquoted spelling. Punctuation, CSS keywords,
and names requiring preserved whitespace are quoted as literals. Quotes and
backslashes are CSS-escaped, then SVG attribute publication applies XML escaping
once. Test-only raw drawing fixtures also carry explicit typed stacks.
Production SVG publication requires resolved text and bracket-label plans;
direct raw SVG token emitters are confined to tests.

The built-in CSS spelling remains byte-for-byte
`Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif`.
Role sizes, paint, weights, leading bits, axes and geometric rounding are not
rederived by this representation. For example, Publication title leading keeps
the existing binary64 `1.3` value and its upward-rounded occupancy.

## Verification coverage

Focused tests exercise literal comma selection versus separate entries, named
keywords versus host generics, ordered/duplicate census pair identity,
construction rejection, exact clone storage and refusal boundaries, reflow
profile mismatch, all built-in role defaults, complete contrast-role selection,
Unicode Mark shaping, lifecycle/clone admission, frozen CSS/XML escaping, dirty
composition caches, and representative preserved SVG snapshots and Chinese
system-font fallback.

### Content-admission boundary evidence

The focused refusal tests exercise the production helpers and their actual
cache/census paths, using zero, exact-minimum and one-under work/storage limits.
The measured/independently counted fixtures are:

| Actual path / fixture | Processing visits at successful boundary |
| --- | ---: |
| `αZ` versus `αY`: first named entry, final UTF-8 byte mismatch | 6 |
| Equal `αZ` plus a generic: includes final generic entry | 7 |
| Prepared `Test Sans` pair lookup | 13 |
| Incremental cached `Test Sans` pair with one retained scalar | 16 |
| Normalized-stack cache hit for `Test Sans` | 14 |
| Complete query hash: source `A`, owner `owner`, named `α, A` | 79 |
| Actual retained-profile text payload hit: source `A`, owner `owner`, shared named `Test Sans` | 52 |
| Retained-theme matching: all eighteen shared immutable stacks | 18 |
| Two-record staged census (`Alpha0`, `Alpha1`): count, first uses, copies and ordinals | 48 |
| Staged copy: named `αX`, named `A, B`, generic Serif | 10 |
| Effective significance copy: named `αX`, named `A, B` | 9 |
| Empty-source census requirement, named `α`: one entry plus two UTF-8 bytes | 3 |
| CSS count/materialization, named `α, A`: two entry/UTF-8 passes | 12 |

Explicit staged-copy storage is `3 * size_of::<FontFamily>() + 7` bytes plus the
retained Arc allocation layout; explicit significance-copy storage is
`2 * size_of::<FontFamily>() + 7` bytes plus that layout. The complete-theme test
checks one-under color-byte/handle-work refusal and all eighteen shared immutable
entry/name buffers. It uses the same helper as Matrix measurement.
No system-font discovery is needed for those clone tests.

For the existing `Test Sans` census fixture, source admission stays three visits
(two scalars and one unit), and its one owned stack copy costs ten:
**13 visits**. The earlier 23-visit assertion counted an imaginary unparsed-stack
pass and has been removed. Copy admission still precedes all source/font-buffer
allocation. For `Alpha0`/`Alpha1`, the **48-visit materialization** boundary covers
two comparison passes at 10 each, two distinct pair-owned copies at 7 each, and
ordinal lookups at 2 and 12 (exact immutable references cost one comparison,
whereas the different literal examines its terminating byte). The two requirements are constructed beforehand at
9 visits each (one 7-visit copy and two source visits), so construction plus
materialization costs **66 visits**, not an extra stack-processing pass.
Likewise the performance/cache tests now count complete hash/content work rather
than pinning hits to only binary-index comparisons. Default sizes, leading,
retained labels and snapshot expectations are not repinned to accommodate this
admission correction.

The corrections add fourteen focused admission/refusal tests. Their successful
boundaries and one-under refusals pass, together with the affected census,
indexed-cache, batch-pair, significance, Matrix, Unicode Mark and dirty-child
regressions. Strict locked/offline all-target Clippy, formatting and diff checks
pass. Scatter, Plot/Canvas root-title and Matrix snapshots and the Chinese
host-font fallback test pass without snapshot updates. These bounded checks
specifically verify the content/copy admission gaps; an earlier broader passing
test count is not evidence for those previously uncovered limits.
The final audit also checks the actual child-cache changed-tick-stack route and
the standalone StandardPanelInputs mismatch route at zero, exact-minimum and
one-under budgets. A typography miss cannot short-circuit those content charges.
