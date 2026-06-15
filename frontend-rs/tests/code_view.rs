//! E4 CodeView + CommandPalette 单元测试
//! Spec: logos/resources/test/core-PE-design-system-test-cases.md §5

use std::fs;
use std::path::PathBuf;

fn load(path: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(path);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {} failed", p.display()))
}

#[test]
fn ut_e4_01_code_view_module_exists() {
    let src = load("src/code_view.rs");
    assert!(src.contains("pub fn CodeView") || src.contains("pub fn code_view"),
        "UT-E4-01 FAIL: CodeView component missing");
    assert!(src.contains("data-testid=\"code-view\""),
        "UT-E4-01 FAIL: data-testid=\"code-view\" missing");
}

#[test]
fn ut_e4_02_code_view_language_enum() {
    let src = load("src/code_view.rs");
    for lang in ["Sql", "Dbml", "Json"] {
        assert!(src.contains(lang), "UT-E4-02 FAIL: CodeLanguage `{}` missing", lang);
    }
}

#[test]
fn ut_e4_03_view_mode_canvas_code() {
    let src = load("src/code_view.rs");
    assert!(src.contains("Canvas") && src.contains("Code"),
        "UT-E4-03 FAIL: ViewMode::Canvas/Code enum missing");
    assert!(src.contains("btn-code-view"),
        "UT-E4-03 FAIL: btn-code-view data-testid missing");
}

#[test]
fn ut_e4_04_command_palette_module() {
    let src = load("src/command_palette.rs");
    assert!(src.contains("pub fn CommandPalette") || src.contains("pub fn command_palette"),
        "UT-E4-04 FAIL: CommandPalette component missing");
    assert!(src.contains("data-testid=\"command-palette\""),
        "UT-E4-04 FAIL: command-palette data-testid missing");
    assert!(src.contains("setup_command_palette_shortcut"),
        "UT-E4-04 FAIL: Ctrl+K shortcut setup function missing");
}

#[test]
fn ut_e4_05_palette_item_kinds() {
    let src = load("src/command_palette.rs");
    for kind in ["Table", "Area", "Enum", "Note", "Reference", "CustomType", "Action"] {
        assert!(src.contains(kind), "UT-E4-05 FAIL: PaletteKind `{}` missing", kind);
    }
}

#[test]
fn ut_e4_06_css_styles_present() {
    let css = load("src/styles.css");
    assert!(css.contains(".cdb-monaco-container"),
        "UT-E4-06 FAIL: .cdb-monaco-container CSS missing");
    assert!(css.contains(".cdb-command-palette"),
        "UT-E4-06 FAIL: .cdb-command-palette CSS missing");
    assert!(css.contains(".cdb-code-view__copy"),
        "UT-E4-06 FAIL: .cdb-code-view__copy CSS missing");
    assert!(css.contains("var(--cdb-z-modal)"),
        "UT-E4-06 FAIL: z-modal token usage missing in E4 styles");
}

#[test]
fn ut_e4_07_cargo_toml_monaco_placeholder() {
    let toml = load("Cargo.toml");
    assert!(toml.contains("monaco-editor-wasm"),
        "UT-E4-07 FAIL: monaco-editor-wasm placeholder comment missing in Cargo.toml");
}
