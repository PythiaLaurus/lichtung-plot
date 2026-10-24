# Architecture

## Purpose And Boundary

`lichtung-plot` is a standalone, strongly typed scientific renderer for owned
immutable display values. It owns construction, validation, display statistics,
visual annotations, layout, fonts, errors, scene creation and artifact publication.
It must not depend on, import or dispatch into another `lichtung-*` crate.

Callers convert their containers and analysis results into native values. They own
inferential decisions and supply explicit display annotations. The renderer must
not select hypothesis tests, invoke application analysis or provision assets. It
has no network access, font downloads, application runtime, report registration,
Python/R integration, dynamic plugins, browser automation or PDF conversion.

```text
caller data / analysis results
          |
          v
owned semantic values -> Plot / Canvas
          |
          v
validation -> scientific preparation -> initial fonts -> layout / text admission
          |
          v
accepted plan -> final font seal -> capture -> Null validation -> canonical scene
          |
          +-> outlined SVG
          +-> preserve-text SVG
          +-> PNG
          |
          v
        Output -> further publications of the same scene
```

## Specification Structure

These specifications define one current system. Each concern has one owner;
cross-references connect requirements rather than creating competing contracts.

| Specification | Responsibility |
| --- | --- |
| [Rendering contract](02_RENDERING_CONTRACT.md) | Scientific inputs, validation, lifecycle, errors and publication |
| [Theme system](03_THEME_SYSTEM.md) | Complete appearance values, construction, scales and local metric references |
| [Visual property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md) | Policy/style boundaries, precedence, actual paint consumers and data-alpha units |
| [Layout system](05_LAYOUT_SYSTEM.md) | Constraints, canonical geometry, text/fonts, allocation, feedback and frozen plans |
| [Numeric presentation](06_NUMERIC_PRESENTATION.md) | Formats, tick generation, complete semantic sets and dependencies |
| [Certified power conversion](07_LOG10_DISPLAY.md) | Mathematical Forest operand, interval proof, representability and independent oracle |
| [Guide system](08_GUIDE_SYSTEM.md) | Typed explanations, composition, compatibility, measurement and parent ownership |
| [Development](90_DEVELOPMENT.md) | Verification, semantic tests, dependency and publication discipline |
| [Deferred features](99_DEFERRED_FEATURES.md) | User-requested deferred reviews and their checkpoints |

Formal requirements refer to source owners by stable file/module names, not line
numbers, implementation-stage identifiers, execution logs or historical counts.

## Public And Private Boundaries

`src/lib.rs` defines crate-root exports. Only `kind` and display-only `stats` are
public modules. Backend, scene, layout, resolved-theme, prepared-source and font
operation types stay private. Implementation modules use `foo.rs` with `foo/`
siblings. There is one API and representation for each project-owned concern;
replacement migrates callers and removes the old implementation without aliases,
version selectors, fallback chains or deprecations.

Public configuration edits use `property(value)`. A reader paired with an edit on
the same type uses `get_property()`; read-only or write-only properties retain
their bare names. Theme's read-only components and Output's published dimensions
therefore retain their names. This convention preserves consuming ownership,
borrowed/copy results, const capability, fallibility and validation timing.

Semantic constructors return `Plot<K>` with an exact zero-sized family marker.
Sealed capabilities expose only supported configuration. There is no unselected
plot state. `Plot::erase` is the public transition to `Plot<kind::Any>`; erasure
retains common configuration, `kind()`, `preflight()` and output, but cannot regain
family-specific setters. `Canvas::panel` accepts an exact Plot and erases it at
its heterogeneous storage boundary. Canvas is not Clone.

There are 25 scientific families. Matrix owns a native variable grid and is not
Canvas-admissible or generic-facetable. Marginal is Canvas-admissible but not
generic-facetable. The other families retain their sealed capabilities and
data-dependent restrictions. An already faceted Plot cannot be embedded in Canvas.
`src/kind.rs` owns the closed capability sets; no erased fallback expands them.

## Ownership And Preparation

- Native numeric/category/boolean values own immutable semantic input.
  `Plot::source_rows` owns the one aligned diagnostic provenance map. Original
  input slots, diagnostic IDs and raw semantic identities remain distinct.
- Closed inputs own required channels and sample-versus-supplied-geometry choices.
  A later setter cannot replace a constructor-required group or color source.
- A Plot owns its complete scientific configuration, labels, annotations and
  optional explicit complete Theme. Standalone theme omission selects Publication;
  composed omission inherits the Canvas root; an explicit Plot Theme replaces it
  completely. Facets introduce no Theme scope.
- Canvas owns declared row-major topology, Plot/TextPanel cells, root appearance,
  root prose, headers, shared ranges and Shared-guide policy/chrome. Embedded Plot
  standalone sizing does not control cell allocation.
  src/canvas/placement.rs stores one payload/validated-footprint record per anchor
  and derives slot occupancy without copying payloads. Prepared theme/domain/text
  arrays retain anchor slot identities. Declared track edges are independent of
  assigned owner rectangles and supply header/row-title geometry.
  panel_span places one rectangular Plot owner across rows/columns and internal
  gaps. panel and single-cell TextPanels use the same full-footprint occupancy path.
  Covered nonanchors are occupied but never become independent producers or range
  group aliases; all inventories/capture visit each anchor once.
- Plot/Canvas prose, x/y titles and TextPanel title/body use the common owned `Text`
  value from the private text module. Its optional complete TextTheme and TextLayout request
  resolve locally over the owning roles before font preparation. Optional prose
  slots retain alignment instructions before content exists; plain replacement
  retains instructions and a present layout replaces the whole request. Complete
  content/instruction equality participates in root-frame, child and height reuse.
  Prepared axes retain Text, including for shared facet titles; each title's selected
  appearance owns its intrinsic measurement and font-relative axis clearances.
- PanelTextAnnotation retains the same Text value over the owning Plot's complete
  panel-text role. Its corner places the block independently of line alignment;
  selected local size also owns its annotation padding metric.
- Each operation owns prepared sources, immutable font bytes, exact dependencies,
  caches and bounded root coordination. Clones of operation context share identity;
  a new operation has a new identity and fresh installed-font discovery.
- One resolved complete appearance, final scales and semantic domains feed text
  census, intrinsics, constrained measurement, collision planning and capture.
  Backends receive concrete frozen operands, not public configuration to resolve.
- `Output` owns an opaque immutable scene. Its dimensions are canonical CSS pixels;
  SVG form and PNG options belong only to each publication call. Reuse does not
  perform scientific preparation, font discovery, formatting or layout again.

## Validation And Failure

Standalone and composed Plots pass through the same canonical plot validator.
Canvas adds composition rules after those checks and cannot weaken them.
Construction validates data-independent value invariants; preparation validates
alignment, scientific applicability, domains, exact mappings and parent guide
compatibility before fonts. Geometry-dependent checks use the actual selected
styles, fonts, constraints and containing boxes.

A measurement returns one immutable local plan or a typed fit miss. Parents
allocate and position children. Named bounded local choices and at most eight
root rounds coordinate fit; there is no exhaustive sibling product or global
quality objective. Deterministic policy execution does not promise every feasible
arrangement. Required content must fit or fail, except where an explicit owned
policy permits scientific data-window clipping or a specific label transformation.

Arithmetic, real allocation, I/O, font and lifecycle errors propagate. They must
not trigger another renderer, reduced content, substituted typography or a
previous-best scene. Production rendering has no cumulative resource-price ledger;
real representability, fallible first-party storage, algorithmic termination and
format-local output limits remain mandatory.
