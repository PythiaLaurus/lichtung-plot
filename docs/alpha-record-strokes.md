# Explicit Alpha For Single Curves And Record Intervals

Source baseline: `a8f53c6`. This leaf connects seven of the seventeen admitted
scientific stroke families to their explicit alpha units. It follows the completed
observation-point census and retains the private API boundary.

## Units And Final Paint

- Qq reference, Regression fit and optional Histogram KDE each prepare a Single
  unit before fonts, including schema validation when their optional geometry is
  absent. Their prepared operands retain member index and byte. The selected
  stroke and its intrinsic ink replace the role's alpha after normal RGB/style
  resolution, without deriving a value from sample observations.
- PointInterval, Coefficient and Forest prepare per-record line operands apart
  from point operands. PointInterval ordering transports both with the complete
  record. Model display transforms preserve original membership, including
  unavailable rows and point-only records that produce no uncertainty stroke.
- Each selected estimate row freezes its own interval/cap style. Point fill,
  reference/zero line, adjacent labels and unavailable-row text remain separately
  owned. Present interval ink supplies the same final-style hard pitch/width;
  transparent paint retains the interval/cap geometry and scientific record.
- Dumbbell likewise retains independent per-record connector styles alongside
  endpoint styles. Its point bytes can differ from its connector byte, with exact
  endpoint RGB and independent outlines preserved.
- `alpha::marks::line_style` is the shared final-byte replacement boundary.
  Capture reads frozen styles, not alpha inputs. Histogram clearance measures its
  actual selected bins, curve and rugs. Estimate automatic clearance now measures
  actual point/interval/cap ink separately from reference strokes and required text.

## Automatic Ink Boundary

The additional estimate clearance exposed an older PointInterval caller that
accepted silently clipped automatic endpoint ink at `[-f64::MAX,f64::MAX]`.
`specs/05_LAYOUT_SYSTEM.md` requires complete automatic painted ink or a fit miss;
only an explicit data window authorizes that clip. No finite binary64 domain can
extend beyond both extrema to contain their nonzero endpoint strokes.

The existing public case now verifies that specific scoped Render refusal and
successful explicit-window clipping over the same scientific extrema. Adjacent
ordinary-range, constant-row, asymmetric-cap and explicit crossing/clip callers
pass. This is a semantic containment correction, not a changed scene golden or
a layout-search fallback.

## Focused Evidence

**16 library tests passed**: all eight Qq tests, all four Regression style tests,
the two Histogram hard-input/owner-rug contracts, the estimate alpha source-order
contract and the Dumbbell exact-paint contract. Existing tests were strengthened;
no library tests were added or deleted in this leaf.

Witnesses include a 140px Qq reference and Regression fit made visible by data
alpha 128 over theme-zero strokes; independent Regression point bytes
`[0,128,255]` with fit byte 64; zero-alpha Histogram KDE retaining identical bins,
density curve and rugs; PointInterval slots `[1,2,0]` retaining distinct point and
interval bytes, unavailable text, present caps and a 40px theme-zero interval
override; and Dumbbell endpoint bytes 64/230 alongside connector bytes 0/191 and
independent outline byte 61.

**14 existing public integration contracts passed**:

- `distributions`: supplied Histogram/sample parity, every rug observation and
  duplicate, and deterministic Qq/reference modes (3).
- `estimates`: automatic interval endpoints, asymmetric fixed-pixel caps,
  unavailable estimate rows, Forest raw exact Log10 ticks, the corrected
  automatic-extrema/explicit-window distinction, extreme explicit crossing,
  exact explicit-limit clipping, a constant row and Coefficient significance
  with bounds (9).
- `categorical`: all Dumbbell replicates/endpoints/connectors (1).
- `layout`: Regression automatic frame clearance (1).

Each `cargo test --locked --offline` command used its explicit library/module or
integration target filter and ran nonzero tests. Strict `just lint`,
`just fmt-check` and `git diff --check` passed. Discovery remains **671 library
tests including 17 ignored**, recorded in
`/tmp/opencode/alpha-record-stroke-{count,ignored}.log`.

Actual keyed series/trace/aggregate summary strokes remain to complete the stroke
census. Fill consumers, unified automatic guide preparation/projection/parents,
public cutover and integrated full/release verification remain subsequent leaves.
