<a id="python"></a>
## Python: ordinary inputs, exact families and reusable output

[简体中文](../zh-CN/index.html#python)

The implemented `lichtung_plot` package exposes all [25 families](#families) through
snake-case constructors and exact classes such as `Scatter`, `Line` and `Bar`.
Methods follow each family's capabilities; Matrix does not acquire arbitrary
numeric axes, and Bar does not acquire Line's layer legend. Use a compatible local
wheel on ordinary CPython 3.10 or newer; see [installation](#installation).
The package has not yet been published to PyPI.

This is a usage guide, not another method matrix. For exact overloads and immutable
value fields, inspect the installed `lichtung_plot/__init__.py`,
`lichtung_plot/_owners.py` and `lichtung_plot/_native.pyi`, or the corresponding
files under `bindings/python/python/lichtung_plot/`. The maintained
`bindings/python/README.md` explains input and lifecycle rules. Native scientific
and rendering semantics remain in `specs/02_RENDERING_CONTRACT.md`, and Rust
adapter behavior is defined in `specs/09_DATA_ADAPTERS.md`. Python-specific call
forms are defined by the shipped declarations above.

### Select variables without inventing a data language

The four routes are native Python lists/tuples, NumPy arrays (including
MaskedArray), pandas DataFrame/Series and eager Polars DataFrame/Series. Providers
are optional; the Polars route does not require PyArrow. In constructors and
independent source-plus-variable calls, `data` is the only positional argument;
scientific fields are keywords. Direct variables and source selectors may be mixed.

Use `col[key]` for one variable and `cols[...]` for a nonempty ordered collection,
even when that collection contains one column. Bare lists/tuples always mean data,
not a collection of selectors. NumPy source integers are positions; pandas keys
are exact complete original labels, including legal non-string keys. A pandas
integer key is not reinterpreted as a position. Signed-step slices select logical
column positions and refuse out-of-range bounds rather than clipping them.
Display labels and aliases never replace lookup identity or scientific membership.

Supported primitive numeric data retain null slots; bool is not a numeric
observation. A true NumPy MaskedArray mask means missing. pandas object columns are
admitted only as string/null categories, not a general object conversion route.
Unsupported temporal, decimal or nested values need an explicit upstream projection.
Fields that are not bound are not imported just because a table contains them.

Each binding snapshots its bound slots. Retained main `data` supports explicitly
requested fresh lookup for later edits such as `.hue("condition")`; it does not
refresh old coordinates. A direct Series is not an implicit parent table.
Independent `data=None` is direct-only, never fallback to the main source.

### Owners, values and render operations

Plot and Canvas configuration methods mutate and return the same owner.
Use `.copy()` for independent configuration, not to refresh observations.
Canvas insertion captures the Plot's current native configuration and callback
roots without keeping the external Python Plot alive. Later edits of that Plot
do not change the inserted panel.

Theme, Text, numeric policies and other configuration values are immutable;
derive them with keyword-only `.copy_with(...)`, then replace the complete parent
value rather than mutating a nested handle. Omission retains a field. `None` clears
only where that API admits clearing: it is not a universal reset. For text-taking
owner methods, omitted `style`/`layout` preserve instructions; explicit `None`
clears the corresponding local instruction and allows owner/Theme defaults again.

Each Plot/Canvas terminal starts a fresh render operation and does not consume its
configuration on success or failure. An `Output` instead shares one immutable
frozen scene: publishing it again does not repeat callbacks, statistics or layout.
The examples below publish outlined SVG first, then preserve-text SVG and PNG
preview from that same Output. Existing [font, error and publication rules](#output)
still apply; neither the binding nor the gallery downloads fonts.

Foreign lookup and Python callbacks run outside owner borrows. A pending original
Python exception is not replaced by a later native or stale-candidate error.
Successful preparation still cannot overwrite an edit made by reentrant Python.
These lifecycle rules are not a promise of free-threaded interpreter support.

### Numeric policies and eight callback slots

Use complete `NumericFormat`, `NumericPrecision`, unit/locale/factor policies and
`TickGeneration` locators rather than converting coordinates to formatted strings.
`NumericFormat.from_specifier(...)` compiles a display policy, not a data transform.
Theme and Text use the same native appearance and instruction ownership described
under [themes](#themes) and [text](#text).

There are eight independent persistent callback attachments: x/y formats,
continuous-color labels, Heatmap values, Coefficient estimate labels, Waterfall
totals and x/y tick generation. Callback contexts and nested views are valid only
during that invocation. Copies retain unedited callback identity; a new terminal
may invoke callbacks again, while frozen Output publication does not.

### Alpha, independent observations and supplied results

Alpha is an exact byte in `0..255`, not floating opacity. Native list/tuple members
and literal values reject bools and floats; foreign byte variables require u8/UInt8.
Ordinary length-one variables do not broadcast. Choose an explicit `_single(...)`
form for that scope and `_theme()` for reset, not `None`.
Numeric mapped forms select one native `linear`, `log10`, `threshold` or
`continuous` factory; categorical forms use a scalar `mapper`. These factories
retain native guide declarations and representatives, not just final bytes.
Semantic keys use raw identities, never display aliases.

Alpha mappers run at binding time after foreign loans end; they are not stored as
a ninth persistent callback. Alpha does not filter observations, alter values or
change scientific membership. See the [coverage example](#python-alpha).

Line `point_layer(data=None, *, x, y, ...)` binds independent points, with optional
labels, Color, radius, byte alpha and complete-layer legend Text instructions.
Optional `None` is absent, except explicit legend style/layout `None` clears that
instruction; an instruction without a legend label is incomplete.
Bar/HorizontalBar instead require category/value and complete Color/u32 radius,
with optional positioned series: no point labels, legend, layout or independent
alpha parameter. A layer's count and source do not inherit the primary plot's.

Attached results belong to Bar, HorizontalBar and Coefficient. Pairwise results
belong to Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin and
Raincloud. Each placement has six forms: ordinary and `_from_data`, `_in_facet`
and `_in_facet_from_data`, `_in_grid` and `_in_grid_from_data`.
Targets are ordinary categorical variables, optional category/series pairs, or
explicit `target_value`/`left_value`/`right_value` literals. Only explicit literal
branches set scalar flags; an ordinary length-one target is still a vector.
`facet_value` similarly selects a literal level or grid pair.

Marks are numeric `marks` with mapping omitted, categorical `marks` with explicit
`mapping`, or literal `mark` with both omitted. Literal marks are p-values or
`ns`/one through four stars. `mapping=None` is invalid; ordered duplicate mapping
pairs remain available for native consumed-key validation. Target/facet values are
raw levels, not semantic-field lookup keys or display aliases. Results have their
own count; native producers retain binding, broadcasting and preflight validation
stages. The caller supplies tests and adjustments: the renderer computes neither.

### Complete Python reference programs

These examples use the four supported input routes with one native renderer,
not an additional plotting backend.
With the installed interpreter selected as in [building](#build-docs), run, for
example, `"$LICHTUNG_PYTHON" -I -B bindings/python/examples/scientific_composition.py`
from the workspace root. No argument writes into the working directory; one
optional directory redirects output. The complete sources below include input
construction, composition and all three publications.

<a id="python-inputs"></a>
### One assay through four input routes

[简体中文](../zh-CN/index.html#python-inputs)

![The same synthetic assay through native Python, NumPy, pandas and eager Polars](gallery/python-inputs/four_inputs.svg)

All four panels bind the same time, response and cohort values. The NumPy route
selects positional columns; pandas and Polars select named fields. The unbound
pandas Decimal and Polars Struct columns demonstrate that binding selected
variables does not convert an entire table. This is one input-transport comparison,
not a family-by-provider matrix. Each panel labels the same units.

```python
{{#include ../../../../bindings/python/examples/four_inputs.py:example}}
```

[Complete source](examples/lichtung-plot-python/four_inputs.py) ·
[Full-size SVG](gallery/python-inputs/four_inputs.svg) ·
[Selectable-text SVG](gallery/python-inputs/four_inputs_text.svg) ·
[PNG preview](gallery/python-inputs/four_inputs.png) · [Python interface](#python)

<a id="python-figures"></a>
### Scientific composition: four distinct input contracts

[简体中文](../zh-CN/index.html#python-figures)

![Python Line, Bar intervals, unavailable Forest estimate and annotated Heatmap](gallery/python-figures/scientific-composition.svg)

The wide Line retains actual time coordinates and separate series. The Bar uses
supplied illustrative bounds, not a sample-derived confidence calculation. Forest
marks an unavailable estimate with an explicit status, rather than inferring it
from nulls. Heatmap highlights are independent Boolean data, not a filter on cells.
All four panels use synthetic values and label their quantities.

```python
{{#include ../../../../bindings/python/examples/scientific_composition.py:example}}
```

[Complete source](examples/lichtung-plot-python/scientific_composition.py) ·
[Full-size SVG](gallery/python-figures/scientific-composition.svg) ·
[Selectable-text SVG](gallery/python-figures/scientific-composition-text.svg) ·
[PNG preview](gallery/python-figures/scientific-composition-preview.png) · [Python interface](#python)

<a id="python-alpha"></a>
### Alpha mappings: supplied coverage is not confidence

[简体中文](../zh-CN/index.html#python-alpha)

![Python categorical observation alpha and numeric Heatmap cell alpha](gallery/python-alpha/alpha-mappings.svg)

The Scatter mapper converts supplied coverage classes into byte alpha; the Heatmap
uses a native linear map of supplied coverage fractions. Signal values and color
remain separate from coverage. Neither representation claims statistical
confidence, removes an observation or changes its value.

```python
{{#include ../../../../bindings/python/examples/alpha_mappings.py:example}}
```

[Complete source](examples/lichtung-plot-python/alpha_mappings.py) ·
[Full-size SVG](gallery/python-alpha/alpha-mappings.svg) ·
[Selectable-text SVG](gallery/python-alpha/alpha-mappings-text.svg) ·
[PNG preview](gallery/python-alpha/alpha-mappings-preview.png) · [Python interface](#python)

<a id="python-results"></a>
### Independent observations and supplied display comparisons

[简体中文](../zh-CN/index.html#python-results)

![Python supplied trend and band beside Bar intervals, observations and comparison brackets](gallery/python-results/supplied-results.svg)

Four supplied trend positions and their band coexist with six independent
observations and one whole-layer legend identity. The Bar has three supplied
intervals, six independent points and two supplied comparisons. Literal
`left_value="Control"` explicitly broadcasts across the two right-hand results;
it is not a guessed one-element variable. Bounds and display p-values are
illustrative: the renderer does not fit the trend, calculate a test or adjust p-values.

```python
{{#include ../../../../bindings/python/examples/supplied_results.py:example}}
```

[Complete source](examples/lichtung-plot-python/supplied_results.py) ·
[Full-size SVG](gallery/python-results/supplied-results.svg) ·
[Selectable-text SVG](gallery/python-results/supplied-results-text.svg) ·
[PNG preview](gallery/python-results/supplied-results-preview.png) ·
[Python interface](#python) · [Back to gallery](#gallery)
