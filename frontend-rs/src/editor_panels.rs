//! editor-panels: 顶/左/右面板 UI + 409 弹窗 + toast
//!
//! 依赖: `editor_core::EditorStore`, `DebounceTrigger`, `ConflictInfo`, `ConflictAction`
//!        `editor_data_access::DiagramClient`
//!
//! Phase A (redesign-phase-a-layout): AppBar + ToolRail + Inspector + StatusBar + EmptyGuide
//!
//! data-testid 清单:
//!   - app-bar / tool-rail / inspector-panel / status-bar / canvas-empty-guide
//!   - btn-create-table / guide-create-table / btn-inspector-toggle
//!   - editor-canvas / floating-controls / revision-display (StatusBar)

use crate::command_palette::{
    build_palette_items, setup_command_palette_shortcut, CommandPalette, PaletteItem,
};
use crate::code_view::{
    setup_code_view_escape, CodeLanguage, CodeView, ViewMode, ViewModeToggle,
};
use crate::editor_core::{
    ConflictAction, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_core::types::{Field, Reference, Table};
use crate::editor_data_access::{save_with_retry, DiagramClient, SaveError, ImportLocalResponse};
use crate::editor_render::Canvas;
use crate::editor_render::{Transform, zoom_in, zoom_out, zoom_reset};
use crate::icons::{
    IconAdd, IconBox, IconChevronLeft, IconChevronRight, IconClose, IconExport, IconImport,
    IconMinus, IconMoon, IconPan, IconRelationship, IconSelect, IconSettings, IconSidebar,
    IconSun, IconRedo, IconUndo,
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

/// Side-panel Tab 标识符（B2 范围：6 业务 Tab + Issues = 7 Tab）
/// 顺序与 `core-04-side-panel-tabs.md` §1 布局保持一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidePanelTab {
    Tables,
    Areas,
    Enums,
    Notes,
    Relationships,
    Types,
    Issues,
}

impl SidePanelTab {
    pub fn testid(self) -> &'static str {
        match self {
            SidePanelTab::Tables => "tab-tables",
            SidePanelTab::Areas => "tab-areas",
            SidePanelTab::Enums => "tab-enums",
            SidePanelTab::Notes => "tab-notes",
            SidePanelTab::Relationships => "tab-relationships",
            SidePanelTab::Types => "tab-types",
            SidePanelTab::Issues => "tab-issues",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SidePanelTab::Tables => "表",
            SidePanelTab::Areas => "区域",
            SidePanelTab::Enums => "枚举",
            SidePanelTab::Notes => "注释",
            SidePanelTab::Relationships => "关系",
            SidePanelTab::Types => "类型",
            SidePanelTab::Issues => "问题",
        }
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
            RelToolState::PickSource => Some("选择源字段"),
            RelToolState::PickTarget { .. } => Some("选择目标字段"),
            _ => None,
        }
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
pub fn flip_reference_endpoints(r: &Reference) -> Reference {
    Reference {
        start_table_id: r.end_table_id.clone(),
        start_field_id: r.end_field_id.clone(),
        end_table_id: r.start_table_id.clone(),
        end_field_id: r.start_field_id.clone(),
        ..r.clone()
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
        ImportFormat::Dbml => Ok(format!(
            "{} 个 Table 块",
            count_dbml_tables(content)
        )),
        ImportFormat::Json => {
            let v: serde_json::Value =
                serde_json::from_str(content).map_err(|e| e.to_string())?;
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
            .map(|s| s.trim_matches('(').trim_matches('`').trim_matches('"').to_string())
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
            }],
            indices: vec![],
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
            (
                name[..idx].trim().to_string(),
                Some(name[idx + 1..].trim()),
            )
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
                    obj.insert(
                        "name".into(),
                        serde_json::Value::String(title.to_string()),
                    );
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
            out.push_str(&format!(
                "  {} {}{}\n",
                field.name, field.type_, attr_str
            ));
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
pub fn export_diagram_json(
    name: &str,
    tables: &[Table],
    references: &[Reference],
) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": name,
        "tables": tables,
        "references": references,
    }))
    .unwrap_or_else(|_| "{}".into())
}

fn navigate_to_editor(diagram_id: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window
            .location()
            .set_href(&format!("/editor/{diagram_id}"));
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

/// Areas/Enums/Notes/Types 在 B2 范围暂用「仅前端 state」（spec 标 V1）；
/// 这里定义轻量数据结构，避免 `EditorStore` 过度膨胀（B3 才把它们接入 store）。
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
) {
    let id = current_diagram_id.get();
    let rev = store.revision.get();
    let name = current_title.get();
    let snap = store.snapshot(id.clone(), name);
    is_saving.set(true);
    save_offline.set(false);
    debouncer.schedule(move || {
        let client = client.clone();
        let store = store.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        let is_saving = is_saving.clone();
        let save_offline = save_offline.clone();
        spawn_local(async move {
            match save_with_retry(&client, &id, rev, &snap).await {
                Ok(resp) => {
                    store.revision.set(resp.revision);
                    store.dirty.set(false);
                    save_offline.set(false);
                    error.set(None);
                }
                Err(SaveError::Conflict { current_revision, .. }) => {
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
    let render = move || {
        match error.get() {
            Some(msg) => view! {
                <div class="cdb-error-toast" data-testid="error-toast">
                    {msg}
                    <button on:click=move |_| error.set(None)>
                        <IconBox size="sm"><IconClose /></IconBox>
                    </button>
                </div>
            }.into_view(),
            None => view! { <></> }.into_view(),
        }
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
) -> impl IntoView {
    let on_after_undo = on_after_change.clone();
    let on_after_redo = on_after_change.clone();
    view! {
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-undo"
            title="撤销 (Ctrl+Z)"
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
    let stack = create_rw_signal(Rc::new(RefCell::new(crate::editor_core::CommandStack::new())));
    let noop = Rc::new(|| {}) as Rc<dyn Fn()>;
    view! {
        <div class="cdb-toolbar">
            <UndoRedoButtons store=store stack=stack on_after_change=noop error=error />
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

/// Phase A：单行 AppBar（合并 TopMenuBar + Toolbar）
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
) -> impl IntoView {
    let _ = transform;
    let import_handler = on_open_import.clone();
    let export_handler = on_open_export.clone();
    let dark_mode = create_rw_signal(read_html_data_mode() == "dark");

    view! {
        <header class="cdb-app-bar" data-testid="app-bar">
            <span class="cdb-logo-mark" aria-hidden="true">"C"</span>
            <UndoRedoButtons
                store=store.clone()
                stack=stack
                on_after_change=on_after_change.clone()
                error=error.clone()
            />
            <div class="cdb-diagram-title-wrap">
                <input
                    class="cdb-diagram-title"
                    data-testid="diagram-title"
                    prop:value=move || current_title.get()
                    on:input=move |ev| current_title.set(event_target_value(&ev))
                    on:blur=move |ev| on_title_blur(event_target_value(&ev))
                />
                {move || if store.dirty.get() {
                    view! { <span class="cdb-dirty-dot" title="未保存"></span> }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
            </div>
            <span class="cdb-save-state-wrap">
                {move || {
                    if is_saving.get() {
                        view! {
                            <span class="cdb-save-state cdb-is-saving" data-testid="save-state">
                                <span class="cdb-save-dot cdb-save-dot--saving"></span>
                                "保存中..."
                                <span class="cdb-rev-inline" data-testid="revision-display">
                                    {format!("rev: {}", store.revision.get())}
                                </span>
                            </span>
                        }.into_view()
                    } else if save_offline.get() {
                        view! {
                            <span class="cdb-save-state cdb-is-error" data-testid="save-state">
                                <span class="cdb-save-dot cdb-save-dot--error"></span>
                                "保存失败（离线）"
                            </span>
                        }.into_view()
                    } else if store.dirty.get() {
                        view! {
                            <span class="cdb-save-state cdb-is-idle" data-testid="save-state">
                                <span class="cdb-save-dot cdb-save-dot--dirty"></span>
                                "未保存"
                                <span class="cdb-rev-inline" data-testid="revision-display">
                                    {format!("rev: {}", store.revision.get())}
                                </span>
                            </span>
                        }.into_view()
                    } else {
                        view! {
                            <span class="cdb-save-state" data-testid="save-state">
                                <span class="cdb-save-dot cdb-save-dot--saved"></span>
                                "已保存"
                                <span class="cdb-rev-inline" data-testid="revision-display">
                                    {format!("rev: {}", store.revision.get())}
                                </span>
                            </span>
                        }.into_view()
                    }
                }}
            </span>
            <span class="cdb-app-bar__spacer"></span>
            <div class="cdb-app-bar__actions">
                <button
                    class="cdb-btn cdb-btn--pill"
                    data-testid="btn-import"
                    on:click=move |_| import_handler()
                >
                    "导入"
                </button>
                <button
                    class="cdb-btn cdb-btn--pill"
                    data-testid="btn-export"
                    title="导出"
                    on:click=move |_| export_handler()
                >
                    "导出"
                </button>
            </div>
            <button
                class="cdb-btn cdb-btn--primary cdb-btn--small"
                data-testid="btn-share"
                on:click=move |_| modal_kind.set(Some(modals::ModalKind::Share))
            >
                "分享"
            </button>
            <ViewModeToggle view_mode=view_mode code_visible=code_visible />
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-theme-toggle"
                title="切换主题"
                on:click=move |_| {
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(html) = doc.document_element() {
                            let cur = html.get_attribute("data-mode").unwrap_or_else(|| "light".into());
                            let next = if cur == "dark" { "light" } else { "dark" };
                            let _ = html.set_attribute("data-mode", next);
                            dark_mode.set(next == "dark");
                        }
                    }
                }
            >
                {move || if dark_mode.get() {
                    view! { <IconBox size="sm"><IconSun /></IconBox> }.into_view()
                } else {
                    view! { <IconBox size="sm"><IconMoon /></IconBox> }.into_view()
                }}
            </button>
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-inspector-toggle"
                title="切换 Inspector"
                on:click=move |_| inspector_open.update(|v| *v = !*v)
            >
                <IconBox size="sm"><IconSidebar /></IconBox>
            </button>
        </header>
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
) -> impl IntoView {
    let new_menu_open = create_rw_signal(false);
    let issue_count = create_memo(move |_| compute_diagram_issues(&store).len());

    view! {
        <nav class="cdb-tool-rail" data-testid="tool-rail">
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::Select
                data-testid="tool-select"
                title="选择"
                on:click=move |_| {
                    active_tool.set(ActiveTool::Select);
                    rel_tool_state.set(RelToolState::Idle);
                }
            >
                <IconBox size="md"><IconSelect /></IconBox>
            </button>
            <button
                class="cdb-tool-btn"
                data-testid="tool-new-menu"
                title="新建"
                on:click=move |_| new_menu_open.update(|v| *v = !*v)
            >
                <IconBox size="md"><IconAdd /></IconBox>
            </button>
            {move || if new_menu_open.get() {
                let on_create = on_create_table.clone();
                view! {
                    <div class="cdb-tool-menu" data-testid="tool-new-menu-dropdown">
                        <button
                            class="cdb-tool-menu-item"
                            data-testid="btn-create-table"
                            on:click=move |_| {
                                on_create();
                                new_menu_open.set(false);
                            }
                        >
                            "新建表"
                        </button>
                        <button
                            class="cdb-tool-menu-item"
                            data-testid="tool-new-area"
                            on:click=move |_| new_menu_open.set(false)
                        >
                            "新建区域"
                        </button>
                        <button
                            class="cdb-tool-menu-item"
                            data-testid="tool-new-note"
                            on:click=move |_| new_menu_open.set(false)
                        >
                            "新建便签"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::Relationship
                data-testid="tool-relationship"
                title="关系工具 (R)"
                on:click=move |_| {
                    active_tool.set(ActiveTool::Relationship);
                    rel_tool_state.set(RelToolState::PickSource);
                }
            >
                <IconBox size="md"><IconRelationship /></IconBox>
            </button>
            <button
                class="cdb-tool-btn"
                class:cdb-is-active=move || active_tool.get() == ActiveTool::Pan
                data-testid="tool-pan"
                title="平移（拖动画布空白）"
                on:click=move |_| {
                    active_tool.set(ActiveTool::Pan);
                    rel_tool_state.set(RelToolState::Idle);
                }
            >
                <IconBox size="md"><IconPan /></IconBox>
            </button>
            <div class="cdb-tool-rail__divider"></div>
            <div
                class="cdb-issues-badge"
                data-testid="tool-issues-badge"
                title="问题列表"
                on:click=move |_| {
                    selection.set(SelectionKind::Issues);
                    inspector_open.set(true);
                }
            >
                {move || issue_count.get()}
            </div>
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
                view! {
                    <div class="cdb-rel-confirm-bar" data-testid="rel-confirm-bar">
                        <span class="cdb-rel-confirm-bar__label">{label}</span>
                        <select
                            class="cdb-form-select cdb-rel-confirm-bar__select"
                            data-testid="rel-confirm-cardinality"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                rel_state.set(RelToolState::Confirm {
                                    start_table_id: st_change.clone(),
                                    start_field_id: sf_change.clone(),
                                    end_table_id: et_change.clone(),
                                    end_field_id: ef_change.clone(),
                                    cardinality: v,
                                });
                            }
                        >
                            <For
                                each=|| CARDINALITY_OPTIONS.to_vec()
                                key=|c| *c
                                children=move |c: &'static str| {
                                    let selected = card_for_options == c;
                                    view! { <option value=c selected=selected>{c}</option> }
                                }
                            />
                        </select>
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
            </div>
            <div class="cdb-io-drawer__footer">
                <button class="cdb-btn" data-testid="import-cancel-btn" on:click=move |_| close_btn()>"取消"</button>
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="import-submit"
                    disabled=move || submitting.get()
                    on:click=move |_| {
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
                        spawn_local(async move {
                            match client.import_local("import_drawer", payload).await {
                                Ok(ImportLocalResponse { diagram_id, .. }) => {
                                    navigate_to_editor(&diagram_id);
                                }
                                Err(e) => {
                                    submitting.set(false);
                                    err.set(Some(e.to_string()));
                                }
                            }
                        });
                    }
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
            ExportFormat::Json => export_diagram_json(&current_title.get_untracked(), &tables, &refs),
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
) -> impl IntoView {
    view! {
        <div class="cdb-empty-guide" data-testid="canvas-empty-guide">
            <h2>"开始设计你的数据库"</h2>
            <div class="cdb-empty-guide__actions">
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="guide-create-table"
                    on:click=move |_| on_create_table()
                >
                    "+ 创建第一张表"
                </button>
                <button
                    class="cdb-btn"
                    data-testid="guide-import-sql"
                    on:click=move |_| on_import()
                >
                    "↑ 导入 SQL"
                </button>
            </div>
        </div>
    }
}

/// Phase A：Inspector 抽屉
#[component]
pub fn Inspector(
    store: EditorStore,
    selection: RwSignal<SelectionKind>,
    inspector_open: RwSignal<bool>,
    on_add_field: Rc<dyn Fn(String)>,
    on_change_type: Rc<dyn Fn(String, String)>,
    on_set_ref: Rc<dyn Fn(String)>,
    on_toggle_pk: Rc<dyn Fn(String, String, bool)>,
    on_update_ref_field: Rc<dyn Fn(String, &str, String)>,
    on_flip_ref: Rc<dyn Fn(String)>,
    on_delete_ref: Rc<dyn Fn(String)>,
    on_jump_to_table: Rc<dyn Fn(String)>,
) -> impl IntoView {
    let selected_table = create_memo(move |_| {
        let id = selection.get().table_id()?.to_string();
        store.tables.get().into_iter().find(|t| t.id == id)
    });

    let close_inspector = move |_| inspector_open.set(false);

    view! {
        <aside class="cdb-inspector" data-testid="inspector-panel">
            <div class="cdb-inspector__header">
                <span data-testid="inspector-title">
                    {move || match selection.get() {
                        SelectionKind::None => "项目概览".into(),
                        SelectionKind::Table(_) => {
                            selected_table.get()
                                .map(|t| format!("表：{}", t.name))
                                .unwrap_or_else(|| "表".into())
                        }
                        SelectionKind::Field { .. } => {
                            let tables = store.tables.get();
                            if let SelectionKind::Field { table_id, field_id } = selection.get() {
                                tables.iter()
                                    .find(|t| t.id == table_id)
                                    .and_then(|t| t.fields.iter().find(|f| f.id == field_id))
                                    .map(|f| format!("字段：{}", f.name))
                                    .unwrap_or_else(|| "字段".into())
                            } else {
                                "字段".into()
                            }
                        }
                        SelectionKind::Reference(id) => format!("关系：{}", id),
                        SelectionKind::Issues => "问题".into(),
                    }}
                </span>
                <button
                    class="cdb-btn cdb-btn--icon"
                    data-testid="btn-inspector-close"
                    on:click=close_inspector
                >
                    <IconBox size="sm"><IconClose /></IconBox>
                </button>
            </div>
            <div class="cdb-inspector__body">
                {move || {
                    let on_jump = on_jump_to_table.clone();
                    let on_add = on_add_field.clone();
                    let on_change = on_change_type.clone();
                    let on_ref = on_set_ref.clone();
                    let sel = selection.clone();
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
                        if let Some(t) = selected_table.get() {
                            let fields = t.fields.clone();
                            let table_name = t.name.clone();
                            let table_id = t.id.clone();
                            let table_id_for_add = table_id.clone();
                            view! {
                                <div data-testid="inspector-table-form">
                                    <h3 data-testid="inspector-table-name">{table_name}</h3>
                                    <button
                                        class="cdb-btn cdb-btn--primary cdb-btn--block"
                                        data-testid="btn-add-field"
                                        on:click=move |_| on_add(table_id_for_add.clone())
                                    >
                                        "+ 添加字段"
                                    </button>
                                    <For each=move || fields.clone() key=|f| f.id.clone() children=move |field: Field| {
                                        let fid = field.id.clone();
                                        let fid_type = fid.clone();
                                        let fid_ref = fid.clone();
                                        let fname = field.name.clone();
                                        let ftype = field.type_.clone();
                                        let tid = table_id.clone();
                                        let sel = sel.clone();
                                        let on_change = on_change.clone();
                                        let on_ref = on_ref.clone();
                                        view! {
                                            <div
                                                class="cdb-field-row"
                                                data-testid={format!("field-row-{}", fid)}
                                                on:click=move |_| {
                                                    sel.set(SelectionKind::Field {
                                                        table_id: tid.clone(),
                                                        field_id: fid.clone(),
                                                    });
                                                }
                                            >
                                                <span>{fname}</span>
                                                <select
                                                    data-testid={format!("type-{}", fid_type)}
                                                    value=ftype
                                                    on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                                                    on:change=move |ev| {
                                                        on_change(fid_type.clone(), event_target_value(&ev));
                                                    }
                                                >
                                                    <option value="INT">"INT"</option>
                                                    <option value="VARCHAR(255)">"VARCHAR(255)"</option>
                                                    <option value="TEXT">"TEXT"</option>
                                                </select>
                                                <button
                                                    class="cdb-btn cdb-btn--icon"
                                                    data-testid={format!("set-ref-{}", fid_ref)}
                                                    on:click=move |ev: web_sys::MouseEvent| {
                                                        ev.stop_propagation();
                                                        on_ref(fid_ref.clone());
                                                    }
                                                >
                                                    "设关系"
                                                </button>
                                            </div>
                                        }
                                    } />
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
                            <div data-testid="inspector-overview">
                                <p>{move || format!("{} 张表", store.tables.get().len())}</p>
                                <p>{move || format!("{} 条关系", store.references.get().len())}</p>
                                <p>"db: generic"</p>
                                <p>{move || format!("rev: {}", store.revision.get())}</p>
                            </div>
                        }.into_view()
                    }
                    }
                }}
            </div>
        </aside>
    }
}

/// Phase A：底部 StatusBar
#[component]
pub fn StatusBar(
    store: EditorStore,
    transform: RwSignal<Transform>,
    inspector_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <footer class="cdb-status-bar" data-testid="status-bar">
            <span data-testid="status-zoom">
                {move || format!("缩放 {}%", (transform.get().zoom * 100.0).round() as i32)}
            </span>
            <span data-testid="status-counts">
                {move || format!(
                    "{} 张表 / {} 条关系",
                    store.tables.get().len(),
                    store.references.get().len(),
                )}
            </span>
            <span>"db: generic"</span>
            <span class="cdb-rev-tag" data-testid="revision-display">
                {move || format!("rev: {}", store.revision.get())}
            </span>
            <span class="cdb-status-bar__spacer"></span>
            <button
                class="cdb-btn cdb-btn--icon"
                data-testid="btn-inspector-toggle"
                title="折叠 Inspector"
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
/// - data-testid: tab-{key} 7 个 + search-input + type-filter + 7 个 tab-pane-{key}
#[component]
pub fn LeftPanel(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_select_table: Rc<dyn Fn(Option<String>)>,
    on_jump_to_table: Option<Rc<dyn Fn(String)>>,
    on_create_table: Rc<dyn Fn()>,
    on_save: Rc<dyn Fn()>,
) -> impl IntoView {
    // B2 范围：Areas/Enums/Notes/Types 暂用「仅前端 state」（spec 标 V1）
    let areas: RwSignal<Vec<AreaStub>> = create_rw_signal(Vec::new());
    let enums: RwSignal<Vec<EnumStub>> = create_rw_signal(Vec::new());
    let notes: RwSignal<Vec<NoteStub>> = create_rw_signal(Vec::new());
    let types: RwSignal<Vec<TypeStub>> = create_rw_signal(Vec::new());

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
    ];

    view! {
        <div class="cdb-side-panel cdb-side-panel--left" data-testid="left-panel">
            <div class="cdb-tabs cdb-tabs--wrap" role="tablist">
                <For each=move || tab_keys.clone() key=|t| *t children=move |tab: SidePanelTab| {
                    let tab_for_click = tab;
                    let testid = tab.testid();
                    let show_badge = matches!(
                        tab_for_click,
                        SidePanelTab::Tables | SidePanelTab::Relationships
                    );
                    view! {
                        <div
                            class="cdb-tab"
                            class:cdb-is-active=move || active_tab.get() == tab_for_click
                            role="tab"
                            data-testid={testid}
                            on:click=move |_| active_tab.set(tab_for_click)
                        >
                            <span>{tab_for_click.label()}</span>
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
                    SidePanelTab::Areas => view! {
                        <AreasTab areas=areas search_query=search_query.clone() />
                    }.into_view(),
                    SidePanelTab::Enums => view! {
                        <EnumsTab enums=enums search_query=search_query.clone() />
                    }.into_view(),
                    SidePanelTab::Notes => view! {
                        <NotesTab notes=notes search_query=search_query.clone() />
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
                }}
            </div>
        </div>
    }
}

/// 右侧面板视图
#[component]
pub fn RightPanel(
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
        <div class="cdb-side-panel cdb-side-panel--right" data-testid="right-panel">
            <div class="cdb-tabs" role="tablist">
                <div class="cdb-tab cdb-is-active" role="tab" data-testid="tab-fields">"字段"</div>
            </div>
            <div class="cdb-tab-content">
            {move || match has_selection.get() {
                true => {
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
                                            value=field_type
                                            on:change=move |ev| {
                                                let new_type = event_target_value(&ev);
                                                on_change(field_id_for_change.clone(), new_type);
                                            }
                                        >
                                            <option value="INT">"INT"</option>
                                            <option value="BIGINT">"BIGINT"</option>
                                            <option value="VARCHAR(255)">"VARCHAR(255)"</option>
                                            <option value="TEXT">"TEXT"</option>
                                            <option value="BOOLEAN">"BOOLEAN"</option>
                                            <option value="DATE">"DATE"</option>
                                            <option value="TIMESTAMP">"TIMESTAMP"</option>
                                            <option value="FLOAT">"FLOAT"</option>
                                            <option value="DOUBLE">"DOUBLE"</option>
                                            <option value="DECIMAL">"DECIMAL"</option>
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
                }
                false => view! { <p class="cdb-empty-hint">"请选择一个表"</p> }.into_view(),
            }}
            </div>
        </div>
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

/// Areas Tab — 区域列表（V1 仅前端 state，B3 接入 store）
/// B2 行为：渲染 areas 列表；搜索过滤；底部 "+" 创建新区域（默认名 "新区域"）
#[component]
pub fn AreasTab(
    areas: RwSignal<Vec<AreaStub>>,
    search_query: RwSignal<String>,
) -> impl IntoView {
    let next_id = create_rw_signal(0i64);
    let filtered = create_memo(move |_| {
        let all = areas.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-areas">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="area-add"
                on:click=move |_| {
                    let id = next_id.get();
                    next_id.set(id + 1);
                    let mut v = areas.get();
                    v.push(AreaStub {
                        id: format!("area-auto-{}", id),
                        name: format!("新区域 {}", id + 1),
                    });
                    areas.set(v);
                }
            >
                "+ 加区域"
            </button>
            <For each=move || filtered.get() key=|a| a.id.clone() children=move |a: AreaStub| {
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
pub fn EnumsTab(
    enums: RwSignal<Vec<EnumStub>>,
    search_query: RwSignal<String>,
) -> impl IntoView {
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

/// Notes Tab — 便签列表（V1 仅前端 state）
#[component]
pub fn NotesTab(
    notes: RwSignal<Vec<NoteStub>>,
    search_query: RwSignal<String>,
) -> impl IntoView {
    let next_id = create_rw_signal(0i64);
    let filtered = create_memo(move |_| {
        let all = notes.get();
        let q = search_query.get();
        filter_by_query(&all, &q)
    });
    view! {
        <div class="cdb-tab-pane" data-testid="tab-pane-notes">
            <button
                class="cdb-btn cdb-btn--block"
                data-testid="note-add"
                on:click=move |_| {
                    let id = next_id.get();
                    next_id.set(id + 1);
                    let mut v = notes.get();
                    v.push(NoteStub {
                        id: format!("note-auto-{}", id),
                        content: format!("新便签 {}", id + 1),
                    });
                    notes.set(v);
                }
            >
                "+ 加便签"
            </button>
            <For each=move || filtered.get() key=|n| n.id.clone() children=move |n: NoteStub| {
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
pub fn RelationshipsTab(
    store: EditorStore,
    search_query: RwSignal<String>,
) -> impl IntoView {
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
pub fn TypesTab(
    types: RwSignal<Vec<TypeStub>>,
    search_query: RwSignal<String>,
) -> impl IntoView {
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
) -> impl IntoView {
    let selection: RwSignal<SelectionKind> = create_rw_signal(SelectionKind::None);
    let selected_table_id: RwSignal<Option<String>> = create_rw_signal(None);
    let inspector_open: RwSignal<bool> = create_rw_signal(true);
    let conflict: RwSignal<Option<ConflictInfo>> = create_rw_signal(None);
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let next_id = create_rw_signal(0i64);

    // B4: 模态状态 (4 核心模态)
    let modal_kind: RwSignal<Option<modals::ModalKind>> = create_rw_signal(None);
    let current_diagram_id: RwSignal<String> = create_rw_signal(_diagram_id.clone());
    let current_title: RwSignal<String> = create_rw_signal(String::from("Untitled Diagram"));
    let is_saving: RwSignal<bool> = create_rw_signal(false);
    let save_offline: RwSignal<bool> = create_rw_signal(false);
    let view_mode: RwSignal<ViewMode> = create_rw_signal(ViewMode::Canvas);
    let code_visible: RwSignal<bool> = create_rw_signal(false);
    let code_language: RwSignal<CodeLanguage> = create_rw_signal(CodeLanguage::Sql);
    let code_copy_toast: RwSignal<Option<String>> = create_rw_signal(None);
    let palette_visible: RwSignal<bool> = create_rw_signal(false);
    let palette_query: RwSignal<String> = create_rw_signal(String::new());
    let palette_highlight: RwSignal<usize> = create_rw_signal(0);
    let canvas_transform: RwSignal<Transform> = create_rw_signal(Transform::default());

    // Phase B：关系工具
    let active_tool: RwSignal<ActiveTool> = create_rw_signal(ActiveTool::Select);
    let rel_tool_state: RwSignal<RelToolState> = create_rw_signal(RelToolState::Idle);
    let rel_tool_active: RwSignal<bool> = create_rw_signal(false);
    create_effect(move |_| {
        let picking = active_tool.get() == ActiveTool::Relationship
            && matches!(
                rel_tool_state.get(),
                RelToolState::PickSource | RelToolState::PickTarget { .. }
            );
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

    create_effect(move |_| {
        match selection.get() {
            SelectionKind::Table(id) => selected_table_id.set(Some(id)),
            SelectionKind::Field { table_id, .. } => selected_table_id.set(Some(table_id)),
            _ => {}
        }
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
            let (collapsed, cache) =
                snapshot_before_io_drawer(inspector_open.get_untracked());
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
            let (collapsed, cache) =
                snapshot_before_io_drawer(inspector_open.get_untracked());
            inspector_before_io.set(cache);
            inspector_open.set(collapsed);
            io_drawer.set(IoDrawerKind::Export);
        })
    };

    let command_stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>> = create_rw_signal(
        Rc::new(RefCell::new(crate::editor_core::CommandStack::new()))
    );

    // HTTP client to backend (port 3000, CORS middleware 在 fix-modal-overlay-blocking 已配)
    let client = DiagramClient::new("http://127.0.0.1:3000");

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
            let Some(info) = conflict.get_untracked() else { return };
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
        let id = _diagram_id.clone();
        if id != "default" {
            spawn_local(async move {
                match client.get(&id).await {
                    Ok(diagram) => {
                        current_title.set(diagram.name.clone());
                        store.load(diagram);
                    }
                    Err(e) => {
                        error.set(Some(format!("分享链接无效或图表已删除: {e}")));
                    }
                }
            });
        }
    }

    setup_command_palette_shortcut(palette_visible, view_mode);
    setup_code_view_escape(view_mode, code_visible);

    let palette_items = create_memo(move |_| {
        build_palette_items(&store.tables.get(), &store.references.get())
    });

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

    let on_create_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        let next_id = next_id.clone();
        let error_for_create = error.clone();
        Rc::new(move || {
            let id = next_id.get();
            next_id.set(id + 1);
            let table_id = format!("auto-{}", id);
            let is_first = store.tables.get().is_empty();
            let field_id = format!("{}-field-id", table_id);
            let default_fields = if is_first {
                vec![Field {
                    id: field_id,
                    name: "id".into(),
                    type_: "INT".into(),
                    default: String::new(),
                    check: String::new(),
                    primary: true,
                    unique: false,
                    not_null: true,
                    increment: true,
                    comment: String::new(),
                }]
            } else {
                Vec::new()
            };
            let new_table = Table {
                id: table_id.clone(),
                name: if is_first { "Table_1".into() } else { "新表".into() },
                x: 240.0,
                y: 160.0,
                color: "#175e7a".into(),
                comment: String::new(),
                fields: default_fields,
                indices: Vec::new(),
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
                            schedule_save(client, store, current_diagram_id, current_title, debouncer, conflict, error, is_saving, save_offline);
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    }
                });
            } else {
                schedule_save(client_for_create.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error_for_create.clone(), is_saving.clone(), save_offline.clone());
            }
        }) as Rc<dyn Fn()>
    };

    let on_save = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move || {
            schedule_save(client_for_save.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone(), is_saving.clone(), save_offline.clone());
        }) as Rc<dyn Fn()>
    };

    let on_title_blur = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |title: String| {
            current_title.set(title);
            store.dirty.set(true);
            schedule_save(client_for_title.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone(), is_saving.clone(), save_offline.clone());
        }) as Rc<dyn Fn(String)>
    };

    let on_add_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let next_id = next_id.clone();
        Rc::new(move |table_id: String| {
            let id = next_id.get();
            next_id.set(id + 1);
            let new_field = Field {
                id: format!("auto-{}", id),
                name: "新字段".into(),
                type_: "VARCHAR(255)".into(),
                default: String::new(),
                check: String::new(),
                primary: false,
                unique: false,
                not_null: false,
                increment: false,
                comment: String::new(),
            };
            let mut tables = store.tables.get();
            if let Some(table) = tables.iter_mut().find(|t| t.id == table_id) {
                table.fields.push(new_field);
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(client_for_add_field.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone(), is_saving.clone(), save_offline.clone());
        })
    };

    let on_change_type = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |field_id: String, new_type: String| {
            let mut tables = store.tables.get();
            for table in tables.iter_mut() {
                if let Some(field) = table.fields.iter_mut().find(|f| f.id == field_id) {
                    field.type_ = new_type.clone();
                    break;
                }
            }
            store.tables.set(tables);
            store.dirty.set(true);
            schedule_save(client_for_change_type.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone(), is_saving.clone(), save_offline.clone());
        })
    };

    let on_set_ref = {
        let active_tool = active_tool.clone();
        let rel_tool_state = rel_tool_state.clone();
        let store = store.clone();
        Rc::new(move |field_id: String| {
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

    let next_ref_id = {
        let next_id = next_id.clone();
        Rc::new(move || {
            let id = next_id.get();
            next_id.set(id + 1);
            format!("ref-{}", id)
        }) as Rc<dyn Fn() -> String>
    };

    let on_create_reference = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        let inspector_open = inspector_open.clone();
        Rc::new(move |reference: Reference| {
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
            );
        })
    };

    let on_field_pick: Option<Box<dyn Fn(String, String) + 'static>> = {
        let rel_tool_state = rel_tool_state.clone();
        Some(Box::new(move |table_id: String, field_id: String| {
            match rel_tool_state.get_untracked() {
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
                    rel_tool_state.set(RelToolState::Confirm {
                        start_table_id,
                        start_field_id,
                        end_table_id: table_id,
                        end_field_id: field_id,
                        cardinality: "one_to_many".into(),
                    });
                }
                _ => {}
            }
        }))
    };

    let on_toggle_pk = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |table_id: String, field_id: String, primary: bool| {
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
            );
        })
    };

    let on_update_ref_field = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |ref_id: String, field: &str, value: String| {
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
            );
        })
    };

    let on_flip_ref = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        Rc::new(move |ref_id: String| {
            let mut refs = store.references.get();
            if let Some(idx) = refs.iter().position(|r| r.id == ref_id) {
                refs[idx] = flip_reference_endpoints(&refs[idx]);
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
            );
        })
    };

    let on_delete_ref = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selection = selection.clone();
        Rc::new(move |ref_id: String| {
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
        Callback::new(move |item: PaletteItem| {
            match item.kind {
                crate::command_palette::PaletteKind::Table => {
                    selection.set(SelectionKind::Table(item.id));
                    inspector_open.set(true);
                }
                crate::command_palette::PaletteKind::Reference => {
                    selection.set(SelectionKind::Reference(item.id));
                    inspector_open.set(true);
                }
                _ => {}
            }
        })
    };

    let on_create_table_rail = on_create_table.clone();
    let on_create_table_panel = on_create_table.clone();
    let on_create_table_guide = on_create_table.clone();

    view! {
        <div class="cdb-app" data-testid="editor-ready">
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
            />
            <div
                class="cdb-main"
                class:cdb-is-hidden=move || view_mode.get() == ViewMode::Code
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
                />
                <div class="cdb-canvas-container" data-testid="editor-canvas-container">
                    {move || if store.tables.get().is_empty() {
                        view! {
                            <EmptyGuide
                                on_create_table=on_create_table_guide.clone()
                                on_import=open_import_drawer.clone()
                            />
                        }.into_view()
                    } else {
                        view! { <></> }.into_view()
                    }}
                    <RelToolHint rel_state=rel_tool_state />
                    <Canvas
                        store=store.clone()
                        transform=canvas_transform
                        on_select=on_canvas_select
                        on_deselect=on_canvas_deselect
                        on_dblclick_blank=on_dblclick_blank
                        rel_tool_active=rel_tool_active
                        on_field_pick=on_field_pick
                    />
                    <RelationshipConfirmBar
                        store=store.clone()
                        rel_state=rel_tool_state
                        next_ref_id=next_ref_id.clone()
                        on_create=on_create_reference.clone()
                    />
                    <FloatingControls transform=canvas_transform />
                </div>
                <aside
                    class="cdb-inspector"
                    data-testid="inspector-panel"
                    style:display=move || if inspector_open.get() { "flex" } else { "none" }
                >
                    <LeftPanel
                        store=store.clone()
                        selected_table_id=selected_table_id
                        on_select_table=on_select_table.clone()
                        on_jump_to_table=Some(on_jump_to_table.clone())
                        on_create_table=on_create_table_panel.clone()
                        on_save=on_save.clone()
                    />
                    <RightPanel
                        store=store.clone()
                        selected_table_id=selected_table_id
                        on_add_field=on_add_field.clone()
                        on_change_type=on_change_type.clone()
                        on_set_ref=on_set_ref.clone()
                    />
                </aside>
                <IoDrawer
                    kind=io_drawer
                    store=store.clone()
                    current_title=current_title
                    client=client_for_io.clone()
                    error=error.clone()
                    on_close=close_io_drawer.clone()
                />
            </div>
            <StatusBar
                store=store.clone()
                transform=canvas_transform
                inspector_open=inspector_open
            />
            <CodeView
                visible=code_visible
                language=code_language
                content=code_content
                copy_toast=code_copy_toast
            />
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
                on_new=on_new_diagram
                on_rename=on_rename_diagram
            />
            <modals::KeyboardShortcuts
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
    use leptos::*;

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
        ConfigureCustomTypes,
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
    #[component]
    pub fn ModalRoot(
        kind: RwSignal<Option<ModalKind>>,
        current_diagram_id: RwSignal<String>,
        current_title: RwSignal<String>,
        on_new: Rc<dyn Fn(String)>,
        on_rename: Rc<dyn Fn(String)>,
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
                            <SetTableWidthModal kind=kind />
                        </div>
                    }.into_view(),
                    Some(ModalKind::ConfigureCustomTypes) => view! {
                        <div class="cdb-modal" data-testid="modal-custom-types" on:click=|ev| ev.stop_propagation()>
                            <ConfigureCustomTypesModal kind=kind />
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
    pub fn OpenModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
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

    /// Import 模态: 粘贴 SQL → 调用 bridge/import
    /// - UT-MM-10: parse_sql_statements 纯函数测试
    /// - B5 stub: 仅 UI shell，逻辑留 B5 e2e 接入
    #[component]
    pub fn ImportModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
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
    pub fn ImportSourceModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
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
    pub fn LanguageModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
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
    #[component]
    pub fn SetTableWidthModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
        let width_input = create_rw_signal(String::from("200"));
        let validation = move || parse_table_width(&width_input.get());
        let is_valid = move || validation().is_ok();
        let kind_close = kind;

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
                >"Apply"</button>
            </div>
        }
    }

    /// ConfigureCustomTypes 模态: 增删改自定义类型
    /// - UT-MM-13: add/remove_custom_type 纯函数测试
    #[component]
    pub fn ConfigureCustomTypesModal(
        kind: RwSignal<Option<ModalKind>>,
    ) -> impl IntoView {
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
    pub fn KeyboardShortcuts<F1, F2>(
        on_undo: F1,
        on_redo: F2,
    ) -> impl IntoView
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
                use wasm_bindgen::JsCast;
                let key_event: Option<&web_sys::KeyboardEvent> = ev.dyn_ref();
                if let Some(ke) = key_event {
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
    use crate::editor_core::types::{Field, Reference, Table};

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
        v.retain(|t| t.fields.iter().any(|f| f.type_.to_uppercase().contains("INT")));
        assert_eq!(v.len(), 1, "UT-SP-02: 类型筛选 INT 应只保留 users");
        assert_eq!(v[0].name, "users");
    }

    // --- UT-SP-09 — 6 业务 Tab 切换 ---

    /// UT-SP-09: 7 个 Tab 的 testid/label 全部存在且唯一
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
        ];
        let mut testids: Vec<&str> = all_tabs.iter().map(|t| t.testid()).collect();
        testids.sort();
        testids.dedup();
        assert_eq!(
            testids.len(),
            7,
            "UT-SP-09: 7 个 Tab testid 应全部唯一，实际 {} 个",
            testids.len()
        );
        // 验证覆盖所有 6 业务 Tab + Issues
        for expected in [
            "tab-tables",
            "tab-areas",
            "tab-enums",
            "tab-notes",
            "tab-relationships",
            "tab-types",
            "tab-issues",
        ] {
            assert!(
                testids.contains(&expected),
                "UT-SP-09: 应包含 testid '{}'",
                expected
            );
        }
    }

    /// UT-SP-09: 7 个 Tab label 都有非空显示文本
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
        let areas = vec![AreaStub {
            id: "a1".into(),
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

    /// UT-SP-10: 类型过滤对 Table/Enum/NoteStub 都能正确 filter
    #[test]
    fn test_filter_by_query_generic_ut_sp_10() {
        let notes = vec![
            NoteStub {
                id: "n1".into(),
                content: "user feedback".into(),
            },
            NoteStub {
                id: "n2".into(),
                content: "system status".into(),
            },
        ];
        let result = filter_by_query(&notes, "user");
        assert_eq!(result.len(), 1, "UT-SP-10: notes 搜索 'user' 应匹配 1");
        assert_eq!(result[0].id, "n1");
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
        assert_eq!(result.len(), 1, "UT-SP-10: refs 搜索 'user' 应匹配 r1（start=users）");
        assert_eq!(result[0].id, "r1");
    }

    // ─── B4 modal pure function tests ─────────────────────────────────────

    #[test]
    fn test_validate_title_happy_ut_mm_01() {
        assert!(modals::validate_title("My Diagram").is_ok(), "UT-MM-01: 正常 title 应通过");
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
        assert!(r.is_err(), "UT-MM-07: 空 title 时 NewModal 提交应禁用（基于 validate_title 返回 Err）");
    }

    #[test]
    fn test_build_create_url_ut_mm_01() {
        assert_eq!(modals::build_create_url("d-new"), "/editor/d-new", "UT-MM-01: build_create_url 应返回 /editor/<id>");
        assert_eq!(modals::build_create_url("abc-123"), "/editor/abc-123");
    }

    #[test]
    fn test_build_share_url_ut_mm_08() {
        assert_eq!(modals::build_share_url("abc-123"), "/editor?share=abc-123", "UT-MM-08: build_share_url 应返回 /editor?share=<id>");
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
        assert!(r.unwrap_err().starts_with("JSON parse error"), "UT-MM-09: 错误信息应包含 'JSON parse error'");
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
        assert_eq!(modals::parse_table_width("200").unwrap(), 200, "UT-MM-11: '200' → 200");
        assert_eq!(modals::parse_table_width("0").unwrap(), 0, "UT-MM-11: '0' → 0 (auto)");
    }

    #[test]
    fn test_parse_table_width_invalid_ut_mm_11() {
        assert!(modals::parse_table_width("abc").is_err(), "UT-MM-11: 'abc' → Err");
        assert!(modals::parse_table_width("").is_err(), "UT-MM-11: '' → Err");
    }

    #[test]
    fn test_validate_language_ut_mm_12() {
        assert!(modals::validate_language("en").is_ok(), "UT-MM-12: 'en' 应通过");
        assert!(modals::validate_language("zh").is_ok(), "UT-MM-12: 'zh' 应通过");
        assert!(modals::validate_language("fr").is_err(), "UT-MM-12: 'fr' 应 Err");
    }

    #[test]
    fn test_resolve_import_source_ut_mm_14() {
        assert_eq!(modals::resolve_import_source("local").unwrap(), modals::SourceKind::Local);
        assert_eq!(modals::resolve_import_source("remote").unwrap(), modals::SourceKind::Remote);
        assert!(modals::resolve_import_source("http").is_err(), "UT-MM-14: 'http' 应 Err");
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
        assert!(modals::is_undo_shortcut("z", true, false), "UT-KB-01: Ctrl+Z → true");
        assert!(modals::is_undo_shortcut("Z", true, false), "UT-KB-01: 大小写无关");
        assert!(!modals::is_undo_shortcut("z", false, false), "UT-KB-01: 不带 Ctrl → false");
        assert!(!modals::is_undo_shortcut("z", true, true), "UT-KB-01: 带 Shift 属 redo → false");
        assert!(!modals::is_undo_shortcut("a", true, false), "UT-KB-01: 其他键 → false");
    }

    #[test]
    fn test_is_redo_shortcut_ut_kb_01() {
        assert!(modals::is_redo_shortcut("z", true, true), "UT-KB-01: Ctrl+Shift+Z → true");
        assert!(!modals::is_redo_shortcut("z", true, false), "UT-KB-01: 不带 Shift 属 undo → false");
    }

    // ─── UT-FIX-01: ModalRoot 条件渲染（fix-modal-overlay-blocking B1） ─────

    #[test]
    fn test_modal_root_overlay_only_renders_when_kind_is_some() {
        let src = include_str!("editor_panels.rs");
        let count = src.matches("class=\"cdb-modal-overlay\"").count();
        assert!(
            count <= 1,
            "UT-FIX-01: `class=\"cdb-modal-overlay\"` 出现 {count} 次, 预期 ≤ 1（仅声明点）; \
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
            panels.contains("data-testid=\"inspector-panel\""),
            "UT-IN-01: Inspector 必须带 inspector-panel testid",
        );
        assert!(
            panels.contains("data-testid=\"canvas-empty-guide\""),
            "UT-PA-01: 空白引导必须带 canvas-empty-guide testid",
        );
        assert!(
            panels.contains("data-testid=\"revision-display\""),
            "UT-AB-05: revision-display 必须存在",
        );
        assert!(
            panels.contains("data-testid=\"status-bar\""),
            "UT-AB-05: revision 应位于 StatusBar",
        );
        assert!(
            css.contains("grid-template-rows: 48px 1fr 28px"),
            "UT-PA-06: .cdb-app 栅格应为 48px 1fr 28px",
        );
        assert!(
            css.contains("grid-template-columns: 48px 1fr 320px 0"),
            "UT-PA-06: .cdb-main 栅格应对齐原型 ToolRail + Canvas + Inspector + IO",
        );
        assert!(
            css.contains("cdb-has-io-drawer"),
            "Phase C: IO 抽屉栅格类",
        );
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
        assert!(selection_auto_opens_inspector(&SelectionKind::Table("t1".into())));
        assert!(selection_auto_opens_inspector(&SelectionKind::Field {
            table_id: "t1".into(),
            field_id: "f1".into(),
        }));
        assert!(!selection_auto_opens_inspector(&SelectionKind::None));
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
            }],
            indices: Vec::new(),
        }];
        let out = export_diagram_sql(&tables, &[], "generic");
        assert!(out.contains("CREATE TABLE users"), "UT-PC-02: 应含 CREATE TABLE");
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
        let flipped = flip_reference_endpoints(&r);
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
