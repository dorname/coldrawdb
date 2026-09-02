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

// Import Area/Note from types module (used by B3 EditorStore areas/notes signal)
use crate::editor_core::types::{Area, Note};

pub mod types {
    //! DTO 类型层，与 `RUST_WEB_REFACTOR_PLAN.md §5.3` 字段对齐，
    //! 与后端 `backend/src/diagrams_v1.rs:96-156` API contract 一致。
    //!
    //! 所有类型 derive `Clone, Debug, PartialEq, Serialize, Deserialize` 以支持
    //! Leptos signal diff + 单测断言 + B4 Open 模态 JSON 解析。

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Table {
        pub id: String,
        pub name: String,
        pub x: f64,
        pub y: f64,
        pub color: String,
        pub comment: String,
        pub fields: Vec<Field>,
        pub indices: Vec<Index>,
        // feat-table-resize: 表宽度。`None` 表示用渲染层硬编码默认 (`TABLE_WIDTH = 230.0`)。
        // serde 默认 `None` —— 老 JSON 无此字段时反序列化为 None，向后兼容。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub width: Option<u32>,
        // feat-table-resize: 表最小高度（最小高度语义，operator Q4 裁决）。
        // 渲染 `render_height = max(min_height, TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT × field_count)`。
        // `None` 表示不强制最小高度，按字段数自动撑高（与改造前视觉一致）。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub min_height: Option<u32>,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Index {
        pub id: String,
        pub name: String,
        pub fields: Vec<String>,
        pub unique: bool,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Note {
        pub id: String,
        pub x: f64,
        pub y: f64,
        pub content: String,
        pub color: String,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub areas: RwSignal<Vec<Area>>,
    pub notes: RwSignal<Vec<Note>>,
    pub revision: RwSignal<i64>,
    pub dirty: RwSignal<bool>,
    pub database: RwSignal<Database>,
}

/// 从现有 tables/fields/references/areas/notes 中解析最大数字 id，
/// 用于把 next_id 初始化为 max+1，避免与已保存的 id 冲突
/// （e.g. 已存在 ref-0 时前端再生成 ref-0 → UNIQUE constraint failed）。
pub fn next_id_from_store(store: &EditorStore) -> i64 {
    fn parse_num_suffix(id: &str) -> Option<i64> {
        // 接受 "ref-12" / "auto-7" / "auto-7-field-id" / "auto-9-field-id"
        let last = id.rsplit('-').next()?;
        last.parse::<i64>().ok()
    }
    let mut max_id = 0i64;
    for t in store.tables.get() {
        if let Some(n) = parse_num_suffix(&t.id) {
            max_id = max_id.max(n);
        }
        for f in &t.fields {
            if let Some(n) = parse_num_suffix(&f.id) {
                max_id = max_id.max(n);
            }
        }
    }
    for r in store.references.get() {
        if let Some(n) = parse_num_suffix(&r.id) {
            max_id = max_id.max(n);
        }
    }
    for a in store.areas.get() {
        if let Some(n) = parse_num_suffix(&a.id) {
            max_id = max_id.max(n);
        }
    }
    for n in store.notes.get() {
        if let Some(n) = parse_num_suffix(&n.id) {
            max_id = max_id.max(n);
        }
    }
    max_id
}

/// 全局唯一实体 id 生成器（fix-global-entity-id-uniqueness）。
///
/// 背景：后端 DB 中 table/field/reference/area/note 的 id 是**全局单列主键**，
/// 旧的「图内 max+1 计数器」（`auto-1` / `ref-0` …）在任何新 diagram 上都会从
/// `auto-1` 重新计数，与其他 diagram 已占用的全局 id 冲突 → 保存 500
/// （新账户建表保存失败的根因）。
///
/// 产出格式 `{prefix}-{16位hex}`，如 `auto-3f9a2c8e1b7d40f6`：
/// - 保留原前缀（auto/ref/area/note），兼容 data-testid、OT op 与既有字符串断言；
/// - 后缀为 splitmix64 混合后的 64bit 值：会话内由原子计数器保证**确定不重复**，
///   跨会话由时间+随机种子保证唯一（碰撞概率可忽略）；
/// - 非数字后缀使 `next_id_from_store` 的 parse_num_suffix 返回 None，
///   不影响存量 `auto-N` 数据加载后的 max+1 解析。
///
/// 随机源：wasm 侧 `js_sys`（Date.now + Math.random）；host 侧（`cargo test --lib`）
/// 走 `std::time`，保证单测可在非 wasm 环境直接调用。
pub fn new_entity_id(prefix: &str) -> String {
    format!("{}-{:016x}", prefix, next_unique_u64())
}

fn next_unique_u64() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    static SEED: OnceLock<u64> = OnceLock::new();

    let seed = *SEED.get_or_init(rand_seed);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    // splitmix64 finalizer 是双射：每个不同的 counter 输入必得不同输出
    let mut z = seed.wrapping_add(n).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(target_arch = "wasm32")]
fn rand_seed() -> u64 {
    let t = js_sys::Date::now() as u64;
    let r = (js_sys::Math::random() * (u32::MAX as f64)) as u64;
    (t << 32) ^ r ^ 0x9e37_79b9_7f4a_7c15
}

#[cfg(not(target_arch = "wasm32"))]
fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ 0x9e37_79b9_7f4a_7c15
}

impl EditorStore {
    /// Create a fresh empty store. Intended to be called once at app boot from `lib.rs`
    /// (`create_store` factory); panels / render / data-access must NOT call this themselves
    /// (spec R-3 防御：避免循环 store)。
    pub fn new() -> Self {
        Self {
            tables: create_rw_signal(Vec::new()),
            references: create_rw_signal(Vec::new()),
            areas: create_rw_signal(Vec::new()),
            notes: create_rw_signal(Vec::new()),
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
        self.areas.set(diagram.areas);
        self.notes.set(diagram.notes);
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
            areas: self.areas.get(),
            notes: self.notes.get(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabPendingOp {
    pub client_rev: i64,
    pub op_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollabConnectionState {
    Offline,
    Connecting,
    Connected,
    Reconnecting,
    ReadOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollabOtState {
    pub connection: CollabConnectionState,
    pub server_rev: i64,
    pub next_client_rev: i64,
    pub pending_ops: Vec<CollabPendingOp>,
    pub queued_while_offline: Vec<CollabPendingOp>,
    /// ST-S05-UI-05：重连失败后用户显式选择「仅本地编辑」→ 持续提示 409 风险，
    /// 本地可继续编辑（op 进入 queued_while_offline），但不得误报 OT 已同步。
    pub local_only: bool,
}

impl Default for CollabOtState {
    fn default() -> Self {
        Self {
            connection: CollabConnectionState::Offline,
            server_rev: 0,
            next_client_rev: 1,
            pending_ops: Vec::new(),
            queued_while_offline: Vec::new(),
            local_only: false,
        }
    }
}

impl CollabOtState {
    pub fn connected(server_rev: i64) -> Self {
        Self {
            connection: CollabConnectionState::Connected,
            server_rev,
            ..Default::default()
        }
    }

    pub fn enqueue_local_op(&mut self, op_type: impl Into<String>) -> Option<i64> {
        if matches!(self.connection, CollabConnectionState::ReadOnly) {
            return None;
        }
        let op = CollabPendingOp {
            client_rev: self.next_client_rev,
            op_type: op_type.into(),
        };
        self.next_client_rev += 1;
        if matches!(self.connection, CollabConnectionState::Connected) {
            self.pending_ops.push(op.clone());
        } else {
            self.queued_while_offline.push(op.clone());
        }
        Some(op.client_rev)
    }

    pub fn ack(&mut self, client_rev: Option<i64>, server_rev: i64) {
        self.server_rev = self.server_rev.max(server_rev);
        if let Some(client_rev) = client_rev {
            self.pending_ops.retain(|op| op.client_rev != client_rev);
        } else if !self.pending_ops.is_empty() {
            self.pending_ops.remove(0);
        }
    }

    pub fn mark_reconnecting(&mut self) {
        self.connection = CollabConnectionState::Reconnecting;
    }

    pub fn apply_sync(&mut self, server_rev: i64) {
        self.server_rev = self.server_rev.max(server_rev);
        self.connection = CollabConnectionState::Connected;
        self.local_only = false;
        self.pending_ops.extend(self.queued_while_offline.drain(..));
    }

    /// 进入仅本地模式（ST-S05-UI-05）：保留排队 op，状态栏/横幅持续提示 409 风险。
    pub fn enter_local_only(&mut self) {
        self.local_only = true;
    }

    /// 仅本地模式下的 OT 入队与离线一致：op 进入 queued_while_offline 等待 sync。
    pub fn is_local_only(&self) -> bool {
        self.local_only
    }

    /// ST-S01-409-SCOPE / ST-S01-NO-409-OT：协作已连接且非「仅本地」时，
    /// 快照 PUT 409 视为服务器合并——禁止弹 S01 冲突模态（改走 Activity 反馈 + 采纳服务器 rev）。
    /// 仅本地（ST-S01-409-LOCAL-ONLY）与非协作路径返回 true，保留 modal-conflict。
    pub fn snapshot_conflict_shows_modal(&self) -> bool {
        !(matches!(self.connection, CollabConnectionState::Connected) && !self.local_only)
    }

    pub fn mark_read_only(&mut self) {
        self.connection = CollabConnectionState::ReadOnly;
        self.pending_ops.clear();
        self.queued_while_offline.clear();
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

    /// Pop the most recent command from the undo stack and push it onto the redo stack.
    ///
    /// B5: 底座实现 — 仅栈管理，不做 inverse apply（spec V1 边界：实际状态回滚
    /// 需要 per-Command 变体反向逻辑，工作量超出本批次；UT-MM-15 验证栈语义）
    ///
    /// 返回值：Some(cmd) 表示成功弹出，None 表示 undo 栈空
    pub fn undo(&mut self) -> Option<Command> {
        let cmd = self.undo.pop()?;
        self.redo.push(cmd.clone());
        Some(cmd)
    }

    /// Pop the most recent command from the redo stack and push it back onto the undo stack.
    ///
    /// B5: 底座实现 — 与 undo 对称，仅栈管理。
    /// UT-MM-16 验证栈语义。
    pub fn redo(&mut self) -> Option<Command> {
        let cmd = self.redo.pop()?;
        self.undo.push(cmd.clone());
        Some(cmd)
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

    /// 撤销 apply 的副作用（V1：覆盖 AddTable / AddField / AddReference / DeleteField / DeleteReference / ChangeType）。
    pub fn revert(store: &EditorStore, cmd: &Command) -> Result<(), CoreError> {
        match cmd {
            Command::AddTable(table) => {
                let mut tables = store.tables.get();
                tables.retain(|t| t.id != table.id);
                store.tables.set(tables);
            }
            Command::AddField { table_id, field } => {
                let mut tables = store.tables.get();
                if let Some(table) = tables.iter_mut().find(|t| t.id == *table_id) {
                    table.fields.retain(|f| f.id != field.id);
                    store.tables.set(tables);
                }
            }
            Command::DeleteField { table_id, field_id: _ } => {
                return Err(CoreError::new("DeleteField revert not supported in V1"));
            }
            Command::AddReference(reference) => {
                let mut refs = store.references.get();
                refs.retain(|r| r.id != reference.id);
                store.references.set(refs);
            }
            Command::DeleteReference { reference_id: _ } => {
                return Err(CoreError::new("DeleteReference revert not supported in V1"));
            }
            Command::ChangeType { field_id: _, new_type: _ } => {
                return Err(CoreError::new("ChangeType revert not supported in V1"));
            }
        }
        store.dirty.set(true);
        Ok(())
    }

    /// 仅执行命令副作用，不写入 undo 栈（redo 路径使用）。
    pub fn execute(store: &EditorStore, cmd: &Command) -> Result<(), CoreError> {
        match cmd {
            Command::AddTable(table) => {
                let mut tables = store.tables.get();
                if tables.iter().any(|t| t.id == table.id) {
                    return Err(CoreError::new(format!("table id '{}' already exists", table.id)));
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
                table.fields.push(field.clone());
                store.tables.set(tables);
            }
            Command::AddReference(reference) => {
                let mut refs = store.references.get();
                refs.push(reference.clone());
                store.references.set(refs);
            }
            Command::DeleteField { .. } | Command::DeleteReference { .. } | Command::ChangeType { .. } => {
                return Err(CoreError::new("command not supported for execute-only path"));
            }
        }
        store.dirty.set(true);
        Ok(())
    }

    pub fn record(&mut self, cmd: Command) {
        self.undo.push(cmd);
        self.redo.clear();
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

#[cfg(test)]
mod tests {
    //! Unit tests for editor_core (B1: add-frontend-completeness)
    //!
    //! Covered OpenLogos cases:
    //!   - UT-S01-07: 表创建触发 undo 栈 (CommandStack::apply AddTable)
    //!   - UT-S01-08: debounce 触发 save (DebounceTrigger::schedule)
    //!
    //! Spec: logos/resources/test/core-S01-test-cases.md §2 line 107/120

    use super::*;
    use crate::editor_core::types::Field;

    /// UT-S01-07 — 表创建触发 undo 栈
    /// Spec: `core-S01-test-cases.md` line 107
    /// 验证：3 次 AddTable 后 stack.undo.len() == 3
    #[test]
    fn test_undo_stack_push_addtable_ut_s01_07() {
        let store = EditorStore::new();
        let mut stack = CommandStack::new();

        for i in 0..3 {
            let t = Table {
                id: format!("t-{}", i),
                name: format!("T{}", i),
                x: 0.0,
                y: 0.0,
                color: "#000".into(),
                comment: String::new(),
                fields: Vec::new(),
                indices: Vec::new(),
                width: None,
                min_height: None,
            };
            let cmd = Command::AddTable(t);
            CommandStack::apply(&store, &mut stack, cmd)
                .expect("AddTable should succeed for unique id");
        }

        // 验证：3 次 AddTable 后 undo_stack.len() == 3
        assert_eq!(stack.undo.len(), 3, "UT-S01-07: 3 AddTable 后 undo_stack 长度应为 3");
        // 副作用：store.tables 也应该有 3 项
        assert_eq!(store.tables.get().len(), 3, "UT-S01-07: store.tables 应含 3 项");
        // 重置：redo 应被清空
        assert_eq!(stack.redo.len(), 0, "UT-S01-07: 新增命令应清空 redo 栈");
    }

    /// UT-S01-07 扩展：重复 id 触发 CoreError
    #[test]
    fn test_undo_stack_rejects_duplicate_id_ut_s01_07() {
        let store = EditorStore::new();
        let mut stack = CommandStack::new();
        let t = Table {
            id: "dup".into(),
            name: "T".into(),
            x: 0.0, y: 0.0, color: "#000".into(),
            comment: String::new(), fields: Vec::new(), indices: Vec::new(),
            width: None,
            min_height: None,
        };
        CommandStack::apply(&store, &mut stack, Command::AddTable(t.clone())).unwrap();
        let result = CommandStack::apply(&store, &mut stack, Command::AddTable(t));
        assert!(result.is_err(), "UT-S01-07: 重复 id 应返回 Err");
    }

    /// UT-S01-07 扩展：DeleteField 正常 + 错误
    #[test]
    fn test_delete_field_happy_and_missing_ut_s01_07() {
        let store = EditorStore::new();
        let mut stack = CommandStack::new();
        let t = Table {
            id: "t1".into(), name: "T".into(),
            x: 0.0, y: 0.0, color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "F".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: false, unique: false, not_null: false, increment: false,
                comment: String::new(),
            }],
            indices: Vec::new(),
            width: None,
            min_height: None,
        };
        CommandStack::apply(&store, &mut stack, Command::AddTable(t)).unwrap();
        // 正常删除
        assert!(CommandStack::apply(&store, &mut stack, Command::DeleteField {
            table_id: "t1".into(), field_id: "f1".into(),
        }).is_ok());
        // 字段不存在 → Err
        let r = CommandStack::apply(&store, &mut stack, Command::DeleteField {
            table_id: "t1".into(), field_id: "missing".into(),
        });
        assert!(r.is_err(), "UT-S01-07: 字段不存在应返回 Err");
    }

    /// UT-S01-08 — debounce 触发 save
    /// Spec: `core-S01-test-cases.md` line 120
    /// 验证：DebounceTrigger schedule 1s 静默期后触发回调
    ///
    /// 注：DebounceTrigger 内部用 gloo_timers::callback::Timeout，在 native 目标下
    /// gloo-timers 通过 std::thread 实现，但 callback 需要 Send + 'static。
    /// 真实测试需在 wasm 目标下用 wasm-bindgen-test 跑（依赖 wasm-pack test 环境）。
    /// B1 阶段标 #[ignore]，reporter 中 status=skip。B5 接入 wasm-pack test 后取消 ignore。
    #[test]
    #[ignore = "requires wasm-pack test (B5); gloo-timers callback needs Send + 'static"]
    fn test_debounce_trigger_fires_after_delay_ut_s01_08() {
        use std::rc::Rc;
        use std::cell::Cell;
        use std::thread;
        use std::time::Duration;

        let fired = Rc::new(Cell::new(false));
        let fired_clone = fired.clone();
        let trigger = DebounceTrigger::new(50);
        trigger.schedule(move || {
            fired_clone.set(true);
        });

        // 等待超过 50ms
        thread::sleep(Duration::from_millis(150));

        assert!(fired.get(), "UT-S01-08: 50ms debounce 后回调应已触发");
    }

    /// UT-S01-08 扩展：连续 schedule 取消前一次（debounce 语义）
    /// 同 UT-S01-08：wasm-pack test only (B5)
    #[test]
    #[ignore = "requires wasm-pack test (B5); gloo-timers callback needs Send + 'static"]
    fn test_debounce_retrigger_resets_ut_s01_08() {
        use std::rc::Rc;
        use std::cell::Cell;
        use std::thread;
        use std::time::Duration;

        let count = Rc::new(Cell::new(0));
        let c1 = count.clone();
        let c2 = count.clone();

        let trigger = DebounceTrigger::new(80);
        trigger.schedule(move || { c1.set(c1.get() + 1); });

        thread::sleep(Duration::from_millis(30));
        // 在 80ms 静默期内再次 schedule → 取消前一次
        trigger.schedule(move || { c2.set(c2.get() + 1); });

        thread::sleep(Duration::from_millis(150));
        // 只应触发 1 次（第二次）
        assert_eq!(count.get(), 1, "UT-S01-08: debounce 多次 schedule 仅最后一次触发");
    }

    // --- UT-CR-01 — Areas 渲染（store.areas → draw_area） ---

    /// UT-CR-01: EditorStore::new() 时 areas 为空；load 注入 2 个 area 后变 2
    #[test]
    fn test_editor_store_areas_default_empty_ut_cr_01() {
        let store = EditorStore::new();
        assert!(store.areas.get().is_empty(), "UT-CR-01: 初始 areas 应为空");
    }

    /// UT-CR-01: load() 注入 areas 后 store.areas.get() 返回正确数量
    #[test]
    fn test_editor_store_areas_load_ut_cr_01() {
        let store = EditorStore::new();
        let mut diagram = Diagram {
            id: "d1".into(),
            name: "D".into(),
            revision: 1,
            database: Database::Generic,
            tables: Vec::new(),
            references: Vec::new(),
            notes: Vec::new(),
            areas: vec![
                Area {
                    id: "a1".into(),
                    x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                    color: "#000".into(), name: "Area 1".into(),
                },
                Area {
                    id: "a2".into(),
                    x: 200.0, y: 200.0, width: 50.0, height: 50.0,
                    color: "#111".into(), name: "Area 2".into(),
                },
            ],
        };
        store.load(diagram.clone());
        assert_eq!(store.areas.get().len(), 2, "UT-CR-01: load 后 areas 应有 2 项");
        assert_eq!(store.areas.get()[0].name, "Area 1");

        // snapshot 同步
        let snap = store.snapshot("d1".into(), "D".into());
        assert_eq!(snap.areas.len(), 2, "UT-CR-01: snapshot.areas 应有 2 项");
        assert_eq!(snap.areas[1].id, "a2");
    }

    // --- UT-CR-02 — Notes 渲染（store.notes → draw_note） ---

    /// UT-CR-02: load() 注入 3 个 note 后 store.notes.get() 返回 3
    #[test]
    fn test_editor_store_notes_load_ut_cr_02() {
        let store = EditorStore::new();
        assert!(store.notes.get().is_empty(), "UT-CR-02: 初始 notes 应为空");
        let diagram = Diagram {
            id: "d".into(),
            name: "D".into(),
            revision: 0,
            database: Database::Generic,
            tables: Vec::new(),
            references: Vec::new(),
            areas: Vec::new(),
            notes: vec![
                Note {
                    id: "n1".into(), x: 0.0, y: 0.0,
                    content: "Note 1".into(), color: "#fff".into(),
                },
                Note {
                    id: "n2".into(), x: 50.0, y: 50.0,
                    content: "Note 2".into(), color: "#fff".into(),
                },
                Note {
                    id: "n3".into(), x: 100.0, y: 100.0,
                    content: "Note 3".into(), color: "#fff".into(),
                },
            ],
        };
        store.load(diagram);
        assert_eq!(store.notes.get().len(), 3, "UT-CR-02: load 后 notes 应有 3 项");
        let snap = store.snapshot("d".into(), "D".into());
        assert_eq!(snap.notes.len(), 3);
    }

    // ─── B5 CommandStack undo/redo tests (UT-MM-15 / UT-MM-16) ────────────

    #[test]
    fn test_command_stack_undo_ut_mm_15() {
        let mut stack = CommandStack::new();
        let cmd = Command::AddTable(Table {
            id: "t-undo-1".into(),
            name: "t1".into(),
            x: 0.0,
            y: 0.0,
            color: "".into(),
            comment: "".into(),
            fields: Vec::new(),
            indices: Vec::new(),
            width: None,
            min_height: None,
        });
        // 手动 push（避免 store 依赖）
        stack.undo.push(cmd.clone());
        assert_eq!(stack.undo.len(), 1, "UT-MM-15: push 后 undo 长度 1");
        let popped = stack.undo();
        assert!(popped.is_some(), "UT-MM-15: undo 弹出一条");
        assert_eq!(popped.unwrap(), cmd);
        assert_eq!(stack.undo.len(), 0, "UT-MM-15: undo 后 undo 栈为空");
        assert_eq!(stack.redo.len(), 1, "UT-MM-15: 弹出的 cmd 应进 redo 栈");
    }

    #[test]
    fn test_command_stack_undo_empty_ut_mm_15() {
        let mut stack = CommandStack::new();
        let popped = stack.undo();
        assert!(popped.is_none(), "UT-MM-15: 空 undo 栈应返回 None");
        assert_eq!(stack.redo.len(), 0, "UT-MM-15: 空 undo 栈调用后 redo 仍为空");
    }

    #[test]
    fn test_command_stack_redo_ut_mm_16() {
        let mut stack = CommandStack::new();
        let cmd = Command::AddReference(Reference {
            id: "r-redo-1".into(),
            name: "fk".into(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "1:N".into(),
            on_delete: "".into(),
            on_update: "".into(),
        });
        stack.undo.push(cmd.clone());
        stack.undo();
        assert_eq!(stack.redo.len(), 1, "UT-MM-16: undo 后 redo 长度 1");
        let popped = stack.redo();
        assert!(popped.is_some(), "UT-MM-16: redo 弹出一条");
        assert_eq!(popped.unwrap(), cmd);
        assert_eq!(stack.redo.len(), 0, "UT-MM-16: redo 后 redo 栈为空");
        assert_eq!(stack.undo.len(), 1, "UT-MM-16: 弹出的 cmd 应回 undo 栈");
    }

    #[test]
    fn test_command_stack_redo_empty_ut_mm_16() {
        let mut stack = CommandStack::new();
        let popped = stack.redo();
        assert!(popped.is_none(), "UT-MM-16: 空 redo 栈应返回 None");
    }

    #[test]
    fn ut_fe_s05_07_collab_ot_queue_and_ack() {
        let mut state = CollabOtState::connected(3);
        assert_eq!(state.enqueue_local_op("table.create"), Some(1));
        assert_eq!(state.pending_ops.len(), 1);
        state.ack(Some(1), 4);
        assert_eq!(state.server_rev, 4);
        assert!(state.pending_ops.is_empty());
    }

    #[test]
    fn ut_fe_s05_08_collab_ot_reconnect_flushes_offline_queue() {
        let mut state = CollabOtState::connected(4);
        state.mark_reconnecting();
        assert_eq!(state.enqueue_local_op("field.update"), Some(1));
        assert_eq!(state.queued_while_offline.len(), 1);
        assert!(state.pending_ops.is_empty());

        state.apply_sync(6);
        assert_eq!(state.connection, CollabConnectionState::Connected);
        assert_eq!(state.server_rev, 6);
        assert!(state.queued_while_offline.is_empty());
        assert_eq!(state.pending_ops.len(), 1);
    }

    #[test]
    fn ut_fe_s05_09_collab_ot_read_only_blocks_local_op() {
        let mut state = CollabOtState::connected(0);
        state.mark_read_only();
        assert_eq!(state.enqueue_local_op("table.create"), None);
        assert!(state.pending_ops.is_empty());
        assert!(state.queued_while_offline.is_empty());
    }

    /// feat-table-resize 批次3 步骤3 验证项:
    /// OT op 应用器(CommandStack::apply)对 Table.width/min_height 字段
    /// 是 silently ignore(不报错)还是崩溃 —— 验证 silently ignore 成立。
    #[test]
    fn feat_table_resize_ot_add_table_accepts_width_and_min_height() {
        use crate::editor_core::types::{Field, Table};
        let store = EditorStore::new();
        let mut stack = CommandStack::new();
        let t = Table {
            id: "t-ot-width".into(),
            name: "OT-Width".into(),
            x: 0.0, y: 0.0,
            color: "#000".into(),
            comment: String::new(),
            fields: vec![Field {
                id: "f1".into(), name: "id".into(), type_: "INT".into(),
                default: String::new(), check: String::new(),
                primary: true, unique: false, not_null: true, increment: false,
                comment: String::new(),
            }],
            indices: Vec::new(),
            width: Some(400),
            min_height: Some(250),
        };
        // 应用 op:width=Some(400),min_height=Some(250) 应正常写入(不 silently drop)
        CommandStack::apply(&store, &mut stack, Command::AddTable(t))
            .expect("AddTable with width/min_height 应成功(OT op 应用器 silently ignore 未知字段)");
        let stored = &store.tables.get()[0];
        assert_eq!(stored.width, Some(400), "width 随 Table struct 序列化,OT 无需改动");
        assert_eq!(stored.min_height, Some(250), "min_height 随 Table struct 序列化,OT 无需改动");
    }
}
