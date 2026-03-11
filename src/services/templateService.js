import { get, post, del } from "../utils/requestApi";

const unwrap = (resp) => {
  if (!resp || resp.code !== 200) {
    throw new Error(resp?.message || "Template API error");
  }
  return resp.data;
};

const mapFromBackend = (vo) => {
  if (!vo) return null;
  return {
    id: vo.id,
    title: vo.title || "",
    description: vo.description || "",
    database: vo.database || null,
    custom: vo.custom ?? 0,
    tables: vo.tables || [],
    relationships: vo.relationships || [],
    notes: vo.notes || [],
    subjectAreas: vo.subjectAreas || [],
    todos: vo.todos || [],
    types: vo.types || [],
    enums: vo.enums || [],
    pan: vo.pan ? JSON.parse(vo.pan) : { x: 0, y: 0 },
    zoom: vo.zoom ? parseFloat(vo.zoom) : 1,
  };
};

const mapToBackend = (template) => {
  const {
    id,
    title,
    description,
    database,
    custom,
    tables,
    relationships,
    notes,
    subjectAreas,
    todos,
    types,
    enums,
    pan,
    zoom,
  } = template;

  return {
    id: id ? String(id) : "",
    title,
    description,
    database: database || null,
    custom: custom ?? 0,
    tables,
    relationships,
    notes,
    subjectAreas,
    todos,
    types: types || [],
    enums: enums || [],
    pan: JSON.stringify(pan || { x: 0, y: 0 }),
    zoom: String(zoom ?? 1),
  };
};

export const templateService = {
  async list() {
    const resp = await get("/templates/queryAll");
    const list = unwrap(resp) || [];
    return list.map(mapFromBackend);
  },

  async getById(id) {
    const resp = await get(`/templates/query/${id}`);
    const data = unwrap(resp);
    return mapFromBackend(data);
  },

  async create(template) {
    const payload = mapToBackend(template);
    const resp = await post("/templates/add", payload);
    const data = unwrap(resp);
    return mapFromBackend({
      ...payload,
      id: data.id ?? payload.id,
    });
  },

  async update(template) {
    const payload = mapToBackend(template);
    const resp = await post("/templates/update", payload);
    unwrap(resp);
    return template;
  },

  async delete(id) {
    const resp = await del(`/templates/delete/${id}`);
    unwrap(resp);
  },
};

