# 合并指令

## 变更提案
- 提案名称：reposition-referenced-source
- 提案目录：logos/changes/reposition-referenced-source/

## 提案内容

# 变更提案：reposition-referenced-source

> module: core | created: 2026-09-11

## 变更原因
用户指正（2026-09-11，附 README 致谢截图）：现行「全部代码均为纯 Rust 自研，未使用、未移植、未衍生上述任何项目的源代码」的表述过于绝对，与事实不符——开发过程中实际**参考了 drawDB 的源码**作为能力对齐参照（各功能规格的「对齐参考源」章节即记录了 drawdb 源码路径对账）。准确表述应为：**有参考 drawDB 源码，但无直接复用与引用；整体产品理念借鉴 drawDB 与 PDManer**。该过强声明分布于 README（标语 + 致谢）、`logos/logos-project.yaml` description 与 `core-02-product-vision.md` §1.1 共 4 处，需统一修正以保持对外口径一致。

## 变更类型
需求级（产品定位表述修订：不改变任何功能、接口与验收条件，仅修订 §1.1 定位叙述与对外文档措辞）

## 变更范围
- 影响的需求文档：`core-02-product-vision.md` §1.1 产品定位（首段重写；一句话定位与核心差异点不变）
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 另涉及（非规格）：根 `README.md`（头部标语、致谢段）、`logos/logos-project.yaml`（project.description）

## 部署影响
- 是否需要部署：否
- 部署原因：纯文档措辞修订，无运行时行为变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（git 历史即回滚）
- 是否需要 smoke：否

## 变更概述
1. **[delta]** 修订 `core-02-product-vision.md` §1.1 首段：「未使用、未移植、未衍生任何上游项目的源代码」→「开发过程中参考了 drawDB 的源码作为能力对齐参照，但未直接复用、未引用其任何代码；全部代码均为纯 Rust 重新实现」。§1.1 其余内容（一句话定位、核心差异点）保持不变。
2. **[code]** 同步修正对外文档同一口径：README 头部标语（「产品灵感源自……代码为纯 Rust 自研」→「产品理念借鉴 drawDB 与 PDManer，开发中参考 drawDB 源码对齐，代码为纯 Rust 重新实现」）、README 致谢段（同口径重写）、`logos-project.yaml` description。


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-02-product-vision.md

- Delta 文件：`logos/changes/reposition-referenced-source/deltas/prd/1-product-requirements/core-02-product-vision.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
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
   git add -A && git commit -m "docs(reposition-referenced-source): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive reposition-referenced-source`。
