# Explicit Alpha For Aggregate Uncertainty And Summaries

Source baseline: `cf0ffa1`. Bar, HorizontalBar, Boxplot, Violin and sampled/supplied
Raincloud complete the **17 of 17 scientific stroke consumers**.

## Actual Units And Paint Ownership

- Ordinary bars resolve uncertainty alpha by the raw category and configured
  positioned-series fields, plus any selected actual owning facet fields. Sample
  aggregation, generated intervals, precomputed bounds, ordering and scientific
  ranges retain their existing owners. A hidden or absent interval validates the
  binding schema but consumes no alpha member. Transparent present intervals must
  still resolve completely.
- `PreparedBar` retains the selected member/byte. The final uncertainty style now
  belongs to each `BarShape`, replacing the former global payload style throughout
  draw, retained geometry and automatic numeric-ink expansion. Intrinsic cap pitch
  and numeric ink use that same final stroke. Stacked bars gain no line capability.
- Sample distributions retain a separate aggregate line operand beside the original
  observation operands in `DistributionGroup`. Boxplot summaries and Violin inner
  whiskers/caps/median replace only their stroke alpha. Raincloud replaces the
  inner median stroke in both sample and supplied forms. Body fills, inner-summary
  fill/outline, raw/flier/mandatory-strip points and supplemental layers retain
  independent paint ownership.
- Supplied Raincloud resolves exact category/optional-hue keys directly from its
  geometry dimensions and groups, independently of its zero-row synthetic input.
  Group/point order and supplied coordinates remain unchanged.
- Intrinsics use actual final summary ink, including theme-zero strokes made
  visible by data alpha. Boxplot categorical clearance includes the complete
  median span, and Violin includes both summary and median widths. Selection,
  automatic clearance, capture and replay share the existing final primitives.

## Focused Evidence

**Six library contracts passed** using nonzero `cargo test --locked --offline --lib`
filters:

- `plots::bar_data::alpha_tests`: both orientations, eight sample rows and four
  explicit category/series/facet units, plus an unused pooled tuple containing
  `U+10FFFF`. Actual scopes retain bytes `[128,64]` and `[0,230]`; independent
  expected means are `[2,3]` and `[6,7]`. A theme-zero 40px uncertainty stroke
  becomes visible with the selected byte, preserves RGB `(17,41,67)`, changes
  actual hard ink, and leaves body paint unchanged. Selected marks reread no source
  columns. Incomplete keys fail before additional font discovery, including when
  the supplied real key is transparent. Absent intervals consume no members and
  an observation binding is refused even for an absent interval.
- `plots::boxplot::alpha_tests`: duplicate outlier slots/paints remain independent
  of a category-keyed summary byte 128. A theme-zero 100px summary drives actual
  intrinsic ink; all five summary strokes preserve their RGB and selected alpha,
  while body fill and point paints retain their independent values.
- `plots::violin::alpha_tests`: reversed raw category/positioned-series selector
  tuples produce summary bytes 0 and 230 while original point membership, source
  jitter, outlines and contiguous paint order remain intact.
- `geometry::validation_tests::supplied_geometry_preserves_support_and_rejects_nonfinite_values`:
  supplied zero-row Raincloud uses category/hue keys for summary bytes `[128,64]`
  independently of point bytes `[0,230]`, retains support, and rejects nonfinite
  geometry.
- `theme::mark::tests::sampled_and_supplied_strips_keep_their_own_style_and_guide_when_raw_roles_change`:
  both inputs retain independent strip bytes and the explicit Single summary byte
  64 when raw-point roles change.
- `layout::interior::preference::tests::categorical_preferences_keep_unavailable_slots_and_occupied_grouped_bars`:
  neighboring categorical hard/preferred sizing and occupied-slot ownership pass.

The new bar alpha contract replaces
`layout::interior::preference::tests::private_bar_outline_changes_preference_but_not_exact_root_box`.
That narrower role-only preference test is superseded by the actual keyed
uncertainty-paint/intrinsic contract; independent outline provenance remains covered
by the existing discrete source tests. No deletion is banked. Discovery remains
**671 library tests including 17 ignored**; the ignored-only listing was executed.

**Ten existing public integration contracts passed**:

- `categorical`: grouped vertical/horizontal bar output, sparse faceted series
  dodge slots, duplicate-row stacked sums, Violin inner whiskers and Boxplot fliers.
- `distributions`: supplied/sample Raincloud recorded-mark parity, supplied repeated
  categories with distinct hues, positioned raw-overlay color/dodge order, and
  composed Canvas overlay lifecycle.

Strict `just lint`, `just fmt-check` and `git diff --check` passed. Fill consumers,
unified guides/parents, public API cutover and integrated full/release verification
remain subsequent work.
