//! layout-after-import-command / #23
//!
//! 覆盖：
//!   UT-PB-16 / UT-CR-LAYOUT-01  力导向确定性 / 无重叠 / 孤立不动
//!   ST-PB-09                     palette Action 触发后坐标变化
//!
//! 断言主体在 `src/layout.rs` 与 `src/command_palette.rs` 单测；
//! 本文件复跑关键断言并写入 OpenLogos reporter。

mod verify_reporter;

use frontend_rs::command_palette::{build_palette_items, PaletteKind};
use frontend_rs::editor_core::types::{Field, Reference, Table};
use frontend_rs::layout::{force_directed_layout, LayoutParams};
use std::time::Instant;

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

fn make_table(id: &str, x: f64, y: f64) -> Table {
    Table {
        id: id.into(),
        name: id.into(),
        x,
        y,
        color: String::new(),
        comment: String::new(),
        fields: vec![Field {
            id: format!("{id}_f"),
            name: "id".into(),
            type_: "INT".into(),
            default: String::new(),
            check: String::new(),
            primary: true,
            unique: false,
            not_null: true,
            increment: false,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        }],
        indices: vec![],
        width: None,
        min_height: None,
    }
}

fn make_ref(start: &str, end: &str) -> Reference {
    Reference {
        id: format!("{start}_{end}"),
        name: format!("{start}->{end}"),
        start_table_id: start.into(),
        end_table_id: end.into(),
        start_field_id: format!("{start}_f"),
        end_field_id: format!("{end}_f"),
        type_: "one_to_many".into(),
        on_delete: "RESTRICT".into(),
        on_update: "RESTRICT".into(),
        color: String::new(),
        line_type: String::new(),
        stroke_style: String::new(),
    }
}

/// UT-PB-16 + UT-CR-LAYOUT-01：确定性、无重叠、孤立不动
#[test]
fn ut_pb_16_and_cr_layout_01() {
    let start = Instant::now();
    let params = LayoutParams {
        iterations: 100,
        spacing: 180.0,
    };

    // 确定性
    let tables = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 100.0, 0.0),
        make_table("c", 50.0, 100.0),
    ];
    let refs = vec![make_ref("a", "b"), make_ref("b", "c")];
    let r1 = force_directed_layout(&tables, &refs, &params);
    let r2 = force_directed_layout(&tables, &refs, &params);
    assert_eq!(r1, r2);

    // 无重叠
    let crowded = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 10.0, 10.0),
        make_table("c", 20.0, 20.0),
    ];
    let laid = force_directed_layout(&crowded, &refs, &params);
    for i in 0..laid.len() {
        for j in (i + 1)..laid.len() {
            let dx = laid[i].x - laid[j].x;
            let dy = laid[i].y - laid[j].y;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(dist > 50.0, "距离过近: {dist}");
        }
    }

    // 孤立不动
    let with_iso = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 100.0, 0.0),
        make_table("isolated", 500.0, 500.0),
    ];
    let only_ab = vec![make_ref("a", "b")];
    let out = force_directed_layout(&with_iso, &only_ab, &params);
    let iso = out.iter().find(|t| t.id == "isolated").unwrap();
    assert!((iso.x - 500.0).abs() < 5.0);
    assert!((iso.y - 500.0).abs() < 5.0);

    // 无边原样
    let no_edge = vec![make_table("a", 10.0, 20.0), make_table("b", 30.0, 40.0)];
    assert_eq!(
        force_directed_layout(&no_edge, &[], &params),
        no_edge
    );

    report("UT-PB-16", start);
    report("UT-CR-LAYOUT-01", start);
}

/// ST-PB-09：palette Action 存在且布局后坐标变化
#[test]
fn st_pb_09_palette_layout_trigger() {
    let start = Instant::now();
    let tables = vec![make_table("a", 0.0, 0.0), make_table("b", 5.0, 5.0)];
    let refs = vec![make_ref("a", "b")];

    let items = build_palette_items(&tables, &refs);
    let action = items
        .iter()
        .find(|i| i.id == "action:layout" && i.kind == PaletteKind::Action)
        .expect("须含 action:layout");
    assert_eq!(action.label, "整理布局");

    let laid = force_directed_layout(&tables, &refs, &LayoutParams::default());
    let changed = laid
        .iter()
        .zip(tables.iter())
        .any(|(a, b)| (a.x - b.x).abs() > 0.01 || (a.y - b.y).abs() > 0.01);
    assert!(changed, "整理布局后连通表坐标应变化");

    report("ST-PB-09", start);
}
