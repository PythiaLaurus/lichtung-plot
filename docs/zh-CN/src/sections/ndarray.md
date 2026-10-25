<a id="ndarray"></a>
## ndarray：保留逻辑轴

[English](../en/index.html#ndarray)

![从隔列选取的源数据生成带标签响应网格](gallery/ndarray/strided_heatmap.svg)

`lichtung-plot-ndarray` 仅在转换时借用 `ArrayView1` 或 `ArrayView2`，随后返回
自有的原生快照，核心渲染器不依赖 ndarray。本合成实验刻意交错放置质量控制列；
非连续视图只选响应列，不要求先复制为连续数组。

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-values}}
```

网格单元 `(r,c)` 对应 x 键 `c`、y 键 `r`；即使转置或反向，仍按逻辑行优先
顺序展开。轴键长度必须对应维度，同一轴内不得重复。之后直接使用原生绘图配置：

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-plot}}
```

| 转换 | 接受的数据与语义 |
| --- | --- |
| `numeric(name, view1)` | `f32`／`f64`、至 128 位的整数及 `isize`／`usize`，包含 `Option` 形式；正常舍入到 `f64`，不设精确表示门槛。 |
| `categories(name, view1)` | `String`／`&str` 及其 `Option` 形式；保留原始键，按首次出现顺序建立原生类别。 |
| `booleans(name, view1)` | `bool`／`Option<bool>`，不隐式转换为 0/1。 |
| `source_rows(view1)` | 整数行标识，精确检查 `usize` 范围并使用原生唯一性检查；空值报错。 |
| `numeric_columns(names, view2)` | 行是观测、列是命名变量，供原生散点矩阵等使用，不隐式变成热图。 |
| `heatmap_values(name, view2, x_keys, y_keys)` | 将显式带标签网格展开为原生 x/y/fill。 |

每种值、表格和网格转换都有 `_with_validity` 形式。布尔有效性视图必须与数据
形状完全一致，**真值表示有效**，与 NumPy 掩码数组的约定相反。
假值和 `Option::None` 均保留空槽位；有效的 `NaN`／无穷大不会被改为空值。
不丢行、排序、跨通道广播或填补，支持的空视图仍为空。科学有效性与通道长度
对齐仍由后续原生消费者检查。

复数、自定义对象和带科学语义的数据类型需要调用方先显式投影。不修剪或格式化名字与键。
`SourceRows` 不经过 `f64`，也不从切片猜测原始行标识；展开的热图需要每个单元一个标识。
不保留外部缓冲区，因此后续修改源数据不会改变快照。适配器错误保留结构／逻辑
槽位上下文与原生错误来源。

运行 `cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap`。
无需输出参数；可用一个目录重定向 SVG，目录不存在时由示例创建。两次发布复用同一份
冻结的原生输出对象。
[完整源码](examples/lichtung-plot-ndarray/strided_heatmap.rs) ·
[转换接口文档](https://docs.rs/lichtung-plot-ndarray/latest/lichtung_plot_ndarray/index.html) ·
[返回原生数据](#data)
