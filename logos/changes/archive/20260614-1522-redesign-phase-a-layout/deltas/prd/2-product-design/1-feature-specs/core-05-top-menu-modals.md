# Delta — core-05-top-menu-modals.md
# 模块：core | 提案：redesign-phase-a-layout
# 路径：`logos/changes/redesign-phase-a-layout/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`

## MODIFIED — §1 顶部菜单布局

> **替换** 主文档 `core-05-top-menu-modals.md` §1 整节。

## 1. AppBar 布局（V2 / Phase A）

V2 将 V1 的「顶栏菜单 + 工具栏」双行合并为单行 **AppBar**（高 48px）。

```
+--------------------------------------------------------------------------------+
| [Logo] [项目名 ✎] ●已保存 | [导入] [导出▾] | ↶ ↷ | [分享] | File▾ Edit▾ View▾ Help▾ | ⚙ |
+--------------------------------------------------------------------------------+
```

### 1.1 区域划分

| 区块 | 内容 | testid |
|------|------|--------|
| 品牌区 | Logo + 项目名（可编辑） | `app-bar` / `diagram-title` |
| 保存区 | SaveState 指示器 | `save-state` |
| 项目 IO 区 | 导入、导出下拉 | `btn-import` / `btn-export` |
| 编辑区 | 撤销、重做 | `btn-undo` / `btn-redo` |
| 协作区 | 分享 | `btn-share` |
| 菜单区 | File / Edit / View / Help 下拉（保留 V1 子项） | `cdb-menu-file` 等 |
| 设置区 | ⚙ 图标 | `btn-settings` |

### 1.2 与 V1 双行布局的差异

| V1 组件 | V2 归属 |
|---------|---------|
| `TopMenuBar` Logo + 4 下拉 | AppBar 菜单区 |
| `TopMenuBar` SaveState | AppBar 保存区 |
| `Toolbar` 撤销/重做 | AppBar 编辑区 |
| `Toolbar` 标题编辑 | AppBar 品牌区（项目名） |
| `Toolbar` rev 标签 | StatusBar（见 `core-00` §1.1） |
| `Toolbar` Share / Export | AppBar 协作区 + 项目 IO 区 |

### 1.3 项目名编辑

- 单击项目名 → 切换为 inline input（与 V1 双击标题等价，改为单击）
- 失焦或 Enter → 触发 `on_title_blur` + debounce 保存
- 显示未保存星号：`项目名 *`（`store.dirty == true`）

### 1.4 导入 / 导出按钮（Phase A 占位）

| 按钮 | Phase A 行为 | Phase C 行为 |
|------|--------------|--------------|
| `btn-import` | disabled；tooltip「导入功能即将推出」 | 打开导入侧边抽屉 |
| `btn-export` | 下拉可用；子项 SQL/DBML/JSON 打开 Export 模态（保留 V1 模态） | 导出抽屉 + 预览 |

### 1.5 SaveState 三级反馈

| 状态 | 显示 | 样式 class |
|------|------|------------|
| 已保存 | `● 已保存 · {相对时间}` | `cdb-save-state` |
| 保存中 | `◌ 保存中...` | `cdb-save-state cdb-is-saving` |
| 失败 | `● 保存失败` + 点击重试 | `cdb-save-state cdb-is-error` |
| 有未保存修改 | 项目名旁 `*` + 可选 `● 未保存` | `cdb-is-dirty` |

### 1.6 移除的 V1 元素

- 独立 `Toolbar` 行（`data-testid="toolbar"` 移除）
- Toolbar 内 `rev:` 标签（迁移至 StatusBar）

## MODIFIED — §6 工具栏组件

> **替换** §6 标题为「§6 AppBar 与 StatusBar 组件」，并更新 §6.3 revision 归属。

### 6.3 revision 状态（迁移至 StatusBar）

- 显示位置：StatusBar 右侧 `rev: {n}`
- testid：`revision-display`（保留）
- 鼠标悬停 tooltip 显示最后保存时间

其余 §6.1 撤销栈、§6.2 标题编辑器、§6.4 SaveState 逻辑不变，**渲染位置**改为 AppBar（见 §1）。

## ADDED — §10 Phase A 测试 ID 索引

| TC ID | 描述 | 对齐实现 |
|-------|------|----------|
| UT-AB-01 | AppBar 单行渲染；DOM 中无独立 `toolbar` testid | `editor_panels.rs` |
| UT-AB-02 | `diagram-title` 单击进入编辑态 | `AppBar` |
| UT-AB-03 | dirty 时项目名显示 `*` | `AppBar` + `store.dirty` |
| UT-AB-04 | `btn-import` Phase A 为 disabled | `AppBar` |
| UT-AB-05 | `revision-display` 位于 StatusBar | `StatusBar` |
| ST-AB-01 | e2e：AppBar 保存状态从 saving → saved 切换 | Playwright |

> 详细步骤见 `core-PA-layout-test-cases.md` §2 AppBar 组。

## ADDED — §11 Phase A 边界

- ❌ 导入侧边抽屉（Phase C）
- ❌ 合并 File 菜单与按钮的进一步精简（Phase C 后评估）
- ✅ 单行 AppBar 合并双顶栏
- ✅ 导入/导出按钮视觉常驻
- ✅ revision 迁移 StatusBar
