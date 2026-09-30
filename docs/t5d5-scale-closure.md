# T5d5 — Resolved Data-Scale Consumer And Precedence Matrix

This is the current implementation inventory after the migration in
`specs/92_T5B_T5E_EXECUTION_PLAN.md`. `specs/03_THEME_SYSTEM.md` defines
the target semantics; `docs/t5d0-scale-consumer-census.md` records the
historical starting routes and their deletion owners. T6 still owns the
public theme-component and general scale-configuration surface.

## Qualitative And Accent Consumers

Every categorical entry resolves from the normalized *raw* prepared string
before display aliases. A `CategoryColors` result is opaque RGB; its mark
role independently applies fill/stroke alpha, and a guide retains the same
complete final glyph. A configured exact `CategoricalColors` map never
falls back for a missing used key; unused keys and repeated colors are
valid. Built-in theme seeds preserve their ordered prefix and generate
additional distinct candidates without modulo cycling. `AccentColors`
resolves unkeyed positions independently of category/hue cardinality.

| Consumer | Domain and selected data paint | Guide and precedence |
| --- | --- | --- |
| Scatter | Finite observed hue groups under complete parent order; unkeyed primary is accent zero | Point glyph shares the resolved point style; exact point paint stays narrower than the theme |
| Line, Trajectory, LinePath | Raw hue/condition for colored paths; supplemental `PointLayer` occupies its own accent slot even when exactly painted | Path/ribbon/point glyphs retain owned RGBA, pattern and shape; a keyed primary consumes no accent slot |
| Density | Raw hue independently of group/curve order; group-only curves use the primary accent | Curve and translucent area use the same mapped RGB with independent alpha, including pair identity |
| Ordinary Bar and both stacked orientations | Category when ungrouped, otherwise the positioned series/hue across full facet slots | Prepared body color and originating series guide share exact fill and outline; missing slots invent no marks |
| Signed HorizontalBar | Nondecreasing (including zero) and decreasing use two independent accent slots, overriding hue color | Typed sign entries use exact bar body swatches; the series-color guide is suppressed |
| Boxplot, sampled Violin/Raincloud, supplied Raincloud | Raw category or positioned series under the complete parent distribution domain | Body, outlier/strip/cloud and originating guide preserve independent alpha/outline roles |
| Interaction | Prepared raw trace hue, or group when hue is absent | Trace markers/connectors/intervals and typed guide retain the same mapped identity; local shape/pattern remain exact |
| Marginal and Matrix | Complete prepared hue domain, or unkeyed primary accent for group-only cases | Central/off-diagonal points, linked/diagonal densities and originating guide agree on RGB; Matrix variable coordinates do not invent hue slots |
| Histogram, Qq, Regression, Estimate | Primary unkeyed accent zero for bin/point/observation/band and estimate marks | Regression fit/band entries use their final style; neutral reference lines and unavailable text are separate non-data roles |
| Waterfall | Signed positive/negative spans occupy independent accent slots | Connector/reference/total-label roles are not categorical colors; a one-seed accent extends without aliasing |
| Dumbbell | Exact caller endpoint colors with typed group/category marks | Endpoint entries use caller paint; connecting/reference strokes remain non-data roles |

`src/theme/scale.rs` owns immutable ordered key/slot values and admitted
lookups. The family producers listed above own their source identities and
pass selected colors into T5c mark-style roles. There is no remaining
`hue_color_index`, family modulo fallback, or project-owned
`categorical_color` lookup route.

The extendable palette's costly first-party OKLab candidate conversion is
retained per **operation-resource owner** and exact seed prefix when a domain
exceeds the built-in seed count. Repeated guide and selected-mark probes
admit a cache call and checked prefix copy but do not regenerate the same
candidates; a new root operation or changed theme seed recomputes them under
its own ledger. Candidate RGB/OKLab pairs are evaluated at most once per
ordinal within an extension, while each new maximin decision still compares
the current selected set. This keeps the original deterministic prefix and
the 18-entry historical guide within its approved work/byte limits without
refunds or a global cache.

## Continuous Consumers And Compositions

| Owner | Final scale/domain and mark | Guide/annotation handoff |
| --- | --- | --- |
| Heatmap | Finite active cell values under a sequential or centered diverging domain, with exact-limit saturation, symmetric automatic divergence and constant-domain midpoint/neutral | Pre-font retained fills select complete light/dark text roles; a local gradient and Canvas contributor retain ordered semantic stops, observed-value anchors and separate missing-key RGBA |
| Beeswarm | Sorted prepared numeric colors under exact or automatic sequential limits; point positions remain a separate scientific input | Prepared point RGB and local sampled stops agree; a shared Canvas re-installs the final domain before freezing each child |
| Shared Canvas | One kind/field/title/stop/center/missing identity; automatic envelopes union, exact domains agree and contain all automatic contributors | The descriptor freezes the final sampled stop sequence and generates one Canvas-owned label vector before fonts; children retain their Plot theme/style and receive that domain |

The private built-in default gradients live in `src/theme/scale.rs`. A
private Plot-local ordered replacement is validated before fonts for
Heatmap or Beeswarm and has narrower precedence than the built-in default;
its `Plot` clone admits stop storage. Plot-local Science/Nature theme
selection does not discard that replacement. A mixed-theme Canvas can
merge matching replacements; a changed stop or missing-key paint rejects
instead of letting one child win. The public replacement setters remain T6.

`ContinuousColors` performs the pinned first-party OKLab conversion,
gamut reduction and opaque byte rounding. It computes one normalized
position for both diverging center and data marks. Finite observed values
are admitted and inserted as guide stops using the backend's canonical
six-decimal offset spelling. Equal paints at one spelled position coalesce;
distinct paints at that position refuse rather than silently choosing a
backend RGB interpolation. A constant domain uses two identical midpoint
or neutral stops. Selected capture copies already frozen operands.

## Validation, Override And Error Order

| Input or failure | Owner and first observable result |
| --- | --- |
| Missing used exact categorical key, fixed private palette too short, malformed normalized domain | Prepared raw-key/scale validation refuses `Error::Data` before fonts or artifacts; unused exact keys and same-color entries remain valid |
| Invalid or incompatible continuous stop/domain/center/missing identity | Root-prepared family/Canvas contributor rejects before font discovery; no descriptor substitutes a default gradient for an invalid override |
| Shared automatic versus exact domains | One descriptor enforces exact equality and containment before fonts; no per-panel winner or second guide |
| Ordered stop arithmetic, gamut, admission, canonical offset collapse | The current operation's resource/scale boundary returns fatal `Error`, never a layout fit miss or a repaired stop |
| Local exact element paint, supplemental point color, typed significance target | Exact style overrides theme roles without changing occupied accents or raw identity; display aliases do not become annotation keys |
| Transparent Heatmap missing cell/row/key | One Plot-owned `MarkAppearance::heatmap_missing_fill` supplies full RGBA, separate from gradient endpoints; the key's outline retains its own RGBA role |
| Variable-fill text and planning backdrop | The final resolved fill and Plot/Canvas surfaces select one complete contrast role before the font census; changing domain, fill, backdrop or role metrics invalidates the corresponding intrinsic/cache identity |
| Clipped data versus required annotation | Exact caller data windows may clip data paint, while T5c typed attached/pairwise labels retain required ink and recover only through their named owner |
| Post-font assigned geometry or nonfinite/collapsed canonical output | Typed fit misses request only their owner; arithmetic, resource and malformed-output failures are fatal and publish no artifact |

Representative evidence: `theme::color` and `theme::scale` pin arithmetic,
prefixes, stops and exact resource limits; `plots::{scatter,line,density,
bar_data,categorical_distribution,interaction,marginal,matrix,beeswarm,
heatmap,waterfall}` exercise originating marks/guides and capture; Canvas
`d1_tests` exercise pre-font stop/glyph compatibility, hidden domains,
cache-on/off equality and final shared domains; `layout::text::requirements`
and Heatmap intrinsics exercise contrast roles; Significance and composition
tests exercise typed facet/Canvas targets. `tests/heatmap` decodes a PNG
interior mark and gradient-center pixel against the sampled SVG stop.
