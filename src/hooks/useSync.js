import { useContext } from "react";
import { SyncContext } from "../context/SyncContext";

export default function useSync() {
  return useContext(SyncContext);
}

