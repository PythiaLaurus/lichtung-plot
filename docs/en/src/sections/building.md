<a id="build-docs"></a>
## Building the guide and gallery

[简体中文](../zh-CN/index.html#build-docs)

The [gallery](#gallery) runs public examples and displays their real outputs, not
another plotting implementation or reused stale images. Thumbnails jump straight
to six scientific walkthroughs: [scatter](#first-plot), [distributions](#distributions),
[heatmap](#expression-heatmap), [effect intervals](#effect-intervals),
[time response](#time-response) and [calibration](#calibration). The
[numeric policies](#numeric-policies) and [text layout](#text-layout) figures remain
technical references in the guide, not showcase thumbnails. Outlined SVG is the
display form; preserve-text downloads replay the same frozen Output. This is not
yet an image catalog for all 25 families; see [family selection](#families).

Each showcase has meaningful variables, units or an explicit reference, with a
common example-local theme and readable figure proportions. Synthetic data and
supplied illustrative intervals are identified as such. Inspect the actual figures
at full size and gallery scale; successful compilation and valid SVG alone do not
establish visual quality. Never crop away labels or data to improve a thumbnail.

### Build locally

Install the ordinary developer CLI once with
`cargo install mdbook --version 0.5.4 --locked`. From the workspace root:

```sh
just book-check
just book
```

The first command checks both editions' tutorial anchors, image-to-code links,
peer navigation and semantic source includes without rendering. The second runs
each gallery program once into a fresh isolated directory, assembles the two
single-page guides, then calls official `mdbook build`. The finished site is
`target/book/site/index.html`, opening on a shared visual gallery with equal links
to both language walkthroughs. The Rust assembly helper lives in `docs/book-helper/`.
`just gallery` assembles fresh gallery sources/artifacts without building HTML.

Generated pages/images and staging sources stay under `target/book/`; only source
guide fragments, catalog and executable examples are maintained. Generation records OS,
architecture, compiler, lockfile and font policy in its fresh work directory; exact
resolved font files are not exposed by the renderer. Use documented preinstalled
fonts, and do not interpret generated output as a cross-environment byte baseline.
Mechanical checks prove structure/source correspondence, not equivalent prose meaning.
