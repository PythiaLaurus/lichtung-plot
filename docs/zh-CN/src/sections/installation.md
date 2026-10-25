<a id="installation"></a>
## 安装与首张图

[English](../en/index.html#installation)

当前开发仓库使用 Rust 1.96.0。绘图核心位于 `crates/lichtung-plot`，是工作区的默认
成员。[ndarray 适配器](#ndarray)与 [Polars 适配器](#polars)位于独立包中，语言绑定尚未实现。
在正式版本发布之前，相邻的 Rust 应用可以引用实际成员路径：

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot/crates/lichtung-plot" }
```

绘图器无需初始化。生成图形前，请自行安装合适的系统字体；库不会下载字体。
默认有序字体栈为 Noto Sans、Noto Sans CJK SC、DejaVu Sans、Liberation Sans、Arial、
sans-serif。选定字体栈必须真正覆盖可见文字，详见[输出与字体](#output)。

<a id="first-plot"></a>
### 代码讲解：从数值到首张图

![不同底物浓度下的合成酶反应数据](gallery/first-plot/quick_start.svg)

先准备逐行对齐的底物浓度、反应速率和酶身份。两种响应使用明确的示意饱和方程与固定
的正乘性扰动，不是实验测量，也不是拟合得到的动力学。完整示例在内存里生成这些数值，
不下载数据集：

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-values}}
```

将两个数值系列包装成具名 `NumericValues`，传入 `Plot::scatter`；`hue` 单独保留酶身份，
不改变坐标。共用图库主题只改变外观，不改变科学数据。`base_path` 是当前工作目录下
的 `quick_start`；`.svg` 补上 SVG 后缀，返回冻结的 Output，而不是一个可变后端：

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-publication}}
```

示例明确请求 840 × 560 CSS 像素，让完整标签在图库中仍然可读。这是示例自己的发布
选择，不是渲染器默认值。共用主题同样通过普通公开 API 配置：

```rust
{{#include ../../../../crates/lichtung-plot/examples/support/gallery.rs:gallery-theme}}
```

可以无参数运行精确的示例源码，无需自己拼接摘录。从工作区根目录运行：

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start
```

每个示例都可以无参数直接运行。此示例默认向当前工作目录写入 `quick_start.svg`，
并报告冻结后的图形尺寸。有需要时可追加 `-- <directory>` 重定向输出；该可选目录会在
不存在时创建，相对路径以当前工作目录为基准。图库构建会自动安排隔离目录，
不要求用户手动提供。`--offline` 要求 Rust 依赖已经缓存。

`.svg` 发布轮廓 SVG。终端渲染返回不透明的 `Output`，后续可发布其他格式而不重新渲染。
[原生数据](#data)解释构造函数所接收的数据。

[完整源码](examples/lichtung-plot/01_quick_start.rs) · [共享输出辅助代码](examples/lichtung-plot/support/output.rs) ·
[图库主题](examples/lichtung-plot/support/gallery.rs) · [合成输入](examples/lichtung-plot/support/synthetic.rs) ·
[原尺寸 SVG](gallery/first-plot/quick_start.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)
