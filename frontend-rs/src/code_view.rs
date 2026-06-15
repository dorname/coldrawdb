//! E4 CodeView — Monaco Editor 集成 (skeleton)
//! Spec: logos/resources/prd/2-product-design/1-feature-specs/core-0a-code-editor.md §2–§6
//! 注意：V1 沙箱无 wasm-pack / 浏览器环境，monaco-editor-wasm 依赖暂注释。
//! 完整激活需要：cargo install wasm-pack + monaco-editor + monaco-editor-wasm + wasm32 target。

use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeLanguage {
    #[default]
    Sql,
    Dbml,
    Json,
}

/// SQL/DBML 全屏代码视图（V1 skeleton：仅渲染容器，实际 Monaco 挂载在 wasm-pack 环境下激活）
#[component]
pub fn CodeView(
    _visible: RwSignal<bool>,
    _language: RwSignal<CodeLanguage>,
    #[prop(default = true)] _show_copy: bool,
    #[prop(default = true)] _readonly: bool,
) -> impl IntoView {
    // 完整实现见 core-0a-code-editor.md §3.2：wasm_bindgen extern "C" 块 + create_effect 调 monaco.editor.create()
    view! {
        <div
            class="cdb-monaco-container"
            data-testid="code-view"
            data-language="sql"
            data-mono-co-loaded="false"
            style="width: 100%; height: 100%; min-height: 400px; background: var(--cdb-color-bg-1); border: 1px solid var(--cdb-color-border); border-radius: var(--cdb-radius-md); display: flex; align-items: center; justify-content: center; color: var(--cdb-color-text-2);"
        >
            <span>"Monaco Editor 待 wasm-pack 环境激活（V1 placeholder）"</span>
        </div>
    }
}

/// SQL/DBML/JSON 视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Canvas,
    Code,
}

#[component]
pub fn ViewModeToggle(
    view_mode: RwSignal<ViewMode>,
    on_open_code: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            class="cdb-btn cdb-btn--secondary"
            data-testid="btn-code-view"
            on:click=move |_| {
                if matches!(view_mode.get(), ViewMode::Canvas) {
                    view_mode.set(ViewMode::Code);
                    on_open_code.call(());
                } else {
                    view_mode.set(ViewMode::Canvas);
                }
            }
        >
            {move || if matches!(view_mode.get(), ViewMode::Code) { "返回" } else { "代码" }}
        </button>
    }
}
