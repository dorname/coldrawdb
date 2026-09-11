//! ST-PC-07: 导出→导入 round-trip 注释不丢（fix-canvas-zoom-invite-comment-resize）
//!
//! Spec: `logos/resources/test/core-PC-import-export-test-cases.md` ST-PC-07
//! 场景：构造 store（1 表，表 comment + 2 列各带 comment）→
//! `export_diagram_sql(store, "postgres")` → 输出喂入 `parse_sql_import_tables`
//! 断言：解析结果表/列 comment 与原始 store 逐字段一致
//! （COMMENT ON TABLE / COMMENT ON COLUMN 均回填，含单引号转义往返）

use frontend_rs::editor_core::types::{Field, Table};
use frontend_rs::editor_panels::{export_diagram_sql, parse_sql_import_tables};

fn fixture_table() -> Table {
    Table {
        id: "t1".to_string(),
        name: "users".to_string(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: "用户表".to_string(),
        fields: vec![
            Field {
                id: "f1".to_string(),
                name: "id".to_string(),
                type_: "SERIAL".to_string(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: true,
                increment: true,
                comment: "主键".to_string(),
                tag: String::new(),
                dict_code: String::new(),
            },
            Field {
                id: "f2".to_string(),
                name: "note".to_string(),
                type_: "TEXT".to_string(),
                default: String::new(),
                check: String::new(),
                primary: false,
                unique: false,
                not_null: false,
                increment: false,
                comment: "it's a note".to_string(),
                tag: String::new(),
                dict_code: String::new(),
            },
        ],
        indices: vec![],
        width: None,
        min_height: None,
    }
}

#[test]
fn st_pc_07_export_import_comment_roundtrip() {
    let store = vec![fixture_table()];
    let ddl = export_diagram_sql(&store, &[], "postgres");

    // 导出形态：COMMENT ON TABLE 与 COMMENT ON COLUMN 均出现在 CREATE TABLE 之后
    let create_idx = ddl.find("CREATE TABLE users").expect("应含 CREATE TABLE");
    let table_comment_idx = ddl
        .find("COMMENT ON TABLE users IS '用户表';")
        .expect("应含 COMMENT ON TABLE");
    let col_comment_idx = ddl
        .find("COMMENT ON COLUMN users.note IS 'it''s a note';")
        .expect("应含转义后的 COMMENT ON COLUMN");
    assert!(table_comment_idx > create_idx);
    assert!(col_comment_idx > create_idx);

    // round-trip：导出 DDL 喂回 SQL 导入解析，comment 逐字段一致
    let (tables, refs) =
        parse_sql_import_tables(&ddl).expect("ST-PC-07: round-trip 应解析成功");
    assert!(refs.is_empty(), "ST-PC-07: 无 FK 不产生 references");
    assert_eq!(tables.len(), 1, "ST-PC-07: 应解析出 1 表");
    let t = &tables[0];
    assert_eq!(t.name, "users", "ST-PC-07: 表名一致");
    assert_eq!(
        t.comment, "用户表",
        "ST-PC-07: COMMENT ON TABLE 回填表 comment 一致"
    );
    assert_eq!(t.fields.len(), 2, "ST-PC-07: 字段数一致");
    assert_eq!(
        t.fields[0].comment, "主键",
        "ST-PC-07: COMMENT ON COLUMN 回填列 comment 一致"
    );
    assert_eq!(
        t.fields[1].comment, "it's a note",
        "ST-PC-07: 单引号转义往返（'' → '）一致"
    );
    // 约束 round-trip 不受注释增强影响
    assert!(t.fields[0].primary, "ST-PC-07: 主键置位保持");
    assert!(t.fields[0].increment, "ST-PC-07: SERIAL → increment 保持");
}
