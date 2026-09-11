//! wire-frontend-collab-ws：S05 生产前端真实 WS 协作接入。
//!
//! 分层：
//! - **纯逻辑**（host/wasm 均可编译、cargo test 可测）：实体快照 diff → CollabOp 映射、
//!   remote_op 应用、重连退避策略、viewer 拦截判定。
//! - **wasm 传输层**：`CollabWs`（web_sys WebSocket 封装）与 `CollabController`
//!   （连接生命周期 / 帧分发 / 离线队列 / presence 节流 / 断线重连 + sync flush）。
//!
//! op 生成策略：diff-based——监听 store 集合信号，按实体 id 计算 added/changed/removed，
//! 映射为 `collab.yaml` 的 15 种 op type。payload `changes` 携带实体全量 JSON
//! （table.update 例外：剔除 `fields`，字段级变更走 field.* op），
//! 与服务端不透明转发 + operation_log 持久化兼容（`CollabOp.changes` 为 additionalProperties）。

use std::collections::BTreeMap;

use leptos::*;
use serde_json::{json, Value};

use crate::editor_core::types::{Area, DataDictionary, Field, Note, Reference, Table};
use crate::editor_core::{CollabConnectionState, CollabOtState, EditorStore};

// ---------------------------------------------------------------------------
// 纯逻辑：实体快照
// ---------------------------------------------------------------------------

/// 某一时刻 store 实体集合的可比较快照（按 id 索引）。
/// `tables` 存剔除 `fields` 的标量 JSON；字段单独平铺（含父表 id），
/// 使字段级变更映射为 field.* op 而非整表 table.update。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntitySnapshot {
    pub tables: BTreeMap<String, Value>,
    /// field_id → (table_id, field JSON)
    pub fields: BTreeMap<String, (String, Value)>,
    /// fix-collab-autosave-race：table_id → 有序 field id 列表。
    /// id 键控的 `fields` 对「纯换序」产生空 diff（ST-SP-LIST-02 暴露），
    /// 顺序单独成列，供 diff 产出 fields.reorder op。
    pub field_order: BTreeMap<String, Vec<String>>,
    pub references: BTreeMap<String, Value>,
    pub areas: BTreeMap<String, Value>,
    pub notes: BTreeMap<String, Value>,
    /// feat-data-dictionary（S07）：字典随既有 op 通路上行（扁平集合，同 area/note 口径）。
    pub dictionaries: BTreeMap<String, Value>,
}

fn table_scalars_json(table: &Table) -> Value {
    let mut v = serde_json::to_value(table).unwrap_or(Value::Null);
    if let Some(obj) = v.as_object_mut() {
        obj.remove("fields");
    }
    v
}

pub fn capture_entity_snapshot(store: &EditorStore) -> EntitySnapshot {
    let mut snap = EntitySnapshot::default();
    for t in store.tables.get_untracked() {
        snap.tables.insert(t.id.clone(), table_scalars_json(&t));
        snap.field_order
            .insert(t.id.clone(), t.fields.iter().map(|f| f.id.clone()).collect());
        for f in &t.fields {
            snap.fields.insert(
                f.id.clone(),
                (t.id.clone(), serde_json::to_value(f).unwrap_or(Value::Null)),
            );
        }
    }
    for r in store.references.get_untracked() {
        snap.references
            .insert(r.id.clone(), serde_json::to_value(r).unwrap_or(Value::Null));
    }
    for a in store.areas.get_untracked() {
        snap.areas
            .insert(a.id.clone(), serde_json::to_value(a).unwrap_or(Value::Null));
    }
    for n in store.notes.get_untracked() {
        snap.notes
            .insert(n.id.clone(), serde_json::to_value(n).unwrap_or(Value::Null));
    }
    for d in store.dictionaries.get_untracked() {
        snap.dictionaries
            .insert(d.id.clone(), serde_json::to_value(d).unwrap_or(Value::Null));
    }
    snap
}

// ---------------------------------------------------------------------------
// 纯逻辑：快照 diff → CollabOp（UT-FE-S05-13）
// ---------------------------------------------------------------------------

fn op_create(kind: &str, target_id: &str, parent_id: Option<&str>, changes: Value) -> Value {
    let mut op = json!({ "type": format!("{kind}.create"), "targetId": target_id, "changes": changes });
    if let Some(p) = parent_id {
        op["parentId"] = json!(p);
    }
    op
}

fn op_update(kind: &str, target_id: &str, parent_id: Option<&str>, changes: Value) -> Value {
    let mut op = json!({ "type": format!("{kind}.update"), "targetId": target_id, "changes": changes });
    if let Some(p) = parent_id {
        op["parentId"] = json!(p);
    }
    op
}

fn op_delete(kind: &str, target_id: &str, parent_id: Option<&str>) -> Value {
    let mut op = json!({ "type": format!("{kind}.delete"), "targetId": target_id });
    if let Some(p) = parent_id {
        op["parentId"] = json!(p);
    }
    op
}

/// 计算 prev → next 的 op 序列（确定性顺序：table → field → reference → area → note，
/// 每组内 create → update → delete）。table.create 的 changes 携带**含字段的**完整表 JSON，
/// 因此新建/删除表当次 diff 中其字段不再单独产生 field.* op。
pub fn diff_snapshots(prev: &EntitySnapshot, next: &EntitySnapshot, next_full_tables: &BTreeMap<String, Value>) -> Vec<Value> {
    let mut ops = Vec::new();

    // table：scalars 比较；create 时用全量（含 fields）
    for (id, next_v) in &next.tables {
        match prev.tables.get(id) {
            None => {
                let full = next_full_tables.get(id).cloned().unwrap_or_else(|| next_v.clone());
                ops.push(op_create("table", id, None, full));
            }
            Some(prev_v) if prev_v != next_v => ops.push(op_update("table", id, None, next_v.clone())),
            _ => {}
        }
    }
    let mut deleted_tables: Vec<String> = Vec::new();
    for id in prev.tables.keys() {
        if !next.tables.contains_key(id) {
            ops.push(op_delete("table", id, None));
            deleted_tables.push(id.clone());
        }
    }
    let created_table = |tid: &str| !prev.tables.contains_key(tid) && next.tables.contains_key(tid);
    let deleted_table = |tid: &str| deleted_tables.iter().any(|d| d == tid);

    // field：父表本次被整体 create/delete 时跳过（已由 table op 覆盖）
    for (fid, (tid, next_v)) in &next.fields {
        if created_table(tid) {
            continue;
        }
        match prev.fields.get(fid) {
            None => ops.push(op_create("field", fid, Some(tid), next_v.clone())),
            Some((_, prev_v)) if prev_v != next_v => ops.push(op_update("field", fid, Some(tid), next_v.clone())),
            _ => {}
        }
    }
    for (fid, (tid, _)) in &prev.fields {
        if next.fields.contains_key(fid) || created_table(tid) || deleted_table(tid) {
            continue;
        }
        ops.push(op_delete("field", fid, Some(tid)));
    }

    // fields.reorder（fix-collab-autosave-race）：同一表（非整表 create/delete）字段 id
    // 序列变化 → 携带全量目标序。排在 field create/update/delete 之后，同窗口增删+换序
    // 合并时服务端先应用增删再按目标序收敛（列表外字段保持原相对序追加，幂等）。
    for (tid, next_order) in &next.field_order {
        if created_table(tid) || deleted_table(tid) {
            continue;
        }
        let Some(prev_order) = prev.field_order.get(tid) else {
            continue;
        };
        if prev_order != next_order {
            ops.push(json!({
                "type": "fields.reorder",
                "targetId": tid,
                "changes": { "fieldIds": next_order },
            }));
        }
    }

    // reference / area / note / dict：扁平集合
    for (kind, prev_map, next_map) in [
        ("reference", &prev.references, &next.references),
        ("area", &prev.areas, &next.areas),
        ("note", &prev.notes, &next.notes),
        ("dict", &prev.dictionaries, &next.dictionaries),
    ] {
        for (id, next_v) in next_map {
            match prev_map.get(id) {
                None => ops.push(op_create(kind, id, None, next_v.clone())),
                Some(prev_v) if prev_v != next_v => ops.push(op_update(kind, id, None, next_v.clone())),
                _ => {}
            }
        }
        for id in prev_map.keys() {
            if !next_map.contains_key(id) {
                ops.push(op_delete(kind, id, None));
            }
        }
    }

    ops
}

/// 构造上行 op 帧（collab.yaml CollabFrameOpClient）。
pub fn build_op_frame(client_rev: i64, op: &Value) -> String {
    json!({ "type": "op", "clientRev": client_rev, "op": op }).to_string()
}

/// 构造 sync 请求帧（S05.4）。
pub fn build_sync_frame(last_rev: i64) -> String {
    json!({ "type": "sync", "lastRev": last_rev }).to_string()
}

/// 构造 presence 帧（节流由调用方负责）。
pub fn build_presence_frame(user_id: &str, x: f64, y: f64) -> String {
    json!({ "type": "presence", "userId": user_id, "cursor": { "x": x, "y": y } }).to_string()
}

// ---------------------------------------------------------------------------
// 纯逻辑：remote_op 应用（UT-FE-S05-14）
// ---------------------------------------------------------------------------

fn upsert_by_id<T: Clone + PartialEq>(items: &mut Vec<T>, id_of: impl Fn(&T) -> &str, item: T, id: &str) {
    if let Some(slot) = items.iter_mut().find(|e| id_of(e) == id) {
        *slot = item;
    } else {
        items.push(item);
    }
}

/// 将远端 op 应用到 store。返回是否实际产生变更。
/// 不产生回声由调用方保证（应用后立即推进 diff baseline，见 CollabController::apply_remote）。
pub fn apply_op_to_store(store: &EditorStore, op: &Value) -> bool {
    let Some(op_type) = op.get("type").and_then(|v| v.as_str()) else {
        return false;
    };
    let target_id = op.get("targetId").and_then(|v| v.as_str()).unwrap_or_default();
    if target_id.is_empty() {
        return false;
    }
    let parent_id = op.get("parentId").and_then(|v| v.as_str());
    let changes = op.get("changes").cloned().unwrap_or(Value::Null);
    let mut parts = op_type.splitn(2, '.');
    let (kind, action) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));

    match (kind, action) {
        ("table", "create") | ("table", "update") => {
            // update 的 changes 不含 fields（diff 约定）；create 含完整 fields。
            let Ok(mut incoming) = serde_json::from_value::<Table>(changes.clone()) else {
                // changes 缺 fields（table.update 标量载荷）：合并标量到既有表
                if action == "update" {
                    let mut applied = false;
                    store.tables.update(|tables| {
                        if let Some(t) = tables.iter_mut().find(|t| t.id == target_id) {
                            applied = merge_table_scalars(t, &changes);
                        }
                    });
                    return applied;
                }
                return false;
            };
            incoming.id = target_id.to_string();
            store.tables.update(|tables| {
                if let Some(slot) = tables.iter_mut().find(|t| t.id == target_id) {
                    if changes.get("fields").is_some() {
                        *slot = incoming.clone();
                    } else {
                        merge_table_scalars(slot, &changes);
                    }
                } else {
                    tables.push(incoming.clone());
                }
            });
            true
        }
        ("table", "delete") => {
            let before = store.tables.with_untracked(|ts| ts.len());
            store.tables.update(|tables| tables.retain(|t| t.id != target_id));
            store.tables.with_untracked(|ts| ts.len()) != before
        }
        ("field", "create") | ("field", "update") => {
            let Some(tid) = parent_id else { return false };
            let Ok(mut field) = serde_json::from_value::<Field>(changes) else {
                return false;
            };
            field.id = target_id.to_string();
            let mut applied = false;
            store.tables.update(|tables| {
                if let Some(t) = tables.iter_mut().find(|t| t.id == tid) {
                    upsert_by_id(&mut t.fields, |f| &f.id, field.clone(), target_id);
                    applied = true;
                }
            });
            applied
        }
        ("field", "delete") => {
            let Some(tid) = parent_id else { return false };
            let mut applied = false;
            store.tables.update(|tables| {
                if let Some(t) = tables.iter_mut().find(|t| t.id == tid) {
                    let before = t.fields.len();
                    t.fields.retain(|f| f.id != target_id);
                    applied = t.fields.len() != before;
                }
            });
            applied
        }
        ("fields", "reorder") => {
            // fix-collab-autosave-race：targetId=表 id，changes.fieldIds=全量目标序；
            // 列表外字段保持原相对序追加末尾（与后端 apply_op_to_doc 同规则，幂等）。
            let ids: Vec<String> = changes
                .get("fieldIds")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let mut applied = false;
            store.tables.update(|tables| {
                if let Some(t) = tables.iter_mut().find(|t| t.id == target_id) {
                    let pos = |fid: &str| ids.iter().position(|i| i == fid).unwrap_or(ids.len());
                    let before: Vec<String> = t.fields.iter().map(|f| f.id.clone()).collect();
                    t.fields.sort_by(|a, b| pos(&a.id).cmp(&pos(&b.id)));
                    applied = t.fields.iter().map(|f| &f.id).ne(before.iter());
                }
            });
            applied
        }
        ("reference", action) => apply_flat(
            &store.references,
            action,
            target_id,
            changes,
            |r: &Reference| r.id.clone(),
        ),
        ("area", action) => apply_flat(&store.areas, action, target_id, changes, |a: &Area| {
            a.id.clone()
        }),
        ("note", action) => apply_flat(&store.notes, action, target_id, changes, |n: &Note| {
            n.id.clone()
        }),
        // feat-data-dictionary（S07）：dict.* op 应用到 store.dictionaries（同 area/note 扁平口径）
        ("dict", action) => apply_flat(
            &store.dictionaries,
            action,
            target_id,
            changes,
            |d: &DataDictionary| d.id.clone(),
        ),
        _ => false,
    }
}

/// table.update 标量合并：只覆盖 changes 中出现的键，不触碰 fields。
fn merge_table_scalars(t: &mut Table, changes: &Value) -> bool {
    let Some(obj) = changes.as_object() else { return false };
    let mut dirty = false;
    if let Some(v) = obj.get("name").and_then(|v| v.as_str()) {
        if t.name != v {
            t.name = v.to_string();
            dirty = true;
        }
    }
    if let Some(v) = obj.get("x").and_then(|v| v.as_f64()) {
        if t.x != v {
            t.x = v;
            dirty = true;
        }
    }
    if let Some(v) = obj.get("y").and_then(|v| v.as_f64()) {
        if t.y != v {
            t.y = v;
            dirty = true;
        }
    }
    if let Some(v) = obj.get("color").and_then(|v| v.as_str()) {
        if t.color != v {
            t.color = v.to_string();
            dirty = true;
        }
    }
    if let Some(v) = obj.get("comment").and_then(|v| v.as_str()) {
        if t.comment != v {
            t.comment = v.to_string();
            dirty = true;
        }
    }
    if let Some(v) = obj.get("width") {
        let w = v.as_u64().map(|n| n as u32);
        if t.width != w {
            t.width = w;
            dirty = true;
        }
    }
    if let Some(v) = obj.get("min_height") {
        let h = v.as_u64().map(|n| n as u32);
        if t.min_height != h {
            t.min_height = h;
            dirty = true;
        }
    }
    dirty
}

fn apply_flat<T>(
    signal: &RwSignal<Vec<T>>,
    action: &str,
    target_id: &str,
    changes: Value,
    id_of: impl Fn(&T) -> String,
) -> bool
where
    T: Clone + PartialEq + serde::de::DeserializeOwned,
{
    match action {
        "create" | "update" => {
            let Ok(item) = serde_json::from_value::<T>(changes) else {
                return false;
            };
            signal.update(|items| {
                if let Some(slot) = items.iter_mut().find(|e| id_of(e) == target_id) {
                    *slot = item.clone();
                } else {
                    items.push(item.clone());
                }
            });
            true
        }
        "delete" => {
            let before = signal.with_untracked(|items| items.len());
            signal.update(|items| items.retain(|e| id_of(e) != target_id));
            signal.with_untracked(|items| items.len()) != before
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// 纯逻辑：重连退避（UT-FE-S05-17）与 viewer 拦截（UT-FE-S05-16）
// ---------------------------------------------------------------------------

/// 指数退避重连策略：最多 5 次，1s/2s/4s/8s/16s（S05.4「最多 5 次」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReconnectPolicy {
    pub attempts: u32,
}

impl ReconnectPolicy {
    pub const MAX_ATTEMPTS: u32 = 5;
    pub const DELAYS_MS: [u32; 5] = [1000, 2000, 4000, 8000, 16000];

    /// 下一次重连延迟；耗尽返回 None（进入 failed 态）。
    pub fn next_delay_ms(&self) -> Option<u32> {
        Self::DELAYS_MS.get(self.attempts as usize).copied()
    }

    pub fn record_attempt(&mut self) {
        self.attempts += 1;
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

/// watcher 是否应为本地变更生成 op：viewer（ReadOnly）一律拦截（UT-FE-S05-16）；
/// 其余状态（含 Reconnecting/Offline）生成并进入离线队列。
pub fn watcher_active(state: &CollabOtState) -> bool {
    !matches!(state.connection, CollabConnectionState::ReadOnly)
}

/// 是否立即上行（否则入离线队列，待 sync 后 flush）。
pub fn should_send_now(state: &CollabOtState) -> bool {
    matches!(state.connection, CollabConnectionState::Connected)
}

/// fix-collab-autosave-race（UT-FE-S05-19）：全部 op 已 ack 且无离线积压 → 视为「已保存」
/// （room 模式 ops 即保存，dirty 由 op 对账驱动而非全量 PUT）。
pub fn ops_fully_synced(state: &CollabOtState) -> bool {
    state.pending_ops.is_empty() && state.queued_while_offline.is_empty()
}

/// fix-collab-autosave-race（UT-FE-S05-20）：RemoteOp 有序收编决策。
/// 服务端 op log 是唯一权威序（server_rev 严格递增）；客户端必须按序应用，
/// 否则「入房/重连 sync 在途」与「广播到达」交错时会用旧状态覆盖新状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteOpAction {
    /// rev == cur+1：直接应用。
    Apply,
    /// rev <= cur：与已同步内容重复（sync 回放与广播重叠）→ 丢弃。
    SkipStale,
    /// sync 在途：先入缓冲，待 sync 完成后有序回放。
    Buffer,
    /// rev > cur+1：存在缺口（如入房窗口错过广播）→ 缓冲并发起 sync 补齐。
    BufferAndSync,
}

pub fn plan_remote_op(cur_rev: i64, op_rev: i64, sync_in_flight: bool) -> RemoteOpAction {
    if sync_in_flight {
        return RemoteOpAction::Buffer;
    }
    if op_rev <= cur_rev {
        RemoteOpAction::SkipStale
    } else if op_rev == cur_rev + 1 {
        RemoteOpAction::Apply
    } else {
        RemoteOpAction::BufferAndSync
    }
}

/// fix-collab-autosave-race（UT-FE-S05-20）：sync 完成后回放缓冲的广播 op。
/// 返回 (可顺序应用的 op 载荷, 仍需等待的剩余缓冲, 是否仍有缺口需再次 sync)。
/// 输入允许乱序/重复（缓冲期间到达的广播与 sync 回放可能重叠）。
pub fn drain_buffered_ops(
    mut buffered: Vec<(i64, Value)>,
    server_rev: i64,
) -> (Vec<Value>, Vec<(i64, Value)>, bool) {
    buffered.sort_by_key(|(rev, _)| *rev);
    buffered.dedup_by_key(|(rev, _)| *rev);
    let mut rev = server_rev;
    let mut apply = Vec::new();
    let mut rest = Vec::new();
    let mut gap = false;
    for (r, op) in buffered {
        if gap {
            rest.push((r, op));
            continue;
        }
        if r <= rev {
            continue; // sync 回放已覆盖
        }
        if r == rev + 1 {
            apply.push(op);
            rev = r;
        } else {
            gap = true;
            rest.push((r, op));
        }
    }
    (apply, rest, gap)
}

// ---------------------------------------------------------------------------
// wasm 传输层
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod wasm_transport {
    use super::*;
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    use gloo_timers::callback::Timeout;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

    use crate::editor_data_access::{parse_collab_frame, CollabFrame, CollabMemberPresence};
    use crate::editor_render::RemotePresence;

    /// web_sys WebSocket 薄封装：回调以 Rc 注入，关闭时释放闭包打断引用环。
    pub struct CollabWs {
        ws: WebSocket,
        _on_open: Closure<dyn FnMut()>,
        _on_message: Closure<dyn FnMut(MessageEvent)>,
        _on_close: Closure<dyn FnMut(CloseEvent)>,
        _on_error: Closure<dyn FnMut(ErrorEvent)>,
    }

    impl CollabWs {
        pub fn connect(
            url: &str,
            on_open: Rc<dyn Fn()>,
            on_message: Rc<dyn Fn(String)>,
            on_close: Rc<dyn Fn(Option<u16>)>,
        ) -> Result<Self, String> {
            let ws = WebSocket::new(url).map_err(|e| format!("ws connect: {e:?}"))?;

            let cb_open = Closure::wrap(Box::new(move || on_open()) as Box<dyn FnMut()>);
            ws.set_onopen(Some(cb_open.as_ref().unchecked_ref()));

            let cb_message = Closure::wrap(Box::new(move |ev: MessageEvent| {
                if let Some(text) = ev.data().as_string() {
                    on_message(text);
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(cb_message.as_ref().unchecked_ref()));

            let cb_close = Closure::wrap(Box::new(move |ev: CloseEvent| {
                on_close(Some(ev.code()));
            }) as Box<dyn FnMut(CloseEvent)>);
            ws.set_onclose(Some(cb_close.as_ref().unchecked_ref()));

            let on_close_err = Rc::new(|| {});
            let _ = on_close_err;
            let cb_error = Closure::wrap(Box::new(move |_ev: ErrorEvent| {}) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(cb_error.as_ref().unchecked_ref()));

            Ok(Self {
                ws,
                _on_open: cb_open,
                _on_message: cb_message,
                _on_close: cb_close,
                _on_error: cb_error,
            })
        }

        pub fn send(&self, text: &str) -> Result<(), String> {
            self.ws
                .send_with_str(text)
                .map_err(|e| format!("ws send: {e:?}"))
        }

        pub fn close(&self) {
            let _ = self.ws.close();
        }
    }

    impl Drop for CollabWs {
        fn drop(&mut self) {
            let _ = self.ws.close();
        }
    }

    /// ST-FE-S05-09 调试钩子：远端 presence 计数 + userId→坐标映射（画布绘点无法被 Playwright DOM 断言）。
    /// `__cdb_remote_presence` 为在线计数；`__cdb_remote_presence_map` 为 JSON 字符串 `{userId: {x, y, name}}`。
    fn record_remote_presence_debug(list: &[RemotePresence]) {
        if let Some(win) = web_sys::window() {
            let target: &js_sys::Object = win.unchecked_ref();
            let online: Vec<&RemotePresence> = list.iter().filter(|p| p.online).collect();
            let _ = js_sys::Reflect::set(
                target,
                &JsValue::from_str("__cdb_remote_presence"),
                &JsValue::from(online.len() as f64),
            );
            let mut map = serde_json::Map::new();
            for p in online {
                map.insert(
                    p.user_id.clone(),
                    json!({"x": p.x, "y": p.y, "name": p.display_name}),
                );
            }
            let _ = js_sys::Reflect::set(
                target,
                &JsValue::from_str("__cdb_remote_presence_map"),
                &JsValue::from_str(&Value::Object(map).to_string()),
            );
        }
    }

    struct CollabInner {
        ws: Option<CollabWs>,
        token: String,
        room_id: String,
        user_id: Option<String>,
        base_url: String,
        baseline: Option<EntitySnapshot>,
        /// 离线期间待 flush 的完整帧（client_rev, frame JSON）。
        /// CollabOtState 仅存 op_type 供 UI 计数，载荷由这里保存。
        queued_frames: Vec<(i64, String)>,
        /// fix-collab-autosave-race：已发送未 ack 的在途帧。WS 断开/被替换时在途帧
        /// 可能从未到达服务端——close/replace 时回队重发（op 物化幂等，重投安全；
        /// ST-PC-06 间歇丢 op 的根因修复）。
        inflight_frames: Vec<(i64, String)>,
        debounce: Option<Timeout>,
        reconnect_timer: Option<Timeout>,
        policy: ReconnectPolicy,
        /// 区分主动 disconnect（房间退出）与异常断线（触发重连）。
        manual_close: bool,
        /// 已连上过（区分首次 connected 与重连 connected → 重连需 sync）。
        ever_connected: bool,
        last_presence_sent_ms: f64,
        /// fix-collab-autosave-race：sync 请求在途（首次入房补齐 / 重连 / 缺口补齐），
        /// 期间到达的广播 op 先入 buffered_remote_ops，sync 完成后有序回放。
        sync_in_flight: bool,
        /// sync 在途期间缓冲的广播 op（server_rev, op 载荷）。
        buffered_remote_ops: Vec<(i64, Value)>,
        /// 调试用事件环（仅 debug 构建记录；__cdb_collab_state 导出）。
        debug_events: Vec<String>,
    }

    impl Default for CollabInner {
        fn default() -> Self {
            Self {
                ws: None,
                token: String::new(),
                room_id: String::new(),
                user_id: None,
                base_url: String::new(),
                baseline: None,
                queued_frames: Vec::new(),
                inflight_frames: Vec::new(),
                debounce: None,
                reconnect_timer: None,
                policy: ReconnectPolicy::default(),
                manual_close: false,
                ever_connected: false,
                last_presence_sent_ms: 0.0,
                sync_in_flight: false,
                buffered_remote_ops: Vec::new(),
                debug_events: Vec::new(),
            }
        }
    }

    /// S05 前端协作控制器：连接生命周期 + 帧分发 + op 上行 watcher + presence + 重连 sync。
    /// 构造后由房间进入 effect 调用 `connect`；房间退出/登出调用 `disconnect`。
    pub struct CollabController {
        inner: Rc<RefCell<CollabInner>>,
        store: EditorStore,
        collab_state: RwSignal<CollabOtState>,
        activity_feed: RwSignal<Vec<String>>,
        remote_members: RwSignal<Vec<CollabMemberPresence>>,
        remote_presence: RwSignal<Vec<RemotePresence>>,
        /// token 过期重连前由外部注入的续期回调（S03 on_refresh_session）。
        on_token_expired: Rc<dyn Fn()>,
        /// SYNC_GAP_TOO_LARGE 时由外部注入的全量重同步（REST 重载 diagram）。
        on_full_resync: Rc<dyn Fn()>,
    }

    impl CollabController {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            store: EditorStore,
            collab_state: RwSignal<CollabOtState>,
            activity_feed: RwSignal<Vec<String>>,
            remote_members: RwSignal<Vec<CollabMemberPresence>>,
            remote_presence: RwSignal<Vec<RemotePresence>>,
            on_token_expired: Rc<dyn Fn()>,
            on_full_resync: Rc<dyn Fn()>,
        ) -> Rc<Self> {
            Rc::new(Self {
                inner: Rc::new(RefCell::new(CollabInner::default())),
                store,
                collab_state,
                activity_feed,
                remote_members,
                remote_presence,
                on_token_expired,
                on_full_resync,
            })
        }

        fn activity(&self, item: String) {
            crate::editor_panels::prepend_activity(self.activity_feed, item);
        }

        /// 建立 WS 连接（或重连）。token/room/user 每次覆盖，支持 token 刷新后重连。
        pub fn connect(self: &Rc<Self>, base_url: &str, room_id: &str, token: &str, user_id: Option<String>) {
            // 替换仍活跃的 socket 前：其在途未 ack 帧可能从未到达服务端，回队重发
            // （旧 socket 随 Drop 关闭且闭包失效，onclose 不会再触发，只能在这里兜底）。
            if self.inner.borrow().ws.is_some() {
                self.requeue_inflight();
            }
            {
                let mut inner = self.inner.borrow_mut();
                inner.manual_close = false;
                inner.base_url = base_url.to_string();
                inner.room_id = room_id.to_string();
                inner.token = token.to_string();
                inner.user_id = user_id;
                inner.ws = None; // 旧连接随 Drop 关闭
                // fix-collab-autosave-race：新连接重置 sync 收编状态。缓冲 op 从未推进
                // server_rev，丢弃后由新连接的 sync（last_rev=server_rev）重新补拉，不丢。
                inner.sync_in_flight = false;
                inner.buffered_remote_ops.clear();
            }
            set_active_controller(Some(self));
            self.collab_state.update(|s| {
                s.connection = CollabConnectionState::Connecting;
            });

            let url = crate::editor_data_access::build_ws_url(base_url, room_id, token);
            let weak = Rc::downgrade(self);
            let upgrade = move |weak: &Weak<CollabController>| -> Option<Rc<CollabController>> {
                weak.upgrade()
            };

            let on_open: Rc<dyn Fn()> = {
                let weak = weak.clone();
                Rc::new(move || {
                    if let Some(this) = upgrade(&weak) {
                        this.on_ws_open();
                    }
                })
            };
            let on_message: Rc<dyn Fn(String)> = {
                let weak = weak.clone();
                Rc::new(move |text| {
                    if let Some(this) = upgrade(&weak) {
                        this.handle_frame_text(&text);
                    }
                })
            };
            let on_close: Rc<dyn Fn(Option<u16>)> = {
                let weak = weak.clone();
                Rc::new(move |code| {
                    if let Some(this) = upgrade(&weak) {
                        this.on_ws_close(code);
                    }
                })
            };

            match CollabWs::connect(&url, on_open, on_message, on_close) {
                Ok(ws) => self.inner.borrow_mut().ws = Some(ws),
                Err(e) => {
                    self.activity(format!("协作连接失败 · {e}"));
                    self.schedule_reconnect();
                }
            }
        }

        /// 房间退出/登出：主动关闭，不触发重连。
        pub fn disconnect(&self) {
            self.dbg_event("disconnect".to_string());
            let mut inner = self.inner.borrow_mut();
            inner.manual_close = true;
            inner.ws = None;
            inner.debounce = None;
            inner.reconnect_timer = None;
            inner.baseline = None;
            inner.queued_frames.clear();
            inner.inflight_frames.clear();
            inner.policy.reset();
            inner.ever_connected = false;
            drop(inner);
            set_active_controller(None);
            self.collab_state.set(CollabOtState::default());
        }

        fn on_ws_open(&self) {
            // connected 帧到达前不置 Connected（以服务端权威帧为准）
        }

        fn on_ws_close(self: &Rc<Self>, code: Option<u16>) {
            if self.inner.borrow().manual_close {
                return;
            }
            // 连接终止：在途未 ack 帧回队重发（可能从未到达服务端）；
            // sync 收编状态随连接失效（缓冲 op 未推进 server_rev，
            // 重连后的 sync 会重新补拉，不丢）。
            self.requeue_inflight();
            self.inner.borrow_mut().sync_in_flight = false;
            self.inner.borrow_mut().buffered_remote_ops.clear();
            // 4401：token 失效 → 先续期（回调内部更新 session），再重连
            if code == Some(4401) {
                (self.on_token_expired)();
            }
            self.activity("协作连接已断开，正在重连".to_string());
            self.schedule_reconnect();
        }

        fn schedule_reconnect(self: &Rc<Self>) {
            let delay = {
                let mut inner = self.inner.borrow_mut();
                if inner.manual_close {
                    return;
                }
                let delay = inner.policy.next_delay_ms();
                if delay.is_some() {
                    inner.policy.record_attempt();
                }
                delay
            };
            match delay {
                Some(ms) => {
                    self.collab_state.update(|s| s.mark_reconnecting());
                    let weak = Rc::downgrade(self);
                    let timer = Timeout::new(ms, move || {
                        if let Some(this) = weak.upgrade() {
                            let (base, room, token, uid) = {
                                let inner = this.inner.borrow();
                                (
                                    inner.base_url.clone(),
                                    inner.room_id.clone(),
                                    inner.token.clone(),
                                    inner.user_id.clone(),
                                )
                            };
                            this.connect(&base, &room, &token, uid);
                        }
                    });
                    self.inner.borrow_mut().reconnect_timer = Some(timer);
                }
                None => {
                    // 5 次失败：failed 态（UI Banner 提供「刷新 / 仅本地编辑」），队列保留
                    self.collab_state.update(|s| {
                        s.connection = CollabConnectionState::Offline;
                    });
                    self.activity("协作连接失败已达 5 次，可刷新或仅本地编辑（有 409 风险）".to_string());
                }
            }
        }

        /// 本地编辑入口（watcher 调用）：debounce 200ms 合并后 diff → op 上行/入队。
        pub fn notify_local_change(self: &Rc<Self>) {
            if !watcher_active(&self.collab_state.get_untracked()) {
                return;
            }
            if self.inner.borrow().baseline.is_none() {
                return; // 未 connected：无 baseline，不产生 op
            }
            let weak = Rc::downgrade(self);
            let timer = Timeout::new(200, move || {
                if let Some(this) = weak.upgrade() {
                    this.emit_local_ops();
                }
            });
            self.inner.borrow_mut().debounce = Some(timer);
        }

        #[cfg(debug_assertions)]
        fn dbg_event(&self, msg: impl Into<String>) {
            let mut inner = self.inner.borrow_mut();
            if inner.debug_events.len() >= 64 {
                inner.debug_events.remove(0);
            }
            inner.debug_events.push(msg.into());
        }

        #[cfg(not(debug_assertions))]
        #[inline(always)]
        fn dbg_event(&self, _msg: impl Into<String>) {}

        fn emit_local_ops(self: &Rc<Self>) {
            let current = capture_entity_snapshot(&self.store);
            let ops = {
                let inner = self.inner.borrow();
                let Some(baseline) = &inner.baseline else { return };
                let full_tables = full_table_map(&self.store);
                diff_snapshots(baseline, &current, &full_tables)
            };
            if ops.is_empty() {
                // fix-collab-autosave-race：不产生实体 diff 的变更（选中/悬停等）也会
                // 把 dirty 置 true；方案B 下 dirty 语义 = 「有未同步 op」，diff 为空且
                // 无未 ack/积压 op 时回收 dirty，避免保存指示卡死在「有未保存更改」。
                if ops_fully_synced(&self.collab_state.get_untracked()) {
                    self.store.dirty.set(false);
                }
                return;
            }
            for op in ops {
                let op_type = op.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let mut state = self.collab_state.get_untracked();
                let Some(client_rev) = state.enqueue_local_op(op_type.clone()) else {
                    continue; // ReadOnly 拦截
                };
                self.dbg_event(format!(
                    "enqueue rev={} type={} conn={:?}",
                    client_rev, op_type, state.connection
                ));
                let frame = build_op_frame(client_rev, &op);
                if should_send_now(&state) {
                    let sent = self
                        .inner
                        .borrow()
                        .ws
                        .as_ref()
                        .map(|ws| ws.send(&frame))
                        .unwrap_or(Err("no ws".into()));
                    if sent.is_err() {
                        self.dbg_event(format!("send-err rev={}", client_rev));
                        self.inner
                            .borrow_mut()
                            .queued_frames
                            .push((client_rev, frame));
                    } else {
                        self.dbg_event(format!("send-ok rev={}", client_rev));
                        // 在途登记：ack 到达才移除；断连/替换时回队重发
                        self.inner
                            .borrow_mut()
                            .inflight_frames
                            .push((client_rev, frame));
                    }
                } else {
                    self.dbg_event(format!("queue rev={}", client_rev));
                    self.inner
                        .borrow_mut()
                        .queued_frames
                        .push((client_rev, frame));
                }
                self.collab_state.set(state);
            }
            self.inner.borrow_mut().baseline = Some(current);
        }

        /// fix-collab-autosave-race：远端帧（remote_op/sync/全量重同步）应用并推进/重采
        /// baseline 之前，必须先把 debounce 窗口内尚未 diff 上行的本地编辑结算成 op——
        /// 否则 baseline 重采会把这些编辑吞掉：op 永远不发，后续依赖它的 op 上行时服务端
        /// 文档里根本没有目标实体（ST-CR-INSP-01 间歇失败根因：入房后 200ms debounce
        /// 窗口内按 T 建表，sync 往返先于 debounce 完成 → table.create 被吞）。
        /// emit_local_ops 空 diff 时直接返回，无本地编辑时调用无害。
        fn flush_pending_local_ops(self: &Rc<Self>) {
            // 取消待触发的 debounce 定时器（Drop 即取消），立即同步结算一次 diff+上行
            self.inner.borrow_mut().debounce = None;
            self.emit_local_ops();
        }

        /// 远端 op 应用 + baseline 推进（防回声：watcher 下次 diff 为空）。
        fn apply_remote(&self, op: &Value) -> bool {
            let applied = apply_op_to_store(&self.store, op);
            if applied {
                self.inner.borrow_mut().baseline = Some(capture_entity_snapshot(&self.store));
            }
            applied
        }

        /// fix-collab-autosave-race：发起 sync 补拉（last_rev 之后的 op），并置 sync_in_flight。
        /// 在途期间到达的广播 op 由 RemoteOp 处理器缓冲，sync 完成后有序回放。
        fn request_sync(self: &Rc<Self>, last_rev: i64) {
            let frame = build_sync_frame(last_rev);
            let sent = self
                .inner
                .borrow()
                .ws
                .as_ref()
                .map(|ws| ws.send(&frame))
                .unwrap_or(Err("no ws".into()));
            if sent.is_err() {
                self.schedule_reconnect();
                return;
            }
            self.inner.borrow_mut().sync_in_flight = true;
        }

        fn handle_frame_text(self: &Rc<Self>, text: &str) {
            let Ok(frame) = parse_collab_frame(text) else {
                return;
            };
            match frame {
                CollabFrame::Connected {
                    server_rev,
                    members,
                    your_role,
                    ..
                } => {
                    let is_reconnect = self.inner.borrow().ever_connected;
                    self.dbg_event(format!("connected sr={} reconnect={}", server_rev, is_reconnect));
                    self.inner.borrow_mut().ever_connected = true;
                    self.inner.borrow_mut().policy.reset();
                    let role = your_role.clone().unwrap_or_default();
                    // fix-collab-autosave-race：保留离线队列与 pending（首连前/断线期间的
                    // 本地编辑 + 在途未 ack 重发），由 sync 后的 flush_queue 补发；
                    // 在途帧已在 on_ws_close/connect 时回队，ack 按 client_rev 对账。
                    // last_rev 必须先读（断连前已知 rev）再更新 server_rev——否则重连
                    // sync 恒为空、断线期间的远端 op 永远补不回来。
                    let last_rev = self.collab_state.get_untracked().server_rev;
                    self.collab_state.update(|s| {
                        s.connection = CollabConnectionState::Connected;
                        s.server_rev = s.server_rev.max(server_rev);
                    });
                    if role == "viewer" {
                        self.collab_state.update(|s| s.mark_read_only());
                    }
                    self.remote_members.set(members);
                    if is_reconnect {
                        // S05.4：重连后 sync 补发 missed ops
                        self.request_sync(last_rev);
                    } else {
                        // baseline 在入房加载完成时已建立（editor_panels 入房路径调
                        // refresh_baseline）；首连前的本地编辑已 diff 入离线队列——
                        // 此处不得重采 baseline（会把这些编辑吞掉）。None 兜底仅防御。
                        if self.inner.borrow().baseline.is_none() {
                            self.inner.borrow_mut().baseline = Some(capture_entity_snapshot(&self.store));
                        }
                        // fix-collab-autosave-race：入房「先加载后连 WS」存在提交窗口——
                        // REST 全量加载（revision=doc_rev）之后、WS 建连（head=server_rev）
                        // 之前提交的 op 早于本连接，永远不会被广播补发，新加入者将永久丢失
                        // （ST-FE-S05-10 复现）。首次 connected 主动 sync 补齐 (doc_rev, server_rev]，
                        // 与 op 落库即物化共用同一权威序。
                        let doc_rev = self.store.revision.get_untracked();
                        if server_rev > doc_rev {
                            self.request_sync(doc_rev);
                        } else {
                            // 无需补齐：直接收编离线队列（首连前的本地编辑）并 flush。
                            self.collab_state.update(|s| s.apply_sync(server_rev));
                            self.flush_queue();
                        }
                    }
                    self.activity(format!("协作已连接 · rev {server_rev}"));
                }
                CollabFrame::Ack {
                    server_rev,
                    client_rev,
                    ..
                } => {
                    self.collab_state.update(|s| s.ack(client_rev, server_rev));
                    // 在途帧对账：ack 到达即移除（未 ack 的在断连/替换时回队重发）
                    if let Some(cr) = client_rev {
                        self.inner
                            .borrow_mut()
                            .inflight_frames
                            .retain(|(rev, _)| *rev != cr);
                    }
                    self.dbg_event(format!(
                        "ack cr={:?} sr={} pending={:?}",
                        client_rev,
                        server_rev,
                        self.collab_state
                            .get_untracked()
                            .pending_ops
                            .iter()
                            .map(|p| p.client_rev)
                            .collect::<Vec<_>>()
                    ));
                    // fix-collab-autosave-race：ops 即保存——全部 op 已 ack 且无离线积压
                    // → 等价于「已保存」，清除 dirty（保存指示由 op 对账驱动）。
                    if ops_fully_synced(&self.collab_state.get_untracked()) {
                        self.store.dirty.set(false);
                    }
                }
                CollabFrame::RemoteOp {
                    server_rev, op, ..
                } => {
                    // fix-collab-autosave-race：apply_remote 会重采 baseline——先结算
                    // debounce 窗口内的本地编辑，防止被吞（见 flush_pending_local_ops）。
                    self.flush_pending_local_ops();
                    let op_type = op.get("type").and_then(|v| v.as_str()).unwrap_or("?").to_string();
                    let cur = self.collab_state.get_untracked().server_rev;
                    let in_flight = self.inner.borrow().sync_in_flight;
                    match plan_remote_op(cur, server_rev, in_flight) {
                        RemoteOpAction::SkipStale => {}
                        RemoteOpAction::Buffer => {
                            self.inner.borrow_mut().buffered_remote_ops.push((server_rev, op));
                        }
                        RemoteOpAction::BufferAndSync => {
                            self.inner.borrow_mut().buffered_remote_ops.push((server_rev, op));
                            self.request_sync(cur);
                        }
                        RemoteOpAction::Apply => {
                            if self.apply_remote(&op) {
                                self.activity(format!("远端更新 · {op_type}"));
                            }
                            // 有序收编下无论 apply 是否命中都必须推进 server_rev：
                            // 该 op 已提交进服务端 op log（权威序），滞留 rev 会把后续
                            // 每条广播都误判成缺口 → 无限 sync。
                            self.collab_state.update(|s| s.server_rev = server_rev);
                        }
                    }
                }
                CollabFrame::Presence { user_id, cursor, .. } => {
                    if let Some(cursor) = cursor {
                        let x = cursor.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let y = cursor.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let name = self
                            .remote_members
                            .with_untracked(|ms| {
                                ms.iter()
                                    .find(|m| m.user_id == user_id)
                                    .and_then(|m| m.display_name.clone())
                            });
                        self.remote_presence.update(|list| {
                            if let Some(p) = list.iter_mut().find(|p| p.user_id == user_id) {
                                p.x = x;
                                p.y = y;
                                p.online = true;
                            } else {
                                list.push(RemotePresence {
                                    user_id: user_id.clone(),
                                    display_name: name,
                                    x,
                                    y,
                                    online: true,
                                });
                            }
                            record_remote_presence_debug(list);
                        });
                    }
                }
                CollabFrame::Sync {
                    server_rev,
                    ops,
                    snapshot,
                } => {
                    // fix-collab-autosave-race：sync 回放/快照会推进或重采 baseline——
                    // 先结算 debounce 窗口内的本地编辑（ST-CR-INSP-01 间歇失败根因：
                    // 入房 200ms 窗口内建表 + sync 快速往返 → table.create 曾被吞）。
                    self.flush_pending_local_ops();
                    if let Some(snapshot) = snapshot {
                        // EX-5.4：全量快照替换
                        if let Ok(diagram) = serde_json::from_value::<crate::editor_core::types::Diagram>(snapshot) {
                            self.store.load(diagram);
                            self.activity("已同步服务器版本（全量快照）".to_string());
                        }
                    } else {
                        let mut applied = 0usize;
                        for entry in &ops {
                            if self.apply_remote(&entry.payload) {
                                applied += 1;
                            }
                        }
                        if applied > 0 {
                            self.activity(format!("已同步 {applied} 条远端变更"));
                        }
                    }
                    let rev = server_rev.unwrap_or_else(|| self.collab_state.get_untracked().server_rev);
                    self.collab_state.update(|s| s.apply_sync(rev));
                    // fix-collab-autosave-race：回放 sync 在途期间缓冲的广播 op（有序去重）。
                    // 缓冲 op 与 sync 回放可能重叠（rev <= 当前 → 丢弃）；仍有缺口则保留
                    // 剩余缓冲并继续 sync，直到收敛到广播序。
                    let buffered = std::mem::take(&mut self.inner.borrow_mut().buffered_remote_ops);
                    let cur = self.collab_state.get_untracked().server_rev;
                    let (to_apply, rest, gap) = drain_buffered_ops(buffered, cur);
                    let applied = to_apply.len() as i64;
                    for op in to_apply {
                        self.apply_remote(&op);
                    }
                    if applied > 0 {
                        self.collab_state.update(|s| s.server_rev = cur + applied);
                    }
                    if gap {
                        self.inner.borrow_mut().buffered_remote_ops = rest;
                        self.request_sync(cur + applied);
                    } else {
                        self.inner.borrow_mut().sync_in_flight = false;
                    }
                    self.inner.borrow_mut().baseline = Some(capture_entity_snapshot(&self.store));
                    self.flush_queue();
                    self.activity(format!("已恢复协作 · rev {rev}"));
                }
                CollabFrame::Error { code, message } => match code.as_str() {
                    "READ_ONLY" => {
                        self.collab_state.update(|s| s.mark_read_only());
                        self.activity("只读成员，编辑不会同步".to_string());
                    }
                    "SYNC_GAP_TOO_LARGE" => {
                        // 全量重同步走 REST：收编状态清空，重载后的新 head 由后续广播/sync 收敛。
                        // fix-collab-autosave-race：重载会重采 baseline——先结算 debounce
                        // 窗口内的本地编辑成 op（op 已上行/入队则不随 store 重载丢失）。
                        self.flush_pending_local_ops();
                        self.inner.borrow_mut().sync_in_flight = false;
                        self.inner.borrow_mut().buffered_remote_ops.clear();
                        (self.on_full_resync)();
                        self.activity("变更过多，已请求全量同步".to_string());
                    }
                    "token_expired" => {
                        (self.on_token_expired)();
                    }
                    _ => {
                        self.activity(format!("协作错误 · {message}"));
                    }
                },
            }
        }

        /// sync 完成后 flush 离线队列（S05.4 Step 13）。
        /// 发送成功的帧转入 inflight（等 ack）；任一帧失败时，该帧及后续帧整体回队——
        /// 不得丢弃（旧实现 take 后 break 会静默丢帧）。
        fn flush_queue(&self) {
            let frames = std::mem::take(&mut self.inner.borrow_mut().queued_frames);
            self.dbg_event(format!("flush n={}", frames.len()));
            for (i, (client_rev, frame)) in frames.iter().enumerate() {
                let sent = self
                    .inner
                    .borrow()
                    .ws
                    .as_ref()
                    .map(|ws| ws.send(frame))
                    .unwrap_or(Err("no ws".into()));
                if sent.is_err() {
                    let mut inner = self.inner.borrow_mut();
                    let mut back: Vec<(i64, String)> = frames[i..].to_vec();
                    back.append(&mut inner.queued_frames);
                    inner.queued_frames = back;
                    drop(inner);
                    self.activity("离线 op 重发失败，已回队等待重试".to_string());
                    return;
                }
                self.inner
                    .borrow_mut()
                    .inflight_frames
                    .push((*client_rev, frame.clone()));
            }
        }

        /// fix-collab-autosave-race：WS 断开/被替换时，在途未 ack 的帧可能从未到达
        /// 服务端——回队（保持 rev 序）等待重连后 sync+flush 重发；对账簿同步把
        /// pending 挪回 queued_while_offline 前部。物化幂等，重投安全。
        fn requeue_inflight(&self) {
            let mut inner = self.inner.borrow_mut();
            if inner.inflight_frames.is_empty() {
                return;
            }
            let mut frames = std::mem::take(&mut inner.inflight_frames);
            frames.append(&mut inner.queued_frames);
            inner.queued_frames = frames;
            drop(inner);
            self.dbg_event("requeue-inflight".to_string());
            self.collab_state.update(|s| {
                let mut pend = std::mem::take(&mut s.pending_ops);
                pend.append(&mut s.queued_while_offline);
                s.queued_while_offline = pend;
            });
        }

        /// presence 上行（画布 pointermove 钩子调用），100ms 节流。
        pub fn send_presence(&self, x: f64, y: f64) {
            if !should_send_now(&self.collab_state.get_untracked()) {
                return;
            }
            let now = js_sys::Date::now();
            {
                let mut inner = self.inner.borrow_mut();
                if now - inner.last_presence_sent_ms < 100.0 {
                    return;
                }
                inner.last_presence_sent_ms = now;
            }
            let user_id = self.inner.borrow().user_id.clone().unwrap_or_default();
            if user_id.is_empty() {
                return;
            }
            let frame = build_presence_frame(&user_id, x, y);
            if let Some(ws) = self.inner.borrow().ws.as_ref() {
                let _ = ws.send(&frame);
            }
        }

        /// 当前 head server_rev。
        pub fn server_rev(&self) -> i64 {
            self.collab_state.get_untracked().server_rev
        }

        /// 当前 room_id（未连接为 None）。
        pub fn room_id(&self) -> Option<String> {
            let inner = self.inner.borrow();
            if inner.room_id.is_empty() || !inner.ever_connected {
                None
            } else {
                Some(inner.room_id.clone())
            }
        }

        /// 外部全量加载（409 重新加载 / SYNC_GAP_TOO_LARGE 重同步）后刷新 diff baseline，
        /// 防止 watcher 把整图当成「本地新增」回广播。
        pub fn refresh_baseline(&self) {
            self.inner.borrow_mut().baseline = Some(capture_entity_snapshot(&self.store));
        }

        /// 手动重连（Banner「立即重连」）：清零退避计数后再 connect。
        pub fn reset_policy(&self) {
            self.inner.borrow_mut().policy.reset();
        }
    }

    /// table 全量 JSON 映射（diff table.create 载荷用，含 fields）。
    fn full_table_map(store: &EditorStore) -> BTreeMap<String, Value> {
        store
            .tables
            .get_untracked()
            .into_iter()
            .map(|t| {
                (
                    t.id.clone(),
                    serde_json::to_value(&t).unwrap_or(Value::Null),
                )
            })
            .collect()
    }

    // ---------------------------------------------------------------------------
    // 画布光标上报钩子（editor_render pointermove 单行接入，避免组件 props 穿透）
    // ---------------------------------------------------------------------------

    thread_local! {
        static CURSOR_REPORTER: RefCell<Option<Rc<dyn Fn(f64, f64)>>> = const { RefCell::new(None) };
        /// 活跃控制器（presence_active / external_store_reloaded 用），Weak 防泄漏。
        static ACTIVE_CONTROLLER: RefCell<Option<Weak<CollabController>>> = const { RefCell::new(None) };
    }

    pub fn set_cursor_reporter(cb: Option<Rc<dyn Fn(f64, f64)>>) {
        CURSOR_REPORTER.with(|slot| *slot.borrow_mut() = cb);
    }

    /// 注册/注销活跃控制器（connect/disconnect 时调用）。
    pub fn set_active_controller(ctl: Option<&Rc<CollabController>>) {
        ACTIVE_CONTROLLER.with(|slot| *slot.borrow_mut() = ctl.map(Rc::downgrade));
        if ctl.is_some() {
            register_collab_state_hook();
        }
    }

    /// ST-FE-S05-07~09 e2e 调试钩子（仅 debug 构建）：`window.__cdb_collab_state()`
    /// 返回 JSON 字符串 `{connection, serverRev, queueLen, roomId, userId, attempts}`。
    /// WS 内部状态在 Rust 闭包内，Playwright 无法直接读取，需经此钩子暴露。
    #[cfg(debug_assertions)]
    fn register_collab_state_hook() {
        thread_local! {
            static REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        }
        if REGISTERED.with(|r| r.replace(true)) {
            return;
        }
        let cb = Closure::wrap(Box::new(|| -> JsValue {
            ACTIVE_CONTROLLER.with(|slot| {
                let Some(ctl) = slot.borrow().as_ref().and_then(|w| w.upgrade()) else {
                    return JsValue::from_str("{}");
                };
                let inner = ctl.inner.borrow();
                let state = ctl.collab_state.get_untracked();
                let json = json!({
                    "connection": format!("{:?}", state.connection),
                    "serverRev": state.server_rev,
                    "queueLen": inner.queued_frames.len(),
                    "pendingOps": state.pending_ops.len(),
                    "pendingRevs": state.pending_ops.iter().map(|p| p.client_rev).collect::<Vec<_>>(),
                    "offlineQueue": state.queued_while_offline.len(),
                    "roomId": inner.room_id,
                    "userId": inner.user_id,
                    "attempts": inner.policy.attempts,
                    "events": inner.debug_events,
                });
                JsValue::from_str(&json.to_string())
            })
        }) as Box<dyn Fn() -> JsValue>);
        if let Some(win) = web_sys::window() {
            let target: &js_sys::Object = win.unchecked_ref();
            let _ = js_sys::Reflect::set(
                target,
                &JsValue::from_str("__cdb_collab_state"),
                cb.as_ref().unchecked_ref(),
            );
        }
        cb.forget();

        // ST-FE-S05-08：headless Chromium 的 setOffline 仿真不会断开已建立的 WS，
        // 且 offline 下 close 握手帧发不出去导致 onclose 可能延迟数分钟。
        // 调试钩子直接走 on_ws_close 路径，确定性模拟异常断线（退避重连 → sync+flush 全链路仍真实）。
        let drop_cb = Closure::wrap(Box::new(|| {
            ACTIVE_CONTROLLER.with(|slot| {
                if let Some(ctl) = slot.borrow().as_ref().and_then(|w| w.upgrade()) {
                    ctl.inner.borrow_mut().ws = None;
                    ctl.on_ws_close(None);
                }
            });
        }) as Box<dyn Fn()>);
        if let Some(win) = web_sys::window() {
            let target: &js_sys::Object = win.unchecked_ref();
            let _ = js_sys::Reflect::set(
                target,
                &JsValue::from_str("__cdb_debug_ws_drop"),
                drop_cb.as_ref().unchecked_ref(),
            );
        }
        drop_cb.forget();
    }

    #[cfg(not(debug_assertions))]
    fn register_collab_state_hook() {}

    /// presence 上行是否活跃（已连接且有控制器）。pointermove 用它短路掉
    /// get_bounding_client_rect 等坐标换算，保护画布热路径（R-PERF 系列约束）。
    pub fn presence_active() -> bool {
        ACTIVE_CONTROLLER.with(|slot| {
            slot.borrow()
                .as_ref()
                .and_then(|w| w.upgrade())
                .map(|c| should_send_now(&c.collab_state.get_untracked()))
                .unwrap_or(false)
        })
    }

    /// 外部 store.load（非 collab 通道的全量加载）后调用：刷新活跃控制器 baseline。
    pub fn external_store_reloaded() {
        ACTIVE_CONTROLLER.with(|slot| {
            if let Some(c) = slot.borrow().as_ref().and_then(|w| w.upgrade()) {
                c.refresh_baseline();
            }
        });
    }

    /// editor_render 的 pointermove 在拖动早退前调用；内部节流在 CollabController::send_presence。
    pub fn report_cursor(x: f64, y: f64) {
        CURSOR_REPORTER.with(|slot| {
            if let Some(cb) = slot.borrow().as_ref() {
                cb(x, y);
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_transport::{
    external_store_reloaded, presence_active, report_cursor, set_active_controller,
    set_cursor_reporter, CollabController, CollabWs,
};

// ---------------------------------------------------------------------------
// host 侧 no-op stub：editor_panels / editor_render 在 `cargo test --lib`（非 wasm）
// 下也要编译；真实实现仅在 wasm32 目标生效。
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod host_stub {
    use super::*;
    use std::rc::Rc;

    use crate::editor_data_access::CollabMemberPresence;
    use crate::editor_render::RemotePresence;

    pub struct CollabController;

    impl CollabController {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            _store: EditorStore,
            _collab_state: RwSignal<CollabOtState>,
            _activity_feed: RwSignal<Vec<String>>,
            _remote_members: RwSignal<Vec<CollabMemberPresence>>,
            _remote_presence: RwSignal<Vec<RemotePresence>>,
            _on_token_expired: Rc<dyn Fn()>,
            _on_full_resync: Rc<dyn Fn()>,
        ) -> Rc<Self> {
            Rc::new(Self)
        }

        pub fn connect(self: &Rc<Self>, _base_url: &str, _room_id: &str, _token: &str, _user_id: Option<String>) {}
        pub fn disconnect(&self) {}
        pub fn notify_local_change(self: &Rc<Self>) {}
        pub fn send_presence(&self, _x: f64, _y: f64) {}
        pub fn server_rev(&self) -> i64 {
            0
        }
        pub fn room_id(&self) -> Option<String> {
            None
        }
        pub fn refresh_baseline(&self) {}
        pub fn reset_policy(&self) {}
    }

    pub fn set_cursor_reporter(_cb: Option<Rc<dyn Fn(f64, f64)>>) {}
    pub fn set_active_controller(_ctl: Option<&Rc<CollabController>>) {}
    pub fn presence_active() -> bool {
        false
    }
    pub fn external_store_reloaded() {}
    pub fn report_cursor(_x: f64, _y: f64) {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use host_stub::{
    external_store_reloaded, presence_active, report_cursor, set_active_controller,
    set_cursor_reporter, CollabController,
};

// ---------------------------------------------------------------------------
// 测试（UT-FE-S05-13 ~ UT-FE-S05-17）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_core::{Command, CommandStack};

    fn table(id: &str, name: &str, x: f64, y: f64, fields: Vec<Field>) -> Table {
        Table {
            id: id.to_string(),
            name: name.to_string(),
            x,
            y,
            color: "#fff".into(),
            comment: String::new(),
            fields,
            indices: vec![],
            width: None,
            min_height: None,
        }
    }

    fn field(id: &str, name: &str) -> Field {
        Field {
            id: id.to_string(),
            name: name.to_string(),
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
        }
    }

    fn reference(id: &str) -> Reference {
        Reference {
            id: id.to_string(),
            name: String::new(),
            start_table_id: "t1".into(),
            end_table_id: "t2".into(),
            start_field_id: "f1".into(),
            end_field_id: "f2".into(),
            type_: "OneToMany".into(),
            on_delete: "Cascade".into(),
            on_update: "NoAction".into(),
        }
    }

    fn note(id: &str, x: f64) -> Note {
        Note {
            id: id.to_string(),
            x,
            y: 0.0,
            content: "n".into(),
            color: "#ff0".into(),
        }
    }

    fn area(id: &str, w: f64) -> Area {
        Area {
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            width: w,
            height: 10.0,
            color: "#0ff".into(),
            name: "a".into(),
        }
    }

    fn snap_with(tables: Vec<Table>, refs: Vec<Reference>, notes: Vec<Note>, areas: Vec<Area>) -> (EditorStore, EntitySnapshot, BTreeMap<String, Value>) {
        let store = EditorStore::new();
        store.tables.set(tables);
        store.references.set(refs);
        store.notes.set(notes);
        store.areas.set(areas);
        let full: BTreeMap<String, Value> = store
            .tables
            .get_untracked()
            .into_iter()
            .map(|t| (t.id.clone(), serde_json::to_value(&t).unwrap()))
            .collect();
        let snap = capture_entity_snapshot(&store);
        (store, snap, full)
    }

    fn op_types(ops: &[Value]) -> Vec<String> {
        ops.iter()
            .map(|o| o.get("type").unwrap().as_str().unwrap().to_string())
            .collect()
    }

    /// UT-FE-S05-13：15 种变更 → CollabOp 映射（5 实体 × create/update/delete）
    #[test]
    fn ut_fe_s05_13_local_edits_map_to_collab_ops() {
        // create：全部 5 类
        let (_s, empty, _) = snap_with(vec![], vec![], vec![], vec![]);
        let (_s2, created, full) = snap_with(
            vec![table("t1", "orders", 1.0, 2.0, vec![field("f1", "id")])],
            vec![reference("r1")],
            vec![note("n1", 5.0)],
            vec![area("a1", 100.0)],
        );
        let ops = diff_snapshots(&empty, &created, &full);
        assert_eq!(
            op_types(&ops),
            vec!["table.create", "reference.create", "area.create", "note.create"],
            "新建表（含字段）只产 table.create，不再产 field.create"
        );
        // table.create 的 changes 必须含 fields
        let create = &ops[0];
        assert_eq!(create["targetId"], "t1");
        assert!(create["changes"]["fields"].is_array());
        assert!(create.get("parentId").is_none());

        // 既有表上新增字段 → field.create 带 parentId
        let (_s3, with_f2, full2) = snap_with(
            vec![table("t1", "orders", 1.0, 2.0, vec![field("f1", "id"), field("f2", "email")])],
            vec![reference("r1")],
            vec![note("n1", 5.0)],
            vec![area("a1", 100.0)],
        );
        let ops = diff_snapshots(&created, &with_f2, &full2);
        // 字段增删改变 field_order → 随行携带 fields.reorder（fix-collab-autosave-race：
        // 多增/增+移同窗口的目标序收敛，幂等冗余可接受）
        assert_eq!(op_types(&ops), vec!["field.create", "fields.reorder"]);
        assert_eq!(ops[0]["parentId"], "t1");
        assert_eq!(ops[0]["changes"]["name"], "email");
        assert_eq!(ops[1]["changes"]["fieldIds"], json!(["f1", "f2"]));

        // update：5 类各一例
        let (_s4, updated, full3) = snap_with(
            vec![table("t1", "orders_v2", 9.0, 2.0, vec![field("f1", "id"), {
                let mut f = field("f2", "email");
                f.type_ = "TEXT".into();
                f
            }])],
            vec![{
                let mut r = reference("r1");
                r.on_delete = "SetNull".into();
                r
            }],
            vec![note("n1", 50.0)],
            vec![area("a1", 200.0)],
        );
        let ops = diff_snapshots(&with_f2, &updated, &full3);
        assert_eq!(
            op_types(&ops),
            vec!["table.update", "field.update", "reference.update", "area.update", "note.update"]
        );
        // table.update 载荷剔除 fields
        assert!(ops[0]["changes"].get("fields").is_none());

        // delete：5 类各一例（先删字段，再整体删）
        let (_s5, shrunk, full4) = snap_with(
            vec![table("t1", "orders_v2", 9.0, 2.0, vec![field("f1", "id")])],
            vec![],
            vec![],
            vec![],
        );
        let ops = diff_snapshots(&updated, &shrunk, &full4);
        assert_eq!(
            op_types(&ops),
            vec!["field.delete", "fields.reorder", "reference.delete", "area.delete", "note.delete"]
        );
        assert_eq!(ops[0]["parentId"], "t1");
        assert_eq!(ops[1]["changes"]["fieldIds"], json!(["f1"]), "删字段后 reorder 携带剩余序");

        // 整表删除 → 仅 table.delete，不补 field.delete
        let (_s6, gone, full5) = snap_with(vec![], vec![], vec![], vec![]);
        let ops = diff_snapshots(&shrunk, &gone, &full5);
        assert_eq!(op_types(&ops), vec!["table.delete"]);

        // 未变更 → 无 op
        let ops = diff_snapshots(&gone, &gone, &full5);
        assert!(ops.is_empty(), "未映射的变更不产生 op");
    }

    /// UT-FE-S05-14：remote_op 应用到 EditorStore + baseline 推进后 diff 为空（无回声）
    #[test]
    fn ut_fe_s05_14_remote_op_apply_and_no_echo() {
        let (store, base, _full) = snap_with(vec![], vec![], vec![], vec![]);

        // create table（含字段）
        let op = json!({
            "type": "table.create", "targetId": "t9",
            "changes": serde_json::to_value(table("t9", "users", 3.0, 4.0, vec![field("f9", "id")])).unwrap()
        });
        assert!(apply_op_to_store(&store, &op));
        assert_eq!(store.tables.with_untracked(|ts| ts.len()), 1);
        assert_eq!(store.tables.with_untracked(|ts| ts[0].fields.len()), 1);

        // 模拟控制器：应用后推进 baseline → 再 diff 应为空（无回声）
        let new_base = capture_entity_snapshot(&store);
        let full: BTreeMap<String, Value> = store
            .tables
            .get_untracked()
            .into_iter()
            .map(|t| (t.id.clone(), serde_json::to_value(&t).unwrap()))
            .collect();
        assert!(!diff_snapshots(&base, &new_base, &full).is_empty(), "baseline 未推进时 diff 非空（佐证回声风险）");
        assert!(diff_snapshots(&new_base, &capture_entity_snapshot(&store), &full).is_empty(), "baseline 推进后必须无回声");

        // table.update 标量（无 fields 载荷）→ 字段不被清空
        let op = json!({
            "type": "table.update", "targetId": "t9",
            "changes": { "name": "users2", "x": 30.0 }
        });
        assert!(apply_op_to_store(&store, &op));
        store.tables.with_untracked(|ts| {
            assert_eq!(ts[0].name, "users2");
            assert_eq!(ts[0].x, 30.0);
            assert_eq!(ts[0].fields.len(), 1, "标量更新不得触碰 fields");
        });

        // field.create / field.delete
        let op = json!({
            "type": "field.create", "targetId": "f10", "parentId": "t9",
            "changes": serde_json::to_value(field("f10", "email")).unwrap()
        });
        assert!(apply_op_to_store(&store, &op));
        let op = json!({ "type": "field.delete", "targetId": "f9", "parentId": "t9" });
        assert!(apply_op_to_store(&store, &op));
        store.tables.with_untracked(|ts| {
            assert_eq!(ts[0].fields.len(), 1);
            assert_eq!(ts[0].fields[0].id, "f10");
        });

        // reference delete
        let op = json!({
            "type": "reference.create", "targetId": "r9",
            "changes": serde_json::to_value(reference("r9")).unwrap()
        });
        assert!(apply_op_to_store(&store, &op));
        let op = json!({ "type": "reference.delete", "targetId": "r9" });
        assert!(apply_op_to_store(&store, &op));
        assert!(store.references.with_untracked(|rs| rs.is_empty()));

        // note / area create+delete 各一例
        for (kind, id, val) in [
            ("note", "n9", serde_json::to_value(note("n9", 1.0)).unwrap()),
            ("area", "a9", serde_json::to_value(area("a9", 9.0)).unwrap()),
        ] {
            let op = json!({ "type": format!("{kind}.create"), "targetId": id, "changes": val });
            assert!(apply_op_to_store(&store, &op));
            let op = json!({ "type": format!("{kind}.delete"), "targetId": id });
            assert!(apply_op_to_store(&store, &op));
        }
        assert!(store.notes.with_untracked(|n| n.is_empty()));
        assert!(store.areas.with_untracked(|a| a.is_empty()));

        // 未知 op / 缺 targetId → 不变更
        assert!(!apply_op_to_store(&store, &json!({"type": "unknown.x", "targetId": "z"})));
        assert!(!apply_op_to_store(&store, &json!({"type": "table.delete"})));
    }

    /// UT-FE-S05-19：ops_fully_synced 驱动 dirty 语义（全部 ack 且无积压 → 已保存）
    #[test]
    fn ut_fe_s05_19_ops_fully_synced() {
        let mut state = CollabOtState::default();
        assert!(ops_fully_synced(&state), "初始无 pending/积压 → 已同步");

        // 离线编辑 → 进 queued_while_offline → 未同步
        let rev = state.enqueue_local_op("table.create").unwrap();
        assert!(!ops_fully_synced(&state), "离线积压存在 → 未同步");

        // sync 后积压转入 pending（待 ack）→ 仍未同步
        state.apply_sync(1);
        assert!(!ops_fully_synced(&state), "flush 后待 ack → 未同步");

        // 全部 ack → 已同步（dirty 清零条件）
        state.ack(Some(rev), 2);
        assert!(ops_fully_synced(&state), "全部 ack → 已同步");
    }

    /// UT-FE-S05-20：RemoteOp 有序收编 + sync 在途缓冲回放（入房窗口不丢 op）
    #[test]
    fn ut_fe_s05_20_ordered_remote_op_and_buffered_drain() {
        // plan_remote_op 真值表
        assert_eq!(plan_remote_op(5, 6, false), RemoteOpAction::Apply, "顺序到达 → 应用");
        assert_eq!(plan_remote_op(5, 5, false), RemoteOpAction::SkipStale, "重复 rev → 丢弃");
        assert_eq!(plan_remote_op(5, 3, false), RemoteOpAction::SkipStale, "旧 rev → 丢弃");
        assert_eq!(plan_remote_op(5, 8, false), RemoteOpAction::BufferAndSync, "缺口 → 缓冲+sync");
        assert_eq!(plan_remote_op(5, 6, true), RemoteOpAction::Buffer, "sync 在途 → 一律缓冲");
        assert_eq!(plan_remote_op(5, 4, true), RemoteOpAction::Buffer, "sync 在途（含旧 rev）→ 缓冲");

        // drain：乱序+重复缓冲，sync 回放到 rev 3 后
        let buf = vec![
            (5, json!({"type": "table.create", "targetId": "t5"})),
            (3, json!({"type": "table.create", "targetId": "t3"})), // 已被 sync 回放覆盖
            (4, json!({"type": "table.create", "targetId": "t4"})),
            (5, json!({"type": "table.create", "targetId": "t5"})), // 重复广播
        ];
        let (apply, rest, gap) = drain_buffered_ops(buf, 3);
        assert_eq!(apply.len(), 2, "4、5 顺序补齐");
        assert_eq!(apply[0]["targetId"], json!("t4"));
        assert_eq!(apply[1]["targetId"], json!("t5"));
        assert!(rest.is_empty() && !gap, "无缺口 → 收敛");

        // drain：缓冲存在缺口（7 先到、6 未到）→ 保留剩余并要求再次 sync
        let buf = vec![(7, json!({"targetId": "t7"}))];
        let (apply, rest, gap) = drain_buffered_ops(buf, 5);
        assert!(apply.is_empty() && gap, "缺口未补齐 → 再次 sync");
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].0, 7, "缺口的 op 保留等待");
    }

    /// UT-FE-S05-21：字段换序 → fields.reorder op（同 id 集合、仅顺序变化）；
    /// 整表 create/delete 当次不重复产出；apply_op_to_store 按 fieldIds 重排。
    /// （ST-SP-LIST-02 暴露：id 键控快照对纯换序产生空 diff，op 通道无法表达字段顺序）
    #[test]
    fn ut_fe_s05_21_field_reorder_maps_to_op_and_applies() {
        let mk = |fields: Vec<Field>| {
            snap_with(
                vec![table("t1", "orders", 1.0, 2.0, fields)],
                vec![],
                vec![],
                vec![],
            )
        };
        let (_s, prev, _) = mk(vec![field("f1", "id"), field("f2", "email")]);
        let (_s2, swapped, full) = mk(vec![field("f2", "email"), field("f1", "id")]);

        // 纯换序 → 仅 fields.reorder（无 field.* op）
        let ops = diff_snapshots(&prev, &swapped, &full);
        assert_eq!(op_types(&ops), vec!["fields.reorder"], "纯换序只产 fields.reorder");
        assert_eq!(ops[0]["targetId"], "t1");
        assert_eq!(ops[0]["changes"]["fieldIds"], json!(["f2", "f1"]));
        assert!(ops[0].get("parentId").is_none());

        // 增字段 + 换序同窗口：field.create 之后跟 fields.reorder（全量目标序）
        let (_s3, added, full2) = mk(vec![field("f3", "new"), field("f1", "id"), field("f2", "email")]);
        let ops2 = diff_snapshots(&prev, &added, &full2);
        assert_eq!(op_types(&ops2), vec!["field.create", "fields.reorder"], "reorder 排在 field.create 之后");
        assert_eq!(ops2[1]["changes"]["fieldIds"], json!(["f3", "f1", "f2"]));

        // 整表 create/delete 当次不重复产出 reorder（全量表 JSON 已携带顺序）
        let (_s4, empty, _) = snap_with(vec![], vec![], vec![], vec![]);
        assert_eq!(op_types(&diff_snapshots(&empty, &swapped, &full)), vec!["table.create"]);
        assert_eq!(op_types(&diff_snapshots(&swapped, &empty, &full)), vec!["table.delete"]);

        // apply_op_to_store：按 fieldIds 重排；幂等；未知表 false
        let (store, _, _) = mk(vec![field("f1", "id"), field("f2", "email")]);
        let reorder = json!({"type":"fields.reorder","targetId":"t1","changes":{"fieldIds":["f2","f1"]}});
        assert!(apply_op_to_store(&store, &reorder));
        let order: Vec<String> = store.tables.get_untracked()[0]
            .fields
            .iter()
            .map(|f| f.id.clone())
            .collect();
        assert_eq!(order, vec!["f2", "f1"]);
        assert!(!apply_op_to_store(&store, &reorder), "同序重放幂等");
        assert!(!apply_op_to_store(
            &store,
            &json!({"type":"fields.reorder","targetId":"tX","changes":{"fieldIds":["f1"]}})
        ));
    }

    /// UT-FE-S05-16：viewer（ReadOnly）拦截本地 op；其余状态生成
    #[test]
    fn ut_fe_s05_16_viewer_ops_blocked() {
        let mut state = CollabOtState::connected(3);
        assert!(watcher_active(&state));
        assert!(should_send_now(&state));

        state.mark_read_only();
        assert!(!watcher_active(&state), "ReadOnly 时 watcher 不得生成 op");
        assert!(state.enqueue_local_op("table.create").is_none(), "ReadOnly 不入队");
        assert!(!should_send_now(&state));

        // 断线期间仍生成并入离线队列（不入 pending）
        let mut offline = CollabOtState::default();
        assert!(watcher_active(&offline));
        assert!(!should_send_now(&offline));
        let rev = offline.enqueue_local_op("note.create");
        assert!(rev.is_some());
        assert_eq!(offline.queued_while_offline.len(), 1);
        assert!(offline.pending_ops.is_empty());
    }

    /// UT-FE-S05-17：重连退避 ≤5 次、指数延迟、耗尽进 failed；sync 帧构造
    #[test]
    fn ut_fe_s05_17_reconnect_backoff_and_sync_frame() {
        let mut policy = ReconnectPolicy::default();
        let mut delays = Vec::new();
        while let Some(d) = policy.next_delay_ms() {
            delays.push(d);
            policy.record_attempt();
        }
        assert_eq!(delays, vec![1000, 2000, 4000, 8000, 16000], "指数退避 5 次");
        assert!(policy.next_delay_ms().is_none(), "5 次后耗尽 → failed 态");

        policy.reset();
        assert_eq!(policy.next_delay_ms(), Some(1000));

        // sync 帧格式（S05.4 Step 8）
        let frame: Value = serde_json::from_str(&build_sync_frame(42)).unwrap();
        assert_eq!(frame["type"], "sync");
        assert_eq!(frame["lastRev"], 42);

        // op 帧格式（collab.yaml CollabFrameOpClient）
        let frame: Value = serde_json::from_str(&build_op_frame(7, &json!({"type":"table.create","targetId":"t1","changes":{}}))).unwrap();
        assert_eq!(frame["type"], "op");
        assert_eq!(frame["clientRev"], 7);
        assert_eq!(frame["op"]["type"], "table.create");
    }

    /// 防回归：Command 驱动的编辑也走 diff watcher（不经 Command→op 专用通道）
    #[test]
    fn ut_fe_s05_13b_command_driven_edits_visible_to_diff() {
        let (store, base, _) = snap_with(vec![], vec![], vec![], vec![]);
        let mut stack = CommandStack::new();
        CommandStack::apply(
            &store,
            &mut stack,
            Command::AddTable(table("tc", "cmd_table", 0.0, 0.0, vec![])),
        )
        .unwrap();
        let current = capture_entity_snapshot(&store);
        let full: BTreeMap<String, Value> = store
            .tables
            .get_untracked()
            .into_iter()
            .map(|t| (t.id.clone(), serde_json::to_value(&t).unwrap()))
            .collect();
        let ops = diff_snapshots(&base, &current, &full);
        assert_eq!(op_types(&ops), vec!["table.create"]);
    }

    /// feat-data-dictionary（S07）：store.dictionaries 变更 → diff 产出 dict.* op；
    /// 远端 dict.* op 经 apply_flat 应用到 store.dictionaries（协议扁平集合口径）
    #[test]
    fn dict_ops_diff_and_remote_apply() {
        let (store, base, _) = snap_with(vec![], vec![], vec![], vec![]);
        store.dictionaries.set(vec![DataDictionary {
            id: "d1".into(),
            name: "是否".into(),
            code: "yes_no".into(),
            comment: String::new(),
            items: vec![],
        }]);
        let current = capture_entity_snapshot(&store);
        let ops = diff_snapshots(&base, &current, &BTreeMap::new());
        assert_eq!(op_types(&ops), vec!["dict.create"]);
        assert_eq!(ops[0]["targetId"], "d1");
        assert_eq!(ops[0]["changes"]["code"], "yes_no");

        // update：改 code → dict.update
        let prev = current.clone();
        store.dictionaries.update(|ds| ds[0].code = "yn".into());
        let current2 = capture_entity_snapshot(&store);
        let ops2 = diff_snapshots(&prev, &current2, &BTreeMap::new());
        assert_eq!(op_types(&ops2), vec!["dict.update"]);

        // delete：清空 → dict.delete
        store.dictionaries.set(vec![]);
        let current3 = capture_entity_snapshot(&store);
        let ops3 = diff_snapshots(&current2, &current3, &BTreeMap::new());
        assert_eq!(op_types(&ops3), vec!["dict.delete"]);

        // remote apply：dict.create → store.dictionaries 增加；dict.delete → 移除
        let applied = apply_op_to_store(
            &store,
            &json!({"type":"dict.create","targetId":"d9","changes":{"id":"d9","name":"性别","code":"gender","items":[]}}),
        );
        assert!(applied);
        assert_eq!(store.dictionaries.get_untracked().len(), 1);
        assert_eq!(store.dictionaries.get_untracked()[0].code, "gender");
        assert!(apply_op_to_store(
            &store,
            &json!({"type":"dict.delete","targetId":"d9"})
        ));
        assert!(store.dictionaries.get_untracked().is_empty());
    }
}
