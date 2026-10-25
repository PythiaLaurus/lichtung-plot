# User Guides · 使用指南

- [English](en/src/introduction.md)
- [简体中文](zh-CN/src/introduction.md)

These are peer editions with equivalent scope, not an original and a translation.
两套指南地位相同、范围等价，没有原版与翻译版之分。

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
into fresh isolated directories, assembles generated gallery pages and images,
then invokes official `mdbook build` for both editions. No site is deployed.
The root recipe also copies the real example sources and their shared output helper
alongside each Book. Source excerpts are checked against semantic anchors in those
examples; they are not a second independently maintained plotting program.

打开 `target/book/site/index.html`。`just book` 先将清单中的示例运行到全新隔离目录，
组装生成的图库页面和图片，再对两套内容调用官方 `mdbook build`，不会部署站点。
构建同时附带真实示例源码和共享的输出辅助代码。摘录按语义锚点引用这些示例，
不是另一套独立维护的绘图程序。

`just gallery` executes and assembles without building HTML. `docs/gallery.tsv`
owns case identities, source names, required output files, related chapters/API
references and peer-language titles/descriptions; it contains no plotting configuration.
Only the three listed cross-cutting workflows are currently automated gallery cases.
All generated material stays under `target/book/`, never beside maintained source.
Generation records platform/toolchain, the lockfile and font policy. Fonts must be
preinstalled; tools never download data or fonts. A missing output fails the build
instead of reusing an earlier image. Structural checks do not prove prose equivalence.

`just gallery` 只执行和组装，不构建 HTML。`docs/gallery.tsv` 管理案例身份、源码名称、
必需文件、相关章节／API 及两种语言的标题和说明，不包含绘图配置。目前自动化图库只
包含清单中三个跨能力工作流。生成内容全部位于 `target/book/`，不混入维护中的源码。
生成记录包含平台／工具链、锁文件和字体策略。字体须预先安装，工具不下载数据或字体。
缺少产物会使构建失败，不复用旧图。结构检查不能证明文字语义等价。
