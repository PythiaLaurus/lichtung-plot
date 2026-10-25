<a id="text"></a>
## 文本与注释

[English](../en/index.html#text)

<a id="text-layout"></a>
### 代码讲解：固定的数据参照与可适配标题

![数据锚定注释旁是允许局部字号适配的 TextPanel 标题](gallery/text-layout/text_layout_outlined.svg)

左侧面板的注释引用数据坐标 (2,3)，不要求那里恰好有一个观测样本。参照与块锚点
相互独立：数据位置保持固定，`bottom_left` 和旋转决定文字如何围绕它绘制，
不移动科学点：

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-reference}}
```

右侧面板承担不同职责。标题克隆 Theme 中 TextPanel 标题角色的样式并采用 14.25 CSS
像素，然后明确允许在给定的两行边界内局部缩小到 10.25，必要时进行末尾省略。
这一许可不扩散到其他文字或科学内容：

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-fitting}}
```

Canvas 将 Plot 与 TextPanel 组合起来。保留文字的发布与图库中的轮廓图来自同一个冻结
场景，不是两次独立布局：

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-publication}}
```

运行 `cargo run --locked --offline -p lichtung-plot --example 12_text_layout`。

[完整源码](examples/lichtung-plot/12_text_layout.rs) · [共享输出辅助代码](examples/lichtung-plot/support/output.rs) ·
[原尺寸 SVG](gallery/text-layout/text_layout_outlined.svg) ·
[可选择文字 SVG](gallery/text-layout/text_layout.svg) ·
[TextLayout API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.TextLayout.html) · [返回图库](#gallery)

<a id="text-options"></a>
### 文本策略与归属

`Text` 保留原始来源，以及可选的完整 TextTheme／TextLayout 指令。用普通字符串替换
内容会保留先前指令；显式给出样式或布局则替换那一整项指令。显式默认布局恢复拥有者
默认值。标题、副标题、说明、轴标题、色标标题、Canvas 表头、TextPanel 标题／正文及
面板注释都使用这一模型。

手写空白和换行保持原样，包括空行；CRLF 视为一次换行。显式空的说明文字占一个空文本行。
制表符和不支持的控制符会失败。精确 AxisTick 的空标签则抑制文字但保留刻度。
自动名称通过 `Labels` 的易读化、原样保留或别名映射策略展示，不改变原始身份。

Plot 标题和副标题的对齐彼此独立，只有子图未显式指定时才采用 Canvas 面板默认值；
图注对齐也独立。标题指令中的 `Start`／`Center`／`End` 是扣除内容边距后的物理方向。
Text 的行对齐程序还区分逻辑 Start／End 与物理 Left／Right。面板指令不改变 Canvas
根文字、表头或 TextPanel 内容。

TextPanel 标题和正文可各自选择完整样式／布局；省略时使用 Canvas 的独立角色。
局部字号决定自己的内边距及间隔。轴标题样式也拥有其相对留白，包括共享分面标题。
Shared 色标标题要求完整 Text 请求相等，包括显式重置。

### 参照、锚点与局部适配

PanelTextAnnotation::new 将注释放到角落；`at` 接收类型化的 Data／Panel／Figure 参照。
TextLayout 的块锚点是另一个独立选择。数据参照由实际面板投影，不从屏幕坐标猜测。
`position()` 读取类型化参照，TextOffset 选择 Screen 或 TextLocal 的 CSS 偏移。
表头块保持在声明跨度中央，局部对齐只控制块内各行。

TextLayout 控制边界、换行、可见性、含小数的字号适配、旋转及溢出。固定角度或水平
替换与显式有序的自动刻度角度列表不同。AlignAfterRotation 对齐旋转后盒子；默认围绕
所选锚点旋转。类型化数据方向使用真实投影，而不是未缩放的数值斜率。

溢出可以拒绝、隐藏、裁剪、继续可见，或用明确标记及失败策略在 Start／Middle／End
省略。完整文本放得下时不需要标记。原始来源仍保存在不绘制的 SVG 无障碍元数据中。
Heatmap 密集标签的默认三点省略也使用同一引擎；显式默认刻度 TextLayout 恢复完整
换行，不会悄悄删除类别。

`TextFit::shrink_to` 允许局部含小数的最小字号，range 还允许有限放大。ContentFirst
优先完整内容；PreferredSizeFirst 可以在首选字号先尝试允许的省略，再缩小。
Uniform 分组只作用于实际符合条件的角色组，不联动无关轴或独立标题／正文。
可见性 every／greedy 只选择文字，不删除图元、刻度或网格。

HeatmapNullRowAnnotation 持有原始行键及 Text 解释。WaterfallLabels 独立持有起始／
终止名称，样式作用于完整的名称加数字注释。PointLayer::legend_label 的 Text／样式
独立于点注释；重复图层身份须在完整标签请求和图元键上保持一致。
这些设置都不改变科学成员关系。

[文本布局讲解](#text-layout)将数据参照与局部标题适配保持独立。
角色归属见[主题](#themes)，轮廓文字与可选择文字的差异见[输出](#output)。
