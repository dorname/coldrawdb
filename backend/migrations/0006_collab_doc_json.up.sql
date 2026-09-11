-- fix-collab-autosave-race（方案 B 服务端物化单写者）:
-- room_collab_head.doc_json = 房间 canonical 文档（前端 Diagram JSON 形状）。
-- op 落库即物化（同事务），与 op log 恒一致；NULL=未物化，首个 op 时从关系存储初始化。

ALTER TABLE room_collab_head ADD COLUMN doc_json TEXT;
