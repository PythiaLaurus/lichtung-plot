<a id="family-walkthroughs"></a>
## 更多科学绘图讲解

[English](../en/index.html#family-walkthroughs)

这六个程序在前述工作流之外增加十八个图族面板，使用合成测量或调用者给定的示意值，
不报告研究发现。比较图元之前，先看各面板的变量及轴单位。每张图都包含所属程序的完整
源码引用，不维护另一套绘图实现。

<a id="distribution-diagnostics"></a>
### 分布诊断：形状、参照与摘要

[English](../en/index.html#distribution-diagnostics)

![同组合成测量的 Density、正态 Qq、Boxplot 与 Violin 视图](gallery/distribution-diagnostics/distribution_diagnostics.svg)

Density、Boxplot 和 Violin 使用全部三组的合成延迟。Density 突出平滑后的分布形状；
Boxplot 把各组观测压缩成摘要，Violin 则保留其密度形状。Qq 只使用 Warm hold 的
72 个观测，并采用基于四分位数的正态参考线。此处偏离参照只是描述性示意，
不是正态性检验，也不是显著性判断。

源码将样本分别绑定到这些不同图族，而不是把一个图族准备好的几何当作另一个图族的数据：

```rust
{{#include ../../../../crates/lichtung-plot/examples/18_distribution_diagnostics.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 18_distribution_diagnostics`。
[完整源码](examples/lichtung-plot/18_distribution_diagnostics.rs) ·
[原尺寸 SVG](gallery/distribution-diagnostics/distribution_diagnostics.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)

<a id="estimate-comparisons"></a>
### 估计比较：先选问题，再选图元

[English](../en/index.html#estimate-comparisons)

![使用给定数值的 PointInterval、Coefficient、Dumbbell、Bar 与 HorizontalBar](gallery/estimate-comparisons/estimate_comparisons.svg)

PointInterval 展示带边界的位置估计；Coefficient 展示具名加性对比，以零表示无效应。
Dumbbell 连接前后两个位置，明确表达配对。Bar 展示保温后的绝对滴度，
HorizontalBar 则展示后减前的变化量，其零点表示无变化。各面板的问题和数量都不同，
不只是图元朝向不同。

估计、区间端点和配对位置全部由调用者提供。请按标出的单位阅读；渲染器没有估计推断模型、
计算这些区间或检验差异：

```rust
{{#include ../../../../crates/lichtung-plot/examples/19_estimate_comparisons.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 19_estimate_comparisons`。
[完整源码](examples/lichtung-plot/19_estimate_comparisons.rs) ·
[原尺寸 SVG](gallery/estimate-comparisons/estimate_comparisons.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)

<a id="ordered-processes"></a>
### 有序过程：身份、路径、条件与累积

[English](../en/index.html#ordered-processes)

![重复轨迹、有序路径、条件剖面与瀑布图](gallery/ordered-processes/ordered_processes.svg)

Trajectory 保留重复单元的分组身份；LinePath 保持来源顺序，不把回环替换为普通按 x
排序的线；Interaction 比较分类条件上的剖面；Waterfall 从显式起始总量累积有序增量。
这是四种不同的连接或顺序语义，不是四种线条样式。

合成变量及单位由示例给出。单元身份、路径顺序、条件身份和起始总量应保留在输入中，
不能要求主题或展示标签替数据重建这些含义：

```rust
{{#include ../../../../crates/lichtung-plot/examples/20_ordered_processes.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 20_ordered_processes`。
[完整源码](examples/lichtung-plot/20_ordered_processes.rs) ·
[原尺寸 SVG](gallery/ordered-processes/ordered_processes.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)

<a id="composition-profiles"></a>
### 组成剖面：堆叠不是带颜色的观测行

[English](../en/index.html#composition-profiles)

![竖向及横向堆叠剖面与连续颜色 Beeswarm 观测](gallery/composition-profiles/composition_profiles.svg)

StackedBar 与 StackedHorizontalBar 必须同时给出分类和系列身份，才能描述每个堆叠中的
组成部分。Beeswarm 使用的则是分类行、数值观测位置和连续颜色变量。三个面板对照组成
剖面与个体观测；连续颜色不能替代堆叠图必需的系列通道。

这些量均为合成演示。结合标注的轴和颜色图例，区分数值大小、行身份与额外的颜色测量量：

```rust
{{#include ../../../../crates/lichtung-plot/examples/21_composition_profiles.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 21_composition_profiles`。
[完整源码](examples/lichtung-plot/21_composition_profiles.rs) ·
[原尺寸 SVG](gallery/composition-profiles/composition_profiles.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)

<a id="joint-measurements"></a>
### 联合测量：关系及其边际分布

[English](../en/index.html#joint-measurements)

![带原生边际展示的成对合成实验测量](gallery/joint-measurements/joint_measurements.svg)

联合视图关注两个测量量如何一起变化，边际视图分别展示每个变量自身的分布。
Marginal 让这些视图共同对应成对的合成实验输入及其单位。图形本身不证明独立性、
因果关系，也不执行推断检验。

这里发布独立的 Marginal 图，不是通用分面配方：

```rust
{{#include ../../../../crates/lichtung-plot/examples/22_joint_measurements.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 22_joint_measurements`。
[完整源码](examples/lichtung-plot/22_joint_measurements.rs) ·
[原尺寸 SVG](gallery/joint-measurements/joint_measurements.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)

<a id="multivariate-assay"></a>
### 多变量实验：具名变量，而非任意单元

[English](../en/index.html#multivariate-assay)

![具名数值实验变量组成的独立散点矩阵](gallery/multivariate-assay/multivariate_assay.svg)

Matrix 通过两两视图比较多个具名数值实验变量。变量名称与单位使每一对坐标可解释；
它不是从矩形数组猜出的 Heatmap，也不是任意 Canvas 单元的集合。
这些合成关联用于演示，不构成因果发现。

Matrix 自己拥有完整网格，必须独立发布，不能插入 Canvas 或通用分面。

```rust
{{#include ../../../../crates/lichtung-plot/examples/23_multivariate_assay.rs:example}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 23_multivariate_assay`。
[完整源码](examples/lichtung-plot/23_multivariate_assay.rs) ·
[原尺寸 SVG](gallery/multivariate-assay/multivariate_assay.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [返回图库](#gallery)
