<a id="build-docs"></a>
## Building the guide and gallery

[简体中文](../zh-CN/index.html#build-docs)

The [gallery](#gallery) runs public examples and displays their real outputs, not
another plotting implementation or reused stale images. Thumbnails jump straight
to twelve scientific walkthroughs covering [all 25 families](#families).
[Numeric policies](#numeric-policies), [text layout](#text-layout), [ndarray](#ndarray),
[Polars](#polars) and the four [Python examples](#python) are eight technical
references, making twenty executables. Multi-panel programs retain their
separate family consumers; six new Rust programs are not a claim of only six
independent scenarios. Outlined SVG is the first display artifact. Where offered,
preserve-text SVG and PNG replay the same frozen Output, not a second render.

Each showcase has meaningful variables, units or an explicit reference, with a
common example-local theme and readable figure proportions. Synthetic data and
supplied illustrative intervals are identified as such. Inspect the actual figures
at full size and gallery scale; successful compilation and valid SVG alone do not
establish visual quality. Never crop away labels or data to improve a thumbnail.

### Build locally

Use the preinstalled official `mdbook` 0.5.4 CLI, the Rust toolchain and cached
dependencies. The Python references additionally need an environment with the
already-built project wheel installed, as described under [installation](#installation).
Select its interpreter with an explicit absolute path, then run from the workspace root:

```sh
export LICHTUNG_PYTHON=/absolute/path/to/environment/bin/python
just book-check
just book
```

The first command checks both editions' tutorial anchors, image-to-code links,
peer navigation and semantic source includes without rendering. The second runs
each of the twenty gallery programs once into fresh isolated directories, shares
those artifacts between the two single-page guides, then calls official
`mdbook build`. The finished site is
`target/book/site/index.html`, opening on a shared visual gallery with equal links
to both language walkthroughs. The Rust assembly helper lives in `docs/book-helper/`.
The helper does not install mdBook, Python, wheels, data providers or fonts.
Python examples run with the selected installed interpreter, not a source-tree
package substituted for that installation.

`just gallery` is an alternative that executes and assembles without building HTML.
Do not run it before `just book` to prepare images: `just book` already performs
that fresh execution, so chaining them would render the catalog twice.

Generated pages/images and staging sources stay under `target/book/`; only source
guide fragments, catalog and executable examples are maintained. Generation records OS,
architecture, compiler, lockfile and font policy in its fresh work directory; exact
resolved font files are not exposed by the renderer. Use documented preinstalled
fonts, and do not interpret generated output as a cross-environment byte baseline.
Missing artifacts fail the build rather than falling back to old images; the
active site must not retain stale chapter/case HTML.
Mechanical checks prove structure/source correspondence, not equivalent prose meaning.
