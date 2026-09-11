//! Tag — V1 stub
//! Spec: core-09-core-components.md §7
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagColor { #[default] Neutral, Primary, Success, Warning, Error, Info }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagSize { #[default] Small, Medium }

#[component]
pub fn Tag(
    children: Children,
    #[prop(default = TagColor::Neutral)] _color: TagColor,
    #[prop(default = TagSize::Small)] _size: TagSize,
    #[prop(default = false)] _closable: bool,
) -> impl IntoView {
    view! { <span data-testid="cdb-tag" data-color="neutral">{children()}</span> }
}
