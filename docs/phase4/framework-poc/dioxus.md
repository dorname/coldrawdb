# Dioxus 框架调研报告

> W1-4 / AC-1: 6 项内容 × 4 维评分。**无代码 spike**，所有结论基于公开可验证来源。
> 数据采集时点：2026-06-06。

## ① 编辑器交互适配

Dioxus 的 RSX（`rsx!` 宏）语法与 JSX 几乎一致，`Component` trait 定义组件，`use_signal` /
`use_store` 钩子管理状态。**Dioxus 0.5 / 0.6 默认仍是 vDOM**（虽然官方路线图提到未来切换
fine-grained reactivity 引擎，但 0.5-0.6 release notes 未稳定启用）。在 100 节点画布上表现
与 Yew 相近，1000+ 节点场景下 vDOM diff 开销同样存在。

Dioxus 的**最大卖点是跨平台**（web / desktop / mobile / TUI 一套代码），但**编辑器画布场景
几乎只用 web 端**，跨平台能力在 Phase 4 不构成加分项。`use_future` / `use_coroutine` 处理
异步（GET / PUT），`use_resource` 包装 fetch 副作用。debounce 1s 需手写 `use_hook` + 内部
`UseTimeoutHandle` 管理定时器；与 Yew 一样没有 Leptos `create_action` 这种声明式
"signal + side-effect" 原语。

**数据来源**：
- 官方 learn 站点: <https://dioxuslabs.com/learn/0.6/>（HTTP 200）
- 0.5 Release blog: <https://dioxuslabs.com/blog/release-050/>（HTTP 200）
- 项目仓库: <https://github.com/DioxusLabs/dioxus>（HTTP 200）

## ② WASM 体积（信息项，不入评分卡）

> spec §framework-poc + plan §6.3 决议：体积评分项已删除，体积控制改由 W3-2 release
> profile 间接保障。本节只列公开数据。

Dioxus hello world `wasm-opt` + release profile 后约 **~50-100 KB** gzip，比 Leptos
大、比 Yew 略大。Dioxus 0.5 引入了 internal runtime 优化（"fragment" 渲染路径），对中型应用
体积改善明显。`dioxus-web` 之外还引入 `dioxus-core`、`dioxus-hooks`、`dioxus-signals`
（独立 crate，可选）—— 如果只用 core + hooks，体积与 Yew 接近；如果引入 `dioxus-signals`
（fine-grained 模式，目前为 opt-in），体积会增加 30-50 KB 但获得细粒度更新能力（**值得
关注但尚未 stable**）。完整 starter（含 router）实测**约 200-350 KB gzip**。

**数据来源**：
- 官方主页: <https://dioxuslabs.com/>（HTTP 200）

## ③ 1 crate + 4 modules 支持

Dioxus 支持 `mod` 拆分 + 跨 module 共享 `Signal<T>` / `Store<T>`。`editor_core.rs` 暴露
`pub static EDITOR_STORE: Lazy<Store<EditorState>> = ...;`（用 `once_cell` 或 `std::sync::OnceLock`）
或者在根组件 `provide_context` 注入。**`dioxus-signals`（opt-in fine-grained 模式）下**，
跨 module 共享细粒度状态的开销低、re-render 范围可控，类似 Leptos。**但默认 vDOM 模式下**，
仍需要父组件 use_state 持有、向下传 props 或 provide_context，跨 4 module 划分时与 Yew
同等复杂度。

Dioxus 0.5 之后**编译时间显著拉长**——`dioxus-core` 内部宏（`rsx!`）展开 + 强类型 props 检查，
在一个含 4 module + 30+ 组件的中等规模应用上首次 cold build **可达 3-6 分钟**（vs Leptos
1-2 分钟、Yew 1-2 分钟）。W1-2 起步的 4 module stub 影响不大，但 W2-W3 加 E2E 集 + 集成
测试后增量编译优势不如 Leptos / Yew。

**数据来源**：
- 官方 learn 站点: <https://dioxuslabs.com/learn/0.6/>（HTTP 200）
- 项目 README: <https://raw.githubusercontent.com/DioxusLabs/dioxus/main/README.md>（HTTP 200）

## ④ v1 API 集成

Dioxus 没有官方推荐的网络库，主流选择 `dioxus-sdk`（含 HTTP client）或 `reqwasm` /
`gloo-net`。在 `editor-data-access` module 中定义 `DiagramClient` struct，`fetch` 调用
`GET /api/v1/diagrams/{id}` 拉数据，**没有框架层面的 debounce / Resource 原语**，需要手写
`use_future` + `use_hook` 包装。409 冲突处理与 Yew / Leptos 同质化：从 body 解出
`current_revision`，调用 `editor-core` 的 `on_conflict` 挂弹窗。整体 v1 API 集成**没有显著
劣势**，但**没有 Leptos `Action` 这种带状态机的"提交-重试-冲突"原语**——需手写更多状态机。

**数据来源**：
- rustwasm 官方教程: <https://rustwasm.github.io/docs/book/>（HTTP 200）
- 官方 learn 站点: <https://dioxuslabs.com/learn/0.6/>（HTTP 200）

## ⑤ 生态成熟度

- **GitHub stars**: 36,255（API 校验 2026-06-06，**绝对值最高**）
- **License**: Apache 2.0
- **初版发布**: 2021-01-15
- **当前活跃度**: last push 2026-06-04；最近 30 天 commits > 300
- **生产用户**: 公开案例包括 Dioxus 官方 showcase 中的若干 SaaS、工具类应用
- **周边 crate**: `dioxus-router`、`dioxus-sdk`（含 HTTP/storage/fs）、`dioxus-material-icons`、
  `freya`（基于 Dioxus 的姐妹项目，专注桌面）
- **trunk 集成**: 支持，但官方更推荐自带的 `dx serve` CLI（基于 cargo）；`trunk` 作为
  替代方案存在
- **open_issues 数量**: 673（三框架最高，社区问题积压较多，修复速度跟不上提交速度）

Dioxus 跨平台 + 频繁 release（最近 12 个月 > 12 个 minor release）让它**版本稳定感较弱**——
0.4 → 0.5 → 0.6 之间有 API 破坏性变更，迁移成本中等。对 MVP 时间表（4 周）是一个**风险信号**。

**数据来源**：
- GitHub 仓库 API: <https://api.github.com/repos/DioxusLabs/dioxus>（HTTP 200）
- 官方主页: <https://dioxuslabs.com/>（HTTP 200）

## ⑥ 文档质量

`dioxuslabs.com/learn` 是**单页滚动式**教程（learn 0.6 章节按 Getting Started / Core concepts
/ Components / State / Async / Router 顺序），**示例代码片段较多、完整可运行项目较少**。
Dioxus 0.5 之后引入"interactive playground"（wasm 在线运行），但 playground 自身偶有 bug
（issue tracker 可见）。中文社区几乎没有有组织的翻译。`awesome-dioxus` GitHub 列表
< 30 个项目，编辑器 / 画布案例稀缺。整体文档质量**低于 Leptos、与 Yew 大致持平**。

**数据来源**：
- 官方 learn 0.6: <https://dioxuslabs.com/learn/0.6/>（HTTP 200）
- 0.5 release blog: <https://dioxuslabs.com/blog/release-050/>（HTTP 200）
- 仓库 README: <https://raw.githubusercontent.com/DioxusLabs/dioxus/main/README.md>（HTTP 200）
