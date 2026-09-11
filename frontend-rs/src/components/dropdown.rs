//! Dropdown — V1 stub (full impl deferred to E3b)
//! Spec: core-09-core-components.md §4
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownTrigger { #[default] Click, Hover }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownPosition { #[default] BottomLeft, BottomRight, TopLeft, TopRight }

#[component]
pub fn Dropdown(
    _children: Children,
    _menu: View,
    #[prop(default = DropdownTrigger::Click)] _trigger: DropdownTrigger,
    #[prop(default = DropdownPosition::BottomLeft)] _position: DropdownPosition,
) -> impl IntoView {
    view! { <div data-testid="cdb-dropdown-stub" /> }
}

#[component]
pub fn DropdownMenu(_children: Children) -> impl IntoView {
    view! { <div data-testid="cdb-dropdown-menu-stub" /> }
}
#[component]
pub fn DropdownItem(_children: Children) -> impl IntoView {
    view! { <div data-testid="cdb-dropdown-item-stub" /> }
}
#[component]
pub fn DropdownDivider() -> impl IntoView {
    view! { <hr data-testid="cdb-dropdown-divider-stub" /> }
}
