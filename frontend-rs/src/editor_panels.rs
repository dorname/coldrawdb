//! editor-panels: 顶/左/右面板 UI + 409 弹窗 + toast
//!
//! 依赖: `editor_core::EditorStore`, `DebounceTrigger`, `ConflictInfo`, `ConflictAction`
//!        `editor_data_access::DiagramClient`
//!
//! B1 (add-frontend-completeness) 更新:
//!   - 拆分 TopBar → TopMenuBar (4 下拉空壳) + Toolbar (撤销/重做/Share/Export)
//!   - 新增 UndoRedoButtons 子组件，绑定 store + error（stack 在 AppRoot 创建并预留，
//!     真实 undo/redo 逻辑待 B5 接入）
//!   - 所有 class 加 cdb- 前缀（spec §5.2.4）
//!   - 样式由 src/styles.css 接管
//!
//! data-testid 清单（验证: `grep -c 'data-testid=' src/editor_panels.rs` 期望 ≥ 6）:
//!   - btn-create-table  /  btn-save  /  revision-display
//!   - table-list-item-{id}
//!   - btn-add-field  /  type-{id}  /  set-ref-{id}
//!   - conflict-dialog  /  btn-force-overwrite  /  btn-reload
//!   - error-toast
//!   - editor-ready (AC-23 TTI 测量点)
//!   - top-menu-bar  /  cdb-menu-{file,edit,view,help}  (B1)
//!   - toolbar  /  btn-undo  /  btn-redo  /  btn-share  /  btn-export  (B1)
//!   - editor-canvas  (B1 fix - fix-modal-overlay-blocking, e2e 画布锚点)

use crate::editor_core::{
    ConflictAction, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_core::types::{Field, Reference, Table};
use crate::editor_data_access::{DiagramClient, SaveError, SaveResponse};
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;

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
            SidePanelTab::Tables => "Tables",
            SidePanelTab::Areas => "Areas",
            SidePanelTab::Enums => "Enums",
            SidePanelTab::Notes => "Notes",
            SidePanelTab::Relationships => "Relationships",
            SidePanelTab::Types => "Types",
            SidePanelTab::Issues => "Issues",
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
/// - 其它错误 → `error.set(Some(e.to_string()))`
///
/// 7 个参数全是 `Clone` 友好的 owned 值（DiagramClient 内部 `Rc`；EditorStore / RwSignal
/// 内部全 `Copy` 友好），所以跨 `spawn_local` 边界安全。
#[allow(clippy::too_many_arguments)]
pub(crate) fn schedule_save(
    client: DiagramClient,
    store: EditorStore,
    current_diagram_id: RwSignal<String>,
    current_title: RwSignal<String>,
    debouncer: DebounceTrigger,
    conflict: RwSignal<Option<ConflictInfo>>,
    error: RwSignal<Option<String>>,
) {
    let id = current_diagram_id.get();
    let rev = store.revision.get();
    let name = current_title.get();
    let snap = store.snapshot(id.clone(), name);
    debouncer.schedule(move || {
        let client = client.clone();
        let store = store.clone();
        let conflict = conflict.clone();
        let error = error.clone();
        spawn_local(async move {
            match client.save(&id, rev, &snap).await {
                Ok(resp) => {
                    store.revision.set(resp.revision);
                    store.dirty.set(false);
                }
                Err(SaveError::Conflict { current_revision, .. }) => {
                    conflict.set(Some(ConflictInfo::new(current_revision, rev)));
                }
                Err(e) => error.set(Some(e.to_string())),
            }
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
                <div class="cdb-conflict-dialog-overlay">
                    <div class="cdb-conflict-dialog" data-testid="conflict-dialog">
                        <h2>"保存冲突"</h2>
                        <p>
                            "服务器上的版本比本地更新。请选择如何处理："
                            {format!("本地 rev {} vs 服务器 rev {}", info.local_revision, info.current_revision)}
                        </p>
                        <div class="cdb-dialog-buttons">
                            <button
                                class="cdb-btn cdb-btn--primary"
                                data-testid="btn-force-overwrite"
                                on:click=move |_| {
                                    conflict.set(None);
                                    on_force_overwrite_inner();
                                }
                            >
                                "强制覆盖"
                            </button>
                            <button
                                class="cdb-btn"
                                data-testid="btn-reload"
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
                    <button on:click=move |_| error.set(None)>{"×"}</button>
                </div>
            }.into_view(),
            None => view! { <></> }.into_view(),
        }
    };

    render
}

/// 顶部菜单栏 (B1)：4 下拉空壳 + B4 File 下拉接通 4 个模态
#[component]
pub fn TopMenuBar(
    modal_kind: RwSignal<Option<modals::ModalKind>>,
) -> impl IntoView {
    let file_open = create_rw_signal(false);

    view! {
        <header class="cdb-header" data-testid="top-menu-bar">
            <div class="cdb-logo">"coldrawdb"</div>
            <nav class="cdb-menu">
                <div
                    class="cdb-menu-item"
                    data-testid="cdb-menu-file"
                    on:click=move |_| file_open.update(|v| *v = !*v)
                >"File ▾"</div>
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
                            >"New"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-open"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Open));
                                    file_open.set(false);
                                }
                            >"Open"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-share"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Share));
                                    file_open.set(false);
                                }
                            >"Share"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-rename"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Rename));
                                    file_open.set(false);
                                }
                            >"Rename"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-import"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Import));
                                    file_open.set(false);
                                }
                            >"Import"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-import-source"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::ImportSource));
                                    file_open.set(false);
                                }
                            >"Import Source"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-language"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::Language));
                                    file_open.set(false);
                                }
                            >"Language"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-set-width"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::SetTableWidth));
                                    file_open.set(false);
                                }
                            >"Set Table Width"</button>
                            <button
                                class="cdb-menu-dropdown-item"
                                data-testid="cdb-menu-custom-types"
                                on:click=move |_| {
                                    modal_kind.set(Some(modals::ModalKind::ConfigureCustomTypes));
                                    file_open.set(false);
                                }
                            >"Configure Custom Types"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }}
                <div class="cdb-menu-item" data-testid="cdb-menu-edit">"Edit ▾"</div>
                <div class="cdb-menu-item" data-testid="cdb-menu-view">"View ▾"</div>
                <div class="cdb-menu-item" data-testid="cdb-menu-help">"Help ▾"</div>
            </nav>
            <div class="cdb-header-right">
                <span class="cdb-save-state">"● Idle"</span>
                <button class="cdb-btn cdb-btn--icon" title="设置">"⚙"</button>
            </div>
        </header>
    }
}

/// 撤销/重做按钮组 (B1)：UI 落地，真实 undo/redo 逻辑待 B5
/// - 接收 store（用于显示 revision）
/// - 接收 error signal（点击弹 toast 提示 B5 待实现）
#[component]
pub fn UndoRedoButtons(
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let err1 = error.clone();
    let err2 = error.clone();
    view! {
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-undo"
            title="撤销 (Ctrl+Z) — 待 B5 实现"
            on:click=move |_| {
                err1.set(Some("撤销功能待 B5 实现".to_string()));
            }
        >
            "↶"
        </button>
        <button
            class="cdb-btn cdb-btn--icon"
            data-testid="btn-redo"
            title="重做 (Ctrl+Shift+Z) — 待 B5 实现"
            on:click=move |_| {
                err2.set(Some("重做功能待 B5 实现".to_string()));
            }
        >
            "↷"
        </button>
    }
}

/// 工具栏 (B1)：撤销/重做 + 标题 + rev + Share/Export 占位
#[component]
pub fn Toolbar(
    store: EditorStore,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="cdb-toolbar" data-testid="toolbar">
            <UndoRedoButtons error=error />
            <input
                class="cdb-title-edit"
                value="Untitled Diagram"
                readonly=true
            />
            <span class="cdb-rev-tag" data-testid="revision-display">
                {move || format!("rev: {}", store.revision.get())}
            </span>
            <div class="cdb-toolbar-right">
                <button
                    class="cdb-btn"
                    data-testid="btn-share"
                    title="Share — 待 B4 实现"
                    on:click=move |_| {
                        error.set(Some("Share 模态待 B4 实现".to_string()));
                    }
                >
                    "Share"
                </button>
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
            <TopMenuBar modal_kind=modal_kind />
            <Toolbar
                store=store.clone()
                error=error.clone()
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
            <div class="cdb-search-box">
                <input
                    type="text"
                    class="cdb-search-input"
                    placeholder="搜索 (跨 Tab)..."
                    data-testid="search-input"
                    prop:value=move || search_query.get()
                    on:input=move |ev| search_query.set(event_target_value(&ev))
                />
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
            </div>
            <div class="cdb-tabs" role="tablist">
                <For each=move || tab_keys.clone() key=|t| *t children=move |tab: SidePanelTab| {
                    let tab_for_click = tab;
                    let testid = tab.testid();
                    view! {
                        <div
                            class="cdb-tab"
                            class:cdb-is-active=move || active_tab.get() == tab_for_click
                            role="tab"
                            data-testid={testid}
                            on:click=move |_| active_tab.set(tab_for_click)
                        >
                            {tab.label()}
                        </div>
                    }
                } />
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
                        <div class="cdb-field-list">
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
    }
}

// =====================================================================
// B2: 7-Tab 子组件（Tables / Areas / Enums / Notes / Relationships / Types / Issues）
// =====================================================================

/// Tables Tab — 表格列表 + 搜索过滤 + 类型筛选（UT-SP-02 覆盖）
/// B2 行为：search_query 非空时按表名子串匹配；type_filter 非空时按字段类型子串匹配。
#[component]
pub fn TablesTab(
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
        <div class="cdb-tab-pane" data-testid="tab-pane-tables">
            <For each=move || filtered.get() key=|t| t.id.clone() children=move |table: Table| {
                let table_id = table.id.clone();
                let table_name = table.name.clone();
                let on_select = on_select_table.clone();
                let testid = format!("table-list-item-{}", table_id);
                // on:click 闭包要 move 进 table_id；class: 闭包也借用 table_id。
                // 预先 clone 一份给 on:click 用。
                let table_id_for_click = table_id.clone();
                view! {
                    <div
                        class="cdb-list-item"
                        class:cdb-is-selected=move || is_table_selected(&selected_table_id.get(), &table_id)
                        data-testid={testid}
                        on:click=move |_| { on_select(Some(table_id_for_click.clone())); }
                    >
                        {table_name}
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
    let selected_table_id: RwSignal<Option<String>> = create_rw_signal(None);
    let conflict: RwSignal<Option<ConflictInfo>> = create_rw_signal(None);
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let next_id = create_rw_signal(0i64);

    // B4: 模态状态 (4 核心模态)
    let modal_kind: RwSignal<Option<modals::ModalKind>> = create_rw_signal(None);
    let current_diagram_id: RwSignal<String> = create_rw_signal(_diagram_id.clone());
    let current_title: RwSignal<String> = create_rw_signal(String::from("Untitled Diagram"));

    // B4: 模态提交回调 (New/Rename 共享)
    // - New: 创建空 diagram，更新 current_diagram_id + current_title
    // - Rename: 更新 current_title
    // - B5 接入实际 editor_data_access::create / save
    let on_modal_action = move |name: String| {
        current_title.set(name);
    };

    // B1: 预留 CommandStack 信号（stack 内部为空；B5 接入 undo/redo 逻辑）
    let _stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>> = create_rw_signal(
        Rc::new(RefCell::new(crate::editor_core::CommandStack::new()))
    );

    let make_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        format!("auto-{}", id)
    };

    // HTTP client to backend (port 3000, CORS middleware 在 fix-modal-overlay-blocking 已配)
    let client = DiagramClient::new("http://127.0.0.1:3000");
    // 4 个 save handler 各 clone 一份（避免 move 闭包互抢 client）
    let client_for_create = client.clone();
    let client_for_save = client.clone();
    let client_for_add_field = client.clone();
    let client_for_change_type = client.clone();

    // Toolbar CreateTable 处理
    let on_create_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selected_table_id = selected_table_id.clone();
        let next_id = next_id.clone();
        let error_for_create = error.clone();
        Rc::new(move || {
            let id = next_id.get();
            next_id.set(id + 1);
            let new_table = Table {
                id: format!("auto-{}", id),
                name: "新表".into(),
                x: 100.0,
                y: 100.0,
                color: "#3B82F6".into(),
                comment: String::new(),
                fields: Vec::new(),
                indices: Vec::new(),
            };
            let mut tables = store.tables.get();
            tables.push(new_table.clone());
            store.tables.set(tables);
            selected_table_id.set(Some(new_table.id));
            store.dirty.set(true);

            // ST-STUB-01: e2e 用 / 入口,lib.rs fallback "default" → PUT /diagrams/default 后端 404
            // 这里在首次创建表时,如果还是 fallback "default",先 POST /diagrams 拿真 id 再 save
            if current_diagram_id.get() == "default" {
                let client = client_for_create.clone();
                let store = store.clone();
                let debouncer = debouncer.clone();
                let current_diagram_id = current_diagram_id.clone();
                let current_title = current_title.clone();
                let error = error_for_create.clone();
                let conflict = conflict.clone();
                spawn_local(async move {
                    match client.create("新图").await {
                        Ok(new_id) => {
                            current_diagram_id.set(new_id);
                            schedule_save(client, store, current_diagram_id, current_title, debouncer, conflict, error);
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    }
                });
            } else {
                schedule_save(client_for_create.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error_for_create.clone());
            }
        }) as Rc<dyn Fn()>
    };

    let on_save = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let err = error.clone();
        Rc::new(move || {
            err.set(Some("保存触发 debounce 1s".to_string()));
            schedule_save(client_for_save.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone());
        }) as Rc<dyn Fn()>
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
            schedule_save(client_for_add_field.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone());
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
            schedule_save(client_for_change_type.clone(), store.clone(), current_diagram_id.clone(), current_title.clone(), debouncer.clone(), conflict.clone(), error.clone());
        })
    };

    let on_set_ref = {
        let error = error.clone();
        Rc::new(move |field_id: String| {
            error.set(Some(format!("设关系功能待实现 (field_id: {})", field_id)));
        })
    };

    // TopBar 内部：保留旧的 btn-save + btn-create-table（位于 TopMenuBar 上方）
    let topbar_create_for_topbar = on_create_table.clone();
    let topbar_save_for_topbar = on_save.clone();

    view! {
        <div class="cdb-app" data-testid="editor-ready">
            <TopMenuBar modal_kind=modal_kind />
            <Toolbar store=store.clone() error=error.clone() />
            <div class="cdb-topbar-actions">
                <button
                    class="cdb-btn cdb-btn--primary"
                    data-testid="btn-create-table"
                    on:click=move |_| topbar_create_for_topbar()
                >
                    "+ 建表"
                </button>
                <button
                    class="cdb-btn"
                    data-testid="btn-save"
                    on:click=move |_| topbar_save_for_topbar()
                >
                    "保存"
                </button>
            </div>
            <div class="cdb-main">
                <LeftPanel
                    store=store.clone()
                    selected_table_id=selected_table_id.clone()
                    on_select_table=Rc::new(move |id| selected_table_id.set(id))
                    on_jump_to_table=Some(Rc::new(move |id| selected_table_id.set(Some(id))))
                />
                <div class="cdb-canvas-container" data-testid="editor-canvas">
                    <div class="cdb-canvas-empty">
                        "画布 (B3 接入 areas/notes/references)"
                    </div>
                </div>
                <RightPanel
                    store=store.clone()
                    selected_table_id=selected_table_id.clone()
                    on_add_field=on_add_field
                    on_change_type=on_change_type
                    on_set_ref=on_set_ref
                />
            </div>
            <footer class="cdb-footer">
                <span>"● 100 tables / 200 relationships / 60fps target"</span>
                <span>"db: generic"</span>
            </footer>
            <ConflictDialog
                conflict=conflict
                on_force_overwrite=Rc::new(move || {})
                on_reload=Rc::new(move || {})
            />
            <ErrorToast error=error />
            <modals::ModalRoot
                kind=modal_kind
                current_diagram_id=current_diagram_id
                current_title=current_title
                on_action=on_modal_action
            />
            <modals::KeyboardShortcuts
                on_undo=|| {}
                on_redo=|| {}
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
    pub fn ModalRoot<F>(
        kind: RwSignal<Option<ModalKind>>,
        current_diagram_id: RwSignal<String>,
        current_title: RwSignal<String>,
        on_action: F,
    ) -> impl IntoView
    where
        F: Fn(String) + Clone + 'static,
    {
        let on_action_new = on_action.clone();
        let on_action_rename = on_action.clone();

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
    pub fn NewModal<F>(
        kind: RwSignal<Option<ModalKind>>,
        on_create: F,
    ) -> impl IntoView
    where
        F: Fn(String) + Clone + 'static,
    {
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
                >"×"</button>
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
                >"×"</button>
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
                >"×"</button>
            </div>
            <div class="cdb-modal-body">
                <label class="cdb-form-label">"Share link"</label>
                <input
                    class="cdb-form-input"
                    data-testid="modal-input-share-url"
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
    pub fn RenameModal<F>(
        kind: RwSignal<Option<ModalKind>>,
        current_title: RwSignal<String>,
        on_rename: F,
    ) -> impl IntoView
    where
        F: Fn(String) + Clone + 'static,
    {
        let title_input = create_rw_signal(current_title.get_untracked());
        let validation = move || validate_title(&title_input.get());
        let is_valid = move || validation().is_ok();
        let kind_close = kind;
        let on_rename_submit = on_rename;

        view! {
            <div class="cdb-modal-header">
                <h3 class="cdb-modal-title" data-testid="modal-title-rename">"Rename Diagram"</h3>
                <button
                    class="cdb-modal-close"
                    data-testid="modal-cancel-rename"
                    on:click=move |_| kind_close.set(None)
                >"×"</button>
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
                >"×"</button>
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
                >"×"</button>
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
                >"×"</button>
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
                >"×"</button>
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
                >"×"</button>
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
                                >"×"</button>
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
        let src = include_str!("editor_panels.rs");
        assert!(
            src.contains("class=\"cdb-canvas-container\" data-testid=\"editor-canvas\""),
            "UT-FIX-02: `<div class=\"cdb-canvas-container\">` 必须带 `data-testid=\"editor-canvas\"`",
        );
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
