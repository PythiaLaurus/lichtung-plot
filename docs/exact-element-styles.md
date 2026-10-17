# Complete Exact Element Styles

## One Typed Public Paint

Crate-root `Color` exposes exact byte RGB/RGBA construction, strict literal hex
parsing, readonly channels and replacement `with_alpha`. Alpha includes zero and
255; no floating opacity conversion or alpha multiplication remains in exact
styles. `OpaqueColor`, complete Theme components, fonts and scale attachments
retain their next owning public cutover.

The public migration replaces storage, constructors, inspection, consuming edits
and repository callers together:

| Value | Current exact contract |
| --- | --- |
| `GuideStyle` | Complete Color, positive finite CSS width, closed LinePattern; readonly getters and fallible `with_color`, `with_width`, `with_pattern` |
| `PointLayer` | Optional independent Color, byte alpha and positive integral radius; `with_color`, `with_alpha`, fallible `with_radius`; getters inspect caller instructions rather than owner-dependent effective defaults |
| `BarPointLayer` | Constructor-required complete Color and positive radius; unchanged owned category/optional-series/value channels, no opacity argument/field/getter, plain Circle without outline or layer legend |
| `DumbbellEndpoints` | Two distinct validated literal labels and complete Colors; readonly paints, unchanged endpoint/source identities |
| `SignificanceMarkStyle` | Positive CSS font size, closed weight, finite nonnegative exact CSS gap; getters and fallible `with_font_size`, `with_weight`, `with_gap`; gap normalizes signed zero |
| `InteractionStyle` | Fallible raw-selector construction, readonly group/shape/pattern and fallible `with_point_shape`, `with_line_pattern`; Circle/Solid defaults |

The old tuple style inputs, floating-opacity routes, consuming bare-field edits,
private significance `_spec` getters and obsolete opacity quantizer are removed.
Backend fixtures now supply byte paints directly. No alias/deprecation path or
second renderer accompanies the cutover.

## Selected Paint And Scope

PointLayer resolution retains the prior private behavior: absent instructions use
the stable append-slot accent and owner supplemental radius/alpha. Exact Color
replaces the whole fill; independent exact alpha wins in either edit order.
Labels start from their complete text role. Automatic mark RGB/alpha do not
recolor text; exact color replaces text paint, then explicit alpha replaces only
alpha. Stack, size, weight and leading stay independent. Radius-relative offsets,
whole-pixel snapping, Start/Middle placement, complete eligible label ink and
append-slot consumption are preserved. Keys reuse final mark style.

Dumbbell intrinsics, selected marks and literal keys share one endpoint resolver.
Complete exact fill replaces the point-role alpha, including a transparent fill;
theme radius, shape and independent outline still apply. This intentionally makes
an opaque Color opaque rather than applying the former primary-point alpha 209.
Neutral connector paint is independent. Canonical scatter capture retains both
the exact fill and the separately resolved outline.

Guide stroke paint reaches projection, geometry/collision and frozen SVG without
discarding alpha. Bar layers retain prepared ordinary-bar centers and complete
source rows in both orientations. Significance absence still uses the theme;
an explicit default remains 16px/Normal/8px and replaces size/weight/gap only.
Its exact hard-space and canonical selected geometry remain intact.

Interaction construction rejects empty, surrounding-whitespace and control-bearing
selectors immediately. It retains interior whitespace and Unicode bytes rather
than normalizing identities. Actual prepared lookup still validates known groups,
uniqueness and complete supplied coverage before fonts. Shape and pattern edits
do not change mapped paint, alpha, radii, outlines or interval/connector widths.

## Focused Evidence

Three behavior tests replace three field/edit inventories in the same change:

- Actual Dumbbell selected marks, literal keys and canonical capture preserve
  transparent/partial exact fill and independent custom outline/shape/radius;
  replaces the compound stroke/solid edit inventory and its macro support.
- Two opposite PointLayer edit orders produce identical selected/captured paint,
  complete label appearance and guide input; replaces the TextTheme edit inventory.
- Lexically valid `A B` and `A  B` selectors select distinct actual prepared traces
  despite reversed style order, while malformed raw selectors reject; replaces
  the six-axis-spacing edit inventory and its macro support.

Existing parser, significance, numeric-layer and guide tests migrate to the
current API, including strict invalid hex syntax, normalized exact gap zero and
independently specified SVG byte-alpha ratios. Library discovery is **671**, with
the same **17 ignored** tests. No cases move to integration suites to evade that
limit. No scene golden is repinned.

Validation passed:

- **11 focused library tests**: the three replacements, supplemental defaults,
  four significance contracts, literal paint parsing, canonical SVG/PNG storage
  and transparent-fill independent-outline geometry.
- **61 existing external contracts**: twenty numeric-layer, nine Dumbbell,
  eighteen styled Interaction, five connector, three Bar layer, one exact public
  family-builder, one two-orientation significance, two Canvas guide, one parent
  spectrum-legend and one spectrum-source/orientation contract.
- **66 doctests**, including seven current public examples and six removed-API
  compile-fail boundaries added for the changed surface.
- Strict all-target Clippy, formatting, whitespace checks, warning-denied API
  documentation and locked/offline package verification.
- Library and ignored discovery only; no complete library/integration suite.

Execution logs are `/tmp/opencode/exact-styles-*.log`; the package log records
successful compilation from Cargo's unpacked staging directory.
