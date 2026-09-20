# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §4.1 镜像构建片段中 trunk 配置文件名归一为小写

## [code] 构建配置归一

- [x] 合并两份配置语义并写入 `frontend-rs/trunk.toml`（`[build]` + `[tools] wasm-opt = false` + `[serve] no_autoreload = true` + `[watch] ignore = ["tests"]`）
- [x] `git rm --cached frontend-rs/Trunk.toml` — 从索引移除大写重复条目，使索引只剩 `frontend-rs/trunk.toml`
- [x] 更新 `Dockerfile`：`COPY frontend-rs/index.html frontend-rs/trunk.toml ./frontend-rs/`
- [x] 更新 `.dockerignore` 注释与 `README.md` 注释中的路径为小写
- [x] 验证 `git ls-files frontend-rs/` 中 trunk 配置仅剩一条、`git status` 干净（Windows 无「假修改」）；本机无 `trunk` CLI，构建验证可选

### 验证记录

- `git ls-files frontend-rs/` → 仅 `frontend-rs/trunk.toml` 一条（大写条目已消失）
- `git status --porcelain` → ` M frontend-rs/trunk.toml` + `D  frontend-rs/Trunk.toml`（重复条目消除后应有的形态）
- `git grep -I "Trunk\.toml" -- ":(exclude)logos"` → 仅命中新 `trunk.toml` 自身的历史注释，无功能性残留
- `frontend-rs/trunk.toml`：1207 字节，blob `95e396f2`，四段 `[build]` / `[tools]` / `[serve]` / `[watch]` 齐备
- 注：本机未安装 `trunk` CLI，未执行 `trunk build --release` 实构建；配置语义与原两份完全对齐
