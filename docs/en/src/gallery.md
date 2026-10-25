# Executable gallery

[简体中文](../zh-CN/gallery.html)

The gallery runs public examples, then displays their actual outputs. It does not
reimplement plots or reuse old images when execution fails. Initial cases are:

- [A first plot from owned values](gallery-first-plot.md)
- [Numeric policies across one composition](gallery-numeric-policies.md)
- [Data anchoring and independent text fitting](gallery-text-layout.md)

Each page includes the executed source, its default output behavior,
related chapter, API reference and actual SVG files. Outlined SVG is the display
form; a preserve-text download, when supplied, replays the same frozen Output.
The initial gallery is not yet an image catalog for all 25 families; consult
[family selection](families.md) for the full current family catalog.

## Build locally

Install the ordinary developer CLI once with
`cargo install mdbook --version 0.5.4 --locked`. From the workspace root:

```sh
just book-check
just book
```

The first command checks both editions' chapter identities, peer links, local
links and semantic source anchors without rendering. The second runs each gallery
program once into a fresh isolated directory, assembles both editions, then calls
official `mdbook build`. The finished site is `target/book/site/index.html`.
`just gallery` assembles fresh gallery sources/artifacts without building HTML.

Generated pages/images and staging sources stay under `target/book/`; only source
chapters, catalog and executable examples are maintained. Generation records OS,
architecture, compiler, lockfile and font policy in its fresh work directory; exact
resolved font files are not exposed by the renderer. Use documented preinstalled
fonts, and do not interpret generated output as a cross-environment byte baseline.
Mechanical checks prove structure/source correspondence, not equivalent prose meaning.
