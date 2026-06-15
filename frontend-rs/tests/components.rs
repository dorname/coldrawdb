//! E3 Core Components 单元测试
//! Spec: logos/resources/test/core-PE-design-system-test-cases.md §4

use std::fs;
use std::path::PathBuf;

const COMPONENTS_DIR: &str = "src/components";

fn list_components() -> Vec<String> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(COMPONENTS_DIR);
    let mut names: Vec<String> = fs::read_dir(&p)
        .expect("read components dir")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn load_component(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(format!("src/components/{name}.rs"));
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {} failed", p.display()))
}

#[test]
fn ut_e3_01_to_08_eight_components_exist() {
    let names = list_components();
    let required = ["button", "modal", "dropdown", "tooltip", "popover", "tag", "collapse", "sidesheet"];
    for req in required {
        assert!(
            names.contains(&req.to_string()),
            "UT-E3 FAIL: required component `{}` missing (found: {:?})",
            req, names
        );
    }
    assert!(names.len() >= 8, "UT-E3 FAIL: expected ≥8 component files, got {}", names.len());
}

#[test]
fn ut_e3_button_renders_4_variants() {
    let src = load_component("button");
    assert!(src.contains("pub enum ButtonVariant"));
    for v in ["Primary", "Secondary", "Tertiary", "Warning", "Ghost"] {
        assert!(src.contains(v), "UT-E3-01 FAIL: Button variant `{}` missing", v);
    }
    assert!(src.contains("data-testid"), "UT-E3-01 FAIL: data-testid attribute missing");
}

#[test]
fn ut_e3_modal_4_widths() {
    let src = load_component("modal");
    for w in ["Small", "Medium", "Large", "XLarge", "Full"] {
        assert!(src.contains(w), "UT-E3-02 FAIL: Modal width `{}` missing", w);
    }
}

#[test]
fn ut_e3_dropdown_components() {
    let src = load_component("dropdown");
    assert!(src.contains("DropdownTrigger"));
    assert!(src.contains("DropdownPosition"));
    assert!(src.contains("DropdownItem"));
    assert!(src.contains("DropdownDivider"));
}

#[test]
fn ut_e3_tooltip_popover_present() {
    for name in ["tooltip", "popover"] {
        let src = load_component(name);
        assert!(src.contains("data-testid"), "UT-E3 FAIL: {} missing data-testid", name);
    }
}

#[test]
fn ut_e3_tag_6_colors() {
    let src = load_component("tag");
    for c in ["Neutral", "Primary", "Success", "Warning", "Error", "Info"] {
        assert!(src.contains(c), "UT-E3-06 FAIL: Tag color `{}` missing", c);
    }
}

#[test]
fn ut_e3_collapse_sidesheet() {
    let cl = load_component("collapse");
    let ss = load_component("sidesheet");
    assert!(cl.contains("CollapsePanel"), "UT-E3-07 FAIL: CollapsePanel missing");
    assert!(ss.contains("SideSheetPlacement"), "UT-E3-08 FAIL: SideSheet placement missing");
}

#[test]
fn ut_e3_no_hardcoded_colors() {
    for name in list_components() {
        let src = load_component(&name);
        // 排除 token 定义文件中的颜色字面量
        for cap in ["#ffffff", "#000000", "#ff0000"].iter() {
            let count = src.matches(cap).count();
            assert!(count == 0, "UT-E3 FAIL: hardcoded color `{}` in {}", cap, name);
        }
    }
}
