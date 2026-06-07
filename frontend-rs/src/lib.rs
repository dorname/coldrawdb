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
    let store = EditorStore::new();
    let debouncer = DebounceTrigger::default();
    let diagram_id = "default".to_string();

    mount_to_body(move || {
        view! {
            <AppRoot store=store.clone() debouncer=debouncer.clone() _diagram_id=diagram_id.clone() />
        }
    });
}