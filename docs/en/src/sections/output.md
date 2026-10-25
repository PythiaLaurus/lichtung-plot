<a id="output"></a>
## Output, fonts and errors

[简体中文](../zh-CN/index.html#output)

### One rendered scene, several publications

`.svg` outlines every text element using the operation's installed font bytes and
checks that no text elements remain. Recipients need no fonts, but the visible
glyph paths are not selectable/searchable text and depend on generation fonts.
`.svg_preserve_text` publishes canonical SVG with selectable, searchable, editable
text and an ordered fallback declaration, without embedding font data. Its viewer
needs suitable fonts. Both forms come from the same frozen Output; another
publication does not repeat statistics, font discovery, callbacks or layout.

PNG rasterizes that outlined scene. `.png` uses 300 DPI/balanced compression with
physical-density metadata. `.png_preview` uses canonical CSS-pixel dimensions,
balanced compression and no density metadata. `.png_with_options` selects exact
PngSizing/PngCompression values, independently on each call. Output::width/height
remain canonical CSS dimensions even after different raster sizes.

### Dimensions and destinations

Plot/Canvas derive size from content by default. `width(600)` sets exact outer CSS
width and derives height; `max_width(600)` caps the complete natural width without
stretching a smaller figure. `size(600,400)` requests an exact pair. Each replaces
the complete previous request, so size(...).max_width(...) clears explicit height.
Zero refuses during preflight; requested bounds do not enable unrequested text
shrinking or truncation. Embedded dimensions belong to Canvas.

SVG calls append `.svg`, PNG calls append `.png`, unless the final native filename
already ends in the complete suffix under ASCII case-insensitive comparison.
Existing text is not replaced: `plot.v1` becomes `plot.v1.svg`, `plot.png` passed to
svg becomes `plot.png.svg`, and `.plot` becomes `.plot.svg`. Suffix completion
does not require UTF-8 and occurs after conversion. Parents must already exist;
directory destinations, including symlinks to directories, fail. The examples'
CLI helper defaults to the working directory and creates an optional explicitly
requested directory for convenience.

Each file is completed at a sibling staging path before atomic publication.
Preflight/conversion failure preserves an existing destination and removes staging
residue. A chain is not a transaction over all files: each publication is independent.

### Font policy and failure

Each used text role has its own typed font stack. The default order is Noto Sans,
Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif. A missing visible
glyph or an empty matching candidate set is an error, not permission to substitute
an unrelated family. No tool downloads or bundles fonts. An unused replaced role
does not demand its old fonts.

Constructors validate data-independent invariants; preparation checks scientific
alignment, domains and applicability before fonts when possible. Actual geometry
and text fit use the selected fonts/constraints. Arithmetic, allocation, I/O, font
and fit errors propagate; there is no alternate renderer or reduced-content retry.
Deterministic behavior requires matching source, configuration, dependencies and
font environment. Do not expect cross-revision/OS/font-installation byte equality.

PDF and report generation remain application responsibilities. The
[gallery](#gallery) links outlined and preserve-text outputs of the same example
where relevant, without retaining golden hashes or automatic panel coordinates.
