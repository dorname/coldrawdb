# 侧边栏规格（V1）

## 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。侧栏语义以 **Tool Rail + Inspector + 抽屉** 为准，不再以 V1 左栏 7 Tab 为产品主路径。

| 项 | 约定 |
|---|---|
| 页面流 | `auth → rooms → room-editor`；侧栏工具仅在 room-editor |
| 演示 ≠ 生产 | 协作动态抽屉、演示角色切换仅为体验；生产以房间成员 API / WS 为准 |
| 实现状态 | **后端已实现**；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| Inspector | **`data-testid="inspector"`**（禁止 `inspector-panel`） |

## 1. 侧边栏布局

V2 编辑器壳（与主原型 / IA 一致）：

| 区域 | `data-testid` | 职责 |
|---|---|---|
| Tool Rail | `tool-rail` | 建表、关系、区域、便签、命令搜索、协作动态、设置 |
| Canvas | `editor-canvas` | 对象交互；远端光标；连接 Banner |
| Inspector | `inspector` | 选中表/对象属性；可折叠 |
| 成员抽屉 | `room-members-panel` | 成员与角色 |
| 活动抽屉 | `activity-feed` | 协作动态（可选） |
| IO 抽屉 | `import-drawer` / `export-drawer` | 经更多菜单 |

**Tool Rail 主按钮（对齐主原型）**：

| 按钮 | testid | 说明 |
|---|---|---|
| 新建表 | `tool-add-table` | Viewer disabled |
| 关系 | `tool-relationship` | 进入关系工具；见 `core-01b` |
| 区域 / 便签 | — | 画布添加 |
| 搜索与命令 | `tool-search` | 打开 Command Palette |
| 协作动态 | — | `open-drawer` activity |
| 画布设置 | — | 打开设置模态 |

V1 §2～§7 各业务 Tab **仅作历史行为记录**；浏览/搜索迁至 Command Palette，属性编辑迁至 Inspector。

### 1.1 响应式

- ≤1179：Inspector 叠在画布右侧，默认宽度约 330px；可关闭。
- ≤760：Tool Rail 改底栏横排；Inspector 绝对叠层；写工具保留 `mobile-keep` 可达性策略见 `core-05`。

## 2. Tables Tab

### 2.1 列表项

每项展示：表名（可编辑）+ 字段数 + 颜色块（V1 仅前端）

### 2.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击项 | 鼠标左键 | 画布中该表高亮 + 滚动到视口 |
| 双击项 | 鼠标左键 | 进入表重命名编辑模式 |
| 右键项 | 鼠标右键 | 上下文菜单（重命名 / 复制 / 删除 / 改色 / 锁定） |
| "+" | Tab 底部 | 创建新表（焦点跳到画布） |
| 拖拽项到画布 | 拖拽 | 在 (mouse_x, mouse_y) 创建表 |

### 2.3 搜索 / 筛选

- 搜索：模糊匹配表名 + 字段名
- 筛选：按字段类型 / 主键 / 索引存在

## 3. Areas Tab

> **V1 数据源（align-v1-areas-notes-store）**：列表与增删改读写 `EditorStore.areas`（与画布 `draw_area`、PUT payload 同源）。Enums/Types 仍为仅前端 state。

### 3.1 列表项

每项：区域名（可编辑）+ 颜色块 + 表数（区域内）

### 3.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 画布中区域闪烁 |
| 双击 | 鼠标左键 | 重命名 |
| 右键 | 鼠标右键 | 上下文菜单（重命名 / 改色 / 删除） |
| "+" | Tab 底部 | 创建新区域（默认在画布中心） |

## 4. Enums Tab（V1 仅前端 state）

### 4.1 列表项

每项：枚举名 + 值数量

### 4.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 弹出枚举详情面板（值列表） |
| 双击 | 鼠标左键 | 重命名 |
| 右键 | 鼠标右键 | 上下文菜单（重命名 / 删除 / 引用检查） |
| "+" | Tab 底部 | 创建新枚举 |

> 引用检查：删除前提示"被 N 个字段引用，是否继续？"

## 5. Notes Tab

> **V1 数据源**：列表与增删改读写 `EditorStore.notes`（与画布 `draw_note`、PUT payload 同源）。

### 5.1 列表项

每项：便签首行文本（截断 30 字符）+ 位置坐标

### 5.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 画布中便签闪烁 + 滚动 |
| 双击 | 鼠标左键 | 进入便签内容编辑 |
| 右键 | 鼠标右键 | 上下文菜单（编辑 / 改色 / 删除） |
| "+" | Tab 底部 | 创建新便签（默认在画布中心） |

## 6. Relationships Tab

### 6.1 列表项

每项：起点表.起点字段 → 终点表.终点字段（带 cardinality 标签）

### 6.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 画布中关系闪烁 |
| 双击 | 鼠标左键 | 打开 `RelationshipInfo` 侧栏面板（编辑 cardinality/onUpdate/onDelete） |
| 右键 | 鼠标右键 | 上下文菜单（编辑 / 翻转 / 删除） |
| "+" | Tab 底部 | 引导"拖拽表字段到另一表字段"提示 |

## 7. Types Tab（V1 仅前端 state）

### 7.1 列表项

每项：自定义类型名 + 等价基础类型 + 子字段数

### 7.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 打开 `ConfigureCustomTypes` 模态 |
| 右键 | 鼠标右键 | 上下文菜单（编辑 / 删除 / 引用检查） |
| "+" | Tab 底部 | 等同单击 |

> 引用检查：删除前提示"被 N 个字段引用，是否继续？"

## 8. Issues Tab

### 8.1 用途

集中展示当前 diagram 的所有**校验错误**（drawdb 称为 `Issues`）。来源：
- 表名 / 字段名重复
- 表名 / 字段名非法
- 主键缺失
- 字段类型不兼容
- 关系端点不存在
- 自增字段非整数
- 等等

### 8.2 列表项

每项：
- 错误级别（❌ error / ⚠ warning / ℹ info）
- 错误消息
- 涉及对象（表名 / 字段名 / 关系名）
- "跳转到对象"按钮（画布闪烁 + 滚动）

### 8.3 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击项 | 鼠标左键 | 跳转到对象 |
| 过滤下拉 | 顶部 | 按级别筛选 |
| "全部展开" | 顶部 | 折叠/展开 |

### 8.4 校验时机

- 实时：编辑表/字段/关系时立即触发
- 加载：从后端加载 diagram 后立即触发
- 手动：顶部菜单"Validate"按钮强制重新校验

### 8.5 Issues / 校验

Issues 不以左栏 Tab 为唯一入口；可与 StatusBar / 折叠条 / 命令面板并存。主原型未单独展示 Issues Tab 时，生产实现不得倒退回强制 280px 七 Tab 左栏。

## 9. DBML Editor（V1 备选视图）

### 9.1 入口

顶部菜单"View" → "DBML"

### 9.2 布局

DBML Editor 打开时，画布 + 侧栏隐藏，全屏显示一个代码编辑器（textarea）：

```dbml
Table users {
  id INT [pk]
  name VARCHAR(255) [not null]
}
```

### 9.3 行为

- 编辑 DBML → 失焦或点"Apply" → 解析 → 更新 Diagram
- 解析错误 → 顶部显示错误消息
- 同步方向：DBML → Diagram（V1 单向；V2 计划支持双向）

## 10. 搜索 / 筛选通用

| 维度 | 能力 |
|---|---|
| 全局搜索 | 顶部搜索框，跨所有 Tab 模糊匹配（drawdb 行为） |
| 类型筛选 | 按 field type 过滤（drawdb 行为） |
| 引用图谱 | 选中某对象时显示其引用关系（drawdb 行为） |

## 11. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-SP-01 | 单击 Tables Tab 表项 → 画布高亮 + 滚动 |
| UT-SP-02 | 搜索 "user" → 列表过滤只含 user* |
| UT-SP-03 | 双击枚举名 → 进入重命名 |
| UT-SP-04 | 引用检查：删除被引用的枚举 → 弹确认 |
| UT-SP-05 | Issues Tab：表名重复 → 错误项出现 |
| UT-SP-06 | Issues Tab：单击错误 → 跳转 + 画布闪烁 |
| UT-SP-07 | DBML Editor：编辑后 Apply → 解析成功 → Diagram 更新 |
| UT-SP-08 | DBML Editor：编辑非法 DBML → 错误消息 + 不应用 |
| ST-SP-01 | 端到端：编辑 5 表 → Issues Tab 显示 0 error |
| UT-SP-09 | 6 业务 Tab 切换（点击 Tab A→B→C，验证激活态 + 内容区切换）— B2 范围 |
| UT-SP-10 | 全局搜索跨 Tab 过滤（spec §10，搜索框过滤 Tables/Areas/Enums 等多 Tab 列表）— B2 范围 |

## 12. V1 边界

- ❌ DBML ↔ Diagram 双向同步（V1 仅 DBML → Diagram）
- ❌ 自定义 Issues 规则（V1 硬编码 drawdb 内置校验集）
- ❌ Tab 自定义排序（V1 固定 Tab 顺序）
- ❌ Tab 拖拽收纳（V1 全部展开）

### 12.1 Viewer 只读

- Tool Rail 写按钮 disabled。
- Inspector 输入与删除 disabled。
- 仍可打开成员抽屉查看（不可改他人角色，除非 Owner 管理规则另述）。

## 13. 对齐参考源

- drawdb `src/components/EditorSidePanel/`
- drawdb `src/components/EditorSidePanel/TablesTab/`
- drawdb `src/components/EditorSidePanel/AreasTab/`
- drawdb `src/components/EditorSidePanel/EnumsTab/`
- drawdb `src/components/EditorSidePanel/NotesTab/`
- drawdb `src/components/EditorSidePanel/RelationshipsTab/`
- drawdb `src/components/EditorSidePanel/TypesTab/`
- drawdb `src/components/EditorSidePanel/IssuesTab/`
- drawdb `src/components/DBMLEditor/`
- coldrawdb `frontend-rs/src/editor_panels.rs`
- `docs/drawdb-capability-checklist.md` §2.4

---
# Delta — core-04-side-panel-tabs.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3）

## 1 侧边栏布局（Phase A 废弃说明 + Tool Rail 图标 E2 替换）

**merge 时替换** §1 段，更新为：

### §1 侧边栏布局（V2 — Tool Rail + Issues Collapse）

V1 的 280px 左栏 7 Tab 已在 **Phase A** 中废弃（`redesign-phase-a-layout` 已合并）。V2 改为：

- **48px Tool Rail**（左侧图标轨）：5 个核心动作按钮 + Issues 徽章
- **Issues 折叠面板**（位于 AppBar 下方的全宽条带）：仅在 issue 计数 > 0 时展开
- **浏览能力**：通过 E4 Command Palette（`Ctrl+K`）恢复，不在侧栏

```
+--48px--+ +--------------------------------+  +--可变--+
|        | |                                |  |        |
| Tool   | |       EditorCanvas             |  |Inspector|
| Rail   | |                                |  | (L3)   |
| (L2.5) | |                                |  |        |
|        | +--------------------------------+  |        |
| [+Tbl] | | [Issues 5]  ▼ Collapse  (E3 组件)  |        |
| [+Area]| +--------------------------------+  |        |
| [+Note]|                                  |        |
| [ Rel] |                                  |        |
| [ Pan] |                                  |        |
| (5 btns)|                                  |        |
+--------+                                  +--------+
```

**Tool Rail 按钮清单**（E2 图标替换原 emoji）：

| 按钮 | 图标（E2） | 提示 | data-testid |
|---|---|---|---|
| 新建表 | `<IconAddTable />` | "新建表 (T)" | `cdb-tool-rail-add-table` |
| 新建区域 | `<IconAddArea />` | "新建区域" | `cdb-tool-rail-add-area` |
| 新建便签 | `<IconAddNote />` | "新建便签" | `cdb-tool-rail-add-note` |
| 关系工具 | `<IconRelationship />` | "关系工具" | `cdb-tool-rail-relationship` |
| 平移 | `<IconPan />` | "平移画布" | `cdb-tool-rail-pan` |

**Issues 折叠面板**（E3 Collapse 组件）：

| 项 | 规格 |
|---|---|
| 容器 | `<Collapse lazyRender keepDOM={false} bordered={Default}>` |
| Header | `<Tag color=Warning><IconWarning />{count}</Tag>` + 标题"问题"（来自 main `Issues.jsx`） |
| 折叠状态 | issue 计数 = 0 时默认折叠；> 0 时展开 |
| 动画 | 收起/展开 `--cdb-duration-base` |
| 内容 | 最多 160px 高，超出滚动 |

**z-index**：Tool Rail `--cdb-z-side-rail`（L2.5），Issues Collapse 跟随 AppBar 层（L2）

## 2 Tables Tab（Phase A 标记废弃）

**merge 时在 §2 顶部插入废弃说明**：

> ⚠️ **V1 280px 左栏 7 Tab 已在 Phase A 废弃**。Tables Tab 的浏览/搜索能力迁移至：
> - **快捷跳转**：`Ctrl+K` 调出 Command Palette（E4）
> - **创建入口**：Tool Rail `<IconAddTable />`（§1）
> - **属性编辑**：画布选中表 → Inspector 抽屉（`core-01-editor-canvas.md` §6）
>
> 本节 §2 内容**仅作 V1 行为记录**，不构成 V2 规范。V2 行为以 `core-01-editor-canvas.md` 与 `core-09-core-components.md` §8 Collapse 为准。

## 3–7 Areas / Enums / Notes / Relationships / Types Tab（同上废弃说明）

**merge 时在 §3、§4、§5、§6、§7 顶部各插入相同的废弃说明**（与 §2 相同，列表项指向对应 Tool Rail 按钮）。

## 8 Issues Tab（升级为 E3 Collapse）

**merge 时替换** §8 Issues 段，更新为：

### §8 Issues 折叠面板（E3 Collapse）

V1 Issues 是 7 Tab 之一。V2 升级为 AppBar 下方全宽折叠条带，由 E3 `<Collapse>` 组件承载。

**Props**：
```rust
<Collapse
  lazy_render={true}
  keep_dom={false}
  bordered={CollapseBordered::Default}
>
  <CollapsePanel
    header=view! {
      <Tag color=TagColor::Warning size=TagSize::Small>
        <IconWarning />
        {move || count.get()}
      </Tag>
      <span class="cdb-ms-2">"问题"</span>
    }
    item_key="issues"
  >
    {move || issues_list.get().into_iter().map(|i| view! { <div class="cdb-py-2">{i}</div> }).collect_view()}
  </CollapsePanel>
</Collapse>
```

**行为**（对齐 main `Issues.jsx`）：
- issue 计数 = 0 → 默认折叠 + Tag 不显示
- issue 计数 > 0 → 默认展开 + Tag 显示计数（`overflowCount=99`）
- 严格模式（`settings.strictMode=true`）→ 显示"严格模式开启，无问题"占位

**视觉**：
- header 高度 40px，hover `--cdb-color-grey-1`
- 列表项 `--cdb-font-size-sm`，`color: var(--cdb-color-text-1)`
- Tag `color=Warning` 背景 `--cdb-color-warning-soft`
