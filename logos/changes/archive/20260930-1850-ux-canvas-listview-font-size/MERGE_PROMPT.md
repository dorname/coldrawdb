# 合并指令

## 变更提案
- 提案名称：ux-canvas-listview-font-size
- 提案目录：logos/changes/ux-canvas-listview-font-size/

## 提案内容

# 变更提案：ux-canvas-listview-font-size

> module: core | created: 2026-09-30

## 变更原因
用户在 `/goal` 中提出「列表视图的字体可以大一点」。当前 ListView 表树节点、字段网格主文字均为 12px，注释/分组头仅 10px，在高分辨率屏上阅读吃力，需要适度放大以提升可读性。

## 变更类型
设计级 / 代码级

## 变更范围
- 影响的需求文档：无（保持现有 ListView 功能不变）
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` §10.5（补充字体层级说明）
- 影响的业务场景：无新增/删除场景
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：新增 `UT-LV-FONT-01` 样式合同测试

## 部署影响
- 是否需要部署：是
- 部署原因：前端 CSS 变更需随前端产物重新部署才能生效
- 影响环境：生产前端
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否（纯 UI 样式调整）

## 变更概述
将 ListView 主文字字号从 12px 提升至 14px，字段网格输入框同步放大；表树注释、关联摘要等辅助文字按比例从 10px/11px 提升至 12px；分组头保持 11px 以维持层级。同步更新规格文档中的字体层级说明，并新增 Rust 单元测试 `UT-LV-FONT-01` 锁定 styles.css 中的字号合同。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/ux-canvas-listview-font-size/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(ux-canvas-listview-font-size): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive ux-canvas-listview-font-size`。
