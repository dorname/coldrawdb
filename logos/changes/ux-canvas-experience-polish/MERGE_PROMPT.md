# 合并指令

## 变更提案
- 提案名称：ux-canvas-experience-polish
- 提案目录：logos/changes/ux-canvas-experience-polish/

## 提案内容

# 变更提案：ux-canvas-experience-polish

> module: core | created: 2026-09-30

## 变更原因
用户在 `/goal` 中提出「继续优化画布体验」。当前画布存在两处可感知的体验细节：
1. 滚轮缩放步长偏小（每次 10%），快速定位时操作次数多；
2. 关系线默认视觉偏细，在复杂图中不够醒目，选中/相关态的光晕也有提升空间。

## 变更类型
代码级

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：`core-01-editor-canvas.md` §3.3（滚轮缩放因子）与 §5.x（关系线视觉）
- 影响的业务场景：无新增/删除场景
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：
  - 新增 `UT-CR-ZOOM-02` 锁定滚轮缩放因子
  - 更新 `UT-PE-HL-01` 关系线光晕/主线合同

## 部署影响
- 是否需要部署：是
- 部署原因：前端代码变更需重新 build
- 影响环境：生产前端
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
1. 将画布滚轮缩放因子从 `1.1` 提升到 `1.15`，每次滚动放大/缩小 15%，减少快速定位所需轮数。
2. 将关系线默认主线宽度从 `2.0px` 提升到 `2.5px`，默认光晕从 `7.0px` 提升到 `8.0px`；相关态光晕从 `8.0px` 提升到 `10.0px`，选中态光晕从 `10.0px` 提升到 `12.0px`，使关系线在复杂画布中更醒目。
3. 新增 `UT-CR-ZOOM-02` 单元测试锁定滚轮因子常量；同步更新 `UT-PE-HL-01` 源码字符串断言。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/ux-canvas-experience-polish/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
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
   git add -A && git commit -m "docs(ux-canvas-experience-polish): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive ux-canvas-experience-polish`。
