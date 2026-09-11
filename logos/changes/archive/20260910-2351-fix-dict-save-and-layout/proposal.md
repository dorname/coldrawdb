# 变更提案：修复数据字典保存链路 + 抽屉/检查器互斥 + 列表视图字典展示

> module: core | created: 2026-09-10

## 变更原因

用户反馈 S07 数据字典三个问题（附编辑器截图）：

1. **字典无法正常保存** —— 已定位根因：前端 PUT 保存体 `DiagramForSave`（`frontend-rs/src/editor_data_access.rs:1617`）漏序列化 `dictionaries` 字段，仅含 `id/name/database/pan/zoom/tables/references/areas/notes`。后端收到无 `dictionaries` 键的 diagram 后按空数组落库（`backend/src/diagram_persistence.rs:521-527`），**每次保存即清空全部字典**。加载路径（`DiagramOut`）与后端存储均完整，断点只在前端保存序列化。
2. **字典抽屉与检查器重叠** —— `.cdb-dict-panel` 为 `position:absolute; right:编辑器 padding; z-index:35` 浮层（`styles.css:4478`），直接盖在 330px 检查器（z-index 30）之上；`on_toggle_dicts`（`editor_panels.rs:9626`）只翻转开关，没有像 IO 抽屉那样的互斥/缓存恢复逻辑（`snapshot_before_io_drawer` / `restore_inspector_after_io_drawer`，`editor_panels.rs:2722-2729, 9631-9638`）。
3. **列表视图缺少字典展示** —— ListView 目前只有「表」维度的树 + 字段明细（字段行内虽有字典摘要 Tag，见 `editor_panels.rs:8143-8162`），但没有字典本身的入口与展示。

## 变更类型

设计级变更（问题 2/3 涉及 S07 交互与展示设计；问题 1 为同场景内的代码级缺陷修复，一并闭环）

## 变更范围

- 影响的需求文档：无（S07 需求不变，属实现/交互修正）
- 影响的功能规格：`core-01e-data-dictionary.md`（§2 抽屉交互补互斥规则；§3 补列表视图字典区块；补保存链路口径）
- 影响的原型：`core-01-editor-prototype.html`（主原型对齐抽屉互斥 + 列表视图字典区块）
- 影响的业务场景：S07（数据字典）
- 影响的部署方案：无
- 影响的 API：无（后端 `dictionaries` 列与 DTO 已完整支持）
- 影响的 DB 表：无（`diagram.dictionaries`、`field.dict_code` 列已存在）
- 影响的编排测试：无（非 API 变更）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：前后端同仓库本地构建运行，无独立部署任务（对齐 `fix-field-tag-persistence` 等前例）
- 影响环境：无
- 是否涉及数据迁移：否（后端列已存在；前端补发字段即可，历史被清空的字典不恢复）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **保存链路修复**：`DiagramForSave` 补 `dictionaries` 字段（`skip_serializing_if = "Vec::is_empty"` 保持向后兼容），`save()` PUT 体携带字典，保存后字典不再丢失。
2. **抽屉/检查器互斥**：复用 IO 抽屉既有模式——字典抽屉打开时缓存 Inspector 状态并收起，关闭时恢复；反向 Inspector 打开时关闭字典抽屉。消除视觉重叠。
3. **列表视图字典展示**：ListView 树区新增「数据字典」分组，展示字典名称/编码/项数/引用数与映射摘要 Tag；选中字典时详情区展示字典项（只读浏览），编辑入口仍走字典抽屉，视觉与既有列表风格一致。
