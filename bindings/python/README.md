# Lichtung Plot for Python

This package provides Scatter from native Python, NumPy, pandas and eager Polars
inputs, with the reusable Plot/Canvas/Output lifecycle. Other plot families and
the remaining configuration surface are still being implemented.

```python
from lichtung_plot import Canvas, Text, Theme, scatter

plot = scatter(x=[0.0, 1.0, 2.0], y=[1.0, 3.0, 2.0])
plot.title(Text("Synthetic observations")).theme(Theme.publication())
canvas = Canvas.grid(1, 1).panel(0, 0, plot)
output = canvas.svg("observations")
output.png_preview("observations-preview")
plot.title("A later independent edit")
```

## Inputs

`data` is the only positional argument. Variables are keyword-only and may mix
direct lists/tuples, arrays and Series with source selectors:

```python
from lichtung_plot import col, cols, scatter

plot = scatter(frame, x=col["Time"], y=cols["Control", "Treated"])
plot.x_label("Time (hours)")
```

`frame` can be a pandas or eager Polars DataFrame; a two-dimensional NumPy source
uses positional columns, for example `col[0]` and `cols[1, 2]`. `col[...]` selects
one variable; `cols[...]` selects a nonempty ordered collection, even with one
column. Positional selectors admit negative indices and signed-step slices, but
never clip out-of-range bounds. pandas keys are exact complete hashable labels,
not expressions or stringified identities. Bare lists/tuples always mean data.

Scatter preserves actual `x` for multiple y variables. Use
`scatter(frame, y=cols["Control", "Treated"], by_row_position=True)` to request row x,
or `scatter(frame, by_row_position=True, numeric_only=True)` for explicit numeric
discovery. These modes do not infer x or source IDs from a DataFrame index.

Numeric coordinates admit supported primitive numeric data and nulls, not bools.
Categorical `hue` accepts strings/nulls at construction or through `.hue(...)`.
NumPy strided/reversed views and MaskedArray inputs retain logical order and
validity; a true NumPy mask means missing. pandas nullable columns retain their
available null/NaN distinction. Temporal/decimal/nested observations and NumPy
object arrays require an explicit upstream projection. pandas object columns are
admitted only as string/null categories. Unbound columns are not materialized
merely because they belong to a source.

Each binding snapshots its observations. A retained main `data` source supplies
fresh lookup for later `.hue(...)` or `.source_rows(...)`, without refreshing old
x/y. Direct Series and independent DataFrames do not install an implicit parent
source. Copies retain their own real source/key references; Canvas captures only
native configuration and callback roots. The binding cannot make cycles hidden
inside foreign non-GC-traversed storage collectable.

Input providers are optional and imported only when used. The current eager
Polars bridge requires `polars==1.41.2` with `polars-runtime-32==1.41.2`, using its
public Arrow C stream without PyArrow. The sampled NumPy/pandas compatibility
matrix and remaining distribution scope are recorded in the shared plan.
`examples/four_inputs.py` demonstrates all four routes.

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

## Immutable configuration

Theme exposes all eight native components and their nested paint, geometry,
spacing, typography and palette values. TextLayout exposes all thirteen authored
instructions. These appearance, text and PNG values comprise 84 immutable classes;
Labels and numeric policies are separate. Other family-specific policies and the
remaining numeric surface are still being implemented.

| Value | Read-only fields |
| --- | --- |
| `Theme` | `typography`, `surfaces`, `axes`, `legend`, `composition`, `marks`, `annotations`, `color_scales`. |
| `TextTheme` | `stack`, `color`, `font_size`, `weight`, `line_height`. |
| `Text` | `source`, optional `style`, optional `layout`. |
| `TextLayout` | Optional `alignment`, `fit_priority`, `direction`, `rotation_mode`, `offset`, `bounds`, `wrapping`, `overlap`, `fit`, `orientation`, `anchor`, `overflow`, `visibility`. |
| `TextOffset` / `TextBounds` | `dx`, `dy`, `frame` / optional `max_width`, `max_height`, `max_lines`. |
| `PngOptions` / `PngSizing` | `sizing`, `compression` / `kind`, `value`. |
| `NumericFormat` / `TickGeneration` | `fractional_digits` / `is_automatic`, `target_count`, `maximum_count`. |
| `NumericLabels` / `TickPositions` | `labels`, optional `explanation` / `kind`, `positions`. |
| `Labels` | `mode` (`humanized`/`raw`), optional ordered string-pair `aliases`. |
| `Output` | Frozen canonical `width`, `height`. |

Nested values expose typed read-only fields; see the shipped stubs for exact
constructors and closed factories. Derive a leaf, then replace its complete parent:

```python
base = Theme.publication()
primary = base.marks.points.primary.copy_with(shape="triangle_up", outline=None)
theme = base.copy_with(
    marks=base.marks.copy_with(points=base.marks.points.copy_with(primary=primary))
)
```

Values derive through keyword-only `copy_with`, without mutable nested handles.
Omission retains a field; `None` clears only fields that admit it. An optional
branch reader, such as `DataOutlineTheme.fixed_stroke`, does not make that branch's
required payload nullable. Palette/gradient sequences are snapshotted in order;
domain-dependent capacity and text-owner eligibility remain native use-time checks.
`examples/complete_appearance.py` demonstrates nested Theme edits, complete
Text instructions and publication from one frozen Output.

Text-taking owner methods accept
`style` and `layout`: omission preserves incoming/retained instructions, explicit
`None` clears the corresponding local instruction.
`Plot.labels(...)` and `Canvas.labels(...)` replace the native presentation policy.
`Labels.copy_with(aliases=None)` clears aliases; aliases require a sequence of
pairs, not a mapping.
Aliases address native raw identities, not pandas lookup keys; use explicit axis
labels when authoring a display title for a pandas-bound variable.

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
