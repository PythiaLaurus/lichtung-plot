# 安装与首张图

[English](../en/installation.html)

当前开发仓库使用 Rust 1.96.0。绘图核心位于 `crates/lichtung-plot`，是默认 workspace
成员；本指南不声称 adapter 或语言绑定已经交付。在正式版本发布之前，相邻的 Rust
应用可以引用实际成员路径：

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot/crates/lichtung-plot" }
```

绘图器无需初始化。生成图形前，请自行安装合适的系统字体；库不会下载字体。
默认有序字体栈为 Noto Sans、Noto Sans CJK SC、DejaVu Sans、Liberation Sans、Arial、
sans-serif。选定字体栈必须真正覆盖可见文字，详见[输出与字体](output.md)。

从 workspace 根目录运行：

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start
```

每个示例都可以无参数直接运行。此示例默认向当前工作目录写入 `quick_start.svg`，
并报告冻结后的图形尺寸。有需要时可追加 `-- <directory>` 重定向输出；该可选目录会在
不存在时创建，相对路径以当前工作目录为基准。图库构建会自动安排隔离目录，
不要求用户手动提供。`--offline` 要求 Rust 依赖已经缓存。

```rust
{{#include ../../../crates/lichtung-plot/examples/01_quick_start.rs:example}}
```

`.svg` 发布轮廓 SVG。终端渲染返回不透明的 `Output`，后续可发布其他格式而不重新渲染。
[首张图案例](gallery-first-plot.md)展示的就是这份程序生成的结果。
[原生数据](data.md)解释构造函数所接收的数据。
