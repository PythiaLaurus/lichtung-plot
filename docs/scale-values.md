# Immutable validated scale values

This is historical evidence for the private scale-value migration. The complete
public Theme/font/scale cutover supersedes the temporary public map and selector
spellings described below; see [complete foundations](complete-theme-foundations.md).

## Live representations and owner boundaries

`src/theme/scale_values.rs` defines crate-private `QualitativePalette`,
`AccentPalette`, `CategoricalColorScale`, `ColorStop`, `SequentialGradient`,
`DivergingGradient`, `ColorScales`, `ColorDomain`, `ContinuousColorScale` and their
closed capacity/kind discriminants. These are current retained production values,
with private immutable sequence/choice storage and no mutable collection access.

The current public name **`CategoricalColors`** is retained in
`src/spec/categorical.rs`. That one struct directly owns immutable ordered
`(String, OpaqueColor)` entries. Its public tuple constructor converts RGB once at
the API boundary and calls the same private opaque constructor. There is no
second `CategoricalColorMap` struct, wrapper or alias. The target map name and
public categorical capability/attachment cutover remain B4.

`ResolvedTheme.color_scales` owns its qualitative palette and Heatmap sequential/
diverging defaults. `MarkAppearance.accents` independently owns stable unkeyed,
layer and sign slots. The generic resolved `colors` vector and `PaletteSource`
enum are deleted, with every caller migrated onto its actual categorical or
accent owner. Initially equal curated prefixes are separately constructed values;
editing the qualitative palette cannot change accents.

Runtime palette cache prefixes/results, prepared categorical/accent mappings,
palette-extension candidates and retained Heatmap/Beeswarm mapped colors use
`OpaqueColor`. The numerical OKLab kernel keeps its existing transient channel
arithmetic. Private adaptation to current tuple-based paint inputs is explicit
through `OpaqueColor::as_rgb()`; complete mark roles install their own alpha once.

## Validated construction and inspection

All sequence constructors retain input order through fallible checked allocation:

- `QualitativePalette::{fixed,extendable}` require nonempty opaque sequences,
  retain duplicates, and expose `colors()` plus `capacity()`.
- `AccentPalette::new` requires nonempty opaque seeds and exposes `seeds()`.
  It is independently always extendable at render time.
- The current map's private `from_opaque`, `entries()` and `color()` supply the
  target map construction/inspection behavior. Keys must be nonempty, already
  trimmed, control-free and unique by UTF-8 bytes. Interior spaces, case,
  punctuation and canonically equivalent Unicode spellings remain distinct.
  Repeated colors and unused entries are valid. Missing used keys reject before
  fonts through the actual fifteen-family consumers.
- `CategoricalColorScale::{from_palette,from_map}` retain one private choice;
  `palette()` and `map()` inspect it without exposing enum payload variants.
- `ColorStop::new` accepts finite offsets in `[0,1]`, normalizes signed zero and
  retains an opaque color. `offset()` and `color()` inspect exact normalized
  inputs. There is no alpha-dropping admission of `Color`.
- `SequentialGradient::new` requires at least two strictly ordered stops and
  exact endpoints 0 and 1. `DivergingGradient::new` additionally requires an
  explicit unique neutral at exactly 0.5, with stops on both sides. Additional
  ordered stops are valid. Constructors never sort, repair or coalesce. Both
  expose `stops()`; `neutral()` derives its color from the retained 0.5 stop.
- `ColorScales::new` retains three complete validated fields. Its readonly
  getters and consuming `with_qualitative`, `with_sequential`, `with_diverging`
  edits replace only their named component and preserve the source/siblings.
- `ColorDomain::exact` requires finite `maximum >= minimum`, including a constant
  domain, and normalizes both signed zeros. `minimum()` and `maximum()` return
  retained inputs.
- `ContinuousColorScale::{sequential,diverging}` retain one private choice,
  optional typed gradient and optional exact domain. A diverging center must be
  finite and contained inclusively by an exact domain. Constant diverging domains
  therefore equal the center. Getters inspect kind, center, domain and matching
  gradient. `with_domain`, `with_center`, `with_sequential_gradient` and
  `with_diverging_gradient` validate the resulting complete value. Wrong-kind
  edits reject immediately, including `None`; they do not switch kind or store
  inert settings.

Values implement `Clone + Debug + PartialEq + Eq` over their normalized semantic
representation. Arbitrary-size collections share immutable Arc payloads; identity
is complete normalized content, not pointer/cache identity. Target construction
edits waiting for public customization have method-local non-test-build
`expect(dead_code)` annotations and meaningful current tests, not module-wide
dead-code allowances. All target types remain outside the public crate root.

## Preparation, transport and publication

Heatmap preparation explicitly receives the owning `ColorScales`. The existing
generic preparation entry point carries an optional owner operand: Heatmap
requires it, while other current routes do not need it. Production Plot/Canvas
roots supply the resolved owning value; unit fixtures supply complete validated
inputs. A missing Heatmap owner is an error, not late preset reconstruction.

Current `HeatmapScale` and public `.color_limits` instructions normalize into one
retained `ContinuousColorScale` before colors, contrast, fonts or guides are
prepared. Test-only raw gradient input is converted at its attachment boundary
and stores a validated immutable sequence or a retained lexical refusal; it does
not retain backend stop vectors. Setter/preflight timing is preserved. An exact
domain comes from `ColorDomain`, so `-0..0` now has canonical `0..0` identity and
still one numeric label with the same constant-domain mapping.

Prepared Heatmap and Beeswarm values retain complete normalized selections.
Shared contributors transport those immutable selections and compare normalized
kind, center and complete opaque semantic stops. The final merged exact/automatic
domain installs through the validated domain edit. Projected sampling gradients
are separate measured consequences, retained as typed immutable sequences inside
`ContinuousColors`. Backend `GradientStop` vectors are final serialized/sampled
guide operands, not semantic selection inputs. Opaque sampled guide RGB agrees
with mark RGB; the mark's own byte alpha remains independent.

The built-in Heatmap sequence remains `#F7FBFF -> #08306B`; its diverging sequence
remains `(49,130,189) -> (247,247,247) -> (222,45,38)` at offsets 0/0.5/1.
Beeswarm explicitly retains its current typed `#3182BD -> #DE2D26` family source.
The approved common sequential default and fully divergent Beeswarm behavior
remain B2. General diverging values are constructible privately, while the current
three-stop Heatmap/Shared remapping consumer restriction is still explicit until
its B2 migration. Private values are not exposed as a broader public promise.

## Resource and copy ledger evidence

Operation-owned palette/gradient assembly supplies its existing generation ledger.
It admits each immutable Vec/Arc header, element visit/record and buffer allocation
before retention. Known-size input gets one fallible allocation. Unknown-size
input grows through checked capacity arithmetic, admitting the complete new buffer
and possible existing-element copy before reservation. Constructors introduce no
new quota or network/font discovery.

The independent preset oracle is **`31 + 2*n` visits**, where `n` is the curated
seed count (Publication 8, other presets 5): the existing two point relationships,
four sequence headers, two seed sequences and five stops with endpoint/order/
neutral validation. Bytes are the two existing point-outline payloads plus four
Vec/Arc headers, `2*n` opaque colors and five `ColorStop` records. Plot and Canvas
root/explicit-child minus/equal-budget tests verify those formulas and refusal
before fonts/backend begin. Known-size buffers can be fully reserved before a
later element-work refusal; byte-limit refusals still occur before the over-budget
reservation.

Whole-theme copies now admit **22 shared handles**: eighteen existing font stacks,
three scale payloads and one accent payload, with zero copied palette/gradient
buffer bytes. Shared-payload comparison similarly prices the four new handles
before the existing eighteen font profiles. Independent scale payload comparisons
admit their complete fixed-size content before equality. Runtime raw map lookups
price actually examined UTF-8 bytes, and map copies admit one immutable handle.
Fatal refusal never becomes a cache miss or a fallback.

### Heatmap intrinsic retention and cache matching

Heatmap intrinsic cache retention uses `ContinuousColors::share` on the supplied
operation ledger: **one handle visit, zero copied stop records and zero stop-buffer
bytes**, regardless of gradient length. The cache object's existing storage charge,
its separately admitted cache entry and its four existing font-stack copies retain
their own accounting. The former stop-slice deep-copy charge is removed.

The focused storage oracle uses four one-entry generic stacks: five copy visits
(one gradient handle plus four font copies), four font-entry records, and bytes for
the cache object plus four Vec/Arc/font-entry payloads. Both two- and five-stop
gradients pass with exactly those bytes/records and no remaining stop storage;
minus-one work, byte or record capacity refuses. Retained stop-payload addresses
are unchanged through copies and cache hits.

Cache matching is fallible and content-priced. It admits the normalized domain,
then a shared-payload/length witness; independent equal or differing gradient
payloads admit every examined normalized offset/opaque-color pair. The four text
roles use their existing admitted complete-role comparisons as well. Full-content
matching and last-stop differences are checked at exact/minus-one work boundaries.
A live intrinsic hit consumes its matching work without new records, bytes or cache
entries; refusal propagates before reanalysis rather than becoming a cache miss.

### Legacy signed-zero centers

Heatmap's scientific domain calculation validates a finite legacy diverging center
and canonicalizes `-0.0` to `+0.0` before crossing the retained-selection boundary.
It therefore agrees with `ContinuousColorScale` for both automatic and exact
domains. The actual cell scientific value can retain its original minus-zero bits;
center identity normalization does not rewrite observations. Production preparation
selects the expected neutral RGB before fonts, and full terminal SVG output for
negative-zero and positive-zero centers is identical in both domain modes, with
all three source cells retained.

## Source migration and verification

Primary owners are `src/theme/{scale_values,scale,resolved,color,paint}.rs`,
`src/spec/categorical.rs`, builder gradient instruction/copy paths, prepared
Heatmap/Beeswarm code, Shared continuous descriptors, and Plot/Canvas preparation.
Existing private preparation callers are migrated atomically, including affected
Rust unit and ignored benchmark fixtures. Categorical, accent, Matrix/Marginal and
guide paint callers consume their actual opaque owner. Production tuple/opacity
exact-style APIs keep their B3/B4 cutovers.

Focused evidence includes constructor/field/edit coverage; exact raw-key/Unicode
identity and byte admission; fixed capacity and stable extension prefixes;
independent accent/palette edits; immutable operation-cache reuse; an edited owning
Heatmap gradient reaching actual selected SVG fills; sequential/diverging/constant
domains; sampled-guide collision refusal; local/facet/Shared transport; and unchanged
default RGB/alpha. Decoded PNG evidence confirms constant mark/guide agreement.

The selected Beeswarm, Matrix and Marginal snapshots match existing expectations.
No snapshot or hash expectations are repinned in this leaf. Signed-zero identity
and admission fixtures change only for their authoritative normalization/new actual
storage and shared-copy costs.

Verification uses bounded `cargo test --locked --offline` filters for these owners
and representative categorical/Heatmap/composition/visual-regression contracts,
`just fmt-check`, strict `just lint`
(`cargo clippy --locked --offline --all-targets -- -D warnings`), and whitespace
checks. The complete theme/group and B2 consumer/default migrations retain their
own later closure gates.

The completed focused set contains **68 distinct tests**: nine value/owner/admission
fixtures; six runtime scale/cache tests; the independent palette-prefix arithmetic
fixture; four Shared descriptor tests; five complete-preset fixtures; both Plot/
Canvas construction boundaries; theme clone/matching boundaries; twelve Beeswarm
and eleven Heatmap preparation/capture/admission fixtures; representative private
gradient, domain, cache, Shared/facet, categorical, PNG and snapshot callers.
Localized constructor-budget and signed-zero expectation failures were corrected
with their independent storage/copy and canonical-domain oracles, then rerun through
their focused filters. Formatting, strict Clippy and whitespace checks pass.

Independent review follow-up adds **five regression fixtures**, checked with
**13 focused tests** across the signed-center preparation/terminal paths, intrinsic
copy/matching/live-hit paths and their affected existing domain/cache neighbors.
Automatic and exact legacy negative-zero centers keep neutral RGB and unchanged
source cells/output; intrinsic copies have exact zero-stop-storage budgets for
different gradient lengths, and independent content matching plus live-hit refusal
remain fatal on the actual operation ledger. No full-suite command or snapshot
repinning is part of this follow-up.
