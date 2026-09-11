//! E1 Design Tokens 单元测试
//! Spec: logos/resources/test/core-PE-design-system-test-cases.md §2
//! 静态分析 styles.css，验证 token 命名规范与 dark mode 占位接口

use std::fs;
use std::path::PathBuf;

const STYLES_PATH: &str = "src/styles.css";

fn load_styles() -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(STYLES_PATH);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {} failed", p.display()))
}

fn extract_root_tokens(css: &str) -> Vec<String> {
    // 扫描 :root 块和 [data-mode="dark"] 块（两者都定义 token）
    let mut tokens = Vec::new();
    for marker in [":root {", r#"[data-mode="dark"] {"#] {
        if let Some(start) = css.find(marker) {
            if let Some(end_offset) = css[start..].find('}') {
                let block = &css[start..start + end_offset];
                for line in block.lines() {
                    if let Some(rest) = line.trim().strip_prefix("--cdb-") {
                        if let Some(name) = rest.split(':').next() {
                            tokens.push(format!("--cdb-{}", name.trim()));
                        }
                    }
                }
            }
        }
    }
    tokens
}

fn count_var_refs(css: &str) -> usize {
    css.matches("var(--cdb-").count()
}

#[test]
fn ut_e1_01_token_naming() {
    let css = load_styles();
    let tokens = extract_root_tokens(&css);

    // ≥ 100 token
    assert!(
        tokens.len() >= 100,
        "UT-E1-01 FAIL: expected ≥100 --cdb-* tokens, got {}",
        tokens.len()
    );

    // 命名规范：^--cdb-[a-z]+(-[a-z]+)*(-[0-9]+)?$
    let re = regex_lite();
    for name in &tokens {
        assert!(
            re.is_match(name),
            "UT-E1-01 FAIL: token name `{}` violates naming convention",
            name
        );
    }

    // var() 引用必须存在
    let refs = count_var_refs(&css);
    assert!(refs >= 100, "UT-E1-01 FAIL: expected ≥100 var(--cdb-*) refs, got {}", refs);
}

#[test]
fn ut_e1_02_dark_mode_placeholder() {
    let css = load_styles();
    assert!(
        css.contains("[data-mode=\"dark\"]"),
        "UT-E1-02 FAIL: [data-mode=\"dark\"] selector missing (E5 placeholder)"
    );
    assert!(
        css.contains("@media (prefers-color-scheme: dark)") || css.contains("prefers-color-scheme: dark"),
        "UT-E1-02 FAIL: prefers-color-scheme media query missing (E5 placeholder)"
    );
}

// 内联 regex 避免引入 regex 依赖
fn regex_lite() -> TokenNameMatcher {
    TokenNameMatcher
}

struct TokenNameMatcher;

impl TokenNameMatcher {
    fn is_match(&self, name: &str) -> bool {
        // 期望：--cdb-<segment>(-<segment>)*(-[0-9]+)?
        // segment: [a-z]+
        let s = name.strip_prefix("--cdb-").unwrap_or(name);
        let parts: Vec<&str> = s.split('-').collect();
        if parts.is_empty() {
            return false;
        }
        for (i, p) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            if p.chars().all(|c| c.is_ascii_digit()) {
                // 数字段只允许出现在最后
                if !is_last {
                    return false;
                }
            } else if !p.chars().all(|c| c.is_ascii_lowercase()) || p.is_empty() {
                return false;
            }
        }
        true
    }
}
