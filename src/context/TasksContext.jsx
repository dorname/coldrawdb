import { createContext, useState } from "react";
import { useSync } from "../hooks";

export const TasksContext = createContext(null);

export default function TasksContextProvider({ children }) {
  const [tasks, setTasks] = useState([]);
  const sync = useSync();

  const updateTask = (id, values) => {
    setTasks((prev) =>
      prev.map((task, i) => (id === i ? { ...task, ...values } : task))
    );
    sync?.sendOp?.({ op: "task_update", data: { index: id, values } });
  };

  return (
    <TasksContext.Provider value={{ tasks, setTasks, updateTask }}>
      {children}
    </TasksContext.Provider>
  );
}
