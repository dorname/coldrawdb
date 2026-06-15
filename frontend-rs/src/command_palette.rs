//! E4 CommandPalette — Ctrl+K / Cmd+K 唤起 (skeleton)
//! Spec: core-0a-code-editor.md §7

use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    Table,
    Area,
    Enum,
    Note,
    Reference,
    CustomType,
    Action,
}

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub kind: PaletteKind,
    pub id: String,
    pub label: String,
    pub subtitle: Option<String>,
}

/// Command Palette 居中浮层（V1 skeleton：仅渲染容器与占位）
#[component]
pub fn CommandPalette(visible: RwSignal<bool>) -> impl IntoView {
    // 完整实现见 core-0a-code-editor.md §7：键盘全局监听 Ctrl+K + 模糊搜索 + Enter 跳转
    // V1 仅渲染占位结构
    view! {
        <div
            class="cdb-command-palette"
            data-testid="command-palette"
            data-open=move || visible.get().to_string()
            style="display: flex; align-items: center; justify-content: center; padding: var(--cdb-space-3); background: var(--cdb-color-bg-0); border: 1px solid var(--cdb-color-border); border-radius: var(--cdb-radius-lg); box-shadow: var(--cdb-shadow-lg);"
        >
            <span>"Command Palette 待 wasm-pack 环境激活（V1 placeholder）"</span>
        </div>
    }
}

/// Ctrl+K / Cmd+K 全局键盘监听（V1 skeleton：仅暴露 setup 函数）
pub fn setup_command_palette_shortcut(visible: RwSignal<bool>) {
    // 完整实现：window::event_listener(keydown, |ev| { if ev.key() == "k" && (ev.ctrl_key() || ev.meta_key()) { visible.update(|v| *v = !*v); } });
    // V1 无浏览器事件接入，留空
    let _ = visible;
}
