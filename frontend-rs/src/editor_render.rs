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
use std::collections::HashMap;
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

/// R-PERF-10：拖拽活跃期渲染分辨率上限。backing store 像素 = CSS × dpr，软件合成
/// （SwiftShader）成本随像素数线性增长——实测 dpr=2（3200×1900）时 Viz 合成线程
/// ~19ms/帧，超出 16.7ms 帧预算，是拖拽掉帧的根因。拖拽中把有效 dpr 压到 1
/// （像素数 ÷4），松手后下一帧恢复全分辨率。纯函数便于 UT（UT-CR-DPR-01）。
pub const DRAG_RENDER_DPR_CAP: f64 = 1.0;
pub fn capped_drag_dpr(dpr: f64, drag_active: bool) -> f64 {
    if drag_active {
        dpr.min(DRAG_RENDER_DPR_CAP)
    } else {
        dpr
    }
}

thread_local! {
    /// R-PERF-10：render effect 每帧写入的有效 dpr；绘制路径（backing store 换算 /
    /// CTM / 精灵缓存指纹）一律经 `effective_device_pixel_ratio()` 读取以保持一致。
    static RENDER_DPR: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
}

/// 绘制路径统一取 dpr 的入口：render effect 设置的有效值优先，否则回退真实
/// devicePixelRatio（宿主单测无窗口时 fallback 1.0）。
pub fn effective_device_pixel_ratio() -> f64 {
    RENDER_DPR
        .with(|c| c.get())
        .unwrap_or_else(current_device_pixel_ratio)
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

// ux-canvas-batch 批次4 步骤 6 (条目 28/29): rAF 统一调度
// - schedule_render_dedup（可测同步核）—— 无 rAF 副作用，pending 状态机可单元测试
// - schedule_render（壳，已删除——条目 29 修正：原私有 static PENDING 外部不可达，
//   首调后置真永不可清，若有调用方第二次起永远 noop，设计性错误 + 死代码）
// 集成（OffscreenCanvas/drawImage/request_redraw 改道）留后续性能专项

/// ux-canvas-batch 批次4 步骤 6 (条目 28/29): rAF 调度去重可测同步核（UT-MM-30）
/// - 首次入队：返 true 执行；pending 置 true
/// - 二次入队（pending=true）：返 false noop
/// - rAF 回调清 pending（state.set(false)）后再入队可执行
/// - 无 rAF 副作用，纯函数可测
/// - **集成留后续性能专项**（条目 29 外环代决：本批仅交付可测基础设施；C-2 帧率 <16ms
///   不入门禁，request_redraw 全改道 + OffscreenCanvas 渲染接入均不背工程量）
pub fn schedule_render_dedup<F>(state: &std::cell::Cell<bool>, render_fn: F) -> bool
where
    F: FnOnce(),
{
    if state.get() {
        // 已 pending，二次入队 noop
        return false;
    }
    state.set(true);
    render_fn();
    true
}

/// ux-canvas-batch 批次4 步骤 6 (条目 28/29): 文本离屏缓存键
/// - 缓存键 = (text, font_weight, font_px, dpr)
/// - 字体度量（measureText）+ 预渲染（OffscreenCanvas）走同一键
/// - **集成留后续性能专项**（条目 29 外环代决：本批仅交付键结构 + PartialEq/Hash；
///   OffscreenCanvas 预渲染 + drawImage 复用路径属渲染层接入，不背工程量）
/// 注：wasm-bindgen OffscreenCanvas 支持版本 ≥ 0.2.83（frontend-rs Cargo.toml 既有）
#[derive(Clone, Debug)]
pub struct TextCacheKey {
    pub text: String,
    pub font_weight: u32,
    pub font_px: f64,
    pub dpr: u32, // devicePixelRatio × 100（避免浮点键）
}

impl PartialEq for TextCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font_weight == other.font_weight
            && (self.font_px - other.font_px).abs() < 0.01
            && self.dpr == other.dpr
    }
}

impl Eq for TextCacheKey {}

impl std::hash::Hash for TextCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_weight.hash(state);
        self.dpr.hash(state);
        // font_px 不参与 hash（用 eq 的容差比较）
    }
}

/// F-WF-03 / R-PERF-03：探测字体家族是否真正可用，逐级降级。
/// 探测名与加载名严格 1:1 对齐（条目 19/28 P3 修正）：
///   primary → "Plus Jakarta Sans"
///   fallback 1 → "Noto Sans SC"（Google Fonts CDN `Noto+Sans+SC:wght@400;500;700&display=swap`）
///   fallback 2 → "PingFang SC"（macOS 系统字体）
///   fallback 3 → "-apple-system, BlinkMacSystemFont"
///   全部失败 → mono（ui-monospace）
/// 仅作为 set_font 时的 family 段使用；返回的字符串可直接拼到 `format!("...{}")`。
///
/// R-PERF-03：`document.fonts().check()` 探测结果按 `(primary, mono)` 键进程级缓存
/// （thread_local HashMap）；`document.fonts` `loadingdone` 事件触发缓存失效重探。
/// 同一帧内同一字体家族至多 1 次 DOM 探测，禁止每对象每帧重复探测。
pub fn resolve_canvas_font_family(primary: &str, mono: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let key = (primary.to_string(), mono.to_string());
        return FONT_FAMILY_CACHE.with(|c| {
            let mut cache = c.borrow_mut();
            resolve_font_family_cached(&mut cache, key, || probe_canvas_font_family(primary, mono))
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // host 单测环境无 DOM 可探测，与原 None-window 分支一致退化为 mono。
        let _ = primary;
        mono.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static FONT_FAMILY_CACHE: std::cell::RefCell<HashMap<(String, String), String>> =
        std::cell::RefCell::new(HashMap::new());
    static FONT_CACHE_LISTENER_ARMED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

#[cfg(target_arch = "wasm32")]
fn invalidate_font_family_cache() {
    FONT_FAMILY_CACHE.with(|c| c.borrow_mut().clear());
}

/// R-PERF-03：`loadingdone` 监听进程级只挂一次（FontFaceSet 事件不冒泡到 window）。
#[cfg(target_arch = "wasm32")]
fn arm_font_cache_invalidation(doc: &web_sys::Document) {
    if FONT_CACHE_LISTENER_ARMED.with(|a| a.replace(true)) {
        return;
    }
    let cb = wasm_bindgen::closure::Closure::<dyn FnMut()>::wrap(Box::new(move || {
        invalidate_font_family_cache();
    }));
    let fonts = doc.fonts();
    let target: &web_sys::EventTarget = fonts.as_ref();
    let _ = target.add_event_listener_with_callback(
        "loadingdone",
        cb.as_ref().unchecked_ref::<js_sys::Function>(),
    );
    cb.forget();
}

/// R-PERF-03 缓存核（纯逻辑，UT-CR-FONTCACHE-01 可直接单测）：
/// 命中即返；未命中走 probe 并写入缓存——同帧同族至多 1 次探测。
fn resolve_font_family_cached(
    cache: &mut HashMap<(String, String), String>,
    key: (String, String),
    probe: impl FnOnce() -> String,
) -> String {
    if let Some(hit) = cache.get(&key) {
        return hit.clone();
    }
    let resolved = probe();
    cache.insert(key, resolved.clone());
    resolved
}

/// F-WF-03：实际 DOM 探测（每次调用最多 5 次 `fonts.check`，结果会被进程级缓存吸收）。
#[cfg(target_arch = "wasm32")]
fn probe_canvas_font_family(primary: &str, mono: &str) -> String {
    let win = match web_sys::window() {
        Some(w) => w,
        None => return mono.to_string(),
    };
    let doc = match win.document() {
        Some(d) => d,
        None => return mono.to_string(),
    };
    arm_font_cache_invalidation(&doc);
    let fonts = doc.fonts();
    // 探测名 = 加载名严格 1:1（v1 错误：`fonts.check("Source Han Sans CN")` 与 Noto+Sans+SC 加载名不匹配）
    let candidates = [
        primary,
        "Noto Sans SC",
        "PingFang SC",
        "-apple-system",
        "BlinkMacSystemFont",
    ];
    for candidate in candidates.iter() {
        if fonts.check(&format!("1em \"{}\"", candidate)).unwrap_or(false) {
            return candidate.to_string();
        }
    }
    mono.to_string()
}

/// 把 ctx 当前矩阵复位为 `dpr × zoom`，避免 zoom 累乘（UT-RP-03）。
/// R-PERF-10：用有效 dpr（拖拽降采样时 =1），与 backing store 尺寸同源。
pub fn apply_dpr_zoom_transform(ctx: &CanvasRenderingContext2d, zoom: f64) {
    let dpr = effective_device_pixel_ratio();
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

/// 当前主题是否为暗色；`data-mode` 缺失时默认暗色（同主原型与用户决策）。
fn current_theme_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|el| el.get_attribute("data-mode"))
        .map(|m| m != "light")
        .unwrap_or(true)
}

/// 逐帧取当前画布调色板；`data-mode` 缺失时默认暗色（同主原型与用户决策）。
fn current_palette() -> &'static CanvasPalette {
    if current_theme_dark() { &PALETTE_DARK } else { &PALETTE_LIGHT }
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

/// §3.3 / §4：缩放 clamp 边界（与现行 on_wheel / zoom_in / zoom_out 一致口径）。
pub const ZOOM_MIN: f64 = 0.1;
pub const ZOOM_MAX: f64 = 5.0;

/// §3.3 锚定缩放纯函数（UT-CR-ZOOM-01）：缩放前后 anchor（canvas CSS 坐标系内的点）
/// 下的世界点映射到屏幕的位置不变（锚定不变量）。
///   world   = (anchor − pan) / zoom
///   zoom'   = clamp(zoom × factor, ZOOM_MIN, ZOOM_MAX)
///   pan'    = anchor − world × zoom'      // clamp 后仍按 clamp 后 zoom 计算，锚定优先
pub fn zoom_transform_at_anchor(t: &Transform, anchor_css: (f64, f64), factor: f64) -> Transform {
    let new_zoom = (t.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
    let world_x = (anchor_css.0 - t.pan_x) / t.zoom;
    let world_y = (anchor_css.1 - t.pan_y) / t.zoom;
    Transform {
        pan_x: anchor_css.0 - world_x * new_zoom,
        pan_y: anchor_css.1 - world_y * new_zoom,
        zoom: new_zoom,
    }
}

// R-PERF-05：工具栏 zoom 按钮（editor_panels，无 schedule_paint 句柄）经此钩子
// 请求画布重绘；由 Canvas 组件挂载时注册（进程内单画布）。
thread_local! {
    static PAINT_HOOK: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>> =
        std::cell::RefCell::new(None);
}

/// R-PERF-05：请求画布重绘（供 editor_panels 的 zoom_in / zoom_out / zoom_reset 调用）。
pub fn request_canvas_repaint() {
    PAINT_HOOK.with(|h| {
        if let Some(f) = h.borrow().as_ref() {
            f();
        }
    });
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
    // p0-fix 定点 2：区域框选/便签放置的创建拖拽
    create_drag: Option<CreateDrag>,
    // redesign-listview-type-length-canvas-fix：便签/区域拖动（id, 起始 x, 起始 y）——
    // 修复「命中只选中、无拖拽分支」；拖动中只写视觉层，松手才落账（对齐表拖动三段式）
    note_drag: Option<(String, f64, f64)>,
    area_drag: Option<(String, f64, f64)>,
    pointer_id: i32,
    start_mouse_x: f64,
    start_mouse_y: f64,
    start_pan_x: f64,
    start_pan_y: f64,
    start_table_x: f64,
    start_table_y: f64,
}

/// p0-fix 定点 2：画布创建工具（区域 / 便签）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateToolKind {
    Area,
    Note,
}

/// p0-fix 定点 2：创建拖拽进行态（diagram 坐标）
#[derive(Clone, Copy, Debug)]
struct CreateDrag {
    tool: CreateToolKind,
    start_x: f64,
    start_y: f64,
    cur_x: f64,
    cur_y: f64,
    moved: bool,
}

impl Default for DragState {
    fn default() -> Self {
        DragState {
            table_id: None,
            endpoint_drag: None,
            rel_drag: None,
            create_drag: None,
            note_drag: None,
            area_drag: None,
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
    // R-PERF-04：表拖动覆盖层只存被拖对象的 (id, x, y)——pointermove 禁止整 Vec<Table>
    // 深克隆。松手落账路径直接原地改 store（get_untracked + iter_mut），
    // apply_visual_table_position 仅保留为纯函数供 UT 与落账语义验证。
    table_pos: Option<(String, f64, f64)>,
    rubber: Option<(f64, f64, f64, f64)>,
    /// p0-fix 定点 2：区域拖框预览矩形 (x, y, w, h)
    area_preview: Option<(f64, f64, f64, f64)>,
    /// redesign-listview-type-length-canvas-fix：便签/区域拖动中的视觉坐标覆盖（松手才落账 store）
    notes: Option<Vec<Note>>,
    areas: Option<Vec<Area>>,
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
        /// relation-inspector-and-ddl-io：点击连线命中（返回 reference id）→ 选中 + Inspector（不再弹详情模态）
        on_reference_pick: Option<Box<dyn Fn(String) + 'static>>,
        /// p0-fix 定点 2：当前创建工具（Some → 十字光标 + 拖拽创建区域 / 点击放置便签）
        create_tool: RwSignal<Option<CreateToolKind>>,
        /// p0-fix 定点 2：区域拖框落账（x, y, width, height）
        on_area_create: Option<Box<dyn Fn(f64, f64, f64, f64) + 'static>>,
        /// p0-fix 定点 2：便签点击放置落账（x, y）
        on_note_create: Option<Box<dyn Fn(f64, f64) + 'static>>,
        /// p0-fix 定点 2：点击区域命中（返回 area id）→ 选中 + Inspector
        on_area_pick: Option<Box<dyn Fn(String) + 'static>>,
        /// p0-fix 定点 2：点击便签命中（返回 note id）→ 选中 + Inspector
        on_note_pick: Option<Box<dyn Fn(String) + 'static>>,
        // 主题模式（"dark"/"light"）：绘制 effect 需跟踪以在主题切换时重刷调色板
        theme_mode: RwSignal<String>,
    ) -> impl IntoView {
        let canvas_ref = create_node_ref::<html::Canvas>();
        let selected_id = create_rw_signal(None::<String>);
        let selected_ref_id = create_rw_signal(None::<String>);
        let selected_area_id = create_rw_signal(None::<String>);
        let selected_note_id = create_rw_signal(None::<String>);
        let drag_state = create_rw_signal(None::<DragState>);
        let rubber_d = create_rw_signal(None::<String>);
        let follow_path = create_rw_signal(String::new());
        let frame_tick = create_rw_signal(0u32);
        let live = Rc::new(RefCell::new(LivePaint::default()));
        // R-PERF-11：表拖拽幽灵层（无关系线的表拖拽时主画布零重绘，见顶部区块注释）
        let ghost: Rc<RefCell<Option<super::GhostDrag>>> = Rc::new(RefCell::new(None));
        // R-PERF-08：渲染 effect 的 DOM 写守护——记录上一帧值，仅变化才写信号/属性
        let last_rubber_d: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let last_follow_d: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
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
        let on_reference_pick = Rc::new(on_reference_pick);
        let on_area_create = Rc::new(on_area_create);
        let on_note_create = Rc::new(on_note_create);
        let on_area_pick = Rc::new(on_area_pick);
        let on_note_pick = Rc::new(on_note_pick);

        // R-PERF-05：wheel/pan 的 transform 更新经 rAF 合并（pending_transform 意图通道）——
        // 同一帧内多次事件只落账最后一次，一帧至多一次 draw_canvas。
        // 渲染 effect 不订阅 transform（get_untracked 读取），重绘统一由 frame_tick 驱动。
        let pending_transform: Rc<RefCell<Option<Transform>>> = Rc::new(RefCell::new(None));
        {
            let raf_pending = raf_pending.clone();
            let pending_transform = pending_transform.clone();
            let c = Closure::wrap(Box::new(move |_: JsValue| {
                raf_pending.set(false);
                // 先 take 再 set：避免 `if let` 临时借用延长到 transform.set（订阅者
                // 回调可能重入 pending_transform，导致 RefCell 双重借用 panic）
                let pending_next = pending_transform.borrow_mut().take();
                if let Some(next) = pending_next {
                    transform.set(next);
                }
                frame_tick.update(|n| *n = n.wrapping_add(1));
            }) as Box<dyn FnMut(JsValue)>);
            *raf_closure.borrow_mut() = Some(c);
        }

        let schedule_paint: Rc<dyn Fn()> = {
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

        {
            // R-PERF-05：注册工具栏 zoom 按钮的重绘钩子（editor_panels 无 schedule_paint 句柄）
            let schedule_paint = schedule_paint.clone();
            super::PAINT_HOOK.with(|h| *h.borrow_mut() = Some(schedule_paint));
        }

        // R-PERF-05：事件处理内读取 transform 前先把 rAF 待落账的 wheel/pan 意图落账——
        // hit test / 松手落账坐标永远基于已提交的 transform，不会读到上一帧的 stale 值。
        let current_transform: Rc<dyn Fn() -> Transform> = Rc::new({
            let pending_transform = pending_transform.clone();
            move || {
                // 先 take 再 set：避免 `if let` 临时借用延长到 transform.set——
                // set 同步触发订阅者（SVG 覆盖层 / zoom 标签等），若其回调再读
                // pending_transform 会造成 RefCell 双重借用 panic（事件被吞的根因级隐患）。
                let pending_next = pending_transform.borrow_mut().take();
                if let Some(next) = pending_next {
                    transform.set(next);
                }
                transform.get_untracked()
            }
        });

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
            let schedule_paint = schedule_paint.clone();
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
                    schedule_paint(); // R-PERF-10：取消路径同样恢复全分辨率重绘
                    if let Some(cb) = on_relation_drag_cancel.as_ref() {
                        cb();
                    }
                }
            })
            .forget();
        }

        {
            let live = live.clone();
            let ghost = ghost.clone();
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
                // R-PERF-10：任意拖拽活跃（表/关系/便签/区域/框选/平移）期间把有效 dpr
                // 压到 DRAG_RENDER_DPR_CAP，降低软件合成像素负载。用 get_untracked——
                // 本 effect 由 frame_tick 统一驱动，拖拽起止无需额外触发（否则每次
                // pointerdown/up 多一次重绘，破坏 ST-CR-PAN-01 的 paint 有界不变量；
                // 且纯点击不该付出 backing store 重建成本）。dpr 切换实际发生在拖拽
                // 首个 move 帧与松手落账帧。
                let drag_active = drag_state.get_untracked().is_some();
                let dpr = super::capped_drag_dpr(super::current_device_pixel_ratio(), drag_active);
                super::RENDER_DPR.with(|c| c.set(Some(dpr)));
                let w = (css_w * dpr).round() as u32;
                let h = (css_h * dpr).round() as u32;
                if canvas.width() != w || canvas.height() != h {
                    canvas.set_width(w);
                    canvas.set_height(h);
                }
                // CSS 布局尺寸由 `.cdb-canvas-element { width:100%; height:100% }` 控制，
                // backing store 已通过 set_width/set_height 放大为 dpr 倍，无需内联 style
            }

            // R-PERF-05：transform 用 get_untracked（effect 不订阅 transform，重绘由
            // frame_tick 统一驱动）；Vec 信号一律 .with() 传引用，禁止 .get() 深克隆整表。
            let t = transform.get_untracked();
            let (table_pos, rubber, area_preview, live_notes, live_areas) = {
                let live_g = live.borrow();
                (
                    live_g.table_pos.clone(),
                    live_g.rubber,
                    live_g.area_preview,
                    live_g.notes.clone(),
                    live_g.areas.clone(),
                )
            };
            let table_override = table_pos
                .as_ref()
                .map(|(id, x, y)| (id.as_str(), *x, *y));
            let sel = selected_id.with(|s| s.clone());
            let sel_ref = selected_ref_id.with(|s| s.clone());
            let sel_area = selected_area_id.with(|s| s.clone());
            let sel_note = selected_note_id.with(|s| s.clone());
            // R-PERF-11：幽灵层接管中的表 id（主画布跳过绘制）
            let ghost_skip = ghost.borrow().as_ref().map(|g| g.table_id.clone());

            let width = canvas.width() as f64;
            let height = canvas.height() as f64;
            // R-PERF-10：css 尺寸必须按「生效 dpr」反推，与上方 backing store 换算同源
            let css_w = width / super::effective_device_pixel_ratio();
            let css_h = height / super::effective_device_pixel_ratio();

            store.tables.with(|tables| {
                store.references.with(|refs| {
                    store.areas.with(|store_areas| {
                        store.notes.with(|store_notes| {
                            remote_presence.with(|presence| {
                                let areas = live_areas.as_deref().unwrap_or(store_areas);
                                let notes = live_notes.as_deref().unwrap_or(store_notes);
                                super::draw_canvas(
                                    &ctx,
                                    &t,
                                    css_w,
                                    css_h,
                                    tables,
                                    refs,
                                    areas,
                                    notes,
                                    presence,
                                    sel.as_deref(),
                                    sel_ref.as_deref(),
                                    sel_area.as_deref(),
                                    sel_note.as_deref(),
                                    rubber,
                                    area_preview,
                                    table_override,
                                    ghost_skip.as_deref(),
                                );
                            });
                        });
                    });
                });
            });

            {
                // R-PERF-08：rubber 路径仅变化才写信号（原实现每帧 set(None) 空转触发订阅）
                let next = rubber.map(|(x1, y1, x2, y2)| super::rubber_band_path(x1, y1, x2, y2));
                let mut last = last_rubber_d.borrow_mut();
                if *last != next {
                    *last = next.clone();
                    rubber_d.set(next);
                }
            }

            // follow_path 调试通道（ST-CR-02）：端点表坐标消费拖动覆盖层（UT-CR-07 跟手语义）
            store.tables.with(|tables| {
                store.references.with(|refs| {
                    if let Some(first) = refs.first() {
                        let from = tables
                            .iter()
                            .find(|tbl| tbl.id == first.start_table_id)
                            .map(|tbl| super::table_with_override(tbl, table_override));
                        let to_tbl = tables
                            .iter()
                            .find(|tbl| tbl.id == first.end_table_id)
                            .map(|tbl| super::table_with_override(tbl, table_override));
                        if let (Some(from), Some(to_tbl)) = (from, to_tbl) {
                            let d = super::calc_path(&from, &first.start_field_id, &to_tbl, &first.end_field_id)
                                .to_svg_d();
                            // R-PERF-08：仅路径变化才写信号 + DOM 属性
                            if super::dom_write_guard(&mut last_follow_d.borrow_mut(), &d) {
                                follow_path.set(d.clone());
                                let _ = canvas.set_attribute("data-follow-path", &d);
                            }
                        }
                    } else if super::dom_write_guard(&mut last_follow_d.borrow_mut(), "") {
                        follow_path.set(String::new());
                        let _ = canvas.set_attribute("data-follow-path", "");
                    }
                });
            });
        });
        }

        // R-PERF-11 兜底：任何路径把 drag_state 置 None（Escape / cancel / 非常规退出）
        // 都同步撤掉幽灵层；正常 pointerup 已在落账前同步清理，这里是安全网
        {
            let ghost = ghost.clone();
            create_effect(move |_| {
                if drag_state.get().is_none() {
                    ghost.borrow_mut().take();
                }
            });
        }
        {
            create_effect(move |_| {
                let active = create_tool.get().is_some();
                if let Some(canvas) = canvas_ref.get() {
                    let ws: &web_sys::HtmlCanvasElement = &canvas;
                    let el: &web_sys::HtmlElement = ws.unchecked_ref();
                    let _ = el.style().set_property("cursor", if active { "crosshair" } else { "" });
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
            let current_transform = current_transform.clone();
            move |ev: PointerEvent| {
                // R-PERF-05：先把 rAF 待落账的 wheel/pan 意图落账，hit test 基于已提交的 transform
                let t_now = current_transform();
                // 防御：清掉任何 stale drag_state（pointercancel 未触发的情况下，
                // 比如浏览器在 React 树外丢失 pointer capture 的极端场景）。
                // 不清的话上一次 endpoint/table drag 残留会让本次 pointerdown
                // 被 on_pointermove 当作"延续"误更新 store。
                if drag_state.get_untracked().is_some() {
                    drag_state.set(None);
                    live.borrow_mut().rubber = None;
                    live.borrow_mut().table_pos = None;
                    live.borrow_mut().area_preview = None;
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
                    &t_now,
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
                            create_drag: None,
                            note_drag: None,
                            area_drag: None,
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
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                    return;
                }
                if !read_only {
                    if let Some(tool) = create_tool.get_untracked() {
                        // p0-fix 定点 2：创建工具激活 → 整个画布都是创建热区，吞掉本次 pointerdown
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            endpoint_drag: None,
                            rel_drag: None,
                            create_drag: Some(CreateDrag {
                                tool,
                                start_x: dx,
                                start_y: dy,
                                cur_x: dx,
                                cur_y: dy,
                                moved: false,
                            }),
                            note_drag: None,
                            area_drag: None,
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
                }
                if let Some((ref_id, end)) = super::hit_test_endpoint(&tables, &refs, dx, dy) {
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: Some((ref_id, end)),
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
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
                    selected_ref_id.set(None);
                    selected_area_id.set(None);
                    selected_note_id.set(None);
                    if let Some(cb) = on_select.as_ref() {
                        cb(id.clone());
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    live.borrow_mut().table_pos = None;
                    drag_state.set(Some(DragState {
                        table_id: Some(id),
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: table_x,
                        start_table_y: table_y,
                    }));
                } else if let Some(ref_id) = super::hit_test_reference(&tables, &refs, dx, dy) {
                    // relation-inspector-and-ddl-io：点击连线（表未命中时）→ 选中高亮 + Inspector 展示（不再弹详情模态）
                    // 命中顺序在表之后：连线被表遮住时点击应选中表而非不可见的线
                    selected_id.set(None);
                    selected_ref_id.set(Some(ref_id.clone()));
                    selected_area_id.set(None);
                    selected_note_id.set(None);
                    if let Some(cb) = on_reference_pick.as_ref() {
                        cb(ref_id);
                    }
                    schedule_paint();
                } else if let Some(note_id) = super::hit_test_note(&store.notes.get_untracked(), dx, dy) {
                    // p0-fix 定点 2：点击便签 → 选中高亮 + Inspector 编辑面板
                    // redesign-listview-type-length-canvas-fix：同时启动便签拖拽（视觉层跟随，松手落账）
                    selected_id.set(None);
                    selected_ref_id.set(None);
                    selected_area_id.set(None);
                    selected_note_id.set(Some(note_id.clone()));
                    let (note_x, note_y) = store
                        .notes
                        .get_untracked()
                        .iter()
                        .find(|n| n.id == note_id)
                        .map(|n| (n.x, n.y))
                        .unwrap_or((0.0, 0.0));
                    if let Some(cb) = on_note_pick.as_ref() {
                        cb(note_id.clone());
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: Some((note_id, note_x, note_y)),
                        area_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                    schedule_paint();
                } else if let Some(area_id) = super::hit_test_area(&store.areas.get_untracked(), dx, dy) {
                    // p0-fix 定点 2：点击区域 → 选中高亮 + Inspector 编辑面板（命中顺序在便签之后）
                    // redesign-listview-type-length-canvas-fix：同时启动区域拖拽（视觉层跟随，松手落账）
                    selected_id.set(None);
                    selected_ref_id.set(None);
                    selected_note_id.set(None);
                    selected_area_id.set(Some(area_id.clone()));
                    let (area_x, area_y) = store
                        .areas
                        .get_untracked()
                        .iter()
                        .find(|a| a.id == area_id)
                        .map(|a| (a.x, a.y))
                        .unwrap_or((0.0, 0.0));
                    if let Some(cb) = on_area_pick.as_ref() {
                        cb(area_id.clone());
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: Some((area_id, area_x, area_y)),
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                    schedule_paint();
                } else {
                    selected_id.set(None);
                    selected_ref_id.set(None);
                    selected_area_id.set(None);
                    selected_note_id.set(None);
                    if let Some(cb) = on_deselect.as_ref() {
                        cb();
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: 0.0,
                        start_table_y: 0.0,
                    }));
                }
            }
        };

        let on_pointermove = {
            let live = live.clone();
            let ghost = ghost.clone();
            let schedule_paint = schedule_paint.clone();
            let on_relation_drag_start = on_relation_drag_start.clone();
            let current_transform = current_transform.clone();
            let pending_transform = pending_transform.clone();
            move |ev: PointerEvent| {
                // wire-frontend-collab-ws：presence 光标上报。presence_active()=false（未连接/
                // 非房间）时直接短路，不触碰 get_bounding_client_rect，保护 R-PERF 热路径。
                if crate::collab_client::presence_active() {
                    if let Some(canvas) = canvas_ref.get() {
                        let t_now = current_transform();
                        let (px, py) = screen_to_diagram(
                            ev.client_x() as f64,
                            ev.client_y() as f64,
                            &canvas,
                            &t_now,
                        );
                        crate::collab_client::report_cursor(px, py);
                    }
                }
                let Some(drag) = drag_state.get_untracked() else {
                    return;
                };
                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                // R-PERF-05：hit/拖动坐标基于已提交的 transform（先落账 rAF 待处理的 wheel/pan）
                let t_now = current_transform();
                let dx = ev.client_x() as f64 - drag.start_mouse_x;
                let dy = ev.client_y() as f64 - drag.start_mouse_y;

                if let Some(cd) = &drag.create_drag {
                    // p0-fix 定点 2：创建拖拽——区域实时画虚线预览框；便签只记录 moved
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &t_now,
                    );
                    let crossed = super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD);
                    drag_state.update(|d| {
                        if let Some(ds) = d {
                            if let Some(c) = &mut ds.create_drag {
                                c.cur_x = diag_x;
                                c.cur_y = diag_y;
                                c.moved = c.moved || crossed;
                            }
                        }
                    });
                    if cd.tool == CreateToolKind::Area {
                        live.borrow_mut().area_preview =
                            Some((cd.start_x.min(diag_x), cd.start_y.min(diag_y),
                                  (diag_x - cd.start_x).abs(), (diag_y - cd.start_y).abs()));
                        schedule_paint();
                    }
                    return;
                }

                if let Some(rel) = drag.rel_drag.clone() {
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &t_now,
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
                        &t_now,
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
                            // R-PERF-09：最近字段未变则不写 store——避免拖动中每个
                            // pointermove 整 Vec 落账并通知全部订阅者
                            let cur_fid = refs_now
                                .iter()
                                .find(|r| r.id == *ref_id)
                                .map(|r| match end {
                                    EndpointEnd::Start => r.start_field_id.as_str(),
                                    EndpointEnd::End => r.end_field_id.as_str(),
                                });
                            if cur_fid != Some(fid.as_str()) {
                                let updated =
                                    super::update_reference_endpoint(&refs_now, ref_id, *end, &fid);
                                store.references.set(updated);
                            }
                        }
                    }
                } else if let Some((note_id, start_x, start_y)) = &drag.note_drag {
                    // redesign-listview-type-length-canvas-fix：便签拖动中只写视觉层（ST-CR-NOTE-01：
                    // mousemove 不触发 PUT），松手才落账 store
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let mut visual = store.notes.get_untracked();
                    if let Some(n) = visual.iter_mut().find(|n| n.id == *note_id) {
                        n.x = new_x;
                        n.y = new_y;
                    }
                    live.borrow_mut().notes = Some(visual);
                    schedule_paint();
                } else if let Some((area_id, start_x, start_y)) = &drag.area_drag {
                    // redesign-listview-type-length-canvas-fix：区域拖动中只写视觉层（ST-CR-AREA-01）
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let mut visual = store.areas.get_untracked();
                    if let Some(a) = visual.iter_mut().find(|a| a.id == *area_id) {
                        a.x = new_x;
                        a.y = new_y;
                    }
                    live.borrow_mut().areas = Some(visual);
                    schedule_paint();
                } else if let Some(table_id) = &drag.table_id {
                    // R-PERF-04：拖动中只写 (id, x, y) 覆盖层——禁止整 Vec<Table> 深克隆
                    // （UT-CR-DRAG-01）；不量化网格（UT-CR-06），松手才落账 store
                    let new_x = drag.start_table_x + dx / t_now.zoom;
                    let new_y = drag.start_table_y + dy / t_now.zoom;
                    live.borrow_mut().table_pos = Some((table_id.clone(), new_x, new_y));
                    // R-PERF-11：无关系线的表走幽灵层——pointermove 只改 CSS transform，
                    // 主画布零重绘；有关系线回退逐帧重绘（线条需跟随）
                    let mut ghost_g = ghost.borrow_mut();
                    if ghost_g.is_none() {
                        let has_refs = store
                            .references
                            .with(|refs| super::table_has_references(refs, table_id));
                        if !has_refs {
                            let created = store.tables.with(|tables| {
                                tables.iter().find(|tb| &tb.id == table_id).and_then(|tb| {
                                    super::create_table_ghost(
                                        &canvas,
                                        tb,
                                        &super::current_palette(),
                                        &t_now,
                                        new_x,
                                        new_y,
                                    )
                                })
                            });
                            if let Some(g) = created {
                                *ghost_g = Some(g);
                                // 幽灵层创建当帧重绘一次：把被拖表从主画布「抠出」
                                schedule_paint();
                                return;
                            }
                        }
                    }
                    if let Some(g) = ghost_g.as_ref() {
                        if g.transform_matches(&t_now) {
                            g.update_position(new_x, new_y);
                            return; // 零重绘路径：不 schedule_paint
                        }
                        // 拖拽中 zoom/pan 变化（如 wheel）：移除幽灵层回退逐帧重绘
                        *ghost_g = None;
                    }
                    drop(ghost_g);
                    schedule_paint();
                } else {
                    // R-PERF-05：pan 与 wheel 共用 pending_transform 通道，rAF 合并落账——
                    // 一帧至多一次 transform 更新 / 一次 draw_canvas
                    *pending_transform.borrow_mut() = Some(Transform {
                        pan_x: drag.start_pan_x + dx,
                        pan_y: drag.start_pan_y + dy,
                        zoom: t_now.zoom,
                    });
                    schedule_paint();
                }
            }
        };

        let on_pointerup = {
            let live = live.clone();
            let ghost = ghost.clone();
            let schedule_paint = schedule_paint.clone();
            let on_field_pick = on_field_pick.clone();
            let on_relation_drop = on_relation_drop.clone();
            let on_relation_drag_cancel = on_relation_drag_cancel.clone();
            let on_table_drop = on_table_drop.clone();
            let on_area_create = on_area_create.clone();
            let on_note_create = on_note_create.clone();
            let current_transform = current_transform.clone();
            move |ev: PointerEvent| {
                let Some(drag) = drag_state.get_untracked() else {
                    return;
                };
                // R-PERF-05：松手落账前先把 rAF 待落账的 wheel/pan 意图落账，
                // 吸附坐标基于已提交的 transform
                let t_now = current_transform();
                let canvas = canvas_ref.get();
                if let Some(c) = &canvas {
                    let _ = c.release_pointer_capture(drag.pointer_id);
                }

                if let Some(cd) = &drag.create_drag {
                    // p0-fix 定点 2：松开落账——区域按拖框矩形（<10px 不创建），便签在按下点放置
                    live.borrow_mut().area_preview = None;
                    let cd = cd.clone();
                    drag_state.set(None);
                    schedule_paint();
                    match cd.tool {
                        CreateToolKind::Area => {
                            if let Some((x, y, w, h)) =
                                super::area_rect_from_drag(cd.start_x, cd.start_y, cd.cur_x, cd.cur_y)
                            {
                                if let Some(cb) = on_area_create.as_ref() {
                                    cb(x, y, w, h);
                                }
                            }
                        }
                        CreateToolKind::Note => {
                            if let Some(cb) = on_note_create.as_ref() {
                                cb(cd.start_x, cd.start_y);
                            }
                        }
                    }
                    return;
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
                                &t_now,
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

                if let Some((note_id, start_x, start_y)) = drag.note_drag.clone() {
                    // redesign-listview-type-length-canvas-fix：便签松手落账（ST-CR-NOTE-01）
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().notes = None;
                    drag_state.set(None);
                    // 纯点击（位移 < 阈值）= 选中语义：不写回坐标、不触发持久化
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let mut notes = store.notes.get_untracked();
                    if let Some(n) = notes.iter_mut().find(|n| n.id == note_id) {
                        n.x = new_x;
                        n.y = new_y;
                    }
                    store.notes.set(notes);
                    // 复用表拖动落账通路（dirty + 协作 op + S01 保存链路 → PUT notes[0].x/y）
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                if let Some((area_id, start_x, start_y)) = drag.area_drag.clone() {
                    // redesign-listview-type-length-canvas-fix：区域松手落账（ST-CR-AREA-01，宽高不变）
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().areas = None;
                    drag_state.set(None);
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let mut areas = store.areas.get_untracked();
                    if let Some(a) = areas.iter_mut().find(|a| a.id == area_id) {
                        a.x = new_x;
                        a.y = new_y;
                    }
                    store.areas.set(areas);
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                if let Some(table_id) = drag.table_id {
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().table_pos = None;
                    // R-PERF-11：先撤幽灵层再落账/重绘，保证渲染 effect 看到幽灵已清
                    // （Drop 自动从 DOM 移除节点）
                    ghost.borrow_mut().take();
                    drag_state.set(None);
                    // 纯点击（位移 < 4px）= 选中语义：不写回坐标、不触发持久化，仅重绘复位视觉
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    let new_x = drag.start_table_x + dx / t_now.zoom;
                    let new_y = drag.start_table_y + dy / t_now.zoom;
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

                live.borrow_mut().table_pos = None;
                drag_state.set(None);
                // R-PERF-10：pan 等无落账信号的拖拽，松手时主动补一帧——把有效 dpr
                // 恢复全分辨率并重绘（否则 backing store 停留在降采样态，画面发虚）
                schedule_paint();
            }
        };

        // pointercancel：浏览器/系统打断 pointer capture 时触发（Alt+Tab 切窗口、
        // 拖出 viewport、OS 强制收走输入等）。区别于 pointerup：没有 ev.client_x/y 可信，
        // 所以只做防御性清理，不写 store。
        let on_pointercancel = {
            let live = live.clone();
            let ghost = ghost.clone();
            let schedule_paint = schedule_paint.clone();
            move |_ev: PointerEvent| {
                if drag_state.get_untracked().is_none() {
                    return;
                }
                live.borrow_mut().rubber = None;
                live.borrow_mut().table_pos = None;
                live.borrow_mut().area_preview = None;
                live.borrow_mut().notes = None;
                live.borrow_mut().areas = None;
                ghost.borrow_mut().take(); // R-PERF-11
                rubber_d.set(None);
                drag_state.set(None);
                schedule_paint();
            }
        };

        // §3.3 / R-PERF-05：滚轮缩放走锚定纯函数（clamp 后仍保持光标锚定）；
        // transform 更新经 pending_transform 意图通道 rAF 合并——同一帧多次 wheel 只落账一次。
        let on_wheel = {
            let schedule_paint = schedule_paint.clone();
            let current_transform = current_transform.clone();
            let pending_transform = pending_transform.clone();
            move |ev: WheelEvent| {
                ev.prevent_default();
                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                let mouse_x = ev.client_x() as f64;
                let mouse_y = ev.client_y() as f64;
                // §3.3 rect 基准：anchor 必须减去 get_bounding_client_rect() 的 left/top
                // （canvas 在视口内的偏移），否则缩放会累积偏移导致画布"漂"出视口。
                let rect = canvas.get_bounding_client_rect();
                let cur = current_transform();
                let anchor = (mouse_x - rect.left(), mouse_y - rect.top());
                let zoom_factor = if ev.delta_y() < 0.0 { 1.1 } else { 1.0 / 1.1 };
                let next = super::zoom_transform_at_anchor(&cur, anchor, zoom_factor);
                *pending_transform.borrow_mut() = Some(next);
                schedule_paint();
            }
        };

        let on_dblclick = {
            let on_dblclick_blank = on_dblclick_blank.clone();
            let current_transform = current_transform.clone();
            move |ev: MouseEvent| {
                let canvas = match canvas_ref.get() {
                    Some(c) => c,
                    None => return,
                };
                let t_now = current_transform();
                let (dx, dy) = screen_to_diagram(
                    ev.client_x() as f64,
                    ev.client_y() as f64,
                    &canvas,
                    &t_now,
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

// ─── R-PERF-01：视口 AABB 裁剪纯函数（UT-CR-CULL-01 单测覆盖）────────────────

/// AABB 相交判定（含边界贴合）：boxes 形如 `(x, y, w, h)`，世界坐标。
pub fn aabb_intersects(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 <= b.0 + b.2 && b.0 <= a.0 + a.2 && a.1 <= b.1 + b.3 && b.1 <= a.1 + a.3
}

/// 由 transform + canvas CSS 尺寸推导视口的世界坐标 AABB `(x, y, w, h)`。
pub fn viewport_world_aabb(t: &Transform, css_w: f64, css_h: f64) -> (f64, f64, f64, f64) {
    let x0 = -t.pan_x / t.zoom;
    let y0 = -t.pan_y / t.zoom;
    let x1 = (css_w - t.pan_x) / t.zoom;
    let y1 = (css_h - t.pan_y) / t.zoom;
    (x0, y0, x1 - x0, y1 - y0)
}

/// 表渲染 AABB（width/min_height 消费口径与 draw_table 一致）。
pub fn table_aabb(table: &Table) -> (f64, f64, f64, f64) {
    let (w, h) = compute_table_render_size(table);
    (table.x, table.y, w, h)
}

/// 区域 AABB。
pub fn area_aabb(area: &Area) -> (f64, f64, f64, f64) {
    (area.x, area.y, area.width, area.height)
}

/// 便签 AABB（固定 180×100 命中面，与 draw_note / hit_test_note 一致）。
pub fn note_aabb(note: &Note) -> (f64, f64, f64, f64) {
    (note.x, note.y, NOTE_WIDTH, NOTE_HEIGHT)
}

/// R-PERF-01：仅收集与视口 AABB 相交的表（纯函数）。
pub fn collect_visible_tables(tables: &[Table], vp: (f64, f64, f64, f64)) -> Vec<&Table> {
    tables
        .iter()
        .filter(|t| aabb_intersects(table_aabb(t), vp))
        .collect()
}

/// R-PERF-01：仅收集与视口 AABB 相交的区域（纯函数）。
pub fn collect_visible_areas(areas: &[Area], vp: (f64, f64, f64, f64)) -> Vec<&Area> {
    areas
        .iter()
        .filter(|a| aabb_intersects(area_aabb(a), vp))
        .collect()
}

/// R-PERF-01：仅收集与视口 AABB 相交的便签（纯函数）。
pub fn collect_visible_notes(notes: &[Note], vp: (f64, f64, f64, f64)) -> Vec<&Note> {
    notes
        .iter()
        .filter(|n| aabb_intersects(note_aabb(n), vp))
        .collect()
}

/// R-PERF-01/06：关系可见 = 任一表端点可见；端点查找走 `id → &Table` HashMap O(1)。
pub fn collect_visible_refs<'a>(
    refs: &'a [Reference],
    tables: &'a [Table],
    vp: (f64, f64, f64, f64),
) -> Vec<&'a Reference> {
    let table_map: HashMap<&str, &Table> = tables.iter().map(|t| (t.id.as_str(), t)).collect();
    refs.iter()
        .filter(|r| {
            match (
                table_map.get(r.start_table_id.as_str()),
                table_map.get(r.end_table_id.as_str()),
            ) {
                (Some(f), Some(to)) => {
                    aabb_intersects(table_aabb(f), vp) || aabb_intersects(table_aabb(to), vp)
                }
                _ => false,
            }
        })
        .collect()
}

/// R-PERF-04：被拖表的绘制坐标——覆盖层命中时返回覆盖坐标（绘制输入含 `(id, new_x, new_y)`）。
pub fn table_draw_position(table: &Table, table_override: Option<(&str, f64, f64)>) -> (f64, f64) {
    match table_override {
        Some((oid, x, y)) if oid == table.id => (x, y),
        _ => (table.x, table.y),
    }
}

/// R-PERF-04：返回以覆盖坐标绘制的表副本（仅被拖表一帧一克隆；未命中返回原表引用语义）。
pub fn table_with_override(table: &Table, table_override: Option<(&str, f64, f64)>) -> Table {
    let (x, y) = table_draw_position(table, table_override);
    let mut visual = table.clone();
    visual.x = x;
    visual.y = y;
    visual
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
///
/// fix-canvas-zoom-perf / R-PERF-04：pointermove 不再调用本函数（整 Vec 深克隆）——
/// 拖动覆盖层（LivePaint.table_pos）只存被拖对象 `(id, x, y)`，绘制时经
/// `table_with_override` 合并；松手落账路径原地改 store（iter_mut），无需本函数。
/// 保留为纯函数供 UT-CR-06 / UT-CR-DRAG-01 验证落账语义。
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
///
/// R-PERF-01：仅绘制与视口 AABB 相交的表/区域/便签/关系（`viewport_world_aabb` 由
/// transform + canvas CSS 尺寸推导）。R-PERF-06：refs 端点查找先建 `HashMap<&str, &Table>`。
/// R-PERF-04：`table_override` 为被拖表的 `(id, x, y)` 覆盖坐标（LivePaint 拖动层）。
/// width/height 为 canvas **CSS** 尺寸（backing store 像素在函数内按 dpr 换算）。
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
    // p0-fix 定点 3：选中的关系连线 id（点击连线高亮）
    selected_ref_id: Option<&str>,
    // p0-fix 定点 2：选中的区域 / 便签 id（点击高亮 + Inspector 编辑）
    selected_area_id: Option<&str>,
    selected_note_id: Option<&str>,
    rubber_band: Option<(f64, f64, f64, f64)>,
    // p0-fix 定点 2：区域拖框预览矩形 (x, y, w, h)
    area_preview: Option<(f64, f64, f64, f64)>,
    // R-PERF-04：被拖表的 (id, x, y) 覆盖坐标（绘制时替换坐标，松手才落账 store）
    table_override: Option<(&str, f64, f64)>,
    // R-PERF-11：幽灵层接管中的表 id——主画布跳过绘制（等效「抠出」，由 DOM 幽灵层呈现）
    ghost_skip: Option<&str>,
) {
    bump_paint_counter();

    let dpr = effective_device_pixel_ratio();
    // backing store 像素 = CSS × dpr，clear_rect 必须按 backing store 清空
    ctx.clear_rect(0.0, 0.0, width * dpr, height * dpr);
    // 画布底色由 .cdb-canvas-container 壳层 CSS 提供（主原型 .canvas 透明 + 壳层 bg-deep 60%），
    // 栅格层不再自填背景，仅绘制点阵与对象。
    let palette = current_palette();

    draw_grid(ctx, t, width, height, palette);

    // R-PERF-01：视口世界坐标 AABB（width/height 已是 CSS 尺寸）
    let vp = viewport_world_aabb(t, width, height);

    ctx.save();
    // R-DPR-02：每帧 set_transform(dpr*zoom, ...) 复位，避免 zoom 累乘（UT-RP-03）
    let _ = ctx.set_transform(dpr * t.zoom, 0.0, 0.0, dpr * t.zoom, t.pan_x * dpr, t.pan_y * dpr);
    // R-DPR-05：开启图像平滑，让反走样后的栅格过渡更柔和；线条级 1px 通过 set_line_width 对齐即可
    let _ = ctx.set_image_smoothing_enabled(true);

    for area in collect_visible_areas(areas, vp) {
        draw_area(ctx, area, palette, selected_area_id == Some(area.id.as_str()));
    }

    if let Some((x, y, w, h)) = area_preview {
        // p0-fix 定点 2：区域拖框预览——虚线框 + 淡色填充，不写入 store
        let _ = ctx.set_fill_style_str(palette.area_bg);
        ctx.fill_rect(x, y, w, h);
        let _ = ctx.set_stroke_style_str(palette.area_border);
        ctx.set_line_width(1.0);
        let dash = {
            let a = js_sys::Array::new();
            a.push(&wasm_bindgen::JsValue::from(6.0));
            a.push(&wasm_bindgen::JsValue::from(4.0));
            a
        };
        let _ = ctx.set_line_dash(&dash);
        ctx.stroke_rect(x, y, w, h);
        let _ = ctx.set_line_dash(&js_sys::Array::new());
    }

    // R-PERF-06：refs 端点查找先建 id → &Table HashMap，O(refs + tables)
    let table_map: HashMap<&str, &Table> = tables.iter().map(|t| (t.id.as_str(), t)).collect();
    for r in refs {
        let visible = match (
            table_map.get(r.start_table_id.as_str()),
            table_map.get(r.end_table_id.as_str()),
        ) {
            // R-PERF-01：仅当任一表端点与视口相交才绘制该关系
            (Some(f), Some(tbl)) => {
                aabb_intersects(table_aabb(f), vp) || aabb_intersects(table_aabb(tbl), vp)
            }
            _ => false,
        };
        if visible {
            let (from, to) = (table_map[r.start_table_id.as_str()], table_map[r.end_table_id.as_str()]);
            draw_bezier_fields(ctx, from, &r.start_field_id, to, &r.end_field_id, palette, selected_ref_id == Some(&r.id));
        }
    }

    // R-PERF-07：清理已删除表的精灵缓存
    evict_stale_sprites(tables);
    for table in collect_visible_tables(tables, vp) {
        // R-PERF-11：幽灵层接管中的表不在主画布绘制（拖拽期间主画布保持静态零损伤）
        if ghost_skip == Some(table.id.as_str()) {
            continue;
        }
        let is_sel = selected_id == Some(table.id.as_str());
        // R-PERF-04：被拖表以覆盖坐标绘制（一帧至多一张表的一次克隆）
        let visual = table_with_override(table, table_override);
        draw_table(ctx, &visual, is_sel, palette, t.zoom);
    }

    for note in collect_visible_notes(notes, vp) {
        draw_note(ctx, note, palette, selected_note_id == Some(note.id.as_str()));
    }

    for presence in remote_presence {
        draw_remote_presence(ctx, presence, palette);
    }

    if let Some((x1, y1, x2, y2)) = rubber_band {
        draw_rubber_band(ctx, x1, y1, x2, y2, palette);
    }

    ctx.restore();
}

/// ST-CR-PAN-01 / ST-CR-INSP-01 debug 计数：`window.__cdb_paint_count` 每次 draw_canvas +1。
#[cfg(target_arch = "wasm32")]
fn bump_paint_counter() {
    if let Some(win) = web_sys::window() {
        let target: &js_sys::Object = win.unchecked_ref();
        let key = wasm_bindgen::JsValue::from_str("__cdb_paint_count");
        let cur = js_sys::Reflect::get(target, &key)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let _ = js_sys::Reflect::set(target, &key, &wasm_bindgen::JsValue::from(cur + 1.0));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn bump_paint_counter() {}

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

/// R-PERF-02：网格点阵合批绘制抽象——`begin` 一次、全部点 `arc` 进同一 path、`fill` 一次，
/// 绘制调用数为 O(1) 且与点数无关（UT-CR-BATCH-01 以 mock sink 计数断言）。
/// 实现注意：同一 path 内 `dot` 必须先 `move_to` 再 `arc`，否则 Canvas2D 会把相邻
/// 弧的起点两两连线（点阵退化成横线网，fix-canvas-grid-splitter-listview-comment 回归源）。
trait GridSink {
    fn begin(&mut self);
    fn dot(&mut self, x: f64, y: f64);
    fn fill(&mut self);
}

/// R-PERF-02：点阵几何（世界坐标，纯计算可在 host 单测）——只枚举视口 AABB 内的网格点。
fn paint_grid(t: &Transform, css_w: f64, css_h: f64, sink: &mut impl GridSink) {
    sink.begin();
    let vp = viewport_world_aabb(t, css_w, css_h);
    let start_x = (vp.0 / GRID_SIZE).floor() * GRID_SIZE;
    let start_y = (vp.1 / GRID_SIZE).floor() * GRID_SIZE;
    let end_x = vp.0 + vp.2;
    let end_y = vp.1 + vp.3;
    let mut y = start_y;
    while y < end_y {
        let mut x = start_x;
        while x < end_x {
            sink.dot(x, y);
            x += GRID_SIZE;
        }
        y += GRID_SIZE;
    }
    sink.fill();
}

fn draw_grid(ctx: &CanvasRenderingContext2d, t: &Transform, width: f64, height: f64, palette: &CanvasPalette) {
    // 主原型点阵：radial-gradient(text-3 @ 30%) 1px / 24px —— 色值已带透明度，不再叠加 global_alpha。
    // R-PERF-02：所有点合并进单 path 一次 fill；在显式 world CTM 下绘制（与对象层同一坐标系，
    // 不再依赖上一帧残留的 CTM）。width/height 为 CSS 尺寸。
    // R-PERF-10：CTM 用有效 dpr，与 backing store 同源
    let dpr = effective_device_pixel_ratio();
    ctx.save();
    let _ = ctx.set_transform(dpr * t.zoom, 0.0, 0.0, dpr * t.zoom, t.pan_x * dpr, t.pan_y * dpr);
    let _ = ctx.set_fill_style_str(palette.grid_dot);

    struct CtxGridSink<'a>(&'a CanvasRenderingContext2d);
    impl GridSink for CtxGridSink<'_> {
        fn begin(&mut self) {
            self.0.begin_path();
        }
        fn dot(&mut self, x: f64, y: f64) {
            // fix-canvas-grid-splitter-listview-comment：arc 前必须 move_to——
            // Canvas2D 规范下 arc 会把「当前路径点 → 弧线起点」连成直线，
            // 同一 path 内连续 arc 会把整行网格点连成横线、行尾到下行行首连成对角线。
            self.0.move_to(x + 1.0, y);
            let _ = self.0.arc(x, y, 1.0, 0.0, std::f64::consts::TAU);
        }
        fn fill(&mut self) {
            self.0.fill();
        }
    }
    let mut sink = CtxGridSink(ctx);
    paint_grid(t, width, height, &mut sink);
    ctx.restore();
}

/// §3.3：工具栏缩放以视口中心为锚点——anchor = canvas CSS 尺寸 / 2。
/// canvas 不在 DOM（host 单测 / 编辑器未挂载）时退化为 (0, 0)。
fn viewport_center_anchor() -> (f64, f64) {
    let size = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("editor-canvas"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .map(|c| (c.client_width() as f64, c.client_height() as f64));
    size.map(|(w, h)| (w / 2.0, h / 2.0)).unwrap_or((0.0, 0.0))
}

/// 画布缩放：放大（步进 1.25x，clamp ZOOM_MAX）。§3.3：以视口中心为锚点，中心下世界点不漂移。
pub fn zoom_in(transform: RwSignal<Transform>) {
    transform.update(|t| {
        *t = zoom_transform_at_anchor(t, viewport_center_anchor(), 1.25);
    });
    request_canvas_repaint();
}

/// 画布缩放：缩小（步进 0.8x，clamp ZOOM_MIN）。§3.3：以视口中心为锚点。
pub fn zoom_out(transform: RwSignal<Transform>) {
    transform.update(|t| {
        *t = zoom_transform_at_anchor(t, viewport_center_anchor(), 1.0 / 1.25);
    });
    request_canvas_repaint();
}

/// 画布缩放：重置为 1x，平移归零（绝对语义，无需锚点）
pub fn zoom_reset(transform: RwSignal<Transform>) {
    transform.set(Transform::default());
    request_canvas_repaint();
}

// ─── R-PERF-07：表卡片精灵缓存 ─────────────────────────────────────────────
//
// 卡体（投影/渐变/文字）按内容指纹缓存到离屏 canvas，拖动/平移期间每帧仅
// drawImage 位块传输；选中环不含在内、每帧活画（选中切换不失效缓存）。
// 指纹变更（编辑落账 / 主题 / dpr / zoom 分档）才重光栅。
// zoom 分档 k ∈ {1,2}：sprite 比例 s = dpr × k，zoom ≤ 1 取 1、1<zoom≤2 取 2；
// zoom > SPRITE_CACHE_MAX_ZOOM 回退活画（高倍下视口内表少，且避免文字软化）。
// 阴影 blur/offset 为设备像素口径（不随 CTM 缩放），精灵内按 bucket/zoom 补偿，
// 保证任意 zoom 下投影视觉强度与活画一致。

/// 超过该 zoom 回退活画（不走精灵缓存）。
pub const SPRITE_CACHE_MAX_ZOOM: f64 = 2.0;
/// 精灵四周边距：容纳阴影出血（blur 16 + offset 6）与选中环外扩。
const SPRITE_MARGIN: f64 = 24.0;

/// R-PERF-07：zoom → 精灵分辨率分档（UT-CR-SPRITE-01）。
pub fn sprite_zoom_bucket(zoom: f64) -> u32 {
    if zoom <= 1.0 { 1 } else { 2 }
}

/// R-PERF-07：卡体内容指纹（UT-CR-SPRITE-01）。位置（x/y）与选中态不参与——
/// 移动不换缓存；编辑落账产生新内容才失效。FNV-1a。
pub fn table_sprite_fingerprint(table: &Table, theme_dark: bool, dpr_x100: u32, zoom_bucket: u32) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(table.name.as_bytes());
    mix(table.color.as_bytes());
    mix(&table.width.unwrap_or(0).to_le_bytes());
    mix(&table.min_height.unwrap_or(0).to_le_bytes());
    for f in &table.fields {
        mix(f.name.as_bytes());
        mix(f.type_.as_bytes());
        mix(&[f.primary as u8]);
    }
    mix(&[theme_dark as u8]);
    mix(&dpr_x100.to_le_bytes());
    mix(&(zoom_bucket as u32).to_le_bytes());
    h
}

/// R-PERF-08：DOM 写守护——仅值变化才写（返回 true 表示已更新 last，调用方应执行 DOM 写）。
pub fn dom_write_guard(last: &mut String, next: &str) -> bool {
    if last == next {
        false
    } else {
        last.clear();
        last.push_str(next);
        true
    }
}

struct TableSprite {
    canvas: web_sys::HtmlCanvasElement,
    fingerprint: u64,
    margin: f64,
    w_world: f64,
    h_world: f64,
}

thread_local! {
    static TABLE_SPRITES: std::cell::RefCell<HashMap<String, TableSprite>> =
        std::cell::RefCell::new(HashMap::new());
}

/// 渲染表卡体到离屏 canvas。shadow_boost = bucket / zoom——阴影为设备像素口径，
/// 位块传输随 CTM 缩放，需预补偿使落屏模糊半径恒为 16 设备像素。
fn render_table_sprite(
    table: &Table,
    palette: &CanvasPalette,
    scale: f64,
    shadow_boost: f64,
    fingerprint: u64,
) -> Option<TableSprite> {
    let doc = web_sys::window()?.document()?;
    let canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .ok()?
        .dyn_into()
        .ok()?;
    let (w, h) = compute_table_render_size(table);
    let w_world = w + SPRITE_MARGIN * 2.0;
    let h_world = h + SPRITE_MARGIN * 2.0;
    canvas.set_width((w_world * scale).ceil().max(1.0) as u32);
    canvas.set_height((h_world * scale).ceil().max(1.0) as u32);
    let off: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    let _ = off.set_transform(
        scale,
        0.0,
        0.0,
        scale,
        (SPRITE_MARGIN - table.x) * scale,
        (SPRITE_MARGIN - table.y) * scale,
    );
    draw_table_body(&off, table, palette, shadow_boost);
    Some(TableSprite {
        canvas,
        fingerprint,
        margin: SPRITE_MARGIN,
        w_world,
        h_world,
    })
}

/// 命中缓存则位块传输并返回 true；未命中/渲染失败返回 false（调用方回退活画）。
fn blit_table_sprite(
    ctx: &CanvasRenderingContext2d,
    table: &Table,
    palette: &CanvasPalette,
    zoom: f64,
) -> bool {
    // R-PERF-10 修正：精灵恒以真实 dpr 渲染/取指纹——backing 分辨率与主画布有效 dpr
    // 解耦（位块传输按世界坐标 dw/dh 绘制，主画布降采样时由 CTM 自然缩小，观感不劣化）。
    // 这样拖拽起止的有效 dpr 切换不会触发全量精灵重渲染，消除边界帧的突刺成本。
    let dpr = current_device_pixel_ratio();
    let bucket = sprite_zoom_bucket(zoom);
    let scale = dpr * bucket as f64;
    let fp = table_sprite_fingerprint(
        table,
        current_theme_dark(),
        (dpr * 100.0).round() as u32,
        bucket,
    );
    let sprite = TABLE_SPRITES.with(|c| {
        let mut map = c.borrow_mut();
        let fresh = map
            .get(&table.id)
            .map(|s| s.fingerprint == fp)
            .unwrap_or(false);
        if !fresh {
            // shadow_boost = bucket / zoom（见 render_table_sprite 注释）
            let boost = bucket as f64 / zoom.max(0.01);
            if let Some(s) = render_table_sprite(table, palette, scale, boost, fp) {
                map.insert(table.id.clone(), s);
            }
        }
        map.get(&table.id).map(|s| {
            (s.canvas.clone(), s.margin, s.w_world, s.h_world)
        })
    });
    match sprite {
        Some((canvas, margin, w_world, h_world)) => {
            ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                &canvas,
                table.x - margin,
                table.y - margin,
                w_world,
                h_world,
            )
            .is_ok()
        }
        None => false,
    }
}

/// 清理已删除表的精灵（每帧 O(精灵数 × 表数)，常规规模可忽略）。
fn evict_stale_sprites(tables: &[Table]) {
    TABLE_SPRITES.with(|c| {
        c.borrow_mut()
            .retain(|id, _| tables.iter().any(|t| &t.id == id));
    });
}

// ============================================================
// R-PERF-11：表拖拽幽灵层（drag ghost overlay）
// ============================================================
// 实测根因（trace 佐证）：本环境为 SwiftShader 软件合成，只要主画布内容变化，
// 合成器每帧都按「损伤区域 = 整个画布元素」重光栅输出（dpr=2 时 ~15ms/帧），
// 与画布 backing 分辨率无关——R-PERF-10 只省画布自身光栅，损伤区域不缩水。
// 要消除拖拽掉帧，必须让主画布在拖拽期间完全不重绘：被拖表改由绝对定位的
// 小 canvas（幽灵层）经 CSS transform 移动，每帧损伤区域缩到表卡大小，
// 合成成本可忽略。约束：有关系线连接的表仍需逐帧重绘主画布（线条跟随），
// 回退 R-PERF-10 降采样路径。

/// 被拖表是否有关联关系线（有则拖拽需逐帧重绘主画布，不能走幽灵层）。纯函数（UT-CR-GHOST-01）。
pub fn table_has_references(refs: &[Reference], table_id: &str) -> bool {
    refs.iter()
        .any(|r| r.start_table_id == table_id || r.end_table_id == table_id)
}

/// 幽灵层 CSS 定位（相对画布容器左上，px）：sprite 世界矩形含 margin 边距。纯函数（UT-CR-GHOST-01）。
pub fn ghost_css_position(
    table_x: f64,
    table_y: f64,
    margin: f64,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
) -> (f64, f64) {
    (
        (table_x - margin) * zoom + pan_x,
        (table_y - margin) * zoom + pan_y,
    )
}

/// R-PERF-11：幽灵层进行态。Drop 时自动从 DOM 移除，保证任何退出路径
/// （pointerup / pointercancel / Escape / 组件卸载）都不留残留节点。
struct GhostDrag {
    el: web_sys::HtmlCanvasElement,
    table_id: String,
    margin: f64,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
}

impl GhostDrag {
    /// pointermove 只调本函数：CSS transform 更新走合成器，主画布零重绘。
    fn update_position(&self, table_x: f64, table_y: f64) {
        let (left, top) =
            ghost_css_position(table_x, table_y, self.margin, self.pan_x, self.pan_y, self.zoom);
        let _ = self
            .el
            .style()
            .set_property("transform", &format!("translate({left}px, {top}px)"));
    }

    /// 拖拽中 wheel/pan 改变 transform 时幽灵层失效（回退逐帧重绘路径）。
    fn transform_matches(&self, t: &Transform) -> bool {
        (self.zoom - t.zoom).abs() < 1e-9
            && (self.pan_x - t.pan_x).abs() < 1e-9
            && (self.pan_y - t.pan_y).abs() < 1e-9
    }
}

impl Drop for GhostDrag {
    fn drop(&mut self) {
        self.el.remove();
    }
}

/// 创建被拖表的幽灵层：以真实 dpr 重渲染一张卡体 sprite（小位图，一次性成本），
/// 作为绝对定位 DOM 节点挂到画布容器。仅拖拽首次移动时调用一次。
fn create_table_ghost(
    canvas: &web_sys::HtmlCanvasElement,
    table: &Table,
    palette: &CanvasPalette,
    t: &Transform,
    table_x: f64,
    table_y: f64,
) -> Option<GhostDrag> {
    let dpr = current_device_pixel_ratio();
    let bucket = sprite_zoom_bucket(t.zoom);
    let scale = dpr * bucket as f64;
    let boost = bucket as f64 / t.zoom.max(0.01);
    let sprite = render_table_sprite(table, palette, scale, boost, 0)?;
    let el = sprite.canvas;
    let parent = canvas.parent_element()?;
    let css_w = sprite.w_world * t.zoom;
    let css_h = sprite.h_world * t.zoom;
    // outline 负偏移模拟选中环（对齐 draw_table_selection 的 x-2.5 外扩口径）；
    // will-change 提示合成器把幽灵层提升为独立层，transform 移动纯合成器完成
    let ring_inset = (sprite.margin - 2.5) * t.zoom;
    let style = format!(
        "position:absolute;left:0;top:0;width:{css_w}px;height:{css_h}px;\
         pointer-events:none;z-index:2;will-change:transform;\
         outline:2px solid {sel};outline-offset:-{ring_inset}px;border-radius:16px;",
        sel = palette.selected
    );
    el.set_attribute("style", &style).ok()?;
    el.set_attribute("data-testid", "drag-ghost").ok()?;
    parent.append_child(&el).ok()?;
    let ghost = GhostDrag {
        el,
        table_id: table.id.clone(),
        margin: sprite.margin,
        pan_x: t.pan_x,
        pan_y: t.pan_y,
        zoom: t.zoom,
    };
    ghost.update_position(table_x, table_y);
    Some(ghost)
}

fn draw_table(ctx: &CanvasRenderingContext2d, table: &Table, selected: bool, palette: &CanvasPalette, zoom: f64) {
    // R-PERF-07：zoom ≤ SPRITE_CACHE_MAX_ZOOM 走精灵缓存；超出回退活画
    if zoom <= SPRITE_CACHE_MAX_ZOOM && blit_table_sprite(ctx, table, palette, zoom) {
        if selected {
            draw_table_selection(ctx, table, palette);
        }
        return;
    }
    draw_table_body(ctx, table, palette, 1.0);
    if selected {
        draw_table_selection(ctx, table, palette);
    }
}

fn draw_table_body(ctx: &CanvasRenderingContext2d, table: &Table, palette: &CanvasPalette, shadow_boost: f64) {
    let field_count = table.fields.len().max(2);
    let (width, total_height) = compute_table_render_size(table);
    let x = table.x;
    let y = table.y;

    // 表体：主原型 .db-table —— radius 14、surface-solid 底、line-strong 描边、柔和投影
    // 阴影为设备像素口径（不随 CTM 缩放），shadow_boost 用于精灵离屏渲染的预补偿
    ctx.save();
    let _ = ctx.set_shadow_color("rgba(0, 0, 0, 0.18)");
    let _ = ctx.set_shadow_blur(16.0 * shadow_boost);
    let _ = ctx.set_shadow_offset_x(0.0);
    let _ = ctx.set_shadow_offset_y(6.0 * shadow_boost);

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
}

/// 选中态：主原型 .is-selected —— brand 描边 + 3px brand-soft 外环。
/// R-PERF-07：选中环每帧活画（不含在精灵缓存内），选中切换不失效卡体缓存。
fn draw_table_selection(ctx: &CanvasRenderingContext2d, table: &Table, palette: &CanvasPalette) {
    let (_width, total_height) = compute_table_render_size(table);
    let x = table.x;
    let y = table.y;
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
    // p0-fix 定点 3：选中态（点击连线高亮——stroke 加粗 + brand 色）
    selected: bool,
) {
    let path = calc_path(from, from_field_id, to, to_field_id);

    // 主原型 .relation-path-bg：7px surface 光晕垫底，主线 2px brand
    let _ = ctx.set_stroke_style_str(if selected { palette.selected_soft } else { palette.relation_halo });
    ctx.set_line_width(if selected { 10.0 } else { 7.0 });
    ctx.begin_path();
    ctx.move_to(path.x1, path.y1);
    ctx.bezier_curve_to(path.cx1, path.cy1, path.cx2, path.cy2, path.x2, path.y2);
    ctx.stroke();

    let _ = ctx.set_stroke_style_str(if selected { palette.selected } else { palette.relation });
    ctx.set_line_width(if selected { 3.5 } else { 2.0 });
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

fn draw_area(ctx: &CanvasRenderingContext2d, area: &Area, palette: &CanvasPalette, selected: bool) {
    let _ = ctx.set_fill_style_str(palette.area_bg);
    ctx.fill_rect(area.x, area.y, area.width, area.height);

    // p0-fix 定点 2：选中态——描边加粗 + brand 色实线
    let _ = ctx.set_stroke_style_str(if selected { palette.selected } else { palette.area_border });
    ctx.set_line_width(if selected { 2.5 } else { 1.0 });
    let dash_arr = {
        let a = js_sys::Array::new();
        a.push(&wasm_bindgen::JsValue::from(6.0));
        a.push(&wasm_bindgen::JsValue::from(4.0));
        a
    };
    let empty_dash = js_sys::Array::new();
    let _ = ctx.set_line_dash(if selected { &empty_dash } else { &dash_arr });
    ctx.stroke_rect(area.x, area.y, area.width, area.height);
    let _ = ctx.set_line_dash(&js_sys::Array::new());

    let _ = ctx.set_fill_style_str(palette.area_border);
    let _ = ctx.set_font(&dpr_font(750, 11.0, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("top");
    let _ = ctx.fill_text(&area.name, area.x + 10.0, area.y + 10.0);
}

fn draw_note(ctx: &CanvasRenderingContext2d, note: &Note, palette: &CanvasPalette, selected: bool) {
    let note_w = NOTE_WIDTH;
    let note_h = NOTE_HEIGHT;
    let _ = ctx.set_fill_style_str(palette.note_bg);
    ctx.fill_rect(note.x, note.y, note_w, note_h);

    // p0-fix 定点 2：选中态——描边加粗 + brand 色
    let _ = ctx.set_stroke_style_str(if selected { palette.selected } else { palette.note_border });
    ctx.set_line_width(if selected { 2.5 } else { 1.0 });
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

// ─── Area/Note 创建与命中（p0-fix 定点 2） ─────────────────────────────────

/// 便签渲染尺寸（draw_note 与 hit_test_note 共用）
pub const NOTE_WIDTH: f64 = 180.0;
pub const NOTE_HEIGHT: f64 = 100.0;

/// p0-fix 定点 2：纯函数 — 拖拽矩形归一化（UT-AREA-01）
/// 返回 (x, y, width, height)；宽或高 < 10px → None（防误触不创建）
pub fn area_rect_from_drag(x1: f64, y1: f64, x2: f64, y2: f64) -> Option<(f64, f64, f64, f64)> {
    let x = x1.min(x2);
    let y = y1.min(y2);
    let w = (x2 - x1).abs();
    let h = (y2 - y1).abs();
    if w < 10.0 || h < 10.0 {
        return None;
    }
    Some((x, y, w, h))
}

/// p0-fix 定点 2：纯函数 — 构建 Area（默认 未命名区域 / #3b82f6，UT-AREA-01）
pub fn build_area(id: String, x: f64, y: f64, width: f64, height: f64) -> Area {
    Area {
        id,
        x,
        y,
        width,
        height,
        color: "#3b82f6".to_string(),
        name: "未命名区域".to_string(),
    }
}

/// p0-fix 定点 2：纯函数 — 构建 Note（默认空内容 / #f59e0b，UT-NOTE-01）
pub fn build_note(id: String, x: f64, y: f64) -> Note {
    Note {
        id,
        x,
        y,
        content: String::new(),
        color: "#f59e0b".to_string(),
    }
}

/// p0-fix 定点 2：纯函数 — 点命中区域矩形（后创建优先）
pub fn hit_test_area(areas: &[Area], x: f64, y: f64) -> Option<String> {
    for a in areas.iter().rev() {
        if x >= a.x && x <= a.x + a.width && y >= a.y && y <= a.y + a.height {
            return Some(a.id.clone());
        }
    }
    None
}

/// p0-fix 定点 2：纯函数 — 点命中便签（180×100 渲染矩形，后创建优先）
pub fn hit_test_note(notes: &[Note], x: f64, y: f64) -> Option<String> {
    for n in notes.iter().rev() {
        if x >= n.x && x <= n.x + NOTE_WIDTH && y >= n.y && y <= n.y + NOTE_HEIGHT {
            return Some(n.id.clone());
        }
    }
    None
}

// ─── Reference line hit test（p0-fix 定点 3） ───────────────────────────────

/// 三次贝塞尔取点（与 RelationPath/calc_path 同一曲线）
pub fn bezier_point(path: &RelationPath, t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let x = u * u * u * path.x1 + 3.0 * u * u * t * path.cx1 + 3.0 * u * t * t * path.cx2 + t * t * t * path.x2;
    let y = u * u * u * path.y1 + 3.0 * u * u * t * path.cy1 + 3.0 * u * t * t * path.cy2 + t * t * t * path.y2;
    (x, y)
}

/// 点到线段距离
pub fn dist_point_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len2 = dx * dx + dy * dy;
    if len2 <= f64::EPSILON {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = (((px - x1) * dx + (py - y1) * dy) / len2).clamp(0.0, 1.0);
    let cx = x1 + t * dx;
    let cy = y1 + t * dy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

/// p0-fix 定点 3：纯函数 — 点是否在贝塞尔关系线附近（UT-MM-33）
/// 24 段折线近似；阈值 8px（覆盖 7px 光晕 + 2px 主线的可点击区域）
pub fn point_near_bezier(path: &RelationPath, x: f64, y: f64, tol: f64) -> bool {
    const SEGMENTS: usize = 24;
    let (mut px, mut py) = (path.x1, path.y1);
    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let (qx, qy) = bezier_point(path, t);
        if dist_point_segment(x, y, px, py, qx, qy) <= tol {
            return true;
        }
        px = qx;
        py = qy;
    }
    false
}

/// p0-fix 定点 3：纯函数 — (x, y) 命中哪条 reference 连线（UT-MM-33）
/// 几何与 `draw_bezier_fields` 一致（calc_path 贝塞尔）；返回 reference id
pub fn hit_test_reference(tables: &[Table], refs: &[Reference], x: f64, y: f64) -> Option<String> {
    for r in refs {
        let from = tables.iter().find(|t| t.id == r.start_table_id);
        let to = tables.iter().find(|t| t.id == r.end_table_id);
        if let (Some(f), Some(t)) = (from, to) {
            let path = calc_path(f, &r.start_field_id, t, &r.end_field_id);
            if point_near_bezier(&path, x, y, 8.0) {
                return Some(r.id.clone());
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
            dict_code: String::new(),
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

    // ─── UT-MM-33 — p0-fix 定点 3 连线命中检测 ─────────────────────────────

    fn ut_mm_33_fixture() -> (Vec<Table>, Vec<Reference>) {
        use crate::editor_core::types::Field;
        let mk = |id: &str, x: f64, fid: &str| Table {
            id: id.into(),
            name: id.into(),
            x,
            y: 130.0,
            color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: fid.into(),
                name: fid.into(),
                type_: "INT".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: false,
                increment: false,
                comment: String::new(),
                tag: String::new(),
                dict_code: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        };
        let tables = vec![mk("t1", 100.0, "f1"), mk("t2", 600.0, "f2")];
        let refs = vec![Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        }];
        (tables, refs)
    }

    /// UT-MM-33: 连线中点命中 / 偏离 20px 不命中 / 无连线返回 None
    #[test]
    fn test_hit_test_reference_ut_mm_33() {
        let (tables, refs) = ut_mm_33_fixture();
        let anchor_y = 130.0 + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT / 2.0;
        // 连线为水平贝塞尔：起点 (100+TABLE_WIDTH, anchor_y) → 终点 (600, anchor_y)
        let mid_x = (100.0 + TABLE_WIDTH + 600.0) / 2.0;
        assert_eq!(
            hit_test_reference(&tables, &refs, mid_x, anchor_y),
            Some("r1".to_string()),
            "UT-MM-33: 连线中点应命中"
        );
        assert_eq!(
            hit_test_reference(&tables, &refs, mid_x, anchor_y + 20.0),
            None,
            "UT-MM-33: 偏离 20px 不应命中"
        );
        assert_eq!(
            hit_test_reference(&tables, &[], mid_x, anchor_y),
            None,
            "UT-MM-33: 无连线应返回 None"
        );
    }

    /// UT-MM-33: bezier_point 端点 与 dist_point_segment / point_near_bezier 基本性质
    #[test]
    fn test_bezier_point_and_dist_ut_mm_33() {
        let path = RelationPath {
            x1: 0.0, y1: 0.0, cx1: 50.0, cy1: 0.0,
            cx2: 50.0, cy2: 100.0, x2: 100.0, y2: 100.0,
        };
        let (sx, sy) = bezier_point(&path, 0.0);
        assert!((sx - 0.0).abs() < 1e-9 && (sy - 0.0).abs() < 1e-9, "UT-MM-33: t=0 应为起点");
        let (ex, ey) = bezier_point(&path, 1.0);
        assert!((ex - 100.0).abs() < 1e-9 && (ey - 100.0).abs() < 1e-9, "UT-MM-33: t=1 应为终点");
        assert!((dist_point_segment(0.0, 5.0, 0.0, 0.0, 10.0, 0.0) - 5.0).abs() < 1e-9);
        assert!((dist_point_segment(-3.0, 0.0, 0.0, 0.0, 10.0, 0.0) - 3.0).abs() < 1e-9);
        assert!(point_near_bezier(&path, 50.0, 50.0, 8.0), "UT-MM-33: 曲线中点应命中");
        assert!(!point_near_bezier(&path, 50.0, 80.0, 4.0), "UT-MM-33: 远离曲线不应命中");
    }

    // ─── UT-AREA-01 / UT-NOTE-01 — p0-fix 定点 2 区域/便签创建 ─────────────

    /// UT-AREA-01: 拖框归一化 + <10px 不创建 + build_area 默认值 + hit_test_area
    #[test]
    fn test_area_create_ut_area_01() {
        // 正常拖框（含反向拖拽归一化）
        assert_eq!(
            area_rect_from_drag(100.0, 100.0, 260.0, 220.0),
            Some((100.0, 100.0, 160.0, 120.0))
        );
        assert_eq!(
            area_rect_from_drag(260.0, 220.0, 100.0, 100.0),
            Some((100.0, 100.0, 160.0, 120.0)),
            "UT-AREA-01: 反向拖框应归一化"
        );
        // 拖框 < 10px 不创建（防误触）
        assert_eq!(area_rect_from_drag(100.0, 100.0, 105.0, 108.0), None);
        assert_eq!(area_rect_from_drag(100.0, 100.0, 100.0, 100.0), None);
        assert_eq!(area_rect_from_drag(100.0, 100.0, 150.0, 105.0), None);

        let area = build_area("a1".into(), 100.0, 100.0, 160.0, 120.0);
        assert_eq!(area.name, "未命名区域");
        assert_eq!(area.color, "#3b82f6");
        assert_eq!(hit_test_area(&[area.clone()], 150.0, 150.0), Some("a1".to_string()));
        assert_eq!(hit_test_area(&[area.clone()], 50.0, 50.0), None);
        // 后创建优先
        let a2 = build_area("a2".into(), 120.0, 120.0, 160.0, 120.0);
        assert_eq!(
            hit_test_area(&[area, a2], 150.0, 150.0),
            Some("a2".to_string()),
            "UT-AREA-01: 重叠区域后创建优先"
        );
    }

    /// UT-NOTE-01: build_note 默认值 + hit_test_note 固定 180×100 命中
    #[test]
    fn test_note_create_ut_note_01() {
        let note = build_note("n1".into(), 200.0, 300.0);
        assert_eq!(note.content, "");
        assert_eq!(note.color, "#f59e0b");
        assert_eq!(
            hit_test_note(&[note.clone()], 200.0 + NOTE_WIDTH / 2.0, 300.0 + NOTE_HEIGHT / 2.0),
            Some("n1".to_string())
        );
        // 渲染矩形边界外不命中
        assert_eq!(hit_test_note(&[note.clone()], 200.0 + NOTE_WIDTH + 1.0, 300.0), None);
        assert_eq!(hit_test_note(&[note], 200.0, 300.0 + NOTE_HEIGHT + 1.0), None);
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
            dict_code: String::new(),
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
            dict_code: String::new(),
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
            dict_code: String::new(),
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
            dict_code: String::new(),
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
            dict_code: String::new(),
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
            dict_code: String::new(),
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

    // ─── UT-MM-30: rAF 调度去重可测同步核（条目 28） ────────────────────────────

    #[test]
    fn test_schedule_render_dedup_first_call_executes_ut_mm_30() {
        let pending = std::cell::Cell::new(false);
        let called = std::cell::Cell::new(false);
        let did = schedule_render_dedup(&pending, || called.set(true));
        assert!(did, "UT-MM-30: 首次入队返 true");
        assert!(pending.get(), "UT-MM-30: 首次入队后 pending=true");
        assert!(called.get(), "UT-MM-30: 首次入队 render_fn 执行");
    }

    #[test]
    fn test_schedule_render_dedup_second_call_noop_ut_mm_30() {
        let pending = std::cell::Cell::new(true); // 模拟已 pending
        let called = std::cell::Cell::new(false);
        let did = schedule_render_dedup(&pending, || called.set(true));
        assert!(!did, "UT-MM-30: 二次入队返 false");
        assert!(!called.get(), "UT-MM-30: 二次入队 render_fn 不执行");
    }

    #[test]
    fn test_schedule_render_dedup_after_clear_can_enqueue_ut_mm_30() {
        let pending = std::cell::Cell::new(false);
        // 首次入队
        let _ = schedule_render_dedup(&pending, || {});
        assert!(pending.get());
        // 模拟 rAF 回调清 pending
        pending.set(false);
        // 再入队可执行
        let called = std::cell::Cell::new(false);
        let did = schedule_render_dedup(&pending, || called.set(true));
        assert!(did, "UT-MM-30: 清 pending 后再入队返 true");
        assert!(called.get(), "UT-MM-30: render_fn 重新执行");
    }

    #[test]
    fn test_schedule_render_dedup_three_cycles_ut_mm_30() {
        let pending = std::cell::Cell::new(false);
        let count = std::cell::Cell::new(0u32);
        for _ in 0..3 {
            let _ = schedule_render_dedup(&pending, || count.set(count.get() + 1));
            // 模拟 rAF 回调清 pending
            pending.set(false);
        }
        assert_eq!(count.get(), 3, "UT-MM-30: 3 轮入队各执行一次");
    }

    #[test]
    fn test_text_cache_key_eq_partial_ut_mm_30() {
        // TextCacheKey 同 text/weight/dpr + font_px 容差 0.01 内相等
        let k1 = TextCacheKey { text: "hello".into(), font_weight: 400, font_px: 13.0, dpr: 100 };
        let k2 = TextCacheKey { text: "hello".into(), font_weight: 400, font_px: 13.005, dpr: 100 };
        assert_eq!(k1, k2, "UT-MM-30: font_px 差 0.005 容差内相等");
    }

    #[test]
    fn test_text_cache_key_neq_text_ut_mm_30() {
        let k1 = TextCacheKey { text: "hello".into(), font_weight: 400, font_px: 13.0, dpr: 100 };
        let k2 = TextCacheKey { text: "world".into(), font_weight: 400, font_px: 13.0, dpr: 100 };
        assert_ne!(k1, k2, "UT-MM-30: 不同 text 不等");
    }

    // ─── fix-canvas-zoom-perf：§3.3 锚定缩放 + §5.6 R-PERF-01~06 ─────────────

    fn assert_anchor_invariant(t: &Transform, world: (f64, f64), anchor: (f64, f64), tag: &str) {
        let eps = 1e-9;
        assert!(
            (world.0 * t.zoom + t.pan_x - anchor.0).abs() < eps,
            "{tag}: 锚定不变量 world.x × zoom + pan.x == anchor.x"
        );
        assert!(
            (world.1 * t.zoom + t.pan_y - anchor.1).abs() < eps,
            "{tag}: 锚定不变量 world.y × zoom + pan.y == anchor.y"
        );
    }

    #[test]
    fn test_zoom_anchor_invariant_ut_cr_zoom_01() {
        // §3.3 合同值：transform {100, 60, 1.0}；rect.origin (40, 32)；光标 client (440, 332)
        let t0 = Transform { pan_x: 100.0, pan_y: 60.0, zoom: 1.0 };
        let rect_origin = (40.0, 32.0);
        let mouse = (440.0, 332.0);
        let anchor = (mouse.0 - rect_origin.0, mouse.1 - rect_origin.1);
        assert_eq!(anchor, (400.0, 300.0), "UT-CR-ZOOM-01: anchor = client − rect.left/top");
        let world = (
            (anchor.0 - t0.pan_x) / t0.zoom,
            (anchor.1 - t0.pan_y) / t0.zoom,
        );

        let t1 = zoom_transform_at_anchor(&t0, anchor, 1.1);
        assert!((t1.zoom - 1.1).abs() < 1e-9, "UT-CR-ZOOM-01: t1.zoom == 1.1");
        assert!(
            (t1.pan_x - (anchor.0 - world.0 * 1.1)).abs() < 1e-9,
            "UT-CR-ZOOM-01: pan_x == anchor.x − world.x × 1.1"
        );
        assert_anchor_invariant(&t1, world, anchor, "UT-CR-ZOOM-01 第一次缩放");

        let t2 = zoom_transform_at_anchor(&t1, anchor, 1.1);
        assert_anchor_invariant(&t2, world, anchor, "UT-CR-ZOOM-01 第二次缩放");

        // clamp 子用例：zoom=5.0 × 1.1 → clamp 到 ZOOM_MAX=5.0，且仍按 clamp 后 zoom 保持锚定
        let tmax = Transform { pan_x: 100.0, pan_y: 60.0, zoom: 5.0 };
        let tc = zoom_transform_at_anchor(&tmax, anchor, 1.1);
        assert!((tc.zoom - ZOOM_MAX).abs() < 1e-9, "UT-CR-ZOOM-01: clamp 于 ZOOM_MAX=5.0");
        let world_max = (
            (anchor.0 - tmax.pan_x) / tmax.zoom,
            (anchor.1 - tmax.pan_y) / tmax.zoom,
        );
        assert_anchor_invariant(&tc, world_max, anchor, "UT-CR-ZOOM-01 clamp 锚定优先");

        // 下界 clamp：zoom=0.1 ÷ 1.1 → clamp 于 ZOOM_MIN=0.1
        let tmin = Transform { pan_x: 0.0, pan_y: 0.0, zoom: 0.1 };
        let tf = zoom_transform_at_anchor(&tmin, (200.0, 150.0), 1.0 / 1.1);
        assert!((tf.zoom - ZOOM_MIN).abs() < 1e-9, "UT-CR-ZOOM-01: clamp 于 ZOOM_MIN=0.1");
    }

    #[test]
    fn test_aabb_intersects_ut_cr_cull_01() {
        // 相交 / 包含 / 相离 / 边界贴合 4 case
        assert!(aabb_intersects((0.0, 0.0, 100.0, 100.0), (50.0, 50.0, 100.0, 100.0)), "相交");
        assert!(aabb_intersects((0.0, 0.0, 200.0, 200.0), (50.0, 50.0, 10.0, 10.0)), "包含");
        assert!(!aabb_intersects((0.0, 0.0, 100.0, 100.0), (200.0, 200.0, 50.0, 50.0)), "相离");
        assert!(aabb_intersects((0.0, 0.0, 100.0, 100.0), (100.0, 100.0, 50.0, 50.0)), "边界贴合");
    }

    #[test]
    fn test_collect_visible_culls_offscreen_ut_cr_cull_01() {
        let vp = (0.0, 0.0, 1920.0, 1080.0);
        let on = fixture_table("on", "f1", 100.0, 100.0);
        let off = fixture_table("off", "f2", -5000.0, -5000.0);
        let tables = vec![on, off];

        let vis_tables = collect_visible_tables(&tables, vp);
        assert_eq!(vis_tables.len(), 1, "UT-CR-CULL-01: 视口外的表不进入绘制列表");
        assert_eq!(vis_tables[0].id, "on");

        // 视口 AABB 由 transform + CSS 尺寸推导：pan 100/60、zoom 1、1920×1080 → 左上角 (-100,-60)
        let derived = viewport_world_aabb(&Transform { pan_x: 100.0, pan_y: 60.0, zoom: 1.0 }, 1920.0, 1080.0);
        assert_eq!(derived, (-100.0, -60.0, 1920.0, 1080.0));

        // 关系：两端都在视口外 → 剔除；一端可见 → 保留（HashMap O(1) 端点查找路径）
        let far_a = fixture_table("fa", "f1", -5000.0, -5000.0);
        let far_b = fixture_table("fb", "f2", -9000.0, -9000.0);
        let visible_pair_t = fixture_table("vt", "f3", 100.0, 100.0);
        let all = vec![far_a, far_b, visible_pair_t];
        let mk_ref = |id: &str, start_t: &str, end_t: &str, start_f: &str, end_f: &str| Reference {
            id: id.into(),
            name: format!("ref-{id}"),
            start_table_id: start_t.into(),
            end_table_id: end_t.into(),
            start_field_id: start_f.into(),
            end_field_id: end_f.into(),
            type_: "1:N".into(),
            on_delete: String::new(),
            on_update: String::new(),
        };
        let refs = vec![
            mk_ref("r-far", "fa", "fb", "f1", "f2"), // 两端都在视口外
            mk_ref("r-vis", "fa", "vt", "f1", "f3"), // vt 在视口内
        ];
        let vis_refs = collect_visible_refs(&refs, &all, vp);
        assert_eq!(vis_refs.len(), 1, "UT-CR-CULL-01: 双端视口外的关系被剔除");
        assert_eq!(vis_refs[0].id, "r-vis");

        // 区域 / 便签裁剪
        let areas = vec![
            crate::editor_core::types::Area { id: "a-on".into(), x: 10.0, y: 10.0, width: 200.0, height: 120.0, color: "#3b82f6".into(), name: "A".into() },
            crate::editor_core::types::Area { id: "a-off".into(), x: -5000.0, y: -5000.0, width: 200.0, height: 120.0, color: "#3b82f6".into(), name: "B".into() },
        ];
        let vis_areas = collect_visible_areas(&areas, vp);
        assert_eq!(vis_areas.len(), 1, "UT-CR-CULL-01: 视口外的区域不进入绘制列表");
        assert_eq!(vis_areas[0].id, "a-on");

        let notes = vec![
            crate::editor_core::types::Note { id: "n-on".into(), x: 50.0, y: 50.0, content: "hi".into(), color: "#f59e0b".into() },
            crate::editor_core::types::Note { id: "n-off".into(), x: -5000.0, y: -5000.0, content: "far".into(), color: "#f59e0b".into() },
        ];
        let vis_notes = collect_visible_notes(&notes, vp);
        assert_eq!(vis_notes.len(), 1, "UT-CR-CULL-01: 视口外的便签不进入绘制列表");
        assert_eq!(vis_notes[0].id, "n-on");
    }

    /// UT-CR-BATCH-01：mock sink 计数——begin ≤ 1、fill == 1，且与点数无关。
    #[derive(Default)]
    struct MockGridSink {
        begin: usize,
        dots: usize,
        fill: usize,
    }
    impl GridSink for MockGridSink {
        fn begin(&mut self) {
            self.begin += 1;
        }
        fn dot(&mut self, _x: f64, _y: f64) {
            self.dots += 1;
        }
        fn fill(&mut self) {
            self.fill += 1;
        }
    }

    #[test]
    fn test_grid_batch_single_fill_ut_cr_batch_01() {
        let t = Transform::default();
        let mut sink = MockGridSink::default();
        paint_grid(&t, 1920.0, 1080.0, &mut sink);
        assert_eq!(sink.begin, 1, "UT-CR-BATCH-01: begin_path 恰 1 次");
        assert_eq!(sink.fill, 1, "UT-CR-BATCH-01: fill 恰 1 次");
        assert!(sink.dots > 1000, "UT-CR-BATCH-01: 点数 > 1000（96×54 栅格）");

        // 密度翻倍（点数 ×4）后绘制调用数不变 —— O(1)
        let mut dense = MockGridSink::default();
        paint_grid(&t, 3840.0, 2160.0, &mut dense);
        assert_eq!(dense.begin, 1, "UT-CR-BATCH-01: 点数翻倍 begin 仍 1 次");
        assert_eq!(dense.fill, 1, "UT-CR-BATCH-01: 点数翻倍 fill 仍 1 次");
        assert!(dense.dots > sink.dots * 3, "UT-CR-BATCH-01: 点数确随面积增长（批量未丢点）");

        // pan/zoom 下点集仍只覆盖视口（不整屏无限延伸）
        let panned = Transform { pan_x: 100.0, pan_y: 60.0, zoom: 2.0 };
        let mut zs = MockGridSink::default();
        paint_grid(&panned, 1920.0, 1080.0, &mut zs);
        assert_eq!(zs.begin, 1);
        assert_eq!(zs.fill, 1);
        assert!(zs.dots < sink.dots, "UT-CR-BATCH-01: zoom=2 时视口内点数按 1/zoom² 收缩");
    }

    /// UT-CR-SPRITE-01 — 表精灵缓存：zoom 分档边界 + 指纹语义（R-PERF-07）
    #[test]
    fn ut_cr_sprite_01_bucket_and_fingerprint() {
        // zoom 分档：≤1 取 1，>1 取 2（边界 1.0 恰好取 1）
        assert_eq!(sprite_zoom_bucket(0.5), 1);
        assert_eq!(sprite_zoom_bucket(1.0), 1, "UT-CR-SPRITE-01: zoom=1.0 边界取档 1");
        assert_eq!(sprite_zoom_bucket(1.01), 2);
        assert_eq!(sprite_zoom_bucket(2.0), 2);

        // 指纹：位置/选中态不参与（移动不换缓存）
        let t = fixture_table("t1", "f1", 100.0, 80.0);
        let mut moved = t.clone();
        moved.x = 999.0;
        moved.y = -40.0;
        assert_eq!(
            table_sprite_fingerprint(&t, true, 200, 1),
            table_sprite_fingerprint(&moved, true, 200, 1),
            "UT-CR-SPRITE-01: 位置变化不得改变指纹"
        );

        // 内容变更失效：改名 / 改字段类型 / 改主键 / 改色 / 改宽 / 主题 / dpr / 分档
        let base = table_sprite_fingerprint(&t, true, 200, 1);
        let mut renamed = t.clone();
        renamed.name = "other".into();
        assert_ne!(table_sprite_fingerprint(&renamed, true, 200, 1), base, "改名失效");
        let mut retyped = t.clone();
        retyped.fields[0].type_ = "UUID".into();
        assert_ne!(table_sprite_fingerprint(&retyped, true, 200, 1), base, "改字段类型失效");
        let mut unpk = t.clone();
        unpk.fields[0].primary = false;
        assert_ne!(table_sprite_fingerprint(&unpk, true, 200, 1), base, "改主键失效");
        let mut recolor = t.clone();
        recolor.color = "#fff".into();
        assert_ne!(table_sprite_fingerprint(&recolor, true, 200, 1), base, "改色失效");
        let mut resized = t.clone();
        resized.width = Some(320);
        assert_ne!(table_sprite_fingerprint(&resized, true, 200, 1), base, "改宽失效");
        assert_ne!(table_sprite_fingerprint(&t, false, 200, 1), base, "主题切换失效");
        assert_ne!(table_sprite_fingerprint(&t, true, 100, 1), base, "dpr 变化失效");
        assert_ne!(table_sprite_fingerprint(&t, true, 200, 2), base, "zoom 分档切换失效");
    }

    /// UT-CR-GUARD-01 — DOM 写守护：值未变不写（R-PERF-08）
    #[test]
    fn ut_cr_guard_01_dom_write_guard() {
        let mut last = String::new();
        // 首帧空值已相等 → 不写
        assert!(!dom_write_guard(&mut last, ""), "UT-CR-GUARD-01: 空→空 不写");
        // 值变化 → 写并更新 last
        assert!(dom_write_guard(&mut last, "M0 0"), "UT-CR-GUARD-01: 值变化应写");
        assert_eq!(last, "M0 0");
        // 同值连帧 → 不写（拖动中每帧重算同路径的常态）
        assert!(!dom_write_guard(&mut last, "M0 0"), "UT-CR-GUARD-01: 同值不重写");
        assert!(dom_write_guard(&mut last, "M1 1"), "UT-CR-GUARD-01: 新值再写");
        assert!(dom_write_guard(&mut last, ""), "UT-CR-GUARD-01: 清空也是变更");
    }

    /// UT-CR-DPR-01 — 拖拽降采样 dpr 纯函数：活跃压到 ≤cap，非活跃原样透传（R-PERF-10）
    #[test]
    fn ut_cr_dpr_01_capped_drag_dpr() {
        // 拖拽活跃：高于 cap 的 dpr 一律压到 1.0
        assert_eq!(capped_drag_dpr(2.0, true), DRAG_RENDER_DPR_CAP);
        assert_eq!(capped_drag_dpr(3.0, true), DRAG_RENDER_DPR_CAP);
        assert_eq!(capped_drag_dpr(1.5, true), DRAG_RENDER_DPR_CAP);
        // 低 dpr 设备不再降（min 语义，不抬升）
        assert_eq!(capped_drag_dpr(0.8, true), 0.8);
        assert_eq!(capped_drag_dpr(1.0, true), 1.0);
        // 非拖拽：原样透传，全分辨率渲染
        assert_eq!(capped_drag_dpr(2.0, false), 2.0);
        assert_eq!(capped_drag_dpr(1.0, false), 1.0);
    }

    /// UT-CR-GHOST-01 — 幽灵层准入判定与 CSS 定位纯函数（R-PERF-11）
    #[test]
    fn ut_cr_ghost_01_has_refs_and_position() {
        let mk = |id: &str, s: &str, e: &str| Reference {
            id: id.to_string(),
            name: String::new(),
            start_table_id: s.to_string(),
            end_table_id: e.to_string(),
            start_field_id: String::new(),
            end_field_id: String::new(),
            type_: String::new(),
            on_delete: String::new(),
            on_update: String::new(),
        };
        let refs = vec![mk("r1", "a", "b")];
        assert!(table_has_references(&refs, "a"), "UT-CR-GHOST-01: 起点表有关联，禁走幽灵层");
        assert!(table_has_references(&refs, "b"), "UT-CR-GHOST-01: 终点表有关联，禁走幽灵层");
        assert!(!table_has_references(&refs, "c"), "UT-CR-GHOST-01: 无关联表可走幽灵层");
        assert!(!table_has_references(&[], "a"), "UT-CR-GHOST-01: 空 refs 可走幽灵层");

        // 定位：(世界坐标 - margin) × zoom + pan
        let (l, tp) = ghost_css_position(100.0, 50.0, 24.0, 10.0, 20.0, 2.0);
        assert_eq!(l, (100.0 - 24.0) * 2.0 + 10.0, "UT-CR-GHOST-01: left 换算");
        assert_eq!(tp, (50.0 - 24.0) * 2.0 + 20.0, "UT-CR-GHOST-01: top 换算");
        assert_eq!(ghost_css_position(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), (0.0, 0.0));
    }

    #[test]
    fn test_font_family_probe_cached_ut_cr_fontcache_01() {
        let mut cache: HashMap<(String, String), String> = HashMap::new();
        let key = (CANVAS_FONT.to_string(), CANVAS_FONT_MONO.to_string());
        let probes = std::cell::Cell::new(0usize);

        // 第 1 次解析：探测 1 次并写入缓存
        let r1 = resolve_font_family_cached(&mut cache, key.clone(), || {
            probes.set(probes.get() + 1);
            "Noto Sans SC".to_string()
        });
        assert_eq!(r1, "Noto Sans SC");
        assert_eq!(probes.get(), 1, "UT-CR-FONTCACHE-01: 首次解析探测 1 次");

        // 第 2 次解析（同键）：缓存命中，不再探测
        let r2 = resolve_font_family_cached(&mut cache, key.clone(), || {
            probes.set(probes.get() + 1);
            "other".to_string()
        });
        assert_eq!(r2, "Noto Sans SC", "UT-CR-FONTCACHE-01: 缓存命中返回首次结果");
        assert_eq!(probes.get(), 1, "UT-CR-FONTCACHE-01: 二次解析不再调用 check");

        // 模拟同帧 20 张表 draw_table 各触发一次字体解析：探测总次数仍为 1
        for _ in 0..20 {
            let _ = resolve_font_family_cached(&mut cache, key.clone(), || {
                probes.set(probes.get() + 1);
                "x".to_string()
            });
        }
        assert!(probes.get() <= 1, "UT-CR-FONTCACHE-01: 20 张表一帧渲染 check 总次数 ≤ 1");

        // loadingdone 失效：清空缓存后重探（wasm 侧由 loadingdone 监听驱动 invalidate）
        cache.clear();
        let r3 = resolve_font_family_cached(&mut cache, key.clone(), || {
            probes.set(probes.get() + 1);
            "PingFang SC".to_string()
        });
        assert_eq!(r3, "PingFang SC");
        assert_eq!(probes.get(), 2, "UT-CR-FONTCACHE-01: 缓存失效后重探");
    }

    #[test]
    fn test_drag_overlay_stores_only_pos_ut_cr_drag_01() {
        // R-PERF-04：100 张表场景下，20 次 pointermove 只写 (id, x, y) 覆盖——
        // 拖动中 store Vec 不被克隆/修改；落账走 pointerup 原地写。
        let tables: Vec<Table> = (0..100)
            .map(|i| fixture_table(&format!("t{i}"), "f1", 100.0 + i as f64 * 300.0, 100.0))
            .collect();
        let mut live = LivePaint::default();
        assert!(live.table_pos.is_none());

        // 模拟表头 pointermove × 20（与 on_pointermove 表分支同一写入形态）
        for i in 0..20 {
            let new_x = 160.0 + i as f64;
            let new_y = 140.0;
            live.table_pos = Some(("t0".to_string(), new_x, new_y));
        }
        assert_eq!(
            live.table_pos,
            Some(("t0".to_string(), 179.0, 140.0)),
            "UT-CR-DRAG-01: 覆盖层只保留被拖对象最新 (id, x, y)"
        );

        // 拖动中 store 表坐标不被触碰（落账仅发生在 pointerup）
        assert_eq!(tables[0].x, 100.0, "UT-CR-DRAG-01: 拖动中 store 坐标不变（无整 Vec 克隆写回）");
        assert_eq!(tables[99].x, 100.0 + 99.0 * 300.0);

        // 绘制输入消费覆盖坐标（draw_canvas → table_draw_position / table_with_override）
        let (dx, dy) = table_draw_position(&tables[0], Some(("t0", 179.0, 140.0)));
        assert_eq!((dx, dy), (179.0, 140.0), "UT-CR-DRAG-01: 被拖表以覆盖坐标绘制");
        let (sx, sy) = table_draw_position(&tables[1], Some(("t0", 179.0, 140.0)));
        assert_eq!((sx, sy), (tables[1].x, tables[1].y), "UT-CR-DRAG-01: 未命中表用原坐标");
        let visual = table_with_override(&tables[0], Some(("t0", 179.0, 140.0)));
        assert_eq!((visual.x, visual.y), (179.0, 140.0));
        assert_eq!(tables[0].x, 100.0, "UT-CR-DRAG-01: table_with_override 不修改输入表");

        // apply_visual_table_position 保持纯函数语义（松手落账 / UT-CR-06 用）
        let committed = apply_visual_table_position(&tables, "t0", 200.0, 220.0);
        assert_eq!((committed[0].x, committed[0].y), (200.0, 220.0));
        assert_eq!(tables[0].x, 100.0, "UT-CR-DRAG-01: 落账纯函数不修改输入 Vec");
    }
}
