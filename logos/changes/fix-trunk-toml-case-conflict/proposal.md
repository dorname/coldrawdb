# 变更提案：fix-trunk-toml-case-conflict

> module: core | created: 2026-09-20
> 关联问题：Windows 检出后恒出现 `frontend-rs/Trunk.toml` 「假修改」；Linux/Docker 构建读到错误的 trunk 配置

## 变更原因

Git 索引中同时存在两个**仅大小写不同、内容也不同**的路径：

- `frontend-rs/Trunk.toml`（blob `53dd517`）→ 大段「本文件不留任何字段、由调用方传 `--no-wasm-opt`」注释 + `[watch] ignore = ["tests"]`
- `frontend-rs/trunk.toml`（blob `2a96a53`）→ `[build]`（target/dist）+ `[tools] wasm-opt = false` + `[serve] no_autoreload = true`

由此产生三类真实问题：

1. **Windows / 大小写不敏感文件系统（主诉）**：只能落盘一个物理文件（实测落盘为小写 `trunk.toml`），git 把工作区文件映射到 `Trunk.toml` 后判定内容不符，`git status` 恒显示 ` M frontend-rs/Trunk.toml`（14 insertions / 19 deletions）。所有 Windows 伙伴每次检出都会看到该无关改动，极易误提交、污染 diff 与 PR。
2. **`[watch] ignore = ["tests"]` 实际已失效**：该配置只存在于大写那份（其 `[watch]` 段），而物理落盘内容是小写那份，trunk 读不到 → e2e 期间 Playwright 在 `tests/` 写 trace/截图仍会触发重建并向页面广播 reload。
3. **Linux / Docker（大小写敏感）**：两个文件同时存在，trunk 与 `Dockerfile`（`COPY frontend-rs/index.html frontend-rs/Trunk.toml ./frontend-rs/`）都会命中**大写**那份 → 丢掉 `wasm-opt = false` / `no_autoreload` / `build.target`，构建行为与开发、e2e 不一致。

## 变更类型

代码级修复（构建配置归一 + 部署方案文档中的构建片段同步）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §4.1 镜像构建片段中 `COPY ... frontend-rs/Trunk.toml` 改为小写（**需 delta**）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的源文件（无需 delta，归 `[code]`）：`frontend-rs/trunk.toml`（归一为唯一路径并合并两份配置）、`Dockerfile`（`COPY ... trunk.toml`）、`.dockerignore`（注释）、`README.md`（注释中的路径大小写）

## 部署影响

- 是否需要部署：否
- 部署原因：仅仓库内构建配置归一；`Dockerfile` 的改动只是跟随文件改名，镜像内容与运行时行为不变，不触发发布
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **合并两份配置语义**为单一 canonical 文件 `frontend-rs/trunk.toml`（**小写**，命名已确认）：
   - `[build]`：`target = "index.html"` / `dist = "dist"`
   - `[tools]`：`wasm-opt = false`
   - `[serve]`：`no_autoreload = true`
   - `[watch]`：`ignore = ["tests"]`（恢复因大小写混淆而失效的 e2e 保护）
2. **消除重复索引条目**：`git rm --cached frontend-rs/Trunk.toml`，使索引中只剩 `frontend-rs/trunk.toml` 一条；物理文件本就名为小写，无需改名动作。
3. **同步引用**（选择小写的连带影响）：
   - `Dockerfile`：`COPY ... frontend-rs/Trunk.toml` → `frontend-rs/trunk.toml`（Linux 大小写敏感，不改则构建失败）
   - `core-01-deployment-plan.md` §4.1：同步该构建片段（经 `deltas/prd/3-technical-plan/3-deployment/` 合并）
   - `.dockerignore` / `README.md`：注释中的路径改为小写
   - `index.html` / `frontend-rs/E4_ACTIVATION.md`：已是小写，无需改动
4. **验收**（本机未安装 `trunk` CLI，以 git 级校验 + 配置等价性为准）：
   - Windows 检出后 `git status` 不再出现 ` M frontend-rs/Trunk.toml`；
   - `git ls-files frontend-rs/` 中 trunk 配置仅剩一条；
   - 合并后的 `trunk.toml` 同时包含上述四段，语义覆盖原两份配置；
   - `Dockerfile` / 部署方案 / `.dockerignore` / `README.md` 对配置文件的引用全部为小写；
   - 若环境具备 `trunk`，可选执行 `trunk build --release` 确认行为不变。
