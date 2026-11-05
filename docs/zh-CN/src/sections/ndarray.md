<a id="ndarray"></a>
## ndarray：保留逻辑轴

[English](../en/index.html#ndarray)

![从隔列选取的源数据生成带标签响应网格](gallery/ndarray/strided_heatmap.svg)

`lichtung-plot-ndarray` 直接接受逻辑视图。25 个图族都提供直接变量构造器
`new(...)` 和按源列绑定的 `from_data(view2, ...)`，两者返回同一种精确图族
`PlotAdapter`。原生 `Plot`、`Canvas`、策略、`kind`、`stats`、`Error` 和 `Result`
也原样重导出，独立渲染器不依赖 ndarray。

本合成实验交错放置质量控制列。非连续视图只选择响应列，独立布局的有效性网格
明确标记一个缺失测量。真值表示存在，另一份高亮网格中的真值只添加边框。
单元格标识保持精确且唯一，不把同一个源行标识重复到各列：

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-values}}
```

网格标识／高亮也可绑定有序的源列索引集合，所选行列形状必须与网格逐维一致。

单元 `(r,c)` 对应 x 位置 `c`、y 位置 `r`，按逻辑行优先顺序展开，零行位于顶部。
内部轴身份独立于显示标签；连续的 x／y／行标签调用相互保留，而 `.labels(...)`
替换完整策略。直接配置并发布：

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-plot}}
```

| 输入操作 | 语义 |
| --- | --- |
| `Scatter::new(x.view(), y.view())` | 直接一维变量；数值基本类型及其 `Option` 形式正常舍入到 `f64`。 |
| `Scatter::from_data(table.view(), 0, 2)` | 行是观测，`usize` 选择逻辑列；同一次调用可混用选择器和直接视图。 |
| `Line::from_data(table.view(), 0, [2, 1])` | 一个真实 x 与有序多 y，保留不等间距坐标，绝不换成行位置；直接 y 可为二维视图。 |
| `Line::indexed(view2)` 及 Scatter 对应方法 | 显式使用从零开始的行位置作 x；源绑定形式选择索引或 `..`。 |
| `Boxplot::columns(view2)` 及 Violin／Raincloud 对应方法 | 列身份是类别，单元格是样本；普通类别＋多 y 构造器则使用定位系列。 |
| `Heatmap::new(view2)`／`records(x, y, fill)` | 明确区分矩形与分类坐标的单元格记录，不隐式分箱。 |
| `Matrix::new(view2)` | 列是散点矩阵变量，不是热图单元；`column_labels` 只改变显示元数据。 |

空选择、重复和越界索引在绑定时报错。普通 `..` 表示全部列，数值发现另用
`indexed_numeric_from_data` 等显式名称。数组只有一种元素类型，数值列不会自动
变成分类着色；为 `.hue(...)`、`.group(...)`、`.series(...)` 或分面另传字符串视图。
字符串保留原始键，布尔值不转换为数值 0/1。

宽表展开后，普通类别／分面和观测透明度绑定按源行重复；`_cells` 明确绑定展开槽位，
`hue_columns`、`group_columns`、`series_columns` 和列维度分面方法选择生成维度。
绝不按长度猜作用域。多 y 的 Line 上下界必须是匹配的列集合／矩阵，不能复用一对界限。

`estimate_status_boolean` 和 `estimate_status_categorical` 显式指定可估计性。
透明度接受非空精确 `u8` 变量、显式单值（`64u8` 与选择器 `0usize` 不同）、原生映射及
键控／分箱单位。数值／分类 `_map` 回调接收适配器准备的原生值；行映射只在展开前物化一次，
保留完整声明。生成列透明度的键使用从零开始的源索引，直接矩阵则用局部列序号，不使用显示标签。
独立 `_from_data` 选择器／结果源在调用时解析，不与主图联接。

`significance_pairwise`／`significance_attached` 绑定已提供的 p 值或原生标记；类别／系列
目标对可以嵌套 `(values, present)`。分面／网格形式保留精确归属，原生目标／标记标量明确
广播，其余输入按结果数量对齐，不受观测行数限制。根级复用函数返回原生标注集合，不计算
检验或校正 p 值。独立 `PointLayerAdapter` 和 `BarPointLayerAdapter` 同样只绑定自身的
源和记录数量。

有效性对的每一维形状都必须相同，但步长可以不同。假值和 `Option` 缺失均保留空槽位；
有效的 `NaN`／无穷大不改为空值。转置、反向、Fortran 存储和调用方已创建的广播都按逻辑
顺序读取，不新增排序、过滤、填补、数值转字符串或跨通道广播。科学验证仍由原生实现负责。

每次绑定只快照本次输入。`from_data` 为后续选择器保留只读源借用，直接视图则在绑定后释放。
标准 `.into()` 交出 `Plot<kind::Family>` 并结束查找，随后可放入原生 Canvas 或修改源。
冻结的 Output 不再读取源。绑定使用 `AdapterError`，终结方法返回原生结果，原生错误可无损
转换为 `AdapterError`。没有公开转换函数目录或额外结果别名。

运行 `cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap`。
无需输出参数；可用一个目录重定向输出，目录不存在时由示例创建。轮廓／保留文字两份 SVG
复用同一份冻结的原生输出对象。
[完整源码](examples/lichtung-plot-ndarray/strided_heatmap.rs) ·
[绘图接口文档](https://docs.rs/lichtung-plot-ndarray/latest/lichtung_plot_ndarray/index.html) ·
[返回原生数据](#data)
