//! E4 CommandPalette — Ctrl+K / Cmd+K 唤起
//! Spec: core-0a-code-editor.md §7 / core-S01-edit-and-save-design.md §3.5

use crate::code_view::ViewMode;
use crate::editor_core::types::{Reference, Table};
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    Table,
    Area,
    Enum,
    Note,
    Reference,
    CustomType,
    Action,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteItem {
    pub kind: PaletteKind,
    pub id: String,
    pub label: String,
    pub subtitle: Option<String>,
}

/// 模糊过滤 palette 条目（纯函数，可单测）。
pub fn filter_palette_items(items: &[PaletteItem], query: &str) -> Vec<PaletteItem> {
    if query.is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|item| {
            item.label.to_lowercase().contains(&q)
                || item
                    .subtitle
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&q))
        })
        .cloned()
        .collect()
}

/// 从 diagram 表 / 关系构建 palette 列表。
/// 固定前置 Action：`action:layout`（整理布局，#23）。
pub fn build_palette_items(tables: &[Table], references: &[Reference]) -> Vec<PaletteItem> {
    let mut items: Vec<PaletteItem> = vec![PaletteItem {
        kind: PaletteKind::Action,
        id: "action:layout".into(),
        label: "整理布局".into(),
        subtitle: Some("力导向 · 对齐 MCP".into()),
    }];
    for t in tables {
        items.push(PaletteItem {
            kind: PaletteKind::Table,
            id: t.id.clone(),
            label: t.name.clone(),
            subtitle: Some(format!("表 · {} 字段", t.fields.len())),
        });
    }
    for r in references {
        items.push(PaletteItem {
            kind: PaletteKind::Reference,
            id: r.id.clone(),
            label: r.name.clone(),
            subtitle: Some("关系".into()),
        });
    }
    items
}

#[component]
pub fn CommandPalette(
    visible: RwSignal<bool>,
    query: RwSignal<String>,
    highlight: RwSignal<usize>,
    items: Memo<Vec<PaletteItem>>,
    on_select: Callback<PaletteItem>,
) -> impl IntoView {
    let filtered = move || filter_palette_items(&items.get(), &query.get());

    view! {
        <Show when=move || visible.get()>
            <div
                class="cdb-command-palette-overlay"
                style="position:fixed;inset:0;z-index:var(--cdb-z-modal,50);background:rgba(0,0,0,0.35);display:flex;align-items:flex-start;justify-content:center;padding-top:15vh;"
                on:click=move |_| visible.set(false)
            >
                <div
                    class="cdb-command-palette"
                    data-testid="command-palette"
                    data-open="true"
                    style="width:min(560px,92vw);background:var(--cdb-color-bg-0);border:1px solid var(--cdb-color-border);border-radius:var(--cdb-radius-lg);box-shadow:var(--cdb-shadow-lg);padding:var(--cdb-space-3);"
                    on:click=|ev| ev.stop_propagation()
                >
                    <input
                        class="cdb-command-palette__input"
                        data-testid="command-palette-input"
                        type="text"
                        placeholder="搜索表 / 关系…"
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            query.set(event_target_value(&ev));
                            highlight.set(0);
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            let list = filtered();
                            match ev.key().as_str() {
                                "Escape" => visible.set(false),
                                "ArrowDown" => {
                                    if !list.is_empty() {
                                        highlight.update(|h| *h = (*h + 1) % list.len());
                                    }
                                    ev.prevent_default();
                                }
                                "ArrowUp" => {
                                    if !list.is_empty() {
                                        highlight.update(|h| {
                                            *h = if *h == 0 { list.len() - 1 } else { *h - 1 };
                                        });
                                    }
                                    ev.prevent_default();
                                }
                                "Enter" => {
                                    if let Some(item) = list.get(highlight.get()) {
                                        on_select.call(item.clone());
                                        visible.set(false);
                                        query.set(String::new());
                                    }
                                    ev.prevent_default();
                                }
                                _ => {}
                            }
                        }
                    />
                    <ul class="cdb-command-palette__results" style="list-style:none;margin:var(--cdb-space-2) 0 0;padding:0;max-height:240px;overflow:auto;">
                        <For
                            each=move || filtered()
                            key=|item| item.id.clone()
                            children=move |item: PaletteItem| {
                                let idx = filtered().iter().position(|x| x.id == item.id).unwrap_or(0);
                                let is_hi = move || highlight.get() == idx;
                                let item_for_click = item.clone();
                                view! {
                                    <li
                                        class=move || {
                                            if is_hi() {
                                                "cdb-command-palette__item cdb-command-palette__item--highlight"
                                            } else {
                                                "cdb-command-palette__item"
                                            }
                                        }
                                        data-testid={
                                            if item.kind == PaletteKind::Action {
                                                format!("palette-{}", item.id.replace(':', "-"))
                                            } else {
                                                format!("palette-item-{}", item.id)
                                            }
                                        }
                                        on:click=move |_| {
                                            on_select.call(item_for_click.clone());
                                            visible.set(false);
                                            query.set(String::new());
                                        }
                                    >
                                        <strong>{item.label.clone()}</strong>
                                        {item.subtitle.as_ref().map(|s| view! {
                                            <span style="margin-left:8px;color:var(--cdb-color-text-2);">{s.clone()}</span>
                                        })}
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </div>
        </Show>
    }
}

/// Ctrl+K / Cmd+K 全局快捷键；Esc 关闭 palette。
pub fn setup_command_palette_shortcut(visible: RwSignal<bool>, view_mode: RwSignal<ViewMode>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let visible_for_key = visible.clone();
    let visible_for_esc = visible.clone();
    let on_key = move |ev: KeyboardEvent| {
        if ev.key() == "k" && (ev.ctrl_key() || ev.meta_key()) && !matches!(view_mode.get_untracked(), ViewMode::Code) {
            ev.prevent_default();
            visible_for_key.update(|v| *v = !*v);
        }
        if ev.key() == "Escape" && visible_for_esc.get_untracked() {
            visible_for_esc.set(false);
        }
    };
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(on_key));
    let _ = window.add_event_listener_with_callback(
        "keydown",
        closure.as_ref().unchecked_ref(),
    );
    closure.forget();
}

#[cfg(test)]
mod tests {
    use super::{build_palette_items, filter_palette_items, PaletteItem, PaletteKind};
    use crate::editor_core::types::{Field, Table};

    #[test]
    fn ut_e4_08_filter_palette_by_query() {
        let items = vec![
            PaletteItem {
                kind: PaletteKind::Table,
                id: "1".into(),
                label: "users".into(),
                subtitle: None,
            },
            PaletteItem {
                kind: PaletteKind::Table,
                id: "2".into(),
                label: "posts".into(),
                subtitle: None,
            },
        ];
        let out = filter_palette_items(&items, "post");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].label, "posts");
    }

    #[test]
    fn ut_e4_09_build_palette_from_tables() {
        let tables = vec![Table {
            id: "t1".into(),
            name: "orders".into(),
            x: 0.0,
            y: 0.0,
            color: String::new(),
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
                dict_code: String::new(),
            }],
            indices: vec![],
            width: None,
            min_height: None,
        }];
        let items = build_palette_items(&tables, &[]);
        // Action「整理布局」固定前置 + 1 张表
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "action:layout");
        assert_eq!(items[0].kind, PaletteKind::Action);
        assert_eq!(items[0].label, "整理布局");
        assert_eq!(items[1].label, "orders");
    }

    /// ST-PB-09：palette 含整理布局 Action，触发后连通表坐标变化
    #[test]
    fn st_pb_09_palette_layout_action_changes_coords() {
        use crate::layout::apply_default_layout;

        let tables = vec![
            Table {
                id: "a".into(),
                name: "a".into(),
                x: 0.0,
                y: 0.0,
                color: String::new(),
                comment: String::new(),
                fields: vec![Field {
                    id: "af".into(),
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
                indices: vec![],
                width: None,
                min_height: None,
            },
            Table {
                id: "b".into(),
                name: "b".into(),
                x: 5.0,
                y: 5.0,
                color: String::new(),
                comment: String::new(),
                fields: vec![Field {
                    id: "bf".into(),
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
                indices: vec![],
                width: None,
                min_height: None,
            },
        ];
        let refs = vec![crate::editor_core::types::Reference {
            id: "r1".into(),
            name: "a->b".into(),
            start_table_id: "a".into(),
            end_table_id: "b".into(),
            start_field_id: "af".into(),
            end_field_id: "bf".into(),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
            color: String::new(),
            line_type: String::new(),
            stroke_style: String::new(),
        }];

        let items = build_palette_items(&tables, &refs);
        let action = items
            .iter()
            .find(|i| i.id == "action:layout" && i.kind == PaletteKind::Action)
            .expect("palette 必须含 action:layout");
        assert_eq!(action.label, "整理布局");

        // 模拟 on_palette_select(Action) → apply_default_layout
        let laid = apply_default_layout(&tables, &refs);
        let changed = laid.iter().zip(tables.iter()).any(|(a, b)| {
            (a.x - b.x).abs() > 0.01 || (a.y - b.y).abs() > 0.01
        });
        assert!(changed, "整理布局后连通表坐标应变化");
    }
}
