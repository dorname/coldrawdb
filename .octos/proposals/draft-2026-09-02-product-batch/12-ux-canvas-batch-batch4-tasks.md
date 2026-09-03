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

**结论**：**列宽落 `ListViewState` 会话态**（**非契约变更**）。

**决议理由**：
- **Q1「列宽可调」范围澄清**：「列宽可调」是**列表视图**的列（表名/字段名/类型列）宽度可调，**不是**画布表宽拖拽 resize（后者是需求 4 领域，feat-table-resize 已用 `SetTableSizeModal` 交付，画布拖拽增强属另一提案——本批**整体移出**）。
- **会话态 vs 契约态**：`ListViewState` 加列宽字段（会话态），不修改 `Table` / `Field` struct；不写后端；用户切换会话/刷新页面后列宽回归默认值（**约定**：列表视图偏好 = 会话态，不持久化）。
- **不引入 `field.width`** / `table.column_width` 等契约字段（YAGNI；批次 4 仅做 ListView 表格列宽会话态可调）。
- **ListViewState 当前字段**（既有）：`sort_column` / `sort_direction` / `filter_query` / `filter_type` / `filter_has_index` / `batch_type_selection`（:5326 周边）—— 列宽字段同模式追加（`column_widths: RwSignal<ColumnWidths>`）。

**真值表（持久化语义）**：

| 场景 | 启动 | 用户拖拽列头边界 | 用户双击列头边界 | 关闭编辑器重启 |
|---|---|---|---|---|
| `ListViewState.column_widths` 落地值 | 默认（每列 100~280px） | 更新列宽 | 当前列自适应（按内容最长字段字符宽度） | 重置为默认（**会话态不持久化**） |
| 持久化写入 | 不写 | 不写 | 不写 | — |
| 契约变更 | 无 | 无 | 无 | 无 |

> **向 operator/外环的明示**：批次 4 不修改 `Table` / `Field` struct；列宽是 ListView 视图偏好（与 `sort_column` / `filter_query` 同模式——会话态、不写后端、刷新页面重置）。

---

## 范围① 列宽可调（ListView 表格列宽，会话态）

### 真值表（拖拽交互边界——列头右侧边界）

| 输入 | 鼠标位置 | 目标列 | 结果 | 备注 |
|---|---|---|---|---|
| 列头右侧边界 | 距离列头右边框 ≤ 6px | 该列 | 启动水平拖拽（cursor: col-resize） | 仅 ListView 表格列头可拖（不涉及画布） |
| 拖拽中 | — | — | 实时更新 `ListViewState.column_widths[col]`（节流到 16ms / 60fps） | 渲染层不节流——信号更新走 rAF |
| 拖拽释放 | — | — | 写入 ListViewState（会话态，不写后端） | 用户刷新页面后重置为默认 |
| 拖拽中按 Esc | — | — | 取消，恢复拖拽前 `column_widths[col]` | 暂存 `drag_start_col_width` |
| 拖拽边界 | — | — | 最小 60px（防塌陷），最大 480px（防溢出） | 见真值表 |

### 真值表（列宽数值范围 + 边界）

| 输入 | 解析 | 落 `ListViewState.column_widths[col]` | 渲染层 |
|---|---|---|---|
| 拖拽到 30px | `clamp_column_width(30)` → 60（< 60） | `Some(60)` | `render_width = 60` |
| 拖拽到 60px | `clamp_column_width(60)` → 60 | `Some(60)` | `render_width = 60` |
| 拖拽到 200px | `clamp_column_width(200)` → 200 | `Some(200)` | `render_width = 200` |
| 拖拽到 480px | `clamp_column_width(480)` → 480 | `Some(480)` | `render_width = 480` |
| 拖拽到 600px | `clamp_column_width(600)` → 480（> 480） | `Some(480)` | `render_width = 480` |
| 字段名 `LongFieldNameHere` 字符总宽 150px | 双击列头边界 | `auto_calc(150)` → 200（含 padding） | `render_width = 200` |

> 范围由本批自定义（与画布表宽 100~1000 不同——列表视图列宽需更紧凑）；解析函数复用本批新增 `clamp_column_width`（UT-MM-28 已保）。

### 实例推演（C-1 闭环）

- **happy 1**：用户拖拽 `field_name` 列头右侧边界到 220px → `ListViewState.column_widths["field_name"]` 从 `Some(120)` 变为 `Some(220)` → 表格列宽刷新 → 用户切换会话/刷新页面后回归 `Some(120)`（会话态）。
- **happy 2**：用户双击 `field_type` 列头右侧边界 → `auto_calc` 计算 `field.type_` 列最长字段（如 `DECIMAL(10,2)` 字符总宽 ≈ 100px） → 返回 `Some(140)`（含 40px padding）→ 表格列宽自适应。
- **edge 1**：用户拖拽到 30px（小于最小宽 60）→ 钳制为 `Some(60)` → 渲染 60。
- **edge 2**：用户拖拽到 600px（大于最大宽 480）→ 钳制为 `Some(480)` → 渲染 480。
- **edge 3**：用户拖拽中按 Esc → `drag_start_col_width` 恢复 → ListViewState 不变。
- **edge 4**：用户关闭编辑器并重启 → `ListViewState.column_widths` 重置为默认（每列 `Some(120)`，与既有 ListView 表格默认列宽对齐）→ 渲染 120。

### 代码实现（新增 `editor_panels.rs`）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `ColumnWidths` 结构（`HashMap<String, u32>`，键 = 列名 `table_name` / `field_name` / `field_count` / `first_field_type` / `has_index`）+ `ColumnWidths::default()`（每列 `Some(120)`）
- [ ] `frontend-rs/src/editor_panels.rs`: `ListViewState` 加 `column_widths: RwSignal<ColumnWidths>` 字段（**会话态**）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn clamp_column_width(w: u32) -> u32` 纯函数（**UT-MM-28**）：
  - 签名：`pub fn clamp_column_width(w: u32) -> u32`（返回钳制后列宽，min=60, max=480）
  - 真值表（6 行覆盖：30/60/200/480/600/150）+ 实例推演（4 条覆盖 happy/edge）
  - 测试：边界值 + 小于下限 + 大于上限 + 等于边界
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn auto_calc_column_width(max_field_chars: u32) -> u32` 纯函数（**UT-MM-28 同 UT 名追加子用例**）：
  - 签名：`pub fn auto_calc_column_width(max_field_chars: u32) -> u32`（返回自适应列宽，公式 `max(60, min(480, max_field_chars × 8 + 40))`，8 px/字符近似 + 40 px padding）
  - 测试：0 字符 → 60；100 字符 → 480；300 字符 → 480（钳制上限）；30 字符 → 280（钳制下限）
- [ ] `frontend-rs/src/editor_panels.rs`: ListView `<th>` 加列头右侧边界拖拽（≤6px → cursor: col-resize → 启动拖拽）；双击列头右侧边界触发自适应
- [ ] `frontend-rs/src/editor_panels.rs`: ListView `<td>` 列宽渲染读 `ListViewState.column_widths[col]`（无值时 fallback `Some(120)`）
- [ ] **不修改画布渲染**：画布表宽 `Table.width` 既有路径（`SetTableSizeModal` + feat-table-resize）不变

### 测试（新增 UT-MM-28）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-28 | ListView 列宽钳制 + 自适应纯函数测试（min=60, max=480；auto_calc 公式 = max(60, min(480, chars × 8 + 40))；真值表 + 实例推演覆盖 happy/edge/0 字符/超长字符） | `editor_panels.rs::clamp_column_width` + `auto_calc_column_width` |

---

## 范围② 表/字段分组（按 schema / 按 tag）

### 真值表（分组模式）

| 分组模式 | 分组键 | 渲染 | 实例 |
|---|---|---|---|
| **None**（默认） | 无 | 扁平表列表（既有 ListView 表格形态） | 当前行为 |
| **ByTag** | `field.tag: String`（每个 tag 一组） | 按 tag 分组聚合所有表的字段（跨表聚合） | `[tag=pk] → users.id, posts.id / [tag=fk] → posts.user_id` |

> **BySchema 裁撤说明**：Q1「按 schema」在 Table struct 中**无 schema 字段**（亲测 `frontend-rs/src/editor_core.rs:43` Table 定义无 `schema` 字段）。草案 v1 写「每个表一组 = 一组一行 = 没分组」语义空洞。**外环代决**：BySchema 裁撤；如需按 Area 分组另立案（`AreaTab` 已有 Area 概念，Area 内表聚合 = 另一提案）。
>
> **Q1 注释落 12 号文件**：Q1 全 9 项列表视图能力中，**「按 schema 分组」实际无法实现**（Table 无 schema 字段）；其余 8 项均已闭环或在本批实现。本批仅实现 None/ByTag 两模式。

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
- **edge 4**（None 模式）：`GroupByMode::None` → `group_tables` 返回**单桶 `Bucket { key: "_flat", fields: [] }`**（输出形状与 ByTag 统一——消除二义，桶内 `fields` 由 ListView 直接展开所有表的字段）→ ListView 渲染扁平表格（既有行为）。

### 代码实现（新增 `editor_panels.rs`）

- [ ] `frontend-rs/src/editor_core.rs`: `Field` struct 加 `pub tag: String` 字段（serde default = ""，与既有 `width` 同模式）
- [ ] `frontend-rs/src/editor_core.rs`: 全部 `Field { ... }` 构造点补 `tag: String::new()`（grep `Field {` / `Field{` 确认 0 遗漏）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增分组模式 enum（**BySchema 裁撤**，仅 None/ByTag 两模式）：
  ```rust
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub enum GroupByMode {
      #[default]
      None,
      ByTag,
  }
  ```
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub struct Bucket { pub key: String, pub fields: Vec<(table_id: String, field_id: String)> }`（**统一输出形状**——消除 v1 二义留白）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn group_tables(tables: &[Table], mode: GroupByMode) -> Vec<Bucket>` 纯函数（**UT-MM-29**）：
  - 输入：`tables: &[Table]`、`mode: GroupByMode`
  - 输出：`Vec<Bucket>`（统一形状：每桶 `key` + `fields` 列表）
  - `GroupByMode::None` → 返回 `[Bucket { key: "_flat", fields: [所有表的字段 (table_id, field_id)] }]`（扁平直通）
  - `GroupByMode::ByTag` → 按 `field.tag` 分桶（空 tag → `(empty)` 兜底），桶按 key 字典序排序
  - 真值表（5 行覆盖：None/ByTag/混合 tag/空表/单字段多 tag）+ 实例推演（5 条覆盖 happy/edge）
- [ ] `frontend-rs/src/editor_panels.rs`: `ListViewState` 加 `group_by: RwSignal<GroupByMode>` 字段（**会话态**，不持久化——分组偏好是用户当下的视图选择）
- [ ] `frontend-rs/src/editor_panels.rs`: ListView filters 区加分组下拉 `<select data-testid="list-view-group-by">`，on:change 写 `group_by`
- [ ] `frontend-rs/src/editor_panels.rs`: ListView 表格按 `group_tables(..)` 输出分桶渲染（每桶 header `<tr data-testid="list-view-group-{key}">` + 桶内字段行）
- [ ] `frontend-rs/src/editor_panels.rs`: Inspector 字段编辑面板加 tag input（`data-testid="field-tag-input"`，on:change 写 `field.tag` + store.dirty + true）

### 测试（新增 UT-MM-29）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-29 | 表/字段分组纯函数测试（None/ByTag 两模式 + 混合 tag + 空表 + 单字段多 tag + 大小写敏感；统一输出形状 `Vec<Bucket>`） | `editor_panels.rs::group_tables` |

---

## 范围③ 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + rAF 统一调度）

### 真值表（字体探测——探测名与加载名严格一致）

| 字体 | 探测路径 | 加载状态 | 渲染选择 |
|---|---|---|---|
| Plus Jakarta Sans（primary） | `doc.fonts().check("1em \"Plus Jakarta Sans\"")` | true | primary |
| Plus Jakarta Sans | false | 探测 fallback 1：Noto Sans SC（**探测名 = 加载名** = `"Noto Sans SC"`，Google Fonts CDN `Noto+Sans+SC:wght@400;500;700`） | true → 用 |
| Noto Sans SC | false | 探测 fallback 2：苹方 `PingFang SC`（macOS 系统字体，探测名 = 字体名） | true → 用 |
| 苹方 | false | 探测 fallback 3：`-apple-system, BlinkMacSystemFont` | true → 用 |
| 全部失败 | — | 降级既有路径 `ui-monospace` | 既有路径（不破坏） |

> **新增字体加载**：批次 4 在 `frontend-rs/index.html` 加 Google Fonts CDN 加载 `Noto+Sans+SC:wght@400;500;700&display=swap`（CSS 家族名 `"Noto Sans SC"`）。**字体探测字符串必须与加载的 CSS 家族名严格一致**（v1 错误：`fonts.check("Source Han Sans CN")` 与 Google Fonts 加载的 `"Noto Sans SC"` 不匹配——`fonts.check` 必 false，fallback 链第一环永远失效）。v2 探测名修正为 `"Noto Sans SC"`（与加载名 1:1 对齐）。苹方依赖 macOS 系统字体（不外加 CDN），苹方之外的 CJK 字体回退依赖 `-apple-system` 系列。

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
- [ ] `frontend-rs/src/editor_render.rs`: 新增 `pub fn schedule_render_dedup(state: &Cell<bool>, render_fn: Rc<dyn Fn()>)` 纯函数（**UT-MM-30**：仅首次入队执行 / 二次入队 noop / rAF 回调清 pending 后再入队可执行）——**v2 合并落点**：v1 草案 `editor_panels.rs::schedule_render` 与 `editor_render.rs::schedule_render_dedup` 同机制重复定义；v2 统一合并落 `editor_render.rs`，panels 侧引用（不重复定义）
- [ ] **不引入帧率 < 16ms 基准断言**——按 C-2 **仅作代码审查项 + 可选基准脚本**（不作为 verify 门禁断言）

### 测试（新增 UT-MM-30）

| UT | 描述 | 目标 |
|---|---|---|
| UT-MM-30 | rAF 调度去重纯函数测试（pending 状态机：仅首次入队执行 / 二次入队 noop / 清 pending 后再入队可执行） | `editor_render.rs::schedule_render_dedup` |

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
  | UT-MM-30 | rAF 调度去重纯函数测试（pending 状态机：仅首次入队执行 / 二次入队 noop / 清 pending 后再入队可执行） | `editor_render.rs::schedule_render_dedup` |
  ```
- [ ] 确认 UT-MM-28/29/30 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）
- [ ] `Field.tag: String` 契约扩展文档化：在 `docs/phase2/PHASE2_VALIDATION.md` 末尾追加「批次 4 契约扩展：`Field.tag` 新增（serde default = ""，向后兼容老 JSON）」条目

## 实现顺序建议（批次 4 v2）

1. `Field.tag` 契约扩展 + 全部 `Field { ... }` 构造点补 `tag: String::new()`（契约扩展前置——分组依赖 tag）
2. `clamp_column_width` + `auto_calc_column_width` 纯函数（**v2 范围重定义**：ListView 列宽会话态，含钳制 + 自适应两个纯函数）+ UT-MM-28
3. ListView 列宽可调 UI（`<th>` 列头右侧边界拖拽 + 双击自适应 + `ListViewState.column_widths` 会话态）——**不涉及画布**；依赖 ② clamp + auto_calc
4. `group_tables` 纯函数（**v2 收敛**：None/ByTag 两模式 + 统一输出形状 `Vec<Bucket>` + 真值表 + 实例推演）+ UT-MM-29——依赖 ① tag 契约
5. 表/字段分组 UI（ListView filters 分组下拉 `<select>` + ListView 表格分桶渲染 + Inspector `field-tag-input`）
6. 样式优化三件套（字体回退栈补 **Noto Sans SC（探测名 = 加载名）** + 文本离屏缓存 + rAF 统一调度 `editor_render.rs::schedule_render_dedup` + UT-MM-30）——可与 1-5 并行推进
7. spec 登记（UT-MM-28/29/30 行 + PHASE2_VALIDATION.md 契约扩展条目）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 帧率 < 16ms **不作为 verify 门禁断言**——C-2 仅作代码审查项 + 可选基准脚本（**本批不写基准脚本**）
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- **画布表宽拖拽 resize**（feat-table-resize 已用 `SetTableSizeModal` 交付，画布拖拽增强属另一提案；v2 范围①重定义为 ListView 列宽会话态）
- **BySchema 分组模式**（Table 无 schema 字段；v2 裁撤）
- 字段级独立列宽 `field.width: Option<u32>`（批次 4 不做；YAGNI）
- 按 Area 分组（Area 概念已存在 `AreaTab`，但 Area 内表聚合是另一提案）
- 字体加载性能优化（FOIT/FOUT 策略调整——Q5 未点名）
- wasm-bindgen 升级（若 OffscreenCanvas 不支持则降级为隐藏 DOM 节点方案）
- 离屏缓存失效追踪全链路事件溯源（仅 signal 自动驱动）
- 导出 xlsx（C-3 已裁）
- 不修改既有测试断言（外环强制约束）
- 不写 verify / smoke / archive 条目（独立 CLI 节点）
- 批次 1/2/3 已闭环范围（表名/字段名/类型表格化展示、排序、过滤、批量重命名、批量改类型、双击跳画布、导出 CSV）——不在批次 4 范围

## 外环判词强制约束落实（条目 17 + 条目 18 v2 修订）

- **强制 ① 真值表/规则 + 实例推演（C-1）**：
  - 列宽可调（**v2 范围重定义**：ListView 表格列宽会话态，非画布拖拽）——拖拽交互边界真值表（5 行）+ 数值范围真值表（6 行）+ 实例推演（4 条）
  - 表/字段分组（**v2 收敛**：仅 None/ByTag 两模式，BySchema 裁撤）——分组模式真值表（2 行）+ ByTag 实例推演（5 条含 edge 4 None 模式）
  - 样式优化（**v2 字体探测名修正**：`"Noto Sans SC"` 与加载名一致）——字体探测真值表（5 行）+ 实例推演（4 条）+ 文本离屏缓存真值表（4 行）+ rAF 调度真值表（5 行）
- **强制 ② UT 编号**：grep `UT-MM-2[7-9]|UT-MM-3[0-9]` 已确认 UT-MM-27 占用，本批**UT-MM-28/29/30 起**——独立 commit 不抢编号（v2 UT-MM-28 范围改为 ListView 列宽钳制 + 自适应；UT-MM-30 落点改为 `editor_render.rs`）
- **强制 ③ 不写 verify/smoke/archive 条目**：实现顺序 1-7 + spec 登记 1-3，无 verify/smoke/archive
- **强制 ④ 列宽持久化落点明确**（**v2 重定义**）：**ListView 表格列宽落 `ListViewState` 会话态**（非契约变更，不修改 `Table`/`Field` struct，不写后端，刷新页面重置）；画布表宽拖拽 resize **整体移出本批**（feat-table-resize 已交付 `SetTableSizeModal`，画布拖拽增强属另一提案）；契约扩展最小化 = 仅 `Field.tag: String` 新增（serde default 兼容老 JSON，已标注向后兼容）

## 外环判词强制约束落实（条目 18 v2 三项定点修正）

- **P1 — 范围①误读 Q1「列宽可调」**：v2 替换为 ListView 表格列宽会话态（`ListViewState.column_widths`），`clamp_table_width` 改 `clamp_column_width`（min=60, max=480），新增 `auto_calc_column_width` 自适应纯函数；画布拖拽整体移出本批（如需另立案）
- **P2 — BySchema 伪分组裁撤**：`GroupByMode` 收敛 `None/ByTag` 两模式；`group_tables` 输出形状统一为 `Vec<Bucket { key, fields: Vec<(table_id, field_id)> }>`（None 模式 = 单桶 `_flat`，桶内含所有表的字段——消除 v1 二义留白）；Q1「按 schema」在 spec + 12 号文件标注：「Table 无 schema 字段，经外环裁决裁撤；如需按 Area 分组另立案」
- **P3 — 字体探测名与加载名必不匹配（事实错误）**：v2 字体探测名修正为 `"Noto Sans SC"`（与 Google Fonts CDN 加载名 `Noto+Sans+SC` CSS 家族名严格一致）；苹方 `PingFang SC` 系统字体探测不变

**记一笔回应（条目 18）**：rAF 函数合并——`schedule_render`（v1 拟落 `editor_render.rs`）与 `schedule_render_dedup`（v1 拟落 `editor_panels.rs`）同机制重复定义，v2 统一合并落 `editor_render.rs::schedule_render_dedup`（panels 侧引用，不重复定义）。