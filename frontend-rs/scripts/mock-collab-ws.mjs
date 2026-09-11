// fix-collab-autosave-race：方案 B mock WS —— room 模式编辑只走 op 通道（无全量 PUT），
// D/E/F 批「落账」断言的事实源从 PUT 载荷迁移到 mock 服务端物化文档。
//
// 行为与真实后端对齐（backend/src/collab/mod.rs append_op + apply_op_to_doc）：
// - 连接即回 connected 帧（serverRev / yourRole / members）
// - 收到 op 帧 → serverRev+1 → 按 15 种 op 规则幂等物化进 state.doc → 回 ack
//   （ack 驱动前端 dirty=false → save-state=saved，waitSaved 语义不变）
// - sync 帧 → 回空 ops 的 sync（mock 场景 op 全部来自本连接，无需补发）
// - presence 帧忽略
//
// 为最小化断言迁移，每次 ack 后同步更新 legacy 字段：
//   state.putCalls += 1                          （原「PUT 次数」→ 现「op 上行次数」）
//   state.lastPutBody = { diagram: <doc 克隆> }  （原「PUT 载荷」→ 现「物化文档快照」）
//   state.savedDiagram = <doc 克隆>              （E 批 reload 后 GET 返回物化文档）

/** 后端 apply_op_to_doc 的 JS 镜像（保持一致；改协议时两边同步改）。 */
export function applyOpToDoc(doc, op) {
  const opType = op?.type;
  const targetId = op?.targetId;
  if (typeof opType !== "string" || typeof targetId !== "string") return false;
  const [kind, action] = opType.split(".");
  const parentId = typeof op?.parentId === "string" ? op.parentId : null;
  const changes = op?.changes;
  const withId = item => ({ id: targetId, ...(item ?? {}) });
  const upsert = (arr, item) => {
    const i = arr.findIndex(e => e?.id === targetId);
    if (i >= 0) arr[i] = item; else arr.push(item);
    return true;
  };
  const remove = arr => {
    const i = arr.findIndex(e => e?.id === targetId);
    if (i >= 0) arr.splice(i, 1);
    return true;
  };

  if (kind === "table") {
    const tables = doc.tables;
    if (action === "create") {
      if (!changes) return false;
      return upsert(tables, { fields: [], ...withId(changes) });
    }
    if (action === "update") {
      if (!changes || typeof changes !== "object") return false;
      const slot = tables.find(e => e?.id === targetId);
      if (slot) {
        // 合并除 fields 外所有键（fields 走 field.* op）
        for (const [k, v] of Object.entries(changes)) {
          if (k !== "fields") slot[k] = v;
        }
        return true;
      }
      // 目标不存在：按 create 兜底
      return upsert(tables, { fields: [], ...withId(changes) });
    }
    if (action === "delete") return remove(tables);
    return false;
  }
  if (kind === "fields" && action === "reorder") {
    // 字段换序：targetId=表 id，changes.fieldIds=全量目标序；列表外字段保持原相对序追加
    const table = doc.tables.find(e => e?.id === targetId);
    if (!table || !Array.isArray(table.fields)) return false;
    const ids = Array.isArray(changes?.fieldIds) ? changes.fieldIds : null;
    if (!ids) return false;
    const pos = new Map(ids.map((id, i) => [id, i]));
    const key = f => pos.get(f?.id) ?? ids.length;
    const before = table.fields.map(key).join();
    table.fields = [...table.fields].sort((a, b) => key(a) - key(b));
    return table.fields.map(key).join() !== before;
  }
  if (kind === "field") {
    if (!parentId) return false;
    const table = doc.tables.find(e => e?.id === parentId);
    if (!table || !Array.isArray(table.fields)) return false;
    if (action === "create" || action === "update") {
      if (!changes) return false;
      return upsert(table.fields, withId(changes));
    }
    if (action === "delete") return remove(table.fields);
    return false;
  }
  if (kind === "reference" || kind === "area" || kind === "note") {
    const key = kind === "reference" ? "references" : kind === "area" ? "areas" : "notes";
    const arr = doc[key];
    if (action === "create" || action === "update") {
      if (!changes) return false;
      return upsert(arr, withId(changes));
    }
    if (action === "delete") return remove(arr);
    return false;
  }
  return false;
}

/**
 * 在页面上安装 mock 协作 WS（routeWebSocket 拦截 /ws/rooms/ 路径）。
 * options: { yourRole = "owner", serverRev = 7, diagramId = "diagram-new" }
 */
export async function installMockCollabWs(page, state, options = {}) {
  const yourRole = options.yourRole ?? "owner";
  const diagramId = options.diagramId ?? "diagram-new";
  state.doc = state.doc ?? { tables: [], references: [], areas: [], notes: [] };
  state.sentOps = state.sentOps ?? [];
  let serverRev = options.serverRev ?? 7;

  await page.routeWebSocket(new RegExp(`/ws/rooms/`), ws => {
    // ST-FE-S05-11：记录最新连接，供用例主动推送远端 op（模拟「他人编辑」广播）
    state.__mockWs = ws;
    // 远端 op 推送：先物化进服务端文档（与生产 append+物化同事务一致），再广播。
    // 帧必须含 authorId——CollabFrame::RemoteOp 缺该字段反序列化失败，帧被静默丢弃
    state.pushRemoteOp = op => {
      if (!state.__mockWs) return false;
      serverRev += 1;
      applyOpToDoc(state.doc, op);
      state.__mockWs.send(JSON.stringify({ type: "remote_op", serverRev, authorId: "remote-user-1", op }));
      return true;
    };
    ws.send(JSON.stringify({
      type: "connected",
      serverRev,
      diagramId,
      yourRole,
      members: [],
    }));
    ws.onMessage(msg => {
      let frame;
      try {
        frame = JSON.parse(typeof msg === "string" ? msg : msg.toString("utf8"));
      } catch {
        return;
      }
      if (frame.type === "op") {
        serverRev += 1;
        state.sentOps.push(frame.op);
        state.sentClientRevs = state.sentClientRevs ?? [];
        state.sentClientRevs.push(frame.clientRev);
        applyOpToDoc(state.doc, frame.op);
        // legacy 断言字段：op 落账即「保存发生」，物化文档即「保存载荷」
        state.putCalls += 1;
        state.lastPutBody = { diagram: structuredClone(state.doc) };
        state.putBodies = state.putBodies ?? [];
        state.putBodies.push(state.lastPutBody);
        // E 批 reload 链路：GET 返回物化文档（与生产「GET 物化读」一致）
        state.savedDiagram = structuredClone(state.doc);
        ws.send(JSON.stringify({
          type: "ack",
          serverRev,
          clientRev: frame.clientRev,
          appliedOp: frame.op,
        }));
      } else if (frame.type === "sync") {
        ws.send(JSON.stringify({ type: "sync", serverRev, ops: [] }));
      }
      // presence 等其余帧忽略
    });
  });
}
