//! Collapse — V1 stub
//! Spec: core-09-core-components.md §8
use leptos::*;

#[component]
pub fn Collapse(
    children: Children,
    #[prop(default = true)] _lazy_render: bool,
    #[prop(default = false)] _keep_dom: bool,
) -> impl IntoView {
    view! { <div data-testid="cdb-collapse">{children()}</div> }
}
#[component]
pub fn CollapsePanel(
    children: Children,
    _header: View,
    _item_key: String,
    #[prop(default = false)] _disabled: bool,
) -> impl IntoView {
    view! { <div data-testid="cdb-collapse-panel">{children()}</div> }
}
