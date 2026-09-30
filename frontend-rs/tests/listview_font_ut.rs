//! ux-canvas-listview-font-size：ListView 字号合同测试
//!
//! Spec: `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` §10.5
//! 对应 ID：UT-LV-FONT-01

/// 在 styles.css 中查找某个选择器块的第一个 `font-size: <n>px` 并返回 n。
fn font_size_px(css: &str, selector: &str) -> Option<u32> {
    let start = css.find(selector)?;
    let block_start = css[start..].find('{')? + start + 1;
    let block_end = css[block_start..].find('}')? + block_start;
    let block = &css[block_start..block_end];
    let decl = block.split(';').find(|d| d.contains("font-size"))?;
    let val = decl.split(':').nth(1)?.trim();
    val.trim_end_matches("px").parse().ok()
}

/// 读取生产 styles.css，锁定 ListView 关键字号为 14px/12px/11px。
#[test]
fn list_view_font_size_contract() {
    let css = include_str!("../src/styles.css");

    assert_eq!(
        font_size_px(css, ".cdb-list-view-table"),
        Some(14),
        "ListView 字段网格主文字必须为 14px"
    );
    assert_eq!(
        font_size_px(css, ".cdb-list-tree-node"),
        Some(14),
        "ListView 表树节点主文字必须为 14px"
    );
    assert_eq!(
        font_size_px(css, ".cdb-list-view-table td .cdb-form-input"),
        Some(14),
        "ListView 字段行内输入框必须为 14px"
    );
    assert_eq!(
        font_size_px(css, ".cdb-list-tree-comment"),
        Some(12),
        "ListView 表树注释必须为 12px"
    );
    assert_eq!(
        font_size_px(css, ".cdb-list-tree-node__count"),
        Some(12),
        "ListView 表树计数必须为 12px"
    );
    assert_eq!(
        font_size_px(css, ".cdb-list-tree__group"),
        Some(11),
        "ListView 分组头必须为 11px"
    );
}
