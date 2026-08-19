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
use web_sys::{CanvasRenderingContext2d, MouseEvent, WheelEvent};

// ─── Canvas constants ────────────────────────────────────────────────────────

const TABLE_WIDTH: f64 = 220.0;
const TABLE_HEADER_HEIGHT: f64 = 32.0;
const FIELD_ROW_HEIGHT: f64 = 24.0;
const AREA_COLOR: &str = "rgba(59, 130, 246, 0.08)";
const AREA_BORDER_COLOR: &str = "rgba(59, 130, 246, 0.4)";
const NOTE_BG: &str = "#fef3c7";
const NOTE_BORDER: &str = "#f59e0b";
const GRID_SIZE: f64 = 20.0;
const CANVAS_BG: &str = "#f8fafc";
const TABLE_BG: &str = "#ffffff";
const TABLE_BORDER: &str = "#dbe3ea";
const TABLE_HEADER: &str = "#175e7a";
const TABLE_HEADER_SELECTED: &str = "#0e7490";
const TEXT_STRONG: &str = "#1e293b";
const TEXT_MUTED: &str = "#64748b";
const RELATION_COLOR: &str = "#5b7cfa";

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
struct DragState {
    table_id: Option<String>,
    endpoint_drag: Option<(String, EndpointEnd)>, // (ref_id, end) when dragging an endpoint
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
            start_mouse_x: 0.0,
            start_mouse_y: 0.0,
            start_pan_x: 0.0,
            start_pan_y: 0.0,
            start_table_x: 0.0,
            start_table_y: 0.0,
        }
    }
}

// ─── Leptos canvas component ─────────────────────────────────────────────────

mod leptos_canvas {
    use super::*;
    use crate::editor_core::EditorStore;
    use leptos::html;
    use leptos::*;

    /// Main canvas component for the database diagram editor.
    #[component]
    pub fn Canvas(
        store: EditorStore,
        transform: RwSignal<Transform>,
        remote_presence: RwSignal<Vec<RemotePresence>>,
        on_select: Option<Box<dyn Fn(String) + 'static>>,
        on_deselect: Option<Box<dyn Fn() + 'static>>,
        on_dblclick_blank: Option<Box<dyn Fn() + 'static>>,
        rel_tool_active: RwSignal<bool>,
        on_field_pick: Option<Box<dyn Fn(String, String) + 'static>>,
    ) -> impl IntoView {
        let canvas_ref = create_node_ref::<html::Canvas>();
        let selected_id = create_rw_signal(None::<String>);
        let drag_state = create_rw_signal(None::<DragState>);

        let screen_to_diagram = move |screen_x: f64, screen_y: f64, canvas: &web_sys::HtmlCanvasElement, t: &Transform| -> (f64, f64) {
            let rect = canvas.get_bounding_client_rect();
            let canvas_x = screen_x - rect.left();
            let canvas_y = screen_y - rect.top();
            let diagram_x = (canvas_x - t.pan_x) / t.zoom;
            let diagram_y = (canvas_y - t.pan_y) / t.zoom;
            (diagram_x, diagram_y)
        };

        create_effect(move |_| {
            let Some(canvas) = canvas_ref.get() else { return; };
            let ctx = match canvas.get_context("2d") {
                Ok(Some(ctx)) => match ctx.dyn_into::<CanvasRenderingContext2d>() {
                    Ok(ctx) => ctx,
                    Err(_) => return,
                },
                _ => return,
            };

            // 使 canvas 内部分辨率匹配容器尺寸
            if let Some(parent) = canvas.parent_element() {
                let w = parent.client_width().max(1) as u32;
                let h = parent.client_height().max(1) as u32;
                if canvas.width() != w || canvas.height() != h {
                    canvas.set_width(w);
                    canvas.set_height(h);
                }
            }

            let t = transform.get();
            let tables = store.tables.get();
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
            );
        });

        let on_mousedown = move |ev: MouseEvent| {
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
            // 关系工具：优先字段命中
            if rel_tool_active.get_untracked() {
                if let Some((tid, fid)) = super::hit_test_field(&tables, dx, dy) {
                    if let Some(cb) = &on_field_pick {
                        cb(tid, fid);
                    }
                    return;
                }
                return;
            }
            // B3: hit-test endpoint first (priority over table body)
            if let Some((ref_id, end)) = super::hit_test_endpoint(&tables, &refs, dx, dy) {
                drag_state.set(Some(DragState {
                    table_id: None,
                    endpoint_drag: Some((ref_id, end)),
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
                if let Some(cb) = &on_select {
                    cb(id.clone());
                }
                drag_state.set(Some(DragState {
                    table_id: Some(id),
                    endpoint_drag: None,
                    start_mouse_x: ev.client_x() as f64,
                    start_mouse_y: ev.client_y() as f64,
                    start_pan_x: transform.get_untracked().pan_x,
                    start_pan_y: transform.get_untracked().pan_y,
                    start_table_x: table_x,
                    start_table_y: table_y,
                }));
            } else {
                selected_id.set(None);
                if let Some(cb) = &on_deselect {
                    cb();
                }
                let t = transform.get_untracked();
                drag_state.set(Some(DragState {
                    table_id: None,
                    endpoint_drag: None,
                    start_mouse_x: ev.client_x() as f64,
                    start_mouse_y: ev.client_y() as f64,
                    start_pan_x: t.pan_x,
                    start_pan_y: t.pan_y,
                    start_table_x: 0.0,
                    start_table_y: 0.0,
                }));
            }
        };

        let on_mousemove = move |ev: MouseEvent| {
            let Some(drag) = drag_state.get_untracked() else { return; };
            let canvas = match canvas_ref.get() {
                Some(c) => c,
                None => return,
            };
            let dx = ev.client_x() as f64 - drag.start_mouse_x;
            let dy = ev.client_y() as f64 - drag.start_mouse_y;

            if let Some((ref_id, end)) = &drag.endpoint_drag {
                // B3: endpoint drag — find nearest field in the connected table
                let (dx_d, dy_d) = screen_to_diagram(
                    ev.client_x() as f64,
                    ev.client_y() as f64,
                    &canvas,
                    &transform.get_untracked(),
                );
                let tables = store.tables.get_untracked();
                let target_table_id = match end {
                    EndpointEnd::Start => store.references.get_untracked().iter()
                        .find(|r| r.id == *ref_id).map(|r| r.start_table_id.clone()),
                    EndpointEnd::End => store.references.get_untracked().iter()
                        .find(|r| r.id == *ref_id).map(|r| r.end_table_id.clone()),
                };
                if let Some(tid) = target_table_id {
                    let new_field = tables.iter()
                        .find(|t| t.id == tid)
                        .and_then(|t| {
                            t.fields.iter().min_by(|a, b| {
                                let ay = t.y + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT *
                                    t.fields.iter().position(|f| f.id == a.id).unwrap_or(0) as f64;
                                let by = t.y + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT *
                                    t.fields.iter().position(|f| f.id == b.id).unwrap_or(0) as f64;
                                let da = (dx_d - t.x).powi(2) + (dy_d - ay).powi(2);
                                let db = (dx_d - t.x).powi(2) + (dy_d - by).powi(2);
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            })
                        })
                        .map(|f| f.id.clone());
                    if let Some(fid) = new_field {
                        let refs_now = store.references.get_untracked();
                        let updated = super::update_reference_endpoint(&refs_now, ref_id, *end, &fid);
                        store.references.set(updated);
                    }
                }
            } else if let Some(table_id) = &drag.table_id {
                let new_x = drag.start_table_x + dx / transform.get_untracked().zoom;
                let new_y = drag.start_table_y + dy / transform.get_untracked().zoom;
                let mut tables = store.tables.get_untracked();
                if let Some(table) = tables.iter_mut().find(|t| &t.id == table_id) {
                    table.x = new_x;
                    table.y = new_y;
                    store.tables.set(tables);
                }
            } else {
                transform.update(|t| {
                    t.pan_x = drag.start_pan_x + dx;
                    t.pan_y = drag.start_pan_y + dy;
                });
            }
        };

        let on_mouseup = move |_ev: MouseEvent| {
            drag_state.set(None);
        };

        let on_wheel = move |ev: WheelEvent| {
            ev.prevent_default();
            let canvas = match canvas_ref.get() {
                Some(c) => c,
                None => return,
            };
            let mouse_x = ev.client_x() as f64;
            let mouse_y = ev.client_y() as f64;
            let (dx, dy) = screen_to_diagram(mouse_x, mouse_y, &canvas, &transform.get_untracked());

            let zoom_factor = if ev.delta_y() < 0.0 { 1.1 } else { 1.0 / 1.1 };
            transform.update(|t| {
                let new_zoom = (t.zoom * zoom_factor).clamp(0.1, 5.0);
                t.pan_x = mouse_x - dx * new_zoom;
                t.pan_y = mouse_y - dy * new_zoom;
                t.zoom = new_zoom;
            });
        };

        let on_dblclick = move |ev: MouseEvent| {
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
                if let Some(cb) = &on_dblclick_blank {
                    cb();
                }
            }
        };

        view! {
            <canvas
                id="editor-canvas"
                data-testid="editor-canvas"
                class="cdb-canvas-element"
                node_ref=canvas_ref
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
                on:wheel=on_wheel
                on:dblclick=on_dblclick
            ></canvas>
        }
    }
}

pub use leptos_canvas::Canvas;

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
) {
    ctx.clear_rect(0.0, 0.0, width, height);
    let _ = ctx.set_fill_style_str(CANVAS_BG);
    ctx.fill_rect(0.0, 0.0, width, height);

    draw_grid(ctx, t, width, height);

    ctx.save();
    let _ = ctx.translate(t.pan_x, t.pan_y);
    let _ = ctx.scale(t.zoom, t.zoom);

    for area in areas {
        draw_area(ctx, area);
    }

    for r in refs {
        let from = tables.iter().find(|tbl| tbl.id == r.start_table_id);
        let to = tables.iter().find(|tbl| tbl.id == r.end_table_id);
        if let (Some(f), Some(tbl)) = (from, to) {
            draw_bezier_fields(ctx, f, &r.start_field_id, tbl, &r.end_field_id);
        }
    }

    for table in tables {
        let is_sel = selected_id == Some(&table.id);
        draw_table(ctx, table, is_sel);
    }

    for note in notes {
        draw_note(ctx, note);
    }

    for presence in remote_presence {
        draw_remote_presence(ctx, presence);
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

fn draw_remote_presence(ctx: &CanvasRenderingContext2d, presence: &RemotePresence) {
    if !presence.online {
        return;
    }
    let _ = ctx.set_fill_style_str("#ef4444");
    ctx.begin_path();
    let _ = ctx.arc(presence.x, presence.y, 5.0, 0.0, std::f64::consts::TAU);
    ctx.fill();
    let label = presence
        .display_name
        .as_deref()
        .unwrap_or(presence.user_id.as_str());
    let _ = ctx.set_fill_style_str("#0f172a");
    let _ = ctx.set_font("11px sans-serif");
    let _ = ctx.fill_text(label, presence.x + 8.0, presence.y + 4.0);
}

fn draw_grid(ctx: &CanvasRenderingContext2d, t: &Transform, width: f64, height: f64) {
    let _ = ctx.set_fill_style_str("#cbd5e1");
    let _ = ctx.set_global_alpha(0.55);

    let start_x = (-(t.pan_x % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;
    let start_y = (-(t.pan_y % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;

    let end_x = width / t.zoom + GRID_SIZE;
    let end_y = height / t.zoom + GRID_SIZE;

    let mut y = start_y;
    while y < end_y {
        let mut x = start_x;
        while x < end_x {
            ctx.begin_path();
            let _ = ctx.arc(x, y, 0.85, 0.0, std::f64::consts::TAU);
            ctx.fill();
            x += GRID_SIZE;
        }
        y += GRID_SIZE;
    }

    let _ = ctx.set_global_alpha(1.0);
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

fn draw_table(ctx: &CanvasRenderingContext2d, table: &Table, selected: bool) {
    let field_count = table.fields.len().max(2);
    let total_height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
    let x = table.x;
    let y = table.y;

    ctx.save();
    let _ = ctx.set_shadow_color("rgba(15, 23, 42, 0.14)");
    let _ = ctx.set_shadow_blur(12.0);
    let _ = ctx.set_shadow_offset_x(0.0);
    let _ = ctx.set_shadow_offset_y(4.0);

    let _ = ctx.set_fill_style_str(TABLE_BG);
    ctx.begin_path();
    round_rect(ctx, x, y, TABLE_WIDTH, total_height, 8.0);
    ctx.fill();
    ctx.restore();

    let _ = ctx.set_stroke_style_str(TABLE_BORDER);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    round_rect(ctx, x, y, TABLE_WIDTH, total_height, 8.0);
    ctx.stroke();

    let header_color = if selected {
        TABLE_HEADER_SELECTED
    } else if table.color.trim().is_empty() {
        TABLE_HEADER
    } else {
        table.color.as_str()
    };
    let _ = ctx.set_fill_style_str(header_color);
    ctx.begin_path();
    round_rect_top(ctx, x, y, TABLE_WIDTH, TABLE_HEADER_HEIGHT, 8.0);
    ctx.fill();

    let _ = ctx.set_fill_style_str("#ffffff");
    let _ = ctx.set_font("bold 13px sans-serif");
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(&table.name, x + 10.0, y + TABLE_HEADER_HEIGHT / 2.0);

    let _ = ctx.set_stroke_style_str(TABLE_BORDER);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(x, y + TABLE_HEADER_HEIGHT);
    ctx.line_to(x + TABLE_WIDTH, y + TABLE_HEADER_HEIGHT);
    ctx.stroke();

    let _ = ctx.set_font("12px sans-serif");
    for (i, field) in table.fields.iter().enumerate() {
        let fy = y + TABLE_HEADER_HEIGHT + i as f64 * FIELD_ROW_HEIGHT;

        if field.primary {
            draw_pill(ctx, x + 8.0, fy + 5.0, 18.0, 12.0, "#f59e0b", "PK", "#ffffff");
        }

        let name_x = if field.primary { x + 34.0 } else { x + 12.0 };
        let _ = ctx.set_fill_style_str(TEXT_STRONG);
        let _ = ctx.fill_text(&field.name, name_x, fy + FIELD_ROW_HEIGHT / 2.0);

        let type_text = field.type_.as_str();
        let pill_w = (type_text.len() as f64 * 6.2 + 14.0).clamp(38.0, 92.0);
        draw_pill(
            ctx,
            x + TABLE_WIDTH - pill_w - 10.0,
            fy + 5.0,
            pill_w,
            14.0,
            field_type_color(type_text),
            type_text,
            "#ffffff",
        );

        if i + 1 < field_count {
            let _ = ctx.set_stroke_style_str(TABLE_BORDER);
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(x, fy + FIELD_ROW_HEIGHT);
            ctx.line_to(x + TABLE_WIDTH, fy + FIELD_ROW_HEIGHT);
            ctx.stroke();
        }
    }

    if selected {
        let _ = ctx.set_stroke_style_str("#0ea5b7");
        ctx.set_line_width(2.0);
        ctx.begin_path();
        round_rect(ctx, x - 3.0, y - 3.0, TABLE_WIDTH + 6.0, total_height + 6.0, 10.0);
        ctx.stroke();
    }
}

fn draw_pill(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    bg: &str,
    text: &str,
    fg: &str,
) {
    let _ = ctx.set_fill_style_str(bg);
    ctx.begin_path();
    round_rect(ctx, x, y, width, height, 3.0);
    ctx.fill();
    let _ = ctx.set_fill_style_str(fg);
    let _ = ctx.set_font("10px sans-serif");
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(text, x + 6.0, y + height / 2.0);
}

fn field_type_color(type_name: &str) -> &'static str {
    let upper = type_name.to_uppercase();
    if upper.contains("INT") || upper.contains("SERIAL") {
        "#3b82f6"
    } else if upper.contains("CHAR") || upper.contains("TEXT") {
        "#10b981"
    } else if upper.contains("BOOL") {
        "#8b5cf6"
    } else if upper.contains("DATE") || upper.contains("TIME") {
        "#f59e0b"
    } else {
        TEXT_MUTED
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
) {
    let x1 = from.x + TABLE_WIDTH;
    let y1 = field_anchor_y(from, from_field_id);
    let x2 = to.x;
    let y2 = field_anchor_y(to, to_field_id);
    let cx1 = x1 + (x2 - x1) * 0.5;
    let cx2 = x1 + (x2 - x1) * 0.5;

    let _ = ctx.set_stroke_style_str(RELATION_COLOR);
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.bezier_curve_to(cx1, y1, cx2, y2, x2, y2);
    ctx.stroke();

    draw_arrow_head(ctx, cx2, y2, x2, y2);

    let _ = ctx.set_fill_style_str(RELATION_COLOR);
    ctx.begin_path();
    ctx.arc(x1, y1, 4.0, 0.0, std::f64::consts::TAU).ok();
    ctx.fill();
}

fn draw_bezier(ctx: &CanvasRenderingContext2d, from: &Table, to: &Table) {
    let x1 = from.x + TABLE_WIDTH;
    let y1 = from.y + TABLE_HEADER_HEIGHT / 2.0;
    let x2 = to.x;
    let y2 = to.y + TABLE_HEADER_HEIGHT / 2.0;
    let cx1 = x1 + (x2 - x1) * 0.5;
    let cx2 = x1 + (x2 - x1) * 0.5;

    let _ = ctx.set_stroke_style_str(RELATION_COLOR);
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.bezier_curve_to(cx1, y1, cx2, y2, x2, y2);
    ctx.stroke();

    draw_arrow_head(ctx, cx2, y2, x2, y2);

    let _ = ctx.set_fill_style_str(RELATION_COLOR);
    ctx.begin_path();
    ctx.arc(x1, y1, 4.0, 0.0, std::f64::consts::TAU).ok();
    ctx.fill();
}

fn draw_arrow_head(ctx: &CanvasRenderingContext2d, fromx: f64, fromy: f64, tox: f64, toy: f64) {
    let angle = (toy - fromy).atan2(tox - fromx);
    let arrow_len = 10.0;
    let arrow_angle = std::f64::consts::TAU / 6.0;

    let ax1 = tox - arrow_len * (angle - arrow_angle).cos();
    let ay1 = toy - arrow_len * (angle - arrow_angle).sin();
    let ax2 = tox - arrow_len * (angle + arrow_angle).cos();
    let ay2 = toy - arrow_len * (angle + arrow_angle).sin();

    let _ = ctx.set_fill_style_str(RELATION_COLOR);
    ctx.begin_path();
    ctx.move_to(tox, toy);
    ctx.line_to(ax1, ay1);
    ctx.line_to(ax2, ay2);
    ctx.close_path();
    ctx.fill();
}

fn draw_area(ctx: &CanvasRenderingContext2d, area: &Area) {
    let _ = ctx.set_fill_style_str(AREA_COLOR);
    ctx.fill_rect(area.x, area.y, area.width, area.height);

    let _ = ctx.set_stroke_style_str(AREA_BORDER_COLOR);
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

    let _ = ctx.set_fill_style_str(AREA_BORDER_COLOR);
    let _ = ctx.set_font("bold 11px sans-serif");
    let _ = ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&area.name, area.x + 6.0, area.y + 6.0);
}

fn draw_note(ctx: &CanvasRenderingContext2d, note: &Note) {
    let note_w = 180.0;
    let note_h = 100.0;
    let _ = ctx.set_fill_style_str(NOTE_BG);
    ctx.fill_rect(note.x, note.y, note_w, note_h);

    let _ = ctx.set_stroke_style_str(NOTE_BORDER);
    ctx.set_line_width(1.0);
    ctx.stroke_rect(note.x, note.y, note_w, note_h);

    let _ = ctx.set_fill_style_str("#78350f");
    let _ = ctx.set_font("12px sans-serif");
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
        if x < table.x || x > table.x + TABLE_WIDTH {
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
        let h = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * table.fields.len().max(2) as f64;
        if x >= table.x && x <= table.x + TABLE_WIDTH && y >= table.y && y <= table.y + h {
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
    use crate::editor_core::types::Reference;

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
            }],
            indices: Vec::new(),
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
}
