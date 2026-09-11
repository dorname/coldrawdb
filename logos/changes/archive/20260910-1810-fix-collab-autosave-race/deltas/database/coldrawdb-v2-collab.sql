# Delta: coldrawdb-v2-collab.sql（fix-collab-autosave-race · 方案 B 服务端物化单写者）

## MODIFIED — 19. room_collab_head — room 协作头指针（S05.1 connected 帧）

```sql
-- ============================================================
-- 19. room_collab_head — room 协作头指针（S05.1 connected 帧）
-- ============================================================
CREATE TABLE IF NOT EXISTS room_collab_head (
    -- @comment room.id，每 room 一行
    room_id             TEXT    PRIMARY KEY,
    -- @comment 当前最大 server_rev，0 表示尚无 op
    server_rev          INTEGER NOT NULL DEFAULT 0,
    -- @comment diagram 快照哈希（checkpoint 后更新，可选）
    snapshot_hash       TEXT,
    -- @comment 最近一次 REST checkpoint 的 diagram.revision（fix-collab-autosave-race 起不再更新，保留兼容）
    checkpoint_revision INTEGER,
    -- @comment 最后 checkpoint 时间（fix-collab-autosave-race 起不再更新，保留兼容）
    last_checkpoint_at  TEXT,
    -- @comment 房间 canonical 文档（前端 Diagram JSON 形状；op 落库即物化，与 op log 恒一致；NULL=未物化，首个 op 时从关系存储初始化）
    doc_json            TEXT,
    -- @comment 头指针更新时间
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE
);
-- @table-comment room_collab_head S05 协作 revision 头 + 物化文档（单写者）
```

> 迁移：`backend/migrations/0006_collab_doc_json.up.sql` 执行
> `ALTER TABLE room_collab_head ADD COLUMN doc_json TEXT;`（存量行 NULL=未物化）；
> down 迁移 `DROP COLUMN`。
