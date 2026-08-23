//! implement-unified-prototype-spec-parity D 批 — 快捷键 / Esc 层级 / 网格常量 单元用例
//!
//! Spec: `logos/resources/test/core-KB-shortcut-test-cases.md`（ST-KB-T-01/R-01/ESC-01）
//!       `logos/resources/test/core-CR-canvas-test-cases.md` §1（生产 GRID_SIZE=20）
//! 对齐实现：`editor_panels.rs::{tool_shortcut_for_key, is_shortcut_text_target,
//!            setup_editor_tool_shortcuts, setup_escape_layer_handler}`
//!            + `editor_render.rs::GRID_SIZE`
//!
//! 说明：D 批用例 ID 全部由 e2e（scripts/test-spec-parity-d.mjs）上报账本；
//! 本文件为 cargo 侧纯函数与源码接线支撑，不单独上报（无对应注册 UT ID）。

use frontend_rs::editor_panels::{
    is_shortcut_text_target, tool_shortcut_for_key, ToolShortcut,
};
use frontend_rs::editor_render::GRID_SIZE;

/// ST-KB-T-01 / ST-KB-R-01 支撑：T/R 键映射与修饰键排除
#[test]
fn tool_shortcut_key_mapping_matrix() {
    assert_eq!(
        tool_shortcut_for_key("t", false, false, false),
        Some(ToolShortcut::CreateTable),
        "ST-KB-T-01: 无修饰 t 必须映射建表"
    );
    assert_eq!(
        tool_shortcut_for_key("T", false, false, false),
        Some(ToolShortcut::CreateTable),
        "ST-KB-T-01: 大写 T（Shift）同样映射建表"
    );
    assert_eq!(
        tool_shortcut_for_key("r", false, false, false),
        Some(ToolShortcut::Relationship),
        "ST-KB-R-01: 无修饰 r 必须映射关系工具"
    );
    // 修饰键不拦截：Ctrl+R 刷新、Ctrl+T 新标签页等浏览器行为必须放行
    for (key, ctrl, meta, alt) in [
        ("r", true, false, false),
        ("t", true, false, false),
        ("t", false, true, false),
        ("r", false, false, true),
    ] {
        assert_eq!(
            tool_shortcut_for_key(key, ctrl, meta, alt),
            None,
            "带修饰键不得触发工具快捷键"
        );
    }
    assert_eq!(tool_shortcut_for_key("k", false, false, false), None);
    assert_eq!(tool_shortcut_for_key("Escape", false, false, false), None);
}

/// core-KB §1 既有合同支撑：输入框焦点时快捷键不抢焦点
#[test]
fn shortcut_text_target_matrix() {
    for tag in ["INPUT", "input", "TEXTAREA", "SELECT"] {
        assert!(
            is_shortcut_text_target(tag, false),
            "{tag} 必须判定为文本输入目标"
        );
    }
    assert!(
        is_shortcut_text_target("DIV", true),
        "contentEditable 必须判定为文本输入目标"
    );
    assert!(!is_shortcut_text_target("BODY", false));
    assert!(!is_shortcut_text_target("CANVAS", false));
    assert!(!is_shortcut_text_target("BUTTON", false));
    assert!(!is_shortcut_text_target("DIV", false));
}

/// 验收 §7.5 合同：生产松手网格 20（主原型 12 / 点阵 24 均非法）
#[test]
fn grid_size_matches_production_contract() {
    assert_eq!(GRID_SIZE, 20.0, "生产 GRID_SIZE 必须为 20（core-CR §1 / 验收 §7.5）");
}

/// 源码接线断言：AppRoot 必须注册两个 D 批监听，且 Esc 处理器一次只关一层
#[test]
fn approot_wires_d_batch_shortcuts() {
    let panels_src = include_str!("../src/editor_panels.rs");
    assert!(
        panels_src.contains("setup_editor_tool_shortcuts("),
        "AppRoot 必须注册 T/R 工具快捷键监听"
    );
    assert!(
        panels_src.contains("setup_escape_layer_handler("),
        "AppRoot 必须注册 Esc 浮层层级监听"
    );
    // Esc 层级顺序：命令面板/代码视图让位 → 409 不穿透 → 主模态 → 邀请 → IO 抽屉 → 成员面板 → 关系工具
    let esc_fn = panels_src
        .split("pub fn setup_escape_layer_handler")
        .nth(1)
        .expect("setup_escape_layer_handler 存在");
    let pos = |needle: &str| esc_fn.find(needle).unwrap_or_else(|| panic!("Esc 层级缺失：{needle}"));
    assert!(pos("palette_visible.get_untracked()") < pos("ViewMode::Code"));
    assert!(pos("ViewMode::Code") < pos("conflict.get_untracked().is_some()"));
    assert!(pos("conflict.get_untracked().is_some()") < pos("modal_kind.set(None)"));
    assert!(pos("modal_kind.set(None)") < pos("invite_modal_open.set(false)"));
    assert!(pos("invite_modal_open.set(false)") < pos("on_close_io_drawer()"));
    assert!(pos("on_close_io_drawer()") < pos("room_panel_visible.set(false)"));
    assert!(pos("room_panel_visible.set(false)") < pos("ActiveTool::Relationship"));
    // 只读门控必须复用 editor_is_read_only（ST-KB-VIEWER）
    let tool_fn = panels_src
        .split("pub fn setup_editor_tool_shortcuts")
        .nth(1)
        .expect("setup_editor_tool_shortcuts 存在");
    assert!(
        tool_fn.contains("editor_is_read_only(share_mode, current_room)"),
        "ST-KB-VIEWER: T/R 快捷键必须走 editor_is_read_only 只读门控"
    );
    assert!(
        tool_fn.contains("shortcut_event_is_text_target(ke)"),
        "输入焦点门控必须接入 T/R 快捷键"
    );
    // undo/redo 同样接入输入焦点门控（core-KB §1 既有合同回归）
    let kb_fn = panels_src
        .split("pub fn KeyboardShortcuts")
        .nth(1)
        .expect("KeyboardShortcuts 存在");
    assert!(
        kb_fn.contains("shortcut_event_is_text_target(ke)"),
        "undo/redo 快捷键必须在输入焦点时让位"
    );
}
