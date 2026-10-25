# User Guides · 使用指南

- [English guide source](en/src/index.md)
- [中文指南源码](zh-CN/src/index.md)

These are peer editions with equivalent scope, not an original and a translation.
两套指南地位相同、范围等价，没有原版与翻译版之分。

Each edition builds one main tutorial page, opening with real gallery images.
Click a thumbnail to reach its code walkthrough on that same page; use the
in-page contents menu for the rest. `src/index.md` assembles the authoring fragments
in `src/sections/`; those fragments are not separate tutorial pages. The
nonpublished Rust helper lives in `docs/book-helper/`, without a top-level `dev/`.

每种语言构建一个主教程页，开头展示真实图库。点击缩略图直接跳到同页对应的代码讲解，
其他内容通过页内目录导航。`src/index.md` 合并 `src/sections/` 中的编写片段，不把片段
变成独立教程页面。非发布的 Rust 辅助工具位于 `docs/book-helper/`，没有顶层 `dev/`。

Install mdBook as an ordinary developer CLI / 按常规开发工具安装 mdBook：

```sh
cargo install mdbook --version 0.5.4 --locked
```

From the workspace root / 从 workspace 根目录运行：

```sh
just book-check
just book
```

Runnable examples need no directory argument and write to the working directory
by default. An optional argument redirects output. Gallery builds manage their
isolated output directories automatically.

示例无需提供目录参数，默认输出到当前工作目录；有需要时可用可选参数重定向输出。
图库构建自动管理隔离输出目录，不要求用户手动安排。

Open `target/book/site/index.html`. `just book` first runs the catalog's examples
into fresh isolated directories, assembles the gallery grids and images, then
invokes official `mdbook build` for both editions. The shared visual landing page
links equally to both languages' walkthroughs. The previous site is replaced only
after both editions build and their rendered anchors pass validation; old chapter
and case pages are not retained in the active site. No site is deployed.
The recipe also copies the real example sources and their shared output helper
alongside each guide. Excerpts use semantic source anchors, not another plotting program.

打开 `target/book/site/index.html`。`just book` 先将清单中的示例运行到全新隔离目录，
组装图库网格和图片，再对两套内容调用官方 `mdbook build`。共用的图像入口等权链接到
两种语言的讲解。两套内容均构建成功并通过渲染后锚点检查，才替换原站点；当前站点不
保留旧章节页或独立案例页，也不会部署。构建附带真实示例源码和共享输出辅助代码，
摘录按语义锚点引用这些示例，不是另一套绘图程序。

`just gallery` executes and assembles without building HTML. `docs/gallery.tsv`
owns case identities, owning packages, source names, required output files, tutorial anchors/API
references, showcase/reference placement and peer-language titles/descriptions;
it contains no plotting configuration. Six scientific scenes are featured: scatter,
raincloud, heatmap, forest, line with bounds and a calibration composition. Two
additional numeric/text reference workflows and both adapter examples are executed for the guide, without
appearing as showcase cards. This is not complete image coverage of all 25 families.
All generated material stays under `target/book/`, never beside maintained source.
Generation records platform/toolchain, the lockfile and font policy. Fonts must be
preinstalled; tools never download data or fonts. A missing output fails the build
instead of reusing an earlier image. Structural checks do not prove prose equivalence.

`just gallery` 只执行和组装，不构建 HTML。`docs/gallery.tsv` 管理案例身份、所属包、源码名称、
必需文件、教程锚点／API、首页展示或正文参考的归属，以及两种语言的标题和说明，不
包含绘图配置。首页精选散点、雨云、热图、森林、带边界的折线及校准组合六个场景，
另外执行两个数值／文本参考工作流和两个适配器示例供正文使用，不把它们放进首页卡片。尚未覆盖全部
25 个图族的图像。生成内容全部位于 `target/book/`，不混入维护中的源码。
生成记录包含平台／工具链、锁文件和字体策略。字体须预先安装，工具不下载数据或字体。
缺少产物会使构建失败，不复用旧图。结构检查不能证明文字语义等价。

Source downloads use `examples/<package>/...`; includes, API links and executions
use the same explicit allowlist of implemented packages. The TSV has eleven
columns, including the owning package; there is no core-only catalog fallback.

源码下载使用 `examples/<package>/...`；源码引用、API 链接与执行统一使用已实现包的
显式白名单。TSV 共十一列，包含所属包，不保留只支持核心包的兼容清单。

The scientific showcase uses synthetic or supplied illustrative data, clearly
marked in each figure and walkthrough. Its public-API theme lives beside the
examples in `crates/lichtung-plot/examples/support/gallery.rs`, not in renderer
defaults. Review actual images at full size and thumbnail scale, not just SVG syntax.

科学绘图展示使用合成数据或给定示意值，每张图和讲解均明确标注。通过公开 API 构造
的主题位于示例旁的 `crates/lichtung-plot/examples/support/gallery.rs`，不改变渲染器默认
值。审阅必须查看真实原图及缩略图，不能只检查 SVG 语法。
