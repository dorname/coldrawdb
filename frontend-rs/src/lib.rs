#![allow(dead_code)]
#![allow(unused_variables)]

pub mod code_view;
pub mod command_palette;
pub mod components;
pub mod editor_core;
pub mod editor_data_access;
pub mod editor_panels;
pub mod editor_render;
pub mod icons;

use editor_core::{DebounceTrigger, EditorStore};
use editor_panels::AppRoot;
use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn mount() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        let store = EditorStore::new();
        let debouncer = DebounceTrigger::default();

        let (diagram_id, share_mode) = parse_route_from_location();

        #[cfg(debug_assertions)]
        expose_test_hooks(&store);

        view! {
            <AppRoot store=store debouncer=debouncer _diagram_id=diagram_id share_mode=share_mode />
        }
    });
}

/// 从 URL query `?share=<id>` 解析 diagram id（对齐 S02 Phase 2 / `build_share_url`）。
pub fn parse_share_param(search: &str) -> Option<String> {
    let q = search.strip_prefix('?').unwrap_or(search);
    if q.is_empty() {
        return None;
    }
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "share" && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// 从 pathname 末段解析 diagram id（如 `/editor/d-abc-123`）。
pub fn parse_diagram_id_from_pathname_str(pathname: &str) -> Option<String> {
    pathname
        .split('/')
        .filter(|s| !s.is_empty())
        .next_back()
        .map(|s| s.to_string())
        .filter(|s| s != "editor")
}

/// 纯函数：share 参数优先，其次 pathname，最后 fallback `default`。
pub fn diagram_id_from_location(pathname: &str, search: &str) -> String {
    parse_share_param(search)
        .or_else(|| parse_diagram_id_from_pathname_str(pathname))
        .unwrap_or_else(|| "default".to_string())
}

pub fn route_from_location(pathname: &str, search: &str) -> (String, bool) {
    if let Some(id) = parse_share_param(search) {
        return (id, true);
    }
    (
        parse_diagram_id_from_pathname_str(pathname).unwrap_or_else(|| "default".to_string()),
        false,
    )
}

fn parse_route_from_location() -> (String, bool) {
    web_sys::window()
        .map(|w| {
            route_from_location(
                &w.location().pathname().unwrap_or_default(),
                &w.location().search().unwrap_or_default(),
            )
        })
        .unwrap_or_else(|| ("default".to_string(), false))
}

#[cfg(debug_assertions)]
fn expose_test_hooks(store: &EditorStore) {
    let root = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element());
    if let Some(root) = root {
        let rev_signal = store.revision;
        let dirty_signal = store.dirty;
        let root_for_rev = root.clone();
        let root_for_dirty = root;
        create_render_effect(move |_| {
            let _ = root_for_rev.set_attribute("data-cdb-revision", &rev_signal.get().to_string());
        });
        create_render_effect(move |_| {
            let _ = root_for_dirty.set_attribute("data-cdb-dirty", &dirty_signal.get().to_string());
        });
    }
}

#[cfg(test)]
mod location_tests {
    use super::{diagram_id_from_location, parse_share_param, route_from_location};

    #[test]
    fn ut_s02_01_share_param_parsed() {
        assert_eq!(
            parse_share_param("?share=abc-123-def"),
            Some("abc-123-def".into())
        );
        assert_eq!(parse_share_param("/editor?share=d-uuid"), None::<String>);
    }

    #[test]
    fn ut_s02_02_share_takes_priority_over_pathname() {
        assert_eq!(
            diagram_id_from_location("/editor/legacy-id", "?share=abc-123-def"),
            "abc-123-def"
        );
    }

    #[test]
    fn ut_s02_03_pathname_fallback() {
        assert_eq!(
            diagram_id_from_location("/editor/my-diagram", ""),
            "my-diagram"
        );
    }

    #[test]
    fn ut_s02_04_default_when_empty() {
        assert_eq!(diagram_id_from_location("/", ""), "default");
    }

    #[test]
    fn ut_fe_s03_01_share_route_bypasses_auth_gate() {
        assert_eq!(
            route_from_location("/editor/private-id", "?share=public-id"),
            ("public-id".to_string(), true)
        );
        assert_eq!(
            route_from_location("/editor/private-id", ""),
            ("private-id".to_string(), false)
        );
    }
}
