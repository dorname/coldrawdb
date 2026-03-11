import { get, post, del } from "../utils/requestApi";

const unwrap = (resp) => {
  if (!resp || resp.code !== 200) {
    throw new Error(resp?.message || "Diagram API error");
  }
  return resp.data;
};

const mapFromBackend = (vo) => {
  if (!vo) return null;
  return {
    id: vo.id,
    database: vo.database || null,
    title: vo.name || "Untitled Diagram",
    tables: vo.tables || [],
    relationships: vo.references || [],
    notes: vo.notes || [],
    areas: vo.areas || [],
    todos: vo.tasks || [],
    pan: vo.pan ? JSON.parse(vo.pan) : { x: 0, y: 0 },
    zoom: vo.zoom ? parseFloat(vo.zoom) : 1,
    lastModified: vo.lastModified || null,
    gistId: vo.gistId || "",
    loadedFromGistId: vo.loadedFromGistId || "",
    enums: vo.enums || [],
    types: vo.types || [],
  };
};

const mapToBackend = (diagram) => {
  const {
    id,
    database,
    title,
    tables,
    relationships,
    notes,
    areas,
    todos,
    pan,
    zoom,
    gistId,
    loadedFromGistId,
    enums,
    types,
    lastModified,
  } = diagram;

  return {
    id: id ? String(id) : "",
    database: database || null,
    name: title,
    tables,
    areas,
    references: relationships,
    notes,
    tasks: todos,
    pan: JSON.stringify(pan || { x: 0, y: 0 }),
    zoom: String(zoom ?? 1),
    lastModified: lastModified || new Date().toISOString(),
    gistId: gistId || "",
    loadedFromGistId: loadedFromGistId || "",
    enums: enums || [],
    types: types || [],
  };
};

export const diagramService = {
  async list() {
    const resp = await get("/diagrams/queryAll");
    const list = unwrap(resp) || [];
    return list.map(mapFromBackend);
  },

  async getById(id) {
    const resp = await get(`/diagrams/query/${id}`);
    const data = unwrap(resp);
    return mapFromBackend(data);
  },

  async getLatest() {
    const resp = await get("/diagrams/latest");
    const data = unwrap(resp);
    return mapFromBackend(data);
  },

  async create(diagram) {
    const payload = mapToBackend(diagram);
    const resp = await post("/diagrams/add", payload);
    const data = unwrap(resp);
    return mapFromBackend({
      ...payload,
      id: data.id ?? payload.id,
    });
  },

  async update(diagram) {
    const payload = mapToBackend(diagram);
    const resp = await post("/diagrams/update", payload);
    unwrap(resp);
    return diagram;
  },

  async delete(id) {
    const resp = await del(`/diagrams/delete/${id}`);
    unwrap(resp);
  },
};

