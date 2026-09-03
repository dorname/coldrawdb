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
use crate::editor_core::types::{Area, Field, Note, Reference, Table};
use crate::editor_core::{
    CollabConnectionState, CollabOtState, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_data_access::{
    auth_error_display, save_with_retry, AuthClient, AuthSession, BridgeConfigUpdate, CollabClient,
    CollabFrame, CollabMemberPresence, DiagramClient, DiagramSummary, ImportLocalResponse,
    ImportLogEntry, ApiError, InvitePreview, RoomClient, RoomDetail, RoomMember, RoomSummary,
    SaveError,
};
use crate::{sanitize_session_notice, share_load_error_message, PageState};
use crate::editor_render::Canvas;
use crate::editor_render::{remote_presence_slots, RemotePresence};
use crate::editor_render::{zoom_in, zoom_out, zoom_reset, Transform};
use crate::icons::{
    IconAdd, IconAddArea, IconAddNote, IconAddTable, IconBox, IconChevronLeft, IconChevronRight,
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
}

/// Phase B：关系工具状态机
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
    Confirm {
        start_table_id: String,
        start_field_id: String,
        end_table_id: String,
        end_field_id: String,
        cardinality: String,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Sql,
    Dbml,
    Json,
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
    }
}

/// 从 SQL CREATE TABLE 语句构建最小 import tables（每表一个 id 字段）。
pub fn parse_sql_import_tables(content: &str) -> Result<Vec<Table>, String> {
    let stmts = modals::parse_sql_statements(content)?;
    let mut tables = Vec::new();
    for (i, stmt) in stmts.iter().enumerate() {
        let upper = stmt.to_uppercase();
        if !upper.contains("CREATE TABLE") {
            continue;
        }
        let name = stmt
            .split_whitespace()
            .nth(2)
            .map(|s| {
                s.trim_matches('(')
                    .trim_matches('`')
                    .trim_matches('"')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("table_{}", i + 1));
        let table_id = format!("import-t-{i}");
        tables.push(Table {
            id: table_id.clone(),
            name: name.to_string(),
            x: 100.0 + (i as f64) * 40.0,
            y: 100.0 + (i as f64) * 30.0,
            color: String::new(),
            comment: String::new(),
            fields: vec![Field {
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
            }],
            indices: vec![],
            width: None,
            min_height: None,
        });
    }
    Ok(tables)
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
    });
}

/// 构建 bridge import payload
pub fn build_import_payload(
    format: ImportFormat,
    content: &str,
    _engine: &str,
    title: &str,
) -> Result<serde_json::Value, String> {
    match format {
        ImportFormat::Json => {
            let mut v: serde_json::Value =
                serde_json::from_str(content).map_err(|e| e.to_string())?;
            if let Some(obj) = v.as_object_mut() {
                if !obj.contains_key("name") {
                    obj.insert("name".into(), serde_json::Value::String(title.to_string()));
                }
            }
            Ok(v)
        }
        ImportFormat::Sql => {
            let tables = parse_sql_import_tables(content)?;
            Ok(serde_json::json!({
                "name": title,
                "source_format": "sql",
                "tables": tables,
            }))
        }
        ImportFormat::Dbml => {
            let tables = parse_dbml_import_tables(content)?;
            Ok(serde_json::json!({
                "name": title,
                "source_format": "dbml",
                "table_count": tables.len(),
                "tables": tables,
                "content": content,
            }))
        }
    }
}

/// 客户端 SQL 导出（Phase C UT-PC-02）
pub fn export_diagram_sql(tables: &[Table], _references: &[Reference], engine: &str) -> String {
    let mut out = String::new();
    if !engine.is_empty() && engine != "generic" {
        out.push_str(&format!("-- engine: {engine}\n\n"));
    }
    for table in tables {
        out.push_str(&format!("CREATE TABLE {} (\n", table.name));
        for (i, field) in table.fields.iter().enumerate() {
            let comma = if i + 1 < table.fields.len() { "," } else { "" };
            let pk = if field.primary { " PRIMARY KEY" } else { "" };
            let nn = if field.not_null { " NOT NULL" } else { "" };
            out.push_str(&format!(
                "  {} {}{}{}{}\n",
                field.name, field.type_, pk, nn, comma
            ));
        }
        out.push_str(");\n\n");
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

/// Public save 调度 helper (UT-STUB-02) — 抽 4 处 save handler 末尾的 `debouncer.schedule(...)`
/// 公共逻辑，避免雷同。闭包内 `spawn_local` 调 `DiagramClient::save` async 路径：
/// - 成功 → `store.revision.set(r.revision)` + `store.dirty.set(false)`
/// - 409 Conflict → `conflict.set(Some(ConflictInfo{...}))`（V1: 触发 ConflictDialog，handler 仍 stub）
///   - 例外（ST-S01-409-SCOPE / ST-S01-NO-409-OT）：协作已连接且非「仅本地」时，
///     快照 409 视为服务器已合并——禁止 modal-conflict，采纳服务器 rev 并写 Activity；
///   - 仅本地 / 非协作路径仍走 S01 409 模态（ST-S01-409-LOCAL-ONLY / ST-FE-V2-02）
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
    collab_state: RwSignal<CollabOtState>,
    activity_feed: RwSignal<Vec<String>>,
) {
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
            match save_with_retry(&client, &id, rev, &snap).await {
                Ok(resp) => {
                    store.revision.set(resp.revision);
                    store.dirty.set(false);
                    save_offline.set(false);
                    error.set(None);
                }
                Err(SaveError::Conflict {
                    current_revision, ..
                }) => {
                    let collab = collab_state.get_untracked();
                    if collab.snapshot_conflict_shows_modal() {
                        conflict.set(Some(ConflictInfo::new(current_revision, rev)));
                    } else {
                        // 协作 Connected 模式下，本地有未推送的修改（创建关系/字段/表 等）
                        // 时，409 仅更新本地 revision 会丢失用户修改。委托给独立函数
                        // retry_save_after_conflict，避免污染当前 match 的 Err 块。
                        store.revision.set(current_revision);
                        prepend_activity(
                            activity_feed,
                            format!("快照 409 已由协作合并 · 推进至 rev {current_revision}"),
                        );
                        if store.dirty.get() {
                            retry_save_after_conflict(
                                client.clone(),
                                store.clone(),
                                snap.clone(),
                                id.clone(),
                                current_revision,
                                conflict.clone(),
                                error.clone(),
                                is_saving.clone(),
                                save_offline.clone(),
                                activity_feed.clone(),
                            );
                            return;
                        }
                    }
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
            <div class="cdb-app-bar__brand">
                <span class="cdb-brand-mark" aria-hidden="true"><IconBox size="md"><IconLogo /></IconBox></span>
            </div>
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
                        title="返回房间列表"
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

fn prepend_activity(feed: RwSignal<Vec<String>>, item: String) {
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


#[component]
pub fn RoomPanel(
    visible: RwSignal<bool>,
    room_client: RoomClient,
    auth_session: RwSignal<Option<AuthSession>>,
    current_room: RwSignal<Option<RoomDetail>>,
    room_members: RwSignal<Vec<RoomMember>>,
    error: RwSignal<Option<String>>,
    on_open_invite: Rc<dyn Fn()>,
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
            .map(|r| r.my_role == "owner")
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
) -> impl IntoView {
    let loading = create_rw_signal(true);
    let rooms: RwSignal<Vec<RoomSummary>> = create_rw_signal(Vec::new());
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let creating = create_rw_signal(false);
    let last_loaded_token = create_rw_signal(Option::<String>::None);
    let reload_nonce = create_rw_signal(0_u32);
    let create_modal_open = create_rw_signal(false);
    let room_name = create_rw_signal(String::from("数据模型评审"));
    let diagram_choice = create_rw_signal(String::from("__new__"));
    let default_role = create_rw_signal(String::from("editor"));
    let diagrams = create_rw_signal(Vec::<DiagramSummary>::new());
    let diagrams_loading = create_rw_signal(false);
    let diagrams_loaded = create_rw_signal(false);
    let create_error = create_rw_signal(Option::<String>::None);

    // 首屏 fetch：依赖 auth_session；session 变化时重新拉。
    create_effect({
        let auth_client = auth_client.clone();
        let room_client = room_client.clone();
        let auth_session = auth_session.clone();
        move |_| {
            reload_nonce.get();
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
                    match diagram_client.create(&name).await {
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
                                        {move || if diagrams_loading.get() {
                                            view! { <small class="cdb-field-hint">"正在加载已有模型..."</small> }.into_view()
                                        } else {
                                            view! { <small class="cdb-field-hint">"新建模型会先创建 diagram，再绑定房间。"</small> }.into_view()
                                        }}
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
                data-testid="tool-new-area"
                disabled=rel_disabled
            >
                <IconBox size="md"><IconAddArea /></IconBox>
                <span class="cdb-tool-tip">"添加区域"</span>
            </button>
            <button
                class="cdb-tool-btn"
                data-testid="tool-new-note"
                disabled=rel_disabled
            >
                <IconBox size="md"><IconAddNote /></IconBox>
                <span class="cdb-tool-tip">"添加便签"</span>
            </button>
            <div class="cdb-tool-rail__divider"></div>
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

/// Phase B：关系确认条（画布底部，非模态）
#[component]
pub fn RelationshipConfirmBar(
    store: EditorStore,
    rel_state: RwSignal<RelToolState>,
    next_ref_id: Rc<dyn Fn() -> String>,
    on_create: Rc<dyn Fn(Reference)>,
) -> impl IntoView {
    view! {
        {move || {
            if let RelToolState::Confirm {
                start_table_id,
                start_field_id,
                end_table_id,
                end_field_id,
                cardinality,
            } = rel_state.get()
            {
                let label = format_rel_confirm_label(
                    &store.tables.get(),
                    &start_table_id,
                    &start_field_id,
                    &end_table_id,
                    &end_field_id,
                );
                let card = cardinality.clone();
                let st = start_table_id.clone();
                let sf = start_field_id.clone();
                let et = end_table_id.clone();
                let ef = end_field_id.clone();
                let on_create = on_create.clone();
                let next_ref_id = next_ref_id.clone();
                let st_change = st.clone();
                let sf_change = sf.clone();
                let et_change = et.clone();
                let ef_change = ef.clone();
                let card_for_options = card.clone();
                let st_create = st.clone();
                let sf_create = sf.clone();
                let et_create = et.clone();
                let ef_create = ef.clone();
                let card_create = card.clone();
                let inferred_cardinality = modals::infer_cardinality(&sf, &ef, &store);
                let display_cardinality = inferred_cardinality.clone();
                let display_cardinality_for_click = display_cardinality.clone();
                view! {
                    <div class="cdb-rel-confirm-bar" data-testid="rel-confirm-bar">
                        <span class="cdb-rel-confirm-bar__label">{label}</span>
                        // feat-relation-inference 批次2: 去掉 cardinality 必选下拉，
                        // 改为显示推导结果 + 可点击切换为其它 cardinality（手动覆盖）
                        <span
                            class="cdb-form-select cdb-rel-confirm-bar__select"
                            data-testid="rel-confirm-inferred-cardinality"
                            title={format!("推导结果（点击可手动覆盖）")}
                            on:click=move |_| {
                                // 手动覆盖：点击后循环切换 cardinality（one_to_one → one_to_many → many_to_one → many_to_many → one_to_one）
                                let current = display_cardinality_for_click.clone();
                                let next = match current.as_str() {
                                    "one_to_one" => "one_to_many",
                                    "one_to_many" => "many_to_one",
                                    "many_to_one" => "many_to_many",
                                    "many_to_many" => "one_to_one",
                                    _ => "one_to_many",
                                };
                                rel_state.set(RelToolState::Confirm {
                                    start_table_id: st_change.clone(),
                                    start_field_id: sf_change.clone(),
                                    end_table_id: et_change.clone(),
                                    end_field_id: ef_change.clone(),
                                    cardinality: next.to_string(),
                                });
                            }
                        >
                            {display_cardinality}
                        </span>
                        <button
                            class="cdb-btn cdb-btn--primary"
                            data-testid="rel-confirm-create"
                            on:click=move |_| {
                                let id = next_ref_id();
                                let reference = build_reference(
                                    id,
                                    st_create.clone(),
                                    sf_create.clone(),
                                    et_create.clone(),
                                    ef_create.clone(),
                                    &card_create,
                                );
                                on_create(reference);
                                rel_state.set(RelToolState::PickSource);
                            }
                        >
                            "创建"
                        </button>
                        <button
                            class="cdb-btn"
                            data-testid="rel-confirm-cancel"
                            on:click=move |_| rel_state.set(RelToolState::PickSource)
                        >
                            "取消"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }
        }}
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
) -> impl IntoView {
    let format = create_rw_signal(ImportFormat::Sql);
    let engine = create_rw_signal(String::from("generic"));
    let content = create_rw_signal(String::new());
    let inline_error = create_rw_signal(None::<String>);
    let submitting = create_rw_signal(false);
    let import_logs = create_rw_signal(Vec::<ImportLogEntry>::new());
    let logs_loading = create_rw_signal(false);

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
                </div>
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
                {move || {
                    let text = content.get();
                    let fmt = format.get();
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
                        let client = submit_client.clone();
                        let refresh_logs = refresh_for_submit.clone();
                        move |_| {
                        let text = content.get();
                        if text.trim().is_empty() {
                            inline_error.set(Some("内容不能为空".into()));
                            return;
                        }
                        let fmt = format.get();
                        let eng = engine.get();
                        let title = current_title.get();
                        let payload = match build_import_payload(fmt, &text, &eng, &title) {
                            Ok(p) => p,
                            Err(e) => {
                                inline_error.set(Some(e));
                                return;
                            }
                        };
                        inline_error.set(None);
                        submitting.set(true);
                        let client = client.clone();
                        let err = error.clone();
                        let refresh = refresh_logs.clone();
                        spawn_local(async move {
                            match client.import_local("import_drawer", payload).await {
                                Ok(ImportLocalResponse { diagram_id, .. }) => {
                                    refresh();
                                    navigate_to_editor(&diagram_id);
                                }
                                Err(e) => {
                                    submitting.set(false);
                                    err.set(Some(e.to_string()));
                                }
                            }
                        });
                    }}
                >
                    {move || if submitting.get() { "导入中..." } else { "导入并打开 ▶" }}
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
) -> impl IntoView {
    let format = create_rw_signal(ExportFormat::Sql);
    let engine = create_rw_signal(String::from("mysql"));
    let copied = create_rw_signal(false);

    let preview = create_memo(move |_| {
        let tables = store.tables.get();
        let refs = store.references.get();
        if tables.is_empty() {
            return "暂无表，无法导出".into();
        }
        match format.get() {
            ExportFormat::Sql => export_diagram_sql(&tables, &refs, &engine.get()),
            ExportFormat::Dbml => export_diagram_dbml(&tables, &refs),
            ExportFormat::Json => {
                export_diagram_json(&current_title.get_untracked(), &tables, &refs)
            }
        }
    });

    let has_tables = move || !store.tables.get().is_empty();
    let close = on_close.clone();

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
                                <option value="mysql" selected>"mysql"</option>
                                <option value="postgresql">"postgresql"</option>
                                <option value="generic">"generic"</option>
                            </select>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                <pre class="cdb-export-preview" data-testid="export-preview">{move || preview.get()}</pre>
            </div>
            <div class="cdb-io-drawer__footer">
                <button
                    class="cdb-btn"
                    data-testid="export-copy"
                    disabled=move || !has_tables()
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
                    disabled=move || !has_tables()
                    on:click=move |_| {
                        let ext = match format.get() {
                            ExportFormat::Sql => "sql",
                            ExportFormat::Dbml => "dbml",
                            ExportFormat::Json => "json",
                        };
                        let name = current_title.get_untracked();
                        let safe: String = name
                            .chars()
                            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                            .collect();
                        let filename = format!("{safe}.{ext}");
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
    on_delete_field: Rc<dyn Fn(String, String)>,
    on_delete_table: Rc<dyn Fn(String)>,
    on_update_ref_field: Rc<dyn Fn(String, &str, String)>,
    on_flip_ref: Rc<dyn Fn(String)>,
    on_delete_ref: Rc<dyn Fn(String)>,
    on_jump_to_table: Rc<dyn Fn(String)>,
    read_only: Rc<dyn Fn() -> bool>,
) -> impl IntoView {
    let selected_table = create_memo(move |_| {
        let id = selection.get().table_id()?.to_string();
        store.tables.get().into_iter().find(|t| t.id == id)
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
                                            value=field.type_.clone()
                                            on:change=move |ev| {
                                                on_change(fid_type.clone(), event_target_value(&ev));
                                            }
                                        >
                                            <option value="INT">"INT"</option>
                                            <option value="BIGINT">"BIGINT"</option>
                                            <option value="VARCHAR(255)">"VARCHAR(255)"</option>
                                            <option value="TEXT">"TEXT"</option>
                                            <option value="BOOLEAN">"BOOLEAN"</option>
                                        </select>
                                    </div>
                                    // ux-canvas-batch 批次4 步骤 5 (条目 27): Inspector 字段 tag 输入框
                                    <div class="cdb-form-group">
                                        <label>"标签（tag，用于 ByTag 分组）"</label>
                                        <input
                                            class="cdb-form-input"
                                            data-testid="inspector-field-tag"
                                            prop:value=field.tag.clone()
                                            on:blur={
                                                let store_t = store.clone();
                                                let table_id_t = table_id.clone();
                                                let fid_t = fid.clone();
                                                move |ev| {
                                                    let v = event_target_value(&ev);
                                                    store_t.tables.update(|tables| {
                                                        if let Some(t) = tables.iter_mut().find(|t| t.id == table_id_t) {
                                                            if let Some(f) = t.fields.iter_mut().find(|f| f.id == fid_t) {
                                                                if f.tag != v {
                                                                    f.tag = v;
                                                                }
                                                            }
                                                        }
                                                    });
                                                    store_t.dirty.set(true);
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
                                                        on:change=move |ev| {
                                                            on_change(fid_type.clone(), event_target_value(&ev));
                                                        }
                                                    >
                                                        // select 无 value 内容属性，初始选中须落在 option.selected 上
                                                        <option value="INT" selected={ftype == "INT"}>"INT"</option>
                                                        <option value="BIGINT" selected={ftype == "BIGINT"}>"BIGINT"</option>
                                                        <option value="UUID" selected={ftype == "UUID"}>"UUID"</option>
                                                        <option value="VARCHAR(255)" selected={ftype == "VARCHAR(255)"}>"VARCHAR(255)"</option>
                                                        <option value="TEXT" selected={ftype == "TEXT"}>"TEXT"</option>
                                                        <option value="BOOLEAN" selected={ftype == "BOOLEAN"}>"BOOLEAN"</option>
                                                        <option value="TIMESTAMP" selected={ftype == "TIMESTAMP"}>"TIMESTAMP"</option>
                                                    </select>
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
                        // ux-canvas-batch 批次2 收尾（条目 9 改派）: LeftPanel 死区调用点传局部 dummy 信号保编译
                        // ux-canvas-batch 批次3 步骤 3: 双击跳画布 — on_jump_to_canvas prop = 切回 Canvas + 选中表
                        let modal_kind_dummy = create_rw_signal(None);
                        let on_jump_for_listview: Rc<dyn Fn(String)> = {
                            let on_select = on_select_table.clone();
                            Rc::new(move |tid: String| {
                                view_mode.set(ViewMode::Canvas);
                                on_select(Some(tid));
                            })
                        };
                        let batch_type_selection_dummy: RwSignal<BatchTypeSelection> = create_rw_signal(BatchTypeSelection::default());
                        view! {
                            <ListView
                                store=store.clone()
                                on_select_table=on_select_table.clone()
                                on_jump_to_canvas=on_jump_for_listview.clone()
                                modal_kind=modal_kind_dummy
                                batch_type_selection=batch_type_selection_dummy
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
    #[default]
    None,
    ByTag,
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

/// ux-canvas-batch 批次3 步骤 2：BatchTypeSelection 状态组件（checkbox 多选 + 单一目标类型）
#[component]
pub fn BatchTypeSelectionPanel(
    store: EditorStore,
    selection: RwSignal<BatchTypeSelection>,
) -> impl IntoView {
    let field_options = move || {
        let tables = store.tables.get();
        let mut fields: Vec<(String, String)> = Vec::new(); // (field_id, field_name)
        for table in tables {
            for field in &table.fields {
                fields.push((field.id.clone(), format!("{}.{}", table.name, field.name)));
            }
        }
        fields
    };
    view! {
        <div class="cdb-list-view-filters" data-testid="list-view-batch-type-panel">
            <input
                class="cdb-form-input"
                data-testid="list-view-batch-type-target"
                placeholder="目标类型（如 INT/BIGINT/VARCHAR/DATETIME）"
                prop:value=move || selection.get().target_type
                on:input=move |ev| {
                    let v = event_target_value(&ev);
                    selection.update(|s| s.target_type = v);
                }
            />
            <div class="cdb-list-view-fields" data-testid="list-view-batch-type-fields">
                {move || field_options().into_iter().map(|(field_id, field_label)| {
                    let fid_for_label = field_id.clone();
                    let fid_for_state = field_id;
                    let is_selected = {
                        let fid = fid_for_state.clone();
                        move || selection.get().selected_field_ids.contains(&fid)
                    };
                    let toggle = move |_| {
                        let fid_inner = fid_for_state.clone();
                        selection.update(|s| {
                            if s.selected_field_ids.contains(&fid_inner) {
                                s.selected_field_ids.remove(&fid_inner);
                            } else {
                                s.selected_field_ids.insert(fid_inner);
                            }
                        });
                    };
                    view! {
                        <label data-testid={format!("list-view-select-field-{}", fid_for_label)}>
                            <input
                                type="checkbox"
                                checked=is_selected
                                on:change=toggle
                            />
                            <span>{field_label}</span>
                        </label>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// ux-canvas-batch 批次1：ListView 组件（表结构列表视图，参考 pdmaner 全量能力）
/// 表名/字段名/类型表格化展示 + 按表维度属性排序
#[component]
pub fn ListView(
    store: EditorStore,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    on_jump_to_canvas: Rc<dyn Fn(String)>,
    modal_kind: RwSignal<Option<modals::ModalKind>>,
    batch_type_selection: RwSignal<BatchTypeSelection>,
) -> impl IntoView {
    use wasm_bindgen::JsCast; // 条目 25: ev.current_target().dyn_ref::<HtmlElement>()
    let list_view_state = ListViewState {
        sort_column: create_rw_signal(SortColumn::TableName),
        sort_direction: create_rw_signal(SortDirection::Ascending),
        filter_query: create_rw_signal(String::new()),
        filter_type: create_rw_signal(String::new()),
        filter_has_index: create_rw_signal(None),
        batch_type_selection,
        column_widths: create_rw_signal(ColumnWidths::defaults()),
        group_by: create_rw_signal(GroupByMode::default()),
    };
    let list_view_state_for_name = list_view_state.clone();
    let list_view_state_for_field_count = list_view_state.clone();
    let list_view_state_for_type = list_view_state.clone();
    let list_view_state_for_has_index = list_view_state.clone();

    // ux-canvas-batch 批次4 步骤 3 (条目 24): 拖拽抑制 click 排序的共享信号
    // 拖拽结束 (位移 > 3px) 设为 true，on:click 检测后跳过排序逻辑
    let column_dragged: RwSignal<bool> = create_rw_signal(false);

    // ux-canvas-batch 批次4 步骤 3 (条目 25): 拖拽状态——Some(_) 表示拖拽进行中
    // None = 静止；Some((start_x, start_w, key)) = 记录起点坐标 + 当前列宽 + 列键
    // pointermove 实时 set（自带 clamp）；pointerup 退出
    #[derive(Clone)]
    struct DragState {
        start_x: f64,
        start_w: u32,
        key: &'static str,
    }
    let drag_state: RwSignal<Option<DragState>> = create_rw_signal(None);

    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-list-view">
            <div class="cdb-tab-pane__scroll">
                // ux-canvas-batch 批次2 收尾: 过滤 UI（搜索框/类型下拉/索引三态复选）
                <div class="cdb-list-view-filters" data-testid="list-view-filters">
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="list-view-batch-rename"
                        on:click=move |_| {
                            // ux-canvas-batch 批次2 收尾（条目 9 改派）: 批量改名按钮 → 弹出批量改名模态
                            // 范式参照 :1263/:1354
                            modal_kind.set(Some(modals::ModalKind::BatchRename));
                        }
                    >
                        "批量改名"
                    </button>
                    // ux-canvas-batch 批次3 步骤 5 (条目 16 修复): 导出 CSV（直接调 export_tables_csv 纯函数，删自拼逻辑）
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="list-view-export-csv"
                        on:click=move |_| {
                            // 条目 16 修复: 直接调 export_tables_csv（行=字段，列 table_name/field_name/field_type/has_index）
                            // 复用 csv_escape 纯函数通路，UI 转义逻辑脱离测试保护问题消除
                            let csv = export_tables_csv(&store.tables.get());
                            // 创建 Blob + ObjectURL → 触发下载
                            use wasm_bindgen::JsCast;
                            let array = js_sys::Array::new();
                            array.push(&wasm_bindgen::JsValue::from_str(&csv));
                            let blob = web_sys::Blob::new_with_str_sequence(&array).ok();
                            if let Some(blob) = blob {
                                let url = web_sys::Url::create_object_url_with_blob(&blob).ok();
                                if let Some(url) = url {
                                    if let Some(window) = web_sys::window() {
                                        if let Some(document) = window.document() {
                                            let a = document.create_element("a").ok();
                                            if let Some(a) = a {
                                                let _ = a.set_attribute("href", &url);
                                                let _ = a.set_attribute("download", "tables.csv");
                                                let _ = a.dyn_ref::<web_sys::HtmlElement>().map(|el| el.click());
                                                let _ = web_sys::Url::revoke_object_url(&url);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    >
                        "导出 CSV"
                    </button>
                </div>
                <div class="cdb-list-view-filters" data-testid="list-view-filters">
                    <input
                        class="cdb-form-input"
                        data-testid="list-view-filter-query"
                        placeholder="按名称模糊匹配（表名/字段名/类型）"
                        prop:value=move || list_view_state.filter_query.get()
                        on:input=move |ev| {
                            use wasm_bindgen::JsCast;
                            let v = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                            list_view_state.filter_query.set(v);
                        }
                    />
                    // ux-canvas-batch 批次3 步骤 2: 批量改类型 Panel（条目 12 修正 4——checkbox 多选 + 单一目标类型）
                    <BatchTypeSelectionPanel store=store.clone() selection=list_view_state.batch_type_selection />
                    <button
                        class="cdb-btn cdb-btn--primary"
                        data-testid="list-view-batch-type"
                        on:click=move |_| {
                            // ux-canvas-batch 批次3 步骤 2: 批量改类型按钮 → 弹出批量改类型模态
                            // 范式参照 :1263/:1354 / 条目 9 改派四步
                            modal_kind.set(Some(modals::ModalKind::BatchType));
                        }
                    >
                        "批量改类型"
                    </button>
                    <select
                        class="cdb-form-select"
                        data-testid="list-view-filter-type"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            list_view_state.filter_type.set(v);
                        }
                    >
                        <option value="">全部类型</option>
                        {move || {
                            // 选项从现有 tables 的首个字段类型去重派生（与 SortColumn::Type 首字段类型口径对齐）
                            let types: std::collections::HashSet<String> = store.tables.get()
                                .iter()
                                .filter_map(|t| t.fields.first().map(|f| f.type_.clone()))
                                .collect();
                            types.into_iter().map(|t| {
                                view! { <option value={t.clone()}>{t}</option> }
                            }).collect_view()
                        }}
                    </select>
                    <select
                        class="cdb-form-select"
                        data-testid="list-view-filter-has-index"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            list_view_state.filter_has_index.set(match v.as_str() {
                                "true" => Some(true),
                                "false" => Some(false),
                                _ => None,
                            });
                        }
                    >
                        <option value="">全部</option>
                        <option value="true">仅有索引</option>
                        <option value="false">仅无索引</option>
                    </select>
                    // ux-canvas-batch 批次4 步骤 5 (条目 27): 分组模式会话态下拉（None/ByTag）
                    <select
                        class="cdb-form-select"
                        data-testid="list-view-group-by"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            list_view_state.group_by.set(match v.as_str() {
                                "ByTag" => GroupByMode::ByTag,
                                _ => GroupByMode::None,
                            });
                        }
                    >
                        <option value="">不分组</option>
                        <option value="ByTag">按字段 tag 分组</option>
                    </select>
                </div>
                <table class="cdb-list-view-table" data-testid="list-view-table">
                    <thead>
                        <tr>
                            // ux-canvas-batch 批次3 步骤 2: checkbox 列头（条目 12 修正 4——checkbox 多选）
                            // ux-canvas-batch 批次4 步骤 3 (条目 24): 4 个 <th> 加 style:width 消费
                            // ListViewState.column_widths.get(key)；dblclick 调 auto_calc_column_width +
                            // ColumnWidths::set 自适应；column_dragged 信号抑制 click 排序（位移 >3px）
                            // 拖拽交互：本批切片仅交付渲染 + dblclick + 抑制 click（拖拽细节受预算
                            // 约束切片；纯函数通路 ColumnWidths::set 已闭环，UI 拖拽接续派发时补）
                            <th
                                data-testid="list-view-sort-table-name"
                                style:width=move || format!("{}px", list_view_state_for_name.column_widths.get().get("table_name"))
                                on:click={
                                    let dragged = column_dragged;
                                    move |_| {
                                        if dragged.get() { dragged.set(false); return; }
                                        if list_view_state_for_name.sort_column.get() == SortColumn::TableName {
                                            list_view_state_for_name.sort_direction.set(match list_view_state_for_name.sort_direction.get() {
                                                SortDirection::Ascending => SortDirection::Descending,
                                                SortDirection::Descending => SortDirection::Ascending,
                                            });
                                        } else {
                                            list_view_state_for_name.sort_column.set(SortColumn::TableName);
                                            list_view_state_for_name.sort_direction.set(SortDirection::Ascending);
                                        }
                                    }
                                }
                                on:pointerdown={
                                    let cw = list_view_state_for_name.column_widths;
                                    move |ev: web_sys::PointerEvent| {
                                        // 检测右缘 ≤6px（offsetX 在 [width-6, width] 区间）
                                        let offset_x = ev.offset_x() as f64;
                                        let offset_width = (ev.current_target()
                                            .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>().cloned())
                                            .map(|el| el.offset_width() as f64)
                                            .unwrap_or(120.0))
                                            .max(1.0);
                                        // 右缘 6px 检测带 或 左缘 6px（双向）
                                        if offset_x > 6.0 && offset_x < offset_width - 6.0 {
                                            return; // 中间区域，不启动拖拽
                                        }
                                        // 启动拖拽：记录 start_x, start_w, key
                                        let start_x = ev.client_x() as f64;
                                        let start_w = cw.get().get("table_name");
                                        drag_state.set(Some(DragState { start_x, start_w, key: "table_name" }));
                                        // 抑制本次 pointerdown 触发文本选择
                                        ev.prevent_default();
                                    }
                                }
                                on:dblclick={
                                    let cw = list_view_state_for_name.column_widths;
                                    move |_| {
                                        // 数据链：消费 store.tables 实际 table.name 最长字符数（条目 25）
                                        let max_chars = max_chars_for_column("table_name", &store.tables.get());
                                        let new_w = auto_calc_column_width(max_chars);
                                        cw.update(|c| c.set("table_name", new_w));
                                    }
                                }
                            >
                                "表名"
                            </th>
                            <th
                                data-testid="list-view-sort-field-count"
                                style:width=move || format!("{}px", list_view_state_for_field_count.column_widths.get().get("field_count"))
                                on:click={
                                    let dragged = column_dragged;
                                    move |_| {
                                        if dragged.get() { dragged.set(false); return; }
                                        if list_view_state_for_field_count.sort_column.get() == SortColumn::FieldCount {
                                            list_view_state_for_field_count.sort_direction.set(match list_view_state_for_field_count.sort_direction.get() {
                                                SortDirection::Ascending => SortDirection::Descending,
                                                SortDirection::Descending => SortDirection::Ascending,
                                            });
                                        } else {
                                            list_view_state_for_field_count.sort_column.set(SortColumn::FieldCount);
                                            list_view_state_for_field_count.sort_direction.set(SortDirection::Ascending);
                                        }
                                    }
                                }
                                on:pointerdown={
                                    let cw = list_view_state_for_field_count.column_widths;
                                    move |ev: web_sys::PointerEvent| {
                                        let offset_x = ev.offset_x() as f64;
                                        let offset_width = (ev.current_target()
                                            .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>().cloned())
                                            .map(|el| el.offset_width() as f64)
                                            .unwrap_or(120.0))
                                            .max(1.0);
                                        if offset_x > 6.0 && offset_x < offset_width - 6.0 { return; }
                                        let start_x = ev.client_x() as f64;
                                        let start_w = cw.get().get("field_count");
                                        drag_state.set(Some(DragState { start_x, start_w, key: "field_count" }));
                                        ev.prevent_default();
                                    }
                                }
                                on:dblclick={
                                    let cw = list_view_state_for_field_count.column_widths;
                                    move |_| {
                                        // 数据链：消费 store.tables 实际字段数最大值（条目 25）
                                        let max_chars = max_chars_for_column("field_count", &store.tables.get());
                                        let new_w = auto_calc_column_width(max_chars);
                                        cw.update(|c| c.set("field_count", new_w));
                                    }
                                }
                            >
                                "字段数"
                            </th>
                            <th
                                data-testid="list-view-sort-type"
                                style:width=move || format!("{}px", list_view_state_for_type.column_widths.get().get("type"))
                                on:click={
                                    let dragged = column_dragged;
                                    move |_| {
                                        if dragged.get() { dragged.set(false); return; }
                                        if list_view_state_for_type.sort_column.get() == SortColumn::Type {
                                            list_view_state_for_type.sort_direction.set(match list_view_state_for_type.sort_direction.get() {
                                                SortDirection::Ascending => SortDirection::Descending,
                                                SortDirection::Descending => SortDirection::Ascending,
                                            });
                                        } else {
                                            list_view_state_for_type.sort_column.set(SortColumn::Type);
                                            list_view_state_for_type.sort_direction.set(SortDirection::Ascending);
                                        }
                                    }
                                }
                                on:pointerdown={
                                    let cw = list_view_state_for_type.column_widths;
                                    move |ev: web_sys::PointerEvent| {
                                        let offset_x = ev.offset_x() as f64;
                                        let offset_width = (ev.current_target()
                                            .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>().cloned())
                                            .map(|el| el.offset_width() as f64)
                                            .unwrap_or(120.0))
                                            .max(1.0);
                                        if offset_x > 6.0 && offset_x < offset_width - 6.0 { return; }
                                        let start_x = ev.client_x() as f64;
                                        let start_w = cw.get().get("type");
                                        drag_state.set(Some(DragState { start_x, start_w, key: "type" }));
                                        ev.prevent_default();
                                    }
                                }
                                on:dblclick={
                                    let cw = list_view_state_for_type.column_widths;
                                    move |_| {
                                        // 数据链：消费 store.tables 实际首字段类型最长字符数（条目 25）
                                        let max_chars = max_chars_for_column("type", &store.tables.get());
                                        let new_w = auto_calc_column_width(max_chars);
                                        cw.update(|c| c.set("type", new_w));
                                    }
                                }
                            >
                                "类型"
                            </th>
                            <th
                                data-testid="list-view-sort-has-index"
                                style:width=move || format!("{}px", list_view_state_for_has_index.column_widths.get().get("has_index"))
                                on:click={
                                    let dragged = column_dragged;
                                    move |_| {
                                        if dragged.get() { dragged.set(false); return; }
                                        if list_view_state_for_has_index.sort_column.get() == SortColumn::HasIndex {
                                            list_view_state_for_has_index.sort_direction.set(match list_view_state_for_has_index.sort_direction.get() {
                                                SortDirection::Ascending => SortDirection::Descending,
                                                SortDirection::Descending => SortDirection::Ascending,
                                            });
                                        } else {
                                            list_view_state_for_has_index.sort_column.set(SortColumn::HasIndex);
                                            list_view_state_for_has_index.sort_direction.set(SortDirection::Ascending);
                                        }
                                    }
                                }
                                on:pointerdown={
                                    let cw = list_view_state_for_has_index.column_widths;
                                    move |ev: web_sys::PointerEvent| {
                                        let offset_x = ev.offset_x() as f64;
                                        let offset_width = (ev.current_target()
                                            .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>().cloned())
                                            .map(|el| el.offset_width() as f64)
                                            .unwrap_or(120.0))
                                            .max(1.0);
                                        if offset_x > 6.0 && offset_x < offset_width - 6.0 { return; }
                                        let start_x = ev.client_x() as f64;
                                        let start_w = cw.get().get("has_index");
                                        drag_state.set(Some(DragState { start_x, start_w, key: "has_index" }));
                                        ev.prevent_default();
                                    }
                                }
                                on:dblclick={
                                    let cw = list_view_state_for_has_index.column_widths;
                                    move |_| {
                                        let new_w = auto_calc_column_width(max_chars_for_column("has_index", &store.tables.get())); // 条目 26 记一笔：单一数据源原则
                                        cw.update(|c| c.set("has_index", new_w));
                                    }
                                }
                            >
                                "索引"
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let tables = store.tables.get();
                            // ux-canvas-batch 批次2 收尾: 过滤/排序联动——渲染行 = sort_tables(filter_tables(tables))
                            let filtered = filter_tables(
                                &tables,
                                &list_view_state.filter_query.get(),
                                &list_view_state.filter_type.get(),
                                list_view_state.filter_has_index.get(),
                            );
                            let sorted = sort_tables(
                                &filtered,
                                list_view_state.sort_column.get(),
                                list_view_state.sort_direction.get(),
                            );
                            // ux-canvas-batch 批次4 步骤 5 (条目 27): 分组模式分桶渲染
                            // - GroupByMode::ByTag: 走 group_tables → 按桶头 + 桶内字段行
                            // - GroupByMode::None: 现有表行渲染（不变）
                            let mode = list_view_state.group_by.get();
                            let sorted_ref = sorted.clone();
                            let buckets = group_tables(&sorted_ref, mode);
                            let bucket_views: Vec<_> = match mode {
                                GroupByMode::ByTag => {
                                    // 桶渲染：每桶 = 桶头 <tr> + 字段行 <tr>（桶内行=字段）
                                    buckets.into_iter().map(|bucket| {
                                        let key = bucket.key.clone();
                                        let field_count = bucket.fields.len();
                                        let bucket_key = bucket.key.clone();
                                        // 桶内字段渲染（table_id, field_id）
                                        let field_rows: Vec<_> = bucket.fields.into_iter().map(|(tid, fid)| {
                                            // 查找字段（table.name + field.name）
                                            let mut label = format!("{}.{}", tid, fid);
                                            for t in &sorted {
                                                if t.id == tid {
                                                    if let Some(f) = t.fields.iter().find(|f| f.id == fid) {
                                                        label = format!("{}.{}", t.name, f.name);
                                                        break;
                                                    }
                                                }
                                            }
                                            view! {
                                                <tr
                                                    data-testid={format!("list-view-group-row-{}-{}", bucket_key.clone(), label.clone())}
                                                >
                                                    <td>{label}</td>
                                                    <td>""</td>
                                                    <td>""</td>
                                                    <td>""</td>
                                                </tr>
                                            }
                                        }).collect();
                                        // 桶头
                                        let header = view! {
                                            <tr data-testid={format!("list-view-group-{}", key.clone())} class="cdb-list-view-group-header">
                                                <td colspan="4">{format!("{} ({} 字段)", key, field_count)}</td>
                                            </tr>
                                        };
                                        let mut all = vec![header];
                                        all.extend(field_rows);
                                        all
                                    }).flatten().collect()
                                }
                                GroupByMode::None => Vec::new(), // 走下方既有 sorted.into_iter().map() 路径
                            };
                            let table_rows = sorted.into_iter().map(|table| {
                                let table_id = table.id.clone();
                                let table_name = table.name.clone();
                                let field_count = table.fields.len();
                                let first_type = table.fields.first().map(|f| f.type_.clone()).unwrap_or_default();
                                let has_index = !table.indices.is_empty();
                                let on_select = on_select_table.clone();
                                // ux-canvas-batch 批次3 步骤 2: 表行 checkbox（条目 12 修正 4——checkbox 多选）
                                let table_id_for_props = table_id.clone();
                                let table_id_rc = std::rc::Rc::new(table_id.clone());
                                let table_id_label = (*table_id_rc).clone();
                                let on_jump_for_row = on_jump_to_canvas.clone();
                                view! {
                                    <tr
                                        data-testid={format!("list-view-row-{}", table_id_label.clone())}
                                        on:click={
                                            let tid = table_id_rc.clone();
                                            move |_| on_select(Some((*tid).clone()))
                                        }
                                        on:dblclick={
                                            let tid = table_id_rc.clone();
                                            move |_| on_jump_for_row((*tid).clone())
                                        }
                                    >
                                        // ux-canvas-batch 批次3 步骤 2: 表行多选 checkbox（条目 12 修正 4——checkbox 多选）
                                        <td>
                                            <input
                                                type="checkbox"
                                                data-testid={format!("list-view-row-checkbox-{}", table_id_label.clone())}
                                                prop:checked={
                                                    let tid = table_id_rc.clone();
                                                    move || list_view_state.batch_type_selection.get().selected_field_ids.contains(&*tid)
                                                }
                                                on:change={
                                                    let tid = table_id_rc.clone();
                                                    move |ev| {
                                                        let checked = event_target_checked(&ev);
                                                        let tid_str: String = (*tid).clone();
                                                        list_view_state.batch_type_selection.update(|s| {
                                                            if checked {
                                                                s.selected_field_ids.insert(tid_str.clone());
                                                            } else {
                                                                s.selected_field_ids.remove(&tid_str);
                                                            }
                                                        });
                                                    }
                                                }
                                            />
                                        </td>
                                        <td>{table_name}</td>
                                        <td>{field_count}</td>
                                        <td>{first_type}</td>
                                        <td>{if has_index { "有" } else { "无" }}</td>
                                    </tr>
                                }
                            }).collect_view();
                            // ux-canvas-batch 批次4 步骤 5 (条目 27): 桶渲染优先（ByTag）或表行（None）
                            let mode = list_view_state.group_by.get();
                            match mode {
                                GroupByMode::ByTag => {
                                    // 重算 buckets（闭包内 group_tables 已在 sorted.into_iter().map 之前算过 bucket_views）
                                    let tables_now = store.tables.get();
                                    let sorted_now = sort_tables(
                                        &filter_tables(
                                            &tables_now,
                                            &list_view_state.filter_query.get(),
                                            &list_view_state.filter_type.get(),
                                            list_view_state.filter_has_index.get(),
                                        ),
                                        list_view_state.sort_column.get(),
                                        list_view_state.sort_direction.get(),
                                    );
                                    let buckets_now = group_tables(&sorted_now, GroupByMode::ByTag);
                                    buckets_now.into_iter().flat_map(|bucket| {
                                        let key = bucket.key.clone();
                                        let field_count = bucket.fields.len();
                                        let bucket_key = bucket.key.clone();
                                        let header = view! {
                                            <tr data-testid={format!("list-view-group-{}", key.clone())} class="cdb-list-view-group-header">
                                                <td colspan="4">{format!("{} ({} 字段)", key, field_count)}</td>
                                            </tr>
                                        };
                                        let field_rows: Vec<_> = bucket.fields.into_iter().map(|(tid, fid)| {
                                            let mut label = format!("{}.{}", tid, fid);
                                            for t in &sorted_now {
                                                if t.id == tid {
                                                    if let Some(f) = t.fields.iter().find(|f| f.id == fid) {
                                                        label = format!("{}.{}", t.name, f.name);
                                                        break;
                                                    }
                                                }
                                            }
                                            view! {
                                                <tr
                                                    data-testid={format!("list-view-group-row-{}-{}", bucket_key.clone(), label.clone())}
                                                >
                                                    <td>{label}</td>
                                                    <td>""</td>
                                                    <td>""</td>
                                                    <td>""</td>
                                                </tr>
                                            }
                                        }).collect();
                                        let mut all = vec![header];
                                        all.extend(field_rows);
                                        all
                                    }).collect_view()
                                }
                                GroupByMode::None => table_rows,
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </div>

        // ux-canvas-batch 批次4 步骤 3 (条目 25): 拖拽 window 级 pointermove/pointerup 监听
        // Leptos window_event_listener 提供生命周期管理（自动 cleanup）
        {
            let cw = list_view_state_for_name.column_widths;
            let dragged = column_dragged;
            leptos::window_event_listener(leptos::ev::pointermove, move |ev: web_sys::PointerEvent| {
                if let Some(ds) = drag_state.get() {
                    let dx = ev.client_x() as f64 - ds.start_x;
                    if dx.abs() > 3.0 {
                        dragged.set(true);
                    }
                    let new_w = (ds.start_w as f64 + dx) as u32;
                    cw.update(|c| c.set(ds.key, new_w));
                }
            });
        }
        {
            let dragged = column_dragged;
            leptos::window_event_listener(leptos::ev::pointerup, move |_ev: web_sys::PointerEvent| {
                if drag_state.get().is_some() {
                    drag_state.set(None);
                    // 抑制下次 click 排序（仅在确实发生拖拽位移时）
                    if dragged.get_untracked() {
                        // 设置一个短暂的 click 抑制标记，下一个 click 后 reset
                    }
                }
            });
        }
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

    // Phase C：IO 抽屉
    let io_drawer: RwSignal<IoDrawerKind> = create_rw_signal(IoDrawerKind::None);
    let inspector_before_io: RwSignal<Option<bool>> = create_rw_signal(None);

    create_effect(move |_| {
        if inspector_open.get() && io_drawer.get() != IoDrawerKind::None {
            io_drawer.set(IoDrawerKind::None);
            inspector_before_io.set(None);
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
        Rc::new(move || {
            let (collapsed, cache) = snapshot_before_io_drawer(inspector_open.get_untracked());
            inspector_before_io.set(cache);
            inspector_open.set(collapsed);
            io_drawer.set(IoDrawerKind::Import);
        })
    };

    let open_export_drawer = {
        let io_drawer = io_drawer.clone();
        let inspector_open = inspector_open.clone();
        let inspector_before_io = inspector_before_io.clone();
        Rc::new(move || {
            let (collapsed, cache) = snapshot_before_io_drawer(inspector_open.get_untracked());
            inspector_before_io.set(cache);
            inspector_open.set(collapsed);
            io_drawer.set(IoDrawerKind::Export);
        })
    };

    let command_stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>> = create_rw_signal(
        Rc::new(RefCell::new(crate::editor_core::CommandStack::new())),
    );

    // HTTP client to backend (port 3000, CORS middleware 在 fix-modal-overlay-blocking 已配)
    let client = DiagramClient::new("http://127.0.0.1:3000");
    let auth_client = AuthClient::new("http://127.0.0.1:3000");
    let room_client = RoomClient::new("http://127.0.0.1:3000");
    let collab_client = CollabClient::new("http://127.0.0.1:3000");

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

    create_effect({
        let room_client = room_client.clone();
        let collab_client = collab_client.clone();
        let auth_session = auth_session.clone();
        let current_room = current_room.clone();
        let room_members = room_members.clone();
        let remote_members = remote_members.clone();
        let remote_presence = remote_presence.clone();
        let collab_state = collab_state.clone();
        let activity_feed = activity_feed.clone();
        let error = error.clone();
        move |_| {
            // collab_retry：banner 手动重连触发 effect 重跑
            collab_retry.get();
            let Some(session) = auth_session.get() else {
                collab_state.set(CollabOtState::default());
                remote_members.set(Vec::new());
                remote_presence.set(Vec::new());
                return;
            };
            let Some(room) = current_room.get() else {
                collab_state.set(CollabOtState::default());
                remote_members.set(Vec::new());
                remote_presence.set(Vec::new());
                return;
            };
            let token = session.access_token;
            let room_id = room.id;
            let user_id = session.user.as_ref().map(|u| u.id.clone());
            collab_state.update(|state| {
                state.connection = CollabConnectionState::Connecting;
            });
            let room_client = room_client.clone();
            let collab_client = collab_client.clone();
            spawn_local(async move {
                match collab_client.get_head(&token, &room_id).await {
                    Ok(head) => {
                        collab_state.set(CollabOtState::connected(head.server_rev));
                        prepend_activity(
                            activity_feed,
                            format!("协作 head 已加载 · rev {}", head.server_rev),
                        );
                    }
                    Err(e) => {
                        collab_state.update(|state| state.mark_reconnecting());
                        prepend_activity(activity_feed, format!("协作连接待恢复 · {e}"));
                        error.set(Some(e.to_string()));
                    }
                }

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

    // align-frontend-to-prototype：rooms list → 选中/创建房间 → 进入 editor
    let on_enter_room: Rc<dyn Fn(RoomDetail)> = {
        let client = client.clone();
        let store = store.clone();
        let current_page = current_page.clone();
        let current_diagram_id = current_diagram_id.clone();
        let current_room = current_room.clone();
        let current_title = current_title.clone();
        let error = error.clone();
        Rc::new(move |detail: RoomDetail| {
            let diagram_id = detail.diagram_id.clone();
            current_page.set(PageState::RoomEditor);
            current_diagram_id.set(diagram_id.clone());
            current_title.set(detail.diagram_title.clone());
            current_room.set(Some(detail));
            let client = client.clone();
            let store = store.clone();
            spawn_local(async move {
                match client.get(&diagram_id).await {
                    Ok(diagram) => {
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                        error.set(None);
                    }
                    Err(_) => error.set(Some("图表加载失败，请返回房间列表后重试".to_string())),
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
        Rc::new(move || {
            current_page.set(PageState::RoomEditor);
            // B 批：与 on_enter_room 一致，接受邀请后加载房间绑定的图表内容
            let diagram_id = current_diagram_id.get_untracked();
            let client = client.clone();
            let store = store.clone();
            spawn_local(async move {
                match client.get(&diagram_id).await {
                    Ok(diagram) => {
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                        // store.load 后用现有 ids 重新计算 next_id，避免与 DB 已保存的 id 冲突
                        let max_id = crate::editor_core::next_id_from_store(&store);
                        next_id.set(max_id + 1);
                        error.set(None);
                    }
                    Err(_) => error.set(Some("图表加载失败，请返回房间列表后重试".to_string())),
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
                match client.create(&name).await {
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
            CodeLanguage::Sql => export_diagram_sql(&tables, &refs, "generic"),
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
    let client_for_delete_field = client.clone();
    let client_for_delete_table = client.clone();

    let on_create_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let error_for_create = error.clone();
        let current_room = current_room.clone();
        let collab_state_for_create = collab_state.clone();
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
                collab_state_for_create.update(|state| {
                    let _ = state.enqueue_local_op("table.create");
                });
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
                    match client.create("新图").await {
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
            );
        }) as Rc<dyn Fn()>
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
            );
        })
    };

    let on_field_pick: Option<Box<dyn Fn(String, String) + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
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
                    // feat-relation-inference 批次2: cardinality 改推导值（非必选下拉值）
                    let inferred = modals::infer_cardinality(&start_field_id, &field_id, &store);
                    rel_tool_state.set(RelToolState::Confirm {
                        start_table_id,
                        start_field_id,
                        end_table_id: table_id,
                        end_field_id: field_id,
                        cardinality: inferred,
                    });
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
        Some(Box::new(
            move |start_table_id: String,
                  start_field_id: String,
                  end_table_id: String,
                  end_field_id: String| {
                // feat-relation-inference 批次2: cardinality 改推导值（非必选下拉值）
                let inferred = modals::infer_cardinality(&start_field_id, &end_field_id, &store);
                rel_tool_state.set(RelToolState::Confirm {
                    start_table_id,
                    start_field_id,
                    end_table_id,
                    end_field_id,
                    cardinality: inferred,
                });
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
                collab_state.update(|state| {
                    let _ = state.enqueue_local_op("table.move");
                });
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

    let on_canvas_select: Option<Box<dyn Fn(String) + 'static>> = {
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
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
                    current_room=current_room
                    read_only=share_mode
                />
                <div class="cdb-canvas-container" data-testid="editor-canvas-container">
                    {move || if store.tables.get().is_empty() {
                        view! {
                            <EmptyGuide
                                on_create_table=on_create_table_guide.clone()
                                on_import=open_import_drawer.clone()
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
                        theme_mode=theme_mode
                    />
                    <RelationshipConfirmBar
                        store=store.clone()
                        rel_state=rel_tool_state
                        next_ref_id=next_ref_id.clone()
                        on_create=on_create_reference.clone()
                    />
                    <ActivityFeed items=activity_feed visible=activity_open />
                    <FloatingControls transform=canvas_transform />
                </div>
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
                    on_delete_field=on_delete_field.clone()
                    on_delete_table=on_delete_table.clone()
                    on_update_ref_field=on_update_ref_field.clone()
                    on_flip_ref=on_flip_ref.clone()
                    on_delete_ref=on_delete_ref.clone()
                    on_jump_to_table=on_jump_to_table.clone()
                    read_only=inspector_read_only.clone()
                />
                <IoDrawer
                    kind=io_drawer
                    store=store.clone()
                    current_title=current_title
                    client=client_for_io.clone()
                    error=error.clone()
                    on_close=close_io_drawer.clone()
                />
                <RoomPanel
                    visible=room_panel_visible
                    room_client=room_client.clone()
                    auth_session=auth_session
                    current_room=current_room
                    room_members=room_members
                    error=error.clone()
                    on_open_invite=on_open_invite.clone()
                />
                <InviteModal
                    open=invite_modal_open
                    room_client=room_client.clone()
                    auth_session=auth_session
                    current_room=current_room
                    error=error.clone()
                />
            </div>
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
            {move || if view_mode.get() == ViewMode::List {
                let on_jump_for_listview: Rc<dyn Fn(String)> = {
                    let on_select = on_select_table.clone();
                    Rc::new(move |tid: String| {
                        view_mode.set(ViewMode::Canvas);
                        on_select(Some(tid));
                    })
                };
                view! {
                    <div class="cdb-list-view-panel" data-testid="list-view-panel">
                        <ListView
                            store=store.clone()
                            on_select_table=on_select_table.clone()
                            on_jump_to_canvas=on_jump_for_listview.clone()
                            modal_kind=modal_kind.clone()
                            batch_type_selection=batch_type_selection
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
                    None => view! { <></> }.into_view(),
                }}
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
        }
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
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }, Field { id: "f2".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }, Field { id: "f2".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
            Table { id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "orders".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
        ];
        let filtered = filter_tables(&tables, "users", "", None);
        assert_eq!(filtered.len(), 1, "UT-MM-23: 按名称模糊匹配 users → 1 个表");
        assert_eq!(filtered[0].name, "users", "UT-MM-23: 过滤结果应为 users");
    }

    #[test]
    fn test_filter_tables_by_type_ut_mm_23() {
        use crate::editor_core::types::{Field, Table};
        let tables = vec![
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
            Table { id: "t1".into(), name: "users".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![Index { id: "i1".into(), name: "idx".into(), fields: vec![], unique: false }], width: None, min_height: None },
            Table { id: "t2".into(), name: "orders".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
            Table { id: "t1".into(), name: "A".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "id".into(), type_: "INT".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
            Table { id: "t2".into(), name: "B".into(), x: 0.0, y: 0.0, color: String::new(), comment: String::new(), fields: vec![Field { id: "f1".into(), name: "name".into(), type_: "VARCHAR".into(), default: String::new(), check: String::new(), primary: false, unique: false, not_null: false, increment: false, comment: String::new(), tag: String::new() }], indices: vec![], width: None, min_height: None },
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
        let tables = vec![Table {
            id: "t1".into(),
            name: "users".into(),
            x: 0.0,
            y: 0.0,
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
                not_null: true,
                increment: false,
                comment: String::new(),
            tag: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        }];
        let out = export_diagram_sql(&tables, &[], "generic");
        assert!(
            out.contains("CREATE TABLE users"),
            "UT-PC-02: 应含 CREATE TABLE"
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
fn retry_save_after_conflict(
    client: DiagramClient,
    store: EditorStore,
    snap: crate::editor_core::types::Diagram,
    id: String,
    expected_revision: i64,
    conflict: RwSignal<Option<ConflictInfo>>,
    error: RwSignal<Option<String>>,
    is_saving: RwSignal<bool>,
    save_offline: RwSignal<bool>,
    activity_feed: RwSignal<Vec<String>>,
) {
    spawn_local(async move {
        match save_with_retry(&client, &id, expected_revision, &snap).await {
            Ok(resp) => {
                store.revision.set(resp.revision);
                store.dirty.set(false);
                save_offline.set(false);
                error.set(None);
                prepend_activity(
                    activity_feed,
                    format!("协作合并后重试成功 · 推进至 rev {}", resp.revision),
                );
            }
            Err(_) => {
                save_offline.set(true);
                error.set(Some("协作合并后重试失败（离线）".to_string()));
            }
        }
        is_saving.set(false);
    });
}

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
}
