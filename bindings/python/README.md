# Lichtung Plot for Python

This package currently provides a native-data Scatter and the reusable
Plot/Canvas/Output lifecycle. It is a working subset, not the complete 25-family,
four-input-route interface planned in the workspace specifications.

```python
from lichtung_plot import Canvas, Text, Theme, scatter

plot = scatter(x=[0.0, 1.0, 2.0], y=[1.0, 3.0, 2.0])
plot.title(Text("Synthetic observations")).theme(Theme.publication())
canvas = Canvas.grid(1, 1).panel(0, 0, plot)
output = canvas.svg("observations")
output.png_preview("observations-preview")
plot.title("A later independent edit")
```

`x` and `y` accept Python numeric lists or tuples. `None` entries retain native
validity; booleans are not numeric coordinates. Optional categorical `hue` accepts
string/None lists or tuples, at construction or through `.hue(...)`. Values are
snapshotted when bound. Non-None `data` sources, selectors, NumPy, pandas and Polars
inputs are not implemented in this subset.

Configuration methods mutate and return the same owner. Use `.copy()` for an
independent configuration. Canvas captures the Plot's native configuration at
insertion, without keeping the external Python Plot alive. `panel` takes
keyword-only `row_span`, `column_span` and `overwrite`; overwrite removes entire
intersecting owners, not just the covered cells.

Each Plot/Canvas SVG or PNG terminal creates a new render operation. Publish an
existing `Output` to reuse its frozen scene instead. Publication failures preserve
the configuration and earlier outputs. Only installed fonts are used; installation
does not download or provision fonts.

The shared contracts and remaining scope live at the workspace root:
`specs/91_WORKSPACE_AND_INTERFACES_PLAN.md`,
`specs/92_PYTHON_CONSTRUCTORS_PLAN.md`, and `specs/02_RENDERING_CONTRACT.md`.
Exact shipped value fields and methods are described by the installed typing
information, not by the full planned inventory.

The initial immutable value fields are deliberately bounded:

| Value | Read-only fields |
| --- | --- |
| `Theme` | `typography`, `surfaces` (the complete native Theme remains retained). |
| `TextTheme` | `stack`, `color`, `font_size`, `weight`, `line_height`. |
| `Text` | `source`, optional `style`, optional `layout`. |
| `TextLayout` | Optional `alignment`, `fit_priority`, `direction`, `rotation_mode`, `offset`, `bounds`, `wrapping`, `overlap`. |
| `TextOffset` / `TextBounds` | `dx`, `dy`, `frame` / optional `max_width`, `max_height`, `max_lines`. |
| `PngOptions` / `PngSizing` | `sizing`, `compression` / `kind`, `value`. |
| `NumericFormat` / `TickGeneration` | `fractional_digits` / `is_automatic`, `target_count`, `maximum_count`. |
| `NumericLabels` / `TickPositions` | `labels`, optional `explanation` / `kind`, `positions`. |
| `Output` | Frozen canonical `width`, `height`. |

Typography role groups, colors, font stacks and relative spacing expose their
typed native fields too; see the shipped stubs. Values derive through keyword-only
`copy_with`, without mutable nested handles. Text-taking owner methods accept
`style` and `layout`: omission preserves incoming/retained instructions, explicit
`None` clears the corresponding local instruction.

Scatter accepts `NumericFormat.value_function(...)`, `NumericFormat.set_function(...)`
and `TickGeneration.function(...)` through its x/y tick methods. Their context
objects and nested views are valid only during that callback. New terminal
operations may invoke the callable again; frozen Output publication does not.

## Local development

Use CPython 3.10 or newer, Rust 1.96 and already-installed maturin 1.16.0. From the
workspace root, `just python-build` builds a wheel and `just python-sdist` assembles
a source archive through the same package-local backend as PEP 517. Set
`LICHTUNG_PYTHON` and `LICHTUNG_MATURIN` to select installed executables.
After installing the wheel into an isolated environment,
`just python-test` runs that installation's tests. These recipes never install
tools or download dependencies.

The sdist backend keeps maturin's source closure and normalizes only its staged
`Cargo.lock` with offline Cargo metadata. It rejects new/changed package pins and
retargeted dependency edges, preserves every other archive member, and leaves the
single maintained workspace lock untouched. Existing output archives are not
overwritten. Use `just python-sdist` or a PEP 517 frontend, not bare
`maturin sdist`, which omits this projected-lock step. Wheel, editable and metadata
hooks remain maturin's own hooks. The sdist writer rejects nonempty config
settings rather than pretend maturin 1.16 forwards build arguments there; supply
locked/offline build arguments to the subsequent wheel operation.

Build requirements are maturin 1.16.0 and, on Python 3.10, tomli 2.5.0; neither is
a runtime dependency of `lichtung_plot`. Source-only backend tests are selected
explicitly with `python -I -B -m pytest -q bindings/python/tests/packaging_backend.py`;
installed-wheel runtime tests do not collect them.

The extension starts with `abi3-py310` for ordinary CPython. This is not a
certification of other platforms, PyPy or free-threaded interpreters.

Building a wheel from an independently unpacked sdist remains a required separate
distribution check; an in-checkout build alone does not certify source completeness
or platform support.
