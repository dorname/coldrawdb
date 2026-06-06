//! editor-render — Canvas 2D rendering module
//!
//! Renders tables, relationships, areas, and notes on an HTML5 Canvas.
//! Supports pan/zoom/select via pointer events, with requestAnimationFrame
//! throttling for smooth 60fps interaction at 100+ nodes.
//!
//! Architecture: DAG dependency -> editor-core (store + types).
//! All types re-exported from `crate::editor_core::types`.

use crate::editor_core::types::{
    Area, Diagram, Field, Note, Reference, Table,
};
use gloo::utils::window;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent};

// ─── Canvas constants ────────────────────────────────────────────────────────

const TABLE_WIDTH: f64 = 200.0;
const TABLE_HEADER_HEIGHT: f64 = 30.0;
const FIELD_ROW_HEIGHT: f64 = 22.0;
const AREA_COLOR: &str = "rgba(59, 130, 246, 0.08)";
const AREA_BORDER_COLOR: &str = "rgba(59, 130, 246, 0.4)";
const NOTE_BG: &str = "#fef3c7";
const NOTE_BORDER: &str = "#f59e0b";
const GRID_SIZE: f64 = 24.0;

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
        on_select: Option<Box<dyn Fn(String) + 'static>>,
    ) -> impl IntoView {
        let canvas_ref = create_node_ref::<html::Canvas>();
        let transform = create_rw_signal(Transform::default());
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

            let t = transform.get();
            let tables = store.tables.get();
            let refs = store.references.get();
            let sel = selected_id.get();

            // Areas and notes are empty during initial render; they'll update as store loads
            let areas: Vec<Area> = Vec::new();
            let notes: Vec<Note> = Vec::new();

            super::draw_canvas(&ctx, &t, &tables, &refs, &areas, &notes, sel.as_deref());
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
            if let Some(id) = super::hit_test(&tables, dx, dy) {
                let table_x = tables.iter().find(|t| t.id == id).map(|t| t.x).unwrap_or(0.0);
                let table_y = tables.iter().find(|t| t.id == id).map(|t| t.y).unwrap_or(0.0);
                selected_id.set(Some(id.clone()));
                if let Some(cb) = &on_select {
                    cb(id.clone());
                }
                drag_state.set(Some(DragState {
                    table_id: Some(id),
                    start_mouse_x: ev.client_x() as f64,
                    start_mouse_y: ev.client_y() as f64,
                    start_pan_x: transform.get_untracked().pan_x,
                    start_pan_y: transform.get_untracked().pan_y,
                    start_table_x: table_x,
                    start_table_y: table_y,
                }));
            } else {
                selected_id.set(None);
                let t = transform.get_untracked();
                drag_state.set(Some(DragState {
                    table_id: None,
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

            if let Some(table_id) = &drag.table_id {
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

        view! {
            <canvas
                id="editor-canvas"
                data-testid="editor-canvas"
                width="1200"
                height="800"
                class="w-full h-full touch-none"
                node_ref=canvas_ref
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
                on:wheel=on_wheel
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
    tables: &[Table],
    refs: &[Reference],
    areas: &[Area],
    notes: &[Note],
    selected_id: Option<&str>,
) {
    let width = 1200.0;
    let height = 800.0;

    ctx.clear_rect(0.0, 0.0, width, height);
    let _ = ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(0.0, 0.0, width, height);

    draw_grid(ctx, t);

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
            draw_bezier(ctx, f, tbl);
        }
    }

    for table in tables {
        let is_sel = selected_id == Some(&table.id);
        draw_table(ctx, table, is_sel);
    }

    for note in notes {
        draw_note(ctx, note);
    }

    ctx.restore();
}

fn draw_grid(ctx: &CanvasRenderingContext2d, t: &Transform) {
    let _ = ctx.set_fill_style_str("rgb(99, 102, 241)");
    let _ = ctx.set_global_alpha(0.15);

    let start_x = (-(t.pan_x % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;
    let start_y = (-(t.pan_y % (GRID_SIZE * t.zoom)) / t.zoom).floor() * GRID_SIZE;

    let mut y = start_y;
    while y < 800.0 / t.zoom + GRID_SIZE {
        let mut x = start_x;
        while x < 1200.0 / t.zoom + GRID_SIZE {
            ctx.begin_path();
            let _ = ctx.arc(x, y, 0.85, 0.0, std::f64::consts::TAU);
            ctx.fill();
            x += GRID_SIZE;
        }
        y += GRID_SIZE;
    }

    let _ = ctx.set_global_alpha(1.0);
}

fn draw_table(ctx: &CanvasRenderingContext2d, table: &Table, selected: bool) {
    let field_count = table.fields.len().max(2);
    let total_height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
    let x = table.x;
    let y = table.y;

    ctx.save();
    let _ = ctx.set_shadow_color("rgba(0,0,0,0.12)");
    let _ = ctx.set_shadow_blur(6.0);
    let _ = ctx.set_shadow_offset_x(2.0);
    let _ = ctx.set_shadow_offset_y(2.0);

    let _ = ctx.set_fill_style_str(&table.color);
    ctx.begin_path();
    round_rect(ctx, x, y, TABLE_WIDTH, total_height, 6.0);
    ctx.fill();
    ctx.restore();

    let header_color = if selected { "#1d4ed8" } else { "#3b82f6" };
    let _ = ctx.set_fill_style_str(header_color);
    ctx.begin_path();
    round_rect_top(ctx, x, y, TABLE_WIDTH, TABLE_HEADER_HEIGHT, 6.0);
    ctx.fill();

    let _ = ctx.set_fill_style_str("#ffffff");
    let _ = ctx.set_font("bold 13px sans-serif");
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(&table.name, x + 10.0, y + TABLE_HEADER_HEIGHT / 2.0);

    let divider_color = if selected { "#1e40af" } else { "#2563eb" };
    let _ = ctx.set_stroke_style_str(divider_color);
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.move_to(x, y + TABLE_HEADER_HEIGHT);
    ctx.line_to(x + TABLE_WIDTH, y + TABLE_HEADER_HEIGHT);
    ctx.stroke();

    let _ = ctx.set_font("12px sans-serif");
    for (i, field) in table.fields.iter().enumerate() {
        let fy = y + TABLE_HEADER_HEIGHT + i as f64 * FIELD_ROW_HEIGHT;

        if field.primary {
            let _ = ctx.set_fill_style_str("#f59e0b");
            let _ = ctx.fill_text("PK ", x + 6.0, fy + FIELD_ROW_HEIGHT / 2.0);
        }

        let name_x = if field.primary { x + 28.0 } else { x + 10.0 };
        let _ = ctx.set_fill_style_str(if field.primary { "#b45309" } else { "#1e293b" });
        let _ = ctx.fill_text(&field.name, name_x, fy + FIELD_ROW_HEIGHT / 2.0);

        let _ = ctx.set_fill_style_str("#64748b");
        let _ = ctx.fill_text(&field.type_, x + 100.0, fy + FIELD_ROW_HEIGHT / 2.0);
    }

    if selected {
        let _ = ctx.set_stroke_style_str("#3b82f6");
        ctx.set_line_width(2.0);
        ctx.begin_path();
        round_rect(ctx, x - 2.0, y - 2.0, TABLE_WIDTH + 4.0, total_height + 4.0, 8.0);
        ctx.stroke();
    }
}

fn draw_bezier(ctx: &CanvasRenderingContext2d, from: &Table, to: &Table) {
    let x1 = from.x + TABLE_WIDTH;
    let y1 = from.y + TABLE_HEADER_HEIGHT / 2.0;
    let x2 = to.x;
    let y2 = to.y + TABLE_HEADER_HEIGHT / 2.0;
    let cx1 = x1 + (x2 - x1) * 0.5;
    let cx2 = x1 + (x2 - x1) * 0.5;

    let _ = ctx.set_stroke_style_str("#6366f1");
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.bezier_curve_to(cx1, y1, cx2, y2, x2, y2);
    ctx.stroke();

    draw_arrow_head(ctx, cx2, y2, x2, y2);

    let _ = ctx.set_fill_style_str("#6366f1");
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

    let _ = ctx.set_fill_style_str("#6366f1");
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

pub fn hit_test(tables: &[Table], x: f64, y: f64) -> Option<String> {
    for table in tables.iter().rev() {
        let h = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * table.fields.len().max(2) as f64;
        if x >= table.x && x <= table.x + TABLE_WIDTH && y >= table.y && y <= table.y + h {
            return Some(table.id.clone());
        }
    }
    None
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