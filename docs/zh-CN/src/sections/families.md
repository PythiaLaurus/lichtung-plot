<a id="families"></a>
## 选择图族

[English](../en/index.html#families)

构造函数返回精确的 `Plot<kind::Family>`。封闭能力只暴露适用的配置方法；
不存在的 setter 不是一个可以在运行时忽略的选项。
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html)
给出签名和各图族专属配置。

| 构造函数 | 用途与输入区别 |
| --- | --- |
| `scatter` | 成对数值观测 |
| `histogram` | 基于样本或调用者给定几何的分布 |
| `density` | 平滑的样本分布 |
| `qq` | 样本分位数与正态分位数比较 |
| `point_interval` | 带位置的估计及区间 |
| `line` | 普通数值线系列 |
| `trajectory` | 构造时持有分组身份的重复轨迹 |
| `line_path` | 保持来源顺序的数值路径，不采用普通线的排序语义 |
| `interaction` | 分类条件上的分组估计、边界与剖面 |
| `coefficient` | 具名模型估计，可带状态、详情和数值标签 |
| `forest` | 相对参考值的估计及边界，包括 Log10 展示 |
| `dumbbell` | 成对位置及连接线 |
| `heatmap` | 分类单元坐标与构造时持有的连续颜色通道 |
| `bar` | 竖向样本或给定估计 |
| `horizontal_bar` | 横向样本或给定估计 |
| `stacked_bar` | 必须给定系列身份的竖向堆叠 |
| `stacked_horizontal_bar` | 必须给定系列身份的横向堆叠 |
| `boxplot` | 分组样本分布摘要 |
| `violin` | 分组镜像样本密度 |
| `raincloud` | 样本或完整给定的分布几何 |
| `regression` | 数值关系及展示回归 |
| `marginal` | 带原生边际展示的数值关系 |
| `matrix` | 具名数值变量的散点矩阵，不是任意单元矩阵 |
| `beeswarm` | 分类行、连续颜色与点打包 |
| `waterfall` | 有序增量及显式起始总量 |

构造函数要求的通道不能用可选 hue／group／color setter 替换。给定几何也不意味着
允许重新分析数据。应按科学表达语法选择图族，而不是仅凭图元外形选择。

`Plot::erase()` 显式产生用于异构存储的 `Plot<kind::Any>`，保留公共配置、预检和发布，
但不会重新获得图族专属 setter。应在插入 Canvas 前配置精确类型的图。
Matrix 自己拥有网格，不能进入 Canvas 或通用分面；Marginal 可进入 Canvas，但不支持
通用分面。其他图族仍有其依赖数据的限制，参见[组合](#composition)。

本目录包含全部 25 个图族。初始[图库](#gallery)只选择若干跨能力工作流，
并不声称已经为每个图族提供图像案例。
