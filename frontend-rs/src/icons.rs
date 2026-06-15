//! E2 SVG Icon Library
//! Spec: logos/resources/prd/2-product-design/1-feature-specs/core-08-icon-library.md
//! 50 core icons converted from @douyinfe/semi-icons (simplified SVG paths for V1)

use leptos::*;

/// 通用 Icon 组件 trait
#[component]
pub fn Icon(
    /// SVG path d 属性
    path: &'static str,
    #[prop(default = 16)] size: u32,
    #[prop(default = 1.5)] stroke_width: f32,
    #[prop(default = "currentColor")] color: &'static str,
    #[prop(default = "none")] fill: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 26 26"
            fill=fill
            stroke=color
            stroke-width=stroke_width
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d=path />
        </svg>
    }
}

// ============================================================
// 4.1 基础操作（8 个）
// ============================================================
#[component]
pub fn IconAdd() -> impl IntoView {
    view! { <Icon path="M13 5 L13 21 M5 13 L21 13" /> }
}
#[component]
pub fn IconMinus() -> impl IntoView {
    view! { <Icon path="M5 13 L21 13" /> }
}
#[component]
pub fn IconClose() -> impl IntoView {
    view! { <Icon path="M6 6 L20 20 M20 6 L6 20" /> }
}
#[component]
pub fn IconEdit() -> impl IntoView {
    view! { <Icon path="M4 20 L4 16 L16 4 L20 8 L8 20 Z M14 6 L18 10" /> }
}
#[component]
pub fn IconDelete() -> impl IntoView {
    view! { <Icon path="M5 7 L21 7 M9 7 L9 4 L17 4 L17 7 M7 7 L7 22 L19 22 L19 7 M10 11 L10 18 M16 11 L16 18" /> }
}
#[component]
pub fn IconMore() -> impl IntoView {
    view! { <Icon path="M6 13 A2 2 0 1 1 6.01 13 M13 13 A2 2 0 1 1 13.01 13 M20 13 A2 2 0 1 1 20.01 13" fill="currentColor" /> }
}
#[component]
pub fn IconCheck() -> impl IntoView {
    view! { <Icon path="M5 13 L10 18 L21 7" /> }
}
#[component]
pub fn IconSearch() -> impl IntoView {
    view! { <Icon path="M11 4 A7 7 0 1 1 11 18 A7 7 0 1 1 11 4 M16 16 L21 21" /> }
}

// ============================================================
// 4.2 导航（8 个）
// ============================================================
#[component]
pub fn IconChevronUp() -> impl IntoView { view! { <Icon path="M6 16 L13 9 L20 16" /> } }
#[component]
pub fn IconChevronDown() -> impl IntoView { view! { <Icon path="M6 10 L13 17 L20 10" /> } }
#[component]
pub fn IconChevronLeft() -> impl IntoView { view! { <Icon path="M16 6 L9 13 L16 20" /> } }
#[component]
pub fn IconChevronRight() -> impl IntoView { view! { <Icon path="M10 6 L17 13 L10 20" /> } }
#[component]
pub fn IconCaretDown() -> impl IntoView { view! { <Icon path="M6 10 L13 17 L20 10" /> } }
#[component]
pub fn IconArrowLeft() -> impl IntoView { view! { <Icon path="M19 13 L5 13 M11 7 L5 13 L11 19" /> } }
#[component]
pub fn IconArrowRight() -> impl IntoView { view! { <Icon path="M7 13 L21 13 M15 7 L21 13 L15 19" /> } }
#[component]
pub fn IconExternalLink() -> impl IntoView {
    view! { <Icon path="M14 5 L21 5 L21 12 M21 5 L12 14 M19 14 L19 20 L5 20 L5 6 L11 6" /> }
}

// ============================================================
// 4.3 撤销重做 / 保存（4 个）
// ============================================================
#[component]
pub fn IconUndo() -> impl IntoView {
    view! { <Icon path="M9 14 L4 9 L9 4 M4 9 L15 9 A7 7 0 0 1 22 16 A7 7 0 0 1 15 23 L8 23" /> }
}
#[component]
pub fn IconRedo() -> impl IntoView {
    view! { <Icon path="M17 14 L22 9 L17 4 M22 9 L11 9 A7 7 0 0 0 4 16 A7 7 0 0 0 11 23 L18 23" /> }
}
#[component]
pub fn IconSave() -> impl IntoView {
    view! { <Icon path="M4 2 L18 2 L22 6 L22 22 L4 22 Z M8 2 L8 10 L18 10 L18 2 M8 14 L18 14" /> }
}
#[component]
pub fn IconShare() -> impl IntoView {
    view! { <Icon path="M4 12 A3 3 0 1 1 4.01 12 M19 5 A3 3 0 1 1 19.01 5 M19 19 A3 3 0 1 1 19.01 19 M7 10 L16 6 M7 14 L16 18" fill="currentColor" />
    }
}

// ============================================================
// 4.4 画布对象（5 个，main 自建）
// ============================================================
#[component]
pub fn IconAddTable() -> impl IntoView {
    view! { <Icon path="M4 2 L20 2 A4 4 0 0 1 22 4 L22 14 M14 22 L4 22 A4 4 0 0 1 1 18 L1 4 A4 4 0 0 1 5 2 M22 17 L22 25 M18 21 L26 21 M1 8 L22 8" /> }
}
#[component]
pub fn IconAddArea() -> impl IntoView {
    view! { <Icon path="M3 5 L23 5 L23 21 L3 21 Z M7 3 L9 3 L9 7 L7 7 Z M19 19 L21 19 L21 23 L19 23 Z" /> }
}
#[component]
pub fn IconAddNote() -> impl IntoView {
    view! { <Icon path="M4 4 L20 4 L20 20 L4 20 Z M4 9 L20 9 M4 14 L20 14 M9 4 L9 20 M14 4 L14 20" /> }
}
#[component]
pub fn IconRelationship() -> impl IntoView {
    view! { <Icon path="M4 7 L22 7 M4 13 L22 13 M4 19 L22 19 M2 7 L4 7 L4 9 M22 13 L24 13 L24 15 M2 19 L4 19 L4 21" /> }
}
#[component]
pub fn IconPan() -> impl IntoView {
    view! { <Icon path="M9 11 L9 5 A1.5 1.5 0 0 1 12 5 L12 11 M12 11 L12 4 A1.5 1.5 0 0 1 15 4 L15 11 M15 11 L15 6 A1.5 1.5 0 0 1 18 6 L18 13 M18 13 L18 9 A1.5 1.5 0 0 1 21 9 L21 16 A6 6 0 0 1 15 22 L11 22 A4 4 0 0 1 7 18 L7 13" /> }
}

// ============================================================
// 4.5 字段类型徽章（12 个）
// ============================================================
#[component]
pub fn IconKey() -> impl IntoView {
    view! { <Icon path="M16 5 A4 4 0 1 1 16 13 A4 4 0 1 1 16 5 M11 13 L4 20 L4 16 L8 16" /> }
}
#[component]
pub fn IconLink() -> impl IntoView {
    view! { <Icon path="M9 13 L17 13 M11 9 L13 9 A4 4 0 0 1 13 17 L11 17 M15 9 L17 9 A4 4 0 0 1 17 17 L15 17" /> }
}
#[component]
pub fn IconIndex() -> impl IntoView {
    view! { <Icon path="M4 6 L22 6 M4 12 L22 12 M4 18 L22 18" /> }
}
#[component]
pub fn IconUnique() -> impl IntoView {
    view! { <Icon path="M12 3 L15 9 L21 10 L16 14 L18 20 L12 17 L6 20 L8 14 L3 10 L9 9 Z" /> }
}
#[component]
pub fn IconNotNull() -> impl IntoView {
    view! { <Icon path="M12 3 L21 20 L3 20 Z M12 9 L12 14 M12 17 L12 17.5" /> }
}
#[component]
pub fn IconString() -> impl IntoView { view! { <Icon path="M6 6 L20 6 M6 6 L10 6 L10 20 L6 20 M20 6 L16 6 L16 20 L20 20" /> } }
#[component]
pub fn IconInt() -> impl IntoView { view! { <Icon path="M4 7 L10 7 L8 19 L4 19 L6 7 M22 7 L16 7 L18 19 L22 19 L20 7" /> } }
#[component]
pub fn IconDecimal() -> impl IntoView { view! { <Icon path="M4 7 L10 7 L8 19 L4 19 L6 7 M11 17 L15 17 M22 7 L16 7 L18 19 L22 19 L20 7" /> } }
#[component]
pub fn IconBoolean() -> impl IntoView { view! { <Icon path="M11 6 L17 6 A5 5 0 0 1 17 16 L11 16 A5 5 0 0 1 11 6 M11 6 A5 5 0 0 0 11 16 L17 16 M19 4 L23 4 L23 18 L19 18" /> } }
#[component]
pub fn IconDate() -> impl IntoView { view! { <Icon path="M5 6 L21 6 L21 22 L5 22 Z M5 11 L21 11 M9 4 L9 8 M17 4 L17 8" /> } }
#[component]
pub fn IconEnum() -> impl IntoView { view! { <Icon path="M4 7 L22 7 M4 7 L8 4 L8 10 Z M4 13 L22 13 M4 13 L8 10 L8 16 Z M4 19 L22 19 M4 19 L8 16 L8 22 Z" /> } }
#[component]
pub fn IconBinary() -> impl IntoView { view! { <Icon path="M3 6 L9 6 L9 12 L3 12 Z M11 14 L17 14 L17 20 L11 20 Z M17 6 L23 6 L23 12 L17 12 Z M5 12 L9 14 M15 12 L19 12 M11 6 L13 8" /> } }

// ============================================================
// 4.6 IO 抽屉 / 导出导入（5 个）
// ============================================================
#[component]
pub fn IconImport() -> impl IntoView {
    view! { <Icon path="M5 19 L21 19 M12 4 L12 16 M7 9 L12 4 L17 9" /> }
}
#[component]
pub fn IconExport() -> impl IntoView {
    view! { <Icon path="M5 19 L21 19 M12 16 L12 4 M7 11 L12 16 L17 11" /> }
}
#[component]
pub fn IconCopy() -> impl IntoView {
    view! { <Icon path="M8 8 L20 8 L20 20 L8 20 Z M4 4 L16 4 L16 8 M4 4 L4 16 L8 16" /> }
}
#[component]
pub fn IconDownload() -> impl IntoView {
    view! { <Icon path="M5 19 L21 19 M12 4 L12 16 M7 11 L12 16 L17 11" /> }
}
#[component]
pub fn IconUpload() -> impl IntoView {
    view! { <Icon path="M5 19 L21 19 M12 16 L12 4 M7 9 L12 4 L17 9" /> }
}

// ============================================================
// 4.7 Inspector 操作（4 个）
// ============================================================
#[component]
pub fn IconMove() -> impl IntoView {
    view! { <Icon path="M13 4 L13 9 L18 9 L13 9 L13 4 Z M13 22 L13 17 L8 17 L13 17 L13 22 Z M4 13 L9 13 L9 8 L9 13 L4 13 Z M22 13 L17 13 L17 18 L17 13 L22 13 Z" /> }
}
#[component]
pub fn IconColorPicker() -> impl IntoView {
    view! { <Icon path="M4 20 L8 16 L18 6 A2 2 0 0 1 21 9 L11 19 L7 19 A2 2 0 0 1 5 21 Z M16 8 L19 11" /> }
}
#[component]
pub fn IconLock() -> impl IntoView {
    view! { <Icon path="M7 11 L19 11 L19 22 L7 22 Z M9 11 L9 7 A4 4 0 0 1 17 7 L17 11" /> }
}

// ============================================================
// 4.8 主题与设置（4 个）
// ============================================================
#[component]
pub fn IconSun() -> impl IntoView {
    view! { <Icon path="M13 8 A5 5 0 1 1 13 18 A5 5 0 1 1 13 8 M13 2 L13 4 M13 22 L13 24 M2 13 L4 13 M22 13 L24 13 M5 5 L7 7 M19 19 L21 21 M5 21 L7 19 M19 7 L21 5" /> }
}
#[component]
pub fn IconMoon() -> impl IntoView {
    view! { <Icon path="M20 14 A8 8 0 1 1 12 6 A6 6 0 0 0 20 14 Z" fill="currentColor" /> }
}
#[component]
pub fn IconSettings() -> impl IntoView {
    view! { <Icon path="M13 8 A5 5 0 1 1 13 18 A5 5 0 1 1 13 8 M13 2 L15 5 L18 4 L19 7 L22 8 L21 11 L23 13 L21 15 L22 18 L19 19 L18 22 L15 21 L13 24 L11 21 L8 22 L7 19 L4 18 L5 15 L3 13 L5 11 L4 8 L7 7 L8 4 L11 5 Z" /> }
}
#[component]
pub fn IconHelp() -> impl IntoView {
    view! { <Icon path="M13 4 A9 9 0 1 1 13 22 A9 9 0 1 1 13 4 M9 10 A4 4 0 0 1 17 10 C17 12 13 12 13 14 M13 17 L13 17.5" /> }
}

// 额外：Phase A/C 用的 Issues / Theme / Code 等图标
#[component]
pub fn IconWarning() -> impl IntoView {
    view! { <Icon path="M12 3 L23 21 L1 21 Z M12 10 L12 15 M12 17 L12 17.5" /> }
}
#[component]
pub fn IconCode() -> impl IntoView {
    view! { <Icon path="M9 7 L4 13 L9 19 M17 7 L22 13 L17 19 M14 5 L12 21" /> }
}
#[component]
pub fn IconType() -> impl IntoView {
    view! { <Icon path="M4 6 L22 6 M13 6 L13 22" /> }
}
