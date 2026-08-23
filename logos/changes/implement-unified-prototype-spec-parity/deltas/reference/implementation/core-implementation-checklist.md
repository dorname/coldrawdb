# Delta — core-implementation-checklist.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 13. 统一原型规格收口状态

本清单对统一原型相关能力采用**三列状态**，禁止把未实现或未逐项验证的项目标记为完成。

| 列 | 含义 |
|---|---|
| 已有能力 | 仓库中已存在且可运行的事实能力（可为部分接入） |
| 规格已收口 | 上一变更 `align-all-docs-to-unified-prototype` 已对齐文档/测试合同 |
| 本提案实现 | `implement-unified-prototype-spec-parity` 按 A～D 批对照主原型补齐与验收；代码完成前不得把该项标为生产完成 |

统一措辞：后端已实现；生产前端部分接入。本提案执行逐项对齐，完成前不得勾选「相对主原型已对齐」。

### 13.1 S01～S05 / 壳层三列总表

| 能力项 | 已有能力 | 规格已收口 | 本提案实现 |
|---|---|---|---|
| S01 保存 / SaveState / 非 OT 409 | 是（后端+前端保存链路） | 是（含协作禁 409） | C 批 |
| S02 分享只读 / 404 / 无 share→auth | 是（分享加载） | 是 | A 批 |
| S03 auth→rooms / 会话 / 不枚举用户 | 是（API+部分 UI） | 是 | A 批 |
| S04 rooms/invite/成员/Viewer | 是（API+部分 UI） | 是 | B 批 |
| S05 ws-status/ot-rev/presence/reconnect/queue/local-only | 是（WS+部分 UI） | 是 | C 批 |
| 画布拖表 GRID_SIZE=20 + 跟线 | 部分 | 是 | D 批 |
| 关系 4px / rubber-band / 两点 / 确认条 | 部分 | 是 | D 批 |
| IO 更多菜单→抽屉 | 部分 | 是 | D 批 |
| 主模态 Esc 无残留 | 部分 | 是 | D 批 |
| Inspector 锚点 + 响应式抽屉 | 部分 | 是 | D 批 |
| ⌘K / Esc / T / R | 部分 | 是 | D 批 |
| Design system 主题/motion | 部分（E1–E6 已落地基础） | 是（与统一壳层对齐合同） | D 批 |
| 主原型演示器本身 | 是（静态 HTML） | N/A（不改原型） | 禁止标生产完成 |

### 13.2 既有勾选区解读规则

- 历史 `[x]` 仅表示**已有能力**列意义上的存在，**不**自动等于「相对主原型逐项对齐完成」。
- 凡涉及 auth/rooms/invite/room-editor 视觉与交互贴合主原型的项：在本提案对应批次验证前保持未完成，不得提前勾选。
- Monaco 完整挂载、Mermaid/PNG 导出、K8s 等原边界项：仍为未完成，不得改完成。

### 13.3 本提案执行批次

本提案即第二阶段执行入口。验收输入：已合并测试矩阵 + `core-frontend-alignment-acceptance.md` §7。

| 批次 | 范围 | 主要用例 |
|---|---|---|
| A | auth / share / 页面流入口 | ST-S03-UI-*、S02 SHARE/*、ST-FE-ALIGN-01/02、ST-PU-22 |
| B | rooms / invite | ST-S04-UI-*、ST-PU-23 |
| C | room-editor 壳层 / 保存 / 协作 | S01-SS/*、S01-409/*、ST-S05-UI-*、ST-FE-ALIGN-03/04、ST-PU-24 |
| D | IO / 快捷键 / 主题 / 响应式 | ST-KB-*、ST-PC-*、ST-PU-25/26 |
