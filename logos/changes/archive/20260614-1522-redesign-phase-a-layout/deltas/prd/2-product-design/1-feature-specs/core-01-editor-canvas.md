# Delta — core-01-editor-canvas.md
# 模块：core | 提案：redesign-phase-a-layout
# 路径：`logos/changes/redesign-phase-a-layout/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
# 对齐参考源：UX 重规划 Phase A + `core-00` §1 V2 布局

## MODIFIED — §5.2.3 布局栅格

> **替换** 主文档 `core-01-editor-canvas.md` §5.2.3「布局栅格」小节。

#### 5.2.3 布局栅格（V2 / Phase A）

- App 顶层：`display: grid; grid-template-rows: 48px 1fr 28px;`（AppBar / 主体 / StatusBar）
- 主体：`display: grid; grid-template-columns: 48px 1fr auto;`（Tool Rail / 画布 / Inspector）
- Inspector 折叠：`grid-template-columns: 48px 1fr 0;`，Inspector 面板 `visibility: hidden` 或 `width: 0`
- 最小宽度 1024px，< 1024px 提示「请使用更大屏幕」（V1 不做响应式）
- **禁止**恢复 V1 的 `240px 1fr 320px` 三栏或 `auto auto 1fr auto` 双顶栏

## MODIFIED — §6 与侧栏 / 顶部菜单的联动

> **替换** 主文档 `core-01-editor-canvas.md` §6 整节。

## 6. 选中态统一与面板联动（V2 / Phase A）

### 6.1 选中对象模型

全局唯一 `SelectionState`（`editor_core` 持有），语义：

```ts
type SelectionState =
  | { kind: "none" }
  | { kind: "table"; id: string }
  | { kind: "field"; tableId: string; fieldId: string }
  | { kind: "reference"; id: string }
  | { kind: "area"; id: string }
  | { kind: "note"; id: string }
  | { kind: "multi"; ids: string[] }  // Phase A 仅支持多选表，V2 扩展
```

**同步规则**（三处一致）：

| 触发源 | 效果 |
|--------|------|
| 画布单击表 | `SelectionState = table`；Inspector 展开并显示表属性 |
| 画布单击字段行 | `SelectionState = field`；Inspector 展开并显示字段表单 |
| 画布单击关系线 | `SelectionState = reference`；Inspector 展开并显示关系属性 |
| 画布单击空白 | `SelectionState = none`；Inspector **不自动折叠**（用户手动折叠） |
| 双击空白 | `SelectionState = none`；Inspector 折叠 |
| StatusBar「Inspector」按钮 | 切换 Inspector 展开/折叠（不改变 SelectionState） |

### 6.2 Inspector 联动

- 选中表 → Inspector 标题显示表名；内容区：表名、注释、颜色、字段列表摘要、索引区入口
- 选中字段 → Inspector 标题 `字段：{name}`；内容区：完整字段属性表单（对齐 `core-01a` §2）
- 选中关系 → Inspector 标题 `关系：{start}.{field} → {end}.{field}`；内容区：cardinality / onUpdate / onDelete
- 无选中 + Inspector 展开 → 显示**项目概览**（表数、关系数、最近修改时间）

### 6.3 AppBar 联动

- 撤销/重做：作用于任意画布修改（与 V1 一致）
- 保存状态指示器：读取 `store.dirty` + 最近保存时间（与 V1 一致）
- 导入/导出按钮：Phase A 仅 UI 占位（disabled + tooltip「即将推出」）；Phase C 接入抽屉

### 6.4 Tool Rail 联动

- 当前激活工具（选择 / 关系 / 平移）高亮 `cdb-is-active`
- Issues 徽章数字 = `IssuesValidator` 错误 + 警告计数（与 V1 Issues Tab 同源）
- 点击 Issues 徽章：Phase A 在 Inspector 展开并切换至「Issues」子视图（替代 V1 Issues Tab）

### 6.5 移除的 V1 联动

- ~~单击左栏 Tables Tab 表项 → 画布高亮~~（左栏列表 Phase A 移除；Phase D 由 Command Palette 承接）
- ~~单击 Relationships Tab → 关系闪烁~~（同上）

Phase A 保留 programmatic API：`jump_to_table(id)` / `jump_to_reference(id)` 供 Issues 面板「定位」按钮调用。

## ADDED — §5.7 空白画布引导（Phase A）

> merge 时在主文档 §5.6 验收要点之后、§6 之前插入本节（原 §6 已被 MODIFIED 替换，注意章节 renumber 由 merge 工具处理）。

### 5.7 空白画布引导（Empty State Guide）

**触发条件**：`store.tables.get().is_empty()` 且当前为 Canvas 视图。

**渲染**：画布视口居中叠加半透明引导卡片（不阻挡 Tool Rail 与 AppBar 点击）。

```
┌─────────────────────────────────┐
│      开始设计你的数据库           │
│                                 │
│   [ + 创建第一张表 ]            │
│   [ ↑ 导入 SQL ]（Phase C 启用） │
└─────────────────────────────────┘
```

**交互**：

| 操作 | 效果 |
|------|------|
| 点击「创建第一张表」 | 等价 Tool Rail `⊕ 表`：在视口中心创建 `Table_1`，自动选中，Inspector 展开并聚焦表名 |
| 点击「导入 SQL」 | Phase A：toast「导入功能即将推出」；Phase C：打开导入抽屉 |
| 创建第一张表后 | 引导卡片从 DOM 移除（`tables.len() > 0`） |
| 删除全部表后 | 引导卡片重新显示 |

**样式约束**：

- 容器：`data-testid="canvas-empty-guide"`
- class：`cdb-empty-guide`（居中 flex，`pointer-events: auto`）
- 背景：`var(--cdb-color-bg)` + `var(--cdb-shadow-lg)` + `border-radius: var(--cdb-radius-lg)`
- 不遮挡 FloatingControls（z-index L2，低于 Inspector L3）

**默认首表**：

- 表名：`Table_1`（创建后 Inspector 表名输入框全选，便于立即重命名）
- 默认字段：`id` / `INT` / `primary=true` / `increment=true`（用户可一键删除）

## ADDED — §11 Phase A 测试 ID 索引

> merge 时在主文档 §10 对齐参考源之后追加。

| TC ID | 描述 | 对齐实现 |
|-------|------|----------|
| UT-PA-01 | 零表时渲染 `canvas-empty-guide` | `editor_render` 或 `editor_panels` |
| UT-PA-02 | 点击引导「创建第一张表」→ `tables.len()==1` + 选中 + Inspector 展开 | `AppRoot` |
| UT-PA-03 | `SelectionState` 画布选表 → Inspector 标题含表名 | `editor_core` + `editor_panels` |
| UT-PA-04 | 画布选字段 → Inspector 显示字段表单 | `Inspector` |
| UT-PA-05 | 双击空白 → Inspector 折叠 | `AppRoot` |
| UT-PA-06 | `cdb-main` 栅格为 `48px 1fr auto`（grep/styles 断言） | `styles.css` |
| ST-PA-01 | e2e：空白进入 → 引导 → 建表 → 引导消失 | Playwright |

> 详细步骤见 `logos/resources/test/core-PA-layout-test-cases.md`（与本 delta 同步新增）。
