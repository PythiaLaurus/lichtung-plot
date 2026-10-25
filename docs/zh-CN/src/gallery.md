# 可执行图库

[English](../en/gallery.html)

图库运行公开示例，再展示实际结果；不会重新实现绘图，也不会在执行失败时复用旧图。
初始案例包括：

- [从自有数据绘制首张图](gallery-first-plot.md)
- [组合图中的数值策略](gallery-numeric-policies.md)
- [数据锚定与独立文本适配](gallery-text-layout.md)

每个页面包含实际执行的源码、调用者指定的输出目录、相关章节、API 参考及真实 SVG。
页面展示轮廓 SVG；若另有保留文字的下载文件，它来自同一个冻结 Output。
初始图库尚未为全部 25 个图族生成图像目录，完整当前图族列表见[图族选择](families.md)。

## 本地构建

使用 `cargo install mdbook --version 0.5.4 --locked` 安装一次普通开发 CLI，
然后从 workspace 根目录运行：

```sh
just book-check
just book
```

第一条命令只检查两套内容的章节身份、对应语言链接、本地链接和语义源码锚点，不渲染。
第二条命令将每份图库程序运行一次，写入全新隔离目录，组装两套内容，再调用官方
`mdbook build`。完成的站点位于 `target/book/site/index.html`。
`just gallery` 只组装新的图库源码和产物，不构建 HTML。

生成页面、图片及暂存源码均位于 `target/book/`，只维护源章节、目录清单和可执行示例。
每次生成会在新工作目录记录操作系统、架构、编译器、锁文件和字体策略；绘图器不暴露
精确解析到的字体文件。请使用已说明、预先安装的字体，不将生成结果视为跨环境字节基准。
机械检查证明的是结构及源码对应关系，不证明文字语义等价。
