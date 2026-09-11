# 合并指令

## 变更提案
- 提案名称：reposition-self-developed
- 提案目录：logos/changes/reposition-self-developed/

## 提案内容

# 变更提案：reposition-self-developed

> module: core | created: 2026-09-11

## 变更原因

实际代码与 drawDB、PDManer 两个上游灵感来源项目完全无关，属纯 Rust 自研，而非「重写 / 派生」。项目 README 已率先调整为「产品灵感源自 drawDB 与 PDManer，代码为纯 Rust 自研」定位；但方法论文档元数据仍声明「Rust Web 重写版 drawdb」，与事实及新定位相矛盾，需同步修正，避免对外表述不一致。

## 变更类型

文档级（定位声明 / 元数据变更）。不触发需求级 / 设计级 / 接口级传播规则：

- 不改变任何场景（S01～S07 名称、范围、验收均不变）
- 不涉及 API、DB、编排测试、smoke
- 无代码行为变化

## 变更范围

- 影响的需求文档：`logos/resources/prd/1-product-requirements/core-02-product-vision.md` §1.1 产品定位（「定位为 drawdb 的 Rust Web 重写版本」→ 灵感源自 + 纯自研声明）
- 影响的功能规格：无
- 影响的业务场景：无（S01～S07 不变）
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 另涉及（方法论索引，非 `logos/resources/` 主文档，直接修改不走 delta）：`logos/logos-project.yaml` `project.description`
- 另涉及（非方法论文件，guard 豁免，已随本次定位调整先行完成于工作区）：`README.md`（头部标语 / 技术栈注 / 致谢章节）

影响范围排查结论：全库 `重写 / 重写版 / rewrite of / based on drawdb` 定位类表述仅命中上述两处；`core-01-editor-canvas.md:194`「复用 不重写」为技术语境，不在范围；其余 30+ 文档中的 drawdb 均为对比引用（如「相对 drawdb 等方案」），与新定位兼容，不修改。

## 部署影响

- 是否需要部署：否
- 部署原因：纯文档与元数据变更，无运行时行为变化
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

将项目定位声明从「drawDB 的 Rust Web 重写版」统一调整为「产品灵感源自 drawDB 与 PDManer，全部代码为纯 Rust 自研（未使用、未移植、未衍生上游源代码）」：

1. **delta**：修改 `core-02-product-vision.md` §1.1 产品定位段落，明确灵感来源与自研声明，差异点对比基准改为「相对 drawDB / PDManer 等同类工具」。
2. **直接修改**：`logos/logos-project.yaml` 的 `project.description`（该文件为方法论索引本身，不经 delta/merge 流程）。
3. **已完成**：`README.md` 三处定位调整（头部标语、技术栈注、致谢章节）已在提案创建前完成于工作区，纳入本提案范围以便追溯。


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-02-product-vision.md

- Delta 文件：`logos/changes/reposition-self-developed/deltas/prd/1-product-requirements/core-02-product-vision.md`
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
   git add -A && git commit -m "docs(reposition-self-developed): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive reposition-self-developed`。
