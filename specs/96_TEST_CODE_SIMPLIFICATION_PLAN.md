# Test Code Simplification

**状态：用户已批准，实施中。**

用户要求先完成计费清理尾项和性能结果，再检查测试层级、无意义确认及 Rust
代码规模，按计费清理的方式规划测试清理。前两项已完成，证据分别在
`../docs/resource-accounting-cleanup.md` 与 `../docs/caption-cleanup-performance.md`。
用户随后明确要求开始清理，批准按本文删除／合并并逐段验证提交。后续主题／
公开功能继续暂停。已完成的易变 SVG／快照清理不在这里重复实施。

## 一、结论与基线

审计基线为 `f417320`。当前 245 个跟踪 Rust 文件的 cloc code 为 **156,936 行**：
手写生产实现 67,852；独立 Unicode 生成表 13,017；测试相关 75,966；示例 101。
测试相关包括 `src/` 内测试／辅助／观察 59,627 行、普通集成目标 13,174 行、
共享辅助 473 行、现有基准支持 2,692 行，约占全体 **48.4%**。

完整分类和具体低价值例子在 `../docs/test-code-audit.md`，逐文件数在
`../docs/test-code-census.csv`。不能只数 `tests/`，也不能把 Unicode 表算成
可删测试。生产实现本身仍较大；本方案以测试为范围，不以删测试冒充生产
架构精简，更不以总行数先判定所有算法或测试都无价值。

**并非缺少端到端覆盖。** Cargo 的编译入口是库测试／集成目标，实际内容还
包括跨模块组件集成、公开库内端到端、真实 SVG／PNG／文件发布与部分 PNG
解码，以及文档编译案例。Rust 库的端到端路径不要求另有一个 CLI 或 `e2e/`
目录。真正要解决的是：保留少量清晰、有独立预期的端到端骨架，同时删掉
无独立验收意义的内部重复；不是再建一套昂贵测试体系。

## 二、测试类别的处置

| 类别 | 处置 | 依据与保留边界 |
| --- | --- | --- |
| 相同函数／相同参数计算 expected 与 actual | **删除** | 没有独立预期；例如 Histogram 的重复 `prepare_histogram` 循环 |
| 旧额度成功／失败目的消失后的重复调用 | **删除重复部分，混合测试留语义** | 计费已删除；同输入、同主题、无结果或副作用断言的再次成功不能证明新契约 |
| 直接存储／getter 回声、手动复制 setter 实现的 expected | **删减或合并** | 保留合法／非法构造类、真正字段选择、兄弟不变和实际消费者效果；不是每字段都要多层重证 |
| 私有容器形状、指针地址、逐步调用次数／完整轨迹 | **改为语义关系或诊断** | 保留操作／源归属、真实缓存失效、不得重做科学工作／字体准备的关键生命周期；不固定一次实现的索引或所有内部计数 |
| 同一科学／验证规则跨层反复执行大矩阵 | **按契约合并** | 核心拥有者给独立数学／错误预期；公开调用路径给少量路由／错误传播检查；不同缺失／finite／区间政策仍分开 |
| 每生产者复制 preparation／poison／capture／replay 装置 | **共享核心证明，缩短家族差异** | 保留不同图元、来源、样式、裁剪与字体输入的唯一失效面；不可只留一个通用 smoke |
| 根／Facet／Canvas／Shared 的所有尺寸×预设×family 组合 | **压缩为真实交互场景** | 保留所有公开 family 的合法派发覆盖；额外交叉只覆盖确有独立语义的稀疏、共享域、所有权、文字或特殊图形路径 |
| 基准观察与调试数据 | **留必要测量，移除稳定值作为正确性 gate** | 耗时／RSS／shape 次数可报告；来源、同操作一致性、生命周期和真实规则界限可断言；旧 figure 的精确 count tuple 不可变成永久合同 |
| 独立数值／Unicode／统计／溢出 oracle | **保留，优先避开削减** | MPFR、标准数据、正确舍入、OLS／KDE／bootstrap、源完整性与真实长度／容量错误不属于易变确认 |
| 公开 API 与实际发布端到端 | **保留并明确承担的契约** | 实际正常库编译、源→统计／字体／布局→SVG／PNG→文件／解码；不能全换成内部 mock |
| 类型／可见性／capability 的编译案例 | **去重并补齐执行归属** | 无 runtime assert 不等于无价值；`--all-targets` 不执行 doctest，`just docs` 也不是 doctest 验证 |

## 三、删除／合并判据

逐项判断的是**失效面**，不是名字、长度或断言数。需要能指出：输入中的哪个
科学／API／输出契约，被什么独立预期观察到，以及哪里已经承担相同检查。

1. expected 来自同一函数、同一输入时，删自我对照；确定性必须是明确需要的
   契约，并同时有内容／结构事实，不能让两次相同错误互相认证。
2. 同操作的重复调用若检查 cache hit、payload 不变或没有重做受禁止的工作，
   有意义；如果只 `.unwrap()` 且结果丢弃，通常删除。克隆／新操作／异源的
   **身份校验**仍独立保留，不因 `OperationContext` 出现就把调用全删。
3. 数值坐标只有在由明确调用者输入和独立数学关系推出时才作为预期。
   自动面板大小、内部排列和旧整图字节不重新固定，亦不用宽 tolerance 包装。
4. 指针相同不是通用公开要求。准备源／操作归属需要身份见证；普通主题 clone
   的每个 Arc／地址计数不能替代实际不变性和正确 owner 观察。
5. 公共数据错误与发布失败要保留类别、来源及 no-artifact／existing-artifact
   契约；删只是内部失败地点、全文 Debug 格式或分配层次的精确追踪。
6. 避免测试里复制生产 binary search／layout／formatting 算法再验证那个副本。
   合理独立 oracle 必须来自数学、标准、明确原始输入或不同真实执行边界。
7. 不能按“零直接断言”自动删：断言 helper、编译类型约束和应不触发的 callback
   本身可构成有效预期。长测试也可能覆盖一个真实不可分的生命周期。

## 四、首批源已确认的目标

| 对象 | 当前问题 | 首批动作／保留的证据 |
| --- | --- | --- |
| `plots/histogram.rs::histogram_domains_and_kde_maxima_match_mark_preparation` 前半段 | expected／actual 是完全相同调用的三重参数循环 | 删除该循环；curve／maximum 分支单独按关系审阅，不整段机械删除 |
| `plots/regression.rs::regression_axis_channel_and_fit_preserve_source_results` | 重复 axis_ranges，另分配两份不用的 x/y buffer | 删除；保留统计 oracle、source rows、实际 fit/band/window 证据 |
| `layout/interior/coordinate/tests.rs::every_native_route_admits_intrinsics_and_selected_candidates_at_exact_resource_caps` | 不存在 caps，结果未验，重复调用同一 consume | 删除旧 smoke 矩阵；实际 intrinsic／payload／根反馈证据留存 |
| Scatter、Line、Heatmap、Matrix、Marginal、Waterfall、Bar、Estimate 的 semantic preparation fixtures | 新建“exact”操作后的第二次成功结果丢弃 | 仅删无意义重复；保留原始科学值／顺序／Source ID／attach ownership |
| `layout/interior/tests.rs` 的字面枚举不等断言 | 检查直接写出的两常量 | 删除；错误 family／route attach 拒绝继续保留 |
| `theme/resolved.rs::indexed_text_queries_preserve_matching_and_collision_order` 的 copied search | 测试只给自己重写的 binary search 计数 | 删该副本和 comparisons 阈值；留真正 hit／碰撞／provenance 失效 |
| 旧 `admission/resource_caps/legacy` 名称与计费说明 | 名称不能表达当前实际语义，部分已完全失去目的 | 与相应删除／合并同步清理；不为旧名称恢复机制 |

这些是明确首批，不是对所有 1,800 余个测试逐函数裁定完成的宣称。后续混合
文件的保留／合并要在所属阶段先核对当前调用与唯一失效面；不能用审计 CSV
自动删除整文件。

## 五、重点合并面与应保留的差异

### 生产者、capture 与真实组合

`src/plots/` 的测试相关 14,735 行，`src/layout/` 12,258 行，orchestrator 9,899
行，是主要审查面。优先比对这些重复生命周期 fixtures：

- Scatter、Line、Bar、Estimate、Dumbbell、categorical distributions 的
  `selected/poisoned_capture...no_source/projection/axis/shaping`。
- `layout/interior/{bars,estimates,distributions,categorical_distributions}` 的
  同族 preparation、selected payload、facets／Canvas／cache-disabled 流程。
- `orchestrator/tests/lifecycle.rs::every_canonical_plot_kind_renders_through_one_lifecycle`
  与 `canvas/lifecycle_tests.rs::mixed_family_parent_audit`、各家族 parent checks。
- `canvas.rs`、`orchestrator/facets.rs` 与 `decorations/plan/children.rs` 的 sibling
  复用／反馈失效。通用依赖 key 与实际父子路由是不同失效面，不可只留一边。

保留所有 25 个 family 的一次有效派发覆盖，不为每一 family 再乘四预设、三尺寸、
所有 parent 和所有可选装饰。特殊差异要留：有序 path／band、不可估计 row、
signed stack、linked Marginal、Matrix cell、Heatmap missing grid、Beeswarm packing、
Waterfall cumulative spans，以及 Facet／Shared 的源映射和统一域。

### 私有 witness 与 schema

审阅 theme edit/schema、root theme-address witnesses、private cache 容器条数与完整
阶段事件数组。用真正 field/owner/input 变化和不可变性检查承担语义，把重复 storage
echo 和实现地址日志删掉。碰撞键不能错误复用、operation clone／新 operation 区分、
缓存上限后仍能正常渲染等规则继续有专门证据。

### 基准与支持代码

`tests/benchmarks` 有 22 个具名 tests，其中仅十个是 ignored release measurement，
其他有实际合同；不能整目录删。`assert_outside_trace`、`shared_case` 与
`assert_child_reuse` 先移除旧固定 counter/trace tuples，再收敛计时／输出核对辅助。
必要观察仅按语义分别报告，不恢复总分。

剩余测试需要什么，支持代码才保留什么。重点是 RecordingBackend、重复的 tempdir／
SVG／PNG parser 小 helper、STAGES／thread-local probes、各处 test-edit/poison bridges。
删最后消费者后同步删专用 hook／字段／宏／helper，避免测试变少但 supporting code
还留着。只用短的现有局部 helper 或数据表消除真实重复；不新建泛型 testing DSL、
trait hierarchy、统一 fault injection、测试 backend 框架或 mutation 系统。

## 六、端到端保留骨架

不是新增一个全面组合矩阵；从现有有效 tests 中选择并合并，记录主承担者：

| 合同 | 现有承担者与保留方式 |
| --- | --- |
| 实际外部库编译和合法发布 | `tests/output/multiformat.rs`、`tests/rendering/public_contracts.rs`；公开 Plot／Canvas 输入到实际文件，不全部移入 lib tests |
| 科学数据与图形对应 | 现有 line/order、signed bar、estimate status、Histogram/KDE、`marginal_matrix/scientific.rs`；每种独立变换留独立值／源／图元 oracle |
| SVG 文本／outline 与 Output 复用 | 合并 multiformat 相关 cases；同一 Output 当前路线的一致性与完整文本／path 结构保留 |
| PNG 实际生成／解码、颜色、call-local sizing | 现有 Heatmap decoded pixel 与 output PNG cases；像素位置来自实际 scene，大小来自用户请求／DPI 数学，不固定全图布局 |
| 发布成功／失败的文件状态 | `tests/output/paths.rs` 和必要 artifacts unit；保留 suffix、existing destination、no partial/no staging 的不同 OS/错误类 |
| Font、Unicode、完整长 caption | 现有中文 fallback、bidi/mark/soft-hyphen/frozen text 与完整 caption fixture；不得删字减点以降低成本 |
| Parent／Shared／sparse 交互 | 现有 composition targets；普通 mixed Canvas、稀疏 facet、Shared domain/guide conflict 与 explicit owner replacement，各保留不同语义 |
| 类型和 capability | 现有 positive marker checks 和 compile-fail docs；压缩重复输入，但保留 exact／Any／unsupported-neighbor 边界，明确 doctest 执行 |

内部 immutable replay 不重做布局／shaping／统计，需要少量精确 lifecycle witness；
仅比较两份 SVG 相同不能证明内部没重做，也不能证明科学结果本来就正确。
这是保留必要内部集成与外部 E2E 两层的理由，不是要求每层复制所有场景。

## 七、实施阶段与提交

按下表推进，每个完成的逻辑单元先验证、提交再继续：

| 顺序 | 范围 | 完成标准 | 目前状态 |
| --- | --- | --- | --- |
| 1：删除明确自我确认／残余额度 smoke | 第四节具名目标；检查是否还有同型断言 | 已确认的无独立合同自我对照、额度消失后的丢弃结果重跑清除；混合科学断言保留 | 已完成 |
| 2：合并生产者及父子重复 | 第五节 capture/route/parent fixtures，按图元／独立语义拆小提交 | 每个去掉的重复有明确保留承担者；25 family 派发、特殊源／图形、关键 cache/feedback/replay 不丢 | 已完成 |
| 3：简化 schema/witness／基准支持 | 私有地址／容器／counter tuples；相应支持消费者清点 | 不固定内部轨迹，必要诊断还能运行；删尽无人用的 helper/hooks，无新大型 harness | 已完成 |
| 4：公开 E2E／API 编译归属 | 精简并标注第六节已有骨架；决定 doctest 在 Justfile/checks 中的正确 gate | 正常库编译到文件／部分解码和 API boundary 可定向运行；不能只剩 mock 与编译 smoke | 待开始 |
| 5：整体收尾 | 当前规范、census、支持删除和针对性的总检查 | 一次有理由的 broad closure、strict Clippy、前后测试相关 SLOC／运行范围报告；保持功能暂停 | 待开始 |

提交标题写长期行为，如 `test(layout): consolidate immutable replay contracts`，不写
本表序号或临时 stage ID。复杂混合阶段按独立 owner 拆分，不能等到最后一个大删除。
只剩一个私有表示；迁移旧 helper 调用后删旧实现，不加兼容 wrapper／别名。

## 八、验证与规模验收

- 每小段用最窄的受影响 unit／integration tests 加严格
  `cargo clippy --locked --offline --all-targets -- -D warnings`（或 `just lint`）。
- 实施删除前先说明保留了哪个独立合同；没有替代者且合同仍有效时保留原测试，
  或对既有测试补最小缺失语义，不能把它改成 `.unwrap()` smoke 来获得绿色。
- 在最后完成前述迁移时，说明 scope 后做一次必要整体 gate；局部修复只复验局部。
  API 编译检查单独使用相关 doctests，不能把 docs build 当作运行过。
- 保留当前科学／格式／源／字体／checked arithmetic／真实分配／循环终止等错误。
  可测试的机制不因删除测试改变；失去最后测试消费者的专用 hooks 同步删除。
- 以相同 tracked-file cloc 和 test-only span 规则报告每阶段净变化；分别报 production、
  generated tables、test bodies/support/bench、具名／ignored cases、实际执行层级。
- 行数是交付证据，不是“删除到 X 行就算正确”的配额。第一轮确认的小项不会单独
  消除 7.6 万行测试；主要收益须来自后续真正重复的 fixtures 和支持代码收敛。
  现在不虚构可可靠净删几万行的数值承诺。
- 不把源文件搬到另一个目录、压成长行或扩大宏生成来冒充精简；不移走测试代码而
  保留原数量的私有 helpers；不以测试个数／覆盖率百分点代替失效面分析。
- 不添加 Python、`tools/`、依赖、资产下载、公开调试 API、生产模式开关、新 backend
  或另一条 renderer。独立 Unicode 表／标准数据与外部语料既有 ignored 政策保留。

## 九、此次交付与下一步

实施证据与每段保留的独立合同、实际验证及规模变化记录在
`../docs/test-code-cleanup.md`。首批 40 个定向测试及严格 Clippy／格式通过，
净删除 120 行 Rust code；后续按已批准范围继续合并重复。

审计、可复核的行数分类、已有端到端路径和具名删除／合并对象已记录。用户已批准：
**先删已确认的自我确认，随后有保留承担者地合并重复，收敛支持代码，留下独立
数学／科学 oracle 与小型真实公开端到端骨架。**

按第七节实施；主题／公开接口功能继续暂停，直至这项测试清理完成后再
向用户汇报并决定后续工作，不因这里列出原功能阶段而自动开始。
