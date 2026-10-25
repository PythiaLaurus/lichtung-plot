<a id="polars"></a>
## Polars：已求值的数据列与原生快照

[English](../en/index.html#polars)

![从可空 Polars 数据框转换得到的分组合成观测](gallery/polars/grouped_observations.svg)

`lichtung-plot-polars` 接受 `&Column`，即现代数据框列查询返回的类型。
独立的 `Series` 使用 Polars 已有的 `IntoColumn` 转换。适配器直接依赖
`polars-core` 0.55.2，匹配版本的 `polars` 主包重导出相同类型；核心渲染器不依赖这些外部框架。

本合成示例将两个响应数据块追加在一起，并保留一个空响应。样本标识对应原始
观测，与当前槽位编号相互独立：

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:chunked-values}}
```

转换保留全部输入槽位。原生散点图准备阶段决定哪些观测适用，适配器不过滤或
重编号。转换后数据均由原生值持有，因此示例在发布前释放数据框，再复用冻结的
原生输出对象发布第二份 SVG：

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:observation-plot}}
```

| 函数／数据类型 | 语义 |
| --- | --- |
| `numeric(&Column)` | 有符号／无符号 8/16/32/64/128 位整数及 16/32/64 位浮点数；正常舍入到 `f64`，不设精确表示门槛。 |
| `categories(&Column)` | 字符串、分类或枚举类型，解码精确字符串身份，不使用字典码。原生顺序取首次出现顺序，不继承枚举的声明顺序。 |
| `booleans(&Column)` | 布尔真值、假值或空值，不隐式转成数值或字符串。 |
| 未指定类型的空值列 | 显式选择上述任一值通道，得到逻辑长度不变的全空快照。 |
| `source_rows(&Column)` | 整数行标识精确检查 `usize` 范围，再检查原生唯一性；空值、负数、越界及非整数均报错。 |
| `numeric_columns(&DataFrame, names)` | 仅转换明确指定的数值变量，保留选择顺序；无正则、表达式或自动发现，缺列／错类型报错。 |
| 时间、十进制定点数、二进制、嵌套、对象和未知数据类型 | 不做隐式解释，空列也不例外；调用方先明确单位或显式投影。 |

转换遍历全部数据块并尊重切片有效性偏移。重复标量列使用逻辑长度，不把
单元素底层序列当成整列；完整物化可能分配内存并填充 Polars 内部缓存。支持类型
的空列和全空值列保持原样，有效的 `NaN`、无穷大和带符号零不等于空值。后续修改源值
或重新分块不会改变返回的快照。

不修剪名字或原始键，不进行人类化格式化。未观测的字典级别不变成额外行；如需精确
类别顺序，对实际渲染类别显式设置原生 `CategoryOrder::explicit`。转换不排序、丢行、
填补、聚合、解析字符串，也不执行延迟查询或输入输出操作。错误链保留 Polars／原生来源；
无效行标识的诊断使用逻辑槽位，不猜测原始标识。

运行 `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`。
无需输出参数；可用一个目录重定向 SVG，目录不存在时由示例创建。适配器不启用
主包的输入输出或延迟查询功能，但上游 `polars-core` 仍包含 Tokio 和异步支持依赖，不能宣称依赖图
不含运行时。
[完整源码](examples/lichtung-plot-polars/grouped_observations.rs) ·
[转换接口文档](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[返回原生数据](#data)
