<a id="python"></a>
## Python：普通输入、精确图族与可复用输出

[English](../en/index.html#python)

已实现的 `lichtung_plot` 通过 snake_case 构造函数及 `Scatter`、`Line`、`Bar` 等精确类
提供[全部 25 个图族](#families)。方法按图族能力开放：Matrix 不会因此获得任意数值轴，
Bar 也不会获得 Line 的图层图例。在普通 CPython 3.10 或更新版本中使用兼容的本地 wheel，
详见[安装](#installation)；该包尚未发布到 PyPI。

这里解释用法，不另列一份方法矩阵。精确重载及不可变值字段请查已安装的
`lichtung_plot/__init__.py`、`lichtung_plot/_owners.py` 和 `lichtung_plot/_native.pyi`，
或仓库中 `bindings/python/python/lichtung_plot/` 下的对应文件。
持续维护的 `bindings/python/README.md` 解释输入与生命周期。
共享契约仍以 `specs/92_PYTHON_CONSTRUCTORS_PLAN.md`、
`specs/94_PYTHON_SCIENTIFIC_INTERFACE_PLAN.md` 为准；工作区计划另记有界验证与剩余验收工作。

### 选择变量，不另造数据语言

四条输入路线是原生 Python list／tuple、NumPy 数组（含 MaskedArray）、
pandas DataFrame／Series 和 eager Polars DataFrame／Series。提供库为可选依赖，
Polars 路线不要求 PyArrow。在构造函数及独立来源加变量的调用中，`data` 是唯一位置参数，
科学字段使用关键字；直接变量与来源选择器可以混用。

`col[key]` 选择一个变量；`cols[...]` 选择非空有序集合，即使集合只有一列也仍是集合。
裸 list／tuple 一律表示数据，不表示选择器集合。NumPy 来源的整数表示位置；
pandas 使用完整、精确的原始标签，包括合法的非字符串键，整数键不会被重新解释为位置。
带符号步长切片按逻辑列位置选择，越界边界报错而不是裁剪。
展示标签和别名不会替换查询身份或科学成员关系。

支持的原始数值类型保留 null 槽位；bool 不是数值观测。NumPy MaskedArray 的 true mask
表示缺失。pandas object 列只允许字符串／null 分类，不是任意对象转换通道。
不支持的时间、decimal 或嵌套值须先在上游显式投影。
表中未绑定的字段不会仅因存在于来源中就被导入。

每次绑定都会快照其所绑定的槽位。保留的主 `data` 可供后续 `.hue("condition")` 等显式编辑
重新查询，但不会刷新旧坐标。直接 Series 不会隐式安装父表；
独立 `data=None` 只允许直接输入，绝不回退到主来源。

### 拥有者、配置值与渲染操作

Plot 和 Canvas 的配置方法修改并返回同一个拥有者。`.copy()` 得到独立配置，
不是重新读取观测。Canvas 插入时捕获 Plot 当时的原生配置及回调根引用，
不保留外部 Python Plot 对象；随后修改该 Plot 不影响已插入面板。

Theme、Text、数值策略等配置值不可变。用关键字形式的 `.copy_with(...)` 派生新值，
再替换完整父值，不修改嵌套句柄。省略保留原字段；只有接口允许清除时 `None` 才表示清除，
它不是通用重置。接收文本的拥有者方法中，省略 `style`／`layout` 保留指令；
显式 `None` 清除对应局部指令，使拥有者／Theme 默认值重新生效。

Plot／Canvas 的每次终端调用都启动新的渲染操作，成功或失败均不消耗配置。
`Output` 则共享一个不可变的冻结场景，再次发布不会重复回调、统计或布局。
下方示例先发布轮廓 SVG，再从同一 Output 发布保留文字 SVG 和 PNG 预览。
[字体、错误及发布规则](#output)不变；绑定和图库都不下载字体。

外部查询和 Python 回调在拥有者借用之外执行。待返回的原始 Python 异常不会被后来的
原生错误或 stale 候选错误替换；准备成功也不能覆盖重入 Python 已做的编辑。
这些生命周期规则不构成对自由线程解释器的支持承诺。

### 数值策略与八个回调槽位

使用完整的 `NumericFormat`、`NumericPrecision`、单位／区域／因子策略和
`TickGeneration` 定位器，而不是把坐标先转成格式化字符串。
`NumericFormat.from_specifier(...)` 编译的是展示策略，不是数据变换。
Theme 和 Text 遵循[主题](#themes)与[文本](#text)章节所述的原生外观及指令所有权。

八个独立持久回调附件分别是 x／y 格式、连续颜色标签、Heatmap 数值、
Coefficient 估计标签、Waterfall 合计及 x／y 刻度生成。
回调上下文及嵌套视图仅在本次回调内有效。复制保留未编辑的回调身份；
新终端操作可能再次调用回调，冻结 Output 的发布不会。

### Alpha、独立观测与给定结果

Alpha 是 `0..255` 的精确字节，不是浮点不透明度。原生 list／tuple 成员及字面值拒绝
bool 和 float；外部字节变量要求 u8／UInt8。普通长度为一的变量不广播，
需要广播时选择显式 `_single(...)` 形式；重置使用 `_theme()`，不是 `None`。
数值映射在原生 `linear`、`log10`、`threshold`、`continuous` 工厂中选一个；
分类映射使用标量 `mapper`。这些工厂保留原生图例声明及代表值，不只是最终字节。
语义键使用原始身份，不使用展示别名。

Alpha mapper 在绑定时、外部借用结束后执行，不作为第九个持久回调存储。
Alpha 不过滤观测、不改数值，也不改变科学成员关系，见[覆盖度示例](#python-alpha)。

Line 的 `point_layer(data=None, *, x, y, ...)` 独立绑定观测点，可附标签、Color、
半径、字节 alpha 及整层 legend Text 指令。可选 `None` 表示未设置，
但显式 legend style／layout `None` 会清除相应指令；无 legend label 却给指令是不完整形式。
Bar／HorizontalBar 则要求 category／value、完整 Color 和 u32 半径，可选定位系列；
没有点标签、legend、layout 或独立 alpha 参数。图层的记录数和来源不继承主图。

Attached 结果适用于 Bar、HorizontalBar、Coefficient；pairwise 适用于 Bar、
HorizontalBar、StackedBar、StackedHorizontalBar、Boxplot、Violin、Raincloud。
每种 placement 都有六种形式：普通与 `_from_data`、`_in_facet` 与
`_in_facet_from_data`、`_in_grid` 与 `_in_grid_from_data`。
目标可以是普通分类变量、可选 category／series 对，或显式
`target_value`／`left_value`／`right_value` 字面值。只有显式字面分支设置 scalar 标记；
普通长度为一的 target 仍是向量。`facet_value` 同样表示字面层级或网格二元组。

Marks 有三种形式：数值 `marks` 且省略 mapping；分类 `marks` 加显式 `mapping`；
或字面 `mark` 且另外两项均省略。字面 mark 是 p 值、`ns` 或一到四个星号。
`mapping=None` 无效；有序 mapping 中的重复 pair 会保留，交原生逻辑检查实际消费的键。
目标／分面值是原始层级，不是语义字段查询键或展示别名。结果具有独立记录数；
原生 producer 保留绑定、广播及预检验证的阶段。检验和调整由调用者完成，渲染器不计算。

### 完整 Python 参考程序

这些示例通过四条受支持的输入路线使用同一个原生绘图器，不是另一套绘图后端。
按[构建说明](#build-docs)选好已安装
解释器后，例如从工作区根目录运行
`"$LICHTUNG_PYTHON" -I -B bindings/python/examples/scientific_composition.py`。
无参数时写入当前工作目录，一个可选目录参数可重定向输出。下方完整源码包含输入构造、
组合及三次发布。

<a id="python-inputs"></a>
### 四条输入路线中的同一实验

[English](../en/index.html#python-inputs)

![通过原生 Python、NumPy、pandas 和 eager Polars 展示相同的合成实验](gallery/python-inputs/four_inputs.svg)

四个面板绑定相同的时间、响应和队列值。NumPy 路线按位置选择列，pandas 和 Polars
按名称选择字段。未绑定的 pandas Decimal 列与 Polars Struct 列说明：绑定选定变量
不会转换整张表。这是一次输入传递方式的比较，不是图族与提供库的交叉矩阵。
每个面板都标出相同单位。

```python
{{#include ../../../../bindings/python/examples/four_inputs.py:example}}
```

[完整源码](examples/lichtung-plot-python/four_inputs.py) ·
[原尺寸 SVG](gallery/python-inputs/four_inputs.svg) ·
[可选文字 SVG](gallery/python-inputs/four_inputs_text.svg) ·
[PNG 预览](gallery/python-inputs/four_inputs.png) · [Python 接口](#python)

<a id="python-figures"></a>
### 科学组合图：四种不同输入契约

[English](../en/index.html#python-figures)

![Python Line、Bar 区间、不可用 Forest 估计及带注释 Heatmap](gallery/python-figures/scientific-composition.svg)

多列 Line 保留真实时间坐标及独立系列。Bar 使用给定示意边界，不从样本计算置信区间；
Forest 通过显式状态标记不可用估计，不从 null 推断；Heatmap 高亮是独立布尔数据，
不是单元过滤器。四个面板均使用合成值，并标明各自变量。

```python
{{#include ../../../../bindings/python/examples/scientific_composition.py:example}}
```

[完整源码](examples/lichtung-plot-python/scientific_composition.py) ·
[原尺寸 SVG](gallery/python-figures/scientific-composition.svg) ·
[可选文字 SVG](gallery/python-figures/scientific-composition-text.svg) ·
[PNG 预览](gallery/python-figures/scientific-composition-preview.png) · [Python 接口](#python)

<a id="python-alpha"></a>
### Alpha 映射：给定覆盖度不等于置信度

[English](../en/index.html#python-alpha)

![Python 分类观测 alpha 与数值 Heatmap 单元 alpha](gallery/python-alpha/alpha-mappings.svg)

Scatter mapper 将给定覆盖类别转成字节 alpha；Heatmap 用原生线性映射处理给定覆盖比例。
信号数值和颜色仍与覆盖度分开。两种表达都不声称统计置信度，不删除观测，也不改变数值。

```python
{{#include ../../../../bindings/python/examples/alpha_mappings.py:example}}
```

[完整源码](examples/lichtung-plot-python/alpha_mappings.py) ·
[原尺寸 SVG](gallery/python-alpha/alpha-mappings.svg) ·
[可选文字 SVG](gallery/python-alpha/alpha-mappings-text.svg) ·
[PNG 预览](gallery/python-alpha/alpha-mappings-preview.png) · [Python 接口](#python)

<a id="python-results"></a>
### 独立观测与给定展示比较

[English](../en/index.html#python-results)

![Python 给定趋势与带，以及 Bar 区间、观测和比较括号](gallery/python-results/supplied-results.svg)

四个给定趋势位置及其带，与六个独立观测和一个整层图例身份共存。
Bar 有三个给定区间、六个独立点和两项给定比较。字面 `left_value="Control"`
显式广播到两个右侧结果，不是猜测长度为一的变量。边界和展示 p 值均为示意：
渲染器没有拟合趋势、计算检验或调整 p 值。

```python
{{#include ../../../../bindings/python/examples/supplied_results.py:example}}
```

[完整源码](examples/lichtung-plot-python/supplied_results.py) ·
[原尺寸 SVG](gallery/python-results/supplied-results.svg) ·
[可选文字 SVG](gallery/python-results/supplied-results-text.svg) ·
[PNG 预览](gallery/python-results/supplied-results-preview.png) ·
[Python 接口](#python) · [返回图库](#gallery)
