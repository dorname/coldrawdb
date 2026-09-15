# 变更提案：fix-remote-github-issues

> module: core | created: 2026-09-15
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues （OPEN #1～#6）

## 变更原因

远程仓库 `dorname/coldrawdb` 记录了 6 条 open issue（3 BUG + 3 FEATURE），需按 OpenLogos 变更流程修复并关闭。代码对照证据如下：

| Issue | 类型 | 标题 | 根因摘要（对照当前工作树） |
|-------|------|------|---------------------------|
| #4 | BUG | 撤销和回滚操作有问题 | 创建关系走 `on_create_reference` **直接** `store.references.push`，**未** `CommandStack::record(AddReference)`；Ctrl+Z 实际撤销的是栈顶的 `AddTable`，表现为「撤线却把表撤了」。Redo 仅绑定 Ctrl+Shift+Z，**不支持 Ctrl+Y**；且 undo 路径在 `revert` 失败时仍已 pop 栈（键盘路径忽略错误）。 |
| #6 | BUG | 主体颜色调整后无法持久化 | 主题切换只改 `<html data-mode>` + `theme_mode` signal，**不写 localStorage**；`index.html` 默认 `data-mode="dark"`，刷新必回深色。 |
| #1 | BUG | 协作邀请链接没有走后端的地址 | 截图 inviteUrl=`http://192.168.20.106/invite/...`，来自后端 `public_base_url()`（Host 推导 / `PUBLIC_BASE_URL`）。规格与历史变更要求指向**公开 SPA 入口**（nginx:80 / trunk:8080），**不是**裸后端 `:3000`。`docker-compose.yml` **未注入** `PUBLIC_BASE_URL`；若部署仅暴露错误端口或占位 host，链接会「无效」。本提案按「可配置的公开基址 + 部署默认值」修复，而非改回 `:3000`。 |
| #2 | FEATURE | 新增聚焦能力 | 新增表 / 画布搜索 / 列表点击跳回画布后，无 pan/zoom 聚焦到目标表；画布表多时无法定位。 |
| #3 | FEATURE | 字段关系构建交互 | 现需先点 ToolRail「关系」工具再点字段；期望字段→字段直接拖连，免单独点连线按钮。 |
| #5 | FEATURE | 选中区域内多表拖动 | 框选多表后不支持整体拖动（仅单表拖）。 |

## 变更类型

设计级变更（含代码级 BUG 修复；FEATURE 需更新功能规格 / 原型行为 / 测试用例）

## 变更范围

- 影响的需求文档：无（存量能力缺口与交互增强，不改产品 Why）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/` 中与画布交互、关系工具、主题、房间邀请相关的规格（按 delta 增补；主原型 `core-01-editor-prototype.html` 行为对齐说明）
  - 主题持久化：对齐 `core-0b-dark-mode` / 设计系统既有 `data-mode` 合同，补充 localStorage key 约定
- 影响的业务场景：S01（编辑/撤销）、S04（邀请链接）、画布交互（聚焦 / 多选拖动 / 关系手势）；无新场景编号
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` §12（`docker-compose` 注入 `PUBLIC_BASE_URL` 约定）
- 影响的 API：无强制变更（邀请仍由后端生成 `inviteUrl`；可选文档澄清基址语义）
- 影响的 DB 表：无
- 影响的编排测试：无（前端行为为主）
- 影响的 smoke 测试：无强制；若部署方案变更则补充配置检查说明

## 部署影响

- 是否需要部署：否（本提案交付本地可验证的规格 + 代码；生产部署由后续人类确认点单独执行）
- 部署原因：无独立 staging/生产发布任务；`docker-compose.yml` 的 env 默认值变更随下次部署自然生效
- 影响环境：本地 / 后续生产
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

分两批交付同一提案：

**批次 A — BUG（优先）**：#4 关系创建接入 `CommandStack::AddReference`，undo/redo 与 Ctrl+Y 对齐，失败时回滚栈状态；#6 主题读写 `localStorage`（建议 key `cdb.theme`）并在启动时恢复 `data-mode`；#1 在 `docker-compose.yml` / 部署方案补齐 `PUBLIC_BASE_URL` 可配置默认，保证邀请链接指向公开 SPA 入口（禁止把「应走后端 :3000」当作正确目标，避免回退历史缺陷）。

**批次 B — FEATURE**：#2 提供 `focus_table(id)`（pan/zoom 使目标表入视口）；#3 支持字段节点拖放到另一字段直接建关系（关系工具仍保留）；#5 框选多表后拖动同步位移。每批同时交付 UT/ST + OpenLogos reporter。

## 待用户确认的一点（#1）

截图中的 `http://192.168.20.106/invite/...` 按现行 S04/部署规格是**正确形态**（公开入口 Host，非 trunk 私货拼接）。若你的真实期望是强制带 `:3000`，请在确认提案时明确说明——那与 §12「公开基址=SPA 入口」冲突，需先改规格再改代码。默认按「可配置公开基址 + compose 注入」推进。
