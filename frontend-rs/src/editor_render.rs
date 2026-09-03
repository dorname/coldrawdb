//! editor-render — Canvas 2D rendering module
//!
//! Renders tables, relationships, areas, and notes on an HTML5 Canvas.
//! Supports pan/zoom/select via pointer events, with requestAnimationFrame
//! throttling for smooth 60fps interaction at 100+ nodes.
//!
//! Architecture: DAG dependency -> editor-core (store + types).
//! All types re-exported from `crate::editor_core::types`.

use crate::editor_core::types::{Area, Note, Reference, Table};
use leptos::{RwSignal, SignalGet, SignalSet, SignalUpdate};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, MouseEvent, PointerEvent, WheelEvent};

// ─── Canvas constants ────────────────────────────────────────────────────────

// 几何尺寸为主原型事实（core-01：表宽 230 / 表头 43 / 字段行 35 / 点阵 24px）
const TABLE_WIDTH: f64 = 230.0;
const TABLE_HEADER_HEIGHT: f64 = 43.0;
const FIELD_ROW_HEIGHT: f64 = 35.0;
/// 生产端网格尺寸：松手吸附 20px（core-CR-canvas-test-cases.md §1 合同；
/// 主原型演示 GRID=12、点阵视觉 24px，均不得写成生产合同）。仅在 pointerup 时对齐，拖动中不量化。
pub const GRID_SIZE: f64 = 20.0;
/// 关系工具：屏幕像素欧氏位移达到该阈值才视为拖线（UT-PB-06）。
pub const DRAG_THRESHOLD: f64 = 4.0;
/// 画布字体族（与壳层一致，core-07 §字体配对）
const CANVAS_FONT: &str = "\"Plus Jakarta Sans\", sans-serif";
const CANVAS_FONT_MONO: &str = "ui-monospace, monospace";

/// 返回当前 `window.devicePixelRatio`（fallback 1）。封装于一处便于单测 mock。
pub fn current_device_pixel_ratio() -> f64 {
    web_sys::window()
        .map(|w| w.device_pixel_ratio() as f64)
        .unwrap_or(1.0)
}

/// R-DPR-06：dpr ≥ 1.5 时画布小字上浮 1px，避免 HiDPI 下密度过低。仅作用于 Canvas 文字，
/// DOM 字号走 `--cdb-font-size-*` token 不变（core-07 §10.2）。
fn dpr_font_boost(base: f64) -> f64 {
    if current_device_pixel_ratio() >= 1.5 {
        base + 1.0
    } else {
        base
    }
}

/// 组装 dpr 缩放后的画布字号字符串（`"750 {px} {font_family}"`）。
fn dpr_font(weight: u32, px: f64, family: &str) -> String {
    format!("{} {}px {}", weight, dpr_font_boost(px), family)
}

/// F-WF-03：探测 Plus Jakarta Sans 是否真正可用，不可用则降级 ui-monospace。
/// 仅作为 set_font 时的 family 段使用；返回的字符串可直接拼到 `format!("...{}")`。
fn resolve_canvas_font_family(primary: &str, mono: &str) -> String {
    let win = match web_sys::window() {
        Some(w) => w,
        None => return mono.to_string(),
    };
    let doc = match win.document() {
        Some(d) => d,
        None => return mono.to_string(),
    };
    let fonts = doc.fonts();
    // primary 是 "Plus Jakarta Sans"（无 weight / size），只检测 family 是否已加载
    if fonts.check(&format!("1em \"{}\"", primary)).unwrap_or(false) {
        primary.to_string()
    } else {
        mono.to_string()
    }
}

/// 把 ctx 当前矩阵复位为 `dpr × zoom`，避免 zoom 累乘（UT-RP-03）。
pub fn apply_dpr_zoom_transform(ctx: &CanvasRenderingContext2d, zoom: f64) {
    let dpr = current_device_pixel_ratio();
    let _ = ctx.set_transform(dpr * zoom, 0.0, 0.0, dpr * zoom, 0.0, 0.0);
}

/// 画布调色板 — 主原型 core-01 画布对象事实值的亮/暗双份。
/// Canvas 2D 为栅格渲染，无法直接消费 CSS var，故按 `data-mode` 逐帧取色。
#[derive(Clone, Copy, Debug)]
struct CanvasPalette {
    /// 点阵颜色（text-3 @ 30%）
    grid_dot: &'static str,
    /// 表体背景（surface-solid 84%/94%）
    table_bg: &'static str,
    /// 表体描边（line-strong）
    table_border: &'static str,
    /// 表头默认渐变起点（brand-soft；表自带 color 时优先用表色）
    header_tint: &'static str,
    text_strong: &'static str,
    text_muted: &'static str,
    /// 字段行分隔线
    row_separator: &'static str,
    /// PK 标记（amber）
    pk_color: &'static str,
    /// 关系主线（brand 72% × text-2）
    relation: &'static str,
    /// 关系底层光晕（surface-solid 70%，7px）
    relation_halo: &'static str,
    /// 选中描边（brand）
    selected: &'static str,
    /// 选中外环（brand-soft，3px）
    selected_soft: &'static str,
    note_bg: &'static str,
    note_border: &'static str,
    note_text: &'static str,
    area_bg: &'static str,
    area_border: &'static str,
    /// 远端光标点（accent）
    presence: &'static str,
}

/// 亮色事实：core-01 :root（brand #1e8393 / text #142c34 / line rgba(49,78,88,.24)…）
const PALETTE_LIGHT: CanvasPalette = CanvasPalette {
    grid_dot: "rgba(123,141,147,.3)",
    table_bg: "rgba(255,255,255,.84)",
    table_border: "rgba(49,78,88,.24)",
    header_tint: "rgba(30,131,147,.13)",
    text_strong: "#142c34",
    text_muted: "#7b8d93",
    row_separator: "rgba(49,78,88,.09)",
    pk_color: "#e59b24",
    relation: "rgb(34,115,129)",
    relation_halo: "rgba(255,255,255,.7)",
    selected: "#1e8393",
    selected_soft: "rgba(30,131,147,.13)",
    note_bg: "rgba(229,155,36,.18)",
    note_border: "rgba(229,155,36,.32)",
    note_text: "#2c4a53",
    area_bg: "rgba(124,92,231,.05)",
    area_border: "rgba(124,92,231,.48)",
    presence: "#7c5ce7",
};

/// 暗色事实：core-01 [data-mode="dark"]（brand #5ee9dc / text #f2fdfe / line rgba(194,232,238,.3)…）
const PALETTE_DARK: CanvasPalette = CanvasPalette {
    grid_dot: "rgba(134,163,171,.3)",
    table_bg: "rgba(16,38,45,.94)",
    table_border: "rgba(194,232,238,.3)",
    header_tint: "rgba(79,209,197,.18)",
    text_strong: "#f2fdfe",
    text_muted: "#86a3ab",
    row_separator: "rgba(194,232,238,.10)",
    pk_color: "#f5c45c",
    relation: "rgb(128,234,225)",
    relation_halo: "rgba(16,38,45,.7)",
    selected: "#5ee9dc",
    selected_soft: "rgba(79,209,197,.18)",
    note_bg: "rgba(242,184,75,.24)",
    note_border: "rgba(242,184,75,.38)",
    note_text: "#f2fdfe",
    area_bg: "rgba(185,160,255,.08)",
    area_border: "rgba(185,160,255,.55)",
    presence: "#b9a0ff",
};

/// 逐帧取当前画布调色板；`data-mode` 缺失时默认暗色（同主原型与用户决策）。
fn current_palette() -> &'static CanvasPalette {
    let dark = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|el| el.get_attribute("data-mode"))
        .map(|m| m != "light")
        .unwrap_or(true);
    if dark { &PALETTE_DARK } else { &PALETTE_LIGHT }
}

// ─── Transform ───────────────────────────────────────────────────────────────

/// Canvas 2D transform state (pan + zoom).
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Transform { pan_x: 0.0, pan_y: 0.0, zoom: 1.0 }
    }
}

// ─── Drag state ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct RelFieldDrag {
    start_table_id: String,
    start_field_id: String,
    anchor_x: f64,
    anchor_y: f64,
    moved: bool,
}

#[derive(Clone, Debug)]
struct DragState {
    table_id: Option<String>,
    endpoint_drag: Option<(String, EndpointEnd)>, // (ref_id, end) when dragging an endpoint
    rel_drag: Option<RelFieldDrag>,
    pointer_id: i32,
    start_mouse_x: f64,
    start_mouse_y: f64,
    start_pan_x: f64,
    start_pan_y: f64,
    start_table_x: f64,
    start_table_y: f64,
}

impl Default for DragState {
    fn default() -> Self {
        DragState {
            table_id: None,
            endpoint_drag: None,
            rel_drag: None,
            pointer_id: 0,
            start_mouse_x: 0.0,
            start_mouse_y: 0.0,
            start_pan_x: 0.0,
            start_pan_y: 0.0,
            start_table_x: 0.0,
            start_table_y: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct LivePaint {
    tables: Option<Vec<Table>>,
    rubber: Option<(f64, f64, f64, f64)>,
}

// ─── Leptos canvas component ─────────────────────────────────────────────────

mod leptos_canvas {
    use super::*;
    use crate::editor_core::EditorStore;
    use leptos::html;
    use leptos::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsValue;

    /// Main canvas component for the database diagram editor.
    #[component]
    pub fn Canvas(
        store: EditorStore,
        transform: RwSignal<Transform>,
        read_only: bool,
        remote_presence: RwSignal<Vec<RemotePresence>>,
        on_select: Option<Box<dyn Fn(String) + 'static>>,
        on_deselect: Option<Box<dyn Fn() + 'static>>,
        on_dblclick_blank: Option<Box<dyn Fn() + 'static>>,
        rel_tool_active: RwSignal<bool>,
        on_field_pick: Option<Box<dyn Fn(String, String) + 'static>>,
        on_relation_drag_start: Option<Box<dyn Fn(String, String) + 'static>>,
        on_relation_drop: Option<Box<dyn Fn(String, String, String, String) + 'static>>,
        on_relation_drag_cancel: Option<Box<dyn Fn() + 'static>>,
        /// 表拖动松手（吸附写回 store 后）通知调用方持久化（D 批：dirty + schedule_save）
        on_table_drop: Option<Box<dyn Fn() + 'static>>,
        // 主题模式（"dark"/"light"）：绘制 effect 需跟踪以在主题切换时重刷调色板
        theme_mode: RwSignal<String>,
    ) -> impl IntoView {
        let canvas_ref = create_node_ref::<html::Canvas>();
        let selected_id = create_rw_signal(None::<String>);
        let drag_state = create_rw_signal(None::<DragState>);
        let rubber_d = create_rw_signal(None::<String>);
        let follow_path = create_rw_signal(String::new());
        let frame_tick = create_rw_signal(0u32);
        let live = Rc::new(RefCell::new(LivePaint::default()));
        let raf_pending = Rc::new(Cell::new(false));
        let raf_closure: Rc<RefCell<Option<Closure<dyn FnMut(JsValue)>>>> =
            Rc::new(RefCell::new(None));

        let on_select = Rc::new(on_select);
        let on_deselect = Rc::new(on_deselect);
        let on_dblclick_blank = Rc::new(on_dblclick_blank);
        let on_field_pick = Rc::new(on_field_pick);
        let on_relation_drag_start = Rc::new(on_relation_drag_start);
        let on_relation_drop = Rc::new(on_relation_drop);
        let on_relation_drag_cancel = Rc::new(on_relation_drag_cancel);
        let on_table_drop = Rc::new(on_table_drop);

        {
            let raf_pending = raf_pending.clone();
            let c = Closure::wrap(Box::new(move |_: JsValue| {
                raf_pending.set(false);
                frame_tick.update(|n| *n = n.wrapping_add(1));
            }) as Box<dyn FnMut(JsValue)>);
            *raf_closure.borrow_mut() = Some(c);
        }

        let schedule_paint = {
            let raf_pending = raf_pending.clone();
            let raf_closure = raf_closure.clone();
            Rc::new(move || {
                if raf_pending.get() {
                    return;
                }
                raf_pending.set(true);
                match (web_sys::window(), raf_closure.borrow().as_ref()) {
                    (Some(window), Some(cb)) => {
                        if window
                            .request_animation_frame(cb.as_ref().unchecked_ref())
                            .is_err()
                        {
                            raf_pending.set(false);
                        }
                    }
                    _ => raf_pending.set(false),
                }
            })
        };

        let screen_to_diagram = move |screen_x: f64,
                                      screen_y: f64,
                                      canvas: &web_sys::HtmlCanvasElement,
                                      t: &Transform|
              -> (f64, f64) {
            let rect = canvas.get_bounding_client_rect();
            let canvas_x = screen_x - rect.left();
            let canvas_y = screen_y - rect.top();
            let diagram_x = (canvas_x - t.pan_x) / t.zoom;
            let diagram_y = (canvas_y - t.pan_y) / t.zoom;
            (diagram_x, diagram_y)
        };

        {
            let live = live.clone();
            let on_relation_drag_cancel = on_relation_drag_cancel.clone();
            gloo::events::EventListener::new(&gloo::utils::document(), "keydown", move |ev| {
                let Some(ke) = ev.dyn_ref::<web_sys::KeyboardEvent>() else {
                    return;
                };
                if ke.key() != "Escape" {
                    return;
                }
                let dragging = drag_state
                    .get_untracked()
                    .and_then(|d| d.rel_drag)
                    .is_some();
                if dragging {
                    live.borrow_mut().rubber = None;
                    rubber_d.set(None);
                    drag_state.set(None);
                    if let Some(cb) = on_relation_drag_cancel.as_ref() {
                        cb();
                    }
                }
            })
            .forget();
        }

        {
            let live = live.clone();
            create_effect(move |_| {
            frame_tick.get();
            theme_mode.get();
            let Some(canvas) = canvas_ref.get() else {
                return;
            };
            let ctx = match canvas.get_context("2d") {
                Ok(Some(ctx)) => match ctx.dyn_into::<CanvasRenderingContext2d>() {
                    Ok(ctx) => ctx,
                    Err(_) => return,
                },
                _ => return,
            };

            if let Some(parent) = canvas.parent_element() {
                // R-DPR-01：backing store 像素 = CSS × devicePixelRatio
                let css_w = parent.client_width().max(1) as f64;
                let css_h = parent.client_height().max(1) as f64;
                let dpr = super::current_device_pixel_ratio();
                let w = (css_w * dpr).round() as u32;
                let h = (css_h * dpr).round() as u32;
                if canvas.width() != w || canvas.height() != h {
                    canvas.set_width(w);
                    canvas.set_height(h);
                }
                // CSS 布局尺寸由 `.cdb-canvas-element { width:100%; height:100% }` 控制，
                // backing store 已通过 set_width/set_height 放大为 dpr 倍，无需内联 style
            }

            let t = transform.get();
            let store_tables = store.tables.get();
            let live_g = live.borrow();
            let tables = live_g.tables.clone().unwrap_or(store_tables);
            let rubber = live_g.rubber;
            drop(live_g);
            let refs = store.references.get();
            let areas = store.areas.get();
            let notes = store.notes.get();
            let presence = remote_presence.get();
            let sel = selected_id.get();

            let width = canvas.width() as f64;
            let height = canvas.height() as f64;
            super::draw_canvas(
                &ctx,
                &t,
                width,
                height,
                &tables,
                &refs,
                &areas,
                &notes,
                &presence,
                sel.as_deref(),
                rubber,
            );

            if let Some((x1, y1, x2, y2)) = rubber {
                rubber_d.set(Some(super::rubber_band_path(x1, y1, x2, y2)));
            } else {
                rubber_d.set(None);
            }

            if let Some(first) = refs.first() {
                if let (Some(from), Some(to_tbl)) = (
                    tables.iter().find(|tbl| tbl.id == first.start_table_id),
                    tables.iter().find(|tbl| tbl.id == first.end_table_id),
                ) {
                    let d = super::calc_path(from, &first.start_field_id, to_tbl, &first.end_field_id)
                        .to_svg_d();
                    follow_path.set(d.clone());
                    let _ = canvas.set_attribute("data-follow-path", &d);
                }
            } else {
                follow_path.set(String::new());
                let _ = canvas.set_attribute("data-follow-path", "");
            }
        });
        }

        // ── R-DPR-04：matchMedia DPR 变化触发 redraw（UT-RP-04）─────────────
        let frame_tick_for_dpr = frame_tick;
        if let Some(win) = web_sys::window() {
            if let Ok(Some(mq)) =
                win.match_media("(resolution: 1dppx), (resolution: 2dppx), (resolution: 3dppx)")
            {
                let cb = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                    frame_tick_for_dpr.update(|n| *n += 1);
                }) as Box<dyn FnMut(_)>);
                let _ = mq.add_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
                cb.forget();
            }
        }

        let capture_pointer = move |canvas: &web_sys::HtmlCanvasElement, pointer_id: i32| {
            let _ = canvas.set_pointer_capture(pointer_id);
        };

        let on_pointerdown = {
            let live = live.clone();
            let on_select = on_select.clone();
            let on_deselect = on_deselect.clone();
            let schedule_paint = schedule_paint.clone();
            move |ev: PointerEvent| {
                // 防御：清掉任何 stale drag_state（pointercancel 未触发的情况下，
                // 比如浏览器在 React 树外丢失 pointer capture 的极端场景）。
                // 不清的话上一次 endpoint/table drag 残留会让本次 pointerdown
                // 被 on_pointermove 当作"延续"误更新 store。
                if drag_state.get_untracked().is_some() {
                    drag_state.set(None);
                    live.borrow_mut().rubber = None;
                    live.borrow_mut().tables = None;
                    rubber_d.set(None);
                    schedule_paint();
                }

                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                let (dx, dy) = screen_to_diagram(
                    ev.client_x() as f64,
                    ev.client_y() as f64,
                    &canvas,
                    &transform.get_untracked(),
                );

                let tables = store.tables.get_untracked();
                let refs = store.references.get_untracked();
                if read_only {
                    if let Some(id) = super::hit_test(&tables, dx, dy) {
                        selected_id.set(Some(id.clone()));
                        if let Some(cb) = on_select.as_ref() {
                            cb(id);
                        }
                        return;
                    }
                }
                if rel_tool_active.get_untracked() {
                    if let Some((tid, fid)) = super::hit_test_field(&tables, dx, dy) {
                        let (anchor_x, anchor_y) = tables
                            .iter()
                            .find(|t| t.id == tid)
                            .map(|t| super::field_anchor_start(t, &fid))
                            .unwrap_or((dx, dy));
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            endpoint_drag: None,
                            rel_drag: Some(RelFieldDrag {
                                start_table_id: tid,
                                start_field_id: fid,
                                anchor_x,
                                anchor_y,
                                moved: false,
                            }),
                            pointer_id: ev.pointer_id(),
                            start_mouse_x: ev.client_x() as f64,
                            start_mouse_y: ev.client_y() as f64,
                            start_pan_x: 0.0,
                            start_pan_y: 0.0,
                            start_table_x: 0.0,
                            start_table_y: 0.0,
                        }));
                        return;
                    }
                    // 关系工具下未命中字段：回落到 pan 模式而不是吞掉 pointerdown
                    // （之前 return 会导致选择连线/点空白后画布无法拖动）
                    let t = transform.get_untracked();
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t.pan_x,
                        start_pan_y: t.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                    return;
                }
                if let Some((ref_id, end)) = super::hit_test_endpoint(&tables, &refs, dx, dy) {
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: Some((ref_id, end)),
                        rel_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: 0.0,
                        start_pan_y: 0.0,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                    return;
                }
                if let Some(id) = super::hit_test(&tables, dx, dy) {
                    let table_x = tables.iter().find(|t| t.id == id).map(|t| t.x).unwrap_or(0.0);
                    let table_y = tables.iter().find(|t| t.id == id).map(|t| t.y).unwrap_or(0.0);
                    selected_id.set(Some(id.clone()));
                    if let Some(cb) = on_select.as_ref() {
                        cb(id.clone());
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    live.borrow_mut().tables = None;
                    drag_state.set(Some(DragState {
                        table_id: Some(id),
                        endpoint_drag: None,
                        rel_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: transform.get_untracked().pan_x,
                        start_pan_y: transform.get_untracked().pan_y,
                        start_table_x: table_x,
                        start_table_y: table_y,
                    }));
                } else {
                    selected_id.set(None);
                    if let Some(cb) = on_deselect.as_ref() {
                        cb();
                    }
                    let t = transform.get_untracked();
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t.pan_x,
                        start_pan_y: t.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                }
            }
        };

        let on_pointermove = {
            let live = live.clone();
            let schedule_paint = schedule_paint.clone();
            let on_relation_drag_start = on_relation_drag_start.clone();
            move |ev: PointerEvent| {
                let Some(drag) = drag_state.get_untracked() else {
                    return;
                };
                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                let dx = ev.client_x() as f64 - drag.start_mouse_x;
                let dy = ev.client_y() as f64 - drag.start_mouse_y;

                if let Some(rel) = drag.rel_drag.clone() {
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &transform.get_untracked(),
                    );
                    let crossed = super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD);
                    if !rel.moved && crossed {
                        drag_state.update(|d| {
                            if let Some(ds) = d {
                                if let Some(r) = &mut ds.rel_drag {
                                    r.moved = true;
                                }
                            }
                        });
                        if let Some(cb) = on_relation_drag_start.as_ref() {
                            cb(rel.start_table_id.clone(), rel.start_field_id.clone());
                        }
                    }
                    if rel.moved || crossed {
                        live.borrow_mut().rubber =
                            Some((rel.anchor_x, rel.anchor_y, diag_x, diag_y));
                        schedule_paint();
                    }
                    return;
                }

                if let Some((ref_id, end)) = &drag.endpoint_drag {
                    let (dx_d, dy_d) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &transform.get_untracked(),
                    );
                    let tables = store.tables.get_untracked();
                    let target_table_id = match end {
                        EndpointEnd::Start => store
                            .references
                            .get_untracked()
                            .iter()
                            .find(|r| r.id == *ref_id)
                            .map(|r| r.start_table_id.clone()),
                        EndpointEnd::End => store
                            .references
                            .get_untracked()
                            .iter()
                            .find(|r| r.id == *ref_id)
                            .map(|r| r.end_table_id.clone()),
                    };
                    if let Some(tid) = target_table_id {
                        let new_field = tables
                            .iter()
                            .find(|t| t.id == tid)
                            .and_then(|t| {
                                t.fields.iter().min_by(|a, b| {
                                    let ay = t.y
                                        + TABLE_HEADER_HEIGHT
                                        + FIELD_ROW_HEIGHT
                                            * t.fields.iter().position(|f| f.id == a.id).unwrap_or(0)
                                                as f64;
                                    let by = t.y
                                        + TABLE_HEADER_HEIGHT
                                        + FIELD_ROW_HEIGHT
                                            * t.fields.iter().position(|f| f.id == b.id).unwrap_or(0)
                                                as f64;
                                    let da = (dx_d - t.x).powi(2) + (dy_d - ay).powi(2);
                                    let db = (dx_d - t.x).powi(2) + (dy_d - by).powi(2);
                                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                                })
                            })
                            .map(|f| f.id.clone());
                        if let Some(fid) = new_field {
                            let refs_now = store.references.get_untracked();
                            let updated =
                                super::update_reference_endpoint(&refs_now, ref_id, *end, &fid);
                            store.references.set(updated);
                        }
                    }
                } else if let Some(table_id) = &drag.table_id {
                    // 拖动中只用临时视觉坐标，禁止 store.tables.set（UT-CR-06 / ST-CR-02）
                    let new_x = drag.start_table_x + dx / transform.get_untracked().zoom;
                    let new_y = drag.start_table_y + dy / transform.get_untracked().zoom;
                    let visual = super::apply_visual_table_position(
                        &store.tables.get_untracked(),
                        table_id,
                        new_x,
                        new_y,
                    );
                    live.borrow_mut().tables = Some(visual);
                    schedule_paint();
                } else {
                    transform.update(|t| {
                        t.pan_x = drag.start_pan_x + dx;
                        t.pan_y = drag.start_pan_y + dy;
                    });
                }
            }
        };

        let on_pointerup = {
            let live = live.clone();
            let schedule_paint = schedule_paint.clone();
            let on_field_pick = on_field_pick.clone();
            let on_relation_drop = on_relation_drop.clone();
            let on_relation_drag_cancel = on_relation_drag_cancel.clone();
            let on_table_drop = on_table_drop.clone();
            move |ev: PointerEvent| {
                let Some(drag) = drag_state.get_untracked() else {
                    return;
                };
                let canvas = canvas_ref.get();
                if let Some(c) = &canvas {
                    let _ = c.release_pointer_capture(drag.pointer_id);
                }

                if let Some(rel) = drag.rel_drag {
                    let tables = store.tables.get_untracked();
                    let (diag_x, diag_y) = canvas
                        .as_ref()
                        .map(|c| {
                            screen_to_diagram(
                                ev.client_x() as f64,
                                ev.client_y() as f64,
                                c,
                                &transform.get_untracked(),
                            )
                        })
                        .unwrap_or((0.0, 0.0));
                    live.borrow_mut().rubber = None;
                    rubber_d.set(None);
                    drag_state.set(None);
                    schedule_paint();
                    if !rel.moved {
                        if let Some(cb) = on_field_pick.as_ref() {
                            cb(rel.start_table_id, rel.start_field_id);
                        }
                        return;
                    }
                    match super::hit_test_field(&tables, diag_x, diag_y) {
                        Some((tid, fid))
                            if tid != rel.start_table_id || fid != rel.start_field_id =>
                        {
                            if let Some(cb) = on_relation_drop.as_ref() {
                                cb(rel.start_table_id, rel.start_field_id, tid, fid);
                            }
                        }
                        _ => {
                            if let Some(cb) = on_relation_drag_cancel.as_ref() {
                                cb();
                            }
                        }
                    }
                    return;
                }

                if drag.endpoint_drag.is_some() {
                    // 关系 endpoint 拖动：on_pointermove 已实时写回 store.references，
                    // 这里只需清理 drag_state 并触发一次重绘（让实时预览复位）。
                    // 显式 return 避免走到 fallthrough 误触其他清理路径。
                    live.borrow_mut().rubber = None;
                    rubber_d.set(None);
                    drag_state.set(None);
                    schedule_paint();
                    return;
                }

                if let Some(table_id) = drag.table_id {
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().tables = None;
                    drag_state.set(None);
                    // 纯点击（位移 < 4px）= 选中语义：不写回坐标、不触发持久化，仅重绘复位视觉
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    let new_x = drag.start_table_x + dx / transform.get_untracked().zoom;
                    let new_y = drag.start_table_y + dy / transform.get_untracked().zoom;
                    let (sx, sy) = super::snap_to_grid(new_x, new_y, super::GRID_SIZE);
                    let mut tables = store.tables.get_untracked();
                    if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                        table.x = sx;
                        table.y = sy;
                    }
                    store.tables.set(tables);
                    // D 批：松手吸附后必须通知持久化（S01 保存链路），否则拖表位置不落账
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                live.borrow_mut().tables = None;
                drag_state.set(None);
            }
        };

        // pointercancel：浏览器/系统打断 pointer capture 时触发（Alt+Tab 切窗口、
        // 拖出 viewport、OS 强制收走输入等）。区别于 pointerup：没有 ev.client_x/y 可信，
        // 所以只做防御性清理，不写 store。
        let on_pointercancel = {
            let live = live.clone();
            let schedule_paint = schedule_paint.clone();
            move |_ev: PointerEvent| {
                if drag_state.get_untracked().is_none() {
                    return;
                }
                live.borrow_mut().rubber = None;
                live.borrow_mut().tables = None;
                rubber_d.set(None);
                drag_state.set(None);
                schedule_paint();
            }
        };

        let on_wheel = move |ev: WheelEvent| {
            ev.prevent_default();
            let canvas = match canvas_ref.get() {
                Some(c) => c,
                None => return,
            };
            let mouse_x = ev.client_x() as f64;
            let mouse_y = ev.client_y() as f64;
            // screen_to_diagram 内部用 `mouse - rect.left` 减去 canvas 偏移，
            // 这里反向计算 new_pan 也必须减去 rect.left，否则缩放会累积 rect.left
            // 导致画布"漂"出 viewport。
            let rect = canvas.get_bounding_client_rect();
            let (dx, dy) = screen_to_diagram(mouse_x, mouse_y, &canvas, &transform.get_untracked());
            let anchor_x = mouse_x - rect.left();
            let anchor_y = mouse_y - rect.top();

            let zoom_factor = if ev.delta_y() < 0.0 { 1.1 } else { 1.0 / 1.1 };
            transform.update(|t| {
                let new_zoom = (t.zoom * zoom_factor).clamp(0.1, 5.0);
                t.pan_x = anchor_x - dx * new_zoom;
                t.pan_y = anchor_y - dy * new_zoom;
                t.zoom = new_zoom;
            });
        };

        let on_dblclick = {
            let on_dblclick_blank = on_dblclick_blank.clone();
            move |ev: MouseEvent| {
                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                let (dx, dy) = screen_to_diagram(
                    ev.client_x() as f64,
                    ev.client_y() as f64,
                    &canvas,
                    &transform.get_untracked(),
                );
                let tables = store.tables.get_untracked();
                if super::hit_test(&tables, dx, dy).is_none() {
                    if let Some(cb) = on_dblclick_blank.as_ref() {
                        cb();
                    }
                }
            }
        };

        view! {
            <div class="cdb-canvas-stack">
                <canvas
                    id="editor-canvas"
                    data-testid="editor-canvas"
                    class="cdb-canvas-element"
                    node_ref=canvas_ref
                    on:pointerdown=on_pointerdown
                    on:pointermove=on_pointermove
                    on:pointerup=on_pointerup
                    on:pointercancel=on_pointercancel
                    on:wheel=on_wheel
                    on:dblclick=on_dblclick
                ></canvas>
                <svg class="cdb-rel-overlay" aria-hidden="true">
                    <g transform=move || {
                        let t = transform.get();
                        format!("translate({},{}) scale({})", t.pan_x, t.pan_y, t.zoom)
                    }>
                        <path
                            data-testid="rel-follow-path"
                            fill="none"
                            stroke="transparent"
                            attr:d=move || follow_path.get()
                        ></path>
                        <path
                            class="cdb-rel-rubber-band"
                            data-testid="rel-rubber-band"
                            fill="none"
                            stroke="#5b7cfa"
                            stroke-width="1.5"
                            attr:d=move || rubber_d.get().unwrap_or_default()
                            prop:hidden=move || rubber_d.get().is_none()
                        ></path>
                    </g>
                </svg>
            </div>
        }
    }
}

pub use leptos_canvas::Canvas;

// ─── Pure geometry helpers（可在非 wasm 单测中覆盖 UT-PB-06 / UT-CR-06 / UT-CR-07）──

/// 关系拖线阈值：屏幕像素欧氏距离（除以 zoom 之前）。
pub fn is_relation_drag(dx: f64, dy: f64, threshold: f64) -> bool {
    (dx * dx + dy * dy).sqrt() >= threshold
}

/// 松手网格对齐：`round(n / grid) * grid`。
pub fn snap_to_grid(x: f64, y: f64, grid: f64) -> (f64, f64) {
    ((x / grid).round() * grid, (y / grid).round() * grid)
}

/// feat-table-resize 批次3: draw_table 渲染尺寸纯函数化,
/// 供单测独立验证 width/min_height 消费逻辑(免依赖 CanvasRenderingContext2d)。
/// 返回 (render_width, render_height)。
pub fn compute_table_render_size(table: &Table) -> (f64, f64) {
    let field_count = table.fields.len().max(2);
    let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
    let auto_height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
    let total_height = table
        .min_height
        .map(|h| h as f64)
        .map(|min| min.max(auto_height))
        .unwrap_or(auto_height);
    (width, total_height)
}

/// 源字段右侧锚点（与正式关系线起点一致）。
pub fn field_anchor_start(table: &Table, field_id: &str) -> (f64, f64) {
    // feat-table-resize: 端点 x 消费 table.width,fallback 到 TABLE_WIDTH 默认 230.0
    let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
    (table.x + width, field_anchor_y(table, field_id))
}

/// 拖动中写入临时视觉坐标，不量化网格。
pub fn apply_visual_table_position(tables: &[Table], table_id: &str, x: f64, y: f64) -> Vec<Table> {
    tables
        .iter()
        .map(|t| {
            if t.id == table_id {
                let mut cloned = t.clone();
                cloned.x = x;
                cloned.y = y;
                cloned
            } else {
                t.clone()
            }
        })
        .collect()
}

/// 贝塞尔关系路径（与 `draw_bezier_fields` 同一算法）。
#[derive(Clone, Debug, PartialEq)]
pub struct RelationPath {
    pub x1: f64,
    pub y1: f64,
    pub cx1: f64,
    pub cy1: f64,
    pub cx2: f64,
    pub cy2: f64,
    pub x2: f64,
    pub y2: f64,
}

impl RelationPath {
    pub fn to_svg_d(&self) -> String {
        format!(
            "M{} {} C{} {},{} {},{} {}",
            self.x1, self.y1, self.cx1, self.cy1, self.cx2, self.cy2, self.x2, self.y2
        )
    }
}

fn bezier_controls(x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64, f64, f64) {
    let cx1 = x1 + (x2 - x1) * 0.5;
    let cx2 = x1 + (x2 - x1) * 0.5;
    (cx1, y1, cx2, y2)
}

pub fn calc_path(from: &Table, from_field_id: &str, to: &Table, to_field_id: &str) -> RelationPath {
    let x1 = from.x + TABLE_WIDTH;
    let y1 = field_anchor_y(from, from_field_id);
    let x2 = to.x;
    let y2 = field_anchor_y(to, to_field_id);
    let (cx1, cy1, cx2, cy2) = bezier_controls(x1, y1, x2, y2);
    RelationPath {
        x1,
        y1,
        cx1,
        cy1,
        cx2,
        cy2,
        x2,
        y2,
    }
}

/// 橡皮筋 SVG `d`：起点为源字段锚点，终点为指针坐标。
pub fn rubber_band_path(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let (cx1, cy1, cx2, cy2) = bezier_controls(x1, y1, x2, y2);
    RelationPath {
        x1,
        y1,
        cx1,
        cy1,
        cx2,
        cy2,
        x2,
        y2,
    }
    .to_svg_d()
}

// ─── Pure rendering functions ────────────────────────────────────────────────

/// Main draw dispatcher — clears and redraws all layers.
pub fn draw_canvas(
    ctx: &CanvasRenderingContext2d,
    t: &Transform,
    width: f64,
    height: f64,
    tables: &[Table],
    refs: &[Reference],
    areas: &[Area],
    notes: &[Note],
    remote_presence: &[RemotePresence],
    selected_id: Option<&str>,
    rubber_band: Option<(f64, f64, f64, f64)>,
) {
    let dpr = current_device_pixel_ratio();
    // width/height 是 backing store 像素，clear_rect 必须按 backing store 清空
    ctx.clear_rect(0.0, 0.0, width, height);
    // 画布底色由 .cdb-canvas-container 壳层 CSS 提供（主原型 .canvas 透明 + 壳层 bg-deep 60%），
    // 栅格层不再自填背景，仅绘制点阵与对象。
    let palette = current_palette();

    draw_grid(ctx, t, width, height, palette);

    ctx.save();
    // R-DPR-02：每帧 set_transform(dpr*zoom, ...) 复位，避免 zoom 累乘（UT-RP-03）
    let _ = ctx.set_transform(dpr * t.zoom, 0.0, 0.0, dpr * t.zoom, t.pan_x * dpr, t.pan_y * dpr);
    // R-DPR-05：开启图像平滑，让反走样后的栅格过渡更柔和；线条级 1px 通过 set_line_width 对齐即可
    let _ = ctx.set_image_smoothing_enabled(true);

    for area in areas {
        draw_area(ctx, area, palette);
    }

    for r in refs {
        let from = tables.iter().find(|tbl| tbl.id == r.start_table_id);
        let to = tables.iter().find(|tbl| tbl.id == r.end_table_id);
        if let (Some(f), Some(tbl)) = (from, to) {
            draw_bezier_fields(ctx, f, &r.start_field_id, tbl, &r.end_field_id, palette);
        }
    }

    for table in tables {
        let is_sel = selected_id == Some(&table.id);
        draw_table(ctx, table, is_sel, palette);
    }

    for note in notes {
        draw_note(ctx, note, palette);
    }

    for presence in remote_presence {
        draw_remote_presence(ctx, presence, palette);
    }

    if let Some((x1, y1, x2, y2)) = rubber_band {
        draw_rubber_band(ctx, x1, y1, x2, y2, palette);
    }

    ctx.restore();
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemotePresence {
    pub user_id: String,
    pub display_name: Option<String>,
    pub x: f64,
    pub y: f64,
    pub online: bool,
}

pub fn remote_presence_slots(
    members: impl IntoIterator<Item = (String, Option<String>, bool)>,
) -> Vec<RemotePresence> {
    members
        .into_iter()
        .enumerate()
        .map(|(idx, (user_id, display_name, online))| RemotePresence {
            user_id,
            display_name,
            x: 80.0 + (idx as f64) * 18.0,
            y: 72.0 + (idx as f64) * 14.0,
            online,
        })
        .collect()
}

fn draw_remote_presence(ctx: &CanvasRenderingContext2d, presence: &RemotePresence, palette: &CanvasPalette) {
    if !presence.online {
        return;
    }
    let _ = ctx.set_fill_style_str(palette.presence);
    ctx.begin_path();
    let _ = ctx.arc(presence.x, presence.y, 5.0, 0.0, std::f64::consts::TAU);
    ctx.fill();
    let label = presence
        .display_name
        .as_deref()
        .unwrap_or(presence.user_id.as_str());
    let _ = ctx.set_fill_style_str(palette.text_muted);
    let _ = ctx.set_font(&dpr_font(400, 11.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.fill_text(label, presence.x + 8.0, presence.y + 4.0);
}

fn draw_grid(ctx: &CanvasRenderingContext2d, t: &Transform, width: f64, height: f64, palette: &CanvasPalette) {
    // 主原型点阵：radial-gradient(text-3 @ 30%) 1px / 24px —— 色值已带透明度，不再叠加 global_alpha
    let _ = ctx.set_fill_style_str(palette.grid_dot);

    let dpr = current_device_pixel_ratio();
    let start_x = (-(t.pan_x % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;
    let start_y = (-(t.pan_y % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;

    // width/height 是 backing store 像素（CSS × dpr），转回 CSS 空间再算栅格边界
    let end_x = (width / dpr) / t.zoom + GRID_SIZE;
    let end_y = (height / dpr) / t.zoom + GRID_SIZE;

    let mut y = start_y;
    while y < end_y {
        let mut x = start_x;
        while x < end_x {
            ctx.begin_path();
            let _ = ctx.arc(x, y, 1.0, 0.0, std::f64::consts::TAU);
            ctx.fill();
            x += GRID_SIZE;
        }
        y += GRID_SIZE;
    }
}

/// 画布缩放：放大（步进 1.25x，上限 5x）
pub fn zoom_in(transform: RwSignal<Transform>) {
    transform.update(|t| {
        t.zoom = (t.zoom * 1.25).min(5.0);
    });
}

/// 画布缩放：缩小（步进 0.8x，下限 0.1x）
pub fn zoom_out(transform: RwSignal<Transform>) {
    transform.update(|t| {
        t.zoom = (t.zoom / 1.25).max(0.1);
    });
}

/// 画布缩放：重置为 1x，平移归零
pub fn zoom_reset(transform: RwSignal<Transform>) {
    transform.set(Transform::default());
}

fn draw_table(ctx: &CanvasRenderingContext2d, table: &Table, selected: bool, palette: &CanvasPalette) {
    let field_count = table.fields.len().max(2);
    let (width, total_height) = compute_table_render_size(table);
    let x = table.x;
    let y = table.y;

    // 表体：主原型 .db-table —— radius 14、surface-solid 底、line-strong 描边、柔和投影
    ctx.save();
    let _ = ctx.set_shadow_color("rgba(0, 0, 0, 0.18)");
    let _ = ctx.set_shadow_blur(16.0);
    let _ = ctx.set_shadow_offset_x(0.0);
    let _ = ctx.set_shadow_offset_y(6.0);

    let _ = ctx.set_fill_style_str(palette.table_bg);
    ctx.begin_path();
    round_rect(ctx, x, y, width, total_height, 14.0);
    ctx.fill();
    ctx.restore();

    let _ = ctx.set_stroke_style_str(palette.table_border);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    round_rect(ctx, x, y, TABLE_WIDTH, total_height, 14.0);
    ctx.stroke();

    // 表头：主原型 .table-head —— 自左向右的 tint 渐变（表色或 brand-soft → 透明），非实心填充
    let header_tint = if table.color.trim().is_empty() {
        palette.header_tint
    } else {
        table.color.as_str()
    };
    ctx.save();
    ctx.begin_path();
    round_rect_top(ctx, x, y, TABLE_WIDTH, TABLE_HEADER_HEIGHT, 14.0);
    ctx.clip();
    let gradient = ctx.create_linear_gradient(x, y, x + TABLE_WIDTH, y);
    gradient.add_color_stop(0.0, header_tint).ok();
    gradient.add_color_stop(1.0, "rgba(0,0,0,0)").ok();
    let _ = ctx.set_fill_style_str("rgba(0,0,0,0)");
    ctx.set_fill_style_canvas_gradient(&gradient);
    ctx.fill_rect(x, y, TABLE_WIDTH, TABLE_HEADER_HEIGHT);
    ctx.restore();

    // 表名（750/13px 强色）+ 字段计数（text-3 10px 右对齐）
    let _ = ctx.set_fill_style_str(palette.text_strong);
    let _ = ctx.set_font(&dpr_font(750, 13.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.set_text_align("left");
    let _ = ctx.fill_text(&table.name, x + 11.0, y + TABLE_HEADER_HEIGHT / 2.0);
    let _ = ctx.set_fill_style_str(palette.text_muted);
    let _ = ctx.set_font(&dpr_font(500, 10.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_align("right");
    let _ = ctx.fill_text(
        &table.fields.len().to_string(),
        x + TABLE_WIDTH - 11.0,
        y + TABLE_HEADER_HEIGHT / 2.0,
    );
    let _ = ctx.set_text_align("left");

    // 表头分隔线（line）
    let _ = ctx.set_stroke_style_str(palette.table_border);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(x, y + TABLE_HEADER_HEIGHT);
    ctx.line_to(x + TABLE_WIDTH, y + TABLE_HEADER_HEIGHT);
    ctx.stroke();

    // 字段行：PK 纯文本琥珀标 + 名称 650/11px + 类型等宽 10px text-3（主原型 .table-field）
    for (i, field) in table.fields.iter().enumerate() {
        let fy = y + TABLE_HEADER_HEIGHT + i as f64 * FIELD_ROW_HEIGHT;

        if field.primary {
            let _ = ctx.set_fill_style_str(palette.pk_color);
            let _ = ctx.set_font(&dpr_font(900, 9.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
            let _ = ctx.fill_text("PK", x + 11.0, fy + FIELD_ROW_HEIGHT / 2.0);
        }

        let name_x = if field.primary { x + 36.0 } else { x + 11.0 };
        let _ = ctx.set_fill_style_str(palette.text_strong);
        let _ = ctx.set_font(&dpr_font(650, 11.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let _ = ctx.fill_text(&field.name, name_x, fy + FIELD_ROW_HEIGHT / 2.0);

        let _ = ctx.set_fill_style_str(palette.text_muted);
        let _ = ctx.set_font(&dpr_font(500, 10.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let _ = ctx.set_text_align("right");
        let _ = ctx.fill_text(
            &field.type_,
            x + TABLE_WIDTH - 11.0,
            fy + FIELD_ROW_HEIGHT / 2.0,
        );
        let _ = ctx.set_text_align("left");

        if i + 1 < field_count {
            let _ = ctx.set_stroke_style_str(palette.row_separator);
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(x, fy + FIELD_ROW_HEIGHT);
            ctx.line_to(x + TABLE_WIDTH, fy + FIELD_ROW_HEIGHT);
            ctx.stroke();
        }
    }

    // 选中态：主原型 .is-selected —— brand 描边 + 3px brand-soft 外环
    if selected {
        let _ = ctx.set_stroke_style_str(palette.selected_soft);
        ctx.set_line_width(3.0);
        ctx.begin_path();
        round_rect(ctx, x - 2.5, y - 2.5, TABLE_WIDTH + 5.0, total_height + 5.0, 16.0);
        ctx.stroke();

        let _ = ctx.set_stroke_style_str(palette.selected);
        ctx.set_line_width(1.0);
        ctx.begin_path();
        round_rect(ctx, x, y, TABLE_WIDTH, total_height, 14.0);
        ctx.stroke();
    }
}

fn field_anchor_y(table: &Table, field_id: &str) -> f64 {
    let idx = table
        .fields
        .iter()
        .position(|f| f.id == field_id)
        .unwrap_or(0);
    table.y + TABLE_HEADER_HEIGHT + idx as f64 * FIELD_ROW_HEIGHT + FIELD_ROW_HEIGHT / 2.0
}

fn draw_bezier_fields(
    ctx: &CanvasRenderingContext2d,
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
    palette: &CanvasPalette,
) {
    let path = calc_path(from, from_field_id, to, to_field_id);

    // 主原型 .relation-path-bg：7px surface 光晕垫底，主线 2px brand
    let _ = ctx.set_stroke_style_str(palette.relation_halo);
    ctx.set_line_width(7.0);
    ctx.begin_path();
    ctx.move_to(path.x1, path.y1);
    ctx.bezier_curve_to(path.cx1, path.cy1, path.cx2, path.cy2, path.x2, path.y2);
    ctx.stroke();

    let _ = ctx.set_stroke_style_str(palette.relation);
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(path.x1, path.y1);
    ctx.bezier_curve_to(path.cx1, path.cy1, path.cx2, path.cy2, path.x2, path.y2);
    ctx.stroke();

    draw_arrow_head(ctx, path.cx2, path.cy2, path.x2, path.y2, palette);

    let _ = ctx.set_fill_style_str(palette.relation);
    ctx.begin_path();
    ctx.arc(path.x1, path.y1, 4.0, 0.0, std::f64::consts::TAU).ok();
    ctx.fill();
}

fn draw_rubber_band(ctx: &CanvasRenderingContext2d, x1: f64, y1: f64, x2: f64, y2: f64, palette: &CanvasPalette) {
    let (cx1, cy1, cx2, cy2) = bezier_controls(x1, y1, x2, y2);
    let dash_arr = {
        let a = js_sys::Array::new();
        a.push(&wasm_bindgen::JsValue::from(6.0));
        a.push(&wasm_bindgen::JsValue::from(4.0));
        a
    };
    let _ = ctx.set_line_dash(&dash_arr);
    let _ = ctx.set_stroke_style_str(palette.relation);
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.bezier_curve_to(cx1, cy1, cx2, cy2, x2, y2);
    ctx.stroke();
    let _ = ctx.set_line_dash(&js_sys::Array::new());
}

fn draw_arrow_head(ctx: &CanvasRenderingContext2d, fromx: f64, fromy: f64, tox: f64, toy: f64, palette: &CanvasPalette) {
    let angle = (toy - fromy).atan2(tox - fromx);
    let arrow_len = 10.0;
    let arrow_angle = std::f64::consts::TAU / 6.0;

    let ax1 = tox - arrow_len * (angle - arrow_angle).cos();
    let ay1 = toy - arrow_len * (angle - arrow_angle).sin();
    let ax2 = tox - arrow_len * (angle + arrow_angle).cos();
    let ay2 = toy - arrow_len * (angle + arrow_angle).sin();

    let _ = ctx.set_fill_style_str(palette.relation);
    ctx.begin_path();
    ctx.move_to(tox, toy);
    ctx.line_to(ax1, ay1);
    ctx.line_to(ax2, ay2);
    ctx.close_path();
    ctx.fill();
}

fn draw_area(ctx: &CanvasRenderingContext2d, area: &Area, palette: &CanvasPalette) {
    let _ = ctx.set_fill_style_str(palette.area_bg);
    ctx.fill_rect(area.x, area.y, area.width, area.height);

    let _ = ctx.set_stroke_style_str(palette.area_border);
    ctx.set_line_width(1.0);
    let dash_arr = {
        let a = js_sys::Array::new();
        a.push(&wasm_bindgen::JsValue::from(6.0));
        a.push(&wasm_bindgen::JsValue::from(4.0));
        a
    };
    let _ = ctx.set_line_dash(&dash_arr);
    ctx.stroke_rect(area.x, area.y, area.width, area.height);
    let _ = ctx.set_line_dash(&js_sys::Array::new());

    let _ = ctx.set_fill_style_str(palette.area_border);
    let _ = ctx.set_font(&dpr_font(750, 11.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&area.name, area.x + 10.0, area.y + 10.0);
}

fn draw_note(ctx: &CanvasRenderingContext2d, note: &Note, palette: &CanvasPalette) {
    let note_w = 180.0;
    let note_h = 100.0;
    let _ = ctx.set_fill_style_str(palette.note_bg);
    ctx.fill_rect(note.x, note.y, note_w, note_h);

    let _ = ctx.set_stroke_style_str(palette.note_border);
    ctx.set_line_width(1.0);
    ctx.stroke_rect(note.x, note.y, note_w, note_h);

    let _ = ctx.set_fill_style_str(palette.note_text);
    let _ = ctx.set_font(&dpr_font(400, 11.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("top");

    let words: Vec<&str> = note.content.split_whitespace().collect();
    let mut line = String::new();
    let mut y = note.y + 8.0;
    let mut lines_drawn = 0;
    for word in words {
        let test = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if test.len() > 26 && lines_drawn < 4 {
            let _ = ctx.fill_text(&line, note.x + 8.0, y);
            y += 16.0;
            line = word.to_string();
            lines_drawn += 1;
        } else {
            line = test;
        }
    }
    if lines_drawn < 4 {
        let _ = ctx.fill_text(&line, note.x + 8.0, y);
    }
}

// ─── Hit testing ─────────────────────────────────────────────────────────────

pub fn hit_test_field(tables: &[Table], x: f64, y: f64) -> Option<(String, String)> {
    for table in tables.iter().rev() {
        // feat-table-resize: 命中宽度跟随 table.width,fallback 到 TABLE_WIDTH 默认
        let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
        if x < table.x || x > table.x + width {
            continue;
        }
        if y < table.y + TABLE_HEADER_HEIGHT {
            continue;
        }
        let field_count = table.fields.len().max(1);
        let body_bottom =
            table.y + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
        if y > body_bottom {
            continue;
        }
        let idx = ((y - table.y - TABLE_HEADER_HEIGHT) / FIELD_ROW_HEIGHT).floor() as usize;
        if let Some(field) = table.fields.get(idx) {
            return Some((table.id.clone(), field.id.clone()));
        }
    }
    None
}

pub fn hit_test(tables: &[Table], x: f64, y: f64) -> Option<String> {
    for table in tables.iter().rev() {
        // feat-table-resize: 命中宽度跟随 table.width,fallback 到 TABLE_WIDTH 默认
        let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
        let h = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * table.fields.len().max(2) as f64;
        if x >= table.x && x <= table.x + width && y >= table.y && y <= table.y + h {
            return Some(table.id.clone());
        }
    }
    None
}

// ─── Reference endpoint hit test (B3) ────────────────────────────────────────

/// 端点位置（reference start 或 end）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointEnd {
    Start,
    End,
}

/// 检测 (x, y) 是否命中某条 reference 的端点；返回 (ref_id, end) 或 None
/// 端点几何：起点 = (from.x + TABLE_WIDTH, from.y + HEADER/2)，
///           终点 = (to.x, to.y + HEADER/2)，命中半径 6 像素（zoom 后实际尺寸更小）
pub fn hit_test_endpoint(
    tables: &[Table],
    refs: &[Reference],
    x: f64,
    y: f64,
) -> Option<(String, EndpointEnd)> {
    for r in refs {
        let from = tables.iter().find(|t| t.id == r.start_table_id);
        let to = tables.iter().find(|t| t.id == r.end_table_id);
        if let (Some(f), Some(t)) = (from, to) {
            let sx = f.x + TABLE_WIDTH;
            let sy = f.y + TABLE_HEADER_HEIGHT / 2.0;
            let ex = t.x;
            let ey = t.y + TABLE_HEADER_HEIGHT / 2.0;
            let r2 = 36.0; // 6^2 squared radius
            if (x - sx).powi(2) + (y - sy).powi(2) <= r2 {
                return Some((r.id.clone(), EndpointEnd::Start));
            }
            if (x - ex).powi(2) + (y - ey).powi(2) <= r2 {
                return Some((r.id.clone(), EndpointEnd::End));
            }
        }
    }
    None
}

// ─── Pure function: update reference endpoint (B3) ──────────────────────────

/// 端点 drag 改 start_field_id 或 end_field_id；pure function（不修改原 Vec）
/// - ref_id 不存在 → 返回原 Vec（no-op）
/// - new_field_id == "" → 不更新（避免空值）
pub fn update_reference_endpoint(
    refs: &[Reference],
    ref_id: &str,
    end: EndpointEnd,
    new_field_id: &str,
) -> Vec<Reference> {
    if new_field_id.is_empty() {
        return refs.to_vec();
    }
    refs.iter()
        .map(|r| {
            if r.id != ref_id {
                r.clone()
            } else {
                let mut updated = r.clone();
                match end {
                    EndpointEnd::Start => updated.start_field_id = new_field_id.to_string(),
                    EndpointEnd::End => updated.end_field_id = new_field_id.to_string(),
                }
                updated
            }
        })
        .collect()
}

// ─── Canvas 2D path helpers ──────────────────────────────────────────────────

fn round_rect(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    ctx.arc_to(x + w, y, x + w, y + r, r).ok();
    ctx.line_to(x + w, y + h - r);
    ctx.arc_to(x + w, y + h, x + w - r, y + h, r).ok();
    ctx.line_to(x + r, y + h);
    ctx.arc_to(x, y + h, x, y + h - r, r).ok();
    ctx.line_to(x, y + r);
    ctx.arc_to(x, y, x + r, y, r).ok();
    ctx.close_path();
}

fn round_rect_top(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    ctx.arc_to(x + w, y, x + w, y + r, r).ok();
    ctx.line_to(x + w, y + h);
    ctx.line_to(x, y + h);
    ctx.line_to(x, y + r);
    ctx.arc_to(x, y, x + r, y, r).ok();
    ctx.close_path();
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_core::types::{Field, Reference, Table};

    fn make_ref(id: &str, start_f: &str, end_f: &str) -> Reference {
        Reference {
            id: id.into(),
            name: format!("ref-{id}"),
            start_table_id: "t1".into(),
            end_table_id: "t1".into(),
            start_field_id: start_f.into(),
            end_field_id: end_f.into(),
            type_: "1:N".into(),
            on_delete: "".into(),
            on_update: "".into(),
        }
    }

    #[test]
    fn test_update_reference_endpoint_start_ut_cr_03() {
        let refs = vec![make_ref("r1", "f1", "f2")];
        let original = refs.clone();
        let result = update_reference_endpoint(&refs, "r1", EndpointEnd::Start, "f2");
        assert_eq!(result.len(), 1, "UT-CR-03: 返回 Vec 长度应为 1");
        assert_eq!(result[0].start_field_id, "f2", "UT-CR-03: start_field_id 应更新为 f2");
        assert_eq!(result[0].end_field_id, "f2", "UT-CR-03: end_field_id 应保持 f2");
        assert_eq!(refs[0].start_field_id, original[0].start_field_id, "UT-CR-03: 原始 Vec 不应被修改（pure function）");
    }

    #[test]
    fn test_update_reference_endpoint_end_ut_cr_04() {
        let refs = vec![make_ref("r1", "f1", "f2")];
        let result = update_reference_endpoint(&refs, "r1", EndpointEnd::End, "f3");
        assert_eq!(result.len(), 1, "UT-CR-04: 返回 Vec 长度应为 1");
        assert_eq!(result[0].end_field_id, "f3", "UT-CR-04: end_field_id 应更新为 f3");
        assert_eq!(result[0].start_field_id, "f1", "UT-CR-04: start_field_id 应保持 f1");
    }

    #[test]
    fn test_update_reference_endpoint_nonexistent_ut_cr_05() {
        let refs = vec![make_ref("r1", "f1", "f2")];
        let original_first = refs[0].clone();
        let result = update_reference_endpoint(&refs, "nonexistent", EndpointEnd::Start, "f2");
        assert_eq!(result.len(), 1, "UT-CR-05: 返回 Vec 长度应为 1");
        assert_eq!(result[0].start_field_id, original_first.start_field_id, "UT-CR-05: 不存在的 ref_id 应 no-op");
        assert_eq!(result[0].end_field_id, original_first.end_field_id, "UT-CR-05: end_field_id 也应不变");
    }

    // --- UT-PB-01 — 字段级 hit test ---

    #[test]
    fn test_hit_test_field_ut_pb_01() {
        use crate::editor_core::types::{Field, Table};

        let table = Table {
            id: "t1".into(),
            name: "users".into(),
            x: 100.0,
            y: 130.0,
            color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: "f1".into(),
                name: "id".into(),
                type_: "INT".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: false,
                increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        };
        let tables = vec![table];
        let hit_y = 130.0 + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT / 2.0;
        let result = hit_test_field(&tables, 150.0, hit_y);
        assert_eq!(
            result,
            Some(("t1".into(), "f1".into())),
            "UT-PB-01: 字段行命中应返回 (table_id, field_id)"
        );
    }

    #[test]
    fn ut_fe_s05_10_remote_presence_slots_are_stable() {
        let slots = remote_presence_slots(vec![
            ("u1".to_string(), Some("Dev".to_string()), true),
            ("u2".to_string(), None, false),
        ]);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].x, 80.0);
        assert_eq!(slots[0].y, 72.0);
        assert_eq!(slots[1].x, 98.0);
        assert_eq!(slots[1].y, 86.0);
        assert!(!slots[1].online);
    }

    fn fixture_table(id: &str, field_id: &str, x: f64, y: f64) -> Table {
        Table {
            id: id.into(),
            name: id.into(),
            x,
            y,
            color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: field_id.into(),
                name: "id".into(),
                type_: "INT".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: false,
                increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        }
    }

    #[test]
    fn ut_cr_06_snap_to_grid_rounds_on_release() {
        // 生产合同：松手网格即 GRID_SIZE 常量本身（core-CR §1：20，禁止写成主原型的 12/24）
        assert_eq!(GRID_SIZE, 20.0, "UT-CR-06: 生产松手网格必须是 20（验收 §7.5）");
        let (x, y) = snap_to_grid(133.4, 87.1, GRID_SIZE);
        assert_eq!((x, y), (140.0, 80.0), "UT-CR-06: snap_to_grid(133.4, 87.1, 20) → (140, 80)");
        let visual = apply_visual_table_position(
            &[fixture_table("a", "f1", 100.0, 100.0)],
            "a",
            133.4,
            87.1,
        );
        assert_eq!(visual[0].x, 133.4, "UT-CR-06: 拖动中保持未量化坐标");
        assert_eq!(visual[0].y, 87.1);
    }

    #[test]
    fn ut_cr_07_calc_path_uses_current_visual_coords() {
        let a = fixture_table("a", "fa", 100.0, 100.0);
        let b = fixture_table("b", "fb", 400.0, 100.0);
        let before = calc_path(&a, "fa", &b, "fb");
        let mut moved = a.clone();
        moved.x = 160.0;
        moved.y = 140.0;
        let during = calc_path(&moved, "fa", &b, "fb");
        assert_ne!(before.x1, during.x1, "UT-CR-07: 不得仍使用 100/100");
        assert_eq!(during.x1, 160.0 + TABLE_WIDTH);
        assert_eq!(during.y1, field_anchor_y(&moved, "fa"));
    }

    // ─── feat-table-resize 批次3 步骤2：draw_table + hit_test 消费 table.width/min_height ───

    /// draw_table.width 跟随 table.width,None 走 TABLE_WIDTH 默认。
    /// 通过检测 round_rect 入参验证（公开 helper: `compute_table_render_size`）
    #[test]
    fn feat_table_resize_draw_table_width_uses_table_width_some() {
        use crate::editor_core::types::{Field, Table};
        let mut t = Table {
            id: "t".into(),
            name: "T".into(),
            x: 0.0, y: 0.0,
            color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "f".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: Some(400),
            min_height: None,
        };
        let (w, h) = compute_table_render_size(&t);
        assert_eq!(w, 400.0, "feat-table-resize: width=Some(400) → 400");
        // height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT × max(2, fields.len())
        assert_eq!(h, TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * 2.0);
    }

    #[test]
    fn feat_table_resize_draw_table_width_uses_default_when_none() {
        use crate::editor_core::types::{Field, Table};
        let t = Table {
            id: "t".into(), name: "T".into(),
            x: 0.0, y: 0.0,
            color: "#000".into(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "f".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        };
        let (w, _) = compute_table_render_size(&t);
        assert_eq!(w, TABLE_WIDTH, "feat-table-resize: width=None → TABLE_WIDTH 默认");
    }

    #[test]
    fn feat_table_resize_draw_table_min_height_overrides_auto() {
        use crate::editor_core::types::{Field, Table};
        // 1 字段 → auto = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT × max(2,1) = 43 + 35*2 = 113
        // min_height=300 应胜出
        let t = Table {
            id: "t".into(), name: "T".into(),
            x: 0.0, y: 0.0,
            color: "#000".into(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "f".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: Some(300),
        };
        let (_, h) = compute_table_render_size(&t);
        assert_eq!(h, 300.0, "feat-table-resize: min_height=Some(300) 胜出 auto=113");
    }

    #[test]
    fn feat_table_resize_draw_table_min_height_none_uses_auto() {
        use crate::editor_core::types::{Field, Table};
        // 5 字段 → auto = 43 + 35*5 = 218;min_height=None 走 auto
        let fields: Vec<Field> = (0..5).map(|i| Field {
            id: format!("f{i}"), name: format!("f{i}"), type_: "INT".into(),
            default: String::new(), check: String::new(),
            primary: false, unique: false, not_null: false, increment: false,
            comment: String::new(),
            tag: String::new(),
        }).collect();
        let t = Table {
            id: "t".into(), name: "T".into(),
            x: 0.0, y: 0.0,
            color: "#000".into(), comment: String::new(),
            fields,
            indices: Vec::new(),
            width: None,
            min_height: None,
        };
        let (_, h) = compute_table_render_size(&t);
        assert_eq!(h, TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * 5.0);
    }

    #[test]
    fn feat_table_resize_hit_test_field_uses_table_width() {
        let mut t = fixture_table("t", "f1", 100.0, 100.0);
        t.width = Some(400);
        // 字段 y 起点 ≈ 165;x=350 在 width=400 内,x=600 严格超出(边界 500)
        assert!(hit_test_field(&[t.clone()], 350.0, 165.0).is_some(),
            "feat-table-resize: x=350 在 width=400 内 → 命中");
        assert!(hit_test_field(&[t.clone()], 600.0, 165.0).is_none(),
            "feat-table-resize: x=600 严格超出 width=400(边界 500) → 不命中");
    }

    #[test]
    fn feat_table_resize_hit_test_uses_table_width() {
        let mut t = fixture_table("t", "f1", 100.0, 100.0);
        t.width = Some(400);
        // 表范围 x ∈ [100, 500];y ∈ [100, 100+auto_height]
        assert!(hit_test(&[t.clone()], 450.0, 110.0).is_some(),
            "feat-table-resize: 表级命中 x=450 在 width=400 内");
        assert!(hit_test(&[t.clone()], 600.0, 110.0).is_none(),
            "feat-table-resize: 表级未命中 x=600 超出 width=400(边界 500)");
    }

    /// Apply 闭环的纯函数级单元测试：
    /// 模拟 SetTableWidthModal.Apply 的写入语义,验证 store.tables[*].width 被更新。
    #[test]
    fn feat_table_resize_apply_writes_width_to_all_tables() {
        use crate::editor_core::types::{Field, Table};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        let t1 = Table {
            id: "t1".into(), name: "A".into(),
            x: 0.0, y: 0.0, color: "#000".into(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "f".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None, min_height: None,
        };
        let t2 = t1.clone();
        let mut t2 = t2; t2.id = "t2".into(); t2.name = "B".into();
        store.tables.set(vec![t1, t2]);
        // 模拟 Apply 写入
        store.tables.update(|tables| {
            for t in tables.iter_mut() { t.width = Some(350); }
        });
        store.dirty.set(true);
        let widths: Vec<u32> = store.tables.get().iter().map(|t| t.width.unwrap_or(0)).collect();
        assert_eq!(widths, vec![350, 350], "feat-table-resize: Apply 后所有 table.width=350");
        assert!(store.dirty.get(), "feat-table-resize: Apply 后 dirty=true");
    }
}
