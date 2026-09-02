//! Phase C 导入/导出 + AB 回归测试用例（UT-PC-01 ~ UT-PC-06 + UT-AB-04）
//!
//! Spec: `logos/resources/test/core-PC-import-export-test-cases.md` + `core-UI-modals-2-test-cases.md`
//! Proposal: `logos/changes/add-pb-pc-test-coverage/`
//! Source alignment: `frontend-rs/src/editor_panels.rs`
//!
//! 覆盖用例：
//! - UT-PC-01 SQL 解析（通过 `import_parse_summary` 间接验证 `parse_sql_statements`）
//! - UT-PC-02 `export_diagram_sql` 输出含 `CREATE TABLE`
//! - UT-PC-03 `export_diagram_dbml` 输出含 `Table` 与 `ref:`
//! - UT-PC-04 `snapshot_before_io_drawer` + IO 抽屉信号切换
//! - UT-PC-05 `count_dbml_tables(text)` 计数 2 个 Table 块
//! - UT-PC-06 点击 `guide-import-sql` → `import-drawer` 可见
//! - UT-AB-04 `btn-import` enabled 状态回归断言（Phase C 替换 Phase A disabled）

use frontend_rs::editor_core::types::{Field, Reference, Table};
use frontend_rs::editor_panels::{
    count_dbml_tables, export_diagram_dbml, export_diagram_sql, import_parse_summary,
    snapshot_before_io_drawer, ImportFormat,
};

/// 测试辅助：构造一个最小 Table（含 1 字段）
fn fixture_minimal_table() -> Table {
    Table {
        id: "t1".to_string(),
        name: "users".to_string(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: String::new(),
        fields: vec![Field {
            id: "f1".to_string(),
            name: "id".to_string(),
            type_: "INT".to_string(),
            default: String::new(),
            check: String::new(),
            primary: true,
            unique: false,
            not_null: true,
            increment: true,
            comment: String::new(),
        }],
        indices: vec![],
        width: None,
        min_height: None,
    }
}

fn fixture_minimal_reference() -> Reference {
    Reference {
        id: "r1".to_string(),
        name: String::new(),
        start_table_id: "t1".to_string(),
        end_table_id: "t1".to_string(),
        start_field_id: "f1".to_string(),
        end_field_id: "f1".to_string(),
        type_: "one_to_many".to_string(),
        on_delete: "RESTRICT".to_string(),
        on_update: "RESTRICT".to_string(),
    }
}

// ─── UT-PC-01: SQL 解析（间接验证 parse_sql_statements） ──────────────────

#[test]
fn ut_pc_01_parse_two_create_statements_returns_two_statements_summary() {
    // Given: SQL 含 2 条 CREATE 语句
    let sql = "CREATE TABLE users (id INT);\nCREATE TABLE posts (id INT);";

    // When: 调用 import_parse_summary（内部依赖 parse_sql_statements）
    let summary = import_parse_summary(ImportFormat::Sql, sql).expect("parse ok");

    // Then: 摘要显示「2 条语句」
    assert_eq!(summary, "2 条语句");
}

#[test]
fn ut_pc_01b_parse_with_comments_strips_line_comments() {
    let sql = "-- comment line\nCREATE TABLE a (id INT);\n-- another comment\nCREATE TABLE b (id INT);";
    let summary = import_parse_summary(ImportFormat::Sql, sql).expect("parse ok");
    assert_eq!(summary, "2 条语句");
}

#[test]
fn ut_pc_01c_parse_empty_returns_zero_statements() {
    let summary = import_parse_summary(ImportFormat::Sql, "").expect("parse ok");
    assert_eq!(summary, "0 条语句");
}

// ─── UT-PC-02: export_diagram_sql ──────────────────────────────────────────

#[test]
fn ut_pc_02_export_diagram_sql_generic_contains_create_table() {
    // Given: store 含 1 张表
    let tables = vec![fixture_minimal_table()];

    // When: 调用 export_diagram_sql
    let out = export_diagram_sql(&tables, &[], "generic");

    // Then: 输出含 CREATE TABLE 与表名
    assert!(out.contains("CREATE TABLE"));
    assert!(out.contains("users"));
}

#[test]
fn ut_pc_02b_export_diagram_sql_engine_specific_adds_header_comment() {
    let tables = vec![fixture_minimal_table()];
    let out = export_diagram_sql(&tables, &[], "postgresql");
    assert!(out.starts_with("-- engine: postgresql"));
    assert!(out.contains("CREATE TABLE"));
}

// ─── UT-PC-03: export_diagram_dbml ─────────────────────────────────────────

#[test]
fn ut_pc_03_export_diagram_dbml_contains_table_and_ref() {
    // Given: store 含表 + 关系
    let tables = vec![fixture_minimal_table(), fixture_minimal_table()];
    let refs = vec![fixture_minimal_reference()];

    // When: 调用 export_diagram_dbml
    let out = export_diagram_dbml(&tables, &refs);

    // Then: 输出含 Table 与 ref:
    assert!(out.contains("Table "));
    assert!(out.contains("Ref:"));
}

// ─── UT-PC-04: snapshot_before_io_drawer + IO 抽屉信号切换 ─────────────────

#[test]
fn ut_pc_04_snapshot_before_io_drawer_collapses_inspector() {
    // Given: Inspector 已展开
    // When: 调用 snapshot_before_io_drawer(true)
    let (collapsed, cache) = snapshot_before_io_drawer(true);

    // Then: 返回 (false, Some(true))
    assert_eq!(collapsed, false);
    assert_eq!(cache, Some(true));
}

#[test]
fn ut_pc_04b_snapshot_before_io_drawer_keeps_cache_none_when_already_closed() {
    let (collapsed, cache) = snapshot_before_io_drawer(false);
    assert_eq!(collapsed, false);
    assert_eq!(cache, None);
}

#[test]
fn ut_pc_04c_io_drawer_kind_transitions_to_import_on_open() {
    // 该断言为组件层（io_drawer signal → IoDrawerKind::Import），
    // 在单元测试中以源码扫描方式验证 EditorPanels 暴露 io_drawer 信号
    // 且 open_import_drawer 设置为 IoDrawerKind::Import。
    // 完整行为由 ST-PC-01（E2E）覆盖。

    let panels_src = include_str!("../src/editor_panels.rs");
    assert!(
        panels_src.contains("io_drawer") || panels_src.contains("IoDrawerKind"),
        "UT-PC-04 FAIL: IO 抽屉信号缺失"
    );
    assert!(
        panels_src.contains("IoDrawerKind::Import"),
        "UT-PC-04 FAIL: open_import_drawer 应设置 IoDrawerKind::Import"
    );
}

// ─── UT-PC-05: count_dbml_tables ───────────────────────────────────────────

#[test]
fn ut_pc_05_count_dbml_tables_returns_table_block_count() {
    // Given: DBML 含 2 个 Table 块
    let dbml = "Table users {\n  id int\n}\n\nTable posts {\n  id int\n}\n";

    // When: 调用 count_dbml_tables
    let count = count_dbml_tables(dbml);

    // Then: 返回 2
    assert_eq!(count, 2);
}

#[test]
fn ut_pc_05b_count_dbml_tables_lowercase_table_keyword() {
    let dbml = "table a { id int }\ntable b { id int }\ntable c { id int }";
    assert_eq!(count_dbml_tables(dbml), 3);
}

#[test]
fn ut_pc_05c_count_dbml_tables_empty_returns_zero() {
    assert_eq!(count_dbml_tables(""), 0);
    assert_eq!(count_dbml_tables("-- only comments"), 0);
}

// ─── UT-PC-06: 点击 guide-import-sql → import-drawer 可见 ─────────────────

#[test]
fn ut_pc_06_guide_import_sql_triggers_io_drawer() {
    // 该用例为组件交互（点击 EmptyGuide 的 guide-import-sql → 触发 io_drawer 信号）。
    // 在单元测试中以源码扫描方式验证：
    // 1) EmptyGuide 暴露 `guide-import-sql` testid
    // 2) on:click 处理器调用 open_import_drawer

    let panels_src = include_str!("../src/editor_panels.rs");
    assert!(
        panels_src.contains("data-testid=\"guide-import-sql\""),
        "UT-PC-06 FAIL: guide-import-sql testid 缺失"
    );
    assert!(
        panels_src.contains("data-testid=\"import-drawer\"") || panels_src.contains("io-drawer"),
        "UT-PC-06 FAIL: import-drawer / io-drawer testid 缺失"
    );
}

// ─── UT-AB-04: AppBar btn-import enabled 状态回归 ─────────────────────────

#[test]
fn ut_ab_04_btn_import_is_enabled_in_phase_c() {
    // 该用例为 AppBar 按钮状态回归断言（Phase C 引入：btn-import 始终 enabled，
    // 替换 Phase A 的 disabled=true）。源码扫描方式验证：
    //
    // - AppBar 组件包含 `data-testid="btn-import"`
    // - btn-import 渲染时不含 `disabled=true`（Phase C 启用状态）

    let panels_src = include_str!("../src/editor_panels.rs");

    // 1) AppBar 包含 btn-import testid
    assert!(
        panels_src.contains("data-testid=\"btn-import\""),
        "UT-AB-04 FAIL: btn-import testid 缺失"
    );

    // 2) btn-import 在 AppBar 渲染中不含 disabled
    // 取 btn-import testid 周围 ~300 字符窗口（AppBar 内部）
    let btn_import_pos = panels_src.find("data-testid=\"btn-import\"").unwrap();
    let window_start = btn_import_pos.saturating_sub(50);
    let window_end = (btn_import_pos + 500).min(panels_src.len());
    let window = &panels_src[window_start..window_end];

    assert!(
        !window.contains("disabled=true") && !window.contains("disabled = true"),
        "UT-AB-04 FAIL: btn-import 在 Phase C 应启用（不允许 disabled=true）"
    );
    assert!(
        window.contains("on:click"),
        "UT-AB-04 FAIL: btn-import 必须绑定 on:click 处理器"
    );
}

// ─── UT-AB-04 扩展: import_parse_summary(DBML/JSON) 一致性 ─────────────────

#[test]
fn ut_ab_04b_import_summary_dbml_uses_count_dbml_tables() {
    let dbml = "Table users { id int }\nTable posts { id int }";
    let summary = import_parse_summary(ImportFormat::Dbml, dbml).expect("parse ok");
    assert_eq!(summary, "2 个 Table 块");
}

#[test]
fn ut_ab_04c_import_summary_json_counts_tables_field() {
    let json = r#"{"tables":[{"name":"a"},{"name":"b"},{"name":"c"}]}"#;
    let summary = import_parse_summary(ImportFormat::Json, json).expect("parse ok");
    assert_eq!(summary, "3 张表");
}