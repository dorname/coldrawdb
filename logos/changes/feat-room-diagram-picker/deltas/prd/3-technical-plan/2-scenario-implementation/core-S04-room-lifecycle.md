## ADDED — 3A. 时序图 — 图表候选与个人建模（feat-room-diagram-picker / #56）

```mermaid
sequenceDiagram
    participant U as User/Browser
    participant UI as RoomsListPage
    participant DC as DiagramClient
    participant API as LegacyDiagramAPI
    participant DB as SQLite
    participant Editor as AppRoot/Editor

    U->>UI: Step 1: 打开创建房间或个人建模
    UI->>DC: Step 2: list_summaries()
    DC->>API: Step 3: GET /diagrams/queryAll
    API->>DB: Step 4: 单条 SELECT diagram + table/reference COUNT 子查询
    DB-->>API: Step 5: id/name/revision/updated_at/table_count/reference_count
    API-->>DC: Step 6: 200 摘要数组（既有字段保持兼容）
    DC-->>UI: Step 7: 排除活跃 room 绑定图，搜索并最近修改倒序
    UI-->>U: Step 8: 名称 + 短 ID + rev + 表/关系数 + 时间

    alt 个人新建空白模型
        U->>UI: Step 9a: 输入图名/引擎，打开个人画布
        UI->>DC: Step 10a: create(name,database)
        DC->>API: Step 11a: POST /api/v1/diagrams
        API-->>DC: Step 12a: 新 diagram id
    else 个人打开已有图
        U->>UI: Step 9b: 搜索并选择未绑定图，打开个人画布
        UI->>UI: Step 10b: 读取已选 diagram id，不创建 diagram/room
    end

    UI->>DC: Step 13: get(diagram_id) + enter_nonce
    DC->>API: Step 14: GET /api/v1/diagrams/{id}
    API-->>DC: Step 15: 完整 diagram
    DC-->>Editor: Step 16: 仅最后进入者生效，store.load
    Editor-->>U: Step 17: current_room=None 的个人编辑器，无私有写收编
    Note over UI,Editor: 全程不调用 POST /api/v1/rooms；room 绑定图不进入个人候选
```

- 创建协作房间继续走 §3 既有 `POST /api/v1/rooms` 时序；图表选择器只替换候选呈现，不改变房间创建合同。
- 危险删除仍走既有 `DELETE /api/v1/diagrams/{id}` + 二次确认，确认上下文增加完整 ID 与规模信息。
- 新增响应字段仅用于显示与搜索；无 DB migration、无新增 API 端点、无逐图补充查询。
