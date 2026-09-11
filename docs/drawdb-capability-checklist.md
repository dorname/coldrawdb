# drawdb 能力比对清单（V1 写作母版）

> 提案：add-baseline-docs
> 拉取时间：2026-06-08
> 源仓库：https://github.com/drawdb-io/drawdb.git（GitHub 公开仓库 `main` 分支）
> 本地路径：`/tmp/drawdb-ref`（`git clone --depth 1` 拉取，仅作只读查阅）
> 用途：作为 `add-baseline-docs` V1 阶段的事实锚点；V1 文档中提及的任何"功能 / 界面 / 能力"必须能在此清单中找到对应项
> 维护规则：drawdb 主分支有变更时，由 `add-baseline-docs` 维护者手工同步本表

## 0. 仓库元信息

| 字段 | 值 |
|---|---|
| Name | `drawdb` |
| 技术栈 | React 18 + Vite 6 + TailwindCSS 4 + Semi UI + Lexical + Monaco + Dexie + Framer Motion + i18next |
| 关键依赖 | `@dbml/core` / `node-sql-parser` / `oracle-sql-parser` / `html-to-image` / `jspdf` / `jszip` / `framer-motion` |
| 包大小（clone） | 5.4 MB（`--depth 1`） |
| 国际化 | 30+ 地区（`src/i18n/locales/*.js`） |
| 模板 | 6 个（`src/templates/template1..6.js`） |
| 描述 | Free, simple, and intuitive database schema editor and SQL generator |

## 1. 功能（Capabilities）

### 1.1 核心画布对象（`src/components/EditorCanvas/`）

| 能力 ID | 能力 | 实现位置 | 后端实体（coldrawdb） |
|---|---|---|---|
| CAP-CANVAS-01 | 表（Table）：含字段、主键、唯一、非空、自增、默认值、check、注释、颜色、坐标、锁定 | `Table.jsx` | `table` + `field` + `table_link` |
| CAP-CANVAS-02 | 字段（Field）：类型 + 大小 + 默认 + check + 主键/唯一/非空/自增 + 注释 | `Table.jsx` + `src/data/datatypes.js` | `field` |
| CAP-CANVAS-03 | 关系（Relationship）：一对一/一对多/多对多；起点/终点字段；ON UPDATE/DELETE | `Relationship.jsx` | `reference` |
| CAP-CANVAS-04 | 索引（Index）：字段组合、唯一 | （属于表的能力一部分） | `indice` + `indice_link` |
| CAP-CANVAS-05 | 区域（Subject Area / Area）：画布上的视觉分组 | `Area.jsx` | `area` |
| CAP-CANVAS-06 | 便签（Note）：自由文本注释，锚定到画布 | `Note.jsx` | `note` |
| CAP-CANVAS-07 | 枚举（Enum）：命名枚举值集合 | `src/components/EditorSidePanel/EnumsTab/` | （coldrawdb V1 暂未在 DB 中建独立 enum 表，**待对齐**） |
| CAP-CANVAS-08 | 自定义类型（Custom Type）：用户扩展的数据类型 | `src/components/EditorHeader/ConfigureCustomTypes/` | （coldrawdb V1 暂未独立建模，**待对齐**） |
| CAP-CANVAS-09 | 画布平移/缩放/选择 | `Canvas.jsx` + `useTransform.js` + `useSelect.js` | （前端状态，**无后端实体**） |

### 1.2 编辑器能力

| 能力 ID | 能力 | 实现位置 | 后端实体 |
|---|---|---|---|
| CAP-EDIT-01 | 撤销/重做（Undo/Redo） | `UndoRedoContext.jsx` + `useUndoRedo.js` | `operation_log`（coldrawdb V1 **未持久化**，仅前端内存） |
| CAP-EDIT-02 | 拖拽排序（dnd-kit） | `src/components/SortableList/` | （前端状态） |
| CAP-EDIT-03 | 自动布局 | `src/utils/arrangeTables.js` | （前端算法） |
| CAP-EDIT-04 | 校验（Issues）：重复表名、空字段名、字段类型合法 | `src/utils/validateSchema.js` + `src/components/EditorSidePanel/Issues.jsx` | （前端校验） |
| CAP-EDIT-05 | 多语言 | `src/i18n/locales/*.js` | （前端） |
| CAP-EDIT-06 | 主题（themed page） | `useThemedPage.js` | （前端） |
| CAP-EDIT-07 | 模板（Templates）：6 个示例图 | `src/templates/template1..6.js` | （前端） |
| CAP-EDIT-08 | 客户端持久化（IndexedDB / Dexie） | `src/data/db.js` + `src/utils/cache.js` | （coldrawdb V1 替换为 SQLite 持久化） |
| CAP-EDIT-09 | 全屏模式 | `useFullscreen.js` | （前端） |

### 1.3 桥接（Bridge）— 与 coldrawdb 后端 API 对应

| 能力 ID | 能力 | 实现位置 | coldrawdb V1 路由 |
|---|---|---|---|
| CAP-BRIDGE-01 | SQL 导出 | `src/utils/exportSQL/{index,mysql,postgres,sqlite,mariadb,mssql,oraclesql,generic,shared}.js` | `POST /bridge/...` |
| CAP-BRIDGE-02 | SQL 导入 | `src/utils/importSQL/{index,mysql,postgres,sqlite,mariadb,mssql,oraclesql,shared}.js` | `POST /bridge/import/local` |
| CAP-BRIDGE-03 | DBML 导入/导出 | `src/utils/importFrom/dbml.js` + `src/utils/exportAs/dbml.js` | `GET/PUT /bridge/config`（DBML 模板） |
| CAP-BRIDGE-04 | JSON 导入/导出 | （drawdb 使用 `file-saver` + Blob） | `POST /diagrams/import` |
| CAP-BRIDGE-05 | Mermaid 导出 | `src/utils/exportAs/mermaid.js` | （coldrawdb V1 **未实现**，**待对齐**） |
| CAP-BRIDGE-06 | PNG / JPG / SVG 图片导出 | `src/utils/exportAs/{documentation,...}.js` + `html-to-image` | （coldrawdb V1 **未实现**，**待对齐**） |
| CAP-BRIDGE-07 | PDF 导出 | `jspdf` | （coldrawdb V1 **未实现**，**待对齐**） |
| CAP-BRIDGE-08 | ZIP 批量导出 | `jszip` | （coldrawdb V1 **未实现**，**待对齐**） |
| CAP-BRIDGE-09 | bridge 配置 | `src/data/editorConfig.js` | `GET/PUT /bridge/config` |
| CAP-BRIDGE-10 | 导入日志 | （drawdb 无服务端） | `GET /bridge/import/local/logs` |
| CAP-BRIDGE-11 | 本地重试 | （drawdb 无服务端） | `POST /bridge/import/local/retry/{id}` |

### 1.4 数据库引擎（`src/data/databases.js` + `constants.js`）

| DB 引擎 ID | 引擎名 | hasTypes | hasUnsignedTypes | hasEnums | hasArrays |
|---|---|---|---|---|---|
| `mysql` | MySQL | ❌ | ✅ | ❌ | ❌ |
| `postgresql` | PostgreSQL | ✅ | ❌ | ✅ | ✅ |
| `sqlite` | SQLite | ❌ | ❌ | ❌ | ❌ |
| `mariadb` | MariaDB | ❌ | ✅ | ❌ | ❌ |
| `transactsql` | Microsoft SQL Server | ❌ | ❌ | ❌ | ❌ |
| `oraclesql` | Oracle SQL | ❌ | ❌ | ❌ | ❌ |
| `generic` | Generic SQL | ❌ | ❌ | ❌ | ❌ |

### 1.5 数据类型（`src/data/datatypes.js`，2,259 行）

按引擎分组的预置类型集合，覆盖：

- 整数族：INT / INTEGER / BIGINT / SMALLINT / MEDIUMINT / TINYINT
- 浮点：FLOAT / DOUBLE / DECIMAL / REAL
- 字符串：VARCHAR / VARCHAR2 / CHAR / TEXT / TINYTEXT / MEDIUMTEXT / LONGTEXT
- 布尔：BOOLEAN / BIT
- 日期时间：DATE / DATETIME / TIMESTAMP / TIMESTAMPTZ / TIME / YEAR
- 二进制：BLOB / TINYBLOB / MEDIUMBLOB / LONGBLOB / BYTEA
- 文档：JSON / JSONB / XML
- 几何：POINT / LINE / POLYGON / CIRCLE
- 网络：INET / CIDR / MACADDR
- 唯一标识：UUID / SERIAL / BIGSERIAL
- 枚举：ENUM / SET
- 向量：VECTOR（自定义）
- 颜色编码（用于画布色块）：stringColor / intColor / booleanColor / dateColor / decimalColor / binaryColor / documentColor / enumSetColor / geometricColor / networkIdColor / vectorColor / otherColor

### 1.6 持久化

| 能力 ID | 能力 | drawdb 实现 | coldrawdb V1 实现 |
|---|---|---|---|
| CAP-PERSIST-01 | 本地持久化 | Dexie / IndexedDB | SQLite（`backend/db.sqlite`） + `sqlx` 迁移 |
| CAP-PERSIST-02 | 云端持久化 | Gist（GitHub Gist API） | 自建 REST API + revision 乐观锁 |
| CAP-PERSIST-03 | 分享链接 | （drawdb V1 Gist + URL） | `?share=xxx` URL（**待对齐**：coldrawdb V1 通过桥接配置实现） |
| CAP-PERSIST-04 | 导入/导出 JSON | ✅ | ✅（`POST /diagrams/import`） |
| CAP-PERSIST-05 | 撤销/重做持久化 | ❌（仅内存） | ❌（**待对齐**：V1 仍仅内存） |

### 1.7 API 客户端（`src/api/`）

| 能力 ID | 能力 | 实现位置 | coldrawdb 对应 |
|---|---|---|---|
| CAP-API-01 | Email 收集 | `src/api/email.js` | （coldrawdb V1 **未实现**，**待对齐**） |
| CAP-API-02 | Gist 分享 | `src/api/gists.js` | `POST /bridge/import/local`（功能相似但不同源） |

## 2. 界面（UI）

### 2.1 顶层布局（`src/components/Editor.jsx` 入口 → `Workspace.jsx`）

```
+----------------------------------------------------------+
| EditorHeader（顶部菜单：新建/打开/导入/导出/撤销/重做/分享）|
+--------+----------------------------------+--------------+
|        |                                  |              |
|        |                                  |              |
| Control|        EditorCanvas              |   Editor     |
| Panel  |   （Table / Area / Note /        |   SidePanel  |
| (左)   |    Relationship / Canvas）       |   (右)        |
|        |                                  |              |
|        |                                  |              |
+--------+----------------------------------+--------------+
| Status bar（保存状态、撤销/重做指示、协作提示）          |
+----------------------------------------------------------+
```

### 2.2 顶部菜单（`src/components/EditorHeader/`）

| 模态 | 用途 | 对应 coldrawdb V1 |
|---|---|---|
| `Modal/New.jsx` | 新建空白图 | `POST /api/v1/diagrams` |
| `Modal/Open.jsx` | 打开本地缓存列表 | Dexie 替换为 SQLite 查询 |
| `Modal/ImportDiagram.jsx` | 导入图（JSON/SQL/DBML/Mermaid） | `POST /api/v1/diagrams/import` + bridge |
| `Modal/ImportSource.jsx` | 选择导入源 | `POST /api/v1/bridge/import/local` |
| `Modal/Language.jsx` | 切换语言 | （前端） |
| `Modal/SetTableWidth.jsx` | 表宽度调整 | （前端） |
| `Modal/Share.jsx` | 分享链接生成 | （coldrawdb V1 **未实现**，**待对齐**） |
| `Modal/Rename.jsx` | 重命名图 | `PUT /api/v1/diagrams/{id}` |
| `ConfigureCustomTypes/` | 自定义类型管理 | （coldrawdb V1 **未独立建模**） |
| `LayoutDropdown.jsx` | 布局下拉 | （前端） |
| `ColorPicker.jsx` | 颜色选择 | （前端） |

### 2.3 画布（`src/components/EditorCanvas/`）

- `Canvas.jsx`：容器，含平移/缩放/选区
- `Table.jsx`：单表渲染（标题栏 + 字段列表 + 调整手柄 + 右键菜单）
- `Area.jsx`：主题区域矩形 + 标签
- `Note.jsx`：便签（Lexical 富文本）
- `Relationship.jsx`：贝塞尔连线 + 端点标签 + 箭头

### 2.4 侧边栏（`src/components/EditorSidePanel/`）

| Tab | 内容 | 关键文件 |
|---|---|---|
| Tables | 表列表 + 搜索 | `TablesTab/TablesTab.jsx` |
| Areas | 区域列表 + 搜索 | `AreasTab/AreasTab.jsx` |
| Enums | 枚举列表 + 详情 | `EnumsTab/EnumsTab.jsx` |
| Notes | 便签列表 + 富文本编辑 | `NotesTab/NotesTab.jsx` |
| Relationships | 关系列表 + 详情 | `RelationshipsTab/RelationshipsTab.jsx` |
| Types | 自定义类型管理 | `TypesTab/TypesTab.jsx` |
| DBMLEditor | DBML 全文编辑 | `DBMLEditor.jsx` |
| Issues | 校验问题列表 | `Issues.jsx` |

### 2.5 控制面板（`src/components/`）

- `ControlPanel.jsx`：缩放/网格/主题切换
- `FloatingControls.jsx`：悬浮操作按钮
- `Navbar.jsx`：全站导航
- `SimpleCanvas.jsx`：简化版画布（用于 LandingPage）
- `Thumbnail.jsx`：缩略图
- `Workspace.jsx`：编辑器主工作区组合

### 2.6 动画（`src/animations/`）

- `FadeIn.jsx`、`SlideIn.jsx`：Framer Motion 包装

## 3. 状态管理（`src/context/` + `src/hooks/`）

| Context / Hook | 职责 | coldrawdb V1 对应 |
|---|---|---|
| `AreasContext` | 区域状态 | `frontend-rs/editor_core` 内 |
| `CanvasContext` | 画布状态 | `frontend-rs/editor_core` |
| `CollabContext` | 协作上下文（**drawdb V1 是 stub**） | （coldrawdb V1 不实现服务端 collab） |
| `DiagramContext` | 表/字段/索引/关系状态 | `frontend-rs/editor_core` |
| `EnumsContext` | 枚举状态 | `frontend-rs/editor_core` |
| `ExtensionsContext` | 扩展点 | （待对齐） |
| `LayoutContext` | 布局状态 | `frontend-rs/editor_panels` |
| `NotesContext` | 便签状态 | `frontend-rs/editor_core` |
| `SaveStateContext` | 保存状态指示 | `frontend-rs/editor_data_access` |
| `SelectContext` | 选择状态 | `frontend-rs/editor_render` |
| `TransformContext` | 平移/缩放变换 | `frontend-rs/editor_render` |
| `UndoRedoContext` | 撤销/重做 | `frontend-rs/editor_core` |

## 4. 路由 / 页面（`src/pages/`）

| 路由 | 页面 | 说明 |
|---|---|---|
| `/` | `LandingPage.jsx` | 首页 + hero diagram + survey |
| `/editor` | `Editor.jsx` | 编辑器主页面 |
| `/templates` | `Templates.jsx` | 模板画廊 |
| `/bug-report` | `BugReport.jsx` | Bug 上报 |
| `/*` | `NotFound.jsx` | 404 |

## 5. 第三方集成（`src/data/socials.js` + 组件）

- 社交分享：Twitter Tweet（`react-tweet`）
- Vercel Analytics（`@vercel/analytics`）
- i18n 30+ 语言
- Discord 链接
- Warp 赞助

## 6. V1 已实现 vs 缺失能力（与 drawdb 对齐矩阵）

| 能力 | drawdb 主分支 | coldrawdb V1 | V1 是否必须补齐 |
|---|---|---|---|
| 表/字段/关系/索引/区域/便签编辑 | ✅ | ✅ | — |
| 撤销/重做 | ✅（内存） | ✅（内存） | — |
| 7 数据库引擎 SQL 导出 | ✅ | ✅ | — |
| SQL 导入 | ✅ | ✅ | — |
| JSON 导入/导出 | ✅ | ✅ | — |
| DBML 导入/导出 | ✅ | ✅（部分） | **V1 必补** |
| Mermaid 导出 | ✅ | ❌ | **V1 不补**（V2 候选） |
| PNG/JPG/SVG 图片导出 | ✅ | ❌ | **V1 不补** |
| PDF 导出 | ✅ | ❌ | **V1 不补** |
| ZIP 批量导出 | ✅ | ❌ | **V1 不补** |
| 模板（6 个） | ✅ | ❌ | **V1 不补** |
| 国际化（30+） | ✅ | ❌ | **V1 不补** |
| 主题切换 | ✅ | ❌ | **V1 不补** |
| 客户端 Dexie 缓存 | ✅ | （替换为 SQLite） | — |
| 服务端持久化 + revision 乐观锁 | ❌ | ✅ | — |
| 桥接桥接配置（DBML 模板等） | ❌ | ✅ | — |
| 协作（服务端 collab） | ❌ | ❌ | **V2 实现** |
| Email 收集 | ✅ | ❌ | **V1 不补** |
| Gist 分享 | ✅ | ❌ | **V1 不补** |
| 全屏模式 | ✅ | ❌ | **V1 不补** |
| 校验 Issues | ✅ | ❌ | **V1 必补** |
| 自动布局 | ✅ | ❌ | **V1 不补** |
| dnd-kit 拖拽排序 | ✅ | ❌ | **V1 不补** |

## 7. V1 文档 → 能力映射（写作检查表）

> 每份 V1 文档必须在文档开头（或合适位置）引用本表的能力 ID，作为"已对齐 drawdb 母版"的证据。

| V1 文档 | 应覆盖的能力 ID（部分） |
|---|---|
| `core-00-scenario-overview.md` | 全部 CAP-*（场景总览） |
| `core-01-requirements.md` | CAP-CANVAS-01..09 + CAP-EDIT-* + CAP-PERSIST-* |
| `core-00-information-architecture.md` | 顶层布局 §2.1 + 路由 §4 |
| `core-01-editor-canvas.md` | CAP-CANVAS-01..09 + §2.3 |
| `core-02-diagram-persistence.md` | CAP-PERSIST-* |
| `core-03-bridge-io.md`（如新增） | CAP-BRIDGE-01..11 |
| `core-04-side-panel-tabs.md`（如新增） | §2.4 + Enums/Types/Areas/Notes/Relationships |
| `core-05-top-menu-modals.md`（如新增） | §2.2 + Share / New / Open / Import |
| `core-01-architecture-overview.md` | §3 + coldrawdb 4 模块映射 |
| `core-01-deployment-plan.md` | 部署拓扑 + 环境 + Docker |
| `diagrams.yaml` | `/api/v1/diagrams/*` 5 端点 + 409 revision |
| `bridge.yaml` | CAP-BRIDGE-01..11 + `/api/v1/bridge/*` 5 端点 |
| `coldrawdb-v1.sql` | 11 张表对齐 §1.1 + §1.3 |
| `core-S01-test-cases.md` | CAP-EDIT-* + CAP-PERSIST-01/02 |
| `core-S02-test-cases.md` | CAP-PERSIST-04 + CAP-BRIDGE-04 |
| `core-implementation-checklist.md` | 6 矩阵行的"coldrawdb V1"列（✅/❌） |

## 8. 引用与维护

- 源仓库：https://github.com/drawdb-io/drawdb
- 官方文档站：https://drawdb.app/
- 配套后端：https://github.com/drawdb-io/drawdb-server
- 拉取命令：`git clone --depth 1 https://github.com/drawdb-io/drawdb.git /tmp/drawdb-ref`
- 维护触发：drawdb 主分支有功能新增/删除/重命名时，由 `add-baseline-docs` 维护者重新拉取并更新本表
- 同步 PR：建议将本表变更与 `add-baseline-docs` 的 V1 文档同步提交
