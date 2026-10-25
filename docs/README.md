# User Guides

- [English guide source](en/src/index.md)
- [Chinese guide source](zh-CN/src/index.md)

These are peer editions with equivalent scope, not an original and a translation.

Each edition builds one main tutorial page, opening with real gallery images.
Click a thumbnail to reach its code walkthrough on that same page; use the
in-page contents menu for the rest. `src/index.md` assembles the authoring fragments
in `src/sections/`; those fragments are not separate tutorial pages. The
nonpublished Rust helper lives in `docs/book-helper/`, without a top-level `dev/`.

Install mdBook as an ordinary developer CLI:

```sh
cargo install mdbook --version 0.5.4 --locked
```

From the workspace root:

```sh
just book-check
just book
```

Runnable examples need no directory argument and write to the working directory
by default. An optional argument redirects output. Gallery builds manage their
isolated output directories automatically.

Open `target/book/site/index.html`. `just book` first runs the catalog's examples
into fresh isolated directories, assembles the gallery grids and images, then
invokes official `mdbook build` for every edition. The shared visual landing page
links equally to each language's walkthroughs. The previous site is replaced only
after all editions build and their rendered anchors pass validation; old chapter
and case pages are not retained in the active site. No site is deployed.
The recipe also copies the real example sources and their shared output helper
alongside each guide. Excerpts use semantic source anchors, not another plotting program.

`just gallery` executes and assembles without building HTML. `docs/gallery.tsv`
owns case identities, owning packages, source names, required output files,
tutorial anchors/API references, showcase/reference placement and peer-language titles/descriptions;
it contains no plotting configuration. Six scientific scenes are featured: scatter,
raincloud, heatmap, forest, line with bounds and a calibration composition. Two
additional numeric/text reference workflows and both adapter examples are executed
for the guide, without appearing as showcase cards. This is not complete image
coverage of all 25 families.
All generated material stays under `target/book/`, never beside maintained source.
Generation records platform/toolchain, the lockfile and font policy. Fonts must be
preinstalled; tools never download data or fonts. A missing output fails the build
instead of reusing an earlier image. Structural checks do not prove prose equivalence.

Source downloads use `examples/<package>/...`; includes, API links and executions
use the same explicit allowlist of implemented packages. The TSV has eleven
columns, including the owning package; there is no core-only catalog fallback.

The scientific showcase uses synthetic or supplied illustrative data, clearly
marked in each figure and walkthrough. Its public-API theme lives beside the
examples in `crates/lichtung-plot/examples/support/gallery.rs`, not in renderer
defaults. Review actual images at full size and thumbnail scale, not just SVG syntax.
