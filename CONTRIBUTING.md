# 参与贡献 coldrawdb

感谢你愿意为 coldrawdb 贡献力量。

本文说明本仓库的维护约定。项目按 **OpenLogos** 方法论管理规格与变更；文档语言为 **中文**（见 `logos/logos.config.json` → `locale: "zh"`）。贡献前请先阅读 [README.md](README.md) 与 [`logos/logos-project.yaml`](logos/logos-project.yaml)。

## 目录

1. [我能做什么](#我能做什么)
2. [开发环境](#开发环境)
3. [变更与 PR 流程](#变更与-pr-流程)
4. [代码与文档规范](#代码与文档规范)
5. [测试与验收](#测试与验收)
6. [Issue / PR 标签](#issue--pr-标签)
7. [获取帮助](#获取帮助)

## 我能做什么

### 报告缺陷（Bug）

请使用 GitHub Issue，标题建议以 `[BUG]` 开头，并尽量包含：

- 清晰、可检索的标题
- 完整复现步骤（环境：OS / 浏览器 / 缩放比例等）
- 期望行为与实际行为
- 截图或录屏（UI 问题强烈建议附上）
- 若涉及导入，请附上可复现的样例文件（注意脱敏）

模板：[`.github/ISSUE_TEMPLATE/bug_report.md`](.github/ISSUE_TEMPLATE/bug_report.md)

### 提出增强（Enhancement）

标题建议以 `[Enhancement]` / `[FEATURE]` / `[UI]` 等前缀区分类型，并说明：

- 要解决什么问题、对谁有用
- 期望的交互或能力边界（可附原型截图）
- 非目标（本期不做的范围）有助于评审

模板：[`.github/ISSUE_TEMPLATE/feature_request.md`](.github/ISSUE_TEMPLATE/feature_request.md)

### 提交代码（Pull Request）

- **先 Issue 后实现**：较大功能或行为变更请先开 Issue / 讨论，避免与现行规格冲突。
- **原子 PR**：一个 PR 只做一件事（修一个 bug 或交付一个可验收的小特性）。
- 从最新 `main` 拉分支；PR 需说明动机、方案与测试方式，并链接相关 Issue。
- 涉及规格的改动须走 OpenLogos 变更流程（见下节），不能只改代码不改规格。

## 开发环境

本地运行、构建与 Docker 说明以 [README.md](README.md) 为准。常用入口：

```bash
./scripts/start-local.sh   # 后端 + 前端
./scripts/stop-local.sh
```

技术栈摘要：

| 层 | 路径 | 说明 |
|---|---|---|
| 前端 | `frontend-rs/` | Rust + Leptos CSR + WASM（`trunk`） |
| 后端 | `backend/` | actix-web + SQLite / SeaORM |
| MCP | `mcp-server/` | stdio MCP adapter |
| 规格 | `logos/` | OpenLogos 资源与变更提案 |

无 React / npm 前端构建链；请勿引入与现行栈冲突的前端框架。

## 变更与 PR 流程

本仓库用 `logos/.openlogos-guard` 追踪活跃变更：

- **有 guard** → 可改源码，但须落在当前提案范围内
- **无 guard** → **禁止**修改业务源码；须先 `openlogos change <slug>`

维护者 / 使用 AI 协作时的标准流程：

1. `cd` 到仓库根目录（含 `logos/logos.config.json`），再运行任何 `openlogos` 命令  
2. `openlogos change <slug>` 创建提案并写入 guard  
3. 填写 `logos/changes/<slug>/proposal.md` 与 `tasks.md`，确认后再产出 delta  
4. 用户明确授权后执行 `openlogos merge <slug>`  
5. 按合并后的规格实现代码与测试  
6. 用户明确授权后执行 `openlogos verify`（及如有需要的 deploy / smoke / archive）  
7. `openlogos merge` / `verify` / `smoke` / `archive` / `git push` 均为**人类确认点**，不可隐式自动执行

外部贡献者若不便跑完整 OpenLogos CLI：请在 PR 中写清影响的场景/API/文档路径，并与维护者协调由维护者补齐变更提案与 merge。

快速查看状态：

```bash
openlogos status
openlogos next
```

## 代码与文档规范

### 语言与范围

- **规格、产品文档、用户可见文案**：中文
- **代码标识符、API 字段名、测试 ID**：英文，与现有 crate / OpenAPI 一致
- **新增代码注释**：建议中文（与项目 `locale: "zh"` 对齐）
- 文件命名遵循 OpenLogos 约定（如 `core-SXX-*.md`）；场景编号全局唯一，勿在新模块从 S01 重新编号

### Rust 风格

- 使用 `rustfmt` 格式化；提交前对改动 crate 跑 `cargo fmt`
- 关注 `clippy`（CI 中对 `backend/`、`frontend-rs/` 有检查）
- 前端模块边界受 `frontend-rs/scripts/check_module_deps.sh` 约束，勿随意跨层依赖
- 设计须源于场景时序与 API 规格；测试须带 OpenLogos reporter（见 [`logos/spec/test-results.md`](logos/spec/test-results.md)）

### Git 提交说明

- 使用祈使语气、现在时（如 `fix: 修复导入 dropzone 未绑定 drop`）
- 首行概括「为什么 / 修什么」；正文可链 Issue（如 `Fixes #11`）
- 避免无意义的巨型提交；规格合并与代码实现可按仓库惯例分 commit

## 测试与验收

改动应附带可验证证据，按影响面选择：

```bash
# 后端
cargo test --manifest-path backend/Cargo.toml -- --test-threads=1

# 前端（宿主）
cargo test --manifest-path frontend-rs/Cargo.toml

# 前端 WASM（可选）
cd frontend-rs && wasm-pack test --chrome --headless

# E2E（可选）
cd frontend-rs && npx playwright test

# 本地脚本冒烟（可选）
./scripts/smoke-local-scripts.sh
```

- UT / ST 用例 ID 须与 `logos/resources/test/*.md` 对齐
- 测试结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）
- 大改动可分批，但**每批**须同时交付：业务代码 + 对应测试 + reporter，禁止把测试全部留到最后

CI 参考：[`.github/workflows/build.yml`](.github/workflows/build.yml)。

## Issue / PR 标签

常用标签：

| 标签 | 含义 |
|---|---|
| `bug` | 缺陷 |
| `enhancement` | 增强 / 新功能 |
| `documentation` | 文档 |
| `good first issue` | 适合新人 |
| `help wanted` | 需要协助 |
| `question` | 讨论 / 澄清 |

标题前缀建议与近期 Issue 一致：`[BUG]`、`[Enhancement]`、`[UI]` 等。

## 获取帮助

- 先查 [README.md](README.md)、[`logos/resources/`](logos/resources/)、相关 Issue
- 仍不清楚：在对应 Issue / PR 中提问，或新开 `question` Issue

---

再次感谢你的贡献。
