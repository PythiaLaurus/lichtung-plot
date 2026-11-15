<a id="families"></a>
## 选择图族

[English](../en/index.html#families)

构造函数返回精确的 `Plot<kind::Family>`。封闭能力只暴露适用的配置方法；
不存在的 setter 不是一个可以在运行时忽略的选项。
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html)
给出签名和各图族专属配置。

每一行都链接到本页含实际图形及所属源码的讲解。多面板示例分别展示各图族的输入语法；
共用一个可执行程序，不会把多个面板变成一个科学场景。

| 构造函数 | 用途与输入区别 | 图形及源码讲解 |
| --- | --- | --- |
| `scatter` | 成对数值观测 | [酶反应](#first-plot) |
| `histogram` | 样本或调用者给定几何 | [校准残差](#calibration) |
| `density` | 平滑的样本分布 | [分布诊断](#distribution-diagnostics) |
| `qq` | 样本分位数与正态分位数比较 | [分布诊断](#distribution-diagnostics) |
| `point_interval` | 带位置的估计及区间 | [估计比较](#estimate-comparisons) |
| `line` | 普通数值线系列 | [时间响应](#time-response) |
| `trajectory` | 构造时持有分组身份的重复轨迹 | [有序过程](#ordered-processes) |
| `line_path` | 保持来源顺序的数值路径 | [有序过程](#ordered-processes) |
| `interaction` | 分类条件上的分组估计、边界与剖面 | [有序过程](#ordered-processes) |
| `coefficient` | 具名估计，可带状态、详情和数值标签 | [估计比较](#estimate-comparisons) |
| `forest` | 相对参考值的估计及边界，包括 Log10 展示 | [效应与区间](#effect-intervals) |
| `dumbbell` | 成对位置及连接线 | [估计比较](#estimate-comparisons) |
| `heatmap` | 分类单元坐标与构造时持有的连续颜色通道 | [响应热图](#expression-heatmap) |
| `bar` | 竖向样本或给定估计 | [估计比较](#estimate-comparisons) |
| `horizontal_bar` | 横向样本或给定估计 | [估计比较](#estimate-comparisons) |
| `stacked_bar` | 必须给定系列身份的竖向堆叠 | [组成剖面](#composition-profiles) |
| `stacked_horizontal_bar` | 必须给定系列身份的横向堆叠 | [组成剖面](#composition-profiles) |
| `boxplot` | 分组样本分布摘要 | [分布诊断](#distribution-diagnostics) |
| `violin` | 分组镜像样本密度 | [分布诊断](#distribution-diagnostics) |
| `raincloud` | 样本或完整给定的分布几何 | [不止于均值](#distributions) |
| `regression` | 数值关系及展示回归 | [校准](#calibration) |
| `marginal` | 带原生边际展示的数值关系 | [联合测量](#joint-measurements) |
| `matrix` | 具名数值变量的散点矩阵，不是任意单元矩阵 | [多变量实验](#multivariate-assay) |
| `beeswarm` | 分类行、连续颜色与点打包 | [组成剖面](#composition-profiles) |
| `waterfall` | 有序增量及显式起始总量 | [有序过程](#ordered-processes) |

构造函数要求的通道不能用可选 hue／group／color setter 替换。给定几何也不意味着
允许重新分析数据。应按科学表达语法选择图族，而不是仅凭图元外形选择。

`Plot::erase()` 显式产生用于异构存储的 `Plot<kind::Any>`，保留公共配置、预检和发布，
但不会重新获得图族专属 setter。应在插入 Canvas 前配置精确类型的图。
Matrix 自己拥有网格，不能进入 Canvas 或通用分面；Marginal 可进入 Canvas，但不支持
通用分面。其他图族仍有其依赖数据的限制，参见[组合](#composition)。

[Python 接口](#python)同样提供这 25 个图族的精确类，而不是拥有全部方法的通用对象。
[图库](#gallery)按科学问题组织图形，不按 setter 清单组织。
