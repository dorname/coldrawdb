#![allow(dead_code)]
#![allow(unused_variables)]

pub mod editor_core;
pub mod editor_render;
pub mod editor_panels;
pub mod editor_data_access;

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
        let diagram_id = "default".to_string();

        view! {
            <AppRoot store=store debouncer=debouncer _diagram_id=diagram_id />
        }
    });
}