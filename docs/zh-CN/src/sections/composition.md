<a id="composition"></a>
## 分面与 Canvas

[English](../en/index.html#composition)

分面拆分同一个科学图；Canvas 则拥有分别撰写的 Plot／TextPanel。
FacetSpec::row／column／grid／wrap 附着完整布局，wrap 要求正列数。分面保留 Plot 的
主题，不新增作用域。通用分面图不能进入 Canvas；Matrix 有原生变量网格，Marginal
有自己的非通用边际布局。

Canvas::grid 声明按行优先的拓扑。`panel` 消费精确 Plot，在异构存储边界擦除标记，
但不弱化验证。`panel_span(row,column,row_span,column_span,plot)` 拥有一个正矩形，
锚位于左上角。覆盖的槽位只是被占用，不是额外生产者。Plot／TextPanel 无论插入顺序
如何，严格插入遇到重叠都会失败；范围组引用锚位置。

`set_panel(row,column,plot)` 和 `set_panel_span(...)` 明确覆盖区域。所有相交的 Plot／
TextPanel 都被整体移除，即使只重叠其非锚单元；旧占用的全部单元都释放，包括新矩形
之外的部分。例如布局 `A A B / A A C` 中只替换 `(1,1)`，结果为 `. . B / . X C`。
仅接触边缘而没有共享单元不算重叠。先验证跨度、边界并准备替换资源，再移除原有拥有者。
网格、表头和范围组请求不被静默改写，新的拓扑仍接受正常预检；再次导出会渲染完整组合。

```rust
{{#include ../../../../crates/lichtung-plot/examples/11_canvas_spans.rs:example}}
```

跨格 Plot 有一个连续绘图区及一套坐标轴、文字和图例。行标题不能切过其占用范围。
共享分类行带要求相同垂直占用范围及完整类别／标签选择策略；FirstColumnPerRow 不会
猜测不等高行带的对应关系。父级几何覆盖子图独立尺寸请求，声明轨道仍独立定位表头。

Canvas 根默认使用 Publication。未指定主题的子图继承它，显式子主题完整替换它。
根文字、表头、TextPanel 和 Shared 图例外框使用 Canvas 外观，科学子图图元及坐标轴
使用 Plot 外观。

```rust
{{#include ../../../../crates/lichtung-plot/examples/04_canvas_themes.rs:example}}
```

共享范围和共享图例是不同关系。相同范围不意味着刻度生成或格式化选择相同。
Shared 图例保留原始身份／画笔兼容要求，使用 Canvas 的组合策略；alias 或样式冲突
会报错，不作启发式合并，参见[主题](#themes)。

应在插入前配置精确类型的子图。Canvas 不是 Clone，Rust 消费式方法表示所有权转移，
不代表 Python 原地修改语义。
