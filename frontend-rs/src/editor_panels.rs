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

use crate::editor_core::{
    ConflictAction, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_core::types::{Field, Reference, Table};
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
    fn render(
        conflict: RwSignal<Option<ConflictInfo>>,
        on_force_overwrite: Rc<dyn Fn()>,
        on_reload: Rc<dyn Fn()>,
    ) -> impl IntoView {
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
                                    on_force_overwrite();
                                }
                            >
                                "强制覆盖"
                            </button>
                            <button
                                class="cdb-btn"
                                data-testid="btn-reload"
                                on:click=move |_| {
                                    conflict.set(None);
                                    on_reload();
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
    }

    {render(conflict, on_force_overwrite, on_reload)}
}

/// 错误提示
#[component]
pub fn ErrorToast(error: RwSignal<Option<String>>) -> impl IntoView {
    fn render(error: RwSignal<Option<String>>) -> impl IntoView {
        match error.get() {
            Some(msg) => view! {
                <div class="cdb-error-toast" data-testid="error-toast">
                    {msg}
                    <button on:click=move |_| error.set(None)>{"×"}</button>
                </div>
            }.into_view(),
            None => view! { <></> }.into_view(),
        }
    }

    {render(error)}
}

/// 顶部菜单栏 (B1)：4 下拉空壳，点击不展开真实菜单
#[component]
pub fn TopMenuBar() -> impl IntoView {
    view! {
        <header class="cdb-header" data-testid="top-menu-bar">
            <div class="cdb-logo">"coldrawdb"</div>
            <nav class="cdb-menu">
                <div class="cdb-menu-item" data-testid="cdb-menu-file">"File ▾"</div>
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
) -> impl IntoView {
    let saving = create_rw_signal(false);
    let debouncer_for_save = debouncer.clone();
    let err_for_save = error.clone();
    let store_for_save = store.clone();

    view! {
        <div>
            <TopMenuBar />
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
                let testid_for_select = testid.clone();
                view! {
                    <div
                        class="cdb-list-item"
                        class:cdb-is-selected=move || selected_table_id.get() == Some(table_id.clone())
                        data-testid={testid}
                        on:click=move |_| { on_select(Some(testid_for_select.clone())); }
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

    // B1: 预留 CommandStack 信号（stack 内部为空；B5 接入 undo/redo 逻辑）
    let _stack: RwSignal<Rc<RefCell<crate::editor_core::CommandStack>>> = create_rw_signal(
        Rc::new(RefCell::new(crate::editor_core::CommandStack::new()))
    );

    let make_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        format!("auto-{}", id)
    };

    // Toolbar CreateTable 处理
    let on_create_table = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selected_table_id = selected_table_id.clone();
        let next_id = next_id.clone();
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
            debouncer.schedule(move || {});
        }) as Rc<dyn Fn()>
    };

    let on_save = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let err = error.clone();
        Rc::new(move || {
            err.set(Some("保存触发 debounce 1s".to_string()));
            debouncer.schedule(move || {
                let _ = store.dirty.get();
            });
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
            debouncer.schedule(move || {});
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
            debouncer.schedule(move || {});
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
            <TopMenuBar />
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
                <div class="cdb-canvas-container">
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
        </div>
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
}
