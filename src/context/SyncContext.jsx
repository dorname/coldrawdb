import { createContext, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Toast } from "@douyinfe/semi-ui";
import { diagramMapper, diagramService } from "../services/diagramService";
import {
  useAreas,
  useDiagram,
  useNotes,
  useTasks,
  useTransform,
  useTypes,
  useEnums,
} from "../hooks";

export const SyncContext = createContext(null);

const makeClientId = () =>
  `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;

export default function SyncContextProvider({ children }) {
  const { setTables, setRelationships, setDatabase } = useDiagram();
  const { setNotes } = useNotes();
  const { setAreas } = useAreas();
  const { setTasks } = useTasks();
  const { setTransform } = useTransform();
  const { setTypes } = useTypes();
  const { setEnums } = useEnums();

  const wsRef = useRef(null);
  const clientIdRef = useRef(makeClientId());
  const applyOpRef = useRef(() => {});
  const [connected, setConnected] = useState(false);
  const [diagramId, setDiagramId] = useState(null);
  const [revision, setRevision] = useState(0);
  const [onlineClients, setOnlineClients] = useState({});
  const [cursors, setCursors] = useState({});

  const applyDiagram = useCallback(
    (mapped) => {
      if (!mapped) return;
      if (typeof mapped.revision === "number" && mapped.revision <= revision) return;
      setRevision(mapped.revision ?? 0);
      if (mapped.database) setDatabase(mapped.database);
      setTables(mapped.tables ?? []);
      setRelationships(mapped.relationships ?? []);
      setNotes(mapped.notes ?? []);
      setAreas(mapped.areas ?? []);
      setTasks(mapped.todos ?? []);
      setTransform({ pan: mapped.pan, zoom: mapped.zoom });
      setTypes(mapped.types ?? []);
      setEnums(mapped.enums ?? []);
    },
    [
      revision,
      setAreas,
      setDatabase,
      setEnums,
      setNotes,
      setRelationships,
      setTables,
      setTasks,
      setTransform,
      setTypes,
    ],
  );

  const connect = useCallback((nextDiagramId) => {
    if (!nextDiagramId) return;
    if (wsRef.current && diagramId === String(nextDiagramId)) return;

    // close old
    try {
      wsRef.current?.close();
    } catch {
      // ignore close errors
    }

    const host = window.location.host;
    const proto = window.location.protocol === "https:" ? "wss" : "ws";
    const url = `${proto}://${host}/api/diagrams/ws/${nextDiagramId}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    setDiagramId(String(nextDiagramId));
    setConnected(false);

    ws.onopen = () => {
      setConnected(true);
      ws.send(
        JSON.stringify({
          type: "join",
          payload: { clientId: clientIdRef.current, diagramId: String(nextDiagramId) },
        }),
      );
    };

    ws.onclose = () => setConnected(false);
    ws.onerror = () => setConnected(false);

    ws.onmessage = (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (msg?.type === "diagram_snapshot") {
        const vo = msg?.payload?.diagram;
        const mapped = diagramMapper.mapFromBackend(vo);
        applyDiagram(mapped);
      } else if (msg?.type === "conflict") {
        Toast.warning("文档已被其他端更新，请刷新后再保存。");
      } else if (msg?.type === "op_edit") {
        if (msg?.payload?.senderClientId === clientIdRef.current) return;
        applyOpRef.current(msg.payload);
      } else if (msg?.type === "op_awareness") {
        const p = msg.payload;
        if (!p?.clientId || p.clientId === clientIdRef.current) return;
        setOnlineClients((prev) => {
          const next = { ...prev };
          if (p.status === "leave") {
            delete next[p.clientId];
          } else {
            next[p.clientId] = {
              user: p.user || { id: p.clientId, name: p.clientId },
              lastSeen: Date.now(),
            };
          }
          return next;
        });
      } else if (msg?.type === "op_cursor") {
        const p = msg.payload;
        if (!p?.clientId || p.clientId === clientIdRef.current) return;
        if (p.diagramId !== diagramId) return;
        setCursors((prev) => ({
          ...prev,
          [p.clientId]: {
            focusedType: p.focusedType,
            focusedId: p.focusedId,
            timestamp: p.timestamp || Date.now(),
          },
        }));
      }
    };
  }, [applyDiagram, diagramId]);

  const disconnect = useCallback(() => {
    try {
      wsRef.current?.close();
    } catch {
      // ignore close errors
    }
    wsRef.current = null;
    setConnected(false);
  }, []);

  const sendSnapshot = useCallback((diagram) => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    const payload = diagramMapper.mapToBackend(diagram);
    ws.send(
      JSON.stringify({
        type: "diagram_snapshot_broadcast",
        payload: { diagram: payload },
      }),
    );
  }, []);

  const pullLatest = useCallback(
    async (targetDiagramId) => {
      if (!targetDiagramId) return;
      try {
        const d = await diagramService.getById(targetDiagramId);
        applyDiagram(d);
      } catch (e) {
        console.error(e);
        Toast.error("刷新失败");
      }
    },
    [applyDiagram],
  );

  const applyOp = useCallback(
    (payload) => {
      if (!payload?.op) return;
      const { op, data } = payload;
      switch (op) {
        case "table_add":
          if (data) setTables((prev) => [...prev, data]);
          break;
        case "table_remove":
          if (data?.id) {
            setRelationships((prev) =>
              prev.filter(
                (r) => r.startTableId !== data.id && r.endTableId !== data.id,
              ),
            );
            setTables((prev) => prev.filter((t) => t.id !== data.id));
          }
          break;
        case "table_update":
          if (data?.id)
            setTables((prev) =>
              prev.map((t) => (t.id === data.id ? { ...t, ...data } : t)),
            );
          break;
        case "relationship_add":
          if (data)
            setRelationships((prev) =>
              [...prev, data].map((e, i) => ({ ...e, id: i })),
            );
          break;
        case "relationship_remove":
          if (data?.id !== undefined)
            setRelationships((prev) =>
              prev
                .filter((r) => r.id !== data.id)
                .map((e, i) => ({ ...e, id: i })),
            );
          break;
        case "relationship_update":
          if (data?.id !== undefined)
            setRelationships((prev) =>
              prev.map((r) => (r.id === data.id ? { ...r, ...data } : r)),
            );
          break;
        case "note_add":
          if (data) setNotes((prev) => [...prev, data]);
          break;
        case "note_remove":
          if (data?.id !== undefined)
            setNotes((prev) =>
              prev
                .filter((n) => n.id !== data.id)
                .map((e, i) => ({ ...e, id: i })),
            );
          break;
        case "note_update":
          if (data?.id !== undefined)
            setNotes((prev) =>
              prev.map((n) => (n.id === data.id ? { ...n, ...data } : n)),
            );
          break;
        case "area_add":
          if (data) setAreas((prev) => [...prev, data]);
          break;
        case "area_remove":
          if (data?.id !== undefined)
            setAreas((prev) =>
              prev
                .filter((a) => a.id !== data.id)
                .map((e, i) => ({ ...e, id: i })),
            );
          break;
        case "area_update":
          if (data?.id !== undefined)
            setAreas((prev) =>
              prev.map((a) => (a.id === data.id ? { ...a, ...data } : a)),
            );
          break;
        case "task_update":
          if (data?.index !== undefined && data?.values)
            setTasks((prev) =>
              prev.map((t, i) =>
                i === data.index ? { ...t, ...data.values } : t,
              ),
            );
          break;
        case "field_update":
          if (data?.tableId && data?.fieldId && data?.values) {
            setTables((prev) =>
              prev.map((table) => {
                if (table.id !== data.tableId) return table;
                return {
                  ...table,
                  fields: table.fields.map((field) =>
                    field.id === data.fieldId
                      ? { ...field, ...data.values }
                      : field,
                  ),
                };
              }),
            );
          }
          break;
        case "field_remove":
          if (data?.tableId && data?.fieldId) {
            setTables((prev) =>
              prev.map((table) => {
                if (table.id !== data.tableId) return table;
                return {
                  ...table,
                  fields: table.fields.filter(
                    (field) => field.id !== data.fieldId,
                  ),
                };
              }),
            );
          }
          break;
        case "field_reorder":
          if (
            data?.tableId &&
            typeof data.fromIndex === "number" &&
            typeof data.toIndex === "number"
          ) {
            setTables((prev) =>
              prev.map((table) => {
                if (table.id !== data.tableId) return table;
                const fields = table.fields.slice();
                const [moved] = fields.splice(data.fromIndex, 1);
                fields.splice(data.toIndex, 0, moved);
                return { ...table, fields };
              }),
            );
          }
          break;
        default:
          break;
      }
    },
    [
      setAreas,
      setNotes,
      setRelationships,
      setTables,
      setTasks,
    ],
  );

  const sendOp = useCallback(
    (opPayload) => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN || !diagramId) return;
      const payload = {
        diagramId,
        senderClientId: clientIdRef.current,
        op: opPayload.op,
        data: opPayload.data,
      };
      ws.send(JSON.stringify({ type: "op_edit", payload }));
    },
    [diagramId],
  );

  applyOpRef.current = applyOp;

  const sendAwareness = useCallback(
    (status) => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN || !diagramId) return;
      const payload = {
        diagramId,
        clientId: clientIdRef.current,
        user: { id: clientIdRef.current, name: clientIdRef.current },
        status,
      };
      ws.send(JSON.stringify({ type: "op_awareness", payload }));
    },
    [diagramId],
  );

  const sendCursor = useCallback(
    (focusedType, focusedId) => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN || !diagramId) return;
      const payload = {
        diagramId,
        clientId: clientIdRef.current,
        focusedType,
        focusedId,
        timestamp: Date.now(),
      };
      ws.send(JSON.stringify({ type: "op_cursor", payload }));
    },
    [diagramId],
  );

  useEffect(() => {
    if (!connected) return;
    sendAwareness("join");
    const id = setInterval(() => {
      sendAwareness("ping");
    }, 30000);
    return () => {
      sendAwareness("leave");
      clearInterval(id);
      disconnect();
    };
  }, [connected, sendAwareness, disconnect]);

  const value = useMemo(
    () => ({
      connect,
      disconnect,
      sendSnapshot,
      sendOp,
      pullLatest,
      applyOp,
      connected,
      diagramId,
      revision,
      setRevision,
      clientId: clientIdRef.current,
      onlineClients,
      cursors,
      sendCursor,
    }),
    [
      connect,
      disconnect,
      sendSnapshot,
      sendOp,
      pullLatest,
      applyOp,
      connected,
      diagramId,
      revision,
      onlineClients,
      cursors,
      sendCursor,
    ],
  );

  return <SyncContext.Provider value={value}>{children}</SyncContext.Provider>;
}

