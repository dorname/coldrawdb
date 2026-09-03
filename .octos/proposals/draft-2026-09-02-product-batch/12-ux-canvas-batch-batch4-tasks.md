# 实现任务 — ux-canvas-batch 批次 4（细化 tasks，列宽可调 + 表/字段分组 + 样式优化）

> 状态：**细化 tasks**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`08-ux-canvas-batch-proposal.md`（Q1 全 9 项 + Q5 样式子集）+ 批次 1/2/3 tasks 已闭环
> 上游裁决链：黑板条目 6（operator Q1 = 全 9 项列表视图候选）+ Q5（外环代决样式子集：字体回退栈 + 子像素抗锯齿 + 中文字体思源黑体/苹方 + Canvas 文本离屏缓存 + 关键交互 <16ms + rAF 统一调度；大图虚拟化暂缓）→ 条目 11 C-1/C-3 裁决 → 条目 13 修复 ACK → 条目 16 修复 ACK → 条目 17 派批次 4
> 批次 1/2/3 闭环范围（已闭环，本批不再做）：表名/字段名/类型表格化展示、排序、过滤、批量重命名、批量改类型、双击跳画布、导出 CSV schema 内容
> **批次 4 范围（条目 17 steer）**：
>   - 列宽可调（**列宽持久化落点明确**——见强制 ④）
>   - 表/字段分组（按 schema / 按 tag）
>   - 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + rAF 统一调度；帧率 <16ms 按 C-2 **仅作代码审查项 + 可选基准脚本，不作为 verify 门禁断言**）
> **外环强制约束（条目 17 末段）**：
>   1. 涉及规则推导的给真值表或明确规则 + 实例推演（C-1）
>   2. 新 UT 编号先 grep 取下一空闲——**当前占用至 UT-MM-27，UT-MM-28 起可用**
>   3. tasks 不写 verify / smoke / archive 条目（独立 CLI 节点）
>   4. **列宽持久化落点明确**：落 `ListViewState` 会话态（非契约变更）还是 `Table` 数据契约（涉契约变更须标注 + 说明向后兼容）

---

## 强制 ④ 列宽持久化落点裁决（前置决议，下文代码块以本决议为准）

**结论**：列宽沿用 `Table.width: Option<u32>` 数据契约（**会话态扩展 = 不变**，**契约扩展 = 不变**）。

**决议理由**：
- **既有契约**：批次 0（feat-table-resize，commit 历史已闭环）已将 `pub width: Option<u32>` 落入 `frontend-rs/src/editor_core.rs:53`（`#[serde(default, skip_serializing_if = "Option::is_none")]`），向后兼容老 JSON（老 JSON 无此字段 → `None` → 渲染层用 `TABLE_WIDTH = 230.0` 硬编码默认）。
- **批次 4 不引入新列宽字段**——既有的 `Table.width` 已支持「整表宽度」，足以覆盖批次 4「列宽可调」需求（表整体宽度 = 字段列宽之和 + 内边距；按表缩放既影响所有字段列宽）。
- **不引入 `field.width: Option<u32>`**——超范围：批次 4 不做字段级独立列宽（避免字段级 + 表级双层缩放交互复杂度；YAGNI）。
- **持久化路径**：`Table.width` 通过既有 PUT /api/v1/diagrams 写入后端（`backend/crud` 既有路径），老 JSON 兼容（serde default = None）。
- **会话态 vs 契约态：本批仅复用既有契约，不引入新契约 → 无需标注向后兼容**。

**真值表（持久化语义）**：

| 场景 | 老 JSON (无 width) | 新建表 (默认) | 用户拖拽 resize | 拖拽 0 = auto | 老版本读取新 JSON |
|---|---|---|---|---|---|
| `Table.width` 落地值 | `None` | `None` | `Some(200..1000)` | `Some(0)` | `Some(...)` 正常反序列化 |
| 渲染层行为 | 用 `TABLE_WIDTH = 230.0` | 同左 | 用 `Some(N)` | 回退到 `TABLE_WIDTH = 230.0`（渲染层判断 `0 = auto`） | 渲染层用 `Some(N)` |
| 持久化写入 | skip_serializing_if = None → 不写字段 | 同左 | 写 `Some(N)` | 写 `Some(0)` | — |
| 契约变更 | 无 | 无 | 无 | 无 | 无 |

> **向 operator/外环的明示**：批次 4 不修改 `Table` / `Field` struct（沿用既有 `width`）；若后续要字段级列宽，则需新提案（`field.width: Option<u32>` + serde default + 渲染层优先级：字段 width > 表 width > 默认）。

---

## 范围① 列宽可调（沿用既有 Table.width，UI 层扩展）

### 真值表（拖拽交互边界）

| 输入 | 鼠标位置 | 目标宽 | 结果 | 备注 |
|---|---|---|---|---|
| 表边框左/右 | 距离边框 ≤ 6px | 整个表 | 启动水平拖拽（cursor: col-resize） | 仅右侧边框可拖（左侧受 x 坐标控制） |
| 拖拽中 | — | — | 实时更新 `Table.width`（节流到 16ms / 60fps） | 渲染层不节流——信号更新走 rAF |
| 拖拽释放 | — | — | 写入 store → `store.dirty.set(true)` → 走 OT 通路 | 后端 PUT 自动持久化 |
| 拖拽中按 Esc | — | — | 取消，恢复拖拽前 `Table.width` | 暂存 `drag_start_width` |
| 拖拽边界 | — | — | 最小 100px（防塌陷），最大 1000px（防溢出） | 见真值表 |

### 真值表（宽度数值范围 + 边界）

| 输入 | 解析 | 落 `Table.width` | 渲染层 |
|---|---|---|---|
| 拖拽到 50px | `parse_table_width("50")` → Err（< 100） | 不写入（保留旧值） | 不重渲染 |
| 拖拽到 100px | Ok(100) | `Some(100)` | `render_width = 100` |
| 拖拽到 1000px | Ok(1000) | `Some(1000)` | `render_width = 1000` |
| 拖拽到 1500px | `parse_table_width("1500")` → Err（> 1000） | 不写入（保留旧值） | 不重渲染 |
| `parse_table_width("0")` | Ok(0)（既有 UT-MM-11 语义：`0 = auto`） | `Some(0)` | 回退 `TABLE_WIDTH = 230.0`（既有路径） |
| 字段 < 100 字符总宽 | — | — | 渲染层补足至 100px 最小宽（视觉防塌陷，与既有逻辑一致） |

> 解析函数复用既有 `modals::parse_table_width`（`frontend-rs/src/editor_panels.rs:8230`，UT-MM-11 已保），不在本批新增解析逻辑。

### 实例推演（C-1 闭环）

- **happy 1**：用户拖拽 `users` 表右侧边框到 320px → `Table.width` 从 `None` 变为 `Some(320)` → 渲染层 `render_width = 320` → store.dirty → true → 下次 PUT 持久化 → 刷新页面后 `Table.width = Some(320)` → 渲染 `320`。
- **happy 2**：用户拖拽到 0 → `Some(0)` → 渲染层 `render_width = TABLE_WIDTH = 230.0`（`0 = auto` 语义，对称既有 parse_table_width UT-MM-11）。
- **edge 1**：用户拖拽到 50px（小于最小宽 100）→ `parse_table_width` 不被调用（拖拽边界钳制）→ 实际落 `Some(100)` → 渲染 100。
- **edge 2**：用户拖拽到 1500px → 钳制为 `Some(1000)` → 渲染 1000。
- **edge 3**：用户拖拽中按 Esc → `drag_start_width` 恢复 → store 不变。
- **edge 4**：老 JSON 反序列化（无 width 字段）→ `Table.width = None` → 渲染 `TABLE_WIDTH = 230.0`（向后兼容）。

### 代码实现（新增 `editor_render.rs` + `editor_panels.rs`）

- [ ] `frontend-rs/src/editor_render.rs`: 在表 hover/拖拽路径加水平拖拽检测（指针 x 距离右侧边框 ≤ 6px → cursor: col-resize → 启动拖拽）
- [ ] `frontend-rs/src/editor_render.rs`: 拖拽中实时更新 `store.tables.set(..)`（节流到 rAF 调度——见范围③ rAF 统一调度）
- [ ] `frontend-rs/src/editor_render.rs`: 拖拽边界钳制 `width ∈ [100, 1000]`；Esc 取消恢复 `drag_start_width`
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn clamp_table_width(w: u32) -> u32` 纯函数（**UT-MM-28**）：
  - 签名：`pub fn clamp_table_width(w: u32) -> u32`（返回钳制后宽度，min=100, max=1000, 0 保持 0）
  - 真值表（6 行覆盖：0/50/100/320/1000/1500）+ 实例推演（4 条覆盖 happy/edge）
  - 测试：边界值 + 0 保持语义 + 大于上限 + 小于下限

### 测试（新增 UT-MM-28）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-28 | 列宽钳制纯函数测试（min=100, max=1000, 0 保持 0） | `editor_panels.rs::clamp_table_width` |

---

## 范围② 表/字段分组（按 schema / 按 tag）

### 真值表（分组模式）

| 分组模式 | 分组键 | 渲染 | 实例 |
|---|---|---|---|
| **None**（默认） | 无 | 扁平表列表（既有 ListView 表格形态） | 当前行为 |
| **BySchema** | `table.id`（每个表一组） | 表按 id 升序分组展示（每表 = 一组，组内字段扁平） | `[users(id, name)] / [posts(id, user_id, body)]` |
| **ByTag** | `field.tag: String`（每个 tag 一组） | 按 tag 分组聚合所有表的字段（跨表聚合） | `[tag=pk] → users.id, posts.id / [tag=fk] → posts.user_id` |

> **字段 tag 字段新增契约标注**：批次 4 **新增 `Field.tag: String`（serde default = ""）**——**契约扩展**（非纯会话态）。向后兼容：老 JSON 无 `tag` 字段 → 反序列化为 `""` → `ByTag` 分组时归入 `(empty)` 兜底组。
>
> **判定依据**：批次 4 分组是核心 UI 能力，无 tag 字段则 `ByTag` 分组无可分组键；持久化用户自定义分组标签是关键 UX；serde default + 老 JSON 兼容已保。**契约变更最小化 = 仅加一个 `String` 字段**（无嵌套结构、无 Option，0 成本兼容）。
>
> **向后兼容说明**：写入路径 `modals` / `canvas` / `ListView` 三处更新 `Field` 构造点（grep `Field {` / `Field{` 找到全部构造点补 `tag: String::new()`）；读取路径 serde default 保兼容。

### 实例推演（C-1 闭环，ByTag）

- **happy 1**：用户给 `users.id` 打 tag `pk`，给 `posts.id` 打 tag `pk`，给 `posts.user_id` 打 tag `fk` → `ByTag` 分组 → `[pk] users.id, posts.id / [fk] posts.user_id / [other] users.name, posts.body`。
- **happy 2**：用户给所有字段 tag = `""`（默认）→ `ByTag` 分组 → 所有字段归入 `(empty)` 单组（兜底）。
- **edge 1**：混合场景——部分字段有 tag，部分无 → `ByTag` 分组 → 有 tag 的字段按 tag 分组 + 无 tag 的字段归入 `(empty)`。
- **edge 2**：老 JSON 无 tag 字段 → serde default `""` → 等同 happy 2。
- **edge 3**：分组键大小写敏感性——tag **大小写敏感**（`Pk` ≠ `pk`，不同组），与字段名一致性约定一致。

### 代码实现（新增 `editor_panels.rs`）

- [ ] `frontend-rs/src/editor_core.rs`: `Field` struct 加 `pub tag: String` 字段（serde default = ""，与既有 `width` 同模式）
- [ ] `frontend-rs/src/editor_core.rs`: 全部 `Field { ... }` 构造点补 `tag: String::new()`（grep `Field {` / `Field{` 确认 0 遗漏）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增分组模式 enum：
  ```rust
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub enum GroupByMode {
      #[default]
      None,
      BySchema,
      ByTag,
  }
  ```
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn group_tables(tables: &[Table], mode: GroupByMode) -> Vec<GroupBucket>` 纯函数（**UT-MM-29**）：
  - 输入：`tables: &[Table]`、`mode: GroupByMode`
  - 输出：`Vec<GroupBucket>`（按模式排序的桶数组；桶含 `key: String`、`tables: Vec<&Table>` 或 `fields: Vec<(table_id, field_id)>`）
  - 真值表（5 行覆盖：None/BySchema/ByTag/混合/空）+ 实例推演（4 条覆盖 happy/edge）
- [ ] `frontend-rs/src/editor_panels.rs`: `ListViewState` 加 `group_by: RwSignal<GroupByMode>` 字段（**会话态**，不持久化——分组偏好是用户当下的视图选择）
- [ ] `frontend-rs/src/editor_panels.rs`: ListView filters 区加分组下拉 `<select data-testid="list-view-group-by">`，on:change 写 `group_by`
- [ ] `frontend-rs/src/editor_panels.rs`: ListView 表格按 `group_tables(..)` 输出分桶渲染（每桶 header `<tr data-testid="list-view-group-{key}">` + 桶内字段/表行）
- [ ] `frontend-rs/src/editor_panels.rs`: Inspector 字段编辑面板加 tag input（`data-testid="field-tag-input"`，on:change 写 `field.tag` + store.dirty + true）

### 测试（新增 UT-MM-29）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-29 | 表/字段分组纯函数测试（None/BySchema/ByTag 三模式 + 混合 tag + 空表 + 大小写敏感） | `editor_panels.rs::group_tables` |

---

## 范围③ 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + rAF 统一调度）

### 真值表（字体探测）

| 字体 | 探测路径 | 加载状态 | 渲染选择 |
|---|---|---|---|
| Plus Jakarta Sans（primary） | `doc.fonts().check("1em \"Plus Jakarta Sans\"")` | true | primary |
| Plus Jakarta Sans | false | 探测 fallback 1：思源黑体 `Source Han Sans CN` / `Noto Sans CJK SC` | true → 用 |
| 思源黑体 | false | 探测 fallback 2：苹方 `PingFang SC` | true → 用 |
| 苹方 | false | 探测 fallback 3：`-apple-system, BlinkMacSystemFont` | true → 用 |
| 全部失败 | — | 降级既有路径 `ui-monospace` | 既有路径（不破坏） |

> **新增字体加载**：批次 4 在 `frontend-rs/index.html` 加 Google Fonts CDN 加载 `Noto+Sans+SC:wght@400;500;700&display=swap`（思源黑体/苹方无 Google Fonts；Noto Sans CJK SC 是 Google Fonts 上的等价替代）。苹方依赖 macOS 系统字体（不外加 CDN），思源黑体同理。

### 实例推演（C-1 闭环）

- **happy 1**：Chrome 桌面 + 思源黑体已加载（Google Fonts） → primary 用 Noto Sans CJK SC → 渲染清晰。
- **happy 2**：macOS Safari + 苹方系统字体可用 → fallback 2 命中 → 渲染清晰。
- **happy 3**：Linux 桌面无中文字体 → fallback 3 命中 `-apple-system`（无中文也无 CJK，回退 latin） → 用户看到 latin 字符（与既有行为一致，不破坏）。
- **edge 1**：Plus Jakarta Sans 未加载 + 思源黑体已加载 → 用思源黑体 → 渲染清晰。
- **edge 2**：所有字体均未加载 → 降级既有 `ui-monospace`（不破坏）。

### Canvas 文本离屏缓存（真值表）

| 场景 | 缓存策略 | 命中 | 失效条件 |
|---|---|---|---|
| 表头绘制 `table.name` | 按 `(table.id, table.name, font_size, dpr)` 缓存 `OffscreenCanvas` | 缓存命中 → `drawImage(offscreen, x, y)` | table.name 改 / dpr 改 / font_size 改 |
| 字段名绘制 `field.name` | 按 `(field.id, field.name, font_size, dpr)` 缓存 | 同上 | field.name 改 |
| 索引标记 `PK` / `FK` | 静态字符串 → 全局唯一 cache（无需按 id） | 始终命中 | — |
| 类型字符串 `field.type_` | 按 `(field.id, field.type_, font_size, dpr)` 缓存 | 同上 | field.type_ 改 |

> **实现路径**：`OffscreenCanvas` API + `getContext('2d')` 渲染文本 → 主画布 `drawImage` 绘制位图。**wasm-bindgen 暴露 `OffscreenCanvas`**——验证 frontend-rs Cargo.toml `wasm-bindgen` 版本 ≥ 0.2.83（OffscreenCanvas 支持版本）。

### rAF 统一调度（真值表）

| 触发源 | 当前行为 | 批次 4 后行为 |
|---|---|---|
| 拖拽表（平移） | 每帧 set_state 直接重渲染 | 走 `request_animation_frame` 调度，合并同帧多次 set |
| 缩放 Canvas（wheel） | 同上 | 同上 |
| 拖拽表 resize | 每帧 set_state 直接重渲染 | 同上 |
| hover 检测 | mousemove 触发立即重渲染 | 节流到 rAF（每帧最多一次） |
| 字段编辑（input） | on:input 立即重渲染 | on:input 写 store 信号；rAF 调度渲染 |

> **实现路径**：在 `editor_render.rs` 引入 `pub fn schedule_render(render_fn: Rc<dyn Fn()>)` 工具函数（内部维护 `pending: Cell<bool>` + `request_animation_frame`）；所有 `set_state` 后续的 `request_redraw()` 调用改走此函数。
> **不引入新 UT**——rAF 调度是浏览器异步行为，纯函数不可测；测试覆盖 `schedule_render` 的 `pending` 状态机（仅首次入队执行，二次入队 noop 直到 rAF 回调清 pending）即可。

### 代码实现

- [ ] `frontend-rs/index.html`: 加 Google Fonts `<link>` 加载 `Noto+Sans+SC:wght@400;500;700&display=swap`
- [ ] `frontend-rs/src/styles.css`: `--cdb-font-family-base` 字体回退栈补 `Source Han Sans CN`, `Noto Sans CJK SC`, `PingFang SC`, `Hiragino Sans GB`, `Microsoft YaHei`（在既有 `Plus Jakarta Sans`, `-apple-system` 之后）
- [ ] `frontend-rs/src/editor_render.rs`: `resolve_canvas_font_family` 加思源黑体/苹方 fallback 探测（按真值表顺序）
- [ ] `frontend-rs/src/editor_render.rs`: 新增文本离屏缓存模块 `pub struct TextCache`（`HashMap<CacheKey, OffscreenCanvas>` + `invalidate` 方法）
- [ ] `frontend-rs/src/editor_render.rs`: 表头/字段名/类型文本绘制改走 `TextCache::get_or_render`（按真值表）
- [ ] `frontend-rs/src/editor_render.rs`: 新增 `pub fn schedule_render(render_fn: Rc<dyn Fn()>)` 工具函数（rAF 统一调度 + pending 状态机）
- [ ] `frontend-rs/src/editor_render.rs`: 所有 `request_redraw()` 调用点改走 `schedule_render`
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn schedule_render_dedup(state: &Cell<bool>, render_fn: Rc<dyn Fn()>)` 纯函数（**UT-MM-30**：仅首次入队执行 / 二次入队 noop / rAF 回调清 pending 后再入队可执行）
- [ ] **不引入帧率 < 16ms 基准断言**——按 C-2 **仅作代码审查项 + 可选基准脚本**（不作为 verify 门禁断言）

### 测试（新增 UT-MM-30）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-30 | rAF 调度去重纯函数测试（pending 状态机：仅首次入队执行 / 二次入队 noop / 清 pending 后再入队可执行） | `editor_panels.rs::schedule_render_dedup` |

### 不在范围（C-2 落实）

- 帧率 < 16ms **不作为 verify 门禁断言**——仅作代码审查项（人工 review rAF 调度 + 离屏缓存命中）+ 可选基准脚本（`scripts/benchmark-render.rs`，非 verify 节点，**本批不写**，留待后续性能专项）
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 离屏缓存失效追踪**不写**全链路事件溯源（仅按 `field.name`/`field.type_`/`table.name` 变更触发失效——signal 自动驱动）
- wasm-bindgen 升级**不在本批**（若 `OffscreenCanvas` 不支持则降级为 `HTMLCanvasElement` + 隐藏 DOM 节点）

---

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 spec 文件）追加 UT-MM-28/29/30 行：
  ```
  | UT-MM-28 | 列宽钳制纯函数测试（min=100, max=1000, 0 保持 0；真值表 + 实例推演覆盖 happy/edge） | `editor_panels.rs::clamp_table_width` |
  | UT-MM-29 | 表/字段分组纯函数测试（None/BySchema/ByTag 三模式 + 混合 tag + 空表 + 大小写敏感） | `editor_panels.rs::group_tables` |
  | UT-MM-30 | rAF 调度去重纯函数测试（pending 状态机：仅首次入队执行 / 二次入队 noop / 清 pending 后再入队可执行） | `editor_panels.rs::schedule_render_dedup` |
  ```
- [ ] 确认 UT-MM-28/29/30 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）
- [ ] `Field.tag: String` 契约扩展文档化：在 `docs/phase2/PHASE2_VALIDATION.md` 末尾追加「批次 4 契约扩展：`Field.tag` 新增（serde default = ""，向后兼容老 JSON）」条目

## 实现顺序建议（批次 4）

1. `Field.tag` 契约扩展 + 全部 `Field { ... }` 构造点补 `tag: String::new()`（契约扩展前置——分组依赖 tag）
2. `clamp_table_width` 纯函数（真值表 + 实例推演）+ UT-MM-28
3. 表 resize UI（拖拽检测 + 边界钳制 + rAF 调度 + store.dirty）——依赖 ② clamp + ⑥ rAF
4. `group_tables` 纯函数（None/BySchema/ByTag + 真值表 + 实例推演）+ UT-MM-29——依赖 ① tag 契约
5. 表/字段分组 UI（ListView filters 分组下拉 + ListView 表格分桶渲染 + Inspector field-tag-input）
6. 样式优化三件套（字体回退栈补 Noto Sans CJK SC + 文本离屏缓存 + rAF 统一调度 + UT-MM-30）——可与 1-5 并行推进
7. spec 登记（UT-MM-28/29/30 行 + PHASE2_VALIDATION.md 契约扩展条目）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 帧率 < 16ms **不作为 verify 门禁断言**——C-2 仅作代码审查项 + 可选基准脚本（**本批不写基准脚本**）
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 字段级独立列宽 `field.width: Option<u32>`（批次 4 不做；YAGNI）
- 字体加载性能优化（FOIT/FOUT 策略调整——Q5 未点名）
- wasm-bindgen 升级（若 OffscreenCanvas 不支持则降级为隐藏 DOM 节点方案）
- 离屏缓存失效追踪全链路事件溯源（仅 signal 自动驱动）
- 导出 xlsx（C-3 已裁）
- 不修改既有测试断言（外环强制约束）
- 不写 verify / smoke / archive 条目（独立 CLI 节点）
- 批次 1/2/3 已闭环范围（表名/字段名/类型表格化展示、排序、过滤、批量重命名、批量改类型、双击跳画布、导出 CSV）——不在批次 4 范围

## 外环判词强制约束落实（条目 17）

- **强制 ① 真值表/规则 + 实例推演（C-1）**：
  - 列宽可调——拖拽交互边界真值表（5 行）+ 数值范围真值表（6 行）+ 实例推演（4 条）
  - 表/字段分组——分组模式真值表（3 行）+ ByTag 实例推演（4 条）
  - 样式优化——字体探测真值表（5 行）+ 实例推演（4 条）+ 文本离屏缓存真值表（4 行）+ rAF 调度真值表（5 行）
- **强制 ② UT 编号**：grep `UT-MM-2[7-9]\|UT-MM-3[0-9]` 已确认 UT-MM-27 占用，本批**UT-MM-28/29/30 起**——独立 commit 不抢编号
- **强制 ③ 不写 verify/smoke/archive 条目**：实现顺序 1-7 + spec 登记 1-3，无 verify/smoke/archive
- **强制 ④ 列宽持久化落点明确**：**沿用既有 `Table.width: Option<u32>` 数据契约**（feat-table-resize 已闭环，serde default + skip_serializing_if 双向兼容）；**不引入 `field.width`**；契约扩展最小化 = 仅 `Field.tag: String` 新增（serde default 兼容老 JSON，已标注向后兼容）