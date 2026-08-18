# Delta — core-S05-ot-collab-design.md（修改）

> module: core | proposal: improve-unified-collab-prototype | 2026-08-18
> merge 时修改原型引用，并将下列章节追加到正式规格。

## MODIFIED — 1. 产品类型与原型策略

> 替换主文档 `core-S05-ot-collab-design.md` §1 整节；顶部元数据中的原型路径作为阶段历史记录保留。

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（房间内编辑器 + 实时同步层） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（模拟双端：本地编辑 + 远端 op / 光标 / 重连） |
| 历史参考 | `core-05-ot-collab-prototype.html`（保留，不再作为验收入口） |
| 视觉基准 | 在统一协作编辑器上叠加 presence、远端光标、连接态 Banner 与演示控制台 |
| 痛点关联 | **P03**——消除「邮件传 JSON + 手动 merge」；多人同时改 schema 可收敛 |

## ADDED — §10 单文件协作模拟器

### 10.1 模拟器定位

协作模拟器是主原型中的显式演示控制台，用确定性计时器和本地事件模拟 WebSocket 帧。它用于评审 UI 状态与业务逻辑，不建立真实网络连接，也不宣称 OT 算法已经生产接通。

### 10.2 模拟动作

| 演示动作 | 模拟帧/状态 | UI 结果 |
|---|---|---|
| Alice 移动光标 | `presence.cursor` | 紫色远端光标平滑移动并带姓名标签 |
| Alice 创建 `orders` | `remote_op: table.create` | 画布插入表、Activity 增项、server revision +1 |
| Bob 修改字段 | `remote_op: field.update` | Inspector/画布同步更新，字段短暂显示远端高亮 |
| 并发编辑 | 本地 `field.update` + remote op | Toast 提示 OT 已合并，不出现 409 模态 |
| 模拟断线 | `connected → reconnecting` | 黄色 Banner，本地写操作进入队列 |
| 恢复连接 | `sync(last_rev)` | 回放队列、补发远端操作、revision 递增、成功 Toast |
| 重连失败 | `reconnecting → failed` | 红色 Banner，显示刷新与仅本地编辑 |
| Viewer 模式 | `role=viewer` | 禁止发送 op，仍接收 presence 与 remote op |

### 10.3 状态不变量

- `connected`：本地命令立即 optimistic 应用并在短延迟后 ack；server revision 单调递增。
- `reconnecting`：本地命令可 optimistic 应用，但必须进入可见队列，StatusBar 显示待同步数量。
- `failed`：默认禁止继续协作写；选择「仅本地编辑」后允许本地命令，同时持续警告 409 风险。
- `viewer`：无论连接状态如何，均不得进入待发送队列或制造 ack；仍可接收远端状态。
- 远端操作必须同时更新画布、Activity 与 revision，避免只有动画而数据未变化。

### 10.4 操作队列与 OT 反馈

原型 store 保存 `serverRev`、`connection`、`pendingOps` 与 `activity`。本地命令经统一 dispatcher：先校验角色，再更新编辑历史；连接正常时模拟 ack，重连中则排队。恢复连接时按顺序清空队列并生成一条同步完成 Activity。

并发字段修改模拟结果必须可观察：最终字段同时保留本地约束修改与远端类型修改，并显示「服务器已合并 2 个并发操作」。协作模式不得弹出 S01 的 409 冲突模态。

### 10.5 锚点与动效

保留 `ws-status`、`ot-rev`、`remote-cursor`、`activity-feed`、`reconnect-banner`、`room-presence`。远端光标位置更新不超过 20fps；`prefers-reduced-motion: reduce` 下直接跳到目标位置并关闭脉冲动画。
