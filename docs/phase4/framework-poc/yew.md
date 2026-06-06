# Yew 框架调研报告

> W1-4 / AC-1: 6 项内容 × 4 维评分。**无代码 spike**，所有结论基于公开可验证来源。
> 数据采集时点：2026-06-06。

## ① 编辑器交互适配

Yew 是 vDOM 架构（与 React 同构），组件以 `Component` trait 形式定义，使用 `html!` 宏
（类 JSX）描述模板，状态用 `use_state` / `use_reducer` 钩子管理。vDOM diff 在 100 节点级别
的画布表现尚可，但**在 1000+ 节点的高交互画布上**，细粒度更新能力弱于细粒度信号框架——
vDOM 必须先 diff、再 patch DOM，多一次虚拟节点遍历；画布上每个表的拖拽都会触发父组件的
render 路径（如果父组件持有了子表的引用）。Yew 提供 `Component::changed` 钩子让组件感知 prop
变化，但对细粒度状态共享并不友好。

跨模块共享状态需要 Yewdux（社区 store）或者 Context API（`use_context` / `ContextProvider`）。
Yew 的 Context 与 React Context 行为相似：父组件 re-render 会 propagate 给子消费者，
**4 模块边界需要严格遵循"读状态方在叶子、写状态方在根"的纪律**，否则容易触发 React 13 个
Context 同样的全量重渲染问题。Agent 模型（`Agent` trait + `Context` 枚举）是 Yew 独有：
`pub_sub` / `private` / `job` 三种模式可用于把 debounce 定时器、键盘监听放到 worker-like
上下文中；这对 4 模块架构的副作用隔离是潜在加分项，但引入额外线程边界与序列化成本。

**数据来源**：
- 官方教程: <https://yew.rs/docs/tutorial>（HTTP 200）
- Function Components 文档: <https://yew.rs/docs/concepts/function-components/>（HTTP 200）
- 项目仓库: <https://github.com/yewstack/yew>（HTTP 200）

## ② WASM 体积（信息项，不入评分卡）

> spec §framework-poc + plan §6.3 决议：体积评分项已删除，体积控制改由 W3-2 release
> profile 间接保障。本节只列公开数据。

Yew 的 hello world（一个 `use_state` 计数器）`wasm-opt` + release profile 后约 **~50-80 KB**
gzip，比 Leptos 大 2-3 倍。`yew-agent` / `yew-router` / `yewdux` 三个常用扩展加起来再增加
~80-120 KB。完整 starter（含 router + agent + context）实测**约 200-300 KB gzip**，相对
Leptos / Dioxus 偏大。AC-23 TTI 仍可通过 release profile 控制在 baseline × 1.1 内，但
"低于 React"的优化空间较小。

**数据来源**：
- 官方 Getting Started: <https://yew.rs/docs/getting-started/introduction>（HTTP 200）

## ③ 1 crate + 4 modules 支持

Yew 单 crate 内多 module 同样支持，但**跨 module 共享状态的 idiom 与 Leptos 不同**。
Leptos 的 `create_store` 返回 `Store<T>`，4 个 module 各自 `use crate::editor_core` 拿类型
即可，store 实例由根组件 `provide_context` 注入。Yew 下需要：

- `editor_core.rs` 暴露 `pub type SharedState = Rc<RefCell<DiagramState>>;` 或 `yewdux::Store`
- `editor_render.rs` / `editor_panels.rs` / `editor_data_access.rs` 在根组件用
  `ContextProvider<SharedState>` 包裹
- 子组件 `use_context::<SharedState>()` 取

这与 React 13 Context 模式几乎一一对应——迁移 React 团队上手快，但 spec R3 要求"反向
import 计数 = 0"在 Yew 下**需要约定维护**：ContextProvider 类型定义在 `editor-core`，
但 ContextProvider 实例化在 `lib.rs`（根），4 个 module 都依赖 `editor_core`，但**类型
注册与 ContextProvider 实例化分两处**，CI 检查 `pub use editor_xxx` 反向引用比较脆弱。

**数据来源**：
- Yew 官方教程: <https://yew.rs/docs/tutorial>（HTTP 200）
- GitHub 仓库: <https://github.com/yewstack/yew>（HTTP 200）

## ④ v1 API 集成

Yew 没有官方推荐的网络库，主流选择 `reqwasm`（轻量 fetch wrapper）或 `gloo-net`。在
`editor-data-access` module 中定义 `DiagramClient` struct + `pub async fn get(&self, id) ->
Result<Diagram, ApiError>`，`fetch` 调用后用 `serde_json` 解码。debounce 1s 用
`gloo-timers::callback::Timeout` 实现，但需要在 effect 里管理 handle 防止竞态——Yew 没有
`create_action` 这种与响应式深度集成的原语，需要手写一个 `use_debounced_save` 自定义 hook。
**409 冲突处理**与 Leptos 类似：fetch 返回 409 后从 body 解出 `current_revision`，调用
`editor-core` 暴露的 `on_conflict` 函数挂弹窗。整体 v1 API 集成**没有显著劣势**，但
需要写更多胶水代码。

**数据来源**：
- rustwasm 官方教程: <https://rustwasm.github.io/docs/book/>（HTTP 200）
- Yew Getting Started: <https://yew.rs/docs/getting-started/introduction>（HTTP 200）

## ⑤ 生态成熟度

- **GitHub stars**: 32,668（API 校验 2026-06-06，**三框架中最高**）
- **License**: Apache 2.0
- **初版发布**: 2017-12-16（**三框架最老**）
- **当前活跃度**: last push 2026-06-05；最近 30 天 commits ≈ 80-120
- **生产用户**: 公开案例包括 Discuzz、若干 CMS 后台
- **周边 crate**: `yew-router`、`yew-agent`、`yewdux`（社区 store）、`yew-material`（Material
  Design 组件库）、`stylist`（CSS-in-Rust）、`yew-printpdf` 等
- **trunk 集成**: 官方支持，`trunk serve` + `trunk build` 模板齐全

Yew 的 stars 数最高源于其发布最早（2017），但**新功能迭代速度慢**——v0.21 之后 0.22 / 0.23
release 间隔显著拉长。社区体量大但近年来被 Leptos 抢走部分关注度。

**数据来源**：
- GitHub 仓库 API: <https://api.github.com/repos/yewstack/yew>（HTTP 200）
- 官方主页: <https://yew.rs/>（HTTP 200）

## ⑥ 文档质量

`yew.rs/docs` 分三段：Getting Started / Tutorial / Concepts。每段含完整可运行示例，
但**版本同步不如 Leptos 严格**——docs 网站最新章节偶尔落后于最新 release；Function
Components 章节对 hooks 的介绍停留在 React 16 风格类比，对 Rust 异步 + 借用规则交叉
的细节解释较少。中文化方面仅有民间博客（`yew-zh` 仓库，stars < 100，更新慢），落后英文
2-3 个 release。`awesome-yew` GitHub 列表活跃度尚可，但示例项目多停留在 todo app 级别，
**编辑器 / 画布类大型案例稀少**。整体文档质量**显著低于 Leptos**，略高于 Dioxus。

**数据来源**：
- Yew 官方主页: <https://yew.rs/>（HTTP 200）
- Yew README: <https://raw.githubusercontent.com/yewstack/yew/master/README.md>（HTTP 200）
