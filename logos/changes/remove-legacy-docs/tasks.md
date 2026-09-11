# 实现任务

## [delta] 规格变更
- [ ] `core-01-requirements.md` §6：drawdb 能力清单验收条目历史化（[x] + 墓碑注记）
- [ ] `core-02-product-vision.md` §1.2 G1 行 + §1.4 launch gate 条目历史化
- [ ] `core-00-information-architecture.md` §7：移除 4 行 docs/ 引用，追加墓碑注记
- [ ] `core-01-editor-canvas.md` §10 / `core-01c-index-enum-custom-type.md` §8 / `core-02-diagram-persistence.md` §10 / `core-03-bridge-io.md` §12 / `core-04-side-panel-tabs.md` §13 / `core-05-top-menu-modals.md` §9：各移除 1 行 docs/ 引用，追加墓碑注记
- [ ] `core-01-deployment-plan.md` §11：移除 2 行 docs/ 引用，追加墓碑注记
- [ ] `core-implementation-checklist.md` §12：移除 2 行 docs/ 引用，追加墓碑注记

## [code] 代码实现
- [ ] 迁移 `docs/assets/app-editor.png` → 仓库根 `app-editor.png`；随后删除 `docs/` 整目录
- [ ] `README.md`：头图 `src` 改为根目录 `app-editor.png`；架构行移除 `architecture.mmd` 引用；文档索引表删除 docs 两行
- [ ] `RUST_WEB_REFACTOR_PLAN.md`：顶部追加「历史重构文档，docs/ 产物已移除、见 git 历史」声明
- [ ] `.github/workflows/build.yml`：删除 mmdc 渲染步骤（architecture.svg）；perf `writeFileSync` 改写 `perf-results/perf-w3-actual.txt`；基线注释同步更新
- [ ] `frontend-rs/tests/soak/4h.sh`：`RESULTS` 默认值改为 `perf-results/soak-4h.txt`（自动建目录，移除绝对路径硬编码）
- [ ] `frontend-rs/scripts/capture-readme-screenshot.mjs`：`OUT_DIR` 改为仓库根，头图产物注释同步
- [ ] 全仓 grep 复核：除 git 历史与 `logos/changes/archive/` 归档外无残留 docs/ 引用
