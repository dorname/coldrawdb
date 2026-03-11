import JSZip from "jszip";
import { saveAs } from "file-saver";
import { diagramService } from "../services/diagramService";
import { templateService } from "../services/templateService";

export async function exportSavedData() {
  const zip = new JSZip();
  const diagramsFolder = zip.folder("diagrams");

  const diagrams = await diagramService.list();
  for (const d of diagrams) {
    diagramsFolder.file(
      `${d.title || d.id}(${d.id}).json`,
      JSON.stringify(d, null, 2),
    );
  }

  const templatesFolder = zip.folder("templates");
  const templates = await templateService.list();
  const customTemplates = templates.filter((t) => (t.custom ?? 0) === 1);
  for (const t of customTemplates) {
    templatesFolder.file(
      `${t.title || t.id}(${t.id}).json`,
      JSON.stringify(t, null, 2),
    );
  }

  const content = await zip.generateAsync({ type: "blob" });
  const date = new Date();
  saveAs(
    content,
    `${date.getFullYear()}_${date.getMonth()}_${date.getDay()}_export.zip`,
  );
}
