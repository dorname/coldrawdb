# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §4.1 镜像构建片段中 trunk 配置文件名归一为小写

## [code] 构建配置归一

- [ ] 合并两份配置语义并写入 `frontend-rs/trunk.toml`（`[build]` + `[tools] wasm-opt = false` + `[serve] no_autoreload = true` + `[watch] ignore = ["tests"]`）
- [ ] `git rm --cached frontend-rs/Trunk.toml` — 从索引移除大写重复条目，使索引只剩 `frontend-rs/trunk.toml`
- [ ] 更新 `Dockerfile`：`COPY frontend-rs/index.html frontend-rs/trunk.toml ./frontend-rs/`
- [ ] 更新 `.dockerignore` 注释与 `README.md` 注释中的路径为小写
- [ ] 验证 `git ls-files frontend-rs/` 中 trunk 配置仅剩一条、`git status` 干净（Windows 无「假修改」）；本机无 `trunk` CLI，构建验证可选
