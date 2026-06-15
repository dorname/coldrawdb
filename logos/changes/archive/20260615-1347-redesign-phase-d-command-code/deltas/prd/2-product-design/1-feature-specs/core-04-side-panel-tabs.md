# Delta — core-04-side-panel-tabs.md

## ADDED — §12 Phase D：浏览能力迁移至 Command Palette

> merge 时在文档末尾追加。

### 12.1 能力对照

| V1 左栏能力 | Phase A | Phase D |
|-------------|---------|---------|
| Tables 列表浏览 | ❌ 移除 UI | ✅ Command Palette |
| Areas / Enums / Notes / Types 列表 | ❌ | ✅ Palette 分组 |
| Relationships 列表 | ❌ | ✅ Palette + Inspector 摘要 |
| 全局搜索 | ❌ | ✅ Palette `filter_palette_items` |
| Issues Tab | Tool Rail 徽章 | 不变 |

### 12.2 跳转语义（不变）

- 单击 Palette 表项 ≡ 原左栏单击：画布选中 + Inspector 展开
- `jump_to_table` / `jump_to_reference` 纯函数保留

### 12.3 测试迁移

| 原 TC | Phase D 替代 |
|-------|--------------|
| UT-SP-09 | UT-PD-07 |
| UT-SP-10 | UT-PD-08 |

## MODIFIED — §1 侧边栏布局（补充说明）

> 在 §1 首段后追加备注。

**Phase D 备注**：V1 左栏 UI 不恢复；`LeftPanel` 组件保留供 UT 回归或删除批次，AppRoot 不挂载。用户浏览路径统一为 `Ctrl+K` Command Palette。
