<a id="data"></a>
## 原生数据与科学责任

[English](../en/index.html#data)

绘图器接收 `NumericValues`、`CategoryValues`、`BooleanValues` 及明确的科学输入类型，
不依赖 dataframe 运行时。它们持有不可变快照。外部容器转换属于应用边界；调用者之后
修改容器，不会改变已经构造的图。

数值保留原始对齐槽位，并将有效性单独保存。null 与 NaN 不能互换；非有限值或不可用
数据是否适用，由实际图族判断。`SourceRows` 提供诊断身份，不是压缩后的数组下标。
请区分行位置、来源 ID 和原始类别键；过滤一个通道时，其他关联通道必须同步对齐。

类别默认按首次观察顺序排列。显式顺序必须恰好包含每个渲染类别；按值排序需要每个
类别有无歧义的值。`Labels::humanized()` 修改展示文字，`Labels::raw()` 保留原始拼写，
alias 覆盖展示标签。它们都不改变科学身份，完整类别域仍会接受展示名称歧义检查。

### 样本、估计和给定几何

- `BarInput::samples` 展示单元均值，默认使用确定性的 95% 百分位 bootstrap 区间。
  `ErrorBar::confidence_interval().level(...)?` 可指定水平，并选择 Student-t，
  或显式配置 `Bootstrap` 的种子和重采样次数。
- `BarInput::estimates`、`estimates_with_error_bars` 接收调用者给定的估计及边界，
  不自行赋予置信区间或可信区间含义。
- `HistogramInput`、`RaincloudInput` 在构造时选择样本或完整给定几何，之后不能混用。
- 样本直方图可以保留自己的分箱和 rug，同时通过
  `HistogramComposite::KdeAndRug(HistogramKde::Precomputed(...))` 添加调用者计算的 KDE；
  坐标使用具名的 `HistogramKdePoint`。`HistogramStat::Density` 选择展示的统计量。

绘图器可以计算明确约定的展示统计，但不选择假设检验。请将调整后的 p 值作为
`Significance` 展示数据传入。下面的完整示例构造显式两两比较，本身不进行推断检验：

```rust
{{#include ../../../../crates/lichtung-plot/examples/03_inject_stats.rs:example}}
```

`Significance::between` 推断 `Pairwise` 放置类型；附着式注释使用另一种类型化放置。
各图族的能力决定允许哪些放置方式。参见[图族选择](#families)和[文本注释](#text)。
依赖数据的对齐、域及注释错误，会尽可能在字体工作开始前检查。
