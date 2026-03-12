import { createContext, useState } from "react";
import { Action, ObjectType, defaultNoteTheme } from "../data/constants";
import { useUndoRedo, useTransform, useSelect, useSync } from "../hooks";
import { Toast } from "@douyinfe/semi-ui";
import { useTranslation } from "react-i18next";

export const NotesContext = createContext(null);

export default function NotesContextProvider({ children }) {
  const { t } = useTranslation();
  const [notes, setNotes] = useState([]);
  const { transform } = useTransform();
  const { setUndoStack, setRedoStack } = useUndoRedo();
  const { selectedElement, setSelectedElement } = useSelect();
  const sync = useSync();

  const addNote = (data, addToHistory = true) => {
    let noteToAdd;
    if (data) {
      noteToAdd = data;
      setNotes((prev) => {
        const temp = prev.slice();
        temp.splice(data.id, 0, data);
        return temp.map((t, i) => ({ ...t, id: i }));
      });
    } else {
      const height = 88;
      noteToAdd = {
        id: notes.length,
        x: transform.pan.x,
        y: transform.pan.y - height / 2,
        title: `note_${notes.length}`,
        content: "",
        color: defaultNoteTheme,
        height,
      };
      setNotes((prev) => [...prev, noteToAdd]);
    }
    if (addToHistory) {
      setUndoStack((prev) => [
        ...prev,
        {
          action: Action.ADD,
          element: ObjectType.NOTE,
          message: t("add_note"),
        },
      ]);
      setRedoStack([]);
      sync?.sendOp?.({ op: "note_add", data: noteToAdd });
    }
  };

  const deleteNote = (id, addToHistory = true) => {
    if (addToHistory) {
      Toast.success(t("note_deleted"));
      setUndoStack((prev) => [
        ...prev,
        {
          action: Action.DELETE,
          element: ObjectType.NOTE,
          data: notes[id],
          message: t("delete_note", { noteTitle: notes[id].title }),
        },
      ]);
      setRedoStack([]);
    }
    setNotes((prev) =>
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
    if (addToHistory) sync?.sendOp?.({ op: "note_remove", data: { id } });
  };

  const updateNote = (id, values) => {
    setNotes((prev) =>
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
    sync?.sendOp?.({ op: "note_update", data: { id, ...values } });
  };

  return (
    <NotesContext.Provider
      value={{
        notes,
        setNotes,
        updateNote,
        addNote,
        deleteNote,
        notesCount: notes.length,
      }}
    >
      {children}
    </NotesContext.Provider>
  );
}
