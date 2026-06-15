//! Button — primary/secondary/tertiary/warning/ghost variants × small/medium/large sizes
//! Spec: core-09-core-components.md §2
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Secondary,
    Primary,
    Tertiary,
    Warning,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[component]
pub fn Button(
    children: Children,
    #[prop(default = ButtonVariant::Secondary)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(default = false)] block: bool,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let variant_class = match variant {
        ButtonVariant::Primary => "cdb-btn cdb-btn--primary",
        ButtonVariant::Secondary => "cdb-btn cdb-btn--secondary",
        ButtonVariant::Tertiary => "cdb-btn cdb-btn--tertiary",
        ButtonVariant::Warning => "cdb-btn cdb-btn--warning",
        ButtonVariant::Ghost => "cdb-btn cdb-btn--ghost",
    };
    let size_class = match size {
        ButtonSize::Small => "cdb-btn--sm",
        ButtonSize::Medium => "cdb-btn--md",
        ButtonSize::Large => "cdb-btn--lg",
    };
    let block_class = if block { " cdb-btn--block" } else { "" };
    let disabled_attr = if disabled { "true" } else { "false" };
    let data_testid = class.unwrap_or("cdb-button");

    view! {
        <button
            class=format!("{variant_class} {size_class}{block_class}")
            data-testid=data_testid
            data-variant=format!("{:?}", variant).to_lowercase()
            data-size=format!("{:?}", size).to_lowercase()
            disabled=disabled_attr
            on:click=move |ev| {
                if !disabled && !loading {
                    if let Some(cb) = on_click { cb.call(ev); }
                }
            }
        >
            <span class=if loading { "cdb-spinner" } else { "cdb-btn__content" }>{children()}</span>
        </button>
    }
}
