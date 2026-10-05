# Validated compound mark paints

## Current value schema

`src/theme/mark_paint.rs` owns the nine current compound construction values from
spec94 §5. `src/theme.rs` makes them reachable inside the crate. The live
`ResolvedTheme` factory and existing aggregate appearance fields retain these
values directly. All values implement `Clone + Debug + PartialEq`;
`PointOutlineTheme` also implements `Eq` over its exact rational relationship.

| Former representation | Current retained value | Construction inputs |
| --- | --- | --- |
| `StrokeAppearance` | `StrokeTheme` | `Color`, positive finite CSS `f32` width, closed `StrokePattern` |
| `BracketStrokeAppearance` | `SolidStrokeTheme` | `Color`, positive finite CSS `f32` width; solid topology is fixed |
| `FillOutlineAppearance` | `OutlineTheme` | `Color`, positive finite CSS `f32` width |
| Backend outline used as point appearance | `PointOutlineTheme` | `Color`, positive `RelativeSpacing` width/radius relationship |
| `FillOutlinePaint` plus width | `DataOutlineTheme` | Complete fixed `OutlineTheme`, or positive finite mapped width and independent `u8` alpha |
| `DataFillAppearance` | `DataFillTheme` | Fill `u8` alpha, `Option<DataOutlineTheme>` |
| `FixedFillAppearance` | `FixedFillTheme` | Complete fill `Color`, `Option<OutlineTheme>` |
| `PointAppearance` | `PointTheme` | Fill `u8` alpha, closed `PointShape`, `Option<PointOutlineTheme>` |
| `DataLineAppearance` | `DataLineTheme` | Positive finite CSS `f32` width, `u8` alpha, closed `StrokePattern` |

The former type definitions, re-exports and factory helper `stroke_appearance`
are removed. Production compound construction uses the validated constructors.
Existing aggregate groups and radius fields keep their current scope for the
later complete-group leaf.

The values and scalar fields remain externally private. Existing internal
geometry and negative fixtures can access `pub(crate)` scalar fields. The
`DataOutlineTheme` choice and payload enum are fully private; callers inspect
`fixed_stroke()`, `mapped_width()` and `mapped_alpha()`, then replace the whole
validated choice through its parent. There is no raw choice variant input.

## Construction, inspection and edits

The eight ordinary `new` constructors return the crate's `Result<Self>`.
`DataOutlineTheme::fixed(OutlineTheme)` consumes an already validated outline and
returns `Self`; `mapped(f32, u8)` validates width and returns `Result<Self>`.

Every retained field has the spec94 getter and consuming `with_field` edit:
scalar getters return their Copy value, relative-spacing getters borrow the exact
value, and optional-outline getters return `Option<&T>`. Edits return
`Result<Self>`, reconstruct the complete value and replace only their named
input. Parents also validate present nested outlines. Repeated edits replace
their own field, and editing a clone preserves the source. `DataOutlineTheme`
has precisely its explicit constructor/inspection vocabulary from §5; it does
not expose kind-changing scalar edits.

Widths reject zero (including negative zero), negatives, NaN and infinities.
Positive subnormal widths and `f32::MAX` retain their exact source bits at
construction; normalization, representability and containing fit keep their
existing later owners. Point-outline relationships require a nonzero exact
`RelativeSpacing`; there is no new upper bound or minimum painted width.
Alpha is one inclusive byte, including 0 and 255. Complete fixed colors may be
transparent. There is no universal contrast validation on these values.

The one existing `StrokePattern` enum retains exactly Solid, ShortDash (`3 2`),
MediumDash (`4 3`) and LongDash (`5 3`). Both patterned values consume this same
enum. Solid bracket strokes and outlines have no pattern field/edit. The enum's
current backend location and the narrow exact-style/public tuple/opacity inputs
retain their scheduled public-cutover owners.

Construction edits awaiting complete customization have method-local
`cfg_attr(not(test), expect(dead_code, reason = ...))` annotations. Every getter
and edit is exercised by the current component fixtures. These values have no
module-wide dead-code allowance.

## Paint, absence and operation ownership

Mapped fill and line adapters take the current opaque RGB mapping and install
their role alpha once. A mapped outline takes the same RGB and installs its
**independent** outline alpha once. A fixed outline retains all its own RGBA
channels. Changing fill alpha therefore cannot recolor or multiply the outline.

Line bands, regression bands and density areas now retain structural `None`
outlines. Their built-in fill alphas remain 46, 64 and 56. Present body outlines
retain their original paints and `f32` bits; mapped Marginal/Matrix outlines
explicitly retain alpha 255. Primary/raw point defaults retain alpha 209, Circle
and the white outline relationship with the original binary64 `0.1` bits.

An absent fill outline materializes as transparent neutral black and width
positive zero. `FillStyle::new` applies that one operational normalization to all
zero-width backend fills, so discarded outline paint cannot enter guide equality,
cache or selected-feedback identity. Width validation still runs on transparent
fills: NaN, infinity and negative width remain errors. Present transparent paints
remain complete present values rather than being inferred as structural absence.

`PointOutlineTheme::publication` constructs the exact relationship on its supplied
operation ledger. The preset factory calls it for the same two existing point
roles. Backend `PointOutline` remains a distinct operational operand; its ratio
shares the foundation's same immutable payload when materialized, with no second
ratio construction. The existing `width_for_radius` helper consumes the final
positive radius and actual operation resources. Existing outline visibility can
suppress a present outline once; an absent outline stays absent even when that
policy is enabled. Backend `PointOutline::publication` is now only a test adapter
to the current foundation constructor.

The Plot/Canvas constructor admission fixtures retain their exact work/byte
expectations and refuse before fonts/backend begin. Theme clone admission still
covers its eighteen existing font handles with the prior refusal boundaries.

## Consumer and source migration

- `src/theme/{mark_paint,resolved}.rs` and `src/theme.rs`: sole compound values,
  checked preset assembly and existing aggregate consumers.
- `src/theme/mark_paint/tests.rs` and the test-only sharing witness in
  `src/theme/spacing.rs`: exhaustive field/edit, alpha, width and ratio evidence.
- `src/render/backend.rs`: operational point-outline test adapter and canonical
  absent fill-stroke metadata.
- `src/significance.rs`, `src/orchestrator.rs`, `src/plots/bar_data.rs`: solid
  bracket inputs consumed through their typed getters.
- `src/plots/heatmap.rs`, `src/decorations/legend/{descriptor,continuous}.rs`,
  `src/layout/interior/sizing.rs`: current outline types in retained profiles,
  missing keys and fixtures.
- Point, body, ribbon, density and guide fixtures migrate to optional validated
  outlines in `src/plots/`, `src/decorations/{legend,plan/tests}.rs` and
  `src/layout/interior/preference.rs`. Former outline-width mutations reconstruct
  their actual fixed/mapped choice, including explicit mapped outlines when a
  fixture adds one to a default borderless role.
- `src/canvas/admission_tests.rs` and `src/orchestrator/lifecycle.rs`: constructor
  boundary inspection uses the structurally present point outline.

## Output evidence

The four Bar-family full-SVG snapshots and Marginal structural snapshot pass
with their existing expectations. The distribution/reference snapshot case
passes for Histogram, Density, Q-Q and Regression and retains its PNG dimension
checks.

Two full-SVG expectations have inspected **inactive stroke metadata** updates:

| Snapshot | Exact change | Preserved source/geometry/paint |
| --- | --- | --- |
| `visual_regression__native_density.snap` | Four zero-width polygons now use `stroke="#000000" stroke-opacity="0"`: two body fills and two guide fills | Two hue groups, both body polygons, both guide keys, every KDE segment/coordinate, labels, fonts, fill RGB/alpha 56, active curve strokes, frame, axes and clips |
| `visual_regression__native_regression.snap` | One zero-width band polygon uses that same canonical absent stroke | The complete band polygon/vertices and alpha 64, all six observation points and outlines, fit path, labels, fonts, frame, axes and clips |

The diffs contain only those five attribute sets (four changed lines plus one
changed line), with snapshot headers retained. No source is removed, no coordinate
or active paint changes, and no hash expectation is repinned. This follows the
structural absence contract rather than storing discarded RGB for serialization.

An independent backend fixture compares the prior zero-width RGB stroke metadata
with the current `DataFillTheme` absent-outline adapter. The SVG bytes differ as
specified, while their PNG bytes are **identical**. Decoded dimensions are 80×60
and an interior painted pixel has alpha **56**, proving the comparison is not an
empty-image equivalence.

## Verification

All test commands use `cargo test --locked --offline` with the bounded filters
below. **38 distinct tests** pass, including nine new tests (eight compound-value
fixtures and one Shared-guide absence fixture). Snapshot failures were resolved
only by the inspected metadata updates above, then their focused snapshot case
was rerun.

| Target/filter | Tests |
| --- | ---: |
| Library: `theme::mark_paint::tests` plus the added PNG-metadata test | 8 |
| Library: `render::backend::relative_outline_tests` | 3 |
| Library: `theme::resolved::tests::built_in_` | 5 |
| Library: `preset_construction` | 2 |
| Library: custom primary-point, series-line and Bar paint/guide tests | 3 |
| Library: absent-fill Shared-guide identity and exact point-outline guide identity | 2 |
| Library: polygon painted bounds and independent fill/stroke/point alpha emission | 3 |
| Library: ribbon full-stroke containment, density hard size, prepared Marginal and Matrix complete ink | 4 |
| Library: complete custom local/Shared guide ink and admitted theme-clone boundaries | 2 |
| Library: Regression custom ink and the two thick/containing bracket-stroke tests | 3 |
| Visual regression: Bar family, Marginal, distribution/reference filters | 3 |

Strict validation is `just lint`, which runs
`cargo clippy --locked --offline --all-targets -- -D warnings`, plus
`just fmt-check` and whitespace checks. No full-suite command is needed for this
bounded compound-value migration.

This closes the current compound-paint portion of spec94 §5. The following
spacing/component and complete role-group leaves retain their own construction
scope, including `SizedPointTheme`, `SupplementalPointTheme` and complete radius
groups. Complete immutable theme ownership and full local metric resolution remain
the subsequent theme-resolution cutover.
