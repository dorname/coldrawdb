//! E5 暗色模式 + E6 动效 单元测试
//! Spec: logos/resources/test/core-PE-design-system-test-cases.md §6/§7

use std::fs;
use std::path::PathBuf;

fn load_css() -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src/styles.css");
    fs::read_to_string(&p).unwrap()
}

#[test]
fn ut_e5_01_dark_token_block_complete() {
    let css = load_css();
    let dark_block_start = css.find(r#"[data-mode="dark"] {"#).expect("dark block missing");
    let dark_block_end = css[dark_block_start..].find("}").unwrap();
    let block = &css[dark_block_start..dark_block_start + dark_block_end];

    // 必须有完整 primary / grey / text / bg / shadow token 覆盖
    for token in [
        "--cdb-color-primary:",
        "--cdb-color-grey-0:",
        "--cdb-color-grey-9:",
        "--cdb-color-text-0:",
        "--cdb-color-bg-0:",
        "--cdb-color-bg-3:",
        "--cdb-shadow-sm:",
        "--cdb-shadow-md:",
        "--cdb-shadow-lg:",
    ] {
        assert!(block.contains(token), "UT-E5-01 FAIL: dark block missing `{}`", token);
    }

    // darkBgTheme 引用
    assert!(block.contains("#16161a"), "UT-E5-01 FAIL: darkBgTheme #16161a missing");
}

#[test]
fn ut_e5_02_prefers_color_scheme_media() {
    let css = load_css();
    let pref = css.find("prefers-color-scheme: dark").expect("media query missing");
    // 至少存在一处 media query
    assert!(pref < css.len(), "UT-E5-02 FAIL: prefers-color-scheme media query not in valid position");
}

#[test]
fn ut_e5_03_z_modal_token_in_dark() {
    let css = load_css();
    // [data-mode="dark"] 不应该改变 z-index（z-index 是行为，不是视觉）
    let dark_start = css.find(r#"[data-mode="dark"] {"#).unwrap();
    let dark_end = css[dark_start..].find("}").unwrap();
    let dark_block = &css[dark_start..dark_start + dark_end];
    // z-index 保留（dark 模式不改变 z）
    assert!(!dark_block.contains("--cdb-z-modal:") || dark_block.contains("--cdb-z-modal: 50"),
        "UT-E5-03 FAIL: z-modal in dark block should remain unchanged (50)");
}

#[test]
fn ut_e6_01_keyframes_present() {
    let css = load_css();
    for kf in [
        "@keyframes cdb-fade-in",
        "@keyframes cdb-fade-out",
        "@keyframes cdb-slide-in-right",
        "@keyframes cdb-slide-out-right",
        "@keyframes cdb-slide-down",
        "@keyframes cdb-slide-up",
        "@keyframes cdb-pulse",
        "@keyframes cdb-spin",
    ] {
        assert!(css.contains(kf), "UT-E6-01 FAIL: @keyframes `{}` missing", kf);
    }
}

#[test]
fn ut_e6_02_component_animations() {
    let css = load_css();
    for sel in [".cdb-modal", ".cdb-side-sheet", ".cdb-tooltip", ".cdb-dropdown-menu", ".cdb-popover", ".cdb-btn", ".cdb-tag--warning", ".cdb-spinner"] {
        assert!(css.contains(sel), "UT-E6-02 FAIL: animation selector `{}` missing", sel);
    }
}

#[test]
fn ut_e6_03_animation_tokens_used() {
    let css = load_css();
    assert!(css.contains("var(--cdb-duration-fast)"), "UT-E6-03 FAIL: --cdb-duration-fast not used");
    assert!(css.contains("var(--cdb-duration-base)"), "UT-E6-03 FAIL: --cdb-duration-base not used");
    assert!(css.contains("var(--cdb-duration-slow)"), "UT-E6-03 FAIL: --cdb-duration-slow not used");
    assert!(css.contains("var(--cdb-easing-out)"), "UT-E6-03 FAIL: --cdb-easing-out not used");
}

#[test]
fn ut_e6_04_reduced_motion() {
    let css = load_css();
    assert!(css.contains("prefers-reduced-motion: reduce"),
        "UT-E6-04 FAIL: prefers-reduced-motion media query missing");
    assert!(css.contains("animation-duration: 0.01ms !important"),
        "UT-E6-04 FAIL: reduced-motion duration override missing");
    assert!(css.contains("transition-duration: 0.01ms !important"),
        "UT-E6-04 FAIL: reduced-motion transition override missing");
}

#[test]
fn ut_r6_01_pulse_naming_split() {
    let css = load_css();
    assert!(css.contains("@keyframes cdb-pulse-opacity"), "UT-R6-01 FAIL: cdb-pulse-opacity missing");
    assert!(
        css.contains(".cdb-save-dot--saving") && css.contains("cdb-pulse-opacity"),
        "UT-R6-01 FAIL: save dot should use cdb-pulse-opacity"
    );

    let pulse_count = css
        .match_indices("@keyframes ")
        .filter(|(i, _)| {
            let rest = &css[*i + 11..];
            rest.starts_with("cdb-pulse {") || rest.starts_with("cdb-pulse\n")
        })
        .count();
    assert_eq!(pulse_count, 1, "UT-R6-01 FAIL: expected exactly one @keyframes cdb-pulse, got {}", pulse_count);

    let start = css
        .match_indices("@keyframes ")
        .find(|(i, _)| {
            let rest = &css[*i + 11..];
            rest.starts_with("cdb-pulse {") || rest.starts_with("cdb-pulse\n")
        })
        .map(|(i, _)| i)
        .expect("UT-R6-01 FAIL: @keyframes cdb-pulse missing");
    let after_start = start + 1;
    let end = css[after_start..]
        .find("@keyframes")
        .map(|i| after_start + i)
        .unwrap_or(css.len());
    let block = &css[start..end];
    assert!(block.contains("scale"), "UT-R6-01 FAIL: cdb-pulse should animate scale");
    assert!(!block.contains("opacity"), "UT-R6-01 FAIL: cdb-pulse must not define opacity");
}

#[test]
fn ut_r6_02_button_focus_and_active() {
    let css = load_css();
    assert!(
        css.contains(".cdb-btn:focus-visible") && css.contains("var(--cdb-shadow-focus)"),
        "UT-R6-02 FAIL: .cdb-btn:focus-visible should use --cdb-shadow-focus"
    );
    assert!(
        css.contains(".cdb-btn--primary:active") && css.contains("var(--cdb-color-primary-active)"),
        "UT-R6-02 FAIL: primary active state missing"
    );
}

#[test]
fn ut_r6_03_panel_spring_entrance() {
    let css = load_css();
    for (sel, label) in [
        (".cdb-inspector", "inspector"),
        (".cdb-has-io-drawer .cdb-io-drawer", "io drawer"),
        (".cdb-app-bar__overflow-menu", "overflow menu"),
    ] {
        let idx = css.find(sel).unwrap_or_else(|| panic!("UT-R6-03 FAIL: selector `{}` missing", sel));
        let chunk = &css[idx..idx.saturating_add(400)];
        assert!(
            chunk.contains("var(--cdb-easing-spring)"),
            "UT-R6-03 FAIL: {} should use spring easing",
            label
        );
    }
}
