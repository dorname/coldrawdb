# Bug List

> Generated on 2026-03-10  
> Sources: Conversation context

## Summary
- Total: 1
- Critical: 0, High: 1, Medium: 0, Low: 0

## Bugs

### #1 保存新建图表时 `/diagrams/add` 返回 400
- **Severity**: High
- **Location**: `src/services/diagramService.js`, `backend/src/entity/vo/diagram_vo.rs`, `backend/src/entity/vo/table_vo.rs`, API `POST /diagrams/add`
- **Description**: 当前端在编辑器中新建图表（包含至少一个表，例如默认的 `table_0`）并触发保存时，请求 `POST /diagrams/add` 返回 HTTP 400 Bad Request，响应为纯文本而非封装的 `CommonResponse`。根因是后端 `DiagramVo` 中的 `tables: Option<Vec<TableVo>>` 需要反序列化为强类型 `TableVo`，而 `TableVo` 结构中 `diagram_id: String` 为必填字段；前端发送的 `tables` 元素并不包含 `diagram_id` 字段，导致 Serde 反序列化失败，Actix 将该错误转换为 400。
- **Reproduction**:
  1. 启动后端（监听 `http://127.0.0.1:6666`）和前端 Vite dev server（`http://localhost:5173`）。
  2. 在浏览器访问 `http://localhost:5173` 打开编辑器，保持默认新建的 `table_0` 即可。
  3. 使用编辑器中的保存操作（例如菜单或快捷键）保存当前图表。
  4. 打开浏览器 DevTools 的 Network 面板，观察 `POST /diagrams/add` 请求返回状态码 400，响应类型为 `text/plain; charset=utf-8`，而非预期的 JSON `CommonResponse`，图表未成功持久化到后端。
- **Status**: Open

