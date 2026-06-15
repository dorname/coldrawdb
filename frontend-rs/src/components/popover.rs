//! Popover — V1 stub
//! Spec: core-09-core-components.md §6
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverTrigger { #[default] Click, Hover }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverPlacement { #[default] BottomLeft, BottomRight, TopLeft, TopRight }

#[component]
pub fn Popover(_children: Children, _content: View) -> impl IntoView {
    view! { <div data-testid="cdb-popover-stub" /> }
}
