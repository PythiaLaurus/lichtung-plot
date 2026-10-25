<a id="about"></a>
## 关于本指南

[English](../en/index.html#about)

`lichtung-plot` 是面向自有、不可变展示数据的独立科学绘图器，负责展示统计、
验证、文本、布局及 SVG／PNG 发布。应用负责科学分析和推断决策。
绘图器不访问网络、不配置字体资源，不包含应用运行时或 PDF 后端。

从[安装与首张图](#installation)开始，然后了解[原生数据](#data)、
[图族选择](#families)、[数值呈现](#axes)、[外观](#themes)、[文本](#text)、
[组合](#composition)及[输出](#output)。[图库](#gallery)中的图片直接链接到本页的代码
讲解，不再经过独立案例页面。

中文与英文是地位相同、范围等价的两套指南，没有原版、翻译待办版或回退版之分。
两者共享主题锚点和示例引用；文字审阅关注语义一致，而不是逐句对应。

Book 解释使用方法；[Rust API 文档](https://docs.rs/lichtung-plot)提供精确公开签名，
workspace 中的[规范](https://github.com/PythiaLaurus/lichtung-plot/tree/main/specs)
定义架构及契约。代码摘录直接来自随包提供的可执行示例，不是另一套绘图实现；
请运行链接所指的完整示例，不要将上下文中的摘录当作独立 doctest。
