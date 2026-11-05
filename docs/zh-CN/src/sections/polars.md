<a id="polars"></a>
## Polars：直接使用字段与列变量

[English](../en/index.html#polars)

![从可空 Polars 数据框转换得到的分组合成观测](gallery/polars/grouped_observations.svg)

`lichtung-plot-polars` 直接接受数据框加字面字段名，或借用的 `Series`／`Column`。
`Scatter::from_data(frame, "x", "y")` 保留该数据框供后续 `.hue("condition")`
查找字段；`Scatter::new(&x, &y)` 直接绑定变量，不制造隐式表。两者均返回精确图族
的 `PlotAdapter`，原生渲染器 API 也通过适配器原样重导出。适配器直接依赖
`polars-core` 0.55.2，匹配版本的主包重导出相同 Polars 类型；核心渲染器不依赖这些外部框架。

本合成示例将两个响应数据块追加在一起，并保留一个空响应。样本标识对应原始
观测，与当前槽位编号相互独立：

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:chunked-values}}
```

绑定保留全部输入槽位。原生散点图准备阶段决定哪些观测适用，适配器不过滤或
重编号。已选数据由原生值持有，但后续字段查找仍借用数据框；示例显式转换为原生
Plot 以释放该上下文，再于发布前释放数据框，复用冻结的输出对象发布第二份 SVG：

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:observation-plot}}
```

| 输入角色／数据类型 | 语义 |
| --- | --- |
| 数值坐标、样本、估计值或上下界 | 有符号／无符号 8/16/32/64/128 位整数及 16/32/64 位浮点数；正常舍入到 `f64`，不设精确表示门槛。 |
| `.hue("condition")` 等分类角色 | 字符串、分类或枚举类型，解码精确字符串身份，不使用字典码。原生顺序取首次出现顺序，不继承枚举的声明顺序。 |
| 布尔热图高亮 | 布尔真值、假值或空值，不隐式转成数值或字符串。 |
| 未指定类型的空值列 | 数值／分类／布尔角色决定目标通道，得到逻辑长度不变的全空快照；轨迹 x 含义不明时须使用显式轴构造器。 |
| `.source_rows("sample_id")` | 整数行标识精确检查 `usize` 范围，再检查原生唯一性；空值、负数、越界及非整数均报错。 |
| 热图／散点矩阵列选择 | 按顺序选择字面字段；`..` 表示全部字段而非自动筛选数值列。`Matrix::numeric_from_data` 显式发现支持的数值字段。 |
| 时间、十进制定点数、二进制、嵌套、对象和未知数据类型 | 不做隐式解释，空列也不例外；调用方先明确单位或显式投影。 |

转换遍历全部数据块并尊重切片有效性偏移。重复标量列使用逻辑长度，不把
单元素底层序列当成整列；完整物化可能分配内存并填充 Polars 内部缓存。支持类型
的空列和全空值列保持原样，有效的 `NaN`、无穷大和带符号零不等于空值。后续修改源值
或重新分块不会改变返回的快照。

不修剪名字或原始键，不进行人类化格式化。未观测的字典级别不变成额外行；如需精确
类别顺序，对实际渲染类别显式设置原生 `CategoryOrder::explicit`。转换不排序、丢行、
填补、聚合、解析字符串，也不执行延迟查询或输入输出操作。错误链保留 Polars／原生来源；
无效行标识的诊断使用逻辑槽位，不猜测原始标识。

目前 25 个图族均支持记录式单变量绑定，同时支持网格热图、散点矩阵、显式区间、
柱图输入模式和独立点层。多 y、行位置／列分布模式、展开后的行／单元格元数据，以及
外部透明度／状态／显著性映射仍在实现；原生策略和原生标注已可使用。绑定错误使用
`AdapterError`，输出保留原生 `Error`，组合工作流可通过无损转换使用 `?`。
旧的公开值转换函数不再导出。

运行 `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`。
无需输出参数；可用一个目录重定向 SVG，目录不存在时由示例创建。适配器不启用
主包的输入输出或延迟查询功能，但上游 `polars-core` 仍包含 Tokio 和异步支持依赖，不能宣称依赖图
不含运行时。
[完整源码](examples/lichtung-plot-polars/grouped_observations.rs) ·
[绘图接口文档](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[返回原生数据](#data)
