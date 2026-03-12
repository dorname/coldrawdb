import { createContext, useState } from "react";
import { Action, ObjectType, defaultBlue } from "../data/constants";
import { useUndoRedo, useTransform, useSelect, useSync } from "../hooks";
import { Toast } from "@douyinfe/semi-ui";
import { useTranslation } from "react-i18next";

export const AreasContext = createContext(null);

export default function AreasContextProvider({ children }) {
  const { t } = useTranslation();
  const [areas, setAreas] = useState([]);
  const { transform } = useTransform();
  const { selectedElement, setSelectedElement } = useSelect();
  const { setUndoStack, setRedoStack } = useUndoRedo();
  const sync = useSync();

  const addArea = (data, addToHistory = true) => {
    let areaToAdd;
    if (data) {
      areaToAdd = data;
      setAreas((prev) => {
        const temp = prev.slice();
        temp.splice(data.id, 0, data);
        return temp.map((t, i) => ({ ...t, id: i }));
      });
    } else {
      const width = 200;
      const height = 200;
      areaToAdd = {
        id: areas.length,
        name: `area_${areas.length}`,
        x: transform.pan.x - width / 2,
        y: transform.pan.y - height / 2,
        width,
        height,
        color: defaultBlue,
      };
      setAreas((prev) => [...prev, areaToAdd]);
    }
    if (addToHistory) {
      setUndoStack((prev) => [
        ...prev,
        {
          action: Action.ADD,
          element: ObjectType.AREA,
          message: t("add_area"),
        },
      ]);
      setRedoStack([]);
      sync?.sendOp?.({ op: "area_add", data: areaToAdd });
    }
  };

  const deleteArea = (id, addToHistory = true) => {
    if (addToHistory) {
      Toast.success(t("area_deleted"));
      setUndoStack((prev) => [
        ...prev,
        {
          action: Action.DELETE,
          element: ObjectType.AREA,
          data: areas[id],
          message: t("delete_area", areas[id].name),
        },
      ]);
      setRedoStack([]);
    }
    setAreas((prev) =>
      prev.filter((e) => e.id !== id).map((e, i) => ({ ...e, id: i })),
    );
    if (id === selectedElement.id) {
      setSelectedElement((prev) => ({
        ...prev,
        element: ObjectType.NONE,
        id: -1,
        open: false,
      }));
    }
    if (addToHistory) sync?.sendOp?.({ op: "area_remove", data: { id } });
  };

  const updateArea = (id, values) => {
    setAreas((prev) =>
      prev.map((t) => {
        if (t.id === id) {
          return {
            ...t,
            ...values,
          };
        }
        return t;
      }),
    );
    sync?.sendOp?.({ op: "area_update", data: { id, ...values } });
  };

  return (
    <AreasContext.Provider
      value={{
        areas,
        setAreas,
        updateArea,
        addArea,
        deleteArea,
        areasCount: areas.length,
      }}
    >
      {children}
    </AreasContext.Provider>
  );
}
