## ADDED — 顶部菜单 + 模态规格

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
> 对齐参考源：drawdb §2.2 顶部菜单 + 9 个模态（New/Open/Import/ImportSource/Language/SetTableWidth/Share/Rename/ConfigureCustomTypes）

# 顶部菜单 + 模态规格（V1）

## 1. 顶部菜单布局

```
+------------------------------------------------------------+
| [Logo] [File▼] [Edit▼] [View▼] [Help▼]      [SaveState] [⚙]|
+------------------------------------------------------------+
| [↶][↷] [Title editor]    [revision: 5]  [Share] [Export▼] |
+------------------------------------------------------------+
```

- **Logo**：左上角 drawdb → coldrawdb 重命名（V1）
- **菜单**：4 个下拉（File / Edit / View / Help）
- **工具栏**：撤销 / 重做 / 标题编辑 / revision 状态 / Share / Export
- **右侧**：SaveState 指示器 + 设置图标

## 2. 菜单项

### 2.1 File

| 项 | 快捷键 | 行为 |
|---|---|---|
| New | Ctrl/Cmd + N | 打开 New 模态 |
| Open | Ctrl/Cmd + O | 触发文件选择器，导入 `.json` |
| Save | Ctrl/Cmd + S | 立即保存（绕过 debounce） |
| Import | — | 打开 Import 模态 |
| Export | — | 打开 Export 模态 |
| Share | — | 打开 Share 模态 |
| Rename | — | 打开 Rename 模态 |
| Delete | — | 删除当前 diagram（确认） |

### 2.2 Edit

| 项 | 快捷键 | 行为 |
|---|---|---|
| Undo | Ctrl/Cmd + Z | 撤销栈弹一步 |
| Redo | Ctrl/Cmd + Shift + Z | 重做栈弹一步 |
| Cut | Ctrl/Cmd + X | 剪切选中对象 |
| Copy | Ctrl/Cmd + C | 复制选中对象 |
| Paste | Ctrl/Cmd + V | 粘贴 |
| Duplicate | Ctrl/Cmd + D | 复制选中 |
| Select All | Ctrl/Cmd + A | 全选画布对象 |
| Find | Ctrl/Cmd + F | 聚焦搜索框 |

### 2.3 View

| 项 | 行为 |
|---|---|
| Zoom In | 画布放大（步进 0.25x） |
| Zoom Out | 画布缩小 |
| Zoom Reset | 重置为 1x |
| Show Grid | 切换网格显示（drawdb 行为；V1 可选） |
| DBML Editor | 切换到 DBML 视图（详见 core-04 §9） |
| Settings | 打开设置页 |

### 2.4 Help

| 项 | 行为 |
|---|---|
| About | 打开 About 模态 |
| Shortcuts | 快捷键速查 |
| Report Bug | 跳转 drawdb issue 页面（V1 复用 drawdb 链接） |

## 3. 9 个模态清单

| 模态 | 用途 | 字段 |
|---|---|---|
| New | 新建 diagram | title（必填） |
| Open | 打开已有 diagram | diagram id（URL 输入） |
| Import | 导入文件 | file（拖拽 / 选择） + format（SQL/DBML/JSON） |
| ImportSource | 选择导入源 | local / remote（V1 仅 local） |
| Language | 切换 UI 语言 | en / zh（V1 双语） |
| SetTableWidth | 批量设置表宽 | target width（0 = auto） |
| Share | 生成分享链接 | visibility（public/private，V1 实际无差别） |
| Rename | 重命名 diagram | title |
| ConfigureCustomTypes | 管理自定义类型 | 列表 + 增删改 |

## 4. 模态通用模式

### 4.1 打开 / 关闭

- 打开：从菜单 / 工具栏触发
- 关闭：右上角 × / ESC / 背景点击
- 取消前若有未保存修改 → 弹确认

### 4.2 布局

```
+--------------------------------------+
| [Title]                    [×]       |
+--------------------------------------+
|                                      |
|         Form fields                   |
|                                      |
+--------------------------------------+
|                          [Cancel][OK]|
+--------------------------------------+
```

### 4.3 校验

- 必填字段失焦时红框 + 提示
- OK 按钮在表单未通过校验时禁用
- 校验规则：与对应实体对象一致（如 diagram.title 非空 + 长度 ≤ 64）

## 5. 模态详细规格

### 5.1 New 模态

- 字段：`title`（text）
- OK：POST `/api/v1/diagrams` → 跳转到 `/editor/{id}`

### 5.2 Open 模态

- 字段：`diagram_id`（text，UUID 格式）
- OK：跳转 `/editor/{id}`

### 5.3 Import 模态

- 字段：`file`（file input）+ `format`（radio: SQL/DBML/JSON）
- 拖拽支持：拖文件到模态区域
- 大小限制：5 MB（来自 bridge config）
- OK：调用 `POST /api/v1/bridge/import/local` → 完成后跳转到新 diagram

### 5.4 ImportSource 模态

- 字段：`source`（radio: local / remote）
- V1 仅 local 实际生效；remote 选项**预留 UI**，后端待 V2

### 5.5 Language 模态

- 字段：`language`（radio: en / zh）
- 立即生效（不需 OK 按钮）
- 持久化到 `localStorage`

### 5.6 SetTableWidth 模态

- 字段：`width`（number，0 = auto）
- 应用：遍历所有 table，更新 `width`
- 立即生效，触发自动保存

### 5.7 Share 模态

- 字段：`share_link`（text，read-only）+ Copy 按钮
- 链接格式：`/editor/{id}`（V1 无权限控制，所有人可访问）
- 复制后按钮文案变 "Copied!" 2 秒

### 5.8 Rename 模态

- 字段：`title`（text）
- OK：PUT `/api/v1/diagrams/{id}`（仅 title 字段）

### 5.9 ConfigureCustomTypes 模态

- 列表：所有自定义类型
- 操作：增 / 删 / 改（详见 core-01c §3.2）
- 关闭：自动保存（V1 仅前端 state；reload 后丢失）— ⚠️ V1 限制

## 6. 工具栏组件

### 6.1 撤销 / 重做栈深度指示

- 撤销栈：`[撤销步数 / 总步数]`
- 例：撤销 3 步后显示 `3/50`

### 6.2 标题编辑器

- 双击 diagram 标题 → 文本输入框
- 失焦或回车 → 触发保存

### 6.3 revision 状态

- 显示 `rev: 5` 标签
- 鼠标悬停 → tooltip 显示时间

### 6.4 SaveState 指示器

- `Saved`（绿）/ `Saving...`（黄）/ `Error`（红）/ `Idle`（灰）
- 鼠标悬停 → 显示最后保存时间

## 7. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-MM-01 | File → New → 填写 title → OK → 创建 diagram |
| UT-MM-02 | Edit → Undo → 撤销栈 -1 |
| UT-MM-03 | View → Zoom In → 画布放大 0.25x |
| UT-MM-04 | 模态背景点击 → 关闭 |
| UT-MM-05 | 模态 ESC → 关闭 |
| UT-MM-06 | 必填字段失焦 → 红框 |
| UT-MM-07 | New 模态 title 为空 → OK 禁用 |
| UT-MM-08 | Share 模态 Copy 按钮 → 剪贴板内容正确 |
| UT-MM-09 | ConfigureCustomTypes 关闭 → 自定义类型保留（仅当前 session） |
| ST-MM-01 | 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路操作 |

## 8. V1 边界

- ❌ Remote Import（V1 仅 local）
- ❌ 自定义快捷键（V1 硬编码 drawdb 默认）
- ❌ 自定义 UI 主题（V1 仅 light）
- ❌ ConfigureCustomTypes 跨刷新保留（V1 仅前端 session state）
- ❌ Share 链接权限控制（V1 公开访问，V2 计划私有房间）
- ❌ 多语言扩展（V1 仅 en / zh）

## 9. 对齐参考源

- drawdb `src/components/EditorHeader/`
- drawdb `src/components/Modals/`
- drawdb `src/components/Modals/Share/`
- drawdb `src/components/Modals/ConfigureCustomTypes/`
- drawdb `src/components/Modals/Import/`
- coldrawdb `frontend-rs/src/editor_panels.rs`（标题编辑器等）
- `docs/drawdb-capability-checklist.md` §2.2
## ADDED — §9.1 B4 测试 ID 索引（提案：add-frontend-completeness）

> 模块：core | 提案：add-frontend-completeness
> 路径：deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md
> 对齐参考源：`core-05-top-menu-modals.md` §7 + `test/core-UI-modals-test-cases.md`

# B4 模态补全 — 测试 ID 索引

## 1. 范围

B4 在 §3 的 9 个模态清单中，**仅实现 4 个核心模态**：
- New（§5.1）
- Open（§5.2）
- Share（§5.7）
- Rename（§5.8）

其余 5 个（Import / ImportSource / Language / SetTableWidth / ConfigureCustomTypes）放 B5。

## 2. 测试 ID 索引

| TC ID | 描述 | 对齐实现 | B4 状态 |
|---|---|---|---|
| UT-MM-01 | New 模态创建 diagram（validate_title + build_create_url） | `editor_panels.rs::modals::validate_title` | ✅ B4 实现 |
| UT-MM-04 | 模态背景点击关闭 | `editor_panels.rs::modals::ModalRoot` | ✅ B4 实现 |
| UT-MM-05 | 模态 ESC 键关闭 | `editor_panels.rs::modals::ModalRoot` | ✅ B4 实现 |
| UT-MM-06 | 必填字段失焦红框 | `editor_panels.rs::modals::{NewModal,RenameModal}` | ✅ B4 实现 |
| UT-MM-07 | New 模态 title 为空 → OK 禁用 | `editor_panels.rs::modals::NewModal` | ✅ B4 实现 |
| UT-MM-08 | Share 模态 URL 格式正确（build_share_url） | `editor_panels.rs::modals::build_share_url` | ✅ B4 实现 |
| UT-MM-09 | Open 模态 JSON 解析（parse_diagram_json） | `editor_panels.rs::modals::parse_diagram_json` | ✅ B4 实现 |
| ST-MM-01 | 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e |

未在本索引中的 §7 编号（UT-MM-02/03 + UT-MM-09 ConfigureCustomTypes 部分）属于 B5 范围（撤销/重做、缩放、ConfigureCustomTypes）。

## 3. B4 spec 修正

- 原 §7 编号 `UT-MM-09 ConfigureCustomTypes 关闭 → 自定义类型保留` 是 ConfigureCustomTypes 模态的测试，不在本 B4 范围。本 B4 delta 将 `Open 模态 JSON 解析` 也归为 `UT-MM-09`（spec 第 9 项的复用，详见 `core-UI-modals-test-cases.md` §2）。
- `ST-S02-01` / `ST-S02-02` / `ST-S02-03` 是 backend `core-S02-test-cases.md` 中的 API 端到端用例，**不在前端 B4 范围**。前端 B4 仅覆盖 UT-MM-01~09 + ST-MM-01。

## 4. 对齐参考源

- `core-05-top-menu-modals.md` §3 / §4 / §5.1 / §5.2 / §5.7 / §5.8
- `core-UI-modals-test-cases.md`（详细 UT 步骤）
- `frontend-rs/src/editor_panels.rs::modals`（新增子模块）
