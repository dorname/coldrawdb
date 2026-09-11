//! Splitter 分隔条（core-09 §14，fix-canvas-zoom-invite-comment-resize）。
//!
//! 两个实例：
//! - `splitter-inspector`：`.cdb-main` 第 2/3 列之间，拖动直写根 CSS 变量 `--cdb-inspector-w`
//! - `splitter-list-tree`：ListView 表树列右缘，拖动直写根变量 `--cdb-list-tree-w`
//!   并同步树容器内联 `grid-template-columns`（UT-PU-24 锚点）
//!
//! 实现约定（§14.4）：宽度 `RwSignal<u32>` 仅作初始值与持久化来源；拖动期间不更新
//! signal、不触发 Leptos 重渲染——`pointermove` 里直接写 CSS 变量；`pointerup`
//! 一次性写回 signal 并持久化 localStorage。mount 时读 localStorage，合法值直写
//! 变量，非法/缺失回落默认（330 / 230）。
//!
//! 事件注意：Leptos 0.5 事件为全局委托（current_target 未模拟），且拖动中指针
//! 会离开 6px 分隔条——故 pointermove/up/cancel 在 pointerdown 时临时挂到 window
//! （gloo EventListener，会话结束即移除），与主原型 document 监听口径一致。

use gloo::events::EventListener;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

/// 分隔条实例种类。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitterKind {
    /// Inspector 左缘（`.cdb-main` 第 2/3 列之间）
    Inspector,
    /// ListView 表树列右缘
    ListTree,
}

impl SplitterKind {
    /// 默认宽度（§14.1：330 / 230）
    pub fn default_width(self) -> u32 {
        match self {
            SplitterKind::Inspector => 330,
            SplitterKind::ListTree => 230,
        }
    }
    /// localStorage key（§14.1）
    pub fn storage_key(self) -> &'static str {
        match self {
            SplitterKind::Inspector => "cdb.inspector.width",
            SplitterKind::ListTree => "cdb.list-tree.width",
        }
    }
    /// 最小宽度（§14.3.1：Inspector 240 / 树列 160）
    pub fn min_width(self) -> u32 {
        match self {
            SplitterKind::Inspector => 240,
            SplitterKind::ListTree => 160,
        }
    }
    /// 最大宽度（§14.3.1：Inspector 520 且不超过视口 - 200 / 树列 420）
    pub fn max_width(self, viewport_w: u32) -> u32 {
        match self {
            SplitterKind::Inspector => 520
                .min(viewport_w.saturating_sub(200))
                .max(self.min_width()),
            SplitterKind::ListTree => 420,
        }
    }
    /// data-testid（§14.5）
    pub fn testid(self) -> &'static str {
        match self {
            SplitterKind::Inspector => "splitter-inspector",
            SplitterKind::ListTree => "splitter-list-tree",
        }
    }
    /// 根 CSS 变量名
    pub fn css_var(self) -> &'static str {
        match self {
            SplitterKind::Inspector => "--cdb-inspector-w",
            SplitterKind::ListTree => "--cdb-list-tree-w",
        }
    }
    /// aria-label
    pub fn aria_label(self) -> &'static str {
        match self {
            SplitterKind::Inspector => "调整检查器宽度",
            SplitterKind::ListTree => "调整表树列宽",
        }
    }
    /// 拖动方向符号（core-09 §14.2 第 2 条，fix-canvas-grid-splitter-listview-comment）：
    /// 宽度跟随「分隔条相对面板的边缘位移」——Inspector 在右侧面板左缘，
    /// 向右拖应变窄（-1）；ListTree 在左侧树列右缘，向右拖应变宽（+1）。
    pub fn drag_sign(self) -> i64 {
        match self {
            SplitterKind::Inspector => -1,
            SplitterKind::ListTree => 1,
        }
    }
}

/// 宽度钳制（§14.3.1）：i64 输入容纳超大/超小拖动位移。
pub fn clamp_width(kind: SplitterKind, w: i64, viewport_w: u32) -> u32 {
    let max = kind.max_width(viewport_w) as i64;
    w.clamp(kind.min_width() as i64, max) as u32
}

/// 拖动数学：起始宽 + 方向符号 × 水平位移 → 钳制后宽度（UT-PU-22 指针序列模拟口径；
/// Inspector 取 -dx，ListTree 取 +dx，core-09 §14.2 第 2 条）。
pub fn drag_width(kind: SplitterKind, start_w: u32, dx: i64, viewport_w: u32) -> u32 {
    clamp_width(kind, start_w as i64 + kind.drag_sign() * dx, viewport_w)
}

/// localStorage 原始值 → 合法宽度；非数字 / 越界 / 缺失一律回落默认（§14.4.4，UT-PU-23）。
pub fn sanitize_stored_width(raw: Option<&str>, kind: SplitterKind, viewport_w: u32) -> u32 {
    match raw.and_then(|s| s.trim().parse::<i64>().ok()) {
        Some(w) if w >= kind.min_width() as i64 && w <= kind.max_width(viewport_w) as i64 => {
            w as u32
        }
        _ => kind.default_width(),
    }
}

/// 拖动会话：起点的坐标/宽度 + 父容器 + 临时 window 监听（会话结束 drop 即移除）。
struct DragSession {
    start_x: i32,
    start_w: u32,
    parent: web_sys::Element,
    _listeners: [EventListener; 3],
}

fn viewport_width() -> u32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(1024.0) as u32
}

fn root_style() -> Option<web_sys::CssStyleDeclaration> {
    let win = web_sys::window()?;
    let root = win.document()?.document_element()?;
    root.dyn_into::<web_sys::HtmlElement>().ok().map(|e| e.style())
}

/// 直写 CSS 变量（§14.4：绕过响应式，拖动期每帧调用不触发重渲染）。
/// 树列实例同时直写容器内联 `grid-template-columns`（UT-PU-24 即时性锚点）。
fn apply_width(kind: SplitterKind, w: u32, container: Option<&web_sys::Element>) {
    if let Some(style) = root_style() {
        let _ = style.set_property(kind.css_var(), &format!("{w}px"));
    }
    if kind == SplitterKind::ListTree {
        if let Some(c) = container {
            if let Ok(el) = c.clone().dyn_into::<web_sys::HtmlElement>() {
                let _ = el
                    .style()
                    .set_property("grid-template-columns", &format!("{w}px minmax(0, 1fr)"));
            }
        }
    }
}

fn persist_width(kind: SplitterKind, w: u32) {
    if let Some(local) = web_sys::window().and_then(|w| win_local_storage(&w)) {
        let _ = local.set_item(kind.storage_key(), &w.to_string());
    }
}

fn win_local_storage(win: &web_sys::Window) -> Option<web_sys::Storage> {
    win.local_storage().ok().flatten()
}

fn stored_width(kind: SplitterKind) -> Option<String> {
    web_sys::window()
        .and_then(|w| win_local_storage(&w))
        .and_then(|s| s.get_item(kind.storage_key()).ok().flatten())
}

/// 布局宽度拖动分隔条。拖动期直写 CSS 变量（零响应式重渲染），
/// `pointerup` 写回 signal + localStorage；双击复位默认；键盘方向键 ±16
///（Shift ±64）、Home/End 跳 min/max、Esc 复位（§14.2）。
#[component]
pub fn Splitter(kind: SplitterKind) -> impl IntoView {
    let width = create_rw_signal(kind.default_width());
    let session: Rc<RefCell<Option<DragSession>>> = Rc::new(RefCell::new(None));
    let el_ref = create_node_ref::<html::Div>();

    // mount：读 localStorage → 合法值直写 CSS 变量，非法/缺失用默认（§14.4.4）
    create_effect(move |_| {
        let w = sanitize_stored_width(stored_width(kind).as_deref(), kind, viewport_width());
        width.set(w);
        apply_width(kind, w, None);
    });

    let drag_down = session.clone();
    let el_down = el_ref.clone();

    view! {
        <div
            node_ref=el_ref
            class=match kind {
                SplitterKind::Inspector => "cdb-splitter cdb-splitter--inspector",
                SplitterKind::ListTree => "cdb-splitter cdb-splitter--list-tree",
            }
            data-testid=kind.testid()
            role="separator"
            aria-orientation="vertical"
            tabindex="0"
            attr:aria-label=kind.aria_label()
            attr:title="拖动调整宽度 · 双击复位默认"
            attr:aria-valuenow=move || width.get().to_string()
            on:pointerdown=move |ev: web_sys::PointerEvent| {
                ev.prevent_default();
                let Some(el) = el_down.get() else { return };
                let Some(parent) = el.parent_element() else { return };
                let Some(win) = web_sys::window() else { return };
                // 已有会话（上次 up/cancel 丢失）直接作废，监听器随 drop 移除
                drop(drag_down.borrow_mut().take());

                let start_x = ev.client_x();
                let start_w = width.get_untracked();

                // move：直写 CSS 变量，零响应式重渲染
                let parent_for_move = parent.clone();
                let move_listener = EventListener::new(&win, "pointermove", move |e: &web_sys::Event| {
                    let Some(e) = e.dyn_ref::<web_sys::PointerEvent>() else { return };
                    let w = drag_width(kind, start_w, (e.client_x() - start_x) as i64, viewport_width());
                    apply_width(kind, w, Some(&parent_for_move));
                });

                // up / cancel：结束会话 → drop 监听器、写回 signal、持久化
                let session_for_end = drag_down.clone();
                let el_for_end = el.clone();
                let parent_for_end = parent.clone();
                let make_end_listener = move |event_name: &'static str| {
                    let session_for_end = session_for_end.clone();
                    let el_for_end = el_for_end.clone();
                    let parent = parent_for_end.clone();
                    EventListener::new(&win, event_name, move |e: &web_sys::Event| {
                        let Some(e) = e.dyn_ref::<web_sys::PointerEvent>() else { return };
                        let Some(sess) = session_for_end.borrow_mut().take() else { return };
                        let _ = el_for_end.class_list().remove_1("is-active");
                        let _ = sess.parent.class_list().remove_1("is-resizing");
                        let w = drag_width(kind, sess.start_w, (e.client_x() - sess.start_x) as i64, viewport_width());
                        width.set(w);
                        persist_width(kind, w);
                        apply_width(kind, w, Some(&parent));
                    })
                };
                let up_listener = make_end_listener("pointerup");
                let cancel_listener = make_end_listener("pointercancel");

                *drag_down.borrow_mut() = Some(DragSession {
                    start_x,
                    start_w,
                    parent,
                    _listeners: [move_listener, up_listener, cancel_listener],
                });
                let _ = el.class_list().add_1("is-active");
                // §14.3.3：拖动期禁用 .cdb-main grid 过渡
                let _ = drag_down
                    .borrow()
                    .as_ref()
                    .map(|s| s.parent.class_list().add_1("is-resizing"));
            }
            on:dblclick=move |_| {
                let w = kind.default_width();
                width.set(w);
                persist_width(kind, w);
                apply_width(kind, w, None);
            }
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                let cur = width.get_untracked() as i64;
                let step = if ev.shift_key() { 64 } else { 16 };
                let next = match ev.key().as_str() {
                    "ArrowLeft" => cur - step,
                    "ArrowRight" => cur + step,
                    "Home" => kind.min_width() as i64,
                    "End" => kind.max_width(viewport_width()) as i64,
                    "Escape" => kind.default_width() as i64,
                    _ => return,
                };
                ev.prevent_default();
                let w = clamp_width(kind, next, viewport_width());
                width.set(w);
                persist_width(kind, w);
                apply_width(kind, w, None);
            }
        ></div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_and_drag_boundaries() {
        // Inspector：240–520（视口 1440 时 max = min(520, 1240) = 520）；方向符号 -1
        //（右拖 dx>0 变窄、左拖 dx<0 变宽，core-09 §14.2 第 2 条）
        assert_eq!(clamp_width(SplitterKind::Inspector, 330, 1440), 330);
        assert_eq!(drag_width(SplitterKind::Inspector, 330, 190, 1440), 240);
        assert_eq!(drag_width(SplitterKind::Inspector, 330, 10_000, 1440), 240);
        assert_eq!(drag_width(SplitterKind::Inspector, 330, -90, 1440), 420);
        assert_eq!(drag_width(SplitterKind::Inspector, 330, -10_000, 1440), 520);
        // 视口 - 200 上限
        assert_eq!(drag_width(SplitterKind::Inspector, 330, -10_000, 700), 500);
        // 树列：160–420；方向符号 +1（右拖变宽）
        assert_eq!(drag_width(SplitterKind::ListTree, 230, 300, 1440), 420);
        assert_eq!(drag_width(SplitterKind::ListTree, 230, -300, 1440), 160);
    }

    #[test]
    fn sanitize_stored_fallback() {
        assert_eq!(sanitize_stored_width(Some("400"), SplitterKind::Inspector, 1440), 400);
        assert_eq!(sanitize_stored_width(None, SplitterKind::Inspector, 1440), 330);
        assert_eq!(sanitize_stored_width(Some("abc"), SplitterKind::Inspector, 1440), 330);
        assert_eq!(sanitize_stored_width(Some(""), SplitterKind::Inspector, 1440), 330);
        assert_eq!(sanitize_stored_width(Some("999"), SplitterKind::Inspector, 1440), 330);
        assert_eq!(sanitize_stored_width(Some("100"), SplitterKind::Inspector, 1440), 330);
        assert_eq!(sanitize_stored_width(Some("300"), SplitterKind::ListTree, 1440), 300);
        assert_eq!(sanitize_stored_width(Some("500"), SplitterKind::ListTree, 1440), 230);
        assert_eq!(sanitize_stored_width(None, SplitterKind::ListTree, 1440), 230);
    }
}
