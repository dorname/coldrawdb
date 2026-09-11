//! fix-canvas-zoom-invite-comment-resize — UI 面 UT
//!
//! 覆盖：
//!   UT-PU-22  splitter-inspector 拖动纯逻辑（钳制 240–520 / 视口 - 200）+ styles.css 锚点
//!   UT-PU-23  localStorage 预置宽度恢复（合法值直写 / 非法回落默认 330）
//!   UT-PU-24  splitter-list-tree 拖动纯逻辑（钳制 160–420）+ 树列 grid 变量锚点
//!   UT-PC-29  表树注释行锚点（testid 仅在 comment 非空分支 + 单行截断 CSS）
//!   UT-PC-30  Inspector 表/字段注释编辑锚点（受控输入 + blur 落账通路）
//!
//! 对应规格：`core-09` §14、`core-01a` §1.4 / §2.3。
//! 断言通过后按 logos/spec/test-results.md 追加 reporter 记录。

mod verify_reporter;

use frontend_rs::splitter::{drag_width, sanitize_stored_width, SplitterKind};
use std::time::Instant;

const PANELS: &str = include_str!("../src/editor_panels.rs");
const CSS: &str = include_str!("../src/styles.css");
const SPLITTER_RS: &str = include_str!("../src/splitter.rs");

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

/// 取 CSS 规则块（`.x { ... }` 首层括号内容）。
fn css_block(css: &str, selector: &str) -> String {
    let start = css
        .find(selector)
        .unwrap_or_else(|| panic!("缺少 CSS 规则：{selector}"));
    let open = css[start..]
        .find('{')
        .map(|i| start + i)
        .expect("规则块必须有 {");
    let close = css[open..]
        .find('}')
        .map(|i| open + i)
        .expect("规则块必须有 }");
    css[open..=close].to_string()
}

/// UT-PU-22：inspector 拖动钳制 + 生产 CSS 锚点（变量消费 / is-resizing 禁过渡 / testid）。
#[test]
fn ut_pu_22_splitter_inspector_drag_and_css_anchors() {
    let start = Instant::now();
    // ── 拖动序列模拟：dx 为相对 pointerdown 起点的累积位移，每次 pointermove 即时产出钳制后宽度 ──
    // Inspector 位于右侧面板左缘：width = start_w - dx（core-09 §14.2 第 2 条，
    // fix-canvas-grid-splitter-listview-comment 修正方向）；视口 1440 → max = min(520, 1240) = 520；min = 240
    let seq: [(i64, u32); 6] = [
        (-50, 380),
        (-100, 430),
        (-190, 520),   // 左拖 190 → 520 达上限
        (-10_000, 520), // 左拖越过 520 边界停在边界值
        (90, 240),     // 右拖 90 → 240 达下限
        (10_000, 240), // 右拖越过 240 边界停在边界值
    ];
    for (dx, expect) in seq {
        assert_eq!(
            drag_width(SplitterKind::Inspector, 330, dx, 1440),
            expect,
            "UT-PU-22: dx={dx} 应钳制为 {expect}"
        );
    }
    // 第 3 列随变量变化：.cdb-main 栅格消费 --cdb-inspector-w
    assert!(
        CSS.contains("grid-template-columns: var(--cdb-toolrail-w) minmax(0, 1fr) var(--cdb-inspector-w)"),
        "UT-PU-22: .cdb-main 第 3 列必须消费 var(--cdb-inspector-w)"
    );
    // 拖动期禁 160ms 过渡
    let resizing = css_block(CSS, ".cdb-main.is-resizing");
    assert!(
        resizing.contains("transition: none"),
        "UT-PU-22: .cdb-main.is-resizing 必须 transition: none，实际：{resizing}"
    );
    // 分隔条本体与 testid
    assert!(
        CSS.contains(".cdb-splitter--inspector"),
        "UT-PU-22: styles.css 必须含 .cdb-splitter--inspector 定位规则"
    );
    assert!(
        SPLITTER_RS.contains("splitter-inspector"),
        "UT-PU-22: splitter 组件必须提供 splitter-inspector testid"
    );
    // 拖动期直写 CSS 变量（绕过响应式），禁过渡期间内联变量即时生效
    assert!(
        SPLITTER_RS.contains("set_property(kind.css_var()"),
        "UT-PU-22: pointermove 必须直写根 CSS 变量（无 Leptos 重渲染）"
    );
    report("UT-PU-22", start);
}

/// UT-PU-23：localStorage 预置宽度恢复口径。
#[test]
fn ut_pu_23_stored_width_restore() {
    let start = Instant::now();
    assert_eq!(
        sanitize_stored_width(Some("400"), SplitterKind::Inspector, 1440),
        400,
        "UT-PU-23: 预置 cdb.inspector.width=400 应恢复 400"
    );
    // 非法值（非数字 / 空串 / 越界）与缺失一律回落默认 330
    for raw in ["abc", "", "999", "100", "-1"] {
        assert_eq!(
            sanitize_stored_width(Some(raw), SplitterKind::Inspector, 1440),
            330,
            "UT-PU-23: 预置非法值 {raw:?} 应回落默认 330"
        );
    }
    assert_eq!(
        sanitize_stored_width(None, SplitterKind::Inspector, 1440),
        330,
        "UT-PU-23: 缺失 key 应回落默认 330"
    );
    // 520 硬上限优先于视口 - 200：1000px 视口下 600 超界 → 默认
    assert_eq!(
        sanitize_stored_width(Some("600"), SplitterKind::Inspector, 1000),
        330,
        "UT-PU-23: 1000px 视口下 600 超 520 硬上限应回落默认 330"
    );
    assert_eq!(
        sanitize_stored_width(Some("510"), SplitterKind::Inspector, 1000),
        510,
        "UT-PU-23: 1000px 视口下 510 ≤ 520 合法"
    );
    report("UT-PU-23", start);
}

/// UT-PU-24：list-tree 拖动钳制 160–420 + 树列 grid 变量锚点 + localStorage key。
#[test]
fn ut_pu_24_splitter_list_tree_drag_and_css_anchors() {
    let start = Instant::now();
    // 拖动序列：默认 230，右拖 +300 → 530 超界停 420；左拖 -300 → 160
    assert_eq!(
        drag_width(SplitterKind::ListTree, 230, 300, 1440),
        420,
        "UT-PU-24: 树列右拖超界应停 420"
    );
    assert_eq!(
        drag_width(SplitterKind::ListTree, 230, -300, 1440),
        160,
        "UT-PU-24: 树列左拖超界应停 160"
    );
    assert_eq!(
        drag_width(SplitterKind::ListTree, 230, 100, 1440),
        330,
        "UT-PU-24: 树列 +100 应到 330"
    );
    // localStorage key 与默认宽
    assert!(
        SPLITTER_RS.contains("cdb.list-tree.width"),
        "UT-PU-24: localStorage key 必须为 cdb.list-tree.width"
    );
    assert_eq!(
        sanitize_stored_width(None, SplitterKind::ListTree, 1440),
        230,
        "UT-PU-24: 树列缺失 key 应回落默认 230"
    );
    // 树列 grid 消费 CSS 变量（容器重排即时跟随）
    let body = css_block(CSS, ".cdb-list-view-body");
    assert!(
        body.contains("grid-template-columns: var(--cdb-list-tree-w, 230px) minmax(0, 1fr)"),
        "UT-PU-24: .cdb-list-view-body 第 1 列必须消费 var(--cdb-list-tree-w)，实际：{body}"
    );
    // 拖动期直写容器内联 grid-template-columns（即时性锚点，§14.4.3）
    assert!(
        SPLITTER_RS.contains("set_property(\"grid-template-columns\""),
        "UT-PU-24: 拖动期必须直写容器内联 grid-template-columns"
    );
    assert!(
        SPLITTER_RS.contains("splitter-list-tree"),
        "UT-PU-24: splitter 组件必须提供 splitter-list-tree testid"
    );
    report("UT-PU-24", start);
}

/// UT-PC-29：表树注释行锚点（非空分支渲染 + 单行截断 CSS）。
#[test]
fn ut_pc_29_list_tree_comment_anchor() {
    let start = Instant::now();
    assert!(
        PANELS.contains("list-table-comment-"),
        "UT-PC-29: 表树注释行 testid list-table-comment-{{table_id}} 必须存在"
    );
    assert!(
        PANELS.contains("!t.comment.is_empty()"),
        "UT-PC-29: 注释行必须仅在 t.comment 非空分支渲染（无注释不占位）"
    );
    let block = css_block(CSS, ".cdb-list-tree-comment");
    for required in ["overflow: hidden", "text-overflow: ellipsis", "white-space: nowrap"] {
        assert!(
            block.contains(required),
            "UT-PC-29: 注释行 CSS 必须含单行截断规则 {required}，实际：{block}"
        );
    }
    report("UT-PC-29", start);
}

/// UT-PC-30：Inspector 表/字段注释编辑锚点（受控输入 + blur 落账通路）。
#[test]
fn ut_pc_30_inspector_comment_anchors() {
    let start = Instant::now();
    // 表注释：受控输入（prop:value 绑定当前表 comment）+ blur 落账
    assert!(
        PANELS.contains("data-testid=\"inspector-table-comment\""),
        "UT-PC-30: inspector-table-comment 输入必须存在"
    );
    assert!(
        PANELS.contains("prop:value=table_comment"),
        "UT-PC-30: 表注释必须受控绑定当前表 Table.comment"
    );
    assert!(
        PANELS.contains("on_set_comment(table_id_for_comment.clone(), event_target_value(&ev))"),
        "UT-PC-30: 表注释 blur 必须走落账通路（仿 on_rename_table）"
    );
    // 字段卡注释（新增双轨）
    assert!(
        PANELS.contains("inspector-field-comment-"),
        "UT-PC-30: Inspector 字段卡注释 testid inspector-field-comment-{{field_id}} 必须存在"
    );
    // ListView 字段网格注释（既有锚点保留）
    assert!(
        PANELS.contains("list-field-comment-"),
        "UT-PC-30: list-field-comment- 既有锚点必须保留"
    );
    // 落账通路存在（store 写 → schedule_save）
    assert!(
        PANELS.contains("let on_set_table_comment = {"),
        "UT-PC-30: 表注释落账 handler 必须存在"
    );
    assert!(
        PANELS.contains("let on_set_field_comment = {"),
        "UT-PC-30: 字段注释落账 handler 必须存在"
    );
    report("UT-PC-30", start);
}
