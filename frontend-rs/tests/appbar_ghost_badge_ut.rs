//! fix-remote-github-issues-7-18（issue #13/#14/#15）— AppBar ghost / room-badge 截断 UT
//!
//! 覆盖：
//!   UT-PE-AB-01  返回按钮 ghost 样式锚点：`.cdb-btn` inline-flex 居中 +
//!                `.cdb-btn--ghost` 透明边框 + hover 底色 + `.cdb-back-to-rooms` 高度 ≥36px
//!   UT-PE-AB-02  room-badge 截断规则锚点：`.cdb-room-badge strong` 三件套 +
//!                badge 容器 max-width + AppBar 中部 min-width:0 + 生产 DOM title 全文兜底
//!
//! 对应规格：`core-05-top-menu-modals.md`「AppBar 统一工作空间信息分层补充」R-AB-01~09。
//! 断言通过后按 logos/spec/test-results.md 追加 reporter 记录。

mod verify_reporter;

use std::time::Instant;

const CSS: &str = include_str!("../src/styles.css");
const PANELS: &str = include_str!("../src/editor_panels.rs");

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

/// 取 CSS 规则块（行首 `.x { ... }`）的并集：同一选择器可能出现多次
/// （如媒体查询覆盖、`.cdb-app-bar__status .cdb-save-state` 这类复合选择器
/// 也含目标子串），只收集行首精确匹配的所有块并拼接。
fn css_block(css: &str, selector: &str) -> String {
    let mut out = String::new();
    let mut from = 0;
    loop {
        let Some(rel) = css[from..].find(selector) else { break };
        let start = from + rel;
        from = start + selector.len();
        // 必须是行首规则（前一个字符为换行），排除复合选择器尾部命中
        if start > 0 && !css[..start].ends_with('\n') {
            continue;
        }
        let open = css[start..]
            .find('{')
            .map(|i| start + i)
            .expect("规则块必须有 {");
        let close = css[open..]
            .find('}')
            .map(|i| open + i)
            .expect("规则块必须有 }");
        out.push_str(&css[open..=close]);
        out.push('\n');
    }
    assert!(!out.is_empty(), "缺少 CSS 规则：{selector}");
    out
}

/// UT-PE-AB-01：返回按钮 ghost 样式锚点（R-AB-01/02/04）。
#[test]
fn ut_pe_ab_01_back_button_ghost_contract() {
    let start = Instant::now();

    // 1) R-AB-01：.cdb-btn 基础类 inline-flex + align-items:center（图标文字同行居中）
    let btn = css_block(CSS, ".cdb-btn {");
    assert!(
        btn.contains("display: inline-flex") && btn.contains("align-items: center"),
        "R-AB-01：.cdb-btn 必须 inline-flex + align-items:center，实际块：{btn}"
    );

    // 2) R-AB-02：.cdb-btn--ghost 透明边框 + hover 独立底色
    let ghost = css_block(CSS, ".cdb-btn--ghost {");
    assert!(
        ghost.contains("border-color: transparent"),
        "R-AB-02：.cdb-btn--ghost 必须透明边框（不得常驻可见边框），实际块：{ghost}"
    );
    let ghost_hover = css_block(CSS, ".cdb-btn--ghost:hover {");
    assert!(
        ghost_hover.contains("background"),
        "R-AB-02：ghost hover 必须显现底色，实际块：{ghost_hover}"
    );

    // 3) R-AB-04：.cdb-back-to-rooms 可点击高度 ≥36px + 横排居中 + 不被压缩
    let back = css_block(CSS, ".cdb-back-to-rooms {");
    assert!(
        back.contains("display: inline-flex") && back.contains("align-items: center"),
        "R-AB-01：返回入口必须单一横排（inline-flex 居中），实际块：{back}"
    );
    assert!(
        back.contains("min-height: 36px"),
        "R-AB-04：可点击区域高度必须 ≥36px，实际块：{back}"
    );

    // 4) 生产 DOM：返回按钮使用 ghost + back-to-rooms 类组合且保留 title
    assert!(
        PANELS.contains("cdb-btn cdb-btn--ghost cdb-back-to-rooms"),
        "生产返回按钮必须使用 ghost 类组合"
    );
    assert!(
        PANELS.contains("title=\"返回空间列表\""),
        "R-AB-05：返回按钮必须保留 title 可访问性"
    );

    report("UT-PE-AB-01", start);
}

/// UT-PE-AB-02：room-badge 截断规则锚点（R-AB-06/07/08）。
#[test]
fn ut_pe_ab_02_room_badge_truncation_contract() {
    let start = Instant::now();

    // 1) R-AB-06：截断三件套落在文本节点 .cdb-room-badge strong 上
    let strong = css_block(CSS, ".cdb-room-badge strong {");
    for rule in ["overflow: hidden", "text-overflow: ellipsis", "white-space: nowrap"] {
        assert!(
            strong.contains(rule),
            "R-AB-06：.cdb-room-badge strong 缺 `{rule}`，实际块：{strong}"
        );
    }
    assert!(
        strong.contains("min-width: 0"),
        "R-AB-06：strong 必须 min-width:0 才允许 flex 压缩，实际块：{strong}"
    );

    // 2) R-AB-08：badge 容器保留 max-width + 弹性收缩（flex:0 1 auto + min-width:0）
    let badge = css_block(CSS, ".cdb-room-badge {");
    assert!(
        badge.contains("max-width"),
        "R-AB-08：.cdb-room-badge 必须保留 max-width，实际块：{badge}"
    );
    assert!(
        badge.contains("min-width: 0"),
        "R-AB-08：badge 容器必须 min-width:0，实际块：{badge}"
    );
    // 容器上不再承担文本截断（三件套已下沉 strong；overflow 保留无伤但不强制断言移除）

    // 3) R-AB-08：AppBar 中部容器 min-width:0；save-state / actions 不被压缩
    let appbar = css_block(CSS, ".cdb-app-bar {");
    assert!(
        appbar.contains("min-width: 0"),
        "R-AB-08：AppBar 容器必须 min-width:0，实际块：{appbar}"
    );
    let save = css_block(CSS, ".cdb-save-state {");
    assert!(
        save.contains("flex: 0 0 auto"),
        "R-AB-08：save-state 不得被压缩，实际块：{save}"
    );

    // 4) R-AB-07：生产 DOM title 全文兜底——room-badge 按钮 title=房间全名；diagram-title title=图名
    assert!(
        PANELS.contains("title=format!(\"{}（点击返回房间列表）\", room.name)"),
        "R-AB-07：room-badge 按钮必须带房间全名 title"
    );
    assert!(
        PANELS.contains("title=move || current_title.get()"),
        "R-AB-07：diagram-title 必须带图名全文 title"
    );

    report("UT-PE-AB-02", start);
}
