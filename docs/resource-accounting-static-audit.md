# Resource-accounting static audit

## Scope and verdict

**Audited source:** `582b79946682588e824a790b59362b7ce98ad046`
(`refactor(composition): retain exact owner-local chrome and prose`). The worktree
was clean at the initial audit's start. This source-and-history audit incorporates
the independent review's reachability qualifications and cross-references the
separate performance study. Only this report was written by this audit and its
revision; no source changes, builds, tests, commits, network access, or subagents
were used.

**Verdict:** the shared counters are useful deterministic admission controls, but
the current implementation does **not** support interpreting them as actual
machine work, total retained storage, peak memory, RSS, or elapsed time. It also
does not completely implement the stronger “actual examined entries / every
first-party retention” language in the resource specification. There are concrete
uncharged allocations and traversals, duplicate storage-slot charges, and
conservative potential-work charges. The problem is not merely that dependencies
have hidden costs.

The central distinction is:

* The counter implementation reliably bounds **successful calls to its claim
  methods** on one shared operation. That is a strong, narrow property.
* Whether a claim covers the intended operation, precedes it, uses the right
  class, and charges actual rather than potential work depends on each producer.
  Several inspected producers fail one or more of these conditions.
* The constants are policy choices with historical provenance. No inspected
  evidence calibrates one million mixed visits to a latency, CPU, or memory
  service level. The byte ceiling has an arithmetic historical derivation, not
  a measured renderer-memory derivation.

The 2,048-word publication at **999,771 visits** has **229 visits (0.0229%)** of
nominal headroom. The separate [performance study](resource-accounting-performance.md)
now reproduces that total: preserve-text rendering has medians of **101.158 ms**
normally and **96.978 ms** with bookkeeping disabled, with a median paired
difference of **3.745 ms / 3.835%**. Normal outlined-SVG conversion of the same
reference scene is refused by the visit cap. These measured results reinforce
the distinction between policy admission and physical cost; they do not calibrate
a portable threshold.

References refer to this commit. Source paths below omit the common `src/`
prefix; `canonical.rs` means `src/font/canonical.rs`, `kernel.rs` means
`src/layout/kernel.rs`, and `requirements.rs` means
`src/layout/text/requirements.rs`. Short filenames in a finding refer to its
already named module. Documentation/specification paths are repository-relative.
Function names are included because line numbers will move. “Confirmed” means
supported by the inspected call path, not newly reproduced by executing it. Existing test
source and documentation are evidence of assertions and historical observations,
respectively, not tests rerun here.

## 1. Exact caps, units, and ownership

### 1.1 Independent resource rows

| Resource | Inclusive production cap | Actual owner and admission boundary | Meaning and limitations |
| --- | ---: | --- | --- |
| Processing visits | 1,000,000 | `font/canonical.rs:1423–1444`, `OperationGenerationResources::claim_visits`; `PROCESSING_VISIT_LIMIT` at line 31 | Sum of caller-supplied logical work charges. Mixed units include scalars, UTF-8 bytes, list entries, sort-input counts, proposed comparison bounds, arithmetic iterations, and squared-limb estimates. Not an actual-instruction counter. |
| Planning records (`slots`) | 1,000,000 | `canonical.rs:1357–1363,2636–2653`, `claim_slots` / `claim_text_generation_resource` | Cumulative claimed logical headers/elements/reservation slots. Some callers charge capacity that is never populated; some real arrays are not charged. Not a complete live-record census. |
| Prepared topology slots | 1,000,000 | `canonical.rs:1377–1389`; `layout/kernel.rs:463–468`; `data/source.rs:75–113,129–135` | Structural source/declaration census: root, rows, cells, categories, addressed roles, targets, etc. Independent of visits and storage. A million topology slots does not entitle a root to process a million rows. |
| Measurement/intrinsic call attempts | 1,000,000 | `canonical.rs:1365–1375,1582–1591`; root loop `kernel.rs:829–835` | Named API/owner attempts, including hits at those boundaries. Not every function call, iteration, comparison, or draw call. Palette resolution also uses this class (`theme/scale.rs:32`). |
| Root-local cache entries | 1,000,000 | `canonical.rs:1396–1406`; producer insertion sites | Cumulative admitted cache insertions, including many replacements/new consequences. Not current occupancy, payload size, or number of all cached subobjects. Coverage varies; see below. |
| Font discovery entries | 1,000,000 | `canonical.rs:326–338`; `FontBudget::claim_discovery` at 273–276 | Fontconfig/filesystem discovery events/directives, including rejected/filtered entries. The wrapper also charges the visit counter. Not a million font files or a byte allowance. |
| Font/source/scratch byte admissions | 536,870,912 bytes = 512 MiB ≈ 536.87 MB | `canonical.rs:1414–1421`; aliases `claim_unicode_bytes`, `claim_source_bytes`, etc. | One cumulative logical byte counter. The diagnostic still says **`system-font bytes`**, even when a source vector, exact spacing integer, geometry buffer, or cache key exhausts it. It is not a complete allocation total or a peak-live limit. |
| Root feedback rounds | 8 for the entire root tree | `kernel.rs:761–939`, especially `history.len() == 7` | Structural convergence/cycle policy. A successful eighth round is allowed; a nonconverged eighth refuses. It does not bound work inside a round. |
| Automatic outside-guide trials | 8 per owner per root round | `decorations/plan.rs:399–435`; `kernel/feedback.rs:385–413` | Four ordered edges × normal/compact flow, with unchanged compact alternatives skipped. Explicit edge uses 2. Owner-local limit, not eight trials for the entire scene. With eight rounds, one automatic owner can have up to 64 such attempts, subject to other limits. |
| Optional exact text-query memo | 256 entries | `theme/resolved.rs:1025–1041`, `cache_text_query` | Stops optional insertion and returns success. Distinct from fatal root cache-entry exhaustion. Paragraph/lazy/font/child caches are not all limited to 256. |
| PNG source/target axis | 32,768 pixels | `scene.rs:9,168–183,209–228`; `render/svg_backend.rs::PngTargetPlan` | PNG-local viability policy; positive/finite/representable dimensions are separate checks. Does not constrain SVG layout. |
| PNG source/target area | 100,000,000 pixels | Same owners | Source CSS-pixel area and requested raster area are checked separately. One RGBA8 image of this area is 400,000,000 bytes, before parser, rasterizer, compression, and copies. |
| PNG parser input | 67,108,864 bytes = 64 MiB | `scene.rs:12,128–165,196–207` | Actual outlined-SVG parser-input length, checked before the derived buffer/parser route. Not the preserved SVG length, root byte ledger, or raster backing-store budget. |

All four generation count classes use `TEXT_GENERATION_LIMIT`; visits and bytes
have separate limit fields. Font discovery has a separate mutex/counter. Equality
with a cap is accepted; checked addition overflow and a total above the cap are
different failures. Tests can change visit/byte limits; production constructors
install the constants above.

### 1.2 Shared handles and lifecycle

`layout/kernel.rs::ResourceState::new` creates `OperationFontResources` and takes
its generation handle **before** invoking preparation. Font resources contain an
`Arc<Mutex<FontResourceCounters>>` plus the generation resources. Generation
resources contain an `Arc<Mutex<GenerationResourceCounters>>` and a separate
`Arc<Mutex<PaletteCache>>`. `PreparedStage` borrows the root's state and reservation
driver. The prepared font environment wraps `CanonicalFontCore` in
`Arc<RwLock<_>>`; its budget and generation fields refer to the same generation
ledger (`canonical.rs:293–384,1291–1461,2551–2632`).

Cloning these handles shares consumption; it does not create fresh credit.
`PreparedFontEnvironment::share_generation_resources` compares counter-Arc
identity and refuses a different ledger without rebinding (`1601–1614`). Source
rows, descendants, fonts, later rounds, and appropriately threaded child calls
therefore compete for the same root allowances. Independent Plot operations
construct new roots. A retained `Output` reuses its scene and fonts; outlining
can still consume the retained environment's face-lookup visits, while repeated
PNG rasterization has its own terminal-local gates.

Each individual claim locks, computes a checked total, and writes only on success.
This establishes atomic **single-counter admission** under concurrent claims. It
does not make a multi-claim producer transactional. For example,
`PlotData::reserve_source` claims records, then bytes, then calls
`try_reserve_exact`; byte refusal leaves the record claim spent. `FontBudget`
can spend discovery before a visit refusal. No destructor, clear, truncation,
failed trial, or allocation failure refunds successful claims. Feedback trial
rollback restores semantic effects, not resources.

`MeasurementCall(())` is a private ownership convention: outer measurement admits
once and passes a reference into an `*_in_call` implementation. The token has no
ledger identity and is not consumed linearly; the type alone does not prove
exactly-once accounting. Inspected root/text/axis/Matrix/child callers use it to
avoid repricing an implementation of the same logical call. Independent child
and text calls are additional attempts.

### 1.3 Structural safety is not a heuristic budget

Keep the following findings separate:

* Checked `u128` length/product calculations, conversion to `usize`, signed
  `i64` layout quanta, finite-number validation, source spans represented by
  `u32`, and noncollapse checks protect representability and geometry. They are
  genuine representation boundaries. See `kernel.rs::LayoutScalar`,
  `render/coordinates.rs::CanonicalCoordinate`, and
  `theme/spacing.rs::ExactLength::round` / `scalar_pixels`.
* Source-row/address tickets and matching exact inputs protect ownership and
  semantic identity. Font sealing prevents late selection. Those are structural
  invariants, independent of choosing a million as the maximum census.
* Eight rounds and ordered guide alternatives define bounded coordination. Their
  numerical values are policy, but their roles differ from pricing source work.
* One million records/visits/calls/etc. and 512 MiB are heuristic admission
  magnitudes. Reaching 1,000,001 does not imply integer overflow or infeasible RAM.
* PNG dimensions/area/parser length are separate terminal safety policies.
  Neither PNG refusal nor its backing-store arithmetic establishes a shared
  layout-memory guarantee.

## 2. Provenance and what was actually measured

Read-only Git history gives a concrete origin, rather than a calibration story:

| Evidence | What it establishes |
| --- | --- |
| `git log -L 29,32:src/font/canonical.rs` | `0f2ab3f` introduced discovery = 1M, font work = 1M, text generation = 1M, and bytes = 536,870,912. `4a24e20` renamed `FONT_WORK_LIMIT` to `PROCESSING_VISIT_LIMIT`; it did not derive a new magnitude from measurements. |
| `specs/91_IMPLEMENTATION_PLAN.md:2417–2421` | Historical adjudication reused existing million-record/work magnitudes and chose the least power of two covering the admitted maximum RGBA backing store: `next_power_of_two(4 × 100,000,000) = 536,870,912`. This is a documented arithmetic origin. A cumulative font/source/scratch admission is a different quantity from one live raster image. |
| `4a24e20` and `docs/resource-admission.md` | Coarse shared processing replaced earlier synthetic rule prices; historical corrections covered proposal/geometry storage, retained-face/list traversal, shaping context, and ledger rebinding. These are real targeted fixes, not proof of every subsequent producer. |
| `cc6880b` and later native-layout history | Source admission and many scientific producers were added after the initial coarse-policy change. Current native family coverage cannot be inferred from PF4 alone. |
| `5f2c705`, `9951f08` | Typed font stacks and shared font/query work added byte-content hashing/copy/equality charges and exact relative-spacing admission. The unchanged cap now has a different effective unit mix. |
| `85c7448`, `41dc420`, `582b799` | Exact axis, guide, and composition relationships added arithmetic work and operation-local retained consequences. Latest caption headroom belongs to this changed implementation. |

`docs/resource-admission.md:239–285,287–379` records old release wall-time
samples, counters, and output hashes. It explicitly says the times are not CPU
time, the measurements are noisy, and admitted bytes are not RSS. The PF4
2,048-word direct text cold/reflow/hit sequence ended at **709,818 visits**;
that is not the same operation as the latest complete publication. The old hit
costs of 2/3 visits are also not current: the current shaping test charges source
profile equality plus presentation hashing and owner equality
(`layout/text/shaping_tests.rs:99–109`).

`docs/font-work-admission.md` records a million-visit failure on `5f2c705`, followed
by corrected publication at **991,968** visits. `docs/composition-values.md:193–205`
records another failure on the isolated `41dc420` baseline at **1,000,001**, then
the complete 2,048-word publication at **999,771 visits, 56,366,327 admitted bytes,
234,560 records, 1,152 calls, 477 cache entries, 2,175 proposal visits, and 2,403
exact trials**. These are historical documented observations, not independently
reproduced here. The actual fixture
`theme/resolved.rs::caption_publication_work_evidence_keeps_complete_required_sources`
asserts success, `visits <= 1_000_000`, and all 2,048 words in preserved-text SVG;
it does **not** assert a portable constant of 999,771 or a wall-time target.

Thus there is evidence of **ordinary complete content being refused by this
policy**, and of changes restoring admission. Those historical accounting
corrections were not isolated timing experiments. The current planning note
explicitly says budget thresholds are not optimization targets
(`specs/93_T6_EXECUTION_PLAN.md:202–211`).

### 2.1 Subsequent performance evidence on the same base

`docs/resource-accounting-performance.md` reports a separate 2026-10-07 study on
`582b799`, using a detached experimental worktree. Its policies are normal limits,
counting without limits, and no bookkeeping/no limits. Structural checks and
algorithms remain active. These are attributed study results, not fresh execution
by this audit.

* The exact two-point **reference2048** fixture reproduces **999,771 visit units**
  from **415,556 visit-claim invocations**, with **516,328 total ledger claims**.
  The units are not a count of mutex acquisitions or machine instructions.
* Preserve-text medians are **101.158 ms normal / 96.978 ms bookkeeping-off**.
  The median of paired differences is **3.745 ms / 3.835%**, not the subtraction
  of those independent medians. Bypassing enforcement alone has no consistent
  timing benefit in the study.
* Normal **reference2048 outlined SVG fails during outlining** after preserve
  succeeds. Counting-unlimited and bookkeeping-off complete identical outlined
  bytes, with medians **234.921 / 233.094 ms**. Refusal timing is not the timing
  of a completed render. The 100-point caption2048 is a different fixture and
  already refuses before preserve-text completion.
* Fresh-process reference2048 normal peak RSS has median **49,020 KiB**. Dense
  bee128's counting-unlimited run admits **273,245,121 cumulative bytes** but has
  median whole-process peak RSS **40,512 KiB**. These are distinct quantities;
  neither measures the exact allocator traffic or one buffer's residency.

The study documents one Rust test thread, optimized release test executables,
two warmup triples and eleven measured triples per timing group, separate claim
instrumentation, exact output comparisons, and separate fresh-process RSS/coarse
CPU observations. It is scoped evidence of workload-dependent bookkeeping cost
and policy refusal, not a universal per-visit price or an all-family profile.
The remaining evidence gaps in section 7.2 are narrowed accordingly.

## 3. What one processing “visit” buys

Let `V = Σ admitted_amount_i`. This is a well-defined integer sum, but not a
homogeneous count of executed operations. Its effective weights depend on the
producer and representation:

| Producer / exact source | Increment unit and guarded work | Assessment |
| --- | --- | --- |
| `data/source.rs::source_position`, `source_levels` | One per examined entry, including the terminating match; levels also charge each input | Actual list-visit unit. The cost of `String`/compound equality inside one entry is not generally charged per byte. |
| `data/source.rs::sort_source_by` / `sift` (`43–72`) | `n` before an in-place **first-party heapsort** | Intentional PF4 coarse sort-input admission, not comparator count. `sift` comparisons/swaps can take `O(n log n)` with `n` visits. The private comparator is not instrumented; this helper is not std sort. The coarse rule itself is not a missing-comparison-charge defect. |
| `canonical.rs::canonicalize_candidate_occurrences`, discovery sorts | Input count before std sort, input count before dedup, survivor count before second sort | Distinct passes, coarse input units. Std stable sorting may allocate scratch. Comparator byte/path costs are not counted. |
| `plots/point_interval.rs::apply_category_order`, `sort_rows` (`150–207`) | `4n(floor(log2(max(n,1)))+1)` before each std unstable sort; one per cycle swap | Synthetic potential-work allowance, unlike the common heapsort's `n`. Value ordering invokes three priced sorts. Not actual comparisons, even on already ordered data. |
| `layout/kernel/tracks.rs:133–142,162–192` | `n(floor(log2(n))+1)` for nonempty remainder sort; `2 × active.len()` before distribution/filter pass | Algorithm-shaped precharges, not a uniform sort rule. |
| `labels.rs::resolve_source` (`85–98`) | `raw.len()` plus `alias_count × (raw.len()+1)` before BTreeMap lookup | UTF-8 bytes and a pessimistic whole-map comparison bound. BTreeMap lookup is not a linear scan of every alias. Unused aliases can cause refusal. |
| `labels.rs::humanize` (`211–217`) | Intermediate byte length and **22 × word byte length** | Fixed weighted allowance persists outside Unicode rule handling. `acronym` has 16 entries and stops at a match; 22 is not an observed scan count. |
| `canonical.rs::FontStack::{byte_work,copy_with_generation,admit_copy,css,hash_into,same,share}` | Family-entry + name-byte totals; CSS twice that total; hash encoding bytes; actual comparison bytes; one per Arc share | Representation-dependent mixture. `byte_work` itself loops over families before admission; it reads lengths without scanning name contents. Its quoted “copy pass” is not instruction counting. |
| `theme/resolved.rs::CachedTextQuery::{shaping_state,presentation_fingerprint}`, `cache_text_query` | Source/owner bytes plus fixed encoding lengths; copy bytes on insertion | Current code explicitly prices hashing and copying, contrary to old blanket exemptions. A hit can still scan a long mutable caller source. |
| `font/unicode16/nfc.rs::{decompose,canonical_order,recompose}` | Scalar/decomposition nodes; each preceding ordering comparison; recomposition inputs | Good guard on adversarial mark ordering. Table-search steps are deliberately unpriced. Input `.chars().count()` is a real prescan before its bulk claim. |
| `layout/unicode16.rs::linebreaks`; `layout/text.rs::measure_text`, `paragraph_items`, `itemize_bidi_run`, `select_line` | Scalar inputs, script/bracket work, endpoint trials, binary-search steps, proposal visits | Many important loops are guarded, but this is not every scalar traversal or every lookup; see font selection/census findings. Fixed Unicode lookup costs and dependency internals are outside the unit. |
| `canonical.rs::shape_retained_run` (`3402–3517`) | Actual run scalars + at most five pre-/five post-context scalars before buffer/shaping | Counts shaper input, not GSUB/GPOS instructions or output geometry work. Output glyph count/bytes are admitted **after dependency shaping**, before first-party retention and ink probes. |
| `stats/distribution.rs::gaussian_kde_with_source` | `n + g` validation inputs, then `n` for each of `g` evaluation points | On successful finite inputs, `n + g + ng`; each kernel unit includes an `exp` and arithmetic. Public `kde` passes `None`, intentionally outside a root. |
| `plots/regression.rs::PreparedRegression::new` (`248–266`) | `10n` before OLS; one per band point (40) | Ten scalar scans correspond to validation, scales, means, products, and residuals over two channels on success. One band-point unit contains `statrs` inverse-CDF work, not one arithmetic instruction. |
| `plots/bar_data.rs::{percentile_bootstrap,SplitMix64::index,stable_mean,generated_interval}` | Each random rejection-sampling draw; scans per sample/statistic; seed-field hash bytes; one Student-t call | Important repeat work is guarded. Inverse-CDF internals are not. Grouped bootstrap repeatedly repopulates reserved buffers and may recharge their slots. |
| `plots/beeswarm/packing.rs::pack` | Point, prior neighbor, tangent direction, candidate, and each collision pair | Actual first-party collision guard; sort uses coarse `n`. Nested candidate/prior-point loops can approach cubic work; no exponential claim is needed to explain the cap's value here. |
| `numeric.rs::admit` / `ln_bounds` | **One** per input/exact-power comparison/refinement/setup/series iteration/final rounding admission | Each iteration contains multiple variable-width BigUint expressions. Dependency limb arithmetic is explicitly unpriced. Precision doubles if certification fails. |
| `theme/spacing.rs::admit_math` (`602–626`) | `limbs² × operands`, `limbs=max(1,ceil(bits/64))` | A schoolbook-style potential-work estimate, not actual limb operations. Copies, shifts, division, GCD iterations and rounding use this price even though their algorithms differ. |
| `theme/color.rs::admit` (`793–800`) | One visit plus 2,560 scratch bytes for exact cube comparison | Another BigUint pricing convention, unlike spacing and Log10. Fixed color-threshold binary search steps are charged (`encoded_channel`), unlike fixed Unicode-table searches. |
| `render/geometry.rs::admit_scratch` | `count` visits and `count × sizeof(T)` bytes | A scratch-boundary work allowance, sometimes multiple per polygon or triangle. Not a count of all geometry predicates or actual heap allocations. |

These are legitimate ways to define **different logical metrics**. Adding them
to one number does not make them physically commensurate. In particular,
normalization scalar count, UTF-8 byte count, `exp` kernel count, and BigUint
limb-square estimates cannot be used as equal-priced work to rank performance.

Two concrete static refusal examples illustrate the distortion:

1. `Labels::resolve_source` with a nine-byte key and 100,000 aliases attempts
   `9 + 100,000 × 10 = 1,000,009` visits on a fresh ledger, before even performing
   the map lookup. The actual lookup need not examine the unused aliases. This
   establishes a mismatch between the allowance and examined-entry work. The
   actual cost of that lookup with the cap bypassed is unmeasured; neither this
   example nor the performance study assigns it a latency. An entire Plot may
   refuse at an earlier boundary.
2. For `n` finite box samples whose endpoints are both in the fences,
   `box_summary_with_source` prepays `3n` for low-whisker `find`, high-whisker
   `find`, and flier filtering. Those three traversals examine `n+2` values, not
   `3n` (`plots/boxplot.rs:443–461`). With no fliers it still reserves/charges
   `n` flier slots. Successful-data overcharging is present, not just early-error
   overcharging.

### 3.1 Repeated rows: distinct passes versus duplicate claims

Do not remove repeated amounts merely because they are adjacent:

* `aligned_finite_rows` / `aligned_finite_f64` read x, read y, then zip/filter:
  **three actual passes**, each admitted. A retained row can later participate
  in more range/projection passes.
* `PreparedScatter::new` claims `rows.len()` twice at `scatter.rs:148–152` for
  distinct x and y min/max passes. Its count-then-fill hue grouping also traverses
  the rows twice (`164–175`). These are not duplicate charges for one pass.
* `Matrix::prepare_axes` has adjacent sample-length claims at `matrix.rs:528–529`;
  `matrix_support_range` separately folds minimum and maximum (`1583–1584`).
* Scott-bandwidth callers prepay five scans; the successful helper really has
  finite validation, minimum, maximum, absolute scale, and variance traversal
  (`stats/distribution.rs:133–171`). Density adds its own support min/max pass.
  Early failures can execute fewer scans than prepaid.
* Matrix's weighted-density update and following finite validation are two
  passes (`1368–1373`), not a double charge. KDE evaluation is separate again.

By contrast, the label-domain **record** charges described next count the same
reserved slots twice. Calling both situations “double counting” obscures a real
bug and would encourage removing valid processing admission.

## 4. Confirmed accounting defects and mismatches

Priorities express impact on the stated accounting contract, not measured CPU
hotspots. Findings intentionally distinguish omitted accounting, overcharging,
late guards, and policy scope.

### F1 — High: prepared font storage bypasses record/byte admission

`PreparedStage::reserve_prepared` and `reserve_prepared_string` only enter a
reservation phase and call `try_reserve_exact`. `ReservationDriver` has **no
resource claims** (`layout/kernel.rs:482–507,605–637`).
`canonical.rs::reserve_prepared_or_local` delegates directly to that path. A
reservation-stage gate is not a resource gate.

Confirmed production examples, not just unused helpers:

* `PreparedTextCensus::prepare_system_fonts` calls
  `PreparedFontEnvironment::reserve_system_discovery` (`requirements.rs:494–513`).
  It requests **1,000,000 `DiscoveryRecord` slots unconditionally**
  (`canonical.rs:1510–1523`). No record or byte claim covers this buffer. The
  discovery routine also reserves the full discovery limit when not supplied
  such storage (`3544–3554`). Requested storage is exactly
  `1,000,000 × size_of::<DiscoveryRecord>()`, regardless of actual discovery.
  A conventional 48-byte x86-64 layout would mean about 48 MB; the actual
  `size_of` was not measured by this audit. This is reserved capacity, not a
  claim of 48 MB resident memory. The performance study keeps that reservation
  and reports normal scatter100 whole-process peak RSS of **40,264 KiB**, below
  the illustrative buffer size. Its RSS evidence does not measure this buffer's
  layout or page residency. The unconditional request still contradicts the
  specification's no mandatory million-entry allocation statement.
* `PreparedStage::reserve_scalar_presence_map` requests **17,408 u64 words =
  139,264 bytes (136 KiB)** and zeroes them (`kernel.rs:470–479`), with no
  byte/record debit for the map.
* The active batch `CanonicalFontCore::prepare_pairs_with_stage` allocates pair
  shells, resolved-name tuples, candidate/provenance arrays, scalar selections,
  and glyph probes via the same unpriced helper (`canonical.rs:2014–2279`).
  It explicitly prices retained font-family arrays/name bytes and Arc headers,
  but that does not cover the other arrays. The maximum probe allocation is
  `rendering_scalar_count × candidate_count`, checked for representability,
  **not admitted against records/bytes before allocation**. All these probes
  are subsequently executed on success (`3337–3364`); the defect is not a false
  claim that the maximum is unused, but allocation before the work cap can stop it.
* `from_retained_with_stage` reserves `FaceMetadata` for declared face capacity;
  `decode_family_names` charges occurrence records and a decoded-name byte bound,
  but not the occurrence array's backing bytes (`2561–2578,2691–2702`).
  Discovery-record paths, retained-file headers/Arcs, and fontconfig
  tasks/directories/aliases also have first-party allocations through the
  unpriced helper or ordinary clones (`3526–3767,4022–4325`). Font-file contents
  **are** separately admitted; these omissions do not erase that protection.

The census's own `reserve_census_vec` does debit records and bytes
(`requirements.rs:668–696`). That demonstrates why one cannot infer an omission
from a raw reservation call without checking its caller; the examples above
remain after making that distinction.

### F2 — High: prepared-pair hits deep-copy storage without admission

`PreparedFontEnvironment::prepared_pair` charges list/stack comparison, then
returns `prepared.resolved.clone()` (`canonical.rs:1809–1825`).
`ResolvedFontPair`, `PairProvenance`, `SelectedScalarFaces`, and `SelectedFontUnit`
derive `Clone` over **Vecs and Strings**, not shared immutable payloads
(`932–990`). Candidates, all selected scalars, selected-unit headers, and unit
source strings are copied without record/byte claims or fallible reservation.

This is called by production `layout/text.rs::measure_text` at line 935, not only
test font setup. Later admission of output provenance does not retroactively
guard the already allocated scalar/unit clones. Query payload hits that avoid
`measure_text` avoid this copy; a prepared-font-pair hit inside fresh text
analysis does not.

### F3 — High: variable first-party font selection/census scans are unpriced

`ResolvedFontPair::selection_for_unit` searches selected-unit strings and scalar
faces with `.find()` and no resources (`canonical.rs:993–1019`). The active path
is `selected_paragraph_items` → `paragraph_items` → `selected_face`
(`layout/text.rs:1981–1997,2031–2068,2185–2204`). The caller charges one scalar
item, **not every examined entry of either list**. A paragraph using many distinct
scalars therefore has potentially quadratic selection comparisons hidden behind
linear scalar charges. Long base-plus-mark units are repeatedly scanned by
`unit.chars().count()` / `any` as well.

Related concrete omissions:

* `SelectedScalarFaces::binary_search_by_key` is actually a linear `.position()`
  (`canonical.rs:959–968`); the method name is not a complexity guarantee.
* `prepare_source_units` searches prepared pairs with uncharged `.position()`
  (`1707–1711`). Its later candidate/face/coverage loops are separately guarded.
* `PreparedTextCensus::structural_address_count` scans all earlier requirements
  for each identity without visit admission (`requirements.rs:324–342`), is
  called before and inside materialization, and materialization also sorts
  requirements without a sort-input charge (`362–374,455`).

Separately, `reserve_required_scalar_capacities`, `materialize_required_scalars`,
and `scalar_selection_capacities` repeat bitmap resets and count/materialize
passes; `source_work_admitted` suppresses later source claims. Bitmap clearing
and storage-only materialization are **not included** in the visit total, but
PF4 explicitly excludes storage-only copying/materialization. Their absence
alone is not another semantic-search defect. The uncharged variable identity,
unit, and scalar searches above are the confirmed visit gaps; unadmitted buffer
allocations belong to F1/F2's storage findings. This distinction does not require
pricing every first-party instruction.

The retained-face search fix remains present: `find_face` does charge each face
before comparison, then directly indexes the retained file. That fix does not
cover these other lists.

### F4 — High: configuration and census paths omit the operation-bearing data view

`SourceAdmission for Option<PlotData>` is a no-op for `None`, and `PlotData::new`
has no `axis_resources`. This is valid for standalone public statistics and
input construction, but active root preparation still uses it in substantial
first-party work:

* `PlotPlan::new_with_theme_factory` invokes `source.validate_configuration`
  inside preparation (`orchestrator/lifecycle.rs:1013–1019`). That validator
  takes no resources and constructs bare `self.data()` views. Faceted explicit
  category validation calls `strict_strings` / normalization and `stable_levels`
  (`orchestrator/validation.rs:124–227`). `data.rs::stable_levels` is an
  uncharged quadratic `.contains()` loop with ordinary allocation (`603–611`).
  This precedes scientific topology admission in `prepare_plot`.
* Legend text census for Scatter and Marginal calls
  `observed_levels_for_finite_pairs` or
  `observed_levels_for_retained_finite_pairs` with **`plot.data()`**
  (`requirements.rs:1203–1235`). These reread columns, allocate temporaries, and
  deduplicate with uncharged searches (`data.rs:714–770`). A later admitted copy
  of each final label does not pay for those preceding temporary buffers/scans.
* Facet `base_plot` does carry resources into string extraction, but then invokes
  its own uncharged `stable_levels(Vec<String>)` for groups/Trajectory conditions
  (`orchestrator/facets/data.rs:225–245,358–365`).

These are concrete coverage gaps, not a request to charge caller-owned data
construction retroactively. Exhausting a counter in a later source helper cannot
prove that earlier configuration/census work was bounded by that counter.

### F5 — High: significance has partial collision admission, not full planning admission

`significance.rs::plan_comparisons_at_with_label` validates unique categories with
a quadratic prefix search, resolves target indices, checks comparison pairs,
sorts comparisons, allocates nested occupied shelves, and scans bracket spans
(`278–473`). Only the later shelf-level and occupied-interval comparisons at
`431–439` have local visit claims. Vec pushes, level arrays, and those other
variable traversals lack local record/byte/work admission. The entry function
and its wrappers do not prepay the missing variable work.

`place_lines` allocates a new Vec and visits levels; `expand_upper_limit` repeats
placement while doubling span and then for **64 bisections** (`955–1087`). This
path is **production-active** through Bar plans (`plots/bar.rs:107–117`,
`plots/horizontal_bar.rs:110`): automatic ranges call it when initial placement
fails (`significance.rs:529–548`), and explicit-limit failures call it to compute
the suggested limit in their diagnostic (`497–526`). Those level visits and
repeated local arrays remain unpriced. The `#[cfg(test)]` measurement counters
are observations, not resource claims. Doubling the numeric span is not
exponential growth in operation count; finite-number failure and the fixed
bisection count supply termination structure, not resource admission.

**Reachability qualification:** `fit_data_below_ceiling` (`1090–1145`) has a
similar unpriced growth/bisection sequence, but its caller requires
`LegendAnnotationConstraint::TopCeiling`, automatic limits, and no resolved
comparisons (`367–377`). All production entries—`plan`, `plan_grouped`, and `plan_horizontal`—
supply `NoObstacle` (`84,675,708`). The enum's own documentation states that
nondefault constraints remain only for direct regression tests (`40–52`). Thus
this helper is test-triggered, despite lacking a function-level `#[cfg(test)]`;
it is not another live consumer or an additional affected-family count. The
production `expand_upper_limit` and storage/search findings above sustain F5.

### F6 — High: glyph outlining and final-storage classes are incomplete

`render/svg_backend.rs::outline_planned_line` calls retained-face outlining once
per glyph. The `SvgGlyphPath` callback validates/formats each command but has no
resource handle (`244–321,606–654`). Its dynamic `WriteBuffer` can grow via
`try_reserve` (`837–848`). There is no per-command record, byte, or work debit.
`SvgOutlinePlan::derive` counts and then emits outlines, so the count pass also
performs outline traversal; only face-list visits are charged through the font
environment. `derive_counted` reserves the final string without a ledger debit
(`485–567`). PNG limits apply to the aggregate parser result, not each glyph's
intermediate command generation. The strongest explicit contract mismatch is
**outline-element record/storage admission**, required by spec 05 (`1363–1374`).
Lack of a per-command visit is not independently a defect where formatting and
storage-only materialization are excluded from processing charges; dependency
outline traversal likewise is not an actual first-party instruction counter.

Final canonical glyph origins, lines/runs, path vertices, triangles, and gradient
stops use `render/primitives.rs::storage` → `FinalStage::reserve_capture`. This
does debit bytes, but **does not charge an element record count**. The enclosing
`TextPlanningBackend::claim_call` charges one record per draw command
(`orchestrator/backend.rs:598–615`), not its variable payload. This is another
specific mismatch with the header-plus-elements rule, even when the input
glyphs/points were admitted in an earlier representation.

Final serialized SVG buffers use `reserve_storage` → `reserve_final`, which is a
phase/fallibility gate only. Terminal output may have a separate admission policy;
the absence of shared 512 MiB charges for all serialized SVG bytes is not itself
a bug. Those buffers, derived outline strings, and terminal copies are
**not included** in that ledger. `scene.rs::svg_with_text_as_paths` also copies
the cached byte vector outside it. This output-boundary observation is separate
from the explicit outline/capture element-storage mismatch. PNG parser, axis,
and area limits remain PNG-local and must not become shared SVG/layout limits.

### F7 — Medium: record “actualness” fails in both directions

`PlotData::reserve_source` always claims `additional` records, even when the Vec
already has enough capacity (`data/source.rs:291–317`). That convention supports
repeated materialization, but it requires callers not to preclaim the same future
elements twice.

`Labels::resolve_domain_source` first reserves the iterator's minimum length for
**both** `identities` and `resolved`, then reserves one more in both vectors for
every item before pushing (`labels.rs:124–155`). For an exact-size iterator of
`n` successful labels, the arrays consume **4n record claims for 2n elements**,
plus separate string claims. The second calls commonly request no extra bytes;
the slot debit still happens. This is a confirmed same-materialization duplicate.

Other cases charge upper-bound retention rather than actual records:

* `aligned_finite_rows` reserves `x.len()` pair slots even when null filtering
  retains fewer; that bounds potential output, not actual output.
* Boxplot reserves one possible flier per input, even with no fliers.
* Beeswarm reserves `2 × previous.len() + 1` candidates even when x separation
  generates only the zero candidate (`packing.rs:88–124`).
* `theme/color.rs::admit_candidates` reserves the maximum candidate prefix
  before generating it; refusal during earlier work leaves unused slots spent.

In the opposite direction, track allocator `admitted<T>` debits only bytes
(`layout/kernel/tracks.rs:34–47`), including returned `TrackAllocation::sizes`.
Its actual retained track elements have no record debit there. Font batch arrays
and final geometry arrays provide further examples. Records therefore mean
neither all actual elements nor a consistently defined upper bound on them.

### F8 — Medium: whole-list preprices violate “only examined entries”

Besides the label/box examples, confirmed producers include:

| Site | Charged amount versus actual traversal |
| --- | --- |
| `data.rs::explicit_category_order:623–655` | Full `seen.len()`/`observed.len()` before `contains`; even charges observed length when `allow_unobserved` short-circuits that lookup. The retain pass then has separate full-list precharges. |
| `plots/distribution_sample.rs::prepare_scoped:79–119` | Whole level/group/hue lists before early-exit `contains`, `find`, `position`, and sometimes absent optional channels. |
| `plots/matrix.rs::build_groups:1515–1522` | Whole hue list before an optional hue lookup, regardless of match position. |
| `plots/point_interval.rs::apply_category_order:127–140` | Whole row/request lists before prefix/position searches; row-list charge occurs even for subset data where the full-plot predicate skips the search. |
| `plots/heatmap.rs::triangular_family_order:1811–1831` | Every existing level is charged for each value, even an immediate first-level hit. |
| `plots/violin/precomputed.rs::prepare:150–158` | Whole hue list before `.any`; the test observation also increments by that length, so matching counter observations do not establish actual comparisons. |
| `theme/scale_values.rs::{ColorScales,AccentPalette}::admit_comparison` | For independent payloads, both lengths + 1 are prepaid before derived equality, even an immediate mismatch or length difference. |

These do impose upper-bound allowances at those sites. They are not evidence for
the current documentation's universal actual-examined-entry claim. String byte
costs inside list comparisons are also inconsistently priced: `same_bytes` counts
actual byte pairs, while many source-level `String == String` comparisons count
only one entry.

### F9 — Medium: admission is not universally before the expensive pass

Several known-size claims require computing the size by traversing content first:

* `normalize_source_string` copies, then calls `source.chars().count()`, then
  claims that count, then performs `split_whitespace` (`data/source.rs:332–344`).
* `PreparedTextRequirement::new_with_stage` counts scalars and selection units
  and checks soft hyphens before its source-work claim (`requirements.rs:211–225`).
* `format_census_source` formats and traverses into `CensusFormatCount` before
  work admission, then formats again into reserved output (`canonical.rs:108–167`).
* `measure_text` counts before admission, counts again for reserve length, then
  extends required scalars (`layout/text.rs:922–930`). Unicode normalization,
  line breaking, and canonical shaping have similar scalar-count prescans.
* `parse_bounded_fontconfig` checks entities and counts markup before parser
  input admission (`canonical.rs:4251–4266`); decoded font-name counting itself
  decodes UTF-16 before the decode-work claim (`2715–2723`).

Their byte-length/u32/input constraints still bound representability and some
prior storage. It would be wrong to assert an actual oversized-input exploit
from these lines alone. It is nevertheless false that a zero remaining work
budget prevents all source traversal, or that the declared scalar count measures
all decoding passes. `FontStack::byte_work` has the analogous family-entry
metadata prepass, although it does not rescan the name bytes.

There is also a small concrete late-allocation case in exact color arithmetic:
`cube_root` creates the two BigUints and shifted/summed `middle_sig` before the
next `admit(resources)` at `theme/color.rs:730–736`. The preceding comparison
admissions are for preceding expressions; the “before each exact expression”
story does not cover this construction in order.

### F10 — Medium: “all hashing/copying is exempt” is no longer the implementation

The current query hash/copy and font-stack paths explicitly charge bytes as work.
Bootstrap hashes its field bytes (`bar_data.rs:1856–1868`). Scale-input growth
charges `values.len()` copying work (`scale_values.rs:66–85`). Qq also charges
`values.len()` immediately before `observed.extend_from_slice(values)`
(`plots/qq.rs:475–479`), separately from validation and sorting. Other source copies,
Arc attachment, Vec relocation, and hashing font-file contents are unpriced as
visits. This is policy drift/inconsistency, not proof that either all copying
must be charged or all copying must be free. The exact policy needs naming and
ownership before its score is used as an optimization objective.

### F11 — Medium: theme construction/share/equality are partial work certificates

`ResolvedTheme::admitted_clone` charges a formula over 18 font roles, four scale/
accent payloads, six axis relationships, present point-outline ratios, guide
relationships, and 18 composition relationships, then calls derived `clone`
(`theme/resolved.rs:616–631`). This is a **handle-operation formula**, not bytes
deep-copied or measured instructions. The font environment and text cache are
shared too; scalar fields and ordinary copying are not instruction-priced.

`resolved_theme` calls the unbudgeted `builtin_font_stack`, which allocates five
named Strings, a six-family Vec, and an Arc, then shares it into the roles
(`resolved.rs:1666–1695`, `canonical.rs:905–919,531–589`). New font-environment/
text-cache synchronization shells also allocate. Their fixed costs are small,
but they are absent from the claimed complete preset byte certificate. Later
census copies do not make the original retained stack allocation disappear.

`RelativeSpacing`, font stacks, and theme components expose derived `PartialEq`
and `Clone` internally as well as admitted `same`/`share` methods. A trait does
not automatically route into the ledger. The actual child-cache path does call
`theme.same` (`decorations/plan/children.rs:79–89`), and exact pointer equality
for immutable payloads is a valid fast path. Mutable caller text must still be
compared by value; current profile lookup and its mutation regression address
that distinction. Do not infer universal priced equality from those tests.

`RelativeSpacing::{new,ratio,from_pixels}` create local generation resources;
their `_in` variants accept the operation. The standalone wrappers are private,
documented as awaiting exposure/assembly, while inspected production preset and
consumer paths use the supplied ledger. `PointStyle::new` and
`PointOutline::publication` convenience construction here is `#[cfg(test)]`.
This audit found no basis to report those test constructors as a production
root-reset bug. Conversely, types alone do not prevent a future production
caller from selecting the wrong constructor.

### F12 — Medium: observable caches are not a uniform occupancy census

Text query insertion, lazy paragraph analysis, coordinate intrinsics, Matrix
ticks, child results, root-frame results, incremental font selection, normalized
stacks, and palette entries do have explicit cache-entry admissions. Calls are
generally claimed before their cache lookup, hits do not insert, and independently
owned payload storage has additional claims. Concrete owners include:

* `theme/resolved.rs::cache_text_query` / `TextQueryCache`,
  `layout/text.rs::{ParagraphWhole::wrapping,intrinsic_in_call}`;
* `layout/interior/coordinate.rs::Intrinsics::prepare`,
  `plots/matrix.rs::matrix_ticks`, Heatmap intrinsic cache;
* `decorations/plan/children.rs::ChildPlans::{measure,plan}`,
  `orchestrator/lifecycle.rs::PreparedPlotInputs::root_frame`;
* `canonical.rs::resolve_pair_with_stage` / `resolve_stack_names_into`,
  `theme/scale.rs::extend_categorical_palette`.

But the batch `prepared_pairs` vector and selected-unit retention do not claim
cache entries, whereas incremental selected scalars do. Whether sealed preflight
tables are to be classified as prepared state or cache must be explicit; the
counter is not “all memoized records.” Palette growth replaces an existing
entry's colors without a new cache-entry claim, while root-frame replacement
does claim one. Both still spend applicable work/storage. These choices are
observable accounting semantics, not simply live occupancy.

Child cache `.retain` replacement scans are unpriced
(`children.rs:291–292`), even though its preceding hit lookup is priced.
Some prepared-panel lookups still use bare `.find` in Qq, Regression, Dumbbell,
and Estimate intrinsic/measurement paths, while
`layout/interior/preference.rs::panel` uses admitted traversal. These are further
first-party lookup gaps, not dependency internals.

## 5. Byte accounting: exact requests, estimates, hidden capacity

“Admitted bytes” includes several different accounting classes:

| Class | What is actually charged | What is not established |
| --- | --- | --- |
| Retained font/config file contents | Metadata length before bounded read; font read and SHA-256 stream share one byte admission (`canonical.rs:3712–3793,4194–4230`) | File header/path/Arc/metadata/database costs are not all included. It is not just the font actually selected for text; canonical discovery retains discovered supported font files. |
| Source Vec and String requests | Checked `needed × sizeof(T)` or string length on growth; clone helpers charge requested payload length (`data/source.rs`) | `len` is not allocator capacity. Nested payloads require their own claims. Error-message allocation and std internals are not covered automatically. |
| Arc wrappers | `source_arc` uses a padded layout of two atomic counts plus `T` (`183–194`); FontStack and text-query source have corresponding explicit header claims | Cloning an Arc does not duplicate its Vec/String buffer. Some other Arc creation sites have no such claim. These layout calculations do not include allocator metadata. |
| Replacement storage | Source/font-cache/text-record helpers charge the **entire requested replacement**; Unicode and feedback usually grow geometrically and charge the complete new target | Old allocations remain charged after free. Actual allocator growth can occur in place or use excess capacity. This is not peak-live memory. |
| Final capture growth | `FinalStage::reserve_capture` doubles requested capacity but charges **capacity delta**, not whole replacement (`kernel.rs:541–564`) | Inconsistent with the universal complete-replacement wording. It cannot account for old/new coexistence as a full replacement stream. |
| Imported owned census String | `new_with_stage` can charge existing `.capacity()` when taking ownership (`requirements.rs:249–258`) | Admission occurs after the semantic producer allocated it. Capacity admission is not proof that the initial allocation was guarded. |
| Log10 scratch | `temporaries × 16 × ceil((2p+128)/64)` per admission (`numeric.rs:174–202`) | Conservative native-limb/capacity formula, not observed BigUint allocation bytes; dependency-internal multiplication/division scratch is not counted. |
| Relative-spacing scratch | `operands × (16 × limbs + sizeof(BigUint))`, plus retained payload/header formula (`spacing.rs:582–625`) | Includes operand headers even when they are stack temporaries; mixes estimated work storage with retained heap storage. No proof of allocator/RSS upper bound. |
| Exact color scratch | Five nominal 4096-bit operands = 2,560 bytes per comparison (`color.rs:793–800`) | Different convention: no measured capacities or dependency scratch. The pre-admission midpoint issue remains. |
| Bidi/shaper/parser output | Some dependency-input/output arrays are estimated or actual-sized before first-party retention; shaped glyph arrays use actual `shaped.len()` | Engine allocations can already have occurred. Small input count does not uniformly bound GSUB output or outline complexity. |
| Raw values and terminal buffers | Caller-owned `NumericValues`, `CategoryValues`, `BooleanValues`, `SourceRows`, public statistics, SVG/PNG terminal buffers have separate construction/publication paths | They are not fully included in the root ledger. Do not advertise 512 MiB as whole-process admission. |

For a successful `try_reserve_exact`, the requested capacity is a lower bound on
the resulting available capacity, not a portable equality. `Vec::capacity()`
does not reveal allocator metadata, arenas, rounding, or transient old/new blocks.
The code generally claims the requested size before reservation and does not
reconcile a larger actual allocation afterward. Arithmetic/fallibility checks
remain useful; they do not close that measurement gap.

Cumulative replacement charging can greatly exceed live storage. Illustratively,
if an eight-byte element vector obtains capacities `1,2,…,10,000` through
incremental exact source reservations, the logical byte sum is
`8 × 10,000 × 10,001 / 2 = 400,040,000`, while its final payload is 80,000 bytes.
At 12,000 such capacities the sum would be 576,048,000, beyond the byte cap,
although final payload would be only 96,000 bytes. These are conditional
arithmetic examples, not measured allocator behavior. Geometric full-buffer
charging bounds the sum much more closely to final requested capacity; feedback
already uses that approach. Delta charging describes a different metric.

Because some charges are conservative/repeated and some allocations are omitted,
the global byte number is **neither a proven lower bound nor a proven upper bound
on real peak/live/cumulative allocator bytes**. It is exactly a sum of admitted
logical requests and estimates. The same applies to global records versus actual
retained elements.

### 5.1 Original f64 data and row identity

`values.rs` owns immutable Arc-backed numeric slices, category dictionaries/codes,
Boolean slices and optional validity bitmaps. Constructors collect/allocate
before any root exists. Plot clones share those scientific inputs; source
preparation materializes new admitted vectors where explicitly threaded.
`builder/admission.rs::Plot::admitted_clone` exhaustively lists Plot fields and
prices heap-owning configuration copies, not another copy of the source Arcs.

One million f64 values have 8,000,000 bytes of numeric payload; that is **not** a
maximum Plot memory calculation. Names, category strings/codes, nullable
construction temporaries, row maps, multiple channels, prepared tuples, font
files, marks, caches, geometry and output add different storage. Root topology
alone neither pays those bytes nor covers their multiple processing passes.

`RowTopologyAdmission` binds original Plot identity and shared counter identity.
Facet children reuse original row admission without recounting rows; source-row
IDs are retained separately from local indices. Structural source rows include
null/filtered rows; selected marks need not have the same cardinality. Matrix
has its own multiplicative census (`n²` cells and `n(n−1)r` off-diagonal marks,
plus density targets), rather than a generic one-slot-per-f64 rule.

## 6. First-party producer coverage

The source dispatch, not just the earlier documentation, reconciles **25 Plot
kinds → 16 mark-payload families**: `layout/interior.rs:40–44,93–119` and
`plots/shared_x/prepared.rs::PreparedReference` / `prepare_plot`. Eight kinds use
standard coordinates, twelve one categorical axis, five specialized grids.
Canvas admits 24 kinds (not Matrix), generic facets 23 (not Matrix/Marginal), and
TextPanel is separate. These exclusions are checked in
`CompositionChildPolicy::validate_child`; they are not accounting shortcuts.

The following census records admission granularity and its limits. A covered
family is not a blanket claim that every line in it is priced.

| Family / kinds | Inspected producer boundaries | Accounting coverage and specific limits |
| --- | --- | --- |
| Scatter | `plots/scatter.rs::PreparedScatter::new`, `measure_marks`; common data helpers | Original rows, aligned extraction, separate range/group passes, selected points, geometry. Bare legend census and some semantic label work remain outside admission (F4). |
| Line / Trajectory / LinePath | `plots/line.rs` preparation/grouping, `measure_marks`; `shared_x/prepared.rs` | Source order, collective grouping, coarse admitted sort, paths/bands and projection payloads. Categorical Trajectory validation/facet level extraction can use unpriced normalization/deduplication paths. |
| Histogram | `plots/histogram.rs::{HistogramSemantics::new,measure_marks}`, bin-count/edge/KDE helpers | Validation/edge passes, bin assignment, bins/rugs/KDE storage and selection. KDE uses 128 grid points. Input-pass admissions and early-exit validation are not actual instruction counts. |
| Density | `plots/density.rs::prepare_density_ordered`, `density_support`, `visible_curve`, polygon clipping | Group scans, six successful support/statistic passes, 128–2,048 point grid, `ng` KDE work, selected curve geometry. Grid generation at `615–619` executes arithmetic without its own visit claim before later validation claims. |
| Qq | `plots/qq.rs::{PreparedQq::new,prepare_qq_admitted,project}` | Finite extraction, coarse numeric sorting, normal-quantile generation, reference and point geometry. Normal inverse approximation's arithmetic is not priced individually. Prepared-panel `.find` is uncharged. |
| Regression | `plots/regression.rs::PreparedRegression::new`, `project`/measurement; `stats/regression.rs::OlsFit` | Ten scalar scans are prepaid, fit retained, 40 optional band samples. Student-t inverse-CDF dependency work has only an outer sample/call unit. Prepared-panel lookup unpriced. |
| PointInterval / Coefficient / Forest | `plots/interval_estimate.rs::PreparedEstimate::new`, tuple/label/log transforms, `measure_marks`; `point_interval.rs::prepare` | Rows/interval fields, categories, exact labels and selected geometry; Forest invokes certified Log10 on the root. Explicit/value order uses whole-list and `4n log n` preprices, not common sort semantics. |
| Dumbbell | `plots/dumbbell.rs::{prepare,project,measure_marks}` | Original rows, category/group duplicate/position scans, two endpoint ranges, paired projection and point/connector geometry. Whole-list precharges and bare panel lookup remain. |
| Bar / HorizontalBar / StackedBar / StackedHorizontalBar | `plots/bar_data.rs::{prepare,generated_interval,percentile_bootstrap,stable_mean,stable_sum}`, vertical/horizontal selectors | Groups, signed stacks, interval statistics, deterministic bootstrap, point layers, labels and significance targets. RNG rejection draws admitted individually; seed hashing is byte-priced. Group searches often preprice tails. Significance owner has F5 gaps. |
| Boxplot / Violin / Raincloud samples | `distribution_sample.rs::prepare_scoped`, `boxplot.rs::box_summary_with_source`, `violin.rs` KDE/statistic preparation and selection | Shared categorical grouping, sort/statistic/KDE passes, point/curve/summary payloads. Box whisker/flier overcount and group whole-list overcount confirmed. |
| Raincloud supplied geometry | `plots/violin/precomputed.rs::prepare`, `project_cloud_and_summary`, `project_strip`; input `validate(Some(data))` | Explicit source density/summary/strip validation and copied vectors. Hue `.any` is whole-list-priced; no KDE is invented for supplied curves. |
| Interaction | `plots/interaction.rs::{prepare,measure_marks}` | Condition/trace/group scans, sparse membership, optional bounds, connectors and point payloads; source counts and style mapping share root resources. Several position/filter checks precharge full lists. `interaction/native.rs` contains tests, not another runtime renderer. |
| Heatmap | `plots/heatmap.rs::{prepare,prepare_intrinsics,assigned_columns,assigned_rows,select_cells}` | Original rows plus declared grid, normalized levels, merged missing rows, highlights/value text, color mapping and retained intrinsics. Binary search bounds `ilog2(len)+2` are potential work. Whole-list order/annotation tests overcharge; configuration/census paths have F4 gaps. |
| Matrix | `plots/matrix.rs::{prepare_axes,build_groups,validate_group_kdes,prepare_density,matrix_ticks,select_diagonal_kde}` | `n² + n(n−1)r` topology slots before density targets, five-scan Scott, distinct min/max, **60**-point KDE per variable/group, selected cell vertices; tick calls before hit/miss and insertion separately admitted. Array capacities and coarse list charges are not actual visits. Errors from KDE are wrapped as `Error::Data` at `1362–1365`, although still terminal, so not all quota errors preserve an external typed resource class. |
| Marginal | `plots/marginal.rs::{prepare,finish}`, support/grid/linked-panel selection | Three subpanels, original grouped pairs/source IDs, two density targets per group, **256**-point density grids, repeated dependent grid evaluation and projection. Configuration/legend census still has bare-data route; successful Scott pass counts are real, not duplicate scans. |
| Beeswarm | `plots/beeswarm.rs::{prepare,natural_body_height,measure}`, `packing::pack` | Rows/categories, numeric color mapping, tangencies, actual candidate/prior-point collision checks, selected offsets. Packing can run for natural sizing and assigned measurement; each execution is real work. Candidate capacity can overcharge records; common heapsort does not count comparisons. |
| Waterfall | `plots/waterfall.rs::{prepare,measure,plan_annotations}` | Signed deltas/endpoints, categories/bars/connectors/total targets, source IDs, annotation shelf/collision work and selected geometry. Scientific zero and clipped-total policy are semantic, not budget fallback. |

Shared non-Plot producers were also traced:

* **Axes/numeric labels:** `layout/panel/sources.rs::{Sources::next,
  NumericTickPlan::measure_in_call,format_label}` and axis frame/grid. Source
  seeds have a separate 10,000-tick bound; additive interval enumeration has at
  most `4 × 633 + 1 = 2,533` source slots, integer-log intervals `4 × 20 + 1 = 81`.
  Precision trials run 0–17 excluding 1. These finite source/format policies are
  not eight-root-round or eight-guide-trial limits. Text plans consume their own
  shared allowances; some adjacent-label comparisons lack an independent scan
  charge. Exact scientific tick lists are not silently coarsened on quota failure.
* **Guides, root prose, TextPanel, headers:** `decorations/legend.rs`,
  `legend/continuous.rs`, `title_block/plan.rs`, `canvas/{text_panel,plot_text,
  tracks,sizing}.rs`, and facet planning/tracks. They use common text, exact
  spacing, source storage, geometry and parent feedback. Thus both the valid
  common guards and the common omissions apply. “No global layout search” does
  not mean their local trials, legal-break analysis, and nested traversals are
  constant-cost.
* **Annotations/layers/collision:** `decorations/panel_text.rs`,
  `orchestrator/layers.rs`, `render/geometry.rs`, and significance. Panel-text and
  Beeswarm pair guards do not prove all significance/range/geometry work priced.
  `planned_text_ink_bounds` traverses retained lines without its own environment;
  charging one positioned-text entry is not charging every line visit.
* **Parent/range composition:** Canvas/facet declared topology, common
  `layout/kernel/tracks.rs`, shared-axis preparation, child feedback/cache,
  exact-range bit comparisons, and source row tickets. Source range min/max and
  projection are primarily f64 passes; no uniform cost-per-floating-point-op
  claim is made. Bare panel lookups and track record omissions remain.
* **Scales/colors/themes:** `theme/{scale,scale_values,color,spacing,resolved}.rs`.
  Palette selection/maximin loops, sampled continuous guides and exact RGB
  mapping have guards. Stop/key comparisons, fixed color searches, BigUint
  arithmetic and immutable-handle queries have different prices. Palette cache
  lifetime is operation-local; no global font/palette cache is introduced.
* **Capture/publication:** canonical primitives, deferred capture, counted SVG,
  Null validation, outline conversion and PNG. Zero shaping/selection during
  replay does not mean zero validation, copying, formatting, allocation, path
  extraction, rasterization or encoding.

## 7. Real performance implications, without inventing measurements

### 7.1 The million cap is still useful, but not a total-cost bound

It can stop actual NFC quadratic ordering, Beeswarm collision pairs, bootstrap
draws, KDE pairs, repeated text trials and exact-number refinement. The absence
of global layout search does not make those guards pointless. Conversely,
bounded root rounds do not bound one expensive pass, and a one-unit BigUint or
shaper call can hide substantial work. Log10 doubles precision; that is
exponentially growing precision by refinement ordinal, not proof that the whole
renderer has exponential runtime. Significance doubles its numeric span; that
does not make its growth loop exponential in operation count either.

Actual performance depends on at least:

* Number/size/metadata of installed fonts, filesystem traversal and read latency,
  face parsing, scalar probes and outline complexity. Canonical preparation
  includes supported discovered files, not merely the selected Latin font.
* String length, UTF-8 width, number/order of distinct scalars, mark sequences,
  fallback stack, bidi/script boundaries, legal endpoints, and query reuse.
* Distribution grid size, observation/group counts, collision density, number
  of category identities, bootstrap resamples, and scientific numeric magnitude.
* Exact arithmetic operand widths and how many times layout queries reconstruct
  equivalent rational lengths, as well as dependency multiplication/division
  algorithms and allocator behavior.
* Cache hit verification, source hashing, value equality, replacement and
  retained payload sharing, not merely hit count.

Costs requiring attribution beyond the aggregate study:

* `FontStack::byte_work` is an O(families) metadata scan before charging; CSS can
  perform validation/quoting scans beyond the declared two input passes.
* Every per-entry or per-byte `claim_visits(1)` is a mutex acquisition. The font
  environment wrappers can also acquire an RwLock and clone an Arc before locking
  the generation counter. The bookkeeping is excluded from its own visit total.
  Batching known passes reduces lock calls, but then often charges unused tails.
  The study measures aggregate claim-path differences and invocation counts;
  it does not isolate mutex/Arc latency or multi-thread contention.
* Exact spacing compares/multiplies/rounds BigUints for ordinary metric queries.
  A binary64 minimum-subnormal dyadic uses a 1,075-bit denominator: its four-
  operand dyadic admission alone prices `4 × 17² = 1,156` work units, plus the
  retained header admission. This is a type/representation example, not a
  measured latency or proof that a full layout using it can succeed spatially.
* Current text-cache hit verification can revisit all 18,432 caption bytes while
  doing no shaping. The old “two visits” table cannot estimate that cost.
* Exact source growth/repeated full-buffer admission can reject small live
  structures, while uncharged prepared font buffers can allocate large capacity.

Changing prices can change which inputs are accepted without changing CPU work.
Removing a duplicate pass can improve both, but only timing/allocation evidence
can establish the size of the physical improvement. Optimizing until a fixture
fits just under one million risks optimizing the accounting rather than the
renderer, and can conceal gaps by making them “necessary” for acceptance.

### 7.2 Confidence in existing evidence

High confidence from source: independent inclusive counters; shared Arc identity;
no counter refunds; ordinary current root creates its resources before prepare;
typed internal arithmetic/limit errors; distinct fit/fatal routes; source-row
ticket binding; finite bounded guide ordering; PNG-local gates; explicit producer
omissions/overcharges above.

Existing boundary tests such as `generation_limits_are_independent_inclusive_and_shared`,
`root_visits_are_atomic_shared_and_cannot_be_rebound_after_preparation`,
`source_levels_charge_only_examined_entries_and_sort_charges_input_once`,
`collision_pairs_share_the_font_processing_cap`, and Matrix cache tests provide
useful narrow assertions. They cannot establish a producer census by themselves.

Several “exact allowance” tests run an operation, read its counters, then repeat
with that allowance and allowance-minus-one; for example Beeswarm packing
`candidate_visits_bytes_and_dense_refusal_use_the_operation_ledger`. This proves
repeatability/refusal at existing claim sites, not that all real work was claimed
or that units are calibrated. An uncharged loop remains invisible in both runs.
Independently calculated constructor fixtures are stronger for their named
expressions, but still omit unpriced allocations such as the built-in font stack.

`Observations` stores integer ledger totals and separate `Duration` timers
(`orchestrator/measurements.rs:224–255,423–460`). Its capture counters cover a
named list of test instrumentation, not all executable operations. A zero
capture counter, a stage-gate forced allocation refusal, successful compilation,
or matching consumed statistics is **not** the complete accounting contract.
This audit did not compile; it makes no claim of a new assertion failure or a
passing Clippy/test gate.

The [performance study](resource-accounting-performance.md) now supplies
same-base release-test wall-time distributions, successful/refused output routes,
claim frequencies, and fresh-process whole-process RSS/coarse CPU observations
for its named cases, including the complete 2,048-word reference. Its one-thread
execution and release profile are documented, not unknown. Remaining gaps are:

1. All-family/adversarial coverage and an uninstrumented per-stage or dynamic
   call-stack cost profile. The study's separate diagnostic stage timers include
   instrumentation and can nest; they are not an additive timing decomposition.
   A stripped production-binary comparison is also absent.
2. Allocator traffic, allocation counts/bytes, peak live heap, and attribution
   of residency to discovery capacity, prepared-pair clones, and outline buffers.
   The available whole-process RSS does not answer these questions.
3. Isolated lock/Arc costs, multi-thread contention, and per-hot-path attribution
   for source verification and exact spacing. Aggregate bookkeeping deltas and
   file/line claim frequencies do not establish those costs.
4. Adversarial distinct-scalar/long-unit text, large family/alias lists,
   configuration-before-topology work, complex glyph outlines, many significance
   categories, unused reservation tails, and all resource refusal boundaries
   across those concrete paths.
5. Dependency-internal parsing/shaping/BigUint/statrs/raster/encoder allocation
   and complexity. Only call sites and locked versions were used here; this is
   not a dependency-internals proof or a fresh official Unicode conformance run.
6. Platform fontconfig/non-Linux discovery differences and font-environment
   sensitivity of quota headroom. The canonical ordering within a fixed
   environment does not make a host-independent resource total.

## 8. Documentation claims needing reconciliation

The older report is useful historical evidence and already disclaims CPU/RSS
guarantees. It should not be treated as a blanket current implementation audit.

| Documented claim in `docs/resource-admission.md` / spec 05, or identified inference to avoid | Current disposition |
| --- | --- |
| “Actual processing visits” across all producers | Too broad: mixed bytes/scalars/potential comparisons/limb estimates, plus uncharged first-party loops. |
| No processing surcharge for hashing/copying/materialization | Contradicted by current font/query/hash/scale/bootstrap paths; keep historical statements attributed to their versions. |
| Variable lists charge only examined entries | True for `source_position`, face search and several collision owners; false for numerous full-list preprices and bare searches listed above. |
| Actual records and checked bytes before storage | True for many source/census/cache helpers; false for unpriced prepared font storage, prepared-pair clone, significance arrays, final element records, and duplicate label-domain claims. |
| Complete replacement capacity charged on growth | True for source/font-cache/Unicode/feedback helpers; final capture uses a delta. Allocator capacity remains hidden. |
| No mandatory million-entry discovery allocation | Directly contradicted by the active `reserve_system_discovery` call chain. |
| Glyph outline elements require record/storage admission | Missing in `SvgGlyphPath` / dynamic outline buffers. This does not independently require per-command visits for excluded formatting, or a shared PNG-derived SVG byte cap. |
| Inference to avoid: million-visit boundary acceptance proves workload suitability | The documents already disclaim general speedups. Acceptance proves only that this priced operation fit this policy in that environment; it cannot calibrate suitability or efficiency. |
| Calls, caches, topology, records and bytes are separate | Confirmed at ledger level; their complete producer coverage and naming still need the qualifications above. |
| Quota refusal is terminal, not content dropping | Confirmed in the common fit/fatal separation; external error variants/messages sometimes erase the internal resource classification. Optional memo stop at 256 is a separate intentional policy. |

## 9. Ranked actions for the integrated performance work

These are recommendations, not code changes performed by this audit.

1. **Separate the contract from the score before optimizing it.** Name the
   existing visit metric as logical admission/work units unless it is brought
   back to one precisely defined actual-visit unit. Name bytes as cumulative
   admitted payload/scratch estimates, and distinguish input bytes, reserved
   capacity, actual materialized records, estimated arithmetic work, calls and
   cache insertions. Preserve the distinct structural/representability and
   PNG-terminal contracts. Do not use “under one million” as a speed target.
2. **Close first-party omissions with explicit ownership.** Highest-value
   accounting fixes are prepared font/discovery buffers and pair clones (F1/F2),
   scalar/unit/census and bare-data traversals (F3/F4), significance (F5), and
   outline/capture element policy (F6). Decide which retained preflight tables
   count as caches. Make admission visible before each covered allocation or
   semantic pass; phase-gated reservation alone must not certify it.
3. **Correct proven duplicate/potential charges separately.** Remove the
   label-domain same-slot double debit as a logical correctness issue. Specify
   whether filtered/candidate/flier capacity is a reservation metric or actual
   record metric. Reconcile full-tail early-exit preprices, label alias/22×
   prices, and the three inconsistent sort conventions. PF4 deliberately prices
   the common heapsort by input count; reconciliation does not require charging
   every comparator execution. Preserve the explicit storage-only work
   exclusions, and do not delete real x/y, count/fill, min/max, validation/statistic
   passes to lower the total.
4. **Use the completed study and close its remaining measurement gaps before
   selecting thresholds.** `docs/resource-accounting-performance.md` supplies
   same-base timing, refusal, equality, claim-frequency and whole-process memory
   evidence. Extend that evidence where allocator attribution, actual hot-path
   profiles, concurrency, other fonts, and unmeasured families matter. Keep
   successful completion distinct from refusal and separate root/layout costs
   from outline/PNG terminals. Neither report derives a defensible new numeric
   threshold.
5. **Optimize demonstrated physical hot paths while preserving semantics.**
   Candidate investigations include avoiding quota-sized discovery reservation,
   admitted immutable sharing instead of prepared-pair deep copies, indexed
   scalar/identity lookups, reused exact metric consequences, and fewer lock
   acquisitions for truly known passes. Rank them with physical measurements;
   do not assume the largest charged term is the slowest term.
6. **Strengthen evidence at the gaps, not just counter replay.** Future focused
   checks should independently count comparisons in actual-list-visit classes
   and materialized elements, verify preallocation refusal at actual owners, and
   cover mutable value equality, fresh versus shared operation identity, rejected trials,
   cache replacement, and terminal outlining. A failing existing acceptance
   assertion is a signal to investigate the workload/cost change, not to shorten
   the fixture, repin a count, or redefine a stage counter as proof of safety.

**Bottom line:** retain confidence in the structural root/ownership architecture
and synchronized inclusive claims, but withhold confidence in a universal
actual-work/actual-storage census or a calibrated performance budget. The report
identifies concrete corrections and measurement questions; it does not approve
the current accounting as complete, prescribe a renderer rewrite, or infer
performance from admission totals.
