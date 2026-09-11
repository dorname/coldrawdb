# 变更提案：fix-field-tag-persistence

> module: core | created: 2026-09-10

## 变更原因
Bug（S07 验收时发现的既有缺口）：前端 `Field.tag`（ListView「说明」列、ByTag 分组键，core-01a 已定义该属性）在编辑器内可用，但后端规范化存储未承载——`field` 表无 `tag` 列、`FieldDto` 无 `tag` 字段、SELECT/INSERT 均未接线。非房间图保存 → 重新打开后 tag 丢失。房间图（`room_collab_head.doc_json` JSON 透传）不受影响。规格已定义该能力，此次为实现对齐规格的持久化缺口修复。

## 变更类型
代码级修复（含 DB 加列 migration；无 API 变更、无规格行为变化）

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：无（core-01a 字段属性已含 tag，无需变更）
- 影响的业务场景：S01（保存/加载通路；测试用例文件登记新用例 UT-S01-11）
- 影响的部署方案：无
- 影响的 API：无（FieldDto 为内部 DTO，REST 载荷 JSON 新增 `tag` 字段，serde default 向后兼容）
- 影响的 DB 表：`field`（新增 `tag VARCHAR NOT NULL DEFAULT ''` 列，migration 0008）
- 影响的编排测试：无
- 影响的 smoke 测试：无

## 部署影响
- 是否需要部署：否
- 部署原因：migration 随服务启动自动应用（同 0007 模式），无独立部署动作
- 影响环境：无
- 是否涉及数据迁移：是——schema 加列；`DEFAULT ''` 自动回填存量行，无需数据脚本
- 是否需要回滚预案：否（down migration 提供 `DROP COLUMN` 回滚）
- 是否需要 smoke：否

## 变更概述
为 `field` 表新增 `tag` 列（migration `0008_field_tag.up/down.sql`，模式照 0007），`FieldDto` 增加 `#[serde(default)] pub tag: String`（老 JSON 无此字段反序列化为 `""`，向后兼容），SELECT/INSERT 接线补齐。前端零改动——`Field.tag` 已有 `#[serde(default)]`（`editor_core.rs`），后端 DTO 透出后自然恢复保存/加载往返。补充后端往返测试（tag 保存→加载一致，照 0007 dict_code 测试模式）并在 `core-S01-test-cases.md` 登记 UT-S01-11。
