# Delta — core-05-top-menu-modals.md

## ADDED — §12 AppBar IO 按钮与 IO 抽屉（Phase C）

> merge 时在主文档末尾（§11 或最后一节之后）追加。

### 12.1 AppBar 导入 / 导出（Phase C 生效）

| 按钮 | testid | Phase C 行为 |
|------|--------|--------------|
| 导入 | `btn-import` | **启用**；点击 → `io_drawer = Import` |
| 导出 ▾ | `btn-export` | 点击 → `io_drawer = Export`（V1 无下拉子项，单按钮开抽屉） |

- 移除 Phase A 占位：`disabled` + tooltip「导入功能即将推出」
- 保存状态、撤销/重做、分享行为不变

### 12.2 File 菜单「导入」改线

| 菜单项 | V1 行为 | Phase C 行为 |
|--------|---------|--------------|
| 导入 | 打开 `ImportModal` | 打开 `ImportDrawer`（`io_drawer = Import`） |

New / Open / Rename / Share 仍走模态。

### 12.3 Import 模态降级

- `ModalKind::Import` 保留组件与 `parse_sql_statements` UT，**默认 UI 路径不再触发**
- e2e HP-04（SQL 模态 parse）迁移为 ST-PC-01（ImportDrawer parse summary）或保留模态仅测试路径

### 12.4 Phase C 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-AB-04 | **更新**：`btn-import` Phase C 为 **enabled** |
| ST-PC-01 | e2e：AppBar 导入 → 抽屉 → 解析摘要 |

## MODIFIED — §5.3 Import 模态（补充说明）

> 在 §5.3 Import 模态段落末尾追加一段。

**Phase C 备注**：主交互迁移至 `core-01d-import-export.md` ImportDrawer；本节模态规格保留供回归 UT-MM-10，不作为默认用户路径。
