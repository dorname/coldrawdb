//! fix-open-issues-19-22 — 画布拖动 click 抑制 / 表宽自适应 / 关系线型色
//!
//! 覆盖：
//!   UT-CR-CLICK-01  有效拖动后 suppress click
//!   UT-CR-WIDTH-01  None/0=auto；正数固定宽
//!   UT-CR-WIDTH-02  长表名撑宽 ∈ (230, 480]
//!   UT-CR-COLOR-02  picker hex → table_border_color
//!   UT-PB-12        描边色：显式 > 源表 > palette
//!   UT-PB-13        正交折线消费 pick_port_sides
//!   UT-PB-14        dashed/solid dash 数组
//!   UT-PB-15        选中态密度降噪 alpha

mod verify_reporter;

use frontend_rs::editor_core::types::{Field, Table};
use frontend_rs::editor_core::CommentDisplay;
use frontend_rs::editor_render::{
    arm_suppress_next_click, calc_orthogonal_path, estimate_content_width, pick_port_sides,
    relation_opacity, relation_stroke_color, resolve_table_width, should_suppress_click_after_drag,
    stroke_dash_for_style, table_border_color, take_suppress_next_click, FieldPortSide,
    DRAG_THRESHOLD, TABLE_WIDTH, TABLE_WIDTH_MAX,
};
use std::time::Instant;

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

fn make_table(name: &str, width: Option<u32>) -> Table {
    Table {
        id: "t1".into(),
        name: name.into(),
        x: 100.0,
        y: 100.0,
        color: String::new(),
        comment: String::new(),
        fields: vec![Field {
            id: "f1".into(),
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
        width,
        min_height: None,
    }
}

/// UT-CR-CLICK-01：位移 ≥ DRAG_THRESHOLD → suppress；< 阈值 → 不 suppress；
/// arm/take 标志可消费一次。
#[test]
fn ut_cr_click_01_suppress_after_drag() {
    let start = Instant::now();
    assert!(
        should_suppress_click_after_drag(DRAG_THRESHOLD, 0.0),
        "位移 = 阈值应 suppress"
    );
    assert!(
        should_suppress_click_after_drag(5.0, 0.0),
        "位移 > 阈值应 suppress"
    );
    assert!(
        !should_suppress_click_after_drag(3.0, 0.0),
        "位移 < 阈值不 suppress"
    );
    assert!(
        !should_suppress_click_after_drag(2.0, 2.0),
        "欧氏 < 阈值不 suppress"
    );

    // arm / take 一次消费
    let _ = take_suppress_next_click(); // 清零
    assert!(!take_suppress_next_click());
    arm_suppress_next_click();
    assert!(take_suppress_next_click());
    assert!(!take_suppress_next_click(), "第二次 take 应为 false");

    report("UT-CR-CLICK-01", start);
}

/// UT-CR-WIDTH-01：None/Some(0)=auto ∈ [230,480]；Some(350)=350。
#[test]
fn ut_cr_width_01_auto_and_fixed() {
    let start = Instant::now();
    let mode = CommentDisplay::NameComment;

    let none_w = resolve_table_width(&make_table("users", None), mode);
    assert!(
        (TABLE_WIDTH..=TABLE_WIDTH_MAX).contains(&none_w),
        "None → auto 夹紧，got {none_w}"
    );

    let zero_w = resolve_table_width(&make_table("users", Some(0)), mode);
    assert!(
        (TABLE_WIDTH..=TABLE_WIDTH_MAX).contains(&zero_w),
        "Some(0) → auto 夹紧，got {zero_w}"
    );
    assert_eq!(none_w, zero_w, "None 与 Some(0) 同语义");

    let fixed = resolve_table_width(&make_table("users", Some(350)), mode);
    assert_eq!(fixed, 350.0, "正数 width 原样尊重");

    report("UT-CR-WIDTH-01", start);
}

/// UT-CR-WIDTH-02：长表名撑宽 > 230 且 ≤ 480。
#[test]
fn ut_cr_width_02_long_name() {
    let start = Instant::now();
    let mode = CommentDisplay::NameComment;
    let name = "asset_v2.virtualization_cluster_profile";
    let t = make_table(name, None);
    let w = resolve_table_width(&t, mode);
    let estimated = estimate_content_width(&t, mode);
    assert!(
        estimated > TABLE_WIDTH,
        "长表名估算应 > 230，got {estimated}"
    );
    assert!(
        w > TABLE_WIDTH && w <= TABLE_WIDTH_MAX,
        "有效宽应 ∈ (230, 480]，got {w}"
    );
    report("UT-CR-WIDTH-02", start);
}

/// UT-CR-COLOR-02：合法 hex 写入后 table_border_color 派生；清空回退 palette。
#[test]
fn ut_cr_color_02_picker_hex() {
    let start = Instant::now();
    assert_eq!(table_border_color("#ff6600", "BORDER"), "#ff6600");
    assert_eq!(table_border_color("", "BORDER"), "BORDER");
    assert_eq!(table_border_color("  ", "BORDER"), "BORDER");
    // 锚点：Inspector 存在 color picker
    let panels = include_str!("../src/editor_panels.rs");
    assert!(
        panels.contains("data-testid=\"inspector-table-color-picker\""),
        "必须存在 inspector-table-color-picker"
    );
    report("UT-CR-COLOR-02", start);
}

/// UT-PB-12：显式色 > 源表色 > palette。
#[test]
fn ut_pb_12_stroke_color_priority() {
    let start = Instant::now();
    assert_eq!(
        relation_stroke_color("#fff", "#aaa", "PAL"),
        "#fff",
        "显式优先"
    );
    assert_eq!(
        relation_stroke_color("", "#4fd1c5", "PAL"),
        "#4fd1c5",
        "空显式 → 源表色"
    );
    assert_eq!(
        relation_stroke_color("", "", "PAL"),
        "PAL",
        "皆空 → palette"
    );
    assert_eq!(
        relation_stroke_color("  ", "  ", "PAL"),
        "PAL",
        "空白视同空"
    );
    report("UT-PB-12", start);
}

/// UT-PB-13：目标在左时正交折线锚点 = 左出右进。
#[test]
fn ut_pb_13_orthogonal_sides() {
    let start = Instant::now();
    let mut from = make_table("A", Some(230));
    from.id = "a".into();
    from.x = 400.0;
    from.fields[0].id = "fa".into();
    let mut to = make_table("B", Some(230));
    to.id = "b".into();
    to.x = 100.0; // 目标在左
    to.fields[0].id = "fb".into();

    let (out, inn) = pick_port_sides(&from, &to);
    assert_eq!(out, FieldPortSide::Start, "目标在左 → 左出");
    assert_eq!(inn, FieldPortSide::End, "目标在左 → 右进");

    let pts = calc_orthogonal_path(&from, "fa", &to, "fb");
    assert_eq!(pts.len(), 4, "正交折线 4 顶点");
    // 起点 x = from.x（左缘），终点 x = to.x + width（右缘）
    assert!(
        (pts[0].0 - from.x).abs() < 0.01,
        "起点应为源表左缘，got {}",
        pts[0].0
    );
    assert!(
        (pts[3].0 - (to.x + 230.0)).abs() < 0.01,
        "终点应为目标表右缘，got {}",
        pts[3].0
    );
    report("UT-PB-13", start);
}

/// UT-PB-14：dashed → dash 非空；solid → 空。
#[test]
fn ut_pb_14_dash_array() {
    let start = Instant::now();
    assert!(!stroke_dash_for_style("dashed").is_empty());
    assert!(stroke_dash_for_style("solid").is_empty());
    assert!(stroke_dash_for_style("").is_empty());
    report("UT-PB-14", start);
}

/// UT-PB-15：非相关线 alpha≤0.25；相关/无选中 = 1.0。
#[test]
fn ut_pb_15_relation_opacity() {
    let start = Instant::now();
    assert_eq!(
        relation_opacity("r1", "t1", "t2", &[], None, None),
        1.0,
        "无选中"
    );
    assert_eq!(
        relation_opacity("r1", "t1", "t2", &[], Some("t1"), None),
        1.0,
        "相关表选中"
    );
    assert_eq!(
        relation_opacity("r1", "t1", "t2", &[], Some("t9"), None),
        0.25,
        "非相关表选中"
    );
    assert_eq!(
        relation_opacity("r1", "t1", "t2", &[], None, Some("r1")),
        1.0,
        "自身关系选中"
    );
    assert_eq!(
        relation_opacity("r1", "t1", "t2", &[], None, Some("r9")),
        0.25,
        "其他关系选中"
    );
    assert!(
        relation_opacity("r1", "t1", "t2", &["t9".into()], None, None) <= 0.25
    );
    report("UT-PB-15", start);
}
