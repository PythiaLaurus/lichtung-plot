# 坐标轴与数值呈现

[English](../en/axes.html)

请区分科学坐标及域、刻度**位置**（`TickGeneration`）和数值**拼写**（`NumericFormat`）。
设置范围不会选择格式，添加后缀也不会改变数据。精确的 `x_ticks`／`y_ticks` 是不可拆分
且优先的来源，无论 setter 顺序如何；空标签也会保留刻度。生成数量上限不能截断精确
列表。实际分类轴拒绝数值指令；Any／Matrix 及 Heatmap 专用轴不会因此获得通用数值 setter。

## 位置来源

| `TickGeneration` 构造函数 | 含义 |
| --- | --- |
| `automatic()` | 将完整选择重置为基于测量的默认策略 |
| `target(n)` | 约 n 个位置，允许漂亮步长偏差及变稀 |
| `maximum(n)` | 限制域内完整生成列表的数量，包括端点 |
| `additive(step)` | 首先尝试给定的正线性间距 |
| `log10_decades(step)` | 首先尝试 Forest Log10 的整数指数间距 |
| `exact_count(n, space)` | 精确 n 个包含端点的仿射位置，零表示空来源 |
| `interval(step, origin, space, requirement)` | 与原点对齐的固定或优先格点 |
| `multiplicative(anchor, ratio, requirement)` | 两个方向均可的锚定几何格点 |
| `function(callback)` | 根据最终实际视图产生可能失败的位置 |

数量使用 `usize`，没有统一的 10,000 位置上限。Target／Maximum 必须为正。
`maximum_count(Some(n))` 添加一个调用者明确指定的正上限，不替换来源；`None` 清除上限。
上限不能小于 ExactCount。数量及参数提供只读查看，不暴露实现枚举。

ExactCount(1) 默认取 Lower；`.singleton(...)` 可选 Lower／Midpoint／Upper／Position。
n≥2 时端点保持固定。[0,10] 上的 count 4 请求 [0,10/3,20/3,10]，每个仿射位置独立舍入
一次。标签放不下时不会自行变稀。`TickSpace::Data` 使用原始单位，Scale 使用科学尺度
坐标。Log10 [0.1,10] 的三个 Scale 位置对应指数 [-1,0,1]，不同于 Data 中点 5.05。

`interval(20,5,Data,Fixed)` 在 [5,95] 上给出 [5,25,45,65,85]。Fixed 保留完整来源，
或者在真实的上限、适配、算术约束下失败；有效空列表保持为空。Preferred 可以尝试完整
的更稀来源及其约定的最终候选。每个 origin+k×step 使用精确整数算术和一次 binary64
舍入，包括端点舍入单元，而不是 epsilon 判断或 f64 下标。

乘法格点的锚和比率须为有限正数，且 ratio≠1。小于一的比率保留原方向，不转换成舍入
后的倒数。每个成员使用原始锚及精确有符号下标，不递推前一个舍入值。即使超出物理
f64 幂范围，Log10 操作数仍保留数学含义；回调收到带规范有符号十进制下标的
`Multiplicative` 操作数。全负线性视图或幂之间的空隙可得到空来源；同时包含零和正值的
视图具有趋零的无限尾部，因此拒绝。Preferred 尝试锚定的步幅子格点，最后取原格点
中间成员，不凭空制造离开格点的中点。

```rust
{{#include ../../../crates/lichtung-plot/examples/07_tick_generation.rs:example}}
```

## 实际视图回调与分类标签

TickContext 提供轴、最终域及来源、原始拥有者和实际 CSS 跨度。固有尺寸偏好不会调用
定位函数。返回 Data 或显式类型化的 Log10Exponents；只有真正保留了原始范围时才会
暴露它。普通视图处理按来源顺序保留窗口内无序及重复位置，并保留原下标。重合且相等
的标签可共享可绘制单元；不兼容的绘制文本属于布局不适配，而不是新增唯一性规则。
空输出保留图元、边框及标题。没有自动修复、回退定位器或隐式减量重试。

```rust
{{#include ../../../crates/lichtung-plot/examples/10_tick_functions.rs:example}}
```

`CategoryLabelSelection::every(stride)` 只选择标签，不删除类别、图元、刻度或网格。
offset 必须小于 stride；First／Last／Both／None 端点策略将完整域端点并入集合且不重复。
stride 为一即 All。x／y 独立，并使用显式、按值或分面局部的最终类别顺序。
未标标签的类别仍可能在注释或图例中需要文字。

```rust
{{#include ../../../crates/lichtung-plot/examples/09_category_labels.rs:example}}
```

## 数值策略

NumericFormat 支持 Decimal／Scientific／Engineering、有效数字 General，以及
Decimal／Binary／Octal／HexLower／HexUpper 的 Integer。小数精度为 0..=20，
有效数字为 1..=21，自适应上下界必须有序；十进制缩放范围为 -324..=324。
格式化保留 binary64 的精确值：小数精度 20 下，`0.1` 显示为
`0.10000000000000000555`。单独的百分号后缀只追加 `%`；十进制缩放或
`NumericRate::Percent` 才显式提供数值因子。

`multiplier` 在十进制缩放和舍入之前精确应用有限有符号因子，不经过中间 f64 乘积。
舍入可选默认 NearestEven、NearestAway、NearestTowardZero、TowardZero、AwayFromZero、
Floor、Ceiling。NegativeZero 与 TrailingZeros 独立；删除小数尾零不影响整数尾零。
Engineering 的指数为三的倍数，也包括 e0。

固定的 d3-format 3.1.0 locale 包含 59 个具名 ID，也支持完整自定义 token／分组定义，
不会推断宿主 locale。分组、会计符号、货币和比率后缀可显式组合；货币替换比率后缀，
不取消因子。`NumericField` 宽度按 Unicode 标量计数，而不是 CSS 像素。
非空字面前后缀不会被当作数值数字本地化。

specifier 语法为 `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`，
类型包括 f/e/g/r/s/%/p/d/b/o/x/X/n 或省略。示例有 `.3s`、`,.2f`、`.1%`、`#08X`
和 Unicode 填充。省略类型使用去尾零 General／12 有效数字；显式类型默认精度 6，
整数精度则为零。不合法组合会失败。

SI 支持 PerValue／WholeSet／Fixed 选择及舍入进位提升，并在 quecto／quetta 饱和。
十进制 compact 和自定义精确阈值／除数是独立选择。KeepThreshold 使用未舍入量级；
PromoteOnRounding 冻结一个与顺序无关的全集合选择，Fixed 要求 KeepThreshold。
公共因子可以是有限精确除数、任意大小的十次幂或 AutomaticDecimal。PerLabel／Shared
单位及唯一的经测量说明槽避免重复因子，必要说明不能悄悄消失。

`value_function` 单独替换标签；`set_function` 为完整有序语义集合及其公共说明返回
NumericLabels。惰性的内置助手保留自适应精度、locale 和因子，不递归调用回调。
返回的最终文字不再舍入或本地化。上下文区分固有尺寸探测与实际集合、原始身份、
拥有者和域。clone 共享回调身份；独立构造的回调即使返回相同文字，也不自动具有
Shared 兼容性。`builtin()` 只移除回调，不重置其他选项。

回调在每次终端渲染中按顺序串行执行，对每个实际到达的上下文只执行一次，不依赖
可选布局缓存。不同候选可产生不同集合。精确刻度和空拥有者抑制回调；异常及基数错误
向外传播。冻结 Output 的后续发布不会再次执行回调。

消费者包括生成数值轴、Heatmap／Beeswarm 色标、Heatmap 有限活动单元、Coefficient
可用的仅点估计标签，以及窗口内 Waterfall 合计。自适应精度对完整语义集合统一确定，
不是为每个标签适配尺寸而单独选择。生成轴遇到文本碰撞可尝试另一完整来源，固定
数值标签则可以重复。Forest Log10 格式化认证的数学幂，而不是舍入过的物理 f64 幂。
Shared 图例要求完整策略相等，并对合并域统一格式化；共享范围本身不强制刻度一致。

```rust
{{#include ../../../crates/lichtung-plot/examples/05_numeric_formats.rs:example}}
```

参见[数值策略案例](gallery-numeric-policies.md)及
[NumericFormat API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.NumericFormat.html)。
