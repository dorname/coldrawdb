//! editor-core: types + EditorStore + 命令栈 + debounce 1s 触发器 + 409 hook
//!
//! 状态底座 / 业务模型。panels / render / data-access 全部 `use editor_core::types` 共享 DTO。
//! 依赖方向：core ← {render, panels, data_access}，反向 import 在 CI 由 ast-grep gate 防御（详
//! `frontend-rs/scripts/check_module_deps.sh`，plan R-3）。
//!
//! Spec 关联：
//! - 5 大功能 happy path 触发器：建表 / 加字段 / 设关系 / 改类型 / 保存（plan W2-2）
//! - debounce 1s 自动保存：spec §Constraints §mvp-minimum-link
//! - 409 协议：后端 `backend/src/diagrams_v1.rs:137-144` 返回 `current_revision`，
//!   前端弹窗二选一（强制覆盖 / 重新加载）
//! - undo/redo 底座存在但不暴露 UI：spec §Non-Goals

use crate::editor_core::types::*;
use gloo_timers::callback::Timeout;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;

pub mod types {
    //! DTO 类型层，与 `RUST_WEB_REFACTOR_PLAN.md §5.3` 字段对齐，
    //! 与后端 `backend/src/diagrams_v1.rs:96-156` API contract 一致。
    //!
    //! 所有类型 derive `Clone, Debug, PartialEq` 以支持 Leptos signal diff 与单测断言。
    //! serde Serialize/Deserialize 在 W2-1 editor-data-access 接入网络层时再加，避免本层
    //! 提前承担 codec 复杂度。

    #[derive(Clone, Debug, PartialEq)]
    pub struct Diagram {
        pub id: String,
        pub name: String,
        pub revision: i64,
        pub database: Database,
        pub tables: Vec<Table>,
        pub references: Vec<Reference>,
        pub notes: Vec<Note>,
        pub areas: Vec<Area>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Table {
        pub id: String,
        pub name: String,
        pub x: f64,
        pub y: f64,
        pub color: String,
        pub comment: String,
        pub fields: Vec<Field>,
        pub indices: Vec<Index>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Field {
        pub id: String,
        pub name: String,
        pub type_: String,
        pub default: String,
        pub check: String,
        pub primary: bool,
        pub unique: bool,
        pub not_null: bool,
        pub increment: bool,
        pub comment: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Reference {
        pub id: String,
        pub name: String,
        pub start_table_id: String,
        pub end_table_id: String,
        pub start_field_id: String,
        pub end_field_id: String,
        pub type_: String,
        pub on_delete: String,
        pub on_update: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Index {
        pub id: String,
        pub name: String,
        pub fields: Vec<String>,
        pub unique: bool,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Note {
        pub id: String,
        pub x: f64,
        pub y: f64,
        pub content: String,
        pub color: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Area {
        pub id: String,
        pub x: f64,
        pub y: f64,
        pub width: f64,
        pub height: f64,
        pub color: String,
        pub name: String,
    }

    /// 后端 dialect（与后端 `Database` 枚举对齐；详 backend/src/diagrams_v1.rs:96）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Database {
        Generic,
        Mysql,
        Postgresql,
        Sqlite,
        Mssql,
        Oracle,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EditorStore {
    pub tables: RwSignal<Vec<Table>>,
    pub references: RwSignal<Vec<Reference>>,
    pub revision: RwSignal<i64>,
    pub dirty: RwSignal<bool>,
    pub database: RwSignal<Database>,
}

impl EditorStore {
    /// Create a fresh empty store. Intended to be called once at app boot from `lib.rs`
    /// (`create_store` factory); panels / render / data-access must NOT call this themselves
    /// (spec R-3 防御：避免循环 store)。
    pub fn new() -> Self {
        Self {
            tables: create_rw_signal(Vec::new()),
            references: create_rw_signal(Vec::new()),
            revision: create_rw_signal(0),
            dirty: create_rw_signal(false),
            database: create_rw_signal(Database::Generic),
        }
    }

    /// Replace the in-memory store with a freshly loaded diagram (used by `editor-data-access`
    /// after a `GET /api/v1/diagrams/{id}` succeeds).
    pub fn load(&self, diagram: Diagram) {
        self.tables.set(diagram.tables);
        self.references.set(diagram.references);
        self.revision.set(diagram.revision);
        self.database.set(diagram.database);
        self.dirty.set(false);
    }

    /// Snapshot the current store as a `Diagram` for `PUT /api/v1/diagrams/{id}`.
    pub fn snapshot(&self, id: String, name: String) -> Diagram {
        Diagram {
            id,
            name,
            revision: self.revision.get(),
            database: self.database.get(),
            tables: self.tables.get(),
            references: self.references.get(),
            notes: Vec::new(),
            areas: Vec::new(),
        }
    }
}

impl Default for EditorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    AddTable(Table),
    AddField {
        table_id: String,
        field: Field,
    },
    DeleteField {
        table_id: String,
        field_id: String,
    },
    AddReference(Reference),
    DeleteReference {
        reference_id: String,
    },
    ChangeType {
        field_id: String,
        new_type: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoreError {
    pub message: String,
}

impl CoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommandStack {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl CommandStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a command to the store and push onto the undo stack.
    ///
    /// Spec §Non-Goals：undo UI 不暴露（按钮不渲染），但底座存在；Phase 5 零成本开启 UI。
    /// 调用方在状态变更后必须 `store.dirty.set(true)` + `debouncer.schedule(save)`，本函数
    /// 不自动触发保存，避免 apply 与 debouncer 之间的隐式耦合。
    pub fn apply(store: &EditorStore, stack: &mut CommandStack, cmd: Command) -> Result<(), CoreError> {
        match &cmd {
            Command::AddTable(table) => {
                let mut tables = store.tables.get();
                if tables.iter().any(|t| t.id == table.id) {
                    return Err(CoreError::new(format!(
                        "table id '{}' already exists",
                        table.id
                    )));
                }
                tables.push(table.clone());
                store.tables.set(tables);
            }
            Command::AddField { table_id, field } => {
                let mut tables = store.tables.get();
                let table = tables
                    .iter_mut()
                    .find(|t| t.id == *table_id)
                    .ok_or_else(|| CoreError::new(format!("table '{}' not found", table_id)))?;
                if table.fields.iter().any(|f| f.id == field.id) {
                    return Err(CoreError::new(format!(
                        "field id '{}' already exists in table '{}'",
                        field.id, table_id
                    )));
                }
                table.fields.push(field.clone());
                store.tables.set(tables);
            }
            Command::DeleteField { table_id, field_id } => {
                let mut tables = store.tables.get();
                let table = tables
                    .iter_mut()
                    .find(|t| t.id == *table_id)
                    .ok_or_else(|| CoreError::new(format!("table '{}' not found", table_id)))?;
                let before = table.fields.len();
                table.fields.retain(|f| f.id != *field_id);
                if table.fields.len() == before {
                    return Err(CoreError::new(format!(
                        "field '{}' not found in table '{}'",
                        field_id, table_id
                    )));
                }
                store.tables.set(tables);
            }
            Command::AddReference(reference) => {
                let mut refs = store.references.get();
                if refs.iter().any(|r| r.id == reference.id) {
                    return Err(CoreError::new(format!(
                        "reference id '{}' already exists",
                        reference.id
                    )));
                }
                refs.push(reference.clone());
                store.references.set(refs);
            }
            Command::DeleteReference { reference_id } => {
                let mut refs = store.references.get();
                let before = refs.len();
                refs.retain(|r| r.id != *reference_id);
                if refs.len() == before {
                    return Err(CoreError::new(format!(
                        "reference '{}' not found",
                        reference_id
                    )));
                }
                store.references.set(refs);
            }
            Command::ChangeType { field_id, new_type } => {
                let mut tables = store.tables.get();
                let mut found = false;
                for table in tables.iter_mut() {
                    if let Some(f) = table.fields.iter_mut().find(|f| f.id == *field_id) {
                        f.type_ = new_type.clone();
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(CoreError::new(format!("field '{}' not found", field_id)));
                }
                store.tables.set(tables);
            }
        }
        store.dirty.set(true);
        stack.undo.push(cmd);
        stack.redo.clear();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictAction {
    ForceOverwrite,
    Reload,
}

#[derive(Clone)]
pub struct ConflictInfo {
    pub current_revision: i64,
    pub local_revision: i64,
    pub on_resolve: Rc<RefCell<Option<Box<dyn FnOnce(ConflictAction)>>>>,
}

impl ConflictInfo {
    pub fn new(current_revision: i64, local_revision: i64) -> Self {
        Self {
            current_revision,
            local_revision,
            on_resolve: Rc::new(RefCell::new(None)),
        }
    }

    /// UI 弹窗选中后调用；`on_resolve` 必须由 `editor-data-access` 在收到 409 时注入。
    pub fn resolve(&self, action: ConflictAction) {
        if let Some(cb) = self.on_resolve.borrow_mut().take() {
            cb(action);
        }
    }
}

/// 1s debounce 触发器。`schedule` 复用同一 handle：每次调用取消旧 Timeout 并启动新的，
/// 实现「1s 静默期」语义（spec §mvp-minimum-link）。`gloo-timers::callback::Timeout` 在
/// Drop 时自动取消未触发的 callback，无需手动清理。
#[derive(Clone)]
pub struct DebounceTrigger {
    handle: Rc<RefCell<Option<Timeout>>>,
    delay_ms: u32,
}

impl DebounceTrigger {
    pub fn new(delay_ms: u32) -> Self {
        Self {
            handle: Rc::new(RefCell::new(None)),
            delay_ms,
        }
    }

    /// Reset the debounce timer. `f` runs after `delay_ms` of inactivity.
    pub fn schedule<F: FnOnce() + 'static>(&self, f: F) {
        let handle = self.handle.clone();
        let delay = self.delay_ms;
        *self.handle.borrow_mut() = Some(Timeout::new(delay, move || {
            handle.borrow_mut().take();
            f();
        }));
    }

    pub fn cancel(&self) {
        self.handle.borrow_mut().take();
    }
}

impl Default for DebounceTrigger {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[allow(dead_code)]
fn init() {}
