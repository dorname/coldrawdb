# 实现任务：align-unified-prototype-and-add-mcp

> 本变更先完成 Why → What → How 规格闭环并等待 merge 授权，再分批实现；每个代码批次必须同时交付业务代码、对应 UT/ST 和 OpenLogos reporter。

## [delta] D1 文档真值与原型入口对齐

- [x] D1.1 更新 `logos/logos-project.yaml`：统一主原型入口、S03～S05 状态说明、S06 场景与 `scenario_counter.next_id: 7`
- [x] D1.2 更新 `core-S03-user-auth-design.md`、`core-S04-room-lifecycle-design.md`、`core-S05-ot-collab-design.md` 顶部元数据和操作指南，旧原型仅标记为历史参考
- [x] D1.3 更新 Phase 1/Phase 3 场景总览：S03～S05 后端已实现、生产前端未接入、静态原型不代表生产能力
- [x] D1.4 更新 `core-implementation-checklist.md`，校正后端模块、编排测试、原型数量与前端缺口
- [x] D1.5 更新 README、AGENTS、CLAUDE、架构说明及统计口径，避免模块/端点/原型数量漂移

## [delta] D2 S06 MCP 需求与设计

- [x] D2.1 新增 S06 产品需求、用户故事、功能/非功能需求、边界和验收条件
- [x] D2.2 新增 `core-S06-*-design.md`：stdio 生命周期、七个工具、权限、人工审批、配置和四客户端接入体验
- [x] D2.3 新增 `core-S06-*.md` 场景时序图：客户端初始化、工具发现、读调用、写调用、409 冲突及错误分支
- [x] D2.4 从 S06 时序图推导 MCP tool contract，定义 JSON Schema、返回结构、tool annotations 和错误映射
- [x] D2.5 更新架构/部署/安全文档：独立 adapter、REST 边界、Token 处理、日志脱敏、超时和回滚
- [x] D2.6 更新 `logos/logos-project.yaml` resource_index，登记 S06 需求、设计、场景、契约、测试和部署资源

## [delta] D3 测试设计与编排

- [x] D3.1 将 ST-PU-01～ST-PU-19 补齐为可自动执行的统一原型验收定义（`deltas/test/core-PU-unified-prototype-test-cases.md`）
- [x] D3.2 新增 `deltas/test/core-S06-test-cases.md`，至少覆盖配置、初始化、tools/list、七工具、权限、revision、错误映射和日志脱敏
- [x] D3.3 新增 S06 MCP 编排测试定义，覆盖完整创建→读取→更新→导出→删除链路及 409/401/403/404/422 分支
- [x] D3.4 为 Claude、Codex、Cursor、OpenCode 配置建立结构校验和兼容性测试
- [x] D3.5 明确所有自动化用例的 OpenLogos reporter ID、写入格式和失败恢复策略

## [code] C1 原型回归固化

- [x] C1.1 [ST-PU-01～ST-PU-18] 将全交互浏览器审计固化为仓库内 Playwright 回归脚本
- [x] C1.2 [ST-PU-19] 复用既有渲染稳定性测试，统一入口和报告输出
- [x] C1.3 [ST-PU-01～ST-PU-19] 接入 verify 预跑并写入 OpenLogos reporter

## [code] C2 MCP 服务骨架与只读工具

- [x] C2.1 [UT-MCP-01～06、UT-MCP-09、UT-MCP-14、ST-MCP-01～02] 列出并覆盖本批 UT/ST ID
- [x] C2.2 新增独立 Rust MCP adapter/service、stdio transport、配置加载和固定路径 HTTP client
- [x] C2.3 实现 MCP initialize、tools/list、优雅 EOF/退出与结构化日志脱敏
- [x] C2.4 实现 `list_diagrams`、`get_diagram`、`export_schema`
- [x] C2.5 同批补齐单元测试、真实 stdio 握手和 OpenLogos reporter

## [code] C3 MCP 写工具与一致性

- [x] C3.1 [UT-MCP-07～08、UT-MCP-10、UT-MCP-12～13、UT-MCP-15、ST-MCP-03～05] 列出并覆盖本批 UT/ST ID
- [x] C3.2 实现 `create_diagram`、`update_diagram`、`delete_diagram`、`import_schema`
- [x] C3.3 实现 readOnly/destructive/idempotent annotations，写操作保持客户端人工批准语义
- [x] C3.4 实现认证透传、revision 乐观锁、401/403/404/409/422/5xx 和网络错误映射
- [x] C3.5 同批补齐单元测试、mock HTTP 编排和 OpenLogos reporter

## [code] C4 四客户端接入与分发

- [x] C4.1 [UT-MCP-11、ST-MCP-06～09] 提供并校验 Claude、Codex、Cursor、OpenCode stdio 配置
- [x] C4.2 提供 `COLDRAWDB_BASE_URL`、可选 Token、启动和安全边界说明
- [x] C4.3 四套 fixture 均以同一真实 MCP 子进程完成 initialize/tools/list
- [x] C4.4 增加 release 构建脚本，不安装客户端配置、不引入公开 HTTP 监听端口

## [verify] 验证

- [x] V1 运行统一原型 ST-PU-01～ST-PU-19，全部通过并生成 reporter
- [x] V2 运行 MCP 单元测试和完整编排测试，校验 reporter 与定义用例一一对应
- [x] V3 验证七个工具 schema、annotations、成功结果和错误映射
- [x] V4 验证服务不直连 SQLite、不输出 Token、不提供任意 SQL 工具
- [x] V5 校验 Claude、Codex、Cursor、OpenCode 配置可解析；执行 MCP 初始化、tools/list 和只读 smoke
- [x] V6 运行现有后端、前端及全量 verify 预跑，确认无回归
- [x] V7 从磁盘读回所有 Markdown/文本规格变更片段，向用户展示实际原文

## [deploy] 部署（须用户明确授权）

- [x] DP1 跳过：用户明确决定本次无需部署并直接归档
- [x] DP2 跳过：未修改任何本地/测试/预发客户端配置
- [x] DP3 跳过：没有部署目标或公网监听端口需要记录

## [smoke] 冒烟（须用户明确授权）

- [x] SM1 跳过：无部署目标，用户授权直接归档
- [x] SM2 跳过：无部署环境需要冒烟
- [x] SM3 跳过：不生成部署 smoke 报告

## [follow-up] 独立后续变更

- [ ] F1 创建独立提案实现 S03～S05 生产前端接入（登录/注册、房间生命周期、WS/OT/presence）；本提案只登记真实缺口，不把原型模拟能力标记为已实现
- [ ] F2 评估 Streamable HTTP、Bearer/OAuth 和远程 MCP 部署需求，未明确授权前不纳入 MVP

## 人类确认点

- [x] H1 用户确认本提案后，才开始产出 delta
- [x] H2 delta 完成后，等待用户明确授权 `openlogos merge align-unified-prototype-and-add-mcp`
- [x] H3 merge 完成后自动提交规格文档，并按合并规格分批实现和提交代码
- [x] H4 用户已授权且 `openlogos verify align-unified-prototype-and-add-mcp` 通过
- [x] H5 用户明确决定无需部署
- [x] H6 用户明确决定无需 smoke
- [x] H7 用户明确授权直接归档，归档已完成
- [ ] H8 归档提交完成后，询问用户是否执行 `git push`
