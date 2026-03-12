import { createContext, useEffect, useState } from "react";
import { ObjectType, Tab } from "../data/constants";
import { useSync } from "../hooks";

export const SelectContext = createContext(null);

export default function SelectContextProvider({ children }) {
  const [selectedElement, setSelectedElement] = useState({
    element: ObjectType.NONE,
    id: -1,
    openDialogue: false,
    openCollapse: false,
    currentTab: Tab.TABLES,
    open: false, // open popover or sidesheet when sidebar is disabled
    openFromToolbar: false, // this is to handle triggering onClickOutside when sidebar is disabled
  });
  const [bulkSelectedElements, setBulkSelectedElements] = useState([]);
  const sync = useSync();

  useEffect(() => {
    if (!sync) return;
    let focusedType = "canvas";
    let focusedId = null;
    if (selectedElement.element === ObjectType.TABLE) {
      focusedType = "table";
      focusedId = selectedElement.id;
    } else if (selectedElement.element === ObjectType.NOTE) {
      focusedType = "note";
      focusedId = selectedElement.id;
    } else if (selectedElement.element === ObjectType.AREA) {
      focusedType = "area";
      focusedId = selectedElement.id;
    }
    sync.sendCursor?.(focusedType, focusedId);
  }, [selectedElement, sync]);

  return (
    <SelectContext.Provider
      value={{
        selectedElement,
        setSelectedElement,
        bulkSelectedElements,
        setBulkSelectedElements,
      }}
    >
      {children}
    </SelectContext.Provider>
  );
}
