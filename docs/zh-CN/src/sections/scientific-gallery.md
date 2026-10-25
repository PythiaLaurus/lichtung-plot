<a id="gallery-recipes"></a>
## 科学绘图案例

[English](../en/index.html#gallery-recipes)

每张图从一个问题出发，而不是罗列 setter。首页展示的数值都是合成数据或明确给定的
示意值，不是研究结果。示例共用[图库主题](examples/support/gallery.rs)与
[输出辅助代码](examples/support/output.rs)；需要生成扰动的示例还使用随包提供的
[合成输入](examples/support/synthetic.rs)。

<a id="distributions"></a>
### 雨云图：不要只看均值

![样本分布、统计摘要与逐个观测值](gallery/distributions/distributions.svg)

练习可能改变分布中心，却未必消除长尾。雨云图同时保留密度、摘要及每个观测值。
这里每种条件有 80 个固定合成样本，中心、离散程度不同，并加入少量右偏贡献，
没有执行显著性检验。

```rust
{{#include ../../../../crates/lichtung-plot/examples/13_distributions.rs:distribution-values}}
```

向 `RaincloudInput::samples` 提供样本，而不是预计算四分位数。绘图器负责展示统计摘要，
示例不作推断声明。类别身份已由构造输入拥有，无需再添加一个 hue 通道。

```rust
{{#include ../../../../crates/lichtung-plot/examples/13_distributions.rs:distribution-plot}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 13_distributions`。
[完整源码](examples/13_distributions.rs) · [原尺寸 SVG](gallery/distributions/distributions.svg) ·
[RaincloudInput API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.RaincloudInput.html) · [返回图库](#gallery)

<a id="expression-heatmap"></a>
### 热图：观察随时间协同变化的响应

![有序矩阵中的合成表达模式](gallery/expression-heatmap/expression_heatmap.svg)

八行八列的矩阵呈现早期诱导、短暂抑制与后期恢复，无需把每个单元变成一张小图。
行与时间标签按预期顺序提供，绘图器不推断聚类或生物学含义。列间距是分类间距，
不与经过的时间成正比。

```rust
{{#include ../../../../crates/lichtung-plot/examples/14_expression_heatmap.rs:expression-values}}
```

显式发散色阶把零放在中性色，并固定整个域为 [-2, 2]。这个统一参照比给每个单元装饰
一个数字更重要，让正负两侧保持可比较。

```rust
{{#include ../../../../crates/lichtung-plot/examples/14_expression_heatmap.rs:expression-plot}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 14_expression_heatmap`。
[完整源码](examples/14_expression_heatmap.rs) · [原尺寸 SVG](gallery/expression-heatmap/expression_heatmap.svg) ·
[ContinuousColorScale API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.ContinuousColorScale.html) · [返回图库](#gallery)

<a id="effect-intervals"></a>
### 森林图：把估计放回不确定性中

![围绕零参照的给定效应估计及区间](gallery/effect-intervals/effect_intervals.svg)

一个点并不代表全部结果。完整区间让人看到哪些示意效应仍接近无效应参照，哪些分离
得更明显。下面每个端点都由调用者给定；它们不是临床研究结果、计算得到的置信区间，
也不代表自动作出的显著性判断。

```rust
{{#include ../../../../crates/lichtung-plot/examples/15_effect_intervals.rs:effect-plot}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 15_effect_intervals`。
[完整源码](examples/15_effect_intervals.rs) · [原尺寸 SVG](gallery/effect-intervals/effect_intervals.svg) ·
[Bounds API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Bounds.html) · [返回图库](#gallery)

<a id="time-response"></a>
### 折线与带：响应如何随时间展开

![带有给定包络的三种响应动力学](gallery/time-response/time_response.svg)

这三条合成曲线的幅度和速度均不同。阴影包络是给定的示意边界，不是拟合得到的
置信区间。上下界与各曲线保留完全相同的逐行对应关系。

```rust
{{#include ../../../../crates/lichtung-plot/examples/16_time_response.rs:response-values}}
```

`hue` 区分三个系列，`band` 消费与它们对齐的边界。这里关闭点标记，因为图形要表达
完整轨迹及包络，而不是独立抽样的观测值。

```rust
{{#include ../../../../crates/lichtung-plot/examples/16_time_response.rs:response-plot}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 16_time_response`。
[完整源码](examples/16_time_response.rs) · [原尺寸 SVG](gallery/time-response/time_response.svg) ·
[Bounds API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Bounds.html) · [返回图库](#gallery)

<a id="calibration"></a>
### Canvas：校准关系与残差结构

![校准关系旁的残差分布](gallery/calibration/calibration.svg)

一个面板询问信号是否随浓度变化，另一个呈现相对于已知生成直线的误差分布。
这些误差不是图中描述性 OLS 拟合的残差。明确区分两者，避免把方便的合成示例变成
容易误解的分析。

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-values}}
```

回归图只作描述性展示，显式关闭区间带。直方图采用密度归一化、KDE，以及保留原始
残差的 rug 标记。

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-panels}}
```

两幅图继承同一个完整 Canvas 主题，在统一的发布框架中组合，但不将两个含义不同的
x 轴伪装成共享轴。

```rust
{{#include ../../../../crates/lichtung-plot/examples/17_calibration.rs:calibration-canvas}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 17_calibration`。
[完整源码](examples/17_calibration.rs) · [原尺寸 SVG](gallery/calibration/calibration.svg) ·
[Canvas API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Canvas.html) · [返回图库](#gallery)
