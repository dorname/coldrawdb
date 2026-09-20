//! fix-remote-github-issues-7-18（issue #10/#12）— 画布注释三态 / 表与关系颜色 UT
//!
//! 覆盖：
//!   UT-CR-COMMENT-01  注释渲染口径：三态显示模式 + 空 comment 不留占位
//!                     （CommentDisplay::primary/secondary 纯函数 + 精灵指纹含 comment/mode）
//!   UT-CR-COMMENT-02  显示开关读写 localStorage["cdb.comment-display"]，默认 name+comment
//!   UT-CR-COLOR-01    表边框用色：table.color 非空跟随、为空回退 palette；关系线 ref.color 同理
//!   UT-PB-11          关系 color 非空时渲染用色取自 Reference.color，为空回退 palette.relation
//!
//! 对应规格：`core-01-editor-canvas.md` §5.8 R-CMT-01~04 / R-COLOR-01~03、
//! `core-01a-table-and-field.md` §1.5/§1.6、`core-01b-relationship.md` §4.1。
//! 断言通过后按 logos/spec/test-results.md 追加 reporter 记录。

mod verify_reporter;

use frontend_rs::editor_core::types::Table;
use frontend_rs::editor_core::{CommentDisplay, COMMENT_DISPLAY_STORAGE_KEY};
use frontend_rs::editor_render::{
    relation_stroke_color, table_border_color, table_sprite_fingerprint,
};
use std::time::Instant;

const PANELS: &str = include_str!("../src/editor_panels.rs");
const RENDER: &str = include_str!("../src/editor_render.rs");
const CORE: &str = include_str!("../src/editor_core.rs");

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

fn table_with_comment(tcomment: &str, fcomment: &str) -> Table {
    Table {
        id: "t1".into(),
        name: "users".into(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: tcomment.into(),
        fields: vec![frontend_rs::editor_core::types::Field {
            id: "f1".into(),
            name: "id".into(),
            type_: "INT".into(),
            default: String::new(),
            check: String::new(),
            primary: true,
            unique: false,
            not_null: true,
            increment: true,
            comment: fcomment.into(),
            tag: String::new(),
            dict_code: String::new(),
        }],
        indices: vec![],
        width: None,
        min_height: None,
    }
}

/// UT-CR-COMMENT-01：三态显示模式 + 空 comment 不留占位 + 精灵指纹含 comment/mode。
#[test]
fn ut_cr_comment_01_display_mode_semantics() {
    let start = Instant::now();

    // 1) primary：comment 模式且有 comment → 注释；否则英文名（空 comment 回退英文名）
    assert_eq!(CommentDisplay::Name.primary("users", "用户表"), "users");
    assert_eq!(CommentDisplay::NameComment.primary("users", "用户表"), "users");
    assert_eq!(CommentDisplay::Comment.primary("users", "用户表"), "用户表");
    assert_eq!(
        CommentDisplay::Comment.primary("users", ""),
        "users",
        "R-CMT-04：comment 模式无 comment 必须回退英文名，不留空白"
    );
    assert_eq!(
        CommentDisplay::Comment.primary("users", "  "),
        "users",
        "空白 comment 视同空，回退英文名"
    );

    // 2) secondary：仅 name+comment 模式且 comment 非空（R-CMT-03 空 comment 不渲染不留占位）
    assert_eq!(CommentDisplay::Name.secondary("用户表"), None);
    assert_eq!(CommentDisplay::NameComment.secondary("用户表"), Some("用户表"));
    assert_eq!(CommentDisplay::Comment.secondary("用户表"), None);
    assert_eq!(CommentDisplay::NameComment.secondary(""), None);
    assert_eq!(CommentDisplay::NameComment.secondary("  "), None);

    // 3) R-CMT-03：注释参与精灵指纹——表 comment / 字段 comment / 显示模式变化 → 指纹不同
    let t = table_with_comment("用户表", "主键");
    let base = table_sprite_fingerprint(&t, true, 200, 1, CommentDisplay::NameComment);
    let mut t2 = t.clone();
    t2.comment = "账户表".into();
    assert_ne!(
        table_sprite_fingerprint(&t2, true, 200, 1, CommentDisplay::NameComment),
        base,
        "表 comment 变化必须触发重光栅"
    );
    let mut t3 = t.clone();
    t3.fields[0].comment = "编号".into();
    assert_ne!(
        table_sprite_fingerprint(&t3, true, 200, 1, CommentDisplay::NameComment),
        base,
        "字段 comment 变化必须触发重光栅"
    );
    assert_ne!(
        table_sprite_fingerprint(&t, true, 200, 1, CommentDisplay::Name),
        base,
        "显示模式切换必须触发重光栅（R-CMT-04 切换即重绘）"
    );

    // 4) 渲染锚点：draw_table_body 消费 primary/secondary 绘制表头与字段行注释
    assert!(
        RENDER.contains("comment_mode.primary(&table.name, &table.comment)"),
        "表头主文本必须按显示模式取值"
    );
    assert!(
        RENDER.contains("comment_mode.secondary(&table.comment)"),
        "表头注释副文本必须按显示模式渲染"
    );
    assert!(
        RENDER.contains("comment_mode.primary(&field.name, &field.comment)"),
        "字段主文本必须按显示模式取值"
    );
    assert!(
        RENDER.contains("comment_mode.secondary(&field.comment)"),
        "字段注释副文本必须按显示模式渲染"
    );

    report("UT-CR-COMMENT-01", start);
}

/// UT-CR-COMMENT-02：显示开关读写 localStorage["cdb.comment-display"]，默认 name+comment。
#[test]
fn ut_cr_comment_02_storage_contract() {
    let start = Instant::now();

    // 1) 存储 key 合同
    assert_eq!(COMMENT_DISPLAY_STORAGE_KEY, "cdb.comment-display");

    // 2) from_stored：三态解析 + 缺省/非法回落 name+comment（规格默认，用户已拍板）
    assert_eq!(CommentDisplay::from_stored(None), CommentDisplay::NameComment);
    assert_eq!(
        CommentDisplay::from_stored(Some("name+comment")),
        CommentDisplay::NameComment
    );
    assert_eq!(CommentDisplay::from_stored(Some("name")), CommentDisplay::Name);
    assert_eq!(
        CommentDisplay::from_stored(Some("comment")),
        CommentDisplay::Comment
    );
    assert_eq!(
        CommentDisplay::from_stored(Some("bogus")),
        CommentDisplay::NameComment,
        "非法值必须回落默认 name+comment"
    );

    // 3) as_str 与 from_stored 往返一致
    for mode in [
        CommentDisplay::Name,
        CommentDisplay::NameComment,
        CommentDisplay::Comment,
    ] {
        assert_eq!(CommentDisplay::from_stored(Some(mode.as_str())), mode);
    }

    // 4) next 三态循环：name → name+comment → comment → name
    assert_eq!(CommentDisplay::Name.next(), CommentDisplay::NameComment);
    assert_eq!(CommentDisplay::NameComment.next(), CommentDisplay::Comment);
    assert_eq!(CommentDisplay::Comment.next(), CommentDisplay::Name);

    // 5) label 文案（开关按钮「注释：{label}」）
    assert_eq!(CommentDisplay::Name.label(), "仅英文名");
    assert_eq!(CommentDisplay::NameComment.label(), "英文名+注释");
    assert_eq!(CommentDisplay::Comment.label(), "仅注释");

    // 6) UI 锚点：FloatingControls 开关 testid + localStorage 写入 + R-CMT-04 不落库
    //    （comment_display 不参与 snapshot——EditorStore::snapshot 不含该字段）
    assert!(
        PANELS.contains("data-testid=\"canvas-comment-display\""),
        "画布工具区必须有注释显示开关"
    );
    assert!(
        PANELS.contains("COMMENT_DISPLAY_STORAGE_KEY"),
        "开关必须经统一常量写 localStorage"
    );
    assert!(
        PANELS.contains("local.set_item("),
        "切换必须写回 localStorage"
    );
    let snap_idx = CORE.find("pub fn snapshot(&self").expect("snapshot 存在");
    let snap_block = &CORE[snap_idx..snap_idx + 700.min(CORE.len() - snap_idx)];
    assert!(
        !snap_block.contains("comment_display"),
        "R-CMT-04：注释显示模式不得写入 diagram snapshot（视图偏好不落库）"
    );

    report("UT-CR-COMMENT-02", start);
}

/// UT-CR-COLOR-01：表边框/关系线用色纯函数 + 渲染与 Inspector 入口锚点。
#[test]
fn ut_cr_color_01_border_and_stroke_contract() {
    let start = Instant::now();

    // 1) 表边框：color 非空跟随、为空/空白回退 palette
    assert_eq!(table_border_color("#4fd1c5", "BORDER"), "#4fd1c5");
    assert_eq!(table_border_color("", "BORDER"), "BORDER");
    assert_eq!(table_border_color("   ", "BORDER"), "BORDER");
    assert_eq!(
        table_border_color("rgba(79,209,197,.18)", "BORDER"),
        "rgba(79,209,197,.18)"
    );

    // 2) 关系线：同理
    assert_eq!(relation_stroke_color("#f2b84b", "", "REL"), "#f2b84b");
    assert_eq!(relation_stroke_color("", "", "REL"), "REL");
    assert_eq!(relation_stroke_color("", "#abc", "REL"), "#abc");

    // 3) 渲染锚点：draw_table_body 表边框消费 table_border_color
    assert!(
        RENDER.contains("table_border_color(&table.color, palette.table_border)"),
        "R-COLOR-01：表边框必须经 table_border_color 取色"
    );
    // R-COLOR-03：选中态外环仍用 palette.selected（draw_table_selection 不消费 table.color）
    let sel_idx = RENDER.find("fn draw_table_selection").expect("draw_table_selection 存在");
    let sel_block = &RENDER[sel_idx..sel_idx + 900.min(RENDER.len() - sel_idx)];
    assert!(
        !sel_block.contains("table.color"),
        "R-COLOR-03：选中高亮环不得被自定义表色覆盖"
    );

    // 4) Inspector 入口锚点：表色板 + 关系色板 testid 与「默认」清空项
    assert!(
        PANELS.contains("data-testid=\"inspector-table-color\""),
        "Inspector 必须有表颜色选择器"
    );
    assert!(
        PANELS.contains("data-testid=\"inspector-relation-color\""),
        "Inspector 必须有关系颜色选择器"
    );
    assert!(
        PANELS.contains("(\"\", \"默认\")"),
        "表色板必须有「默认」项（清空回 ''）"
    );

    // 5) 落账链路锚点：表颜色走 Command::SetTableColor（进 UndoRedoContext）；
    //    关系颜色走 on_update_ref_field "color" 分支
    assert!(
        PANELS.contains("Command::SetTableColor"),
        "表颜色变更必须进 UndoRedoContext"
    );
    assert!(
        PANELS.contains("\"color\" => r.color = value"),
        "on_update_ref_field 必须支持 color 字段"
    );

    report("UT-CR-COLOR-01", start);
}

/// UT-PB-11：关系 color 非空时渲染用色取自 Reference.color，为空回退源表色 / palette.relation。
#[test]
fn ut_pb_11_relation_color_render_contract() {
    let start = Instant::now();

    // 1) 纯函数行为（与 UT-CR-COLOR-01 / UT-PB-12 同一合同）
    assert_eq!(relation_stroke_color("#aa8cff", "", "PALETTE_REL"), "#aa8cff");
    assert_eq!(relation_stroke_color("", "", "PALETTE_REL"), "PALETTE_REL");
    assert_eq!(relation_stroke_color("  ", "", "PALETTE_REL"), "PALETTE_REL");
    assert_eq!(
        relation_stroke_color("", "#4fd1c5", "PALETTE_REL"),
        "#4fd1c5",
        "空显式色跟随源表"
    );

    // 2) 渲染锚点：draw_relation 经 relation_stroke_color 取色
    let bez_idx = RENDER.find("fn draw_relation").expect("draw_relation 存在");
    let bez_block = &RENDER[bez_idx..bez_idx + 2500.min(RENDER.len() - bez_idx)];
    assert!(
        bez_block.contains("relation_stroke_color(ref_color, source_table_color, palette.relation)"),
        "关系线必须经 relation_stroke_color(显式, 源表, palette) 取色"
    );
    assert!(
        bez_block.contains("draw_arrow_head"),
        "箭头必须绘制"
    );
    assert!(
        RENDER.contains("if selected { palette.selected } else { stroke }")
            || RENDER.contains("let stroke_main = if selected { palette.selected } else { stroke }"),
        "R-COLOR-03：选中高亮优先级高于自定义色"
    );

    // 3) 调用点锚点：渲染循环把 r.color 与源表色传入 draw_relation
    assert!(
        RENDER.contains("&r.color") && RENDER.contains("&from.color"),
        "渲染循环必须把 Reference.color 与源表色传入绘制"
    );

    // 4) 数据兼容锚点：Reference.color serde default（存量 JSON 无字段 → ''）
    assert!(
        CORE.contains("#[serde(default)]\n        pub color: String,")
            || CORE.contains("#[serde(default)]\r\n        pub color: String,"),
        "Reference.color 必须 serde(default) 向后兼容"
    );

    report("UT-PB-11", start);
}
