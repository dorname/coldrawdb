# ADDED — core-02-product-vision.md（新文档，Phase 1 第一节「产品背景与目标」）

> 新增到 `logos/resources/prd/1-product-requirements/core-02-product-vision.md`
> 对应 prd-writer SKILL §Step 6 输出规范第 1 节「产品背景与目标」

---

## ADDED — 一、产品背景与目标

### 1.1 产品定位

**coldrawdb** 是一款**自托管、浏览器端**的数据库 ER 图设计工具，定位为 drawdb 的 Rust Web 重写版本。

**一句话定位**：让数据库设计者无需账号、无需联网、无需客户端安装，即可在浏览器中完成 ER 图的全生命周期管理（设计 → 导入 → 导出 → 持久化 → 分享）。

核心差异点（相对 drawdb 等纯客户端方案）：

- ✅ **真实后端持久化**：基于 SQLite + actix-web，自动保存到服务端，不再受 IndexedDB 缓存清理影响
- ✅ **跨设备访问**：通过 share link 与 HTTP API 在任何浏览器加载同一张图
- ✅ **409 revision 乐观锁**：服务端检测并发覆盖，UI 引导用户决策
- ✅ **Rust 全栈**：单一语言栈（WASM 前端 + actix-web 后端），类型安全 + 部署简单

### 1.2 核心目标

| 目标 | 衡量指标 | 验收方式 |
|---|---|---|
| G1 | drawdb 主分支能力对齐 | 能力清单 ✅ 行 100% 在 V1 中可演示（`docs/drawdb-capability-checklist.md` §6） |
| G2 | 编辑响应 P95 < 200ms | W4 perf 实测：100 张表 / 200 条关系 / 60fps |
| G3 | 自动保存 1s debounce | PUT 触发；失败重试；409 弹冲突对话框 |
| G4 | 7 引擎 SQL 导入导出 | MySQL / PostgreSQL / SQLite / MariaDB / MSSQL / OracleSQL / Generic |
| G5 | 部署零运维 | Docker 单文件 + GitHub Actions CI green |
| G6 | 11 张表可读写无错 | init.sql + database_design.json 双轨对齐 |

### 1.3 目标用户画像

#### 画像 A：自托管偏好的独立开发者 "Alex"

- **角色**：全栈开发者 / 独立 SaaS 创始人
- **典型场景**：业余时间设计个人项目的数据库 schema
- **典型痛点**：不愿意用 dbdiagram.io 等云服务（隐私 + 成本）；drawdb 客户端工具换设备就丢图
- **核心期望**：一键 Docker 部署，所有图保存在自己的服务器上
- **V1 覆盖度**：✅ 完整覆盖（S01 + S02）

#### 画像 B：内部技术团队 DBA "Dana"

- **角色**：中小企业内部 DBA / 数据架构师
- **典型场景**：维护 5-10 个微服务的 schema，需要团队评审
- **典型痛点**：现有工具不支持分享链接、版本散落各处
- **核心期望**：能发链接给团队评审，但不需要公共云账号
- **V1 覆盖度**：⚠️ 部分覆盖（仅私有分享链接，无房间协作；多人并发仅靠 409 冲突检测）

#### 画像 C：教学/学习者 "Lee"

- **角色**：正在学习数据库课程的学生
- **典型场景**：课堂作业需要画 ER 图
- **典型痛点**：客户端工具功能强大但上手成本高；教学平台集成难
- **核心期望**：浏览器即开即用，能导出 SQL 直接在课堂数据库上执行
- **V1 覆盖度**：✅ 覆盖核心需求（导出 7 引擎 SQL）

### 1.4 成功指标（Success Metrics）

#### V1 launch gate（必须全部满足）

- [ ] Phase 4 CI 全绿
- [ ] drawdb 能力清单 ✅ 行 ≥ 95% 可演示
- [ ] P95 编辑响应 < 200ms（W4 perf 实测）
- [ ] 11 张表可读写无错
- [ ] 7 引擎 SQL 导入导出可演示
- [ ] 409 revision 冲突可演示

#### V1 launch 后的扩展指标（V2 候选，非 gate）

- DAU/MAU 留存率 > 30%
- 单实例承载 100 并发用户
- Docker 部署安装 < 5 分钟
- 部署后 7 天内崩溃率 < 0.1%