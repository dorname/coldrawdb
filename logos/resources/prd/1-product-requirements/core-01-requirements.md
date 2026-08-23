# V1 产品需求（事实锚点）

## 产品需求定位（V1 + V2 工作空间）

> V1 仍覆盖匿名编辑/分享（S01/S02）。V2 工作空间需求（S03～S05）以统一主原型 `core-01-editor-prototype.html` 为体验基线；生产语义以真实 REST/WS 为准。本提案只收口规格，不声明生产前端已逐项完成。


## 1. 用户故事

| 编号 | 故事 | 优先级 | 场景 |
|---|---|---|---|
| US-01 | 作为数据库设计者，我能在浏览器中创建、编辑、保存数据库 ER 图 | P0 | S01 |
| US-02 | 作为数据库设计者，我能将图导出为 SQL 脚本（7 种引擎），便于迁移到真实数据库 | P0 | S01 + 桥接 |
| US-03 | 作为数据库设计者，我能从 SQL 脚本 / DBML / JSON 导入图 | P0 | 桥接 |
| US-04 | 作为数据库设计者，我的编辑会自动保存，且不会静默并发覆盖 | P0 | S01 |
| US-05 | 作为数据库设计者，我能在分享链接中以只读方式加载图（无需登录） | P1 | S02 |
| US-06 | 作为数据库设计者，我能撤销/重做误操作 | P0 | CAP-EDIT-01 |
| US-07 | 作为数据库设计者，我能校验图的合法性（重复表名、空字段名等） | P1 | CAP-EDIT-04 |
| US-08 | 作为团队成员，我能注册/登录并进入房间列表，而不是直接落到空白 Landing | P0 | S03 |
| US-09 | 作为房间 Owner/Editor，我能创建房间、邀请成员并按角色进入协作编辑器 | P0 | S04 |
| US-10 | 作为协作参与者，我能看到在线成员、远端操作与连接态，断线时可感知排队/降级 | P0 | S05 |
| US-11 | 作为 Viewer，我只能查看与接收更新，不能触发写操作或发送协作 op | P0 | S04 / S05 |

## 2. 场景总览

| 场景 ID | 名称 | 触发条件 | 关联痛点 | 优先级 | 状态 |
|---|---|---|---|---|---|
| S01 | 编辑并保存图表 | 用户在 room-editor / 编辑器画布进行任意操作 | P02 / P03 | P0 | ✅ V1 |
| S02 | 加载分享链接图表 | URL 含 `?share=<id>` | P01 | P1 | ✅ V1 |
| S03 | 用户注册 / 登录 / Token 续期 | 未登录用户访问工作空间入口 | P03 | P0 | 🟡 V2 规格对齐中 |
| S04 | 创建/加入协作房间 | 登录后进入 rooms；创建/打开/接受邀请 | P03 | P0 | 🟡 V2 规格对齐中 |
| S05 | OT 实时协作 | 进入 room-editor 后建立 WS | P03 | P0 | 🟡 V2 规格对齐中 |

## 3. 功能性需求（Functional Requirements）

| FR ID | 需求 | 验收条件 | 对齐能力 |
|---|---|---|---|
| FR-01 | 支持表（含字段、主键、唯一、非空、自增、默认值、check、注释） | 创建 / 编辑 / 删除 / 移动；字段类型 7 引擎全覆盖 | CAP-CANVAS-01/02 |
| FR-02 | 支持关系（一对一/一对多/多对多；ON UPDATE/DELETE） | 起止字段可指定；关系类型正确生成 SQL | CAP-CANVAS-03 |
| FR-03 | 支持索引（单字段/复合/唯一） | 表内管理；导出 SQL 含索引 | CAP-CANVAS-04 |
| FR-04 | 支持区域（subject area 视觉分组） | 拖拽创建 / 编辑 / 删除 | CAP-CANVAS-05 |
| FR-05 | 支持便签（自由文本注释） | 富文本编辑；锚定到画布坐标 | CAP-CANVAS-06 |
| FR-06 | 撤销/重做（仅前端内存） | undo/redo 键可用；不持久化 | CAP-EDIT-01 |
| FR-07 | 编辑自动保存到 SQLite | debounce 1s；PUT 触发；失败有重试 | CAP-PERSIST-01 |
| FR-08 | 409 revision 冲突语义 | 携带 `expected_revision`；不匹配返回 409，前端提示用户 | CAP-PERSIST-02 |
| FR-09 | SQL 导出（7 引擎：MySQL / PostgreSQL / SQLite / MariaDB / MSSQL / OracleSQL / Generic） | 选择引擎 → 一键生成 SQL | CAP-BRIDGE-01 |
| FR-10 | SQL 导入（7 引擎） | 选择 SQL 文件 → 解析 → 渲染到画布 | CAP-BRIDGE-02 |
| FR-11 | DBML 导入/导出 | 双向支持 | CAP-BRIDGE-03 |
| FR-12 | JSON 导入/导出 | 通过 `POST /api/v1/diagrams/import` | CAP-BRIDGE-04 / CAP-PERSIST-04 |
| FR-13 | 分享链接加载 | URL `?share=xxx` → 加载图 | CAP-PERSIST-03 |
| FR-14 | 校验 Issues | 重复表名 / 空字段名 / 字段类型合法 | CAP-EDIT-04 |
| FR-15 | 11 张表的数据模型 | diagrams / tables / fields / references / indices / areas / notes 等 | init.sql / database_design.json |

### 3.1 工作空间功能性需求（V2）

在既有 FR-01～FR-15 之上追加：

| FR ID | 需求 | 验收条件 | 对齐能力 |
|---|---|---|---|
| FR-16 | 统一页面流 `auth → rooms → room-editor` | 登录成功进入房间列表；创建/打开/接受邀请进入协作编辑器；全程同壳切换，不以独立历史原型为准 | IA §10 |
| FR-17 | 鉴权会话可见 | 登录/注册校验、通用错误文案、会话续期与退出确认可观察；不把 refresh_token 写入 localStorage | S03 |
| FR-18 | 房间与成员管理 | 房间列表、创建房间、邀请 URL、成员 SideSheet、Owner/Editor/Viewer 权限即时生效 | S04 |
| FR-19 | 协作可见状态 | StatusBar 显示 WS 状态与 server_rev；presence 头像；Activity；重连 Banner 与待同步队列 | S05 |
| FR-20 | 角色只读边界 | Viewer 禁用写工具与 PUT/op 发送，仍可查看远端更新；禁用须同时阻断事件，不只灰显 | S04 / S05 |
| FR-21 | 分享旁路 | `?share=` 只读链路不被鉴权拦截阻断；可与工作空间登录流并存 | S02 |
| FR-22 | 响应式可达性 | ≥1180 / 760～1179 / <760 三档下，登录、进房、建表、邀请、角色切换、断线恢复、导出均可达 | IA §10.5 |
| FR-23 | 可访问性基线 | 关键控件有 label；错误有 `aria-describedby` / `role="alert"`；浮层 Esc 可关且遮罩不残留 | S03～S05 设计 |
| FR-24 | 主题一致性 | auth / rooms / room-editor 共享 light/dark；对比度满足 WCAG AA 4.5:1（见 dark-mode 规格） | NFR-15 |

## 4. 非功能性需求（Non-Functional Requirements）

| NFR ID | 需求 | 度量 |
|---|---|---|
| NFR-08 | 主题 | **已纳入**：支持 light/dark 全局切换；auth/rooms/editor 一致；默认 light，遵循 `prefers-color-scheme` |
| NFR-10 | 协作 | **V2 已纳入需求范围**：房间 + OT/presence/重连；生产实现对齐以主原型规格与 REST/WS 契约为准 |
| NFR-12 | 安全 | V1 匿名 diagram 链路仍可按 id 访问；V2 工作空间路径需鉴权；refresh_token 不得存 localStorage；登录失败不区分用户是否存在 |

其余 NFR-01～07、09、11、13～16 保持既有表述，除非与上表冲突则以本表为准。

## 5. 约束与边界

### 5.1 技术约束

| 约束 | 说明 | 应对策略 |
|---|---|---|
| 前端语言 | 必须用 Rust + Leptos（不能用 JS / TS） | WASM bundle 由 `trunk build` 产出 |
| 后端语言 | 必须用 Rust + actix-web 4（不能用 Node / Go） | 单一语言栈便于团队复用 |
| 数据库 | 仅支持 SQLite WAL 模式（V1 不支持 PostgreSQL / MySQL） | 11 张表 DDL 见 `database_design.json` + `coldrawdb-v1.sql` |
| 部署形态 | 仅支持 Docker 单文件部署（V1 不支持 K8s / 多实例） | 反代用 nginx；数据卷挂 SQLite 文件 |
| 浏览器兼容 | 仅保证 Chrome / Edge / Firefox / Safari 最新版可用 | 不支持 IE / 旧版移动浏览器 |
| 第三方依赖 | 无邮件 / SMS / OAuth / 支付等外部服务 | 见 `core-01-architecture-overview.md` §12 |

### 5.2 资源与时间约束

| 约束 | 说明 |
|---|---|
| 团队规模 | V1 由单人 / 小团队（≤3 人）维护 |
| 开发周期 | V1 launch 须在 Phase 4 W4 perf 实测通过后 |
| 服务端资源 | 单实例 1 CPU / 1 GB RAM 即可承载 100 并发用户 |
| 客户端资源 | WASM bundle < 5 MB（gzipped，含 Monaco lazy-load 后总增量 ≤ 3 MB） |
| 存储预算 | SQLite 单文件 < 1 GB 即可（1000 张图平均 1 MB/张） |

### 5.3 "不做"清单（Out of Scope）

> 明确仍不做的能力；**S03～S05 已进入产品需求范围，不再列作 V2 候选排除项**

- ❌ Mermaid 导出（后续候选）
- ❌ PNG/JPG/SVG/PDF/ZIP 导出（后续候选）
- ❌ 模板库（后续候选）
- ❌ 完整国际化（后续候选；界面文案以中文为主）
- ❌ 客户端 Dexie 离线缓存（当前完全后端化；协作断线仅支持可见排队/本地降级，不等价完整离线）
- ❌ 全屏模式（后续候选）
- ❌ 自动布局（后续候选）
- ❌ 将主原型演示器/模拟远端事件直接当作生产网络能力（演示 ≠ 生产）
- ❌ 以 `core-03/04/05-*-prototype.html` 作为新增功能或验收事实来源

## 6. 验收条件

V1 通过条件：
- [ ] Phase 4 CI 全绿（W4 perf 已记录）
- [ ] drawdb 主分支能力清单（`docs/drawdb-capability-checklist.md` §6）的 ✅ 行 100% 在 coldrawdb V1 中可演示
- [ ] 11 张表可读写无错
- [ ] 7 引擎 SQL 导入导出可演示
- [ ] 409 revision 冲突可演示

### 6.1 V2 工作空间验收条件（规格级）

在既有 V1 验收条件之外，V2 规格收口通过条件：

- [ ] 需求/设计/技术场景对 `auth → rooms → room-editor` 叙述一致
- [ ] S03～S05 状态不再写成「❌ V2 / 范围外」或「仅后端、前端完全未接入」的过时表述
- [ ] 主原型关键 `data-testid` 已映射到测试/验收文档
- [ ] 明确区分：已有部分生产接入 vs 下一变更 `implement-unified-prototype-spec-parity` 待实现的逐项对齐
- [ ] 演示器行为不写入 API/DB 强制契约，除非场景时序已推导
