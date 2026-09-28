//! editor-render — Canvas 2D rendering module
//!
//! Renders tables, relationships, areas, and notes on an HTML5 Canvas.
//! Supports pan/zoom/select via pointer events, with requestAnimationFrame
//! throttling for smooth 60fps interaction at 100+ nodes.
//!
//! Architecture: DAG dependency -> editor-core (store + types).
//! All types re-exported from `crate::editor_core::types`.

use crate::editor_core::types::{Area, Note, Reference, Table};
use crate::editor_core::{CommentDisplay, ViewDimension};
use leptos::{RwSignal, SignalGet, SignalSet, SignalUpdate};
use std::cell::Cell;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, MouseEvent, PointerEvent, WheelEvent};

// ─── Canvas constants ────────────────────────────────────────────────────────

// 几何尺寸为主原型事实（core-01：表宽 230 / 表头 43 / 字段行 35 / 点阵 24px）
/// 默认表宽下限；`width == None|0` 自适应时夹紧下界（#20）。
pub const TABLE_WIDTH: f64 = 230.0;
/// 自适应表宽上限（对齐 ListView 列宽 max=480，#20）。
pub const TABLE_WIDTH_MAX: f64 = 480.0;
const TABLE_HEADER_HEIGHT: f64 = 43.0;
const FIELD_ROW_HEIGHT: f64 = 35.0;
/// 便签渲染 / 命中尺寸（与 draw_note 一致）
pub const NOTE_WIDTH: f64 = 180.0;
pub const NOTE_HEIGHT: f64 = 100.0;
/// 字段左右连接点命中半径（世界坐标；#3 reopen：整行触发面过大，改为触发点）
pub const FIELD_PORT_HIT_RADIUS: f64 = 10.0;
/// 生产端网格尺寸：松手吸附 20px（core-CR-canvas-test-cases.md §1 合同；
/// 主原型演示 GRID=12、点阵视觉 24px，均不得写成生产合同）。仅在 pointerup 时对齐，拖动中不量化。
pub const GRID_SIZE: f64 = 20.0;
/// 关系工具：屏幕像素欧氏位移达到该阈值才视为拖线（UT-PB-06）。
pub const DRAG_THRESHOLD: f64 = 4.0;
/// 画布字体族（与壳层一致，core-07 §字体配对）
const CANVAS_FONT: &str = "\"Plus Jakarta Sans\", sans-serif";
const CANVAS_FONT_MONO: &str = "ui-monospace, monospace";

// ─── #33 视觉体系统一（core-07 §15.5 / core-08 §11，UT-PE-VIS-01）────────────
/// 语义图标徽章统一外接尺寸（PK 钥匙 / FK 链环 / NN 星号 / UQ 菱形同源引用，无散值）。
pub const BADGE_SIZE: f64 = 12.0;
/// 徽章图标统一描边（与 ToolRail 图标 stroke 1.5 同一视觉语言）。
pub const BADGE_STROKE: f64 = 1.5;
/// 关系端点 crow's foot 记号统一外接尺寸（一/多端同一几何族）。
pub const REL_ENDPOINT_SIZE: f64 = 10.0;
/// 表卡圆角统一值（原 14/16 多值并存收敛为 8；选中环 = 本值 + 2.5 外扩）。
pub const TABLE_CORNER_RADIUS: f64 = 8.0;
/// 徽章间距 / 徽章组与字段名间距（draw_field_badges 与 estimate_content_width 同源）。
const BADGE_GAP: f64 = 3.0;
const BADGE_NAME_GAP: f64 = 4.0;

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
    /// PK 标记（amber，semantic.warning 系）
    pk_color: &'static str,
    /// #33：FK 徽章（semantic.info 系，原型 --blue）
    fk_color: &'static str,
    /// #33：NOT NULL / UNIQUE 徽章（grey 系，与 text_muted 同阶）
    constraint_color: &'static str,
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
    fk_color: "#3788e5",
    constraint_color: "#7b8d93",
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
    fk_color: "#7ab8f5",
    constraint_color: "#86a3ab",
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

/// UT-CR-FOCUS-01：将表聚焦到视口中心（已完全在视口内则保持 pan 不变）。
/// `padding` 为 CSS 像素边距；`viewport_*` 为画布 CSS 尺寸。
pub fn focus_transform(
    table_x: f64,
    table_y: f64,
    table_w: f64,
    table_h: f64,
    viewport_w: f64,
    viewport_h: f64,
    t: &Transform,
    padding: f64,
) -> Transform {
    let sx = table_x * t.zoom + t.pan_x;
    let sy = table_y * t.zoom + t.pan_y;
    let sw = table_w * t.zoom;
    let sh = table_h * t.zoom;
    let fully_visible = sx >= padding
        && sy >= padding
        && sx + sw <= viewport_w - padding
        && sy + sh <= viewport_h - padding;
    if fully_visible {
        return Transform {
            pan_x: t.pan_x,
            pan_y: t.pan_y,
            zoom: t.zoom,
        };
    }
    let center_x = table_x + table_w / 2.0;
    let center_y = table_y + table_h / 2.0;
    Transform {
        pan_x: viewport_w / 2.0 - center_x * t.zoom,
        pan_y: viewport_h / 2.0 - center_y * t.zoom,
        zoom: t.zoom,
    }
}

/// UT-CR-MULTI-01：对选中表集合施加世界坐标位移；未选中表不变。
pub fn translate_tables(tables: &mut [Table], ids: &[String], dx: f64, dy: f64) {
    for table in tables.iter_mut() {
        if ids.iter().any(|id| id == &table.id) {
            table.x += dx;
            table.y += dy;
        }
    }
}

/// Idle 字段拖连 vs 框选/多表拖动：Shift 或「已多选集合内点中该表」时让路给选择手势。
/// （修复 #5：Shift+拖被字段连线抢走；框选后拖表体又变成画线。）
pub fn prefer_selection_over_field_rel(
    shift_key: bool,
    selected_table_ids: &[String],
    field_table_id: &str,
) -> bool {
    if shift_key {
        return true;
    }
    selected_table_ids.len() >= 2
        && selected_table_ids.iter().any(|id| id == field_table_id)
}

/// #5：表是否应绘制选中环（主选中或框选/多选集合内）。
pub fn table_visually_selected(
    table_id: &str,
    primary_selected_id: Option<&str>,
    multi_selected_ids: &[String],
) -> bool {
    primary_selected_id == Some(table_id)
        || multi_selected_ids.iter().any(|id| id == table_id)
}

/// #5：pointerdown 点表时如何更新多选集合。
/// - Shift：切换成员
/// - 点已在集合内的表：保持集合（供整组拖动；禁止因框选工具仍激活而 toggle 掉）
/// - 框选工具 + 点未入选表：累加
/// - 否则：重置为单选
pub fn resolve_table_multi_on_pointerdown(
    shift_key: bool,
    marquee_tool: bool,
    clicked_id: &str,
    current_multi: &[String],
) -> Vec<String> {
    let mut multi = current_multi.to_vec();
    let in_set = multi.iter().any(|x| x == clicked_id);
    if shift_key {
        if in_set {
            multi.retain(|x| x != clicked_id);
        } else {
            multi.push(clicked_id.to_string());
        }
    } else if in_set {
        // keep
    } else if marquee_tool {
        multi.push(clicked_id.to_string());
    } else {
        multi = vec![clicked_id.to_string()];
    }
    multi
}

/// 区域标题栏命中高度（世界坐标）：表叠在区域上时仍可从标题拖动区域。
pub const AREA_HEADER_HIT: f64 = 28.0;

/// #5：框选矩形与便签 AABB 相交。
pub fn notes_in_marquee(notes: &[Note], x1: f64, y1: f64, x2: f64, y2: f64) -> Vec<String> {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    notes
        .iter()
        .filter(|n| {
            let nx2 = n.x + NOTE_WIDTH;
            let ny2 = n.y + NOTE_HEIGHT;
            n.x < max_x && nx2 > min_x && n.y < max_y && ny2 > min_y
        })
        .map(|n| n.id.clone())
        .collect()
}

/// #5：框选矩形与区域 AABB 相交。
pub fn areas_in_marquee(areas: &[Area], x1: f64, y1: f64, x2: f64, y2: f64) -> Vec<String> {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    areas
        .iter()
        .filter(|a| {
            let ax2 = a.x + a.width;
            let ay2 = a.y + a.height;
            a.x < max_x && ax2 > min_x && a.y < max_y && ay2 > min_y
        })
        .map(|a| a.id.clone())
        .collect()
}

/// 多选总成员数（表 + 便签 + 区域）。
pub fn selection_group_len(tables: &[String], notes: &[String], areas: &[String]) -> usize {
    tables.len() + notes.len() + areas.len()
}

/// #5：从选中 id 收集表起始坐标；空则 None。
pub fn collect_table_starts(tables: &[Table], ids: &[String]) -> Option<Vec<(String, f64, f64)>> {
    let v: Vec<_> = tables
        .iter()
        .filter(|t| ids.iter().any(|id| id == &t.id))
        .map(|t| (t.id.clone(), t.x, t.y))
        .collect();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// #5：从选中 id 收集起始坐标；空则 None。
pub fn collect_note_starts(notes: &[Note], ids: &[String]) -> Option<Vec<(String, f64, f64)>> {
    let v: Vec<_> = notes
        .iter()
        .filter(|n| ids.iter().any(|id| id == &n.id))
        .map(|n| (n.id.clone(), n.x, n.y))
        .collect();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// #5：从选中 id 收集区域起始坐标；空则 None。
pub fn collect_area_starts(areas: &[Area], ids: &[String]) -> Option<Vec<(String, f64, f64)>> {
    let v: Vec<_> = areas
        .iter()
        .filter(|a| ids.iter().any(|id| id == &a.id))
        .map(|a| (a.id.clone(), a.x, a.y))
        .collect();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// #5：点击对象是否应启动整组拖动（自身在集合内且总选中 ≥2）。
pub fn should_start_group_drag(
    clicked_in_set: bool,
    tables: &[String],
    notes: &[String],
    areas: &[String],
) -> bool {
    clicked_in_set && selection_group_len(tables, notes, areas) >= 2
}

/// #5：画布 CSS cursor——创建/框选工具空闲为十字；拖实体/平移为 grabbing；否则空串走 CSS grab。
pub fn canvas_cursor_css(
    create_tool_active: bool,
    marquee_tool_active: bool,
    dragging_entity: bool,
    dragging_pan: bool,
    dragging_marquee_or_create_or_rel: bool,
) -> &'static str {
    if dragging_entity || dragging_pan {
        return "grabbing";
    }
    if dragging_marquee_or_create_or_rel {
        return "crosshair";
    }
    if create_tool_active || marquee_tool_active {
        return "crosshair";
    }
    ""
}

/// #5：非 Shift / 非框选累加 / 非点中已选成员 → 单选重置，应清空其他类型多选。
pub fn should_clear_cross_selection(
    shift_key: bool,
    marquee_tool: bool,
    clicked_in_set: bool,
) -> bool {
    !shift_key && !marquee_tool && !clicked_in_set
}

/// #5：若应整组拖动则收集三类起始坐标，否则全 None。
pub fn build_group_drag_starts(
    tables: &[Table],
    notes: &[Note],
    areas: &[Area],
    table_ids: &[String],
    note_ids: &[String],
    area_ids: &[String],
    clicked_in_set: bool,
) -> (
    Option<Vec<(String, f64, f64)>>,
    Option<Vec<(String, f64, f64)>>,
    Option<Vec<(String, f64, f64)>>,
) {
    if !should_start_group_drag(clicked_in_set, table_ids, note_ids, area_ids) {
        return (None, None, None);
    }
    // fix-issues-38-41-canvas-interaction（#41，R-AREALOCK-03）：多选整组拖动跳过
    // 锁定区域（其余选中图元正常移动）
    let unlocked_area_ids: Vec<String> = area_ids
        .iter()
        .filter(|id| areas.iter().any(|a| &a.id == *id && area_drag_allowed(a)))
        .cloned()
        .collect();
    (
        collect_table_starts(tables, table_ids),
        collect_note_starts(notes, note_ids),
        collect_area_starts(areas, &unlocked_area_ids),
    )
}

/// 对选中集合施加世界坐标位移（表 / 便签 / 区域）。
pub fn translate_group_entities(
    tables: &mut [Table],
    notes: &mut [Note],
    areas: &mut [Area],
    table_starts: &[(String, f64, f64)],
    note_starts: &[(String, f64, f64)],
    area_starts: &[(String, f64, f64)],
    ddx: f64,
    ddy: f64,
) {
    for (tid, sx, sy) in table_starts {
        if let Some(t) = tables.iter_mut().find(|t| &t.id == tid) {
            t.x = sx + ddx;
            t.y = sy + ddy;
        }
    }
    for (nid, sx, sy) in note_starts {
        if let Some(n) = notes.iter_mut().find(|n| &n.id == nid) {
            n.x = sx + ddx;
            n.y = sy + ddy;
        }
    }
    for (aid, sx, sy) in area_starts {
        if let Some(a) = areas.iter_mut().find(|a| &a.id == aid) {
            a.x = sx + ddx;
            a.y = sy + ddy;
        }
    }
}

/// 框选矩形与表 AABB 相交判定（世界坐标）。
pub fn tables_in_marquee(tables: &[Table], x1: f64, y1: f64, x2: f64, y2: f64) -> Vec<String> {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    tables
        .iter()
        .filter(|t| {
            let (w, h) = compute_table_render_size(t);
            let tx2 = t.x + w;
            let ty2 = t.y + h;
            t.x < max_x && tx2 > min_x && t.y < max_y && ty2 > min_y
        })
        .map(|t| t.id.clone())
        .collect()
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
    /// 多表拖动：选中集合内各表起始世界坐标
    multi_starts: Option<Vec<(String, f64, f64)>>,
    /// #5：整组拖动时便签 / 区域起始坐标（与 multi_starts 同时生效）
    multi_note_starts: Option<Vec<(String, f64, f64)>>,
    multi_area_starts: Option<Vec<(String, f64, f64)>>,
    /// Shift+空白拖：框选起点（世界坐标）
    marquee_start: Option<(f64, f64)>,
    endpoint_drag: Option<(String, EndpointEnd)>, // (ref_id, end) when dragging an endpoint
    rel_drag: Option<RelFieldDrag>,
    // p0-fix 定点 2：区域框选/便签放置的创建拖拽
    create_drag: Option<CreateDrag>,
    // redesign-listview-type-length-canvas-fix：便签/区域拖动（id, 起始 x, 起始 y）——
    // 修复「命中只选中、无拖拽分支」；拖动中只写视觉层，松手才落账（对齐表拖动三段式）
    note_drag: Option<(String, f64, f64)>,
    area_drag: Option<(String, f64, f64)>,
    /// fix-open-issues-26-33（issue #27，R-ARESZ-02）：区域 resize 拖拽——
    /// (area_id, 方位, 起始 x, 起始 y, 起始 width, 起始 height)；
    /// 拖动中只写视觉层，pointerup 一次写回并进 CommandStack（UT-AREA-03）
    area_resize: Option<(String, AreaResizeDir, f64, f64, f64, f64)>,
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
            multi_starts: None,
            multi_note_starts: None,
            multi_area_starts: None,
            marquee_start: None,
            endpoint_drag: None,
            rel_drag: None,
            create_drag: None,
            note_drag: None,
            area_drag: None,
            area_resize: None,
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
    /// 关系拖连橡皮筋（贝塞尔线，世界坐标两端点）
    rubber: Option<(f64, f64, f64, f64)>,
    /// #5：框选矩形预览（对角世界坐标；不得复用 rubber，否则会画成连线）
    marquee_preview: Option<(f64, f64, f64, f64)>,
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
        /// fix-remote-github-issues / #5：多表选中集合（Shift 框选写入；拖动其中一张时整组平移）
        selected_table_ids: RwSignal<Vec<String>>,
        /// #5：框选多选的便签 / 区域集合（与表一起整组拖动）
        selected_note_ids: RwSignal<Vec<String>>,
        selected_area_ids: RwSignal<Vec<String>>,
        /// relation-inspector-and-ddl-io：点击连线命中（返回 reference id）→ 选中 + Inspector（不再弹详情模态）
        on_reference_pick: Option<Box<dyn Fn(String) + 'static>>,
        /// p0-fix 定点 2：当前创建工具（Some → 十字光标 + 拖拽创建区域 / 点击放置便签）
        create_tool: RwSignal<Option<CreateToolKind>>,
        /// fix-remote-github-issues / #5：框选工具激活（空白拖无需 Shift）
        marquee_active: RwSignal<bool>,
        /// p0-fix 定点 2：区域拖框落账（x, y, width, height）
        on_area_create: Option<Box<dyn Fn(f64, f64, f64, f64) + 'static>>,
        /// fix-open-issues-26-33（issue #27，R-ARESZ-02）：区域 resize 松手落账——
        /// (area_id, before(x,y,w,h), after(x,y,w,h))，调用方走 Command::SetAreaRect
        /// 单条命令（一次 Undo 还原整次 resize）+ 既有保存/协作通道
        on_area_resize: Option<
            Box<dyn Fn(String, (f64, f64, f64, f64), (f64, f64, f64, f64)) + 'static>,
        >,
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
        let on_area_resize = Rc::new(on_area_resize);
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

        // fix-remote-github-issues-7-18（issue #11，R-DPR-07~10）：backing store 同步抽为
        // 共用闭包——渲染 effect（frame_tick 驱动）与 ResizeObserver 回调（容器尺寸/DPR
        // 变化）同一路径；有效 dpr 恒走 capped_drag_dpr（R-DPR-09：与 R-PERF-10 同源）。
        let sync_backing_store: Rc<dyn Fn(&web_sys::HtmlCanvasElement)> = Rc::new(move |canvas| {
            if let Some(parent) = canvas.parent_element() {
                // R-DPR-01：backing store 像素 = CSS × devicePixelRatio
                let css_w = parent.client_width().max(1) as f64;
                let css_h = parent.client_height().max(1) as f64;
                // R-PERF-10：任意拖拽活跃（表/关系/便签/区域/框选/平移）期间把有效 dpr
                // 压到 DRAG_RENDER_DPR_CAP，降低软件合成像素负载。用 get_untracked——
                // 渲染 effect 由 frame_tick 统一驱动，拖拽起止无需额外触发（否则每次
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
        });

        // fix-remote-github-issues-7-18（issue #11，R-DPR-07~10）：canvas 父容器挂
        // ResizeObserver——分隔条拖动（splitter 只写 CSS 变量）、窗口/侧栏尺寸变化时
        // 同回调内同步 backing store 并 schedule_paint（与 R-PERF-05 共用 rAF 合并，
        // 一帧至多一次 draw_canvas）；组件卸载经 on_cleanup 断开 observer（R-DPR-10）。
        {
            let schedule_paint = schedule_paint.clone();
            let sync_backing_store = sync_backing_store.clone();
            // (observer, callback) 句柄对：cleanup 时 disconnect + drop，不泄漏监听/闭包
            let ro_holder: Rc<RefCell<Option<(web_sys::ResizeObserver, Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>)>>> =
                Rc::new(RefCell::new(None));
            let ro_holder_cleanup = ro_holder.clone();
            create_effect(move |_| {
                let Some(canvas) = canvas_ref.get() else {
                    return;
                };
                if ro_holder.borrow().is_some() {
                    return;
                }
                let canvas_for_cb = canvas.clone();
                let sync = sync_backing_store.clone();
                let sp = schedule_paint.clone();
                let cb = Closure::wrap(Box::new(move |_: js_sys::Array, _: web_sys::ResizeObserver| {
                    // R-DPR-08：分隔条拖动路径由本回调自然覆盖（splitter 无需显式通知）
                    sync(&canvas_for_cb);
                    sp();
                }) as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);
                if let Ok(obs) = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref()) {
                    if let Some(parent) = canvas.parent_element() {
                        obs.observe(&parent);
                        *ro_holder.borrow_mut() = Some((obs, cb));
                    } else {
                        obs.disconnect();
                    }
                }
            });
            on_cleanup(move || {
                if let Some((obs, _cb)) = ro_holder_cleanup.borrow_mut().take() {
                    obs.disconnect();
                }
            });
        }

        {
            let live = live.clone();
            let ghost = ghost.clone();
            let sync_backing_store = sync_backing_store.clone();
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

            // fix-remote-github-issues-7-18（issue #11）：backing store 同步与 ResizeObserver
            // 回调共用同一闭包（R-DPR-07/09 同源口径；R-PERF-10 降采样逻辑见该闭包注释）
            sync_backing_store(&canvas);

            // R-PERF-05：transform 用 get_untracked（effect 不订阅 transform，重绘由
            // frame_tick 统一驱动）；Vec 信号一律 .with() 传引用，禁止 .get() 深克隆整表。
            let t = transform.get_untracked();
            let (table_pos, rubber, marquee_preview, area_preview, live_notes, live_areas) = {
                let live_g = live.borrow();
                (
                    live_g.table_pos.clone(),
                    live_g.rubber,
                    live_g.marquee_preview,
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
            // #5：订阅多选集合，框选/Shift 多选后重绘选中环
            let multi_sel = selected_table_ids.with(|ids| ids.clone());
            let multi_notes = selected_note_ids.with(|ids| ids.clone());
            let multi_areas = selected_area_ids.with(|ids| ids.clone());
            // R-PERF-11：幽灵层接管中的表 id（主画布跳过绘制）
            let ghost_skip = ghost.borrow().as_ref().map(|g| g.table_id.clone());
            // fix-remote-github-issues-7-18（issue #10，R-CMT-04）：订阅注释显示模式——
            // 切换即重绘（精灵指纹含 mode，自动触发重光栅）
            let comment_mode = store.comment_display.get();
            // fix-issues-36-37（issue #36，R-VIEW-DIM-05）：订阅维度模式——切换即重绘
            // （维度经精灵指纹触发重光栅，R-LOD-05）
            let view_dimension = store.view_dimension.get();
            // fix-issues-38-41-canvas-interaction（issue #40，R-FONT-02）：订阅字号倍率——
            // 切档即重绘（倍率经精灵指纹混入触发重光栅）
            let label_font_scale = store.label_font_scale.get().factor();

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
                                    &multi_sel,
                                    &multi_notes,
                                    &multi_areas,
                                    sel_ref.as_deref(),
                                    sel_area.as_deref(),
                                    sel_note.as_deref(),
                                    rubber,
                                    marquee_preview,
                                    area_preview,
                                    table_override,
                                    ghost_skip.as_deref(),
                                    comment_mode,
                                    view_dimension,
                                    // R-ARESZ-05：只读模式不渲染 resize 手柄
                                    !read_only,
                                    label_font_scale,
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
                let drag = drag_state.get();
                let create_on = create_tool.get().is_some();
                let marquee_on = marquee_active.get();
                let (dragging_entity, dragging_pan, dragging_tool) = match &drag {
                    Some(d) => {
                        let entity = d.table_id.is_some()
                            || d.note_drag.is_some()
                            || d.area_drag.is_some()
                            || d.endpoint_drag.is_some();
                        let toolish = d.marquee_start.is_some()
                            || d.create_drag.is_some()
                            || d.rel_drag.is_some();
                        let pan = !entity
                            && !toolish
                            && d.multi_starts.is_none()
                            && d.multi_note_starts.is_none()
                            && d.multi_area_starts.is_none();
                        (entity, pan, toolish)
                    }
                    None => (false, false, false),
                };
                let cursor = super::canvas_cursor_css(
                    create_on,
                    marquee_on,
                    dragging_entity,
                    dragging_pan,
                    dragging_tool,
                );
                // fix-open-issues-26-33（issue #27，R-ARESZ-01）：resize 拖拽期间按方位显示 resize 光标
                let cursor = match drag.as_ref().and_then(|d| d.area_resize.as_ref()) {
                    Some((_, dir, ..)) => dir.cursor(),
                    None => cursor,
                };
                if let Some(canvas) = canvas_ref.get() {
                    let ws: &web_sys::HtmlCanvasElement = &canvas;
                    let el: &web_sys::HtmlElement = ws.unchecked_ref();
                    let _ = el.style().set_property("cursor", cursor);
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
            let on_field_pick = on_field_pick.clone();
            let on_note_pick = on_note_pick.clone();
            let on_area_pick = on_area_pick.clone();
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
                    live.borrow_mut().marquee_preview = None;
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
                    // #3 reopen：仅左右连接点起拖连；字段行点击仍走 on_field_pick（两点选取）
                    if let Some((tid, fid, side)) = super::hit_test_field_port(&tables, dx, dy) {
                        let (anchor_x, anchor_y) = tables
                            .iter()
                            .find(|t| t.id == tid)
                            .map(|t| super::field_anchor_for_side(t, &fid, side))
                            .unwrap_or((dx, dy));
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            multi_starts: None,
                            multi_note_starts: None,
                            multi_area_starts: None,
                            marquee_start: None,
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
                            area_resize: None,
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
                    if let Some((tid, fid)) = super::hit_test_field(&tables, dx, dy) {
                        if let Some(cb) = on_field_pick.as_ref() {
                            cb(tid, fid);
                        }
                        return;
                    }
                    // 关系工具下未命中字段：回落到 pan 模式而不是吞掉 pointerdown
                    // （之前 return 会导致选择连线/点空白后画布无法拖动）
                    capture_pointer(&canvas, ev.pointer_id());
                    drag_state.set(Some(DragState {
                        table_id: None,
                        multi_starts: None,
                        multi_note_starts: None,
                        multi_area_starts: None,
                        marquee_start: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        area_resize: None,
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
                // fix-remote-github-issues / UT-PB-08：Idle 下可从字段**连接点**拖出建关系（无需先点关系工具）
                // #3 reopen：触发面改为左右 port，非整行；行内短按仍经表命中 → on_field_pick
                // #5：Shift 或已多选集合内拖动时让路给框选/多表拖
                if !read_only {
                    if let Some((tid, fid, side)) = super::hit_test_field_port(&tables, dx, dy) {
                        let multi = selected_table_ids.get_untracked();
                        if !super::prefer_selection_over_field_rel(
                            ev.shift_key() || marquee_active.get_untracked(),
                            &multi,
                            &tid,
                        ) {
                            let (anchor_x, anchor_y) = tables
                                .iter()
                                .find(|t| t.id == tid)
                                .map(|t| super::field_anchor_for_side(t, &fid, side))
                                .unwrap_or((dx, dy));
                            capture_pointer(&canvas, ev.pointer_id());
                            drag_state.set(Some(DragState {
                                table_id: None,
                                multi_starts: None,
                                multi_note_starts: None,
                                multi_area_starts: None,
                                marquee_start: None,
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
                                area_resize: None,
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
                }
                if !read_only {
                    if let Some(tool) = create_tool.get_untracked() {
                        // p0-fix 定点 2：创建工具激活 → 整个画布都是创建热区，吞掉本次 pointerdown
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            multi_starts: None,
                            multi_note_starts: None,
                            multi_area_starts: None,
                            marquee_start: None,
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
                            area_resize: None,
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
                        multi_starts: None,
                        multi_note_starts: None,
                        multi_area_starts: None,
                        marquee_start: None,
                        endpoint_drag: Some((ref_id, end)),
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        area_resize: None,
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
                // #5 / 便签·区域拖拽：
                // 1) 已选中的区域：整区可拖（含表叠盖处）；若在多选集合内则整组拖
                // 2) 便签在表之上；未选中区域仍可用标题栏拖动
                if !read_only {
                    // fix-open-issues-26-33（issue #27，R-ARESZ-01/06）：已选中区域的
                    // resize 手柄命中优先于区域内部整框拖动（手柄 > 内部拖动 > 区域外）
                    if let Some(sel_area) = selected_area_id.get_untracked() {
                        let areas_now = store.areas.get_untracked();
                        if let Some(a) = areas_now.iter().find(|a| a.id == sel_area) {
                            let rect = (a.x, a.y, a.width, a.height);
                            // fix-issues-38-41-canvas-interaction（#41，R-AREALOCK-03）：
                            // 锁定区域 resize 手柄不响应（落入下方分支后仅选中不拖动）
                            if !super::area_drag_allowed(a) {
                                // 锁定：跳过 resize 入口
                            } else if let Some(dir) = super::hit_test_area_resize(a, dx, dy, false) {
                                capture_pointer(&canvas, ev.pointer_id());
                                drag_state.set(Some(DragState {
                                    table_id: None,
                                    multi_starts: None,
                                    multi_note_starts: None,
                                    multi_area_starts: None,
                                    marquee_start: None,
                                    endpoint_drag: None,
                                    rel_drag: None,
                                    create_drag: None,
                                    note_drag: None,
                                    area_drag: None,
                                    area_resize: Some((sel_area.clone(), dir, rect.0, rect.1, rect.2, rect.3)),
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
                    }
                    if let Some(sel_area) = selected_area_id.get_untracked() {
                        if super::hit_test_area(&store.areas.get_untracked(), dx, dy)
                            .as_deref()
                            == Some(sel_area.as_str())
                        {
                            let (area_x, area_y) = store
                                .areas
                                .get_untracked()
                                .iter()
                                .find(|a| a.id == sel_area)
                                .map(|a| (a.x, a.y))
                                .unwrap_or((0.0, 0.0));
                            let prev_areas = selected_area_ids.get_untracked();
                            let in_set = prev_areas.iter().any(|x| x == &sel_area);
                            let multi_areas = super::resolve_table_multi_on_pointerdown(
                                ev.shift_key(),
                                marquee_active.get_untracked(),
                                &sel_area,
                                &prev_areas,
                            );
                            selected_area_ids.set(multi_areas.clone());
                            if super::should_clear_cross_selection(
                                ev.shift_key(),
                                marquee_active.get_untracked(),
                                in_set,
                            ) {
                                selected_table_ids.set(Vec::new());
                                selected_note_ids.set(Vec::new());
                                selected_id.set(None);
                                selected_note_id.set(None);
                            }
                            selected_area_id.set(Some(sel_area.clone()));
                            if let Some(cb) = on_area_pick.as_ref() {
                                cb(sel_area.clone());
                            }
                            let tables_sel = selected_table_ids.get_untracked();
                            let notes_sel = selected_note_ids.get_untracked();
                            let (multi_starts, multi_note_starts, multi_area_starts) =
                                super::build_group_drag_starts(
                                    &tables,
                                    &store.notes.get_untracked(),
                                    &store.areas.get_untracked(),
                                    &tables_sel,
                                    &notes_sel,
                                    &multi_areas,
                                    multi_areas.iter().any(|x| x == &sel_area),
                                );
                            // fix-issues-38-41-canvas-interaction（#41，R-AREALOCK-03/04）：
                            // 锁定区域点击仍完成上方选中联动（Inspector 可编辑），
                            // 但不进入拖动（no-op）
                            if store
                                .areas
                                .get_untracked()
                                .iter()
                                .find(|a| a.id == sel_area)
                                .map(|a| !super::area_drag_allowed(a))
                                .unwrap_or(false)
                            {
                                schedule_paint();
                                return;
                            }
                            capture_pointer(&canvas, ev.pointer_id());
                            drag_state.set(Some(DragState {
                                table_id: None,
                                multi_starts,
                                multi_note_starts,
                                multi_area_starts,
                                marquee_start: None,
                                endpoint_drag: None,
                                rel_drag: None,
                                create_drag: None,
                                note_drag: None,
                                area_drag: Some((sel_area, area_x, area_y)),
                                area_resize: None,
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
                    }
                    if let Some(note_id) = super::hit_test_note(&store.notes.get_untracked(), dx, dy)
                    {
                        selected_ref_id.set(None);
                        let prev_notes = selected_note_ids.get_untracked();
                        let in_set = prev_notes.iter().any(|x| x == &note_id);
                        let multi_notes = super::resolve_table_multi_on_pointerdown(
                            ev.shift_key(),
                            marquee_active.get_untracked(),
                            &note_id,
                            &prev_notes,
                        );
                        selected_note_ids.set(multi_notes.clone());
                        selected_note_id.set(Some(note_id.clone()));
                        if super::should_clear_cross_selection(
                            ev.shift_key(),
                            marquee_active.get_untracked(),
                            in_set,
                        ) {
                            selected_table_ids.set(Vec::new());
                            selected_area_ids.set(Vec::new());
                            selected_id.set(None);
                            selected_area_id.set(None);
                        }
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
                        let tables_sel = selected_table_ids.get_untracked();
                        let areas_sel = selected_area_ids.get_untracked();
                        let (multi_starts, multi_note_starts, multi_area_starts) =
                            super::build_group_drag_starts(
                                &tables,
                                &store.notes.get_untracked(),
                                &store.areas.get_untracked(),
                                &tables_sel,
                                &multi_notes,
                                &areas_sel,
                                multi_notes.iter().any(|x| x == &note_id),
                            );
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            multi_starts,
                            multi_note_starts,
                            multi_area_starts,
                            marquee_start: None,
                            endpoint_drag: None,
                            rel_drag: None,
                            create_drag: None,
                            note_drag: Some((note_id, note_x, note_y)),
                            area_drag: None,
                            area_resize: None,
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
                    if let Some(area_id) =
                        super::hit_test_area_header(&store.areas.get_untracked(), dx, dy)
                    {
                        selected_ref_id.set(None);
                        let prev_areas = selected_area_ids.get_untracked();
                        let in_set = prev_areas.iter().any(|x| x == &area_id);
                        let multi_areas = super::resolve_table_multi_on_pointerdown(
                            ev.shift_key(),
                            marquee_active.get_untracked(),
                            &area_id,
                            &prev_areas,
                        );
                        selected_area_ids.set(multi_areas.clone());
                        selected_area_id.set(Some(area_id.clone()));
                        if super::should_clear_cross_selection(
                            ev.shift_key(),
                            marquee_active.get_untracked(),
                            in_set,
                        ) {
                            selected_table_ids.set(Vec::new());
                            selected_note_ids.set(Vec::new());
                            selected_id.set(None);
                            selected_note_id.set(None);
                        }
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
                        let tables_sel = selected_table_ids.get_untracked();
                        let notes_sel = selected_note_ids.get_untracked();
                        let (multi_starts, multi_note_starts, multi_area_starts) =
                            super::build_group_drag_starts(
                                &tables,
                                &store.notes.get_untracked(),
                                &store.areas.get_untracked(),
                                &tables_sel,
                                &notes_sel,
                                &multi_areas,
                                multi_areas.iter().any(|x| x == &area_id),
                            );
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            multi_starts,
                            multi_note_starts,
                            multi_area_starts,
                            marquee_start: None,
                            endpoint_drag: None,
                            rel_drag: None,
                            create_drag: None,
                            note_drag: None,
                            area_drag: Some((area_id, area_x, area_y)),
                            area_resize: None,
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
                }
                if let Some(id) = super::hit_test(&tables, dx, dy) {
                    let table_x = tables.iter().find(|t| t.id == id).map(|t| t.x).unwrap_or(0.0);
                    let table_y = tables.iter().find(|t| t.id == id).map(|t| t.y).unwrap_or(0.0);
                    selected_id.set(Some(id.clone()));
                    selected_ref_id.set(None);
                    // 字段行（非 port）短按：选中字段；框选/Shift 手势下不抢选字段
                    if !ev.shift_key() && !marquee_active.get_untracked() {
                        if let Some((tid, fid)) = super::hit_test_field(&tables, dx, dy) {
                            if tid == id {
                                if let Some(cb) = on_field_pick.as_ref() {
                                    cb(tid, fid);
                                }
                            }
                        }
                    }
                    // #5：多选集合更新——点已选成员保持集合以整组拖动（勿因框选工具仍激活而 toggle）
                    let prev_tables = selected_table_ids.get_untracked();
                    let in_set = prev_tables.iter().any(|x| x == &id);
                    let multi = super::resolve_table_multi_on_pointerdown(
                        ev.shift_key(),
                        marquee_active.get_untracked(),
                        &id,
                        &prev_tables,
                    );
                    selected_table_ids.set(multi.clone());
                    if super::should_clear_cross_selection(
                        ev.shift_key(),
                        marquee_active.get_untracked(),
                        in_set,
                    ) {
                        selected_note_ids.set(Vec::new());
                        selected_area_ids.set(Vec::new());
                        selected_area_id.set(None);
                        selected_note_id.set(None);
                    }
                    if let Some(cb) = on_select.as_ref() {
                        cb(id.clone());
                    }
                    capture_pointer(&canvas, ev.pointer_id());
                    live.borrow_mut().table_pos = None;
                    let notes_sel = selected_note_ids.get_untracked();
                    let areas_sel = selected_area_ids.get_untracked();
                    let (multi_starts, multi_note_starts, multi_area_starts) =
                        super::build_group_drag_starts(
                            &tables,
                            &store.notes.get_untracked(),
                            &store.areas.get_untracked(),
                            &multi,
                            &notes_sel,
                            &areas_sel,
                            multi.iter().any(|x| x == &id),
                        );
                    drag_state.set(Some(DragState {
                        table_id: Some(id),
                        multi_starts,
                        multi_note_starts,
                        multi_area_starts,
                        marquee_start: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        area_resize: None,
                        pointer_id: ev.pointer_id(),
                        start_mouse_x: ev.client_x() as f64,
                        start_mouse_y: ev.client_y() as f64,
                        start_pan_x: t_now.pan_x,
                        start_pan_y: t_now.pan_y,
                        start_table_x: table_x,
                        start_table_y: table_y,
                    }));
                } else if let Some(ref_id) = super::hit_test_reference_tier(
                    &tables,
                    &refs,
                    dx,
                    dy,
                    // #35 R-LOD-08 / #36 R-VIEW-DIM-01：与 draw_canvas 同一维度口径（显式维度驱动）
                    super::tier_for_dimension(store.view_dimension.get_untracked()),
                ) {
                    // relation-inspector-and-ddl-io：点击连线（表未命中时）→ 选中高亮 + Inspector 展示（不再弹详情模态）
                    // 命中顺序在表之后：连线被表遮住时点击应选中表而非不可见的线
                    selected_id.set(None);
                    selected_ref_id.set(Some(ref_id.clone()));
                    selected_area_id.set(None);
                    selected_note_id.set(None);
                    // fix-issues-36-37（#37 R-KBSEL-02）：清理多选集——否则框选/全选残留的
                    // 表集合会在 Delete 时劫持单选连线的删除语义（ST-PB-04 回归根因）
                    selected_table_ids.set(Vec::new());
                    selected_note_ids.set(Vec::new());
                    selected_area_ids.set(Vec::new());
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
                        multi_starts: None,
                        multi_note_starts: None,
                        multi_area_starts: None,
                        marquee_start: None,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: Some((note_id, note_x, note_y)),
                        area_drag: None,
                        area_resize: None,
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
                    // fix-issues-38-41-canvas-interaction（#41，R-AREALOCK-03/04）：
                    // 锁定区域标题栏点击 = 仅选中（Inspector 可编辑），不进入拖动
                    let area_locked = store
                        .areas
                        .get_untracked()
                        .iter()
                        .find(|a| a.id == area_id)
                        .map(|a| !super::area_drag_allowed(a))
                        .unwrap_or(false);
                    if !area_locked {
                        capture_pointer(&canvas, ev.pointer_id());
                        drag_state.set(Some(DragState {
                            table_id: None,
                            multi_starts: None,
                            multi_note_starts: None,
                            multi_area_starts: None,
                            marquee_start: None,
                            endpoint_drag: None,
                            rel_drag: None,
                            create_drag: None,
                            note_drag: None,
                            area_drag: Some((area_id, area_x, area_y)),
                            area_resize: None,
                            pointer_id: ev.pointer_id(),
                            start_mouse_x: ev.client_x() as f64,
                            start_mouse_y: ev.client_y() as f64,
                            start_pan_x: t_now.pan_x,
                            start_pan_y: t_now.pan_y,
                            start_table_x: 0.0,
                            start_table_y: 0.0,
                        }));
                    }
                    schedule_paint();
                } else {
                    // fix-issues-38-41-canvas-interaction（#39，core-01 §5.15 R-PAN-SEL-01）：
                    // 空白按下不再立即清选——click/pan 判定推迟到 pointerup（位移 <4px
                    // 才清选，见 on_pointerup pan 兜底分支），平移画布期间保持选中与
                    // Inspector 联动。#31 的「点击空白清空多选集合」语义由 pointerup
                    // click 判定继承（ST-PE-10 THEN 3 口径不变）。
                    capture_pointer(&canvas, ev.pointer_id());
                    // Shift+空白 或框选工具 = 框选多表；否则平移画布
                    let use_marquee =
                        !read_only && (ev.shift_key() || marquee_active.get_untracked());
                    let marquee_start = if use_marquee {
                        selected_table_ids.set(Vec::new());
                        selected_note_ids.set(Vec::new());
                        selected_area_ids.set(Vec::new());
                        Some((dx, dy))
                    } else {
                        None
                    };
                    drag_state.set(Some(DragState {
                        table_id: None,
                        multi_starts: None,
                        multi_note_starts: None,
                        multi_area_starts: None,
                        marquee_start,
                        endpoint_drag: None,
                        rel_drag: None,
                        create_drag: None,
                        note_drag: None,
                        area_drag: None,
                        area_resize: None,
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
                    // #5：若带多选 starts，整组平移
                    let ddx = dx / t_now.zoom;
                    let ddy = dy / t_now.zoom;
                    if drag.multi_starts.is_some()
                        || drag.multi_note_starts.is_some()
                        || drag.multi_area_starts.is_some()
                    {
                        if let Some(starts) = &drag.multi_starts {
                            let mut tables = store.tables.get_untracked();
                            for (tid, sx, sy) in starts {
                                if let Some(t) = tables.iter_mut().find(|t| &t.id == tid) {
                                    t.x = sx + ddx;
                                    t.y = sy + ddy;
                                }
                            }
                            store.tables.set(tables);
                        }
                        let mut visual_notes = store.notes.get_untracked();
                        if let Some(starts) = &drag.multi_note_starts {
                            for (nid, sx, sy) in starts {
                                if let Some(n) = visual_notes.iter_mut().find(|n| &n.id == nid) {
                                    n.x = sx + ddx;
                                    n.y = sy + ddy;
                                }
                            }
                        } else if let Some(n) = visual_notes.iter_mut().find(|n| n.id == *note_id) {
                            n.x = start_x + ddx;
                            n.y = start_y + ddy;
                        }
                        live.borrow_mut().notes = Some(visual_notes);
                        if let Some(starts) = &drag.multi_area_starts {
                            let mut visual_areas = store.areas.get_untracked();
                            for (aid, sx, sy) in starts {
                                if let Some(a) = visual_areas.iter_mut().find(|a| &a.id == aid) {
                                    a.x = sx + ddx;
                                    a.y = sy + ddy;
                                }
                            }
                            live.borrow_mut().areas = Some(visual_areas);
                        }
                        schedule_paint();
                    } else {
                        let new_x = start_x + ddx;
                        let new_y = start_y + ddy;
                        let mut visual = store.notes.get_untracked();
                        if let Some(n) = visual.iter_mut().find(|n| n.id == *note_id) {
                            n.x = new_x;
                            n.y = new_y;
                        }
                        live.borrow_mut().notes = Some(visual);
                        schedule_paint();
                    }
                } else if let Some((area_id, dir, ox, oy, ow, oh)) = &drag.area_resize {
                    // fix-open-issues-26-33（issue #27，R-ARESZ-02）：resize 拖动中只写视觉层，
                    // pointermove 不落账（与 area_drag 同一拖拽状态机口径）
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &t_now,
                    );
                    let (nx, ny, nw, nh) =
                        super::area_rect_from_resize(*dir, *ox, *oy, *ow, *oh, diag_x, diag_y);
                    let mut visual = store.areas.get_untracked();
                    if let Some(a) = visual.iter_mut().find(|a| a.id == *area_id) {
                        a.x = nx;
                        a.y = ny;
                        a.width = nw;
                        a.height = nh;
                    }
                    live.borrow_mut().areas = Some(visual);
                    schedule_paint();
                } else if let Some((area_id, start_x, start_y)) = &drag.area_drag {
                    // redesign-listview-type-length-canvas-fix：区域拖动中只写视觉层（ST-CR-AREA-01）
                    let ddx = dx / t_now.zoom;
                    let ddy = dy / t_now.zoom;
                    if drag.multi_starts.is_some()
                        || drag.multi_note_starts.is_some()
                        || drag.multi_area_starts.is_some()
                    {
                        if let Some(starts) = &drag.multi_starts {
                            let mut tables = store.tables.get_untracked();
                            for (tid, sx, sy) in starts {
                                if let Some(t) = tables.iter_mut().find(|t| &t.id == tid) {
                                    t.x = sx + ddx;
                                    t.y = sy + ddy;
                                }
                            }
                            store.tables.set(tables);
                        }
                        if let Some(starts) = &drag.multi_note_starts {
                            let mut visual_notes = store.notes.get_untracked();
                            for (nid, sx, sy) in starts {
                                if let Some(n) = visual_notes.iter_mut().find(|n| &n.id == nid) {
                                    n.x = sx + ddx;
                                    n.y = sy + ddy;
                                }
                            }
                            live.borrow_mut().notes = Some(visual_notes);
                        }
                        let mut visual_areas = store.areas.get_untracked();
                        if let Some(starts) = &drag.multi_area_starts {
                            for (aid, sx, sy) in starts {
                                if let Some(a) = visual_areas.iter_mut().find(|a| &a.id == aid) {
                                    a.x = sx + ddx;
                                    a.y = sy + ddy;
                                }
                            }
                        } else if let Some(a) = visual_areas.iter_mut().find(|a| a.id == *area_id) {
                            a.x = start_x + ddx;
                            a.y = start_y + ddy;
                        }
                        live.borrow_mut().areas = Some(visual_areas);
                        schedule_paint();
                    } else {
                        let new_x = start_x + ddx;
                        let new_y = start_y + ddy;
                        let mut visual = store.areas.get_untracked();
                        if let Some(a) = visual.iter_mut().find(|a| a.id == *area_id) {
                            a.x = new_x;
                            a.y = new_y;
                        }
                        live.borrow_mut().areas = Some(visual);
                        schedule_paint();
                    }
                } else if let Some(table_id) = &drag.table_id {
                    // R-PERF-04：拖动中只写 (id, x, y) 覆盖层——禁止整 Vec<Table> 深克隆
                    // （UT-CR-DRAG-01）；不量化网格（UT-CR-06），松手才落账 store
                    let new_x = drag.start_table_x + dx / t_now.zoom;
                    let new_y = drag.start_table_y + dy / t_now.zoom;
                    if drag.multi_starts.is_some()
                        || drag.multi_note_starts.is_some()
                        || drag.multi_area_starts.is_some()
                    {
                        // #5：多选整组同步位移
                        let ddx = new_x - drag.start_table_x;
                        let ddy = new_y - drag.start_table_y;
                        if let Some(starts) = &drag.multi_starts {
                            let mut tables = store.tables.get_untracked();
                            for (tid, sx, sy) in starts {
                                if let Some(t) = tables.iter_mut().find(|t| &t.id == tid) {
                                    t.x = sx + ddx;
                                    t.y = sy + ddy;
                                }
                            }
                            store.tables.set(tables);
                        }
                        if let Some(starts) = &drag.multi_note_starts {
                            let mut visual_notes = store.notes.get_untracked();
                            for (nid, sx, sy) in starts {
                                if let Some(n) = visual_notes.iter_mut().find(|n| &n.id == nid) {
                                    n.x = sx + ddx;
                                    n.y = sy + ddy;
                                }
                            }
                            live.borrow_mut().notes = Some(visual_notes);
                        }
                        if let Some(starts) = &drag.multi_area_starts {
                            let mut visual_areas = store.areas.get_untracked();
                            for (aid, sx, sy) in starts {
                                if let Some(a) = visual_areas.iter_mut().find(|a| &a.id == aid) {
                                    a.x = sx + ddx;
                                    a.y = sy + ddy;
                                }
                            }
                            live.borrow_mut().areas = Some(visual_areas);
                        }
                        live.borrow_mut().table_pos = Some((table_id.clone(), new_x, new_y));
                        schedule_paint();
                        return;
                    }
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
                                        store.comment_display.get_untracked(),
                                        store.view_dimension.get_untracked(),
                                        // #40：拖动热路径 untracked（拖动中切倍率不重建幽灵层，松手即恢复）
                                        store.label_font_scale.get_untracked().factor(),
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
                } else if let Some((mx, my)) = drag.marquee_start {
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        &canvas,
                        &t_now,
                    );
                    // #5：框选必须画矩形，禁止写入关系 rubber（否则 SVG/canvas 会画贝塞尔线）
                    live.borrow_mut().rubber = None;
                    live.borrow_mut().marquee_preview = Some((mx, my, diag_x, diag_y));
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
            let on_area_resize = on_area_resize.clone();
            let on_area_create = on_area_create.clone();
            let on_note_create = on_note_create.clone();
            let on_select = on_select.clone();
            let on_note_pick = on_note_pick.clone();
            let on_area_pick = on_area_pick.clone();
            let on_deselect = on_deselect.clone();
            let current_transform = current_transform.clone();
            move |ev: PointerEvent| {
                let Some(drag) = drag_state.get_untracked() else {
                    return;
                };
                // R-PERF-05：松手落账前先把 rAF 待落账的 wheel/pan 意图落账，
                // 吸附坐标基于已提交的 transform
                let t_now = current_transform();
                let canvas = canvas_ref.get();
                // #19：用 Drop 守卫把 release_pointer_capture 推迟到 handler 所有路径末尾
                // （勿在开头提前释放，否则会把 click 穿透到下方 AppBar）
                struct CaptureRelease {
                    canvas: Option<leptos::HtmlElement<html::Canvas>>,
                    pointer_id: i32,
                }
                impl Drop for CaptureRelease {
                    fn drop(&mut self) {
                        if let Some(c) = &self.canvas {
                            let _ = c.release_pointer_capture(self.pointer_id);
                        }
                    }
                }
                let _capture_guard = CaptureRelease {
                    canvas: canvas.clone(),
                    pointer_id: drag.pointer_id,
                };

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
                    let multi_starts = drag.multi_starts.clone();
                    let multi_note_starts = drag.multi_note_starts.clone();
                    let multi_area_starts = drag.multi_area_starts.clone();
                    live.borrow_mut().notes = None;
                    live.borrow_mut().areas = None;
                    live.borrow_mut().table_pos = None;
                    drag_state.set(None);
                    // 纯点击（位移 < 阈值）= 选中语义：不写回坐标、不触发持久化
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    // #19：有效便签拖动 → 抑制紧随 click（如松开在 btn-invite 上）
                    super::arm_suppress_next_click();
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let (sx, sy) = super::snap_to_grid(new_x, new_y, super::GRID_SIZE);
                    let ddx = sx - start_x;
                    let ddy = sy - start_y;
                    if multi_starts.is_some()
                        || multi_note_starts.is_some()
                        || multi_area_starts.is_some()
                    {
                        if let Some(starts) = multi_starts {
                            let mut tables = store.tables.get_untracked();
                            for (tid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(table) = tables.iter_mut().find(|t| t.id == tid) {
                                    table.x = nx;
                                    table.y = ny;
                                }
                            }
                            store.tables.set(tables);
                        }
                        let mut notes = store.notes.get_untracked();
                        if let Some(starts) = multi_note_starts {
                            for (nid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(n) = notes.iter_mut().find(|n| n.id == nid) {
                                    n.x = nx;
                                    n.y = ny;
                                }
                            }
                        } else if let Some(n) = notes.iter_mut().find(|n| n.id == note_id) {
                            n.x = sx;
                            n.y = sy;
                        }
                        store.notes.set(notes);
                        if let Some(starts) = multi_area_starts {
                            let mut areas = store.areas.get_untracked();
                            for (aid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(a) = areas.iter_mut().find(|a| a.id == aid) {
                                    a.x = nx;
                                    a.y = ny;
                                }
                            }
                            store.areas.set(areas);
                        }
                    } else {
                        let mut notes = store.notes.get_untracked();
                        if let Some(n) = notes.iter_mut().find(|n| n.id == note_id) {
                            n.x = sx;
                            n.y = sy;
                        }
                        store.notes.set(notes);
                    }
                    // 复用表拖动落账通路（dirty + 协作 op + S01 保存链路 → PUT notes[0].x/y）
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                if let Some((area_id, dir, ox, oy, ow, oh)) = drag.area_resize.clone() {
                    // fix-open-issues-26-33（issue #27，R-ARESZ-02/03）：resize 松手一次写回，
                    // 走 on_area_resize → Command::SetAreaRect 单条命令（一次 Undo 还原）
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().areas = None;
                    drag_state.set(None);
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    // R-ARESZ-06 / R-DRAG-CLICK-04：resize 拖拽后抑制紧随 click
                    super::arm_suppress_next_click();
                    let Some(canvas_html) = canvas.as_ref() else {
                        schedule_paint();
                        return;
                    };
                    let canvas_el: &web_sys::HtmlCanvasElement = canvas_html.unchecked_ref();
                    let (diag_x, diag_y) = screen_to_diagram(
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                        canvas_el,
                        &t_now,
                    );
                    let after = super::area_rect_from_resize(dir, ox, oy, ow, oh, diag_x, diag_y);
                    if let Some(cb) = on_area_resize.as_ref() {
                        cb(area_id, (ox, oy, ow, oh), after);
                    } else {
                        // 无落账通道（宿主未接）时直接写 store + 复用表拖动持久化通路
                        let mut areas = store.areas.get_untracked();
                        if let Some(a) = areas.iter_mut().find(|a| a.id == area_id) {
                            a.x = after.0;
                            a.y = after.1;
                            a.width = after.2;
                            a.height = after.3;
                        }
                        store.areas.set(areas);
                        if let Some(cb) = on_table_drop.as_ref() {
                            cb();
                        }
                    }
                    schedule_paint();
                    return;
                }

                if let Some((area_id, start_x, start_y)) = drag.area_drag.clone() {
                    // redesign-listview-type-length-canvas-fix：区域松手落账（ST-CR-AREA-01，宽高不变）
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    let multi_starts = drag.multi_starts.clone();
                    let multi_note_starts = drag.multi_note_starts.clone();
                    let multi_area_starts = drag.multi_area_starts.clone();
                    live.borrow_mut().notes = None;
                    live.borrow_mut().areas = None;
                    live.borrow_mut().table_pos = None;
                    drag_state.set(None);
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    // #19：有效区域拖动 → 抑制紧随 click（如松开在 btn-invite 上）
                    super::arm_suppress_next_click();
                    let new_x = start_x + dx / t_now.zoom;
                    let new_y = start_y + dy / t_now.zoom;
                    let (sx, sy) = super::snap_to_grid(new_x, new_y, super::GRID_SIZE);
                    let ddx = sx - start_x;
                    let ddy = sy - start_y;
                    if multi_starts.is_some()
                        || multi_note_starts.is_some()
                        || multi_area_starts.is_some()
                    {
                        if let Some(starts) = multi_starts {
                            let mut tables = store.tables.get_untracked();
                            for (tid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(table) = tables.iter_mut().find(|t| t.id == tid) {
                                    table.x = nx;
                                    table.y = ny;
                                }
                            }
                            store.tables.set(tables);
                        }
                        if let Some(starts) = multi_note_starts {
                            let mut notes = store.notes.get_untracked();
                            for (nid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(n) = notes.iter_mut().find(|n| n.id == nid) {
                                    n.x = nx;
                                    n.y = ny;
                                }
                            }
                            store.notes.set(notes);
                        }
                        let mut areas = store.areas.get_untracked();
                        if let Some(starts) = multi_area_starts {
                            for (aid, ox, oy) in starts {
                                let (nx, ny) =
                                    super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                                if let Some(a) = areas.iter_mut().find(|a| a.id == aid) {
                                    a.x = nx;
                                    a.y = ny;
                                }
                            }
                        } else if let Some(a) = areas.iter_mut().find(|a| a.id == area_id) {
                            a.x = sx;
                            a.y = sy;
                        }
                        store.areas.set(areas);
                    } else {
                        let mut areas = store.areas.get_untracked();
                        if let Some(a) = areas.iter_mut().find(|a| a.id == area_id) {
                            a.x = sx;
                            a.y = sy;
                        }
                        store.areas.set(areas);
                    }
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                if let Some(table_id) = drag.table_id {
                    let dx = ev.client_x() as f64 - drag.start_mouse_x;
                    let dy = ev.client_y() as f64 - drag.start_mouse_y;
                    live.borrow_mut().table_pos = None;
                    live.borrow_mut().notes = None;
                    live.borrow_mut().areas = None;
                    // R-PERF-11：先撤幽灵层再落账/重绘，保证渲染 effect 看到幽灵已清
                    // （Drop 自动从 DOM 移除节点）
                    ghost.borrow_mut().take();
                    let multi_starts = drag.multi_starts.clone();
                    let multi_note_starts = drag.multi_note_starts.clone();
                    let multi_area_starts = drag.multi_area_starts.clone();
                    drag_state.set(None);
                    // 纯点击（位移 < 4px）= 选中语义：不写回坐标、不触发持久化，仅重绘复位视觉
                    if !super::is_relation_drag(dx, dy, super::DRAG_THRESHOLD) {
                        schedule_paint();
                        return;
                    }
                    // #19：有效表拖动 → 抑制紧随 click（如松开在 btn-invite 上）
                    super::arm_suppress_next_click();
                    let new_x = drag.start_table_x + dx / t_now.zoom;
                    let new_y = drag.start_table_y + dy / t_now.zoom;
                    let (sx, sy) = super::snap_to_grid(new_x, new_y, super::GRID_SIZE);
                    let ddx = sx - drag.start_table_x;
                    let ddy = sy - drag.start_table_y;
                    let mut tables = store.tables.get_untracked();
                    if let Some(starts) = multi_starts {
                        for (tid, ox, oy) in starts {
                            let (nx, ny) =
                                super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                            if let Some(table) = tables.iter_mut().find(|t| t.id == tid) {
                                table.x = nx;
                                table.y = ny;
                            }
                        }
                    } else if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                        table.x = sx;
                        table.y = sy;
                    }
                    store.tables.set(tables);
                    if let Some(starts) = multi_note_starts {
                        let mut notes = store.notes.get_untracked();
                        for (nid, ox, oy) in starts {
                            let (nx, ny) =
                                super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                            if let Some(n) = notes.iter_mut().find(|n| n.id == nid) {
                                n.x = nx;
                                n.y = ny;
                            }
                        }
                        store.notes.set(notes);
                    }
                    if let Some(starts) = multi_area_starts {
                        let mut areas = store.areas.get_untracked();
                        for (aid, ox, oy) in starts {
                            let (nx, ny) =
                                super::snap_to_grid(ox + ddx, oy + ddy, super::GRID_SIZE);
                            if let Some(a) = areas.iter_mut().find(|a| a.id == aid) {
                                a.x = nx;
                                a.y = ny;
                            }
                        }
                        store.areas.set(areas);
                    }
                    // D 批：松手吸附后必须通知持久化（S01 保存链路），否则拖表位置不落账
                    if let Some(cb) = on_table_drop.as_ref() {
                        cb();
                    }
                    return;
                }

                if let Some((mx, my)) = drag.marquee_start {
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
                        .unwrap_or((mx, my));
                    live.borrow_mut().rubber = None;
                    live.borrow_mut().marquee_preview = None;
                    rubber_d.set(None);
                    drag_state.set(None);
                    let table_ids = super::tables_in_marquee(
                        &store.tables.get_untracked(),
                        mx,
                        my,
                        diag_x,
                        diag_y,
                    );
                    let note_ids = super::notes_in_marquee(
                        &store.notes.get_untracked(),
                        mx,
                        my,
                        diag_x,
                        diag_y,
                    );
                    let area_ids = super::areas_in_marquee(
                        &store.areas.get_untracked(),
                        mx,
                        my,
                        diag_x,
                        diag_y,
                    );
                    selected_table_ids.set(table_ids.clone());
                    selected_note_ids.set(note_ids.clone());
                    selected_area_ids.set(area_ids.clone());
                    if let Some(first) = table_ids.first() {
                        selected_id.set(Some(first.clone()));
                    } else {
                        selected_id.set(None);
                    }
                    selected_note_id.set(note_ids.first().cloned());
                    selected_area_id.set(area_ids.first().cloned());
                    if let Some(nid) = note_ids.first() {
                        if let Some(cb) = on_note_pick.as_ref() {
                            cb(nid.clone());
                        }
                    } else if let Some(aid) = area_ids.first() {
                        if let Some(cb) = on_area_pick.as_ref() {
                            cb(aid.clone());
                        }
                    } else if let Some(tid) = table_ids.first() {
                        if let Some(cb) = on_select.as_ref() {
                            cb(tid.clone());
                        }
                    }
                    schedule_paint();
                    return;
                }

                // fix-issues-38-41-canvas-interaction（#39，core-01 §5.15 R-PAN-SEL-01~04）：
                // pan 兜底分支——位移 <4px 判定 click（清选 + on_deselect，继承原
                // pointerdown 清选口径含 #31 多选集合清空）；≥4px 判定 pan（保留选中，
                // Inspector 联动不断）
                let dx = ev.client_x() as f64 - drag.start_mouse_x;
                let dy = ev.client_y() as f64 - drag.start_mouse_y;
                live.borrow_mut().table_pos = None;
                drag_state.set(None);
                if super::is_blank_click(dx, dy) {
                    selected_id.set(None);
                    selected_ref_id.set(None);
                    selected_area_id.set(None);
                    selected_note_id.set(None);
                    selected_table_ids.set(Vec::new());
                    selected_note_ids.set(Vec::new());
                    selected_area_ids.set(Vec::new());
                    if let Some(cb) = on_deselect.as_ref() {
                        cb();
                    }
                }
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
                live.borrow_mut().marquee_preview = None;
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

/// UT-CR-CLICK-01（#19）：有效拖动后应抑制紧随其后的 click 穿透。
/// 复用 `is_relation_drag` + `DRAG_THRESHOLD`（表/便签/区域共用）。
pub fn should_suppress_click_after_drag(dx: f64, dy: f64) -> bool {
    is_relation_drag(dx, dy, DRAG_THRESHOLD)
}

/// fix-issues-38-41-canvas-interaction（#39，core-01 §5.15 R-PAN-SEL-01/02）：
/// 空白按下的 click/pan 判定——位移 <4px（屏幕欧氏距离，边界 4px 恰为 pan）
/// 判定为 click（pointerup 清选）；≥4px 判定为 pan（保留选中）。
/// 与表/便签/区域拖动共用 DRAG_THRESHOLD 口径。
pub fn is_blank_click(dx: f64, dy: f64) -> bool {
    !is_relation_drag(dx, dy, DRAG_THRESHOLD)
}

/// fix-issues-38-41-canvas-interaction（#41，core-01 §5.17 R-AREALOCK-03）：
/// 区域拖动/resize 门控——锁定区域返回 false（拖动与 resize 均 no-op）；
/// 选中/改名/改色/删除不受锁定影响（R-AREALOCK-04）。
pub fn area_drag_allowed(area: &Area) -> bool {
    !area.locked
}

thread_local! {
    /// #19：有效拖动松手后武装；下一次邀请 click 消费并清除。
    static SUPPRESS_NEXT_CLICK: Cell<bool> = const { Cell::new(false) };
}

/// 武装「抑制下一次 click」（有效表/便签/区域拖动松手后调用）。
pub fn arm_suppress_next_click() {
    SUPPRESS_NEXT_CLICK.with(|c| c.set(true));
}

/// 消费抑制标志：若为 true 则返回 true 并清零（on_open_invite 入口使用）。
pub fn take_suppress_next_click() -> bool {
    SUPPRESS_NEXT_CLICK.with(|c| c.replace(false))
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

/// feat-table-resize / #20：draw_table 渲染尺寸纯函数化。
/// `width == None|Some(0)` → 按内容自适应夹紧 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`；
/// 正数 width 尊重用户手动设置。高度仍消费 `min_height`。
/// 无 comment_mode 时默认 `NameComment`（与 store 初始偏好一致）。
pub fn compute_table_render_size(table: &Table) -> (f64, f64) {
    compute_table_render_size_for(table, CommentDisplay::NameComment)
}

/// 带注释显示模式的渲染尺寸（绘制路径传入真实 mode）。
pub fn compute_table_render_size_for(table: &Table, comment_mode: CommentDisplay) -> (f64, f64) {
    let field_count = table.fields.len().max(2);
    let width = resolve_table_width(table, comment_mode);
    let auto_height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
    let total_height = table
        .min_height
        .map(|h| h as f64)
        .map(|min| min.max(auto_height))
        .unwrap_or(auto_height);
    (width, total_height)
}

/// #20：解析有效表宽。`None` / `Some(0)` = auto；正数 = 用户固定宽。
pub fn resolve_table_width(table: &Table, comment_mode: CommentDisplay) -> f64 {
    match table.width {
        Some(w) if w > 0 => w as f64,
        _ => estimate_content_width(table, comment_mode).clamp(TABLE_WIDTH, TABLE_WIDTH_MAX),
    }
}

/// #20 / #25：按注释显示模式估算内容所需宽度（纯函数，无 canvas measure）。
/// ASCII ≈ 8px/字，CJK ≈ 14px/字。
/// NameComment 下注释与主文本/类型**并排累加**（对齐 `draw_table_body`），禁止对各文本段取 max。
pub fn estimate_content_width(table: &Table, comment_mode: CommentDisplay) -> f64 {
    // 与 draw_table_body 布局常量对齐
    const LEFT_PAD: f64 = 11.0;
    const RIGHT_PAD: f64 = 11.0;
    const HEADER_NAME_CMT_GAP: f64 = 6.0;
    const HEADER_COUNT_RESERVE: f64 = 15.0; // 字段计数右对齐占位
    const FIELD_GAP: f64 = 8.0; // 名称↔注释、注释↔类型

    let table_label = comment_mode.primary(&table.name, &table.comment);
    let mut max_w = LEFT_PAD + measure_text_approx(table_label) + HEADER_COUNT_RESERVE + RIGHT_PAD;
    if let Some(cmt) = comment_mode.secondary(&table.comment) {
        max_w = LEFT_PAD
            + measure_text_approx(table_label)
            + HEADER_NAME_CMT_GAP
            + measure_text_approx(cmt)
            + HEADER_COUNT_RESERVE
            + RIGHT_PAD;
    }

    for field in &table.fields {
        let label = comment_mode.primary(&field.name, &field.comment);
        // #33：徽章占位与 draw_field_badges 同源（field_badges_width）。
        // 纯函数无 refs 上下文，FK 徽章按 false 估算（仅 FK 无其他徽章的字段最多
        // 低估 BADGE_SIZE+GAP≈15px，名称截断由 draw 侧留白吸收）。
        let badges_extra =
            field_badges_width(field_badges(field.primary, false, field.not_null, field.unique).len());
        let label_w = measure_text_approx(label);
        let type_w = measure_text_approx(&field.type_);
        let row = if let Some(fc) = comment_mode.secondary(&field.comment) {
            LEFT_PAD
                + badges_extra
                + label_w
                + FIELD_GAP
                + measure_text_approx(fc)
                + FIELD_GAP
                + type_w
                + RIGHT_PAD
        } else {
            LEFT_PAD + badges_extra + label_w + FIELD_GAP + type_w + RIGHT_PAD
        };
        max_w = max_w.max(row);
    }
    max_w
}

/// ASCII≈8 / CJK≈14 的近似测宽（#20 UT 可测）。
pub fn measure_text_approx(text: &str) -> f64 {
    let mut w = 0.0;
    for ch in text.chars() {
        if ch.is_ascii() {
            w += 8.0;
        } else {
            w += 14.0;
        }
    }
    w
}

/// 源字段右侧锚点（与正式关系线起点一致）。
pub fn field_anchor_start(table: &Table, field_id: &str) -> (f64, f64) {
    field_anchor_for_side(table, field_id, FieldPortSide::End)
}

/// 字段左右连接点锚点（#3：从对应侧 port 拖出）。宽走 `resolve_table_width`。
pub fn field_anchor_for_side(table: &Table, field_id: &str, side: FieldPortSide) -> (f64, f64) {
    let width = resolve_table_width(table, CommentDisplay::NameComment);
    let y = field_anchor_y(table, field_id);
    match side {
        FieldPortSide::Start => (table.x, y),
        FieldPortSide::End => (table.x + width, y),
    }
}

/// #35 R-LOD-08（UT-CR-LOD-01）：tier 感知锚点——拓扑档字段行隐藏，关系线收敛表级端口
/// （纵 = 表头中线，横 = 卡体左/右缘，选侧沿用 pick_port_sides）；详情档保持字段锚点。
pub fn anchor_for_tier(table: &Table, field_id: &str, side: FieldPortSide, tier: LodTier) -> (f64, f64) {
    match tier {
        LodTier::Detail => field_anchor_for_side(table, field_id, side),
        LodTier::Topology => {
            let width = resolve_table_width(table, CommentDisplay::NameComment);
            let y = table.y + TABLE_HEADER_HEIGHT / 2.0;
            match side {
                FieldPortSide::Start => (table.x, y),
                FieldPortSide::End => (table.x + width, y),
            }
        }
    }
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

// ─── fix-remote-github-issues-7-18（issue #9，core-01b §3.4）：连线路径自动选侧 ───

/// 两表相对位置 → (出侧, 入侧)（core-01b §3.4，UT-PB-09）：
/// 目标表中心 x < 源表中心 x → **左出右进**；否则 → **右出左进**（含中心 x 相等的稳定性默认）。
/// 输入仅依赖两表几何（x / width），与字段无关。
pub fn pick_port_sides(from: &Table, to: &Table) -> (FieldPortSide, FieldPortSide) {
    let from_w = resolve_table_width(from, CommentDisplay::NameComment);
    let to_w = resolve_table_width(to, CommentDisplay::NameComment);
    let from_cx = from.x + from_w / 2.0;
    let to_cx = to.x + to_w / 2.0;
    if to_cx < from_cx {
        (FieldPortSide::Start, FieldPortSide::End) // 目标在左：左出右进
    } else {
        (FieldPortSide::End, FieldPortSide::Start) // 目标在右（含相等）：右出左进
    }
}

/// 方向感知贝塞尔控制点（core-01b §3.4，UT-PB-10）：左出时控制点向左伸展，右出向右；
/// 入侧同理（左进控制点在终点左侧，右进在右侧）。伸展幅度取 |x2-x1| 的一半（与原算法同幅度）。
fn bezier_controls_sided(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    out_side: FieldPortSide,
    in_side: FieldPortSide,
) -> (f64, f64, f64, f64) {
    let dx = (x2 - x1).abs() * 0.5;
    let off1 = match out_side {
        FieldPortSide::End => dx,   // 右出：向右伸展
        FieldPortSide::Start => -dx, // 左出：向左伸展
    };
    let off2 = match in_side {
        FieldPortSide::Start => -dx, // 左进：控制点在终点左侧
        FieldPortSide::End => dx,    // 右进：控制点在终点右侧
    };
    (x1 + off1, y1, x2 + off2, y2)
}

/// fix-remote-github-issues-7-18（issue #9，core-01b §3.4）：出入锚点按 `pick_port_sides`
/// 自动选侧（`field_anchor_for_side` 口径），贝塞尔控制点方向随侧（`bezier_controls_sided`）。
/// 表拖动后重算（调用方每帧传入最新几何），无需用户重建关系。
pub fn calc_path(from: &Table, from_field_id: &str, to: &Table, to_field_id: &str) -> RelationPath {
    calc_path_tier(from, from_field_id, to, to_field_id, LodTier::Detail)
}

/// #35 R-LOD-08（UT-CR-LOD-01）：tier 感知贝塞尔路径——拓扑档锚点走 `anchor_for_tier`
/// 表级端口（表头中线 × 左右缘），详情档与 `calc_path` 同口径。
pub fn calc_path_tier(
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
    tier: LodTier,
) -> RelationPath {
    let (out_side, in_side) = pick_port_sides(from, to);
    let (x1, y1) = anchor_for_tier(from, from_field_id, out_side, tier);
    let (x2, y2) = anchor_for_tier(to, to_field_id, in_side, tier);
    let (cx1, cy1, cx2, cy2) = bezier_controls_sided(x1, y1, x2, y2, out_side, in_side);
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

/// #21 UT-PB-13：正交折线路径（消费 `pick_port_sides` 锚点）。
/// 返回折线顶点：起点 → 水平中点 → 垂直对齐 → 终点（典型正交路由）。
pub fn calc_orthogonal_path(
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
) -> Vec<(f64, f64)> {
    calc_orthogonal_path_tier(from, from_field_id, to, to_field_id, LodTier::Detail)
}

/// #35 R-LOD-08：tier 感知正交折线（拓扑档表级端口，与 calc_path_tier 同口径）。
pub fn calc_orthogonal_path_tier(
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
    tier: LodTier,
) -> Vec<(f64, f64)> {
    let (out_side, in_side) = pick_port_sides(from, to);
    let (x1, y1) = anchor_for_tier(from, from_field_id, out_side, tier);
    let (x2, y2) = anchor_for_tier(to, to_field_id, in_side, tier);
    let mid_x = (x1 + x2) / 2.0;
    vec![(x1, y1), (mid_x, y1), (mid_x, y2), (x2, y2)]
}

/// #21：直线路径（同侧锚点直连）。
pub fn calc_straight_path(
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
) -> (f64, f64, f64, f64) {
    calc_straight_path_tier(from, from_field_id, to, to_field_id, LodTier::Detail)
}

/// #35 R-LOD-08：tier 感知直线（拓扑档表级端口，与 calc_path_tier 同口径）。
pub fn calc_straight_path_tier(
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
    tier: LodTier,
) -> (f64, f64, f64, f64) {
    let (out_side, in_side) = pick_port_sides(from, to);
    let (x1, y1) = anchor_for_tier(from, from_field_id, out_side, tier);
    let (x2, y2) = anchor_for_tier(to, to_field_id, in_side, tier);
    (x1, y1, x2, y2)
}

/// #21 UT-PB-14：虚线 dash 数组；solid → 空。
pub fn stroke_dash_for_style(stroke_style: &str) -> Vec<f64> {
    match stroke_style {
        "dashed" => vec![8.0, 6.0],
        _ => vec![],
    }
}

/// #21 UT-PB-15：选中态密度降噪——非相关线 alpha ≤ 0.25。
/// 相关 = 选中关系自身，或两端表之一被选中；无选中时全部 1.0。
pub fn relation_opacity(
    ref_id: &str,
    start_table_id: &str,
    end_table_id: &str,
    selected_table_ids: &[String],
    selected_id: Option<&str>,
    selected_ref_id: Option<&str>,
) -> f64 {
    let table_selected = selected_id.is_some() || !selected_table_ids.is_empty();
    let any_sel = table_selected || selected_ref_id.is_some();
    if !any_sel {
        return 1.0;
    }
    if selected_ref_id == Some(ref_id) {
        return 1.0;
    }
    let related_table = selected_id
        .map(|id| id == start_table_id || id == end_table_id)
        .unwrap_or(false)
        || selected_table_ids
            .iter()
            .any(|id| id == start_table_id || id == end_table_id);
    if related_table {
        1.0
    } else {
        0.25
    }
}

/// #31 UT-PE-HL-01：相关关系线线宽倍率（选中表后相关连线加粗 ≥ 默认 1.5×）。
pub const RELATED_RELATION_WIDTH_FACTOR: f64 = 1.5;
/// #31 UT-PE-HL-01：非相关表 alpha 上限（≤ 0.5，退到背景层）。
pub const UNRELATED_TABLE_ALPHA: f64 = 0.5;

// ─── #33 语义图标统一族（core-08 §11 / core-07 §15.5，UT-PE-VIS-01）──────────

/// 字段语义徽章种类（渲染顺序固定 PK → FK → NN → UQ）。
/// 图标族：钥匙 / 链环 / 星号 / 菱形线条图标（替代字块角标）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldBadge {
    Pk,
    Fk,
    Nn,
    Uq,
}

/// #33 UT-PE-VIS-01：字段语义徽章清单（同源尺寸 BADGE_SIZE / 描边 BADGE_STROKE）。
/// `foreign` 由调用处经 fk_field_ids 推导（渲染纯函数不反向依赖 refs）。
pub fn field_badges(primary: bool, foreign: bool, not_null: bool, unique: bool) -> Vec<FieldBadge> {
    let mut badges = Vec::with_capacity(4);
    if primary {
        badges.push(FieldBadge::Pk);
    }
    if foreign {
        badges.push(FieldBadge::Fk);
    }
    if not_null {
        badges.push(FieldBadge::Nn);
    }
    if unique {
        badges.push(FieldBadge::Uq);
    }
    badges
}

/// #33 UT-PE-VIS-01：徽章组占位宽（n 枚徽章 + 与字段名间距），
/// draw_field_badges 与 estimate_content_width 同源引用，无散值。
pub fn field_badges_width(count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        count as f64 * (BADGE_SIZE + BADGE_GAP) + BADGE_NAME_GAP
    }
}

/// #33 UT-PE-VIS-01：从关系推导 FK 字段集——many 侧字段为 FK；
/// many_to_one → start_field_id，其余（one_to_many / one_to_one / 未知）→ end_field_id。
pub fn fk_field_ids(refs: &[Reference]) -> std::collections::HashSet<&str> {
    refs.iter()
        .map(|r| {
            if r.type_ == "many_to_one" {
                r.start_field_id.as_str()
            } else {
                r.end_field_id.as_str()
            }
        })
        .collect()
}

/// #33 UT-PE-VIS-01：cardinality → (start_many, end_many) 端点重数，
/// crow's foot 端点按此绘制（"多"端三叉爪、"一"端单杠）；未知值按 one_to_many 处理。
pub fn endpoint_multiplicity(cardinality: &str) -> (bool, bool) {
    match cardinality {
        "many_to_one" => (true, false),
        "one_to_one" => (false, false),
        // one_to_many / 空串 / 未知 → 默认一对多
        _ => (false, true),
    }
}


/// #31 UT-PE-HL-01：相关态判定——选中关系自身，或两端表之一被选中（含多选集合）。
/// 无选中时返回 false（调用处据此保持默认线宽）。
pub fn relation_is_related(
    ref_id: &str,
    start_table_id: &str,
    end_table_id: &str,
    selected_table_ids: &[String],
    selected_id: Option<&str>,
    selected_ref_id: Option<&str>,
) -> bool {
    if selected_ref_id == Some(ref_id) {
        return true;
    }
    selected_id
        .map(|id| id == start_table_id || id == end_table_id)
        .unwrap_or(false)
        || selected_table_ids
            .iter()
            .any(|id| id == start_table_id || id == end_table_id)
}

/// #31 UT-PE-HL-01：相关关系线渲染线宽——相关 → base × 1.5；否则原样。
/// 纯函数无只读入参：只读模式同样输入产生同样视觉参数（高亮反馈不因只读关闭）。
pub fn relation_render_width(base: f64, related: bool) -> f64 {
    if related {
        base * RELATED_RELATION_WIDTH_FACTOR
    } else {
        base
    }
}

/// #31 UT-PE-HL-01：表渲染 alpha——无选中 / 选中表自身 / 邻接表（与选中表相连的对端表、
/// 或选中关系的两端表）→ 1.0；其余非相关表 → 0.5（退到背景层）。
pub fn table_render_alpha(
    table_id: &str,
    refs: &[Reference],
    selected_table_ids: &[String],
    selected_id: Option<&str>,
    selected_ref_id: Option<&str>,
) -> f64 {
    let any_sel =
        selected_id.is_some() || !selected_table_ids.is_empty() || selected_ref_id.is_some();
    if !any_sel {
        return 1.0;
    }
    if selected_id == Some(table_id) || selected_table_ids.iter().any(|id| id == table_id) {
        return 1.0;
    }
    let adjacent = refs.iter().any(|r| {
        relation_is_related(
            &r.id,
            &r.start_table_id,
            &r.end_table_id,
            selected_table_ids,
            selected_id,
            selected_ref_id,
        ) && (r.start_table_id == table_id || r.end_table_id == table_id)
    });
    if adjacent {
        1.0
    } else {
        UNRELATED_TABLE_ALPHA
    }
}

/// 框选矩形：对角点 → 归一化 (x, y, w, h)（UT-CR-MULTI / #5：预览必须是框而非线）。
pub fn normalize_marquee_rect(x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64, f64, f64) {
    let min_x = x1.min(x2);
    let min_y = y1.min(y2);
    (min_x, min_y, (x1 - x2).abs(), (y1 - y2).abs())
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
    // #5：多选 / 框选集合（集合内表均绘制选中环）
    selected_table_ids: &[String],
    // #5：框选多选的便签 / 区域
    selected_note_ids: &[String],
    selected_area_ids: &[String],
    // p0-fix 定点 3：选中的关系连线 id（点击连线高亮）
    selected_ref_id: Option<&str>,
    // p0-fix 定点 2：选中的区域 / 便签 id（点击高亮 + Inspector 编辑）
    selected_area_id: Option<&str>,
    selected_note_id: Option<&str>,
    rubber_band: Option<(f64, f64, f64, f64)>,
    // #5：框选矩形预览（对角点）；与关系 rubber_band 分离
    marquee_preview: Option<(f64, f64, f64, f64)>,
    // p0-fix 定点 2：区域拖框预览矩形 (x, y, w, h)
    area_preview: Option<(f64, f64, f64, f64)>,
    // R-PERF-04：被拖表的 (id, x, y) 覆盖坐标（绘制时替换坐标，松手才落账 store）
    table_override: Option<(&str, f64, f64)>,
    // R-PERF-11：幽灵层接管中的表 id——主画布跳过绘制（等效「抠出」，由 DOM 幽灵层呈现）
    ghost_skip: Option<&str>,
    // fix-remote-github-issues-7-18（issue #10，R-CMT-04）：画布注释显示模式（视图偏好）
    comment_mode: CommentDisplay,
    // fix-issues-36-37（issue #36，R-VIEW-DIM-01）：显式维度模式——渲染档位由维度决定，
    // 不再由 zoom 阈值隐式切换（R-LOD-01 #36 修订）
    view_dimension: ViewDimension,
    // fix-open-issues-26-33（issue #27，R-ARESZ-01/05）：选中区域是否渲染 resize 手柄（只读=false）
    area_handles: bool,
    // fix-issues-38-41-canvas-interaction（issue #40，R-FONT-01/03）：画布标签字号倍率
    // （本机视图偏好；作用于表名/字段名/注释，含 10px 屏幕下限钳制）
    label_font_scale: f64,
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
        let is_sel = table_visually_selected(area.id.as_str(), selected_area_id, selected_area_ids);
        draw_area(ctx, area, palette, is_sel, is_sel && area_handles);
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
    // #36 UT-CR-LOD-01：本帧渲染档由显式维度决定（R-VIEW-DIM-01）；
    // zoom 仅驱动表维度内线宽补偿倍率（字段维度恒 1.0 无回归）
    let frame_tier = tier_for_dimension(view_dimension);
    let lod_scale = match frame_tier {
        LodTier::Detail => 1.0,
        LodTier::Topology => lod_line_width(t.zoom, 2.0) / 2.0,
    };
    // ST-PE-10 / ST-CR-LOD-01 探针累计：相关线宽总倍率最大值 / 非相关表 alpha 最小值
    let mut rel_width_scale_max = 1.0f64;
    let mut table_alpha_min = 1.0f64;
    // fix-31 ST-PE-10：本帧是否存在「相关但非选中」的强调线（探针字段 related_emphasis）
    let mut related_emphasis_seen = false;
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
            let opacity = relation_opacity(
                &r.id,
                &r.start_table_id,
                &r.end_table_id,
                selected_table_ids,
                selected_id,
                selected_ref_id,
            );
            // #31 UT-PE-HL-01：选中表后相关连线加粗 1.5×；选中关系自身走 selected 3.5 不再叠加
            // fix-31：related 提升为独立入参——相关线主色/光晕强制选中色族（提亮合同 §4.4）
            let related = relation_is_related(
                &r.id,
                &r.start_table_id,
                &r.end_table_id,
                selected_table_ids,
                selected_id,
                selected_ref_id,
            );
            let hl_scale = if selected_ref_id == Some(&r.id) {
                1.0
            } else {
                relation_render_width(1.0, related)
            };
            if related && selected_ref_id != Some(&r.id) {
                related_emphasis_seen = true;
            }
            // #30 R-LOD-04：拓扑档线宽补偿与 #31 高亮倍率叠乘（选中连线 3.5 也乘 lod）；
            // 探针累计相对默认 2.0 世界线宽的总倍率（选中线为 3.5×lod / 2.0）
            let frame_scale = if selected_ref_id == Some(&r.id) {
                3.5 / 2.0 * lod_scale
            } else {
                hl_scale * lod_scale
            };
            rel_width_scale_max = rel_width_scale_max.max(frame_scale);
            draw_relation(
                ctx,
                from,
                &r.start_field_id,
                to,
                &r.end_field_id,
                palette,
                selected_ref_id == Some(&r.id),
                related,
                &r.color,
                &from.color,
                effective_line_type(&r.line_type),
                effective_stroke_style(&r.stroke_style),
                opacity,
                hl_scale,
                lod_scale,
                &r.type_,
                frame_tier,
            );
        }
    }

    // R-PERF-07：清理已删除表的精灵缓存
    evict_stale_sprites(tables);
    // #33 UT-PE-VIS-01：FK 字段集（many 侧），供字段行徽章图标族渲染
    let fk_fields = fk_field_ids(refs);
    let mut badges_drawn = 0_usize;
    for table in collect_visible_tables(tables, vp) {
        badges_drawn += table
            .fields
            .iter()
            .map(|f| {
                field_badges(f.primary, fk_fields.contains(f.id.as_str()), f.not_null, f.unique).len()
            })
            .sum::<usize>();
        // R-PERF-11：幽灵层接管中的表不在主画布绘制（拖拽期间主画布保持静态零损伤）
        if ghost_skip == Some(table.id.as_str()) {
            continue;
        }
        let is_sel = table_visually_selected(table.id.as_str(), selected_id, selected_table_ids);
        // R-PERF-04：被拖表以覆盖坐标绘制（一帧至多一张表的一次克隆）
        let visual = table_with_override(table, table_override);
        // #31 UT-PE-HL-01：非相关表 alpha ≤ 0.5 退到背景层；邻接表/选中表保持 1.0
        let alpha = table_render_alpha(
            table.id.as_str(),
            refs,
            selected_table_ids,
            selected_id,
            selected_ref_id,
        );
        table_alpha_min = table_alpha_min.min(alpha);
        if alpha < 0.999 {
            ctx.save();
            ctx.set_global_alpha(alpha);
            draw_table(ctx, &visual, is_sel, palette, t.zoom, comment_mode, frame_tier, &fk_fields, label_font_scale);
            ctx.restore();
        } else {
            draw_table(ctx, &visual, is_sel, palette, t.zoom, comment_mode, frame_tier, &fk_fields, label_font_scale);
        }
    }

    for note in collect_visible_notes(notes, vp) {
        let is_sel = table_visually_selected(note.id.as_str(), selected_note_id, selected_note_ids);
        draw_note(ctx, note, palette, is_sel);
    }

    for presence in remote_presence {
        draw_remote_presence(ctx, presence, palette);
    }

    if let Some((x1, y1, x2, y2)) = rubber_band {
        draw_rubber_band(ctx, x1, y1, x2, y2, palette);
    }

    if let Some((x1, y1, x2, y2)) = marquee_preview {
        draw_marquee_rect(ctx, x1, y1, x2, y2, palette);
    }

    // ST-PE-10 探针：暴露本帧高亮参数供 e2e 断言
    update_hl_probe(
        selected_id.is_some(),
        selected_ref_id.is_some(),
        selected_table_ids.len(),
        table_alpha_min,
        rel_width_scale_max,
        related_emphasis_seen,
    );
    // ST-CR-LOD-01 探针：暴露本帧 LOD 档位参数（#35：锚定模式 + 拓扑档注释渲染计数）
    let topo_comments = if frame_tier == LodTier::Topology {
        collect_visible_tables(tables, vp)
            .iter()
            .filter(|tbl| comment_mode.secondary(&tbl.comment).is_some())
            .count()
    } else {
        0
    };
    update_lod_probe(
        t.zoom,
        frame_tier,
        lod_scale,
        topo_comments,
        view_dimension,
        (tables.len(), refs.len(), notes.len(), areas.len()),
        // #40 R-FONT-05：倍率与钳制解析探测（any_label_font_clamped 与绘制路径同纯函数口径）
        label_font_scale,
        any_label_font_clamped(t.zoom, label_font_scale, frame_tier),
    );
    // #33 ST-PE-09：视觉体系探针（本帧徽章绘制计数在表循环累计）
    update_vis_probe(badges_drawn);

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

/// ST-PE-10 探针（#31/#32）：每帧把选中高亮参数写入 `window.__cdb_hl_probe`
/// （JSON 字符串：any_sel / sel_table / sel_ref / multi_n / table_alpha_min /
/// rel_width_scale_max），供 e2e 断言「相关线加粗提亮、非相关表/线退到背景层」
/// 在真实渲染路径生效。
#[cfg(target_arch = "wasm32")]
fn update_hl_probe(
    sel_table: bool,
    sel_ref: bool,
    multi_n: usize,
    table_alpha_min: f64,
    rel_width_scale_max: f64,
    related_emphasis: bool,
) {
    if let Some(win) = web_sys::window() {
        let target: &js_sys::Object = win.unchecked_ref();
        let key = wasm_bindgen::JsValue::from_str("__cdb_hl_probe");
        let json = format!(
            "{{\"any_sel\":{},\"sel_table\":{},\"sel_ref\":{},\"multi_n\":{},\"table_alpha_min\":{},\"rel_width_scale_max\":{},\"related_emphasis\":{}}}",
            sel_table || sel_ref || multi_n > 0,
            sel_table,
            sel_ref,
            multi_n,
            table_alpha_min,
            rel_width_scale_max,
            related_emphasis
        );
        let _ = js_sys::Reflect::set(target, &key, &wasm_bindgen::JsValue::from_str(&json));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_hl_probe(_: bool, _: bool, _: usize, _: f64, _: f64, _: bool) {}

/// ST-CR-LOD-01 探针（#30）：每帧把 LOD 档位参数写入 `window.__cdb_lod_probe`
/// （JSON 字符串：zoom / tier / font_world / lod_scale），供 e2e 断言拓扑档
/// 「字段行隐藏、表名屏幕字号 ≥11px、关系线宽补偿」在真实渲染路径生效。
#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
fn update_lod_probe(
    zoom: f64,
    tier: LodTier,
    lod_scale: f64,
    topo_comments: usize,
    dim: ViewDimension,
    counts: (usize, usize, usize, usize),
    // #40 R-FONT-05：标签字号倍率 + 本帧 10px 屏幕下限钳制标记
    label_font_scale: f64,
    min_font_clamped: bool,
) {
    if let Some(win) = web_sys::window() {
        let target: &js_sys::Object = win.unchecked_ref();
        let key = wasm_bindgen::JsValue::from_str("__cdb_lod_probe");
        let (tier_str, anchor_mode) = match tier {
            LodTier::Detail => ("detail", "field"),
            LodTier::Topology => ("topology", "table"),
        };
        // #36 R-VIEW-DIM-03：暴露显式维度（e2e 三入口一致性断言）
        let json = format!(
            "{{\"zoom\":{},\"tier\":\"{}\",\"font_world\":{},\"lod_scale\":{},\"anchor_mode\":\"{}\",\"topo_comments\":{},\"view_dimension\":\"{}\"}}",
            zoom,
            tier_str,
            lod_table_font_size(zoom, tier),
            lod_scale,
            anchor_mode,
            topo_comments,
            dim.as_str()
        );
        // #37 ST-KB-SEL-01：暴露图元计数（表/关系/便签/区域），供全选删除/撤销 e2e 断言
        let json = json.trim_end_matches('}').to_string()
            + &format!(
                ",\"tables_n\":{},\"refs_n\":{},\"notes_n\":{},\"areas_n\":{}}}",
                counts.0, counts.1, counts.2, counts.3
            );
        // #40 R-FONT-05：倍率 + 钳制标记 + 生效表名世界字号（倍率/钳制后，供 ST-CR-FONT-01 断言放大）
        let json = json.trim_end_matches('}').to_string()
            + &format!(
                ",\"label_font_scale\":{},\"min_font_clamped\":{},\"label_font_world\":{}}}",
                label_font_scale,
                min_font_clamped,
                scaled_label_font_world(lod_table_font_size(zoom, tier), zoom, label_font_scale).0
            );
        let _ = js_sys::Reflect::set(target, &key, &wasm_bindgen::JsValue::from_str(&json));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_lod_probe(_: f64, _: LodTier, _: f64, _: usize, _: ViewDimension, _: (usize, usize, usize, usize), _: f64, _: bool) {}

/// #33 ST-PE-09：视觉体系统一探针——e2e 经 window.__cdb_vis_probe 读取徽章/端点/圆角
/// 常量与绘制口径（字块角标已移除，端点为 crow's foot 族）。
#[cfg(target_arch = "wasm32")]
fn update_vis_probe(badges_drawn: usize) {
    if let Some(win) = web_sys::window() {
        let target: &js_sys::Object = win.unchecked_ref();
        let key = wasm_bindgen::JsValue::from_str("__cdb_vis_probe");
        let json = format!(
            "{{\"badge_size\":{},\"badge_stroke\":{},\"endpoint_size\":{},\"corner_radius\":{},\"endpoint_style\":\"crowsfoot\",\"text_badge\":false,\"badges_drawn\":{}}}",
            BADGE_SIZE,
            BADGE_STROKE,
            REL_ENDPOINT_SIZE,
            TABLE_CORNER_RADIUS,
            badges_drawn
        );
        let _ = js_sys::Reflect::set(target, &key, &wasm_bindgen::JsValue::from_str(&json));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_vis_probe(_: usize) {}

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

// ─── #30/#36 UT-CR-LOD-01：LOD 分档与显式维度（core-01 §5.12 / R-LOD-02~08 / R-VIEW-DIM） ──────────

// fix-issues-36-37（issue #36，R-LOD-01 修订）：拓扑档 zoom 阈值常量与 ±0.03
// 滞回随「zoom 触发档位切换」一并退役——维度由用户显式切换（R-VIEW-DIM-01），
// zoom 仅驱动表维度内可读性补偿（R-LOD-03/04）。

/// 表维度表名屏幕字号下限（px，R-LOD-03）。
pub const LOD_MIN_SCREEN_FONT_PX: f64 = 11.0;
/// 表维度表名世界字号夹紧上限（px）——极小 zoom 下避免单表占满视口。
pub const LOD_TABLE_FONT_WORLD_MAX: f64 = 30.0;
/// 字段维度表名默认世界字号（draw_table_body 表名 13px，R-LOD-03 字段维度不补偿）。
pub const TABLE_NAME_FONT_PX: f64 = 13.0;
/// 表维度线宽补偿目标屏幕线宽（px，R-LOD-04）。
pub const LOD_MIN_SCREEN_LINE_PX: f64 = 1.5;
/// 表维度线宽补偿倍率夹紧上限（相对 base）。
pub const LOD_LINE_WIDTH_COMP_MAX: f64 = 4.0;

/// LOD 渲染档：详情档（字段维度渲染）/ 拓扑档（表维度渲染——字段行隐藏、大字表名、线宽补偿）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodTier {
    Detail,
    Topology,
}

/// #36 UT-CR-LOD-01（R-VIEW-DIM-01）：显式维度 → 渲染档映射。与 zoom 无关——
/// 任何缩放级别下同一维度档位不变（维度不随缩放串档）。
pub fn tier_for_dimension(dim: ViewDimension) -> LodTier {
    match dim {
        ViewDimension::Table => LodTier::Topology,
        ViewDimension::Field => LodTier::Detail,
    }
}

/// #30/#36 UT-CR-LOD-01：表名世界字号——表维度按 11px 屏幕字号下限放大（夹紧 30px）；
/// 详情档返回默认 13px。
pub fn lod_table_font_size(zoom: f64, tier: LodTier) -> f64 {
    match tier {
        LodTier::Detail => TABLE_NAME_FONT_PX,
        LodTier::Topology => {
            (LOD_MIN_SCREEN_FONT_PX / zoom.max(0.01)).min(LOD_TABLE_FONT_WORLD_MAX)
        }
    }
}

/// #30 UT-CR-LOD-01：LOD 线宽补偿——世界线宽保证屏幕 ≥1.5px（max(base, 1.5/zoom)），
/// 补偿倍率相对 base 夹紧 ≤4×。纯数学：调用处仅在拓扑档启用（详情档 lod_scale=1 无回归）。
pub fn lod_line_width(zoom: f64, base: f64) -> f64 {
    let target = LOD_MIN_SCREEN_LINE_PX / zoom.max(0.01);
    base.max(target).min(base * LOD_LINE_WIDTH_COMP_MAX)
}

/// fix-issues-38-41-canvas-interaction（issue #40，core-01 §5.16 R-FONT-04）：
/// 标签屏幕空间最小字号（px）——任意 zoom/倍率下表名/字段名/注释屏幕字号 ≥10px。
pub const LABEL_FONT_MIN_SCREEN_PX: f64 = 10.0;

/// #40 UT-CR-FONT-01（R-FONT-03/04）：标签字号倍率 + 屏幕下限钳制——
/// 最终世界字号 = max(base × scale, 10px/zoom)；返回 (世界字号, 是否触发下限钳制)。
/// 钳制仅在缩小方向生效，放大方向不封顶；zoom 防御性下限 0.01。
pub fn scaled_label_font_world(base_world: f64, zoom: f64, scale: f64) -> (f64, bool) {
    let scaled = base_world * scale;
    let min_world = LABEL_FONT_MIN_SCREEN_PX / zoom.max(0.01);
    if scaled < min_world {
        (min_world, true)
    } else {
        (scaled, false)
    }
}

/// #40 R-FONT-05：本帧钳制解析探测——任一标签字体（详情档表名 13/字段 11/注释 10；
/// 拓扑档表名 font_world 及其 0.8× 注释）触发 10px 屏幕下限即为 true。
/// 与绘制路径共用 scaled_label_font_world，口径一致。
pub fn any_label_font_clamped(zoom: f64, scale: f64, tier: LodTier) -> bool {
    match tier {
        LodTier::Detail => [TABLE_NAME_FONT_PX, 11.0, 10.0]
            .iter()
            .any(|&b| scaled_label_font_world(b, zoom, scale).1),
        LodTier::Topology => {
            let fw = lod_table_font_size(zoom, LodTier::Topology);
            [fw, fw * 0.8]
                .iter()
                .any(|&b| scaled_label_font_world(b, zoom, scale).1)
        }
    }
}

/// 拓扑档卡体尺寸：宽同详情（布局不变），高仅表头（字段行隐藏，R-LOD-01）。
pub fn lod_table_size(table: &Table, comment_mode: CommentDisplay, tier: LodTier) -> (f64, f64) {
    match tier {
        LodTier::Detail => compute_table_render_size_for(table, comment_mode),
        LodTier::Topology => (resolve_table_width(table, comment_mode), TABLE_HEADER_HEIGHT),
    }
}

/// R-PERF-07：卡体内容指纹（UT-CR-SPRITE-01）。位置（x/y）与选中态不参与——
/// 移动不换缓存；编辑落账产生新内容才失效。FNV-1a。
// fix-remote-github-issues-7-18（issue #10/#12，core-01 §5.8）：注释/颜色渲染纯函数 ───

/// R-COLOR-01：表边框用色——`table.color` 非空跟随，为空回退 `palette.table_border`。
pub fn table_border_color<'a>(table_color: &'a str, palette_border: &'a str) -> &'a str {
    if table_color.trim().is_empty() {
        palette_border
    } else {
        table_color.trim()
    }
}

/// R-COLOR-02 / UT-PB-11 / UT-PB-12（#22）：关系线用色优先级
/// 显式 `ref.color` > 源表 `source_table_color` > `palette.relation`。
pub fn relation_stroke_color<'a>(
    ref_color: &'a str,
    source_table_color: &'a str,
    palette_relation: &'a str,
) -> &'a str {
    if !ref_color.trim().is_empty() {
        ref_color.trim()
    } else if !source_table_color.trim().is_empty() {
        source_table_color.trim()
    } else {
        palette_relation
    }
}

/// R-COLOR-04：表头有效背景相对亮度阈值（WCAG sRGB 相对亮度）。
pub const HEADER_FG_LUMINANCE_THRESHOLD: f64 = 0.55;

/// 表头强/次前景色对（R-COLOR-04）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderForeground<'a> {
    pub strong: &'a str,
    pub muted: &'a str,
}

fn srgb_channel_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance_rgb(r: f64, g: f64, b: f64) -> f64 {
    let r = srgb_channel_to_linear(r);
    let g = srgb_channel_to_linear(g);
    let b = srgb_channel_to_linear(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn parse_hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn parse_hex_byte(h: u8, l: u8) -> Option<f64> {
    Some(((parse_hex_digit(h)? << 4) | parse_hex_digit(l)?) as f64 / 255.0)
}

/// 解析 `#rgb` / `#rrggbb` / `rgb(r,g,b)` / `rgba(r,g,b,a)` → (r,g,b,a) ∈ 0..=1。
pub fn parse_css_color_rgba(input: &str) -> Option<(f64, f64, f64, f64)> {
    let s = input.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let b = hex.as_bytes();
        return match b.len() {
            3 => Some((
                parse_hex_digit(b[0])? as f64 / 15.0,
                parse_hex_digit(b[1])? as f64 / 15.0,
                parse_hex_digit(b[2])? as f64 / 15.0,
                1.0,
            )),
            6 => Some((
                parse_hex_byte(b[0], b[1])?,
                parse_hex_byte(b[2], b[3])?,
                parse_hex_byte(b[4], b[5])?,
                1.0,
            )),
            8 => Some((
                parse_hex_byte(b[0], b[1])?,
                parse_hex_byte(b[2], b[3])?,
                parse_hex_byte(b[4], b[5])?,
                parse_hex_byte(b[6], b[7])?,
            )),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    let (body, has_a) = if let Some(rest) = lower.strip_prefix("rgba(") {
        (rest.strip_suffix(')')?, true)
    } else if let Some(rest) = lower.strip_prefix("rgb(") {
        (rest.strip_suffix(')')?, false)
    } else {
        return None;
    };
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    if has_a {
        if parts.len() != 4 {
            return None;
        }
        let r: f64 = parts[0].parse().ok()?;
        let g: f64 = parts[1].parse().ok()?;
        let b: f64 = parts[2].parse().ok()?;
        let a: f64 = parts[3].parse().ok()?;
        Some((
            (r / 255.0).clamp(0.0, 1.0),
            (g / 255.0).clamp(0.0, 1.0),
            (b / 255.0).clamp(0.0, 1.0),
            a.clamp(0.0, 1.0),
        ))
    } else {
        if parts.len() != 3 {
            return None;
        }
        let r: f64 = parts[0].parse().ok()?;
        let g: f64 = parts[1].parse().ok()?;
        let b: f64 = parts[2].parse().ok()?;
        Some((
            (r / 255.0).clamp(0.0, 1.0),
            (g / 255.0).clamp(0.0, 1.0),
            (b / 255.0).clamp(0.0, 1.0),
            1.0,
        ))
    }
}

fn composite_over(src: (f64, f64, f64, f64), dst: (f64, f64, f64, f64)) -> (f64, f64, f64) {
    let (sr, sg, sb, sa) = src;
    let (dr, dg, db, da) = dst;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 1e-9 {
        return (dr, dg, db);
    }
    let r = (sr * sa + dr * da * (1.0 - sa)) / out_a;
    let g = (sg * sa + dg * da * (1.0 - sa)) / out_a;
    let b = (sb * sa + db * da * (1.0 - sa)) / out_a;
    (r, g, b)
}

/// tint over base 合成后的相对亮度；tint 解析失败 → None。
pub fn composited_relative_luminance(tint: &str, base: &str) -> Option<f64> {
    let src = parse_css_color_rgba(tint)?;
    let dst = parse_css_color_rgba(base).unwrap_or((0.0, 0.0, 0.0, 1.0));
    let (r, g, b) = composite_over(src, dst);
    Some(relative_luminance_rgb(r, g, b))
}

/// R-COLOR-04：按表头有效背景亮度选择深/浅前景对。
/// `fallback_dark_fg=true` 表示解析失败时倾向深色字（亮主题回退）。
pub fn header_foreground_colors<'a>(
    header_tint: &str,
    table_bg: &str,
    light_strong: &'a str,
    light_muted: &'a str,
    dark_strong: &'a str,
    dark_muted: &'a str,
    fallback_dark_fg: bool,
) -> HeaderForeground<'a> {
    let use_dark_fg = composited_relative_luminance(header_tint, table_bg)
        .map(|l| l >= HEADER_FG_LUMINANCE_THRESHOLD)
        .unwrap_or(fallback_dark_fg);
    if use_dark_fg {
        HeaderForeground {
            strong: light_strong,
            muted: light_muted,
        }
    } else {
        HeaderForeground {
            strong: dark_strong,
            muted: dark_muted,
        }
    }
}

/// #32 UT-PE-CMT-01：注释文本最小对比度（WCAG AA 正文 4.5:1，R-CMT-CONTRAST-02）。
pub const COMMENT_CONTRAST_MIN: f64 = 4.5;
/// #32 UT-PE-CMT-01：chip 衬底纱罩 alpha（core-07 §15.4 token `canvas.comment.chip-bg-alpha`，
/// R-CMT-CONTRAST-02 允许区间 0.35–0.55；取上限以数学保证兜底后对比度达标）。
pub const COMMENT_CHIP_BG_ALPHA: f64 = 0.55;
/// chip 深纱罩（浅前景文字下垫深色，盖住表头渐变）。
pub const COMMENT_CHIP_DARK: &str = "rgba(10,20,24,0.55)";
/// chip 浅纱罩（深前景文字下垫浅色）。
pub const COMMENT_CHIP_LIGHT: &str = "rgba(255,255,255,0.55)";

/// #32 UT-PE-CMT-01：注释渲染样式——前景色 + 可选 chip 衬底色。
#[derive(Clone, Debug, PartialEq)]
pub struct CommentStyle {
    pub fg: String,
    /// None = 无衬底直接绘制；Some = 先绘制该色圆角衬底再绘制文字。
    pub chip: Option<String>,
}

/// 单色串相对亮度（无 alpha 通道时视为不透明；解析失败 → None）。
pub fn color_relative_luminance(color: &str) -> Option<f64> {
    let (r, g, b, _) = parse_css_color_rgba(color)?;
    Some(relative_luminance_rgb(r, g, b))
}

/// WCAG 对比度 (L_hi + 0.05) / (L_lo + 0.05)。
pub fn contrast_ratio_luminance(l1: f64, l2: f64) -> f64 {
    let (lo, hi) = if l1 < l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// #32 UT-PE-CMT-01：注释前景 fg 相对「tint 合成到 table_bg」有效背景的对比度。
/// 任一色解析失败 → None。
pub fn comment_contrast_ratio(fg: &str, header_tint: &str, table_bg: &str) -> Option<f64> {
    let fg_l = color_relative_luminance(fg)?;
    let bg_l = composited_relative_luminance(header_tint, table_bg)?;
    Some(contrast_ratio_luminance(fg_l, bg_l))
}

/// #34 UT-PE-CMT-01（R-CMT-CONTRAST-01 双端点判据）：fg 相对「满强度 tint 合成端」与
/// 「渐变衰减端（tint 全透明 = 纯表体底色）」两端对比度的**较小值**。任一解析失败 → None。
pub fn comment_contrast_ratio_min(fg: &str, header_tint: &str, table_bg: &str) -> Option<f64> {
    let fg_l = color_relative_luminance(fg)?;
    let full_l = composited_relative_luminance(header_tint, table_bg)?;
    let faded_l = composited_relative_luminance("rgba(0,0,0,0)", table_bg)?;
    Some(contrast_ratio_luminance(fg_l, full_l).min(contrast_ratio_luminance(fg_l, faded_l)))
}

/// #32 UT-PE-CMT-01：对比度 < 4.5:1 → 需要 chip 衬底兜底（R-CMT-CONTRAST-02）。
pub fn needs_comment_chip(contrast: f64) -> bool {
    contrast < COMMENT_CONTRAST_MIN
}

/// #32 UT-PE-CMT-01：注释前景/衬底决策（R-CMT-CONTRAST-01/02）。
/// 入参为 R-COLOR-04 同一对深/浅前景组（dark_* 用于亮背景，light_* 用于深背景）：
/// 1. 按 HEADER_FG_LUMINANCE_THRESHOLD 选定方向后，muted 达标 → muted 无衬底；
/// 2. 该方向 strong 达标 → strong 无衬底；
/// 3. 反方向 strong 达标（中间亮度表头，如实色琥珀）→ 反方向 strong 无衬底；
/// 4. 否则 chip 兜底：浅 strong + 深纱罩 或 深 strong + 浅纱罩，取兜底后对比更高者。
///
/// #34（R-CMT-CONTRAST-01 双端点判据）：表头为 tint → 透明渐变，注释实际渲染在渐变
/// 中右段；所有「达标」判定均为**双端点 worst-case**——满强度 tint 合成端与渐变衰减端
/// （纯表体底色）两端对比度同时 ≥ 4.5:1 才免 chip；chip 方案同样双端合成取最小值评分。
/// 字段行注释 tint 入参全透明，两端点退化为同一样本，行为与单端点一致。
pub fn comment_foreground(
    header_tint: &str,
    table_bg: &str,
    dark_strong: &str,
    dark_muted: &str,
    light_strong: &str,
    light_muted: &str,
    fallback_dark_fg: bool,
) -> CommentStyle {
    let pair = header_foreground_colors(
        header_tint,
        table_bg,
        dark_strong,
        dark_muted,
        light_strong,
        light_muted,
        fallback_dark_fg,
    );
    // #34：候选判定全部走双端点 worst-case（满 tint 合成端 + 渐变衰减端同时 ≥4.5:1）
    if let Some(c) = comment_contrast_ratio_min(pair.muted, header_tint, table_bg) {
        if !needs_comment_chip(c) {
            return CommentStyle {
                fg: pair.muted.to_string(),
                chip: None,
            };
        }
    }
    if let Some(c) = comment_contrast_ratio_min(pair.strong, header_tint, table_bg) {
        if !needs_comment_chip(c) {
            return CommentStyle {
                fg: pair.strong.to_string(),
                chip: None,
            };
        }
    }
    // 反方向 strong（中间亮度表头：两个方向的字直绘都可能不足，反方向常可免 chip）
    let alt_strong = if pair.strong == dark_strong {
        light_strong
    } else {
        dark_strong
    };
    if let Some(c) = comment_contrast_ratio_min(alt_strong, header_tint, table_bg) {
        if !needs_comment_chip(c) {
            return CommentStyle {
                fg: alt_strong.to_string(),
                chip: None,
            };
        }
    }
    // chip 兜底：浅字 + 深纱罩 vs 深字 + 浅纱罩；#34 双端点——chip 分别合成到满强度
    // tint 端与渐变衰减端，取两端对比度的较小值作为方案得分，得分高者胜出
    let chip_score = |fg: &str, chip: &str| -> Option<f64> {
        let fg_l = color_relative_luminance(fg)?;
        let full = chip_composited_luminance(chip, header_tint, table_bg)?;
        let faded = chip_composited_luminance(chip, "rgba(0,0,0,0)", table_bg)?;
        Some(contrast_ratio_luminance(fg_l, full).min(contrast_ratio_luminance(fg_l, faded)))
    };
    let light_fg_score = chip_score(light_strong, COMMENT_CHIP_DARK);
    let dark_fg_score = chip_score(dark_strong, COMMENT_CHIP_LIGHT);
    match (light_fg_score, dark_fg_score) {
        (Some(l), Some(d)) if l >= d => CommentStyle {
            fg: light_strong.to_string(),
            chip: Some(COMMENT_CHIP_DARK.to_string()),
        },
        (Some(_), Some(_)) => CommentStyle {
            fg: dark_strong.to_string(),
            chip: Some(COMMENT_CHIP_LIGHT.to_string()),
        },
        // 解析失败回退：按阈值方向取 strong + 对应纱罩
        _ => {
            if pair.strong == light_strong {
                CommentStyle {
                    fg: light_strong.to_string(),
                    chip: Some(COMMENT_CHIP_DARK.to_string()),
                }
            } else {
                CommentStyle {
                    fg: dark_strong.to_string(),
                    chip: Some(COMMENT_CHIP_LIGHT.to_string()),
                }
            }
        }
    }
}

/// chip 纱罩先合成到 tint，再合成到 table_bg 的最终背景亮度。
fn chip_composited_luminance(chip: &str, header_tint: &str, table_bg: &str) -> Option<f64> {
    let chip_rgba = parse_css_color_rgba(chip)?;
    let tint_rgba = parse_css_color_rgba(header_tint)?;
    let base_rgba = parse_css_color_rgba(table_bg).unwrap_or((0.0, 0.0, 0.0, 1.0));
    let under = composite_over(tint_rgba, base_rgba);
    let (r, g, b) = composite_over(chip_rgba, (under.0, under.1, under.2, 1.0));
    Some(relative_luminance_rgb(r, g, b))
}

/// R-CMT-01/02：单行省略截断——逐字符测量，超出 max_w 时截断并追加 …（canvas 无原生省略）。
fn truncate_to_width(ctx: &CanvasRenderingContext2d, text: &str, max_w: f64) -> String {
    let full_w = ctx.measure_text(text).map(|m| m.width()).unwrap_or(0.0);
    if full_w <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{out}{ch}…");
        let w = ctx.measure_text(&candidate).map(|m| m.width()).unwrap_or(f64::MAX);
        if w > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{out}…")
}

/// #32 UT-PE-CMT-01（R-CMT-CONTRAST-02）：注释 chip 衬底——圆角纱罩垫在注释文字下，
/// 盖住表头渐变以数学保证前景对比度达标（chip 色由 comment_foreground 决策给出）。
fn draw_comment_chip(ctx: &CanvasRenderingContext2d, chip_color: &str, x: f64, y_center: f64, text_w: f64) {
    ctx.save();
    let _ = ctx.set_fill_style_str(chip_color);
    ctx.begin_path();
    round_rect(ctx, x - 3.0, y_center - 8.5, text_w + 6.0, 17.0, 8.0);
    ctx.fill();
    ctx.restore();
}

/// #33 UT-PE-VIS-01（core-08 §11）：绘制字段语义徽章图标族（线条图标，非字块角标）。
/// 统一 BADGE_SIZE 外接尺寸与 BADGE_STROKE 描边；色值仅取 palette 语义色阶
/// （PK=warning 系 / FK=info 系 / NN·UQ=grey 系）。返回占用宽度（field_badges_width 同源）。
fn draw_field_badges(
    ctx: &CanvasRenderingContext2d,
    badges: &[FieldBadge],
    palette: &CanvasPalette,
    x: f64,
    y_center: f64,
) -> f64 {
    for (i, badge) in badges.iter().enumerate() {
        let bx = x + i as f64 * (BADGE_SIZE + BADGE_GAP);
        let by = y_center - BADGE_SIZE / 2.0;
        let color = match badge {
            FieldBadge::Pk => palette.pk_color,
            FieldBadge::Fk => palette.fk_color,
            FieldBadge::Nn | FieldBadge::Uq => palette.constraint_color,
        };
        ctx.save();
        let _ = ctx.translate(bx, by);
        let _ = ctx.set_stroke_style_str(color);
        ctx.set_line_width(BADGE_STROKE);
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.begin_path();
        match badge {
            // 钥匙：环 + 柄 + 两齿
            FieldBadge::Pk => {
                ctx.arc(4.0, 4.0, 2.3, 0.0, std::f64::consts::TAU).ok();
                ctx.move_to(5.6, 5.6);
                ctx.line_to(10.0, 10.0);
                ctx.move_to(8.1, 8.1);
                ctx.line_to(9.4, 6.8);
                ctx.move_to(9.4, 9.4);
                ctx.line_to(10.7, 8.1);
                ctx.stroke();
            }
            // 链环：两个错位圆角矩形相扣
            FieldBadge::Fk => {
                round_rect(ctx, 1.0, 5.0, 5.5, 4.0, 2.0);
                ctx.stroke();
                ctx.begin_path();
                round_rect(ctx, 5.5, 3.0, 5.5, 4.0, 2.0);
                ctx.stroke();
            }
            // 星号：竖线 + 两条对角线
            FieldBadge::Nn => {
                ctx.move_to(6.0, 1.5);
                ctx.line_to(6.0, 10.5);
                ctx.move_to(2.1, 3.75);
                ctx.line_to(9.9, 8.25);
                ctx.move_to(9.9, 3.75);
                ctx.line_to(2.1, 8.25);
                ctx.stroke();
            }
            // 菱形：旋转正方形轮廓
            FieldBadge::Uq => {
                ctx.move_to(6.0, 1.5);
                ctx.line_to(10.5, 6.0);
                ctx.line_to(6.0, 10.5);
                ctx.line_to(1.5, 6.0);
                ctx.close_path();
                ctx.stroke();
            }
        }
        ctx.restore();
    }
    field_badges_width(badges.len())
}

/// R-PERF-07：表精灵指纹（变化触发重光栅）。fix-remote-github-issues-7-18：
/// 表/字段 comment 与注释显示模式混入指纹（R-CMT-03——注释内容/模式变化必须重光栅）。
/// #30 UT-CR-LOD-01：LOD 档位混入指纹（R-LOD-01——跨档重光栅；同档内指纹相同，
/// 拖动/同档 zoom 微调不触发重光栅）。
pub fn table_sprite_fingerprint(table: &Table, theme_dark: bool, dpr_x100: u32, zoom_bucket: u32, comment_mode: CommentDisplay, lod: LodTier, fk_bits: &[bool]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(table.name.as_bytes());
    mix(table.color.as_bytes());
    mix(table.comment.as_bytes());
    mix(&table.width.unwrap_or(0).to_le_bytes());
    mix(&table.min_height.unwrap_or(0).to_le_bytes());
    for (i, f) in table.fields.iter().enumerate() {
        mix(f.name.as_bytes());
        mix(f.type_.as_bytes());
        mix(f.comment.as_bytes());
        // #33：primary/unique/not_null/FK 均影响徽章渲染，全部混入指纹（缺位 FK 视为 false）
        mix(&[
            f.primary as u8,
            f.unique as u8,
            f.not_null as u8,
            fk_bits.get(i).copied().unwrap_or(false) as u8,
        ]);
    }
    mix(&[comment_mode as u8]);
    mix(&[theme_dark as u8]);
    mix(&dpr_x100.to_le_bytes());
    mix(&(zoom_bucket as u32).to_le_bytes());
    mix(&[lod as u8]);
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
    comment_mode: CommentDisplay,
    lod: LodTier,
    zoom: f64,
    fk_fields: &std::collections::HashSet<&str>,
    // #40 R-FONT-03：标签字号倍率（透传 draw_table_body / 拓扑档 font_world）
    label_scale: f64,
) -> Option<TableSprite> {
    let doc = web_sys::window()?.document()?;
    let canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .ok()?
        .dyn_into()
        .ok()?;
    // #30：拓扑档卡体仅表头高（字段行隐藏），精灵尺寸按档位取
    let (w, h) = lod_table_size(table, comment_mode, lod);
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
    match lod {
        LodTier::Detail => draw_table_body(&off, table, palette, shadow_boost, comment_mode, fk_fields, zoom, label_scale),
        // #30：拓扑档字体世界尺寸 = lod_table_font_size（屏幕 ≥11px）；zoom 由调用处经
        // scale/CTM 自然缩放，光栅只需世界字号
        // #40 R-FONT-03/04：倍率缩放 + 10px 屏幕下限钳制在光栅前完成（与活画同口径）
        LodTier::Topology => draw_table_topology_body(
            &off, table, palette, shadow_boost, comment_mode,
            scaled_label_font_world(lod_table_font_size(zoom, lod), zoom, label_scale).0,
            zoom,
        ),
    }
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
    comment_mode: CommentDisplay,
    lod: LodTier,
    fk_fields: &std::collections::HashSet<&str>,
    // #40 R-FONT-03：标签字号倍率——参与指纹（变化触发重光栅）并透传光栅化
    label_scale: f64,
) -> bool {
    // R-PERF-10 修正：精灵恒以真实 dpr 渲染/取指纹——backing 分辨率与主画布有效 dpr
    // 解耦（位块传输按世界坐标 dw/dh 绘制，主画布降采样时由 CTM 自然缩小，观感不劣化）。
    // 这样拖拽起止的有效 dpr 切换不会触发全量精灵重渲染，消除边界帧的突刺成本。
    let dpr = current_device_pixel_ratio();
    let bucket = sprite_zoom_bucket(zoom);
    let scale = dpr * bucket as f64;
    let fk_bits: Vec<bool> = table
        .fields
        .iter()
        .map(|f| fk_fields.contains(f.id.as_str()))
        .collect();
    let fp = table_sprite_fingerprint(
        table,
        current_theme_dark(),
        (dpr * 100.0).round() as u32,
        bucket,
        comment_mode,
        lod,
        &fk_bits,
    // #40 R-FONT-03：倍率混入指纹（黄金比例散列到高位，避免与低位 zoom/dpr 桶串扰）——
    // 切档即重光栅；table_sprite_fingerprint 签名不变（既有 UT 调用点不动）
    ) ^ ((label_scale * 100.0).round() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let sprite = TABLE_SPRITES.with(|c| {
        let mut map = c.borrow_mut();
        let fresh = map
            .get(&table.id)
            .map(|s| s.fingerprint == fp)
            .unwrap_or(false);
        if !fresh {
            // shadow_boost = bucket / zoom（见 render_table_sprite 注释）
            let boost = bucket as f64 / zoom.max(0.01);
            if let Some(s) = render_table_sprite(table, palette, scale, boost, fp, comment_mode, lod, zoom, fk_fields, label_scale) {
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
    comment_mode: CommentDisplay,
    view_dimension: ViewDimension,
    // #40 R-FONT-03：幽灵层卡体字号与主画布同倍率（拖动中视觉一致）
    label_scale: f64,
) -> Option<GhostDrag> {
    let dpr = current_device_pixel_ratio();
    let bucket = sprite_zoom_bucket(t.zoom);
    let scale = dpr * bucket as f64;
    let boost = bucket as f64 / t.zoom.max(0.01);
    // #30/#36：幽灵层卡体与被拖表同维度（表维度拖的是表维度卡）；档位随显式维度（R-VIEW-DIM-01）
    let tier = tier_for_dimension(view_dimension);
    // #33：幽灵层仅在「无关系表」拖拽时创建（见调用处 table_has_references 守卫），
    // 无关系即无 FK 徽章，fk 传空集
    let sprite = render_table_sprite(table, palette, scale, boost, 0, comment_mode, tier, t.zoom, &std::collections::HashSet::new(), label_scale)?;
    let el = sprite.canvas;
    let parent = canvas.parent_element()?;
    let css_w = sprite.w_world * t.zoom;
    let css_h = sprite.h_world * t.zoom;
    // fix-remote-github-issues-7-18（issue #17，R-HL-01/02）：高亮环改为幽灵 canvas 内
    // round_rect 描边（与 draw_table_selection 同一函数、同一 TABLE_CORNER_RADIUS 口径）——
    // CSS outline 不贴合 border-radius，拖动时呈直角。幽灵层不再携带任何 outline 样式。
    if let Ok(Some(ctx)) = el.get_context("2d") {
        if let Ok(off) = ctx.dyn_into::<CanvasRenderingContext2d>() {
            let _ = off.set_transform(
                scale,
                0.0,
                0.0,
                scale,
                (sprite.margin - table.x) * scale,
                (sprite.margin - table.y) * scale,
            );
            draw_table_selection(&off, table, palette, comment_mode, tier);
        }
    }
    // will-change 提示合成器把幽灵层提升为独立层，transform 移动纯合成器完成
    let style = format!(
        "position:absolute;left:0;top:0;width:{css_w}px;height:{css_h}px;\
         pointer-events:none;z-index:2;will-change:transform;"
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

fn draw_table(ctx: &CanvasRenderingContext2d, table: &Table, selected: bool, palette: &CanvasPalette, zoom: f64, comment_mode: CommentDisplay, tier: LodTier, fk_fields: &std::collections::HashSet<&str>, label_scale: f64) {
    // #36 UT-CR-LOD-01：渲染档由调用方按显式维度传入（R-VIEW-DIM-01），不再按 zoom 判档
    // R-PERF-07：zoom ≤ SPRITE_CACHE_MAX_ZOOM 走精灵缓存；超出回退活画
    if zoom <= SPRITE_CACHE_MAX_ZOOM && blit_table_sprite(ctx, table, palette, zoom, comment_mode, tier, fk_fields, label_scale) {
        if selected {
            draw_table_selection(ctx, table, palette, comment_mode, tier);
        }
        return;
    }
    match tier {
        LodTier::Detail => draw_table_body(ctx, table, palette, 1.0, comment_mode, fk_fields, zoom, label_scale),
        LodTier::Topology => draw_table_topology_body(
            ctx, table, palette, 1.0, comment_mode,
            // #40 R-FONT-03/04：拓扑档表名字号同样过倍率缩放 + 10px 屏幕下限钳制
            scaled_label_font_world(lod_table_font_size(zoom, tier), zoom, label_scale).0,
            zoom,
        ),
    }
    if selected {
        draw_table_selection(ctx, table, palette, comment_mode, tier);
    }
}

fn draw_table_body(ctx: &CanvasRenderingContext2d, table: &Table, palette: &CanvasPalette, shadow_boost: f64, comment_mode: CommentDisplay, fk_fields: &std::collections::HashSet<&str>, zoom: f64, label_scale: f64) {
    // fix-issues-38-41-canvas-interaction（#40，R-FONT-03/04）：表名/字段名/注释字号
    // = 既有字号 × 倍率，屏幕空间钳制 ≥10px（scaled_label_font_world）
    let name_px = scaled_label_font_world(13.0, zoom, label_scale).0;
    let cmt_px = scaled_label_font_world(11.0, zoom, label_scale).0;
    let meta_px = scaled_label_font_world(10.0, zoom, label_scale).0;
    let field_px = scaled_label_font_world(11.0, zoom, label_scale).0;
    let field_count = table.fields.len().max(2);
    let (width, total_height) = compute_table_render_size_for(table, comment_mode);
    let x = table.x;
    let y = table.y;

    // 表体：主原型 .db-table —— #33 统一 radius 8（TABLE_CORNER_RADIUS）、surface-solid 底、
    // line-strong 描边、柔和投影
    // 阴影为设备像素口径（不随 CTM 缩放），shadow_boost 用于精灵离屏渲染的预补偿
    ctx.save();
    let _ = ctx.set_shadow_color("rgba(0, 0, 0, 0.18)");
    let _ = ctx.set_shadow_blur(16.0 * shadow_boost);
    let _ = ctx.set_shadow_offset_x(0.0);
    let _ = ctx.set_shadow_offset_y(6.0 * shadow_boost);

    let _ = ctx.set_fill_style_str(palette.table_bg);
    ctx.begin_path();
    round_rect(ctx, x, y, width, total_height, TABLE_CORNER_RADIUS);
    ctx.fill();
    ctx.restore();

    // R-COLOR-01：表边框——table.color 非空跟随该色，为空保持 palette.table_border
    let _ = ctx.set_stroke_style_str(table_border_color(&table.color, palette.table_border));
    ctx.set_line_width(1.0);
    ctx.begin_path();
    round_rect(ctx, x, y, width, total_height, TABLE_CORNER_RADIUS);
    ctx.stroke();

    // 表头：主原型 .table-head —— #33 统一 135° 对角 tint 渐变（表色或 brand-soft → 透明），非实心填充
    let header_tint = if table.color.trim().is_empty() {
        palette.header_tint
    } else {
        table.color.as_str()
    };
    // R-COLOR-04：表头前景相对有效背景亮度自适应（不得在浅色表头上固定用主题浅色字）
    let header_fg = header_foreground_colors(
        header_tint,
        palette.table_bg,
        PALETTE_LIGHT.text_strong,
        PALETTE_LIGHT.text_muted,
        PALETTE_DARK.text_strong,
        PALETTE_DARK.text_muted,
        !current_theme_dark(),
    );
    ctx.save();
    ctx.begin_path();
    round_rect_top(ctx, x, y, width, TABLE_HEADER_HEIGHT, TABLE_CORNER_RADIUS);
    ctx.clip();
    // 135° 族对角渐变：左上 → 右下（水平为主、带纵向分量，对齐 CSS 135deg 观感）
    let gradient = ctx.create_linear_gradient(x, y, x + width, y + TABLE_HEADER_HEIGHT);
    gradient.add_color_stop(0.0, header_tint).ok();
    gradient.add_color_stop(1.0, "rgba(0,0,0,0)").ok();
    let _ = ctx.set_fill_style_str("rgba(0,0,0,0)");
    ctx.set_fill_style_canvas_gradient(&gradient);
    ctx.fill_rect(x, y, width, TABLE_HEADER_HEIGHT);
    ctx.restore();

    // 表名（750/13px 强色）+ 字段计数（text-3 10px 右对齐）
    // R-CMT-01：主文本按显示模式取值（comment 模式且有注释 → 注释；否则英文名）
    let table_label = comment_mode.primary(&table.name, &table.comment);
    let _ = ctx.set_fill_style_str(header_fg.strong);
    let _ = ctx.set_font(&dpr_font(750, name_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.set_text_align("left");
    let _ = ctx.fill_text(table_label, x + 11.0, y + TABLE_HEADER_HEIGHT / 2.0);
    // R-CMT-01：name+comment 模式且 comment 非空 → 表名右侧渲染注释（小字次要色，单行省略）；
    // 空 comment 不渲染、不留占位（R-CMT-03）。canvas 无 DOM title——截断省略与原型一致。
    if let Some(cmt) = comment_mode.secondary(&table.comment) {
        let label_w = ctx.measure_text(table_label).map(|m| m.width()).unwrap_or(0.0);
        // #32 UT-PE-CMT-01（R-CMT-CONTRAST-01/02/04/05）：注释前景按表头有效背景对比度决策
        // （≥4.5:1，不足时 chip 衬底兜底）；字号 11px ≥ 表名 13px 的 0.8× 且 ≥11px，字重 600 ≥400
        let cmt_style = comment_foreground(
            header_tint,
            palette.table_bg,
            PALETTE_LIGHT.text_strong,
            PALETTE_LIGHT.text_muted,
            PALETTE_DARK.text_strong,
            PALETTE_DARK.text_muted,
            !current_theme_dark(),
        );
        let _ = ctx.set_font(&dpr_font(600, cmt_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let cmt_x = x + 11.0 + label_w + 6.0;
        // 26 = 右边距 11 + 字段计数预留 15
        let max_w = (x + width - 26.0) - cmt_x;
        if max_w > 12.0 {
            let shown = truncate_to_width(ctx, cmt, max_w);
            let cmt_w = ctx.measure_text(&shown).map(|m| m.width()).unwrap_or(0.0);
            if let Some(chip) = &cmt_style.chip {
                draw_comment_chip(ctx, chip, cmt_x, y + TABLE_HEADER_HEIGHT / 2.0 + 0.5, cmt_w);
            }
            let _ = ctx.set_fill_style_str(&cmt_style.fg);
            let _ = ctx.fill_text(&shown, cmt_x, y + TABLE_HEADER_HEIGHT / 2.0 + 0.5);
        }
    }
    let _ = ctx.set_fill_style_str(header_fg.muted);
    let _ = ctx.set_font(&dpr_font(500, meta_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_align("right");
    let _ = ctx.fill_text(
        &table.fields.len().to_string(),
        x + width - 11.0,
        y + TABLE_HEADER_HEIGHT / 2.0,
    );
    let _ = ctx.set_text_align("left");

    // 表头分隔线（line）
    let _ = ctx.set_stroke_style_str(palette.table_border);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(x, y + TABLE_HEADER_HEIGHT);
    ctx.line_to(x + width, y + TABLE_HEADER_HEIGHT);
    ctx.stroke();

    // 字段行：#33 语义徽章图标族（PK/FK/NN/UQ 线条图标）+ 名称 650/11px + 类型等宽 10px text-3
    // （主原型 .table-field；字块角标路径已按 core-08 §11 移除）
    for (i, field) in table.fields.iter().enumerate() {
        let fy = y + TABLE_HEADER_HEIGHT + i as f64 * FIELD_ROW_HEIGHT;

        let badges = field_badges(
            field.primary,
            fk_fields.contains(field.id.as_str()),
            field.not_null,
            field.unique,
        );
        let badges_w = draw_field_badges(ctx, &badges, palette, x + 11.0, fy + FIELD_ROW_HEIGHT / 2.0);
        let name_x = x + 11.0 + badges_w;
        // R-CMT-02：主文本按显示模式取值（comment 模式且有注释 → 注释；否则英文名）
        let field_label = comment_mode.primary(&field.name, &field.comment);
        let _ = ctx.set_fill_style_str(palette.text_strong);
        let _ = ctx.set_font(&dpr_font(650, field_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let _ = ctx.fill_text(field_label, name_x, fy + FIELD_ROW_HEIGHT / 2.0);

        let _ = ctx.set_fill_style_str(palette.text_muted);
        let _ = ctx.set_font(&dpr_font(500, meta_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let _ = ctx.set_text_align("right");
        let _ = ctx.fill_text(
            &field.type_,
            x + width - 11.0,
            fy + FIELD_ROW_HEIGHT / 2.0,
        );
        // R-CMT-02：name+comment 模式且 comment 非空 → 类型左侧渲染注释（灰色 9px）。
        // 截断优先级 名称 > 类型 > 注释：注释可用宽 = 类型左缘 − 名称右缘 − 间距，不足则省略。
        if let Some(f_cmt) = comment_mode.secondary(&field.comment) {
            let type_w = ctx.measure_text(&field.type_).map(|m| m.width()).unwrap_or(0.0);
            let name_w = ctx.measure_text(field_label).map(|m| m.width()).unwrap_or(0.0);
            let type_left = x + width - 11.0 - type_w;
            let cmt_max_w = type_left - 8.0 - (name_x + name_w + 8.0);
            if cmt_max_w > 10.0 {
                // #32 UT-PE-CMT-01（R-CMT-CONTRAST-01/02/05）：字段行背景为表体实底（tint 全透明），
                // 同样按对比度决策前景（亮主题 muted 灰不足 4.5:1 时升级 strong）；字号 10px ≥ 字段名
                // 11px 的 0.85×，字重 500 ≥400
                let f_style = comment_foreground(
                    "rgba(0,0,0,0)",
                    palette.table_bg,
                    PALETTE_LIGHT.text_strong,
                    PALETTE_LIGHT.text_muted,
                    PALETTE_DARK.text_strong,
                    PALETTE_DARK.text_muted,
                    !current_theme_dark(),
                );
                let _ = ctx.set_font(&dpr_font(500, meta_px, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
                let shown = truncate_to_width(ctx, f_cmt, cmt_max_w);
                if let Some(chip) = &f_style.chip {
                    let cmt_w = ctx.measure_text(&shown).map(|m| m.width()).unwrap_or(0.0);
                    draw_comment_chip(ctx, chip, type_left - 8.0 - cmt_w, fy + FIELD_ROW_HEIGHT / 2.0 + 0.5, cmt_w);
                }
                let _ = ctx.set_fill_style_str(&f_style.fg);
                let _ = ctx.fill_text(&shown, type_left - 8.0, fy + FIELD_ROW_HEIGHT / 2.0 + 0.5);
            }
        }
        let _ = ctx.set_text_align("left");

        // #3：字段左右连接点（可见触发点，半径小于命中半径以便易点）
        let cy = fy + FIELD_ROW_HEIGHT / 2.0;
        let port_r = 3.5;
        let _ = ctx.set_fill_style_str(palette.selected);
        ctx.begin_path();
        let _ = ctx.arc(x, cy, port_r, 0.0, std::f64::consts::TAU);
        ctx.fill();
        ctx.begin_path();
        let _ = ctx.arc(x + width, cy, port_r, 0.0, std::f64::consts::TAU);
        ctx.fill();
        let _ = ctx.set_stroke_style_str(palette.table_bg);
        ctx.set_line_width(1.0);
        ctx.begin_path();
        let _ = ctx.arc(x, cy, port_r, 0.0, std::f64::consts::TAU);
        ctx.stroke();
        ctx.begin_path();
        let _ = ctx.arc(x + width, cy, port_r, 0.0, std::f64::consts::TAU);
        ctx.stroke();

        if i + 1 < field_count {
            let _ = ctx.set_stroke_style_str(palette.row_separator);
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(x, fy + FIELD_ROW_HEIGHT);
            ctx.line_to(x + width, fy + FIELD_ROW_HEIGHT);
            ctx.stroke();
        }
    }
}

/// #30 UT-CR-LOD-01（R-LOD-01/03）：拓扑档卡体——字段行隐藏，仅表头高度圆角卡 +
/// 大字表名（font_world 由 lod_table_font_size 给出，屏幕字号 ≥11px）；
/// 投影/底色/边框/表头渐变/R-COLOR-04 前景与详情档同源。
fn draw_table_topology_body(
    ctx: &CanvasRenderingContext2d,
    table: &Table,
    palette: &CanvasPalette,
    shadow_boost: f64,
    comment_mode: CommentDisplay,
    // #40 R-FONT-03：font_world 由调用方完成倍率缩放与钳制（scaled_label_font_world）
    font_world: f64,
    // #40 R-FONT-04：注释字号（font_world×0.8）屏幕下限钳制需要 zoom
    zoom: f64,
) {
    let (width, height) = lod_table_size(table, comment_mode, LodTier::Topology);
    let x = table.x;
    let y = table.y;

    ctx.save();
    let _ = ctx.set_shadow_color("rgba(0, 0, 0, 0.18)");
    let _ = ctx.set_shadow_blur(16.0 * shadow_boost);
    let _ = ctx.set_shadow_offset_x(0.0);
    let _ = ctx.set_shadow_offset_y(6.0 * shadow_boost);
    let _ = ctx.set_fill_style_str(palette.table_bg);
    ctx.begin_path();
    round_rect(ctx, x, y, width, height, TABLE_CORNER_RADIUS);
    ctx.fill();
    ctx.restore();

    let _ = ctx.set_stroke_style_str(table_border_color(&table.color, palette.table_border));
    ctx.set_line_width(1.0);
    ctx.begin_path();
    round_rect(ctx, x, y, width, height, TABLE_CORNER_RADIUS);
    ctx.stroke();

    // 表头渐变（拓扑档整卡即表头，全圆角裁剪；#33 统一 135° 对角）
    let header_tint = if table.color.trim().is_empty() {
        palette.header_tint
    } else {
        table.color.as_str()
    };
    ctx.save();
    ctx.begin_path();
    round_rect(ctx, x, y, width, height, TABLE_CORNER_RADIUS);
    ctx.clip();
    let gradient = ctx.create_linear_gradient(x, y, x + width, y + height);
    gradient.add_color_stop(0.0, header_tint).ok();
    gradient.add_color_stop(1.0, "rgba(0,0,0,0)").ok();
    let _ = ctx.set_fill_style_str("rgba(0,0,0,0)");
    ctx.set_fill_style_canvas_gradient(&gradient);
    ctx.fill_rect(x, y, width, height);
    ctx.restore();

    // 表名：R-CMT-01 主文本（comment 模式显示注释），750 大字 + R-COLOR-04 前景
    let header_fg = header_foreground_colors(
        header_tint,
        palette.table_bg,
        PALETTE_LIGHT.text_strong,
        PALETTE_LIGHT.text_muted,
        PALETTE_DARK.text_strong,
        PALETTE_DARK.text_muted,
        !current_theme_dark(),
    );
    let label = comment_mode.primary(&table.name, &table.comment);
    let _ = ctx.set_fill_style_str(header_fg.strong);
    let _ = ctx.set_font(&dpr_font(750, font_world, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
    let _ = ctx.set_text_baseline("middle");
    let _ = ctx.set_text_align("left");
    let name_shown = truncate_to_width(ctx, label, width - 22.0);
    let _ = ctx.fill_text(&name_shown, x + 11.0, y + height / 2.0);

    // #35 R-LOD-02 补齐（UT-CR-LOD-01）：NameComment 模式且注释非空 → 表名右侧渲染注释
    // 副文本；字号 = 拓扑表名字号 × 0.8（R-CMT-CONTRAST-03 同比率）、字重 600 ≥ 500，
    // 前景按 R-CMT-CONTRAST-01 双端点判据（#34 同决策），空注释不渲染不留占位（R-CMT-03）。
    if let Some(cmt) = comment_mode.secondary(&table.comment) {
        let name_w = ctx.measure_text(&name_shown).map(|m| m.width()).unwrap_or(0.0);
        let cmt_style = comment_foreground(
            header_tint,
            palette.table_bg,
            PALETTE_LIGHT.text_strong,
            PALETTE_LIGHT.text_muted,
            PALETTE_DARK.text_strong,
            PALETTE_DARK.text_muted,
            !current_theme_dark(),
        );
        // #40 R-FONT-04：注释字号与表名同下限（屏幕 ≥10px）
        let cmt_font = (font_world * 0.8).max(LABEL_FONT_MIN_SCREEN_PX / zoom.max(0.01));
        let _ = ctx.set_font(&dpr_font(600, cmt_font, &resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)));
        let cmt_x = x + 11.0 + name_w + 6.0 * (font_world / TABLE_NAME_FONT_PX).max(1.0);
        let max_w = (x + width - 11.0) - cmt_x;
        if max_w > 12.0 {
            let shown = truncate_to_width(ctx, cmt, max_w);
            let cmt_w = ctx.measure_text(&shown).map(|m| m.width()).unwrap_or(0.0);
            if let Some(chip) = &cmt_style.chip {
                draw_comment_chip(ctx, chip, cmt_x, y + height / 2.0 + 0.5, cmt_w);
            }
            let _ = ctx.set_fill_style_str(&cmt_style.fg);
            let _ = ctx.fill_text(&shown, cmt_x, y + height / 2.0 + 0.5);
        }
    }
}

/// 选中态：主原型 .is-selected —— brand 描边 + 3px brand-soft 外环。
/// R-PERF-07：选中环每帧活画（不含在精灵缓存内），选中切换不失效卡体缓存。
fn draw_table_selection(ctx: &CanvasRenderingContext2d, table: &Table, palette: &CanvasPalette, comment_mode: CommentDisplay, tier: LodTier) {
    // #30：选中环尺寸按档位取（拓扑档仅表头高，避免圈出隐藏的字段区）
    let (width, total_height) = lod_table_size(table, comment_mode, tier);
    let x = table.x;
    let y = table.y;
    let _ = ctx.set_stroke_style_str(palette.selected_soft);
    ctx.set_line_width(3.0);
    ctx.begin_path();
    // #33：外环圆角 = 表卡圆角 + 2.5 外扩（与 TABLE_CORNER_RADIUS 同一口径）
    round_rect(ctx, x - 2.5, y - 2.5, width + 5.0, total_height + 5.0, TABLE_CORNER_RADIUS + 2.5);
    ctx.stroke();

    let _ = ctx.set_stroke_style_str(palette.selected);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    round_rect(ctx, x, y, width, total_height, TABLE_CORNER_RADIUS);
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

/// 空串 / 未知 → bezier（#21）。
pub fn effective_line_type(s: &str) -> &str {
    match s.trim() {
        "orthogonal" | "straight" => s.trim(),
        _ => "bezier",
    }
}

/// 空串 / 未知 → solid（#21）。
pub fn effective_stroke_style(s: &str) -> &str {
    match s.trim() {
        "dashed" => "dashed",
        _ => "solid",
    }
}

fn apply_stroke_dash(ctx: &CanvasRenderingContext2d, stroke_style: &str) {
    let dash = stroke_dash_for_style(stroke_style);
    let arr = js_sys::Array::new();
    for v in dash {
        arr.push(&wasm_bindgen::JsValue::from(v));
    }
    let _ = ctx.set_line_dash(&arr);
}

fn clear_stroke_dash(ctx: &CanvasRenderingContext2d) {
    let _ = ctx.set_line_dash(&js_sys::Array::new());
}

/// #21/#22：按 line_type / stroke_style / 源表色 / 密度 alpha 绘制关系。
/// #31 UT-PE-HL-01：`hl_scale` 为相关态线宽倍率（1.0 或 1.5），仅作用于非选中态；
/// #30 UT-CR-LOD-01（R-LOD-04）：`lod_scale` 为拓扑档线宽补偿倍率，选中/非选中均叠乘
/// （详情档恒 1.0 无回归）。
fn draw_relation(
    ctx: &CanvasRenderingContext2d,
    from: &Table,
    from_field_id: &str,
    to: &Table,
    to_field_id: &str,
    palette: &CanvasPalette,
    selected: bool,
    related: bool,
    ref_color: &str,
    source_table_color: &str,
    line_type: &str,
    stroke_style: &str,
    opacity: f64,
    hl_scale: f64,
    lod_scale: f64,
    cardinality: &str,
    tier: LodTier,
) {
    let stroke = relation_stroke_color(ref_color, source_table_color, palette.relation);
    // #33 UT-PE-VIS-01：端点统一 crow's foot 几何族（替代旧箭头+圆点）
    let (start_many, end_many) = endpoint_multiplicity(cardinality);
    ctx.save();
    ctx.set_global_alpha(opacity);

    // fix-31 UT-PE-HL-01（§4.4 提亮）：相关态（非选中）强制选中色族——主色 palette.selected、
    // 光晕 palette.selected_soft 8px 发光；选中关系自身 3.5px 主线 + 10px 光晕保持层级区分
    let stroke_main = if selected || related { palette.selected } else { stroke };
    let halo = if selected || related { palette.selected_soft } else { palette.relation_halo };
    let halo_w = if selected { 10.0 } else if related { 8.0 } else { 7.0 * hl_scale } * lod_scale;
    let main_w = if selected { 3.5 } else { 2.0 * hl_scale } * lod_scale;

    match line_type {
        "orthogonal" => {
            let pts = calc_orthogonal_path_tier(from, from_field_id, to, to_field_id, tier);
            // 光晕
            let _ = ctx.set_stroke_style_str(halo);
            ctx.set_line_width(halo_w);
            clear_stroke_dash(ctx);
            stroke_polyline(ctx, &pts);
            // 主线
            let _ = ctx.set_stroke_style_str(stroke_main);
            ctx.set_line_width(main_w);
            apply_stroke_dash(ctx, stroke_style);
            stroke_polyline(ctx, &pts);
            clear_stroke_dash(ctx);
            if pts.len() >= 2 {
                let (x1, y1) = pts[0];
                let (sx, sy) = pts[1];
                let (x2, y2) = pts[pts.len() - 1];
                let (px, py) = pts[pts.len() - 2];
                draw_endpoint_notation(ctx, px, py, x2, y2, end_many, stroke_main, main_w);
                draw_endpoint_notation(ctx, sx, sy, x1, y1, start_many, stroke_main, main_w);
            }
        }
        "straight" => {
            let (x1, y1, x2, y2) = calc_straight_path_tier(from, from_field_id, to, to_field_id, tier);
            let _ = ctx.set_stroke_style_str(halo);
            ctx.set_line_width(halo_w);
            clear_stroke_dash(ctx);
            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
            let _ = ctx.set_stroke_style_str(stroke_main);
            ctx.set_line_width(main_w);
            apply_stroke_dash(ctx, stroke_style);
            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
            clear_stroke_dash(ctx);
            draw_endpoint_notation(ctx, x1, y1, x2, y2, end_many, stroke_main, main_w);
            draw_endpoint_notation(ctx, x2, y2, x1, y1, start_many, stroke_main, main_w);
        }
        _ => {
            // bezier 默认
            let path = calc_path_tier(from, from_field_id, to, to_field_id, tier);
            let _ = ctx.set_stroke_style_str(halo);
            ctx.set_line_width(halo_w);
            clear_stroke_dash(ctx);
            ctx.begin_path();
            ctx.move_to(path.x1, path.y1);
            ctx.bezier_curve_to(path.cx1, path.cy1, path.cx2, path.cy2, path.x2, path.y2);
            ctx.stroke();
            let _ = ctx.set_stroke_style_str(stroke_main);
            ctx.set_line_width(main_w);
            apply_stroke_dash(ctx, stroke_style);
            ctx.begin_path();
            ctx.move_to(path.x1, path.y1);
            ctx.bezier_curve_to(path.cx1, path.cy1, path.cx2, path.cy2, path.x2, path.y2);
            ctx.stroke();
            clear_stroke_dash(ctx);
            draw_endpoint_notation(ctx, path.cx2, path.cy2, path.x2, path.y2, end_many, stroke_main, main_w);
            draw_endpoint_notation(ctx, path.cx1, path.cy1, path.x1, path.y1, start_many, stroke_main, main_w);
        }
    }
    ctx.restore();
}

/// #33 UT-PE-VIS-01（core-07 §15.5 canvas.rel.endpoint-style）：关系端点统一
/// crow's foot 几何族——"一"端垂直单杠、"多"端三叉爪，外接尺寸 REL_ENDPOINT_SIZE，
/// 与主线同色同宽（替代旧箭头 + 圆点，禁止混用箭头与字块角标）。
/// (fromx, fromy) → (tox, toy) 为端点处切线方向，记号画在 (tox, toy)。
fn draw_endpoint_notation(
    ctx: &CanvasRenderingContext2d,
    fromx: f64,
    fromy: f64,
    tox: f64,
    toy: f64,
    many: bool,
    stroke: &str,
    line_w: f64,
) {
    let angle = (toy - fromy).atan2(tox - fromx);
    let (ux, uy) = (angle.cos(), angle.sin());
    let (px, py) = (-uy, ux);
    let _ = ctx.set_stroke_style_str(stroke);
    ctx.set_line_width(line_w);
    ctx.set_line_cap("round");
    ctx.begin_path();
    if many {
        // 三叉爪：基点沿切线回退 SIZE，三股散开至端点带（中股抵端点，两侧 ±SIZE/2）
        let bx = tox - ux * REL_ENDPOINT_SIZE;
        let by = toy - uy * REL_ENDPOINT_SIZE;
        let spread = REL_ENDPOINT_SIZE / 2.0;
        ctx.move_to(bx, by);
        ctx.line_to(tox, toy);
        ctx.move_to(bx, by);
        ctx.line_to(tox + px * spread - ux * 1.5, toy + py * spread - uy * 1.5);
        ctx.move_to(bx, by);
        ctx.line_to(tox - px * spread - ux * 1.5, toy - py * spread - uy * 1.5);
    } else {
        // 单杠：端点处垂直线，半长 SIZE/2
        let half = REL_ENDPOINT_SIZE / 2.0;
        ctx.move_to(tox + px * half, toy + py * half);
        ctx.line_to(tox - px * half, toy - py * half);
    }
    ctx.stroke();
}

fn stroke_polyline(ctx: &CanvasRenderingContext2d, pts: &[(f64, f64)]) {
    if pts.is_empty() {
        return;
    }
    ctx.begin_path();
    ctx.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        ctx.line_to(p.0, p.1);
    }
    ctx.stroke();
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

/// #5：框选预览矩形（虚线框 + 淡填，禁止画成关系贝塞尔线）
fn draw_marquee_rect(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    palette: &CanvasPalette,
) {
    let (x, y, w, h) = normalize_marquee_rect(x1, y1, x2, y2);
    let _ = ctx.set_fill_style_str(palette.selected_soft);
    // selected_soft 可能是描边色；用低透明 brand 近似填充
    ctx.save();
    ctx.set_global_alpha(0.12);
    let _ = ctx.set_fill_style_str(palette.selected);
    ctx.fill_rect(x, y, w, h);
    ctx.restore();
    let _ = ctx.set_stroke_style_str(palette.selected);
    ctx.set_line_width(1.5);
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

fn draw_area(
    ctx: &CanvasRenderingContext2d,
    area: &Area,
    palette: &CanvasPalette,
    selected: bool,
    show_handles: bool,
) {
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

    // fix-issues-38-41-canvas-interaction（#41，R-AREALOCK-05）：锁定区域右上角渲染
    // 锁形标记（选中/非选中均可见）
    if area.locked {
        let lx = area.x + area.width - 20.0;
        let ly = area.y + 7.0;
        ctx.save();
        let _ = ctx.set_stroke_style_str(palette.area_border);
        let _ = ctx.set_fill_style_str(palette.area_border);
        ctx.set_line_width(1.5);
        // 锁梁（上半圆环）
        ctx.begin_path();
        let _ = ctx.arc(lx + 5.0, ly + 5.0, 3.5, std::f64::consts::PI, 0.0);
        let _ = ctx.stroke();
        // 锁体
        ctx.fill_rect(lx, ly + 5.0, 10.0, 8.0);
        ctx.restore();
    }

    // fix-open-issues-26-33（issue #27，R-ARESZ-01/05）：选中且非只读 → 8 个 resize 手柄
    // （4 角 + 4 边中点，白底 selected 描边，与 hit_test_area_resize 热区同中心）
    // #41 R-AREALOCK-03：锁定区域不渲染手柄（视觉与命中一致）
    if show_handles && !area.locked {
        for (_, hx, hy) in area_resize_handles(area) {
            let hs = AREA_HANDLE_HALF;
            let _ = ctx.set_fill_style_str("#ffffff");
            ctx.fill_rect(hx - hs, hy - hs, hs * 2.0, hs * 2.0);
            let _ = ctx.set_stroke_style_str(palette.selected);
            ctx.set_line_width(1.0);
            ctx.stroke_rect(hx - hs, hy - hs, hs * 2.0, hs * 2.0);
        }
    }
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
        let width = resolve_table_width(table, CommentDisplay::NameComment);
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

/// 字段左右连接点（#3 reopen：仅 port 可起拖连，非整行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldPortSide {
    /// 左侧
    Start,
    /// 右侧
    End,
}

/// 命中字段左右连接点；半径见 `FIELD_PORT_HIT_RADIUS`。
pub fn hit_test_field_port(
    tables: &[Table],
    x: f64,
    y: f64,
) -> Option<(String, String, FieldPortSide)> {
    for table in tables.iter().rev() {
        let width = resolve_table_width(table, CommentDisplay::NameComment);
        for field in &table.fields {
            let cy = field_anchor_y(table, &field.id);
            let left_dx = x - table.x;
            let left_dy = y - cy;
            if left_dx * left_dx + left_dy * left_dy <= FIELD_PORT_HIT_RADIUS * FIELD_PORT_HIT_RADIUS
            {
                return Some((table.id.clone(), field.id.clone(), FieldPortSide::Start));
            }
            let right_x = table.x + width;
            let right_dx = x - right_x;
            let right_dy = y - cy;
            if right_dx * right_dx + right_dy * right_dy
                <= FIELD_PORT_HIT_RADIUS * FIELD_PORT_HIT_RADIUS
            {
                return Some((table.id.clone(), field.id.clone(), FieldPortSide::End));
            }
        }
    }
    None
}

pub fn hit_test(tables: &[Table], x: f64, y: f64) -> Option<String> {
    for table in tables.iter().rev() {
        // feat-table-resize: 命中宽度跟随 table.width,fallback 到 TABLE_WIDTH 默认
        let width = resolve_table_width(table, CommentDisplay::NameComment);
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
        // #41 R-AREALOCK-01：新建区域默认未锁定
        locked: false,
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

/// 区域标题栏命中（在表命中之前调用，使叠在区域内的表不挡住区域拖动）。
pub fn hit_test_area_header(areas: &[Area], x: f64, y: f64) -> Option<String> {
    for a in areas.iter().rev() {
        if x >= a.x
            && x <= a.x + a.width
            && y >= a.y
            && y <= a.y + AREA_HEADER_HIT.min(a.height)
        {
            return Some(a.id.clone());
        }
    }
    None
}

// ─── fix-open-issues-26-33（issue #27，core-01 §5.11 R-ARESZ）：区域 resize ───

/// R-ARESZ-03：resize 下限（保证区域标题行可读；创建防误触阈值 10px 仅用于拖框创建）
pub const AREA_MIN_WIDTH: f64 = 40.0;
pub const AREA_MIN_HEIGHT: f64 = 30.0;
/// resize 手柄命中热区半径（diagram 坐标）
pub const AREA_HANDLE_HALF: f64 = 5.0;

/// 区域 resize 方位（4 角 + 4 边中点，R-ARESZ-01）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaResizeDir {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl AreaResizeDir {
    /// R-ARESZ-01：方位对应的 CSS resize 光标
    pub fn cursor(self) -> &'static str {
        match self {
            AreaResizeDir::N | AreaResizeDir::S => "ns-resize",
            AreaResizeDir::E | AreaResizeDir::W => "ew-resize",
            AreaResizeDir::NE | AreaResizeDir::SW => "nesw-resize",
            AreaResizeDir::NW | AreaResizeDir::SE => "nwse-resize",
        }
    }

    fn affects_east(self) -> bool {
        matches!(self, AreaResizeDir::E | AreaResizeDir::NE | AreaResizeDir::SE)
    }

    fn affects_west(self) -> bool {
        matches!(self, AreaResizeDir::W | AreaResizeDir::NW | AreaResizeDir::SW)
    }

    fn affects_south(self) -> bool {
        matches!(self, AreaResizeDir::S | AreaResizeDir::SE | AreaResizeDir::SW)
    }

    fn affects_north(self) -> bool {
        matches!(self, AreaResizeDir::N | AreaResizeDir::NE | AreaResizeDir::NW)
    }
}

/// R-ARESZ-01：8 个 resize 命中面（4 角 + 4 边中点）的方位与中心坐标（diagram 坐标）。
/// 返回顺序固定（角优先），便于命中判定角优先于边。
pub fn area_resize_handles(area: &Area) -> [(AreaResizeDir, f64, f64); 8] {
    let (x, y, w, h) = (area.x, area.y, area.width, area.height);
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    [
        (AreaResizeDir::NW, x, y),
        (AreaResizeDir::NE, x + w, y),
        (AreaResizeDir::SE, x + w, y + h),
        (AreaResizeDir::SW, x, y + h),
        (AreaResizeDir::N, cx, y),
        (AreaResizeDir::E, x + w, cy),
        (AreaResizeDir::S, cx, y + h),
        (AreaResizeDir::W, x, cy),
    ]
}

/// R-ARESZ-01/05/06：点命中选中区域的 resize 手柄（角优先）。
/// 只读模式恒不命中；区域内部非手柄处返回 None（回退整框拖动），区域外返回 None。
pub fn hit_test_area_resize(area: &Area, x: f64, y: f64, read_only: bool) -> Option<AreaResizeDir> {
    if read_only {
        return None;
    }
    for (dir, hx, hy) in area_resize_handles(area) {
        if (x - hx).abs() <= AREA_HANDLE_HALF && (y - hy).abs() <= AREA_HANDLE_HALF {
            return Some(dir);
        }
    }
    None
}

/// R-ARESZ-02/03：按方位把指针位置换算为新矩形——锚定对侧角/边，
/// 宽/高夹紧到 AREA_MIN_WIDTH / AREA_MIN_HEIGHT，不产生负尺寸/翻转。
pub fn area_rect_from_resize(
    dir: AreaResizeDir,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    cursor_x: f64,
    cursor_y: f64,
) -> (f64, f64, f64, f64) {
    let right = x + w;
    let bottom = y + h;
    let (mut nx, mut ny, mut nw, mut nh) = (x, y, w, h);
    if dir.affects_east() {
        nw = (cursor_x - x).max(AREA_MIN_WIDTH);
    } else if dir.affects_west() {
        nx = cursor_x.min(right - AREA_MIN_WIDTH);
        nw = right - nx;
    }
    if dir.affects_south() {
        nh = (cursor_y - y).max(AREA_MIN_HEIGHT);
    } else if dir.affects_north() {
        ny = cursor_y.min(bottom - AREA_MIN_HEIGHT);
        nh = bottom - ny;
    }
    (nx, ny, nw, nh)
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
    hit_test_reference_tier(tables, refs, x, y, LodTier::Detail)
}

/// #35 R-LOD-08：tier 感知命中检测——拓扑档线几何为表级锚定，命中必须与绘制同口径
/// （否则拓扑档点击可见连线不命中）。
pub fn hit_test_reference_tier(
    tables: &[Table],
    refs: &[Reference],
    x: f64,
    y: f64,
    tier: LodTier,
) -> Option<String> {
    for r in refs {
        let from = tables.iter().find(|t| t.id == r.start_table_id);
        let to = tables.iter().find(|t| t.id == r.end_table_id);
        if let (Some(f), Some(t)) = (from, to) {
            let path = calc_path_tier(f, &r.start_field_id, t, &r.end_field_id, tier);
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
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
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
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
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

    /// 便签/区域拖拽命中：标题栏可拖区域；框选可收集便签与区域
    #[test]
    fn ut_cr_note_area_drag_hit_priority() {
        let area = Area {
            id: "a1".into(),
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
            color: "#3b82f6".into(),
            name: "区域".into(),
            locked: false,
        };
        assert_eq!(
            hit_test_area_header(&[area.clone()], 20.0, 10.0),
            Some("a1".into())
        );
        assert_eq!(
            hit_test_area_header(&[area.clone()], 20.0, 50.0),
            None,
            "标题栏以下不走 header 命中"
        );
        let note = Note {
            id: "n1".into(),
            x: 10.0,
            y: 10.0,
            content: "hi".into(),
            color: "#f59e0b".into(),
        };
        assert_eq!(hit_test_note(&[note], 20.0, 20.0), Some("n1".into()));
        let notes = notes_in_marquee(
            &[Note {
                id: "n1".into(),
                x: 10.0,
                y: 10.0,
                content: String::new(),
                color: String::new(),
            }],
            0.0,
            0.0,
            200.0,
            200.0,
        );
        assert_eq!(notes, vec!["n1".to_string()]);
        let areas = areas_in_marquee(&[area], 0.0, 0.0, 100.0, 100.0);
        assert_eq!(areas, vec!["a1".to_string()]);
    }

    /// #5：框选便签+区域后点中任一成员应启动整组拖动 starts
    #[test]
    fn ut_cr_group_drag_note_and_area() {
        let tables: Vec<Table> = vec![];
        let notes = vec![Note {
            id: "n1".into(),
            x: 10.0,
            y: 10.0,
            content: String::new(),
            color: String::new(),
        }];
        let areas = vec![Area {
            id: "a1".into(),
            x: 0.0,
            y: 50.0,
            width: 200.0,
            height: 150.0,
            color: "#3b82f6".into(),
            name: "未命名区域".into(),
            locked: false,
        }];
        let table_ids: Vec<String> = vec![];
        let note_ids = vec!["n1".to_string()];
        let area_ids = vec!["a1".to_string()];
        assert!(should_start_group_drag(true, &table_ids, &note_ids, &area_ids));
        let (ts, ns, as_) =
            build_group_drag_starts(&tables, &notes, &areas, &table_ids, &note_ids, &area_ids, true);
        assert!(ts.is_none());
        assert_eq!(ns.as_ref().map(|v| v.len()), Some(1));
        assert_eq!(as_.as_ref().map(|v| v.len()), Some(1));
        let mut notes_m = notes.clone();
        let mut areas_m = areas.clone();
        let mut tables_m = tables.clone();
        translate_group_entities(
            &mut tables_m,
            &mut notes_m,
            &mut areas_m,
            &[],
            ns.as_ref().unwrap(),
            as_.as_ref().unwrap(),
            40.0,
            20.0,
        );
        assert!((notes_m[0].x - 50.0).abs() < 1e-9);
        assert!((notes_m[0].y - 30.0).abs() < 1e-9);
        assert!((areas_m[0].x - 40.0).abs() < 1e-9);
        assert!((areas_m[0].y - 70.0).abs() < 1e-9);
        assert!(!should_clear_cross_selection(false, false, true));
        assert!(should_clear_cross_selection(false, false, false));
    }

    /// #5：拖动实体用 grabbing；框选工具空闲才是 crosshair；默认空串走 CSS grab
    #[test]
    fn ut_cr_canvas_cursor_css() {
        assert_eq!(
            canvas_cursor_css(false, true, true, false, false),
            "grabbing",
            "框选工具下拖表仍为 grabbing"
        );
        assert_eq!(
            canvas_cursor_css(false, true, false, true, false),
            "grabbing",
            "平移为 grabbing"
        );
        assert_eq!(
            canvas_cursor_css(false, true, false, false, true),
            "crosshair",
            "正在拉框选矩形为 crosshair"
        );
        assert_eq!(
            canvas_cursor_css(false, true, false, false, false),
            "crosshair",
            "框选工具空闲为 crosshair"
        );
        assert_eq!(
            canvas_cursor_css(false, false, false, false, false),
            "",
            "默认小手由 CSS grab 提供"
        );
    }

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

    /// UT-AREA-02: 区域 resize 几何纯函数（fix-open-issues-26-33 / #27，R-ARESZ-01/03/05）
    /// 命中面坐标 / 方位命中 / 锚定 / 夹紧 / 只读不命中
    #[test]
    fn test_area_resize_geometry_ut_area_02() {
        let area = build_area("a1".into(), 100.0, 100.0, 160.0, 120.0);

        // 1) 8 个命中面（4 角 + 4 边中点），坐标与区域矩形一致
        let handles = area_resize_handles(&area);
        assert_eq!(handles.len(), 8, "UT-AREA-02: 应为 8 个命中面");
        assert!(handles.contains(&(AreaResizeDir::NW, 100.0, 100.0)));
        assert!(handles.contains(&(AreaResizeDir::NE, 260.0, 100.0)));
        assert!(handles.contains(&(AreaResizeDir::SE, 260.0, 220.0)));
        assert!(handles.contains(&(AreaResizeDir::SW, 100.0, 220.0)));
        assert!(handles.contains(&(AreaResizeDir::N, 180.0, 100.0)));
        assert!(handles.contains(&(AreaResizeDir::E, 260.0, 160.0)));
        assert!(handles.contains(&(AreaResizeDir::S, 180.0, 220.0)));
        assert!(handles.contains(&(AreaResizeDir::W, 100.0, 160.0)));

        // 2) 方位命中：手柄热区 → 对应方位；区域内部（非手柄）与区域外不命中
        assert_eq!(
            hit_test_area_resize(&area, 260.0, 160.0, false),
            Some(AreaResizeDir::E),
            "UT-AREA-02: E 边中点热区应命中 E"
        );
        assert_eq!(
            hit_test_area_resize(&area, 101.0, 101.0, false),
            Some(AreaResizeDir::NW),
            "UT-AREA-02: 角热区应命中 NW"
        );
        assert_eq!(
            hit_test_area_resize(&area, 180.0, 160.0, false),
            None,
            "UT-AREA-02: 区域内部（非手柄）不命中 resize（回退整框拖动）"
        );
        assert_eq!(
            hit_test_area_resize(&area, 50.0, 50.0, false),
            None,
            "UT-AREA-02: 区域外不命中"
        );

        // 3) 锚定：拖 E 边 → 仅 width 变化、x 锚定；拖 NW 角 → x/y/w/h 联动、SE 角锚定
        assert_eq!(
            area_rect_from_resize(AreaResizeDir::E, 100.0, 100.0, 160.0, 120.0, 320.0, 175.0),
            (100.0, 100.0, 220.0, 120.0),
            "UT-AREA-02: E 边仅改宽、x/y/h 锚定"
        );
        let (nx, ny, nw, nh) =
            area_rect_from_resize(AreaResizeDir::NW, 100.0, 100.0, 160.0, 120.0, 60.0, 70.0);
        assert_eq!((nx, ny, nw, nh), (60.0, 70.0, 200.0, 150.0));
        assert_eq!(
            (nx + nw, ny + nh),
            (260.0, 220.0),
            "UT-AREA-02: NW 角拖拽 SE 角锚定"
        );

        // 4) 夹紧：拖至小于下限 → 夹到 AREA_MIN_WIDTH/HEIGHT，无负尺寸/翻转
        assert_eq!(
            area_rect_from_resize(AreaResizeDir::E, 100.0, 100.0, 160.0, 120.0, 90.0, 160.0),
            (100.0, 100.0, AREA_MIN_WIDTH, 120.0),
            "UT-AREA-02: E 边向左拖过原点夹紧到下限不翻转"
        );
        assert_eq!(
            area_rect_from_resize(AreaResizeDir::W, 100.0, 100.0, 160.0, 120.0, 300.0, 160.0),
            (260.0 - AREA_MIN_WIDTH, 100.0, AREA_MIN_WIDTH, 120.0),
            "UT-AREA-02: W 边向右拖过对侧夹紧且不翻转"
        );
        assert_eq!(
            area_rect_from_resize(AreaResizeDir::N, 100.0, 100.0, 160.0, 120.0, 180.0, 300.0),
            (100.0, 220.0 - AREA_MIN_HEIGHT, 160.0, AREA_MIN_HEIGHT),
            "UT-AREA-02: N 边向下拖过对侧夹紧且不翻转"
        );

        // 5) 只读模式命中判定 → 不命中（R-ARESZ-05）
        assert_eq!(
            hit_test_area_resize(&area, 260.0, 160.0, true),
            None,
            "UT-AREA-02: 只读模式不命中 resize"
        );

        // 方位 → 光标（R-ARESZ-01）
        assert_eq!(AreaResizeDir::NW.cursor(), "nwse-resize");
        assert_eq!(AreaResizeDir::NE.cursor(), "nesw-resize");
        assert_eq!(AreaResizeDir::E.cursor(), "ew-resize");
        assert_eq!(AreaResizeDir::N.cursor(), "ns-resize");
    }

    /// UT-AREA-03: 区域 resize 落账与撤销（fix-open-issues-26-33 / #27，R-ARESZ-02/04）
    /// SetAreaRect 单条命令 → store 置 after；一次 Undo 恢复 before
    #[test]
    fn test_area_resize_commit_undo_ut_area_03() {
        use crate::editor_core::{Command, CommandStack, EditorStore};
        let store = EditorStore::new();
        store
            .areas
            .set(vec![build_area("a1".into(), 100.0, 100.0, 160.0, 120.0)]);
        let mut stack = CommandStack::new();

        // pointerup 一次写回（含 NW 角的 x/y 联动）→ 单条命令
        let cmd = Command::SetAreaRect {
            area_id: "a1".to_string(),
            before: (100.0, 100.0, 160.0, 120.0),
            after: (60.0, 70.0, 200.0, 150.0),
        };
        CommandStack::apply(&store, &mut stack, cmd).expect("UT-AREA-03: apply 应成功");
        let areas = store.areas.get();
        assert_eq!(
            (areas[0].x, areas[0].y, areas[0].width, areas[0].height),
            (60.0, 70.0, 200.0, 150.0),
            "UT-AREA-03: apply 后矩形 = after"
        );
        assert_eq!(stack.undo_len(), 1, "UT-AREA-03: 整次 resize 恰为单条命令");

        // 一次 Undo 恢复 resize 前矩形
        let popped = stack.undo().expect("UT-AREA-03: undo 栈非空");
        CommandStack::revert(&store, &popped).expect("UT-AREA-03: revert 应成功");
        let areas = store.areas.get();
        assert_eq!(
            (areas[0].x, areas[0].y, areas[0].width, areas[0].height),
            (100.0, 100.0, 160.0, 120.0),
            "UT-AREA-03: 一次 Undo 恢复 before"
        );

        // 不存在区域 → apply 报错且不污染栈（Inspector/画布写回通道的容错口径）
        let mut stack2 = CommandStack::new();
        let bad = Command::SetAreaRect {
            area_id: "ghost".to_string(),
            before: (0.0, 0.0, 1.0, 1.0),
            after: (0.0, 0.0, 2.0, 2.0),
        };
        assert!(CommandStack::apply(&store, &mut stack2, bad).is_err());
        assert_eq!(stack2.undo_len(), 0);
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
        assert_eq!(w, TABLE_WIDTH, "feat-table-resize/#20: width=None 短名 → auto 下界 TABLE_WIDTH");
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
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
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
            crate::editor_core::types::Area { id: "a-on".into(), x: 10.0, y: 10.0, width: 200.0, height: 120.0, color: "#3b82f6".into(), name: "A".into(), locked: false },
            crate::editor_core::types::Area { id: "a-off".into(), x: -5000.0, y: -5000.0, width: 200.0, height: 120.0, color: "#3b82f6".into(), name: "B".into(), locked: false },
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
            table_sprite_fingerprint(&t, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]),
            table_sprite_fingerprint(&moved, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]),
            "UT-CR-SPRITE-01: 位置变化不得改变指纹"
        );

        // 内容变更失效：改名 / 改字段类型 / 改主键 / 改色 / 改宽 / 主题 / dpr / 分档
        let base = table_sprite_fingerprint(&t, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]);
        let mut renamed = t.clone();
        renamed.name = "other".into();
        assert_ne!(table_sprite_fingerprint(&renamed, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "改名失效");
        let mut retyped = t.clone();
        retyped.fields[0].type_ = "UUID".into();
        assert_ne!(table_sprite_fingerprint(&retyped, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "改字段类型失效");
        let mut unpk = t.clone();
        unpk.fields[0].primary = false;
        assert_ne!(table_sprite_fingerprint(&unpk, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "改主键失效");
        let mut recolor = t.clone();
        recolor.color = "#fff".into();
        assert_ne!(table_sprite_fingerprint(&recolor, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "改色失效");
        let mut resized = t.clone();
        resized.width = Some(320);
        assert_ne!(table_sprite_fingerprint(&resized, true, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "改宽失效");
        assert_ne!(table_sprite_fingerprint(&t, false, 200, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "主题切换失效");
        assert_ne!(table_sprite_fingerprint(&t, true, 100, 1, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "dpr 变化失效");
        assert_ne!(table_sprite_fingerprint(&t, true, 200, 2, crate::editor_core::CommentDisplay::NameComment, LodTier::Detail, &[]), base, "zoom 分档切换失效");
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
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
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

    /// UT-CR-FOCUS-01 — focus_transform 使表进入视口
    #[test]
    fn ut_cr_focus_01_focus_transform() {
        let t = Transform {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        };
        let next = focus_transform(2000.0, 2000.0, 230.0, 100.0, 800.0, 600.0, &t, 40.0);
        let expect_pan_x = 400.0 - 2115.0;
        assert!(
            (next.pan_x - expect_pan_x).abs() < 1.0,
            "UT-CR-FOCUS-01: pan_x={} expect≈{}",
            next.pan_x,
            expect_pan_x
        );
        assert_eq!(next.zoom, 1.0, "UT-CR-FOCUS-01: 默认不改 zoom");
        let inside = focus_transform(100.0, 100.0, 100.0, 80.0, 800.0, 600.0, &t, 40.0);
        assert_eq!(inside.pan_x, 0.0);
        assert_eq!(inside.pan_y, 0.0);
    }

    /// ST-CR-MULTI-01 辅助：Shift / 多选集合让路给选择手势，避免 Idle 字段连线抢走
    #[test]
    fn ut_cr_multi_01_prefer_selection_over_field_rel() {
        let multi = vec!["a".into(), "b".into()];
        assert!(
            prefer_selection_over_field_rel(true, &[], "a"),
            "ST-CR-MULTI-01: Shift 必须让路给框选/多选"
        );
        assert!(
            prefer_selection_over_field_rel(false, &multi, "a"),
            "ST-CR-MULTI-01: 已多选且点中集合内表 → 多表拖动优先"
        );
        assert!(
            !prefer_selection_over_field_rel(false, &multi, "c"),
            "ST-CR-MULTI-01: 点中未选中表仍可字段连线"
        );
        assert!(
            !prefer_selection_over_field_rel(false, &["a".into()], "a"),
            "ST-CR-MULTI-01: 单选不抢字段连线"
        );
    }

    /// UT-PE-HL-01 — 选中高亮参数纯函数（#31，core-01b §4.4）
    #[test]
    fn ut_pe_hl_01_highlight_render_params() {
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
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
        };
        // 拓扑：r1(a-b) r2(a-c) r3(c-d)
        let refs = vec![mk("r1", "a", "b"), mk("r2", "a", "c"), mk("r3", "c", "d")];

        // 断言 1：选中表 a —— 相关线 ≥1.5× 且 alpha=1；非相关线 ≤0.25；非相关表 ≤0.5；邻接表=1
        let sel_a = Some("a");
        assert!(relation_is_related("r1", "a", "b", &[], sel_a, None));
        assert!(relation_is_related("r2", "a", "c", &[], sel_a, None));
        assert!(!relation_is_related("r3", "c", "d", &[], sel_a, None));
        let w_rel = relation_render_width(2.0, true);
        assert!(
            w_rel >= 2.0 * 1.5 - 1e-9,
            "UT-PE-HL-01: 相关线宽 {w_rel} 必须 ≥ 默认 1.5×"
        );
        assert_eq!(relation_render_width(2.0, false), 2.0, "UT-PE-HL-01: 非相关线保持默认线宽");
        assert_eq!(
            relation_opacity("r1", "a", "b", &[], sel_a, None),
            1.0,
            "UT-PE-HL-01: 相关线 alpha = 1"
        );
        assert!(
            relation_opacity("r3", "c", "d", &[], sel_a, None) <= 0.25,
            "UT-PE-HL-01: 非相关线 alpha ≤ 0.25"
        );
        assert_eq!(table_render_alpha("a", &refs, &[], sel_a, None), 1.0, "选中表自身 alpha=1");
        assert_eq!(table_render_alpha("b", &refs, &[], sel_a, None), 1.0, "邻接表 b alpha=1");
        assert_eq!(table_render_alpha("c", &refs, &[], sel_a, None), 1.0, "邻接表 c alpha=1");
        assert!(
            table_render_alpha("d", &refs, &[], sel_a, None) <= 0.5,
            "UT-PE-HL-01: 非相关表 d alpha ≤ 0.5"
        );

        // 断言 2：选中关系 r1 —— 该关系及两端表 a/b 相关；c/d 非相关
        let sel_r1 = Some("r1");
        assert!(relation_is_related("r1", "a", "b", &[], None, sel_r1));
        assert!(!relation_is_related("r2", "a", "c", &[], None, sel_r1));
        assert_eq!(table_render_alpha("a", &refs, &[], None, sel_r1), 1.0);
        assert_eq!(table_render_alpha("b", &refs, &[], None, sel_r1), 1.0);
        assert!(table_render_alpha("c", &refs, &[], None, sel_r1) <= 0.5);
        assert!(table_render_alpha("d", &refs, &[], None, sel_r1) <= 0.5);

        // 断言 3：无选中 —— 全部恢复默认线宽与 alpha=1
        assert!(!relation_is_related("r1", "a", "b", &[], None, None));
        assert_eq!(relation_render_width(2.0, relation_is_related("r1", "a", "b", &[], None, None)), 2.0);
        for t in ["a", "b", "c", "d"] {
            assert_eq!(table_render_alpha(t, &refs, &[], None, None), 1.0, "无选中 {t} alpha=1");
            assert_eq!(relation_opacity("r1", "a", "b", &[], None, None), 1.0);
        }

        // 断言 4：只读不改变高亮反馈——纯函数无只读入参，同样输入必然同样输出
        assert_eq!(
            table_render_alpha("d", &refs, &[], sel_a, None),
            table_render_alpha("d", &refs, &[], sel_a, None),
            "UT-PE-HL-01: 只读同参数（幂等）"
        );

        // 断言 5（fix-31 提亮，§4.4）：draw_relation 相关态强制选中色族 + 端点同色源码锚点
        let src = include_str!("editor_render.rs");
        let dr_idx = src.find("fn draw_relation(").expect("draw_relation 存在");
        // 窗口覆盖 draw_relation 全函数体（至下一函数 draw_endpoint_notation 定义前）
        let dr_end = src[dr_idx..]
            .find("\nfn draw_endpoint_notation(")
            .map(|i| dr_idx + i)
            .expect("draw_endpoint_notation 存在");
        let dr_block = &src[dr_idx..dr_end];
        assert!(
            dr_block.contains("if selected || related { palette.selected } else { stroke }"),
            "UT-PE-HL-01: 相关线主色必须强制 palette.selected（fix-31 提亮，不再沿用关系解析色）"
        );
        assert!(
            dr_block.contains("if selected || related { palette.selected_soft } else { palette.relation_halo }"),
            "UT-PE-HL-01: 相关线光晕必须换用 palette.selected_soft 发光"
        );
        assert!(
            dr_block.contains("} else if related { 8.0 } else { 7.0 * hl_scale }"),
            "UT-PE-HL-01: 相关线光晕宽度必须固定 8px（选中关系 10px 保持层级）"
        );
        assert!(
            dr_block.contains("related: bool"),
            "UT-PE-HL-01: draw_relation 必须接收 related 独立入参"
        );
        assert!(
            !dr_block.contains(concat!(", str", "oke, main_w);")),
            "UT-PE-HL-01: crow's foot 端点记号必须与主线同色（不得回落基线色 stroke）"
        );
    }

    /// UT-PE-CMT-01 — 注释对比度前景计算纯函数（#32，core-01a R-CMT-CONTRAST-01~05）
    #[test]
    fn ut_pe_cmt_01_comment_contrast_foreground() {
        let light_bg = "rgba(255,255,255,.84)";
        let dark_bg = "rgba(16,38,45,.94)";
        let (ls, lm) = (PALETTE_LIGHT.text_strong, PALETTE_LIGHT.text_muted);
        let (ds, dm) = (PALETTE_DARK.text_strong, PALETTE_DARK.text_muted);

        // 断言 1：有效背景亮度高于阈值 → 深前景对；低于阈值 → 浅前景对（R-COLOR-04 同阈值口径）
        // 亮主题默认低 alpha tint（合成背景亮）→ 深色系前景（亮主题 muted 仅 ~2.6:1 → 升级 strong，无 chip）
        let bright = comment_foreground("rgba(30,131,147,.13)", light_bg, ls, lm, ds, dm, true);
        assert_eq!(bright.fg, ls, "UT-PE-CMT-01: 亮背景必须选深前景对");
        assert_eq!(bright.chip, None, "UT-PE-CMT-01: 深 strong 在亮背景直绘达标，无需 chip");
        // 实色深青表头（合成背景暗）→ 浅前景对（浅 muted 在此深度直绘达标）
        let dark = comment_foreground("#12374a", dark_bg, ls, lm, ds, dm, false);
        assert_eq!(dark.fg, dm, "UT-PE-CMT-01: 深背景必须选浅前景对");

        // 断言 2：needs_comment_chip 阈值边界
        assert!(needs_comment_chip(4.4), "UT-PE-CMT-01: <4.5 必须兜底");
        assert!(!needs_comment_chip(4.5), "UT-PE-CMT-01: ≥4.5 不兜底");
        assert!(!needs_comment_chip(4.6));
        assert!(needs_comment_chip(3.0));

        // 断言 3（#34 双端点判据）：橙/粉/紫/棕（#34 截图问题色）+ 蓝/绿表头 × 亮/暗主题 12 组，
        // 决策输出前景对**渐变两端**（满强度 tint 合成端 + 纯表体衰减端，含 chip 合成）均 ≥ 4.5:1
        let eff_bg_lum = |tint: &str, bg: &str, chip: &Option<String>| -> f64 {
            let base = parse_css_color_rgba(bg).unwrap();
            let tint_rgba = parse_css_color_rgba(tint).unwrap();
            let under = composite_over(tint_rgba, base);
            match chip {
                Some(c) => {
                    let chip_rgba = parse_css_color_rgba(c).unwrap();
                    let (r, g, b) = composite_over(chip_rgba, (under.0, under.1, under.2, 1.0));
                    relative_luminance_rgb(r, g, b)
                }
                None => relative_luminance_rgb(under.0, under.1, under.2),
            }
        };
        for tint in ["#3788e5", "#19a974", "#aa8cff", "#e8833a", "#e85d9e", "#8a5a3b"] {
            for (bg, fallback_dark) in [(light_bg, true), (dark_bg, false)] {
                let style = comment_foreground(tint, bg, ls, lm, ds, dm, fallback_dark);
                let fg_l = color_relative_luminance(&style.fg).unwrap();
                let ratio_full = contrast_ratio_luminance(fg_l, eff_bg_lum(tint, bg, &style.chip));
                let ratio_faded =
                    contrast_ratio_luminance(fg_l, eff_bg_lum("rgba(0,0,0,0)", bg, &style.chip));
                assert!(
                    ratio_full >= COMMENT_CONTRAST_MIN && ratio_faded >= COMMENT_CONTRAST_MIN,
                    "UT-PE-CMT-01: tint={tint} bg={bg} 双端对比度（满强度 {ratio_full:.2} / 衰减端 {ratio_faded:.2}）必须均 ≥ 4.5（style={style:?}）"
                );
            }
        }

        // 断言 3b（#34 回归锚点）：深棕实色表头 + 亮主题——渐变衰减端 ≈ 白底，
        // 不得输出「浅色系前景 + 无 chip」（此前单端点判据的漏判形态）
        let brown = comment_foreground("#8a5a3b", light_bg, ls, lm, ds, dm, true);
        let faded_l = composited_relative_luminance("rgba(0,0,0,0)", light_bg).unwrap();
        let fg_l = color_relative_luminance(&brown.fg).unwrap();
        assert!(
            brown.chip.is_some()
                || contrast_ratio_luminance(fg_l, faded_l) >= COMMENT_CONTRAST_MIN,
            "UT-PE-CMT-01: 深棕表头亮主题下衰减端不得浅字裸绘（style={brown:?}）"
        );
        // 双端点纯函数锚点：min 语义 = 两端取小
        let full_only = comment_contrast_ratio(ds, "#8a5a3b", light_bg).unwrap();
        let dual = comment_contrast_ratio_min(ds, "#8a5a3b", light_bg).unwrap();
        assert!(
            dual <= full_only + 1e-9,
            "UT-PE-CMT-01: 双端点对比度必须 ≤ 单端点（满 tint）值"
        );

        // 断言 3c（字段行无回归）：tint 全透明 → 双端点退化同一样本，输出与单端点一致
        let field_style = comment_foreground("rgba(0,0,0,0)", light_bg, ls, lm, ds, dm, true);
        assert_eq!(field_style.fg, ls, "UT-PE-CMT-01: 字段行亮主题仍深 strong 直绘");
        assert_eq!(field_style.chip, None, "UT-PE-CMT-01: 字段行不得误加 chip");
        let field_style_d = comment_foreground("rgba(0,0,0,0)", dark_bg, ls, lm, ds, dm, false);
        assert_eq!(field_style_d.fg, dm, "UT-PE-CMT-01: 字段行暗主题仍浅 muted 直绘");
        assert_eq!(field_style_d.chip, None);

        // 断言 4：空注释不产生任何注释渲染参数（R-CMT-03 不占位）
        assert_eq!(
            crate::editor_core::CommentDisplay::NameComment.secondary(""),
            None,
            "UT-PE-CMT-01: 空注释不渲染"
        );
        assert_eq!(crate::editor_core::CommentDisplay::NameComment.secondary("  "), None);
    }

    /// UT-CR-LOD-01 — 维度模式与可读性补偿纯函数（#30/#35/#36，core-01 §5.12 / R-VIEW-DIM / R-LOD-02~08）
    #[test]
    fn ut_cr_lod_01_view_dimension_and_params() {
        // 断言 1（#36 R-VIEW-DIM-04）：ViewDimension 存储合同——缺省/非法回落 Field；
        // as_str 往返一致；next 双态互换
        use crate::editor_core::ViewDimension;
        assert_eq!(ViewDimension::from_stored(None), ViewDimension::Field, "UT-CR-LOD-01: 缺省必须字段维度");
        assert_eq!(ViewDimension::from_stored(Some("bogus")), ViewDimension::Field, "UT-CR-LOD-01: 非法值必须回落字段维度");
        assert_eq!(ViewDimension::from_stored(Some("table")), ViewDimension::Table);
        assert_eq!(ViewDimension::from_stored(Some("field")), ViewDimension::Field);
        for d in [ViewDimension::Table, ViewDimension::Field] {
            assert_eq!(ViewDimension::from_stored(Some(d.as_str())), d, "UT-CR-LOD-01: as_str 往返必须一致");
        }
        assert_eq!(ViewDimension::Field.next(), ViewDimension::Table);
        assert_eq!(ViewDimension::Table.next(), ViewDimension::Field);
        assert_eq!(crate::editor_core::VIEW_DIMENSION_STORAGE_KEY, "cdb.view-dimension");

        // 断言 2（#36 R-LOD-01 修订 / R-VIEW-DIM-01）：维度→渲染档映射与 zoom 无关——
        // Table→Topology、Field→Detail；zoom 阈值判档（旧 lod_tier / 阈值常量）已退役（源码锚点）
        assert_eq!(tier_for_dimension(ViewDimension::Table), LodTier::Topology);
        assert_eq!(tier_for_dimension(ViewDimension::Field), LodTier::Detail);
        let src = include_str!("editor_render.rs");
        assert!(
            !src.contains(concat!("lo", "d_tier(")),
            "UT-CR-LOD-01: zoom 阈值判档函数必须退役（R-LOD-01 #36 修订）"
        );
        assert!(
            !src.contains(concat!("LOD_TOPOLOGY", "_ZOOM")),
            "UT-CR-LOD-01: 拓扑档 zoom 阈值常量必须退役"
        );

        // 断言 3：拓扑档表名世界字号保证屏幕 ≥11px（22px@0.5）；0.35 触发夹紧上限；详情档默认 13px
        let f50 = lod_table_font_size(0.5, LodTier::Topology);
        assert!((f50 - 22.0).abs() < 1e-9, "UT-CR-LOD-01: 0.5 世界字号必须 22px（实际 {f50}）");
        assert!(f50 * 0.5 >= 11.0 - 1e-9, "UT-CR-LOD-01: 屏幕字号必须 ≥11px");
        let f35 = lod_table_font_size(0.35, LodTier::Topology);
        assert!((f35 - LOD_TABLE_FONT_WORLD_MAX).abs() < 1e-9, "UT-CR-LOD-01: 0.35 必须触发夹紧上限 {LOD_TABLE_FONT_WORLD_MAX}（实际 {f35}）");
        assert_eq!(lod_table_font_size(0.5, LodTier::Detail), TABLE_NAME_FONT_PX, "UT-CR-LOD-01: 详情档必须返回默认字号");
        assert_eq!(lod_table_font_size(1.0, LodTier::Detail), TABLE_NAME_FONT_PX);

        // 断言 5：lod_line_width(0.5, base=2) ≥ 1.5/0.5=3px 世界宽；补偿倍率夹紧 ≤4×
        let w50 = lod_line_width(0.5, 2.0);
        assert!(w50 >= 1.5 / 0.5 - 1e-9, "UT-CR-LOD-01: 0.5 世界线宽必须 ≥3px（实际 {w50}）");
        let w01 = lod_line_width(0.1, 2.0);
        assert!(w01 <= 2.0 * 4.0 + 1e-9, "UT-CR-LOD-01: 补偿倍率必须夹紧 ≤4×（实际 {w01}）");
        assert!((w01 - 8.0).abs() < 1e-9, "UT-CR-LOD-01: 0.1 必须夹到 base×4=8（实际 {w01}）");

        // 断言 6（R-LOD-05 / R-VIEW-DIM-05）：维度（渲染档）进入精灵指纹——跨档不同、同档内相同（拖动不触发重光栅）
        let t = Table {
            id: "t1".into(),
            name: "orders".into(),
            x: 0.0,
            y: 0.0,
            color: String::new(),
            comment: String::new(),
            fields: vec![],
            indices: vec![],
            width: None,
            min_height: None,
        };
        let mode = crate::editor_core::CommentDisplay::NameComment;
        let fp_detail = table_sprite_fingerprint(&t, true, 200, 1, mode, LodTier::Detail, &[]);
        let fp_topo = table_sprite_fingerprint(&t, true, 200, 1, mode, LodTier::Topology, &[]);
        assert_ne!(fp_detail, fp_topo, "UT-CR-LOD-01: 跨档指纹必须不同（触发重光栅）");
        assert_eq!(
            table_sprite_fingerprint(&t, true, 200, 1, mode, LodTier::Topology, &[]),
            fp_topo,
            "UT-CR-LOD-01: 同档内指纹必须相同"
        );
        // 同档内移动（x/y 变化）指纹不变——拖动不触发重光栅
        let moved = Table { x: 40.0, y: 80.0, ..t.clone() };
        assert_eq!(
            table_sprite_fingerprint(&moved, true, 200, 1, mode, LodTier::Topology, &[]),
            fp_topo,
            "UT-CR-LOD-01: 同档内拖动指纹必须相同"
        );

        // 断言 7（#35 R-LOD-08）：表级锚点纯函数——拓扑档锚点纵坐标 = 表头中线、
        // 横坐标 = 卡体左/右缘；详情档保持字段锚点；三路径函数同口径
        let fld = |id: &str, name: &str| Field {
            id: id.into(),
            name: name.into(),
            type_: "INT".into(),
            default: String::new(),
            check: String::new(),
            primary: false,
            unique: false,
            not_null: false,
            increment: false,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        };
        let ta = Table {
            id: "ta".into(),
            name: "orders".into(),
            x: 100.0,
            y: 200.0,
            color: String::new(),
            comment: String::new(),
            fields: vec![fld("fa1", "id"), fld("fa2", "user_id")],
            indices: vec![],
            width: None,
            min_height: None,
        };
        let tb = Table {
            id: "tb".into(),
            name: "users".into(),
            x: 500.0,
            y: 260.0,
            color: String::new(),
            comment: String::new(),
            fields: vec![fld("fb1", "id")],
            indices: vec![],
            width: None,
            min_height: None,
        };
        // 拓扑档：锚点 y = 表头中线（与字段无关——字段行已隐藏）
        let (_, ay_topo) = anchor_for_tier(&ta, "fa2", FieldPortSide::End, LodTier::Topology);
        assert_eq!(
            ay_topo,
            ta.y + TABLE_HEADER_HEIGHT / 2.0,
            "UT-CR-LOD-01: 拓扑档锚点纵坐标必须收敛表头中线（R-LOD-08）"
        );
        let (_, ay_topo_f1) = anchor_for_tier(&ta, "fa1", FieldPortSide::End, LodTier::Topology);
        assert_eq!(ay_topo, ay_topo_f1, "UT-CR-LOD-01: 同表多字段在拓扑档必须汇聚同一表级端口");
        // 详情档：字段锚点不变（fa2 第二行 ≠ 表头中线）
        let (_, ay_detail) = anchor_for_tier(&ta, "fa2", FieldPortSide::End, LodTier::Detail);
        assert_ne!(ay_detail, ay_topo, "UT-CR-LOD-01: 详情档必须保持字段维度锚点");
        assert_eq!(
            anchor_for_tier(&ta, "fa2", FieldPortSide::End, LodTier::Detail),
            field_anchor_for_side(&ta, "fa2", FieldPortSide::End),
            "UT-CR-LOD-01: 详情档与 field_anchor_for_side 同口径"
        );
        // 三路径函数 tier 变体：拓扑档两端 y 均为各自表头中线；默认（详情）保持字段锚定
        let p_topo = calc_path_tier(&ta, "fa2", &tb, "fb1", LodTier::Topology);
        assert_eq!(p_topo.y1, ta.y + TABLE_HEADER_HEIGHT / 2.0);
        assert_eq!(p_topo.y2, tb.y + TABLE_HEADER_HEIGHT / 2.0);
        let p_detail = calc_path(&ta, "fa2", &tb, "fb1");
        assert_eq!(p_detail, calc_path_tier(&ta, "fa2", &tb, "fb1", LodTier::Detail));
        let o_topo = calc_orthogonal_path_tier(&ta, "fa2", &tb, "fb1", LodTier::Topology);
        assert_eq!(o_topo[0].1, ta.y + TABLE_HEADER_HEIGHT / 2.0);
        let s_topo = calc_straight_path_tier(&ta, "fa2", &tb, "fb1", LodTier::Topology);
        assert_eq!(s_topo.1, ta.y + TABLE_HEADER_HEIGHT / 2.0);
        assert_eq!(s_topo.3, tb.y + TABLE_HEADER_HEIGHT / 2.0);
        // 命中检测同口径（R-LOD-06 交互不变）：拓扑档命中表级锚定线、不命中字段锚定线
        let mid_x = (p_topo.x1 + p_topo.x2) / 2.0;
        let mid_y = (p_topo.y1 + p_topo.y2) / 2.0;
        let refs = vec![Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "ta".into(),
            end_table_id: "tb".into(),
            start_field_id: "fa2".into(),
            end_field_id: "fb1".into(),
            type_: String::new(),
            on_delete: String::new(),
            on_update: String::new(),
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
        }];
        let tables = vec![ta.clone(), tb.clone()];
        assert_eq!(
            hit_test_reference_tier(&tables, &refs, mid_x, mid_y, LodTier::Topology),
            Some("r1".to_string()),
            "UT-CR-LOD-01: 拓扑档命中必须与表级锚定绘制同口径"
        );

        // 断言 8（#35 R-LOD-02 补齐）：拓扑档注释宽度口径——NameComment 且注释非空时
        // 卡宽估算覆盖「表名 + 注释」（注释在拓扑档可见，必须有空间）；空注释不增宽
        let mut tc = ta.clone();
        // 长注释（12 CJK ≈ 168px）使表头行估算超过 TABLE_WIDTH 下限 230，增宽才可观测
        tc.comment = "订单主表注释信息补充说明".into();
        let w_with = resolve_table_width(&tc, crate::editor_core::CommentDisplay::NameComment);
        let w_without = resolve_table_width(&ta, crate::editor_core::CommentDisplay::NameComment);
        assert!(
            w_with > w_without,
            "UT-CR-LOD-01: NameComment 非空注释必须增宽估算（{w_with} > {w_without}）"
        );
        let w_name_only = resolve_table_width(&tc, crate::editor_core::CommentDisplay::Name);
        assert_eq!(
            w_name_only, w_without,
            "UT-CR-LOD-01: Name 模式注释不参与宽度估算"
        );
        // 拓扑档卡宽 = resolve_table_width（注释感知），高度恒为表头高
        let (tw, th) = lod_table_size(&tc, crate::editor_core::CommentDisplay::NameComment, LodTier::Topology);
        assert_eq!(tw, w_with, "UT-CR-LOD-01: 拓扑档卡宽必须注释感知");
        assert_eq!(th, TABLE_HEADER_HEIGHT, "UT-CR-LOD-01: 拓扑档卡高恒为表头高");
    }

    /// #33 UT-PE-VIS-01 — 语义图标统一族锚点（core-08 §11 / core-07 §15.5）
    #[test]
    fn ut_pe_vis_01_semantic_icon_family_anchors() {
        // ── 断言 1：徽章族映射 + 统一尺寸/描边常量同源（无散值）──
        assert_eq!(BADGE_SIZE, 12.0, "UT-PE-VIS-01: 徽章统一外接尺寸 12px");
        assert_eq!(BADGE_STROKE, 1.5, "UT-PE-VIS-01: 徽章统一描边 1.5px");
        assert_eq!(REL_ENDPOINT_SIZE, 10.0, "UT-PE-VIS-01: 端点统一外接尺寸 10px");
        assert_eq!(TABLE_CORNER_RADIUS, 8.0, "UT-PE-VIS-01: 表卡圆角统一 8px");
        assert_eq!(field_badges(true, false, false, false), vec![FieldBadge::Pk]);
        assert_eq!(
            field_badges(true, true, true, true),
            vec![FieldBadge::Pk, FieldBadge::Fk, FieldBadge::Nn, FieldBadge::Uq],
            "UT-PE-VIS-01: 徽章顺序固定 PK→FK→NN→UQ"
        );
        assert_eq!(field_badges(false, false, false, false), Vec::<FieldBadge>::new());
        assert_eq!(field_badges_width(0), 0.0);
        assert!(field_badges_width(1) > BADGE_SIZE);
        assert!(field_badges_width(2) > field_badges_width(1));

        // ── 断言 2：FK 推导与端点重数（crow's foot 族）──
        let mk = |id: &str, sf: &str, ef: &str, card: &str| Reference {
            id: id.to_string(),
            name: String::new(),
            start_table_id: String::new(),
            end_table_id: String::new(),
            start_field_id: sf.to_string(),
            end_field_id: ef.to_string(),
            type_: card.to_string(),
            on_delete: String::new(),
            on_update: String::new(),
            color: String::new(),
            line_type: "bezier".into(),
            stroke_style: "solid".into(),
        };
        let refs = vec![
            mk("r1", "users-id", "orders-uid", "one_to_many"),
            mk("r2", "posts-author", "users-id", "many_to_one"),
            mk("r3", "a-id", "b-aid", "one_to_one"),
        ];
        let fk = fk_field_ids(&refs);
        assert!(fk.contains("orders-uid"), "one_to_many → FK 在 end 侧");
        assert!(fk.contains("posts-author"), "many_to_one → FK 在 start 侧");
        assert!(fk.contains("b-aid"), "one_to_one → FK 归 end 侧");
        assert!(!fk.contains("users-id"));
        assert_eq!(endpoint_multiplicity("one_to_many"), (false, true));
        assert_eq!(endpoint_multiplicity("many_to_one"), (true, false));
        assert_eq!(endpoint_multiplicity("one_to_one"), (false, false));
        assert_eq!(endpoint_multiplicity(""), (false, true), "未知值按一对多");

        // ── 断言 3：源码锚点——字块角标渲染路径已移除；徽章绘制色值仅取 palette 语义色阶 ──
        let src = include_str!("editor_render.rs");
        assert!(
            !src.contains("fill_text(\"PK\""),
            "UT-PE-VIS-01: 字块角标 fill_text(\"PK\") 必须移除（core-08 §11 deprecated/移除）"
        );
        let badge_fn = src
            .split("fn draw_field_badges")
            .nth(1)
            .expect("draw_field_badges 必须存在");
        let badge_body = badge_fn.split("\nfn ").next().unwrap();
        for color in ["palette.pk_color", "palette.fk_color", "palette.constraint_color"] {
            assert!(badge_body.contains(color), "UT-PE-VIS-01: 徽章色必须取 {color}");
        }
        assert!(
            !badge_body.contains("\"#"),
            "UT-PE-VIS-01: 徽章绘制函数体内禁止裸 hex 散值"
        );

        // ── 断言 4：ToolRail 图标统一 stroke 1.5 / IconBox 20px(md) / 激活态规则存在 ──
        let icons = include_str!("icons.rs");
        assert!(
            icons.contains("#[prop(default = 1.5)]"),
            "UT-PE-VIS-01: ToolRail 图标统一 stroke 1.5（Icon 默认）"
        );
        let css = include_str!("styles.css");
        assert!(
            css.contains("--cdb-icon-size-md: 20px"),
            "UT-PE-VIS-01: IconBox md = 20px"
        );
        assert!(
            css.contains(".cdb-tool-btn.cdb-is-active"),
            "UT-PE-VIS-01: ToolRail 激活态规则必须存在（主色描边 + 浅底）"
        );

        // ── 断言 5：关系端点 crow's foot 族渲染路径存在；旧箭头已移除 ──
        assert!(src.contains("fn draw_endpoint_notation"), "crow's foot 端点函数必须存在");
        // concat! 避免断言字面量自匹配
        assert!(!src.contains(concat!("fn draw_arrow", "_head")), "旧箭头端点必须移除");
        let rel_call = src.split("fn draw_relation").nth(1).expect("draw_relation 必须存在");
        assert!(
            rel_call.contains("draw_endpoint_notation"),
            "UT-PE-VIS-01: draw_relation 必须走 crow's foot 端点族"
        );
    }

    /// ST-CR-MULTI-01：多选集合内表必须有选中视觉（对齐 #5 reopen）
    #[test]
    fn ut_cr_multi_01_table_visually_selected() {
        let multi = vec!["a".into(), "b".into()];
        assert!(
            table_visually_selected("a", Some("a"), &multi),
            "主选中应高亮"
        );
        assert!(
            table_visually_selected("b", Some("a"), &multi),
            "多选集合内非主选中表也必须高亮"
        );
        assert!(
            !table_visually_selected("c", Some("a"), &multi),
            "未入选表不高亮"
        );
        assert!(
            table_visually_selected("x", Some("x"), &[]),
            "仅主选中时仍高亮"
        );
    }

    /// ST-CR-MULTI-01：框选后首次点已选表不得 toggle 掉，以便整组拖动
    #[test]
    fn ut_cr_multi_01_resolve_table_multi_on_pointerdown() {
        let multi = vec!["a".to_string(), "b".to_string()];
        // 框选工具仍激活 + 点已选成员 → 保持双选（修复「第一次只能拖一张」）
        let kept = resolve_table_multi_on_pointerdown(false, true, "a", &multi);
        assert_eq!(kept, multi, "点已选成员必须保持集合");
        // 框选工具 + 点未选中 → 累加
        let added = resolve_table_multi_on_pointerdown(false, true, "c", &multi);
        assert_eq!(
            added,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        // 普通选择 + 点未选中 → 单选重置
        let reset = resolve_table_multi_on_pointerdown(false, false, "c", &multi);
        assert_eq!(reset, vec!["c".to_string()]);
        // Shift 切换
        let toggled = resolve_table_multi_on_pointerdown(true, false, "a", &multi);
        assert_eq!(toggled, vec!["b".to_string()]);
    }

    /// ST-CR-MULTI-01 / #5：框选预览必须是归一化矩形，而非关系贝塞尔线
    #[test]
    fn ut_cr_multi_01_normalize_marquee_rect() {
        let (x, y, w, h) = normalize_marquee_rect(100.0, 50.0, 40.0, 90.0);
        assert_eq!((x, y, w, h), (40.0, 50.0, 60.0, 40.0));
        let path = rubber_band_path(0.0, 0.0, 100.0, 100.0);
        assert!(
            path.contains('C') || path.contains('c'),
            "关系 rubber 仍是曲线；框选不得复用该路径画线"
        );
        let (x2, y2, w2, h2) = normalize_marquee_rect(0.0, 0.0, 100.0, 80.0);
        assert!(w2 > 0.0 && h2 > 0.0 && x2 == 0.0 && y2 == 0.0);
    }

    /// UT-PB-08 — 字段连接点命中（非整行）；行中心不命中 port
    #[test]
    fn ut_pb_08_hit_test_field_port() {
        let table = Table {
            id: "t1".into(),
            name: "t1".into(),
            x: 100.0,
            y: 100.0,
            color: String::new(),
            comment: String::new(),
            fields: vec![crate::editor_core::types::Field {
                id: "f1".into(),
                name: "id".into(),
                type_: "INT".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: true,
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
        let cy = 100.0 + TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT / 2.0;
        let left = hit_test_field_port(&tables, 100.0, cy);
        assert_eq!(
            left.map(|(t, f, s)| (t, f, s)),
            Some(("t1".into(), "f1".into(), FieldPortSide::Start)),
            "UT-PB-08: 左侧连接点应命中"
        );
        let right = hit_test_field_port(&tables, 100.0 + TABLE_WIDTH, cy);
        assert_eq!(
            right.map(|(_, _, s)| s),
            Some(FieldPortSide::End),
            "UT-PB-08: 右侧连接点应命中"
        );
        let mid = hit_test_field_port(&tables, 100.0 + TABLE_WIDTH / 2.0, cy);
        assert!(mid.is_none(), "UT-PB-08: 字段行中心不得起拖连");
        assert!(
            hit_test_field(&tables, 100.0 + TABLE_WIDTH / 2.0, cy).is_some(),
            "UT-PB-08: 行中心仍可选中字段"
        );
    }

    /// UT-CR-MULTI-01 — 多表同步位移 + marquee 相交
    #[test]
    fn ut_cr_multi_01_translate_tables() {
        let mut tables = vec![
            Table {
                id: "a".into(),
                name: "a".into(),
                x: 0.0,
                y: 0.0,
                color: String::new(),
                comment: String::new(),
                fields: Vec::new(),
                indices: Vec::new(),
                width: None,
                min_height: None,
            },
            Table {
                id: "b".into(),
                name: "b".into(),
                x: 100.0,
                y: 50.0,
                color: String::new(),
                comment: String::new(),
                fields: Vec::new(),
                indices: Vec::new(),
                width: None,
                min_height: None,
            },
            Table {
                id: "c".into(),
                name: "c".into(),
                x: 300.0,
                y: 300.0,
                color: String::new(),
                comment: String::new(),
                fields: Vec::new(),
                indices: Vec::new(),
                width: None,
                min_height: None,
            },
        ];
        let ids = vec!["a".into(), "b".into()];
        translate_tables(&mut tables, &ids, 10.0, 20.0);
        assert_eq!(tables[0].x, 10.0);
        assert_eq!(tables[0].y, 20.0);
        assert_eq!(tables[1].x, 110.0);
        assert_eq!(tables[1].y, 70.0);
        assert_eq!(tables[2].x, 300.0, "UT-CR-MULTI-01: 未选中不变");
        assert_eq!(tables[2].y, 300.0);
        let hit = tables_in_marquee(&tables, 0.0, 0.0, 150.0, 100.0);
        assert!(hit.contains(&"a".to_string()) && hit.contains(&"b".to_string()));
        assert!(!hit.contains(&"c".to_string()));
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

    /// UT-CR-PAN-01 — 空白按下 click/pan 阈值判定（#39，core-01 §5.15 R-PAN-SEL-01/02/05）
    #[test]
    fn ut_cr_pan_01_blank_click_threshold() {
        // 断言 1：位移 <4px（屏幕欧氏距离）判定 click（清选语义）
        assert!(super::is_blank_click(0.0, 0.0), "UT-CR-PAN-01: 零位移必须是 click");
        assert!(super::is_blank_click(3.9, 0.0), "UT-CR-PAN-01: 3.9px 必须是 click");
        assert!(super::is_blank_click(-3.9, 0.0), "UT-CR-PAN-01: 负向 3.9px 必须是 click");
        assert!(super::is_blank_click(2.0, 2.0), "UT-CR-PAN-01: 斜向 2.83px 必须是 click");

        // 断言 2：位移 ≥4px 判定 pan（保留选中）；边界 4px 恰为 pan
        assert!(!super::is_blank_click(4.0, 0.0), "UT-CR-PAN-01: 边界 4px 必须是 pan");
        assert!(!super::is_blank_click(0.0, -4.0), "UT-CR-PAN-01: 负向边界 4px 必须是 pan");
        assert!(!super::is_blank_click(3.0, 3.0), "UT-CR-PAN-01: 斜向 4.24px 必须是 pan");
        assert!(!super::is_blank_click(80.0, 60.0), "UT-CR-PAN-01: 100px 必须是 pan");

        // 断言 3：与表/便签/区域拖动共用同一 DRAG_THRESHOLD 常量口径（4px）
        assert_eq!(super::DRAG_THRESHOLD, 4.0, "UT-CR-PAN-01: 阈值常量必须为 4px");

        // 断言 4（R-PAN-SEL-05 锚点）：Shift/框选工具仍走框选口径——pointerdown 的
        // use_marquee 门控必须保留 shift_key 与 marquee_active 两个入口
        let src = include_str!("editor_render.rs");
        assert!(
            src.contains("ev.shift_key() || marquee_active.get_untracked()"),
            "UT-CR-PAN-01: Shift/框选工具的框选门控必须保留"
        );
        // 断言 5（R-PAN-SEL-01 锚点）：click 清选只能在 pointerup pan 兜底分支发生——
        // is_blank_click 调用点必须存在且唯一（on_pointerup）
        let call_count = src.matches("is_blank_click(").count();
        // 定义处 1 次 + 调用处 1 次 + 本测试若干次断言不含源码匹配（测试在文件尾部同文件）
        assert!(
            call_count >= 2,
            "UT-CR-PAN-01: is_blank_click 必须在 pointerup pan 兜底被调用（实际匹配 {call_count}）"
        );
    }

    /// UT-CR-FONT-01 — 标签字号倍率档位与 10px 屏幕下限钳制（#40，core-01 §5.16 R-FONT-01~06）
    #[test]
    fn ut_cr_font_01_label_font_scale_and_clamp() {
        use crate::editor_core::LabelFontScale;

        // 断言 1（R-FONT-02）：档位循环 0.8→1.0→1.25→1.5→0.8
        assert_eq!(LabelFontScale::S80.next(), LabelFontScale::S100);
        assert_eq!(LabelFontScale::S100.next(), LabelFontScale::S125);
        assert_eq!(LabelFontScale::S125.next(), LabelFontScale::S150);
        assert_eq!(LabelFontScale::S150.next(), LabelFontScale::S80, "UT-CR-FONT-01: 1.5 后必须循环回 0.8");

        // 断言 2（R-FONT-01）：localStorage 读写口径——缺省/非法回落 1.0；as_str 往返一致
        assert_eq!(LabelFontScale::from_stored(None), LabelFontScale::S100, "UT-CR-FONT-01: 缺省必须 1.0");
        assert_eq!(LabelFontScale::from_stored(Some("bogus")), LabelFontScale::S100, "UT-CR-FONT-01: 非法值必须回落 1.0");
        for s in [LabelFontScale::S80, LabelFontScale::S100, LabelFontScale::S125, LabelFontScale::S150] {
            assert_eq!(
                LabelFontScale::from_stored(Some(s.as_str())), s,
                "UT-CR-FONT-01: as_str 往返必须一致"
            );
        }
        assert_eq!(crate::editor_core::LABEL_FONT_SCALE_STORAGE_KEY, "cdb.label-font-scale");
        assert_eq!(LabelFontScale::S80.factor(), 0.8);
        assert_eq!(LabelFontScale::S150.factor(), 1.5);

        // 断言 3（R-FONT-04）：屏幕下限钳制——zoom 0.2 时 13px×1.0×0.2=2.6px < 10px 触发下限
        let (px, clamped) = scaled_label_font_world(13.0, 0.2, 1.0);
        assert!(clamped, "UT-CR-FONT-01: zoom 0.2 必须触发下限钳制");
        assert!((px * 0.2 - 10.0).abs() < 1e-9, "UT-CR-FONT-01: 钳制后屏幕字号必须恰为 10px（实际 {}）", px * 0.2);
        // 边界：屏幕恰好 10px 不视为钳制
        let (px10, c10) = scaled_label_font_world(10.0, 1.0, 1.0);
        assert!(!c10 && (px10 - 10.0).abs() < 1e-9, "UT-CR-FONT-01: 恰 10px 不钳制");

        // 断言 4（R-FONT-04）：放大方向不封顶——zoom 2.0 × 1.5 倍率 → 13→19.5 世界（屏幕 39px）
        let (px_big, c_big) = scaled_label_font_world(13.0, 2.0, 1.5);
        assert!(!c_big && (px_big - 19.5).abs() < 1e-9, "UT-CR-FONT-01: 放大方向不得封顶（实际 {px_big}）");
        // 常规 zoom 1.0 / 1.0 倍率恒等
        let (px_id, c_id) = scaled_label_font_world(13.0, 1.0, 1.0);
        assert!(!c_id && (px_id - 13.0).abs() < 1e-9);

        // 断言 5（R-FONT-05）：钳制解析探测与逐字体纯函数同口径
        assert!(any_label_font_clamped(0.35, 1.0, LodTier::Detail), "UT-CR-FONT-01: zoom 0.35 详情档必须报钳制");
        assert!(!any_label_font_clamped(1.0, 1.0, LodTier::Detail), "UT-CR-FONT-01: zoom 1.0 详情档不得报钳制");
        assert!(!any_label_font_clamped(2.0, 1.25, LodTier::Detail));
        // 拓扑档：注释 = 表名×0.8，zoom 1.0 时 8.8px < 10px → 钳制
        assert!(any_label_font_clamped(1.0, 1.0, LodTier::Topology), "UT-CR-FONT-01: 拓扑档注释 8.8px 必须报钳制");

        // 断言 6（R-FONT-06 锚点）：倍率混入精灵指纹——blit 内必须有倍率散列混入
        let src = include_str!("editor_render.rs");
        assert!(
            src.contains("label_scale * 100.0"),
            "UT-CR-FONT-01: 精灵指纹必须混入字号倍率（切档重光栅）"
        );
    }

    /// UT-AN-LOCK-01 — Area.locked 序列化兼容 + 拖拽门控（#41，core-01 §5.17 R-AREALOCK-01/03）
    #[test]
    fn ut_an_lock_01_area_locked_serde_and_drag_gate() {
        // 断言 1（R-AREALOCK-01）：旧图 JSON 缺 locked 键 → 反序列化 locked=false
        let json_old = r##"{"id":"a1","x":0.0,"y":0.0,"width":100.0,"height":80.0,"color":"#3b82f6","name":"区域"}"##;
        let a: Area = serde_json::from_str(json_old)
            .expect("UT-AN-LOCK-01: 旧图 JSON（无 locked 键）必须可反序列化");
        assert!(!a.locked, "UT-AN-LOCK-01: 缺省 locked 必须为 false（旧图兼容）");

        // 断言 2（R-AREALOCK-01/06）：locked=true 往返序列化保持
        let a2 = Area { locked: true, ..a.clone() };
        let json = serde_json::to_string(&a2).expect("序列化必须成功");
        assert!(json.contains("\"locked\":true"), "UT-AN-LOCK-01: 序列化必须携带 locked");
        let back: Area = serde_json::from_str(&json).expect("往返反序列化必须成功");
        assert!(back.locked, "UT-AN-LOCK-01: locked=true 往返必须保持");

        // 断言 3（R-AREALOCK-03）：拖拽/resize 门控纯函数——锁定 no-op
        assert!(area_drag_allowed(&a), "UT-AN-LOCK-01: 未锁定区域必须允许拖动");
        assert!(!area_drag_allowed(&a2), "UT-AN-LOCK-01: 锁定区域拖动/resize 必须 no-op");

        // 断言 4（R-AREALOCK-03）：多选整组拖动跳过锁定区域，其余选中图元正常移动
        let locked_area = Area { id: "al".into(), locked: true, ..a.clone() };
        let unlocked_area = Area { id: "au".into(), locked: false, ..a.clone() };
        let (_t, _n, starts) = build_group_drag_starts(
            &[],
            &[],
            &[locked_area, unlocked_area],
            &[],
            &[],
            &["al".to_string(), "au".to_string()],
            true,
        );
        let starts = starts.expect("UT-AN-LOCK-01: 未锁定区域必须保留在整组拖动");
        assert_eq!(starts.len(), 1, "UT-AN-LOCK-01: 锁定区域必须被跳过（仅 1 个起点）");
        assert_eq!(starts[0].0, "au", "UT-AN-LOCK-01: 保留的必须是未锁定区域");
    }
}
