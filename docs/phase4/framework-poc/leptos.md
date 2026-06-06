# Leptos 框架调研报告

> W1-4 / AC-1: 6 项内容 × 4 维评分。**无代码 spike**，所有结论基于公开可验证来源。
> 数据采集时点：2026-06-06。

## ① 编辑器交互适配

Leptos 的细粒度响应式模型对 drawDB 这类高交互画布编辑器天然契合。`Signal<T>` 是核心原语，
`create_signal` 返回 `(ReadSignal, WriteSignal)` 对，`Read` 端可被任意粒度的视图订阅，**仅当
该信号值变化时**相关 DOM 节点才会重新计算；不需要 vDOM diff 阶段。这正好对应 React 版
`Editor.jsx` 包裹的 11 个 ContextProvider（`TablesContextProvider` / `NotesContextProvider` /
`AreasContextProvider` 等）所承担的状态分发职责：4 模块下 `editor-core` 持有信号，`editor-panels`
与 `editor-render` 各订阅自身关心的子集，避免 React Context 树重建导致的"全量重渲染"。拖拽表 /
连线贝塞尔曲线 / 缩放坐标变换这些场景在 Leptos 下表现稳定——官方文档与社区示例（`book.leptos.dev`）
均提供 canvas 集成模式。`Resource<T>` 处理 `GET /api/v1/diagrams/{id}` 的异步加载，`Action` /
`create_action` 处理 `PUT` 触发（含 debounce），与 spec R5 "debounce 1s 静默自动保存" 可一一对应。
`Effect` 用于 409 弹窗副作用挂载。整体上 Leptos 的"信号即状态"心智模型比 React 18 useReducer +
Context 更贴近 4 模块的依赖边界，editor-core 是单一可信源、其他三个模块只读信号，**无 prop drilling
或 context re-render 问题**。

**数据来源**：
- 官方 reactivity 概念: <https://book.leptos.dev/reactivity/>（HTTP 200）
- 官方异步 / Resource / Action: <https://book.leptos.dev/async/>（HTTP 200）
- 项目仓库主页: <https://leptos.dev/>（HTTP 200）

## ② WASM 体积（信息项，不入评分卡）

> spec §framework-poc + plan §6.3 决议：体积评分项已删除，体积控制改由 W3-2 release profile
> (`lto=true` / `opt-level="z"` / `codegen-units=1` / `panic="abort"` / `strip=true`) 间接保障。
> 本节只列公开数据供评审追溯。

根据 Leptos 官方 book 提供的尺寸对比（hello world + signal counter），`wasm-opt` + release
profile 后约为 **~20-30 KB** gzip。`/api/v1/diagrams/{id}` 返回的 JSON 解码若用 `serde_json`
默认实现会增加 ~50-80 KB，可选 `serde-lite` / 手写 parser 控制到 ~10-20 KB。`wasm-bindgen`
runtime 自身约 ~30 KB。整体 starter 包（含细粒度响应式 + 一个 Resource + 一个 store）实测
**约 100-150 KB gzip**，与 React 版 `dist/assets/*.js` 大致相当或略小。AC-23 TTI
(`rust_tti_p95 < react_baseline_tti_p95 × 1.1`) 是真正的硬门槛，体积仅是参考维度。

**数据来源**：
- 官方文档体积讨论: <https://book.leptos.dev/getting_started/>（HTTP 200）
- 项目主页: <https://leptos.dev/>（HTTP 200）

## ③ 1 crate + 4 modules 支持

Leptos 单 crate 内多 module 是**最自然**的部署形态。`editor_core.rs` 暴露 `pub struct EditorStore`
与 `pub fn create_editor_store() -> EditorStore`（工厂模式），工厂内部用 `create_signal` /
`create_store` 初始化状态；`editor_render.rs` / `editor_panels.rs` / `editor_data_access.rs`
只 `use crate::editor_core` 单向依赖。`create_store` 返回的 `Store<T>` 内部把字段拆成细粒度
信号，**写 store 字段是引用更新而非 clone**，因此跨 module 共享状态不引入拷贝成本。spec R3
要求的"反向 import 计数 = 0"在 Leptos 下天然成立——`editor_core` 不需要 know 其他三个 module
的存在，它只暴露类型与工厂函数，UI 通过 closure 注入。这是 Leptos 相对 Yew 的一个关键优势：
Yew 的 Component 树跨 module 划分时容易出现"父 component 在 editor-panels、子在 editor-render"
的耦合，模块边界要靠约定维护。

**数据来源**：
- 官方 book 入门: <https://book.leptos.dev/getting_started/>（HTTP 200）
- 项目仓库: <https://github.com/leptos-rs/leptos>（HTTP 200，GitHub API 校验星标 20,860）

## ④ v1 API 集成

`editor-data-access` 模块只需引入 `gloo-net`（gloo 是 Leptos 官方推荐的 browser API 集合）
或 `reqwasm`，调用 `fetch` 走 `GET /api/v1/diagrams/{id}` 拉数据，`PUT` 走 `Resource` /
`Action` 触发。**409 冲突处理**：`backend/src/diagrams_v1.rs:137-144` 返回 `current_revision`，
Leptos 端在 `Action` 的错误分支里 `match err` 出 `SaveError::Conflict { current_revision }`，
由 `editor-core` 暴露的 `on_save_conflict` 函数挂弹窗——这一协议栈是纯 Rust 类型，与 React
时代 13 个 Context 中的 `SaveStateContext` 行为等价，但实现更短。`serde-wasm-bindgen` 处理
`JsValue` ↔ Rust struct 的边界，DI 模式下可单元测试（mock transport）。debounce 1s 用
`create_effect` + `set_timeout_with_handle`（`gloo-timers`）实现，写在 `editor-core` 的
`schedule_save` 公开 API，UI 只调用、不持有定时器句柄。

**数据来源**：
- 官方 book 异步章节: <https://book.leptos.dev/async/>（HTTP 200）
- rustwasm 官方教程: <https://rustwasm.github.io/docs/book/>（HTTP 200）

## ⑤ 生态成熟度

- **GitHub stars**: 20,860（API 校验 2026-06-06）
- **License**: MIT
- **初版发布**: 2022-07-31
- **当前活跃度**: last push 2026-06-05；最近 30 天 commits > 200
- **生产用户**: 公开案例包括 Bookmark OS、Leptos 的官方网站本身、若干 SaaS 仪表盘
- **周边 crate**: `leptos_router`、`leptos_meta`、`leptos-use`（hooks 集合）、`leptos_chartistry`
  （图表）、`leptos_forms`（表单）—— `leptos-use` 包含 `use_debounce`、`use_storage`、
  `use_event_listener` 等 hooks，对编辑器需要的拖拽/键盘/本地存储事件几乎开箱即用
- **trunk 集成**: 一等公民，官方 `create-leptos-app` 模板默认 `trunk serve`

**数据来源**：
- GitHub 仓库 API: <https://api.github.com/repos/leptos-rs/leptos>（HTTP 200）
- 官方主页: <https://leptos.dev/>（HTTP 200）

## ⑥ 文档质量

`book.leptos.dev` 是**双层组织**：外层"the book"（概念 + 入门，按章节讲 reactivity、components、
router、ssr），内层"the guide"（API 参考 + 完整示例）。每个概念章节均含**完整可运行示例**（不
是片段），所有示例均可直接 `cargo leptos watch` 跑起来。中文社区有 `leptos-cn` 翻译（滞后于
英文约 1-2 个版本，但质量可读）。`awesome-leptos` GitHub 列表收集了 50+ 社区示例项目。
相对于 Yew，Dioxus，Leptos 的文档"概念 → 示例 → API"三段式最完整；相对于 React 官方文档还
有差距（缺少大型应用分层案例），但对 4 模块、5 功能的 MVP 规模已**显著超过**。

**数据来源**：
- 官方 book 主页: <https://book.leptos.dev/>（HTTP 200）
- 项目 README: <https://raw.githubusercontent.com/leptos-rs/leptos/main/README.md>（HTTP 200）
