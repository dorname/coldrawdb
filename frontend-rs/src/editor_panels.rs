//! editor-panels: 顶/左/右面板 UI + 409 弹窗 + toast
//!
//! 依赖: `editor_core::EditorStore`, `DebounceTrigger`, `ConflictInfo`, `ConflictAction`
//!        `editor_data_access::DiagramClient`
//!
//! data-testid 清单（验证: `grep -c 'data-testid=' src/editor_panels.rs` 期望 ≥ 6）:
//!   - btn-create-table  /  btn-save  /  revision-display
//!   - table-list-item-{id}
//!   - btn-add-field  /  type-{id}  /  set-ref-{id}
//!   - conflict-dialog  /  btn-force-overwrite  /  btn-reload
//!   - error-toast
//!   - editor-ready (AC-23 TTI 测量点)

use crate::editor_core::{
    ConflictAction, ConflictInfo, DebounceTrigger, EditorStore,
};
use crate::editor_core::types::{Field, Table};
use leptos::*;
use std::rc::Rc;

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
                <div class="conflict-dialog-overlay">
                    <div class="conflict-dialog" data-testid="conflict-dialog">
                        <h2>"保存冲突"</h2>
                        <p>
                            "服务器上的版本比本地更新。请选择如何处理："
                            {format!("本地 rev {} vs 服务器 rev {}", info.local_revision, info.current_revision)}
                        </p>
                        <div class="dialog-buttons">
                            <button
                                data-testid="btn-force-overwrite"
                                on:click=move |_| {
                                    conflict.set(None);
                                    on_force_overwrite();
                                }
                            >
                                "强制覆盖"
                            </button>
                            <button
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
                <div class="error-toast" data-testid="error-toast">
                    {msg}
                    <button on:click=move |_| error.set(None)>{"×"}</button>
                </div>
            }.into_view(),
            None => view! { <></> }.into_view(),
        }
    }

    {render(error)}
}

/// 工具栏视图
#[component]
pub fn TopBar(
    store: EditorStore,
    debouncer: DebounceTrigger,
    on_toolbar_action: Rc<dyn Fn(ToolbarAction)>,
) -> impl IntoView {
    let saving = create_rw_signal(false);
    let action = on_toolbar_action.clone();

    view! {
        <div class="topbar" data-testid="top-bar">
            <button
                data-testid="btn-create-table"
                on:click=move |_| action(ToolbarAction::CreateTable)
            >
                "建表"
            </button>
            <button
                data-testid="btn-save"
                disabled=saving.get()
                on:click=move |_| on_toolbar_action(ToolbarAction::Save)
            >
                {move || if saving.get() { "保存中..." } else { "保存" }}
            </button>
            <span data-testid="revision-display">
                {move || format!("rev: {}", store.revision.get())}
            </span>
        </div>
    }
}

/// 左侧面板视图
#[component]
pub fn LeftPanel(
    store: EditorStore,
    selected_table_id: RwSignal<Option<String>>,
    on_select_table: Rc<dyn Fn(Option<String>)>,
) -> impl IntoView {
    view! {
        <div class="left-panel" data-testid="left-panel">
            <h3>"表列表"</h3>
            <div class="table-list">
                <For each=move || store.tables.get() key=|table| table.id.clone() children=move |table: Table| {
                    let table_id = table.id.clone();
                    let table_name = table.name.clone();
                    let on_select = on_select_table.clone();
                    let testid = format!("table-list-item-{}", table_id);
                    let testid_for_select = testid.clone();
                    view! {
                        <div
                            class="table-list-item"
                            class:selected=move || selected_table_id.get() == Some(table_id.clone())
                            data-testid={testid}
                            on:click=move |_| {
                                on_select(Some(testid_for_select.clone()));
                            }
                        >
                            {table_name}
                        </div>
                    }
                } />
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
        <div class="right-panel" data-testid="right-panel">
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
                        <div class="field-list">
                            <h3>{table_name}</h3>
                            <button data-testid="btn-add-field" on:click=on_add_click>"加字段"</button>
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
                                    <div class="field-row" data-testid={testid_row}>
                                        <span>{field_name}</span>
                                        <select
                                            data-testid={testid_type}
                                            value=field_type
                                            on:change=move |ev| {
                                                let new_type = event_target_value(&ev);
                                                on_change(field_id_for_change.clone(), new_type);
                                            }
                                        >
                                            <option value="INT">INT</option>
                                            <option value="BIGINT">BIGINT</option>
                                            <option value="VARCHAR(255)">VARCHAR(255)</option>
                                            <option value="TEXT">TEXT</option>
                                            <option value="BOOLEAN">BOOLEAN</option>
                                            <option value="DATE">DATE</option>
                                            <option value="TIMESTAMP">TIMESTAMP</option>
                                            <option value="FLOAT">FLOAT</option>
                                            <option value="DOUBLE">DOUBLE</option>
                                            <option value="DECIMAL">DECIMAL</option>
                                        </select>
                                        <button data-testid={testid_ref} on:click=move |_| { on_ref(field_id_for_ref.clone()); }>"设关系"</button>
                                    </div>
                                }
                            } />
                        </div>
                    }.into_view()
                }
                false => view! { <p>"请选择一个表"</p> }.into_view(),
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

    let make_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        format!("auto-{}", id)
    };

    // Clone for TopBar ( Rc<dyn Fn> doesn't impl Copy)
    let toolbar_action_for_topbar = {
        let store = store.clone();
        let debouncer = debouncer.clone();
        let selected_table_id = selected_table_id.clone();
        let next_id = next_id.clone();
        Rc::new(move |action: ToolbarAction| {
            match action {
                ToolbarAction::CreateTable => {
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
                }
                ToolbarAction::Save => {}
            }
        }) as Rc<dyn Fn(ToolbarAction)>
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

    view! {
        <div class="app-root" data-testid="editor-ready">
            <TopBar
                store=store.clone()
                debouncer=debouncer.clone()
                on_toolbar_action=toolbar_action_for_topbar
            />
            <div class="panels-container">
                <LeftPanel
                    store=store.clone()
                    selected_table_id=selected_table_id.clone()
                    on_select_table=Rc::new(move |id| selected_table_id.set(id))
                />
                <RightPanel
                    store=store.clone()
                    selected_table_id=selected_table_id.clone()
                    on_add_field=on_add_field
                    on_change_type=on_change_type
                    on_set_ref=on_set_ref
                />
            </div>
            <ConflictDialog
                conflict=conflict
                on_force_overwrite=Rc::new(move || {})
                on_reload=Rc::new(move || {})
            />
            <ErrorToast error=error />
        </div>
    }
}