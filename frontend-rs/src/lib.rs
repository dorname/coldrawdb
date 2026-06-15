#![allow(dead_code)]
#![allow(unused_variables)]

pub mod editor_core;
pub mod editor_render;
pub mod editor_panels;
pub mod editor_data_access;
pub mod icons;
pub mod components;
pub mod code_view;
pub mod command_palette;

use editor_panels::AppRoot;
use editor_core::{DebounceTrigger, EditorStore};
use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn mount() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        let store = EditorStore::new();
        let debouncer = DebounceTrigger::default();

        // 解析 window.location.pathname 拿真 diagram_id（取最后一段），
        // 失败时 fallback "default"（与现状一致）。fix-add-frontend-stub-leftover
        // 提案 4.2 决策：fallback "default" 而非空串/抛错。
        let diagram_id = parse_diagram_id_from_pathname();

        // e2e HP-02 测试钩子: 把 store 暴露到 window.__cdb_revision（仅 debug 构建）
        // HP-02 强断言 `window.__cdb_revision >= 1` 验证 save 链路真接通
        #[cfg(debug_assertions)]
        expose_test_hooks(&store);

        view! {
            <AppRoot store=store debouncer=debouncer _diagram_id=diagram_id />
        }
    });
}

/// 从 `window.location.pathname`（如 `/editor/d-abc-123`）取最后一段作为 diagram_id。
/// 失败（无 window / 路径为空 / 多段解析失败）时 fallback "default"。
fn parse_diagram_id_from_pathname() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .and_then(|p| {
            p.split('/')
                .filter(|s| !s.is_empty())
                .next_back()
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "default".to_string())
}

/// e2e 测试钩子（仅 `#[cfg(debug_assertions)]` 编译）：把 store 的 revision / dirty signal
/// 暴露到 `<html>` 元素的 `data-cdb-revision` / `data-cdb-dirty` 属性。
/// HP-02 强断言 `page.evaluate('() => document.documentElement.getAttribute("data-cdb-revision")')`
/// 读出数字 ≥ 1 验证 save 链路真接通。
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