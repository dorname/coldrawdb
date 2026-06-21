//! E4 CodeView — SQL / DBML / JSON 只读预览（Monaco 可选升级，见 E4_ACTIVATION.md）
//! Spec: core-0a-code-editor.md / core-S01-edit-and-save-design.md §3.6

use crate::icons::{IconBox, IconCode};
use leptos::*;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeLanguage {
    #[default]
    Sql,
    Dbml,
    Json,
}

/// SQL/DBML/JSON 全屏代码视图
#[component]
pub fn CodeView(
    visible: RwSignal<bool>,
    language: RwSignal<CodeLanguage>,
    content: Memo<String>,
    copy_toast: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || visible.get()>
            <div
                class="cdb-code-view-modal"
                data-testid="code-view-modal"
                style="position:fixed;inset:48px 0 28px 0;z-index:var(--cdb-z-inspector,30);background:var(--cdb-color-bg-0);display:flex;flex-direction:column;padding:var(--cdb-space-3);"
            >
                <div class="cdb-code-view__tabs" style="display:flex;gap:8px;margin-bottom:8px;">
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || language.get() == CodeLanguage::Sql
                        data-testid="code-tab-sql"
                        on:click=move |_| language.set(CodeLanguage::Sql)
                    >"SQL"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || language.get() == CodeLanguage::Dbml
                        data-testid="code-tab-dbml"
                        on:click=move |_| language.set(CodeLanguage::Dbml)
                    >"DBML"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || language.get() == CodeLanguage::Json
                        data-testid="code-tab-json"
                        on:click=move |_| language.set(CodeLanguage::Json)
                    >"JSON"</button>
                </div>
                <pre
                    class="cdb-monaco-container"
                    data-testid="code-view"
                    data-language=move || match language.get() {
                        CodeLanguage::Sql => "sql",
                        CodeLanguage::Dbml => "dbml",
                        CodeLanguage::Json => "json",
                    }
                    style="flex:1;overflow:auto;margin:0;padding:12px;background:var(--cdb-color-bg-1);border:1px solid var(--cdb-color-border);border-radius:var(--cdb-radius-md);font-family:var(--cdb-font-mono,monospace);font-size:13px;white-space:pre-wrap;"
                >{move || content.get()}</pre>
                <div style="display:flex;justify-content:flex-end;gap:8px;margin-top:8px;">
                    <button
                        class="cdb-btn cdb-code-view__copy"
                        data-testid="btn-copy-code"
                        on:click=move |_| {
                            if copy_text_to_clipboard(&content.get()) {
                                copy_toast.set(Some("已复制到剪贴板".into()));
                            }
                        }
                    >"复制"</button>
                </div>
                {move || copy_toast.get().map(|msg| view! {
                    <div class="cdb-toast" data-testid="code-copy-toast">{msg}</div>
                })}
            </div>
        </Show>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Canvas,
    Code,
}

#[component]
pub fn ViewModeToggle(
    view_mode: RwSignal<ViewMode>,
    code_visible: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-code-view"
            title=move || {
                if matches!(view_mode.get(), ViewMode::Code) {
                    "返回画布"
                } else {
                    "代码视图"
                }
            }
            on:click=move |_| {
                if matches!(view_mode.get(), ViewMode::Canvas) {
                    view_mode.set(ViewMode::Code);
                    code_visible.set(true);
                } else {
                    view_mode.set(ViewMode::Canvas);
                    code_visible.set(false);
                }
            }
        >
            {move || if matches!(view_mode.get(), ViewMode::Code) {
                view! { "返回" }.into_view()
            } else {
                view! { <IconBox size="sm"><IconCode /></IconBox> }.into_view()
            }}
        </button>
    }
}

fn copy_text_to_clipboard(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(document) = window.document() else {
        return false;
    };
    let Ok(el) = document.create_element("textarea") else {
        return false;
    };
    let Ok(ta) = el.dyn_into::<web_sys::HtmlTextAreaElement>() else {
        return false;
    };
    ta.set_value(text);
    let Some(body) = document.body() else {
        return false;
    };
    let _ = body.append_child(&ta);
    ta.select();
    let ok = document
        .dyn_ref::<web_sys::HtmlDocument>()
        .and_then(|d| d.exec_command("copy").ok())
        .unwrap_or(false);
    let _ = body.remove_child(&ta);
    ok
}

/// Esc 关闭 Code View
pub fn setup_code_view_escape(view_mode: RwSignal<ViewMode>, code_visible: RwSignal<bool>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let on_key = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" && matches!(view_mode.get_untracked(), ViewMode::Code) {
            view_mode.set(ViewMode::Canvas);
            code_visible.set(false);
        }
    };
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(on_key));
    let _ = window.add_event_listener_with_callback(
        "keydown",
        closure.as_ref().unchecked_ref(),
    );
    closure.forget();
}
