# 贡献指南

欢迎提交问题、改进文档、补充测试或参与功能开发。请先阅读 [README](README.md)、[项目资源索引](logos/logos-project.yaml) 与 [仓库协作规则](AGENTS.md)。本文以当前仓库的开发入口为依据；涉及产品行为的变更按 OpenLogos 的 Why → What → How 流程推进。

## 报告问题与提出建议

报告缺陷使用 [Bug 模板](.github/ISSUE_TEMPLATE/bug_report.md)，说明版本或提交号、操作系统、浏览器、部署方式、复现步骤、期望行为和实际行为。画布问题请补充浏览器缩放比例、设备像素比及截图；导入问题请提供最小脱敏样例。日志中应移除 Token、密码、数据库连接凭据及业务数据。

功能建议使用 [功能模板](.github/ISSUE_TEMPLATE/feature_request.md)，说明使用场景、现有问题、预期行为与范围。较大的功能或行为调整，先在 Issue 中与维护者确认方向，再实现。小型文档修正可以直接提交 PR。

疑似安全漏洞不要在公开 Issue 中贴出凭据、可利用细节或真实用户数据。若仓库提供 GitHub 私密漏洞报告入口，请使用该入口；否则先请求维护者提供私密沟通渠道。本文不约定尚未公布的安全邮箱或响应时限。

讨论和评审应尊重参与者，围绕可复现事实、设计约束和代码展开；遇到分歧时提供最小示例与取舍理由。

## 准备开发环境

Fork 仓库并从最新 `main` 创建主题分支，例如 `fix/import-persistence` 或 `docs/contributing`。以下命令均从仓库根目录执行，括号中的子目录切换不会改变当前终端位置。

项目没有根 Cargo workspace，三个 crate 分别位于 `backend/`、`frontend-rs/`、`mcp-server/`。

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
./scripts/start-local.sh
# 默认前端：http://127.0.0.1:8080/editor
# 默认后端：http://127.0.0.1:3000
```

脚本需要 Bash、curl、Rust / Cargo 与 Trunk；Windows 建议使用 WSL2 或 Docker。默认使用本地 SQLite，连接数据库相关测试可能还需要 PostgreSQL 或下载嵌入式 PostgreSQL。结束后运行 `./scripts/stop-local.sh`。

前端主体使用 Rust / Leptos，浏览器测试使用 JavaScript / TypeScript。运行浏览器测试需要 Node.js / npm（CI 使用 Node.js 22），两个测试入口的依赖分别安装：

```bash
# 原型与规格一致性脚本
(cd frontend-rs && npm ci && npx playwright install chromium)

# Playwright 场景测试
(cd frontend-rs/tests/e2e && npm ci && npx playwright install chromium)
```

Linux 如缺浏览器系统依赖，可按环境需要使用 `npx playwright install --with-deps chromium`。应用启动、构建、端口及编译期 API 地址配置见 [README](README.md)。

## 找到改动位置

| 范围 | 主要入口 |
|---|---|
| 编辑状态与持久化请求 | `frontend-rs/src/editor_core.rs`、`frontend-rs/src/editor_data_access.rs` |
| 界面与画布 | `frontend-rs/src/editor_panels.rs`、`frontend-rs/src/editor_render.rs`、`frontend-rs/src/components/` |
| 前端协作与数据字典 | `frontend-rs/src/collab_client.rs`、`frontend-rs/src/editor_dict.rs` |
| HTTP 与协作路由 | `backend/src/main.rs`、`diagrams_v1.rs`、`auth_v1.rs`、`rooms_v1.rs`、`collab_v1.rs` |
| 数据库迁移 | `backend/migrations/` |
| MCP 协议、工具与房间写入 | `mcp-server/src/`、`mcp-server/tests/` |
| 产品、API、测试设计 | `logos/resources/` |
| 活跃与历史变更 | `logos/changes/`、`logos/changes/archive/` |

`logos/logos-project.yaml` 是设计资源入口；其中流程状态可能与已存在的代码实现范围不同。不要把历史原型或历史重构计划当作当前实现要求。

## 设计与变更流程

源码变更必须先完成设计。API 设计源于场景时序图；代码变更应有对应的 API 编排测试。涉及已有功能的迭代使用 Delta 流程。

所有 `openlogos` 命令必须在包含 `logos/logos.config.json` 的仓库根目录执行。

1. 查看 `logos/.openlogos-guard` 和当前提案。已有 guard 时仅能修改提案范围内的源码，不覆盖或删除其他变更的 guard。
2. 无活跃提案时运行 `openlogos change <slug>`，按 CLI 返回路径填写 `proposal.md` 与 `tasks.md`。AI 协作时使用 change-writer Skill。
3. 等用户 / 维护者确认提案后产出 delta；明确授权执行 `openlogos merge <slug>` 后再按合并规格实现。
4. 交付代码、对应 UT / ST 与 OpenLogos reporter。大任务可分批，但每批均须闭环；实现前列出与测试规格对应的用例 ID。
5. 提交测试证据，明确授权后执行 `openlogos verify <slug>`。有部署任务时，部署与 `openlogos smoke` 分别需要明确授权。
6. 验收通过且无部署任务，或部署后 smoke 通过，再明确授权 `openlogos archive <slug>`。规格合并、代码实现和归档按阶段分别提交；AI 按仓库规则自动提交并告知用户。
7. AI 执行 `git push` 前仍需明确授权；维护者合并 PR 与发布也应遵循仓库审批安排。

`merge`、`verify`、部署、`smoke`、`archive`、`git push` 是明确的人类确认点，“按流程走完”不替代授权。README、贡献指南等非方法论文档与不改变语义的纯 typo 可直接修订；产品规格或源码行为变更不能套用此例外。

外部贡献者无法运行 OpenLogos 时，可先提交 Issue 或设计草案，请维护者创建并确认提案、合并规格，再在约定范围内实现。不要以事后补提案替代源码变更的前置设计。

## 编码与文档约定

- 文档、用户文案和新增代码注释使用中文；标识符、API 字段与测试 ID 延续已有命名。
- 保持 Rust / Leptos 技术栈与现有模块边界，避免顺带重构或大面积格式化无关代码。
- 对受影响 crate 使用 `cargo fmt`、`cargo clippy` 检查；已有告警需与新增问题区分，不批量压制告警。
- API、MCP 契约、序列化字段、数据库迁移变更需说明兼容性与旧数据处理方式。已有迁移的历史语义不应随意改写。
- 不提交数据库、日志、Token、真实业务样例、`node_modules/` 或本地产物；依赖变更同时维护对应锁文件。
- 规格文件遵循模块前缀命名，如 `core-SXX-*.md`；场景编号由资源索引统一分配。
- 修改 Markdown / 文本规格后，从磁盘读回受影响片段，并提供实际原文或 diff 供核对。
- 提交标题简明说明变更，例如 `fix: 修复图表导入后字段丢失`；一个 PR 聚焦一个可评审的目标。

## 测试与证据

按改动影响面执行测试；只有文档变化时，检查链接、路径、命令与 diff 即可。后端部分测试依赖相对路径，建议在对应 crate 目录运行：

```bash
# Rust 测试
(cd backend && cargo test -- --test-threads=1)
(cd frontend-rs && cargo test)
(cd mcp-server && cargo test)

# 将 backend 替换为本次改动的 crate，可分别执行格式与静态检查
(cd backend && cargo fmt --check)
(cd backend && cargo clippy --all-targets -- -D warnings)

# 前端模块依赖检查
bash frontend-rs/scripts/check_module_deps.sh
```

WASM 浏览器测试需要另行安装 `wasm-pack` 及其所需的 Chrome / WebDriver 环境：

```bash
(cd frontend-rs && wasm-pack test --chrome --headless)
```

Playwright 场景测试使用独立配置与 OpenLogos reporter：

```bash
(cd frontend-rs/tests/e2e && npx playwright test)
# 单个场景示例
(cd frontend-rs/tests/e2e && npx playwright test specs/s03-auth.spec.ts)
```

配置默认访问 `http://127.0.0.1:8080`，会调用启动脚本，也会在非 CI 环境复用已启动服务。首次编译建议先手动运行启动脚本并等待就绪。测试会创建账号、房间等数据，应使用隔离的开发环境，避免指向生产服务。不要用 `--reporter=list` 覆盖配置，否则会丢失 OpenLogos reporter。

原型或前端交互改动可按范围追加：

```bash
(cd frontend-rs && npm run test:unified-prototype)
(cd frontend-rs && npm run test:spec-parity-a)
# 其他入口见 frontend-rs/package.json，如 test:spec-parity-b～e、test:canvas-perf
```

UT / ST 用例 ID 必须与 `logos/resources/test/` 对齐，新增测试按 [结果契约](logos/spec/test-results.md) 写入 `logos/resources/verify/test-results.jsonl`。可参考 MCP 测试和 `frontend-rs/tests/e2e/reporter/openlogos.ts` 的接入方式。

当前验证链存在以下限制，提交 PR 时须如实说明：

- `build.yml` 中部分后端测试、Clippy、浏览器测试允许失败或忽略退出码；CI 绿色不等于这些检查全部通过。
- `scripts/run-verify-tests.sh` 会跳过一个嵌入式 PostgreSQL 用例，部分浏览器回归失败不阻断，末尾还会重跑声明式 reporter 覆盖部分记录。因此账本中的 `pass` 不能单独作为真实执行成功的证明。
- 报告应保留实际命令、退出状态、失败 / 跳过原因及必要日志。不要把未执行、声明式覆盖或环境失败写成验证通过；本次范围内的失败应修复或交由维护者明确评估。

## 提交与评审 PR

PR 描述应让未参与讨论的人也能复验变更，建议直接使用以下结构：

```markdown
## 问题与结果
说明触发条件、原行为、修改后行为；关联 Issue 与变更提案。

## 范围与兼容性
列出受影响场景、API / 数据 / 配置变化和已知限制。

## 验证
列出实际运行的命令、用例 ID 与结果；标明未运行、失败或跳过项。
界面变化附截图 / 录屏，性能变化附可复现的环境与测量方式。
```

提交前检查 `git diff --check`、确认没有凭据和无关产物、同步必要文档。未准备好合并时使用草稿 PR。根据评审意见补充测试或说明；涉及迁移与部署时附升级、回退和数据备份要求。

贡献者不应在未确认的情况下重写他人的分支历史、推送标签或发布镜像。发布流程由维护者协调：`docker.yml` 负责镜像，`release.yml` 负责部署包；贡献 PR 无需触发发布。

## 许可证与署名

本项目采用 [MIT 许可证](LICENSE)。提交前确认有权贡献相关内容，贡献内容应与项目许可证兼容；引入第三方代码、图片、字体或依赖时保留必要的许可证与署名，并在 PR 中说明来源。不要把参考项目的代码或素材当作无授权限制的内容直接复制。
