<a id="build-docs"></a>
## 构建指南与图库

[English](../en/index.html#build-docs)

[图库](#gallery)运行公开示例并展示真实产物，不维护另一套绘图实现，也不复用旧图。
缩略图直接跳到本页[首张图](#first-plot)、[数值策略](#numeric-policies)和
[文本布局](#text-layout)的代码讲解。页面展示轮廓 SVG；保留文字的下载复用同一个冻结
Output。目前尚未完成全部 25 个图族的图像目录，完整列表见[图族选择](#families)。

### 本地构建

使用 `cargo install mdbook --version 0.5.4 --locked` 安装一次普通开发 CLI，
然后从 workspace 根目录运行：

```sh
just book-check
just book
```

第一条命令只检查两套内容的教程锚点、图像到代码的链接、语言跳转及语义源码引用，
不渲染。第二条命令将每份图库程序运行一次，写入全新隔离目录，组装两套单页指南，
再调用官方 `mdbook build`。站点入口为 `target/book/site/index.html`，直接展示共用的
真实图库，并等权链接到两种语言的讲解。Rust 组装辅助工具位于 `docs/book-helper/`。
`just gallery` 只组装新的图库源码和产物，不构建 HTML。

生成页面、图片及暂存源码均位于 `target/book/`，只维护指南片段、目录清单和可执行示例。
每次生成会在新工作目录记录操作系统、架构、编译器、锁文件和字体策略；绘图器不暴露
精确解析到的字体文件。请使用已说明、预先安装的字体，不将生成结果视为跨环境字节基准。
机械检查证明的是结构及源码对应关系，不证明文字语义等价。
