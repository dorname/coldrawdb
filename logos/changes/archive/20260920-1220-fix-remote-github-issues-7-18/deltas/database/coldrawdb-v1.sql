-- Delta — coldrawdb-v1.sql（修改）
-- 模块：core | 提案：fix-remote-github-issues-7-18
-- 关联 issue：#12（关系连线颜色配置，reference 表落库）
--
-- merge 操作：MODIFIED — reference 表定义，追加 color 列。

-- MODIFIED — reference 表（追加 color 列）
CREATE TABLE IF NOT EXISTS reference (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    diagram_id          TEXT    NOT NULL,
    name                TEXT    NOT NULL DEFAULT '',
    start_table_id      INTEGER NOT NULL,
    start_field_id      INTEGER NOT NULL,
    end_table_id        INTEGER NOT NULL,
    end_field_id        INTEGER NOT NULL,
    cardinality         TEXT    NOT NULL CHECK(cardinality IN ('one_to_one', 'one_to_many', 'many_to_one', 'many_to_many')),
    on_update           TEXT    NOT NULL DEFAULT 'NO ACTION' CHECK(on_update IN ('CASCADE', 'RESTRICT', 'SET NULL', 'NO ACTION', 'SET DEFAULT')),
    on_delete           TEXT    NOT NULL DEFAULT 'NO ACTION' CHECK(on_delete IN ('CASCADE', 'RESTRICT', 'SET NULL', 'NO ACTION', 'SET DEFAULT')),
    color               TEXT    NOT NULL DEFAULT '',
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    FOREIGN KEY (start_table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    FOREIGN KEY (end_table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    FOREIGN KEY (start_field_id) REFERENCES field(id) ON DELETE CASCADE,
    FOREIGN KEY (end_field_id) REFERENCES field(id) ON DELETE CASCADE
);

-- 既有库迁移路径（backend/migrations/0006_reference_color.up.sql / .down.sql）：
--   up:   ALTER TABLE reference ADD COLUMN color TEXT NOT NULL DEFAULT '';
--   down: ALTER TABLE reference DROP COLUMN color;
-- 语义：'' = 未配置（渲染回退默认主题色）；存量行自动取 ''，无数据回填需求。
