//! ST-PC-10: 双路径（SQL dump vs import/connect）对称（fix-open-issues-26-33 / #29）
//!
//! Spec: `logos/resources/test/core-PC-import-export-test-cases.md` ST-PC-10
//! 场景：同一 PG schema（3 表、2 条 FK、schema 限定 COMMENT ON）分别经
//!   路径 A：SQL Tab 粘贴 pg_dump 风格 DDL → `parse_sql_import_tables`
//!   路径 B：数据库 Tab import/connect 结构化响应 → `parse_bridge_import_tables`
//! 断言：两条路径的表数相等、关系数相等、非空表/列 comment 数一致
//!   （允许实体 ID 与坐标差异；host 集成沿用 ST-PC-07 先例）

use frontend_rs::editor_core::types::{Reference, Table};
use frontend_rs::editor_data_access::{ImportConnectField, ImportConnectReference, ImportConnectTable};
use frontend_rs::editor_panels::{parse_bridge_import_tables, parse_sql_import_tables};

/// pg_dump 风格 DDL：schema 限定 CREATE TABLE / REFERENCES / COMMENT ON（#29 加固面）
const DDL: &str = r#"
CREATE TABLE public.users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
COMMENT ON TABLE public.users IS '用户表';
COMMENT ON COLUMN public.users.id IS '主键';

CREATE TABLE public.orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES public.users(id),
    status TEXT
);
COMMENT ON COLUMN public.orders.status IS '状态';

CREATE TABLE public.order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES public.orders(id)
);
"#;

fn field(name: &str, type_: &str, primary: bool, not_null: bool, comment: &str) -> ImportConnectField {
    ImportConnectField {
        name: name.to_string(),
        type_: type_.to_string(),
        primary,
        unique: false,
        not_null,
        increment: primary,
        default: String::new(),
        comment: comment.to_string(),
    }
}

/// 路径 B：同构 schema 的 import/connect 结构化响应（名址 references，同 bridge 契约 core-03 §13.1）
fn bridge_tables() -> Vec<ImportConnectTable> {
    vec![
        ImportConnectTable {
            name: "users".to_string(),
            comment: "用户表".to_string(),
            fields: vec![
                field("id", "SERIAL", true, true, "主键"),
                field("name", "TEXT", false, true, ""),
            ],
        },
        ImportConnectTable {
            name: "orders".to_string(),
            comment: String::new(),
            fields: vec![
                field("id", "SERIAL", true, true, ""),
                field("user_id", "INTEGER", false, false, ""),
                field("status", "TEXT", false, false, "状态"),
            ],
        },
        ImportConnectTable {
            name: "order_items".to_string(),
            comment: String::new(),
            fields: vec![
                field("id", "SERIAL", true, true, ""),
                field("order_id", "INTEGER", false, false, ""),
            ],
        },
    ]
}

fn bridge_references() -> Vec<ImportConnectReference> {
    let mk = |t: &str, c: &str, rt: &str, rc: &str| ImportConnectReference {
        table: t.to_string(),
        column: c.to_string(),
        ref_table: rt.to_string(),
        ref_column: rc.to_string(),
        on_delete: String::new(),
        on_update: String::new(),
    };
    vec![
        mk("orders", "user_id", "users", "id"),
        mk("order_items", "order_id", "orders", "id"),
    ]
}

fn comment_counts(tables: &[Table]) -> (usize, usize) {
    let table_comments = tables.iter().filter(|t| !t.comment.is_empty()).count();
    let col_comments = tables
        .iter()
        .flat_map(|t| t.fields.iter())
        .filter(|f| !f.comment.is_empty())
        .count();
    (table_comments, col_comments)
}

#[test]
fn st_pc_10_import_path_symmetry() {
    // 路径 A：SQL dump 粘贴导入
    let (a_tables, a_refs): (Vec<Table>, Vec<Reference>) =
        parse_sql_import_tables(DDL).expect("ST-PC-10: 路径 A（SQL dump）应解析成功");
    // 路径 B：import/connect 结构化响应导入
    let (b_tables, b_refs): (Vec<Table>, Vec<Reference>) =
        parse_bridge_import_tables(&bridge_tables(), &bridge_references())
            .expect("ST-PC-10: 路径 B（import/connect）应解析成功");

    // 表数相等
    assert_eq!(
        a_tables.len(),
        b_tables.len(),
        "ST-PC-10: 双路径表数应相等（A={} B={}）",
        a_tables.len(),
        b_tables.len()
    );
    assert_eq!(a_tables.len(), 3, "ST-PC-10: fixture 应为 3 表");

    // 关系数相等（#29 主诉：import/connect 不再静默丢失 FK）
    assert_eq!(
        a_refs.len(),
        b_refs.len(),
        "ST-PC-10: 双路径关系数应相等（A={} B={}）",
        a_refs.len(),
        b_refs.len()
    );
    assert_eq!(a_refs.len(), 2, "ST-PC-10: fixture 应为 2 条关系");

    // 非空表/列 comment 数一致（schema 限定 COMMENT ON 在路径 A 不丢，见 UT-PC-33）
    let (a_tc, a_cc) = comment_counts(&a_tables);
    let (b_tc, b_cc) = comment_counts(&b_tables);
    assert_eq!(
        (a_tc, a_cc),
        (b_tc, b_cc),
        "ST-PC-10: 非空 comment 数应一致（A 表 {a_tc}/列 {a_cc}，B 表 {b_tc}/列 {b_cc}）"
    );
    assert_eq!((a_tc, a_cc), (1, 2), "ST-PC-10: fixture 应为 1 表注释 + 2 列注释");
}
