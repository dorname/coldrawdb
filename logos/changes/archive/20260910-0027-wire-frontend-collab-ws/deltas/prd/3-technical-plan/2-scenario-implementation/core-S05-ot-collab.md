## MODIFIED — 0. 现行文档与原型基线

## 0. 现行文档与原型基线

> 版本：V2 | 前置：S03 + S04 | 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-05-ot-collab-prototype.html`（非验收入口）
> 生产状态：后端 WS/OT 已实现；生产前端已接入真实 WS（`wire-frontend-collab-ws`：连接 / op 双向 / presence / 重连 sync / checkpoint 头 / viewer 只读）
> 入口页面状态：在 **`room-editor`** 内建立 WS（废止仅写 `/editor/{diagramId}?room=` 为唯一表述）
> API/DB：不新增；既有 `collab.yaml` + WS `/ws/rooms/{roomId}`
