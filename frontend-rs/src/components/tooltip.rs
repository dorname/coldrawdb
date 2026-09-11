//! Tooltip — V1 stub (full impl deferred to E3b)
//! Spec: core-09-core-components.md §5
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPlacement { #[default] Top, Bottom, Left, Right }

#[component]
pub fn Tooltip(_children: Children, _content: String) -> impl IntoView {
    view! { <span data-testid="cdb-tooltip-stub" /> }
}
