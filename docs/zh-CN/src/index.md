# lichtung-plot

[English](../en/index.html#gallery) · [本页目录](#contents)

这些科学图形来自可运行的 Rust 和 Python 示例。点击图片即可直接跳到下方对应的代码讲解，
主要教程都在本页，无需反复换页。

<a id="gallery"></a>
## 图库与代码讲解

{{#include generated/gallery.md}}

十二个科学展示将全部 25 个图族连接到图形和源码；八个技术参考涵盖数值／文本策略、
Rust 适配器及 [Python](#python)。数据均为合成演示或调用者给定的示意值，不是研究结果。
可按科学表达语法从[图族目录](#families)找到对应图形。

<a id="contents"></a>
<nav class="page-toc" aria-label="本页目录">
<h2>本页目录</h2>
<ul>
<li><a href="#gallery">图库</a></li>
<li><a href="#first-plot">首张图讲解</a></li>
<li><a href="#distributions">分布形状</a></li>
<li><a href="#expression-heatmap">响应热图</a></li>
<li><a href="#effect-intervals">效应与区间</a></li>
<li><a href="#time-response">时间响应</a></li>
<li><a href="#calibration">校准与残差</a></li>
<li><a href="#distribution-diagnostics">分布诊断</a></li>
<li><a href="#estimate-comparisons">比较给定估计</a></li>
<li><a href="#ordered-processes">有序过程</a></li>
<li><a href="#composition-profiles">堆叠与连续颜色观测</a></li>
<li><a href="#joint-measurements">联合测量与边际分布</a></li>
<li><a href="#multivariate-assay">多变量实验矩阵</a></li>
<li><a href="#installation">安装</a></li>
<li><a href="#data">原生数据</a></li>
<li><a href="#ndarray">ndarray 视图</a></li>
<li><a href="#polars">Polars 数据列</a></li>
<li><a href="#families">图族选择</a></li>
<li><a href="#python">Python 接口</a></li>
<li><a href="#python-inputs">Python 四条输入路线</a></li>
<li><a href="#python-figures">Python 科学组合图</a></li>
<li><a href="#python-alpha">Python alpha 映射</a></li>
<li><a href="#python-results">Python 独立观测与给定结果</a></li>
<li><a href="#numeric-policies">数值案例讲解</a></li>
<li><a href="#axes">坐标轴与数值</a></li>
<li><a href="#themes">主题与图例</a></li>
<li><a href="#text-layout">文本案例讲解</a></li>
<li><a href="#composition">分面与 Canvas</a></li>
<li><a href="#output">输出与错误</a></li>
<li><a href="#about">关于本指南</a></li>
<li><a href="#build-docs">构建文档</a></li>
</ul>
</nav>

{{#include sections/installation.md}}

{{#include sections/scientific-gallery.md}}

{{#include sections/family-gallery.md}}

{{#include sections/data.md}}

{{#include sections/ndarray.md}}

{{#include sections/polars.md}}

{{#include sections/families.md}}

{{#include sections/python.md}}

{{#include sections/axes.md}}

{{#include sections/themes.md}}

{{#include sections/text.md}}

{{#include sections/composition.md}}

{{#include sections/output.md}}

{{#include sections/about.md}}

{{#include sections/building.md}}
