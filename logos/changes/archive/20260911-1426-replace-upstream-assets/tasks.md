# 实现任务

## [code] 仓库资产与 CI 适配（非 `logos/resources/` 主文档，无 delta）
- [x] 适配 `.github/workflows/build.yml`：分支触发 `main`；修正后端测试步骤过时注释；actions 升级 v4
- [x] 适配 `.github/workflows/docker.yml`：分支触发 `drawdb-web` → `main`
- [x] 删除 `.github/FUNDING.yml`（上游作者赞助配置）
- [x] 新增 `frontend-rs/scripts/capture-readme-screenshot.mjs`（真实后端 + 真实登录 + 建房 + Playwright 编辑器截图）
- [x] 截取本应用编辑器界面（暗色）至 `docs/assets/app-editor.png`，README 头图改用新截图
- [x] 删除上游截图资产 `drawdb.png` 与 `public/`（含 hero_ss.png）
