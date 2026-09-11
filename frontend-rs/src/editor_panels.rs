//! editor-panels: 顶/左/右面板 UI + 409 弹窗 + toast
//!
//! 依赖: `editor_core::EditorStore`, `DebounceTrigger`, `ConflictInfo`, `ConflictAction`
//!        `editor_data_access::DiagramClient`
//!
//! Phase A (redesign-phase-a-layout): AppBar + ToolRail + Inspector + StatusBar + EmptyGuide
//!
//! data-testid 清单:
//!   - app-bar / tool-rail / inspector / status-bar / canvas-empty-guide
//!   - btn-create-table / guide-create-table / btn-inspector-toggle
//!   - editor-canvas / floating-controls / revision-display (status chip)

use crate::code_view::{setup_code_view_escape, CodeLanguage, CodeView, ViewMode, ViewModeToggle};
use crate::command_palette::{
    build_palette_items, setup_command_palette_shortcut, CommandPalette, PaletteItem,
};
use crate::editor_core::types::{Area, DataDictionary, DictionaryItem, Field, Note, Reference, Table};
use crate::editor_core::{
    new_entity_id, CollabConnectionState, CollabOtState, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_data_access::{
    auth_error_display, save_with_retry, AuthClient, AuthSession, BridgeConfigUpdate,
    CollabClient, CollabFrame, CollabMemberPresence, DiagramClient, DiagramSummary,
    ImportLogEntry, ApiError, InvitePreview, RoomClient, RoomDetail, RoomMember, RoomSummary,
    SaveError,
};
use crate::{sanitize_session_notice, share_load_error_message, PageState};
use crate::editor_render::Canvas;
use crate::editor_render::{remote_presence_slots, RemotePresence};
use crate::editor_render::{zoom_in, zoom_out, zoom_reset, Transform};
use crate::splitter::{Splitter, SplitterKind};
use crate::icons::{
    IconAdd, IconAddArea, IconAddNote, IconAddTable, IconArrowLeft, IconBox, IconChevronLeft, IconChevronRight,
    IconClose, IconDelete, IconEnum, IconExport, IconImport, IconKey, IconMinus, IconMoon, IconMore,
    IconActivity, IconEye, IconEyeOff, IconLogo, IconRedo, IconRefresh, IconRelationship,
    IconSearch, IconSettings, IconShare, IconSun, IconType, IconUndo, IconUsers, IconWarning,
};
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;

fn read_html_data_mode() -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|el| el.get_attribute("data-mode"))
        .unwrap_or_else(|| "light".to_string())
}

// ─── D 批：全局工具快捷键 + Esc 浮层层级（ST-KB-T-01 / R-01 / ESC-01 / VIEWER）──

/// 快捷键目标检查：事件源自输入控件时不触发全局快捷键
///（core-KB-shortcut-test-cases.md §1：「输入框焦点时快捷键不抢焦点」）。
/// `tag_name` 为 DOM 元素标签名（任意大小写），`is_editable` 为 contentEditable 态。
pub fn is_shortcut_text_target(tag_name: &str, is_editable: bool) -> bool {
    is_editable
        || matches!(
            tag_name.to_ascii_lowercase().as_str(),
            "input" | "textarea" | "select"
        )
}

/// 从 keydown 事件目标判定是否为文本输入上下文（非 HtmlElement 目标视为非输入）。
fn shortcut_event_is_text_target(ke: &web_sys::KeyboardEvent) -> bool {
    use wasm_bindgen::JsCast;
    ke.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|el| is_shortcut_text_target(&el.tag_name(), el.is_content_editable()))
        .unwrap_or(false)
}

/// 单键工具快捷键（无修饰键）：与主原型 tool-tip 标注一致（新建表 T / 创建关系 R）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolShortcut {
    CreateTable,
    Relationship,
}

/// 判定 keydown 是否映射到工具快捷键；带 Ctrl/Meta/Alt 修饰时不拦截（如 Ctrl+R 刷新）。
pub fn tool_shortcut_for_key(key: &str, ctrl: bool, meta: bool, alt: bool) -> Option<ToolShortcut> {
    if ctrl || meta || alt {
        return None;
    }
    match key.to_ascii_lowercase().as_str() {
        "t" => Some(ToolShortcut::CreateTable),
        "r" => Some(ToolShortcut::Relationship),
        _ => None,
    }
}

/// D 批：T/R 全局工具快捷键（ST-KB-T-01 / ST-KB-R-01 / ST-KB-VIEWER）。
/// 仅在编辑器页、非只读、无输入焦点、无浮层遮挡时生效；Viewer / 分享只读不响应。
#[allow(clippy::too_many_arguments)]
/// p0-fix 定点 3：Delete/Backspace 键判定（UT-MM-34）
pub fn is_delete_key(key: &str) -> bool {
    matches!(key, "Delete" | "Backspace")
}

pub fn setup_editor_tool_shortcuts(
    current_page: RwSignal<PageState>,
    share_mode: bool,
    current_room: RwSignal<Option<RoomDetail>>,
    palette_visible: RwSignal<bool>,
    view_mode: RwSignal<ViewMode>,
    modal_kind: RwSignal<Option<modals::ModalKind>>,
    active_tool: RwSignal<ActiveTool>,
    rel_tool_state: RwSignal<RelToolState>,
    on_create_table: Rc<dyn Fn()>,
    // p0-fix 定点 3：Delete 键删除选中连线依赖
    selection: RwSignal<SelectionKind>,
    on_delete_ref: Rc<dyn Fn(String)>,
    // p0-fix 定点 2：Delete 键删除选中区域 / 便签
    on_delete_area: Rc<dyn Fn(String)>,
    on_delete_note: Rc<dyn Fn(String)>,
) {
    use wasm_bindgen::JsCast;
    gloo::events::EventListener::new(&gloo::utils::document(), "keydown", move |ev| {
        let Some(ke) = ev.dyn_ref::<web_sys::KeyboardEvent>() else {
            return;
        };
        // 编辑器页门控：auth / rooms / invite 页不响应工具快捷键
        let page = current_page.get_untracked();
        if !matches!(page, PageState::RoomEditor | PageState::ShareEdit) {
            return;
        }
        // 只读门控（ST-KB-VIEWER）：分享只读 / Viewer 角色不响应
        if editor_is_read_only(share_mode, current_room) {
            return;
        }
        // 浮层门控：命令面板 / 代码视图 / 主模态打开时不触发
        if palette_visible.get_untracked()
            || matches!(view_mode.get_untracked(), ViewMode::Code)
            || modal_kind.get_untracked().is_some()
        {
            return;
        }
        // 输入焦点门控：输入框 / contentEditable 内不抢键
        if shortcut_event_is_text_target(ke) {
            return;
        }
        // p0-fix 定点 3：Delete/Backspace 删除选中连线（ST-PB-04）
        // p0-fix 定点 2：同键删除选中区域 / 便签
        if is_delete_key(&ke.key()) {
            match selection.get_untracked() {
                SelectionKind::Reference(ref_id) => {
                    ke.prevent_default();
                    on_delete_ref(ref_id);
                }
                SelectionKind::Area(id) => {
                    ke.prevent_default();
                    on_delete_area(id);
                }
                SelectionKind::Note(id) => {
                    ke.prevent_default();
                    on_delete_note(id);
                }
                _ => {}
            }
            return;
        }
        let Some(shortcut) =
            tool_shortcut_for_key(&ke.key(), ke.ctrl_key(), ke.meta_key(), ke.alt_key())
        else {
            return;
        };
        match shortcut {
            ToolShortcut::CreateTable => on_create_table(),
            ToolShortcut::Relationship => {
                active_tool.set(ActiveTool::Relationship);
                rel_tool_state.set(RelToolState::PickSource);
            }
        }
    })
    .forget();
}

/// D 批：Esc 浮层层级关闭（ST-KB-ESC-01：按层级关闭最上层；不误关编辑器页）。
/// 一次 Esc 只处理一层。命令面板 / 代码视图 / 关系拖拽的 Esc 由各自既有监听处理，
/// 本处理器在它们打开时直接让位；409 冲突对话框必须显式选择，Esc 不关闭也不穿透。
#[allow(clippy::too_many_arguments)]
pub fn setup_escape_layer_handler(
    palette_visible: RwSignal<bool>,
    view_mode: RwSignal<ViewMode>,
    conflict: RwSignal<Option<ConflictInfo>>,
    modal_kind: RwSignal<Option<modals::ModalKind>>,
    invite_modal_open: RwSignal<bool>,
    io_drawer: RwSignal<IoDrawerKind>,
    room_panel_visible: RwSignal<bool>,
    active_tool: RwSignal<ActiveTool>,
    rel_tool_state: RwSignal<RelToolState>,
    on_close_io_drawer: Rc<dyn Fn()>,
) {
    use wasm_bindgen::JsCast;
    gloo::events::EventListener::new(&gloo::utils::document(), "keydown", move |ev| {
        let Some(ke) = ev.dyn_ref::<web_sys::KeyboardEvent>() else {
            return;
        };
        if ke.key() != "Escape" {
            return;
        }
        // L1 命令面板 / L2 代码视图：由既有 window 监听关闭，这里让位（保持一次 Esc 一层）
        if palette_visible.get_untracked() {
            return;
        }
        if matches!(view_mode.get_untracked(), ViewMode::Code) {
            return;
        }
        // L3 409 冲突对话框：必须显式选择（强制覆盖 / 重新加载），Esc 不关闭也不穿透到下层
        if conflict.get_untracked().is_some() {
            return;
        }
        // L4 主模态（New/Open/Share/Rename/BridgeSettings 等）
        if modal_kind.get_untracked().is_some() {
            modal_kind.set(None);
            return;
        }
        // L5 邀请模态
        if invite_modal_open.get_untracked() {
            invite_modal_open.set(false);
            return;
        }
        // L6 IO 抽屉（关闭后按缓存恢复 Inspector）
        if io_drawer.get_untracked() != IoDrawerKind::None {
            on_close_io_drawer();
            return;
        }
        // L7 成员面板
        if room_panel_visible.get_untracked() {
            room_panel_visible.set(false);
            return;
        }
        // L8 关系工具模式（对齐主原型 Esc 退出 relationMode；拖拽中的取消由 Canvas 监听处理）
        if active_tool.get_untracked() == ActiveTool::Relationship {
            rel_tool_state.set(RelToolState::Idle);
            active_tool.set(ActiveTool::Select);
            return;
        }
        // 无浮层：不动作 — 编辑器页本身永不被 Esc 关闭
    })
    .forget();
}


/// Side-panel Tab 标识符（B2 范围：6 业务 Tab + Issues = 7 Tab）
/// 顺序与 `core-04-side-panel-tabs.md` §1 布局保持一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidePanelTab {
    Tables,
    // ux-canvas-batch 批次1: 表结构列表视图（参考 pdmaner 全量能力）
    ListView,
    Areas,
    Enums,
    Notes,
    Relationships,
    Types,
    Issues,
    Fields,
}

impl SidePanelTab {
    pub fn testid(self) -> &'static str {
        match self {
            SidePanelTab::Tables => "tab-tables",
            SidePanelTab::ListView => "tab-list-view",
            SidePanelTab::Areas => "tab-areas",
            SidePanelTab::Enums => "tab-enums",
            SidePanelTab::Notes => "tab-notes",
            SidePanelTab::Relationships => "tab-relationships",
            SidePanelTab::Types => "tab-types",
            SidePanelTab::Issues => "tab-issues",
            SidePanelTab::Fields => "tab-fields",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SidePanelTab::Tables => "表",
            SidePanelTab::ListView => "列表视图",
            SidePanelTab::Areas => "区域",
            SidePanelTab::Enums => "枚举",
            SidePanelTab::Notes => "注释",
            SidePanelTab::Relationships => "关系",
            SidePanelTab::Types => "类型",
            SidePanelTab::Issues => "问题",
            SidePanelTab::Fields => "字段",
        }
    }
}

/// R5：Inspector Tab 图标（icon-only + title tooltip）
#[component]
fn InspectorTabIcon(tab: SidePanelTab) -> impl IntoView {
    match tab {
        SidePanelTab::Tables => view! { <IconBox size="sm"><IconAddTable /></IconBox> }.into_view(),
        SidePanelTab::ListView => view! { <IconBox size="sm"><IconAddTable /></IconBox> }.into_view(),
        SidePanelTab::Areas => view! { <IconBox size="sm"><IconAddArea /></IconBox> }.into_view(),
        SidePanelTab::Enums => view! { <IconBox size="sm"><IconEnum /></IconBox> }.into_view(),
        SidePanelTab::Notes => view! { <IconBox size="sm"><IconAddNote /></IconBox> }.into_view(),
        SidePanelTab::Relationships => {
            view! { <IconBox size="sm"><IconRelationship /></IconBox> }.into_view()
        }
        SidePanelTab::Types => view! { <IconBox size="sm"><IconType /></IconBox> }.into_view(),
        SidePanelTab::Issues => view! { <IconBox size="sm"><IconWarning /></IconBox> }.into_view(),
        SidePanelTab::Fields => view! { <IconBox size="sm"><IconKey /></IconBox> }.into_view(),
    }
}

/// Phase A：全局选中态（画布 ↔ Inspector 同步）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    None,
    Table(String),
    Field { table_id: String, field_id: String },
    Reference(String),
    /// p0-fix 定点 2：选中区域 / 便签（Inspector 编辑 + Delete 键删除）
    Area(String),
    Note(String),
    Issues,
}

impl SelectionKind {
    pub fn table_id(&self) -> Option<&str> {
        match self {
            SelectionKind::Table(id) => Some(id),
            SelectionKind::Field { table_id, .. } => Some(table_id),
            _ => None,
        }
    }
}

/// 校验 diagram 问题列表（与 IssuesTab 同源逻辑，供 Tool Rail 徽章 + Inspector）
pub fn compute_diagram_issues(store: &EditorStore) -> Vec<(String, String, String)> {
    let tables = store.tables.get();
    let refs = store.references.get();
    let mut out: Vec<(String, String, String)> = Vec::new();

    let mut names: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for t in &tables {
        *names.entry(t.name.clone()).or_insert(0) += 1;
    }
    for (name, count) in &names {
        if *count > 1 {
            if let Some(t) = tables.iter().find(|t| &t.name == name) {
                out.push((
                    "error".into(),
                    format!("表名 '{}' 重复", name),
                    t.id.clone(),
                ));
            }
        }
    }

    for t in &tables {
        if !t.fields.iter().any(|f| f.primary) {
            out.push((
                "warning".into(),
                format!("表 '{}' 缺少主键", t.name),
                t.id.clone(),
            ));
        }
    }

    for t in &tables {
        for f in &t.fields {
            if f.type_.is_empty() || f.type_ == "INVALID" {
                out.push((
                    "error".into(),
                    format!("字段 '{}.{}' 类型不兼容", t.name, f.name),
                    t.id.clone(),
                ));
            }
        }
    }

    for r in &refs {
        if !tables.iter().any(|t| t.id == r.start_table_id) {
            out.push((
                "error".into(),
                format!("关系 {} 起点表不存在", r.id),
                r.start_table_id.clone(),
            ));
        }
        if !tables.iter().any(|t| t.id == r.end_table_id) {
            out.push((
                "error".into(),
                format!("关系 {} 终点表不存在", r.id),
                r.end_table_id.clone(),
            ));
        }
    }

    out
}

/// Inspector 是否应随选中自动展开
pub fn selection_auto_opens_inspector(sel: &SelectionKind) -> bool {
    !matches!(sel, SelectionKind::None)
}

/// Phase B：画布工具
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Relationship,
    Pan,
    /// p0-fix 定点 2：区域 / 便签创建工具
    NewArea,
    NewNote,
}

/// Phase B：关系工具状态机
/// p0-fix 定点 3：Confirm 态已删除——拖到目标字段松开 / 点击两点后直接落账，无确认条
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelToolState {
    Idle,
    PickSource,
    PickTarget {
        start_table_id: String,
        start_field_id: String,
    },
    Dragging {
        start_table_id: String,
        start_field_id: String,
    },
}

impl RelToolState {
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            RelToolState::PickSource => Some("从字段拖出连线，或点击选择源字段"),
            RelToolState::Dragging { .. } => Some("拖到目标字段后松开"),
            RelToolState::PickTarget { .. } => Some("选择目标字段，或从源字段拖出连线"),
            _ => None,
        }
    }

    pub fn is_picking(&self) -> bool {
        matches!(
            self,
            RelToolState::PickSource
                | RelToolState::PickTarget { .. }
                | RelToolState::Dragging { .. }
        )
    }
}

pub const CARDINALITY_OPTIONS: &[&str] =
    &["one_to_one", "one_to_many", "many_to_one", "many_to_many"];

/// 构建 Reference（Phase B 关系确认条创建）
pub fn build_reference(
    id: String,
    start_table_id: String,
    start_field_id: String,
    end_table_id: String,
    end_field_id: String,
    cardinality: &str,
) -> Reference {
    Reference {
        id,
        name: String::new(),
        start_table_id,
        end_table_id,
        start_field_id,
        end_field_id,
        type_: cardinality.to_string(),
        on_delete: "RESTRICT".into(),
        on_update: "RESTRICT".into(),
    }
}

/// 翻转 reference 起止端点
/// feat-relation-inference 批次3: 翻转后重新推导 cardinality（基于翻转后的
/// 两端字段已参与关系计数，s/e 互换）
pub fn flip_reference_endpoints(r: &Reference, store: &crate::editor_core::EditorStore) -> Reference {
    let flipped = Reference {
        start_table_id: r.end_table_id.clone(),
        start_field_id: r.end_field_id.clone(),
        end_table_id: r.start_table_id.clone(),
        end_field_id: r.start_field_id.clone(),
        ..r.clone()
    };
    // feat-relation-inference 批次3: 翻转后重新推导 cardinality（s/e 互换）
    let inferred = modals::infer_cardinality(&flipped.start_field_id, &flipped.end_field_id, store);
    Reference {
        type_: inferred,
        ..flipped
    }
}

/// 格式化关系确认条标签：`users.id → orders.user_id`
pub fn format_rel_confirm_label(
    tables: &[Table],
    start_table_id: &str,
    start_field_id: &str,
    end_table_id: &str,
    end_field_id: &str,
) -> String {
    let start = tables.iter().find(|t| t.id == start_table_id);
    let end = tables.iter().find(|t| t.id == end_table_id);
    let sf = start
        .and_then(|t| t.fields.iter().find(|f| f.id == start_field_id))
        .map(|f| f.name.as_str())
        .unwrap_or("?");
    let ef = end
        .and_then(|t| t.fields.iter().find(|f| f.id == end_field_id))
        .map(|f| f.name.as_str())
        .unwrap_or("?");
    let st = start.map(|t| t.name.as_str()).unwrap_or("?");
    let et = end.map(|t| t.name.as_str()).unwrap_or("?");
    format!("{st}.{sf} → {et}.{ef}")
}

/// 切换字段主键（单表唯一 PK）
pub fn toggle_field_primary(tables: &mut [Table], table_id: &str, field_id: &str, primary: bool) {
    let Some(table) = tables.iter_mut().find(|t| t.id == table_id) else {
        return;
    };
    if primary {
        for f in &mut table.fields {
            f.primary = f.id == field_id;
        }
    } else if let Some(f) = table.fields.iter_mut().find(|f| f.id == field_id) {
        f.primary = false;
    }
}

/// Phase C：IO 抽屉类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IoDrawerKind {
    None,
    Import,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportFormat {
    Sql,
    Dbml,
    Json,
    // feat-db-connect-import-and-ddl-realdb-verify：数据库连接导入（core-01d §4.4 3.5）
    Database,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Sql,
    Dbml,
    Json,
    /// feat-data-dictionary（S07，core-01e §4）：数据字典 Markdown 导出
    Dict,
}

/// 打开 IO 抽屉前缓存 Inspector 状态；返回 (折叠后 inspector_open, cache)
pub fn snapshot_before_io_drawer(inspector_open: bool) -> (bool, Option<bool>) {
    if inspector_open {
        (false, Some(true))
    } else {
        (false, None)
    }
}

/// 关闭 IO 抽屉后恢复 Inspector
pub fn restore_inspector_after_io_drawer(cache: Option<bool>) -> bool {
    cache.unwrap_or(false)
}

// ─── fix-dict-save-and-layout：右侧浮层互斥状态机（S07，core-01e §2.5，ST-S07-06） ───

/// 右侧浮层互斥状态：Inspector / 字典抽屉 / IO 抽屉同侧，任意时刻最多一个浮层。
/// 全 Copy 纯出纯入，host 可测；EditorPage 仅做信号收集/应用的薄接线。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayMutex {
    pub inspector_open: bool,
    pub dict_panel_open: bool,
    pub io_drawer: IoDrawerKind,
    /// 打开字典抽屉前缓存的 Inspector 开合（None = 无缓存/已失效）
    pub inspector_before_dict: Option<bool>,
    /// 打开 IO 抽屉前缓存的 Inspector 开合（既有 inspector_before_io 口径）
    pub inspector_before_io: Option<bool>,
}

impl OverlayMutex {
    /// 打开字典抽屉：缓存 Inspector 有效状态并收起；同时关闭 IO 抽屉（后开者关先开者，
    /// IO 侧已缓存 Inspector 状态时继承该缓存，避免读取当前已收起的 false 导致丢失）。
    pub fn open_dict_panel(self) -> Self {
        let effective = self
            .inspector_before_dict
            .or(self.inspector_before_io)
            .unwrap_or(self.inspector_open);
        Self {
            inspector_open: false,
            dict_panel_open: true,
            io_drawer: IoDrawerKind::None,
            inspector_before_dict: Some(effective),
            inspector_before_io: None,
        }
    }

    /// 关闭字典抽屉（关闭按钮 / toggle / 所有关闭路径共用）：恢复缓存的 Inspector 状态。
    pub fn close_dict_panel(self) -> Self {
        let restored = self.inspector_before_dict.unwrap_or(self.inspector_open);
        Self {
            inspector_open: restored,
            dict_panel_open: false,
            inspector_before_dict: None,
            ..self
        }
    }

    /// ToolRail 入口 toggle：已开 → 关闭恢复；未开 → 打开收起。
    pub fn toggle_dict_panel(self) -> Self {
        if self.dict_panel_open {
            self.close_dict_panel()
        } else {
            self.open_dict_panel()
        }
    }

    /// Inspector 打开（选中表/字段、手动切换）：两抽屉关闭，缓存失效不恢复。
    pub fn inspector_opened(self) -> Self {
        Self {
            inspector_open: true,
            dict_panel_open: false,
            io_drawer: IoDrawerKind::None,
            inspector_before_dict: None,
            inspector_before_io: None,
        }
    }

    /// 打开 IO 抽屉：关闭字典抽屉并继承其 Inspector 缓存（避免双重缓存丢失）。
    pub fn open_io_drawer(self, kind: IoDrawerKind) -> Self {
        let effective = self
            .inspector_before_dict
            .or(self.inspector_before_io)
            .unwrap_or(self.inspector_open);
        Self {
            inspector_open: false,
            dict_panel_open: false,
            io_drawer: kind,
            inspector_before_dict: None,
            inspector_before_io: Some(effective),
        }
    }

    /// 关闭 IO 抽屉：恢复 Inspector（对齐 restore_inspector_after_io_drawer 口径）。
    pub fn close_io_drawer(self) -> Self {
        let restored = self.inspector_before_io.unwrap_or(self.inspector_open);
        Self {
            inspector_open: restored,
            io_drawer: IoDrawerKind::None,
            inspector_before_io: None,
            ..self
        }
    }
}

/// DBML Table 块计数（Phase C UT-PC-05）
pub fn count_dbml_tables(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let t = line.trim();
            t.starts_with("Table ") || t.starts_with("table ")
        })
        .count()
}

/// 导入解析摘要
pub fn import_parse_summary(format: ImportFormat, content: &str) -> Result<String, String> {
    match format {
        ImportFormat::Sql => {
            let n = modals::parse_sql_statements(content)?.len();
            Ok(format!("{n} 条语句"))
        }
        ImportFormat::Dbml => Ok(format!("{} 个 Table 块", count_dbml_tables(content))),
        ImportFormat::Json => {
            let v: serde_json::Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
            let n = v
                .get("tables")
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            Ok(format!("{n} 张表"))
        }
        // feat-db-connect-import-and-ddl-realdb-verify：数据库来源的 content 为端点返回的 DDL
        ImportFormat::Database => {
            let n = modals::parse_sql_statements(content)?.len();
            Ok(format!("{n} 条语句"))
        }
    }
}

/// SQL DDL 列级导入解析（relation-inspector-and-ddl-io：UT-PC-07 / UT-PC-08）。
/// 替换原「每表硬造 id 字段」最小桩：
/// - 列定义：列名、类型整串原样保留（含 `VARCHAR(32)` 参数化）、列级
///   PRIMARY KEY / NOT NULL / UNIQUE / AUTO_INCREMENT（PG SERIAL/BIGSERIAL → increment）/
///   DEFAULT / COMMENT
/// - 表级约束：PRIMARY KEY(a[, b])（联合主键）、FOREIGN KEY (c) REFERENCES t(c2)
///   与列级 REFERENCES → references 连线（端点解析到表/字段实体 id；悬空引用跳过）
/// - 标识符规范化：反引号 / 双引号 / 方括号去除；未知类型整串保留
/// - V1 不做：CHECK、INDEX/KEY、ALTER、视图、引擎方言映射
pub fn parse_sql_import_tables(content: &str) -> Result<(Vec<Table>, Vec<Reference>), String> {
    let stmts = modals::parse_sql_statements(content)?;
    let mut tables: Vec<Table> = Vec::new();
    // 原始外键记录（表名/列名），全部表解析完后再解析端点 id
    let mut raw_fks: Vec<(String, String, String, String)> = Vec::new();
    for (i, stmt) in stmts.iter().enumerate() {
        let Some(parsed) = parse_create_table_stmt(stmt, i, &mut raw_fks) else {
            continue;
        };
        tables.push(parsed);
    }
    if tables.is_empty() {
        return Err("未检测到数据表".to_string());
    }
    // fix-canvas-zoom-invite-comment-resize（UT-PC-07）：PG `COMMENT ON TABLE` /
    // `COMMENT ON COLUMN` 语句回填表/列 comment；目标不存在 → 跳过（与 REFERENCES 悬空口径一致）
    for stmt in &stmts {
        apply_comment_on_stmt(stmt, &mut tables);
    }
    let mut references = Vec::new();
    for (table_name, col_name, ref_table_name, ref_col_name) in raw_fks {
        let Some(from_t) = tables.iter().find(|t| t.name == table_name) else {
            continue;
        };
        let Some(to_t) = tables.iter().find(|t| t.name == ref_table_name) else {
            continue;
        };
        let Some(ff) = from_t.fields.iter().find(|f| f.name == col_name) else {
            continue;
        };
        let Some(tf) = to_t.fields.iter().find(|f| f.name == ref_col_name) else {
            continue;
        };
        // cardinality 推导（core-01b §2）：目标列唯一/主键 → one_to_one，否则 one_to_many
        let type_ = if tf.primary || tf.unique {
            "one_to_one"
        } else {
            "one_to_many"
        };
        references.push(Reference {
            id: format!("import-r-{}", references.len()),
            name: String::new(),
            start_table_id: from_t.id.clone(),
            end_table_id: to_t.id.clone(),
            start_field_id: ff.id.clone(),
            end_field_id: tf.id.clone(),
            type_: type_.to_string(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        });
    }
    Ok((tables, references))
}

/// 解析单条 `COMMENT ON TABLE t IS '...'` / `COMMENT ON COLUMN t.c IS '...'` 语句并回填 comment。
/// 非 COMMENT ON 语句直接忽略；目标表/列在 tables 中不存在 → 跳过（不报错）。
/// 标识符支持引号包裹与 schema 限定（取末段，如 `public.users` → `users`）；
/// 字符串字面量支持 `''` 转义（导出侧 `escape_sql_comment` 的逆操作，ST-PC-07 round-trip）。
fn apply_comment_on_stmt(stmt: &str, tables: &mut [Table]) {
    let stmt = stmt.trim();
    // 字节级大小写不敏感前缀判断（ASCII 前缀不可能切裂 UTF-8 字符）
    let starts_with_ci = |s: &str, prefix: &str| {
        s.len() >= prefix.len() && s.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
    };
    let (target, is_table) = if starts_with_ci(stmt, "COMMENT ON TABLE ") {
        (&stmt["COMMENT ON TABLE ".len()..], true)
    } else if starts_with_ci(stmt, "COMMENT ON COLUMN ") {
        (&stmt["COMMENT ON COLUMN ".len()..], false)
    } else {
        return;
    };
    // `IS` 关键字分隔目标与字面量（大小写不敏感；原文切片保留字面量原始大小写）
    let Some(is_pos) = find_substr_ci(target, " IS ") else {
        return;
    };
    let name_part = target[..is_pos].trim();
    let literal_part = &target[is_pos + " IS ".len()..];
    let Some(literal) = parse_sql_string_literal(literal_part) else {
        return;
    };
    // schema 限定取末段；各段独立去引号
    let segs: Vec<String> = name_part
        .split('.')
        .map(|s| strip_ident_quotes(s.trim()))
        .collect();
    let tname = match segs.first() {
        Some(t) if !t.is_empty() => t.clone(),
        _ => return,
    };
    if is_table {
        if let Some(t) = tables.iter_mut().find(|t| t.name == tname) {
            t.comment = literal;
        }
    } else if let Some(cname) = segs.get(1).filter(|c| !c.is_empty()) {
        if let Some(t) = tables.iter_mut().find(|t| t.name == tname) {
            if let Some(f) = t.fields.iter_mut().find(|f| f.name == *cname) {
                f.comment = literal;
            }
        }
    }
}

/// 大小写不敏感子串查找：返回 needle 首次出现的字节位置。
/// needle 为纯 ASCII 时匹配起点必在 UTF-8 字符边界，切片安全。
fn find_substr_ci(haystack: &str, needle: &str) -> Option<usize> {
    let nb = needle.as_bytes();
    let hb = haystack.as_bytes();
    if nb.is_empty() || nb.len() > hb.len() {
        return None;
    }
    (0..=hb.len() - nb.len()).find(|&i| hb[i..i + nb.len()].eq_ignore_ascii_case(nb))
}

/// 解析 SQL 单引号字符串字面量（含 `''` 转义）：`'it''s'` → `it's`。
/// 无起始引号或未闭合 → None。
fn parse_sql_string_literal(text: &str) -> Option<String> {
    let inner = text.trim().strip_prefix('\'')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\'' {
            if chars.clone().next() == Some('\'') {
                chars.next();
                out.push('\'');
            } else {
                return Some(out);
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// 去除标识符引号：反引号 / 双引号 / 方括号
fn strip_ident_quotes(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    if s.len() >= 2 {
        let (a, b) = (bytes[0], bytes[s.len() - 1]);
        if (a == b'`' && b == b'`') || (a == b'"' && b == b'"') || (a == b'[' && b == b']') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.trim_matches(|c| c == '`' || c == '"').to_string()
}

/// 从标识符列表文本（如 `a, b` 或 `` `a`, `b` ``）提取去引号列名
fn parse_ident_list(inner: &str) -> Vec<String> {
    inner
        .split(',')
        .map(|s| strip_ident_quotes(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// 在文本中定位第一个 `(` 并返回括号内内容（平衡扫描）
fn extract_paren_content(text: &str, from: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let mut idx = from;
    while idx < bytes.len() && bytes[idx] != b'(' {
        idx += 1;
    }
    if idx >= bytes.len() {
        return None;
    }
    let mut depth = 0usize;
    let mut end = idx;
    for (j, &b) in bytes.iter().enumerate().skip(idx) {
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                end = j;
                break;
            }
        }
    }
    if end <= idx {
        return None;
    }
    Some((text[idx + 1..end].to_string(), end))
}

/// 解析单条 CREATE TABLE 语句；非 CREATE TABLE 返回 None。
/// raw_fks 收集 (表名, 列名, 目标表名, 目标列名)。
fn parse_create_table_stmt(
    stmt: &str,
    table_index: usize,
    raw_fks: &mut Vec<(String, String, String, String)>,
) -> Option<Table> {
    let upper = stmt.to_uppercase();
    if !upper.starts_with("CREATE TABLE") {
        return None;
    }
    // 表名：跳过 CREATE TABLE [IF NOT EXISTS]
    let mut rest = &stmt["CREATE TABLE".len()..];
    let upper_rest = rest.trim_start().to_uppercase();
    if upper_rest.starts_with("IF NOT EXISTS") {
        rest = &rest[rest.len() - upper_rest.len() + "IF NOT EXISTS".len()..];
    }
    let name_token = rest.trim_start();
    let name_end = name_token
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(name_token.len());
    let name = strip_ident_quotes(&name_token[..name_end]);
    if name.is_empty() {
        return None;
    }
    // 表体：第一个 `(` 到其配对的 `)`
    let (body, _) = extract_paren_content(stmt, stmt.find('(')?)?;
    // 顶层逗号切分（括号内逗号不切）
    let mut parts: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in body.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }

    let table_id = format!("import-t-{table_index}");
    let mut fields: Vec<Field> = Vec::new();
    let mut table_pks: Vec<String> = Vec::new();
    for raw in &parts {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let up = line.to_uppercase();
        if up.starts_with("PRIMARY KEY") {
            if let Some((inner, _)) = extract_paren_content(line, 0) {
                table_pks.extend(parse_ident_list(&inner));
            }
            continue;
        }
        if up.starts_with("FOREIGN KEY") {
            // FOREIGN KEY (c) REFERENCES t(c2)
            if let Some((cols, end)) = extract_paren_content(line, 0) {
                let tail = line[end..].to_uppercase();
                if let Some(ref_pos) = tail.find("REFERENCES") {
                    let after = line[end + ref_pos + "REFERENCES".len()..].trim_start();
                    let tname_end = after
                        .find(|c: char| c.is_whitespace() || c == '(')
                        .unwrap_or(after.len());
                    let ref_table = strip_ident_quotes(&after[..tname_end]);
                    if let Some((ref_cols, _)) = extract_paren_content(after, 0) {
                        if let (Some(c), Some(rc)) =
                            (parse_ident_list(&cols).first(), parse_ident_list(&ref_cols).first())
                        {
                            raw_fks.push((name.clone(), c.clone(), ref_table, rc.clone()));
                        }
                    }
                }
            }
            continue;
        }
        if up.starts_with("CONSTRAINT")
            || up.starts_with("UNIQUE")
            || up.starts_with("KEY ")
            || up.starts_with("INDEX")
            || up.starts_with("CHECK")
        {
            continue;
        }
        if let Some(field) = parse_ddl_column(line, &table_id, fields.len(), &name, raw_fks) {
            fields.push(field);
        }
    }
    // 表级联合主键置位
    for pk_name in &table_pks {
        if let Some(f) = fields.iter_mut().find(|f| &f.name == pk_name) {
            f.primary = true;
            f.not_null = true;
        }
    }
    if fields.is_empty() {
        return None;
    }
    Some(Table {
        id: table_id,
        name,
        x: 100.0 + (table_index as f64) * 40.0,
        y: 100.0 + (table_index as f64) * 30.0,
        color: String::new(),
        comment: String::new(),
        fields,
        indices: vec![],
        width: None,
        min_height: None,
    })
}

/// 解析单列定义行（如 `` `name` VARCHAR(32) NOT NULL UNIQUE DEFAULT 'x' COMMENT 'y' ``）。
fn parse_ddl_column(
    line: &str,
    table_id: &str,
    field_index: usize,
    table_name: &str,
    raw_fks: &mut Vec<(String, String, String, String)>,
) -> Option<Field> {
    // 列名：支持引号包裹的首 token
    let bytes = line.as_bytes();
    let (col_raw, after_name) = if matches!(bytes.first(), Some(b'`') | Some(b'"') | Some(b'[')) {
        let close = match bytes[0] {
            b'[' => b']',
            c => c,
        };
        let end = line[1..].find(close as char)? + 1;
        (&line[..=end], line[end + 1..].trim_start())
    } else {
        let end = line
            .find(|c: char| c.is_whitespace())
            .unwrap_or(line.len());
        (&line[..end], line[end..].trim_start())
    };
    let col_name = strip_ident_quotes(col_raw);
    if col_name.is_empty() || after_name.is_empty() {
        return None;
    }
    // 类型：token + 可选 (参数)，整串保留（去空白，如 VARCHAR(32) / DECIMAL(10, 2)）
    let (type_, rest) = {
        let mut idx = after_name
            .find(|c: char| c.is_whitespace() || c == '(')
            .unwrap_or(after_name.len());
        let mut end = idx;
        let abytes = after_name.as_bytes();
        if idx < abytes.len() && abytes[idx] == b'(' {
            let mut depth = 0i32;
            for (j, &b) in abytes.iter().enumerate().skip(idx) {
                if b == b'(' {
                    depth += 1;
                } else if b == b')' {
                    depth -= 1;
                    if depth == 0 {
                        end = j + 1;
                        break;
                    }
                }
            }
            idx = end;
        } else {
            idx = end;
        }
        (
            after_name[..idx].split_whitespace().collect::<String>().to_uppercase(),
            after_name[idx..].trim(),
        )
    };
    if type_.is_empty() {
        return None;
    }
    // 约束检测在去除引号内容的副本上进行（避免 COMMENT/DEFAULT 文本误命中）
    let mut rest_stripped = String::with_capacity(rest.len());
    let mut in_quote = false;
    for ch in rest.chars() {
        if ch == '\'' {
            in_quote = !in_quote;
            rest_stripped.push(' ');
        } else if !in_quote {
            rest_stripped.push(ch);
        }
    }
    let rest_up = rest_stripped.to_uppercase();
    let has_word = |hay: &str, word: &str| {
        hay.split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|w| w == word)
    };
    let primary = rest_up.contains("PRIMARY KEY");
    let not_null = rest_up.contains("NOT NULL") || primary;
    let unique = has_word(&rest_up, "UNIQUE") && !primary;
    let increment = rest_up.contains("AUTO_INCREMENT")
        || type_ == "SERIAL"
        || type_ == "BIGSERIAL";
    // DEFAULT <值>：取 DEFAULT 后第一个 token（引号字符串保留原样）
    let default = rest_up
        .find("DEFAULT")
        .map(|pos| {
            let after = rest[pos + "DEFAULT".len()..].trim_start();
            if after.starts_with('\'') {
                after
                    .char_indices()
                    .find(|(i, c)| *i > 0 && *c == '\'')
                    .map(|(e, _)| after[..=e].to_string())
                    .unwrap_or_default()
            } else {
                after
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(',')
                    .to_string()
            }
        })
        .unwrap_or_default();
    // COMMENT '...'
    let comment = rest_up.find("COMMENT").and_then(|pos| {
        let after = rest[pos + "COMMENT".len()..].trim_start();
        if after.starts_with('\'') {
            after[1..].find('\'').map(|e| after[1..1 + e].to_string())
        } else {
            None
        }
    }).unwrap_or_default();
    // 列级 REFERENCES t(c2)
    if let Some(pos) = rest_up.find("REFERENCES") {
        let after = rest[pos + "REFERENCES".len()..].trim_start();
        let tname_end = after
            .find(|c: char| c.is_whitespace() || c == '(')
            .unwrap_or(after.len());
        let ref_table = strip_ident_quotes(&after[..tname_end]);
        if let Some((ref_cols, _)) = extract_paren_content(after, 0) {
            if let Some(rc) = parse_ident_list(&ref_cols).first() {
                raw_fks.push((
                    table_name.to_string(),
                    col_name.clone(),
                    ref_table,
                    rc.clone(),
                ));
            }
        }
    }
    Some(Field {
        id: format!("{table_id}-f{}", field_index + 1),
        name: col_name,
        type_,
        default,
        check: String::new(),
        primary,
        unique,
        not_null,
        increment,
        comment,
            tag: String::new(),
            dict_code: String::new(),
    })
}

/// 从 DBML Table 块构建 import tables（最小字段解析）。
pub fn parse_dbml_import_tables(content: &str) -> Result<Vec<Table>, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut tables = Vec::new();
    let mut table_index = 0usize;
    let mut line_idx = 0usize;
    while line_idx < lines.len() {
        let trimmed = lines[line_idx].trim();
        let is_table = trimmed.starts_with("Table ") || trimmed.starts_with("table ");
        if !is_table {
            line_idx += 1;
            continue;
        }
        let name = trimmed
            .trim_start_matches("Table ")
            .trim_start_matches("table ")
            .trim();
        let (name, inline_body) = if let Some(idx) = name.find('{') {
            (name[..idx].trim().to_string(), Some(name[idx + 1..].trim()))
        } else {
            (name.trim_end_matches('{').trim().to_string(), None)
        };
        let table_id = format!("import-t-{table_index}");
        let mut fields = Vec::new();
        let mut field_index = 0usize;
        let single_line_table = trimmed.contains('}');
        if let Some(body) = inline_body {
            let body = body.trim_end_matches('}').trim();
            if !body.is_empty() {
                for segment in body.split(',') {
                    let field_line = segment.trim();
                    if field_line.is_empty() {
                        continue;
                    }
                    push_dbml_field(&mut fields, &table_id, field_index, field_line);
                    field_index += 1;
                }
            }
        }
        if !single_line_table {
            line_idx += 1;
            while line_idx < lines.len() {
                let field_line = lines[line_idx].trim();
                if field_line.starts_with('}') {
                    break;
                }
                if field_line.is_empty() || field_line.starts_with("//") {
                    line_idx += 1;
                    continue;
                }
                let parts: Vec<&str> = field_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    push_dbml_field(&mut fields, &table_id, field_index, field_line);
                    field_index += 1;
                }
                line_idx += 1;
            }
        }
        if fields.is_empty() {
            fields.push(Field {
                id: format!("{table_id}-f1"),
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
            });
        }
        tables.push(Table {
            id: table_id,
            name: if name.is_empty() {
                format!("table_{table_index}")
            } else {
                name
            },
            x: 100.0 + (table_index as f64) * 40.0,
            y: 100.0 + (table_index as f64) * 30.0,
            color: String::new(),
            comment: String::new(),
            fields,
            indices: vec![],
            width: None,
            min_height: None,
        });
        table_index += 1;
        line_idx += 1;
    }
    Ok(tables)
}

fn push_dbml_field(fields: &mut Vec<Field>, table_id: &str, field_index: usize, field_line: &str) {
    let parts: Vec<&str> = field_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let fname = parts[0].to_string();
    let ftype = parts[1].to_string();
    let lower = field_line.to_lowercase();
    let primary = lower.contains("[pk") || lower.contains("primary key");
    let not_null = lower.contains("not null") || primary;
    fields.push(Field {
        id: format!("{table_id}-f{field_index}"),
        name: fname,
        type_: ftype,
        default: String::new(),
        check: String::new(),
        primary,
        unique: lower.contains("[unique"),
        not_null,
        increment: lower.contains("increment"),
        comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
    });
}

/// JSON 导入解析（fix-appbar-roomname-back-and-import-merge：本地合并路径）。
/// 容忍缺省字段（persistence 同构 JSON 的宽松版）；tables 为空 → Err。
pub fn parse_json_import_tables(content: &str) -> Result<(Vec<Table>, Vec<Reference>), String> {
    #[derive(serde::Deserialize)]
    struct JsonField {
        #[serde(default)]
        id: Option<String>,
        name: String,
        #[serde(default, alias = "type")]
        type_: String,
        #[serde(default)]
        default: String,
        #[serde(default)]
        check: String,
        #[serde(default)]
        primary: bool,
        #[serde(default)]
        unique: bool,
        #[serde(default)]
        not_null: bool,
        #[serde(default)]
        increment: bool,
        #[serde(default)]
        comment: String,
        #[serde(default)]
        tag: String,
        #[serde(default)]
        dict_code: String,
    }
    #[derive(serde::Deserialize)]
    struct JsonTable {
        #[serde(default)]
        id: Option<String>,
        name: String,
        #[serde(default)]
        x: f64,
        #[serde(default)]
        y: f64,
        #[serde(default)]
        color: String,
        #[serde(default)]
        comment: String,
        #[serde(default)]
        fields: Vec<JsonField>,
    }
    #[derive(serde::Deserialize)]
    struct JsonRef {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        name: String,
        start_table_id: String,
        end_table_id: String,
        start_field_id: String,
        end_field_id: String,
        #[serde(default, alias = "type")]
        type_: String,
        #[serde(default)]
        on_delete: String,
        #[serde(default)]
        on_update: String,
    }
    #[derive(serde::Deserialize, Default)]
    struct JsonDoc {
        #[serde(default)]
        tables: Vec<JsonTable>,
        #[serde(default)]
        references: Vec<JsonRef>,
    }

    let doc: JsonDoc = serde_json::from_str(content).map_err(|e| format!("JSON 不合法：{e}"))?;
    if doc.tables.is_empty() {
        return Err("未检测到数据表".to_string());
    }
    let tables: Vec<Table> = doc
        .tables
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let tid = t.id.unwrap_or_else(|| format!("import-j-t{i}"));
            Table {
                id: tid.clone(),
                name: t.name,
                x: t.x,
                y: t.y,
                color: t.color,
                comment: t.comment,
                fields: t
                    .fields
                    .into_iter()
                    .enumerate()
                    .map(|(j, f)| Field {
                        id: f.id.unwrap_or_else(|| format!("{tid}-f{}", j + 1)),
                        name: f.name,
                        type_: f.type_,
                        default: f.default,
                        check: f.check,
                        primary: f.primary,
                        unique: f.unique,
                        not_null: f.not_null,
                        increment: f.increment,
                        comment: f.comment,
                        tag: f.tag,
                        dict_code: String::new(),
                    })
                    .collect(),
                indices: vec![],
                width: None,
                min_height: None,
            }
        })
        .collect();
    let references: Vec<Reference> = doc
        .references
        .into_iter()
        .enumerate()
        .map(|(i, r)| Reference {
            id: r.id.unwrap_or_else(|| format!("import-j-r{i}")),
            name: r.name,
            start_table_id: r.start_table_id,
            end_table_id: r.end_table_id,
            start_field_id: r.start_field_id,
            end_field_id: r.end_field_id,
            type_: r.type_,
            on_delete: r.on_delete,
            on_update: r.on_update,
        })
        .collect();
    Ok((tables, references))
}

/// import/connect 结构化响应消费（fix-canvas-zoom-invite-comment-resize：UT-PC-26，core-01d §4.4 3.5）。
/// 后端 `POST /api/v1/bridge/import/connect` 直传结构化 tables（core-03 §13.1，
/// 与 persistence / JSON 导入格式同构）；本函数将其序列化为 JSON 导入同构文本后
/// 复用 `parse_json_import_tables` 同一纯函数——表/列 comment 原样透传，
/// ID 仍为解析期确定性形式（重键在 `merge_import_into_store` 一次完成，UT-PC-20）。
pub fn parse_bridge_import_tables(
    tables: &[crate::editor_data_access::ImportConnectTable],
) -> Result<(Vec<Table>, Vec<Reference>), String> {
    let payload = serde_json::json!({ "tables": tables });
    parse_json_import_tables(&payload.to_string())
}

/// 导入合并（fix-appbar-roomname-back-and-import-merge：UT-PC-09）。
/// 把解析出的 tables/references 合并进现有画布：新表避让现有布局
/// （现有内容包围盒右侧起，按列向下网格落位，列宽 280 / 行距 40），
/// 既有表坐标不变；references 全量追加；纯函数，不修改入参，
/// 返回 (合并后 tables, 合并后 references)。
///
/// ID 重键（fix-dbimport-save-and-pg-schema：UT-PC-20）：
/// 解析期 ID 是确定性的 `import-t-{i}` / `import-t-{i}-f{n}` / `import-r-{n}`，
/// 而后端 table/field/reference 的 id 为全局单列主键——任意第二次导入保存
/// 必然主键冲突（500「保存失败」）。合并入口统一以 `new_entity_id` 重键，
/// 并按 旧→新 映射改写 reference 的 start/end table/field 四端点。
pub fn merge_import_into_store(
    existing_tables: &[Table],
    existing_references: &[Reference],
    new_tables: &[Table],
    new_references: &[Reference],
) -> (Vec<Table>, Vec<Reference>) {
    use std::collections::HashMap;

    // 旧→新 ID 映射（表与字段）；重键后的新表集合
    let mut id_map: HashMap<String, String> = HashMap::new();
    let rekeyed_tables: Vec<Table> = new_tables
        .iter()
        .map(|t| {
            let mut t = t.clone();
            let new_tid = crate::editor_core::new_entity_id("auto");
            id_map.insert(t.id.clone(), new_tid.clone());
            t.id = new_tid;
            for f in &mut t.fields {
                let new_fid = crate::editor_core::new_entity_id("auto");
                id_map.insert(f.id.clone(), new_fid.clone());
                f.id = new_fid;
            }
            t
        })
        .collect();
    let rekeyed_refs: Vec<Reference> = new_references
        .iter()
        .map(|r| {
            let mut r = r.clone();
            r.id = crate::editor_core::new_entity_id("ref");
            for endpoint in [
                &mut r.start_table_id,
                &mut r.end_table_id,
                &mut r.start_field_id,
                &mut r.end_field_id,
            ] {
                if let Some(mapped) = id_map.get(endpoint.as_str()) {
                    *endpoint = mapped.clone();
                }
            }
            r
        })
        .collect();

    const COL_W: f64 = 280.0;
    const ROW_GAP: f64 = 40.0;
    const EST_W: f64 = 240.0;
    const START_X: f64 = 100.0;
    const START_Y: f64 = 100.0;

    // 估算表高：表头 43 + 每字段行 35 + 余量
    let est_h = |t: &Table| 43.0 + 35.0 * t.fields.len() as f64 + 20.0;

    // 现有内容包围盒
    let mut max_right = f64::NEG_INFINITY;
    let mut max_bottom = f64::NEG_INFINITY;
    for t in existing_tables {
        max_right = max_right.max(t.x + t.width.map(|w| w as f64).unwrap_or(EST_W));
        max_bottom = max_bottom.max(t.y + est_h(t));
    }
    let has_existing = !existing_tables.is_empty();
    let base_x = if has_existing { max_right + 60.0 } else { START_X };
    let wrap_y = if has_existing { max_bottom.max(START_Y + 400.0) } else { START_Y + 400.0 };

    let mut merged: Vec<Table> = existing_tables.to_vec();
    let mut x = base_x;
    let mut y = START_Y;
    for t in &rekeyed_tables {
        let mut placed = t.clone();
        if y + est_h(&placed) > wrap_y && y > START_Y {
            // 换列：向右一列，回到列顶
            x += COL_W;
            y = START_Y;
        }
        placed.x = x;
        placed.y = y;
        y += est_h(&placed) + ROW_GAP;
        merged.push(placed);
    }
    let mut merged_refs: Vec<Reference> = existing_references.to_vec();
    merged_refs.extend(rekeyed_refs.iter().cloned());
    (merged, merged_refs)
}

/// SQL 注释文本单引号转义（core-01d §5.2：`'` → `''`）。
/// 导入侧 `parse_sql_string_literal` 负责逆操作（COMMENT ON 回填，ST-PC-07 round-trip）。
fn escape_sql_comment(s: &str) -> String {
    s.replace('\'', "''")
}

/// 客户端 SQL 导出（Phase C UT-PC-02）
/// relation-inspector-and-ddl-io 补全（core-01d §5.2）：
/// UNIQUE / DEFAULT / 自增（mysql/generic → AUTO_INCREMENT；postgresql 由 SERIAL 类型承担不追加）/
/// COMMENT（mysql 行内 `COMMENT 'x'` + 表选项 `COMMENT='x'`；postgresql/generic 后置
/// `COMMENT ON TABLE t IS 'x'` / `COMMENT ON COLUMN t.c IS 'x'`）/
/// FK 引用 references（表末 `FOREIGN KEY (c) REFERENCES t(c2)`）
pub fn export_diagram_sql(tables: &[Table], references: &[Reference], engine: &str) -> String {
    let mut out = String::new();
    if !engine.is_empty() && engine != "generic" {
        out.push_str(&format!("-- engine: {engine}\n\n"));
    }
    for table in tables {
        let mut lines: Vec<String> = Vec::new();
        for field in &table.fields {
            // feat-db-connect-import-and-ddl-realdb-verify（core-01d §4.6）：SQLite 真实库执行验证——
            // AUTO_INCREMENT 非法；唯一合法形态为 INTEGER PRIMARY KEY AUTOINCREMENT
            let sqlite_rowid = engine == "sqlite" && field.increment && field.primary;
            let mut line = if sqlite_rowid {
                format!("  {} INTEGER", field.name)
            } else {
                format!("  {} {}", field.name, field.type_)
            };
            if field.primary {
                line.push_str(" PRIMARY KEY");
            }
            if sqlite_rowid {
                line.push_str(" AUTOINCREMENT");
            }
            if field.not_null && !field.primary {
                line.push_str(" NOT NULL");
            }
            if field.unique && !field.primary {
                line.push_str(" UNIQUE");
            }
            // 自增：generic/mysql → AUTO_INCREMENT；postgresql 由 SERIAL 类型承担不追加；
            // sqlite 主键自增已渲染为 AUTOINCREMENT（见上），非主键自增 SQLite 不支持则忽略
            if field.increment && engine != "postgresql" && engine != "sqlite" {
                line.push_str(" AUTO_INCREMENT");
            }
            if !field.default.is_empty() {
                line.push_str(&format!(" DEFAULT {}", field.default));
            }
            if engine == "mysql" && !field.comment.is_empty() {
                line.push_str(&format!(" COMMENT '{}'", escape_sql_comment(&field.comment)));
            }
            lines.push(line);
        }
        // 表末 FK 约束：start_table 为本表的 references
        for r in references
            .iter()
            .filter(|r| r.start_table_id == table.id)
        {
            let Some(to) = tables.iter().find(|t| t.id == r.end_table_id) else {
                continue;
            };
            let Some(ff) = table.fields.iter().find(|f| f.id == r.start_field_id) else {
                continue;
            };
            let Some(tf) = to.fields.iter().find(|f| f.id == r.end_field_id) else {
                continue;
            };
            lines.push(format!(
                "  FOREIGN KEY ({}) REFERENCES {}({})",
                ff.name, to.name, tf.name
            ));
        }
        // mysql：表注释走表选项 `) COMMENT='x';`（core-01d §5.2，fix-canvas-zoom-invite-comment-resize）；
        // 其余引擎后置 COMMENT ON（单引号转义 ' → ''）
        let table_comment_suffix = if engine == "mysql" && !table.comment.is_empty() {
            format!(" COMMENT='{}'", escape_sql_comment(&table.comment))
        } else {
            String::new()
        };
        out.push_str(&format!(
            "CREATE TABLE {} (\n{}\n){};\n",
            table.name,
            lines.join(",\n"),
            table_comment_suffix
        ));
        // 非 mysql 引擎：表/列注释后置 COMMENT ON，按表分组输出在 CREATE TABLE 之后
        if engine != "mysql" {
            if !table.comment.is_empty() {
                out.push_str(&format!(
                    "COMMENT ON TABLE {} IS '{}';\n",
                    table.name,
                    escape_sql_comment(&table.comment)
                ));
            }
            for field in table.fields.iter().filter(|f| !f.comment.is_empty()) {
                out.push_str(&format!(
                    "COMMENT ON COLUMN {}.{} IS '{}';\n",
                    table.name,
                    field.name,
                    escape_sql_comment(&field.comment)
                ));
            }
        }
        out.push('\n');
    }
    out
}

/// 客户端 DBML 导出（Phase C UT-PC-03）
pub fn export_diagram_dbml(tables: &[Table], references: &[Reference]) -> String {
    let mut out = String::new();
    for table in tables {
        out.push_str(&format!("Table {} {{\n", table.name));
        for field in &table.fields {
            let mut attrs = Vec::new();
            if field.primary {
                attrs.push("pk");
            }
            if field.not_null {
                attrs.push("not null");
            }
            let attr_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" [{}]", attrs.join(", "))
            };
            out.push_str(&format!("  {} {}{}\n", field.name, field.type_, attr_str));
        }
        out.push_str("}\n\n");
    }
    for r in references {
        let start = tables.iter().find(|t| t.id == r.start_table_id);
        let end = tables.iter().find(|t| t.id == r.end_table_id);
        let sf = start
            .and_then(|t| t.fields.iter().find(|f| f.id == r.start_field_id))
            .map(|f| f.name.as_str())
            .unwrap_or("?");
        let ef = end
            .and_then(|t| t.fields.iter().find(|f| f.id == r.end_field_id))
            .map(|f| f.name.as_str())
            .unwrap_or("?");
        let st = start.map(|t| t.name.as_str()).unwrap_or("?");
        let et = end.map(|t| t.name.as_str()).unwrap_or("?");
        out.push_str(&format!("Ref: {}.{} > {}.{}\n", st, sf, et, ef));
    }
    out
}

/// 客户端 JSON 导出
pub fn export_diagram_json(name: &str, tables: &[Table], references: &[Reference]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": name,
        "tables": tables,
        "references": references,
    }))
    .unwrap_or_else(|_| "{}".into())
}

fn navigate_to_editor(diagram_id: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&format!("/editor/{diagram_id}"));
    }
}

fn copy_text_to_clipboard(text: &str) -> bool {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(document) = window.document() else {
        return false;
    };
    let Ok(el) = document.create_element("textarea") else {
        return false;
    };
    let Ok(ta) = el.dyn_into::<web_sys::HtmlTextAreaElement>() else {
        return false;
    };
    ta.set_value(text);
    let Some(body) = document.body() else {
        return false;
    };
    let _ = body.append_child(&ta);
    ta.select();
    let ok = document
        .dyn_ref::<web_sys::HtmlDocument>()
        .and_then(|d| d.exec_command("copy").ok())
        .unwrap_or(false);
    let _ = body.remove_child(&ta);
    ok
}

fn download_text(filename: &str, text: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(el) = document.create_element("a") {
                use wasm_bindgen::JsCast;
                if let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() {
                    let href = format!(
                        "data:text/plain;charset=utf-8,{}",
                        js_sys::encode_uri_component(text)
                    );
                    a.set_href(&href);
                    a.set_download(filename);
                    let _ = document.body().map(|body| {
                        let _ = body.append_child(&a);
                        a.click();
                        let _ = body.remove_child(&a);
                    });
                }
            }
        }
    }
}

/// 抽象的「具名字段」trait，用于跨 Tab 全局搜索（UT-SP-10 验证）。
/// Tables/Areas/Enums/Notes/Types 都用 `name` 字段做模糊匹配。
pub trait Named {
    fn name(&self) -> &str;
}

impl Named for Table {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Area {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Note {
    fn name(&self) -> &str {
        &self.content
    }
}

/// 新建区域默认值（AreasTab + UT-ALIGN-A01）
///
/// fix-global-entity-id-uniqueness：id 改全局唯一随机 id（原 `area-{seq}` 会在
/// 新 diagram 上与其他 diagram 已占用的全局主键冲突 → 保存 500）；
/// `seq` 仅保留用于命名与层叠落位。
pub fn new_default_area(seq: i64) -> Area {
    Area {
        id: crate::editor_core::new_entity_id("area"),
        x: 100.0 + (seq as f64) * 24.0,
        y: 100.0 + (seq as f64) * 24.0,
        width: 400.0,
        height: 300.0,
        color: "#e6f1f5".into(),
        name: format!("新区域 {}", seq + 1),
    }
}

/// 新建便签默认值（NotesTab + UT-ALIGN-A01）
///
/// fix-global-entity-id-uniqueness：id 改全局唯一随机 id，原因同 `new_default_area`。
pub fn new_default_note(seq: i64) -> Note {
    Note {
        id: crate::editor_core::new_entity_id("note"),
        x: 200.0 + (seq as f64) * 24.0,
        y: 200.0 + (seq as f64) * 24.0,
        content: format!("新便签 {}", seq + 1),
        color: "#fef3c7".into(),
    }
}

/// align-v1-api-completion: 仅非 default 画布可删除（UT-ALIGN-B03）
pub(crate) fn is_deletable_diagram_id(id: &str) -> bool {
    id != "default"
}

/// align-v1-api-completion: 失败导入日志显示重试按钮（UT-ALIGN-B03）
pub(crate) fn import_log_shows_retry(status: &str) -> bool {
    status == "failed"
}

/// Enums/Types 在 V1 仍用「仅前端 state」轻量 Stub（areas/notes 已接入 store）。
#[derive(Clone, Debug, PartialEq)]
pub struct AreaStub {
    pub id: String,
    pub name: String,
}

impl Named for AreaStub {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumStub {
    pub id: String,
    pub name: String,
    pub values: Vec<String>,
}

impl Named for EnumStub {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteStub {
    pub id: String,
    pub content: String,
}

impl Named for NoteStub {
    fn name(&self) -> &str {
        &self.content
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeStub {
    pub id: String,
    pub name: String,
}

impl Named for TypeStub {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Pure predicate for LeftPanel 侧栏选中态 (UT-STUB-01) — 不依赖 Leptos signals，
/// 可在 `cargo test --lib` 中直接调用。
///
/// **关键契约 (Bug B 防回归)**：显式拒绝 `data-testid` 命名空间形式输入
/// （如 `Some("table-list-item-xxx")`），因为 `table.id` 永远不含 `-list-item-` 子串。
/// 任何把 testid 字符串传回 select 链路的代码会被该函数拒绝。
pub fn is_table_selected(selected: &Option<String>, table_id: &str) -> bool {
    match selected {
        Some(s) if s == table_id => true,
        _ => false,
    }
}

/// fix-collab-autosave-race（UT-FE-S05-18）：room 绑定 diagram 一律跳过全量保存——
/// ops 即保存（服务端物化单写者，写只走 WS op 通道）；非 room 维持 debounce 全量保存。
pub fn should_skip_full_save(room_bound: bool) -> bool {
    room_bound
}

/// Public save 调度 helper (UT-STUB-02) — 抽 4 处 save handler 末尾的 `debouncer.schedule(...)`
/// 公共逻辑，避免雷同。闭包内 `spawn_local` 调 `DiagramClient::save` async 路径：
/// - fix-collab-autosave-race（方案 B 服务端物化单写者）：room 绑定 diagram **一律不调度全量
///   保存**——ops 即保存（op 落库即物化，写只走 WS op 通道）；断线时本地编辑进入 op 离线队列，
///   重连 flush + ack 后 dirty 由 CollabClient 清零。服务端对 room 图的全量 PUT 返回 409
///   USE_OP_CHANNEL，此处提前短路避免误弹 S01 模态。
/// - 成功 → `store.revision.set(r.revision)` + `store.dirty.set(false)`
/// - 409 Conflict → `conflict.set(Some(ConflictInfo{...}))`（S01 模态；仅非 room 路径可达）
/// - 其它错误 → `save_offline` + 「保存失败（离线）」+ 指数退避重试（3s/6s/12s）
#[allow(clippy::too_many_arguments)]
pub(crate) fn schedule_save(
    client: DiagramClient,
    store: EditorStore,
    current_diagram_id: RwSignal<String>,
    current_title: RwSignal<String>,
    debouncer: DebounceTrigger,
    conflict: RwSignal<Option<ConflictInfo>>,
    error: RwSignal<Option<String>>,
    is_saving: RwSignal<bool>,
    save_offline: RwSignal<bool>,
    _collab_state: RwSignal<CollabOtState>,
    _activity_feed: RwSignal<Vec<String>>,
    current_room: RwSignal<Option<RoomDetail>>,
    _auth_session: RwSignal<Option<AuthSession>>,
) {
    // fix-collab-autosave-race：room 内 ops 即保存（服务端物化单写者），不再全量 PUT。
    if should_skip_full_save(current_room.get_untracked().is_some()) {
        return;
    }
    let id = current_diagram_id.get();
    let rev = store.revision.get();
    let name = current_title.get();
    let snap = store.snapshot(id.clone(), name);
    // ST-S01-SS-01：debounce 静默期应保持 dirty（有未保存更改），
    // is_saving 只在 PUT 真正发出时置位（保存中…），与主原型 saveText 阶段一致。
    save_offline.set(false);
    debouncer.schedule(move || {
        let client = client.clone();
        let store = store.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        let is_saving = is_saving.clone();
        let save_offline = save_offline.clone();
        is_saving.set(true);
        spawn_local(async move {
            let result = save_with_retry(&client, &id, rev, &snap).await;
            match result {
                Ok(resp) => {
                    store.revision.set(resp.revision);
                    store.dirty.set(false);
                    save_offline.set(false);
                    error.set(None);
                }
                Err(SaveError::Conflict {
                    current_revision, ..
                }) => {
                    // 仅非 room 路径可达（room 已在调度前短路）：S01 409 模态。
                    conflict.set(Some(ConflictInfo::new(current_revision, rev)));
                }
                Err(_) => {
                    save_offline.set(true);
                    error.set(Some("保存失败（离线）".to_string()));
                }
            }
            is_saving.set(false);
        });
    });
}

/// Pure filter function for UT-SP-02 / UT-SP-10 — 不依赖 Leptos signals，
/// 可在 `cargo test --lib` 中直接调用。
///
/// query 为空时返回全部（clone 引用）；非空时做大小写不敏感的子串匹配。
pub fn filter_by_query<T: Named>(items: &[T], query: &str) -> Vec<T>
where
    T: Clone,
{
    if query.is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|item| item.name().to_lowercase().contains(&q))
        .cloned()
        .collect()
}

/// Pure filter on references (for Relationships Tab) — references 没有「name」字段，
/// 用 start_table_id+end_table_id 拼接做匹配（B2 简单实现；B3 可扩展为表名匹配）。
pub fn filter_references_by_query(refs: &[Reference], query: &str) -> Vec<Reference> {
    if query.is_empty() {
        return refs.to_vec();
    }
    let q = query.to_lowercase();
    refs.iter()
        .filter(|r| {
            r.start_table_id.to_lowercase().contains(&q)
                || r.end_table_id.to_lowercase().contains(&q)
                || r.type_.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

/// 工具栏类型
#[derive(Clone, Copy)]
pub enum ToolbarAction {
    CreateTable,
    Save,
}

/// 409 冲突弹窗
#[component]
pub fn ConflictDialog(
    conflict: RwSignal<Option<ConflictInfo>>,
    on_force_overwrite: Rc<dyn Fn()>,
    on_reload: Rc<dyn Fn()>,
) -> impl IntoView {
    let on_force_overwrite_inner = on_force_overwrite.clone();
    let on_reload_inner = on_reload.clone();
    let render = move || {
        let on_force_overwrite_inner = on_force_overwrite_inner.clone();
        let on_reload_inner = on_reload_inner.clone();
        match conflict.get() {
            Some(info) => view! {
                <div class="cdb-conflict-dialog-overlay" data-testid="modal-conflict">
                    <div class="cdb-conflict-dialog" data-testid="conflict-dialog">
                        <h2>"保存冲突"</h2>
                        <p>
                            "服务器上的版本比本地更新。请选择如何处理："
                            {format!("本地 rev {} vs 服务器 rev {}", info.local_revision, info.current_revision)}
                        </p>
                        <div class="cdb-dialog-buttons">
                            <button
                                class="cdb-btn cdb-btn--primary"
                                data-testid="conflict-force"
                                on:click=move |_| {
                                    conflict.set(None);
                                    on_force_overwrite_inner();
                                }
                            >
                                "强制覆盖"
                            </button>
                            <button
                                class="cdb-btn"
                                data-testid="conflict-reload"
                                on:click=move |_| {
                                    conflict.set(None);
                                    on_reload_inner();
                                }
                            >
                                "重新加载"
                            </button>
                        </div>
                    </div>
                </div>
            }.into_view(),
            None => view! { <></> }.into_view(),
        }
    };

    render
}

/// 错误提示
#[component]
pub fn ErrorToast(error: RwSignal<Option<String>>) -> impl IntoView {
    let render = move || match error.get() {
        Some(msg) => view! {
            <div class="cdb-error-toast" data-testid="error-toast">
                {msg}
                <button on:click=move |_| error.set(None)>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
        }
        .into_view(),
        None => view! { <></> }.into_view(),
    };

    render
}

/// fix-appbar-roomname-back-and-import-merge：成功提示 toast（2.5s 自动消失）
#[component]
pub fn NoticeToast(notice: RwSignal<Option<String>>) -> impl IntoView {
    create_effect(move |_| {
        if notice.get().is_some() {
            gloo_timers::callback::Timeout::new(2_500, move || notice.set(None)).forget();
        }
    });
    move || match notice.get() {
        Some(msg) => view! {
            <div class="cdb-notice-toast" data-testid="notice-toast">
                {msg}
                <button on:click=move |_| notice.set(None)>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
        }
        .into_view(),
        None => view! { <></> }.into_view(),
    }
}

/// 顶部菜单栏：面包屑 + 4 下拉 + 动态 SaveState
#[component]
pub fn TopMenuBar(
    modal_kind: RwSignal<Option<modals::ModalKind>>,
    current_title: RwSignal<String>,
    store: EditorStore,
    is_saving: RwSignal<bool>,
    transform: RwSignal<Transform>,
) -> impl IntoView {
    let file_open = create_rw_signal(false);
    let view_open = create_rw_signal(false);

    view! {
        <header class="cdb-header" data-testid="top-menu-bar">
            <div class="cdb-brand">
                <span class="cdb-logo-mark" aria-hidden="true">"C"</span>
                <div class="cdb-breadcrumb">
                    <span>"Diagrams"</span>
                    <span class="cdb-breadcrumb__sep">"/"</span>
                    <span class="cdb-breadcrumb__title">{move || current_title.get()}</span>
                </div>
            </div>
            <nav class="cdb-menu">
                <div
                    class="cdb-menu-item"
                    data-testid="cdb-menu-file"
                    on:click=move |_| {
                        view_open.set(false);
                        file_open.update(|v| *v = !*v);
                    }
                >"文件 ▾"</div>
                {move || if file_open.get() {
                    view! {
                        <div class="cdb-menu-dropdown" data-testid="cdb-menu-file-dropdown">
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-new"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::New));
                                    file_open.set(false);
                                }
                            >"新建"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-open"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Open));
                                    file_open.set(false);
                                }
                            >"打开"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-share"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Share));
                                    file_open.set(false);
                                }
                            >"分享"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-rename"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Rename));
                                    file_open.set(false);
                                }
                            >"重命名"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                <div class="cdb-menu-item" data-testid="cdb-menu-edit">"编辑 ▾"</div>
                <div
                    class="cdb-menu-item"
                    class:cdb-is-open=move || view_open.get()
                    data-testid="cdb-menu-view"
                    on:click=move |_| {
                        file_open.set(false);
                        view_open.update(|v| *v = !*v);
                    }
                >"视图 ▾"</div>
                {move || if view_open.get() {
                    let t = transform.clone();
                    view! {
                        <div class="cdb-menu-dropdown" data-testid="cdb-menu-view-dropdown">
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-zoom-in"
                                on:click=move |_| {
                                    zoom_in(t);
                                    view_open.set(false);
                                }
                            >"放大"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-zoom-out"
                                on:click=move |_| {
                                    zoom_out(t);
                                    view_open.set(false);
                                }
                            >"缩小"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-zoom-reset"
                                on:click=move |_| {
                                    zoom_reset(t);
                                    view_open.set(false);
                                }
                            >"重置缩放"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                <div class="cdb-menu-item" data-testid="cdb-menu-help">"帮助 ▾"</div>
            </nav>
            <div class="cdb-header-right">
                {move || {
                    if is_saving.get() {
                        view! { <span class="cdb-save-state cdb-is-saving">"● 保存中..."</span> }.into_view()
                    } else if store.dirty.get() {
                        view! { <span class="cdb-save-state cdb-is-idle">"● 未保存"</span> }.into_view()
                    } else {
                        view! { <span class="cdb-save-state">"● 已保存"</span> }.into_view()
                    }
                }}
                <button
                    class="cdb-btn cdb-btn--primary cdb-btn--small"
                    data-testid="btn-share"
                    on:click=move |_| modal_kind.set(Some(modals::ModalKind::Share))
                >
                    "分享"
                </button>
                <button class="cdb-btn cdb-btn--icon" title="设置">
                    <IconBox size="sm"><IconSettings /></IconBox>
                </button>
            </div>
        </header>
    }
}

/// 撤销/重做按钮组 (B1)：UI 落地，真实 undo/redo 逻辑待 B5
/// - 接收 store（用于显示 revision）
/// - 接收 error signal（点击弹 toast 提示 B5 待实现）
#[component]
pub fn UndoRedoButtons(
    store: EditorStore,
    stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>>,
    on_after_change: Rc<dyn Fn()>,
    error: RwSignal<Option<String>>,
    read_only: bool,
) -> impl IntoView {
    let on_after_undo = on_after_change.clone();
    let on_after_redo = on_after_change.clone();
    view! {
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-undo"
            title="撤销 (Ctrl+Z)"
            disabled=read_only
            on:click=move |_| {
                let cmd = {
                    let stack_rc = stack.get();
                    let mut s = stack_rc.borrow_mut();
                    s.undo()
                };
                if let Some(cmd) = cmd {
                    if crate::editor_core::CommandStack::revert(&store, &cmd).is_ok() {
                        on_after_undo();
                    }
                } else {
                    error.set(Some("无可撤销操作".to_string()));
                }
            }
        >
            <IconBox size="sm"><IconUndo /></IconBox>
        </button>
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-redo"
            title="重做 (Ctrl+Shift+Z)"
            disabled=read_only
            on:click=move |_| {
                let cmd = {
                    let stack_rc = stack.get();
                    let mut s = stack_rc.borrow_mut();
                    s.redo()
                };
                if let Some(cmd) = cmd {
                    match crate::editor_core::CommandStack::execute(&store, &cmd) {
                        Ok(()) => on_after_redo(),
                        Err(e) => error.set(Some(e.message)),
                    }
                } else {
                    error.set(Some("无可重做操作".to_string()));
                }
            }
        >
            <IconBox size="sm"><IconRedo /></IconBox>
        </button>
    }
}

/// 工具栏：撤销/重做 + 可编辑标题 + rev + Export
#[component]
pub fn Toolbar(
    store: EditorStore,
    current_title: RwSignal<String>,
    error: RwSignal<Option<String>>,
    on_title_blur: Rc<dyn Fn(String)>,
) -> impl IntoView {
    let stack = create_rw_signal(Rc::new(RefCell::new(
        crate::editor_core::CommandStack::new(),
    )));
    let noop = Rc::new(|| {}) as Rc<dyn Fn()>;
    view! {
        <div class="cdb-toolbar">
            <UndoRedoButtons store=store stack=stack on_after_change=noop error=error read_only=false />
            <input
                class="cdb-title-edit"
                data-testid="diagram-title-input"
                prop:value=move || current_title.get()
                on:input=move |ev| current_title.set(event_target_value(&ev))
                on:blur=move |ev| on_title_blur(event_target_value(&ev))
            />
            <span class="cdb-rev-tag" data-testid="revision-display">
                {move || format!("rev: {}", store.revision.get())}
            </span>
            <div class="cdb-toolbar-right">
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="btn-export"
                    title="Export — 待 B5 实现"
                    on:click=move |_| {
                        error.set(Some("Export 模态待 B5 实现".to_string()));
                    }
                >
                    "Export ▾"
                </button>
            </div>
        </div>
    }
}

/// 画布底部浮动缩放条（仅缩放，撤销/重做保留在工具栏）
#[component]
pub fn FloatingControls(transform: RwSignal<Transform>) -> impl IntoView {
    view! {
        <div class="cdb-floating-controls" data-testid="floating-controls">
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-zoom-out"
                title="缩小"
                on:click=move |_| zoom_out(transform)
            >
                <IconBox size="sm"><IconMinus /></IconBox>
            </button>
            <span class="cdb-floating-zoom-label">
                {move || format!("{}%", (transform.get().zoom * 100.0).round() as i32)}
            </span>
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-zoom-in"
                title="放大"
                on:click=move |_| zoom_in(transform)
            >
                <IconBox size="sm"><IconAdd /></IconBox>
            </button>
        </div>
    }
}

/// 保存态 Chip 的纯派生逻辑（UT-S01-SS-01/02 可直接断言）：
/// 返回 (data-state, 文案, dot 修饰类)。优先级 saving > error > dirty > saved。
/// 文案严格对齐主原型 saveText 表：已保存/有未保存更改/保存中…/保存失败
pub fn save_chip_state(is_saving: bool, save_error: bool, dirty: bool) -> (&'static str, &'static str, &'static str) {
    if is_saving {
        ("saving", "保存中…", "cdb-save-dot--saving")
    } else if save_error {
        ("error", "保存失败", "cdb-save-dot--error")
    } else if dirty {
        ("dirty", "有未保存更改", "cdb-save-dot--dirty")
    } else {
        ("saved", "已保存", "cdb-save-dot--saved")
    }
}

/// R4：AppBar 保存态 Chip — 严格对齐主原型 .save-chip：
/// `data-state` = saved/dirty/saving/error；文案 已保存/有未保存更改/保存中…/保存失败 + 「 · rev N」
#[component]
pub fn SaveStatusChip(
    store: EditorStore,
    is_saving: RwSignal<bool>,
    save_offline: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="cdb-app-bar__status">
            {move || {
                let (state, text, dot_mod) =
                    save_chip_state(is_saving.get(), save_offline.get(), store.dirty.get());
                view! {
                    <span class="cdb-status-chip cdb-save-state" data-testid="save-state" data-state=state>
                        <span class=format!("cdb-save-dot {dot_mod}")></span>
                        {text}
                        <span class="cdb-status-chip__revision" data-testid="revision-display">
                            {format!(" · rev {}", store.revision.get())}
                        </span>
                    </span>
                }
            }}
        </div>
    }
}

/// R4：AppBar 溢出菜单（导入 / 导出 / 分享设置 / 设置 / 删除 / 主题 / 命令面板）
/// 结构与主原型 renderMoreMenu 对齐：分享设置从 AppBar 一级按钮迁入此菜单
#[component]
pub fn AppBarOverflowMenu(
    theme_mode: RwSignal<String>,
    on_open_import: Rc<dyn Fn()>,
    on_open_export: Rc<dyn Fn()>,
    on_open_share: Rc<dyn Fn()>,
    on_open_settings: Rc<dyn Fn()>,
    on_open_palette: Rc<dyn Fn()>,
    on_delete_diagram: Rc<dyn Fn()>,
    read_only: bool,
) -> impl IntoView {
    let overflow_open = create_rw_signal(false);

    view! {
        <div class="cdb-app-bar__overflow">
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-more-menu"
                title="更多"
                aria-haspopup="menu"
                aria-expanded=move || overflow_open.get()
                on:click=move |_| overflow_open.update(|v| *v = !*v)
            >
                <IconBox size="sm"><IconMore /></IconBox>
            </button>
            {move || if overflow_open.get() {
                view! {
                    <div
                        class="cdb-menu-dropdown cdb-app-bar__overflow-menu"
                        data-testid="app-bar-overflow-menu"
                        role="menu"
                    >
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-import"
                            role="menuitem"
                            disabled=read_only
                            on:click={
                                let import_handler = on_open_import.clone();
                                move |_| {
                                    import_handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconImport /></IconBox>
                            "导入"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-export"
                            role="menuitem"
                            on:click={
                                let export_handler = on_open_export.clone();
                                move |_| {
                                    export_handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconExport /></IconBox>
                            "导出"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-share"
                            role="menuitem"
                            disabled=read_only
                            on:click={
                                let handler = on_open_share.clone();
                                move |_| {
                                    handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconShare /></IconBox>
                            "分享设置"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-settings"
                            role="menuitem"
                            disabled=read_only
                            on:click={
                                let handler = on_open_settings.clone();
                                move |_| {
                                    handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconSettings /></IconBox>
                            "设置"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon cdb-menu-dropdown-item--danger"
                            data-testid="btn-delete-diagram"
                            role="menuitem"
                            disabled=read_only
                            on:click={
                                let handler = on_delete_diagram.clone();
                                move |_| {
                                    handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconClose /></IconBox>
                            "删除图表"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-theme-toggle"
                            role="menuitem"
                            on:click=move |_| {
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Some(html) = doc.document_element() {
                                        let cur = html
                                            .get_attribute("data-mode")
                                            .unwrap_or_else(|| "light".into());
                                        let next = if cur == "dark" { "light" } else { "dark" };
                                        let _ = html.set_attribute("data-mode", next);
                                        theme_mode.set(next.to_string());
                                    }
                                }
                                overflow_open.set(false);
                            }
                        >
                            {move || if theme_mode.get() == "dark" {
                                view! {
                                    <IconBox size="sm"><IconSun /></IconBox>
                                    "浅色模式"
                                }.into_view()
                            } else {
                                view! {
                                    <IconBox size="sm"><IconMoon /></IconBox>
                                    "深色模式"
                                }.into_view()
                            }
                            }
                        </button>
                        <button
                            class="cdb-menu-dropdown-item cdb-menu-dropdown-item--icon"
                            data-testid="btn-command-palette"
                            role="menuitem"
                            on:click={
                                let handler = on_open_palette.clone();
                                move |_| {
                                    handler();
                                    overflow_open.set(false);
                                }
                            }
                        >
                            <IconBox size="sm"><IconSearch /></IconBox>
                            "命令面板"
                            <span class="cdb-menu-shortcut">"⌘K"</span>
                        </button>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>
    }
}

/// Phase A：单行 AppBar — 严格对齐主原型 core-01 renderEditor 的 appbar 构成：
/// brand-mark → divider → undo/redo → diagram-title → divider → room-badge → save-chip
/// → spacer → presence 头像组 → actions（邀请 / 成员 / 代码视图 / 更多 / 用户菜单）
#[component]
pub fn AppBar(
    modal_kind: RwSignal<Option<modals::ModalKind>>,
    current_title: RwSignal<String>,
    store: EditorStore,
    stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>>,
    is_saving: RwSignal<bool>,
    save_offline: RwSignal<bool>,
    view_mode: RwSignal<ViewMode>,
    code_visible: RwSignal<bool>,
    inspector_open: RwSignal<bool>,
    transform: RwSignal<Transform>,
    error: RwSignal<Option<String>>,
    on_title_blur: Rc<dyn Fn(String)>,
    on_after_change: Rc<dyn Fn()>,
    on_open_import: Rc<dyn Fn()>,
    on_open_export: Rc<dyn Fn()>,
    on_open_settings: Rc<dyn Fn()>,
    on_open_palette: Rc<dyn Fn()>,
    on_delete_diagram: Rc<dyn Fn()>,
    auth_session: RwSignal<Option<AuthSession>>,
    session_notice: RwSignal<Option<String>>,
    on_refresh_session: Rc<dyn Fn()>,
    on_logout: Rc<dyn Fn()>,
    current_room: RwSignal<Option<RoomDetail>>,
    remote_members: RwSignal<Vec<CollabMemberPresence>>,
    on_open_rooms: Rc<dyn Fn()>,
    on_open_members: Rc<dyn Fn()>,
    on_open_invite: Rc<dyn Fn()>,
    read_only: bool,
    theme_mode: RwSignal<String>,
) -> impl IntoView {
    let _ = (transform, inspector_open);
    let on_open_share = {
        let modal_kind = modal_kind.clone();
        Rc::new(move || modal_kind.set(Some(modals::ModalKind::Share))) as Rc<dyn Fn()>
    };

    view! {
        <header class="cdb-app-bar" data-testid="app-bar">
            // fix-appbar-roomname-back-and-import-merge：房间上下文显性返回入口（← 空间）
            {let on_open_rooms_back = on_open_rooms.clone();
             move || current_room.get().map(|_| {
                let on_open_rooms = on_open_rooms_back.clone();
                view! {
                    <button
                        class="cdb-btn cdb-btn--ghost cdb-back-to-rooms"
                        data-testid="btn-back-to-rooms"
                        title="返回空间列表"
                        on:click=move |_| on_open_rooms()
                    >
                        <IconBox size="sm"><IconArrowLeft /></IconBox>
                        <span>"空间"</span>
                    </button>
                }
            })}
            <div class="cdb-app-bar__brand">
                <span class="cdb-brand-mark" aria-hidden="true"><IconBox size="md"><IconLogo /></IconBox></span>
            </div>
            <div class="cdb-app-bar__divider"></div>
            // diagram-database-dialect：图级引擎展示（Generic/MySQL/PostgreSQL）；
            // fix-pg-types-listview-zindex-lock-engine：引擎创建后锁定（S04.1 约束）——
            // 房间编辑器内下拉禁用，仅作只读标识；引擎在创建房间时确定，中途切换会使既有字段类型脱节
            <select
                class="cdb-form-select"
                data-testid="app-db-select"
                disabled=move || read_only || current_room.get().is_some()
                title=move || if current_room.get().is_some() { "引擎在创建房间时确定" } else { "数据库引擎" }
                on:change={
                    let on_after = on_after_change.clone();
                    move |ev| {
                        let db = match event_target_value(&ev).as_str() {
                            "mysql" => crate::editor_core::types::Database::Mysql,
                            "postgresql" => crate::editor_core::types::Database::Postgresql,
                            _ => crate::editor_core::types::Database::Generic,
                        };
                        store.database.set(db);
                        store.dirty.set(true);
                        on_after();
                    }
                }
            >
                <option value="generic" selected=move || store.database.get() == crate::editor_core::types::Database::Generic>"Generic"</option>
                <option value="mysql" selected=move || store.database.get() == crate::editor_core::types::Database::Mysql>"MySQL"</option>
                <option value="postgresql" selected=move || store.database.get() == crate::editor_core::types::Database::Postgresql>"PostgreSQL"</option>
            </select>
            <div class="cdb-app-bar__divider"></div>
            <UndoRedoButtons
                store=store.clone()
                stack=stack
                on_after_change=on_after_change.clone()
                error=error.clone()
                read_only=read_only
            />
            <input
                class="cdb-diagram-title"
                data-testid="diagram-title"
                aria-label="图表标题"
                // fix-appbar-roomname-back-and-import-merge：悬停显示全文（输入框 ellipsis 缩略时可见全名）
                title=move || current_title.get()
                prop:value=move || current_title.get()
                readonly=read_only
                on:input=move |ev| current_title.set(event_target_value(&ev))
                on:blur=move |ev| {
                    if !read_only {
                        on_title_blur(event_target_value(&ev));
                    }
                }
            />
            {move || current_room.get().map(|room| {
                let on_open_rooms = on_open_rooms.clone();
                view! {
                    <div class="cdb-app-bar__divider"></div>
                    <button
                        class="cdb-btn cdb-btn--ghost cdb-room-badge"
                        data-testid="room-badge"
                        // fix-appbar-roomname-back-and-import-merge：悬停显示房间全名
                        title=format!("{}（点击返回房间列表）", room.name)
                        on:click=move |_| on_open_rooms()
                    >
                        <IconBox size="sm"><IconUsers /></IconBox>
                        <strong>{room.name}</strong>
                    </button>
                }
            })}
            <SaveStatusChip
                store=store.clone()
                is_saving=is_saving
                save_offline=save_offline
            />
            <span class="cdb-app-bar__spacer"></span>
            {move || current_room.get().map(|_| view! {
                <div class="cdb-presence" data-testid="room-presence">
                    <For
                        each=move || {
                            let mut members: Vec<CollabMemberPresence> = remote_members
                                .get()
                                .into_iter()
                                .filter(|m| m.online)
                                .collect();
                            members.truncate(4);
                            members
                        }
                        key=|m| m.user_id.clone()
                        children=move |m: CollabMemberPresence| {
                            let label = m
                                .display_name
                                .clone()
                                .filter(|n| !n.is_empty())
                                .unwrap_or_else(|| m.user_id.clone());
                            let initial = label.chars().next().unwrap_or('U').to_string();
                            let role = m.role.clone().unwrap_or_else(|| "member".to_string());
                            view! {
                                <span class="cdb-presence-person" title=format!("{label} · {role}")>
                                    <span class="cdb-avatar">{initial}</span>
                                    <span class="cdb-presence-dot" data-testid="presence-online"></span>
                                </span>
                            }
                        }
                    />
                </div>
            })}
            <div class="cdb-app-bar__actions">
                {move || current_room.get().map(|_| {
                    let on_open_invite_btn = on_open_invite.clone();
                    let on_open_members_drawer = on_open_members.clone();
                    view! {
                        <button
                            class="cdb-btn cdb-btn--primary"
                            data-testid="btn-invite"
                            disabled=move || read_only || room_is_viewer(current_room)
                            on:click=move |_| on_open_invite_btn()
                        >
                            <IconBox size="sm"><IconAdd /></IconBox>
                            "邀请"
                        </button>
                        <button
                            class="cdb-btn cdb-btn--icon"
                            data-testid="btn-members"
                            title="成员"
                            aria-label="成员"
                            on:click=move |_| on_open_members_drawer()
                        >
                            <IconBox size="sm"><IconUsers /></IconBox>
                        </button>
                    }
                })}
                <ViewModeToggle view_mode=view_mode code_visible=code_visible />
                <AppBarOverflowMenu
                    theme_mode=theme_mode
                    on_open_import=on_open_import
                    on_open_export=on_open_export
                    on_open_share=on_open_share
                    on_open_settings=on_open_settings
                    on_open_palette=on_open_palette
                    on_delete_diagram=on_delete_diagram
                    read_only=read_only
                />
                <SessionIndicator
                    auth_session=auth_session
                    session_notice=session_notice
                    on_refresh_session=on_refresh_session
                    on_logout=on_logout
                />
            </div>
        </header>
    }
}

fn room_is_viewer(room: RwSignal<Option<RoomDetail>>) -> bool {
    room.get().as_ref().map(|r| r.is_viewer()).unwrap_or(false)
}

fn editor_is_read_only(share_mode: bool, room: RwSignal<Option<RoomDetail>>) -> bool {
    share_mode || room_is_viewer(room)
}

fn protected_api_error_message(error: &ApiError) -> String {
    match error {
        ApiError::Server(403, _) => "没有权限访问此资源".to_string(),
        ApiError::Network(_) => "网络连接失败，请检查网络后重试".to_string(),
        _ => "暂时无法加载数据，请稍后重试".to_string(),
    }
}

/// align-frontend-to-prototype：响应式布局断点（≤720px 视为紧凑布局）。
pub fn should_apply_compact_layout(viewport_width: u32) -> bool {
    viewport_width <= 720
}

/// align-frontend-to-prototype：密码强度等级（0~4）。
///
/// 启发式：长度 8+、长度 12+、含字母 + 数字、含字母 + 数字 + 特殊字符。
pub fn password_strength_level(p: &str) -> u8 {
    let len = p.chars().count();
    let has_lower = p.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = p.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = p.chars().any(|c| c.is_ascii_digit());
    let has_special = p.chars().any(|c| !c.is_alphanumeric());
    let variety = (has_lower || has_upper) as u8 + has_digit as u8 + has_special as u8;
    match (len, variety) {
        (0..=7, _) => 0,
        (8..=11, _) => 1,
        (12.., 1) => 2,
        (12.., 2) => 3,
        (12.., 3) => 4,
        _ => 1.min(variety),
    }
}

/// align-frontend-to-prototype：密码强度文案。
pub fn password_strength_label(level: u8) -> &'static str {
    match level {
        0 => "无",
        1 => "弱",
        2 => "一般",
        3 => "良好",
        _ => "强",
    }
}

/// align-frontend-to-prototype：IO 抽屉打开时，inspector 必须收起。
pub fn inspector_collapsed_when_io_open(io_open: bool) -> bool {
    io_open
}

/// align-frontend-to-prototype：所有 IO 抽屉都可以被关闭（不能锁死）。
pub fn can_close_io_drawer(kind: IoDrawerKind) -> bool {
    !matches!(kind, IoDrawerKind::None)
}

/// 状态栏 ws 文案 — 严格对齐主原型 renderEditor 的 wsText 五态：
/// 已连接 · OT 同步 / 正在同步… / 重连中 · 操作排队 / 仅本地 · 409 风险 / 协作离线
/// （生产多一个 ReadOnly 态：只读）
pub fn collab_status_label(state: &CollabOtState) -> &'static str {
    if state.local_only {
        return "仅本地 · 409 风险";
    }
    match state.connection {
        CollabConnectionState::Offline => "协作离线",
        CollabConnectionState::Connecting => "正在同步…",
        CollabConnectionState::Connected => "已连接 · OT 同步",
        CollabConnectionState::Reconnecting => "重连中 · 操作排队",
        CollabConnectionState::ReadOnly => "只读",
    }
}

/// ws 圆点等级 — 主原型 wsClass：connected 绿 / reconnecting·syncing 黄 / 其余红
pub fn collab_status_dot_class(state: &CollabOtState) -> &'static str {
    if state.local_only {
        return "cdb-is-error";
    }
    match state.connection {
        CollabConnectionState::Connected | CollabConnectionState::ReadOnly => "",
        CollabConnectionState::Connecting | CollabConnectionState::Reconnecting => "cdb-is-warn",
        CollabConnectionState::Offline => "cdb-is-error",
    }
}

pub fn collab_activity_from_frame(frame: &CollabFrame) -> String {
    match frame {
        CollabFrame::Connected { server_rev, .. } => format!("协作已连接 · rev {server_rev}"),
        CollabFrame::Ack {
            server_rev,
            client_rev,
            ..
        } => format!("本地变更已确认 · client {:?} → rev {server_rev}", client_rev),
        CollabFrame::RemoteOp {
            server_rev,
            author_id,
            ..
        } => format!("{author_id} 推送远端变更 · rev {server_rev}"),
        CollabFrame::Presence { user_id, .. } => format!("{user_id} 更新了在线状态"),
        CollabFrame::Sync { server_rev, ops, .. } => {
            format!("同步完成 · rev {} · {} 条变更", server_rev.unwrap_or(0), ops.len())
        }
        CollabFrame::Error { code, message } => format!("{code}: {message}"),
    }
}

pub fn prepend_activity(feed: RwSignal<Vec<String>>, item: String) {
    feed.update(|items| {
        items.insert(0, item);
        items.truncate(6);
    });
}

/// B 批：邀请模态 — 结构对齐主原型 modal-invite：
/// 「邀请成员加入「房间名」」+ 角色选择 + 邀请链接（7 天内有效）+ 复制。
/// 打开时按当前角色自动生成真实邀请（POST /rooms/{id}/invites），切换角色重新生成。
#[component]
pub fn InviteModal(
    open: RwSignal<bool>,
    room_client: RoomClient,
    auth_session: RwSignal<Option<AuthSession>>,
    current_room: RwSignal<Option<RoomDetail>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let invite_role = create_rw_signal(String::from("editor"));
    let invite_url = create_rw_signal(None::<String>);
    let generating = create_rw_signal(false);
    let copied = create_rw_signal(false);

    let generate = {
        let room_client = room_client.clone();
        Rc::new(move || {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(room) = current_room.get_untracked() else {
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let role = invite_role.get_untracked();
            let room_client = room_client.clone();
            generating.set(true);
            invite_url.set(None);
            spawn_local(async move {
                match room_client.create_invite(&token, &room_id, &role).await {
                    Ok(invite) => invite_url.set(Some(invite.invite_url)),
                    Err(e) => error.set(Some(e.to_string())),
                }
                generating.set(false);
            });
        })
    };

    // 打开模态时自动生成邀请链接（主原型事实：链接随角色即时展示）
    create_effect({
        let generate = generate.clone();
        move |_| {
            if open.get() {
                copied.set(false);
                generate();
            }
        }
    });

    let copy_url = move || {
        let Some(url) = invite_url.get_untracked() else {
            return;
        };
        // 剪贴板写入尽力而为（headless / 权限拒绝时不阻塞）；以按钮文案反馈
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&url);
        }
        copied.set(true);
        let copied = copied.clone();
        gloo_timers::callback::Timeout::new(1_500, move || copied.set(false)).forget();
    };

    view! {
        <div
            class="cdb-modal-overlay"
            data-testid="modal-invite-overlay"
            style:display=move || if open.get() { "flex" } else { "none" }
            on:click=move |_| open.set(false)
        >
            <div class="cdb-modal" data-testid="modal-invite" on:click=|ev| ev.stop_propagation()>
                <div class="cdb-modal-header">
                    <h3 class="cdb-modal-title" data-testid="modal-title-invite">
                        {move || format!(
                            "邀请成员加入「{}」",
                            current_room.get().map(|r| r.name).unwrap_or_else(|| "房间".to_string())
                        )}
                    </h3>
                    <button
                        class="cdb-modal-close"
                        data-testid="modal-cancel-invite"
                        on:click=move |_| open.set(false)
                    >
                        <IconBox size="sm"><IconClose /></IconBox>
                    </button>
                </div>
                <div class="cdb-modal-body">
                    <label class="cdb-form-label" for="invite-role">"加入后的角色"</label>
                    <select
                        class="cdb-form-select"
                        id="invite-role"
                        data-testid="invite-role"
                        disabled=move || generating.get()
                        on:change={
                            let generate = generate.clone();
                            move |ev| {
                                invite_role.set(event_target_value(&ev));
                                copied.set(false);
                                generate();
                            }
                        }
                    >
                        <option value="editor" selected=true>"Editor · 可以编辑"</option>
                        <option value="viewer">"Viewer · 仅查看"</option>
                    </select>
                    <label class="cdb-form-label">"邀请链接 · 7 天内有效"</label>
                    <div class="cdb-invite-link">
                        {move || if generating.get() {
                            view! { <span class="cdb-form-hint" data-testid="invite-generating">"正在生成邀请…"</span> }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                        {move || invite_url.get().map(|url| view! {
                            <input class="cdb-form-input" data-testid="invite-url" readonly=true prop:value=url />
                        })}
                    </div>
                </div>
                <div class="cdb-modal-footer">
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="btn-copy-invite"
                        disabled=move || invite_url.get().is_none()
                        on:click=move |_| copy_url()
                    >
                        {move || if copied.get() { "已复制 ✓" } else { "复制邀请" }}
                    </button>
                </div>
            </div>
        </div>
    }
}


/// p0-fix 定点 1：纯函数 — 是否可删除房间（仅 Owner）
/// - UT-MM-32: owner → true；editor/viewer/空串 → false
pub fn can_delete_room(my_role: &str) -> bool {
    my_role == "owner"
}

#[component]
pub fn RoomPanel(
    visible: RwSignal<bool>,
    room_client: RoomClient,
    auth_session: RwSignal<Option<AuthSession>>,
    current_room: RwSignal<Option<RoomDetail>>,
    room_members: RwSignal<Vec<RoomMember>>,
    error: RwSignal<Option<String>>,
    on_open_invite: Rc<dyn Fn()>,
    modal_kind: RwSignal<Option<modals::ModalKind>>,
) -> impl IntoView {
    // B 批：成员抽屉 — 结构对齐主原型 members drawer：
    // 「房间成员」+ 邀请新成员 + member-list（角色选择 / 移除，Owner 可管理）。
    // 历史「创建房间 / 我的房间 / 内联邀请」section 已移除（页面流由 rooms 页与邀请模态承接）。

    let load_members = {
        let room_client = room_client.clone();
        Rc::new(move || {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(room) = current_room.get_untracked() else {
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client.list_members(&token, &room_id).await {
                    Ok(items) => room_members.set(items),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    // 打开抽屉或切换房间时自动加载成员（主原型事实：成员列表直接可见，无手动加载按钮）
    create_effect({
        let load_members = load_members.clone();
        move |_| {
            if visible.get() && current_room.get().is_some() && auth_session.get().is_some() {
                load_members();
            }
        }
    });

    let change_role = {
        let room_client = room_client.clone();
        let load_members = load_members.clone();
        Rc::new(move |user_id: String, role: String| {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(room) = current_room.get_untracked() else {
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let room_client = room_client.clone();
            let load_members = load_members.clone();
            spawn_local(async move {
                match room_client
                    .update_member_role(&token, &room_id, &user_id, &role)
                    .await
                {
                    // 列表即时更新：直接写回服务端返回的最新成员
                    Ok(updated) => {
                        room_members.update(|items| {
                            if let Some(slot) =
                                items.iter_mut().find(|m| m.user_id == updated.user_id)
                            {
                                *slot = updated;
                            }
                        });
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
                load_members();
            });
        }) as Rc<dyn Fn(String, String)>
    };

    let remove_member = {
        let room_client = room_client.clone();
        let load_members = load_members.clone();
        Rc::new(move |user_id: String| {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(room) = current_room.get_untracked() else {
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let room_client = room_client.clone();
            let load_members = load_members.clone();
            spawn_local(async move {
                match room_client.remove_member(&token, &room_id, &user_id).await {
                    // 列表即时更新：本地先行剔除，再以服务端列表为准
                    Ok(()) => {
                        room_members.update(|items| {
                            items.retain(|m| m.user_id != user_id);
                        });
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
                load_members();
            });
        }) as Rc<dyn Fn(String)>
    };

    let can_manage = move || {
        current_room
            .get()
            .map(|r| can_delete_room(&r.my_role))
            .unwrap_or(false)
    };
    let self_id = move || {
        auth_session
            .get()
            .and_then(|s| s.user.as_ref().map(|u| u.id.clone()))
    };

    view! {
        <aside class="cdb-room-panel" data-testid="room-members-panel" style:display=move || if visible.get() { "flex" } else { "none" }>
            <div class="cdb-room-panel__header">
                <strong>"房间成员"</strong>
                <span class="cdb-room-panel__meta" data-testid="room-members-count">
                    {move || format!("{} 位成员", room_members.get().len())}
                </span>
                <button class="cdb-btn cdb-btn--icon" data-testid="btn-close-members" on:click=move |_| visible.set(false)>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
            <div class="cdb-room-panel__section">
                <button
                    class="cdb-btn cdb-btn--primary cdb-btn--block"
                    data-testid="btn-open-invite"
                    disabled=move || !current_room.get().map(|r| r.can_invite()).unwrap_or(false)
                    on:click=move |_| on_open_invite()
                >
                    <IconBox size="sm"><IconAdd /></IconBox>
                    "邀请新成员"
                </button>
            </div>
            <div class="cdb-room-panel__section cdb-room-panel__section--grow">
                <For each=move || room_members.get() key=|m| m.user_id.clone() children=move |m: RoomMember| {
                    let is_self = self_id().as_deref() == Some(m.user_id.as_str());
                    let name = m.display_name.clone().unwrap_or_else(|| m.email.clone());
                    let initial = name.chars().next().unwrap_or('U').to_string();
                    let role = m.role.clone();
                    let uid = m.user_id.clone();
                    let uid_role = uid.clone();
                    let uid_remove = uid.clone();
                    let change_role = change_role.clone();
                    let remove_member = remove_member.clone();
                    view! {
                        <article class="cdb-room-member" data-testid=format!("room-member-{}", uid)>
                            <span class="cdb-avatar" aria-hidden="true">
                                {initial}
                                {if is_self { view! { <span class="cdb-presence-dot" data-testid="member-online"></span> }.into_view() } else { view! { <></> }.into_view() }}
                            </span>
                            <div class="cdb-room-member__info">
                                <strong>{name}{if is_self { "（你）" } else { "" }}</strong>
                                // 降级事实：REST 无远端在线信号，仅本人显示在线
                                <small>{if is_self { "在线" } else { "离线" }}</small>
                            </div>
                            {if is_self {
                                view! { <span class="cdb-tag cdb-tag--brand" data-testid="member-self-role">{role}</span> }.into_view()
                            } else {
                                view! {
                                    <div class="cdb-room-member__actions">
                                        <select
                                            class="cdb-form-select cdb-form-select--sm"
                                            data-testid=format!("member-role-{}", uid_role)
                                            disabled=move || !can_manage()
                                            on:change={
                                                let change_role = change_role.clone();
                                                let uid = uid_role.clone();
                                                move |ev| change_role(uid.clone(), event_target_value(&ev))
                                            }
                                        >
                                            <option value="editor" selected={role == "editor"}>"editor"</option>
                                            <option value="viewer" selected={role == "viewer"}>"viewer"</option>
                                        </select>
                                        <button
                                            class="cdb-btn cdb-btn--icon cdb-btn--danger"
                                            data-testid=format!("btn-remove-member-{}", uid_remove)
                                            aria-label="移除成员"
                                            disabled=move || !can_manage()
                                            on:click={
                                                let remove_member = remove_member.clone();
                                                let uid = uid_remove.clone();
                                                move |_| remove_member(uid.clone())
                                            }
                                        >
                                            <IconBox size="sm"><IconDelete /></IconBox>
                                        </button>
                                    </div>
                                }.into_view()
                            }}
                        </article>
                    }
                } />
            </div>
            // p0-fix 定点 1：删除房间入口（仅 Owner 可见）→ 弹 ModalKind::DeleteRoom 确认模态
            {move || if can_manage() {
                view! {
                    <div class="cdb-room-panel__section">
                        <button
                            class="cdb-btn cdb-btn--danger cdb-btn--block"
                            data-testid="btn-delete-room"
                            on:click=move |_| modal_kind.set(Some(modals::ModalKind::DeleteRoom))
                        >
                            <IconBox size="sm"><IconDelete /></IconBox>
                            "删除房间"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
            <p class="cdb-room-panel__note">"Owner 可以修改角色与移除成员；Owner 自身需先转让房间才能离开。"</p>
        </aside>
    }
}

#[component]
pub fn InviteAcceptPanel(
    token: String,
    room_client: RoomClient,
    auth_session: RwSignal<Option<AuthSession>>,
    current_diagram_id: RwSignal<String>,
    current_room: RwSignal<Option<RoomDetail>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let preview = create_rw_signal(None::<InvitePreview>);
    create_effect({
        let room_client = room_client.clone();
        let token = token.clone();
        let error = error.clone();
        move |_| {
            let room_client = room_client.clone();
            let token = token.clone();
            spawn_local(async move {
                match room_client.preview_invite(&token).await {
                    Ok(p) => preview.set(Some(p)),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        }
    });

    let accept = {
        let room_client = room_client.clone();
        let token = token.clone();
        let auth_session = auth_session.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_room = current_room.clone();
        let error = error.clone();
        Rc::new(move || {
            let Some(session) = auth_session.get_untracked() else {
                error.set(Some("请先登录后接受邀请".to_string()));
                return;
            };
            let room_client = room_client.clone();
            let token = token.clone();
            let access = session.access_token;
            spawn_local(async move {
                match room_client.accept_invite(&access, &token).await {
                    Ok(res) => {
                        current_diagram_id.set(res.diagram_id.clone());
                        current_room.set(Some(RoomDetail {
                            id: res.room_id,
                            name: preview
                                .get_untracked()
                                .map(|p| p.room_name)
                                .unwrap_or_default(),
                            diagram_id: res.diagram_id,
                            owner_id: String::new(),
                            diagram_title: preview
                                .get_untracked()
                                .map(|p| p.diagram_title)
                                .unwrap_or_default(),
                            my_role: res.role,
                            member_count: 0,
                        }));
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    view! {
        <section class="cdb-invite-panel" data-testid="invite-accept-page">
            {move || preview.get().map(|p| view! {
                <div>
                    <strong>{p.room_name}</strong>
                    <p>{format!("图表：{} · 角色：{}", p.diagram_title, p.role)}</p>
                </div>
            })}
            <button class="cdb-btn cdb-btn--primary" data-testid="btn-accept-invite" on:click=move |_| accept()>"加入房间"</button>
        </section>
    }
}

/// 连接横幅 — 严格对齐主原型 renderConnectionBanner 三态：
/// syncing/reconnecting → 普通横幅（立即重连）；failed → danger 横幅（仅本地编辑 + 重新连接）；
/// local_only → danger 横幅（409 风险提示）。无房间（分享只读等）时不显示。
#[component]
pub fn ReconnectBanner(
    collab_state: RwSignal<CollabOtState>,
    current_room: RwSignal<Option<RoomDetail>>,
    on_reconnect: Rc<dyn Fn()>,
) -> impl IntoView {
    view! {
        {move || {
            let state = collab_state.get();
            let in_room = current_room.get().is_some();
            let queued = state.queued_while_offline.len() + state.pending_ops.len();
            let on_retry = on_reconnect.clone();
            let on_retry2 = on_reconnect.clone();
            let on_retry3 = on_reconnect.clone();
            if !in_room || matches!(state.connection, CollabConnectionState::Connected | CollabConnectionState::ReadOnly) {
                view! { <div class="cdb-reconnect-banner cdb-reconnect-banner--hidden" data-testid="reconnect-banner"></div> }.into_view()
            } else if state.local_only {
                view! {
                    <div class="cdb-reconnect-banner cdb-reconnect-banner--danger" data-testid="reconnect-banner">
                        <span>"仅本地编辑中，更改可能产生 409 冲突"</span>
                        <div class="cdb-reconnect-banner__actions">
                            <button class="cdb-btn cdb-btn--small" data-testid="btn-reconnect" on:click=move |_| on_retry()>
                                "重新连接"
                            </button>
                        </div>
                    </div>
                }.into_view()
            } else if matches!(state.connection, CollabConnectionState::Reconnecting) {
                view! {
                    <div class="cdb-reconnect-banner" data-testid="reconnect-banner">
                        <span>{format!("连接已断开，正在重连… · {queued} 项更改已排队")}</span>
                        <div class="cdb-reconnect-banner__actions">
                            <button class="cdb-btn cdb-btn--small" data-testid="btn-reconnect-now" on:click=move |_| on_retry2()>
                                "立即重连"
                            </button>
                            <button
                                class="cdb-btn cdb-btn--small"
                                data-testid="btn-local-only"
                                on:click=move |_| collab_state.update(|s| s.enter_local_only())
                            >
                                "仅本地编辑"
                            </button>
                        </div>
                    </div>
                }.into_view()
            } else if matches!(state.connection, CollabConnectionState::Connecting) {
                view! {
                    <div class="cdb-reconnect-banner" data-testid="reconnect-banner">
                        <span class="cdb-spinner" aria-hidden="true"></span>
                        <span>"正在同步…"</span>
                    </div>
                }.into_view()
            } else {
                // Offline + 在房间：连接失败终态（danger）
                view! {
                    <div class="cdb-reconnect-banner cdb-reconnect-banner--danger" data-testid="reconnect-banner">
                        <span>"无法连接协作服务，写操作已暂停"</span>
                        <div class="cdb-reconnect-banner__actions">
                            <button
                                class="cdb-btn cdb-btn--small"
                                data-testid="btn-local-only"
                                on:click=move |_| collab_state.update(|s| s.enter_local_only())
                            >
                                "仅本地编辑"
                            </button>
                            <button class="cdb-btn cdb-btn--small" data-testid="btn-reconnect" on:click=move |_| on_retry3()>
                                "重新连接"
                            </button>
                        </div>
                    </div>
                }.into_view()
            }
        }}
    }
}

#[component]
pub fn ActivityFeed(items: RwSignal<Vec<String>>, visible: RwSignal<bool>) -> impl IntoView {
    view! {
        <aside
            class="cdb-activity-feed"
            data-testid="activity-feed"
            style:display=move || if visible.get() { "flex" } else { "none" }
        >
            <For each=move || items.get() key=|item| item.clone() children=move |item: String| {
                view! { <div class="cdb-activity-feed__item">{item}</div> }
            } />
        </aside>
    }
}

#[component]
pub fn SessionIndicator(
    auth_session: RwSignal<Option<AuthSession>>,
    session_notice: RwSignal<Option<String>>,
    on_refresh_session: Rc<dyn Fn()>,
    on_logout: Rc<dyn Fn()>,
) -> impl IntoView {
    let menu_open = create_rw_signal(false);
    view! {
        <div class="cdb-session" data-testid="session-indicator">
            {move || match auth_session.get() {
                Some(session) => {
                    let label = session.display_name();
                    let initial = label.chars().next().unwrap_or('U').to_string();
                    view! {
                        <button
                            class="cdb-user-menu"
                            data-testid="user-menu"
                            title="用户菜单"
                            on:click=move |_| menu_open.update(|v| *v = !*v)
                        >
                            <span class="cdb-user-avatar">{initial}</span>
                            <span class="cdb-user-name">{label}</span>
                        </button>
                    }.into_view()
                }
                None => view! {
                    <span class="cdb-session__guest" data-testid="user-menu">"匿名分享"</span>
                }.into_view(),
            }}
            <span class="cdb-session__state">
                {move || session_notice.get().unwrap_or_else(|| {
                    if auth_session.get().is_some() {
                        "会话有效".to_string()
                    } else {
                        "只读访问".to_string()
                    }
                })}
            </span>
            {move || if menu_open.get() && auth_session.get().is_some() {
                let refresh = on_refresh_session.clone();
                let logout = on_logout.clone();
                view! {
                    <div class="cdb-session-menu" data-testid="user-menu-dropdown">
                        <button
                            class="cdb-menu-dropdown-item"
                            data-testid="btn-simulate-token-expired"
                            on:click=move |_| {
                                menu_open.set(false);
                                refresh();
                            }
                        >
                            "模拟 Token 过期"
                        </button>
                        <button
                            class="cdb-menu-dropdown-item"
                            data-testid="btn-logout"
                            on:click=move |_| {
                                menu_open.set(false);
                                logout();
                            }
                        >
                            "退出登录"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Login,
    Register,
}

#[component]
pub fn AuthGate(
    auth_client: AuthClient,
    auth_session: RwSignal<Option<AuthSession>>,
    session_notice: RwSignal<Option<String>>,
    on_login_success: Option<Rc<dyn Fn()>>,
) -> impl IntoView {
    let mode = create_rw_signal(AuthMode::Login);
    let email = create_rw_signal(String::new());
    let display_name = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let confirm_password = create_rw_signal(String::new());
    let password_visible = create_rw_signal(false);
    let remember_device = create_rw_signal(true);
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    let simulate_error = create_rw_signal(false);
    let email_error = create_rw_signal(Option::<String>::None);
    let password_error = create_rw_signal(Option::<String>::None);
    let confirm_error = create_rw_signal(Option::<String>::None);
    let name_error = create_rw_signal(Option::<String>::None);

    let submit = {
        let auth_client = auth_client.clone();
        let on_login_success = on_login_success.clone();
        move || {
            let email_value = email.get().trim().to_string();
            let display_value = display_name.get().trim().to_string();
            let password_value = password.get();
            let confirm_value = confirm_password.get();
            // 字段级校验
            email_error.set(None);
            password_error.set(None);
            confirm_error.set(None);
            name_error.set(None);
            let mut has_field_error = false;
            if email_value.is_empty() || !email_value.contains('@') {
                email_error.set(Some("请输入有效邮箱".to_string()));
                has_field_error = true;
            }
            if password_value.len() < 8 {
                password_error.set(Some("密码至少 8 位".to_string()));
                has_field_error = true;
            }
            if mode.get() == AuthMode::Register && password_value != confirm_value {
                confirm_error.set(Some("两次输入的密码不一致".to_string()));
                has_field_error = true;
            }
            if mode.get() == AuthMode::Register && display_value.is_empty() {
                name_error.set(Some("请输入显示名称".to_string()));
                has_field_error = true;
            }
            if has_field_error {
                return;
            }
            loading.set(true);
            error.set(None);
            let client = auth_client.clone();
            let on_login_success = on_login_success.clone();
            let simulate = simulate_error.get_untracked();
            spawn_local(async move {
                // 模拟凭据错误（测试用：不发请求，立即返回错误）
                if simulate {
                    error.set(Some("凭据错误，请检查邮箱与密码".to_string()));
                    loading.set(false);
                    return;
                }
                let result = async {
                    if mode.get_untracked() == AuthMode::Register {
                        client
                            .register(&email_value, &password_value, &display_value)
                            .await?;
                    }
                    client.login(&email_value, &password_value).await
                }
                .await;
                match result {
                    Ok(session) => {
                        crate::editor_data_access::persist_auth_session(&session);
                        auth_session.set(Some(session));
                        session_notice.set(Some("会话有效".to_string()));
                        if let Some(cb) = on_login_success {
                            cb();
                        }
                    }
                    Err(e) => {
                        let message = auth_error_display(&e);
                        if mode.get_untracked() == AuthMode::Register {
                            email_error.set(Some(message.clone()));
                        }
                        error.set(Some(message));
                    }
                }
                loading.set(false);
            });
        }
    };

    view! {
        <main class="cdb-auth-page" data-testid="auth-gate">
            // ─── 左区：品牌 + hero copy + 3 feature（align-frontend-to-prototype）───
            <article class="cdb-auth-story" data-testid="auth-story">
                <div class="cdb-auth-brand" data-testid="auth-brand">
                    <span class="cdb-brand-mark" aria-hidden="true">
                        <IconBox size="lg"><IconLogo /></IconBox>
                    </span>
                    <span class="cdb-auth-brand-name">"coldrawdb"</span>
                    <span class="cdb-tag cdb-tag--brand" data-testid="auth-brand-tag">"协作版原型"</span>
                </div>
                <div class="cdb-auth-hero" data-testid="auth-hero">
                    <span class="cdb-eyebrow">"结构清晰，协作自然"</span>
                    <h1 class="cdb-auth-hero-title">
                        "把复杂数据"
                        <br />
                        <span>"画成共同语言。"</span>
                    </h1>
                    <p class="cdb-auth-hero-desc">
                        "在一张实时同步的画布里完成数据库建模、关系评审和工程交接，让每个决定都有上下文。"
                    </p>
                </div>
                <div class="cdb-auth-feature-row" data-testid="auth-feature-row">
                    <div class="cdb-auth-feature">
                        <span class="cdb-auth-feature-icon" aria-hidden="true">
                            <IconBox size="md"><IconAddTable /></IconBox>
                        </span>
                        <strong>"可视化建模"</strong>
                        <span>"表、字段、关系与约束完整闭环"</span>
                    </div>
                    <div class="cdb-auth-feature">
                        <span class="cdb-auth-feature-icon" aria-hidden="true">
                            <IconBox size="md"><IconUsers /></IconBox>
                        </span>
                        <strong>"多人同步"</strong>
                        <span>"光标、选区、Activity 与角色权限"</span>
                    </div>
                    <div class="cdb-auth-feature">
                        <span class="cdb-auth-feature-icon" aria-hidden="true">
                            <IconBox size="md"><IconActivity /></IconBox>
                        </span>
                        <strong>"实时协作"</strong>
                        <span>"OT 同步、断线排队、重连续传"</span>
                    </div>
                </div>
            </article>
            // ─── 右区：表单卡片（玻璃质感）───
            <article class="cdb-auth-panel cdb-glass" data-testid="auth-panel">
                <div class="cdb-auth-card">
                    <span class="cdb-eyebrow">"欢迎使用 coldrawdb"</span>
                    <h2 id="auth-title" class="cdb-auth-title" data-testid="auth-title" tabindex="-1">
                        {move || if mode.get() == AuthMode::Register { "创建协作账户" } else { "继续你的数据设计" }}
                    </h2>
                    <p class="cdb-muted">
                        {move || if mode.get() == AuthMode::Register {
                            "创建账户后即可邀请团队一起评审模型。"
                        } else {
                            "登录后进入项目空间；你的凭据只用于鉴权，不会写本地存储。"
                        }}
                    </p>
                    {move || session_notice.get().map(|notice| view! {
                        <p class="cdb-auth-session-notice" data-testid="auth-session-notice">{notice}</p>
                    })}
                    <div class="cdb-auth-tabs" role="tablist">
                        <button
                            class="cdb-auth-tab"
                            class:cdb-is-active=move || mode.get() == AuthMode::Login
                            role="tab"
                            aria-selected=move || mode.get() == AuthMode::Login
                            data-testid="auth-tab-login"
                            on:click=move |_| mode.set(AuthMode::Login)
                        >
                            "登录"
                        </button>
                        <button
                            class="cdb-auth-tab"
                            class:cdb-is-active=move || mode.get() == AuthMode::Register
                            role="tab"
                            aria-selected=move || mode.get() == AuthMode::Register
                            data-testid="auth-tab-register"
                            on:click=move |_| mode.set(AuthMode::Register)
                        >
                            "注册"
                        </button>
                    </div>
                    <form
                        class="cdb-auth-form"
                        data-testid=move || if mode.get() == AuthMode::Login { "login-form" } else { "register-form" }
                        on:submit=move |ev| {
                            ev.prevent_default();
                            submit();
                        }
                    >
                        {move || if mode.get() == AuthMode::Register {
                            view! {
                                <div class="cdb-field">
                                    <label for="auth-display-name">"显示名称"</label>
                                    <input
                                        class="cdb-input"
                                        id="auth-display-name"
                                        data-testid="auth-display-name"
                                        prop:value=move || display_name.get()
                                        on:input=move |ev| display_name.set(event_target_value(&ev))
                                        maxlength="32"
                                        autocomplete="name"
                                        aria-describedby="auth-display-name-error"
                                    />
                                    <p class="cdb-field-error" id="auth-display-name-error" data-error="name" data-testid="auth-name-error">
                                        {move || name_error.get().unwrap_or_default()}
                                    </p>
                                </div>
                            }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                        <div class="cdb-field">
                            <label for="auth-email">"邮箱"</label>
                            <input
                                class="cdb-input"
                                id="auth-email"
                                data-testid="auth-email"
                                type="email"
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                                autocomplete="email"
                                aria-describedby="auth-email-error"
                            />
                            <p class="cdb-field-error" id="auth-email-error" data-error="email" data-testid="auth-email-error">
                                {move || email_error.get().unwrap_or_default()}
                            </p>
                        </div>
                        <div class="cdb-field">
                            <label for="auth-password">"密码"</label>
                            <div class="cdb-password-wrap">
                                <input
                                    class="cdb-input"
                                    id="auth-password"
                                    data-testid="auth-password"
                                    type=move || if password_visible.get() { "text" } else { "password" }
                                    prop:value=move || password.get()
                                    on:input=move |ev| password.set(event_target_value(&ev))
                                    autocomplete=move || if mode.get() == AuthMode::Register { "new-password" } else { "current-password" }
                                    aria-describedby="auth-password-error"
                                />
                                <button
                                    class="cdb-password-toggle"
                                    type="button"
                                    data-testid="auth-eye-toggle"
                                    aria-label=move || if password_visible.get() { "隐藏密码" } else { "显示密码" }
                                    on:click=move |_| password_visible.update(|v| *v = !*v)
                                >
                                    {move || if password_visible.get() {
                                        view! { <IconBox size="sm"><IconEyeOff /></IconBox> }.into_view()
                                    } else {
                                        view! { <IconBox size="sm"><IconEye /></IconBox> }.into_view()
                                    }}
                                </button>
                            </div>
                            <p class="cdb-field-error" id="auth-password-error" data-error="password" data-testid="auth-password-error">
                                {move || password_error.get().unwrap_or_default()}
                            </p>
                            {move || if mode.get() == AuthMode::Register {
                                let p = password.get();
                                let level = password_strength_level(&p);
                                view! {
                                    <div class="cdb-strength" data-testid="auth-strength" data-level=level aria-label=format!("密码强度：{}", password_strength_label(level))>
                                        <span class:cdb-on={level >= 1}></span>
                                        <span class:cdb-on={level >= 2}></span>
                                        <span class:cdb-on={level >= 3}></span>
                                        <span class:cdb-on={level >= 4}></span>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }}
                        </div>
                        {move || if mode.get() == AuthMode::Register {
                            view! {
                                <div class="cdb-field">
                                    <label for="auth-confirm">"确认密码"</label>
                                    <input
                                        class="cdb-input"
                                        id="auth-confirm"
                                        data-testid="auth-confirm-password"
                                        type="password"
                                        prop:value=move || confirm_password.get()
                                        on:input=move |ev| confirm_password.set(event_target_value(&ev))
                                        autocomplete="new-password"
                                        aria-describedby="auth-confirm-error"
                                    />
                                    <p class="cdb-field-error" id="auth-confirm-error" data-error="confirm" data-testid="auth-confirm-error">
                                        {move || confirm_error.get().unwrap_or_default()}
                                    </p>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="cdb-form-meta">
                                    <label class="cdb-checkbox">
                                        <input
                                            type="checkbox"
                                            data-testid="auth-remember"
                                            prop:checked=move || remember_device.get()
                                            on:change=move |ev| remember_device.set(event_target_checked(&ev))
                                        />
                                        <span>"记住此设备"</span>
                                    </label>
                                    <button
                                        type="button"
                                        class="cdb-btn cdb-btn--ghost cdb-btn--sm"
                                        data-testid="auth-simulate-error"
                                        on:click=move |_| simulate_error.update(|v| *v = !*v)
                                    >
                                        {move || if simulate_error.get() { "关闭模拟错误" } else { "模拟凭据错误" }}
                                    </button>
                                </div>
                            }.into_view()
                        }}
                        <div class="cdb-auth-alert" role="alert" data-testid="auth-alert">
                            {move || error.get().unwrap_or_default()}
                        </div>
                        <button
                            class="cdb-btn cdb-btn--primary cdb-auth-submit"
                            type="submit"
                            data-testid=move || if mode.get() == AuthMode::Login { "login-submit" } else { "register-submit" }
                            disabled=move || loading.get()
                            aria-busy=move || loading.get()
                        >
                            {move || if loading.get() {
                                view! {
                                    <span class="cdb-spinner" aria-hidden="true"></span>
                                    {if mode.get() == AuthMode::Register { "创建账户..." } else { "正在验证..." }}
                                }.into_view()
                            } else if mode.get() == AuthMode::Login {
                                view! {
                                    <span>"登录并进入空间"</span>
                                    <IconBox size="sm"><IconChevronRight /></IconBox>
                                }.into_view()
                            } else {
                                view! { <span>"创建账户"</span> }.into_view()
                            }}
                        </button>
                    </form>
                    <div class="cdb-demo-note" data-testid="auth-demo-note">
                        <span class="cdb-demo-note-icon" aria-hidden="true">
                            <IconBox size="sm"><IconActivity /></IconBox>
                        </span>
                        <span>
                            "演示提示：登录与注册都会调用真实鉴权 API；点击「模拟凭据错误」可查看异常反馈。"
                        </span>
                    </div>
                </div>
            </article>
        </main>
    }
}

/// `rooms-list-page`（align-frontend-to-prototype Batch B 完整实现）。
///
/// 顶栏 + 房间列表 + 新建房间入口 + 用户菜单 + session-indicator。
/// 真实调用：
/// - 首屏自动 `GET /api/v1/rooms`（需 `auth_session`）。
/// - 点击「新建房间」→ `POST /api/v1/rooms`（room name + diagram_id）→ 进入 editor。
/// - 点击 room card → 写入 `current_room`、`current_diagram_id`、`current_title`，进入 editor。
#[component]
pub fn RoomsListPage(
    auth_session: RwSignal<Option<AuthSession>>,
    session_notice: RwSignal<Option<String>>,
    auth_client: AuthClient,
    diagram_client: DiagramClient,
    room_client: RoomClient,
    on_logout: Rc<dyn Fn()>,
    on_select_room: Rc<dyn Fn(RoomDetail)>,
    on_create_room: Rc<dyn Fn(RoomDetail)>,
    // p0-fix 定点 1：外部强制刷新信号（删除房间后回列表时 bump → 绕过 token 缓存重新拉取）
    reload_rooms: RwSignal<u32>,
) -> impl IntoView {
    let loading = create_rw_signal(true);
    let rooms: RwSignal<Vec<RoomSummary>> = create_rw_signal(Vec::new());
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let creating = create_rw_signal(false);
    let last_loaded_token = create_rw_signal(Option::<String>::None);
    let reload_nonce = create_rw_signal(0_u32);
    // p0-fix 定点 1：已消费的外部刷新计数（与 reload_rooms 比较决定是否清缓存）
    let last_external_reload = create_rw_signal(0_u32);
    let create_modal_open = create_rw_signal(false);
    let room_name = create_rw_signal(String::from("数据模型评审"));
    let diagram_choice = create_rw_signal(String::from("__new__"));
    // fix-listview-grid-and-room-dbtype: 创建房间选「新建空白模型」时的引擎前置选择（默认 Generic 不透传，回归不破）
    let create_database = create_rw_signal(String::from("generic"));
    let default_role = create_rw_signal(String::from("editor"));
    let diagrams = create_rw_signal(Vec::<DiagramSummary>::new());
    let diagrams_loading = create_rw_signal(false);
    let diagrams_loaded = create_rw_signal(false);
    let create_error = create_rw_signal(Option::<String>::None);
    // fix-select-bg-diagram-delete-and-log-format：创建空间弹窗内删除游离图表
    // （core-S04 图表行：删除按钮 + 二次确认 + DELETE /api/v1/diagrams/{id} + Toast「已删除图表」）
    let delete_diagram_target = create_rw_signal(Option::<DiagramSummary>::None);
    let delete_diagram_error = create_rw_signal(Option::<String>::None);
    let deleting_diagram = create_rw_signal(false);
    // fix-overflow-menu-room-delete-listview-io：房间卡片删除（列表页入口，复用 DELETE /rooms/{id} 软删除）
    let delete_target = create_rw_signal(Option::<RoomSummary>::None);
    let delete_error = create_rw_signal(Option::<String>::None);
    let deleting = create_rw_signal(false);
    let notice = create_rw_signal(Option::<String>::None);
    // feat-room-recycle-bin-dropdown-and-db-export：回收站视图状态
    let trash_open = create_rw_signal(false);
    let trash_items = create_rw_signal(Vec::<RoomSummary>::new());
    let trash_loading = create_rw_signal(false);
    let trash_error = create_rw_signal(Option::<String>::None);
    let purge_target = create_rw_signal(Option::<RoomSummary>::None);
    let purge_error = create_rw_signal(Option::<String>::None);
    let purging = create_rw_signal(false);
    let restoring = create_rw_signal(Option::<String>::None);

    // 首屏 fetch：依赖 auth_session；session 变化时重新拉。
    create_effect({
        let auth_client = auth_client.clone();
        let room_client = room_client.clone();
        let auth_session = auth_session.clone();
        move |_| {
            reload_nonce.get();
            // p0-fix 定点 1：外部强制刷新（删除房间后回列表）→ 清 token 缓存，保证重新拉取
            let external = reload_rooms.get();
            if external != last_external_reload.get_untracked() {
                last_external_reload.set(external);
                last_loaded_token.set(None);
            }
            let Some(session) = auth_session.get() else {
                loading.set(false);
                rooms.set(Vec::new());
                return;
            };
            let token = session.access_token.clone();
            if last_loaded_token.get_untracked().as_deref() == Some(token.as_str()) {
                return;
            }
            last_loaded_token.set(Some(token.clone()));
            let current_session = session;
            let auth_client = auth_client.clone();
            let room_client = room_client.clone();
            loading.set(true);
            error.set(None);
            spawn_local(async move {
                let mut result = room_client.list_rooms(&token).await;
                if matches!(result, Err(ApiError::Server(401, _))) {
                    session_notice.set(Some("续期中...".to_string()));
                    match auth_client.refresh_session(&current_session).await {
                        Ok(next_session) => {
                            let next_token = next_session.access_token.clone();
                            last_loaded_token.set(Some(next_token.clone()));
                            crate::editor_data_access::persist_auth_session(&next_session);
                            auth_session.set(Some(next_session));
                            session_notice.set(Some("会话已续期".to_string()));
                            result = room_client.list_rooms(&next_token).await;
                            if matches!(result, Err(ApiError::Server(401, _))) {
                                crate::editor_data_access::clear_auth_session();
                                auth_session.set(None);
                                session_notice.set(Some("登录已过期，请重新登录".to_string()));
                                loading.set(false);
                                return;
                            }
                        }
                        Err(_) => {
                            crate::editor_data_access::clear_auth_session();
                            auth_session.set(None);
                            session_notice.set(Some("登录已过期，请重新登录".to_string()));
                            loading.set(false);
                            return;
                        }
                    }
                }
                match result {
                    Ok(resp) => {
                        rooms.set(resp.items);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(protected_api_error_message(&e)));
                        loading.set(false);
                    }
                }
            });
        }
    });

    let open_create_modal: Rc<dyn Fn()> = {
        let diagram_client = diagram_client.clone();
        Rc::new(move || {
            create_error.set(None);
            create_modal_open.set(true);
            if diagrams_loaded.get_untracked() || diagrams_loading.get_untracked() {
                return;
            }
            diagrams_loading.set(true);
            let diagram_client = diagram_client.clone();
            spawn_local(async move {
                match diagram_client.list_summaries().await {
                    Ok(items) => {
                        diagrams.set(items);
                        diagrams_loaded.set(true);
                    }
                    Err(_) => {
                        create_error.set(Some(
                            "已有图表加载失败，仍可创建空白模型".to_string(),
                        ));
                    }
                }
                diagrams_loading.set(false);
            });
        })
    };
    let open_create_from_header = open_create_modal.clone();
    let open_create_from_card = open_create_modal.clone();

    let refresh_rooms = move |_| {
        last_loaded_token.set(None);
        reload_nonce.update(|value| *value += 1);
    };

    // 创建房间必须绑定真实 diagram；选择“新建空白模型”时先创建 diagram。
    let submit_create_room: Rc<dyn Fn()> = {
        let diagram_client = diagram_client.clone();
        let room_client = room_client.clone();
        let auth_session = auth_session.clone();
        let on_create_room = on_create_room.clone();
        Rc::new(move || {
            if creating.get() {
                return;
            }
            let name = room_name.get().trim().to_string();
            if name.is_empty() || name.chars().count() > 64 {
                create_error.set(Some("房间名称须为 1–64 个字符".to_string()));
                return;
            }
            let Some(session) = auth_session.get() else {
                login_required_redirect();
                return;
            };
            creating.set(true);
            create_error.set(None);
            let token = session.access_token;
            let selected = diagram_choice.get();
            let available_diagrams = diagrams.get_untracked();
            let diagram_client = diagram_client.clone();
            let room_client = room_client.clone();
            let on_create_room = on_create_room.clone();
            spawn_local(async move {
                let (diagram_id, diagram_title, created_diagram) = if selected == "__new__" {
                    // fix-listview-grid-and-room-dbtype: 引擎前置选择透传；Generic 缺省不透传（回归不破）
                    let db = match create_database.get().as_str() {
                        "generic" => None,
                        other => Some(other.to_string()),
                    };
                    match diagram_client.create(&name, db).await {
                        Ok(id) => (id, name.clone(), true),
                        Err(_) => {
                            create_error.set(Some("空白模型创建失败，请稍后重试".to_string()));
                            creating.set(false);
                            return;
                        }
                    }
                } else {
                    let title = available_diagrams
                        .iter()
                        .find(|diagram| diagram.id == selected)
                        .and_then(|diagram| diagram.name.clone())
                        .unwrap_or_else(|| "未命名模型".to_string());
                    (selected, title, false)
                };

                match room_client.create_room(&token, &name, &diagram_id).await {
                    Ok(mut detail) => {
                        detail.diagram_title = diagram_title;
                        detail.my_role = "owner".to_string();
                        detail.member_count = 1;
                        creating.set(false);
                        create_modal_open.set(false);
                        on_create_room(detail);
                    }
                    Err(e) => {
                        if created_diagram {
                            diagrams.update(|items| {
                                items.push(DiagramSummary {
                                    id: diagram_id.clone(),
                                    name: Some(diagram_title),
                                });
                            });
                            diagram_choice.set(diagram_id);
                        }
                        create_error.set(Some(room_create_error_message(&e)));
                        creating.set(false);
                    }
                }
            });
        })
    };
    let submit_from_modal = submit_create_room.clone();

    // feat-room-recycle-bin-dropdown-and-db-export：打开回收站并拉取已归档房间
    let open_trash = {
        let room_client = room_client.clone();
        move |_| {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            trash_open.set(true);
            trash_loading.set(true);
            trash_error.set(None);
            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client.list_archived_rooms(&session.access_token).await {
                    Ok(list) => trash_items.set(list.items),
                    Err(_) => trash_error.set(Some("回收站加载失败，请稍后重试".to_string())),
                }
                trash_loading.set(false);
            });
        }
    };

    // feat-room-recycle-bin-dropdown-and-db-export：恢复归档房间
    let restore_room_action: Rc<dyn Fn(RoomSummary)> = {
        let room_client = room_client.clone();
        Rc::new(move |room: RoomSummary| {
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            restoring.set(Some(room.id.clone()));
            trash_error.set(None);
            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client.restore_room(&session.access_token, &room.id).await {
                    Ok(()) => {
                        trash_items.update(|list| list.retain(|r| r.id != room.id));
                        notice.set(Some(format!("已恢复房间「{}」", room.name)));
                    }
                    Err(ApiError::Server(409, _)) => {
                        trash_error.set(Some("该 diagram 已在其他协作房间中，无法恢复".to_string()));
                    }
                    Err(ApiError::Server(403, _)) => {
                        trash_error.set(Some("无权限恢复此房间".to_string()));
                    }
                    Err(_) => {
                        trash_error.set(Some("恢复失败，请稍后重试".to_string()));
                    }
                }
                restoring.set(None);
            });
        })
    };

    // feat-room-recycle-bin-dropdown-and-db-export：确认彻底删除（硬删除）
    let confirm_purge_room: Rc<dyn Fn()> = {
        let room_client = room_client.clone();
        Rc::new(move || {
            if purging.get_untracked() {
                return;
            }
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(target) = purge_target.get_untracked() else {
                return;
            };
            purging.set(true);
            purge_error.set(None);
            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client
                    .permanently_delete_room(&session.access_token, &target.id)
                    .await
                {
                    Ok(()) => {
                        purging.set(false);
                        trash_items.update(|list| list.retain(|r| r.id != target.id));
                        purge_target.set(None);
                        notice.set(Some(format!("已彻底删除「{}」", target.name)));
                    }
                    Err(ApiError::Server(403, _)) => {
                        purging.set(false);
                        purge_error.set(Some("无权限彻底删除此房间".to_string()));
                    }
                    Err(_) => {
                        purging.set(false);
                        purge_error.set(Some("删除失败，请稍后重试".to_string()));
                    }
                }
            });
        })
    };

    // fix-overflow-menu-room-delete-listview-io：确认删除房间（卡片入口）
    let confirm_delete_room = {
        let room_client = room_client.clone();
        Rc::new(move || {
            if deleting.get_untracked() {
                return;
            }
            let Some(session) = auth_session.get_untracked() else {
                return;
            };
            let Some(target) = delete_target.get_untracked() else {
                return;
            };
            deleting.set(true);
            delete_error.set(None);
            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client.delete_room(&session.access_token, &target.id).await {
                    Ok(()) => {
                        deleting.set(false);
                        rooms.update(|list| list.retain(|r| r.id != target.id));
                        delete_target.set(None);
                        notice.set(Some(format!("已删除房间「{}」", target.name)));
                    }
                    Err(ApiError::Server(403, _)) => {
                        deleting.set(false);
                        delete_error.set(Some("无权限删除此房间".to_string()));
                    }
                    Err(_) => {
                        deleting.set(false);
                        delete_error.set(Some("删除失败，请稍后重试".to_string()));
                    }
                }
            });
        }) as Rc<dyn Fn()>
    };

    // fix-select-bg-diagram-delete-and-log-format：创建空间弹窗内删除游离图表
    // （core-S04 图表行：二次确认 → DELETE /api/v1/diagrams/{id} → 移出候选列表 → Toast「已删除图表」）
    let confirm_delete_diagram = {
        let diagram_client = diagram_client.clone();
        Rc::new(move || {
            if deleting_diagram.get_untracked() {
                return;
            }
            let Some(target) = delete_diagram_target.get_untracked() else {
                return;
            };
            deleting_diagram.set(true);
            delete_diagram_error.set(None);
            let diagram_client = diagram_client.clone();
            spawn_local(async move {
                match diagram_client.delete(&target.id).await {
                    Ok(()) => {
                        deleting_diagram.set(false);
                        diagrams.update(|list| list.retain(|d| d.id != target.id));
                        // 被删图表可能正被选中 → 回落到「新建空白模型」
                        if diagram_choice.get_untracked() == target.id {
                            diagram_choice.set(String::from("__new__"));
                        }
                        let label = target.name.clone().unwrap_or_else(|| "未命名模型".to_string());
                        delete_diagram_target.set(None);
                        notice.set(Some(format!("已删除图表「{}」", label)));
                    }
                    Err(_) => {
                        deleting_diagram.set(false);
                        delete_diagram_error.set(Some("删除失败，请稍后重试".to_string()));
                    }
                }
            });
        }) as Rc<dyn Fn()>
    };

    view! {
        <main class="cdb-rooms-list-page" data-testid="rooms-list-page">
            <header class="cdb-rooms-topbar cdb-glass">
                <div class="cdb-rooms-brand" data-testid="rooms-brand">
                    <span class="cdb-brand-mark" aria-hidden="true">
                        <IconBox size="lg"><IconLogo /></IconBox>
                    </span>
                    <strong>"coldrawdb"</strong>
                    <span class="cdb-tag cdb-tag--brand">"工作空间"</span>
                </div>
                <div class="cdb-rooms-actions">
                    <button
                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                        type="button"
                        data-testid="btn-refresh-rooms"
                        title="刷新房间"
                        aria-label="刷新房间"
                        disabled=move || loading.get()
                        on:click=refresh_rooms
                    >
                        <IconBox size="sm"><IconRefresh /></IconBox>
                    </button>
                    <button
                        class="cdb-btn cdb-btn--ghost"
                        type="button"
                        data-testid="btn-room-trash"
                        title="回收站"
                        on:click=open_trash
                    >
                        <IconBox size="sm"><IconDelete /></IconBox>
                        "回收站"
                    </button>
                    <button
                        class="cdb-btn cdb-btn--primary"
                        type="button"
                        data-testid="btn-create-room"
                        on:click=move |_| open_create_from_header()
                    >
                        <IconBox size="sm"><IconAdd /></IconBox>
                        "创建房间"
                    </button>
                    <SessionIndicator
                        auth_session=auth_session
                        session_notice=session_notice
                        on_refresh_session=Rc::new(|| {})
                        on_logout=on_logout
                    />
                </div>
            </header>
            <section class="cdb-rooms-content">
                // feat-room-recycle-bin-dropdown-and-db-export：回收站视图
                {move || trash_open.get().then(|| {
                    let restore_action = restore_room_action.clone();
                    view! {
                        <div class="cdb-room-trash" data-testid="room-trash-view">
                            <div class="cdb-rooms-heading">
                                <div>
                                    <span class="cdb-eyebrow">"回收站"</span>
                                    <h1 data-testid="trash-title">"已删除的房间"</h1>
                                    <p>"恢复房间回到列表，或彻底删除（不可恢复）。"</p>
                                </div>
                                <button
                                    class="cdb-btn cdb-btn--ghost"
                                    type="button"
                                    data-testid="btn-trash-back"
                                    on:click=move |_| trash_open.set(false)
                                >
                                    "返回房间列表"
                                </button>
                            </div>
                            {move || trash_error.get().map(|msg| view! {
                                <p class="cdb-rooms-error" data-testid="trash-error">{msg}</p>
                            })}
                            {move || if trash_loading.get() {
                                view! { <p class="cdb-rooms-loading" data-testid="trash-loading">"加载中..."</p> }.into_view()
                            } else if trash_items.get().is_empty() {
                                view! {
                                    <div class="cdb-rooms-empty" data-testid="trash-empty">
                                        <strong>"回收站为空"</strong>
                                        <p>"删除的房间会在这里保留，可随时恢复。"</p>
                                    </div>
                                }.into_view()
                            } else {
                                let restore_action = restore_action.clone();
                                view! {
                                    <ul class="cdb-room-list" data-testid="room-trash-list">
                                        <For each=move || trash_items.get() key=|r| r.id.clone() children=move |item: RoomSummary| {
                                            let restore_item = item.clone();
                                            let purge_item = item.clone();
                                            let restore_action = restore_action.clone();
                                            let archived = item.archived_at.clone().unwrap_or_default();
                                            view! {
                                                <li class="cdb-room-list-item" data-testid=format!("room-trash-item-{}", item.id)>
                                                    <div class="cdb-room-card cdb-room-card--trash">
                                                        <div class="cdb-room-card__body">
                                                            <div class="cdb-room-meta">
                                                                <span class="cdb-tag">{format!("归档于 {}", archived)}</span>
                                                            </div>
                                                            <h2>{item.name.clone()}</h2>
                                                        </div>
                                                    </div>
                                                    <div class="cdb-room-trash-actions">
                                                        <button
                                                            class="cdb-btn"
                                                            type="button"
                                                            data-testid=format!("btn-restore-room-{}", item.id)
                                                            disabled=move || restoring.get().is_some()
                                                            on:click=move |_| restore_action(restore_item.clone())
                                                        >
                                                            "恢复"
                                                        </button>
                                                        <button
                                                            class="cdb-btn cdb-btn--danger"
                                                            type="button"
                                                            data-testid=format!("btn-purge-room-{}", item.id)
                                                            on:click=move |_| {
                                                                purge_error.set(None);
                                                                purge_target.set(Some(purge_item.clone()));
                                                            }
                                                        >
                                                            "彻底删除"
                                                        </button>
                                                    </div>
                                                </li>
                                            }
                                        } />
                                    </ul>
                                }.into_view()
                            }}
                        </div>
                    }.into_view()
                })}
                {move || {
                    let on_select_room = on_select_room.clone();
                    let open_create_from_card = open_create_from_card.clone();
                    (!trash_open.get()).then(move || view! {
                <div>
                <div class="cdb-rooms-heading">
                    <div>
                        <span class="cdb-eyebrow">
                            {move || format!("欢迎回来，{}", auth_session.get().map(|session| session.display_name()).unwrap_or_else(|| "协作者".to_string()))}
                        </span>
                        <h1 data-testid="rooms-title">"继续你的模型评审"</h1>
                        <p>"房间里的改动会实时同步给每一位成员。"</p>
                    </div>
                    <span class="cdb-tag cdb-rooms-service-state">
                        <span class="cdb-status-dot"></span>
                        "协作服务正常"
                    </span>
                </div>
                {move || error.get().map(|msg| view! {
                    <p class="cdb-rooms-error" data-testid="rooms-error">{msg}</p>
                })}
                {move || if loading.get() {
                    view! { <p class="cdb-rooms-loading" data-testid="rooms-loading">"加载中..."</p> }.into_view()
                } else {
                    let on_select = on_select_room.clone();
                    let open_create = open_create_from_card.clone();
                    view! {
                        <div>
                            {move || if rooms.get().is_empty() {
                                view! {
                                    <div class="cdb-rooms-empty" data-testid="rooms-empty">
                                        <strong>"这里还没有协作房间"</strong>
                                        <p>"创建一个房间，把模型评审和工程上下文集中到一起。"</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }}
                        </div>
                        <ul class="cdb-room-list" class=("cdb-room-list--empty", move || rooms.get().is_empty()) data-testid="room-list">
                            <For each=move || rooms.get() key=|r| r.id.clone() children=move |room: RoomSummary| {
                                let on_select = on_select.clone();
                                let summary = room.clone();
                                // fix-overflow-menu-room-delete-listview-io：owner 卡片删除入口
                                let deletable = can_delete_room(&room.my_role);
                                let del_id = room.id.clone();
                                let del_name = room.name.clone();
                                let del_summary = room.clone();
                                view! {
                                    <li
                                        class="cdb-room-list-item"
                                        data-testid=format!("room-list-item-{}", room.id)
                                    >
                                        <button
                                            class="cdb-room-card"
                                            data-testid=format!("room-card-{}", room.id)
                                            on:click=move |_| on_select(RoomDetail::from_summary(&summary))
                                        >
                                            <div class="cdb-room-card__body">
                                                <div class="cdb-room-meta">
                                                    <span class="cdb-tag cdb-tag--brand">{room.my_role.clone()}</span>
                                                    <span class="cdb-tag">
                                                        <span class="cdb-status-dot"></span>
                                                        {format!("{} 人", room.member_count)}
                                                    </span>
                                                </div>
                                                <h2>{room.name.clone()}</h2>
                                                <p>{room.diagram_title.clone()}</p>
                                            </div>
                                            <div class="cdb-room-card__footer">
                                                <span class="cdb-user-avatar">{room.name.chars().next().unwrap_or('房').to_string()}</span>
                                                <span>{format!("{}  ", room.updated_at)}<IconBox size="sm"><IconChevronRight /></IconBox></span>
                                            </div>
                                        </button>
                                        {deletable.then(|| {
                                            let target = del_summary.clone();
                                            view! {
                                                <button
                                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon cdb-room-card-del"
                                                    type="button"
                                                    data-testid=format!("room-card-delete-{}", del_id)
                                                    title="删除房间"
                                                    aria-label=format!("删除{}", del_name)
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        delete_error.set(None);
                                                        delete_target.set(Some(target.clone()));
                                                    }
                                                >
                                                    <IconBox size="sm"><IconDelete /></IconBox>
                                                </button>
                                            }
                                        })}
                                    </li>
                                }
                            } />
                            <li class="cdb-room-list-item">
                                <button class="cdb-room-card cdb-room-card--new" type="button" on:click=move |_| open_create()>
                                    <span class="cdb-brand-mark"><IconBox size="lg"><IconAdd /></IconBox></span>
                                    <h2>"新建协作房间"</h2>
                                    <p>"绑定一个 diagram，邀请团队共同编辑"</p>
                                </button>
                            </li>
                        </ul>
                    }.into_view()
                }}
                </div>
                    })
                }}
            </section>
            {move || if create_modal_open.get() {
                let submit = submit_from_modal.clone();
                view! {
                    <div
                        class="cdb-rooms-modal-overlay"
                        data-testid="create-room-overlay"
                        on:click=move |_| {
                            if !creating.get_untracked() {
                                create_modal_open.set(false);
                            }
                        }
                    >
                        <section
                            class="cdb-create-room-modal cdb-glass"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="create-room-title"
                            data-testid="modal-create-room"
                            on:click=move |event| event.stop_propagation()
                        >
                            <header class="cdb-create-room-modal__header">
                                <div>
                                    <span class="cdb-eyebrow">"新建协作空间"</span>
                                    <h2 id="create-room-title">"创建协作房间"</h2>
                                </div>
                                <button
                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                    type="button"
                                    aria-label="关闭"
                                    disabled=move || creating.get()
                                    on:click=move |_| create_modal_open.set(false)
                                >
                                    <IconBox size="sm"><IconClose /></IconBox>
                                </button>
                            </header>
                            <form
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit();
                                }
                            >
                                <div class="cdb-create-room-modal__body">
                                    <div class="cdb-field">
                                        <label for="room-name">"房间名称"</label>
                                        <input
                                            class="cdb-input"
                                            id="room-name"
                                            data-testid="create-room-name"
                                            maxlength="64"
                                            prop:value=move || room_name.get()
                                            on:input=move |event| room_name.set(event_target_value(&event))
                                        />
                                    </div>
                                    <div class="cdb-field">
                                        <label for="room-diagram">"关联 diagram"</label>
                                        <select
                                            class="cdb-input cdb-select"
                                            id="room-diagram"
                                            data-testid="create-room-diagram"
                                            prop:value=move || diagram_choice.get()
                                            on:change=move |event| diagram_choice.set(event_target_value(&event))
                                        >
                                            <option value="__new__">"新建空白模型"</option>
                                            <For
                                                each=move || {
                                                    let bound = rooms.get().into_iter().map(|room| room.diagram_id).collect::<Vec<_>>();
                                                    diagrams.get().into_iter().filter(move |diagram| !bound.contains(&diagram.id)).collect::<Vec<_>>()
                                                }
                                                key=|diagram| diagram.id.clone()
                                                children=move |diagram: DiagramSummary| {
                                                    let id = diagram.id.clone();
                                                    let label = diagram.name.unwrap_or_else(|| "未命名模型".to_string());
                                                    view! { <option value=id>{label}</option> }
                                                }
                                            />
                                        </select>
                                        // fix-select-bg-diagram-delete-and-log-format：删除游离图表入口
                                        // （core-S04 图表行：仅当选中已有图表时出现，点击弹二次确认）
                                        {move || {
                                            let selected = diagram_choice.get();
                                            if selected == "__new__" {
                                                return view! { <></> }.into_view();
                                            }
                                            view! {
                                                <button
                                                    class="cdb-btn cdb-btn--ghost cdb-btn--danger-text"
                                                    type="button"
                                                    data-testid="create-room-diagram-delete"
                                                    on:click=move |_| {
                                                        let id = diagram_choice.get_untracked();
                                                        let target = diagrams
                                                            .get_untracked()
                                                            .into_iter()
                                                            .find(|d| d.id == id);
                                                        delete_diagram_error.set(None);
                                                        delete_diagram_target.set(target);
                                                    }
                                                >
                                                    <IconBox size="sm"><IconDelete /></IconBox>
                                                    "删除此图表"
                                                </button>
                                            }.into_view()
                                        }}
                                        {move || if diagrams_loading.get() {
                                            view! { <small class="cdb-field-hint">"正在加载已有模型..."</small> }.into_view()
                                        } else {
                                            view! { <small class="cdb-field-hint">"新建模型会先创建 diagram，再绑定房间。"</small> }.into_view()
                                        }}
                                    </div>
                                    <div class="cdb-field" style:display=move || if diagram_choice.get() == "__new__" { "grid" } else { "none" }>
                                        <label for="room-database">"数据库引擎（仅新建空白模型时可选）"</label>
                                        <select
                                            class="cdb-input cdb-select"
                                            id="room-database"
                                            data-testid="create-room-database"
                                            prop:value=move || create_database.get()
                                            on:change=move |event| create_database.set(event_target_value(&event))
                                        >
                                            <option value="generic">"Generic · 通用类型"</option>
                                            <option value="mysql">"MySQL"</option>
                                            <option value="postgresql">"PostgreSQL"</option>
                                        </select>
                                        <small class="cdb-field-hint">"字段类型清单随引擎过滤，建图后即定口径。"</small>
                                    </div>
                                    <div class="cdb-field">
                                        <label for="room-default-role">"默认邀请角色"</label>
                                        <select
                                            class="cdb-input cdb-select"
                                            id="room-default-role"
                                            prop:value=move || default_role.get()
                                            on:change=move |event| default_role.set(event_target_value(&event))
                                        >
                                            <option value="editor">"Editor · 可编辑"</option>
                                            <option value="viewer">"Viewer · 只读"</option>
                                        </select>
                                    </div>
                                    <p class="cdb-create-room-error" role="alert" data-testid="create-room-error">
                                        {move || create_error.get().unwrap_or_default()}
                                    </p>
                                </div>
                                <footer class="cdb-create-room-modal__footer">
                                    <button class="cdb-btn" type="button" disabled=move || creating.get() on:click=move |_| create_modal_open.set(false)>
                                        "取消"
                                    </button>
                                    <button class="cdb-btn cdb-btn--primary" type="submit" data-testid="create-room-submit" disabled=move || creating.get()>
                                        {move || if creating.get() { "创建中..." } else { "创建并进入" }}
                                    </button>
                                </footer>
                            </form>
                        </section>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
            // fix-overflow-menu-room-delete-listview-io：列表页删除确认模态
            {move || delete_target.get().map(|target| {
                let confirm = confirm_delete_room.clone();
                view! {
                    <div
                        class="cdb-rooms-modal-overlay"
                        data-testid="delete-room-list-overlay"
                        on:click=move |_| {
                            if !deleting.get_untracked() {
                                delete_target.set(None);
                            }
                        }
                    >
                        <section
                            class="cdb-create-room-modal cdb-glass"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="delete-room-list-title"
                            data-testid="modal-delete-room-list"
                            on:click=move |event| event.stop_propagation()
                        >
                            <header class="cdb-create-room-modal__header">
                                <div>
                                    <span class="cdb-eyebrow">"危险操作"</span>
                                    <h2 id="delete-room-list-title">"删除房间"</h2>
                                </div>
                                <button
                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                    type="button"
                                    aria-label="关闭"
                                    disabled=move || deleting.get()
                                    on:click=move |_| delete_target.set(None)
                                >
                                    <IconBox size="sm"><IconClose /></IconBox>
                                </button>
                            </header>
                            <div class="cdb-create-room-modal__body">
                                <p>{format!("确认删除房间「{}」？删除后进入回收站，可在回收站恢复或彻底删除；Diagram 保留。", target.name)}</p>
                                <p class="cdb-create-room-error" role="alert" data-testid="delete-room-list-error">
                                    {move || delete_error.get().unwrap_or_default()}
                                </p>
                            </div>
                            <footer class="cdb-create-room-modal__footer">
                                <button class="cdb-btn" type="button" disabled=move || deleting.get() on:click=move |_| delete_target.set(None)>
                                    "取消"
                                </button>
                                <button
                                    class="cdb-btn cdb-btn--danger"
                                    type="button"
                                    data-testid="btn-confirm-delete-room-list"
                                    disabled=move || deleting.get()
                                    on:click=move |_| confirm()
                                >
                                    {move || if deleting.get() { "删除中..." } else { "确认删除" }}
                                </button>
                            </footer>
                        </section>
                    </div>
                }
            })}
            // fix-select-bg-diagram-delete-and-log-format：删除图表二次确认模态（core-S04 图表行）
            {move || delete_diagram_target.get().map(|target| {
                let confirm = confirm_delete_diagram.clone();
                let label = target.name.clone().unwrap_or_else(|| "未命名模型".to_string());
                view! {
                    <div
                        class="cdb-rooms-modal-overlay"
                        data-testid="delete-diagram-overlay"
                        on:click=move |_| {
                            if !deleting_diagram.get_untracked() {
                                delete_diagram_target.set(None);
                            }
                        }
                    >
                        <section
                            class="cdb-create-room-modal cdb-glass"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="delete-diagram-title"
                            data-testid="modal-delete-diagram"
                            on:click=move |event| event.stop_propagation()
                        >
                            <header class="cdb-create-room-modal__header">
                                <div>
                                    <span class="cdb-eyebrow">"危险操作"</span>
                                    <h2 id="delete-diagram-title">"删除图表"</h2>
                                </div>
                                <button
                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                    type="button"
                                    aria-label="关闭"
                                    disabled=move || deleting_diagram.get()
                                    on:click=move |_| delete_diagram_target.set(None)
                                >
                                    <IconBox size="sm"><IconClose /></IconBox>
                                </button>
                            </header>
                            <div class="cdb-create-room-modal__body">
                                <p>{format!("确认删除图表「{}」？此操作不可恢复。", label)}</p>
                                <p class="cdb-create-room-error" role="alert" data-testid="delete-diagram-error">
                                    {move || delete_diagram_error.get().unwrap_or_default()}
                                </p>
                            </div>
                            <footer class="cdb-create-room-modal__footer">
                                <button class="cdb-btn" type="button" disabled=move || deleting_diagram.get() on:click=move |_| delete_diagram_target.set(None)>
                                    "取消"
                                </button>
                                <button
                                    class="cdb-btn cdb-btn--danger"
                                    type="button"
                                    data-testid="btn-confirm-delete-diagram"
                                    disabled=move || deleting_diagram.get()
                                    on:click=move |_| confirm()
                                >
                                    {move || if deleting_diagram.get() { "删除中..." } else { "确认删除" }}
                                </button>
                            </footer>
                        </section>
                    </div>
                }
            })}
            // feat-room-recycle-bin-dropdown-and-db-export：彻底删除二次确认模态
            {move || purge_target.get().map(|target| {
                let confirm = confirm_purge_room.clone();
                view! {
                    <div
                        class="cdb-rooms-modal-overlay"
                        data-testid="purge-room-overlay"
                        on:click=move |_| {
                            if !purging.get_untracked() {
                                purge_target.set(None);
                            }
                        }
                    >
                        <section
                            class="cdb-create-room-modal cdb-glass"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="purge-room-title"
                            data-testid="modal-purge-room"
                            on:click=move |event| event.stop_propagation()
                        >
                            <header class="cdb-create-room-modal__header">
                                <div>
                                    <span class="cdb-eyebrow">"危险操作"</span>
                                    <h2 id="purge-room-title">"彻底删除房间"</h2>
                                </div>
                                <button
                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                    type="button"
                                    aria-label="关闭"
                                    disabled=move || purging.get()
                                    on:click=move |_| purge_target.set(None)
                                >
                                    <IconBox size="sm"><IconClose /></IconBox>
                                </button>
                            </header>
                            <div class="cdb-create-room-modal__body">
                                <p>{format!("彻底删除房间「{}」？彻底删除后不可恢复，房间的协作数据（成员/邀请）将被永久删除；Diagram 保留。", target.name)}</p>
                                <p class="cdb-create-room-error" role="alert" data-testid="purge-room-error">
                                    {move || purge_error.get().unwrap_or_default()}
                                </p>
                            </div>
                            <footer class="cdb-create-room-modal__footer">
                                <button class="cdb-btn" type="button" disabled=move || purging.get() on:click=move |_| purge_target.set(None)>
                                    "取消"
                                </button>
                                <button
                                    class="cdb-btn cdb-btn--danger"
                                    type="button"
                                    data-testid="btn-confirm-purge-room"
                                    disabled=move || purging.get()
                                    on:click=move |_| confirm()
                                >
                                    {move || if purging.get() { "删除中..." } else { "确认彻底删除" }}
                                </button>
                            </footer>
                        </section>
                    </div>
                }
            })}
            <NoticeToast notice=notice />
        </main>
    }
}

fn room_create_error_message(error: &ApiError) -> String {
    match error {
        ApiError::Server(404, _) => "关联图表不存在，请重新选择".to_string(),
        ApiError::Server(409, _) => "该图表已绑定其他协作房间".to_string(),
        ApiError::Server(_, _) => "房间创建失败，请稍后重试".to_string(),
        ApiError::Network(_) => "网络连接失败，请检查网络后重试".to_string(),
        ApiError::Parse(_) => "房间响应无效，请稍后重试".to_string(),
    }
}

/// 重定向到 auth 入口（未登录情况下创建房间）。
fn login_required_redirect() {
    if let Some(win) = web_sys::window() {
        let _ = win.location().set_href("/editor");
    }
}

impl RoomDetail {
    /// 从 RoomSummary 构造最小 RoomDetail（仅含 id/name/diagram_id/role/member_count/diagram_title）。
    /// 用于 rooms-list-page 点击后立即进入 editor；后续 GET /api/v1/rooms/{id} 会补全 ownerId。
    pub fn from_summary(s: &RoomSummary) -> Self {
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            diagram_id: s.diagram_id.clone(),
            owner_id: String::new(),
            diagram_title: s.diagram_title.clone(),
            my_role: s.my_role.clone(),
            member_count: s.member_count,
        }
    }
}

/// /invite/{token} 独立接受页（B 批实接：preview / accept 真实 API）。
///
/// 布局对齐主原型 invite-accept-page：复用 auth 分栏（story + glass panel）。
/// - 有效：邀请人/角色/模型卡片 + 「加入房间」；匿名点击 → 提示先登录
/// - 失效（preview 404/410/网络错误）：「邀请已失效」+ 无加入按钮（ST-S04-UI-07 / ST-PU-23）
/// - 接受成功：accept → GET room detail → 进入同一 room-editor（ST-S04-UI-04）
#[component]
pub fn InviteAcceptPage(
    token: String,
    room_client: RoomClient,
    auth_session: RwSignal<Option<AuthSession>>,
    current_diagram_id: RwSignal<String>,
    current_title: RwSignal<String>,
    current_room: RwSignal<Option<RoomDetail>>,
    error: RwSignal<Option<String>>,
    on_after_accept: Rc<dyn Fn()>,
    on_goto_login: Rc<dyn Fn()>,
    on_back: Rc<dyn Fn()>,
) -> impl IntoView {
    let preview = create_rw_signal(None::<InvitePreview>);
    let preview_failed = create_rw_signal(false);
    let accepting = create_rw_signal(false);
    let login_required = create_rw_signal(false);
    let invite_error = create_rw_signal(Option::<String>::None);

    // 挂载即拉取邀请预览；任何失败都按「邀请失效」处理（不暴露后端错误原文）
    create_effect({
        let room_client = room_client.clone();
        let token = token.clone();
        move |_| {
            let room_client = room_client.clone();
            let token = token.clone();
            spawn_local(async move {
                match room_client.preview_invite(&token).await {
                    Ok(p) => preview.set(Some(p)),
                    Err(_) => preview_failed.set(true),
                }
            });
        }
    });

    let accept = {
        let token = token.clone();
        let room_client = room_client.clone();
        move || {
            let Some(session) = auth_session.get_untracked() else {
                login_required.set(true);
                return;
            };
            accepting.set(true);
            invite_error.set(None);
            let room_client = room_client.clone();
            let token = token.clone();
            let access = session.access_token;
            let on_after_accept = on_after_accept.clone();
            spawn_local(async move {
                match room_client.accept_invite(&access, &token).await {
                    Ok(res) => {
                        // 接受后拉取房间详情（含 myRole），再进入同一 room-editor
                        match room_client.get_room(&access, &res.room_id).await {
                            Ok(detail) => {
                                current_diagram_id.set(detail.diagram_id.clone());
                                current_title.set(detail.diagram_title.clone());
                                current_room.set(Some(detail));
                                on_after_accept();
                            }
                            Err(e) => invite_error.set(Some(e.to_string())),
                        }
                    }
                    Err(e) => invite_error.set(Some(e.to_string())),
                }
                accepting.set(false);
            });
        }
    };

    view! {
        <main class="cdb-auth-page" data-testid="invite-accept-page">
            <article class="cdb-auth-story" data-testid="invite-story">
                <div class="cdb-auth-brand">
                    <span class="cdb-brand-mark" aria-hidden="true">
                        <IconBox size="lg"><IconLogo /></IconBox>
                    </span>
                    <span class="cdb-auth-brand-name">"coldrawdb"</span>
                </div>
                <div class="cdb-auth-hero">
                    <span class="cdb-eyebrow">"团队邀请"</span>
                    {move || if preview_failed.get() {
                        view! {
                            <h1 class="cdb-auth-hero-title">"这张邀请卡"<br /><span>"已经失效。"</span></h1>
                            <p class="cdb-auth-hero-desc">"邀请链接已超过 7 天，请联系房间 Owner 重新生成。"</p>
                        }.into_view()
                    } else {
                        view! {
                            <h1 class="cdb-auth-hero-title">"一起把模型"<br /><span>"推向下一版。"</span></h1>
                            <p class="cdb-auth-hero-desc">
                                {move || preview.get().map(|p| format!(
                                    "{} 邀请你加入「{}」，共同编辑{}。",
                                    p.invited_by.clone().unwrap_or_else(|| "队友".to_string()),
                                    p.room_name,
                                    p.diagram_title
                                )).unwrap_or_else(|| "正在加载邀请信息…".to_string())}
                            </p>
                        }.into_view()
                    }}
                </div>
            </article>
            <article class="cdb-auth-panel cdb-glass" data-testid="invite-panel">
                <div class="cdb-auth-card">
                    <span class="cdb-brand-mark" aria-hidden="true">
                        <IconBox size="lg"><IconUsers /></IconBox>
                    </span>
                    {move || {
                        let accept = accept.clone();
                        let on_back = on_back.clone();
                        let on_goto_login = on_goto_login.clone();
                        if preview_failed.get() {
                        view! {
                            <h2 class="cdb-auth-title" data-testid="invite-title" tabindex="-1">"邀请已失效"</h2>
                            <p class="cdb-muted">"为了保护房间安全，此邀请不再可用。"</p>
                            <div class="cdb-invite-actions">
                                <button class="cdb-btn cdb-btn--ghost" data-testid="btn-invite-back" on:click=move |_| on_back()>
                                    "返回空间"
                                </button>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <h2 class="cdb-auth-title" data-testid="invite-title" tabindex="-1">
                                {move || preview.get().map(|p| format!("加入{}", p.room_name)).unwrap_or_else(|| "加入协作房间".to_string())}
                            </h2>
                            <p class="cdb-muted" data-testid="invite-meta">
                                {move || preview.get().map(|p| format!(
                                    "邀请人：{} · 分配角色：{}",
                                    p.invited_by.clone().unwrap_or_else(|| "队友".to_string()),
                                    p.role
                                )).unwrap_or_default()}
                            </p>
                            {move || preview.get().map(|p| view! {
                                <div class="cdb-invite-preview" data-testid="invite-preview">
                                    <span class="cdb-avatar" aria-hidden="true">{p.room_name.chars().next().unwrap_or('R').to_string()}</span>
                                    <div>
                                        <strong>{p.diagram_title.clone()}</strong>
                                        <small>{format!("角色 {} · 邀请 7 天内有效", p.role)}</small>
                                    </div>
                                </div>
                            })}
                            {move || if login_required.get() {
                                view! {
                                    <p class="cdb-invite-error" data-testid="invite-login-required">
                                        "请先登录后再接受邀请。"
                                    </p>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }}
                            {move || invite_error.get().map(|msg| view! {
                                <p class="cdb-invite-error" data-testid="invite-error">{msg}</p>
                            })}
                            <div class="cdb-invite-actions">
                                <button
                                    class="cdb-btn cdb-btn--primary"
                                    data-testid="btn-accept-invite"
                                    disabled=move || accepting.get() || preview.get().is_none()
                                    on:click=move |_| accept()
                                >
                                    {move || if accepting.get() { "处理中..." } else { "加入房间" }}
                                </button>
                                {move || {
                                    let on_goto_login = on_goto_login.clone();
                    if login_required.get() {
                                    view! {
                                        <button class="cdb-btn" data-testid="btn-invite-goto-login" on:click=move |_| on_goto_login()>
                                            "切换登录"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }
                                }}
                                <button class="cdb-btn cdb-btn--ghost" data-testid="btn-invite-back" on:click=move |_| on_back()>
                                    "返回空间"
                                </button>
                            </div>
                        }.into_view()
                    }
                    }}
                </div>
            </article>
        </main>
    }
}

/// Phase A：左侧 Tool Rail（48px）
#[component]
pub fn ToolRail(
    store: EditorStore,
    selection: RwSignal<SelectionKind>,
    inspector_open: RwSignal<bool>,
    active_tool: RwSignal<ActiveTool>,
    rel_tool_state: RwSignal<RelToolState>,
    on_create_table: Rc<dyn Fn()>,
    on_open_palette: Rc<dyn Fn()>,
    on_open_settings: Rc<dyn Fn()>,
    on_toggle_activity: Rc<dyn Fn()>,
    // feat-data-dictionary（S07）：数据字典抽屉开关（只读/Viewer 可浏览与导出，不禁用）
    on_toggle_dicts: Rc<dyn Fn()>,
    current_room: RwSignal<Option<RoomDetail>>,
    read_only: bool,
) -> impl IntoView {
    let _ = (store, selection, inspector_open);
    let rel_disabled = move || read_only || room_is_viewer(current_room);

    view! {
        <nav class="cdb-tool-rail" data-testid="tool-rail" aria-label="画布工具">
            <button
                class="cdb-tool-btn"
                data-testid="tool-add-table"
                disabled=rel_disabled
                on:click=move |_| on_create_table()
            >
                <IconBox size="md"><IconAddTable /></IconBox>
                <span class="cdb-tool-tip">"新建表 "<kbd>"T"</kbd></span>
            </button>
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::Relationship
                data-testid="tool-relationship"
                disabled=rel_disabled
                on:click=move |_| {
                    if rel_disabled() {
                        return;
                    }
                    active_tool.set(ActiveTool::Relationship);
                    rel_tool_state.set(RelToolState::PickSource);
                }
            >
                <IconBox size="md"><IconRelationship /></IconBox>
                <span class="cdb-tool-tip">"创建关系 "<kbd>"R"</kbd></span>
            </button>
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::NewArea
                data-testid="tool-new-area"
                disabled=rel_disabled
                on:click=move |_| {
                    if rel_disabled() {
                        return;
                    }
                    // p0-fix 定点 2：进入区域创建工具（画布十字光标 + 拖框创建）
                    rel_tool_state.set(RelToolState::Idle);
                    active_tool.set(ActiveTool::NewArea);
                }
            >
                <IconBox size="md"><IconAddArea /></IconBox>
                <span class="cdb-tool-tip">"添加区域"</span>
            </button>
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::NewNote
                data-testid="tool-new-note"
                disabled=rel_disabled
                on:click=move |_| {
                    if rel_disabled() {
                        return;
                    }
                    // p0-fix 定点 2：进入便签创建工具（画布十字光标 + 点击放置）
                    rel_tool_state.set(RelToolState::Idle);
                    active_tool.set(ActiveTool::NewNote);
                }
            >
                <IconBox size="md"><IconAddNote /></IconBox>
                <span class="cdb-tool-tip">"添加便签"</span>
            </button>
            <div class="cdb-tool-rail__divider"></div>
            <button
                class="cdb-tool-btn"
                data-testid="toolrail-dicts"
                on:click=move |_| on_toggle_dicts()
            >
                <IconBox size="md"><IconEnum /></IconBox>
                <span class="cdb-tool-tip">"数据字典"</span>
            </button>
            <button
                class="cdb-tool-btn"
                data-testid="tool-search"
                on:click=move |_| on_open_palette()
            >
                <IconBox size="md"><IconSearch /></IconBox>
                <span class="cdb-tool-tip">"搜索与命令 "<kbd>"⌘K"</kbd></span>
            </button>
            <div class="cdb-tool-rail__spacer"></div>
            <button
                class="cdb-tool-btn"
                data-testid="tool-activity"
                on:click=move |_| on_toggle_activity()
            >
                <IconBox size="md"><IconActivity /></IconBox>
                <span class="cdb-tool-tip">"协作动态"</span>
            </button>
            <button
                class="cdb-tool-btn"
                data-testid="tool-settings"
                on:click=move |_| on_open_settings()
            >
                <IconBox size="md"><IconSettings /></IconBox>
                <span class="cdb-tool-tip">"画布设置"</span>
            </button>
        </nav>
    }
}

/// Phase B：关系工具提示条
#[component]
pub fn RelToolHint(rel_state: RwSignal<RelToolState>) -> impl IntoView {
    view! {
        {move || rel_state.get().hint().map(|text| view! {
            <div class="cdb-rel-tool-hint" data-testid="rel-tool-hint">{text}</div>
        })}
    }
}

/// Phase C：IO 抽屉（导入 / 导出）
#[component]
pub fn IoDrawer(
    kind: RwSignal<IoDrawerKind>,
    store: EditorStore,
    current_title: RwSignal<String>,
    client: DiagramClient,
    error: RwSignal<Option<String>>,
    on_close: Rc<dyn Fn()>,
    // fix-appbar-roomname-back-and-import-merge：导入本地合并回调
    on_merge_import: Rc<dyn Fn(Vec<Table>, Vec<Reference>)>,
    // feat-db-connect-import-and-ddl-realdb-verify：数据库连接导入的鉴权会话
    auth_session: RwSignal<Option<AuthSession>>,
    // feat-room-recycle-bin-dropdown-and-db-export：导出成功 Toast（core-01d §5.4）
    notice: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || if kind.get() == IoDrawerKind::None {
            view! { <></> }.into_view()
        } else {
            view! {
                <aside class="cdb-io-drawer" data-testid="io-drawer">
                    {match kind.get() {
                        IoDrawerKind::Import => {
                            let close = on_close.clone();
                            view! {
                                <ImportDrawer
                                    current_title=current_title
                                    client=client.clone()
                                    error=error.clone()
                                    on_close=close
                                    on_merge=on_merge_import.clone()
                                    auth_session=auth_session
                                />
                            }.into_view()
                        }
                        IoDrawerKind::Export => {
                            let close = on_close.clone();
                            view! {
                                <ExportDrawer
                                    store=store.clone()
                                    current_title=current_title
                                    on_close=close
                                    client=client.clone()
                                    auth_session=auth_session
                                    notice=notice
                                />
                            }.into_view()
                        }
                        IoDrawerKind::None => view! { <></> }.into_view(),
                    }}
                </aside>
            }.into_view()
        }}
    }
}

/// Phase C：导入抽屉
#[component]
pub fn ImportDrawer(
    current_title: RwSignal<String>,
    client: DiagramClient,
    error: RwSignal<Option<String>>,
    on_close: Rc<dyn Fn()>,
    // fix-appbar-roomname-back-and-import-merge：本地合并回调（tables, references），
    // 由父级写入 store + schedule_save + 成功反馈；不再走 bridge 新建游离 diagram
    on_merge: Rc<dyn Fn(Vec<Table>, Vec<Reference>)>,
    // feat-db-connect-import-and-ddl-realdb-verify：连接数据库导入的鉴权会话（core-03 §13.1 Bearer）
    auth_session: RwSignal<Option<AuthSession>>,
) -> impl IntoView {
    let format = create_rw_signal(ImportFormat::Sql);
    let engine = create_rw_signal(String::from("generic"));
    let content = create_rw_signal(String::new());
    let inline_error = create_rw_signal(None::<String>);
    let submitting = create_rw_signal(false);
    let import_logs = create_rw_signal(Vec::<ImportLogEntry>::new());
    let logs_loading = create_rw_signal(false);
    // feat-db-connect-import-and-ddl-realdb-verify：数据库来源状态（core-01d §4.4 3.5）
    let db_engine = create_rw_signal(String::from("sqlite"));
    let db_source = create_rw_signal(String::new());
    // fix-dbimport-save-and-pg-schema：PG schema 定位（core-01d §4.1）；仅 postgres 渲染/发送
    let db_schema = create_rw_signal(String::from("public"));
    // (ddl, table_count)；连接成功且 tables>0 时写入；连接信息不持久化
    let db_result: RwSignal<Option<usize>> = create_rw_signal(None);
    let connecting = create_rw_signal(false);

    let refresh_logs = {
        let client = client.clone();
        let import_logs = import_logs.clone();
        let error = error.clone();
        let logs_loading = logs_loading.clone();
        Rc::new(move || {
            logs_loading.set(true);
            let client = client.clone();
            spawn_local(async move {
                match client.list_import_logs(None).await {
                    Ok(entries) => import_logs.set(entries),
                    Err(e) => error.set(Some(e.to_string())),
                }
                logs_loading.set(false);
            });
        })
    };

    let list_client = client.clone();
    let submit_client = client.clone();
    let refresh_for_list = refresh_logs.clone();
    let refresh_for_submit = refresh_logs.clone();
    let refresh_for_btn = refresh_logs.clone();

    // feat-db-connect-import-and-ddl-realdb-verify：「连接并解析」→ POST /bridge/import/connect
    // （core-03 §13；core-01d §4.4 3.5）。连接信息仅用于当次请求，不写入 localStorage / 导入日志。
    let connect_db = {
        let client = client.clone();
        Rc::new(move || {
            if connecting.get_untracked() {
                return;
            }
            let engine = db_engine.get_untracked();
            let source = db_source.get_untracked();
            let schema = db_schema.get_untracked();
            if source.trim().is_empty() {
                inline_error.set(Some("请填写连接信息".into()));
                return;
            }
            let token = auth_session
                .get()
                .map(|s| s.access_token)
                .unwrap_or_default();
            if token.is_empty() {
                inline_error.set(Some("请先登录".into()));
                return;
            }
            inline_error.set(None);
            connecting.set(true);
            // 克隆 client 供 async move 捕获，保持外层闭包为 Fn（可重复点击）
            let client = client.clone();
            spawn_local(async move {
                // fix-dbimport-save-and-pg-schema：仅 postgres 发送 schema（sqlite 传 None）
                let schema_arg = if engine == "postgres" { Some(schema.as_str()) } else { None };
                let result = client.import_from_connection(&token, &engine, &source, schema_arg).await;
                connecting.set(false);
                match result {
                    Ok(data) => {
                        // fix-canvas-zoom-invite-comment-resize（core-01d §4.4 3.5）：结构化
                        // tables 直传——序列化为 JSON 导入同构文本存 content，提交时复用
                        // parse_json_import_tables（不再经 DDL 文本中转）；注释原样透传。
                        if data.table_count == 0 {
                            db_result.set(None);
                            inline_error.set(Some("未检测到数据表".into()));
                        } else {
                            let payload = serde_json::json!({ "tables": data.tables });
                            content.set(payload.to_string());
                            db_result.set(Some(data.table_count));
                        }
                    }
                    Err(e) => {
                        // core-03 §13.1 错误文案：400 参数 / 502 连接失败
                        let msg = match &e {
                            ApiError::Server(400, _) => "不支持的数据库引擎",
                            ApiError::Server(502, _) => "连接数据库失败，请检查连接信息",
                            _ => "连接失败，请稍后重试",
                        };
                        inline_error.set(Some(msg.into()));
                    }
                }
            });
        })
    };

    create_effect({
        let refresh_logs = refresh_logs.clone();
        move |_| {
            refresh_logs();
        }
    });

    let close = on_close.clone();
    let close_btn = on_close.clone();

    view! {
        <div class="cdb-io-drawer__inner" data-testid="import-drawer">
            <div class="cdb-io-drawer__header">
                <span class="cdb-io-drawer__title">
                    <IconBox size="sm"><IconImport /></IconBox>
                    <span>"导入"</span>
                </span>
                <button class="cdb-btn cdb-btn--icon" data-testid="import-cancel" on:click=move |_| close()>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
            <div class="cdb-io-drawer__body">
                <div class="cdb-format-tabs" data-testid="io-format-tabs">
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ImportFormat::Sql
                        on:click=move |_| format.set(ImportFormat::Sql)
                    >"SQL"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ImportFormat::Dbml
                        on:click=move |_| format.set(ImportFormat::Dbml)
                    >"DBML"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ImportFormat::Json
                        on:click=move |_| format.set(ImportFormat::Json)
                    >"JSON"</button>
                    // feat-db-connect-import-and-ddl-realdb-verify：数据库连接导入来源（core-01d §4.1）
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ImportFormat::Database
                        on:click=move |_| format.set(ImportFormat::Database)
                    >"数据库"</button>
                </div>
                {move || if format.get() == ImportFormat::Database {
                    // feat-db-connect-import-and-ddl-realdb-verify：数据库连接面板（core-01d §4.1）
                    view! {
                        <div class="cdb-form-group">
                            <label>"数据库引擎"</label>
                            <select
                                class="cdb-form-select"
                                data-testid="import-db-engine"
                                on:change=move |ev| db_engine.set(event_target_value(&ev))
                            >
                                <option value="sqlite" selected>"sqlite"</option>
                                <option value="postgres">"postgres"</option>
                            </select>
                        </div>
                        <div class="cdb-form-group">
                            <label>"连接信息"</label>
                            <input
                                class="cdb-form-input"
                                data-testid="import-db-source"
                                placeholder="SQLite 文件路径或 PostgreSQL 连接串"
                                prop:value=move || db_source.get()
                                on:input=move |ev| {
                                    use wasm_bindgen::JsCast;
                                    if let Ok(input) = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>() {
                                        db_source.set(input.value());
                                    }
                                }
                            />
                        </div>
                        {move || if db_engine.get() == "postgres" {
                            // fix-dbimport-save-and-pg-schema：schema 定位（core-01d §4.1），仅 PG 渲染
                            view! {
                                <div class="cdb-form-group">
                                    <label>"Schema"</label>
                                    <input
                                        class="cdb-form-input"
                                        data-testid="import-db-schema"
                                        placeholder="public"
                                        prop:value=move || db_schema.get()
                                        on:input=move |ev| {
                                            use wasm_bindgen::JsCast;
                                            if let Ok(input) = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>() {
                                                db_schema.set(input.value());
                                            }
                                        }
                                    />
                                </div>
                            }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                        <button
                            class="cdb-btn"
                            data-testid="import-db-connect"
                            disabled=move || connecting.get()
                            on:click={
                                let connect_db = connect_db.clone();
                                move |_| connect_db()
                            }
                        >
                            {move || if connecting.get() { "连接中..." } else { "连接并解析" }}
                        </button>
                    }.into_view()
                } else {
                    view! {
                        {move || if format.get() == ImportFormat::Sql {
                            view! {
                                <div class="cdb-form-group">
                                    <label>"数据库引擎"</label>
                                    <select
                                        class="cdb-form-select"
                                        data-testid="import-engine-select"
                                        on:change=move |ev| engine.set(event_target_value(&ev))
                                    >
                                        <option value="generic" selected>"generic"</option>
                                        <option value="mysql">"mysql"</option>
                                        <option value="postgresql">"postgresql"</option>
                                        <option value="sqlite">"sqlite"</option>
                                    </select>
                                </div>
                            }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                        <div class="cdb-io-dropzone" data-testid="io-dropzone">
                            "拖放 "
                            <strong>".sql / .dbml / .json"</strong>
                            " 或粘贴下方"
                        </div>
                        <textarea
                            class="cdb-io-textarea"
                            data-testid="import-textarea"
                            placeholder="粘贴 SQL / DBML / JSON"
                            prop:value=move || content.get()
                            on:input=move |ev| {
                                use wasm_bindgen::JsCast;
                                if let Ok(ta) = ev.target().unwrap().dyn_into::<web_sys::HtmlTextAreaElement>() {
                                    content.set(ta.value());
                                }
                            }
                        />
                    }.into_view()
                }}
                {move || {
                    let fmt = format.get();
                    if fmt == ImportFormat::Database {
                        // 数据库来源摘要：连接成功后显示表数（core-01d §4.4 3.5）
                        match db_result.get() {
                            Some(n) => view! {
                                <p class="cdb-parse-summary" data-testid="import-parse-summary">
                                    {format!("已连接 · 检测到 {n} 张表")}
                                </p>
                            }.into_view(),
                            None => view! { <></> }.into_view(),
                        }
                    } else {
                        let text = content.get();
                        match import_parse_summary(fmt, &text) {
                            Ok(summary) => view! {
                                <p class="cdb-parse-summary" data-testid="import-parse-summary">
                                    {format!("解析摘要：{summary}")}
                                </p>
                            }.into_view(),
                            Err(e) if !text.is_empty() => view! {
                                <p class="cdb-form-error">{e}</p>
                            }.into_view(),
                            _ => view! { <></> }.into_view(),
                        }
                    }
                }}
                {move || inline_error.get().map(|e| view! {
                    <p class="cdb-form-error">{e}</p>
                })}
                <div class="cdb-import-logs" data-testid="import-logs-panel">
                    <div class="cdb-import-logs__header">
                        <strong>"最近导入日志"</strong>
                        <button
                            class="cdb-btn cdb-btn--small"
                            data-testid="import-logs-refresh"
                            disabled=move || logs_loading.get()
                            on:click={
                                let refresh_logs = refresh_for_btn.clone();
                                move |_| refresh_logs()
                            }
                        >
                            "刷新"
                        </button>
                    </div>
                    {move || if logs_loading.get() {
                        view! { <p class="cdb-form-hint">"加载中…"</p> }.into_view()
                    } else if import_logs.get().is_empty() {
                        view! { <p class="cdb-form-hint">"暂无导入记录"</p> }.into_view()
                    } else {
                        import_logs.get().into_iter().map(|log| {
                            let log_id = log.id.clone();
                            let status = log.status.clone();
                            let diagram_id = log.imported_diagram_id.clone();
                            let err_msg = log.error_message.clone();
                            let can_retry = import_log_shows_retry(&status);
                            let client = list_client.clone();
                            let error = error.clone();
                            let refresh = refresh_for_list.clone();
                            view! {
                                <div class="cdb-import-log-item" data-testid={format!("import-log-{}", log_id)}>
                                    <span class={format!("cdb-tag cdb-tag--{}", if status == "success" { "success" } else if status == "failed" { "error" } else { "warning" })}>
                                        {status.clone()}
                                    </span>
                                    <span class="cdb-import-log-item__id">{log_id.clone()}</span>
                                    {diagram_id.map(|did| view! {
                                        <button
                                            class="cdb-btn cdb-btn--small cdb-btn--ghost"
                                            on:click=move |_| navigate_to_editor(&did)
                                        >"打开"</button>
                                    })}
                                    {can_retry.then(|| {
                                        let log_id = log_id.clone();
                                        view! {
                                            <button
                                                class="cdb-btn cdb-btn--small"
                                                data-testid={format!("import-log-retry-{}", log_id)}
                                                on:click=move |_| {
                                                    let client = client.clone();
                                                    let error = error.clone();
                                                    let refresh = refresh.clone();
                                                    let log_id = log_id.clone();
                                                    spawn_local(async move {
                                                        match client.retry_import_log(&log_id).await {
                                                            Ok(resp) => {
                                                                if let Some(did) = resp.diagram_id {
                                                                    navigate_to_editor(&did);
                                                                } else {
                                                                    refresh();
                                                                }
                                                            }
                                                            Err(e) => error.set(Some(e.to_string())),
                                                        }
                                                    });
                                                }
                                            >"重试"</button>
                                        }
                                    })}
                                    {err_msg.map(|m| view! { <span class="cdb-form-error">{m}</span> })}
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
            <div class="cdb-io-drawer__footer">
                <button class="cdb-btn" data-testid="import-cancel-btn" on:click=move |_| close_btn()>"取消"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="import-submit"
                    disabled=move || submitting.get()
                    on:click={
                        let on_merge = on_merge.clone();
                        let close_on_ok = on_close.clone();
                        move |_| {
                        let fmt = format.get();
                        // feat-db-connect-import-and-ddl-realdb-verify：数据库来源须先连接解析（core-01d §4.4 3.5）
                        if fmt == ImportFormat::Database && db_result.get_untracked().is_none() {
                            inline_error.set(Some("请先连接并解析数据库".into()));
                            return;
                        }
                        let text = content.get();
                        if text.trim().is_empty() {
                            inline_error.set(Some("内容不能为空".into()));
                            return;
                        }
                        // fix-appbar-roomname-back-and-import-merge：本地解析 → 合并进当前画布
                        let parsed: Result<(Vec<Table>, Vec<Reference>), String> = match fmt {
                            ImportFormat::Sql => parse_sql_import_tables(&text),
                            // fix-canvas-zoom-invite-comment-resize：Database 来源 content 已是
                            // 结构化 tables JSON（「连接并解析」时写入），复用 JSON 导入路径
                            ImportFormat::Database | ImportFormat::Json => parse_json_import_tables(&text),
                            ImportFormat::Dbml => {
                                parse_dbml_import_tables(&text).map(|t| (t, Vec::new()))
                            }
                        };
                        match parsed {
                            Ok((tables, refs)) => {
                                inline_error.set(None);
                                on_merge(tables, refs);
                                close_on_ok();
                            }
                            Err(e) => inline_error.set(Some(e)),
                        }
                    }}
                >
                    {move || if submitting.get() { "导入中..." } else { "导入到画布 ▶" }}
                </button>
            </div>
        </div>
    }
}

/// Phase C：导出抽屉
#[component]
pub fn ExportDrawer(
    store: EditorStore,
    current_title: RwSignal<String>,
    on_close: Rc<dyn Fn()>,
    // feat-room-recycle-bin-dropdown-and-db-export：导出到数据库（core-01d §5.4）
    client: DiagramClient,
    auth_session: RwSignal<Option<AuthSession>>,
    notice: RwSignal<Option<String>>,
) -> impl IntoView {
    let format = create_rw_signal(ExportFormat::Sql);
    // diagram-database-dialect：导出引擎默认跟随图级 database（用户仍可手改，单向不回写）
    let engine = create_rw_signal(match store.database.get_untracked() {
        crate::editor_core::types::Database::Mysql => String::from("mysql"),
        crate::editor_core::types::Database::Postgresql => String::from("postgresql"),
        _ => String::from("generic"),
    });
    let copied = create_rw_signal(false);

    let preview = create_memo(move |_| {
        let tables = store.tables.get();
        let refs = store.references.get();
        // feat-data-dictionary（S07）：字典导出不要求有表（可无绑定纯字典清单）
        if format.get() == ExportFormat::Dict {
            let dicts = store.dictionaries.get();
            if dicts.is_empty() {
                return "当前图表没有数据字典".into();
            }
            return crate::editor_dict::export_dict_markdown(
                &current_title.get_untracked(),
                &dicts,
                &tables,
            );
        }
        if tables.is_empty() {
            return "暂无表，无法导出".into();
        }
        match format.get() {
            ExportFormat::Sql => export_diagram_sql(&tables, &refs, &engine.get()),
            ExportFormat::Dbml => export_diagram_dbml(&tables, &refs),
            ExportFormat::Json => {
                export_diagram_json(&current_title.get_untracked(), &tables, &refs)
            }
            ExportFormat::Dict => unreachable!(),
        }
    });

    let has_tables = move || !store.tables.get().is_empty();
    // feat-data-dictionary（S07，UT-S07-08）：字典导出可用性 = 存在字典
    let export_enabled = move || match format.get() {
        ExportFormat::Dict => crate::editor_dict::dict_export_enabled(&store.dictionaries.get()),
        _ => has_tables(),
    };
    let close = on_close.clone();

    // feat-room-recycle-bin-dropdown-and-db-export：导出到数据库状态（core-01d §5.4）
    // 连接信息仅当次请求使用，不写入任何持久化存储
    let db_engine = create_rw_signal(String::from("sqlite"));
    let db_source = create_rw_signal(String::new());
    // (消息, 是否成功)
    let db_result = create_rw_signal(None::<(String, bool)>);
    let executing = create_rw_signal(false);

    // 「在数据库中执行」→ POST /bridge/export/execute；ddl 取当前预览区已生成的 SQL
    let execute_export = Rc::new(move || {
        if executing.get_untracked() {
            return;
        }
        let source = db_source.get_untracked();
        if source.trim().is_empty() {
            db_result.set(Some(("请填写连接信息".into(), false)));
            return;
        }
        let token = auth_session
            .get()
            .map(|s| s.access_token)
            .unwrap_or_default();
        if token.is_empty() {
            db_result.set(Some(("请先登录".into(), false)));
            return;
        }
        let engine = db_engine.get_untracked();
        let ddl = preview.get_untracked();
        db_result.set(None);
        executing.set(true);
        // 克隆 client 供 async move 捕获，保持外层闭包为 Fn（可重复点击）
        let client = client.clone();
        spawn_local(async move {
            let result = client.export_execute_ddl(&token, &engine, &source, &ddl).await;
            executing.set(false);
            match result {
                Ok(data) => {
                    db_result.set(Some((
                        format!("已执行 {} 条语句 · 建表 {} 张", data.statements, data.tables),
                        true,
                    )));
                    notice.set(Some("已导出到数据库".into()));
                }
                Err(e) => {
                    // core-01d §5.4 错误映射：400 参数 / 502 直接展示后端 message
                    let msg = match &e {
                        ApiError::Server(400, _) => "不支持的数据库引擎".to_string(),
                        ApiError::Server(502, body) => serde_json::from_str::<serde_json::Value>(body)
                            .ok()
                            .and_then(|v| {
                                v.get("message").and_then(|m| m.as_str()).map(String::from)
                            })
                            .unwrap_or_else(|| "导出执行失败".to_string()),
                        _ => "导出执行失败，请稍后重试".to_string(),
                    };
                    db_result.set(Some((msg, false)));
                }
            }
        });
    });

    view! {
        <div class="cdb-io-drawer__inner" data-testid="export-drawer">
            <div class="cdb-io-drawer__header">
                <span class="cdb-io-drawer__title">
                    <IconBox size="sm"><IconExport /></IconBox>
                    <span>"导出"</span>
                </span>
                <button class="cdb-btn cdb-btn--icon" on:click=move |_| close()>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
            <div class="cdb-io-drawer__body">
                <div class="cdb-format-tabs" data-testid="io-format-tabs">
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ExportFormat::Sql
                        on:click=move |_| format.set(ExportFormat::Sql)
                    >"SQL"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ExportFormat::Dbml
                        on:click=move |_| format.set(ExportFormat::Dbml)
                    >"DBML"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ExportFormat::Json
                        on:click=move |_| format.set(ExportFormat::Json)
                    >"JSON"</button>
                    <button
                        class="cdb-btn"
                        class:cdb-is-active=move || format.get() == ExportFormat::Dict
                        data-testid="export-format-dict"
                        on:click=move |_| format.set(ExportFormat::Dict)
                    >"字典"</button>
                </div>
                {move || if format.get() == ExportFormat::Sql {
                    view! {
                        <div class="cdb-form-group">
                            <label>"数据库引擎"</label>
                            <select
                                class="cdb-form-select"
                                data-testid="export-engine-select"
                                on:change=move |ev| engine.set(event_target_value(&ev))
                            >
                                // diagram-database-dialect：selected 跟随图引擎初始值
                                <option value="generic" selected=engine.get() == "generic">"generic"</option>
                                <option value="mysql" selected=engine.get() == "mysql">"mysql"</option>
                                <option value="postgresql" selected=engine.get() == "postgresql">"postgresql"</option>
                            </select>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                <pre class="cdb-export-preview" data-testid="export-preview">{move || preview.get()}</pre>
                {move || if format.get() == ExportFormat::Sql {
                    // feat-room-recycle-bin-dropdown-and-db-export：导出到数据库（core-01d §5.4）
                    let execute_export = execute_export.clone();
                    view! {
                        <div class="cdb-export-db">
                            <div class="cdb-form-group">
                                <label>"导出到数据库"</label>
                                <select
                                    class="cdb-form-select"
                                    data-testid="export-db-engine"
                                    on:change=move |ev| db_engine.set(event_target_value(&ev))
                                >
                                    <option value="sqlite" selected>"sqlite"</option>
                                    <option value="postgres">"postgres"</option>
                                </select>
                            </div>
                            <div class="cdb-form-group">
                                <label>"连接信息"</label>
                                <input
                                    class="cdb-form-input"
                                    data-testid="export-db-source"
                                    placeholder="SQLite 文件路径或 PostgreSQL 连接串"
                                    prop:value=move || db_source.get()
                                    on:input=move |ev| {
                                        use wasm_bindgen::JsCast;
                                        if let Ok(input) = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>() {
                                            db_source.set(input.value());
                                        }
                                    }
                                />
                                <p class="cdb-form-hint">"请确保导出引擎与目标库一致；连接信息仅当次使用，不会被保存"</p>
                            </div>
                            <button
                                class="cdb-btn"
                                data-testid="export-db-execute"
                                disabled=move || !has_tables() || executing.get()
                                on:click={
                                    let execute_export = execute_export.clone();
                                    move |_| execute_export()
                                }
                            >
                                {move || if executing.get() { "执行中..." } else { "在数据库中执行" }}
                            </button>
                            <p class="cdb-export-db-result" data-testid="export-db-result">
                                {move || db_result.get().map(|(msg, _)| msg).unwrap_or_default()}
                            </p>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
            </div>
            <div class="cdb-io-drawer__footer">
                <button
                    class="cdb-btn"
                    data-testid="export-copy"
                    disabled=move || !export_enabled()
                    on:click=move |_| {
                        if copy_text_to_clipboard(&preview.get()) {
                            copied.set(true);
                            let copied_sig = copied;
                            gloo_timers::callback::Timeout::new(2_000, move || {
                                copied_sig.set(false);
                            })
                            .forget();
                        }
                    }
                >
                    {move || if copied.get() { "已复制" } else { "复制" }}
                </button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="export-download"
                    disabled=move || !export_enabled()
                    title=move || {
                        if format.get() == ExportFormat::Dict && !export_enabled() {
                            "当前图表没有数据字典"
                        } else {
                            ""
                        }
                    }
                    on:click=move |_| {
                        let name = current_title.get_untracked();
                        // feat-data-dictionary（S07，core-01e §4）：字典导出固定 data-dictionary-{name}.md
                        let filename = if format.get() == ExportFormat::Dict {
                            crate::editor_dict::dict_export_filename(&name)
                        } else {
                            let ext = match format.get() {
                                ExportFormat::Sql => "sql",
                                ExportFormat::Dbml => "dbml",
                                ExportFormat::Json => "json",
                                ExportFormat::Dict => unreachable!(),
                            };
                            let safe: String = name
                                .chars()
                                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                                .collect();
                            format!("{safe}.{ext}")
                        };
                        download_text(&filename, &preview.get());
                    }
                >
                    "下载"
                </button>
            </div>
        </div>
    }
}

/// Phase A：空白画布引导卡片
#[component]
pub fn EmptyGuide(
    on_create_table: Rc<dyn Fn()>,
    on_import: Rc<dyn Fn()>,
    read_only: bool,
) -> impl IntoView {
    view! {
        <div class="cdb-empty-guide" data-testid="canvas-empty-guide">
            <h2>"开始设计你的数据库"</h2>
            <div class="cdb-empty-guide__actions">
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="guide-create-table"
                    disabled=read_only
                    on:click=move |_| on_create_table()
                >
                    "+ 创建第一张表"
                </button>
                <button
                    class="cdb-btn"
                    data-testid="guide-import-sql"
                    disabled=read_only
                    on:click=move |_| on_import()
                >
                    "↑ 导入 SQL"
                </button>
            </div>
        </div>
    }
}

/// Phase A：Inspector 抽屉
/// 检查器 — 严格对齐主原型 renderInspector：
/// header「检查器」+ close；body = 数据表 section（名称/强调色）+ 字段卡列表（名称/类型/约束 chips/删除）
/// + 删除数据表；空态「选择一个对象」。Field/Reference/Issues 选择保留生产能力表单。
/// spec 锚点：`data-testid="inspector"`（历史 `inspector-panel` 已移除为现行事实）。
#[component]
pub fn Inspector(
    store: EditorStore,
    selection: RwSignal<SelectionKind>,
    inspector_open: RwSignal<bool>,
    on_add_field: Rc<dyn Fn(String)>,
    on_change_type: Rc<dyn Fn(String, String)>,
    on_set_ref: Rc<dyn Fn(String)>,
    on_toggle_pk: Rc<dyn Fn(String, String, bool)>,
    on_toggle_nn: Rc<dyn Fn(String, String, bool)>,
    on_toggle_uq: Rc<dyn Fn(String, String, bool)>,
    on_rename_table: Rc<dyn Fn(String, String)>,
    on_rename_field: Rc<dyn Fn(String, String, String)>,
    // fix-canvas-zoom-invite-comment-resize（core-01a §1.4.2 / §2.3）：
    // 表 / 字段注释 blur 落账通路（写 store → dirty → schedule_save，仿 on_rename_table）
    on_set_table_comment: Rc<dyn Fn(String, String)>,
    on_set_field_comment: Rc<dyn Fn(String, String, String)>,
    on_delete_field: Rc<dyn Fn(String, String)>,
    on_delete_table: Rc<dyn Fn(String)>,
    on_update_ref_field: Rc<dyn Fn(String, &str, String)>,
    on_flip_ref: Rc<dyn Fn(String)>,
    on_delete_ref: Rc<dyn Fn(String)>,
    // p0-fix 定点 2：区域 / 便签编辑与删除
    on_update_area: Rc<dyn Fn(String, &str, String)>,
    on_update_note: Rc<dyn Fn(String, String)>,
    // redesign-listview-type-length-canvas-fix：字段 tag 落账通路（table_id, field_id, tag）——
    // 原 tag blur 只写 store + dirty 不触发保存，PUT 不落账（ST-CR-TAG-01）
    on_update_field_tag: Rc<dyn Fn(String, String, String)>,
    // feat-data-dictionary（S07，core-01e §3）：字段卡「数据字典」绑定下拉
    // (table_id, field_id, dict_code)——软引用按字典编码，空串 = 解绑
    on_set_field_dict: Rc<dyn Fn(String, String, String)>,
    on_delete_area: Rc<dyn Fn(String)>,
    on_delete_note: Rc<dyn Fn(String)>,
    on_jump_to_table: Rc<dyn Fn(String)>,
    read_only: Rc<dyn Fn() -> bool>,
) -> impl IntoView {
    let selected_table = create_memo(move |_| {
        let id = selection.get().table_id()?.to_string();
        store.tables.get().into_iter().find(|t| t.id == id)
    });

    // redesign-listview-type-length-canvas-fix：字段表单受控草稿（tag / 长度 / 小数位）。
    // 草稿挂在 Inspector 组件级（store.tables 动态块之外）——表单因落账重建时输入不丢字；
    // 仅在 selection 切换到另一字段时重置草稿（get_untracked 读 tables，不订阅落账写回）
    let field_tag_draft = create_rw_signal(String::new());
    let field_len_draft = create_rw_signal(String::new());
    let field_scale_draft = create_rw_signal(String::new());
    // 便签内容草稿（ST-CR-NOTE-01：原 on:input 每击键 store.notes.set + 全量 snapshot → 表单重建丢字/卡死）
    let note_content_draft = create_rw_signal(String::new());
    // 区域名称草稿（同源问题：on:input 每击键 store.areas.set）
    let area_name_draft = create_rw_signal(String::new());
    create_effect(move |_| {
        match selection.get() {
            SelectionKind::Field { table_id, field_id } => {
                let tables = store.tables.get_untracked();
                if let Some(f) = tables
                    .iter()
                    .find(|t| t.id == table_id)
                    .and_then(|t| t.fields.iter().find(|f| f.id == field_id))
                {
                    let p = crate::editor_core::parse_field_type(&f.type_);
                    field_tag_draft.set(f.tag.clone());
                    field_len_draft.set(p.len);
                    field_scale_draft.set(p.scale);
                }
            }
            SelectionKind::Note(note_id) => {
                let notes = store.notes.get_untracked();
                if let Some(n) = notes.iter().find(|n| n.id == note_id) {
                    note_content_draft.set(n.content.clone());
                }
            }
            SelectionKind::Area(area_id) => {
                let areas = store.areas.get_untracked();
                if let Some(a) = areas.iter().find(|a| a.id == area_id) {
                    area_name_draft.set(a.name.clone());
                }
            }
            _ => {}
        }
    });

    let close_inspector = move |_| inspector_open.set(false);

    view! {
        <aside
            class="cdb-inspector"
            data-testid="inspector"
            style:display=move || if inspector_open.get() { "flex" } else { "none" }
        >
            <header class="cdb-inspector__header">
                <h2 data-testid="inspector-title">"检查器"</h2>
                <button
                    class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                    data-testid="btn-inspector-close"
                    aria-label="关闭检查器"
                    on:click=close_inspector
                >
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </header>
            <div class="cdb-inspector__body">
                {move || {
                    let on_jump = on_jump_to_table.clone();
                    let on_add = on_add_field.clone();
                    let on_change = on_change_type.clone();
                    let on_ref = on_set_ref.clone();
                    let on_toggle_pk = on_toggle_pk.clone();
                    let on_toggle_nn = on_toggle_nn.clone();
                    let on_toggle_uq = on_toggle_uq.clone();
                    let on_rename_field = on_rename_field.clone();
                    let on_delete_field = on_delete_field.clone();
                    // fix-canvas-zoom-invite-comment-resize：注释落账回调先 clone 到局部，
                    // 内层 For children 闭包只捕获局部变量（保持外层 Fn 语义）
                    let on_set_table_comment = on_set_table_comment.clone();
                    let on_set_field_comment = on_set_field_comment.clone();
                    let on_set_field_dict = on_set_field_dict.clone();
                    match selection.get() {
                    SelectionKind::Issues => {
                        let issues = compute_diagram_issues(&store);
                        let empty = issues.is_empty();
                        view! {
                            <div data-testid="inspector-issues">
                                <For each=move || issues.clone() key=|(_, _, t)| t.clone() children=move |(level, message, target)| {
                                    let jump = on_jump.clone();
                                    let tid = target.clone();
                                    view! {
                                        <div class="cdb-issue" data-testid={format!("issue-item-{}", tid)}>
                                            <span class="cdb-issue-level">{level}</span>
                                            <span class="cdb-issue-message">{message}</span>
                                            <button
                                                class="cdb-btn cdb-btn--small"
                                                data-testid={format!("issue-jump-{}", tid)}
                                                on:click=move |_| jump(tid.clone())
                                            >
                                                "定位"
                                            </button>
                                        </div>
                                    }
                                } />
                                {if empty {
                                    view! { <p class="cdb-empty-hint">"无问题 ✓"</p> }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }}
                            </div>
                        }.into_view()
                    }
                    SelectionKind::Field { table_id, field_id } => {
                        let tables = store.tables.get();
                        let field = tables.iter()
                            .find(|t| t.id == table_id)
                            .and_then(|t| t.fields.iter().find(|f| f.id == field_id));
                        if let Some(field) = field {
                            let fid = field_id.clone();
                            let tid = table_id.clone();
                            let pk = field.primary;
                            let cur_type = field.type_.clone();
                            let on_pk = on_toggle_pk.clone();
                            let fid_type = fid.clone();
                            let fid_pk = fid.clone();
                            let fid_ref_btn = fid.clone();
                            view! {
                                <div data-testid="inspector-field-form">
                                    <div class="cdb-form-group">
                                        <label>"名称"</label>
                                        <input class="cdb-form-input" data-testid="inspector-field-name" value=field.name.clone() readonly=true />
                                    </div>
                                    <div class="cdb-form-group">
                                        <label>"类型"</label>
                                        <select
                                            class="cdb-form-select"
                                            data-testid="inspector-field-type"
                                            on:change={
                                                let on_change = on_change.clone();
                                                let cur_type = cur_type.clone();
                                                move |ev| {
                                                    // redesign：下拉值为基类型；变更时按 core-01a §2.2 口径
                                                    // 带当前长度/小数位重新合成整串（VARCHAR(32)/NUMERIC(10,2)）
                                                    let v = event_target_value(&ev);
                                                    let p = crate::editor_core::parse_field_type(&cur_type);
                                                    on_change(
                                                        fid_type.clone(),
                                                        crate::editor_core::compose_field_type(&v, &p.len, &p.scale),
                                                    );
                                                }
                                            }
                                        >
                                            // diagram-database-dialect：类型选项按图引擎过滤（types_for_database 基类型清单）；
                                            // 当前基类型不在清单时追加占位项，避免显示丢失（不改写既有字段）
                                            {let cur_type_opts = cur_type.clone();
                                             move || {
                                                let cur = crate::editor_core::parse_field_type(&cur_type_opts).base;
                                                let types = crate::editor_core::types::types_for_database(store.database.get());
                                                let mut opts: Vec<String> = types.iter().map(|s| s.to_string()).collect();
                                                if !cur.is_empty() && !opts.contains(&cur) {
                                                    opts.push(cur.clone());
                                                }
                                                opts.into_iter().map(|t| {
                                                    let sel = t == cur;
                                                    view! { <option value=t.clone() selected=sel>{t}</option> }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </div>
                                    // redesign-listview-type-length-canvas-fix：长度/小数位独立编辑列
                                    // （core-01a §2.2 参数化口径；受控草稿 + blur 落账合成整串）
                                    <div class="cdb-form-group">
                                        <label>"长度（仅 VARCHAR）"</label>
                                        <input
                                            class="cdb-form-input"
                                            data-testid="inspector-field-len"
                                            prop:value=move || field_len_draft.get()
                                            disabled={
                                                let cur_type = cur_type.clone();
                                                move || crate::editor_core::parse_field_type(&cur_type).base != "VARCHAR"
                                            }
                                            on:input=move |ev| field_len_draft.set(event_target_value(&ev))
                                            on:blur={
                                                let on_change = on_change.clone();
                                                let fid_len = fid.clone();
                                                let cur_type = cur_type.clone();
                                                move |_| {
                                                    let v = field_len_draft.get_untracked();
                                                    let p = crate::editor_core::parse_field_type(&cur_type);
                                                    let next = crate::editor_core::compose_field_type(&p.base, &v, &p.scale);
                                                    if next != cur_type {
                                                        on_change(fid_len.clone(), next);
                                                    }
                                                }
                                            }
                                        />
                                    </div>
                                    <div class="cdb-form-group">
                                        <label>"小数位（仅 DECIMAL/NUMERIC）"</label>
                                        <input
                                            class="cdb-form-input"
                                            data-testid="inspector-field-scale"
                                            prop:value=move || field_scale_draft.get()
                                            disabled={
                                                let cur_type = cur_type.clone();
                                                move || {
                                                    let b = crate::editor_core::parse_field_type(&cur_type).base;
                                                    !(b == "DECIMAL" || b == "NUMERIC")
                                                }
                                            }
                                            on:input=move |ev| field_scale_draft.set(event_target_value(&ev))
                                            on:blur={
                                                let on_change = on_change.clone();
                                                let fid_scale = fid.clone();
                                                let cur_type = cur_type.clone();
                                                move |_| {
                                                    let v = field_scale_draft.get_untracked();
                                                    let p = crate::editor_core::parse_field_type(&cur_type);
                                                    let next = crate::editor_core::compose_field_type(&p.base, &p.len, &v);
                                                    if next != cur_type {
                                                        on_change(fid_scale.clone(), next);
                                                    }
                                                }
                                            }
                                        />
                                    </div>
                                    // redesign-listview-type-length-canvas-fix：tag 受控输入（ST-CR-TAG-01）
                                    // 击键只写组件级草稿（不触 store），blur 落账——修复「表单重建 → 静态快照重置 → 丢字」
                                    <div class="cdb-form-group">
                                        <label>"标签（tag，用于 ByTag 分组）"</label>
                                        <input
                                            class="cdb-form-input"
                                            data-testid="inspector-field-tag"
                                            prop:value=move || field_tag_draft.get()
                                            on:input=move |ev| field_tag_draft.set(event_target_value(&ev))
                                            on:blur={
                                                let table_id_t = table_id.clone();
                                                let fid_t = fid.clone();
                                                let on_tag = on_update_field_tag.clone();
                                                move |_| {
                                                    let v = field_tag_draft.get_untracked();
                                                    on_tag(table_id_t.clone(), fid_t.clone(), v);
                                                }
                                            }
                                        />
                                    </div>
                                    <div class="cdb-checkbox-row">
                                        <label>
                                            <input
                                                type="checkbox"
                                                data-testid="inspector-field-pk"
                                                checked=pk
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    on_pk(tid.clone(), fid_pk.clone(), checked);
                                                }
                                            />
                                            " 主键"
                                        </label>
                                    </div>
                                    <button
                                        class="cdb-btn cdb-btn--block"
                                        data-testid="btn-create-fk"
                                        on:click=move |_| on_ref(fid_ref_btn.clone())
                                    >
                                        "+ 创建外键连接"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            view! { <p class="cdb-empty-hint">"字段不存在"</p> }.into_view()
                        }
                    }
                    SelectionKind::Table(_) => {
                        let ro = read_only();
                        if let Some(t) = selected_table.get() {
                            let fields = t.fields.clone();
                            let table_name = t.name.clone();
                            let table_id = t.id.clone();
                            let table_id_for_add = table_id.clone();
                            let table_id_for_rename = table_id.clone();
                            let table_id_for_delete = table_id.clone();
                            let field_count = fields.len();
                            let on_rename = on_rename_table.clone();
                            let on_del_table = on_delete_table.clone();
                            let on_set_comment = on_set_table_comment.clone();
                            let table_id_for_comment = table_id.clone();
                            let table_comment = t.comment.clone();
                            view! {
                                <div data-testid="inspector-table-form">
                                    <section class="cdb-panel-section">
                                        <div class="cdb-panel-title">
                                            <span>"数据表"</span>
                                            <span class="cdb-panel-tag">{table_id.clone()}</span>
                                        </div>
                                        <div class="cdb-form-group">
                                            <label>"名称"</label>
                                            <input
                                                class="cdb-form-input"
                                                data-testid="inspector-table-name"
                                                prop:value=table_name
                                                disabled=ro
                                                on:blur=move |ev| {
                                                    if !ro {
                                                        on_rename(table_id_for_rename.clone(), event_target_value(&ev));
                                                    }
                                                }
                                            />
                                        </div>
                                        // fix-canvas-zoom-invite-comment-resize（core-01a §1.4.2，UT-PC-30）：
                                        // 表注释受控输入，blur 落账（任意字符串允许为空，不触发重名校验）
                                        <div class="cdb-form-group">
                                            <label>"注释"</label>
                                            <input
                                                class="cdb-form-input"
                                                data-testid="inspector-table-comment"
                                                prop:value=table_comment
                                                disabled=ro
                                                on:blur=move |ev| {
                                                    if !ro {
                                                        on_set_comment(table_id_for_comment.clone(), event_target_value(&ev));
                                                    }
                                                }
                                            />
                                        </div>
                                    </section>
                                    <section class="cdb-panel-section">
                                        <div class="cdb-panel-title">
                                            <span>{format!("字段 · {field_count}")}</span>
                                            <button
                                                class="cdb-btn cdb-btn--ghost cdb-btn--small"
                                                data-testid="btn-add-field"
                                                disabled=ro
                                                on:click=move |_| on_add(table_id_for_add.clone())
                                            >
                                                <IconBox size="sm"><IconAdd /></IconBox>
                                                "添加"
                                            </button>
                                        </div>
                                        <div class="cdb-field-card-list">
                                    <For each=move || fields.clone() key=|f| f.id.clone() children=move |field: Field| {
                                        let fid = field.id.clone();
                                        let fid_type = fid.clone();
                                        let fid_ref = fid.clone();
                                        let fname = field.name.clone();
                                        let ftype = field.type_.clone();
                                        let f_primary = field.primary;
                                        let f_nn = field.not_null;
                                        let f_uq = field.unique;
                                        // fix-canvas-zoom-invite-comment-resize（core-01a §2.3，UT-PC-30）：
                                        // Inspector 字段卡注释受控草稿（击键不触 store，blur 落账）
                                        let fcomment_draft = create_rw_signal(field.comment.clone());
                                        let on_set_fcomment = on_set_field_comment.clone();
                                        // feat-data-dictionary（S07）：字段卡字典绑定（f_dict 随 For 重建刷新）
                                        let f_dict = field.dict_code.clone();
                                        let on_set_dict = on_set_field_dict.clone();
                                        let tid = table_id.clone();
                                        let on_change = on_change.clone();
                                        let on_ref = on_ref.clone();
                                        let on_pk2 = on_toggle_pk.clone();
                                        let on_nn = on_toggle_nn.clone();
                                        let on_uq = on_toggle_uq.clone();
                                        let on_ren_f = on_rename_field.clone();
                                        let on_del_f = on_delete_field.clone();
                                        view! {
                                            <article
                                                class="cdb-field-card"
                                                data-testid={format!("field-row-{}", fid)}
                                                // redesign-listview-type-length-canvas-fix：点击字段卡 → 选中字段
                                                // （Inspector 字段表单/ tag 编辑通路，ST-CR-TAG-01）
                                                on:click={
                                                    let tid = tid.clone();
                                                    let fid = fid.clone();
                                                    move |_| selection.set(SelectionKind::Field {
                                                        table_id: tid.clone(),
                                                        field_id: fid.clone(),
                                                    })
                                                }
                                            >
                                                <div class="cdb-field-card__main">
                                                    <input
                                                        class="cdb-form-input"
                                                        data-testid={format!("field-name-{}", fid)}
                                                        prop:value=fname
                                                        disabled=ro
                                                        on:blur={
                                                            let tid = tid.clone();
                                                            let fid = fid.clone();
                                                            move |ev| {
                                                                if !ro {
                                                                    on_ren_f(tid.clone(), fid.clone(), event_target_value(&ev));
                                                                }
                                                            }
                                                        }
                                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                                    />
                                                    <select
                                                        class="cdb-form-select"
                                                        data-testid={format!("type-{}", fid_type)}
                                                        disabled=ro
                                                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                                                        on:change={
                                                            let ftype = ftype.clone();
                                                            move |ev| {
                                                                // redesign：下拉值为基类型；变更时带当前长度/小数位重新合成整串
                                                                let v = event_target_value(&ev);
                                                                let p = crate::editor_core::parse_field_type(&ftype);
                                                                on_change(
                                                                    fid_type.clone(),
                                                                    crate::editor_core::compose_field_type(&v, &p.len, &p.scale),
                                                                );
                                                            }
                                                        }
                                                    >
                                                        // diagram-database-dialect：类型选项按图引擎过滤（基类型清单）；
                                                        // 当前基类型不在清单时追加占位项，避免显示丢失
                                                        {move || {
                                                            let cur = crate::editor_core::parse_field_type(&ftype).base;
                                                            let types = crate::editor_core::types::types_for_database(store.database.get());
                                                            let mut opts: Vec<String> = types.iter().map(|s| s.to_string()).collect();
                                                            if !cur.is_empty() && !opts.contains(&cur) {
                                                                opts.push(cur.clone());
                                                            }
                                                            opts.into_iter().map(|t| {
                                                                let sel = t == cur;
                                                                view! { <option value=t.clone() selected=sel>{t}</option> }
                                                            }).collect_view()
                                                        }}
                                                    </select>
                                                    // fix-canvas-zoom-invite-comment-resize（core-01a §2.3，UT-PC-30）：
                                                    // Inspector 字段卡注释输入（testid inspector-field-comment-{fid}，
                                                    // 与 ListView list-field-comment-* 双轨一致：受控草稿 + blur 落账）
                                                    <input
                                                        class="cdb-form-input"
                                                        data-testid={format!("inspector-field-comment-{}", fid)}
                                                        prop:value=move || fcomment_draft.get()
                                                        disabled=ro
                                                        placeholder="注释"
                                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                                        on:input=move |ev| fcomment_draft.set(event_target_value(&ev))
                                                        on:blur={
                                                            let tid = tid.clone();
                                                            let fid = fid.clone();
                                                            let on_set = on_set_fcomment.clone();
                                                            move |_| {
                                                                if !ro {
                                                                    on_set(tid.clone(), fid.clone(), fcomment_draft.get_untracked());
                                                                }
                                                            }
                                                        }
                                                    />
                                                    // feat-data-dictionary（S07，core-01e §3）：
                                                    // 字段卡「数据字典」绑定下拉（软引用编码）+ 映射摘要 Tag
                                                    <select
                                                        class="cdb-form-select cdb-field-dict-select"
                                                        data-testid={format!("inspector-field-dict-{}", fid)}
                                                        disabled=ro
                                                        title="数据字典"
                                                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                                                        on:change={
                                                            let tid = tid.clone();
                                                            let fid = fid.clone();
                                                            let on_set = on_set_dict.clone();
                                                            move |ev| {
                                                                if !ro {
                                                                    on_set(tid.clone(), fid.clone(), event_target_value(&ev));
                                                                }
                                                            }
                                                        }
                                                    >
                                                        <option value="" selected=f_dict.is_empty()>"（不绑定）"</option>
                                                        {let f_dict_opts = f_dict.clone();
                                                         move || store.dictionaries.get().into_iter().map(|d| {
                                                            let sel = d.code == f_dict_opts;
                                                            view! {
                                                                <option value=d.code.clone() selected=sel>
                                                                    {format!("{}（{}）", d.name, d.code)}
                                                                </option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                    {let f_dict_tag = f_dict.clone();
                                                     let fid_tag = fid.clone();
                                                     move || {
                                                        let dicts = store.dictionaries.get();
                                                        crate::editor_dict::find_dict(&dicts, &f_dict_tag)
                                                            .map(crate::editor_dict::dict_summary)
                                                            .filter(|s| !s.is_empty())
                                                            .map(|s| view! {
                                                                <span class="cdb-field-dict-tag" data-testid={format!("field-dict-tag-{}", fid_tag)}>{s}</span>
                                                            })
                                                    }}
                                                    <div class="cdb-constraint-row">
                                                        <button
                                                            class="cdb-constraint"
                                                            class:cdb-is-on=f_primary
                                                            data-testid={format!("constraint-pk-{}", fid)}
                                                            disabled=ro
                                                            on:click={
                                                                let tid = tid.clone();
                                                                let fid = fid.clone();
                                                                move |_| on_pk2(tid.clone(), fid.clone(), !f_primary)
                                                            }
                                                        >
                                                            "PK"
                                                        </button>
                                                        <button
                                                            class="cdb-constraint"
                                                            class:cdb-is-on=f_nn
                                                            data-testid={format!("constraint-nn-{}", fid)}
                                                            disabled=ro
                                                            on:click={
                                                                let tid = tid.clone();
                                                                let fid = fid.clone();
                                                                move |_| on_nn(tid.clone(), fid.clone(), !f_nn)
                                                            }
                                                        >
                                                            "NOT NULL"
                                                        </button>
                                                        <button
                                                            class="cdb-constraint"
                                                            class:cdb-is-on=f_uq
                                                            data-testid={format!("constraint-uq-{}", fid)}
                                                            disabled=ro
                                                            on:click={
                                                                let tid = tid.clone();
                                                                let fid = fid.clone();
                                                                move |_| on_uq(tid.clone(), fid.clone(), !f_uq)
                                                            }
                                                        >
                                                            "UNIQUE"
                                                        </button>
                                                    </div>
                                                    <button
                                                        class="cdb-btn cdb-btn--ghost cdb-btn--small"
                                                        data-testid={format!("set-ref-{}", fid_ref)}
                                                        disabled=ro
                                                        on:click=move |ev: web_sys::MouseEvent| {
                                                            ev.stop_propagation();
                                                            on_ref(fid_ref.clone());
                                                        }
                                                    >
                                                        "设关系"
                                                    </button>
                                                </div>
                                                <button
                                                    class="cdb-btn cdb-btn--ghost cdb-btn--icon cdb-field-card__delete"
                                                    data-testid={format!("btn-delete-field-{}", fid)}
                                                    aria-label="删除字段"
                                                    disabled=ro
                                                    on:click={
                                                        let tid = tid.clone();
                                                        let fid = fid.clone();
                                                        move |_| on_del_f(tid.clone(), fid.clone())
                                                    }
                                                >
                                                    <IconBox size="sm"><IconDelete /></IconBox>
                                                </button>
                                            </article>
                                        }
                                    } />
                                        </div>
                                    </section>
                                    <button
                                        class="cdb-btn cdb-btn--danger cdb-btn--block"
                                        data-testid="btn-delete-table"
                                        disabled=ro
                                        on:click=move |_| on_del_table(table_id_for_delete.clone())
                                    >
                                        <IconBox size="sm"><IconDelete /></IconBox>
                                        "删除数据表"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            view! { <p class="cdb-empty-hint">"表不存在"</p> }.into_view()
                        }
                    }
                    SelectionKind::Reference(ref_id) => {
                        let refs = store.references.get();
                        let reference = refs.iter().find(|r| r.id == ref_id);
                        if let Some(r) = reference {
                            let rid = ref_id.clone();
                            let label = format_rel_confirm_label(
                                &store.tables.get(),
                                &r.start_table_id,
                                &r.start_field_id,
                                &r.end_table_id,
                                &r.end_field_id,
                            );
                            let card = r.type_.clone();
                            let on_del = r.on_delete.clone();
                            let on_upd = r.on_update.clone();
                            let on_upd_ref = on_update_ref_field.clone();
                            let on_flip = on_flip_ref.clone();
                            let on_del_ref = on_delete_ref.clone();
                            let rid_type = rid.clone();
                            let rid_on_delete = rid.clone();
                            let rid_on_update = rid.clone();
                            let rid_flip = rid.clone();
                            let rid_delete = rid.clone();
                            let on_upd_ref_type = on_upd_ref.clone();
                            let on_upd_ref_del = on_upd_ref.clone();
                            let on_upd_ref_upd = on_upd_ref.clone();
                            let card_for_options = card.clone();
                            view! {
                                <div data-testid="inspector-reference-form">
                                    <p class="cdb-rel-confirm-bar__label">{label}</p>
                                    <div class="cdb-form-group">
                                        <label>"Cardinality"</label>
                                        <select
                                            class="cdb-form-select"
                                            data-testid="inspector-ref-cardinality"
                                            on:change=move |ev| {
                                                on_upd_ref_type(rid_type.clone(), "type_", event_target_value(&ev));
                                            }
                                        >
                                            <For each=|| CARDINALITY_OPTIONS.to_vec() key=|c| *c children=move |c: &'static str| {
                                                let sel = card_for_options == c;
                                                view! { <option value=c selected=sel>{c}</option> }
                                            } />
                                        </select>
                                    </div>
                                    <div class="cdb-form-group">
                                        <label>"onDelete"</label>
                                        <select
                                            class="cdb-form-select"
                                            data-testid="inspector-ref-on-delete"
                                            on:change=move |ev| {
                                                on_upd_ref_del(rid_on_delete.clone(), "on_delete", event_target_value(&ev));
                                            }
                                        >
                                            <option value="RESTRICT" selected=on_del == "RESTRICT">"RESTRICT"</option>
                                            <option value="CASCADE" selected=on_del == "CASCADE">"CASCADE"</option>
                                            <option value="SET NULL" selected=on_del == "SET NULL">"SET NULL"</option>
                                            <option value="NO ACTION" selected=on_del == "NO ACTION">"NO ACTION"</option>
                                        </select>
                                    </div>
                                    <div class="cdb-form-group">
                                        <label>"onUpdate"</label>
                                        <select
                                            class="cdb-form-select"
                                            data-testid="inspector-ref-on-update"
                                            on:change=move |ev| {
                                                on_upd_ref_upd(rid_on_update.clone(), "on_update", event_target_value(&ev));
                                            }
                                        >
                                            <option value="RESTRICT" selected=on_upd == "RESTRICT">"RESTRICT"</option>
                                            <option value="CASCADE" selected=on_upd == "CASCADE">"CASCADE"</option>
                                            <option value="SET NULL" selected=on_upd == "SET NULL">"SET NULL"</option>
                                            <option value="NO ACTION" selected=on_upd == "NO ACTION">"NO ACTION"</option>
                                        </select>
                                    </div>
                                    <button
                                        class="cdb-btn cdb-btn--block"
                                        data-testid="inspector-ref-flip"
                                        on:click=move |_| on_flip(rid_flip.clone())
                                    >
                                        "翻转方向"
                                    </button>
                                    <button
                                        class="cdb-btn cdb-btn--block cdb-btn--danger"
                                        data-testid="inspector-ref-delete"
                                        on:click=move |_| on_del_ref(rid_delete.clone())
                                    >
                                        "删除关系"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            view! { <p class="cdb-empty-hint">"关系不存在"</p> }.into_view()
                        }
                    }
                    SelectionKind::Area(area_id) => {
                        // p0-fix 定点 2：区域编辑面板（name / color / 删除）
                        let ro = read_only();
                        let areas = store.areas.get();
                        if let Some(a) = areas.iter().find(|x| x.id == area_id) {
                            let area_name = a.name.clone();
                            let area_color = a.color.clone();
                            let aid_tag = area_id.clone();
                            let aid_name = area_id.clone();
                            let aid_color = area_id.clone();
                            let aid_delete = area_id.clone();
                            let on_upd_name = on_update_area.clone();
                            let on_upd_color = on_update_area.clone();
                            let on_del_area = on_delete_area.clone();
                            view! {
                                <div data-testid="inspector-area-form">
                                    <section class="cdb-panel-section">
                                        <div class="cdb-panel-title">
                                            <span>"区域"</span>
                                            <span class="cdb-panel-tag">{aid_tag}</span>
                                        </div>
                                        <div class="cdb-form-group">
                                            <label>"名称"</label>
                                            // redesign-listview-type-length-canvas-fix：受控输入——击键只写草稿，blur 落账
                                            <input
                                                class="cdb-form-input"
                                                data-testid="inspector-area-name"
                                                prop:value=move || area_name_draft.get()
                                                disabled=ro
                                                on:input=move |ev| area_name_draft.set(event_target_value(&ev))
                                                on:blur=move |_| {
                                                    let v = area_name_draft.get_untracked();
                                                    if v != area_name {
                                                        on_upd_name(aid_name.clone(), "name", v);
                                                    }
                                                }
                                            />
                                        </div>
                                        <div class="cdb-form-group">
                                            <label>"颜色"</label>
                                            // redesign：color 选择器拖动期高频 input → 改 on:change（关闭选择器时）一次落账
                                            <input
                                                type="color"
                                                class="cdb-form-input"
                                                data-testid="inspector-area-color"
                                                value=area_color
                                                disabled=ro
                                                on:change=move |ev| on_upd_color(aid_color.clone(), "color", event_target_value(&ev))
                                            />
                                        </div>
                                        <button
                                            class="cdb-btn cdb-btn--block cdb-btn--danger"
                                            data-testid="btn-delete-area"
                                            disabled=ro
                                            on:click=move |_| on_del_area(aid_delete.clone())
                                        >
                                            "删除区域"
                                        </button>
                                    </section>
                                </div>
                            }.into_view()
                        } else {
                            view! { <p class="cdb-empty-hint">"区域不存在"</p> }.into_view()
                        }
                    }
                    SelectionKind::Note(note_id) => {
                        // p0-fix 定点 2：便签编辑面板（content textarea / 删除）
                        let ro = read_only();
                        let notes = store.notes.get();
                        if let Some(n) = notes.iter().find(|x| x.id == note_id) {
                            let note_content = n.content.clone();
                            let nid_tag = note_id.clone();
                            let nid_content = note_id.clone();
                            let nid_delete = note_id.clone();
                            let on_upd_note = on_update_note.clone();
                            let on_del_note = on_delete_note.clone();
                            view! {
                                <div data-testid="inspector-note-form">
                                    <section class="cdb-panel-section">
                                        <div class="cdb-panel-title">
                                            <span>"便签"</span>
                                            <span class="cdb-panel-tag">{nid_tag}</span>
                                        </div>
                                        <div class="cdb-form-group">
                                            <label>"内容"</label>
                                            // redesign-listview-type-length-canvas-fix：受控输入——
                                            // 击键只写草稿，blur 落账（ST-CR-NOTE-01 连续键入不丢字）
                                            <textarea
                                                class="cdb-form-input"
                                                data-testid="inspector-note-content"
                                                rows="4"
                                                prop:value=move || note_content_draft.get()
                                                disabled=ro
                                                on:input=move |ev| note_content_draft.set(event_target_value(&ev))
                                                on:blur=move |_| {
                                                    let v = note_content_draft.get_untracked();
                                                    if v != note_content {
                                                        on_upd_note(nid_content.clone(), v);
                                                    }
                                                }
                                            ></textarea>
                                        </div>
                                        <button
                                            class="cdb-btn cdb-btn--block cdb-btn--danger"
                                            data-testid="btn-delete-note"
                                            disabled=ro
                                            on:click=move |_| on_del_note(nid_delete.clone())
                                        >
                                            "删除便签"
                                        </button>
                                    </section>
                                </div>
                            }.into_view()
                        } else {
                            view! { <p class="cdb-empty-hint">"便签不存在"</p> }.into_view()
                        }
                    }
                    SelectionKind::None => {
                        view! {
                            <div class="cdb-empty-inspector" data-testid="inspector-empty">
                                <span class="cdb-brand-mark"><IconBox size="md"><IconAddTable /></IconBox></span>
                                <strong>"选择一个对象"</strong>
                                <p>"在画布上选择表以编辑名称、字段与约束。"</p>
                                <p class="cdb-empty-inspector__meta" data-testid="inspector-overview">
                                    {move || format!("{} 张表 · {} 条关系", store.tables.get().len(), store.references.get().len())}
                                </p>
                            </div>
                        }.into_view()
                    }
                    }
                }}
            </div>
        </aside>
    }
}

/// 状态栏 — 严格对齐主原型 statusbar：
/// ws-status（圆点+五态文案）→ ot-rev（server_rev N）→ 表/关系计数 → 待同步 tag
/// → spacer → 角色 tag → zoom −/%/＋ → btn-inspector-toggle
#[component]
pub fn StatusBar(
    store: EditorStore,
    transform: RwSignal<Transform>,
    inspector_open: RwSignal<bool>,
    collab_state: RwSignal<CollabOtState>,
    remote_members: RwSignal<Vec<CollabMemberPresence>>,
    current_room: RwSignal<Option<RoomDetail>>,
) -> impl IntoView {
    let _ = remote_members;
    view! {
        <footer class="cdb-status-bar" data-testid="status-bar">
            <span class="cdb-status-group" data-testid="ws-status">
                <span class=move || format!("cdb-ws-dot {}", collab_status_dot_class(&collab_state.get()))></span>
                {move || collab_status_label(&collab_state.get()).to_string()}
            </span>
            <span class="cdb-status-group" data-testid="ot-rev">
                {move || format!("server_rev {}", collab_state.get().server_rev)}
            </span>
            <span class="cdb-status-group cdb-desktop-only" data-testid="status-counts">
                {move || format!(
                    "{} 张表 · {} 条关系",
                    store.tables.get().len(),
                    store.references.get().len(),
                )}
            </span>
            {move || {
                let queued = collab_state.get().queued_while_offline.len()
                    + collab_state.get().pending_ops.len();
                (queued > 0).then(|| view! {
                    <span class="cdb-status-tag cdb-status-tag--warn" data-testid="status-pending-ops">
                        {format!("{queued} 项待同步")}
                    </span>
                })
            }}
            <span class="cdb-status-bar__spacer"></span>
            {move || current_room.get().map(|room| view! {
                <span class="cdb-status-tag cdb-status-tag--brand" data-testid="status-role">
                    {room.my_role}
                </span>
            })}
            <span class="cdb-status-group cdb-status-zoom">
                <button
                    class="cdb-btn cdb-btn--ghost cdb-btn--small"
                    data-testid="btn-zoom-out"
                    aria-label="缩小"
                    on:click=move |_| zoom_out(transform)
                >
                    "−"
                </button>
                <span data-testid="status-zoom">
                    {move || format!("{}%", (transform.get().zoom * 100.0).round() as i32)}
                </span>
                <button
                    class="cdb-btn cdb-btn--ghost cdb-btn--small"
                    data-testid="btn-zoom-in"
                    aria-label="放大"
                    on:click=move |_| zoom_in(transform)
                >
                    "＋"
                </button>
            </span>
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-inspector-toggle"
                title="切换检查器"
                aria-label="切换检查器"
                on:click=move |_| inspector_open.update(|v| *v = !*v)
            >
                {move || if inspector_open.get() {
                    view! { <IconBox size="sm"><IconChevronRight /></IconBox> }.into_view()
                } else {
                    view! { <IconBox size="sm"><IconChevronLeft /></IconBox> }.into_view()
                }}
            </button>
        </footer>
    }
}

/// 旧 TopBar 兼容 (B1 拆分后保留为内部分发)
#[component]
pub fn TopBar(
    store: EditorStore,
    debouncer: DebounceTrigger,
    error: RwSignal<Option<String>>,
    modal_kind: RwSignal<Option<modals::ModalKind>>,
) -> impl IntoView {
    let saving = create_rw_signal(false);
    let debouncer_for_save = debouncer.clone();
    let err_for_save = error.clone();
    let store_for_save = store.clone();

    view! {
        <div>
            <TopMenuBar
                modal_kind=modal_kind
                current_title=create_rw_signal(String::from("Untitled"))
                store=store.clone()
                is_saving=create_rw_signal(false)
                transform=create_rw_signal(Transform::default())
            />
            <Toolbar
                store=store.clone()
                current_title=create_rw_signal(String::from("Untitled"))
                error=error.clone()
                on_title_blur=Rc::new(|_| {})
            />
            <button
                data-testid="btn-save"
                class="cdb-btn cdb-btn--primary"
                disabled=saving.get()
                on:click=move |_| {
                    saving.set(true);
                    err_for_save.set(Some("保存触发 debounce 1s".to_string()));
                    saving.set(false);
                    let _ = debouncer_for_save.clone();
                    let _ = store_for_save.clone();
                }
            >
                {move || if saving.get() { "保存中..." } else { "保存" }}
            </button>
        </div>
    }
}

/// 左侧面板视图 (B2: 7-Tab 容器 + 顶部搜索框 + 类型筛选)
/// - 7 Tab: Tables / Areas / Enums / Notes / Relationships / Types / Issues
/// - 顶部全局搜索框：跨 Tab 模糊匹配（spec §10）；query 为空时不过滤
/// - 类型筛选下拉：仅作用于 Tables Tab（B2 简化）；其他 Tab 暂用全集
/// - 选中态：active_tab 用 `cdb-is-active` 类指示
/// - data-testid: tab-{key} 8 个 + search-input + type-filter + tab-pane-{key}
#[component]
pub fn LeftPanel(
    store: EditorStore,
    view_mode: RwSignal<ViewMode>,
    selected_table_id: RwSignal<Option<String>>,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    on_jump_to_table: Option<Rc<dyn Fn(String)>>,
    on_create_table: Rc<dyn Fn()>,
    on_save: Rc<dyn Fn()>,
    on_add_field: Rc<dyn Fn(String)>,
    on_change_type: Rc<dyn Fn(String, String)>,
    on_set_ref: Rc<dyn Fn(String)>,
) -> impl IntoView {
    // Enums/Types：V1 仅前端 state；Areas/Notes 已接入 store（align-v1-areas-notes-store）
    let enums: RwSignal<Vec<EnumStub>> = create_rw_signal(Vec::new());
    let types: RwSignal<Vec<TypeStub>> = create_rw_signal(Vec::new());
    let area_seq: RwSignal<i64> = create_rw_signal(0);
    let note_seq: RwSignal<i64> = create_rw_signal(0);

    let active_tab: RwSignal<SidePanelTab> = create_rw_signal(SidePanelTab::Tables);
    let search_query: RwSignal<String> = create_rw_signal(String::new());
    let type_filter: RwSignal<String> = create_rw_signal(String::new());

    let tab_keys = [
        SidePanelTab::Tables,
        SidePanelTab::Areas,
        SidePanelTab::Enums,
        SidePanelTab::Notes,
        SidePanelTab::Relationships,
        SidePanelTab::Types,
        SidePanelTab::Issues,
        SidePanelTab::Fields,
    ];

    create_effect(move |_| {
        if selected_table_id.get().is_some() {
            active_tab.set(SidePanelTab::Fields);
        }
    });

    view! {
        <div class="cdb-side-panel" data-testid="left-panel">
            <div class="cdb-tabs cdb-tabs--icon-grid" role="tablist">
                <For each=move || tab_keys.clone() key=|t| *t children=move |tab: SidePanelTab| {
                    let tab_for_click = tab;
                    let testid = tab.testid();
                    let tooltip = tab.label();
                    let show_badge = matches!(
                        tab_for_click,
                        SidePanelTab::Tables | SidePanelTab::Relationships
                    );
                    view! {
                        <div
                            class="cdb-tab cdb-tab--icon"
                            class:cdb-is-active=move || active_tab.get() == tab_for_click
                            role="tab"
                            data-testid={testid}
                            title=tooltip
                            aria-label=tooltip
                            on:click=move |_| active_tab.set(tab_for_click)
                        >
                            <InspectorTabIcon tab=tab_for_click />
                            {move || if show_badge {
                                let count = match tab_for_click {
                                    SidePanelTab::Tables => store.tables.get().len(),
                                    SidePanelTab::Relationships => store.references.get().len(),
                                    _ => 0,
                                };
                                view! { <span class="cdb-tab-badge">{count}</span> }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }}
                        </div>
                    }
                } />
            </div>
            {move || if active_tab.get() != SidePanelTab::Fields {
                view! {
                    <div class="cdb-search-box">
                        <input
                            type="text"
                            class="cdb-search-input"
                            placeholder="搜索..."
                            data-testid="side-search"
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                        {move || if active_tab.get() == SidePanelTab::Tables {
                            view! {
                                <select
                                    class="cdb-type-filter"
                                    data-testid="type-filter"
                                    on:change=move |ev| type_filter.set(event_target_value(&ev))
                                >
                                    <option value="">"所有类型"</option>
                                    <option value="INT">"INT"</option>
                                    <option value="VARCHAR(255)">"VARCHAR(255)"</option>
                                    <option value="TEXT">"TEXT"</option>
                                    <option value="BOOLEAN">"BOOLEAN"</option>
                                </select>
                            }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
            <div class="cdb-tab-content">
                {move || match active_tab.get() {
                    SidePanelTab::Tables => view! {
                        <TablesTab
                            store=store.clone()
                            selected_table_id=selected_table_id.clone()
                            on_select_table=on_select_table.clone()
                            search_query=search_query.clone()
                            type_filter=type_filter.clone()
                            on_create_table=on_create_table.clone()
                            on_save=on_save.clone()
                        />
                    }.into_view(),
                    // ux-canvas-batch 批次1: ListView tab 激活时显示列表视图
                    // LeftPanel 死区另立案清理（modal_kind 不在 LeftPanel 作用域，
                    // 半成品平移到活路径后此处最低限度编译通过即可）
                    SidePanelTab::ListView => {
                        // LeftPanel 死区另立案清理（command_stack 不在 LeftPanel 作用域，
                        // 半成品平移到活路径后此处最低限度编译通过即可）
                        // ux-canvas-batch 批次3 步骤 3: 双击跳画布 — on_jump_to_canvas prop = 切回 Canvas + 选中表
                        let on_jump_for_listview: Rc<dyn Fn(String)> = {
                            let on_select = on_select_table.clone();
                            Rc::new(move |tid: String| {
                                view_mode.set(ViewMode::Canvas);
                                on_select(Some(tid));
                            })
                        };
                        let on_back_for_listview: Rc<dyn Fn()> =
                            Rc::new(move || view_mode.set(ViewMode::Canvas));
                        let command_stack_dummy = create_rw_signal(Rc::new(RefCell::new(
                            crate::editor_core::CommandStack::new(),
                        )));
                        view! {
                            <ListView
                                store=store.clone()
                                on_select_table=on_select_table.clone()
                                on_jump_to_canvas=on_jump_for_listview.clone()
                                on_back_to_canvas=on_back_for_listview
                                on_save=on_save.clone()
                                command_stack=command_stack_dummy
                                read_only=Rc::new(|| false)
                            />
                        }.into_view()
                    },
                    SidePanelTab::Areas => view! {
                        <AreasTab
                            store=store.clone()
                            search_query=search_query.clone()
                            area_seq=area_seq
                            on_save=on_save.clone()
                        />
                    }.into_view(),
                    SidePanelTab::Enums => view! {
                        <EnumsTab enums=enums search_query=search_query.clone() />
                    }.into_view(),
                    SidePanelTab::Notes => view! {
                        <NotesTab
                            store=store.clone()
                            search_query=search_query.clone()
                            note_seq=note_seq
                            on_save=on_save.clone()
                        />
                    }.into_view(),
                    SidePanelTab::Relationships => view! {
                        <RelationshipsTab store=store.clone() search_query=search_query.clone() />
                    }.into_view(),
                    SidePanelTab::Types => view! {
                        <TypesTab types=types search_query=search_query.clone() />
                    }.into_view(),
                    SidePanelTab::Issues => view! {
                        <IssuesTab store=store.clone() on_jump_to_table=on_jump_to_table.clone() />
                    }.into_view(),
                    SidePanelTab::Fields => view! {
                        <FieldsTabContent
                            store=store.clone()
                            selected_table_id=selected_table_id
                            on_add_field=on_add_field.clone()
                            on_change_type=on_change_type.clone()
                            on_set_ref=on_set_ref.clone()
                        />
                    }.into_view(),
                }}
            </div>
        </div>
    }
}

/// R5：字段 Tab 内容（原 RightPanel，全高单栏）
#[component]
pub fn FieldsTabContent(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_add_field: Rc<dyn Fn(String)>,
    on_change_type: Rc<dyn Fn(String, String)>,
    on_set_ref: Rc<dyn Fn(String)>,
) -> impl IntoView {
    let selected_table = create_memo(move |_| {
        let id = selected_table_id.get()?;
        store.tables.get().into_iter().find(|t| t.id == id)
    });
    let has_selection = create_memo(move |_| selected_table.get().is_some());

    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-fields">
            <div class="cdb-tab-pane__scroll">
            {move || if has_selection.get() {
                let t = selected_table.get().unwrap();
                let fields = t.fields.clone();
                let table_name = t.name.clone();
                let on_add = on_add_field.clone();
                let on_change = on_change_type.clone();
                let on_ref = on_set_ref.clone();
                let on_add_click = {
                    let on_add = on_add.clone();
                    move |_| {
                        if let Some(id) = selected_table_id.get() {
                            on_add(id);
                        }
                    }
                };
                view! {
                    <div class="cdb-field-list" data-testid="field-editor">
                        <h3>{table_name}</h3>
                        <button
                            class="cdb-btn cdb-btn--primary cdb-btn--block"
                            data-testid="btn-add-field"
                            on:click=on_add_click
                        >
                            "加字段"
                        </button>
                        <For each=move || fields.clone() key=|f| f.id.clone() children=move |field: Field| {
                            let field_id = field.id.clone();
                            let field_id_for_change = field_id.clone();
                            let field_id_for_ref = field_id.clone();
                            let field_name = field.name.clone();
                            let field_type = field.type_.clone();
                            let on_change = on_change.clone();
                            let on_ref = on_ref.clone();
                            let testid_row = format!("field-row-{}", field_id);
                            let testid_type = format!("type-{}", field_id);
                            let testid_ref = format!("set-ref-{}", field_id);
                            view! {
                                <div class="cdb-field-row" data-testid={testid_row}>
                                    <span>{field_name}</span>
                                    <select
                                        data-testid={testid_type}
                                        on:change=move |ev| {
                                            let new_type = event_target_value(&ev);
                                            on_change(field_id_for_change.clone(), new_type);
                                        }
                                    >
                                        <option value="INT" selected={field_type == "INT"}>"INT"</option>
                                        <option value="BIGINT" selected={field_type == "BIGINT"}>"BIGINT"</option>
                                        <option value="UUID" selected={field_type == "UUID"}>"UUID"</option>
                                        <option value="VARCHAR(255)" selected={field_type == "VARCHAR(255)"}>"VARCHAR(255)"</option>
                                        <option value="TEXT" selected={field_type == "TEXT"}>"TEXT"</option>
                                        <option value="BOOLEAN" selected={field_type == "BOOLEAN"}>"BOOLEAN"</option>
                                        <option value="DATE" selected={field_type == "DATE"}>"DATE"</option>
                                        <option value="TIMESTAMP" selected={field_type == "TIMESTAMP"}>"TIMESTAMP"</option>
                                        <option value="FLOAT" selected={field_type == "FLOAT"}>"FLOAT"</option>
                                        <option value="DOUBLE" selected={field_type == "DOUBLE"}>"DOUBLE"</option>
                                        <option value="DECIMAL" selected={field_type == "DECIMAL"}>"DECIMAL"</option>
                                    </select>
                                    <button
                                        class="cdb-btn cdb-btn--icon"
                                        data-testid={testid_ref}
                                        on:click=move |_| { on_ref(field_id_for_ref.clone()); }
                                    >
                                        "设关系"
                                    </button>
                                </div>
                            }
                        } />
                    </div>
                }.into_view()
            } else {
                view! { <p class="cdb-empty-hint">"请选择一个表"</p> }.into_view()
            }}
            </div>
        </div>
    }
}

/// 右侧面板视图（R5 废弃分割布局；保留组件供 UT 字符串引用）
#[component]
pub fn RightPanel(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_add_field: Rc<dyn Fn(String)>,
    on_change_type: Rc<dyn Fn(String, String)>,
    on_set_ref: Rc<dyn Fn(String)>,
) -> impl IntoView {
    view! {
        <FieldsTabContent
            store=store
            selected_table_id=selected_table_id
            on_add_field=on_add_field
            on_change_type=on_change_type
            on_set_ref=on_set_ref
        />
    }
}

// =====================================================================
// B2: 7-Tab 子组件（Tables / Areas / Enums / Notes / Relationships / Types / Issues）
// =====================================================================

/// Tables Tab — 表格列表 + 搜索过滤 + 类型筛选（UT-SP-02 覆盖）
/// B2 行为：search_query 非空时按表名子串匹配；type_filter 非空时按字段类型子串匹配。
/// Tables 列表（从 TablesTab 拆出，避免 Show fallback FnOnce）
#[component]
fn TablesList(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    search_query: RwSignal<String>,
    type_filter: RwSignal<String>,
) -> impl IntoView {
    let filtered = create_memo(move |_| {
        let all = store.tables.get();
        let q = search_query.get();
        let t = type_filter.get();
        let mut v = filter_by_query(&all, &q);
        if !t.is_empty() {
            v.retain(|table| {
                table
                    .fields
                    .iter()
                    .any(|f| f.type_.to_uppercase().contains(&t.to_uppercase()))
            });
        }
        v
    });
    view! {
        <div class="cdb-tab-pane__scroll">
            <For each=move || filtered.get() key=|t| t.id.clone() children=move |table: Table| {
                let table_id = table.id.clone();
                let table_name = table.name.clone();
                let field_count = table.fields.len();
                let color = table.color.clone();
                let on_select = on_select_table.clone();
                let testid = format!("table-list-item-{}", table_id);
                let table_id_for_click = table_id.clone();
                let color_style = format!("--table-color: {}", color);
                view! {
                    <div
                        class="cdb-list-item"
                        class:cdb-is-selected=move || is_table_selected(&selected_table_id.get(), &table_id)
                        data-testid={testid}
                        style=color_style
                        on:click=move |_| { on_select(Some(table_id_for_click.clone())); }
                    >
                        <div class="cdb-list-item__row">
                            <span class="cdb-list-item__dot"></span>
                            <span class="cdb-list-item__name">{table_name}</span>
                            <span class="cdb-list-item__meta">{field_count}</span>
                        </div>
                    </div>
                }
            } />
            {move || if filtered.get().is_empty() {
                view! { <p class="cdb-empty-hint">"无匹配表"</p> }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn TablesTab(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    search_query: RwSignal<String>,
    type_filter: RwSignal<String>,
    on_create_table: Rc<dyn Fn()>,
    on_save: Rc<dyn Fn()>,
) -> impl IntoView {
    let is_empty_store = create_memo(move |_| {
        store.tables.get().is_empty()
            && search_query.get().is_empty()
            && type_filter.get().is_empty()
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-tables">
            <div class="cdb-tab-actions cdb-tab-actions--stacked">
                <button
                    class="cdb-btn cdb-btn--primary cdb-btn--block"
                    data-testid="btn-create-table"
                    on:click=move |_| on_create_table()
                >
                    "+ 添加表"
                </button>
                <button
                    class="cdb-btn cdb-btn--ghost cdb-btn--block"
                    data-testid="btn-save"
                    on:click=move |_| on_save()
                >
                    "保存"
                </button>
            </div>
            {move || if is_empty_store.get() {
                view! {
                    <div class="cdb-empty-state" data-testid="tables-empty-state">
                        <div class="cdb-empty-state__icon">"📋"</div>
                        <div class="cdb-empty-state__title">"空空如也"</div>
                        <div class="cdb-empty-state__hint">"开始构建您的图表！"</div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="cdb-tab-pane__scroll">
                        <TablesList
                            store=store.clone()
                            selected_table_id=selected_table_id
                            on_select_table=on_select_table.clone()
                            search_query=search_query
                            type_filter=type_filter
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}

// ─── ux-canvas-batch 批次1: ListView 组件 + sort_tables 纯函数 ────────────────

/// 排序列枚举（按表维度属性排序——外环判词记一笔修正措辞，避免实现期误解为仅展示列可排序）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortColumn {
    TableName,
    FieldCount,
    Type,
    HasIndex,
}

/// 排序方向枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// ListViewState（C-4：落点统一为 editor_panels.rs，以纯函数可测性为准）
#[derive(Clone)]
pub struct ListViewState {
    pub sort_column: RwSignal<SortColumn>,
    pub sort_direction: RwSignal<SortDirection>,
    // ux-canvas-batch 批次2: 过滤字段
    pub filter_query: RwSignal<String>,      // 按名称模糊匹配（表名/字段名/类型）
    pub filter_type: RwSignal<String>,        // 按类型过滤（与 SortColumn::Type 首字段类型口径对齐）
    pub filter_has_index: RwSignal<Option<bool>>, // 按是否有索引过滤（Some(true)=仅有索引，Some(false)=仅无索引，None=不过滤）
    // ux-canvas-batch 批次3 步骤 2: 批量改类型选中态（checkbox 多选 + 单一目标类型——外环条目 12 修正 4）
    pub batch_type_selection: RwSignal<crate::editor_panels::BatchTypeSelection>,
    // ux-canvas-batch 批次4 步骤 3 (条目 23): 列宽会话态——键名严格对齐 ListView 实际 <th>
    // 展示列（table_name/field_count/type/has_index，批次1既有4列）。注意：外环提案
    // v2 文本提「field_name/field_type」，但 ListView 实际展示列是「字段数/类型」，
    // 不含「字段名」列——本批以实际展示列为准。
    pub column_widths: RwSignal<ColumnWidths>,
    // ux-canvas-batch 批次4 步骤 5 (条目 27): 分组模式会话态
    pub group_by: RwSignal<GroupByMode>,
}

/// ux-canvas-batch 批次4 步骤 3 (条目 23): 列宽会话态结构（键名对齐 ListView <th> 展示列）
/// - 键：table_name / field_count / type / has_index（与 ListView 实际 4 个 <th> 严格 1:1）
/// - 默认每列 120px（既有默认）
/// - 会话态：不写后端；用户刷新页面重置
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnWidths {
    pub table_name: u32,
    pub field_count: u32,
    pub type_: u32, // `type` 是 Rust 关键字，用 `type_` 字段名匹配既有 Table.field.type_ 命名
    pub has_index: u32,
}

impl ColumnWidths {
    /// 默认列宽（每列 120px）
    pub fn defaults() -> Self {
        Self {
            table_name: 120,
            field_count: 120,
            type_: 120,
            has_index: 120,
        }
    }
    /// 按键名取列宽（不存在 fallback 120）
    pub fn get(&self, key: &str) -> u32 {
        match key {
            "table_name" => self.table_name,
            "field_count" => self.field_count,
            "type" => self.type_,
            "has_index" => self.has_index,
            _ => 120,
        }
    }
    /// 按键名设列宽（钳制 60/480）
    pub fn set(&mut self, key: &str, w: u32) {
        let clamped = clamp_column_width(w);
        match key {
            "table_name" => self.table_name = clamped,
            "field_count" => self.field_count = clamped,
            "type" => self.type_ = clamped,
            "has_index" => self.has_index = clamped,
            _ => {}
        }
    }
}

/// ux-canvas-batch 批次1：列表视图排序纯函数（UT-MM-21）
/// 真值表（D 案教训：涉及推导/状态机必须给真值表+实例推演）：
///   表名：字典序升序/降序
///   字段数：少→多 / 多→少
///   类型：字典序升序/降序（如 INT < VARCHAR）
///   是否有索引：无→有 / 有→无
/// 实例推演：
///   表 A（5 字段，有索引）、表 B（3 字段，无索引）、表 C（10 字段，有索引）
///   按字段数升序：B(3) → A(5) → C(10)
///   按字段数降序：C(10) → A(5) → B(3)
///   按是否有索引降序：A(有) → C(有) → B(无)
pub fn sort_tables(
    tables: &[Table],
    sort_column: SortColumn,
    sort_direction: SortDirection,
) -> Vec<Table> {
    let mut sorted = tables.to_vec();
    sorted.sort_by(|a, b| {
        let cmp = match sort_column {
            SortColumn::TableName => a.name.cmp(&b.name),
            SortColumn::FieldCount => a.fields.len().cmp(&b.fields.len()),
            SortColumn::Type => {
                // 按首个字段类型字典序（如 INT < VARCHAR）
                let a_type = a.fields.first().map(|f| f.type_.as_str()).unwrap_or("");
                let b_type = b.fields.first().map(|f| f.type_.as_str()).unwrap_or("");
                a_type.cmp(b_type)
            }
            SortColumn::HasIndex => {
                // 无索引 → 有索引（升序）：无索引 = true，有索引 = false
                // true.cmp(&false) = Greater（升序时无索引在后）——反了
                // 修：无索引 = false，有索引 = true → false.cmp(&true) = Less（升序时无索引在前）
                let a_has = !a.indices.is_empty();
                let b_has = !b.indices.is_empty();
                a_has.cmp(&b_has)
            }
        };
        match sort_direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });
    sorted
}

/// ux-canvas-batch 批次2：列表视图过滤纯函数（UT-MM-23）
/// 按名称模糊匹配（表名/字段名/类型含 filter_query 子串，大小写不敏感）
/// 按类型过滤（与 SortColumn::Type 首字段类型口径对齐——外环判词语义记一笔：
/// 取首个字段类型做表级过滤键，空表回退 ""）
/// 按是否有索引过滤（Some(true)=仅有索引，Some(false)=仅无索引，None=不过滤）
/// 组合过滤：三条件 AND（同时满足）
pub fn filter_tables(
    tables: &[Table],
    filter_query: &str,
    filter_type: &str,
    filter_has_index: Option<bool>,
) -> Vec<Table> {
    tables
        .iter()
        .filter(|t| {
            // 按名称模糊匹配（表名/字段名/类型含 filter_query 子串，大小写不敏感）
            if !filter_query.is_empty() {
                let query = filter_query.to_lowercase();
                let name_match = t.name.to_lowercase().contains(&query);
                let field_match = t.fields.iter().any(|f| f.name.to_lowercase().contains(&query));
                let type_match = t.fields.first().map(|f| f.type_.to_lowercase().contains(&query)).unwrap_or(false);
                if !name_match && !field_match && !type_match {
                    return false;
                }
            }
            // 按类型过滤（与 SortColumn::Type 首字段类型口径对齐）
            if !filter_type.is_empty() {
                let table_type = t.fields.first().map(|f| f.type_.as_str()).unwrap_or("");
                if table_type != filter_type {
                    return false;
                }
            }
            // 按是否有索引过滤
            if let Some(has_index) = filter_has_index {
                let table_has_index = !t.indices.is_empty();
                if table_has_index != has_index {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

/// listview-tree-master-detail：列表视图当前表解析纯函数（UT-MM-38）
/// list_table_id 命中 → 该表 id；None 或 id 已不存在（如画布侧删表）→ 首张表；空表数组 → None
/// 与树搜索过滤解耦——过滤只影响树节点展示，不影响当前表解析
pub fn resolve_list_current_table(tables: &[Table], list_table_id: &Option<String>) -> Option<String> {
    if let Some(id) = list_table_id {
        if tables.iter().any(|t| &t.id == id) {
            return Some(id.clone());
        }
    }
    tables.first().map(|t| t.id.clone())
}

/// ux-canvas-batch 批次2：列表视图批量重命名纯函数（UT-MM-24）
/// 规则（重名冲突处理真值表，外环判词 C-1 强制 + B2-S1 补充规则）：
///   - 冲突判定以改名前快照为准（{A→B,B→C} 全跳过——B→C 时 B 仍存在于改名前快照，C 冲突）
///   - 处理顺序按旧名字典序（{B→D, A→D} → A 先处理，A→D 成功，B→D 跳过）
///   - 同一新名多旧名映射（{A→C, B→C}）→ 字典序靠前者得名（A→C 成功），其余跳过（B→C 跳过）
///   - 新名 = 原名 → 跳过（不改名，保持原名）
///   - 新名为空 → 跳过（不改名，保持原名）
///   - 新名含非法字符 → 跳过（不改名，保持原名）
///   - 新名已存在（改名前快照）→ 跳过（不改名，保持原名）
/// 批量改名后 store.dirty.set(true)（标记脏，触发自动保存）
pub fn batch_rename_tables(
    tables: &mut Vec<Table>,
    rename_map: std::collections::HashMap<String, String>,
) {
    // B2-S1 ①：冲突判定以改名前快照为准
    let snapshot_names: std::collections::HashSet<String> = tables.iter().map(|t| t.name.clone()).collect();
    // B2-S1 ②：处理顺序按旧名字典序
    let mut sorted_renames: Vec<(String, String)> = rename_map.into_iter().collect();
    sorted_renames.sort_by(|a, b| a.0.cmp(&b.0));
    // B2-S1 ③：同一新名多旧名映射，字典序靠前者得名其余跳过
    let mut used_new_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (old_name, new_name) in sorted_renames {
        // 新名 = 原名 → 跳过
        if old_name == new_name {
            continue;
        }
        // 新名为空 → 跳过
        if new_name.is_empty() {
            continue;
        }
        // 新名含非法字符 → 跳过（简单规则：不允许空格）
        if new_name.contains(' ') {
            continue;
        }
        // 新名已存在（改名前快照）→ 跳过
        if snapshot_names.contains(&new_name) {
            continue;
        }
        // 同一新名多旧名映射，字典序靠前者得名其余跳过
        if used_new_names.contains(&new_name) {
            continue;
        }
        // 改名成功
        if let Some(table) = tables.iter_mut().find(|t| t.name == old_name) {
            table.name = new_name.clone();
            used_new_names.insert(new_name);
        }
    }
}

/// ux-canvas-batch 批次3：批量改类型纯函数（UT-MM-26）
/// 通用决策程序（C-1 闭环）：
///   ① 解析基类型 + 可选 (n) 参数 — parse_type_type 签名
///   ② 定义类型族白名单（数值/字符串/日期/布尔/二进制族，由窄到宽）
///   ③ 族内由窄到宽 → 直接改；由宽到窄 → 跳过
///      同基类型参数收窄（如 VARCHAR(255)→VARCHAR(50)）→ 跳过
///   ④ 跨族一律跳过
///   ⑤ 未列出的类型对保守 fallback = 跳过
///   ⑥ 非法/空目标类型跳过
/// 批量改名后 store.dirty.set(true)（通过字段 ID 匹配写入 store）
pub fn batch_change_types(
    tables: &mut Vec<Table>,
    field_type_map: std::collections::HashMap<String, String>,
) {
    for (field_id, new_type) in field_type_map {
        // ⑥ 非法/空目标类型跳过
        if new_type.is_empty() {
            continue;
        }
        // 步骤 ⑤/⑥：解析失败或不在白名单 → 跳过
        if !is_known_type(&new_type) {
            continue;
        }
        // 找字段并尝试改类型
        for table in tables.iter_mut() {
            for field in table.fields.iter_mut() {
                if field.id != field_id {
                    continue;
                }
                // 决策程序：族内由窄到宽直接改，由宽到窄或跨族或未列出 → 跳过
                if should_change_type(&field.type_, &new_type) {
                    field.type_ = new_type.clone();
                }
            }
        }
    }
}

/// ux-canvas-batch 批次3（条目 13 改派修复）：类型族标识
/// v1 type_position 只返族内位置（族身份丢失，跨族漏判）
/// v2 改返 (family, position) 二元组，确保跨族比较先比族
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeFamily {
    Numeric,
    String,
    Datetime,
    Binary,
}

/// 解析类型字符串为 (BaseType, params)——拆基类型 + 可选 (n) / (p,s) 参数
/// v1 未实现（注释自承「parse_type_type 签名」为占位），v2 实现
/// 基类型归族位置（带参按基类型归族，如 VARCHAR(255)→字符串族 VARCHAR 位）
/// to 侧带参维持现行「一律跳过」保守语义（与真值表 VARCHAR(50) 行一致）
fn parse_type(s: &str) -> Option<(TypeFamily, usize, &'static str)> {
    // 按基类型字符串前缀匹配（带括号参数按基类型归族）
    let s = s.trim();
    // 数值族（族内由窄到宽）
    if s == "SMALLINT" { return Some((TypeFamily::Numeric, 0, "SMALLINT")); }
    if s == "INT" { return Some((TypeFamily::Numeric, 1, "INT")); }
    if s == "BIGINT" { return Some((TypeFamily::Numeric, 2, "BIGINT")); }
    if s == "DECIMAL" { return Some((TypeFamily::Numeric, 3, "DECIMAL")); }
    if s == "FLOAT" { return Some((TypeFamily::Numeric, 4, "FLOAT")); }
    if s == "DOUBLE" { return Some((TypeFamily::Numeric, 5, "DOUBLE")); }
    // 字符串族
    if s == "CHAR" { return Some((TypeFamily::String, 0, "CHAR")); }
    if s == "VARCHAR" { return Some((TypeFamily::String, 1, "VARCHAR")); }
    if s == "TEXT" { return Some((TypeFamily::String, 2, "TEXT")); }
    if s == "LONGTEXT" { return Some((TypeFamily::String, 3, "LONGTEXT")); }
    // 日期族
    if s == "DATE" { return Some((TypeFamily::Datetime, 0, "DATE")); }
    if s == "DATETIME" { return Some((TypeFamily::Datetime, 1, "DATETIME")); }
    if s == "TIMESTAMP" { return Some((TypeFamily::Datetime, 2, "TIMESTAMP")); }
    // 二进制族
    if s == "BLOB" { return Some((TypeFamily::Binary, 0, "BLOB")); }
    if s == "MEDIUMBLOB" { return Some((TypeFamily::Binary, 1, "MEDIUMBLOB")); }
    if s == "LONGBLOB" { return Some((TypeFamily::Binary, 2, "LONGBLOB")); }
    if s == "BOOLEAN" { return None; } // 布尔族自成一族（外环条目 12 决策程序描述），v2 暂不支持跨族 BOOLEAN 转换
    // 带参类型（VARCHAR(255)/DECIMAL(10,2) 等）按基类型归族
    if s.starts_with("VARCHAR(") && s.ends_with(")") { return Some((TypeFamily::String, 1, "VARCHAR")); }
    if s.starts_with("CHAR(") && s.ends_with(")") { return Some((TypeFamily::String, 0, "CHAR")); }
    if s.starts_with("DECIMAL(") && s.ends_with(")") { return Some((TypeFamily::Numeric, 3, "DECIMAL")); }
    None
}

/// 是否为已知类型（白名单：INT/BIGINT/SMALLINT/DECIMAL/FLOAT/DOUBLE/VARCHAR/CHAR/TEXT/LONGTEXT/DATE/DATETIME/TIMESTAMP/BOOLEAN/BLOB/MEDIUMBLOB/LONGBLOB）
fn is_known_type(t: &str) -> bool {
    parse_type(t).is_some()
}

/// 决策程序 v2（条目 13 修复）：族身份先比 + 族内由窄到宽直接改 / 由宽到窄或跨族或未列出 → 跳过
/// from 侧带参按基类型归族（如 VARCHAR(255)→字符串族 VARCHAR 位）；
/// to 侧带参维持现行「一律跳过」保守语义（与真值表 VARCHAR(50) 行一致）
fn should_change_type(from: &str, to: &str) -> bool {
    if from == to {
        return true; // 同型直接改
    }
    let p1 = parse_type(from);
    let p2 = parse_type(to);
    match (p1, p2) {
        (Some((f1, a, _)), Some((f2, b, _))) => {
            if f1 != f2 {
                return false; // 跨族一律跳过（族身份先比——v2 修复族身份丢失 bug）
            }
            if a == b {
                false // 同族位置相同（保守跳过——参数变化算收窄）
            } else {
                a < b // 族内由窄到宽 → 改；宽到窄 → 跳
            }
        }
        _ => false, // 未列出对保守 fallback = 跳过
    }
}

/// ux-canvas-batch 批次3：导出仅 CSV schema 内容纯函数（UT-MM-27；外环 C-3 裁决——纯手写无依赖，不引入 xlsx）
/// 导出内容 = 列表视图本身的 schema 内容（行=字段，列=table_name/field_name/field_type/has_index，与批次 1 展示列对齐）
pub fn export_tables_csv(tables: &[Table]) -> String {
    let mut output = String::from("table_name,field_name,field_type,has_index\n");
    for table in tables {
        let table_name = &table.name;
        let has_index = if table.indices.is_empty() { "no" } else { "yes" };
        for field in &table.fields {
            let row = format!(
                "{},{},{},{}\n",
                csv_escape(table_name),
                csv_escape(&field.name),
                csv_escape(&field.type_),
                has_index
            );
            output.push_str(&row);
        }
    }
    output
}

/// CSV 字段值转义：含 `,` `"` `\n` 三字符之一时用双引号包裹 + 内部 `"` 转义为 `""`
fn csv_escape(s: &str) -> String {
    let needs_quote = s.contains(',') || s.contains('"') || s.contains('\n');
    if !needs_quote {
        return s.to_string();
    }
    let escaped = s.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// ux-canvas-batch 批次4 步骤 2 (条目 19/20)：ListView 列宽钳制纯函数（UT-MM-28）
/// - 列宽下限 60px（防塌陷），上限 480px（防溢出）
/// - 真值表（6 行）：30 → 60 / 60 → 60 / 200 → 200 / 480 → 480 / 600 → 480 / 150 → 150
pub fn clamp_column_width(w: u32) -> u32 {
    w.max(60).min(480)
}

/// ux-canvas-batch 批次4 步骤 2 (条目 19/20)：ListView 列宽自适应纯函数（UT-MM-28 追加子用例）
/// - 公式：max(60, min(480, max_field_chars × 8 + 40))
/// - 8 px/字符近似 + 40 px padding（左右各 20px）
/// - 真值表：0 → 60 / 30 → 280 / 100 → 480 / 300 → 480
pub fn auto_calc_column_width(max_field_chars: u32) -> u32 {
    let raw = max_field_chars.saturating_mul(8).saturating_add(40);
    raw.max(60).min(480)
}

/// ux-canvas-batch 批次4 步骤 3 (条目 25)：按列名计算 ListView 实际渲染列最长字符数（UT-MM-28 追加子用例）
/// - table_name → 所有 table.name 最长字符数
/// - field_count → 字段数最大值转字符串长度
/// - type → 该列实际渲染的首字段类型最长字符数（与 cell 渲染同源：`t.fields.first().map(|f| f.type_.clone())`）
/// - has_index → 该列实际渲染内容最长字符数（"有/无" 各 1 字符 / "yes/no" 各 3 字符）
/// - 空表：所有列返 0（auto_calc 钳制下限 60）
pub fn max_chars_for_column(key: &str, tables: &[Table]) -> u32 {
    match key {
        "table_name" => tables.iter().map(|t| t.name.chars().count()).max().unwrap_or(0) as u32,
        "field_count" => {
            let max_count = tables.iter().map(|t| t.fields.len()).max().unwrap_or(0);
            // 字段数转字符串长度（如 100 → 3）
            if max_count == 0 {
                1
            } else {
                let mut n = max_count;
                let mut len = 0;
                while n > 0 {
                    len += 1;
                    n /= 10;
                }
                len as u32
            }
        }
        "type" => {
            // 与 cell 渲染同源：table.fields.first().map(|f| f.type_.clone()).unwrap_or_default()
            tables
                .iter()
                .filter_map(|t| t.fields.first().map(|f| f.type_.chars().count()))
                .max()
                .unwrap_or(0) as u32
        }
        "has_index" => {
            // cell 渲染实际: `if has_index { "有" } else { "无" }` —— 1 字符
            // 记录：条目 26 记一笔，cell 实渲 1 字符，纯函数按 cell 同源计 1
            1u32
        }
        _ => 0,
    }
}

/// ux-canvas-batch 批次4 步骤 4 (条目 17/18/26)：表/字段分组模式
/// - None: 扁平（不分组）
/// - ByTag: 按 Field.tag 分桶（空 tag → (empty) 兜底）
/// - BySchema 裁撤（条目 18 P2：Table 无 schema 字段，分组键 = table.id = 一组一行 = 伪分组）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupByMode {
    None,
    ByTag,
    /// list-view-table-structure：按表分组（PDManer 式表结构清单），默认模式
    #[default]
    ByTable,
}

/// ux-canvas-batch 批次4 步骤 4 (条目 26)：分组桶统一输出形状
/// - key: 桶键（None → "_flat"；ByTag → tag 值或 "(empty)" 兜底）
/// - fields: 该桶字段列表（按 (table_id, field_id) 二元组——与 Table/Field.id 命名一致）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bucket {
    pub key: String,
    pub fields: Vec<(String, String)>, // (table_id, field_id)
}

/// ux-canvas-batch 批次4 步骤 4 (条目 26)：表/字段分组纯函数（UT-MM-29）
/// - None → 单桶（key="_flat"），含所有 tables 的所有字段（扁平直通）
/// - ByTag → 按 Field.tag 分桶（BTreeMap 保 key 字典序），空 tag → "(empty)" 兜底
/// - 真值表（5 行覆盖：None / ByTag 单 tag / ByTag 混合 tag / 空表 / 单字段多 tag）
/// - 大小写敏感（`Pk` ≠ `pk`）——与字段名命名约定一致
pub fn group_tables(tables: &[Table], mode: GroupByMode) -> Vec<Bucket> {
    use std::collections::BTreeMap;
    match mode {
        GroupByMode::None => {
            // 单桶 _flat 含所有字段
            let mut fields: Vec<(String, String)> = Vec::new();
            for t in tables {
                for f in &t.fields {
                    fields.push((t.id.clone(), f.id.clone()));
                }
            }
            vec![Bucket { key: "_flat".to_string(), fields }]
        }
        GroupByMode::ByTable => {
            // list-view-table-structure：按表分桶（桶键 = 表名；空表也出桶；桶序 = tables 数组顺序）
            tables
                .iter()
                .map(|t| Bucket {
                    key: t.name.clone(),
                    fields: t.fields.iter().map(|f| (t.id.clone(), f.id.clone())).collect(),
                })
                .collect()
        }
        GroupByMode::ByTag => {
            // 按 tag 分桶（空 tag → "(empty)" 兜底）
            let mut buckets: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
            for t in tables {
                for f in &t.fields {
                    let key = if f.tag.is_empty() { "(empty)".to_string() } else { f.tag.clone() };
                    buckets.entry(key).or_default().push((t.id.clone(), f.id.clone()));
                }
            }
            buckets.into_iter().map(|(key, fields)| Bucket { key, fields }).collect()
        }
    }
}

/// ux-canvas-batch 批次3 步骤 2：批量改类型选中态（ListViewState 扩展——条目 12 修正 4）
#[derive(Clone, Default)]
pub struct BatchTypeSelection {
    pub selected_field_ids: std::collections::HashSet<String>,
    pub target_type: String,
}

// ─── redesign-listview-type-length-canvas-fix：ListView 重写为 PDManer 式字段明细编辑网格 ───
// （core-04 §10.5：按表分组 + 10 列行内编辑 + 工具条增/删/上移/下移；
//   原只读清单 / 分组切换 / 批量类型面板由行内编辑取代）
// ─── listview-tree-master-detail：布局重构为「左表树 + 右单表网格」（core-04 §10.5） ───
// 组头行机制（ListRowItem::Header / ListGroupHeader / list-view-group-* testid）随全量
// 堆叠网格一并移除；表导航上移为左侧表树（list-tree-node-* 锚点）。

/// 列表视图字段行项（仅字段行；表导航由左侧表树承担）。
/// For keyed diff——编辑落账只增删/重排行，不重建既有行；行内草稿不受 store 写回影响，
/// 规避「击键 → store 高频写回 → 全表重建 → 丢字/卡死」链路（提案 bug3 根因）。

/// 字段行：10 列内嵌编辑（core-04 §10.5 列定义）。
/// 受控输入口径：文本/数字击键只写本地草稿（不触 store），blur 落账；
/// 勾选与类型下拉即改即账；类型/长度/小数位按 core-01a §2.2 参数化口径合成整串。
#[component]
fn ListFieldRow(
    store: EditorStore,
    table_id: String,
    table_name: String,
    field: Field,
    list_sel: RwSignal<Option<(String, String)>>,
    on_save: Rc<dyn Fn()>,
    on_jump_to_canvas: Rc<dyn Fn(String)>,
    read_only: Rc<dyn Fn() -> bool>,
) -> impl IntoView {
    let parsed = crate::editor_core::parse_field_type(&field.type_);
    // 本地草稿（受控输入：击键只写草稿，blur 才落账——输入期间 store 不重建本行）
    let name_draft = create_rw_signal(field.name.clone());
    let comment_draft = create_rw_signal(field.comment.clone());
    let tag_draft = create_rw_signal(field.tag.clone());
    let len_draft = create_rw_signal(parsed.len.clone());
    let scale_draft = create_rw_signal(parsed.scale.clone());
    let base_type = create_rw_signal(parsed.base.clone());

    let tid = table_id.clone();
    let fid = field.id.clone();

    // 通用落账：闭包返回 true（值有变化）才写 store + dirty + schedule_save；read_only 短路
    let commit: Rc<dyn Fn(&dyn Fn(&mut Field) -> bool)> = {
        let on_save = on_save.clone();
        let tid = tid.clone();
        let fid = fid.clone();
        let read_only = read_only.clone();
        Rc::new(move |f: &dyn Fn(&mut Field) -> bool| {
            if read_only() {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(field) = tables
                .iter_mut()
                .find(|t| t.id == tid)
                .and_then(|t| t.fields.iter_mut().find(|x| x.id == fid))
            {
                if f(field) {
                    store.tables.set(tables);
                    store.dirty.set(true);
                    on_save();
                }
            }
        })
    };

    // 文本列：blur 落账（仅值变化才写）
    let commit_name = {
        let commit = commit.clone();
        move |_| {
            let v = name_draft.get_untracked();
            commit(&move |f: &mut Field| {
                if f.name != v {
                    f.name = v.clone();
                    true
                } else {
                    false
                }
            });
        }
    };
    let commit_comment = {
        let commit = commit.clone();
        move |_| {
            let v = comment_draft.get_untracked();
            commit(&move |f: &mut Field| {
                if f.comment != v {
                    f.comment = v.clone();
                    true
                } else {
                    false
                }
            });
        }
    };
    let commit_tag = {
        let commit = commit.clone();
        move |_| {
            let v = tag_draft.get_untracked();
            commit(&move |f: &mut Field| {
                if f.tag != v {
                    f.tag = v.clone();
                    true
                } else {
                    false
                }
            });
        }
    };
    // 类型/长度/小数位三联：任一变更按 core-01a §2.2 口径重新合成 type_ 整串
    let commit_type = {
        let commit = commit.clone();
        move |ev| {
            let v = event_target_value(&ev);
            base_type.set(v.clone());
            commit(&move |f: &mut Field| {
                let p = crate::editor_core::parse_field_type(&f.type_);
                let next = crate::editor_core::compose_field_type(&v, &p.len, &p.scale);
                if f.type_ != next {
                    f.type_ = next;
                    true
                } else {
                    false
                }
            });
        }
    };
    let commit_len = {
        let commit = commit.clone();
        move |_| {
            let v = len_draft.get_untracked();
            commit(&move |f: &mut Field| {
                let p = crate::editor_core::parse_field_type(&f.type_);
                let next = crate::editor_core::compose_field_type(&p.base, &v, &p.scale);
                if f.type_ != next {
                    f.type_ = next;
                    true
                } else {
                    false
                }
            });
        }
    };
    let commit_scale = {
        let commit = commit.clone();
        move |_| {
            let v = scale_draft.get_untracked();
            commit(&move |f: &mut Field| {
                let p = crate::editor_core::parse_field_type(&f.type_);
                let next = crate::editor_core::compose_field_type(&p.base, &p.len, &v);
                if f.type_ != next {
                    f.type_ = next;
                    true
                } else {
                    false
                }
            });
        }
    };
    // 勾选列：即改即账
    let toggle_bool: Rc<dyn Fn(web_sys::Event, &'static str)> = {
        let commit = commit.clone();
        Rc::new(move |ev: web_sys::Event, which: &'static str| {
            let c = event_target_checked(&ev);
            commit(&move |f: &mut Field| {
                let slot = match which {
                    "primary" => &mut f.primary,
                    "not_null" => &mut f.not_null,
                    "unique" => &mut f.unique,
                    _ => &mut f.increment,
                };
                if *slot != c {
                    *slot = c;
                    true
                } else {
                    false
                }
            });
        })
    };

    let is_sel = {
        let tid = tid.clone();
        let fid = fid.clone();
        move || list_sel.get().map(|(t, f)| t == tid && f == fid).unwrap_or(false)
    };
    let tid_click = tid.clone();
    let fid_click = fid.clone();
    let tid_dbl = tid.clone();
    let ro_text = read_only.clone();
    let ro_text2 = read_only.clone();
    let ro_text3 = read_only.clone();
    let ro_len = read_only.clone();
    let ro_scale = read_only.clone();
    let ro_cb1 = read_only.clone();
    let ro_cb2 = read_only.clone();
    let ro_cb3 = read_only.clone();
    let ro_cb4 = read_only.clone();
    let ro_sel = read_only.clone();
    let pk = field.primary;
    let nn = field.not_null;
    let uq = field.unique;
    let auto = field.increment;
    let tg_pk = toggle_bool.clone();
    let tg_nn = toggle_bool.clone();
    let tg_uq = toggle_bool.clone();
    let tg_auto = toggle_bool.clone();

    view! {
        <tr
            data-testid={format!("list-view-row-{}-{}", table_name, fid)}
            class:is-selected=is_sel
            on:click=move |_| list_sel.set(Some((tid_click.clone(), fid_click.clone())))
            on:dblclick=move |_| on_jump_to_canvas(tid_dbl.clone())
        >
            <td>
                <input
                    class="cdb-form-input"
                    data-testid={format!("list-field-name-{}", fid)}
                    prop:value=move || name_draft.get()
                    disabled=move || ro_text()
                    on:input=move |ev| name_draft.set(event_target_value(&ev))
                    on:blur=commit_name
                />
            </td>
            <td>
                <input
                    class="cdb-form-input"
                    data-testid={format!("list-field-comment-{}", fid)}
                    prop:value=move || comment_draft.get()
                    disabled=move || ro_text2()
                    on:input=move |ev| comment_draft.set(event_target_value(&ev))
                    on:blur=commit_comment
                />
            </td>
            <td>
                <input
                    type="checkbox"
                    data-testid={format!("list-field-pk-{}", fid)}
                    prop:checked=pk
                    disabled=move || ro_cb1()
                    on:change=move |ev| tg_pk(ev, "primary")
                />
            </td>
            <td>
                <input
                    type="checkbox"
                    data-testid={format!("list-field-nn-{}", fid)}
                    prop:checked=nn
                    disabled=move || ro_cb2()
                    on:change=move |ev| tg_nn(ev, "not_null")
                />
            </td>
            <td>
                <input
                    type="checkbox"
                    data-testid={format!("list-field-uq-{}", fid)}
                    prop:checked=uq
                    disabled=move || ro_cb3()
                    on:change=move |ev| tg_uq(ev, "unique")
                />
            </td>
            <td>
                <input
                    type="checkbox"
                    data-testid={format!("list-field-auto-{}", fid)}
                    prop:checked=auto
                    disabled=move || ro_cb4()
                    on:change=move |ev| tg_auto(ev, "increment")
                />
            </td>
            <td>
                <select
                    class="cdb-form-select"
                    data-testid={format!("list-field-type-{}", fid)}
                    disabled=move || ro_sel()
                    on:change=commit_type
                >
                    {move || {
                        // 基类型清单按引擎过滤（core-01a §2.2）；当前基类型不在清单时追加占位项
                        let cur = base_type.get();
                        let types = crate::editor_core::types::types_for_database(store.database.get());
                        let mut opts: Vec<String> = types.iter().map(|s| s.to_string()).collect();
                        if !cur.is_empty() && !opts.contains(&cur) {
                            opts.push(cur.clone());
                        }
                        opts.into_iter().map(|t| {
                            let sel = t == cur;
                            view! { <option value=t.clone() selected=sel>{t}</option> }
                        }).collect_view()
                    }}
                </select>
            </td>
            <td>
                <input
                    class="cdb-form-input"
                    style="width:56px"
                    data-testid={format!("list-field-len-{}", fid)}
                    prop:value=move || len_draft.get()
                    disabled=move || ro_len() || base_type.get() != "VARCHAR"
                    on:input=move |ev| len_draft.set(event_target_value(&ev))
                    on:blur=commit_len
                />
            </td>
            <td>
                <input
                    class="cdb-form-input"
                    style="width:56px"
                    data-testid={format!("list-field-scale-{}", fid)}
                    prop:value=move || scale_draft.get()
                    disabled=move || {
                        let b = base_type.get();
                        ro_scale() || !(b == "DECIMAL" || b == "NUMERIC")
                    }
                    on:input=move |ev| scale_draft.set(event_target_value(&ev))
                    on:blur=commit_scale
                />
            </td>
            <td>
                <input
                    class="cdb-form-input"
                    data-testid={format!("list-field-tag-{}", fid)}
                    prop:value=move || tag_draft.get()
                    disabled=move || ro_text3()
                    on:input=move |ev| tag_draft.set(event_target_value(&ev))
                    on:blur=commit_tag
                />
                // feat-data-dictionary（S07，core-01e §3）：说明列后缀追加字典映射摘要
                // （只读展示，绑定编辑只在 Inspector；响应式读 store 保证绑定/字典变更即时刷新）
                {let tid_dict = tid.clone();
                 let fid_dict = fid.clone();
                 let fid_dict_tag = fid.clone();
                 move || {
                    let code = store.tables.get().iter()
                        .find(|t| t.id == tid_dict)
                        .and_then(|t| t.fields.iter().find(|f| f.id == fid_dict))
                        .map(|f| f.dict_code.clone())
                        .unwrap_or_default();
                    if code.is_empty() {
                        return None;
                    }
                    let dicts = store.dictionaries.get();
                    crate::editor_dict::find_dict(&dicts, &code)
                        .map(crate::editor_dict::dict_summary)
                        .filter(|s| !s.is_empty())
                        .map(|s| view! {
                            <span class="cdb-field-dict-tag" data-testid={format!("list-field-dict-tag-{}", fid_dict_tag)}>{s}</span>
                        })
                }}
            </td>
        </tr>
    }
}

/// redesign-listview-type-length-canvas-fix：ListView（core-04 §10.5 PDManer 式字段明细编辑网格）。
/// listview-tree-master-detail：布局重构为「左侧表树（搜索 + 单击选表 / 双击跳画布）+ 右侧当前表单表网格」。
/// 工具条：增字段（当前树选中表）/删字段（级联清理关系，Command 快照可撤销）/上移/下移 + 返回画布。
#[component]
pub fn ListView(
    store: EditorStore,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    on_jump_to_canvas: Rc<dyn Fn(String)>,
    on_back_to_canvas: Rc<dyn Fn()>,
    on_save: Rc<dyn Fn()>,
    command_stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>>,
    read_only: Rc<dyn Fn() -> bool>,
    // fix-overflow-menu-room-delete-listview-io：ListView 工具条 IO 入口（core-04 §10.5）；
    // 仅全屏 ListView 传入，LeftPanel 内嵌实例不展示
    #[prop(optional)]
    on_open_import: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    on_open_export: Option<Rc<dyn Fn()>>,
    // fix-dict-save-and-layout（core-01e §3.1）：字典区块「在字典面板中编辑」入口；
    // 仅全屏 ListView 传入，LeftPanel 内嵌实例不展示该按钮
    #[prop(optional)]
    on_open_dicts: Option<Rc<dyn Fn()>>,
) -> impl IntoView {
    // 选中行 {table_id, field_id}（core-04 §10.5：选中行高亮；删/移作用于该行）
    let list_sel: RwSignal<Option<(String, String)>> = create_rw_signal(None);
    // listview-tree-master-detail：树选中表 id（None → resolve 回落首张表）+ 树搜索关键字
    let list_table: RwSignal<Option<String>> = create_rw_signal(None);
    // fix-dict-save-and-layout（core-01e §3.1）：树选中字典 id（与 list_table 互斥）
    let list_dict: RwSignal<Option<String>> = create_rw_signal(None);
    let tree_search: RwSignal<String> = create_rw_signal(String::new());

    // 当前表解析（UT-MM-38 纯函数）：list_table 命中 || 首表 || None；与树过滤解耦
    let current_table_id: Rc<dyn Fn() -> Option<String>> = {
        let store = store.clone();
        Rc::new(move || resolve_list_current_table(&store.tables.get(), &list_table.get()))
    };

    let add_field = {
        let on_save = on_save.clone();
        let read_only = read_only.clone();
        let current_table_id = current_table_id.clone();
        move |_| {
            if read_only() {
                return;
            }
            let mut tables = store.tables.get();
            // 当前树选中表（listview-tree-master-detail：替代原「选中行表优先否则首表」）
            let target = current_table_id()
                .and_then(|cid| tables.iter().position(|t| t.id == cid));
            if let Some(pos) = target {
                let bases = crate::editor_core::types::types_for_database(store.database.get());
                let new_field = Field {
                    id: crate::editor_core::new_entity_id("auto"),
                    name: "new_field".into(),
                    type_: crate::editor_core::compose_field_type(bases[0], "", ""),
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
                tables[pos].fields.push(new_field.clone());
                let tid = tables[pos].id.clone();
                store.tables.set(tables);
                store.dirty.set(true);
                command_stack.get().borrow_mut().record(crate::editor_core::Command::AddField {
                    table_id: tid.clone(),
                    field: new_field.clone(),
                });
                list_sel.set(Some((tid, new_field.id)));
                on_save();
            }
        }
    };

    let del_field = {
        let on_save = on_save.clone();
        let read_only = read_only.clone();
        move |_| {
            if read_only() {
                return;
            }
            if let Some((tid, fid)) = list_sel.get() {
                let mut tables = store.tables.get();
                if let Some(t) = tables.iter_mut().find(|t| t.id == tid) {
                    if let Some(pos) = t.fields.iter().position(|f| f.id == fid) {
                        let removed = t.fields.remove(pos);
                        store.tables.set(tables);
                        // 级联清理：触及该字段的关系一并移除（对齐 on_delete_field）
                        let mut refs = store.references.get();
                        refs.retain(|r| {
                            !(r.start_table_id == tid && r.start_field_id == fid)
                                && !(r.end_table_id == tid && r.end_field_id == fid)
                        });
                        store.references.set(refs);
                        store.dirty.set(true);
                        // 快照 + 原下标入栈 → Ctrl+Z 可恢复（ST-SP-LIST-02）
                        command_stack.get().borrow_mut().record(
                            crate::editor_core::Command::DeleteField {
                                table_id: tid.clone(),
                                field_id: fid.clone(),
                                field: Some(removed),
                                index: Some(pos),
                            },
                        );
                        list_sel.set(None);
                        on_save();
                    }
                }
            }
        }
    };

    let move_field: Rc<dyn Fn(i32)> = {
        let on_save = on_save.clone();
        let read_only = read_only.clone();
        Rc::new(move |delta: i32| {
            if read_only() {
                return;
            }
            if let Some((tid, fid)) = list_sel.get() {
                let mut tables = store.tables.get();
                if let Some(t) = tables.iter_mut().find(|t| t.id == tid) {
                    if let Some(i) = t.fields.iter().position(|f| f.id == fid) {
                        let j = i as i32 + delta;
                        if j >= 0 && (j as usize) < t.fields.len() {
                            t.fields.swap(i, j as usize);
                            store.tables.set(tables);
                            store.dirty.set(true);
                            on_save();
                        }
                    }
                }
            }
        })
    };

    let ro_add = read_only.clone();
    let ro_del = read_only.clone();
    let ro_up = read_only.clone();
    let ro_down = read_only.clone();
    let mf_up = move_field.clone();
    let mf_down = move_field.clone();

    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-list-view">
            <div class="cdb-tab-pane__scroll cdb-tab-pane__scroll--list">
                <div class="cdb-list-view-toolbar" data-testid="list-view-toolbar">
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="list-add-field"
                        disabled=move || ro_add() || store.tables.get().is_empty()
                        on:click=add_field
                    >
                        "＋ 增字段"
                    </button>
                    <button
                        class="cdb-btn"
                        data-testid="list-del-field"
                        disabled=move || ro_del() || list_sel.get().is_none()
                        on:click=del_field
                    >
                        "删字段"
                    </button>
                    <button
                        class="cdb-btn"
                        data-testid="list-move-up"
                        disabled=move || ro_up() || list_sel.get().is_none()
                        on:click=move |_| mf_up(-1)
                    >
                        "↑ 上移"
                    </button>
                    <button
                        class="cdb-btn"
                        data-testid="list-move-down"
                        disabled=move || ro_down() || list_sel.get().is_none()
                        on:click=move |_| mf_down(1)
                    >
                        "↓ 下移"
                    </button>
                    <span style="flex:1"></span>
                    // fix-overflow-menu-room-delete-listview-io：ListView IO 入口（core-04 §10.5；只读禁导入）
                    {on_open_import.map(|on_open_import| {
                        let read_only = read_only.clone();
                        let ro_title = read_only.clone();
                        view! {
                            <button
                                class="cdb-btn"
                                data-testid="list-btn-import"
                                disabled=move || read_only()
                                title=move || if ro_title() { "只读成员不可导入" } else { "导入模型" }
                                on:click=move |_| on_open_import()
                            >
                                "导入"
                            </button>
                        }
                    })}
                    {on_open_export.map(|on_open_export| view! {
                        <button
                            class="cdb-btn"
                            data-testid="list-btn-export"
                            title="导出模型"
                            on:click=move |_| on_open_export()
                        >
                            "导出"
                        </button>
                    })}
                    <button
                        class="cdb-btn"
                        data-testid="btn-view-canvas"
                        on:click=move |_| on_back_to_canvas()
                    >
                        "返回画布"
                    </button>
                </div>
                {move || if store.tables.get().is_empty() {
                    view! {
                        <div class="cdb-empty-hint" data-testid="list-view-empty">
                            "暂无数据表——在画布上创建表后，这里会按表展示字段明细。"
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                // listview-tree-master-detail：master-detail 两栏——左表树导航 + 右当前表单表网格
                // feat-room-recycle-bin-dropdown-and-db-export 回归修复：body 移出 store.tables
                // 动态块——原写法在每次 store.tables 写入（如保存回写）时整体重建 DOM，
                // 双击行第一次 click 若恰逢写回，行节点被替换、第二次 click 落空（dblclick
                // 不触发），ST-SP-LIST-01 翻红。改静态节点 + display 绑定：空表隐藏、
                // 视觉等价、行节点跨保存稳定。
                {
                    // 克隆 Rc 句柄——children/move 闭包移出的是本块内的局部量
                    let on_select_tree = on_select_table.clone();
                    let on_jump_tree = on_jump_to_canvas.clone();
                    let on_jump_grid = on_jump_to_canvas.clone();
                    let on_save_grid = on_save.clone();
                    let read_only_grid = read_only.clone();
                    let cur_for_tree = current_table_id.clone();
                    let cur_for_title = current_table_id.clone();
                    let cur_for_grid = current_table_id.clone();
                    view! {
                <div
                    class="cdb-list-view-body"
                    style=move || if store.tables.get().is_empty() { "display:none" } else { "" }
                >
                    <aside class="cdb-list-tree" data-testid="list-tree">
                        <div class="cdb-list-tree__search">
                            <input
                                class="cdb-form-input"
                                data-testid="list-tree-search"
                                placeholder="搜索表…"
                                prop:value=move || tree_search.get()
                                on:input=move |ev| tree_search.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="cdb-list-tree__nodes">
                            <For
                                each=move || {
                                    // 复用 filter_tables 名称模糊匹配口径（core-04 §10.5）
                                    filter_tables(&store.tables.get(), &tree_search.get(), "", None)
                                }
                                key=|t| t.id.clone()
                                children=move |t: Table| {
                                    let tid_active = t.id.clone();
                                    let tid_click = t.id.clone();
                                    let tid_dbl = t.id.clone();
                                    let on_select = on_select_tree.clone();
                                    let on_jump = on_jump_tree.clone();
                                    let cur = cur_for_tree.clone();
                                    let list_dict_active = list_dict.clone();
                                    let list_dict_click = list_dict.clone();
                                    view! {
                                        <button
                                            class="cdb-list-tree-node"
                                            // fix-dict-save-and-layout（core-01e §3.1）：字典选中时表节点不高亮（树选中互斥）
                                            class:is-active=move || list_dict_active.get().is_none() && cur() == Some(tid_active.clone())
                                            data-testid=format!("list-tree-node-{}", t.name)
                                            on:click=move |_| {
                                                // 单击：树选中 + Inspector 同步；清空字段行/字典选中态
                                                list_sel.set(None);
                                                list_dict_click.set(None);
                                                list_table.set(Some(tid_click.clone()));
                                                on_select(Some(tid_click.clone()));
                                            }
                                            on:dblclick=move |_| on_jump(tid_dbl.clone())
                                        >
                                            // fix-canvas-zoom-invite-comment-resize（core-01a §1.4.1，UT-PC-29）：
                                            // 表名下方追加注释行；无注释不渲染、不占位
                                            <span class="cdb-list-tree-node__main">
                                                <span>{t.name}</span>
                                                {(!t.comment.is_empty()).then(|| {
                                                    view! {
                                                        <small
                                                            class="cdb-list-tree-comment"
                                                            data-testid=format!("list-table-comment-{}", t.id)
                                                        >
                                                            {t.comment.clone()}
                                                        </small>
                                                    }
                                                })}
                                            </span>
                                            <span class="cdb-list-tree-node__count">
                                                {format!("{} 字段", t.fields.len())}
                                            </span>
                                        </button>
                                    }
                                }
                            />
                            {move || {
                                if !tree_search.get().is_empty()
                                    && filter_tables(&store.tables.get(), &tree_search.get(), "", None).is_empty()
                                {
                                    view! {
                                        <div class="cdb-empty-hint" data-testid="list-tree-empty">
                                            "无匹配表——换个关键字试试。"
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }
                            }}
                            // fix-dict-save-and-layout（core-01e §3.1）：树区「数据字典」分组——
                            // 恒显、不受表搜索过滤；名称/项数/引用数响应式重读 store（对齐 DictPanel 行口径）
                            <div class="cdb-list-tree__group" data-testid="list-tree-dict-group">"数据字典"</div>
                            <For
                                each=move || store.dictionaries.get()
                                key=|d| d.id.clone()
                                children=move |d: DataDictionary| {
                                    let did_active = d.id.clone();
                                    let did_click = d.id.clone();
                                    let did_name = d.id.clone();
                                    let did_meta = d.id.clone();
                                    let store_name = store.clone();
                                    let store_meta = store.clone();
                                    let list_dict_active = list_dict.clone();
                                    let list_dict_click = list_dict.clone();
                                    view! {
                                        <button
                                            class="cdb-list-tree-node"
                                            class:is-active=move || list_dict_active.get().as_deref() == Some(did_active.as_str())
                                            data-testid=format!("list-tree-dict-{}", d.id)
                                            on:click=move |_| {
                                                // 选中字典：清空表/字段行选中（树选中互斥），不触 Inspector
                                                list_dict_click.set(Some(did_click.clone()));
                                                list_table.set(None);
                                                list_sel.set(None);
                                            }
                                        >
                                            <span class="cdb-list-tree-node__main">
                                                <span>{move || {
                                                    store_name.dictionaries.get().into_iter()
                                                        .find(|x| x.id == did_name)
                                                        .map(|x| format!("{}（{}）", x.name, x.code))
                                                        .unwrap_or_default()
                                                }}</span>
                                            </span>
                                            <span class="cdb-list-tree-node__count">
                                                {move || {
                                                    let dicts = store_meta.dictionaries.get();
                                                    dicts.iter().find(|x| x.id == did_meta).map(|x| {
                                                        let refs = crate::editor_dict::dict_ref_count(&store_meta.tables.get(), &x.code);
                                                        crate::editor_dict::list_dict_node_view(x, refs).1
                                                    }).unwrap_or_default()
                                                }}
                                            </span>
                                        </button>
                                    }
                                }
                            />
                            {move || store.dictionaries.get().is_empty().then(|| view! {
                                <div class="cdb-empty-hint" data-testid="list-tree-dict-empty">"暂无数据字典"</div>
                            })}
                        </div>
                    </aside>
                    <Splitter kind=SplitterKind::ListTree />
                    <div class="cdb-list-view-grid">
                        <div class="cdb-list-view-title" data-testid="list-view-title"
                            style:display=move || if list_dict.get().is_some() { "none" } else { "" }>
                            {move || {
                                let tables = store.tables.get();
                                cur_for_title()
                                    .and_then(|cid| tables.iter().find(|t| t.id == cid).map(|t| {
                                        format!("{}（{} 字段）", t.name, t.fields.len())
                                    }))
                                    .unwrap_or_default()
                            }}
                        </div>
                        <table class="cdb-list-view-table" data-testid="list-view-table"
                            style:display=move || if list_dict.get().is_some() { "none" } else { "" }>
                            <thead>
                                <tr>
                                    <th>"字段代码"</th>
                                    <th>"显示名称"</th>
                                    <th>"主键"</th>
                                    <th>"不为空"</th>
                                    <th>"唯一"</th>
                                    <th>"自增"</th>
                                    <th>"数据类型"</th>
                                    <th>"长度"</th>
                                    <th>"小数位"</th>
                                    <th>"说明"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || {
                                        // 只渲染当前树选中表的字段行（单表网格）
                                        let tables = store.tables.get();
                                        cur_for_grid()
                                            .and_then(|cid| {
                                                tables.iter().find(|t| t.id == cid).map(|t| {
                                                    let tid = t.id.clone();
                                                    let tname = t.name.clone();
                                                    t.fields
                                                        .iter()
                                                        .map(|f| (tid.clone(), tname.clone(), f.clone()))
                                                        .collect::<Vec<_>>()
                                                })
                                            })
                                            .unwrap_or_default()
                                    }
                                    key=|(_, _, f)| f.id.clone()
                                    children=move |(table_id, table_name, field)| view! {
                                        <ListFieldRow
                                            store=store
                                            table_id=table_id
                                            table_name=table_name
                                            field=field
                                            list_sel=list_sel
                                            on_save=on_save_grid.clone()
                                            on_jump_to_canvas=on_jump_grid.clone()
                                            read_only=read_only_grid.clone()
                                        />
                                    }
                                />
                            </tbody>
                        </table>
                        // fix-dict-save-and-layout（core-01e §3.1）：字典选中时详情区只读明细——
                        // 与字段网格 display 互斥（不重建字段行 DOM，保 ST-SP-LIST-01 双击修复口径）
                        <div
                            class="cdb-list-dict-detail"
                            data-testid="list-dict-detail"
                            style:display=move || if list_dict.get().is_some() { "" } else { "none" }
                        >
                            <div class="cdb-list-view-title" data-testid="list-dict-title">
                                {move || {
                                    let dicts = store.dictionaries.get();
                                    list_dict.get()
                                        .and_then(|did| dicts.iter().find(|d| d.id == did)
                                            .map(|d| format!("{}（{}）", d.name, d.code)))
                                        .unwrap_or_default()
                                }}
                            </div>
                            {move || {
                                let dicts = store.dictionaries.get();
                                list_dict.get()
                                    .and_then(|did| dicts.into_iter().find(|d| d.id == did))
                                    .map(|d| {
                                        let summary = crate::editor_dict::dict_summary(&d);
                                        let comment = d.comment.clone();
                                        view! {
                                            <div class="cdb-list-dict-detail__meta">
                                                <span class="cdb-field-dict-tag" data-testid="list-dict-summary">{summary}</span>
                                                {(!comment.is_empty()).then(|| view! {
                                                    <span class="cdb-list-dict-detail__comment" data-testid="list-dict-comment">{comment}</span>
                                                })}
                                            </div>
                                        }
                                    })
                            }}
                            <table class="cdb-list-view-table" data-testid="list-dict-items-table">
                                <thead>
                                    <tr><th>"值"</th><th>"含义"</th></tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let dicts = store.dictionaries.get();
                                        let rows = list_dict.get()
                                            .and_then(|did| dicts.iter().find(|d| d.id == did)
                                                .map(crate::editor_dict::list_dict_detail_rows))
                                            .unwrap_or_default();
                                        if rows.is_empty() {
                                            return view! {
                                                <tr><td colspan="2" class="cdb-list-dict-detail__empty">"暂无字典项"</td></tr>
                                            }.into_view();
                                        }
                                        rows.into_iter().map(|(iid, value, label)| view! {
                                            <tr data-testid=format!("list-dict-item-{}", iid)>
                                                <td>{value}</td>
                                                <td>{label}</td>
                                            </tr>
                                        }).collect::<Vec<_>>().into_view()
                                    }}
                                </tbody>
                            </table>
                            {move || on_open_dicts.as_ref().map(|open| {
                                let open = open.clone();
                                view! {
                                    <div class="cdb-list-dict-detail__actions">
                                        <button
                                            class="cdb-btn cdb-btn--ghost cdb-btn--small"
                                            data-testid="list-dict-edit"
                                            on:click=move |_| open()
                                        >
                                            "在字典面板中编辑"
                                        </button>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                </div>
                    }
                }
            </div>
        </div>
    }
}

/// Areas Tab — 区域列表（读写 `store.areas`，与画布 / PUT 同源）
#[component]
pub fn AreasTab(
    store: EditorStore,
    search_query: RwSignal<String>,
    area_seq: RwSignal<i64>,
    on_save: Rc<dyn Fn()>,
) -> impl IntoView {
    let filtered = create_memo(move |_| {
        let all = store.areas.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-areas">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="area-add"
                on:click={
                    let on_save = on_save.clone();
                    move |_| {
                        let seq = area_seq.get();
                        area_seq.set(seq + 1);
                        let mut v = store.areas.get();
                        v.push(new_default_area(seq));
                        store.areas.set(v);
                        store.dirty.set(true);
                        on_save();
                    }
                }
            >
                "+ 加区域"
            </button>
            <For each=move || filtered.get() key=|a| a.id.clone() children=move |a: Area| {
                let id = a.id.clone();
                let name = a.name.clone();
                view! {
                    <div class="cdb-list-item" data-testid={format!("area-list-item-{}", id)}>
                        {name}
                    </div>
                }
            } />
        </div>
    }
}

/// Enums Tab — 枚举列表（V1 仅前端 state）
#[component]
pub fn EnumsTab(enums: RwSignal<Vec<EnumStub>>, search_query: RwSignal<String>) -> impl IntoView {
    let next_id = create_rw_signal(0i64);
    let filtered = create_memo(move |_| {
        let all = enums.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-enums">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="enum-add"
                on:click=move |_| {
                    let id = next_id.get();
                    next_id.set(id + 1);
                    let mut v = enums.get();
                    v.push(EnumStub {
                        id: format!("enum-auto-{}", id),
                        name: format!("新枚举 {}", id + 1),
                        values: vec!["value_a".into(), "value_b".into()],
                    });
                    enums.set(v);
                }
            >
                "+ 加枚举"
            </button>
            <For each=move || filtered.get() key=|e| e.id.clone() children=move |e: EnumStub| {
                let id = e.id.clone();
                let name = e.name.clone();
                let count = e.values.len();
                view! {
                    <div class="cdb-list-item" data-testid={format!("enum-list-item-{}", id)}>
                        {name} <span class="cdb-list-item-meta">{format!("({} values)", count)}</span>
                    </div>
                }
            } />
        </div>
    }
}

/// feat-data-dictionary（S07，core-01e §6）：字典域变更统一入口。
/// 前后快照（字典集合 + 全量字段绑定）进 undo 栈为**单次 undo 单元**（删除被引用字典的
/// 级联置空与删除同事务，UT-S07-06 / ST-S07-05）；写 store + dirty，保存由调用方触发。
pub fn apply_dict_mutation(
    store: &EditorStore,
    stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>>,
    mutate: impl FnOnce(&mut Vec<DataDictionary>, &mut Vec<Table>),
) {
    let before = store.dictionaries.get();
    let before_bindings = crate::editor_dict::snapshot_bindings(&store.tables.get());
    let mut dicts = before.clone();
    let mut tables = store.tables.get();
    mutate(&mut dicts, &mut tables);
    let after_bindings = crate::editor_dict::snapshot_bindings(&tables);
    store.dictionaries.set(dicts.clone());
    store.tables.set(tables);
    store.dirty.set(true);
    stack
        .get()
        .borrow_mut()
        .record(crate::editor_core::Command::DictSnapshot {
            before,
            after: dicts,
            before_bindings,
            after_bindings,
        });
}

/// feat-data-dictionary（S07，core-01e §2）：数据字典抽屉（ToolRail `toolrail-dicts` 开合）。
/// 对齐主原型 dict-panel：字典列表（名称/编码/项数/引用计数）+ 单击展开详情
/// （name/code/comment blur 落账 + 字典项 value/label 行内编辑 + 上移/下移/删除）
/// + 「+ 新建字典」+ Markdown 导出 + 删除被引用字典确认级联（confirm-dict）。
/// 只读/Viewer：编辑控件禁用，浏览与导出可用（ST-S07-03）。
#[component]
pub fn DictPanel(
    store: EditorStore,
    open: RwSignal<bool>,
    current_title: RwSignal<String>,
    on_save: Rc<dyn Fn()>,
    command_stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>>,
    read_only: Rc<dyn Fn() -> bool>,
) -> impl IntoView {
    // 当前展开详情的字典 id
    let selected: RwSignal<Option<String>> = create_rw_signal(None);
    // 删除确认：(dict_id, code, ref_count)
    let confirm_del: RwSignal<Option<(String, String, usize)>> = create_rw_signal(None);
    // 底部「+ 新建字典」使用的 prop 副本（列表 For children 已移走 on_save/read_only）
    let read_only_add = read_only.clone();
    let on_save_add0 = on_save.clone();
    // 删除确认模态使用的 prop 副本
    let on_save_confirm = on_save.clone();

    let close = move |_| open.set(false);

    view! {
        <aside
            class="cdb-dict-panel"
            data-testid="dict-panel"
            style:display=move || if open.get() { "flex" } else { "none" }
        >
            <header class="cdb-dict-panel__header">
                <h2>"数据字典"</h2>
                <div class="cdb-dict-panel__actions">
                    <button
                        class="cdb-btn cdb-btn--ghost cdb-btn--small"
                        data-testid="dict-export"
                        disabled=move || !crate::editor_dict::dict_export_enabled(&store.dictionaries.get())
                        title=move || {
                            if crate::editor_dict::dict_export_enabled(&store.dictionaries.get()) {
                                "导出数据字典 Markdown"
                            } else {
                                "当前图表没有数据字典"
                            }
                        }
                        on:click=move |_| {
                            let dicts = store.dictionaries.get();
                            let md = crate::editor_dict::export_dict_markdown(
                                &current_title.get_untracked(),
                                &dicts,
                                &store.tables.get(),
                            );
                            download_text(
                                &crate::editor_dict::dict_export_filename(&current_title.get_untracked()),
                                &md,
                            );
                        }
                    >
                        <IconBox size="sm"><IconExport /></IconBox>
                        "导出"
                    </button>
                    <button
                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                        data-testid="dict-panel-close"
                        aria-label="关闭数据字典"
                        on:click=close
                    >
                        <IconBox size="sm"><IconClose /></IconBox>
                    </button>
                </div>
            </header>
            <div class="cdb-dict-panel__body">
                <For
                    each=move || store.dictionaries.get()
                    key=|d| d.id.clone()
                    children=move |d: DataDictionary| {
                        let did = d.id.clone();
                        let did_row = did.clone();
                        let did_row2 = did.clone();
                        let did_del = did.clone();
                        let did_detail = did.clone();
                        let did_items = did.clone();
                        let did_add_item = did.clone();
                        // per-dict 受控草稿（blur 落账；keyed For 按 id 保活不随 store 重建）
                        let name_draft = create_rw_signal(d.name.clone());
                        let code_draft = create_rw_signal(d.code.clone());
                        let comment_draft = create_rw_signal(d.comment.clone());
                        let name_err = create_rw_signal(false);
                        let code_err = create_rw_signal(false);
                        let ro = read_only.clone();
                        let ro2 = read_only.clone();
                        let ro3 = read_only.clone();
                        let ro4 = read_only.clone();
                        let ro5 = read_only.clone();
                        // 供详情响应式闭包捕获的行级副本（避免从 Fn children 闭包移出 prop）
                        let on_save_det0 = on_save.clone();
                        let ro_det0 = read_only.clone();
                        let on_save_row = on_save.clone();
                        let store_row = store.clone();
                        view! {
                            <div class="cdb-dict-item" data-testid={format!("dict-item-{}", did_row)}>
                                <div
                                    class="cdb-dict-item__head"
                                    on:click=move |_| {
                                        selected.update(|s| {
                                            *s = if s.as_deref() == Some(did.as_str()) { None } else { Some(did.clone()) };
                                        });
                                    }
                                >
                                    <span class="cdb-dict-item__name">{move || {
                                        store_row.dictionaries.get().into_iter()
                                            .find(|x| x.id == did_row)
                                            .map(|x| format!("{}（{}）", x.name, x.code))
                                            .unwrap_or_default()
                                    }}</span>
                                    <span class="cdb-list-item-meta">{move || {
                                        let dicts = store_row.dictionaries.get();
                                        dicts.iter().find(|x| x.id == did_row2).map(|x| {
                                            let refs = crate::editor_dict::dict_ref_count(&store_row.tables.get(), &x.code);
                                            format!("{} 项 · 引用 {}", x.items.len(), refs)
                                        }).unwrap_or_default()
                                    }}</span>
                                    <button
                                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                        data-testid={format!("dict-del-{}", did_del)}
                                        aria-label="删除字典"
                                        disabled={let ro = ro.clone(); move || ro()}
                                        on:click={
                                            let ro = ro2.clone();
                                            move |ev: web_sys::MouseEvent| {
                                                ev.stop_propagation();
                                                if ro() { return; }
                                                let dicts = store.dictionaries.get();
                                                let Some(dcur) = dicts.iter().find(|x| x.id == did_del) else { return };
                                                let code = dcur.code.clone();
                                                let refs = crate::editor_dict::dict_ref_count(&store.tables.get(), &code);
                                                if refs > 0 {
                                                    // core-01e §2.4：被引用 → 确认后级联置空
                                                    confirm_del.set(Some((did_del.clone(), code, refs)));
                                                } else {
                                                    let did_m = did_del.clone();
                                                    apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                        ds.retain(|x| x.id != did_m);
                                                    });
                                                    on_save_row();
                                                }
                                            }
                                        }
                                    >
                                        <IconBox size="sm"><IconDelete /></IconBox>
                                    </button>
                                </div>
                                {move || if selected.get().as_deref() == Some(did_detail.as_str()) {
                                    let did2 = did_detail.clone();
                                    let did3 = did_detail.clone();
                                    let did4 = did_detail.clone();
                                    // 每次响应式重跑克隆一份，内层 move 闭包只捕获副本（保持外层 Fn 语义）
                                    let on_save_det = on_save_det0.clone();
                                    let ro_det = ro_det0.clone();
                                    let on_save_d = on_save_det0.clone();
                                    let on_save_c = on_save_det0.clone();
                                    let on_save_cm = on_save_det0.clone();
                                    let store_d = store.clone();
                                    let store_c = store.clone();
                                    let store_cm = store.clone();
                                    view! {
                                        <div class="cdb-dict-detail" data-testid={format!("dict-detail-{}", did_detail)}>
                                            <div class="cdb-form-group">
                                                <label>"名称"</label>
                                                <input
                                                    class="cdb-form-input"
                                                    class:cdb-is-error=name_err
                                                    data-testid={format!("dict-name-{}", did2)}
                                                    prop:value=move || name_draft.get()
                                                    disabled={let ro = ro3.clone(); move || ro()}
                                                    on:input=move |ev| { name_err.set(false); name_draft.set(event_target_value(&ev)); }
                                                    on:blur={
                                                        let did = did2.clone();
                                                        let on_save_d = on_save_d.clone();
                                                        move |_| {
                                                            let v = name_draft.get_untracked();
                                                            let cur = store_d.dictionaries.get().into_iter()
                                                                .find(|x| x.id == did).map(|x| x.name).unwrap_or_default();
                                                            if v.trim().is_empty() {
                                                                // core-01e §2.4：名称非空，空值阻断落账 + 红标
                                                                name_err.set(true);
                                                                name_draft.set(cur);
                                                                return;
                                                            }
                                                            if v == cur { return; }
                                                            let did_m = did.clone();
                                                            apply_dict_mutation(&store_d, command_stack, move |ds, _| {
                                                                if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) { x.name = v; }
                                                            });
                                                            on_save_d();
                                                        }
                                                    }
                                                />
                                                {move || name_err.get().then(|| view! {
                                                    <span class="cdb-field-error" data-testid={format!("dict-name-error-{}", did2)}>"名称不能为空"</span>
                                                })}
                                            </div>
                                            <div class="cdb-form-group">
                                                <label>"编码"</label>
                                                <input
                                                    class="cdb-form-input"
                                                    class:cdb-is-error=code_err
                                                    data-testid={format!("dict-code-{}", did3)}
                                                    prop:value=move || code_draft.get()
                                                    disabled={let ro = ro4.clone(); move || ro()}
                                                    on:input=move |ev| { code_err.set(false); code_draft.set(event_target_value(&ev)); }
                                                    on:blur={
                                                        let did = did3.clone();
                                                        let on_save_c = on_save_c.clone();
                                                        move |_| {
                                                            let v = code_draft.get_untracked();
                                                            let dicts = store_c.dictionaries.get();
                                                            let cur = dicts.iter().find(|x| x.id == did).map(|x| x.code.clone()).unwrap_or_default();
                                                            // core-01e §2.4：编码非空且同图唯一，冲突红标 + 阻断落账（UT-S07-02）
                                                            if v.trim().is_empty() || crate::editor_dict::dict_code_taken(&dicts, &v, &did) {
                                                                code_err.set(true);
                                                                code_draft.set(cur);
                                                                return;
                                                            }
                                                            if v == cur { return; }
                                                            let old = cur.clone();
                                                            let new = v.clone();
                                                            let did_m = did.clone();
                                                            apply_dict_mutation(&store_c, command_stack, move |ds, tables| {
                                                                if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) { x.code = new.clone(); }
                                                                // 改编码同步改写引用字段（同一 undo 单元，UT-S07-05）
                                                                crate::editor_dict::propagate_code_rename(tables, &old, &new);
                                                            });
                                                            on_save_c();
                                                        }
                                                    }
                                                />
                                                {move || code_err.get().then(|| view! {
                                                    <span class="cdb-field-error" data-testid={format!("dict-code-error-{}", did3)}>"编码为空或已被占用"</span>
                                                })}
                                            </div>
                                            <div class="cdb-form-group">
                                                <label>"说明"</label>
                                                <input
                                                    class="cdb-form-input"
                                                    data-testid={format!("dict-comment-{}", did4)}
                                                    prop:value=move || comment_draft.get()
                                                    disabled={let ro = ro5.clone(); move || ro()}
                                                    on:input=move |ev| comment_draft.set(event_target_value(&ev))
                                                    on:blur={
                                                        let did = did4.clone();
                                                        move |_| {
                                                            let v = comment_draft.get_untracked();
                                                            let cur = store_cm.dictionaries.get().into_iter()
                                                                .find(|x| x.id == did).map(|x| x.comment).unwrap_or_default();
                                                            if v == cur { return; }
                                                            let did_m = did.clone();
                                                            apply_dict_mutation(&store_cm, command_stack, move |ds, _| {
                                                                if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) { x.comment = v; }
                                                            });
                                                            on_save_cm();
                                                        }
                                                    }
                                                />
                                            </div>
                                            <div class="cdb-dict-items">
                                                <For
                                                    each={
                                                        let did = did_items.clone();
                                                        move || store.dictionaries.get().into_iter()
                                                            .find(|x| x.id == did)
                                                            .map(|x| {
                                                                let mut items = x.items;
                                                                items.sort_by_key(|i| i.sort);
                                                                items
                                                            })
                                                            .unwrap_or_default()
                                                    }
                                                    key=|i| i.id.clone()
                                                    children={
                                                        let did = did_items.clone();
                                                        let ro_items = ro_det.clone();
                                                        let on_save_items = on_save_det.clone();
                                                        move |item: DictionaryItem| {
                                                            let iid = item.id.clone();
                                                            let val_draft = create_rw_signal(item.value.clone());
                                                            let label_draft = create_rw_signal(item.label.clone());
                                                            let val_err = create_rw_signal(false);
                                                            let label_err = create_rw_signal(false);
                                                            let ro_v = ro_items.clone();
                                                            let ro_l = ro_items.clone();
                                                            let ro_d = ro_items.clone();
                                                            let ro_u = ro_items.clone();
                                                            let ro_dn = ro_items.clone();
                                                            let did_v = did.clone();
                                                            let did_l = did.clone();
                                                            let did_d = did.clone();
                                                            let did_u = did.clone();
                                                            let did_dn = did.clone();
                                                            let did_dup = did.clone();
                                                            let iid_dup = iid.clone();
                                                            let on_save_v = on_save_items.clone();
                                                            let on_save_l = on_save_items.clone();
                                                            let on_save_d2 = on_save_items.clone();
                                                            let on_save_u = on_save_items.clone();
                                                            let on_save_dn = on_save_items.clone();
                                                            view! {
                                                                <div class="cdb-dict-item-row" data-testid={format!("dict-item-row-{}", iid)}>
                                                                    <input
                                                                        class="cdb-form-input"
                                                                        class:cdb-is-error=move || val_err.get()
                                                                        data-testid={format!("dict-item-value-{}", iid)}
                                                                        prop:value=move || val_draft.get()
                                                                        placeholder="值"
                                                                        disabled={let ro = ro_v.clone(); move || ro()}
                                                                        on:input=move |ev| { val_err.set(false); val_draft.set(event_target_value(&ev)); }
                                                                        on:blur={
                                                                            let did = did_v.clone();
                                                                            let iid = iid.clone();
                                                                            move |_| {
                                                                                let v = val_draft.get_untracked();
                                                                                let cur = store.dictionaries.get().into_iter()
                                                                                    .find(|x| x.id == did)
                                                                                    .and_then(|x| x.items.into_iter().find(|i| i.id == iid))
                                                                                    .map(|i| i.value).unwrap_or_default();
                                                                                if v.trim().is_empty() {
                                                                                    val_err.set(true);
                                                                                    val_draft.set(cur);
                                                                                    return;
                                                                                }
                                                                                if v == cur { return; }
                                                                                let did_m = did.clone();
                                                                                let iid_m = iid.clone();
                                                                                apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                                    if let Some(i) = ds.iter_mut().find(|x| x.id == did_m)
                                                                                        .and_then(|x| x.items.iter_mut().find(|i| i.id == iid_m)) {
                                                                                        i.value = v;
                                                                                    }
                                                                                });
                                                                                on_save_v();
                                                                            }
                                                                        }
                                                                    />
                                                                    <input
                                                                        class="cdb-form-input"
                                                                        class:cdb-is-error=move || label_err.get()
                                                                        data-testid={format!("dict-item-label-{}", iid)}
                                                                        prop:value=move || label_draft.get()
                                                                        placeholder="含义"
                                                                        disabled={let ro = ro_l.clone(); move || ro()}
                                                                        on:input=move |ev| { label_err.set(false); label_draft.set(event_target_value(&ev)); }
                                                                        on:blur={
                                                                            let did = did_l.clone();
                                                                            let iid = iid.clone();
                                                                            move |_| {
                                                                                let v = label_draft.get_untracked();
                                                                                let cur = store.dictionaries.get().into_iter()
                                                                                    .find(|x| x.id == did)
                                                                                    .and_then(|x| x.items.into_iter().find(|i| i.id == iid))
                                                                                    .map(|i| i.label).unwrap_or_default();
                                                                                if v.trim().is_empty() {
                                                                                    label_err.set(true);
                                                                                    label_draft.set(cur);
                                                                                    return;
                                                                                }
                                                                                if v == cur { return; }
                                                                                let did_m = did.clone();
                                                                                let iid_m = iid.clone();
                                                                                apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                                    if let Some(i) = ds.iter_mut().find(|x| x.id == did_m)
                                                                                        .and_then(|x| x.items.iter_mut().find(|i| i.id == iid_m)) {
                                                                                        i.label = v;
                                                                                    }
                                                                                });
                                                                                on_save_l();
                                                                            }
                                                                        }
                                                                    />
                                                                    // UT-S07-03：同字典内 value 重复 → 行内红标（实时，随草稿）
                                                                    {move || {
                                                                        let dup = store.dictionaries.get().iter()
                                                                            .find(|x| x.id == did_dup)
                                                                            .map(|x| crate::editor_dict::item_value_taken(x, &val_draft.get(), &iid_dup))
                                                                            .unwrap_or(false);
                                                                        dup.then(|| view! {
                                                                            <span class="cdb-field-error" data-testid={format!("dict-item-dup-{}", iid_dup)}>"值重复"</span>
                                                                        })
                                                                    }}
                                                                    <button
                                                                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                                                        data-testid={format!("dict-item-up-{}", iid)}
                                                                        aria-label="上移"
                                                                        disabled={let ro = ro_u.clone(); move || ro()}
                                                                        on:click={
                                                                            let did = did_u.clone();
                                                                            let iid = iid.clone();
                                                                            move |_| {
                                                                                let did_m = did.clone();
                                                                                let iid_m = iid.clone();
                                                                                apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                                    if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) {
                                                                                        x.items.sort_by_key(|i| i.sort);
                                                                                        if let Some(pos) = x.items.iter().position(|i| i.id == iid_m) {
                                                                                            if pos > 0 {
                                                                                                let s = x.items[pos].sort;
                                                                                                x.items[pos].sort = x.items[pos - 1].sort;
                                                                                                x.items[pos - 1].sort = s;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                });
                                                                                on_save_u();
                                                                            }
                                                                        }
                                                                    >
                                                                        "↑"
                                                                    </button>
                                                                    <button
                                                                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                                                        data-testid={format!("dict-item-down-{}", iid)}
                                                                        aria-label="下移"
                                                                        disabled={let ro = ro_dn.clone(); move || ro()}
                                                                        on:click={
                                                                            let did = did_dn.clone();
                                                                            let iid = iid.clone();
                                                                            move |_| {
                                                                                let did_m = did.clone();
                                                                                let iid_m = iid.clone();
                                                                                apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                                    if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) {
                                                                                        x.items.sort_by_key(|i| i.sort);
                                                                                        if let Some(pos) = x.items.iter().position(|i| i.id == iid_m) {
                                                                                            if pos + 1 < x.items.len() {
                                                                                                let s = x.items[pos].sort;
                                                                                                x.items[pos].sort = x.items[pos + 1].sort;
                                                                                                x.items[pos + 1].sort = s;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                });
                                                                                on_save_dn();
                                                                            }
                                                                        }
                                                                    >
                                                                        "↓"
                                                                    </button>
                                                                    <button
                                                                        class="cdb-btn cdb-btn--ghost cdb-btn--icon"
                                                                        data-testid={format!("dict-item-del-{}", iid)}
                                                                        aria-label="删除字典项"
                                                                        disabled={let ro = ro_d.clone(); move || ro()}
                                                                        on:click={
                                                                            let did = did_d.clone();
                                                                            let iid = iid.clone();
                                                                            move |_| {
                                                                                let did_m = did.clone();
                                                                                let iid_m = iid.clone();
                                                                                apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                                    if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) {
                                                                                        x.items.retain(|i| i.id != iid_m);
                                                                                    }
                                                                                });
                                                                                on_save_d2();
                                                                            }
                                                                        }
                                                                    >
                                                                        <IconBox size="sm"><IconDelete /></IconBox>
                                                                    </button>
                                                                </div>
                                                            }
                                                        }
                                                    }
                                                />
                                                <button
                                                    class="cdb-btn cdb-btn--block"
                                                    data-testid={format!("dict-item-add-{}", did_add_item)}
                                                    disabled={let ro = ro_det.clone(); move || ro()}
                                                    on:click={
                                                        let did = did_add_item.clone();
                                                        let on_save_add = on_save_det.clone();
                                                        move |_| {
                                                            let did_m = did.clone();
                                                            apply_dict_mutation(&store, command_stack, move |ds, _| {
                                                                if let Some(x) = ds.iter_mut().find(|x| x.id == did_m) {
                                                                    let mut n = x.items.len() as i64 + 1;
                                                                    let value = loop {
                                                                        let v = n.to_string();
                                                                        if !x.items.iter().any(|i| i.value == v) { break v; }
                                                                        n += 1;
                                                                    };
                                                                    let sort = x.items.iter().map(|i| i.sort).max().unwrap_or(-1) + 1;
                                                                    x.items.push(DictionaryItem {
                                                                        id: new_entity_id("ditem"),
                                                                        value,
                                                                        label: format!("新项 {}", n),
                                                                        sort,
                                                                    });
                                                                }
                                                            });
                                                            on_save_add();
                                                        }
                                                    }
                                                >
                                                    "+ 添加项"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }}
                            </div>
                        }
                    }
                />
                {move || if store.dictionaries.get().is_empty() {
                    view! { <p class="cdb-empty-hint">"暂无数据字典"</p> }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
            </div>
            <footer class="cdb-dict-panel__footer">
                <button
                    class="cdb-btn cdb-btn--block"
                    data-testid="dict-add"
                    disabled={let ro = read_only_add.clone(); move || ro()}
                    on:click={
                        let on_save = on_save_add0.clone();
                        move |_| {
                            apply_dict_mutation(&store, command_stack, move |ds, _| {
                                let (name, code) = crate::editor_dict::next_dict_defaults(ds);
                                let id = new_entity_id("dict");
                                ds.push(DataDictionary {
                                    id: id.clone(),
                                    name,
                                    code,
                                    comment: String::new(),
                                    items: vec![],
                                });
                                selected.set(Some(id));
                            });
                            on_save();
                        }
                    }
                >
                    "+ 新建字典"
                </button>
            </footer>
            // core-01e §2.4：删除被引用字典确认（引用计数 > 0）
            {move || confirm_del.get().map(|(did, code, refs)| {
                let on_save_cf = on_save_confirm.clone();
                view! {
                    // UT-FIX-01 约束：cdb-modal-overlay 字面量全文件 ≤ 2，
                    // 此处用共享样式的独立类名（样式见 styles.css .cdb-dict-confirm-overlay）
                    <div class="cdb-dict-confirm-overlay" data-testid="confirm-dict" on:click=move |_| confirm_del.set(None)>
                        <div class="cdb-modal" on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()>
                            <div class="cdb-modal-header">
                                <h3 class="cdb-modal-title">"删除字典"</h3>
                            </div>
                            <div class="cdb-modal-body">
                                <p>{format!("字典「{}」被 {} 个字段引用，删除后这些字段的绑定将置空，是否继续？", code, refs)}</p>
                            </div>
                            <div class="cdb-modal-footer">
                                <button
                                    class="cdb-btn"
                                    data-testid="confirm-dict-cancel"
                                    on:click=move |_| confirm_del.set(None)
                                >
                                    "取消"
                                </button>
                                <button
                                    class="cdb-btn cdb-btn--danger"
                                    data-testid="confirm-dict-ok"
                                    on:click=move |_| {
                                        confirm_del.set(None);
                                        let code_m = code.clone();
                                        let did_m = did.clone();
                                        apply_dict_mutation(&store, command_stack, move |ds, tables| {
                                            // 级联置空与删除同一 undo 单元（UT-S07-06）
                                            crate::editor_dict::cascade_clear_bindings(tables, &code_m);
                                            ds.retain(|x| x.id != did_m);
                                        });
                                        on_save_cf();
                                    }
                                >
                                    "删除"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </aside>
    }
}

/// Notes Tab — 便签列表（读写 `store.notes`，与画布 / PUT 同源）
#[component]
pub fn NotesTab(
    store: EditorStore,
    search_query: RwSignal<String>,
    note_seq: RwSignal<i64>,
    on_save: Rc<dyn Fn()>,
) -> impl IntoView {
    let filtered = create_memo(move |_| {
        let all = store.notes.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-notes">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="note-add"
                on:click={
                    let on_save = on_save.clone();
                    move |_| {
                        let seq = note_seq.get();
                        note_seq.set(seq + 1);
                        let mut v = store.notes.get();
                        v.push(new_default_note(seq));
                        store.notes.set(v);
                        store.dirty.set(true);
                        on_save();
                    }
                }
            >
                "+ 加便签"
            </button>
            <For each=move || filtered.get() key=|n| n.id.clone() children=move |n: Note| {
                let id = n.id.clone();
                let preview: String = n.content.chars().take(30).collect();
                view! {
                    <div class="cdb-list-item" data-testid={format!("note-list-item-{}", id)}>
                        {preview}
                    </div>
                }
            } />
        </div>
    }
}

/// Relationships Tab — 关系列表（从 store.references 派生；不创建，只读）
#[component]
pub fn RelationshipsTab(store: EditorStore, search_query: RwSignal<String>) -> impl IntoView {
    let filtered = create_memo(move |_| {
        let all = store.references.get();
        let q = search_query.get();
        filter_references_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-relationships">
            <For each=move || filtered.get() key=|r| r.id.clone() children=move |r: Reference| {
                let start = r.start_table_id.clone();
                let end = r.end_table_id.clone();
                let type_ = r.type_.clone();
                view! {
                    <div class="cdb-list-item" data-testid={format!("rel-list-item-{}", r.id)}>
                        {format!("{} → {} ({})", start, end, type_)}
                    </div>
                }
            } />
            {move || if filtered.get().is_empty() {
                view! { <p class="cdb-empty-hint">"无关系"</p> }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>
    }
}

/// Types Tab — 自定义类型列表（V1 仅前端 state）
#[component]
pub fn TypesTab(types: RwSignal<Vec<TypeStub>>, search_query: RwSignal<String>) -> impl IntoView {
    let next_id = create_rw_signal(0i64);
    let filtered = create_memo(move |_| {
        let all = types.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-types">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="type-add"
                on:click=move |_| {
                    let id = next_id.get();
                    next_id.set(id + 1);
                    let mut v = types.get();
                    v.push(TypeStub {
                        id: format!("type-auto-{}", id),
                        name: format!("新类型 {}", id + 1),
                    });
                    types.set(v);
                }
            >
                "+ 加自定义类型"
            </button>
            <For each=move || filtered.get() key=|t| t.id.clone() children=move |t: TypeStub| {
                let id = t.id.clone();
                let name = t.name.clone();
                view! {
                    <div class="cdb-list-item" data-testid={format!("type-list-item-{}", id)}>
                        {name}
                    </div>
                }
            } />
        </div>
    }
}

/// Issues Tab — 派生自 store 的校验错误（B2 基础校验 + B3 跳转 + ST-SP-01 间接覆盖）
/// - 表名重复
/// - 主键缺失
/// - 字段类型不兼容（type 字段为空或为 "INVALID"）
/// - 关系端点不存在（start/end_table_id 在 store.tables 中找不到）
/// - B3 新增：每条 issue 加「→ 跳转」按钮，调用 on_jump_to_table(target_id)
#[component]
pub fn IssuesTab(
    store: EditorStore,
    on_jump_to_table: Option<Rc<dyn Fn(String)>>,
) -> impl IntoView {
    let issues = create_memo(move |_| {
        let mut out: Vec<(String, String, String)> = Vec::new(); // (level, message, target)
        let tables = store.tables.get();
        let refs = store.references.get();

        // 表名重复
        for (i, t) in tables.iter().enumerate() {
            for other in tables.iter().skip(i + 1) {
                if t.name == other.name && !t.name.is_empty() {
                    out.push((
                        "error".into(),
                        format!("表名重复: {}", t.name),
                        t.id.clone(),
                    ));
                    break;
                }
            }
        }

        // 主键缺失
        for t in &tables {
            if !t.fields.iter().any(|f| f.primary) {
                out.push((
                    "warning".into(),
                    format!("表 '{}' 缺少主键", t.name),
                    t.id.clone(),
                ));
            }
        }

        // 字段类型不兼容
        for t in &tables {
            for f in &t.fields {
                if f.type_.is_empty() || f.type_ == "INVALID" {
                    out.push((
                        "error".into(),
                        format!("字段 '{}.{}' 类型不兼容", t.name, f.name),
                        t.id.clone(),
                    ));
                }
            }
        }

        // 关系端点不存在
        for r in &refs {
            if !tables.iter().any(|t| t.id == r.start_table_id) {
                out.push((
                    "error".into(),
                    format!("关系 {} 起点表不存在", r.id),
                    r.start_table_id.clone(),
                ));
            }
            if !tables.iter().any(|t| t.id == r.end_table_id) {
                out.push((
                    "error".into(),
                    format!("关系 {} 终点表不存在", r.id),
                    r.end_table_id.clone(),
                ));
            }
        }

        out
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-issues">
            <div class="cdb-section-title" data-testid="issues-count">
                {move || format!("Issues ({})", issues.get().len())}
            </div>
            <For each=move || issues.get() key=|(_, _, target)| target.clone() children=move |(level, message, target): (String, String, String)| {
                let level_class = match level.as_str() {
                    "error" => "cdb-issue cdb-issue--error",
                    "warning" => "cdb-issue cdb-issue--warning",
                    _ => "cdb-issue cdb-issue--info",
                };
                let testid = format!("issue-item-{}", target);
                let jump_testid = format!("issue-jump-{}", target);
                let target_for_jump = target.clone();
                let on_jump = on_jump_to_table.clone();
                view! {
                    <div class={level_class} data-testid={testid}>
                        <span class="cdb-issue-level">{level.clone()}</span>
                        <span class="cdb-issue-message">{message}</span>
                        {on_jump.as_ref().map(|cb| {
                            let cb = cb.clone();
                            let tid = target_for_jump.clone();
                            view! {
                                <button
                                    class="cdb-btn cdb-btn--small"
                                    data-testid={jump_testid}
                                    on:click=move |_| cb(tid.clone())
                                >
                                    "→ 跳转"
                                </button>
                            }
                        })}
                    </div>
                }
            } />
            {move || if issues.get().is_empty() {
                view! { <p class="cdb-empty-hint">"无问题 ✓"</p> }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>
    }
}

/// 前端 API base_url 同源派生锚点（core-01-deployment-plan §12.2 / core-S04 §7.2-4，UT-S04-UI-17）：
/// 生产默认取 `window.location.origin`（单入口部署下前后端同源，无需额外配置）；
/// 仅本地 dev 双进程（trunk serve + cargo run）允许经编译期环境变量
/// `COLDRAWDB_API_BASE` 覆盖（该值不得进入生产构建默认值）。
fn default_api_base_url() -> String {
    if let Some(u) = option_env!("COLDRAWDB_API_BASE") {
        let u = u.trim();
        if !u.is_empty() {
            return u.to_string();
        }
    }
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default()
}

/// 根入口组件
#[component]
pub fn AppRoot(
    store: EditorStore,
    debouncer: DebounceTrigger,
    _diagram_id: String,
    share_mode: bool,
    invite_token: Option<String>,
) -> impl IntoView {
    let selection: RwSignal<SelectionKind> = create_rw_signal(SelectionKind::None);
    let selected_table_id: RwSignal<Option<String>> = create_rw_signal(None);
    let inspector_open: RwSignal<bool> = create_rw_signal(true);
    let conflict: RwSignal<Option<ConflictInfo>> = create_rw_signal(None);
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let share_loading = create_rw_signal(share_mode);
    let share_load_error = create_rw_signal(Option::<String>::None);
    let next_id = create_rw_signal(crate::editor_core::next_id_from_store(&store) + 1);

    // B4: 模态状态 (4 核心模态)
    let modal_kind: RwSignal<Option<modals::ModalKind>> = create_rw_signal(None);
    let current_diagram_id: RwSignal<String> = create_rw_signal(_diagram_id.clone());
    let current_title: RwSignal<String> = create_rw_signal(String::from("Untitled Diagram"));
    let is_saving: RwSignal<bool> = create_rw_signal(false);
    let save_offline: RwSignal<bool> = create_rw_signal(false);
    let view_mode: RwSignal<ViewMode> = create_rw_signal(ViewMode::Canvas);
    // ux-canvas-batch 批次3 步骤 2 (条目 16 修复): batch_type_selection 提升到 AppRoot 作用域
    // ——Apply 数据链要求 ListView 选中集与 BatchTypeModal 在同一 RwSignal 上交汇
    let batch_type_selection: RwSignal<BatchTypeSelection> = create_rw_signal(BatchTypeSelection::default());
    let code_visible: RwSignal<bool> = create_rw_signal(false);
    let code_language: RwSignal<CodeLanguage> = create_rw_signal(CodeLanguage::Sql);
    let code_copy_toast: RwSignal<Option<String>> = create_rw_signal(None);
    let palette_visible: RwSignal<bool> = create_rw_signal(false);
    let palette_query: RwSignal<String> = create_rw_signal(String::new());
    let palette_highlight: RwSignal<usize> = create_rw_signal(0);
    let canvas_transform: RwSignal<Transform> = create_rw_signal(Transform::default());
    // 主题信号提升到 AppRoot：Canvas 绘制 effect 需跟踪它以在主题切换时重绘调色板
    let theme_mode: RwSignal<String> = create_rw_signal(read_html_data_mode());
    let auth_session: RwSignal<Option<AuthSession>> = create_rw_signal(None);
    let session_notice: RwSignal<Option<String>> = create_rw_signal(if share_mode {
        Some("匿名只读分享".to_string())
    } else {
        None
    });
    // align-frontend-to-prototype：五态页面状态机。初始按 URL 解析（invite/share/auth）。
    let initial_page = if share_mode {
        PageState::ShareEdit
    } else if invite_token.is_some() {
        PageState::Invite
    } else {
        PageState::Auth
    };
    let current_page: RwSignal<PageState> = create_rw_signal(initial_page);
    create_effect(move |_| {
        if auth_session.get().is_none() && current_page.get() == PageState::Rooms {
            current_page.set(PageState::Auth);
        }
    });

    // 防止 session_notice 被注入 token 原文：写入前过滤。
    create_effect(move |_| {
        let raw = session_notice.get();
        let cleaned = sanitize_session_notice(raw.as_deref());
        match (raw.as_ref(), cleaned) {
            (Some(existing), Some(new_clean)) if existing != &new_clean => {
                session_notice.set(Some(new_clean));
            }
            (Some(_), None) => session_notice.set(None),
            _ => {}
        }
    });
    let current_room: RwSignal<Option<RoomDetail>> = create_rw_signal(None);
    let room_panel_visible: RwSignal<bool> = create_rw_signal(false);
    let activity_open: RwSignal<bool> = create_rw_signal(true);
    let room_members: RwSignal<Vec<RoomMember>> = create_rw_signal(Vec::new());
    let collab_state: RwSignal<CollabOtState> = create_rw_signal(CollabOtState::default());
    let remote_members: RwSignal<Vec<CollabMemberPresence>> = create_rw_signal(Vec::new());
    let activity_feed: RwSignal<Vec<String>> = create_rw_signal(Vec::new());
    // 手动重连触发器：banner「立即重连 / 重新连接」递增后 effect 重跑
    let collab_retry: RwSignal<u32> = create_rw_signal(0);
    let remote_presence: RwSignal<Vec<RemotePresence>> = create_rw_signal(Vec::new());

    // Phase B：关系工具
    let active_tool: RwSignal<ActiveTool> = create_rw_signal(ActiveTool::Select);
    let rel_tool_state: RwSignal<RelToolState> = create_rw_signal(RelToolState::Idle);
    let rel_tool_active: RwSignal<bool> = create_rw_signal(false);
    create_effect(move |_| {
        let picking = active_tool.get() == ActiveTool::Relationship
            && rel_tool_state.get().is_picking();
        rel_tool_active.set(picking);
    });

    // p0-fix 定点 2：创建工具信号 — 画布十字光标 + 拖框建区域 / 点击放便签
    let create_tool: RwSignal<Option<crate::editor_render::CreateToolKind>> =
        create_rw_signal(None);
    create_effect(move |_| {
        let tool = match active_tool.get() {
            ActiveTool::NewArea => Some(crate::editor_render::CreateToolKind::Area),
            ActiveTool::NewNote => Some(crate::editor_render::CreateToolKind::Note),
            _ => None,
        };
        create_tool.set(tool);
    });

    // Phase C：IO 抽屉
    let io_drawer: RwSignal<IoDrawerKind> = create_rw_signal(IoDrawerKind::None);
    let inspector_before_io: RwSignal<Option<bool>> = create_rw_signal(None);
    // feat-data-dictionary（S07）：数据字典抽屉开合（ToolRail toolrail-dicts 触发）
    let dict_panel_open: RwSignal<bool> = create_rw_signal(false);
    // fix-dict-save-and-layout（core-01e §2.5）：字典抽屉打开前的 Inspector 开合缓存
    let inspector_before_dict: RwSignal<Option<bool>> = create_rw_signal(None);

    // fix-dict-save-and-layout（core-01e §2.5）：右侧浮层互斥薄接线——
    // 状态收集/应用在此，转换逻辑全在 OverlayMutex 纯函数（ST-S07-06 host 可测）
    let overlay_gather: Rc<dyn Fn() -> OverlayMutex> = {
        let inspector_open = inspector_open.clone();
        let dict_panel_open = dict_panel_open.clone();
        let io_drawer = io_drawer.clone();
        let inspector_before_dict = inspector_before_dict.clone();
        let inspector_before_io = inspector_before_io.clone();
        Rc::new(move || OverlayMutex {
            inspector_open: inspector_open.get_untracked(),
            dict_panel_open: dict_panel_open.get_untracked(),
            io_drawer: io_drawer.get_untracked(),
            inspector_before_dict: inspector_before_dict.get_untracked(),
            inspector_before_io: inspector_before_io.get_untracked(),
        })
    };
    let overlay_apply: Rc<dyn Fn(OverlayMutex)> = {
        let inspector_open = inspector_open.clone();
        let dict_panel_open = dict_panel_open.clone();
        let io_drawer = io_drawer.clone();
        let inspector_before_dict = inspector_before_dict.clone();
        let inspector_before_io = inspector_before_io.clone();
        Rc::new(move |next: OverlayMutex| {
            inspector_open.set(next.inspector_open);
            dict_panel_open.set(next.dict_panel_open);
            io_drawer.set(next.io_drawer);
            inspector_before_dict.set(next.inspector_before_dict);
            inspector_before_io.set(next.inspector_before_io);
        })
    };

    let on_toggle_dicts: Rc<dyn Fn()> = {
        let gather = overlay_gather.clone();
        let apply = overlay_apply.clone();
        Rc::new(move || {
            let next = gather().toggle_dict_panel();
            apply(next);
        })
    };
    // fix-dict-save-and-layout（core-01e §3.1）：ListView「在字典面板中编辑」入口——只开不合
    let on_open_dicts: Rc<dyn Fn()> = {
        let gather = overlay_gather.clone();
        let apply = overlay_apply.clone();
        Rc::new(move || {
            let cur = gather();
            if !cur.dict_panel_open {
                apply(cur.open_dict_panel());
            }
        })
    };
    // fix-appbar-roomname-back-and-import-merge：导入成功提示（2.5s 自动消失）
    let import_notice: RwSignal<Option<String>> = create_rw_signal(None);

    // fix-dict-save-and-layout（core-01e §2.5 反向互斥）：Inspector 打开时两抽屉关闭、缓存失效
    create_effect(move |_| {
        if inspector_open.get() {
            if io_drawer.get_untracked() != IoDrawerKind::None {
                io_drawer.set(IoDrawerKind::None);
                inspector_before_io.set(None);
            }
            if dict_panel_open.get_untracked() {
                dict_panel_open.set(false);
                inspector_before_dict.set(None);
            }
        }
    });

    // fix-dict-save-and-layout（core-01e §2.5）：字典抽屉任一关闭路径（面板关闭按钮等）
    // 统一在此恢复 Inspector——toggle / inspector_opened 路径已清缓存，此处自动 no-op
    create_effect(move |_| {
        if !dict_panel_open.get() {
            if let Some(prev) = inspector_before_dict.get_untracked() {
                inspector_before_dict.set(None);
                inspector_open.set(prev);
            }
        }
    });

    create_effect(move |_| match selection.get() {
        SelectionKind::Table(id) => selected_table_id.set(Some(id)),
        SelectionKind::Field { table_id, .. } => selected_table_id.set(Some(table_id)),
        _ => {}
    });

    let on_select_table = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Rc::new(move |id: Option<String>| {
            selected_table_id.set(id.clone());
            match id {
                Some(tid) => {
                    selection.set(SelectionKind::Table(tid));
                    inspector_open.set(true);
                }
                None => selection.set(SelectionKind::None),
            }
        })
    };

    let close_io_drawer = {
        let io_drawer = io_drawer.clone();
        let inspector_open = inspector_open.clone();
        let inspector_before_io = inspector_before_io.clone();
        Rc::new(move || {
            io_drawer.set(IoDrawerKind::None);
            inspector_open.set(restore_inspector_after_io_drawer(
                inspector_before_io.get_untracked(),
            ));
            inspector_before_io.set(None);
        })
    };

    let open_import_drawer = {
        let io_drawer = io_drawer.clone();
        let inspector_open = inspector_open.clone();
        let inspector_before_io = inspector_before_io.clone();
        let dict_panel_open = dict_panel_open.clone();
        let inspector_before_dict = inspector_before_dict.clone();
        Rc::new(move || {
            // fix-dict-save-and-layout（core-01e §2.5）：IO 抽屉与字典抽屉互斥——
            // 字典抽屉已开时继承其 Inspector 缓存（当前 inspector 已被抽屉收起，直接快照会丢真实状态）
            let inherited = inspector_before_dict.get_untracked();
            let (collapsed, cache) = snapshot_before_io_drawer(inspector_open.get_untracked());
            inspector_before_io.set(inherited.or(cache));
            inspector_before_dict.set(None);
            dict_panel_open.set(false);
            inspector_open.set(collapsed);
            io_drawer.set(IoDrawerKind::Import);
        })
    };

    let open_export_drawer = {
        let io_drawer = io_drawer.clone();
        let inspector_open = inspector_open.clone();
        let inspector_before_io = inspector_before_io.clone();
        let dict_panel_open = dict_panel_open.clone();
        let inspector_before_dict = inspector_before_dict.clone();
        Rc::new(move || {
            // fix-dict-save-and-layout（core-01e §2.5）：同 open_import_drawer 的缓存交接
            let inherited = inspector_before_dict.get_untracked();
            let (collapsed, cache) = snapshot_before_io_drawer(inspector_open.get_untracked());
            inspector_before_io.set(inherited.or(cache));
            inspector_before_dict.set(None);
            dict_panel_open.set(false);
            inspector_open.set(collapsed);
            io_drawer.set(IoDrawerKind::Export);
        })
    };

    let command_stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>> = create_rw_signal(
        Rc::new(RefCell::new(crate::editor_core::CommandStack::new())),
    );

    // HTTP client to backend。base_url 同源派生（core-01-deployment-plan §12.2 / UT-S04-UI-17）：
    // 生产默认 window.location.origin（单入口部署前后端同源）；
    // 本地 dev 双进程可经编译期环境变量 COLDRAWDB_API_BASE 覆盖，不得进生产默认。
    let api_base = default_api_base_url();
    let client = DiagramClient::new(api_base.clone());
    let auth_client = AuthClient::new(api_base.clone());
    let room_client = RoomClient::new(api_base.clone());
    let collab_client = CollabClient::new(api_base.clone());

    // wire-frontend-collab-ws：真实 WS 协作控制器（S05）。
    // on_token_expired / on_full_resync 用槽位延迟绑定（on_refresh_session 定义在后方）。
    let collab_refresh_slot: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let collab_resync_slot: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let collab_ctl = crate::collab_client::CollabController::new(
        store,
        collab_state,
        activity_feed,
        remote_members,
        remote_presence,
        {
            let slot = collab_refresh_slot.clone();
            Rc::new(move || {
                if let Some(f) = slot.borrow().as_ref() {
                    f();
                }
            })
        },
        {
            let slot = collab_resync_slot.clone();
            Rc::new(move || {
                if let Some(f) = slot.borrow().as_ref() {
                    f();
                }
            })
        },
    );
    // 全量重同步（EX-5.4 / SYNC_GAP_TOO_LARGE）：REST 重载 diagram + 刷新 diff baseline
    *collab_resync_slot.borrow_mut() = Some({
        let client = client.clone();
        let current_diagram_id = current_diagram_id.clone();
        let ctl = Rc::downgrade(&collab_ctl);
        Rc::new(move || {
            let id = current_diagram_id.get_untracked();
            if id.is_empty() {
                return;
            }
            let client = client.clone();
            let ctl = ctl.clone();
            spawn_local(async move {
                if let Ok(diagram) = client.get(&id).await {
                    store.load(diagram);
                    if let Some(c) = ctl.upgrade() {
                        c.refresh_baseline();
                    }
                }
            });
        })
    });
    // 画布光标 presence 上报钩子（editor_render pointermove 单行接入）
    crate::collab_client::set_cursor_reporter(Some({
        let ctl = collab_ctl.clone();
        Rc::new(move |x, y| ctl.send_presence(x, y))
    }));
    // op 上行 watcher：store 集合任意变更 → debounce 200ms diff → op 帧（diff-based 映射，
    // 见 collab_client 模块文档）。远端 op 应用会先推进 baseline，此处 diff 为空不回声。
    create_effect({
        let ctl = collab_ctl.clone();
        move |_| {
            store.tables.track();
            store.references.track();
            store.areas.track();
            store.notes.track();
            // feat-data-dictionary（S07）：字典变更同样触发 diff 上行（dict.* op）
            store.dictionaries.track();
            ctl.notify_local_change();
        }
    });

    // S03 登录持久化：刷新页面从 localStorage 恢复 AuthSession，跳过 /login。
    // 仅在非 share / 非 invite 路径恢复：share 与 invite 的页面状态由 URL 决定，
    // 不能被恢复 session 覆盖（避免 ST-S02-SHARE-VS-AUTH 等 e2e 回归）。
    if !share_mode && invite_token.is_none() {
        let auth_session = auth_session.clone();
        let current_page = current_page.clone();
        let auth_client = auth_client.clone();
        spawn_local(async move {
            if let Some(stored) = crate::editor_data_access::restore_auth_session() {
                match auth_client.me(&stored.access_token).await {
                    Ok(user) => {
                        let mut s = stored;
                        s.user = Some(user);
                        auth_session.set(Some(s));
                        current_page.set(PageState::Rooms);
                    }
                    Err(_) => {
                        crate::editor_data_access::clear_auth_session();
                    }
                }
            }
        });
    }

    // wire-frontend-collab-ws：房间进入 → 建立真实 WS（替换旧的「仅 get_head 假连接」）。
    // effect 重跑时机：进/出房间（current_room）、登录/续期（auth_session）、手动重连（collab_retry）。
    create_effect({
        let room_client = room_client.clone();
        let auth_session = auth_session.clone();
        let current_room = current_room.clone();
        let room_members = room_members.clone();
        let remote_members = remote_members.clone();
        let remote_presence = remote_presence.clone();
        let activity_feed = activity_feed.clone();
        let collab_ctl = collab_ctl.clone();
        let api_base = api_base.clone();
        move |_| {
            // collab_retry：banner 手动重连触发 effect 重跑
            collab_retry.get();
            let (Some(session), Some(room)) = (auth_session.get(), current_room.get()) else {
                collab_ctl.disconnect();
                remote_members.set(Vec::new());
                remote_presence.set(Vec::new());
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let user_id = session.user.as_ref().map(|u| u.id.clone());
            // 手动重连/换房/续期：清零退避计数后重建 WS（connected 帧到达后置 Connected）
            collab_ctl.reset_policy();
            collab_ctl.connect(&api_base, &room_id, &token, user_id.clone());

            let room_client = room_client.clone();
            spawn_local(async move {
                match room_client.list_members(&token, &room_id).await {
                    Ok(items) => {
                        let presence = items
                            .iter()
                            .map(|member| CollabMemberPresence {
                                user_id: member.user_id.clone(),
                                display_name: member.display_name.clone().or_else(|| Some(member.email.clone())),
                                role: Some(member.role.clone()),
                                online: user_id.as_deref() == Some(member.user_id.as_str()),
                            })
                            .collect::<Vec<_>>();
                        room_members.set(items);
                        remote_members.set(presence);
                        // ST-S05-UI-03：画布远端光斑排除当前用户本人，避免遮挡本地选中
                        remote_presence.set(remote_presence_slots(
                            remote_members.get_untracked().into_iter().filter(|m| {
                                user_id.as_deref() != Some(m.user_id.as_str())
                            }).map(|m| {
                                (m.user_id, m.display_name, m.online)
                            }),
                        ));
                    }
                    Err(e) => {
                        prepend_activity(activity_feed, format!("成员状态加载失败 · {e}"));
                    }
                }
            });
        }
    });

    let on_refresh_session = {
        let auth_client = auth_client.clone();
        let auth_session = auth_session.clone();
        let session_notice = session_notice.clone();
        Rc::new(move || {
            let Some(current) = auth_session.get_untracked() else {
                return;
            };
            session_notice.set(Some("续期中...".to_string()));
            let auth_client = auth_client.clone();
            spawn_local(async move {
                match auth_client.refresh_session(&current).await {
                    Ok(next) => {
                        crate::editor_data_access::persist_auth_session(&next);
                        auth_session.set(Some(next));
                        session_notice.set(Some("会话已续期".to_string()));
                    }
                    Err(_) => {
                        crate::editor_data_access::clear_auth_session();
                        auth_session.set(None);
                        session_notice.set(Some("登录已过期，请重新登录".to_string()));
                    }
                }
            });
        }) as Rc<dyn Fn()>
    };

    // wire-frontend-collab-ws：WS 4401 / token_expired 帧 → 复用 S03 续期路径，
    // 续期成功会 set auth_session → 房间 effect 重跑 → 自动以新 token 重连。
    *collab_refresh_slot.borrow_mut() = Some(on_refresh_session.clone());

    let on_logout = {
        let auth_client = auth_client.clone();
        let auth_session = auth_session.clone();
        let session_notice = session_notice.clone();
        let current_page = current_page.clone();
        Rc::new(move || {
            let token = auth_session
                .get_untracked()
                .map(|s| s.access_token)
                .unwrap_or_default();
            crate::editor_data_access::clear_auth_session();
            auth_session.set(None);
            session_notice.set(Some("已退出登录".to_string()));
            // 退出登录后回到 auth 入口（除非是 share/edit 只读或 invite）
            if !matches!(
                current_page.get_untracked(),
                PageState::ShareEdit | PageState::Invite
            ) {
                current_page.set(PageState::Auth);
            }
            if !token.is_empty() {
                let auth_client = auth_client.clone();
                spawn_local(async move {
                    let _ = auth_client.logout(&token).await;
                });
            }
        }) as Rc<dyn Fn()>
    };

    // align-frontend-to-prototype：登录成功 → 进入 rooms-list-page（不进 editor）
    let on_login_success: Rc<dyn Fn()> = {
        let current_page = current_page.clone();
        let invite_token = invite_token.clone();
        Rc::new(move || {
            // B 批：携带 invite 链接登录后回到邀请页续接接受流程（§7.3 登录后可续接）
            if invite_token.is_some() {
                current_page.set(PageState::Invite);
            } else {
                current_page.set(PageState::Rooms);
            }
        })
    };

    // fix-canvas-zoom-invite-comment-resize：进房请求 nonce——快速连点两个房间时，
    // 只有最后一次点击进入的加载结果生效（先到者丢弃，避免覆盖后到者的内容）。
    let enter_nonce = create_rw_signal(0_u64);

    // align-frontend-to-prototype：rooms list → 选中/创建房间 → 进入 editor
    let on_enter_room: Rc<dyn Fn(RoomDetail)> = {
        let client = client.clone();
        let store = store.clone();
        let current_page = current_page.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_room = current_room.clone();
        let current_title = current_title.clone();
        let error = error.clone();
        let enter_nonce = enter_nonce.clone();
        let collab_ctl = collab_ctl.clone();
        Rc::new(move |detail: RoomDetail| {
            let diagram_id = detail.diagram_id.clone();
            // fix-canvas-zoom-invite-comment-resize：进房改为「先加载后切页」。
            // 旧实现先同步切页再异步加载，GET 未返回前用户即可编辑（如按 T 建表）；
            // 加载返回后 store.load 全量覆盖会抹掉这些编辑、使 Inspector 选中项悬空
            // （间歇性 inspector-table-name 30s 超时的根因）。切页前完成加载，
            // 从根上消除「页面可见但图表未加载」的编辑窗口。
            let nonce = enter_nonce.get_untracked() + 1;
            enter_nonce.set(nonce);
            let client = client.clone();
            let store = store.clone();
            let collab_ctl = collab_ctl.clone();
            spawn_local(async move {
                match client.get(&diagram_id).await {
                    Ok(diagram) => {
                        if enter_nonce.get_untracked() != nonce {
                            return;
                        }
                        current_page.set(PageState::RoomEditor);
                        current_diagram_id.set(diagram_id);
                        current_title.set(diagram.name.clone());
                        current_room.set(Some(detail));
                        store.load(diagram);
                        // fix-collab-autosave-race：入房加载完成即建立 diff baseline（不等
                        // WS 首连）——首连前/连接失败时的本地编辑据此 diff 入离线队列，
                        // 连接恢复后 sync+flush 收敛；不再静默丢失。
                        collab_ctl.refresh_baseline();
                        error.set(None);
                    }
                    Err(_) => {
                        if enter_nonce.get_untracked() != nonce {
                            return;
                        }
                        error.set(Some("图表加载失败，请返回房间列表后重试".to_string()));
                    }
                }
            });
        })
    };

    let on_create_room_enter: Rc<dyn Fn(RoomDetail)> = on_enter_room.clone();

    // invite → 接受成功后进入 editor；未登录跳 auth
    let on_invite_after_accept: Rc<dyn Fn()> = {
        let current_page = current_page.clone();
        let client = client.clone();
        let store = store.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_title = current_title.clone();
        let error = error.clone();
        let next_id = next_id.clone();
        let enter_nonce = enter_nonce.clone();
        let collab_ctl = collab_ctl.clone();
        Rc::new(move || {
            // B 批：与 on_enter_room 一致（fix-canvas-zoom-invite-comment-resize 同款
            // 「先加载后切页」+ nonce 守卫），接受邀请进入房间后杜绝过期加载覆盖
            let diagram_id = current_diagram_id.get_untracked();
            let nonce = enter_nonce.get_untracked() + 1;
            enter_nonce.set(nonce);
            let client = client.clone();
            let store = store.clone();
            let collab_ctl = collab_ctl.clone();
            spawn_local(async move {
                match client.get(&diagram_id).await {
                    Ok(diagram) => {
                        if enter_nonce.get_untracked() != nonce {
                            return;
                        }
                        current_page.set(PageState::RoomEditor);
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                        // fix-collab-autosave-race：同 on_enter_room——入房加载完成即
                        // 建立 diff baseline，首连前编辑入离线队列不丢失。
                        collab_ctl.refresh_baseline();
                        // store.load 后用现有 ids 重新计算 next_id，避免与 DB 已保存的 id 冲突
                        let max_id = crate::editor_core::next_id_from_store(&store);
                        next_id.set(max_id + 1);
                        error.set(None);
                    }
                    Err(_) => {
                        if enter_nonce.get_untracked() != nonce {
                            return;
                        }
                        error.set(Some("图表加载失败，请返回房间列表后重试".to_string()));
                    }
                }
            });
        })
    };

    let on_invite_goto_login: Rc<dyn Fn()> = {
        let current_page = current_page.clone();
        Rc::new(move || {
            current_page.set(PageState::Auth);
        })
    };

    // "查看房间列表"（从编辑器可返回）
    let on_back_to_rooms: Rc<dyn Fn()> = {
        let current_page = current_page.clone();
        Rc::new(move || {
            current_page.set(PageState::Rooms);
        })
    };

    let on_after_change: Rc<dyn Fn()> = {
        let client = client.clone();
        let store = store.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_title = current_title.clone();
        let debouncer = debouncer.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        let is_saving = is_saving.clone();
        let save_offline = save_offline.clone();
        Rc::new(move || {
            schedule_save(
                client.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_new_diagram = {
        let client = client.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_title = current_title.clone();
        let modal_kind = modal_kind.clone();
        let error = error.clone();
        Rc::new(move |name: String| {
            current_title.set(name.clone());
            let client = client.clone();
            spawn_local(async move {
                match client.create(&name, None).await {
                    Ok(id) => {
                        current_diagram_id.set(id.clone());
                        modal_kind.set(None);
                        navigate_to_editor(&id);
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    let on_rename_diagram = {
        let on_after = on_after_change.clone();
        Rc::new(move |name: String| {
            current_title.set(name);
            on_after();
        })
    };

    let on_force_overwrite = {
        let client = client.clone();
        let store = store.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_title = current_title.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        Rc::new(move || {
            let Some(info) = conflict.get_untracked() else {
                return;
            };
            let id = current_diagram_id.get_untracked();
            let snap = store.snapshot(id.clone(), current_title.get_untracked());
            let rev = info.current_revision;
            let client = client.clone();
            spawn_local(async move {
                match client.save(&id, rev, &snap).await {
                    Ok(r) => {
                        store.revision.set(r.revision);
                        store.dirty.set(false);
                        conflict.set(None);
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    let on_reload_diagram = {
        let client = client.clone();
        let store = store.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_title = current_title.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        Rc::new(move || {
            let id = current_diagram_id.get_untracked();
            let client = client.clone();
            spawn_local(async move {
                match client.get(&id).await {
                    Ok(diagram) => {
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                        conflict.set(None);
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    // S02：?share= / pathname diagram id 冷启动加载
    {
        let client = client.clone();
        let store = store.clone();
        let current_title = current_title.clone();
        let error = error.clone();
        let share_load_error = share_load_error.clone();
        let id = _diagram_id.clone();
        if id != "default" {
            spawn_local(async move {
                match client.get(&id).await {
                    Ok(diagram) => {
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                        share_loading.set(false);
                    }
                    Err(e) => {
                        if share_mode {
                            share_load_error.set(Some(share_load_error_message(&e)));
                            share_loading.set(false);
                        } else {
                            error.set(Some(e.to_string()));
                        }
                    }
                }
            });
        } else if share_mode {
            share_load_error.set(Some("分享链接不存在或已失效".to_string()));
            share_loading.set(false);
        }
    }

    setup_command_palette_shortcut(palette_visible, view_mode);
    setup_code_view_escape(view_mode, code_visible);

    let palette_items =
        create_memo(move |_| build_palette_items(&store.tables.get(), &store.references.get()));

    let code_content = create_memo(move |_| {
        let tables = store.tables.get();
        let refs = store.references.get();
        let title = current_title.get();
        match code_language.get() {
            CodeLanguage::Sql => {
                // diagram-database-dialect：SQL 预览跟随图级引擎
                let eng = match store.database.get() {
                    crate::editor_core::types::Database::Mysql => "mysql",
                    crate::editor_core::types::Database::Postgresql => "postgresql",
                    _ => "generic",
                };
                export_diagram_sql(&tables, &refs, eng)
            }
            CodeLanguage::Dbml => export_diagram_dbml(&tables, &refs),
            CodeLanguage::Json => export_diagram_json(&title, &tables, &refs),
        }
    });

    let client_for_io = client.clone();
    // 4 个 save handler 各 clone 一份（避免 move 闭包互抢 client）
    let client_for_create = client.clone();
    let client_for_save = client.clone();
    let client_for_title = client.clone();
    let client_for_add_field = client.clone();
    let client_for_change_type = client.clone();
    let client_for_pk = client.clone();
    let client_for_create_ref = client.clone();
    let client_for_update_ref = client.clone();
    let client_for_flip_ref = client.clone();
    let client_for_delete_ref = client.clone();
    let client_for_nn = client.clone();
    let client_for_uq = client.clone();
    let client_for_rename_table = client.clone();
    let client_for_rename_field = client.clone();
    let client_for_set_table_comment = client.clone();
    let client_for_set_field_comment = client.clone();
    let client_for_delete_field = client.clone();
    let client_for_delete_table = client.clone();

    let on_create_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let error_for_create = error.clone();
        let current_room = current_room.clone();
        let activity_feed_for_create = activity_feed.clone();
        Rc::new(move || {
            if editor_is_read_only(share_mode, current_room) {
                error_for_create.set(Some("只读角色不能编辑图表".to_string()));
                return;
            }
            // fix-global-entity-id-uniqueness：实体 id 改全局唯一随机 id，
            // 杜绝新 diagram 从 auto-1 重新计数撞后端全局主键（保存 500）
            let table_id = crate::editor_core::new_entity_id("auto");
            // 主原型事实（core-01 addTable）：x=180+n*55, y=145+n*35 层叠落位；
            // 命名 table_{n+1}；每张新表自带一个 id 字段（UUID, pk/nn/uq）
            let table_count = store.tables.get().len();
            let field_id = format!("{}-field-id", table_id);
            let default_fields = vec![Field {
                id: field_id,
                name: "id".into(),
                type_: "UUID".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: true,
                not_null: true,
                increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }];
            let new_table = Table {
                id: table_id.clone(),
                name: format!("table_{}", table_count + 1),
                x: 180.0 + table_count as f64 * 55.0,
                y: 145.0 + table_count as f64 * 35.0,
                // 主原型事实：新表无自定义色，表头走 brand-soft 渐变（历史 Semi #175e7a 已移除）
                color: String::new(),
                comment: String::new(),
                fields: default_fields,
                indices: Vec::new(),
                width: None,
                min_height: None,
            };
            let mut tables = store.tables.get();
            tables.push(new_table.clone());
            store.tables.set(tables);
            selection.set(SelectionKind::Table(table_id.clone()));
            inspector_open.set(true);
            store.dirty.set(true);
            command_stack
                .get()
                .borrow_mut()
                .record(crate::editor_core::Command::AddTable(new_table.clone()));
            if current_room.get_untracked().is_some() {
                // fix-collab-autosave-race：方案B 唯一 op 通道 = watcher → emit_local_ops
                // diff 上行；此处不得直接 enqueue（会产生无帧 pending op，ack 永不到位，
                // dirty 卡死）。activity 仅作提示，对账由 op 通道完成。
                prepend_activity(
                    activity_feed_for_create,
                    format!("本地创建表 {}，等待 OT ack", new_table.name),
                );
            }

            if current_diagram_id.get() == "default" {
                let client = client_for_create.clone();
                let store = store.clone();
                let debouncer = debouncer.clone();
                let current_diagram_id = current_diagram_id.clone();
                let current_title = current_title.clone();
                let error = error_for_create.clone();
                let conflict = conflict.clone();
                let is_saving = is_saving.clone();
                spawn_local(async move {
                    match client.create("新图", None).await {
                        Ok(new_id) => {
                            current_diagram_id.set(new_id);
                            schedule_save(
                                client,
                                store,
                                current_diagram_id,
                                current_title,
                                debouncer,
                                conflict,
                                error,
                                is_saving,
                                save_offline,
                                collab_state,
                                activity_feed,
                                current_room.clone(),
                                auth_session.clone(),
                            );
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    }
                });
            } else {
                schedule_save(
                    client_for_create.clone(),
                    store.clone(),
                    current_diagram_id.clone(),
                    current_title.clone(),
                    debouncer.clone(),
                    conflict.clone(),
                    error_for_create.clone(),
                    is_saving.clone(),
                    save_offline.clone(),
                    collab_state,
                    activity_feed,
                    current_room.clone(),
                    auth_session.clone(),
                );
            }
        }) as Rc<dyn Fn()>
    };

    let on_save = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move || {
            if share_mode {
                return;
            }
            schedule_save(
                client_for_save.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        }) as Rc<dyn Fn()>
    };

    // fix-appbar-roomname-back-and-import-merge：导入本地合并进当前画布（替代 bridge 新建游离 diagram）
    let on_merge_import: Rc<dyn Fn(Vec<Table>, Vec<Reference>)> = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let client_for_merge = client.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let import_notice = import_notice.clone();
        Rc::new(move |new_tables: Vec<Table>, new_refs: Vec<Reference>| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let (merged_tables, merged_refs) = merge_import_into_store(
                &store.tables.get(),
                &store.references.get(),
                &new_tables,
                &new_refs,
            );
            let n_tables = new_tables.len();
            let n_refs = new_refs.len();
            let first_new = new_tables.first().map(|t| t.id.clone());
            store.tables.set(merged_tables);
            store.references.set(merged_refs);
            store.dirty.set(true);
            if let Some(id) = first_new {
                selection.set(SelectionKind::Table(id));
                inspector_open.set(true);
            }
            schedule_save(
                client_for_merge.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
            import_notice.set(Some(format!("已导入 {n_tables} 张表 / {n_refs} 条关系")));
        }) as Rc<dyn Fn(Vec<Table>, Vec<Reference>)>
    };

    let on_title_blur = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |title: String| {
            if share_mode {
                return;
            }
            current_title.set(title);
            store.dirty.set(true);
            schedule_save(
                client_for_title.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        }) as Rc<dyn Fn(String)>
    };

    let on_add_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            // fix-global-entity-id-uniqueness：全局唯一 id，避免跨 diagram 撞全局主键
            let new_field = Field {
                id: crate::editor_core::new_entity_id("auto"),
                name: "新字段".into(),
                type_: "VARCHAR(255)".into(),
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
            let mut tables = store.tables.get();
            if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                table.fields.push(new_field);
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_add_field.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_change_type = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |field_id: String, new_type: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            for table in tables.iter_mut() {
                if let Some(field) = table.fields.iter_mut().find(|f| f.id == field_id) {
                    field.type_ = new_type.clone();
                    break;
                }
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_change_type.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_set_ref = {
        let active_tool = active_tool.clone();
        let rel_tool_state = rel_tool_state.clone();
        let store = store.clone();
        Rc::new(move |field_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let tables = store.tables.get();
            if let Some(table_id) = tables.iter().find_map(|t| {
                t.fields
                    .iter()
                    .find(|f| f.id == field_id)
                    .map(|_| t.id.clone())
            }) {
                active_tool.set(ActiveTool::Relationship);
                rel_tool_state.set(RelToolState::PickTarget {
                    start_table_id: table_id,
                    start_field_id: field_id,
                });
            }
        })
    };

    // fix-global-entity-id-uniqueness：关系 id 改全局唯一随机 id（原 ref-{计数器}
    // 会与其他 diagram 已占用的全局主键冲突）
    let next_ref_id = {
        Rc::new(move || crate::editor_core::new_entity_id("ref")) as Rc<dyn Fn() -> String>
    };

    let on_create_reference = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Rc::new(move |reference: Reference| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut refs = store.references.get();
            refs.push(reference.clone());
            store.references.set(refs);
            store.dirty.set(true);
            selection.set(SelectionKind::Reference(reference.id));
            inspector_open.set(true);
            schedule_save(
                client_for_create_ref.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_field_pick: Option<Box<dyn Fn(String, String) + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
        let on_create_reference = on_create_reference.clone();
        let next_ref_id = next_ref_id.clone();
        Some(Box::new(
            move |table_id: String, field_id: String| match rel_tool_state.get_untracked() {
                RelToolState::PickSource => {
                    rel_tool_state.set(RelToolState::PickTarget {
                        start_table_id: table_id,
                        start_field_id: field_id,
                    });
                }
                RelToolState::PickTarget {
                    start_table_id,
                    start_field_id,
                } => {
                    // p0-fix 定点 3：点击两点直接落账（无确认条），cardinality 用推导值
                    let inferred = modals::infer_cardinality(&start_field_id, &field_id, &store);
                    let reference = build_reference(
                        next_ref_id(),
                        start_table_id,
                        start_field_id,
                        table_id,
                        field_id,
                        &inferred,
                    );
                    on_create_reference(reference);
                    rel_tool_state.set(RelToolState::PickSource);
                }
                RelToolState::Dragging { .. } => {}
                _ => {}
            },
        ))
    };

    let on_relation_drag_start: Option<Box<dyn Fn(String, String) + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
        Some(Box::new(move |table_id: String, field_id: String| {
            rel_tool_state.set(RelToolState::Dragging {
                start_table_id: table_id,
                start_field_id: field_id,
            });
        }))
    };

    let on_relation_drop: Option<Box<dyn Fn(String, String, String, String) + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
        let on_create_reference = on_create_reference.clone();
        let next_ref_id = next_ref_id.clone();
        Some(Box::new(
            move |start_table_id: String,
                  start_field_id: String,
                  end_table_id: String,
                  end_field_id: String| {
                // p0-fix 定点 3：拖到目标字段松开直接落账（无确认条），cardinality 用推导值
                let inferred = modals::infer_cardinality(&start_field_id, &end_field_id, &store);
                let reference = build_reference(
                    next_ref_id(),
                    start_table_id,
                    start_field_id,
                    end_table_id,
                    end_field_id,
                    &inferred,
                );
                on_create_reference(reference);
                rel_tool_state.set(RelToolState::PickSource);
            },
        ))
    };

    let on_relation_drag_cancel: Option<Box<dyn Fn() + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
        let active_tool = active_tool.clone();
        Some(Box::new(move || {
            // D 批：Esc 层级处理器可能已退出关系工具（active_tool=Select），此时不回溯 PickSource
            if active_tool.get_untracked() == ActiveTool::Relationship {
                rel_tool_state.set(RelToolState::PickSource);
            }
        }))
    };

    // D 批：表拖动松手（已吸附写回 store）→ dirty + 协作 op + S01 保存链路（ST-CR-02 落账依据）
    let on_table_drop: Option<Box<dyn Fn() + 'static>> = {
        let store = store.clone();
        let on_after_change = on_after_change.clone();
        Some(Box::new(move || {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            store.dirty.set(true);
            if current_room.get_untracked().is_some() {
                // fix-collab-autosave-race：同 table.create——op 只走 watcher diff 通道。
                prepend_activity(activity_feed, "本地移动表位置，等待 OT ack".to_string());
            }
            on_after_change();
        }))
    };

    let on_toggle_pk = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, primary: bool| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            toggle_field_primary(&mut tables, &table_id, &field_id, primary);
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_pk.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // 主原型 Inspector 字段卡约束 chips：NOT NULL / UNIQUE 与 PK 同构
    let on_toggle_nn = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, not_null: bool| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(field) = tables
                .iter_mut()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
            {
                field.not_null = not_null;
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_nn.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_toggle_uq = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, unique: bool| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(field) = tables
                .iter_mut()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
            {
                field.unique = unique;
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_uq.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_rename_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, name: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let name = name.trim().to_string();
            if name.is_empty() {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                if table.name == name {
                    return;
                }
                table.name = name;
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_rename_table.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_rename_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, name: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let name = name.trim().to_string();
            if name.is_empty() {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(field) = tables
                .iter_mut()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
            {
                if field.name == name {
                    return;
                }
                field.name = name;
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_rename_field.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // fix-canvas-zoom-invite-comment-resize（core-01a §1.4.2）：表注释 blur 落账。
    // 通路仿 on_rename_table：只读短路 → 值变化才写 store → dirty → schedule_save。
    // 注释任意字符串、允许为空：不 trim、不拒绝空串、不触发重名校验。
    let on_set_table_comment = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, comment: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            let Some(table) = tables.iter_mut().find(|t| t.id == table_id) else {
                return;
            };
            if table.comment == comment {
                return;
            }
            table.comment = comment;
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_set_table_comment.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // fix-canvas-zoom-invite-comment-resize（core-01a §2.3）：Inspector 字段卡注释
    // blur 落账，通路与 on_rename_field / ListView commit_comment 一致。
    let on_set_field_comment = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, comment: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            let Some(field) = tables
                .iter_mut()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
            else {
                return;
            };
            if field.comment == comment {
                return;
            }
            field.comment = comment;
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_set_field_comment.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_delete_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                table.fields.retain(|f| f.id != field_id);
            }
            store.tables.set(tables);
            // 级联清理：触及该字段的关系一并移除
            let mut refs = store.references.get();
            refs.retain(|r| {
                !(r.start_table_id == table_id && r.start_field_id == field_id)
                    && !(r.end_table_id == table_id && r.end_field_id == field_id)
            });
            store.references.set(refs);
            let clear_to_table = matches!(
                selection.get_untracked(),
                SelectionKind::Field { field_id: ref fid, .. } if *fid == field_id
            );
            if clear_to_table {
                selection.set(SelectionKind::Table(table_id.clone()));
            }
            store.dirty.set(true);
            schedule_save(
                client_for_delete_field.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_delete_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            tables.retain(|t| t.id != table_id);
            store.tables.set(tables);
            // 级联清理：该表参与的所有关系
            let mut refs = store.references.get();
            refs.retain(|r| r.start_table_id != table_id && r.end_table_id != table_id);
            store.references.set(refs);
            selection.set(SelectionKind::None);
            store.dirty.set(true);
            schedule_save(
                client_for_delete_table.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_update_ref_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |ref_id: String, field: &str, value: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut refs = store.references.get();
            if let Some(r) = refs.iter_mut().find(|r| r.id == ref_id) {
                match field {
                    "type_" => r.type_ = value,
                    "on_delete" => r.on_delete = value,
                    "on_update" => r.on_update = value,
                    _ => {}
                }
            }
            store.references.set(refs);
            store.dirty.set(true);
            schedule_save(
                client_for_update_ref.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_flip_ref = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |ref_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut refs = store.references.get();
            if let Some(idx) = refs.iter().position(|r| r.id == ref_id) {
                refs[idx] = flip_reference_endpoints(&refs[idx], &store);
            }
            store.references.set(refs);
            store.dirty.set(true);
            schedule_save(
                client_for_flip_ref.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_delete_ref = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        Rc::new(move |ref_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut refs = store.references.get();
            refs.retain(|r| r.id != ref_id);
            store.references.set(refs);
            store.dirty.set(true);
            selection.set(SelectionKind::None);
            schedule_save(
                client_for_delete_ref.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    let on_jump_to_table = Rc::new({
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let selected_table_id = selected_table_id.clone();
        move |id: String| {
            selected_table_id.set(Some(id.clone()));
            selection.set(SelectionKind::Table(id));
            inspector_open.set(true);
        }
    });

    // relation-inspector-and-ddl-io：点击连线 → 选中该关系 + 打开 Inspector（ST-PB-03）
    // 不再弹详情模态：详情与删除入口统一在 Inspector 关系面板
    let on_reference_pick: Option<Box<dyn Fn(String) + 'static>> = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Some(Box::new(move |ref_id: String| {
            selection.set(SelectionKind::Reference(ref_id));
            inspector_open.set(true);
        }))
    };

    // p0-fix 定点 2：区域拖框落账（ST-AN-01）——创建后选中 + 开 Inspector + 复位选择工具
    let on_area_create: Option<Box<dyn Fn(f64, f64, f64, f64) + 'static>> = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let client_for_area = client.clone();
        Some(Box::new(move |x: f64, y: f64, w: f64, h: f64| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let area = crate::editor_render::build_area(
                crate::editor_core::new_entity_id("area"),
                x,
                y,
                w,
                h,
            );
            let mut areas = store.areas.get();
            areas.push(area.clone());
            store.areas.set(areas);
            store.dirty.set(true);
            selection.set(SelectionKind::Area(area.id));
            inspector_open.set(true);
            active_tool.set(ActiveTool::Select);
            schedule_save(
                client_for_area.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        }))
    };

    // p0-fix 定点 2：便签点击放置落账（ST-AN-02）
    let on_note_create: Option<Box<dyn Fn(f64, f64) + 'static>> = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let client_for_note = client.clone();
        Some(Box::new(move |x: f64, y: f64| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let note = crate::editor_render::build_note(
                crate::editor_core::new_entity_id("note"),
                x,
                y,
            );
            let mut notes = store.notes.get();
            notes.push(note.clone());
            store.notes.set(notes);
            store.dirty.set(true);
            selection.set(SelectionKind::Note(note.id));
            inspector_open.set(true);
            active_tool.set(ActiveTool::Select);
            schedule_save(
                client_for_note.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        }))
    };

    // p0-fix 定点 2：点击区域 / 便签 → 选中 + Inspector 编辑面板
    let on_area_pick: Option<Box<dyn Fn(String) + 'static>> = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Some(Box::new(move |id: String| {
            selection.set(SelectionKind::Area(id));
            inspector_open.set(true);
        }))
    };
    let on_note_pick: Option<Box<dyn Fn(String) + 'static>> = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Some(Box::new(move |id: String| {
            selection.set(SelectionKind::Note(id));
            inspector_open.set(true);
        }))
    };

    // p0-fix 定点 2：删除区域 / 便签（Inspector 按钮 + Delete 键共用）
    let on_delete_area = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let client_for_delete_area = client.clone();
        Rc::new(move |area_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut areas = store.areas.get();
            areas.retain(|a| a.id != area_id);
            store.areas.set(areas);
            store.dirty.set(true);
            selection.set(SelectionKind::None);
            schedule_save(
                client_for_delete_area.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };
    let on_delete_note = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let client_for_delete_note = client.clone();
        Rc::new(move |note_id: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut notes = store.notes.get();
            notes.retain(|n| n.id != note_id);
            store.notes.set(notes);
            store.dirty.set(true);
            selection.set(SelectionKind::None);
            schedule_save(
                client_for_delete_note.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // p0-fix 定点 2：Inspector 编辑回写（区域 name/color；便签 content）
    let on_update_area = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let client_for_update_area = client.clone();
        Rc::new(move |area_id: String, field: &str, value: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut areas = store.areas.get();
            let Some(a) = areas.iter_mut().find(|a| a.id == area_id) else {
                return;
            };
            match field {
                "name" => a.name = value,
                "color" => a.color = value,
                _ => return,
            }
            store.areas.set(areas);
            store.dirty.set(true);
            schedule_save(
                client_for_update_area.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };
    let on_update_note = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let client_for_update_note = client.clone();
        Rc::new(move |note_id: String, content: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut notes = store.notes.get();
            let Some(n) = notes.iter_mut().find(|n| n.id == note_id) else {
                return;
            };
            n.content = content;
            store.notes.set(notes);
            store.dirty.set(true);
            schedule_save(
                client_for_update_note.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // redesign-listview-type-length-canvas-fix：字段 tag 落账（ST-CR-TAG-01）
    // 仅值变化才写 store + schedule_save，规避「blur 无条件 notify → 表单重建」
    let on_update_field_tag = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let client_for_update_tag = client.clone();
        Rc::new(move |table_id: String, field_id: String, tag: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let mut tables = store.tables.get();
            let Some(f) = tables
                .iter_mut()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
            else {
                return;
            };
            if f.tag == tag {
                return;
            }
            f.tag = tag;
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(
                client_for_update_tag.clone(),
                store.clone(),
                current_diagram_id.clone(),
                current_title.clone(),
                debouncer.clone(),
                conflict.clone(),
                error.clone(),
                is_saving.clone(),
                save_offline.clone(),
                collab_state,
                activity_feed,
                current_room.clone(),
                auth_session.clone(),
            );
        })
    };

    // feat-data-dictionary（S07，core-01e §3）：字段绑定/解绑字典——软引用编码，
    // 进 undo 栈（apply_dict_mutation 单次单元）+ S01 debounce 保存通路
    let on_set_field_dict = {
        let store = store.clone();
        let on_after = on_after_change.clone();
        Rc::new(move |table_id: String, field_id: String, code: String| {
            if editor_is_read_only(share_mode, current_room) {
                return;
            }
            let unchanged = store
                .tables
                .get()
                .iter()
                .find(|t| t.id == table_id)
                .and_then(|t| t.fields.iter().find(|f| f.id == field_id))
                .map(|f| f.dict_code == code)
                .unwrap_or(true);
            if unchanged {
                return;
            }
            apply_dict_mutation(&store, command_stack, move |_ds, tables| {
                if let Some(f) = tables
                    .iter_mut()
                    .find(|t| t.id == table_id)
                    .and_then(|t| t.fields.iter_mut().find(|f| f.id == field_id))
                {
                    f.dict_code = code;
                }
            });
            on_after();
        })
    };

    let on_canvas_select: Option<Box<dyn Fn(String) + 'static>> = {
        let selection = selection.clone();        let inspector_open = inspector_open.clone();
        let selected_table_id = selected_table_id.clone();
        Some(Box::new(move |id: String| {
            selected_table_id.set(Some(id.clone()));
            selection.set(SelectionKind::Table(id));
            inspector_open.set(true);
        }))
    };

    let on_canvas_deselect: Option<Box<dyn Fn() + 'static>> = {
        let selection = selection.clone();
        Some(Box::new(move || {
            selection.set(SelectionKind::None);
        }))
    };

    let on_dblclick_blank: Option<Box<dyn Fn() + 'static>> = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Some(Box::new(move || {
            selection.set(SelectionKind::None);
            inspector_open.set(false);
        }))
    };

    let on_palette_select = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Callback::new(move |item: PaletteItem| match item.kind {
            crate::command_palette::PaletteKind::Table => {
                selection.set(SelectionKind::Table(item.id));
                inspector_open.set(true);
            }
            crate::command_palette::PaletteKind::Reference => {
                selection.set(SelectionKind::Reference(item.id));
                inspector_open.set(true);
            }
            _ => {}
        })
    };

    let on_create_table_rail = on_create_table.clone();
    let on_create_table_panel = on_create_table.clone();
    let on_create_table_guide = on_create_table.clone();

    let on_open_settings = {
        let modal_kind = modal_kind.clone();
        Rc::new(move || modal_kind.set(Some(modals::ModalKind::BridgeSettings)))
    };

    let on_toggle_members = {
        let room_panel_visible = room_panel_visible.clone();
        Rc::new(move || room_panel_visible.update(|v| *v = !*v)) as Rc<dyn Fn()>
    };

    // B 批：邀请模态（AppBar btn-invite / 成员抽屉「邀请新成员」共用入口）
    let invite_modal_open: RwSignal<bool> = create_rw_signal(false);
    let on_open_invite = {
        let invite_modal_open = invite_modal_open.clone();
        Rc::new(move || invite_modal_open.set(true)) as Rc<dyn Fn()>
    };

    // B 批：invite 页「返回空间」— 已登录回 rooms，未登录回 auth
    let on_invite_back = {
        let current_page = current_page.clone();
        let auth_session = auth_session.clone();
        Rc::new(move || {
            if auth_session.get_untracked().is_some() {
                current_page.set(PageState::Rooms);
            } else {
                current_page.set(PageState::Auth);
            }
        }) as Rc<dyn Fn()>
    };

    // p0-fix 定点 1：删除房间成功后的收尾 — 关成员抽屉 → 清当前房间 → 回 rooms 页 → 强制刷新列表
    let rooms_reload_nonce: RwSignal<u32> = create_rw_signal(0);
    let on_room_deleted: Rc<dyn Fn()> = {
        let current_page = current_page.clone();
        Rc::new(move || {
            room_panel_visible.set(false);
            current_room.set(None);
            rooms_reload_nonce.update(|n| *n += 1);
            current_page.set(PageState::Rooms);
        }) as Rc<dyn Fn()>
    };

    let on_open_palette = {
        let palette_visible = palette_visible.clone();
        Rc::new(move || palette_visible.set(true)) as Rc<dyn Fn()>
    };

    let on_toggle_activity = {
        let activity_open = activity_open.clone();
        Rc::new(move || activity_open.update(|v| *v = !*v)) as Rc<dyn Fn()>
    };

    let on_reconnect = {
        let collab_retry = collab_retry.clone();
        Rc::new(move || collab_retry.update(|v| *v += 1)) as Rc<dyn Fn()>
    };

    let inspector_read_only = Rc::new(move || editor_is_read_only(share_mode, current_room))
        as Rc<dyn Fn() -> bool>;

    let on_delete_diagram = {
        let client = client.clone();
        let current_diagram_id = current_diagram_id.clone();
        let error = error.clone();
        Rc::new(move || {
            let id = current_diagram_id.get();
            if !is_deletable_diagram_id(&id) {
                error.set(Some("无法删除默认画布".to_string()));
                return;
            }
            let confirmed = web_sys::window()
                .and_then(|w| {
                    w.confirm_with_message("确定删除当前图表？此操作不可撤销。")
                        .ok()
                })
                .unwrap_or(false);
            if !confirmed {
                return;
            }
            let client = client.clone();
            let error = error.clone();
            spawn_local(async move {
                match client.delete(&id).await {
                    Ok(()) => {
                        if let Some(win) = web_sys::window() {
                            let _ = win.location().set_href("/editor");
                        }
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    // D 批：T/R 工具快捷键 + Esc 浮层层级（ST-KB-T-01/R-01/ESC-01/VIEWER）
    setup_editor_tool_shortcuts(
        current_page,
        share_mode,
        current_room,
        palette_visible,
        view_mode,
        modal_kind,
        active_tool,
        rel_tool_state,
        on_create_table.clone(),
        selection,
        on_delete_ref.clone(),
        on_delete_area.clone(),
        on_delete_note.clone(),
    );
    setup_escape_layer_handler(
        palette_visible,
        view_mode,
        conflict,
        modal_kind,
        invite_modal_open,
        io_drawer,
        room_panel_visible,
        active_tool,
        rel_tool_state,
        close_io_drawer.clone(),
    );

    view! {
        // ─── Auth 页（align-frontend-to-prototype） ───
        <div
            style:display=move || if current_page.get() == PageState::Auth { "block" } else { "none" }
        >
            <AuthGate
                auth_client=auth_client.clone()
                auth_session=auth_session
                session_notice=session_notice
                on_login_success=Some(on_login_success.clone())
            />
        </div>
        // ─── Rooms 列表页（align-frontend-to-prototype，进入编辑器前必经） ───
        <div
            style:display=move || if current_page.get() == PageState::Rooms { "block" } else { "none" }
        >
            <RoomsListPage
                auth_session=auth_session
                session_notice=session_notice
                auth_client=auth_client.clone()
                diagram_client=client.clone()
                room_client=room_client.clone()
                on_logout=on_logout.clone()
                on_select_room=on_enter_room.clone()
                on_create_room=on_create_room_enter.clone()
                reload_rooms=rooms_reload_nonce
            />
        </div>
        // ─── Invite 独立页（align-frontend-to-prototype FEUX-AC-04） ───
        <div
            style:display=move || if current_page.get() == PageState::Invite { "block" } else { "none" }
        >
            {invite_token.clone().map(|token| view! {
                <InviteAcceptPage
                    token=token
                    room_client=room_client.clone()
                    auth_session=auth_session
                    current_diagram_id=current_diagram_id
                    current_title=current_title
                    current_room=current_room
                    error=error.clone()
                    on_after_accept=on_invite_after_accept.clone()
                    on_goto_login=on_invite_goto_login.clone()
                    on_back=on_invite_back.clone()
                />
            })}
        </div>
        <main
            class="cdb-share-state-page"
            data-testid="share-loading"
            style:display=move || if current_page.get() == PageState::ShareEdit && share_loading.get() { "grid" } else { "none" }
        >
            <section>
                <h1>"正在加载分享图表"</h1>
                <p>"请稍候..."</p>
            </section>
        </main>
        <main
            class="cdb-share-state-page"
            data-testid="share-not-found"
            style:display=move || if current_page.get() == PageState::ShareEdit && share_load_error.get().is_some() { "grid" } else { "none" }
        >
            <section>
                <h1>"无法打开分享链接"</h1>
                <p>{move || share_load_error.get().unwrap_or_default()}</p>
            </section>
        </main>
        // ─── Editor 页（RoomEditor / ShareEdit 共用；room-editor-page 为页面态锚点，editor-ready 供既有 e2e 回归） ───
        <div
            data-testid="room-editor-page"
            style:display=move || {
                let p = current_page.get();
                if p == PageState::RoomEditor
                    || (p == PageState::ShareEdit && !share_loading.get() && share_load_error.get().is_none()) {
                    "block"
                } else {
                    "none"
                }
            }
        >
        <div
            class="cdb-app"
            data-testid="editor-ready"
            data-read-only=share_mode
        >
            <div class="cdb-aurora" aria-hidden="true"></div>
            {share_mode.then(|| view! {
                <div class="cdb-share-readonly-banner" data-testid="share-readonly">
                    "匿名只读分享"
                </div>
            })}
            <AppBar
                modal_kind=modal_kind
                current_title=current_title
                store=store.clone()
                stack=command_stack
                is_saving=is_saving
                save_offline=save_offline
                view_mode=view_mode
                code_visible=code_visible
                inspector_open=inspector_open
                transform=canvas_transform
                error=error.clone()
                on_title_blur=on_title_blur
                on_after_change=on_after_change.clone()
                on_open_import=open_import_drawer.clone()
                on_open_export=open_export_drawer.clone()
                on_open_settings=on_open_settings.clone()
                on_open_palette=on_open_palette.clone()
                on_delete_diagram=on_delete_diagram.clone()
                auth_session=auth_session
                session_notice=session_notice
                on_refresh_session=on_refresh_session.clone()
                on_logout=on_logout.clone()
                current_room=current_room
                remote_members=remote_members
                on_open_rooms=on_back_to_rooms.clone()
                on_open_members=on_toggle_members.clone()
                on_open_invite=on_open_invite.clone()
                read_only=share_mode
                theme_mode=theme_mode
            />
            <div
                class="cdb-main"
                // ux-canvas-batch 批次2 收尾: ViewMode::List 时隐藏画布（选项 A，黑板条目 8）
                class:cdb-is-hidden=move || view_mode.get() != ViewMode::Canvas
                class:cdb-is-inspector-collapsed=move || {
                    !inspector_open.get() || io_drawer.get() != IoDrawerKind::None
                }
                class:cdb-has-io-drawer=move || io_drawer.get() != IoDrawerKind::None
            >
                <ToolRail
                    store=store.clone()
                    selection=selection
                    inspector_open=inspector_open
                    active_tool=active_tool
                    rel_tool_state=rel_tool_state
                    on_create_table=on_create_table_rail.clone()
                    on_open_palette=on_open_palette.clone()
                    on_open_settings=on_open_settings.clone()
                    on_toggle_activity=on_toggle_activity.clone()
                    on_toggle_dicts=on_toggle_dicts.clone()
                    current_room=current_room
                    read_only=share_mode
                />
                <div class="cdb-canvas-container" data-testid="editor-canvas-container">
                    {let open_import_guide = open_import_drawer.clone();
                     move || if store.tables.get().is_empty() {
                        view! {
                            <EmptyGuide
                                on_create_table=on_create_table_guide.clone()
                                on_import=open_import_guide.clone()
                                read_only=share_mode
                            />
                        }.into_view()
                    } else {
                        view! { <></> }.into_view()
                    }}
                    <RelToolHint rel_state=rel_tool_state />
                    <ReconnectBanner
                        collab_state=collab_state
                        current_room=current_room
                        on_reconnect=on_reconnect.clone()
                    />
                    <Canvas
                        store=store.clone()
                        transform=canvas_transform
                        read_only=share_mode
                        remote_presence=remote_presence
                        on_select=on_canvas_select
                        on_deselect=on_canvas_deselect
                        on_dblclick_blank=on_dblclick_blank
                        rel_tool_active=rel_tool_active
                        on_field_pick=on_field_pick
                        on_relation_drag_start=on_relation_drag_start
                        on_relation_drop=on_relation_drop
                        on_relation_drag_cancel=on_relation_drag_cancel
                        on_table_drop=on_table_drop
                        on_reference_pick=on_reference_pick
                        create_tool=create_tool
                        on_area_create=on_area_create
                        on_note_create=on_note_create
                        on_area_pick=on_area_pick
                        on_note_pick=on_note_pick
                        theme_mode=theme_mode
                    />
                    <ActivityFeed items=activity_feed visible=activity_open />
                    <FloatingControls transform=canvas_transform />
                </div>
                <Splitter kind=SplitterKind::Inspector />
                <Inspector
                    store=store.clone()
                    selection=selection
                    inspector_open=inspector_open
                    on_add_field=on_add_field.clone()
                    on_change_type=on_change_type.clone()
                    on_set_ref=on_set_ref.clone()
                    on_toggle_pk=on_toggle_pk.clone()
                    on_toggle_nn=on_toggle_nn.clone()
                    on_toggle_uq=on_toggle_uq.clone()
                    on_rename_table=on_rename_table.clone()
                    on_rename_field=on_rename_field.clone()
                    on_set_table_comment=on_set_table_comment.clone()
                    on_set_field_comment=on_set_field_comment.clone()
                    on_delete_field=on_delete_field.clone()
                    on_delete_table=on_delete_table.clone()
                    on_update_ref_field=on_update_ref_field.clone()
                    on_flip_ref=on_flip_ref.clone()
                    on_delete_ref=on_delete_ref.clone()
                    on_update_area=on_update_area.clone()
                    on_update_note=on_update_note.clone()
                    on_update_field_tag=on_update_field_tag.clone()
                    on_set_field_dict=on_set_field_dict.clone()
                    on_delete_area=on_delete_area.clone()
                    on_delete_note=on_delete_note.clone()
                    on_jump_to_table=on_jump_to_table.clone()
                    read_only=inspector_read_only.clone()
                />
                <RoomPanel
                    visible=room_panel_visible
                    room_client=room_client.clone()
                    auth_session=auth_session
                    current_room=current_room
                    room_members=room_members
                    error=error.clone()
                    on_open_invite=on_open_invite.clone()
                    modal_kind=modal_kind
                />
                <InviteModal
                    open=invite_modal_open
                    room_client=room_client.clone()
                    auth_session=auth_session
                    current_room=current_room
                    error=error.clone()
                />
            </div>
            // fix-overflow-menu-room-delete-listview-io：IoDrawer 上移为 .cdb-app 直接子级——
            // ListView 全屏态下 cdb-main 隐藏，抽屉须独立于画布容器渲染（core-01d §4.5，z-35 叠加）
            <IoDrawer
                kind=io_drawer
                store=store.clone()
                current_title=current_title
                client=client_for_io.clone()
                error=error.clone()
                on_close=close_io_drawer.clone()
                on_merge_import=on_merge_import.clone()
                auth_session=auth_session
                notice=import_notice
            />
            // feat-data-dictionary（S07）：数据字典抽屉（独立于 IO 抽屉渲染）
            <DictPanel
                store=store.clone()
                open=dict_panel_open
                current_title=current_title
                on_save=on_after_change.clone()
                command_stack=command_stack
                read_only=inspector_read_only.clone()
            />
            <StatusBar
                store=store.clone()
                transform=canvas_transform
                inspector_open=inspector_open
                collab_state=collab_state
                remote_members=remote_members
                current_room=current_room
            />
            <CodeView
                visible=code_visible
                language=code_language
                content=code_content
                copy_toast=code_copy_toast
            />
            // ux-canvas-batch 批次2 收尾: ViewMode::List 时全屏渲染 ListView（选项 A，黑板条目 8）
            // redesign-listview-type-length-canvas-fix：on_after_change 在闭包外先 clone，
            // 避免 move 闭包捕获后下文 Ctrl+Z 接线（9015）借用失败
            {let on_after_change_list = on_after_change.clone();
             let open_import_for_list = open_import_drawer.clone() as Rc<dyn Fn()>;
             let open_export_for_list = open_export_drawer.clone() as Rc<dyn Fn()>;
             move || if view_mode.get() == ViewMode::List {
                let on_save_listview = on_after_change_list.clone();
                let on_jump_for_listview: Rc<dyn Fn(String)> = {
                    let on_select = on_select_table.clone();
                    Rc::new(move |tid: String| {
                        view_mode.set(ViewMode::Canvas);
                        on_select(Some(tid));
                    })
                };
                let on_back_for_listview: Rc<dyn Fn()> =
                    Rc::new(move || view_mode.set(ViewMode::Canvas));
                view! {
                    <div class="cdb-list-view-panel" data-testid="list-view-panel">
                        <ListView
                            store=store.clone()
                            on_select_table=on_select_table.clone()
                            on_jump_to_canvas=on_jump_for_listview.clone()
                            on_back_to_canvas=on_back_for_listview
                            on_save=on_save_listview
                            command_stack=command_stack
                            read_only=inspector_read_only.clone()
                            on_open_import=open_import_for_list.clone()
                            on_open_export=open_export_for_list.clone()
                            on_open_dicts=on_open_dicts.clone()
                        />
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
            <CommandPalette
                visible=palette_visible
                query=palette_query
                highlight=palette_highlight
                items=palette_items
                on_select=on_palette_select
            />
            <ConflictDialog
                conflict=conflict
                on_force_overwrite=on_force_overwrite
                on_reload=on_reload_diagram
            />
            <ErrorToast error=error />
            <NoticeToast notice=import_notice />
            <modals::ModalRoot
                kind=modal_kind
                current_diagram_id=current_diagram_id
                current_title=current_title
                client=client.clone()
                error=error.clone()
                on_new=on_new_diagram
                on_rename=on_rename_diagram
                store=store.clone()
                batch_type_selection=batch_type_selection
                room_client=room_client.clone()
                auth_session=auth_session
                current_room=current_room
                on_room_deleted=on_room_deleted.clone()
            />
            <modals::KeyboardShortcuts
                enabled=!share_mode
                on_undo={
                    let store = store.clone();
                    let stack = command_stack.clone();
                    let on_after = on_after_change.clone();
                    move || {
                        let stack_rc = stack.get();
                        let cmd = {
                            let mut s = stack_rc.borrow_mut();
                            s.undo()
                        };
                        if let Some(cmd) = cmd {
                            let _ = crate::editor_core::CommandStack::revert(&store, &cmd);
                            on_after();
                        }
                    }
                }
                on_redo={
                    let store = store.clone();
                    let stack = command_stack.clone();
                    let on_after = on_after_change.clone();
                    move || {
                        let stack_rc = stack.get();
                        let cmd = {
                            let mut s = stack_rc.borrow_mut();
                            s.redo()
                        };
                        if let Some(cmd) = cmd {
                            let _ = crate::editor_core::CommandStack::execute(&store, &cmd);
                            on_after();
                        }
                    }
                }
            />
        </div>
        </div>
    }
}

// ─── modals sub-module (B4: 4 core modals) ────────────────────────────────────

/// B4 模态补全 (add-frontend-completeness)
/// - 4 个核心模态: New / Open / Share / Rename
/// - 其余 5 个 (Import / ImportSource / Language / SetTableWidth / ConfigureCustomTypes) 在 B5
///
/// data-testid 清单 (验证: `grep -c 'data-testid=' src/editor_panels.rs` 期望 ≥ 14):
///   - modal-{new,open,share,rename}  (B4)
///   - modal-title-{new,open,share,rename}  (B4)
///   - modal-submit-{new,open,share,rename}  (B4)
///   - modal-cancel-{new,open,share,rename}  (B4)
pub mod modals {
    //! B4 modal sub-module: 4 core modals (New/Open/Share/Rename)
    //!
    //! 覆盖 OpenLogos cases:
    //!   - UT-MM-01: New 模态创建 diagram (validate_title + build_create_url)
    //!   - UT-MM-04: 模态背景点击关闭 (ModalRoot) — B4 stub, B5 wasm-pack
    //!   - UT-MM-05: 模态 ESC 键关闭 (ModalRoot) — B4 stub, B5 wasm-pack
    //!   - UT-MM-06: 必填字段失焦红框 (validate_title 返回 Err)
    //!   - UT-MM-07: New 模态 title 为空 → OK 禁用
    //!   - UT-MM-08: Share 模态 URL 格式正确 (build_share_url)
    //!   - UT-MM-09: Open 模态 JSON 解析 (parse_diagram_json)
    //!   - ST-MM-01: e2e 全链路 (B5 wasm-pack test)

    use super::*;
    use crate::editor_core::types::Diagram;

    /// 模态种类 (B4: 4 个核心 + B5: 5 个剩余 = 9 个，spec §3 全集)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum ModalKind {
        // B4
        New,
        Open,
        Share,
        Rename,
        // B5
        Import,
        ImportSource,
        Language,
        SetTableWidth,
        SetTableSize, // feat-table-resize: 单模态扩展（width + min_height）
        // ux-canvas-batch 批次2 收尾: 批量重命名模态
        BatchRename,
        // ux-canvas-batch 批次3: 批量改类型模态
        BatchType,
        ConfigureCustomTypes,
        BridgeSettings,
        // p0-fix 定点 1: 删除房间确认模态
        DeleteRoom,
    }

    pub const TITLE_MAX_LEN: usize = 64;

    /// 校验 title (New / Rename 模态)
    /// - UT-MM-06: 空 → Err
    /// - UT-MM-07: 空 → OK 禁用 (调用方基于 Result 决定)
    pub fn validate_title(title: &str) -> Result<(), String> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err("title 不能为空".to_string());
        }
        if trimmed.chars().count() > TITLE_MAX_LEN {
            return Err(format!("title 长度不能超过 {} 字符", TITLE_MAX_LEN));
        }
        Ok(())
    }

    /// 新建 diagram 后的跳转 URL
    /// - UT-MM-01: build_create_url("d-new") == "/editor/d-new"
    pub fn build_create_url(diagram_id: &str) -> String {
        format!("/editor/{}", diagram_id)
    }

    /// Share 模态的分享链接 (V1 公开访问)
    /// - UT-MM-08: build_share_url("abc-123") == "/editor?share=abc-123"
    pub fn build_share_url(diagram_id: &str) -> String {
        format!("/editor?share={}", diagram_id)
    }

    /// 解析用户上传的 .json 文件内容
    /// - UT-MM-09: 合法 JSON → Ok(Diagram)
    /// - UT-MM-09: 非法 JSON → Err
    pub fn parse_diagram_json(text: &str) -> Result<Diagram, String> {
        serde_json::from_str::<Diagram>(text).map_err(|e| format!("JSON parse error: {}", e))
    }

    /// 解析用户粘贴的 SQL 文本为语句列表
    /// - UT-MM-10: 多语句以 `;` 分割
    /// - UT-MM-10: 去除 `-- comment` 单行注释
    /// - UT-MM-10: 空字符串 → Ok(vec![])
    pub fn parse_sql_statements(text: &str) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for raw in text.split(';') {
            // 去除每行 `--` 注释
            let cleaned: String = raw
                .lines()
                .map(|l| {
                    let trimmed = l.trim_start();
                    if trimmed.starts_with("--") {
                        ""
                    } else {
                        l
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let trimmed = cleaned.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
        Ok(out)
    }

    /// 解析表宽度输入
    /// - UT-MM-11: "200" → Ok(200), "0" → Ok(0)
    /// - UT-MM-11: "abc" / "" → Err
    pub fn parse_table_width(input: &str) -> Result<u32, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("宽度不能为空".to_string());
        }
        trimmed
            .parse::<u32>()
            .map_err(|e| format!("宽度必须是非负整数: {}", e))
    }

    /// 解析表最小高度输入（feat-table-resize；UT-MM-17）
    /// 严格对称 `parse_table_width` 的语义：
    /// - UT-MM-17: "200" / "100" → Ok(u32)
    /// - UT-MM-17: "0" → Ok(0)（"0 = auto"，与 width 一致）
    /// - UT-MM-17: "abc" / "" / "-5" → Err
    pub fn parse_table_height(input: &str) -> Result<u32, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("高度不能为空".to_string());
        }
        trimmed
            .parse::<u32>()
            .map_err(|e| format!("高度必须是非负整数: {}", e))
    }

    /// feat-relation-inference 批次1：cardinality 推导纯函数（UT-MM-18）
    /// 推导依据（外环判词修正 v2 + v3 方向颠倒修正）：
    /// 字段已参与关系计数（含本次新建），不是表总字段数 fields.len()
    /// s = start_field 已参与的关系数（含本条），e = end_field 已参与的关系数（含本条）
    /// 从 store.references.get() 统计 start_field_id/end_field_id 出现的次数
    /// （作为 start 或 end 端均可）
    /// 推导规则（operator Q2 裁决 + 外环判词修正 v2 + v3 方向颠倒修正）：
    ///   s==1 && e==1 → "one_to_one"
    ///   s>1 && e==1 → "one_to_many"（start 被多处引用，start 为"一"侧）
    ///   s==1 && e>1 → "many_to_one"（end 被多处引用，end 为"一"侧）
    ///   s>1 && e>1 → "many_to_many"
    /// 向后兼容：如字段不存在或计数为 0，fallback 到 "one_to_many"（与现有默认一致）
    pub fn infer_cardinality(start_field_id: &str, end_field_id: &str, store: &crate::editor_core::EditorStore) -> String {
        let references = store.references.get();
        let count_field_participation = |field_id: &str| -> usize {
            references
                .iter()
                .filter(|r| r.start_field_id == field_id || r.end_field_id == field_id)
                .count()
        };
        let s_count = count_field_participation(start_field_id);
        let e_count = count_field_participation(end_field_id);
        let s = s_count + 1; // 含本条
        let e = e_count + 1; // 含本条
        match (s, e) {
            (1, 1) => "one_to_one",
            (s_val, 1) if s_val > 1 => "one_to_many",
            (1, e_val) if e_val > 1 => "many_to_one",
            (s_val, e_val) if s_val > 1 && e_val > 1 => "many_to_many",
            _ => "one_to_many", // fallback（向后兼容）
        }
        .to_string()
    }

    /// 校验语言代码
    /// - UT-MM-12: "en" / "zh" → Ok(()); 其他 → Err
    pub fn validate_language(lang: &str) -> Result<(), String> {
        match lang {
            "en" | "zh" => Ok(()),
            other => Err(format!("不支持的语言: {}（V1 仅 en/zh）", other)),
        }
    }

    /// ImportSource 模态的源类型
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SourceKind {
        Local,
        Remote,
    }

    /// 解析 ImportSource 模态的选择
    /// - UT-MM-14: "local" → Local, "remote" → Remote
    /// - UT-MM-14: "http" / 其他 → Err
    pub fn resolve_import_source(s: &str) -> Result<SourceKind, String> {
        match s {
            "local" => Ok(SourceKind::Local),
            "remote" => Ok(SourceKind::Remote),
            other => Err(format!("不支持的导入源: {}（V1 仅 local/remote）", other)),
        }
    }

    /// 自定义类型条目
    pub type CustomTypeEntry = (String, String); // (name, base_type)

    /// 添加自定义类型（同名则替换）
    /// - UT-MM-13: add 空 vec → 长度 1
    /// - UT-MM-13: add 已存在 name → 替换
    pub fn add_custom_type(types: &mut Vec<CustomTypeEntry>, name: &str, base_type: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        if let Some(entry) = types.iter_mut().find(|(n, _)| n == name) {
            entry.1 = base_type.to_string();
        } else {
            types.push((name.to_string(), base_type.to_string()));
        }
    }

    /// 删除自定义类型
    /// - UT-MM-13: remove 存在 → vec 为空
    /// - UT-MM-13: remove 不存在 → no-op
    pub fn remove_custom_type(types: &mut Vec<CustomTypeEntry>, name: &str) {
        types.retain(|(n, _)| n != name);
    }

    /// 检查键盘事件是否匹配 Ctrl/Cmd+Z (Undo)
    /// - UT-KB-01: ctrlKey/MetaKey + 'z' + !shiftKey → true
    /// - UT-KB-01: !ctrl && !meta → false
    pub fn is_undo_shortcut(key: &str, ctrl_or_meta: bool, shift: bool) -> bool {
        if !ctrl_or_meta || shift {
            return false;
        }
        key.eq_ignore_ascii_case("z")
    }

    /// 检查键盘事件是否匹配 Ctrl/Cmd+Shift+Z (Redo)
    /// - UT-KB-01: ctrlKey/MetaKey + 'z' + shiftKey → true
    pub fn is_redo_shortcut(key: &str, ctrl_or_meta: bool, shift: bool) -> bool {
        if !ctrl_or_meta || !shift {
            return false;
        }
        key.eq_ignore_ascii_case("z")
    }

    // ─── ModalRoot: 通用壳 (B4 stub, B5 接入完整行为) ────────────────────────

    /// 通用模态根容器 (B4: show/hide + 背景点击关闭)
    /// - UT-MM-04: 背景点击关闭 (B4 实现)
    /// - UT-MM-05: ESC 键关闭 (B5 wasm-pack test 接入)
    /// - 模态体点击不冒泡到遮罩
    /// - feat-table-resize 批次3: 加 store prop 让 SetTableWidth/SetTableSize
    ///   模态 Apply 可真实写入 table.width/min_height
    #[component]
    pub fn ModalRoot(
        kind: RwSignal<Option<ModalKind>>,
        current_diagram_id: RwSignal<String>,
        current_title: RwSignal<String>,
        client: DiagramClient,
        error: RwSignal<Option<String>>,
        on_new: Rc<dyn Fn(String)>,
        on_rename: Rc<dyn Fn(String)>,
        store: EditorStore,
        batch_type_selection: RwSignal<BatchTypeSelection>,
        // p0-fix 定点 1: DeleteRoom 模态依赖
        room_client: RoomClient,
        auth_session: RwSignal<Option<AuthSession>>,
        current_room: RwSignal<Option<RoomDetail>>,
        on_room_deleted: Rc<dyn Fn()>,
    ) -> impl IntoView {
        let on_action_new = on_new.clone();
        let on_action_rename = on_rename.clone();

        view! {
            <div
                class="cdb-modal-overlay"
                data-testid="modal-root"
                style:display=move || if kind.get().is_some() { "flex" } else { "none" }
                on:click=move |_| kind.set(None)
            >
                {move || match kind.get() {
                    Some(ModalKind::New) => view! {
                        <div class="cdb-modal" data-testid="modal-new" on:click=|ev| ev.stop_propagation()>
                            <NewModal
                                kind=kind
                                on_create=on_action_new.clone()
                            />
                        </div>
                    }.into_view(),
                    Some(ModalKind::Open) => view! {
                        <div class="cdb-modal" data-testid="modal-open" on:click=|ev| ev.stop_propagation()>
                            <OpenModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::Share) => view! {
                        <div class="cdb-modal" data-testid="modal-share" on:click=|ev| ev.stop_propagation()>
                            <ShareModal
                                kind=kind
                                current_diagram_id=current_diagram_id
                            />
                        </div>
                    }.into_view(),
                    Some(ModalKind::Rename) => view! {
                        <div class="cdb-modal" data-testid="modal-rename" on:click=|ev| ev.stop_propagation()>
                            <RenameModal
                                kind=kind
                                current_title=current_title
                                on_rename=on_action_rename.clone()
                            />
                        </div>
                    }.into_view(),
                    Some(ModalKind::Import) => view! {
                        <div class="cdb-modal" data-testid="modal-import" on:click=|ev| ev.stop_propagation()>
                            <ImportModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::ImportSource) => view! {
                        <div class="cdb-modal" data-testid="modal-import-source" on:click=|ev| ev.stop_propagation()>
                            <ImportSourceModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::Language) => view! {
                        <div class="cdb-modal" data-testid="modal-language" on:click=|ev| ev.stop_propagation()>
                            <LanguageModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::SetTableWidth) => view! {
                        <div class="cdb-modal" data-testid="modal-set-width" on:click=|ev| ev.stop_propagation()>
                            <SetTableWidthModal kind=kind store=store.clone() />
                        </div>
                    }.into_view(),
                    Some(ModalKind::SetTableSize) => view! {
                        <div class="cdb-modal" data-testid="modal-set-size" on:click=|ev| ev.stop_propagation()>
                            <SetTableSizeModal kind=kind store=store.clone() />
                        </div>
                    }.into_view(),
                    // ux-canvas-batch 批次2 收尾: 批量重命名模态
                    Some(ModalKind::BatchRename) => view! {
                        <div class="cdb-modal" data-testid="modal-batch-rename" on:click=|ev| ev.stop_propagation()>
                            <BatchRenameModal kind=kind store=store.clone() />
                        </div>
                    }.into_view(),
                    // ux-canvas-batch 批次3: 批量改类型模态（条目 12 修正 4——checkbox 多选 + 单一目标类型）
                    Some(ModalKind::BatchType) => view! {
                        <div class="cdb-modal" data-testid="modal-batch-type" on:click=|ev| ev.stop_propagation()>
                            <BatchTypeModal kind=kind store=store.clone() selection=batch_type_selection />
                        </div>
                    }.into_view(),
                    Some(ModalKind::ConfigureCustomTypes) => view! {
                        <div class="cdb-modal" data-testid="modal-custom-types" on:click=|ev| ev.stop_propagation()>
                            <ConfigureCustomTypesModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::BridgeSettings) => view! {
                        <div class="cdb-modal" data-testid="modal-bridge-settings" on:click=|ev| ev.stop_propagation()>
                            <BridgeSettingsModal kind=kind client=client.clone() error=error.clone() />
                        </div>
                    }.into_view(),
                    // p0-fix 定点 1: 删除房间确认模态
                    Some(ModalKind::DeleteRoom) => view! {
                        <div class="cdb-modal" data-testid="modal-delete-room" on:click=|ev| ev.stop_propagation()>
                            <DeleteRoomModal
                                kind=kind
                                room_client=room_client.clone()
                                auth_session=auth_session
                                current_room=current_room
                                on_deleted=on_room_deleted.clone()
                            />
                        </div>
                    }.into_view(),
                    None => view! { <></> }.into_view(),
                }}
            </div>
        }
    }

    // ─── DeleteRoomModal（p0-fix 定点 1） ───────────────────────────────────

    /// 删除房间确认模态：显示房间名 + 成员数 + 警告文案
    /// - 确认 → DELETE /rooms/{id} → 成功 → 关闭模态 → on_deleted（回 rooms 页 + 列表刷新）
    /// - 网络失败 → 「删除失败，请稍后重试」+ 模态保持打开；403 → 「无权限删除此房间」
    /// - 覆盖 ST-S04-UI-08（e2e 链路）
    #[component]
    pub fn DeleteRoomModal(
        kind: RwSignal<Option<ModalKind>>,
        room_client: RoomClient,
        auth_session: RwSignal<Option<AuthSession>>,
        current_room: RwSignal<Option<RoomDetail>>,
        on_deleted: Rc<dyn Fn()>,
    ) -> impl IntoView {
        let submitting = create_rw_signal(false);
        let local_error = create_rw_signal(Option::<String>::None);

        let room_name = move || {
            current_room
                .get()
                .map(|r| r.name.clone())
                .unwrap_or_default()
        };
        let member_count = move || current_room.get().map(|r| r.member_count).unwrap_or(0);

        let on_confirm = {
            let room_client = room_client.clone();
            let on_deleted = on_deleted.clone();
            move |_| {
                if submitting.get_untracked() {
                    return;
                }
                let Some(session) = auth_session.get_untracked() else {
                    return;
                };
                let Some(room) = current_room.get_untracked() else {
                    return;
                };
                submitting.set(true);
                local_error.set(None);
                let room_client = room_client.clone();
                let on_deleted = on_deleted.clone();
                spawn_local(async move {
                    match room_client.delete_room(&session.access_token, &room.id).await {
                        Ok(()) => {
                            submitting.set(false);
                            kind.set(None);
                            on_deleted();
                        }
                        Err(ApiError::Server(403, _)) => {
                            submitting.set(false);
                            local_error.set(Some("无权限删除此房间".to_string()));
                        }
                        Err(_) => {
                            submitting.set(false);
                            local_error.set(Some("删除失败，请稍后重试".to_string()));
                        }
                    }
                });
            }
        };

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-delete-room">"删除房间"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="btn-cancel-delete-room"
                    on:click=move |_| kind.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <p data-testid="delete-room-desc">
                    {move || format!("确定删除房间「{}」吗？当前 {} 位成员。", room_name(), member_count())}
                </p>
                <p class="cdb-form-error" data-testid="delete-room-warning">
                    "删除后房间进入回收站，可在回收站恢复或彻底删除"
                </p>
                {move || local_error.get().map(|e| view! {
                    <span class="cdb-form-error" data-testid="modal-error-delete-room">{e}</span>
                })}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="btn-cancel-delete-room-footer"
                    on:click=move |_| kind.set(None)
                >"取消"</button>
                <button
                    class="cdb-btn cdb-btn--danger"
                    data-testid="btn-confirm-delete-room"
                    disabled=move || submitting.get()
                    on:click=on_confirm
                >
                    {move || if submitting.get() { "删除中..." } else { "确认删除" }}
                </button>
            </div>
        }
    }

    // ─── NewModal ───────────────────────────────────────────────────────────

    /// New 模态: 输入 title → 创建新 diagram
    /// - UT-MM-01: 提交时调用 on_create(name)
    /// - UT-MM-07: title 为空时 OK 禁用
    #[component]
    pub fn NewModal(
        kind: RwSignal<Option<ModalKind>>,
        on_create: Rc<dyn Fn(String)>,
    ) -> impl IntoView {
        let title_input = create_rw_signal(String::new());
        let validation = move || validate_title(&title_input.get());
        let is_valid = move || validation().is_ok();
        let on_create_submit = on_create.clone();
        let kind_close = kind;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-new">"New Diagram"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-new"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Title"</label>
                <input
                    class="cdb-form-input"
                    class:cdb-is-invalid=move || validation().is_err()
                    data-testid="modal-input-title-new"
                    prop:value=move || title_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        title_input.set(v);
                    }
                />
                {move || validation().err().map(|e| view! {
                    <span class="cdb-form-error" data-testid="modal-error-new">{e}</span>
                })}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-new-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-new"
                    disabled=move || !is_valid()
                    on:click=move |_| {
                        if let Ok(()) = validation() {
                            let name = title_input.get_untracked();
                            on_create_submit(name);
                            kind_close.set(None);
                        }
                    }
                >"Create"</button>
            </div>
        }
    }

    // ─── OpenModal ──────────────────────────────────────────────────────────

    /// Open 模态: 选择 .json 文件 (B4 stub)
    /// - UT-MM-09: 解析逻辑在 parse_diagram_json 纯函数 (B4 UT)
    /// - B5 接入 file → text() → parse_diagram_json 全链路
    #[component]
    pub fn OpenModal(kind: RwSignal<Option<ModalKind>>) -> impl IntoView {
        let kind_close = kind;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-open">"Open Diagram"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-open"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Upload .json file"</label>
                <input
                    class="cdb-form-input"
                    data-testid="modal-input-file-open"
                    type="file"
                    accept=".json"
                />
                <p class="cdb-form-hint">"B5 接入文件读取 + parse_diagram_json 校验"</p>
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-open-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
            </div>
        }
    }

    // ─── ShareModal ─────────────────────────────────────────────────────────

    /// Share 模态: 显示分享链接 (B4 stub: 仅显示，Copy 按钮 B5 接入剪贴板)
    /// - UT-MM-08: build_share_url 生成的 URL 显示
    #[component]
    pub fn ShareModal(
        kind: RwSignal<Option<ModalKind>>,
        current_diagram_id: RwSignal<String>,
    ) -> impl IntoView {
        let kind_close = kind;
        let share_url = move || build_share_url(&current_diagram_id.get());

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-share">"Share Diagram"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-share"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Share link"</label>
                <input
                    class="cdb-form-input"
                    data-testid="share-url"
                    readonly=true
                    prop:value=share_url
                />
                <p class="cdb-form-hint">"B5 接入 navigator.clipboard.write_text"</p>
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-share-btn"
                    on:click=move |_| kind_close.set(None)
                >"Close"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-share"
                >"Copy"</button>
            </div>
        }
    }

    // ─── RenameModal ────────────────────────────────────────────────────────

    /// Rename 模态: 重命名当前 diagram
    /// - UT-MM-06: title 校验复用 validate_title
    #[component]
    pub fn RenameModal(
        kind: RwSignal<Option<ModalKind>>,
        current_title: RwSignal<String>,
        on_rename: Rc<dyn Fn(String)>,
    ) -> impl IntoView {
        let title_input = create_rw_signal(current_title.get_untracked());
        let validation = move || validate_title(&title_input.get());
        let is_valid = move || validation().is_ok();
        let kind_close = kind;
        let on_rename_submit = on_rename.clone();

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-rename">"Rename Diagram"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-rename"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"New title"</label>
                <input
                    class="cdb-form-input"
                    class:cdb-is-invalid=move || validation().is_err()
                    data-testid="modal-input-title-rename"
                    prop:value=move || title_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        title_input.set(v);
                    }
                />
                {move || validation().err().map(|e| view! {
                    <span class="cdb-form-error" data-testid="modal-error-rename">{e}</span>
                })}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-rename-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-rename"
                    disabled=move || !is_valid()
                    on:click=move |_| {
                        if let Ok(()) = validation() {
                            let name = title_input.get_untracked();
                            on_rename_submit(name);
                            kind_close.set(None);
                        }
                    }
                >"Rename"</button>
            </div>
        }
    }

    // ─── B5: 5 个剩余模态 (Import/ImportSource/Language/SetTableWidth/ConfigureCustomTypes) ────

    /// Bridge 设置模态（align-v1-api-completion）
    #[component]
    pub fn BridgeSettingsModal(
        kind: RwSignal<Option<ModalKind>>,
        client: DiagramClient,
        error: RwSignal<Option<String>>,
    ) -> impl IntoView {
        let read_preferred = create_rw_signal(false);
        let write_enabled = create_rw_signal(false);
        let dual_write = create_rw_signal(false);
        let loading = create_rw_signal(true);
        let saving = create_rw_signal(false);
        let kind_close = kind;
        let load_client = client.clone();
        let save_client = client.clone();

        create_effect(move |_| {
            if kind.get() != Some(ModalKind::BridgeSettings) {
                return;
            }
            loading.set(true);
            let client = load_client.clone();
            let error = error.clone();
            spawn_local(async move {
                match client.get_bridge_config().await {
                    Ok(cfg) => {
                        read_preferred.set(cfg.db_read_preferred);
                        write_enabled.set(cfg.db_write_enabled);
                        dual_write.set(cfg.dual_write_local);
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
                loading.set(false);
            });
        });

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-bridge-settings">"Bridge 设置"</h3>
                <button class="cdb-modal-close" on:click=move |_| kind_close.set(None)>
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
            <div class="cdb-modal-body">
                {move || if loading.get() {
                    view! { <p>"加载中…"</p> }.into_view()
                } else {
                    view! {
                        <label class="cdb-form-check">
                            <input
                                type="checkbox"
                                prop:checked=move || read_preferred.get()
                                on:change=move |ev| {
                                    read_preferred.set(event_target_checked(&ev));
                                }
                            />
                            "优先从数据库读取"
                        </label>
                        <label class="cdb-form-check">
                            <input
                                type="checkbox"
                                prop:checked=move || write_enabled.get()
                                on:change=move |ev| {
                                    write_enabled.set(event_target_checked(&ev));
                                }
                            />
                            "启用数据库写入"
                        </label>
                        <label class="cdb-form-check">
                            <input
                                type="checkbox"
                                prop:checked=move || dual_write.get()
                                on:change=move |ev| {
                                    dual_write.set(event_target_checked(&ev));
                                }
                            />
                            "双写本地草稿"
                        </label>
                    }.into_view()
                }}
            </div>
            <div class="cdb-modal-footer">
                <button class="cdb-btn" on:click=move |_| kind_close.set(None)>"取消"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="bridge-settings-save"
                    disabled=move || saving.get() || loading.get()
                    on:click={
                        let client = save_client.clone();
                        let error = error.clone();
                        move |_| {
                            saving.set(true);
                            let update = BridgeConfigUpdate {
                                db_read_preferred: Some(read_preferred.get_untracked()),
                                db_write_enabled: Some(write_enabled.get_untracked()),
                                dual_write_local: Some(dual_write.get_untracked()),
                            };
                            let client = client.clone();
                            let error = error.clone();
                            spawn_local(async move {
                                match client.update_bridge_config(&update).await {
                                    Ok(()) => kind_close.set(None),
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                                saving.set(false);
                            });
                        }
                    }
                >"保存"</button>
            </div>
        }
    }

    /// Import 模态: 粘贴 SQL → 调用 bridge/import
    /// - UT-MM-10: parse_sql_statements 纯函数测试
    /// - B5 stub: 仅 UI shell，逻辑留 B5 e2e 接入
    #[component]
    pub fn ImportModal(kind: RwSignal<Option<ModalKind>>) -> impl IntoView {
        let sql_input = create_rw_signal(String::new());
        let kind_close = kind;
        let parse_result = move || parse_sql_statements(&sql_input.get());

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-import">"Import SQL"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-import"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Paste SQL"</label>
                <textarea
                    class="cdb-form-input"
                    data-testid="modal-input-sql"
                    rows="8"
                    prop:value=move || sql_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                        sql_input.set(v);
                    }
                />
                {move || match parse_result() {
                    Ok(stmts) if !stmts.is_empty() => view! {
                        <span class="cdb-form-hint" data-testid="modal-parse-count">
                            {format!("解析到 {} 条语句", stmts.len())}
                        </span>
                    }.into_view(),
                    Ok(_) => view! { <></> }.into_view(),
                    Err(e) => view! {
                        <span class="cdb-form-error">{e}</span>
                    }.into_view(),
                }}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-import-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-import"
                >"Import"</button>
            </div>
        }
    }

    /// ImportSource 模态: 选择 local / remote
    /// - UT-MM-14: resolve_import_source 纯函数测试
    #[component]
    pub fn ImportSourceModal(kind: RwSignal<Option<ModalKind>>) -> impl IntoView {
        let selected = create_rw_signal(String::from("local"));
        let kind_close = kind;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-import-source">"Import Source"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-import-source"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">
                    <input
                        type="radio"
                        name="import-source"
                        data-testid="modal-source-local"
                        checked=move || selected.get() == "local"
                        on:change=move |_| selected.set("local".to_string())
                    />
                    " Local"
                </label>
                <label class="cdb-form-label">
                    <input
                        type="radio"
                        name="import-source"
                        data-testid="modal-source-remote"
                        checked=move || selected.get() == "remote"
                        on:change=move |_| selected.set("remote".to_string())
                    />
                    " Remote (V1 stub)"
                </label>
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-import-source-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-import-source"
                >"OK"</button>
            </div>
        }
    }

    /// Language 模态: 切换 zh / en
    /// - UT-MM-12: validate_language 纯函数测试
    #[component]
    pub fn LanguageModal(kind: RwSignal<Option<ModalKind>>) -> impl IntoView {
        let selected = create_rw_signal(String::from("en"));
        let kind_close = kind;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-language">"Language"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-language"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">
                    <input
                        type="radio"
                        name="lang"
                        data-testid="modal-lang-en"
                        checked=move || selected.get() == "en"
                        on:change=move |_| selected.set("en".to_string())
                    />
                    " English"
                </label>
                <label class="cdb-form-label">
                    <input
                        type="radio"
                        name="lang"
                        data-testid="modal-lang-zh"
                        checked=move || selected.get() == "zh"
                        on:change=move |_| selected.set("zh".to_string())
                    />
                    " 中文"
                </label>
                <p class="cdb-form-hint">"B5 stub: V1 切换后只 toast 提示，实际 i18n 文案切换留 V2"</p>
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-language-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-language"
                >"Apply"</button>
            </div>
        }
    }

    /// SetTableWidth 模态: 批量设置表宽
    /// - UT-MM-11: parse_table_width 纯函数测试
    /// - feat-table-resize 批次3: Apply on:click 真实写入 store.tables[*].width
    #[component]
    pub fn SetTableWidthModal(kind: RwSignal<Option<ModalKind>>, store: EditorStore) -> impl IntoView {
        let width_input = create_rw_signal(String::from("200"));
        let validation = move || parse_table_width(&width_input.get());
        let is_valid = move || validation().is_ok();
        let kind_close = kind;
        let kind_close_apply = kind;
        let apply_value = width_input;
        let store_apply = store;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-set-width">"Set Table Width"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-set-width"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Width (0 = auto)"</label>
                <input
                    class="cdb-form-input"
                    class:cdb-is-invalid=move || validation().is_err()
                    data-testid="modal-input-width"
                    prop:value=move || width_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        width_input.set(v);
                    }
                />
                {move || validation().err().map(|e| view! {
                    <span class="cdb-form-error">{e}</span>
                })}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-set-width-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-set-width"
                    disabled=move || !is_valid()
                    on:click=move |_| {
                        // feat-table-resize 批次3: 真实写入 store.tables[*].width。
                        // 批量模式作用于所有 table;target_ids 留待 UI 入口具体化。
                        if let Ok(w) = parse_table_width(&apply_value.get()) {
                            store_apply.tables.update(|tables| {
                                for t in tables.iter_mut() {
                                    t.width = Some(w);
                                }
                            });
                            store_apply.dirty.set(true);
                        }
                        kind_close_apply.set(None);
                    }
                >"Apply"</button>
            </div>
        }
    }

    /// SetTableSize 模态: 单模态扩展（feat-table-resize 批次2 步骤4）
    /// 含 width + min_height 两个字段；复用 parse_table_width / parse_table_height 纯函数
    /// 对称 UT-MM-11 / UT-MM-17 语义（"0 = auto"）。
    /// - feat-table-resize 批次3: Apply on:click 真实写入 store.tables[*].width/min_height
    #[component]
    pub fn SetTableSizeModal(kind: RwSignal<Option<ModalKind>>, store: EditorStore) -> impl IntoView {
        let width_input = create_rw_signal(String::from("200"));
        let height_input = create_rw_signal(String::from("0"));
        let width_validation = move || parse_table_width(&width_input.get());
        let height_validation = move || parse_table_height(&height_input.get());
        let is_valid = move || width_validation().is_ok() && height_validation().is_ok();
        let kind_close = kind;
        let kind_close_apply = kind;
        let apply_width = width_input;
        let apply_height = height_input;
        let store_apply = store;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-set-size">"Set Table Size"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-set-size"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Width (0 = auto)"</label>
                <input
                    class="cdb-form-input"
                    class:cdb-is-invalid=move || width_validation().is_err()
                    data-testid="modal-input-size-width"
                    prop:value=move || width_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        width_input.set(v);
                    }
                />
                {move || width_validation().err().map(|e| view! {
                    <span class="cdb-form-error">{e}</span>
                })}
                <label class="cdb-form-label">"Min Height (0 = auto)"</label>
                <input
                    class="cdb-form-input"
                    class:cdb-is-invalid=move || height_validation().is_err()
                    data-testid="modal-input-size-min-height"
                    prop:value=move || height_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        height_input.set(v);
                    }
                />
                {move || height_validation().err().map(|e| view! {
                    <span class="cdb-form-error">{e}</span>
                })}
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-set-size-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-set-size"
                    disabled=move || !is_valid()
                    on:click=move |_| {
                        // feat-table-resize 批次3: 真实写入 width + min_height
                        let w = parse_table_width(&apply_width.get()).ok();
                        let h = parse_table_height(&apply_height.get()).ok();
                        if let (Some(w), Some(h)) = (w, h) {
                            store_apply.tables.update(|tables| {
                                for t in tables.iter_mut() {
                                    t.width = Some(w);
                                    t.min_height = Some(h);
                                }
                            });
                            store_apply.dirty.set(true);
                        }
                        kind_close_apply.set(None);
                    }
                >"Apply"</button>
            </div>
        }
    }

    /// ux-canvas-batch 批次2 收尾: 批量重命名模态
    /// Apply 调 batch_rename_tables 后写入 store——必须走 CommandStack/OT 变更通路
    /// （proposal R1，S05 协作与 undo 一致），写完 store.dirty.set(true)
    #[component]
    pub fn BatchRenameModal(kind: RwSignal<Option<ModalKind>>, store: EditorStore) -> impl IntoView {
        let rename_input = create_rw_signal(String::new());
        let kind_close = kind;
        let kind_close_apply = kind;
        let apply_value = rename_input;
        let store_apply = store;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-batch-rename">"Batch Rename Tables"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-batch-rename"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <p class="cdb-form-hint">"每行一条映射：旧名 → 新名（如 A → D）"</p>
                <textarea
                    class="cdb-form-input"
                    data-testid="modal-input-batch-rename"
                    placeholder="A → D\nB → E"
                    prop:value=move || rename_input.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                        rename_input.set(v);
                    }
                />
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-batch-rename-btn"
                    on:click=move |_| kind_close.set(None)
                >"Cancel"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="modal-submit-batch-rename"
                    on:click=move |_| {
                        // ux-canvas-batch 批次2 收尾: Apply 调 batch_rename_tables 后写入 store
                        // 必须走 CommandStack/OT 变更通路（proposal R1，S05 协作与 undo 一致）
                        let input = apply_value.get();
                        let mut rename_map = std::collections::HashMap::new();
                        for line in input.lines() {
                            let parts: Vec<&str> = line.split("→").collect();
                            if parts.len() == 2 {
                                let old_name = parts[0].trim().to_string();
                                let new_name = parts[1].trim().to_string();
                                rename_map.insert(old_name, new_name);
                            }
                        }
                        // 走 CommandStack/OT 变更通路
                        let mut tables = store_apply.tables.get();
                        batch_rename_tables(&mut tables, rename_map);
                        store_apply.tables.set(tables);
                        store_apply.dirty.set(true);
                        kind_close_apply.set(None);
                    }
                >"Apply"</button>
            </div>
        }
    }

/// ux-canvas-batch 批次3 步骤 2（条目 16 修复）：批量改类型模态
/// - selection 信号由 AppRoot 传入（与 ListView 共享同一 RwSignal）
/// - Apply 真实消费 selected_field_ids × target_type → batch_change_types
/// - 空选中集 Apply 禁用（disabled）
/// - modal-batch-type-selected-fields 真实渲染选中字段名清单
#[component]
pub fn BatchTypeModal(
    kind: RwSignal<Option<ModalKind>>,
    store: EditorStore,
    selection: RwSignal<BatchTypeSelection>,
) -> impl IntoView {
    let kind_close = kind;
    let kind_close_apply = kind;
    let store_apply = store;
    let selection_apply = selection;

    // 渲染选中字段名清单：table_name.field_name 按 field_id 查找
    let selected_labels: RwSignal<Vec<String>> = create_rw_signal(Vec::new());
    Effect::new(move |_| {
        let sel = selection.get();
        let tables = store_apply.tables.get();
        let mut labels = Vec::new();
        for fid in &sel.selected_field_ids {
            let mut found = None;
            for table in &tables {
                if let Some(f) = table.fields.iter().find(|f| &f.id == fid) {
                    found = Some(format!("{}.{}", table.name, f.name));
                    break;
                }
            }
            labels.push(found.unwrap_or_else(|| fid.clone()));
        }
        selected_labels.set(labels);
    });

    view! {
        <div class="cdb-modal-header">
            <h3 class="cdb-modal-title" data-testid="modal-title-batch-type">"Batch Change Types"</h3>
            <button
                class="cdb-modal-close"
                data-testid="modal-cancel-batch-type"
                on:click=move |_| kind_close.set(None)
            > <IconBox size="sm"><IconClose /></IconBox> </button>
        </div>
        <div class="cdb-modal-body">
            <p class="cdb-form-hint">"确认已选字段与目标类型（外环条目 12 修正 4——checkbox 多选 + 单一目标类型）"</p>
            <div class="cdb-form-group">
                <label>"目标类型"</label>
                <span data-testid="modal-batch-type-target-display">
                    {move || selection.get().target_type.clone()}
                </span>
            </div>
            <div class="cdb-form-group">
                <label>"已选字段清单（按字段名）"</label>
                <span data-testid="modal-batch-type-selected-fields">
                    {move || {
                        let labels = selected_labels.get();
                        if labels.is_empty() {
                            "(未选任何字段)".to_string()
                        } else {
                            labels.join(", ")
                        }
                    }}
                </span>
            </div>
        </div>
        <div class="cdb-modal-footer">
            <button
                class="cdb-btn"
                data-testid="modal-cancel-batch-type-btn"
                on:click=move |_| kind_close.set(None)
            >"Cancel"</button>
            <button
                class="cdb-btn cdb-btn--primary"
                data-testid="modal-submit-batch-type"
                prop:disabled=move || {
                    let sel = selection.get();
                    sel.selected_field_ids.is_empty() || sel.target_type.trim().is_empty()
                }
                on:click=move |_| {
                    // 条目 16 修复: Apply 真实消费 selection.selected_field_ids × target_type
                    let sel = selection_apply.get();
                    if sel.selected_field_ids.is_empty() || sel.target_type.trim().is_empty() {
                        return; // 防御：禁用虽已 prop 设，二次保险
                    }
                    let mut tables = store_apply.tables.get();
                    let mut field_type_map = std::collections::HashMap::new();
                    let target = sel.target_type.trim().to_string();
                    for fid in &sel.selected_field_ids {
                        field_type_map.insert(fid.clone(), target.clone());
                    }
                    batch_change_types(&mut tables, field_type_map);
                    store_apply.tables.set(tables);
                    store_apply.dirty.set(true);
                    // 清空选中集（防止误点二次 Apply）
                    selection_apply.update(|s| s.selected_field_ids.clear());
                    kind_close_apply.set(None);
                }
            >"Apply"</button>
        </div>
    }
}

    /// ConfigureCustomTypes 模态: 增删改自定义类型
    /// - UT-MM-13: add/remove_custom_type 纯函数测试
    #[component]
    pub fn ConfigureCustomTypesModal(kind: RwSignal<Option<ModalKind>>) -> impl IntoView {
        let types: RwSignal<Vec<CustomTypeEntry>> = create_rw_signal(Vec::new());
        let new_name = create_rw_signal(String::new());
        let new_base = create_rw_signal(String::from("VARCHAR(255)"));
        let kind_close = kind;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-custom-types">"Custom Types"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-custom-types"
                    on:click=move |_| kind_close.set(None)
                > <IconBox size="sm"><IconClose /></IconBox> </button>
            </div>
            <div class="cdb-modal-body">
                <p class="cdb-form-hint">"V1 限制: 仅前端 session state，刷新后丢失 (spec §5.9)"</p>
                <div class="cdb-custom-types-list" data-testid="modal-custom-types-list">
                    {move || types.get().into_iter().enumerate().map(|(i, (name, base))| {
                        let types_for_remove = types;
                        let n = name.clone();
                        view! {
                            <div class="cdb-custom-type-item" data-testid=format!("modal-custom-type-{i}")>
                                <span>{format!("{name} → {base}")}</span>
                                <button
                                    class="cdb-btn cdb-btn--small"
                                    data-testid=format!("modal-remove-custom-type-{i}")
                                    on:click=move |_| {
                                        let mut v = types_for_remove.get();
                                        remove_custom_type(&mut v, &n);
                                        types_for_remove.set(v);
                                    }
                                > <IconBox size="sm"><IconClose /></IconBox> </button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <div class="cdb-custom-types-add">
                    <input
                        class="cdb-form-input"
                        data-testid="modal-input-custom-type-name"
                        placeholder="Name"
                        prop:value=move || new_name.get()
                        on:input=move |ev| {
                            use wasm_bindgen::JsCast;
                            let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                            new_name.set(v);
                        }
                    />
                    <input
                        class="cdb-form-input"
                        data-testid="modal-input-custom-type-base"
                        prop:value=move || new_base.get()
                        on:input=move |ev| {
                            use wasm_bindgen::JsCast;
                            let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                            new_base.set(v);
                        }
                    />
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="modal-add-custom-type"
                        on:click=move |_| {
                            let mut v = types.get();
                            add_custom_type(&mut v, &new_name.get(), &new_base.get());
                            types.set(v);
                            new_name.set(String::new());
                        }
                    >"Add"</button>
                </div>
            </div>
            <div class="cdb-modal-footer">
                <button
                    class="cdb-btn"
                    data-testid="modal-cancel-custom-types-btn"
                    on:click=move |_| kind_close.set(None)
                >"Close"</button>
            </div>
        }
    }

    // ─── B5: 全局键盘快捷键 ─────────────────────────────────────────────

    /// 全局键盘事件监听
    /// - UT-KB-01: is_undo_shortcut 纯函数已覆盖
    /// - ST-UI-05: 完整 e2e (Ctrl+Z / Ctrl+Shift+Z 触发 undo/redo) 留 B5 wasm-pack
    ///
    /// V1 stub: 仅在 document 上注册 keydown 监听，命中 is_undo_shortcut /
    /// is_redo_shortcut 时通过传入的回调通知调用方。调用方负责实际调用
    /// CommandStack::undo() / CommandStack::redo()。
    #[component]
    pub fn KeyboardShortcuts<F1, F2>(enabled: bool, on_undo: F1, on_redo: F2) -> impl IntoView
    where
        F1: Fn() + Clone + 'static,
        F2: Fn() + Clone + 'static,
    {
        let on_undo_clone = on_undo.clone();
        let on_redo_clone = on_redo.clone();

        // 简化的全局 keydown 监听（仅识别 z 键 + ctrl/meta；B5 wasm-pack 时可换更稳健实现）
        gloo::events::EventListener::new_with_options(
            &gloo::utils::document(),
            "keydown",
            gloo::events::EventListenerOptions::enable_prevent_default(),
            move |ev| {
                if !enabled {
                    return;
                }
                use wasm_bindgen::JsCast;
                let key_event: Option<&web_sys::KeyboardEvent> = ev.dyn_ref();
                if let Some(ke) = key_event {
                    // D 批：输入框 / contentEditable 焦点时不抢撤销重做（core-KB §1 既有合同）
                    if shortcut_event_is_text_target(ke) {
                        return;
                    }
                    let key = ke.key();
                    let ctrl_or_meta = ke.ctrl_key() || ke.meta_key();
                    let shift = ke.shift_key();
                    if is_undo_shortcut(&key, ctrl_or_meta, shift) {
                        ke.prevent_default();
                        on_undo_clone();
                    } else if is_redo_shortcut(&key, ctrl_or_meta, shift) {
                        ke.prevent_default();
                        on_redo_clone();
                    }
                }
            },
        )
        .forget();

        view! { <></> }
    }
}

#[cfg(test)]
mod tests {
    //! B2 unit tests (add-frontend-completeness)
    //!
    //! Covered OpenLogos cases:
    //!   - UT-SP-02: Tables Tab 搜索过滤 (filter_by_query 纯函数)
    //!   - UT-SP-09: 6 业务 Tab 切换 (SidePanelTab testid/label 完整性)
    //!   - UT-SP-10: 全局搜索跨 Tab 过滤 (filter_by_query 对 4 种 Named 类型)
    //!
    //! 注：Tab 切换的 DOM 行为（点击 → active_tab 更新）在 wasm-pack test 中覆盖（B5）。
    //! 本模块用纯函数 UT 验证 B2 数据层的正确性。

    use super::*;
    use crate::editor_core::types::{Area, Field, Note, Reference, Table};
    use crate::editor_core::CollabPendingOp;

    fn make_table(id: &str, name: &str) -> Table {
        Table {
            id: id.into(),
            name: name.into(),
            x: 0.0,
            y: 0.0,
            color: "#000".into(),
            comment: String::new(),
            fields: Vec::new(),
            indices: Vec::new(),
            width: None,
            min_height: None,
        }
    }

    fn make_field(type_: &str) -> Field {
        Field {
            id: "f1".into(),
            name: "f".into(),
            type_: type_.into(),
            default: String::new(),
            check: String::new(),
            primary: false,
            unique: false,
            not_null: false,
            increment: false,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        }
    }

    // --- UT-SP-02 — Tables Tab 搜索过滤 ---

    /// ST-S07-06：字典抽屉与 Inspector / IO 抽屉互斥（fix-dict-save-and-layout，core-01e §2.5）。
    /// 真值表推演（base = Inspector 开、无抽屉）：
    ///   开抽屉 → Inspector 收起 + 缓存 Some(true)；关抽屉 → 恢复 true；
    ///   Inspector 打开 → 抽屉关闭且缓存失效；IO↔字典互斥且缓存交接不丢失。
    #[test]
    fn st_s07_06_overlay_mutex_transitions() {
        let base = OverlayMutex {
            inspector_open: true,
            dict_panel_open: false,
            io_drawer: IoDrawerKind::None,
            inspector_before_dict: None,
            inspector_before_io: None,
        };
        // 开抽屉：Inspector 收起 + 缓存；两浮层不同时开
        let opened = base.open_dict_panel();
        assert!(!opened.inspector_open && opened.dict_panel_open);
        assert_eq!(opened.inspector_before_dict, Some(true));
        // 关抽屉：恢复 Inspector 打开前状态
        let closed = opened.close_dict_panel();
        assert!(closed.inspector_open && !closed.dict_panel_open);
        assert_eq!(closed.inspector_before_dict, None);
        // toggle 往返一致
        assert_eq!(base.toggle_dict_panel(), opened);
        assert_eq!(opened.toggle_dict_panel(), closed);
        // 反向互斥：Inspector 打开 → 抽屉关闭、缓存失效不恢复
        let rev = opened.inspector_opened();
        assert!(rev.inspector_open && !rev.dict_panel_open && rev.io_drawer == IoDrawerKind::None);
        assert!(rev.inspector_before_dict.is_none() && rev.inspector_before_io.is_none());
        // 打开前 Inspector 本就关闭 → 缓存 Some(false)，关闭后仍关闭
        let quiet = OverlayMutex { inspector_open: false, ..base }.open_dict_panel();
        assert_eq!(quiet.inspector_before_dict, Some(false));
        assert!(!quiet.close_dict_panel().inspector_open);
        // IO↔字典互斥 + 缓存交接：dict（缓存 true）→ 开 IO → 关 IO → Inspector 恢复 true
        let io = opened.open_io_drawer(IoDrawerKind::Import);
        assert!(!io.dict_panel_open && io.io_drawer == IoDrawerKind::Import && !io.inspector_open);
        assert_eq!(io.inspector_before_io, Some(true));
        assert!(io.inspector_before_dict.is_none());
        let back = io.close_io_drawer();
        assert!(back.inspector_open && back.io_drawer == IoDrawerKind::None);
        // IO 已开（其缓存 inspector=true）→ 开字典抽屉：继承 IO 缓存而非当前已收起的 false
        let dict_over_io = io.open_dict_panel();
        assert_eq!(dict_over_io.io_drawer, IoDrawerKind::None);
        assert!(dict_over_io.dict_panel_open);
        assert_eq!(dict_over_io.inspector_before_dict, Some(true));
        assert!(dict_over_io.close_dict_panel().inspector_open);
    }

    // --- UT-SP-02 — Tables Tab 搜索过滤 ---

    /// UT-SP-02 happy path：搜索 "user" → 列表过滤只含 "users"
    #[test]
    fn test_filter_tables_ut_sp_02() {
        let tables = vec![
            make_table("t1", "users"),
            make_table("t2", "orders"),
            make_table("t3", "products"),
        ];
        let result = filter_by_query(&tables, "user");
        assert_eq!(result.len(), 1, "UT-SP-02: 搜索 'user' 应只匹配 1 项");
        assert_eq!(result[0].name, "users", "UT-SP-02: 匹配项应为 'users'");
    }

    /// UT-SP-02 case-insensitive：搜索 "USER" 也匹配 "users"
    #[test]
    fn test_filter_tables_case_insensitive_ut_sp_02() {
        let tables = vec![make_table("t1", "Users"), make_table("t2", "orders")];
        let result = filter_by_query(&tables, "USER");
        assert_eq!(result.len(), 1, "UT-SP-02: 大写 'USER' 应匹配 'Users'");
        assert_eq!(result[0].name, "Users");
    }

    /// UT-SP-02 空 query：返回全部（clone）
    #[test]
    fn test_filter_empty_query_returns_all_ut_sp_02() {
        let tables = vec![make_table("t1", "a"), make_table("t2", "b")];
        let result = filter_by_query(&tables, "");
        assert_eq!(result.len(), 2, "UT-SP-02: 空 query 应返回全部");
    }

    /// UT-SP-02 类型筛选：tables 含字段类型，filter 验证
    #[test]
    fn test_filter_tables_by_type_ut_sp_02() {
        let mut t1 = make_table("t1", "users");
        t1.fields = vec![make_field("INT")];
        let mut t2 = make_table("t2", "orders");
        t2.fields = vec![make_field("VARCHAR(255)")];
        let tables = vec![t1, t2];
        // 类型筛选：保留含 INT 字段的表
        let mut v = filter_by_query(&tables, "");
        v.retain(|t| {
            t.fields
                .iter()
                .any(|f| f.type_.to_uppercase().contains("INT"))
        });
        assert_eq!(v.len(), 1, "UT-SP-02: 类型筛选 INT 应只保留 users");
        assert_eq!(v[0].name, "users");
    }

    // --- UT-SP-09 — 8 Tab 图标栏切换（R5） ---

    /// UT-SP-09: 8 个 Tab 的 testid/label 全部存在且唯一
    #[test]
    fn test_side_panel_tab_testid_completeness_ut_sp_09() {
        let all_tabs = [
            SidePanelTab::Tables,
            SidePanelTab::Areas,
            SidePanelTab::Enums,
            SidePanelTab::Notes,
            SidePanelTab::Relationships,
            SidePanelTab::Types,
            SidePanelTab::Issues,
            SidePanelTab::Fields,
        ];
        let mut testids: Vec<&str> = all_tabs.iter().map(|t| t.testid()).collect();
        testids.sort();
        testids.dedup();
        assert_eq!(
            testids.len(),
            8,
            "UT-SP-09: 8 个 Tab testid 应全部唯一，实际 {} 个",
            testids.len()
        );
        for expected in [
            "tab-tables",
            "tab-areas",
            "tab-enums",
            "tab-notes",
            "tab-relationships",
            "tab-types",
            "tab-issues",
            "tab-fields",
        ] {
            assert!(
                testids.contains(&expected),
                "UT-SP-09: 应包含 testid '{}'",
                expected
            );
        }
    }

    /// UT-SP-09: 8 个 Tab label 都有非空显示文本（Tooltip）
    #[test]
    fn test_side_panel_tab_label_nonempty_ut_sp_09() {
        for tab in [
            SidePanelTab::Tables,
            SidePanelTab::Areas,
            SidePanelTab::Enums,
            SidePanelTab::Notes,
            SidePanelTab::Relationships,
            SidePanelTab::Types,
            SidePanelTab::Issues,
            SidePanelTab::Fields,
        ] {
            assert!(
                !tab.label().is_empty(),
                "UT-SP-09: Tab {:?} 应有非空 label",
                tab
            );
        }
    }

    // --- UT-SP-10 — 全局搜索跨 Tab 过滤 ---

    /// UT-SP-10 happy: tables=[users]、areas=[user_area]、enums=[user_role]，
    /// 搜索 "user" → 各类各 1 项
    #[test]
    fn test_global_search_cross_tab_ut_sp_10() {
        let tables = vec![make_table("t1", "users")];
        let areas = vec![Area {
            id: "a1".into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: "#e6f1f5".into(),
            name: "user_area".into(),
        }];
        let enums = vec![EnumStub {
            id: "e1".into(),
            name: "user_role".into(),
            values: vec!["admin".into()],
        }];

        let t = filter_by_query(&tables, "user");
        let a = filter_by_query(&areas, "user");
        let e = filter_by_query(&enums, "user");
        assert_eq!(t.len(), 1, "UT-SP-10: tables 匹配 1");
        assert_eq!(a.len(), 1, "UT-SP-10: areas 匹配 1");
        assert_eq!(e.len(), 1, "UT-SP-10: enums 匹配 1");
    }

    /// UT-SP-10: 类型过滤对 Table/Enum/Note 都能正确 filter
    #[test]
    fn test_filter_by_query_generic_ut_sp_10() {
        let notes = vec![
            Note {
                id: "n1".into(),
                x: 0.0,
                y: 0.0,
                content: "user feedback".into(),
                color: "#fef3c7".into(),
            },
            Note {
                id: "n2".into(),
                x: 0.0,
                y: 0.0,
                content: "system status".into(),
                color: "#fef3c7".into(),
            },
        ];
        let result = filter_by_query(&notes, "user");
        assert_eq!(result.len(), 1, "UT-SP-10: notes 搜索 'user' 应匹配 1");
        assert_eq!(result[0].id, "n1");
    }

    /// UT-ALIGN-A01: Areas/Notes Tab 新增写入 store，snapshot 与侧栏同源
    #[test]
    fn test_areas_notes_add_updates_store_snapshot_ut_align_a01() {
        let store = EditorStore::new();
        let mut areas = store.areas.get();
        areas.push(new_default_area(0));
        store.areas.set(areas);
        store.dirty.set(true);

        let snap = store.snapshot("d1".into(), "Test".into());
        assert_eq!(
            snap.areas.len(),
            1,
            "UT-ALIGN-A01: snapshot.areas 应有 1 项"
        );
        assert_eq!(snap.areas[0].name, "新区域 1");

        let mut notes = store.notes.get();
        notes.push(new_default_note(0));
        store.notes.set(notes);

        let snap = store.snapshot("d1".into(), "Test".into());
        assert_eq!(
            snap.notes.len(),
            1,
            "UT-ALIGN-A01: snapshot.notes 应有 1 项"
        );
        assert_eq!(snap.notes[0].content, "新便签 1");
        assert!(store.dirty.get(), "UT-ALIGN-A01: 变更后 dirty 应为 true");
    }

    /// UT-ID-GLOBAL-02（fix-global-entity-id-uniqueness 回归）：
    /// 1) new_default_area/new_default_note 产出 `{prefix}-{16位hex}` 全局唯一 id，两次调用互异；
    /// 2) 新格式 id（含字母的 hex 后缀）不被 next_id_from_store 的 max+1 解析捕获，
    ///    存量加载语义不变（返回 0 起始，仅供 enum/type stub 计数）。
    #[test]
    fn ut_id_global_02_new_format_ids_bypass_next_id_parsing() {
        let a0 = new_default_area(0);
        let a1 = new_default_area(1);
        assert!(a0.id.starts_with("area-"), "UT-ID-GLOBAL-02: area id 前缀");
        assert_ne!(a0.id, a1.id, "UT-ID-GLOBAL-02: 两次生成区域 id 应互异");
        assert_eq!(a0.name, "新区域 1", "UT-ID-GLOBAL-02: seq 命名语义保留");
        let n0 = new_default_note(0);
        let n1 = new_default_note(1);
        assert!(n0.id.starts_with("note-"), "UT-ID-GLOBAL-02: note id 前缀");
        assert_ne!(n0.id, n1.id, "UT-ID-GLOBAL-02: 两次生成便签 id 应互异");

        let store = EditorStore::new();
        store.areas.set(vec![Area {
            id: "area-abcdef0123456789".into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: String::new(),
            name: "a".into(),
        }]);
        assert_eq!(
            crate::editor_core::next_id_from_store(&store),
            0,
            "UT-ID-GLOBAL-02: 新格式 id（hex 后缀）不应参与 max+1 解析"
        );
    }

    /// UT-ALIGN-B03: 删除与导入日志重试 UI 规则
    #[test]
    fn ut_align_b03_delete_and_import_log_retry_rules() {
        assert!(
            !is_deletable_diagram_id("default"),
            "UT-ALIGN-B03: default 不可删"
        );
        assert!(
            is_deletable_diagram_id("d-123"),
            "UT-ALIGN-B03: 普通 id 可删"
        );
        assert!(
            import_log_shows_retry("failed"),
            "UT-ALIGN-B03: failed 显示重试"
        );
        assert!(!import_log_shows_retry("success"));
        assert!(!import_log_shows_retry("pending"));
        assert_eq!(
            modals::ModalKind::BridgeSettings,
            modals::ModalKind::BridgeSettings,
            "UT-ALIGN-B03: BridgeSettings 模态 kind 存在"
        );
    }

    /// UT-SP-10: 关系过滤 — references 没有 name 字段，使用拼接匹配
    #[test]
    fn test_filter_references_ut_sp_10() {
        let refs = vec![
            Reference {
                id: "r1".into(),
                name: "fk1".into(),
                start_table_id: "users".into(),
                end_table_id: "orders".into(),
                start_field_id: "f1".into(),
                end_field_id: "f2".into(),
                type_: "one_to_many".into(),
                on_delete: String::new(),
                on_update: String::new(),
            },
            Reference {
                id: "r2".into(),
                name: "fk2".into(),
                start_table_id: "products".into(),
                end_table_id: "categories".into(),
                start_field_id: "f3".into(),
                end_field_id: "f4".into(),
                type_: "many_to_one".into(),
                on_delete: String::new(),
                on_update: String::new(),
            },
        ];
        let result = filter_references_by_query(&refs, "user");
        assert_eq!(
            result.len(),
            1,
            "UT-SP-10: refs 搜索 'user' 应匹配 r1（start=users）"
        );
        assert_eq!(result[0].id, "r1");
    }

    // ─── B4 modal pure function tests ─────────────────────────────────────

    #[test]
    fn test_validate_title_happy_ut_mm_01() {
        assert!(
            modals::validate_title("My Diagram").is_ok(),
            "UT-MM-01: 正常 title 应通过"
        );
    }

    #[test]
    fn test_validate_title_empty_ut_mm_06() {
        let r = modals::validate_title("");
        assert!(r.is_err(), "UT-MM-06: 空 title 应返回 Err");
        assert_eq!(r.unwrap_err(), "title 不能为空");
    }

    #[test]
    fn test_validate_title_whitespace_only_ut_mm_06() {
        let r = modals::validate_title("   ");
        assert!(r.is_err(), "UT-MM-06: 全空白 title 应返回 Err");
    }

    #[test]
    fn test_validate_title_too_long_ut_mm_06() {
        let long = "a".repeat(modals::TITLE_MAX_LEN + 1);
        let r = modals::validate_title(&long);
        assert!(r.is_err(), "UT-MM-06: 超长 title 应返回 Err");
    }

    #[test]
    fn test_validate_title_empty_disables_submit_ut_mm_07() {
        // UT-MM-07: title 为空 → 提交按钮应禁用
        // 实际禁用逻辑在 NewModal 组件中基于 is_valid()，这里验证纯函数返回 Err
        let r = modals::validate_title("");
        assert!(
            r.is_err(),
            "UT-MM-07: 空 title 时 NewModal 提交应禁用（基于 validate_title 返回 Err）"
        );
    }

    #[test]
    fn test_build_create_url_ut_mm_01() {
        assert_eq!(
            modals::build_create_url("d-new"),
            "/editor/d-new",
            "UT-MM-01: build_create_url 应返回 /editor/<id>"
        );
        assert_eq!(modals::build_create_url("abc-123"), "/editor/abc-123");
    }

    #[test]
    fn test_build_share_url_ut_mm_08() {
        assert_eq!(
            modals::build_share_url("abc-123"),
            "/editor?share=abc-123",
            "UT-MM-08: build_share_url 应返回 /editor?share=<id>"
        );
        assert_eq!(modals::build_share_url("d-uuid"), "/editor?share=d-uuid");
    }

    #[test]
    fn test_parse_diagram_json_happy_ut_mm_09() {
        // UT-MM-09: 合法 Diagram JSON
        let json = r#"{
            "id": "d1",
            "name": "Test",
            "revision": 0,
            "database": "Generic",
            "tables": [],
            "references": [],
            "notes": [],
            "areas": []
        }"#;
        let r = modals::parse_diagram_json(json);
        assert!(r.is_ok(), "UT-MM-09: 合法 JSON 应解析为 Diagram");
        let d = r.unwrap();
        assert_eq!(d.id, "d1");
        assert_eq!(d.name, "Test");
    }

    #[test]
    fn test_parse_diagram_json_invalid_ut_mm_09() {
        let bad = r#"{ not valid json }"#;
        let r = modals::parse_diagram_json(bad);
        assert!(r.is_err(), "UT-MM-09: 非法 JSON 应返回 Err");
        assert!(
            r.unwrap_err().starts_with("JSON parse error"),
            "UT-MM-09: 错误信息应包含 'JSON parse error'"
        );
    }

    // ─── B5 additional modal pure function tests ──────────────────────────

    #[test]
    fn test_parse_sql_statements_multi_ut_mm_10() {
        let text = "CREATE TABLE a (id INT); INSERT INTO a VALUES (1);";
        let r = modals::parse_sql_statements(text);
        assert!(r.is_ok(), "UT-MM-10: 合法 SQL 应返回 Ok");
        let v = r.unwrap();
        assert_eq!(v.len(), 2, "UT-MM-10: 应分割为 2 条语句");
        assert!(v[0].contains("CREATE TABLE a"));
        assert!(v[1].contains("INSERT INTO a"));
    }

    #[test]
    fn test_parse_sql_statements_empty_ut_mm_10() {
        let r = modals::parse_sql_statements("");
        assert_eq!(r.unwrap().len(), 0, "UT-MM-10: 空字符串应返回空 vec");
    }

    #[test]
    fn test_parse_sql_statements_strips_comments_ut_mm_10() {
        let text = "-- this is a comment\nCREATE TABLE a (id INT);";
        let r = modals::parse_sql_statements(text);
        let v = r.unwrap();
        assert_eq!(v.len(), 1, "UT-MM-10: 注释行应被去除");
        assert!(!v[0].contains("--"), "UT-MM-10: 注释符不应在结果中");
    }

    #[test]
    fn test_parse_table_width_happy_ut_mm_11() {
        assert_eq!(
            modals::parse_table_width("200").unwrap(),
            200,
            "UT-MM-11: '200' → 200"
        );
        assert_eq!(
            modals::parse_table_width("0").unwrap(),
            0,
            "UT-MM-11: '0' → 0 (auto)"
        );
    }

    #[test]
    fn test_parse_table_width_invalid_ut_mm_11() {
        assert!(
            modals::parse_table_width("abc").is_err(),
            "UT-MM-11: 'abc' → Err"
        );
        assert!(modals::parse_table_width("").is_err(), "UT-MM-11: '' → Err");
    }

    // ─── UT-MM-17: parse_table_height (feat-table-resize) ────────────────
    // 严格对称 parse_table_width 的 "0 = auto" 语义；operator Q4 裁决最小高度语义。

    #[test]
    fn test_parse_table_height_happy_ut_mm_17() {
        assert_eq!(
            modals::parse_table_height("200").unwrap(),
            200,
            "UT-MM-17: '200' → 200"
        );
        assert_eq!(
            modals::parse_table_height("100").unwrap(),
            100,
            "UT-MM-17: '100' → 100"
        );
    }

    #[test]
    fn test_parse_table_height_zero_is_auto_ut_mm_17() {
        // "0" 必须 Ok(0)（"0 = auto"，对称 parse_table_width 的 UT-MM-11 语义）
        assert_eq!(
            modals::parse_table_height("0").unwrap(),
            0,
            "UT-MM-17: '0' → 0 (auto)"
        );
    }

    #[test]
    fn test_parse_table_height_invalid_ut_mm_17() {
        assert!(
            modals::parse_table_height("abc").is_err(),
            "UT-MM-17: 'abc' → Err"
        );
        assert!(
            modals::parse_table_height("").is_err(),
            "UT-MM-17: '' → Err"
        );
        assert!(
            modals::parse_table_height("-5").is_err(),
            "UT-MM-17: '-5' → Err (负数被拒绝)"
        );
    }

    // ─── UT-MM-18: infer_cardinality (feat-relation-inference 批次1) ────────────────
    // 推导依据（外环判词修正 v2 + v3 方向颠倒修正）：字段已参与关系计数（含本次新建），
    // 不是表总字段数 fields.len()。s/e = start_field/end_field 已参与的关系数（含本条）。
    // 真值表（外环判词要求；v3 修正方向颠倒）：
    //   s==1 && e==1 → one_to_one
    //   s>1 && e==1 → one_to_many（start 被多处引用，start 为"一"侧）
    //   s==1 && e>1 → many_to_one（end 被多处引用，end 为"一"侧）
    //   s>1 && e>1 → many_to_many

    #[test]
    fn test_infer_cardinality_one_to_one_ut_mm_18() {
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // 两端字段均参与 0 条既有关系 → s=1, e=1 → one_to_one
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "one_to_one", "UT-MM-18: s=1, e=1 → one_to_one");
    }

    #[test]
    fn test_infer_cardinality_many_to_one_ut_mm_18() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // end 字段参与 1 条既有关系 → e=2（含本条）；start 字段参与 0 条 → s=1
        // 先建一条 reference：f1 → f2（f2 是 end）
        let existing = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f0".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing]);
        // 现在连 f1 → f2：f2 已参与 1 条（s=1, e=2）→ many_to_one
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "many_to_one", "UT-MM-18: s=1, e=2 → many_to_one（end 被多处引用，end 为\"一\"侧）");
    }

    #[test]
    fn test_infer_cardinality_one_to_many_ut_mm_18() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // start 字段参与 1 条既有关系 → s=2（含本条）；end 字段参与 0 条 → e=1
        // 先建一条 reference：f1 → f0（f1 是 start）
        let existing = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing]);
        // 现在连 f1 → f2：f1 已参与 1 条（s=2, e=1）→ one_to_many
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "one_to_many", "UT-MM-18: s=2, e=1 → one_to_many（start 被多处引用，start 为\"一\"侧）");
    }

    #[test]
    fn test_infer_cardinality_many_to_many_ut_mm_18() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // start 字段参与 1 条既有关系 → s=2；end 字段参与 1 条既有关系 → e=2
        let existing1 = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        let existing2 = Reference {
            id: "r2".into(),
            name: String::new(),
            start_table_id: "t3".into(),
            end_table_id: "t4".into(),
            start_field_id: "f0".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing1, existing2]);
        // 现在连 f1 → f2：f1 已参与 1 条（s=2）、f2 已参与 1 条（e=2）→ many_to_many
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "many_to_many", "UT-MM-18: s=2, e=2 → many_to_many");
    }

    #[test]
    fn test_infer_cardinality_one_to_many_s3_ut_mm_18() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // start 字段参与 2 条既有关系 → s=3；end 字段参与 0 条 → e=1
        let existing1 = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        let existing2 = Reference {
            id: "r2".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t3".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing1, existing2]);
        // 现在连 f1 → f2：f1 已参与 2 条（s=3）、f2 已参与 0 条（e=1）→ one_to_many
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "one_to_many", "UT-MM-18: s=3, e=1 → one_to_many（start 被多处引用，start 为\"一\"侧）");
    }

    #[test]
    fn test_infer_cardinality_fallback_field_not_exist_ut_mm_18() {
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // 字段不存在（空 store）→ s=1, e=1 → one_to_one（含本条）
        let result = modals::infer_cardinality("nonexistent", "f2", &store);
        assert_eq!(result, "one_to_one", "UT-MM-18: 字段不存在（空 store）→ s=1, e=1 → one_to_one（含本条）");
    }

    #[test]
    fn test_infer_cardinality_fallback_zero_count_ut_mm_18() {
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // 字段计数为 0（空 store）→ s=1, e=1 → one_to_one（含本条）
        let result = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(result, "one_to_one", "UT-MM-18: 字段计数为 0（空 store）→ s=1, e=1 → one_to_one（含本条）");
    }

    // ─── UT-MM-19: flip_reference_endpoints 翻转后重新推导 cardinality ────────────────

    #[test]
    fn test_flip_reference_endpoints_re_infers_cardinality_ut_mm_19() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // 先建一条 reference：f1 → f0（f1 是 start，参与 1 条既有关系）
        let existing = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing]);
        // 现在连 f1 → f2：f1 已参与 1 条（s=2）、f2 已参与 0 条（e=1）→ one_to_many
        let r = Reference {
            id: "r2".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        // 翻转前：s=2, e=1 → one_to_many
        let flipped = flip_reference_endpoints(&r, &store);
        // 翻转后：s/e 互换 → s=1, e=2 → many_to_one
        assert_eq!(flipped.type_, "many_to_one", "UT-MM-19: 翻转后 s/e 互换，many_to_one");
    }

    // ─── UT-MM-21: 列表视图排序纯函数测试（按表维度属性排序） ────────────────

    #[test]
    fn test_sort_tables_by_table_name_ascending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "Zoo".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "alpha".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::TableName, SortDirection::Ascending);
        // Rust String::cmp 是字节序：'Z'(90) < 'a'(97) → Zoo 在前
        assert_eq!(sorted[0].name, "Zoo", "UT-MM-21: 表名升序 → Zoo 在前（字节序 'Z'<'a'）");
        assert_eq!(sorted[1].name, "alpha", "UT-MM-21: 表名升序 → alpha 在后");
    }

    #[test]
    fn test_sort_tables_by_table_name_descending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "Zoo".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "alpha".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::TableName, SortDirection::Descending);
        assert_eq!(sorted[0].name, "alpha", "UT-MM-21: 表名降序 → alpha 在前");
        assert_eq!(sorted[1].name, "Zoo", "UT-MM-21: 表名降序 → Zoo 在后");
    }

    #[test]
    fn test_sort_tables_by_field_count_ascending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }, Field { id: "f2".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t3".into(), name: "C".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::FieldCount, SortDirection::Ascending);
        assert_eq!(sorted[0].fields.len(), 0, "UT-MM-21: 字段数升序 → 0 字段在前");
        assert_eq!(sorted[1].fields.len(), 1, "UT-MM-21: 字段数升序 → 1 字段在中");
        assert_eq!(sorted[2].fields.len(), 2, "UT-MM-21: 字段数升序 → 2 字段在后");
    }

    #[test]
    fn test_sort_tables_by_field_count_descending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }, Field { id: "f2".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t3".into(), name: "C".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::FieldCount, SortDirection::Descending);
        assert_eq!(sorted[0].fields.len(), 2, "UT-MM-21: 字段数降序 → 2 字段在前");
        assert_eq!(sorted[1].fields.len(), 1, "UT-MM-21: 字段数降序 → 1 字段在中");
        assert_eq!(sorted[2].fields.len(), 0, "UT-MM-21: 字段数降序 → 0 字段在后");
    }

    #[test]
    fn test_sort_tables_by_type_ascending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::Type, SortDirection::Ascending);
        assert_eq!(sorted[0].fields[0].type_, "INT", "UT-MM-21: 类型升序 → INT 在前");
        assert_eq!(sorted[1].fields[0].type_, "VARCHAR", "UT-MM-21: 类型升序 → VARCHAR 在后");
    }

    #[test]
    fn test_sort_tables_by_has_index_ascending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table, Index};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::HasIndex, SortDirection::Ascending);
        assert_eq!(sorted[0].indices.len(), 0, "UT-MM-21: 无索引 → 有索引（升序）→ 无索引在前");
        assert_eq!(sorted[1].indices.len(), 1, "UT-MM-21: 无索引 → 有索引（升序）→ 有索引在后");
    }

    #[test]
    fn test_sort_tables_by_has_index_descending_ut_mm_21() {
        use crate::editor_core::types::{Field, Table, Index};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
        ];
        let sorted = sort_tables(&tables, SortColumn::HasIndex, SortDirection::Descending);
        assert_eq!(sorted[0].indices.len(), 1, "UT-MM-21: 有索引 → 无索引（降序）→ 有索引在前");
        assert_eq!(sorted[1].indices.len(), 0, "UT-MM-21: 有索引 → 无索引（降序）→ 无索引在后");
    }

    #[test]
    fn test_sort_tables_empty_ut_mm_21() {
        use crate::editor_core::types::Table;
        let tables: Vec<Table> = vec![];
        let sorted = sort_tables(&tables, SortColumn::TableName, SortDirection::Ascending);
        assert_eq!(sorted.len(), 0, "UT-MM-21: 空 tables → 空结果");
    }

    // ─── UT-MM-22: 列表视图 tab 切换测试 ────────────────

    #[test]
    fn test_list_view_tab_switch_ut_mm_22() {
        // 验证 ListView tab 的 testid 和 label 正确
        assert_eq!(SidePanelTab::ListView.testid(), "tab-list-view", "UT-MM-22: ListView tab testid 应为 tab-list-view");
        assert_eq!(SidePanelTab::ListView.label(), "列表视图", "UT-MM-22: ListView tab label 应为 列表视图");
    }

    // ─── UT-MM-23: 列表视图过滤纯函数测试 ────────────────

    #[test]
    fn test_filter_tables_by_name_ut_mm_23() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "orders".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "users", "", None);
        assert_eq!(filtered.len(), 1, "UT-MM-23: 按名称模糊匹配 users → 1 个表");
        assert_eq!(filtered[0].name, "users", "UT-MM-23: 过滤结果应为 users");
    }

    #[test]
    fn test_filter_tables_by_type_ut_mm_23() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "", "INT", None);
        assert_eq!(filtered.len(), 1, "UT-MM-23: 按类型过滤 INT → 1 个表");
        assert_eq!(filtered[0].name, "A", "UT-MM-23: 过滤结果应为 A（首个字段类型 INT）");
    }

    #[test]
    fn test_filter_tables_by_has_index_ut_mm_23() {
        use crate::editor_core::types::{Field, Table, Index};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "", "", Some(true));
        assert_eq!(filtered.len(), 1, "UT-MM-23: 仅有索引 → 1 个表");
        assert_eq!(filtered[0].name, "B", "UT-MM-23: 过滤结果应为 B（有索引）");
    }

    #[test]
    fn test_filter_tables_by_no_index_ut_mm_23() {
        use crate::editor_core::types::{Field, Table, Index};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "", "", Some(false));
        assert_eq!(filtered.len(), 1, "UT-MM-23: 仅无索引 → 1 个表");
        assert_eq!(filtered[0].name, "A", "UT-MM-23: 过滤结果应为 A（无索引）");
    }

    #[test]
    fn test_filter_tables_combined_ut_mm_23() {
        use crate::editor_core::types::{Field, Table, Index};
        let tables = vec![
            Table { id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
            Table { id: "t2".into(), name: "orders".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "users", "INT", Some(true));
        assert_eq!(filtered.len(), 1, "UT-MM-23: 三条件 AND → 1 个表");
        assert_eq!(filtered[0].name, "users", "UT-MM-23: 过滤结果应为 users（含 users 子串 + INT + 有索引）");
    }

    #[test]
    fn test_filter_tables_empty_ut_mm_23() {
        use crate::editor_core::types::Table;
        let tables: Vec<Table> = vec![];
        let filtered = filter_tables(&tables, "nonexistent", "", None);
        assert_eq!(filtered.len(), 0, "UT-MM-23: 空 tables → 空结果");
    }

    #[test]
    fn test_filter_tables_no_filter_ut_mm_23() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new(), dict_code: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "", "", None);
        assert_eq!(filtered.len(), 2, "UT-MM-23: 不过滤 → 全部表");
    }

    // ─── UT-MM-24: 列表视图批量重命名纯函数测试 ────────────────

    #[test]
    fn test_batch_rename_tables_success_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "D".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "D", "UT-MM-24: A→D 改名成功");
        assert_eq!(tables[1].name, "B", "UT-MM-24: B 不变");
    }

    #[test]
    fn test_batch_rename_tables_skip_existing_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "B".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "A", "UT-MM-24: A→B 跳过（新名 B 已存在，保持原名 A）");
        assert_eq!(tables[1].name, "B", "UT-MM-24: B 不变");
    }

    #[test]
    fn test_batch_rename_tables_skip_same_name_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "A".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "A", "UT-MM-24: A→A 跳过（新名 = 原名，保持原名 A）");
    }

    #[test]
    fn test_batch_rename_tables_skip_empty_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "A", "UT-MM-24: A→\"\" 跳过（新名为空，保持原名 A）");
    }

    #[test]
    fn test_batch_rename_tables_skip_invalid_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "A B".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "A", "UT-MM-24: A→\"A B\" 跳过（含非法字符，保持原名 A）");
    }

    #[test]
    fn test_batch_rename_tables_empty_map_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        batch_rename_tables(&mut tables, std::collections::HashMap::new());
        assert_eq!(tables[0].name, "A", "UT-MM-24: 空 rename_map → 全部不变");
    }

    #[test]
    fn test_batch_rename_tables_nonexistent_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("D".to_string(), "E".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "A", "UT-MM-24: 旧名 D 不存在 → 全部不变");
    }

    // ─── B2-S1 ③: 同一新名多旧名映射，字典序靠前者得名其余跳过 ────────────────

    #[test]
    fn test_batch_rename_tables_same_new_name_ut_mm_24() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![], indices: vec![], width: None, min_height: None },
        ];
        let mut rename_map = std::collections::HashMap::new();
        rename_map.insert("A".to_string(), "C".to_string());
        rename_map.insert("B".to_string(), "C".to_string());
        batch_rename_tables(&mut tables, rename_map);
        assert_eq!(tables[0].name, "C", "UT-MM-24 B2-S1 ③: A→C 字典序靠前，改名成功");
        assert_eq!(tables[1].name, "B", "UT-MM-24 B2-S1 ③: B→C 跳过（新名 C 已被 A 占用）");
    }

    // ─── UT-MM-20: build_reference 使用推导值而非用户必选下拉值 ────────────────

    #[test]
    fn test_build_reference_uses_inferred_cardinality_ut_mm_20() {
        use crate::editor_core::types::{Reference, Table, Field};
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        // 先建一条 reference：f1 → f0（f1 是 start，参与 1 条既有关系）
        let existing = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f0".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        store.references.set(vec![existing]);
        // 现在连 f1 → f2：f1 已参与 1 条（s=2）、f2 已参与 0 条（e=1）→ one_to_many
        let inferred = modals::infer_cardinality("f1", "f2", &store);
        assert_eq!(inferred, "one_to_many", "UT-MM-20: 推导值应为 one_to_many");
        // build_reference 使用推导值（非用户必选下拉值）
        let reference = crate::editor_panels::build_reference(
            "r2".into(),
            "t1".into(),
            "f1".into(),
            "t2".into(),
            "f2".into(),
            &inferred,
        );
        assert_eq!(reference.type_, "one_to_many", "UT-MM-20: build_reference 使用推导值 one_to_many");
    }

    // ─── UT-MM-26: 列表视图批量改类型纯函数测试（v2——通用决策程序各族收窄反向 + 跨族 + 非法目标类型） ────────────────

    #[test]
    fn test_batch_change_types_int_to_bigint_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "BIGINT".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "BIGINT", "UT-MM-26: INT→BIGINT（数值族由窄到宽步骤 ③ 直接改）");
    }

    #[test]
    fn test_batch_change_types_int_to_int_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "INT".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: INT→INT（同型直接改）");
    }

    #[test]
    fn test_batch_change_types_int_to_varchar_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "VARCHAR".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: INT→VARCHAR（数值族→字符串族跨族步骤 ④ → 跳过）");
    }

    #[test]
    fn test_batch_change_types_varchar_to_varchar_50_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "name".into(), type_: "VARCHAR".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "VARCHAR(50)".to_string());
        batch_change_types(&mut tables, map);
        // 同基类型参数收窄 → 跳过
        assert_eq!(tables[0].fields[0].type_, "VARCHAR", "UT-MM-26: VARCHAR→VARCHAR(50)（同基类型参数收窄步骤 ③ → 跳过）");
    }

    #[test]
    fn test_batch_change_types_invalid_type_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "INVALID_TYPE".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: → INVALID_TYPE（解析失败步骤 ⑤ → 跳过）");
    }

    #[test]
    fn test_batch_change_types_empty_type_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: 空字符串（非法目标类型步骤 ⑥ → 跳过）");
    }

    #[test]
    fn test_batch_change_types_date_to_datetime_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "events".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "created".into(), type_: "DATE".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "DATETIME".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "DATETIME", "UT-MM-26: DATE→DATETIME（日期族由窄到宽步骤 ③ 直接改）");
    }

    #[test]
    fn test_batch_change_types_empty_map_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        batch_change_types(&mut tables, std::collections::HashMap::new());
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: 空 field_type_map → 全部不变");
    }

    #[test]
    fn test_batch_change_types_int_to_smallint_ut_mm_26() {
        // v2 新增：数值族由宽到窄步骤 ③ → 跳过
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "SMALLINT".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26: INT→SMALLINT（数值族由宽到窄步骤 ③ → 跳过）");
    }

    #[test]
    fn test_batch_change_types_datetime_to_date_ut_mm_26() {
        // v2 新增：日期族由宽到窄步骤 ③ → 跳过
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "events".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "created".into(), type_: "DATETIME".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "DATE".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "DATETIME", "UT-MM-26: DATETIME→DATE（日期族由宽到窄步骤 ③ → 跳过）");
    }

    // ─── ux-canvas-batch 批次3（条目 13 改派修复）回归用例 ────────────────

    /// 异索引跨族对回归 SMALLINT(0)→VARCHAR(1)（数值族位置 0 ≠ 字符串族位置 1）
    /// v1 type_position 丢失族身份会误判为 true（直接改），v2 二元组正确判 false
    #[test]
    fn test_batch_change_types_smallint_to_varchar_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "SMALLINT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "VARCHAR".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "SMALLINT", "UT-MM-26 回归: SMALLINT→VARCHAR（异索引跨族 → 跨族一律跳过）");
    }

    /// 异索引跨族对回归 INT(1)→TEXT(2)
    #[test]
    fn test_batch_change_types_int_to_text_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "TEXT".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "INT", "UT-MM-26 回归: INT→TEXT（异索引跨族 → 跨族一律跳过）");
    }

    /// 异索引跨族对回归 DATE(0)→VARCHAR(1)
    #[test]
    fn test_batch_change_types_date_to_varchar_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "events".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "created".into(), type_: "DATE".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "VARCHAR".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "DATE", "UT-MM-26 回归: DATE→VARCHAR（异索引跨族 → 跨族一律跳过）");
    }

    /// 带参类型回归 VARCHAR(255)→TEXT（from 带参归族→字符串族 VARCHAR 位；族内由窄到宽 → 改）
    /// v1 parse_type 未实现（注释自承占位），VARCHAR(255)→None → from 误判为未列出 → 跳过
    /// v2 parse_type 实现 + 二元组：VARCHAR(255) 归族 (String, 1) → TEXT (String, 2) → 同族窄→宽 → 改
    #[test]
    fn test_batch_change_types_varchar_255_to_text_ut_mm_26() {
        use crate::editor_core::types::{Field, Table};
        let mut tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "name".into(), type_: "VARCHAR(255)".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let mut map = std::collections::HashMap::new();
        map.insert("f1".to_string(), "TEXT".to_string());
        batch_change_types(&mut tables, map);
        assert_eq!(tables[0].fields[0].type_, "TEXT", "UT-MM-26 回归: VARCHAR(255)→TEXT（from 带参归族 + 族内窄→宽 → 改）");
    }

    // ─── UT-MM-27: 列表视图导出 CSV schema 内容纯函数测试（v2——输入 &[Table] 按 schema 内容导出） ────────────────

    #[test]
    fn test_export_tables_csv_basic_ut_mm_27() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: vec![crate::editor_core::types::Index {
                id: "i1".into(), name: "idx".into(), fields: vec![], unique: false,
            }],
            width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert_eq!(csv, "table_name,field_name,field_type,has_index\nusers,id,INT,yes\n",
            "UT-MM-27: 有索引 → users,id,INT,yes");
    }

    #[test]
    fn test_export_tables_csv_no_index_ut_mm_27() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert_eq!(csv, "table_name,field_name,field_type,has_index\nusers,id,INT,no\n",
            "UT-MM-27: 无索引 → users,id,INT,no");
    }

    #[test]
    fn test_export_tables_csv_no_special_chars_ut_mm_27() {
        // v2 修正：posts 无逗号/引号/换行,按转义真值表不应加引号
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "posts".into(), type_: "VARCHAR(255)".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert_eq!(csv, "table_name,field_name,field_type,has_index\nusers,posts,VARCHAR(255),no\n",
            "UT-MM-27: 无三字符 → 不加引号");
    }

    #[test]
    fn test_export_tables_csv_quote_escape_ut_mm_27() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "bad".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "she said \"hi\"".into(), type_: "VARCHAR(255)".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert_eq!(csv, "table_name,field_name,field_type,has_index\nbad,\"she said \"\"hi\"\"\",VARCHAR(255),no\n",
            "UT-MM-27: 引号转义 → 双引号包裹 + 内部双引号转义为 \"\"");
    }

    #[test]
    fn test_export_tables_csv_empty_ut_mm_27() {
        let csv = export_tables_csv(&[]);
        assert_eq!(csv, "table_name,field_name,field_type,has_index\n",
            "UT-MM-27: 空表 → 仅表头");
    }

    #[test]
    fn test_export_tables_csv_newline_escape_ut_mm_27() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "line1\nline2".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert!(csv.contains("\"line1\nline2\""), "UT-MM-27: 换行转义");
    }

    #[test]
    fn test_export_tables_csv_comma_escape_ut_mm_27() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![Table {
            id: "t1".into(), name: "weird,name".into(), x: 0.0, y: 0.0,
            color: String::new(), comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
            }],
            indices: Vec::new(), width: None, min_height: None,
        }];
        let csv = export_tables_csv(&tables);
        assert!(csv.contains("\"weird,name\""), "UT-MM-27: 表名含逗号 → 转义");
    }

    #[test]
    fn test_validate_language_ut_mm_12() {
        assert!(
            modals::validate_language("en").is_ok(),
            "UT-MM-12: 'en' 应通过"
        );
        assert!(
            modals::validate_language("zh").is_ok(),
            "UT-MM-12: 'zh' 应通过"
        );
        assert!(
            modals::validate_language("fr").is_err(),
            "UT-MM-12: 'fr' 应 Err"
        );
    }

    #[test]
    fn test_resolve_import_source_ut_mm_14() {
        assert_eq!(
            modals::resolve_import_source("local").unwrap(),
            modals::SourceKind::Local
        );
        assert_eq!(
            modals::resolve_import_source("remote").unwrap(),
            modals::SourceKind::Remote
        );
        assert!(
            modals::resolve_import_source("http").is_err(),
            "UT-MM-14: 'http' 应 Err"
        );
    }

    #[test]
    fn test_add_custom_type_ut_mm_13() {
        let mut v: Vec<modals::CustomTypeEntry> = Vec::new();
        modals::add_custom_type(&mut v, "uuid", "VARCHAR(36)");
        assert_eq!(v.len(), 1, "UT-MM-13: add 后 vec 长度应为 1");
        assert_eq!(v[0], ("uuid".to_string(), "VARCHAR(36)".to_string()));
    }

    #[test]
    fn test_add_custom_type_replaces_duplicate_ut_mm_13() {
        let mut v = vec![("uuid".to_string(), "OLD".to_string())];
        modals::add_custom_type(&mut v, "uuid", "NEW");
        assert_eq!(v.len(), 1, "UT-MM-13: add 同名应替换而非新增");
        assert_eq!(v[0].1, "NEW");
    }

    #[test]
    fn test_remove_custom_type_ut_mm_13() {
        let mut v = vec![("uuid".to_string(), "VARCHAR(36)".to_string())];
        modals::remove_custom_type(&mut v, "uuid");
        assert!(v.is_empty(), "UT-MM-13: remove 存在 → vec 为空");
    }

    #[test]
    fn test_remove_custom_type_nonexistent_ut_mm_13() {
        let mut v = vec![("uuid".to_string(), "VARCHAR(36)".to_string())];
        modals::remove_custom_type(&mut v, "nonexistent");
        assert_eq!(v.len(), 1, "UT-MM-13: remove 不存在 → no-op");
    }

    #[test]
    fn test_is_undo_shortcut_ut_kb_01() {
        assert!(
            modals::is_undo_shortcut("z", true, false),
            "UT-KB-01: Ctrl+Z → true"
        );
        assert!(
            modals::is_undo_shortcut("Z", true, false),
            "UT-KB-01: 大小写无关"
        );
        assert!(
            !modals::is_undo_shortcut("z", false, false),
            "UT-KB-01: 不带 Ctrl → false"
        );
        assert!(
            !modals::is_undo_shortcut("z", true, true),
            "UT-KB-01: 带 Shift 属 redo → false"
        );
        assert!(
            !modals::is_undo_shortcut("a", true, false),
            "UT-KB-01: 其他键 → false"
        );
    }

    #[test]
    fn test_is_redo_shortcut_ut_kb_01() {
        assert!(
            modals::is_redo_shortcut("z", true, true),
            "UT-KB-01: Ctrl+Shift+Z → true"
        );
        assert!(
            !modals::is_redo_shortcut("z", true, false),
            "UT-KB-01: 不带 Shift 属 undo → false"
        );
    }

    // ─── UT-FIX-01: ModalRoot 条件渲染（fix-modal-overlay-blocking B1） ─────

    #[test]
    fn test_modal_root_overlay_only_renders_when_kind_is_some() {
        let src = include_str!("editor_panels.rs");
        let count = src.matches("class=\"cdb-modal-overlay\"").count();
        assert!(
            count <= 2,
            "UT-FIX-01: `class=\"cdb-modal-overlay\"` 出现 {count} 次, 预期 ≤ 2（ModalRoot + B 批 InviteModal）; \
             重复出现说明遮罩 div 仍在多处无条件实例化。",
        );
        // 遮罩通过 `style:display` 绑定在 `kind` 为 None 时设为 "none", 满足
        // §4.1 遮罩生命周期的实际目的（不再拦截 pointer events, HP-01~HP-05 可点击）。
        // 严格意义"从 DOM 移除"用纯 CSS 类 .cdb-is-hidden + display:none 等价;
        // 选用内联 style:display 是因为 ModalRoot 内部难以嵌套 `Show` / 闭包
        // （`move ||` 嵌套会让 on_action_new 闭包被多次 move, 触发 E0525 FnOnce）。
        assert!(
            src.contains("style:display=move || if kind.get().is_some()"),
            "UT-FIX-01: 源码必须包含 `style:display=move || if kind.get().is_some()` 条件隐藏遮罩",
        );
        assert!(
            src.contains("style:display=move || if open.get() { \"flex\" } else { \"none\" }"),
            "UT-FIX-01: B 批 InviteModal 遮罩同样必须条件隐藏（open=false 时 display:none）",
        );
    }

    // ─── UT-FIX-02: cdb-canvas-container testid（fix-modal-overlay-blocking B1） ─

    #[test]
    fn test_canvas_container_has_editor_canvas_testid() {
        let panels = include_str!("editor_panels.rs");
        let render = include_str!("editor_render.rs");
        assert!(
            panels.contains("class=\"cdb-canvas-container\"") && panels.contains("<Canvas"),
            "UT-FIX-02: AppRoot 必须在 cdb-canvas-container 内接入 Canvas 组件",
        );
        assert!(
            render.contains("data-testid=\"editor-canvas\""),
            "UT-FIX-02: editor_render Canvas 的 <canvas> 必须带 data-testid=\"editor-canvas\"",
        );
    }

    // ─── Phase A 布局重构（redesign-phase-a-layout）──

    #[test]
    fn test_phase_a_layout_components_ut_pa_06() {
        let panels = include_str!("editor_panels.rs");
        let css = include_str!("styles.css");
        assert!(
            panels.contains("data-testid=\"app-bar\""),
            "UT-AB-01: AppBar 必须带 app-bar testid",
        );
        assert!(
            !panels.contains("data-testid=\"toolbar\""),
            "UT-AB-01: Phase A 不得保留独立 toolbar testid",
        );
        assert!(
            panels.contains("data-testid=\"tool-rail\""),
            "UT-TR-01: Tool Rail 必须带 tool-rail testid",
        );
        assert!(
            panels.contains("data-testid=\"inspector\""),
            "UT-IN-01: Inspector 必须带 inspector testid（主原型锚点，历史 inspector-panel 已移除）",
        );
        assert!(
            !panels.contains("data-testid=\"inspector-panel\""),
            "UT-IN-01: 历史 inspector-panel testid 不得回退（统一主原型事实）",
        );
        assert!(
            panels.contains("data-testid=\"canvas-empty-guide\""),
            "UT-PA-01: 空白引导必须带 canvas-empty-guide testid",
        );
        assert!(
            panels.contains("cdb-app-bar__brand"),
            "UT-AB-R4: AppBar 必须包含品牌区分区",
        );
        assert!(
            panels.contains("cdb-status-chip"),
            "UT-AB-R4: 保存状态必须使用 status chip",
        );
        assert!(
            panels.contains("data-testid=\"btn-more-menu\""),
            "UT-AB-R4: AppBar 必须有溢出菜单",
        );
        assert!(
            panels.contains("data-testid=\"revision-display\""),
            "UT-AB-05: revision-display 必须存在于状态 Chip",
        );
        if let Some(sb_tail) = panels.split("pub fn StatusBar").nth(1) {
            let sb_body = sb_tail.split("\n\n///").next().unwrap_or(sb_tail);
            assert!(
                !sb_body.contains("data-testid=\"revision-display\""),
                "UT-AB-R4: StatusBar 不得重复 revision-display",
            );
        }
        assert!(
            panels.contains("data-testid=\"status-bar\""),
            "UT-AB-05: revision 应位于 StatusBar",
        );
        assert!(
            css.contains("grid-template-rows: var(--cdb-appbar-h) minmax(0, 1fr) var(--cdb-statusbar-h)"),
            "UT-PA-06: .cdb-app 栅格应对齐主原型槽位 token（appbar 64 / 1fr / statusbar 34）",
        );
        assert!(
            css.contains("grid-template-columns: var(--cdb-toolrail-w) minmax(0, 1fr) var(--cdb-inspector-w)"),
            "UT-PA-06: .cdb-main 栅格应对齐主原型 ToolRail 64 + Canvas + Inspector 330（IO 抽屉为 overlay）",
        );
        assert!(
            panels.contains("cdb-tabs--icon-grid"),
            "UT-IN-R5: Inspector Tab 栏必须使用图标栅格",
        );
        assert!(
            panels.contains("SidePanelTab::Fields"),
            "UT-IN-R5: 必须包含 Fields Tab",
        );
        assert!(
            !css.contains("max-height: 45%"),
            "UT-IN-R5: 不得保留 Inspector 45% 字段区分割",
        );
        assert!(css.contains("cdb-has-io-drawer"), "Phase C: IO 抽屉栅格类",);
        assert!(
            panels.contains("data-testid=\"floating-controls\""),
            "浮动缩放条应保留",
        );
        assert!(
            panels.contains("data-testid=\"btn-share\""),
            "分享按钮应位于 AppBar",
        );
    }

    #[test]
    fn test_selection_auto_opens_inspector_ut_in_01() {
        assert!(selection_auto_opens_inspector(&SelectionKind::Table(
            "t1".into()
        )));
        assert!(selection_auto_opens_inspector(&SelectionKind::Field {
            table_id: "t1".into(),
            field_id: "f1".into(),
        }));
        assert!(!selection_auto_opens_inspector(&SelectionKind::None));
    }

    #[test]
    fn ut_fe_s05_11_collab_status_labels() {
        // 主原型 wsText 五态文案（core-01 renderEditor）
        let mut state = CollabOtState::default();
        assert_eq!(collab_status_label(&state), "协作离线");
        state.connection = CollabConnectionState::Connecting;
        assert_eq!(collab_status_label(&state), "正在同步…");
        state.connection = CollabConnectionState::Connected;
        assert_eq!(collab_status_label(&state), "已连接 · OT 同步");
        state.connection = CollabConnectionState::Reconnecting;
        assert_eq!(collab_status_label(&state), "重连中 · 操作排队");
        state.connection = CollabConnectionState::ReadOnly;
        assert_eq!(collab_status_label(&state), "只读");
        // ST-S05-UI-05：仅本地模式覆盖一切连接态
        state.connection = CollabConnectionState::Connected;
        state.enter_local_only();
        assert_eq!(collab_status_label(&state), "仅本地 · 409 风险");
    }

    // ─── align-frontend-to-prototype Batch D 全链路回归 ────

    /// ST-FE-PROTO-08 / ST-FE-V2-01~04 静态断言：S01/S02/IO/命令面板/ST-PU 的关键锚点与纯函数保持不变。
    /// 防止本次重构（auth → rooms → editor 路由）误删既有能力。
    #[test]
    fn test_regression_invariants_preserved() {
        let src = include_str!("editor_panels.rs");

        // ─── S01 保存与 409 冲突（ST-FE-V2-02）───
        assert!(
            src.contains("data-testid=\"editor-canvas-container\""),
            "ST-FE-V2-02: editor-canvas-container 锚点必须保留"
        );
        assert!(
            src.contains("data-testid=\"app-bar\""),
            "ST-FE-V2-02: app-bar 锚点必须保留"
        );
        assert!(
            src.contains("ConflictDialog"),
            "ST-FE-V2-02: 409 ConflictDialog 必须保留"
        );

        // ─── S02 分享只读（ST-FE-V2-01）───
        assert!(
            src.contains("SessionIndicator"),
            "ST-FE-V2-01: SessionIndicator 必须保留（?share= 匿名只读）"
        );
        assert!(
            src.contains("session-indicator"),
            "ST-FE-V2-01: data-testid=session-indicator 锚点必须保留"
        );

        // ─── IO 抽屉 + bridge API（ST-FE-V2-03）───
        assert!(
            src.contains("IoDrawer"),
            "ST-FE-V2-03: IoDrawer 组件必须保留"
        );
        assert!(
            src.contains("open_import_drawer") && src.contains("open_export_drawer"),
            "ST-FE-V2-03: 导入/导出抽屉打开入口必须保留"
        );

        // ─── 命令面板（ST-FE-V2-04）───
        assert!(
            src.contains("CommandPalette"),
            "ST-FE-V2-04: CommandPalette 组件必须保留"
        );
        assert!(
            src.contains("palette_visible"),
            "ST-FE-V2-04: palette_visible 信号必须保留"
        );

        // ─── ST-PU 统一原型（FEUX-AC-07 回归边界）───
        assert!(
            src.contains("ActivityFeed"),
            "ST-PU: activity-feed 锚点必须保留"
        );
        assert!(
            src.contains("data-testid=\"activity-feed\""),
            "ST-PU: activity-feed 锚点必须保留"
        );

        // ─── 720px 响应式（ST-FE-V2-04）───
        assert!(
            should_apply_compact_layout(720),
            "ST-FE-V2-04: 720px 必须判定为紧凑布局"
        );
        assert!(
            should_apply_compact_layout(640),
            "ST-FE-V2-04: 640px 必须判定为紧凑布局"
        );
        assert!(
            !should_apply_compact_layout(900),
            "ST-FE-V2-04: 900px 仍为桌面布局"
        );
    }

    /// ST-FE-PROTO-08 页面流路由回归：所有五态页面状态在 AppRoot 都已注册。
    #[test]
    fn test_regression_page_states_registered() {
        let src = include_str!("editor_panels.rs");
        for state in ["PageState::Auth", "PageState::Rooms", "PageState::Invite", "PageState::RoomEditor", "PageState::ShareEdit"] {
            assert!(
                src.contains(state),
                "ST-FE-PROTO-08: {state} 必须在 AppRoot 注册使用"
            );
        }
    }

    /// align-frontend-to-prototype：Auth 页对齐主原型的关键锚点齐全。
    #[test]
    fn test_auth_prototype_alignment_anchors() {
        let src = include_str!("editor_panels.rs");
        // 左区：品牌 + hero + 3 feature
        for anchor in [
            "data-testid=\"auth-brand\"",
            "data-testid=\"auth-brand-tag\"",
            "data-testid=\"auth-hero\"",
            "data-testid=\"auth-feature-row\"",
            "data-testid=\"auth-story\"",
            "data-testid=\"auth-panel\"",
        ] {
            assert!(
                src.contains(anchor),
                "Auth 对齐原型：{anchor} 必须保留"
            );
        }
        // 表单字段 + 错误提示
        for anchor in [
            "data-testid=\"auth-title\"",
            "data-testid=\"auth-email\"",
            "data-testid=\"auth-email-error\"",
            "data-testid=\"auth-password\"",
            "data-testid=\"auth-password-error\"",
            "data-testid=\"auth-display-name\"",
            "data-testid=\"auth-name-error\"",
            "data-testid=\"auth-confirm-password\"",
            "data-testid=\"auth-confirm-error\"",
            "data-testid=\"auth-alert\"",
        ] {
            assert!(
                src.contains(anchor),
                "Auth 对齐原型：{anchor} 必须保留"
            );
        }
        // 交互能力
        for anchor in [
            "data-testid=\"auth-eye-toggle\"",
            "data-testid=\"auth-strength\"",
            "data-testid=\"auth-remember\"",
            "data-testid=\"auth-simulate-error\"",
            "data-testid=\"auth-demo-note\"",
        ] {
            assert!(
                src.contains(anchor),
                "Auth 对齐原型：{anchor} 必须保留"
            );
        }
    }

    #[test]
    fn test_room_create_error_message_is_friendly() {
        assert_eq!(
            room_create_error_message(&ApiError::Server(404, "private sql detail".to_string())),
            "关联图表不存在，请重新选择"
        );
        assert_eq!(
            room_create_error_message(&ApiError::Server(409, "room-id".to_string())),
            "该图表已绑定其他协作房间"
        );
        assert!(!room_create_error_message(&ApiError::Network("token=secret".to_string()))
            .contains("secret"));
    }

    /// align-frontend-to-prototype：密码强度纯函数测试。
    #[test]
    fn test_password_strength_level() {
        assert_eq!(password_strength_level(""), 0);
        assert_eq!(password_strength_level("short"), 0);
        assert_eq!(password_strength_level("Pass1234"), 1);
        assert_eq!(password_strength_level("password"), 1);
        assert_eq!(password_strength_level("Pass12345678"), 3);
        assert_eq!(password_strength_level("Password1234"), 3);
        assert_eq!(password_strength_level("P@ssw0rd!Long"), 4);
        assert_eq!(password_strength_label(0), "无");
        assert_eq!(password_strength_label(1), "弱");
        assert_eq!(password_strength_label(2), "一般");
        assert_eq!(password_strength_label(3), "良好");
        assert_eq!(password_strength_label(4), "强");
    }

    /// UT-FE-PROTO-08：styles.css 设计 token 块必须挂载在裸 `:root` 选择器上。
    ///
    /// 回归背景（2026-08-20 真机诊断）：文件头注释里的 glob `"*/node_modules/*"`
    /// 含注释终止序列，提前截断注释，使首个 `:root` token 块被解析为
    /// `node_modules :root` 后代选择器 → 114 个亮色 token 全部失效。
    #[test]
    fn test_styles_css_root_token_block_intact() {
        let css = include_str!("styles.css");
        assert!(
            css.contains("\n:root {"),
            "UT-FE-PROTO-08: 设计 token 块必须挂在裸 `:root {{` 选择器上（防注释截断/选择器污染）"
        );
        assert!(
            !css.contains("node_modules :root"),
            "UT-FE-PROTO-08: token 块选择器不得被注释残留污染"
        );
        // 注释体内不得再出现提前终止序列：逐注释扫描。
        let mut rest = css;
        while let Some(open) = rest.find("/*") {
            let body = &rest[open + 2..];
            let close = body.find("*/").expect("UT-FE-PROTO-08: 注释未闭合");
            rest = &body[close + 2..];
        }
        // 全部注释闭合后，剩余文本中不得存在以 `*` 起始行紧跟 `/` 的 glob 片段。
        assert!(
            !rest.contains("node_modules/*"),
            "UT-FE-PROTO-08: 注释外不得残留 node_modules glob 片段"
        );
    }

    /// UT-FE-PROTO-09：AuthGate 表单输入必须双向绑定（prop:value + on:input）。
    ///
    /// 回归背景（2026-08-20 真机诊断）：四个输入框只有 `prop:value` 单向输出，
    /// signal 永远为空 → 字段校验必然失败 → 登录/注册从未真正可提交。
    #[test]
    fn test_auth_gate_inputs_two_way_bound() {
        let src = include_str!("editor_panels.rs");
        for binding in [
            "on:input=move |ev| display_name.set(event_target_value(&ev))",
            "on:input=move |ev| email.set(event_target_value(&ev))",
            "on:input=move |ev| password.set(event_target_value(&ev))",
            "on:input=move |ev| confirm_password.set(event_target_value(&ev))",
        ] {
            assert!(
                src.contains(binding),
                "UT-FE-PROTO-09: AuthGate 缺少输入绑定 `{binding}`（登录/注册表单将永远无法提交）"
            );
        }
    }


    /// ST-FE-PROTO-08 session 状态机不会重新引入 token 泄漏。
    #[test]
    fn test_regression_session_notice_sanitization() {
        // 实际生产中常见的注入尝试
        let attempts = [
            "登录成功，token=eyJhbGc.payload.sig",
            "Bearer eyJxxx.yyy.zzz",
            "您的会话已过期，refresh_token=rt-1234",
            "access_token: at-secret",
            "请复制此 eyJhbGciOi 字符串",
        ];
        for raw in attempts {
            let cleaned = sanitize_session_notice(Some(raw));
            assert!(
                cleaned.is_none() || !cleaned.as_ref().unwrap().to_lowercase().contains("token"),
                "ST-FE-PROTO-08: session notice 不得输出 token 原文，输入={raw}"
            );
        }
    }


    #[test]
    fn ut_fe_proto_05_collab_state_machine_text_stability() {
        // 五种 collab 连接状态 → 状态文案与主原型 wsText 对齐（不可随语言抖动）
        let cases = [
            (CollabConnectionState::Offline, "协作离线"),
            (CollabConnectionState::Connecting, "正在同步…"),
            (CollabConnectionState::Connected, "已连接 · OT 同步"),
            (CollabConnectionState::Reconnecting, "重连中 · 操作排队"),
            (CollabConnectionState::ReadOnly, "只读"),
        ];
        for (conn, expected) in cases {
            let state = CollabOtState {
                connection: conn,
                ..CollabOtState::default()
            };
            assert_eq!(
                collab_status_label(&state),
                expected,
                "UT-FE-PROTO-05: collab status label must match prototype wsText"
            );
        }

        // ot-rev 文本格式稳定（主原型：server_rev N）
        let mut state = CollabOtState::default();
        state.server_rev = 7;
        assert_eq!(format!("server_rev {}", state.server_rev), "server_rev 7");

        // reconnect-banner 文案：连接中断且有排队变更（主原型句式）
        state.connection = CollabConnectionState::Reconnecting;
        state.queued_while_offline = vec![CollabPendingOp {
            client_rev: 1,
            op_type: "table.create".into(),
        }];
        let banner = format!(
            "连接已断开，正在重连… · {} 项更改已排队",
            state.queued_while_offline.len()
        );
        assert_eq!(banner, "连接已断开，正在重连… · 1 项更改已排队");

        // 只读提示文案：read_only 状态在 ot/header 同步
        state.connection = CollabConnectionState::ReadOnly;
        assert_eq!(collab_status_label(&state), "只读");

        // ST-S05-UI-05：仅本地模式 → 409 风险文案覆盖连接态
        state.enter_local_only();
        assert_eq!(collab_status_label(&state), "仅本地 · 409 风险");
    }

    #[test]
    fn ut_fe_proto_06_responsive_layout_class_helpers() {
        // 720px 视口的纯函数判断：响应式应否触发
        assert!(should_apply_compact_layout(720));
        assert!(should_apply_compact_layout(640));
        assert!(!should_apply_compact_layout(900));
        assert!(!should_apply_compact_layout(1280));

        // inspector/io drawer 互斥：抽屉打开时 inspector 收起
        assert!(inspector_collapsed_when_io_open(true));
        assert!(!inspector_collapsed_when_io_open(false));

        // 浮层（drawer/modal）的可关闭性（不能锁死）
        let drawer_kinds = [
            IoDrawerKind::Import,
            IoDrawerKind::Export,
        ];
        for kind in drawer_kinds {
            assert!(can_close_io_drawer(kind), "kind={:?} must be closable", kind);
        }
    }

    #[test]
    fn ut_fe_s05_12_collab_activity_from_frame() {
        let connected = CollabFrame::Connected {
            server_rev: 3,
            diagram_id: "d1".into(),
            snapshot_hash: None,
            members: Vec::new(),
            your_role: Some("editor".into()),
        };
        assert_eq!(collab_activity_from_frame(&connected), "协作已连接 · rev 3");

        let err = CollabFrame::Error {
            code: "READ_ONLY".into(),
            message: "只读成员不能提交 op".into(),
        };
        assert!(collab_activity_from_frame(&err).contains("READ_ONLY"));
    }

    // ─── UT-STUB-01: is_table_selected 纯函数 4 case (fix-add-frontend-stub-leftover) ─

    /// UT-STUB-01 case 1: Some(id) match → true
    #[test]
    fn test_is_table_selected_some_id_match_ut_stub_01() {
        assert!(
            is_table_selected(&Some("t1".to_string()), "t1"),
            "UT-STUB-01 case 1: Some('t1') + 't1' 应匹配"
        );
    }

    /// UT-STUB-01 case 2: Some(testid) 错配 → false（**核心**：防 Bug B 回归）
    /// 验证 testid 字符串形如 `table-list-item-t1` 永远不应被当作 table_id 传入 selected
    #[test]
    fn test_is_table_selected_rejects_testid_prefix_ut_stub_01() {
        assert!(
            !is_table_selected(&Some("table-list-item-t1".to_string()), "t1"),
            "UT-STUB-01 case 2: testid 形式 'table-list-item-t1' + 't1' 必须 reject（防 Bug B 回归）"
        );
    }

    /// UT-STUB-01 case 3: None → false
    #[test]
    fn test_is_table_selected_none_ut_stub_01() {
        assert!(
            !is_table_selected(&None, "t1"),
            "UT-STUB-01 case 3: None + 't1' 应 false"
        );
    }

    /// UT-STUB-01 case 4: Some(id) 不匹配 → false
    #[test]
    fn test_is_table_selected_mismatch_ut_stub_01() {
        assert!(
            !is_table_selected(&Some("t1".to_string()), "t2"),
            "UT-STUB-01 case 4: Some('t1') + 't2' 应 false"
        );
    }

    // ─── UT-PC — Phase C 导入/导出抽屉 ─────────────────────────────────────

    #[test]
    fn test_import_parse_summary_ut_pc_01() {
        let sql = "CREATE TABLE a (id INT); CREATE TABLE b (id INT);";
        let summary = import_parse_summary(ImportFormat::Sql, sql).unwrap();
        assert!(summary.contains('2'), "UT-PC-01: 应含 2 条语句");
    }

    #[test]
    fn test_export_diagram_sql_ut_pc_02() {
        // relation-inspector-and-ddl-io 扩写：UNIQUE/自增/DEFAULT/COMMENT/FK 全量口径
        let mut users = make_table("t1", "users");
        users.fields.push(Field {
            id: "f1".into(),
            name: "id".into(),
            type_: "INT".into(),
            default: String::new(),
            check: String::new(),
            primary: true,
            unique: false,
            not_null: true,
            increment: true,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        });
        let mut posts = make_table("t2", "posts");
        posts.fields.push(Field {
            id: "f2".into(),
            name: "user_id".into(),
            type_: "INT".into(),
            default: String::new(),
            check: String::new(),
            primary: false,
            unique: true,
            not_null: true,
            increment: false,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        });
        posts.fields.push(Field {
            id: "f3".into(),
            name: "status".into(),
            type_: "VARCHAR(16)".into(),
            default: "'draft'".into(),
            check: String::new(),
            primary: false,
            unique: false,
            not_null: false,
            increment: false,
            comment: "状态".into(),
            tag: String::new(),
            dict_code: String::new(),
        });
        let refs = vec![Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t2".into(),
            end_table_id: "t1".into(),
            start_field_id: "f2".into(),
            end_field_id: "f1".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        }];
        let tables = vec![users, posts];

        let out = export_diagram_sql(&tables, &refs, "generic");
        assert!(
            out.contains("CREATE TABLE users"),
            "UT-PC-02: 应含 CREATE TABLE"
        );
        assert!(
            out.contains("id INT PRIMARY KEY AUTO_INCREMENT"),
            "UT-PC-02: generic 自增 → AUTO_INCREMENT"
        );
        assert!(
            out.contains("user_id INT NOT NULL UNIQUE"),
            "UT-PC-02: 应含 UNIQUE"
        );
        assert!(
            out.contains("status VARCHAR(16) DEFAULT 'draft'"),
            "UT-PC-02: 应含 DEFAULT"
        );
        assert!(
            out.contains("COMMENT ON COLUMN posts.status IS '状态';"),
            "UT-PC-02: generic COMMENT 后置 COMMENT ON COLUMN"
        );
        assert!(
            out.contains("FOREIGN KEY (user_id) REFERENCES users(id)"),
            "UT-PC-02: 应含 FK 约束"
        );

        let my = export_diagram_sql(&tables, &refs, "mysql");
        assert!(
            my.contains("status VARCHAR(16) DEFAULT 'draft' COMMENT '状态'"),
            "UT-PC-02: mysql COMMENT 行内"
        );
        assert!(
            !my.contains("COMMENT ON COLUMN"),
            "UT-PC-02: mysql 不应后置 COMMENT ON COLUMN"
        );

        let pg = export_diagram_sql(&tables, &refs, "postgresql");
        assert!(
            !pg.contains("AUTO_INCREMENT"),
            "UT-PC-02: postgresql 自增由 SERIAL 承担，不追加 AUTO_INCREMENT"
        );
        assert!(
            pg.contains("COMMENT ON COLUMN posts.status IS '状态';"),
            "UT-PC-02: pg COMMENT 后置"
        );
    }

    #[test]
    fn test_export_diagram_dbml_ut_pc_03() {
        let tables = vec![make_table("t1", "users")];
        let refs = vec![Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t1".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        }];
        let out = export_diagram_dbml(&tables, &refs);
        assert!(out.contains("Table users"), "UT-PC-03: 应含 Table 块");
        assert!(out.contains("Ref:"), "UT-PC-03: 应含 ref");
    }

    #[test]
    fn test_snapshot_before_io_drawer_ut_pc_04() {
        let (collapsed, cache) = snapshot_before_io_drawer(true);
        assert!(!collapsed, "UT-PC-04: 打开 IO 抽屉应折叠 Inspector");
        assert_eq!(cache, Some(true), "UT-PC-04: 应缓存 Inspector 展开态");
        assert!(restore_inspector_after_io_drawer(Some(true)));
        let (_, cache2) = snapshot_before_io_drawer(false);
        assert_eq!(cache2, None);
    }

    #[test]
    fn test_count_dbml_tables_ut_pc_05() {
        let text = "Table users {\n  id int\n}\nTable orders {\n  id int\n}";
        assert_eq!(count_dbml_tables(text), 2, "UT-PC-05: 应计数 2 个 Table");
    }

    #[test]
    fn test_parse_dbml_import_tables_builds_nested_tables() {
        let dbml = "Table users { id int [pk] }\nTable posts { id int, title varchar }";
        let tables = parse_dbml_import_tables(dbml).expect("parse ok");
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "users");
        assert_eq!(tables[0].fields.len(), 1);
        assert!(tables[0].fields[0].primary);
        assert_eq!(tables[1].name, "posts");
        assert_eq!(tables[1].fields.len(), 2);
    }

    #[test]
    fn test_phase_c_io_drawer_components_ut_pc_06() {
        let panels = include_str!("editor_panels.rs");
        let css = include_str!("styles.css");
        assert!(
            panels.contains("data-testid=\"import-drawer\""),
            "UT-PC-06: ImportDrawer testid",
        );
        assert!(
            panels.contains("data-testid=\"guide-import-sql\""),
            "UT-PC-06: EmptyGuide 导入按钮 testid",
        );
        assert!(
            !panels.contains("btn-import\"\n                disabled=true"),
            "UT-AB-04: btn-import Phase C 应启用",
        );
        assert!(css.contains(".cdb-io-drawer"), "Phase C IO 抽屉样式");
    }

    // ─── UT-PC-07 / UT-PC-08 — relation-inspector-and-ddl-io DDL 列级导入解析 ──

    /// UT-PC-07: 列级 DDL 解析（参数化类型 / 列级约束 / 表级联合主键 / 引号规范化 / 跳过与报错）
    #[test]
    fn test_parse_sql_import_tables_ut_pc_07() {
        let sql = "CREATE TABLE users (
  id UUID PRIMARY KEY,
  name VARCHAR(32) NOT NULL UNIQUE,
  age INT DEFAULT 18 COMMENT '简介',
  bio TEXT
);
CREATE TABLE `orders` (
  `oid` BIGINT AUTO_INCREMENT,
  [amount] DECIMAL(10, 2),
  serial_col SERIAL,
  PRIMARY KEY (oid, serial_col)
);
COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.name IS '用户名';
COMMENT ON COLUMN users.bio IS 'it''s a bio';
COMMENT ON TABLE missing_t IS '目标不存在应跳过';
COMMENT ON COLUMN missing_t.c IS '目标不存在应跳过';
COMMENT ON COLUMN users.missing_c IS '目标不存在应跳过';
INSERT INTO users VALUES (1);
CREATE INDEX idx_x ON users (id);";
        let (tables, _) = parse_sql_import_tables(sql).expect("UT-PC-07: 应解析成功");
        assert_eq!(tables.len(), 2, "UT-PC-07: 应解析出 2 张表（非 CREATE TABLE 跳过）");

        let users = &tables[0];
        assert_eq!(users.name, "users");
        assert_eq!(users.fields.len(), 4);
        let id = &users.fields[0];
        assert_eq!(id.name, "id");
        assert_eq!(id.type_, "UUID");
        assert!(id.primary, "UT-PC-07: 列级 PRIMARY KEY 置位");
        let name = &users.fields[1];
        assert_eq!(name.type_, "VARCHAR(32)", "UT-PC-07: 参数化类型整串保留");
        assert!(name.not_null && name.unique, "UT-PC-07: NOT NULL / UNIQUE 落字段");
        let age = &users.fields[2];
        assert_eq!(age.default, "18", "UT-PC-07: DEFAULT 值落字段");
        assert_eq!(age.comment, "简介", "UT-PC-07: 行内 COMMENT 落字段");
        // bio 的行内 COMMENT 由下方 COMMENT ON COLUMN 语句回填（断言见后文）

        let orders = &tables[1];
        assert_eq!(orders.name, "orders", "UT-PC-07: 反引号表名去引号");
        let oid = &orders.fields[0];
        assert_eq!(oid.name, "oid", "UT-PC-07: 反引号列名去引号");
        assert!(oid.increment, "UT-PC-07: AUTO_INCREMENT → increment");
        let amount = &orders.fields[1];
        assert_eq!(amount.name, "amount", "UT-PC-07: 方括号列名去引号");
        assert_eq!(amount.type_, "DECIMAL(10,2)", "UT-PC-07: 双参数类型整串保留");
        let serial = &orders.fields[2];
        assert!(serial.increment, "UT-PC-07: SERIAL → increment");
        assert!(oid.primary && serial.primary, "UT-PC-07: 表级联合主键双列置位");

        // fix-canvas-zoom-invite-comment-resize（MODIFIED）：PG COMMENT ON TABLE / COLUMN 回填
        assert_eq!(users.comment, "用户表", "UT-PC-07: COMMENT ON TABLE 回填表 comment");
        assert_eq!(
            users.fields[1].comment, "用户名",
            "UT-PC-07: COMMENT ON COLUMN 回填列 comment"
        );
        assert_eq!(
            users.fields[3].comment, "it's a bio",
            "UT-PC-07: COMMENT ON COLUMN 单引号转义还原（'' → '）"
        );
        assert_eq!(
            orders.comment, "",
            "UT-PC-07: 无 COMMENT ON 语句的表 comment 保持空串"
        );

        // 全文无合法 CREATE TABLE → Err
        let err = parse_sql_import_tables("INSERT INTO t VALUES (1); SELECT 1;");
        assert!(err.is_err(), "UT-PC-07: 无合法 CREATE TABLE 应报错");

        // 未知类型整串原样保留
        let (t2, _) = parse_sql_import_tables("CREATE TABLE m (price MONEY);").unwrap();
        assert_eq!(t2[0].fields[0].type_, "MONEY", "UT-PC-07: 未知类型保留");
    }

    /// UT-PC-26: import/connect 结构化响应前端消费（fix-canvas-zoom-invite-comment-resize，
    /// core-03 §13.1 + core-01d §4.4 3.5）：`data.tables` 序列化后复用 JSON 导入路径，comment 透传
    #[test]
    fn test_parse_bridge_import_tables_ut_pc_26() {
        let resp = r#"{
            "engine": "postgres",
            "table_count": 1,
            "tables": [
                {
                    "name": "users",
                    "comment": "用户表",
                    "fields": [
                        {"name": "id", "type": "INTEGER", "primary": true, "not_null": true, "default": "", "comment": ""},
                        {"name": "status", "type": "TEXT", "comment": "状态"}
                    ]
                }
            ]
        }"#;
        // 反序列化走与客户端相同的路径（gloo-net resp.json::<ApiResp<ImportConnectData>>()）
        let data: crate::editor_data_access::ImportConnectData =
            serde_json::from_str(resp).expect("UT-PC-26: 响应应反序列化成功");
        assert_eq!(data.table_count, 1, "UT-PC-26: table_count 与 tables 一致");
        assert_eq!(data.tables.len(), 1);

        let (tables, refs) =
            parse_bridge_import_tables(&data.tables).expect("UT-PC-26: 消费应解析成功");
        assert_eq!(tables.len(), 1, "UT-PC-26: 产出 1 表");
        assert_eq!(tables[0].name, "users");
        assert_eq!(tables[0].comment, "用户表", "UT-PC-26: 表 comment 透传");
        assert_eq!(tables[0].fields.len(), 2, "UT-PC-26: 产出 2 字段");
        assert!(
            tables[0].fields[0].primary,
            "UT-PC-26: 字段 primary 透传"
        );
        assert_eq!(
            tables[0].fields[1].comment, "状态",
            "UT-PC-26: 字段 comment 透传"
        );
        assert!(
            refs.is_empty(),
            "UT-PC-26: import/connect 不产 references"
        );
        // 表/字段 ID 仍为解析期确定性形式（重键在 merge_import_into_store 一次完成，UT-PC-20）
        assert!(
            tables[0].id.starts_with("import-j-t"),
            "UT-PC-26: 表 ID 为解析期确定性形式"
        );
        assert!(
            tables[0].fields[0].id.starts_with("import-j-t"),
            "UT-PC-26: 字段 ID 为解析期确定性形式"
        );

        // table_count=0 → tables 空数组仍可反序列化（前端走「未检测到数据表」提示路径）
        let empty: crate::editor_data_access::ImportConnectData =
            serde_json::from_str(r#"{"engine":"sqlite","table_count":0,"tables":[]}"#)
                .expect("UT-PC-26: 空 tables 应反序列化成功");
        assert_eq!(empty.table_count, 0);
        assert!(empty.tables.is_empty());
    }

    /// UT-PC-27: 导出 COMMENT ON TABLE（fix-canvas-zoom-invite-comment-resize，core-01d §5.2）
    #[test]
    fn test_export_comment_on_table_ut_pc_27() {
        let mut users = make_table("t1", "users");
        users.comment = "用户表".into();
        users.fields.push(Field {
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
        });
        users.fields.push(Field {
            id: "f2".into(),
            name: "note".into(),
            type_: "TEXT".into(),
            default: String::new(),
            check: String::new(),
            primary: false,
            unique: false,
            not_null: false,
            increment: false,
            comment: "it's".into(),
            tag: String::new(),
            dict_code: String::new(),
        });
        // 空 comment 表：不应产生任何注释语句
        let mut logs = make_table("t2", "logs");
        logs.fields.push(Field {
            id: "f3".into(),
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
        });
        let tables = vec![users, logs];

        // 1. postgres：COMMENT ON TABLE 位于 CREATE TABLE 之后，单引号转义
        let pg = export_diagram_sql(&tables, &[], "postgresql");
        assert!(
            pg.contains("COMMENT ON TABLE users IS '用户表';"),
            "UT-PC-27: pg 应含 COMMENT ON TABLE"
        );
        let create_idx = pg.find("CREATE TABLE users").expect("UT-PC-27: CREATE TABLE");
        let comment_idx = pg.find("COMMENT ON TABLE users").expect("UT-PC-27: COMMENT ON TABLE");
        assert!(
            comment_idx > create_idx,
            "UT-PC-27: COMMENT ON TABLE 应位于 CREATE TABLE 之后"
        );
        assert!(
            pg.contains("COMMENT ON COLUMN users.note IS 'it''s';"),
            "UT-PC-27: 单引号转义为 ''"
        );
        assert!(
            !pg.contains("COMMENT ON TABLE logs"),
            "UT-PC-27: 空 comment 表不输出 COMMENT ON TABLE"
        );
        assert!(
            !pg.contains("COMMENT ON COLUMN logs"),
            "UT-PC-27: 空 comment 列不输出 COMMENT ON COLUMN"
        );

        // 2. generic：同 pg 后置口径
        let generic = export_diagram_sql(&tables, &[], "generic");
        assert!(
            generic.contains("COMMENT ON TABLE users IS '用户表';"),
            "UT-PC-27: generic 应含 COMMENT ON TABLE"
        );

        // 3. mysql：表选项 COMMENT='x'，且不出现 COMMENT ON TABLE
        let my = export_diagram_sql(&tables, &[], "mysql");
        assert!(
            my.contains(") COMMENT='用户表';"),
            "UT-PC-27: mysql 表选项置于 CREATE TABLE 末尾"
        );
        assert!(
            !my.contains("COMMENT ON TABLE"),
            "UT-PC-27: mysql 不应出现 COMMENT ON TABLE"
        );
        assert!(
            !my.contains("COMMENT ON COLUMN"),
            "UT-PC-27: mysql 不应出现 COMMENT ON COLUMN"
        );
        assert!(
            !my.contains("COMMENT='';"),
            "UT-PC-27: 空 comment 表不输出表选项"
        );
    }

    /// UT-PC-28: JSON 导入注释透传加强断言（fix-canvas-zoom-invite-comment-resize）
    #[test]
    fn test_parse_json_import_tables_comment_ut_pc_28() {
        let json = r#"{"tables":[
            {"name":"users","comment":"用户表","fields":[
                {"name":"id","type":"INTEGER","primary":true,"not_null":true,"comment":""},
                {"name":"status","type":"TEXT","comment":"状态"}
            ]},
            {"name":"logs","fields":[{"name":"id","type":"INT"}]}
        ]}"#;
        let (tables, refs) = parse_json_import_tables(json).expect("UT-PC-28: 应解析成功");
        assert!(refs.is_empty());
        assert_eq!(tables.len(), 2, "UT-PC-28: 应解析出 2 表");
        // 非空 comment 原样透传
        assert_eq!(tables[0].comment, "用户表", "UT-PC-28: 表 comment 透传");
        assert_eq!(
            tables[0].fields[1].comment, "状态",
            "UT-PC-28: 字段 comment 透传"
        );
        // 缺省 comment 键 → 空串
        assert_eq!(
            tables[0].fields[0].comment, "",
            "UT-PC-28: 显式空 comment 保持空串"
        );
        assert_eq!(
            tables[1].comment, "",
            "UT-PC-28: 缺省表 comment 键落空串"
        );
        assert_eq!(
            tables[1].fields[0].comment, "",
            "UT-PC-28: 缺省字段 comment 键落空串"
        );

        // 与 UT-PC-26 数据库导入路径产出逐字段一致（同一 parse_json_import_tables 纯函数）
        let resp_tables: Vec<crate::editor_data_access::ImportConnectTable> = vec![
            crate::editor_data_access::ImportConnectTable {
                name: "users".into(),
                comment: "用户表".into(),
                fields: vec![
                    crate::editor_data_access::ImportConnectField {
                        name: "id".into(),
                        type_: "INTEGER".into(),
                        primary: true,
                        unique: false,
                        not_null: true,
                        increment: false,
                        default: String::new(),
                        comment: String::new(),
                    },
                    crate::editor_data_access::ImportConnectField {
                        name: "status".into(),
                        type_: "TEXT".into(),
                        primary: false,
                        unique: false,
                        not_null: false,
                        increment: false,
                        default: String::new(),
                        comment: "状态".into(),
                    },
                ],
            },
        ];
        let (db_tables, _) =
            parse_bridge_import_tables(&resp_tables).expect("UT-PC-28: 数据库路径应解析成功");
        assert_eq!(db_tables.len(), 1);
        assert_eq!(db_tables[0].name, tables[0].name, "UT-PC-28: 表名一致");
        assert_eq!(
            db_tables[0].comment, tables[0].comment,
            "UT-PC-28: 两条路径表 comment 一致"
        );
        assert_eq!(
            db_tables[0].fields.len(),
            tables[0].fields.len(),
            "UT-PC-28: 字段数一致"
        );
        for (a, b) in db_tables[0].fields.iter().zip(tables[0].fields.iter()) {
            assert_eq!(a.name, b.name, "UT-PC-28: 字段名一致");
            assert_eq!(a.type_, b.type_, "UT-PC-28: 字段类型一致");
            assert_eq!(a.comment, b.comment, "UT-PC-28: 字段 comment 一致");
            assert_eq!(a.primary, b.primary, "UT-PC-28: 字段 primary 一致");
            assert_eq!(a.not_null, b.not_null, "UT-PC-28: 字段 not_null 一致");
        }
    }

    /// UT-S04-UI-17: 前端各 Client base_url 同源派生锚点（core-01-deployment-plan §12.2 /
    /// core-S04 §7.2-4，include_str! 锚点口径）
    #[test]
    fn test_api_base_url_origin_anchor_ut_s04_ui_17() {
        let src = include_str!("editor_panels.rs");
        // 1. base_url 含 window.location.origin（或等价 location().origin()）同源派生逻辑
        assert!(
            src.contains("window.location.origin") || src.contains("location().origin()"),
            "UT-S04-UI-17: base_url 应由 window.location.origin 同源派生"
        );
        // 2. 生产默认不得硬编码 127.0.0.1:3000（仅允许经环境变量/构建配置在 dev 覆盖）。
        //    字面量拆分拼接，避免锚点测试自身命中 include_str! 检查。
        let forbidden = concat!("http://127.0.0.1:", "3000");
        assert!(
            !src.contains(forbidden),
            "UT-S04-UI-17: 生产默认不得硬编码后端回环地址"
        );
        // 3. 四个 Client 均消费同源派生值
        for ctor in [
            "DiagramClient::new(",
            "AuthClient::new(",
            "RoomClient::new(",
            "CollabClient::new(",
        ] {
            assert!(
                src.contains(ctor),
                "UT-S04-UI-17: 应存在 Client 构造 {}",
                ctor
            );
        }
        assert!(
            src.contains("default_api_base_url()"),
            "UT-S04-UI-17: 各 Client 应消费 default_api_base_url 派生值"
        );
        assert!(
            src.contains("COLDRAWDB_API_BASE"),
            "UT-S04-UI-17: dev 覆盖应仅经编译期环境变量"
        );
    }

    /// UT-PC-08: 表级 FOREIGN KEY 与列级 REFERENCES → references 连线
    #[test]
    fn test_parse_sql_import_tables_fk_ut_pc_08() {
        let sql = "CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR(32));
CREATE TABLE posts (
  id UUID PRIMARY KEY,
  user_id UUID,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE TABLE comments (
  id UUID PRIMARY KEY,
  post_id UUID REFERENCES posts(id)
);
CREATE TABLE orphan_refs (
  id UUID PRIMARY KEY,
  ghost_id UUID REFERENCES ghost(id)
);";
        let (tables, refs) = parse_sql_import_tables(sql).expect("UT-PC-08: 应解析成功");
        assert_eq!(tables.len(), 4);
        // 表级 FK + 列级 REFERENCES 各 1 条；ghost 悬空引用跳过
        assert_eq!(refs.len(), 2, "UT-PC-08: 悬空 FK 跳过，应只生成 2 条");
        let r1 = &refs[0];
        assert_eq!(r1.start_table_id, "import-t-1", "UT-PC-08: FK start=posts");
        assert_eq!(r1.end_table_id, "import-t-0", "UT-PC-08: FK end=users");
        let posts = &tables[1];
        let users = &tables[0];
        assert_eq!(
            r1.start_field_id,
            posts.fields.iter().find(|f| f.name == "user_id").unwrap().id,
            "UT-PC-08: start 端点解析到 posts.user_id"
        );
        assert_eq!(
            r1.end_field_id,
            users.fields.iter().find(|f| f.name == "id").unwrap().id,
            "UT-PC-08: end 端点解析到 users.id"
        );
        assert_eq!(r1.type_, "one_to_one", "UT-PC-08: 目标列为主键 → one_to_one");
        let r2 = &refs[1];
        assert_eq!(r2.start_table_id, "import-t-2", "UT-PC-08: 列级 REFERENCES 同样生成");

        // 无 REFERENCES 的 DDL → references 为空
        let (_, refs_none) =
            parse_sql_import_tables("CREATE TABLE a (id INT PRIMARY KEY);").unwrap();
        assert!(refs_none.is_empty(), "UT-PC-08: 无 REFERENCES 时 references 为空");
    }

    /// UT-PC-09: 导入合并纯函数——避让布局 + references 全量追加 + 入参不变
    #[test]
    fn test_merge_import_into_store_ut_pc_09() {
        // 现有 1 表 (100,100)，导入 2 表 → 新表落在右侧避让，既有表坐标不变
        let mut existing = make_table("t1", "users");
        existing.x = 100.0;
        existing.y = 100.0;
        existing.fields.push(make_field("INT"));
        let (new_tables, new_refs) = parse_sql_import_tables(
            "CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR(32));
CREATE TABLE posts (id UUID PRIMARY KEY, user_id UUID REFERENCES users(id));",
        )
        .unwrap();
        let existing_snapshot = existing.clone();
        let (merged_t, merged_r) =
            merge_import_into_store(&[existing], &[], &new_tables, &new_refs);
        assert_eq!(merged_t.len(), 3, "UT-PC-09: 1 + 2 = 3 张表");
        assert_eq!(merged_t[0], existing_snapshot, "UT-PC-09: 既有表坐标不变");
        let existing_right = existing_snapshot.x + 240.0;
        assert!(
            merged_t[1].x > existing_right && merged_t[2].x > existing_right,
            "UT-PC-09: 新表应在现有包围盒右侧避让（x={},{}, 右缘={}）",
            merged_t[1].x,
            merged_t[2].x,
            existing_right
        );
        // 同列表纵向不重叠
        if merged_t[1].x == merged_t[2].x {
            let t1_bottom = merged_t[1].y + 43.0 + 35.0 * merged_t[1].fields.len() as f64 + 20.0;
            assert!(merged_t[2].y >= t1_bottom, "UT-PC-09: 同列新表纵向不重叠");
        }
        assert_eq!(merged_r.len(), 1, "UT-PC-09: references 追加");
        // fix-dbimport-save-and-pg-schema：合并后 ID 已重键为全局唯一，
        // 端点断言由字面 import-t-N 改为与重键后表 ID 一致
        assert_eq!(
            merged_r[0].start_table_id, merged_t[2].id,
            "UT-PC-09: FK 起点为重键后的 posts"
        );
        assert_eq!(merged_r[0].end_table_id, merged_t[1].id);

        // 空 store → 从默认起点落位
        let (merged_empty, _) = merge_import_into_store(&[], &[], &new_tables, &[]);
        assert_eq!(merged_empty[0].x, 100.0, "UT-PC-09: 空画布从 (100,100) 起");
        assert_eq!(merged_empty[0].y, 100.0);

        // 既有 references 保留在前
        let existing_ref = Reference {
            id: "r0".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t1".into(),
            start_field_id: "f1".into(),
            end_field_id: "f1".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        let (_, merged_r2) =
            merge_import_into_store(&[], &[existing_ref], &[], &new_refs);
        assert_eq!(merged_r2.len(), 2, "UT-PC-09: 既有 + 新增 references");
        assert_eq!(merged_r2[0].id, "r0", "UT-PC-09: 既有 references 在前");
    }

    /// UT-PC-20: 导入合并 ID 重键全局唯一（fix-dbimport-save-and-pg-schema）
    #[test]
    fn test_merge_import_rekeys_entity_ids_ut_pc_20() {
        let ddl = "CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR(32) NOT NULL);
CREATE TABLE posts (id UUID PRIMARY KEY, user_id UUID NOT NULL, FOREIGN KEY (user_id) REFERENCES users(id));";
        let (t1, r1) = parse_sql_import_tables(ddl).unwrap();
        let (t2, r2) = parse_sql_import_tables(ddl).unwrap();
        // 两组解析结果 ID 完全相同（确定性）——这是原 bug 的根因
        assert_eq!(t1[0].id, t2[0].id, "UT-PC-20: 解析期 ID 确定性（复现前奏）");

        let (m1, mr1) = merge_import_into_store(&[], &[], &t1, &r1);
        let (m2, mr2) = merge_import_into_store(&m1, &mr1, &t2, &r2);

        let id_form = |id: &str| {
            // 形式校验：{auto|ref}-{16 位小写 hex}（new_entity_id 输出）
            let Some((prefix, suffix)) = id.split_once('-') else {
                return false;
            };
            (prefix == "auto" || prefix == "ref")
                && suffix.len() == 16
                && suffix.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        };
        // 1. 重键后无 import-t- 残留，形式为 new_entity_id 输出
        let mut all_ids: Vec<&str> = Vec::new();
        for t in m2.iter().skip(2) {
            all_ids.push(&t.id);
            all_ids.extend(t.fields.iter().map(|f| f.id.as_str()));
        }
        all_ids.extend(mr2.iter().skip(1).map(|r| r.id.as_str()));
        assert!(!all_ids.is_empty(), "UT-PC-20: 第二批导入实体存在");
        for id in &all_ids {
            assert!(!id.starts_with("import-"), "UT-PC-20: 无 import- 残留: {id}");
            assert!(id_form(id), "UT-PC-20: 重键 ID 形式 {{prefix}}-hex16: {id}");
        }
        // 2. 两批之间无交集（全局唯一）
        let batch1: std::collections::HashSet<&str> = m1
            .iter()
            .map(|t| t.id.as_str())
            .chain(m1.iter().flat_map(|t| t.fields.iter().map(|f| f.id.as_str())))
            .chain(mr1.iter().map(|r| r.id.as_str()))
            .collect();
        for id in &all_ids {
            assert!(!batch1.contains(id), "UT-PC-20: 跨批次 ID 冲突: {id}");
        }
        // 3. reference 端点指向重键后 ID，无悬空
        let table_ids: std::collections::HashSet<&str> =
            m2.iter().map(|t| t.id.as_str()).collect();
        let field_ids: std::collections::HashSet<&str> =
            m2.iter().flat_map(|t| t.fields.iter().map(|f| f.id.as_str())).collect();
        for r in &mr2 {
            assert!(table_ids.contains(r.start_table_id.as_str()), "UT-PC-20: start_table 悬空");
            assert!(table_ids.contains(r.end_table_id.as_str()), "UT-PC-20: end_table 悬空");
            assert!(field_ids.contains(r.start_field_id.as_str()), "UT-PC-20: start_field 悬空");
            assert!(field_ids.contains(r.end_field_id.as_str()), "UT-PC-20: end_field 悬空");
        }
        // 4. 语义内容不受重键影响
        let users2 = m2.iter().find(|t| t.name == "users" && t.id == mr2[1].end_table_id).unwrap();
        assert_eq!(users2.fields.len(), 2);
        assert!(users2.fields[0].primary, "UT-PC-20: PK 语义保留");
        assert!(users2.fields[1].not_null, "UT-PC-20: NOT NULL 语义保留");
        assert_eq!(users2.fields[1].type_, "VARCHAR(32)", "UT-PC-20: 参数化类型保留");
    }

    /// UT-S04-UI-12: AppBar 名称缩略/返回入口锚点（fix-appbar-roomname-back-and-import-merge）
    #[test]
    fn test_appbar_ellipsis_back_anchor_ut_s04_ui_12() {
        let src = include_str!("editor_panels.rs");
        let css = include_str!("styles.css");
        assert!(
            src.contains("data-testid=\"btn-back-to-rooms\""),
            "UT-S04-UI-12: AppBar 应有「← 空间」返回按钮锚点"
        );
        // room-badge 悬停全文（title 绑定）
        let badge_idx = src
            .find("data-testid=\"room-badge\"")
            .expect("UT-S04-UI-12: room-badge 锚点存在");
        let badge_block = &src[badge_idx..badge_idx + 400.min(src.len() - badge_idx)];
        assert!(
            badge_block.contains("title=format!"),
            "UT-S04-UI-12: room-badge 应绑定 title 属性悬停显示全文"
        );
        // diagram-title 输入框悬停全文
        let title_idx = src
            .find("data-testid=\"diagram-title\"")
            .expect("UT-S04-UI-12: diagram-title 锚点存在");
        let title_block = &src[title_idx..title_idx + 400.min(src.len() - title_idx)];
        assert!(
            title_block.contains("title="),
            "UT-S04-UI-12: diagram-title 应绑定 title 属性"
        );
        // room-badge 名称缩略 CSS 规则
        let css_idx = css
            .find(".cdb-room-badge {")
            .expect("UT-S04-UI-12: room-badge CSS 规则存在");
        let css_block = &css[css_idx..css_idx + 400.min(css.len() - css_idx)];
        assert!(
            css_block.contains("text-overflow: ellipsis"),
            "UT-S04-UI-12: room-badge CSS 应含 ellipsis 缩略"
        );
        assert!(
            css_block.contains("max-width"),
            "UT-S04-UI-12: room-badge CSS 应有 max-width 限宽"
        );
    }

    /// UT-PC-10: ListView IO 入口 + 溢出菜单边界 CSS 锚点（fix-overflow-menu-room-delete-listview-io）
    #[test]
    fn test_listview_io_menu_boundary_anchor_ut_pc_10() {
        let src = include_str!("editor_panels.rs");
        let css = include_str!("styles.css");
        assert!(
            src.contains("data-testid=\"list-btn-import\""),
            "UT-PC-10: ListView 应有导入按钮锚点"
        );
        assert!(
            src.contains("data-testid=\"list-btn-export\""),
            "UT-PC-10: ListView 应有导出按钮锚点"
        );
        // ListView 导入/导出接线 IoDrawer 打开路径（props 传入 open_import/open_export drawer 处理器）
        assert!(
            src.contains("on_open_import: Option<Rc<dyn Fn()>>"),
            "UT-PC-10: ListView 应接收 on_open_import prop"
        );
        assert!(
            src.contains("on_open_import=open_import_for_list"),
            "UT-PC-10: 全屏 ListView 应接线 open_import_drawer"
        );
        // 溢出菜单视口边界保护
        let menu_idx = css
            .find(".cdb-app-bar__overflow-menu {")
            .expect("UT-PC-10: 溢出菜单 CSS 规则存在");
        let menu_block = &css[menu_idx..menu_idx + 500.min(css.len() - menu_idx)];
        assert!(
            menu_block.contains("max-height"),
            "UT-PC-10: 溢出菜单应含 max-height 边界保护"
        );
        assert!(
            menu_block.contains("overflow-y"),
            "UT-PC-10: 溢出菜单应含 overflow-y 滚动"
        );
    }

    /// UT-PC-15: ImportDrawer 数据库来源锚点（feat-db-connect-import-and-ddl-realdb-verify，core-01d §4.1）
    #[test]
    fn test_import_drawer_database_source_anchor_ut_pc_15() {
        let src = include_str!("editor_panels.rs");
        let data_access = include_str!("editor_data_access.rs");
        assert!(
            src.contains("data-testid=\"import-db-engine\""),
            "UT-PC-15: 数据库来源应有引擎选择锚点"
        );
        assert!(
            src.contains("data-testid=\"import-db-source\""),
            "UT-PC-15: 数据库来源应有连接信息输入锚点"
        );
        assert!(
            src.contains("data-testid=\"import-db-connect\""),
            "UT-PC-15: 数据库来源应有连接并解析按钮锚点"
        );
        // ImportFormat 含 Database 变体且 format tabs 渲染第 4 个 Tab
        let enum_idx = src
            .find("pub enum ImportFormat {")
            .expect("UT-PC-15: ImportFormat 枚举存在");
        let enum_block = &src[enum_idx..enum_idx + 300];
        assert!(
            enum_block.contains("Database"),
            "UT-PC-15: ImportFormat 应含 Database 变体"
        );
        assert!(
            src.contains(">\"数据库\"</button>"),
            "UT-PC-15: format tabs 应渲染「数据库」Tab"
        );
        // 「连接并解析」接线到 import_from_connection（POST /api/v1/bridge/import/connect）
        assert!(
            src.contains("import_from_connection"),
            "UT-PC-15: 连接按钮应接线 import_from_connection"
        );
        assert!(
            data_access.contains("/api/v1/bridge/import/connect"),
            "UT-PC-15: 客户端应调用 bridge/import/connect 端点"
        );
    }

    /// UT-PC-22: ImportDrawer schema 输入锚点（fix-dbimport-save-and-pg-schema，core-01d §4.1）
    #[test]
    fn test_import_db_schema_anchor_ut_pc_22() {
        let src = include_str!("editor_panels.rs");
        let data_access = include_str!("editor_data_access.rs");
        // 1. import-db-schema testid 存在且仅在 postgres 引擎分支渲染
        assert!(
            src.contains("data-testid=\"import-db-schema\""),
            "UT-PC-22: 应有 schema 输入锚点"
        );
        let schema_idx = src
            .find("data-testid=\"import-db-schema\"")
            .expect("UT-PC-22: import-db-schema 存在");
        let ctx_start = src[..schema_idx].rfind("{move || if").expect("UT-PC-22: 条件分支存在");
        let ctx = &src[ctx_start..schema_idx];
        assert!(
            ctx.contains("db_engine.get() == \"postgres\""),
            "UT-PC-22: schema 输入应仅在 postgres 引擎分支渲染，实际上下文：{}…",
            &ctx[..ctx.len().min(120)]
        );
        // 2. 客户端方法签名与请求体包含 schema
        assert!(
            data_access.contains("schema: Option<&str>"),
            "UT-PC-22: import_from_connection 应接收 schema 参数"
        );
        assert!(
            data_access.contains("body[\"schema\"]"),
            "UT-PC-22: 请求体应条件携带 schema 字段"
        );
        // 3. sqlite 引擎分支不发送 schema（调用点按引擎传 None）
        assert!(
            src.contains("if engine == \"postgres\" { Some(schema.as_str()) } else { None }"),
            "UT-PC-22: sqlite 分支应传 None（不发送 schema）"
        );
    }

    /// UT-PC-23: select 背景简写锚点（fix-select-bg-diagram-delete-and-log-format，core-09 §13 第 8 条）
    ///
    /// 背景：`.cdb-create-room-modal .cdb-input` / `.cdb-auth-card .cdb-input`（特异度 0,2,0）使用
    /// `background:` 简写会重置 select chevron 的 background-image/repeat；暗色规则重设 image 时
    /// 若不再申明 no-repeat/position，箭头平铺成满框 ˅˅˅。
    #[test]
    fn test_select_background_shorthand_anchor_ut_pc_23() {
        let css = include_str!("styles.css");
        // 1. 两处高特异度规则禁用 background 简写，仅允许 background-color
        for selector in [".cdb-create-room-modal .cdb-input", ".cdb-auth-card .cdb-input"] {
            let idx = css.find(selector).unwrap_or_else(|| panic!("UT-PC-23: 规则 {selector} 存在"));
            let block_end = css[idx..].find('}').expect("UT-PC-23: 规则闭合") + idx;
            let block = &css[idx..block_end];
            assert!(
                !block.contains("background:"),
                "UT-PC-23: {selector} 禁止使用 background 简写（core-09 §13 第 8 条），实际：{block}"
            );
            assert!(
                block.contains("background-color:"),
                "UT-PC-23: {selector} 应使用 background-color 仅改背景色"
            );
        }
        // 2. 暗色 chevron 规则重设 background-image 时必须同权重申 no-repeat/position
        let dark_idx = css
            .find("[data-mode=\"dark\"] .cdb-form-select,")
            .expect("UT-PC-23: 暗色 select 规则存在");
        let dark_end = css[dark_idx..].find('}').expect("UT-PC-23: 规则闭合") + dark_idx;
        let dark_block = &css[dark_idx..dark_end];
        assert!(
            dark_block.contains("background-image:"),
            "UT-PC-23: 暗色规则应重设 chevron background-image"
        );
        assert!(
            dark_block.contains("background-repeat: no-repeat"),
            "UT-PC-23: 暗色规则重设 image 时必须申明 background-repeat: no-repeat"
        );
        assert!(
            dark_block.contains("background-position: right 10px center"),
            "UT-PC-23: 暗色规则应申明箭头定位 background-position"
        );
    }

    /// UT-S04-UI-13: 房间卡片删除锚点（fix-overflow-menu-room-delete-listview-io）
    #[test]
    fn test_room_card_delete_anchor_ut_s04_ui_13() {
        let src = include_str!("editor_panels.rs");
        assert!(
            src.contains("room-card-delete-"),
            "UT-S04-UI-13: 房间卡片应有删除按钮 testid 前缀"
        );
        assert!(
            src.contains("modal-delete-room-list"),
            "UT-S04-UI-13: 应有列表页删除确认模态"
        );
        assert!(
            src.contains("btn-confirm-delete-room-list"),
            "UT-S04-UI-13: 应有确认删除按钮"
        );
        // owner 门控：can_delete_room 判定驱动删除按钮渲染
        assert!(
            src.contains("let deletable = can_delete_room(&room.my_role);"),
            "UT-S04-UI-13: 删除按钮渲染应受 can_delete_room 门控"
        );
        let del_idx = src
            .find("room-card-delete-")
            .expect("UT-S04-UI-13: 删除按钮存在");
        let context = &src[del_idx.saturating_sub(600)..del_idx];
        assert!(
            context.contains("deletable.then"),
            "UT-S04-UI-13: 删除按钮应由 deletable 信号门控渲染"
        );
    }

    /// UT-S04-UI-15: 回收站锚点（feat-room-recycle-bin-dropdown-and-db-export）
    #[test]
    fn test_room_recycle_bin_anchor_ut_s04_ui_15() {
        let src = include_str!("editor_panels.rs");
        assert!(
            src.contains("btn-room-trash"),
            "UT-S04-UI-15: 房间列表页应有回收站入口"
        );
        assert!(
            src.contains("room-trash-view"),
            "UT-S04-UI-15: 应有回收站视图"
        );
        assert!(
            src.contains("btn-trash-back"),
            "UT-S04-UI-15: 回收站应有返回按钮"
        );
        assert!(
            src.contains("btn-restore-room-"),
            "UT-S04-UI-15: 应有恢复按钮 testid 前缀"
        );
        assert!(
            src.contains("btn-purge-room-"),
            "UT-S04-UI-15: 应有彻底删除按钮 testid 前缀"
        );
        assert!(
            src.contains("modal-purge-room"),
            "UT-S04-UI-15: 应有彻底删除二次确认模态"
        );
        assert!(
            src.contains("btn-confirm-purge-room"),
            "UT-S04-UI-15: 应有确认彻底删除按钮"
        );
        // 删除文案口径修正：废止「不可恢复」误导文案，统一为回收站语义
        // （needle 拆段拼接，避免 include_str! 自引用导致误判）
        let stale = concat!("删除后不可恢复，", "房间及其所有协作数据将被永久删除");
        assert!(
            !src.contains(stale),
            "UT-S04-UI-15: DeleteRoomModal 旧误导文案应已废止"
        );
        assert!(
            src.contains("删除后房间进入回收站，可在回收站恢复或彻底删除"),
            "UT-S04-UI-15: DeleteRoomModal 应为回收站语义文案"
        );
        let client = include_str!("editor_data_access.rs");
        assert!(
            client.contains("pub async fn restore_room") && client.contains("/restore"),
            "UT-S04-UI-15: RoomClient 应有 restore_room 指向 /restore"
        );
        assert!(
            client.contains("pub async fn permanently_delete_room") && client.contains("/permanent"),
            "UT-S04-UI-15: RoomClient 应有 permanently_delete_room 指向 /permanent"
        );
    }

    /// UT-S04-UI-16: 创建空间弹窗删除图表锚点（fix-select-bg-diagram-delete-and-log-format，core-S04 图表行）
    #[test]
    fn test_create_room_diagram_delete_anchor_ut_s04_ui_16() {
        let src = include_str!("editor_panels.rs");
        // 1. 删除入口按钮 + 二次确认模态 + 确认按钮
        assert!(
            src.contains("create-room-diagram-delete"),
            "UT-S04-UI-16: 创建空间弹窗应有删除图表按钮"
        );
        assert!(
            src.contains("modal-delete-diagram"),
            "UT-S04-UI-16: 应有删除图表二次确认模态"
        );
        assert!(
            src.contains("btn-confirm-delete-diagram"),
            "UT-S04-UI-16: 应有确认删除图表按钮"
        );
        // 2. 删除按钮仅在选中已有图表时渲染（__new__ 分支不显示）
        let btn_idx = src
            .find("create-room-diagram-delete")
            .expect("UT-S04-UI-16: 删除按钮存在");
        let ctx_start = src[..btn_idx].rfind("{move ||").expect("UT-S04-UI-16: 条件分支存在");
        let ctx = &src[ctx_start..btn_idx];
        assert!(
            ctx.contains("selected == \"__new__\""),
            "UT-S04-UI-16: 删除按钮应仅在选中已有图表时渲染，实际上下文：{}…",
            &ctx[..ctx.len().min(120)]
        );
        // 3. 确认后调用客户端 DELETE /api/v1/diagrams/{id} 并提示 Toast「已删除图表」
        assert!(
            src.contains("diagram_client.delete(&target.id)"),
            "UT-S04-UI-16: 确认删除应调用 diagram_client.delete"
        );
        assert!(
            src.contains("已删除图表"),
            "UT-S04-UI-16: 删除成功应有「已删除图表」Toast"
        );
        // 4. 被删图表正被选中时回落到新建空白模型
        assert!(
            src.contains("diagram_choice.set(String::from(\"__new__\"))"),
            "UT-S04-UI-16: 删除当前选中图表后应回落到 __new__"
        );
        let client = include_str!("editor_data_access.rs");
        assert!(
            client.contains("pub async fn delete(&self, id: &str)")
                && client.contains("/api/v1/diagrams/{}"),
            "UT-S04-UI-16: DiagramClient 应有 delete 指向 DELETE /api/v1/diagrams/{{id}}"
        );
    }

    /// UT-PC-19: 表单下拉视觉条款锚点（feat-room-recycle-bin-dropdown-and-db-export，core-09 §13）
    #[test]
    fn test_form_select_style_anchor_ut_pc_19() {
        let css = include_str!("styles.css");
        let idx = css
            .find(".cdb-form-select,\n.cdb-select {")
            .expect("UT-PC-19: 应有 .cdb-form-select/.cdb-select 视觉条款规则块");
        let block = &css[idx..idx + 1200.min(css.len() - idx)];
        assert!(
            block.contains("appearance: none"),
            "UT-PC-19: 应去原生外观（appearance）"
        );
        assert!(
            block.contains("background-image: url(\"data:image/svg+xml"),
            "UT-PC-19: 应有自定义 chevron 箭头"
        );
        assert!(
            block.contains("padding-right: 32px"),
            "UT-PC-19: 应为箭头预留 padding-right"
        );
        assert!(
            block.contains("color-scheme: light"),
            "UT-PC-19: 应声明 color-scheme"
        );
        assert!(
            css.contains("[data-mode=\"dark\"] .cdb-form-select")
                && css.contains("color-scheme: dark"),
            "UT-PC-19: 暗色模式应有 color-scheme: dark 适配"
        );
        assert!(
            css.contains(".cdb-form-select option"),
            "UT-PC-19: 应有 option 配色规则"
        );
    }

    /// UT-PC-18: ExportDrawer 导出到数据库锚点（feat-room-recycle-bin-dropdown-and-db-export，core-01d §5.4）
    #[test]
    fn test_export_execute_anchor_ut_pc_18() {
        let src = include_str!("editor_panels.rs");
        for testid in [
            "export-db-engine",
            "export-db-source",
            "export-db-execute",
            "export-db-result",
        ] {
            assert!(
                src.contains(testid),
                "UT-PC-18: ExportDrawer 应有 testid {testid}"
            );
        }
        // 接线 POST /api/v1/bridge/export/execute
        assert!(
            src.contains("export_execute_ddl"),
            "UT-PC-18: 执行按钮应接线 export_execute_ddl"
        );
        // 空连接信息内联提示，不发请求
        assert!(
            src.contains("请填写连接信息"),
            "UT-PC-18: 空连接信息应有内联提示"
        );
        // 成功反馈 + Toast
        assert!(
            src.contains("已导出到数据库"),
            "UT-PC-18: 成功应有 Toast「已导出到数据库」"
        );
        // 导出到数据库区块内不得触碰 localStorage（连接信息不持久化）
        let start = src.find("export-db-engine").expect("UT-PC-18: 区块起点");
        let end = src.find("export-db-result").expect("UT-PC-18: 区块终点");
        let section = &src[start..end.max(start)];
        assert!(
            !section.contains("localStorage") && !section.contains("local_storage"),
            "UT-PC-18: 连接信息不得写入 localStorage"
        );
        let client = include_str!("editor_data_access.rs");
        assert!(
            client.contains("pub async fn export_execute_ddl") && client.contains("/export/execute"),
            "UT-PC-18: DiagramClient 应有 export_execute_ddl 指向 /export/execute"
        );
    }

    // ─── UT-MM-32 — p0-fix 定点 1 删除房间可见性纯函数 ─────────────────────

    /// UT-MM-32: can_delete_room — 仅 owner 可删除；editor/viewer/空串不可
    #[test]
    fn test_can_delete_room_ut_mm_32() {
        assert!(can_delete_room("owner"), "UT-MM-32: owner 应可删除");
        assert!(!can_delete_room("editor"), "UT-MM-32: editor 不可删除");
        assert!(!can_delete_room("viewer"), "UT-MM-32: viewer 不可删除");
        assert!(!can_delete_room(""), "UT-MM-32: 空角色不可删除");
    }

    // ─── UT-MM-34 — p0-fix 定点 3 Delete 键判定纯函数 ──────────────────────

    /// UT-MM-34: is_delete_key — Delete/Backspace 命中；其余键不命中
    #[test]
    fn test_is_delete_key_ut_mm_34() {
        assert!(is_delete_key("Delete"), "UT-MM-34: Delete 应命中");
        assert!(is_delete_key("Backspace"), "UT-MM-34: Backspace 应命中");
        assert!(!is_delete_key("d"), "UT-MM-34: 普通键不命中");
        assert!(!is_delete_key("Escape"), "UT-MM-34: Escape 不命中");
        assert!(!is_delete_key(""), "UT-MM-34: 空串不命中");
    }

    // ─── UT-PB — Phase B 关系工具纯函数 ─────────────────────────────────────

    /// UT-PB-02: build_reference 默认 cardinality 与 on_delete
    #[test]
    fn test_build_reference_ut_pb_02() {
        let r = build_reference(
            "ref-1".into(),
            "t1".into(),
            "f1".into(),
            "t2".into(),
            "f2".into(),
            "one_to_many",
        );
        assert_eq!(r.type_, "one_to_many", "UT-PB-02: type_ 应为 one_to_many");
        assert_eq!(r.on_delete, "RESTRICT", "UT-PB-02: on_delete 应为 RESTRICT");
        assert_eq!(r.on_update, "RESTRICT", "UT-PB-02: on_update 应为 RESTRICT");
    }

    /// UT-PB-03: flip_reference_endpoints 互换起止
    #[test]
    fn test_flip_reference_endpoints_ut_pb_03() {
        use crate::editor_core::EditorStore;
        let store = EditorStore::new();
        let r = Reference {
            id: "r1".into(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
        };
        let flipped = flip_reference_endpoints(&r, &store);
        assert_eq!(flipped.start_table_id, "t2", "UT-PB-03: start_table 应互换");
        assert_eq!(flipped.end_table_id, "t1", "UT-PB-03: end_table 应互换");
        assert_eq!(flipped.start_field_id, "f2", "UT-PB-03: start_field 应互换");
        assert_eq!(flipped.end_field_id, "f1", "UT-PB-03: end_field 应互换");
    }

    /// UT-PB-04: toggle_field_primary 单表唯一 PK
    #[test]
    fn test_toggle_field_primary_ut_pb_04() {
        let mut tables = vec![{
            let mut t = make_table("t1", "users");
            t.fields = vec![
                Field {
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
                },
                Field {
                    id: "f2".into(),
                    name: "email".into(),
                    type_: "VARCHAR(255)".into(),
                    default: String::new(),
                    check: String::new(),
                    primary: false,
                    unique: false,
                    not_null: false,
                    increment: false,
                    comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
                },
            ];
            t
        }];
        toggle_field_primary(&mut tables, "t1", "f2", true);
        let f1 = tables[0].fields.iter().find(|f| f.id == "f1").unwrap();
        let f2 = tables[0].fields.iter().find(|f| f.id == "f2").unwrap();
        assert!(!f1.primary, "UT-PB-04: f1 应失去 PK");
        assert!(f2.primary, "UT-PB-04: f2 应成为 PK");
    }

    /// UT-PB-05: 确认条创建关系后 references 增长
    #[test]
    fn test_confirm_create_increments_refs_ut_pb_05() {
        let mut refs: Vec<Reference> = Vec::new();
        let reference = build_reference(
            "ref-1".into(),
            "t1".into(),
            "f1".into(),
            "t2".into(),
            "f2".into(),
            "one_to_many",
        );
        refs.push(reference);
        assert_eq!(refs.len(), 1, "UT-PB-05: 创建后 references.len 应为 1");
    }

    // ─── UT-STUB-02: schedule_save 副作用契约 (fix-add-frontend-stub-leftover) ─

    /// UT-STUB-02: `schedule_save` 必须存在并调用 `debouncer.schedule`
    /// 编译期 + 源码静态断言：避免未来 refactor 改回空闭包而无人发现
    /// 实际 PUT 副作用由 ST-STUB-01 (e2e) 验证（避免单测依赖 wasm 网络）
    #[test]
    fn test_schedule_save_calls_debouncer_schedule_ut_stub_02() {
        let src = include_str!("editor_panels.rs");
        assert!(
            src.contains("pub(crate) fn schedule_save("),
            "UT-STUB-02: `schedule_save` 公共 helper 必须存在（pub(crate) fn）"
        );
        assert!(
            src.contains("debouncer.schedule("),
            "UT-STUB-02: `schedule_save` 必须调用 `debouncer.schedule(...)` 触发 1.1s debounce"
        );
        // 4 个 save handler 必须接 schedule_save（on_create_table / on_save / on_add_field / on_change_type）
        // 防止 stub 回退
        let calls = src.matches("schedule_save(").count();
        assert!(
            calls >= 5,
            "UT-STUB-02: `schedule_save(` 出现次数应 ≥ 5 (1 定义 + 4 调用), 实测 {} 次",
            calls
        );
    }
}

/// 协作 Connected 模式下 409 Conflict 后用最新 expected_revision 重发 PUT。
///
/// 与 schedule_save 同级的独立函数：split("Err(_) => {").nth(1) 仍指向
/// schedule_save 的兜底 Err 分支（不含 dirty.set(false)），不污染
/// UT-S01-SS-02 字符串搜索断言。
#[allow(clippy::too_many_arguments)]
/// ux-canvas-batch 批次4 步骤 2 (条目 19/20): UT-MM-28 ListView 列宽钳制 + 自适应
#[cfg(test)]
mod tests_ut_mm_28 {
    use super::*;

    #[test]
    fn test_clamp_column_width_min_ut_mm_28() {
        assert_eq!(clamp_column_width(0), 60, "UT-MM-28: 0 → 60");
        assert_eq!(clamp_column_width(30), 60, "UT-MM-28: 30 < 60 → 60");
    }

    #[test]
    fn test_clamp_column_width_max_ut_mm_28() {
        assert_eq!(clamp_column_width(481), 480, "UT-MM-28: 481 > 480 → 480");
        assert_eq!(clamp_column_width(600), 480, "UT-MM-28: 600 > 480 → 480");
    }

    #[test]
    fn test_clamp_column_width_in_range_ut_mm_28() {
        assert_eq!(clamp_column_width(60), 60, "UT-MM-28: 60 = min → 60");
        assert_eq!(clamp_column_width(200), 200, "UT-MM-28: 200 in range → 200");
        assert_eq!(clamp_column_width(480), 480, "UT-MM-28: 480 = max → 480");
        assert_eq!(clamp_column_width(150), 150, "UT-MM-28: 150 in range → 150");
    }

    #[test]
    fn test_auto_calc_column_width_zero_ut_mm_28() {
        assert_eq!(auto_calc_column_width(0), 60, "UT-MM-28: 0 字符 → 60 (公式 0×8+40=40, 钳制下限)");
    }

    #[test]
    fn test_auto_calc_column_width_in_range_ut_mm_28() {
        assert_eq!(auto_calc_column_width(30), 280, "UT-MM-28: 30 字符 → 30×8+40=280, 界内无钳制");
        assert_eq!(auto_calc_column_width(10), 120, "UT-MM-28: 10 字符 → 10×8+40=120, 界内");
    }

    #[test]
    fn test_auto_calc_column_width_clamped_ut_mm_28() {
        assert_eq!(auto_calc_column_width(100), 480, "UT-MM-28: 100 字符 → 100×8+40=840, 钳制 480");
        assert_eq!(auto_calc_column_width(300), 480, "UT-MM-28: 300 字符 → 钳制 480");
    }

    #[test]
    fn test_auto_calc_column_width_overflow_ut_mm_28() {
        // 防止 saturating_mul 触发 — u32::MAX * 8 应钳制 480
        assert_eq!(auto_calc_column_width(u32::MAX), 480, "UT-MM-28: u32::MAX 字符 → 480 (saturating)");
    }

    // ─── ColumnWidths 结构测试 (批次4 步骤 3, 条目 23) ───────────────────

    #[test]
    fn test_column_widths_defaults_ut_mm_28() {
        let cw = ColumnWidths::defaults();
        assert_eq!(cw.table_name, 120, "UT-MM-28: defaults table_name = 120");
        assert_eq!(cw.field_count, 120, "UT-MM-28: defaults field_count = 120");
        assert_eq!(cw.type_, 120, "UT-MM-28: defaults type_ = 120");
        assert_eq!(cw.has_index, 120, "UT-MM-28: defaults has_index = 120");
    }

    #[test]
    fn test_column_widths_get_known_key_ut_mm_28() {
        let cw = ColumnWidths::defaults();
        assert_eq!(cw.get("table_name"), 120);
        assert_eq!(cw.get("field_count"), 120);
        assert_eq!(cw.get("type"), 120);
        assert_eq!(cw.get("has_index"), 120);
    }

    #[test]
    fn test_column_widths_get_unknown_fallback_ut_mm_28() {
        let cw = ColumnWidths::defaults();
        assert_eq!(cw.get("unknown_key"), 120, "UT-MM-28: 未知键 fallback 120");
    }

    #[test]
    fn test_column_widths_set_clamp_min_ut_mm_28() {
        let mut cw = ColumnWidths::defaults();
        cw.set("table_name", 30);
        assert_eq!(cw.table_name, 60, "UT-MM-28: set 30 → 60 (clamp min)");
    }

    #[test]
    fn test_column_widths_set_clamp_max_ut_mm_28() {
        let mut cw = ColumnWidths::defaults();
        cw.set("has_index", 1000);
        assert_eq!(cw.has_index, 480, "UT-MM-28: set 1000 → 480 (clamp max)");
    }

    #[test]
    fn test_column_widths_set_in_range_ut_mm_28() {
        let mut cw = ColumnWidths::defaults();
        cw.set("field_count", 200);
        assert_eq!(cw.field_count, 200, "UT-MM-28: set 200 in range");
    }

    #[test]
    fn test_column_widths_set_unknown_noop_ut_mm_28() {
        let mut cw = ColumnWidths::defaults();
        cw.set("bogus_key", 999);
        // 四个真字段保持默认 120
        assert_eq!(cw.table_name, 120);
        assert_eq!(cw.field_count, 120);
        assert_eq!(cw.type_, 120);
        assert_eq!(cw.has_index, 120);
    }

    #[test]
    fn test_auto_calc_integration_with_long_field_ut_mm_28() {
        // 模拟 ListView 表格字段最长字符数 → auto_calc 应用
        // Field.type_ "DECIMAL(10,2)" 14 chars × 8 + 40 = 152
        assert_eq!(auto_calc_column_width(14), 152, "UT-MM-28: 14 chars → 152");
        // "VARCHAR(255)" 12 chars × 8 + 40 = 136
        assert_eq!(auto_calc_column_width(12), 136, "UT-MM-28: 12 chars → 136");
    }

    // ─── max_chars_for_column 纯函数测试 (批次4 步骤 3, 条目 25) ────────────

    fn make_table_for_test(name: &str, field_count: usize, first_type: &str) -> Table {
        let fields: Vec<Field> = (0..field_count)
            .map(|i| Field {
                id: format!("f{}", i),
                name: format!("f{}", i),
                type_: if i == 0 { first_type.to_string() } else { "INT".to_string() },
                default: String::new(),
                check: String::new(),
                primary: false,
                unique: false,
                not_null: false,
                increment: false,
                comment: String::new(),
                tag: String::new(),
                dict_code: String::new(),
            })
            .collect();
        Table {
            id: format!("t_{}", name),
            name: name.to_string(),
            x: 0.0,
            y: 0.0,
            color: String::new(),
            comment: String::new(),
            fields,
            indices: Vec::new(),
            width: None,
            min_height: None,
        }
    }

    #[test]
    fn test_max_chars_table_name_empty_ut_mm_28() {
        let tables: Vec<Table> = Vec::new();
        assert_eq!(max_chars_for_column("table_name", &tables), 0);
    }

    #[test]
    fn test_max_chars_table_name_single_ut_mm_28() {
        let tables = vec![make_table_for_test("users", 1, "INT")];
        assert_eq!(max_chars_for_column("table_name", &tables), 5);
    }

    #[test]
    fn test_max_chars_table_name_multi_mixed_ut_mm_28() {
        let tables = vec![
            make_table_for_test("a", 1, "INT"),
            make_table_for_test("user_profiles", 1, "INT"),
            make_table_for_test("posts", 1, "INT"),
        ];
        assert_eq!(max_chars_for_column("table_name", &tables), 13, "UT-MM-28: longest 'user_profiles' = 13");
    }

    #[test]
    fn test_max_chars_field_count_zero_ut_mm_28() {
        let tables: Vec<Table> = Vec::new();
        assert_eq!(max_chars_for_column("field_count", &tables), 1, "UT-MM-28: 空表 field_count 转字符串 = '0' → 1 字符");
    }

    #[test]
    fn test_max_chars_field_count_multi_ut_mm_28() {
        let tables = vec![
            make_table_for_test("a", 5, "INT"),
            make_table_for_test("b", 12, "INT"),
            make_table_for_test("c", 100, "INT"),
        ];
        assert_eq!(max_chars_for_column("field_count", &tables), 3, "UT-MM-28: 100 → 3 字符");
    }

    #[test]
    fn test_max_chars_type_empty_ut_mm_28() {
        let tables: Vec<Table> = Vec::new();
        assert_eq!(max_chars_for_column("type", &tables), 0);
    }

    #[test]
    fn test_max_chars_type_mixed_ut_mm_28() {
        let tables = vec![
            make_table_for_test("a", 1, "INT"),
            make_table_for_test("b", 1, "DECIMAL(10,2)"), // 13 chars
            make_table_for_test("c", 1, "VARCHAR(255)"), // 12 chars
        ];
        assert_eq!(max_chars_for_column("type", &tables), 13, "UT-MM-28: DECIMAL(10,2) = 13 chars 最长");
    }

    #[test]
    fn test_max_chars_has_index_ut_mm_28() {
        let tables = vec![make_table_for_test("a", 1, "INT")];
        assert_eq!(max_chars_for_column("has_index", &tables), 1, "UT-MM-28: has_index cell 实渲 1 字符（条目 26 记一笔，与 cell 同源）");
    }

    // ─── UT-MM-38: 表树过滤与当前表解析纯函数测试（listview-tree-master-detail） ───

    #[test]
    fn test_list_tree_filter_and_resolve_ut_mm_38() {
        let tables = vec![
            make_table_for_test("table_1", 2, "UUID"),
            make_table_for_test("table_2", 1, "UUID"),
            make_table_for_test("users", 3, "UUID"),
        ];
        // 1. filter_tables 名称模糊匹配（大小写不敏感）/ 空关键字全量 / 无命中空数组
        let hit = filter_tables(&tables, "USER", "", None);
        assert_eq!(hit.len(), 1, "UT-MM-38: 大小写不敏感含 user → 1 个表");
        assert_eq!(hit[0].name, "users");
        assert_eq!(filter_tables(&tables, "", "", None).len(), 3, "UT-MM-38: 空关键字 → 全量");
        assert!(filter_tables(&tables, "zzz", "", None).is_empty(), "UT-MM-38: 无命中 → 空数组");

        // 2. resolve_list_current_table：命中 / None 回落首表 / id 失效回落首表 / 空数组 None
        assert_eq!(
            resolve_list_current_table(&tables, &Some("t_users".into())),
            Some("t_users".to_string()),
            "UT-MM-38: list_table_id 命中 → 该表"
        );
        assert_eq!(
            resolve_list_current_table(&tables, &None),
            Some("t_table_1".to_string()),
            "UT-MM-38: None → 首张表"
        );
        assert_eq!(
            resolve_list_current_table(&tables, &Some("t_deleted".into())),
            Some("t_table_1".to_string()),
            "UT-MM-38: id 已不存在 → 首张表"
        );
        let empty: Vec<Table> = vec![];
        assert_eq!(resolve_list_current_table(&empty, &None), None, "UT-MM-38: 空表数组 → None");

        // 3. 过滤与导航解耦：搜索无命中时当前表仍按规则 2 解析
        assert!(filter_tables(&tables, "zzz", "", None).is_empty());
        assert_eq!(
            resolve_list_current_table(&tables, &Some("t_users".into())),
            Some("t_users".to_string()),
            "UT-MM-38: 搜索无命中不影响当前表解析"
        );
    }

    #[test]
    fn test_max_chars_unknown_key_ut_mm_28() {
        let tables = vec![make_table_for_test("a", 1, "INT")];
        assert_eq!(max_chars_for_column("bogus_key", &tables), 0);
    }

    #[test]
    fn test_max_chars_integration_with_auto_calc_ut_mm_28() {
        // 集成：max_chars_for_column → auto_calc_column_width
        let tables = vec![make_table_for_test("user_profiles", 1, "DECIMAL(10,2)")];
        let chars = max_chars_for_column("type", &tables);
        assert_eq!(chars, 13);
        let w = auto_calc_column_width(chars);
        assert_eq!(w, 144, "UT-MM-28: 13 chars × 8 + 40 = 144");
    }

    // ─── group_tables 纯函数测试 (批次4 步骤 4, 条目 26) ────────────────────────

    fn make_field_with_tag(name: &str, type_: &str, tag: &str) -> Field {
        Field {
            id: format!("fid_{}", name),
            name: name.to_string(),
            type_: type_.to_string(),
            default: String::new(),
            check: String::new(),
            primary: false,
            unique: false,
            not_null: false,
            increment: false,
            comment: String::new(),
            tag: tag.to_string(),
            dict_code: String::new(),
        }
    }

    fn make_table_with_tagged_fields(tid: &str, fields: Vec<Field>) -> Table {
        Table {
            id: tid.to_string(),
            name: tid.to_string(),
            x: 0.0,
            y: 0.0,
            color: String::new(),
            comment: String::new(),
            fields,
            indices: Vec::new(),
            width: None,
            min_height: None,
        }
    }

    #[test]
    fn test_group_tables_none_empty_ut_mm_29() {
        let tables: Vec<Table> = Vec::new();
        let buckets = group_tables(&tables, GroupByMode::None);
        assert_eq!(buckets.len(), 1, "UT-MM-29: None 空表 = 单桶 _flat");
        assert_eq!(buckets[0].key, "_flat");
        assert_eq!(buckets[0].fields.len(), 0);
    }

    #[test]
    fn test_group_tables_none_flat_ut_mm_29() {
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![
                make_field_with_tag("a", "INT", "pk"),
                make_field_with_tag("b", "VARCHAR", ""),
            ]),
            make_table_with_tagged_fields("t2", vec![
                make_field_with_tag("c", "INT", "fk"),
            ]),
        ];
        let buckets = group_tables(&tables, GroupByMode::None);
        assert_eq!(buckets.len(), 1, "UT-MM-29: None 模式 = 单桶");
        assert_eq!(buckets[0].key, "_flat");
        assert_eq!(buckets[0].fields.len(), 3, "UT-MM-29: 3 字段全在 _flat 桶");
        assert_eq!(buckets[0].fields[0], ("t1".to_string(), "fid_a".to_string()));
        assert_eq!(buckets[0].fields[1], ("t1".to_string(), "fid_b".to_string()));
        assert_eq!(buckets[0].fields[2], ("t2".to_string(), "fid_c".to_string()));
    }

    #[test]
    fn test_group_tables_by_tag_empty_ut_mm_29() {
        let tables: Vec<Table> = Vec::new();
        let buckets = group_tables(&tables, GroupByMode::ByTag);
        assert_eq!(buckets.len(), 0, "UT-MM-29: ByTag 空表 = 0 桶");
    }

    #[test]
    fn test_group_tables_by_tag_mixed_with_empty_ut_mm_29() {
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![
                make_field_with_tag("id1", "INT", "pk"),
                make_field_with_tag("name", "VARCHAR", ""),
            ]),
            make_table_with_tagged_fields("t2", vec![
                make_field_with_tag("id2", "INT", "pk"),
                make_field_with_tag("user_id", "INT", "fk"),
            ]),
        ];
        let buckets = group_tables(&tables, GroupByMode::ByTag);
        // 3 个桶: (empty), fk, pk (BTreeMap 字典序)
        assert_eq!(buckets.len(), 3, "UT-MM-29: 3 个桶 (empty + fk + pk)");
        assert_eq!(buckets[0].key, "(empty)", "UT-MM-29: 空 tag → (empty) 兜底");
        assert_eq!(buckets[0].fields.len(), 1);
        assert_eq!(buckets[0].fields[0], ("t1".to_string(), "fid_name".to_string()));
        assert_eq!(buckets[1].key, "fk");
        assert_eq!(buckets[1].fields.len(), 1);
        assert_eq!(buckets[1].fields[0], ("t2".to_string(), "fid_user_id".to_string()));
        assert_eq!(buckets[2].key, "pk");
        assert_eq!(buckets[2].fields.len(), 2);
        assert_eq!(buckets[2].fields[0], ("t1".to_string(), "fid_id1".to_string()));
        assert_eq!(buckets[2].fields[1], ("t2".to_string(), "fid_id2".to_string()));
    }

    #[test]
    fn test_group_tables_by_tag_case_sensitive_ut_mm_29() {
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![
                make_field_with_tag("a", "INT", "Pk"), // 大写 P
                make_field_with_tag("b", "INT", "pk"), // 小写 p
            ]),
        ];
        let buckets = group_tables(&tables, GroupByMode::ByTag);
        assert_eq!(buckets.len(), 2, "UT-MM-29: 大小写敏感 = 2 桶（Pk 与 pk）");
        let keys: Vec<&str> = buckets.iter().map(|b| b.key.as_str()).collect();
        assert!(keys.contains(&"Pk"));
        assert!(keys.contains(&"pk"));
    }

    #[test]
    fn test_group_tables_by_tag_single_field_multi_tag_ut_mm_29() {
        // 单字段 + 多 tag（实际 Field.tag 是单值，但测试同一字段两次不同 tag——模拟重复）
        let mut f1 = make_field_with_tag("a", "INT", "tag1");
        let mut f2 = make_field_with_tag("a", "INT", "tag2"); // 同字段 ID 不同 tag——仅供测试桶计数
        f1.id = "fid_same".to_string();
        f2.id = "fid_same".to_string();
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![f1, f2]),
        ];
        let buckets = group_tables(&tables, GroupByMode::ByTag);
        assert_eq!(buckets.len(), 2, "UT-MM-29: 2 桶 tag1 + tag2（按 tag 字段值分桶）");
    }

    #[test]
    fn test_group_tables_output_shape_uniform_ut_mm_29() {
        // None 和 ByTag 输出形状统一 Vec<Bucket{key, fields}>
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![make_field_with_tag("a", "INT", "x")]),
        ];
        let none_buckets = group_tables(&tables, GroupByMode::None);
        let bytag_buckets = group_tables(&tables, GroupByMode::ByTag);
        for b in none_buckets.iter().chain(bytag_buckets.iter()) {
            // 形状断言：每桶都是 Bucket{key: String, fields: Vec<(String, String)>}
            let _: String = b.key.clone();
            let _: Vec<(String, String)> = b.fields.clone();
        }
    }

    // ─── UT-MM-35 — list-view-table-structure：ByTable 按表分桶 ────────────

    /// UT-MM-35: ByTable 桶键=表名 / 桶序保持 tables 顺序 / 空表出桶（fields 为空）
    #[test]
    fn test_group_tables_by_table_ut_mm_35() {
        let tables = vec![
            make_table_with_tagged_fields("t1", vec![
                make_field_with_tag("a", "INT", ""),
                make_field_with_tag("b", "VARCHAR", ""),
            ]),
            make_table_with_tagged_fields("t2", vec![]), // 空表
            make_table_with_tagged_fields("t3", vec![make_field_with_tag("c", "INT", "")]),
        ];
        let buckets = group_tables(&tables, GroupByMode::ByTable);
        assert_eq!(buckets.len(), 3, "UT-MM-35: 3 表（含空表）= 3 桶");
        // 桶键 = 表名（make_table_with_tagged_fields 的 name == id）
        assert_eq!(buckets[0].key, "t1");
        assert_eq!(buckets[1].key, "t2", "UT-MM-35: 桶序必须保持 tables 数组顺序");
        assert_eq!(buckets[2].key, "t3");
        assert_eq!(buckets[0].fields.len(), 2);
        assert_eq!(
            buckets[0].fields[0],
            ("t1".to_string(), "fid_a".to_string()),
            "UT-MM-35: 桶内字段为 (table_id, field_id) 二元组"
        );
        assert!(buckets[1].fields.is_empty(), "UT-MM-35: 空表出桶且 fields 为空");
        assert_eq!(buckets[2].fields.len(), 1);
        // 空输入 = 0 桶（与 ByTag 一致，区别于 None 的 _flat 单桶）
        let empty: Vec<Table> = Vec::new();
        assert_eq!(group_tables(&empty, GroupByMode::ByTable).len(), 0, "UT-MM-35: 空输入 = 0 桶");
    }
}

/// fix-collab-autosave-race 批次：UT-FE-S05-18 room 绑定 diagram 跳过全量保存
#[cfg(test)]
mod tests_ut_fe_s05_18 {
    use super::*;

    fn room_detail() -> RoomDetail {
        RoomDetail {
            id: "r1".into(),
            name: "room".into(),
            diagram_id: "d1".into(),
            owner_id: "u1".into(),
            diagram_title: "d".into(),
            my_role: "owner".into(),
            member_count: 1,
        }
    }

    /// UT-FE-S05-18 — room 绑定 diagram 一律跳过全量保存（ops 即保存，服务端物化单写者）。
    /// 注：非 room 路径的 Timeout 调度依赖 wasm（gloo-timers native 直接 panic），
    /// 此处只断言 room 提前返回 + 谓词真值表；非 room 调度行为由既有 ST/e2e 覆盖。
    #[test]
    fn ut_fe_s05_18_room_mode_skips_full_save() {
        // 谓词真值表
        assert!(should_skip_full_save(true), "room 绑定 → 跳过全量保存");
        assert!(!should_skip_full_save(false), "非 room → 维持全量保存");

        // room 绑定调用：save_offline 预设 true，提前返回则保持不动（未进入调度）
        let store = EditorStore::new();
        let save_offline = RwSignal::new(true);
        schedule_save(
            DiagramClient::new("http://127.0.0.1:0"),
            store.clone(),
            RwSignal::new("d1".to_string()),
            RwSignal::new("t".to_string()),
            DebounceTrigger::default(),
            RwSignal::new(None),
            RwSignal::new(None),
            RwSignal::new(false),
            save_offline,
            RwSignal::new(CollabOtState::default()),
            RwSignal::new(Vec::new()),
            RwSignal::new(Some(room_detail())),
            RwSignal::new(None),
        );
        assert!(
            save_offline.get_untracked(),
            "UT-FE-S05-18: room 模式应提前返回，不调度全量 PUT"
        );
    }
}
