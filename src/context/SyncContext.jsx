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
  const [connected, setConnected] = useState(false);
  const [diagramId, setDiagramId] = useState(null);
  const [revision, setRevision] = useState(0);

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

  useEffect(() => () => disconnect(), [disconnect]);

  const value = useMemo(
    () => ({
      connect,
      disconnect,
      sendSnapshot,
      pullLatest,
      connected,
      diagramId,
      revision,
      setRevision,
      clientId: clientIdRef.current,
    }),
    [connect, disconnect, sendSnapshot, pullLatest, connected, diagramId, revision],
  );

  return <SyncContext.Provider value={value}>{children}</SyncContext.Provider>;
}

