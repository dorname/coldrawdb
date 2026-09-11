-- fix-collab-autosave-race down: 移除 doc_json（SQLite ≥3.35 支持 DROP COLUMN）

ALTER TABLE room_collab_head DROP COLUMN IF EXISTS doc_json;
