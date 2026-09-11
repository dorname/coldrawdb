## MODIFIED — 新增「场景总览」+ 重写「约束与边界」

> 模块：core | 提案：fill-baseline-gaps
> 路径：`logos/resources/prd/1-product-requirements/core-01-requirements.md`
> 策略：
> 1. 在 §1 用户故事 与 原 §2 功能性需求 之间，**新增 §2 场景总览**（5 行表格）
> 2. 原 §2/§3/§4/§5 依次顺延为 §3/§4/§5/§6（章号重排）
> 3. 原 §4「范围边界（V1 不做）」改写为 §5「约束与边界」，**拆为 5.1 技术约束 / 5.2 资源与时间约束 / 5.3 "不做"清单**
> 4. §5（现 §6）验收条件正文不变，仅章号同步
> 5. US / FR / NFR 三个表格正文保持不变（仅依赖章号同步）

# V1 产品需求（事实锚点）

## 1. 用户故事

| 编号 | 故事 | 优先级 | 场景 |
|---|---|---|---|
| US-01 | 作为数据库设计者，我能在浏览器中创建、编辑、保存数据库 ER 图，无需后端账号 | P0 | S01 |
| US-02 | 作为数据库设计者，我能将图导出为 SQL 脚本（7 种引擎），便于迁移到真实数据库 | P0 | S01 + 桥接 |
| US-03 | 作为数据库设计者，我能从 SQL 脚本 / DBML / JSON 导入图 | P0 | 桥接 |
| US-04 | 作为数据库设计者，我的编辑会自动保存到 SQLite，且不会出现并发覆盖 | P0 | S01 |
| US-05 | 作为数据库设计者，我能在分享链接中加载图 | P1 | S02 |
| US-06 | 作为数据库设计者，我能撤销/重做误操作 | P0 | CAP-EDIT-01 |
| US-07 | 作为数据库设计者，我能校验图的合法性（重复表名、空字段名等） | P1 | CAP-EDIT-04 |
| US-08 | 作为数据库设计者，我的图是私有的（不存在公共分享 / 协作） | P0 | V1 限制 |

## 2. 场景总览

> 场景编号与 `core-00-scenario-overview.md` 一致；详述见 `core-04-scenario-detail.md`；痛点溯源见 `core-03-pain-points.md`

| 场景 ID | 名称 | 触发条件 | 关联痛点 | 优先级 | 状态 |
|---|---|---|---|---|---|
| S01 | 编辑并保存图表 | 用户在 `/editor` 进行任意画布操作 | P02 / P03 | P0 | ✅ V1 |
| S02 | 加载分享链接图表 | URL 含 `?share=<id>` | P01 | P1 | ✅ V1 |
| S03 | 用户注册 / 登录 / Token 续期 | 新用户首次访问 | P03 | P2 | ❌ V2 |
| S04 | 创建/加入协作房间 | 团队评审需求 | P03 | P2 | ❌ V2 |
| S05 | OT 实时协作 | 多端同时编辑 | P03 | P2 | ❌ V2 |

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

## 4. 非功能性需求（Non-Functional Requirements）

| NFR ID | 需求 | 度量 |
|---|---|---|
| NFR-01 | 浏览器端可用 | Chrome / Edge / Firefox / Safari 最新版 |
| NFR-02 | 性能（编辑画布） | 100 张表 60fps 渲染 |
| NFR-03 | 自动保存延迟 | 1s debounce |
| NFR-04 | API P95（保存） | < 200ms（release build，W4 perf 实测） |
| NFR-05 | API P95（读取） | < 100ms |
| NFR-06 | 部署 | Docker / GitHub Actions CI green |
| NFR-07 | 国际化 | V1 不实现（drawdb 30+ 语言，coldrawdb V1 仅有英文） |
| NFR-08 | 主题 | V1 不实现（drawdb 有，coldrawdb V1 单一主题） |
| NFR-09 | 模板 | V1 不实现 |
| NFR-10 | 协作 | **V1 不实现**（V2 计划） |
| NFR-11 | 离线模式 | V1 不实现（drawdb 客户端 Dexie 缓存；coldrawdb V1 完全依赖 SQLite 后端） |
| NFR-12 | 安全 | 无用户系统；diagram 数据仅按 diagram id 区分，不鉴权；适合开发/内部使用 |

| NFR-13 | 前端 WASM 体积（启用 Monaco 后） | Monaco 语言包增量 ≤ 3 MB（gzipped），按需 lazy-load，不阻塞首屏 |
| NFR-14 | 设计 token 体系 | 全部视觉属性通过 `--cdb-*` CSS 变量引用，禁止硬编码色值；token 列表见 `core-07-design-tokens.md` |
| NFR-15 | 主题切换 | 支持 light / dark 模式全局切换，token 覆盖规则见 `core-0b-dark-mode.md`；默认 light，遵循 `prefers-color-scheme` |
| NFR-16 | 动效一致性 | 模态/抽屉/按钮 hover/active 使用统一动效 token（`--cdb-duration-*` + `--cdb-easing-*`），规范见 `core-0c-motion.md` |

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

> 明确列出 V1 不做的功能与场景，避免范围蠕变

- ❌ 用户注册/登录（V2）
- ❌ 协作房间与实时 OT（V2）
- ❌ Mermaid 导出（V2 候选）
- ❌ PNG/JPG/SVG/PDF/ZIP 导出（V2 候选）
- ❌ 模板（V2 候选）
- ❌ 国际化（V2 候选）
- ❌ 客户端 Dexie 离线缓存（V1 完全后端化）
- ❌ 全屏模式（V2 候选）
- ❌ 自动布局（V2 候选）

## 6. 验收条件

V1 通过条件：
- [ ] Phase 4 CI 全绿（W4 perf 已记录）
- [ ] drawdb 主分支能力清单（`docs/drawdb-capability-checklist.md` §6）的 ✅ 行 100% 在 coldrawdb V1 中可演示
- [ ] 11 张表可读写无错
- [ ] 7 引擎 SQL 导入导出可演示
- [ ] 409 revision 冲突可演示