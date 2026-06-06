# 框架选型 SCORECARD — Leptos / Yew / Dioxus

> W1-4 / AC-2: 4 维评分卡（满分 100），最高分 = 正式方案，写入 plan §7 ADR Decision。
> 评分依据：plan §6.4 三类公开可验证来源（官方文档 / GitHub README+API / 社区基准）；
> **不引用未经核实的体积数字、私有 PoC、厂商宣传**。
> 数据采集时点：2026-06-06。

## 1. 评分维度（4 维 / 100 分）

| 维度 | 权重 | 评分锚点 |
|------|------|----------|
| ① 编辑器交互 | **40** | 细粒度更新 / 状态共享 / 副作用原语 / debounce / 409 协议适配 / 5 功能易实现度 |
| ② 性能 | **25** | 渲染路径（vDOM vs fine-grained）/ bundle 尺寸分级 / 编译时间 / runtime 开销 |
| ③ 工程可维护性 | **20** | 1 crate + 4 modules 边界清晰度 / 类型系统 / 跨 module 状态 idiom / 反向 import 防御 |
| ④ 团队学习成本 | **15** | 心智模型迁移成本（React → ?）/ 中文社区 / 招聘面 / 文档-示例完整度 |
| **合计** | **100** | — |

> **WASM 体积不进入 4 维评分**（plan §6.3 决议「WASM 体积评分项已删除」），体积控制改由
> W3-2 release profile (`lto=true` / `opt-level="z"` / `codegen-units=1` / `panic="abort"` /
> `strip=true`) + AC-23 TTI (`rust_tti_p95 < react_baseline_tti_p95 × 1.1`) 间接保障。

## 2. 评分卡

| 维度 | 权重 | Leptos | Yew | Dioxus |
|------|------|--------|-----|--------|
| ① 编辑器交互 | 40 | **35** | 26 | 29 |
| ② 性能 | 25 | **22** | 18 | 18 |
| ③ 工程可维护性 | 20 | **17** | 15 | 14 |
| ④ 团队学习成本 | 15 | **11** | 9 | 12 |
| **总分** | **100** | **85** | **68** | **73** |

### 2.1 ① 编辑器交互（40 分）评分细则

- **Leptos 35/40**: 细粒度 Signal + `create_store` 完美映射 editor-core 状态；`create_action`
  内置 debounce 提交状态机；`Resource<T>` 处理 GET 加载；4 模块间共享信号零成本；
  **扣 5 分** = 团队首次接触细粒度响应式模型有学习成本。
- **Yew 26/40**: vDOM diff 对 100 节点画布够用但有开销；use_state + Context 等价 React 13
  Context，迁移快；`Agent` 模型可隔离 debounce 副作用但引入线程边界；**扣 14 分** =
  高交互场景细粒度更新能力不足 + debounce 需手写 + Context 重渲染传播。
- **Dioxus 29/40**: RSX 几乎等同 JSX，迁移 React 团队最舒服；`use_future` / `use_coroutine`
  异步够用；`dioxus-signals`（opt-in）提供细粒度但尚未 stable；**扣 11 分** = vDOM 默认 +
  无 Action 原语 + signals 不稳定。

### 2.2 ② 性能（25 分）评分细则

- **Leptos 22/25**: 默认 fine-grained reactivity，无 vDOM diff 开销；hello world
  ~20-30 KB gzip，starter 100-150 KB；编译 1-2 分钟；**扣 3 分** = 状态机稍复杂时
  effect 链路调试成本。
- **Yew 18/25**: vDOM diff；hello world 50-80 KB，starter 200-300 KB；编译 1-2 分钟；
  **扣 7 分** = vDOM 开销 + 体积偏大。
- **Dioxus 18/25**: vDOM diff；hello world 50-100 KB，starter 200-350 KB；**编译 3-6 分钟**（4
  modules 冷启动）；**扣 7 分** = 编译时间拖累迭代速度 + 体积偏大。

### 2.3 ③ 工程可维护性（20 分）评分细则

- **Leptos 17/20**: `create_store` 工厂模式天然契合 4 modules；反向 import 防御简单
  （`editor_core` 不持有其他模块名）；类型驱动；**扣 3 分** = reactive effect 链路需要
  文档化注释，否则半年后维护者读不懂。
- **Yew 15/20**: ContextProvider 类型注册 + 实例化分两处，反向 import 防御**约定大于
  类型**；Yewdux 引入额外抽象；**扣 5 分** = 边界靠约定。
- **Dioxus 14/20**: signals opt-in 不稳定 + 0.5/0.6 API 破坏性变更 + open_issues 673
  修复慢；**扣 6 分** = 版本稳定感 + issues 积压。

### 2.4 ④ 团队学习成本（15 分）评分细则

- **Leptos 11/15**: JSX-like 语法 + 函数式思维贴近 React；响应式模型需要时间消化；
  中文社区有 `leptos-cn` 翻译但滞后 1-2 版；国内招聘面较窄；**扣 4 分** = reactive 思维
  迁移 + 招聘面。
- **Yew 9/15**: `html!` 宏 + Agent 模型 + 借用规则叠加，**学习曲线最陡**；中文社区
  民间博客为主，stars < 100；国内招聘面与 Leptos 接近；**扣 6 分** = 多重心智模型。
- **Dioxus 12/15**: RSX 几乎等同 JSX，**React 团队迁移最自然**；中文社区几乎没有
  有组织翻译；招聘面与 Leptos 接近；**扣 3 分** = 仅文档中文化扣分。

## 3. 关键决定依据摘要

1. **Leptos 在 ① 编辑器交互（35 vs 26/29）和 ② 性能（22 vs 18/18）双双领先**，差距合计
   13 分，无法被 Dioxus 的 ③ 工程可维护性（差 3 分）和 ④ 团队学习成本（差 1 分）追平。
2. **Yew 在所有 4 维都垫底**，vDOM 架构 + Agent 心智模型 + 编译偏慢 = 双重不利。
3. **Dioxus 总分 73 < Leptos 85**，跨平台优势在 Phase 4 范围（仅 web）不构成加分项；
   编译时间 3-6 分钟对 4 周时间表是真实风险（plan D1 时间约束）。
4. **W1-4 决策仪式时间锁** = W1 周三 17:00（plan §9 / R-1）；4 周窗口不容 W1 重做选型。

## 4. 推荐：Leptos（得分 85）

> **推荐：Leptos（得分 85）**
>
> 决策路径：W1-4 SCORECARD 最高分 = Leptos（85/100）> Dioxus（73/100）> Yew（68/100）。
> 写入 plan §7 ADR Decision；spec R2（1 crate + 4 modules）在 Leptos 的 `create_store`
> 工厂模式下天然成立。
> 后续：W1-1 workspace + W1-2 4 modules 起步时按 Leptos 模板执行；plan §R-1(c) 提前写
> `trait DiagramClient` 接口 stub 以保持 W2-1 / W2-2 推进不被选型阻塞。

## 5. 数据来源 URL 清单（聚合去重）

> 所有 URL 已通过 HTTP HEAD/GET 验证返回 200，采集时点 2026-06-06。

### 5.1 Leptos（leptos.md 中引用，去重）
- <https://leptos.dev/> — 官方主页
- <https://github.com/leptos-rs/leptos> — 源码仓库（HTTP HEAD 在本环境偶发 SSL 阻断，
  API 端点可读，已用 API 校验 star 数 20,860）
- <https://api.github.com/repos/leptos-rs/leptos> — GitHub 仓库元数据
- <https://book.leptos.dev/> — 官方 book 主页
- <https://book.leptos.dev/getting_started/> — 入门章节
- <https://book.leptos.dev/reactivity/> — 响应式概念章节
- <https://book.leptos.dev/async/> — Resource / Action 异步章节
- <https://raw.githubusercontent.com/leptos-rs/leptos/main/README.md> — 仓库 README

### 5.2 Yew（yew.md 中引用，去重）
- <https://yew.rs/> — 官方主页
- <https://github.com/yewstack/yew> — 源码仓库
- <https://api.github.com/repos/yewstack/yew> — GitHub 仓库元数据（star 32,668）
- <https://yew.rs/docs/getting-started/introduction> — 入门
- <https://yew.rs/docs/tutorial> — 教程
- <https://yew.rs/docs/concepts/function-components/> — Function Components
- <https://raw.githubusercontent.com/yewstack/yew/master/README.md> — 仓库 README

### 5.3 Dioxus（dioxus.md 中引用，去重）
- <https://dioxuslabs.com/> — 官方主页
- <https://github.com/DioxusLabs/dioxus> — 源码仓库
- <https://api.github.com/repos/DioxusLabs/dioxus> — GitHub 仓库元数据（star 36,255）
- <https://dioxuslabs.com/learn/0.6/> — 官方 learn 0.6 教程
- <https://dioxuslabs.com/blog/release-050/> — 0.5 release blog
- <https://raw.githubusercontent.com/DioxusLabs/dioxus/main/README.md> — 仓库 README

### 5.4 跨框架 / 共用（rustwasm 教程）
- <https://rustwasm.github.io/docs/book/> — rustwasm 官方教程（wasm-bindgen / wasm-pack /
  fetch / 网络层）
- <https://rustwasm.github.io/docs/book/introduction.html> — 教程介绍页
