//! E2 SVG Icon Library 单元测试
//! Spec: logos/resources/test/core-PE-design-system-test-cases.md §3

use std::fs;
use std::path::PathBuf;

const ICONS_PATH: &str = "src/icons.rs";

fn load_icons() -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(ICONS_PATH);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {} failed", p.display()))
}

fn count_icon_fns(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            if let Some(name) = rest.split('(').next() {
                names.push(name.trim().to_string());
            }
        }
    }
    names
}

#[test]
fn ut_e2_01_50_icons_registered() {
    let src = load_icons();
    let fns = count_icon_fns(&src);

    // 至少 50 个 pub fn（Icon + 50 个具体图标）
    assert!(
        fns.len() >= 50,
        "UT-E2-01 FAIL: expected ≥50 pub fn in icons.rs, got {} ({:?})",
        fns.len(),
        fns
    );

    // 必备图标名清单
    let required = [
        "Icon", "IconAdd", "IconClose", "IconEdit", "IconDelete",
        "IconAddTable", "IconAddArea", "IconAddNote", "IconUndo", "IconRedo",
        "IconSave", "IconShare", "IconCopy", "IconSearch", "IconSun", "IconMoon",
    ];
    for name in required {
        assert!(
            fns.contains(&name.to_string()),
            "UT-E2-01 FAIL: required icon function `{}` missing",
            name
        );
    }
}

#[test]
fn ut_e2_02_size_and_color_param() {
    let src = load_icons();

    // Icon component 必须接受 size prop（默认 16）
    assert!(
        src.contains("#[prop(default = 16)] size: u32"),
        "UT-E2-02 FAIL: Icon size prop missing or default wrong"
    );

    // 必须用 currentColor（颜色继承）
    assert!(
        src.contains("color: &'static str") && src.contains("\"currentColor\""),
        "UT-E2-02 FAIL: Icon color prop must default to 'currentColor' for parent color inheritance"
    );

    // stroke-width 默认 1.5
    assert!(
        src.contains("#[prop(default = 1.5)] stroke_width: f32"),
        "UT-E2-02 FAIL: stroke_width default 1.5 missing"
    );
}
