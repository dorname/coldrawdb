//! SideSheet — V1 stub
//! Spec: core-09-core-components.md §9
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SideSheetPlacement { #[default] Right, Left }

#[component]
pub fn SideSheet(
    children: Children,
    _visible: RwSignal<bool>,
    #[prop(optional)] _title: Option<String>,
    #[prop(default = SideSheetPlacement::Right)] _placement: SideSheetPlacement,
    #[prop(default = 400)] _width: u32,
    #[prop(default = true)] _mask: bool,
    #[prop(default = true)] _mask_closable: bool,
) -> impl IntoView {
    view! { <aside data-testid="cdb-side-sheet">{children()}</aside> }
}
