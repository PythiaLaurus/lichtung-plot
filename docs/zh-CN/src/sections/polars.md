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

25 个图族均支持直接／源绑定、显式区间、独立的柱图样本／估计模式和原生预供几何。
独立 `PointLayerAdapter` 和 `BarPointLayerAdapter` 使用自己的源与数量，拟合线／区间
不会继承观测层的标签、着色或分面。

| 显式操作 | 语义 |
| --- | --- |
| `Line::from_data(&frame, "time", ["b", "a"])` 及 Scatter／样本分布对应形式 | 保留真实 x／类别和选择顺序，列身份提供原生分组／着色或定位系列；直接多 y 接受有序借用的 Series／Column 集合。 |
| `Line::indexed_from_data(&frame, ["a", "b"])` 及 Scatter 对应方法 | 显式以行位置作 x；`indexed_numeric_from_data` 按模式顺序发现支持的数值列。 |
| `Boxplot::columns_from_data(&frame, ["a", "b"])` 及 Violin／Raincloud 对应方法 | 列身份是类别，单元格是样本；`numeric_columns_from_data` 显式发现数值列。 |
| `Heatmap::from_data(&frame, ["a", "b"])?.row_labels("sample")?` | 矩形与独立的行显示元数据；`records_from_data` 则绑定分类坐标的单元格记录。 |
| `Matrix::from_data(&frame, ["b", "a"])?.column_labels(["B", "A"])?` | 有序散点矩阵变量与独立显示标签，不推断热图。 |

空选择、重复和缺失字段在绑定时报错，选择全部不会静默排除元数据。数值发现排除
未指定类型的空值列，显式数值选择则按数值解释它。真实源名保留为身份，直接集合按局部
序号区分，各自名字作为显示元数据；重复显示名仍由原生碰撞验证处理，不静默合并或改名。

宽表／网格按逻辑行优先展开。普通着色／分组／系列、分面和观测透明度绑定按源行重复，
`_cells` 明确绑定展开槽位，即使长度碰巧相同也不猜作用域。列维度角色和分面方法显式选择
生成维度，不使用魔法保留字段名。单元格唯一标识绝不由重复行标识生成。网格高亮／标识矩形
必须逐维匹配；`heatmap_highlight_cells` 和 `source_rows_cells` 也接受显式扁平向量。
高亮真值只添加边框，与空值有效性无关。多 y 的 Line 上下界要求匹配列集合。
显示标签调用相互保留，`.labels(...)` 替换完整原生策略，后续操作不会复活被替换的旧别名。

状态方法显式映射布尔或分类可估计性，缺失估计值不会自动推断状态。透明度使用非空精确
`u8` 变量、显式单值、原生数值／分类映射和实际键控／分箱单位，不任意归约观测。
`_map` 回调接收适配器准备的原生值，只物化一次，按行展开时保留映射声明。
生成列透明度以原始源名为键，直接列集合则使用从零开始的局部序号。独立键控／分箱
`_from_data` 表在调用时解析，不替换主源，也不保留结果表借用。

`significance_pairwise_from_data`／`significance_attached_from_data` 绑定独立结果表。
目标可为类别绑定、类别／系列对或原生目标；标记可为已提供的 p 值、原生标记或显式分类
映射。原生标量明确广播，其余输入按结果数量对齐，不受观测行数限制；单维／网格分面形式
保留精确归属。根级准备函数返回可复用的原生标注，普通 `.significance(...)` 原样追加。
不执行假设检验或 p 值校正。

原生 Canvas 组合先显式标准转换为精确原生 Plot。解析到同一个兼容核心包时，本适配器和
ndarray 适配器的原生类型完全相同，两个适配器互不依赖。绑定错误使用 `AdapterError`，
输出保留原生 `Error`／`Result`，组合工作流可通过无损转换使用 `?`，无需公开转换函数
目录或额外适配器结果别名。

运行 `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`。
无需输出参数；可用一个目录重定向 SVG，目录不存在时由示例创建。适配器不启用
主包的输入输出或延迟查询功能，但上游 `polars-core` 仍包含 Tokio 和异步支持依赖，不能宣称依赖图
不含运行时。
[完整源码](examples/lichtung-plot-polars/grouped_observations.rs) ·
[绘图接口文档](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[返回原生数据](#data)
