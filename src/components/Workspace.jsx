import { useState, useEffect, useCallback, createContext } from "react";
import ControlPanel from "./EditorHeader/ControlPanel";
import Canvas from "./EditorCanvas/Canvas";
import { CanvasContextProvider } from "../context/CanvasContext";
import SidePanel from "./EditorSidePanel/SidePanel";
import { DB, State } from "../data/constants";
import {
  useLayout,
  useSettings,
  useTransform,
  useDiagram,
  useUndoRedo,
  useAreas,
  useNotes,
  useTypes,
  useTasks,
  useSaveState,
  useEnums,
  useSync,
} from "../hooks";
import FloatingControls from "./FloatingControls";
import { Modal, Tag, Toast } from "@douyinfe/semi-ui";
import { useTranslation } from "react-i18next";
import { databases } from "../data/databases";
import { isRtl } from "../i18n/utils/rtl";
import { useSearchParams } from "react-router-dom";
import { get as getGist } from "../api/gists";

import { get } from "../utils/requestApi";
import { diagramService } from "../services/diagramService";
import { templateService } from "../services/templateService";

export const IdContext = createContext({ gistId: "", setGistId: () => {} });

const SIDEPANEL_MIN_WIDTH = 384;

export default function WorkSpace() {
  const [id, setId] = useState(0);
  const [gistId, setGistId] = useState("");
  const [loadedFromGistId, setLoadedFromGistId] = useState("");
  const [title, setTitle] = useState("Untitled Diagram");
  const [resize, setResize] = useState(false);
  const [width, setWidth] = useState(SIDEPANEL_MIN_WIDTH);
  const [lastSaved, setLastSaved] = useState("");
  const [showSelectDbModal, setShowSelectDbModal] = useState(false);
  const [selectedDb, setSelectedDb] = useState("");
  const { layout } = useLayout();
  const { settings } = useSettings();
  const { types, setTypes } = useTypes();
  const { areas, setAreas } = useAreas();
  const { tasks, setTasks } = useTasks();
  const { notes, setNotes } = useNotes();
  const { saveState, setSaveState } = useSaveState();
  const { transform, setTransform } = useTransform();
  const { enums, setEnums } = useEnums();
  const {
    tables,
    relationships,
    setTables,
    setRelationships,
    database,
    setDatabase,
  } = useDiagram();
  const { undoStack, redoStack, setUndoStack, setRedoStack } = useUndoRedo();
  const sync = useSync();
  const { t, i18n } = useTranslation();
  let [searchParams, setSearchParams] = useSearchParams();
  const handleResize = (e) => {
    if (!resize) return;
    const w = isRtl(i18n.language) ? window.innerWidth - e.clientX : e.clientX;
    if (w > SIDEPANEL_MIN_WIDTH) setWidth(w);
  };

  const save = useCallback(async () => {
    if (saveState !== State.SAVING) return;

    const name = window.name.split(" ");
    const op = name[0];
    const saveAsDiagram = window.name === "" || op === "d" || op === "lt";

    const basePayload = {
      id,
      revision: sync?.revision ?? 0,
      database,
      title,
      tables,
      relationships,
      notes,
      areas,
      todos: tasks,
      pan: transform.pan,
      zoom: transform.zoom,
      gistId: gistId ?? "",
      loadedFromGistId,
      ...(databases[database].hasEnums && { enums }),
      ...(databases[database].hasTypes && { types }),
    };

    try {
      if (saveAsDiagram) {
        searchParams.delete("shareId");
        setSearchParams(searchParams);

        if ((id === 0 && window.name === "") || op === "lt") {
          const created = await diagramService.create(basePayload);
          const newId = created.id;
          setId(newId);
          window.name = `d ${newId}`;
          sync?.setRevision?.(created.revision ?? 0);
          sync?.connect?.(newId);
        } else {
          const updated = await diagramService.update(basePayload);
          sync?.setRevision?.(updated.revision ?? sync?.revision ?? 0);
          sync?.sendSnapshot?.(updated);
        }
      } else {
        if (!id) {
          const created = await templateService.create({
            ...basePayload,
            subjectAreas: areas,
            custom: 1,
          });
          const newId = created.id;
          setId(newId);
          window.name = `t ${newId}`;
        } else {
          await templateService.update({
            ...basePayload,
            subjectAreas: areas,
            custom: 1,
          });
        }
      }
      setSaveState(State.SAVED);
      setLastSaved(new Date().toLocaleString());
    } catch (e) {
      console.error(e);
      // 后端 revision 冲突会返回 code=409，requestApi 会抛 Error(message)
      const msg = String(e?.message || "");
      if (
        msg.toLowerCase().includes("conflict") ||
        msg.includes("冲突") ||
        msg.includes("revision conflict")
      ) {
        Toast.warning("文档已被其他端更新，请刷新后再保存。");
        sync?.pullLatest?.(id);
      }
      setSaveState(State.ERROR);
    }
  }, [
    searchParams,
    setSearchParams,
    tables,
    relationships,
    notes,
    areas,
    types,
    title,
    id,
    tasks,
    transform,
    setSaveState,
    database,
    enums,
    gistId,
    loadedFromGistId,
    saveState,
    sync,
  ]);

  const load = useCallback(async () => {
    const applyDiagram = (d) => {
      if (!d) return;
      if (d.database) {
        setDatabase(d.database);
      } else {
        setDatabase(DB.GENERIC);
      }
      setId(d.id);
      sync?.setRevision?.(d.revision ?? 0);
      setGistId(d.gistId);
      setLoadedFromGistId(d.loadedFromGistId);
      setTitle(d.title);
      setTables(d.tables);
      setRelationships(d.relationships);
      setNotes(d.notes);
      setAreas(d.areas);
      setTasks(d.todos ?? []);
      setTransform({ pan: d.pan, zoom: d.zoom });
      if (databases[d.database || database].hasTypes) {
        setTypes(d.types ?? []);
      }
      if (databases[d.database || database].hasEnums) {
        setEnums(d.enums ?? []);
      }
    };

    const loadLatestDiagram = async () => {
      try {
        const d = await diagramService.getLatest();
        if (d) {
          applyDiagram(d);
          window.name = `d ${d.id}`;
        } else {
          window.name = "";
          if (selectedDb === "") setShowSelectDbModal(true);
        }
      } catch (error) {
        console.log(error);
      }
    };

    const loadDiagram = async (id) => {
      try {
        await get(`/tables/queryTables/${id}`);
      } catch (e) {
        console.log(e);
      }
      try {
        const d = await diagramService.getById(id);
        if (d) {
          applyDiagram(d);
          setUndoStack([]);
          setRedoStack([]);
          window.name = `d ${d.id}`;
        } else {
          window.name = "";
        }
      } catch (error) {
        console.log(error);
      }
    };

    const loadTemplate = async (id) => {
      try {
        const t = await templateService.getById(id);
        if (t) {
          if (t.database) {
            setDatabase(t.database);
          } else {
            setDatabase(DB.GENERIC);
          }
          setId(t.id);
          setTitle(t.title);
          setTables(t.tables);
          setRelationships(t.relationships);
          setAreas(t.subjectAreas);
          setTasks(t.todos ?? []);
          setNotes(t.notes);
          setTransform({
            zoom: 1,
            pan: { x: 0, y: 0 },
          });
          setUndoStack([]);
          setRedoStack([]);
          if (databases[t.database || database].hasTypes) {
            setTypes(t.types ?? []);
          }
          if (databases[t.database || database].hasEnums) {
            setEnums(t.enums ?? []);
          }
        } else {
          if (selectedDb === "") setShowSelectDbModal(true);
        }
      } catch (error) {
        console.log(error);
        if (selectedDb === "") setShowSelectDbModal(true);
      }
    };

    const loadFromGist = async (shareId) => {
      try {
        const res = await getGist(shareId);
        const diagramSrc = res.data.files["share.json"].content;
        const d = JSON.parse(diagramSrc);
        setUndoStack([]);
        setRedoStack([]);
        setLoadedFromGistId(shareId);
        setDatabase(d.database);
        setTitle(d.title);
        setTables(d.tables);
        setRelationships(d.relationships);
        setNotes(d.notes);
        setAreas(d.subjectAreas);
        setTransform(d.transform);
        if (databases[d.database].hasTypes) {
          setTypes(d.types ?? []);
        }
        if (databases[d.database].hasEnums) {
          setEnums(d.enums ?? []);
        }
      } catch (e) {
        console.log(e);
        setSaveState(State.FAILED_TO_LOAD);
      }
    };

    const shareId = searchParams.get("shareId");
    if (shareId) {
      // 目前后端暂未支持通过 loadedFromGistId 查询，仍只做 Gist 加载
      window.name = "";
      setId(0);
      await loadFromGist(shareId);
      return;
    }

    if (window.name === "") {
      await loadLatestDiagram();
    } else {
      const name = window.name.split(" ");
      const op = name[0];
      const id = parseInt(name[1]);
      switch (op) {
        case "d": {
          await loadDiagram(id);
          break;
        }
        case "t":
        case "lt": {
          await loadTemplate(id);
          break;
        }
        default:
          break;
      }
    }
  }, [
    setTransform,
    setRedoStack,
    setUndoStack,
    setRelationships,
    setTables,
    setAreas,
    setNotes,
    setTypes,
    setTasks,
    setDatabase,
    database,
    setEnums,
    selectedDb,
    setSaveState,
    searchParams,
    sync,
  ]);

  useEffect(() => {
    if (
      tables?.length === 0 &&
      areas?.length === 0 &&
      notes?.length === 0 &&
      types?.length === 0 &&
      tasks?.length === 0
    )
      return;

    if (settings.autosave) {
      setSaveState(State.SAVING);
    }
  }, [
    undoStack,
    redoStack,
    settings.autosave,
    tables?.length,
    areas?.length,
    notes?.length,
    types?.length,
    relationships?.length,
    tasks?.length,
    transform.zoom,
    title,
    gistId,
    setSaveState,
  ]);

  useEffect(() => {
    save();
  }, [saveState, save]);

  useEffect(() => {
    document.title = "Editor | drawDB";

    load();
  }, [load]);

  useEffect(() => {
    // 仅对 diagram（非模板）连接 WS
    const name = window.name.split(" ");
    const op = name[0];
    if (op === "d" && id) {
      sync?.connect?.(id);
    }
  }, [id, sync]);

  return (
    <div className="h-full flex flex-col overflow-hidden theme">
      <IdContext.Provider value={{ gistId, setGistId }}>
        <ControlPanel
          diagramId={id}
          setDiagramId={setId}
          title={title}
          setTitle={setTitle}
          lastSaved={lastSaved}
          setLastSaved={setLastSaved}
        />
      </IdContext.Provider>
      <div
        className="flex h-full overflow-y-auto"
        onPointerUp={(e) => e.isPrimary && setResize(false)}
        onPointerLeave={(e) => e.isPrimary && setResize(false)}
        onPointerMove={(e) => e.isPrimary && handleResize(e)}
        onPointerDown={(e) => {
          // Required for onPointerLeave to trigger when a touch pointer leaves
          // https://stackoverflow.com/a/70976017/1137077
          e.target.releasePointerCapture(e.pointerId);
        }}
        style={isRtl(i18n.language) ? { direction: "rtl" } : {}}
      >
        {layout.sidebar && (
          <SidePanel resize={resize} setResize={setResize} width={width} />
        )}
        <div className="relative w-full h-full overflow-hidden">
          <CanvasContextProvider className="h-full w-full">
            <Canvas saveState={saveState} setSaveState={setSaveState} />
          </CanvasContextProvider>
          {!(layout.sidebar || layout.toolbar || layout.header) && (
            <div className="fixed right-5 bottom-4">
              <FloatingControls />
            </div>
          )}
        </div>
      </div>
      <Modal
        centered
        size="medium"
        closable={false}
        hasCancel={false}
        title={t("pick_db")}
        okText={t("confirm")}
        visible={showSelectDbModal}
        onOk={() => {
          if (selectedDb === "") return;
          setDatabase(selectedDb);
          setShowSelectDbModal(false);
        }}
        okButtonProps={{ disabled: selectedDb === "" }}
      >
        <div className="grid grid-cols-3 gap-4 place-content-center">
          {Object.values(databases).map((x) => (
            <div
              key={x.name}
              onClick={() => setSelectedDb(x.label)}
              className={`space-y-3 p-3 rounded-md border-2 select-none ${
                settings.mode === "dark"
                  ? "bg-zinc-700 hover:bg-zinc-600"
                  : "bg-zinc-100 hover:bg-zinc-200"
              } ${selectedDb === x.label ? "border-zinc-400" : "border-transparent"}`}
            >
              <div className="flex items-center justify-between">
                <div className="font-semibold">{x.name}</div>
                {x.beta && (
                  <Tag size="small" color="light-blue">
                    Beta
                  </Tag>
                )}
              </div>
              {x.image && (
                <img
                  src={x.image}
                  className="h-8"
                  style={{
                    filter:
                      "opacity(0.4) drop-shadow(0 0 0 white) drop-shadow(0 0 0 white)",
                  }}
                />
              )}
              <div className="text-xs">{x.description}</div>
            </div>
          ))}
        </div>
      </Modal>
    </div>
  );
}
