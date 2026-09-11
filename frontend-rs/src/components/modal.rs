//! Modal — V1 stub (full impl deferred to E3b after Leptos 0.5 view!+Show trait fix)
//! Spec: core-09-core-components.md §3
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalWidth {
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
    Full,
}

#[component]
pub fn Modal(
    _children: Children,
    _visible: RwSignal<bool>,
    #[prop(optional)] _title: Option<String>,
    #[prop(default = ModalWidth::Medium)] _width: ModalWidth,
    #[prop(default = true)] _centered: bool,
    #[prop(default = true)] _closable: bool,
    #[prop(default = true)] _mask_closable: bool,
    #[prop(default = true)] _esc_closable: bool,
) -> impl IntoView {
    view! { <div data-testid="cdb-modal-stub" /> }
}
