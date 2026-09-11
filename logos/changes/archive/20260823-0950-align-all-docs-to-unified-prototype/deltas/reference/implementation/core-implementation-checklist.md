# Delta — core-implementation-checklist.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 13. 统一原型规格收口状态

本清单对统一原型相关能力采用**三列状态**，禁止把未实现或未逐项验证的项目标记为完成。

| 列 | 含义 |
|---|---|
| 已有能力 | 仓库中已存在且可运行的事实能力（可为部分接入） |
| 本提案规格已收口 | 文档/测试合同已对齐主原型（本变更 merge 后） |
| 第二阶段待验证 | `implement-unified-prototype-spec-parity` 中对照主原型做结构/视觉/交互逐项验证与补齐 |

统一措辞：后端已实现；生产前端部分接入；逐项对齐待第二阶段。

### 13.1 S01～S05 / 壳层三列总表

| 能力项 | 已有能力 | 本提案规格已收口 | 第二阶段待验证 |
|---|---|---|---|
| S01 保存 / SaveState / 非 OT 409 | 是（后端+前端保存链路） | 是（含协作禁 409） | 是（文案/锚点/与主原型一致） |
| S02 分享只读 / 404 / 无 share→auth | 是（分享加载） | 是 | 是 |
| S03 auth→rooms / 会话 / 不枚举用户 | 是（API+部分 UI） | 是 | 是 |
| S04 rooms/invite/成员/Viewer | 是（API+部分 UI） | 是 | 是 |
| S05 ws-status/ot-rev/presence/reconnect/queue/local-only | 是（WS+部分 UI） | 是 | 是 |
| 画布拖表 GRID_SIZE=20 + 跟线 | 部分 | 是 | 是 |
| 关系 4px / rubber-band / 两点 / 确认条 | 部分 | 是 | 是 |
| IO 更多菜单→抽屉 | 部分 | 是 | 是 |
| 主模态 Esc 无残留 | 部分 | 是 | 是 |
| Inspector 锚点 + 响应式抽屉 | 部分 | 是 | 是 |
| ⌘K / Esc / T / R | 部分 | 是 | 是 |
| Design system 主题/motion | 部分（E1–E6 已落地基础） | 是（与统一壳层对齐合同） | 是 |
| 主原型演示器本身 | 是（静态 HTML） | N/A（不改原型） | 禁止标生产完成 |

### 13.2 既有勾选区解读规则

- 历史 `[x]` 仅表示**已有能力**列意义上的存在，**不**自动等于「相对主原型逐项对齐完成」。
- 凡涉及 auth/rooms/invite/room-editor 视觉与交互贴合主原型的项：在第二阶段验证前保持「待验证」，不得新增为完成。
- Monaco 完整挂载、Mermaid/PNG 导出、K8s 等原边界项：仍为未完成，不得改完成。

### 13.3 第二阶段入口

下一变更：`implement-unified-prototype-spec-parity`。验收输入：本提案合并后的测试矩阵 + `core-frontend-alignment-acceptance.md` 区域 checklist。
