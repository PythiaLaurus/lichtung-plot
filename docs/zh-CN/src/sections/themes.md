<a id="themes"></a>
## 主题、字体与色标

[English](../en/index.html#themes)

Theme 是包含八个组件的完整不可变值：SurfaceTheme、TypographyTheme、AxisTheme、
LegendTheme、CompositionTheme、MarkTheme、AnnotationTheme、ColorScales。
`publication`、`nature`、`science`、`chi` 都是普通完整构造函数，不是全局状态。
独立图省略主题时使用 Publication；Canvas 子图省略时继承根主题，显式 Plot Theme
则完整替换继承值。分面不引入新的外观作用域。

`Theme::new` 接收全部组件；`customize` 创建工作副本，ThemeBuilder::new 接收完整基值。
组件的 `property(value)` 消费式编辑会验证完整值，配对读取器使用 `get_property()`。
Theme 的只读组件保留 `typography()` 这样的名称。`build()` 验证但不发现字体。

```rust
{{#include ../../../../crates/lichtung-plot/examples/02_themes.rs:example}}
```

十八个文本角色都独立持有字体栈、RGBA 颜色、正的可含小数 CSS 字号、字重和行高。
FontFamily::named 表示单个字面字体身份，不是 CSS 字体栈表达式；GenericFontFamily
显式选择 Serif／SansSerif／Monospace。FontStack 保留非空有序条目，包括重复项。
可用性在渲染实际消费角色时检查，不在主题构造时检查。
ThemeBuilder::font_stack 一次编辑全部角色，之后的局部替换按通常优先级生效。

RelativeSpacing::new 保留精确二进制有理数，`ratio(u32,u32)` 保留约分后的数学比值。
间距相对自己的文本角色解析，不依赖根尺寸或近似 `as_f64` 查看值。
Shared 图例外框使用 Canvas，来源图元外观仍由 Plot 拥有。

### 颜色映射与精确画笔

数据映射使用不透明的 OpaqueColor；精确／角色画笔使用带独立 alpha 的 RGBA Color。
固定 QualitativePalette 必须覆盖原始域；可扩展色板保留前缀并确定性扩展，不循环复用。
CategoricalColorMap 使用唯一、已去首尾空白且无控制符的原始键，而不是 alias。
未使用条目和重复颜色合法；缺少实际使用的键即使在 Hidden 图例下也失败。

CategoricalColorScale 选择完整色板或映射。Heatmap 与 Beeswarm 共用
ContinuousColorScale::sequential/diverging，可带匹配的梯度及精确 ColorDomain。
省略梯度使用 Theme，省略域使用准备后的数据。常数域合法；发散域必须包含中心。
色标节点严格有序且端点精确为 0／1，发散色标还需 0.5 中性节点。
内置顺序梯度使用 `#F7FBFF → #08306B`。

精确元素样式只覆盖自己拥有的字段。PointLayer 的颜色、半径和 alpha 独立；单独设置
的 alpha 在任意 setter 顺序下都替换完整颜色的 alpha 字节。补充图层、轮廓和文字不
继承科学数据的 alpha 指令。

### 数据 alpha 与解释

`point_alpha`、`line_alpha`、`fill_alpha` 在支持的精确图族上附着完整 AlphaEncoding。
观测按完整原始槽位对齐，包括不可用或过滤的行；SourceRows ID 不是下标。给定 Raincloud
点使用原始组／点顺序。`single` 显式广播一个值。集合式系列／聚合必须提供完整且唯一
的原始键元组；Histogram 分箱使用精确的有限 lower<upper 区间，可附带所属分面键。
不会推断 mean／first／last 缩减或逐行相等规则。

AlphaValues::new 拥有任意字节但不提供解释。Linear／Log10 显式声明有限递增域；
常数数据以及递增、递减、常数字节范围都合法。域外有限值截断到边界，非法、null 或
非有限输入按来源下标报错；Log10 还要求正值及可区分的变换端点。只进行一次
nearest/ties-to-even 量化：0..255 上的四分位点得到 [0,64,128,191,255]。
连续函数按数值顺序对不同必需来源／示例各计算一次；分类函数按首次观察顺序计算。
回调不保留。Threshold 等于边界时归右区间，select 保留完整合并声明以供 Shared 兼容。

零 alpha 不删除科学成员、几何或统计。数据 alpha 替换而非乘以已解析字节。
重复 setter 替换完整指令，`AlphaEncoding::theme()` 重置指令。声明的 Point／Line／Fill
解释段位于已有图例内容之后。连续 alpha 使用至多五个显式原始域示例，不推断逆向色带。
上下文键将数据 alpha 中和为 255；未声明字节不会凭空产生解释。Shared 兼容要求完整
配方一致，而不只是字节一致；Hidden 不跳过绑定验证。

```rust
{{#include ../../../../crates/lichtung-plot/examples/08_data_alpha.rs:example}}
```

LegendComposition::Combined 保留完整的实际观察键；Separate 按组样式、联合、hue
颜色、字面及补充系列顺序解释，再接连续颜色和声明 alpha，始终只有一个外框及放置
拥有者。密集的五组／四色 Interaction 可以展示 20 个组合键或 5+4 个边际键；稀疏输入
不保证 Separate 更小。两者都不会创造未观察到的组合。

独立的 Interaction 组样式／hue 可以拆段；同字段保持联合，不支持的独立组合在字体前
失败。Shared 使用 Canvas 的组合策略和外框，以及来源 Plot 的图元几何；PerPlot 保留
各子图选择。重复身份必须有相同的标签、样式和完整键；显式子图图例位置与 Shared
仍不兼容。Hidden 只省略解释语义，不跳过科学色标。Heatmap／Beeswarm 和 Matrix
保留各自唯一的原生图例轨道。

```rust
{{#include ../../../../crates/lichtung-plot/examples/06_legend_sections.rs:example}}
```

参见[组合](#composition)、[文本](#text)及
[外观属性归属规范](https://github.com/PythiaLaurus/lichtung-plot/blob/main/specs/04_VISUAL_PROPERTY_OWNERSHIP.md)。
