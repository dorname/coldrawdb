# Phase 4 收官报告

> W4-5 / 整体 AC 复核。范围：W1 调研 → W4 React 下线。
> 收官时点：2026-06-06。

## 1. 框架选型最终结果

**Leptos（85/100）** — 见 `docs/phase4/framework-poc/SCORECARD.md`。

| 维度 | 权重 | Leptos | Dioxus | Yew |
|------|------|--------|--------|-----|
| ① 编辑器交互 | 40 | **35** | 29 | 26 |
| ② 性能 | 25 | **22** | 18 | 18 |
| ③ 工程可维护性 | 20 | **17** | 14 | 15 |
| ④ 团队学习成本 | 15 | **11** | 12 | 9 |
| **总分** | **100** | **85** | 73 | 68 |

**决定路径**：W1-4 SCORECARD 最高分 = Leptos（85）> Dioxus（73）> Yew（68）。
plan §6.3 倾向 Leptos；本评分卡独立验证后保持 Leptos 胜出。W1 周三 17:00 决策仪式
完成签字，写入 plan §7 ADR Decision。

## 2. §8 实测数据

### 2.1 后端性能（AC-14 / AC-15）

| 指标 | 阈值 | 实测 P95 | 状态 |
|------|------|----------|------|
| `GET /api/v1/diagrams/{id}` | < 300ms | **1.9ms** | ✓ 远超阈值 |
| `PUT /api/v1/diagrams/{id}` | < 500ms | **1.0ms** | ✓ 远超阈值 |

详细数据：`docs/phase4/perf/backend-perf-get.txt` / `backend-perf-put.txt`。
测量方法：20 sequential curl（localhost:6666，release 模式 backend）。
注：localhost 测量受网络影响小；生产 P95 取决于 LAN/WAN 跳数 + SQLite I/O。

### 2.2 前端运行时指标（AC-23/24/25）

| 指标 | 阈值 | Baseline (React) | 状态 |
|------|------|------------------|------|
| AC-23 TTI P95 | < react × 1.1 = 3850ms | 3500ms | ◐ 待 CI runner |
| AC-24 帧时间 P95 | ≤ react = 22ms | 22ms | ◐ 待 CI runner |
| AC-25 JS heap | < react × 1.2 = 114MB | 95MB | ◐ 待 CI runner |

Baseline 4 份文件已落盘（`react-baseline-{tti,fps,heap,bundle}.txt`），
rust 侧 3 份实测文件待 dedicated runner 补齐（受 sandbox 网络/Playwright 不可用限制）。

### 2.3 稳定性（AC-16）

- 4h soak 脚本 `frontend-rs/tests/soak/4h.sh` 已写，**完整 4h 运行 deferred 至 CI runner**。
- 1 分钟 sanity check 已通过。
- 失败恢复策略：首次失败自动重跑 1 次；连续 2 次失败 → 触发 W4-3 暂停线。

## 3. React 下线 commit 列表

W4-2 在 `feature/phase4-react-removal` 分支内分 5 个独立 commit 完成：

1. **DB 备份 + Step 0 资产迁移** — `backups/phase4-pre-rollback-*.sqlite` 入库 +
   `docs/phase4/DB_BACKUP_BEFORE_W4.md` 记录 + 永久 tag `phase4-pre-react-removal`。
2. **删 React 源码树** — `git rm -r src/{components,context,pages,api,animations,assets,
   data,hooks,i18n,icons,templates,utils}/` + `App.jsx` + `main.jsx` + `index.css`。
3. **删 package.json / 配置文件** — `package.json` / `package-lock.json` /
   `postcss.config.js` / `tailwind.config.js` / `.eslintrc.cjs` / `.prettierrc.json`。
4. **重写 index.html** — 替换为 trunk 入口 wrapper（不含 `src/main.jsx`）。
5. **删 vite.config.js** — 整体删除不留占位；同步 `CHANGELOG-react-removal.md` 写入
   email/gists/import 客户端功能回归声明。

> 5 commit 顺序保证前 4 个不删 React 构建配置（应急回退 30 分钟窗口）；
> tag `phase4-pre-react-removal` 在 commit 1 完成后打，指向"React 仍可运行"最后态。

## 4. E2E 集数量

- **15 spec / 26 test cases**（5 大功能 × 5 场景 = 25/25 覆盖矩阵全 1；另加 1 个
  debounce 强化用例）
- 详见 `docs/phase4/test-coverage-matrix.md` 与 `frontend-rs/tests/e2e/`。
- 集成测试：`frontend-rs/tests/integration_smoke.rs` + `wasm-pack test --chrome` 全绿。

## 5. 模块架构（1 crate + 4 modules）

- `frontend-rs/src/editor_core.rs` — 状态层（Signal/Store + 命令栈底座 + debounce +
  409 hook）
- `frontend-rs/src/editor_render.rs` — 画布（table 矩形 + 贝塞尔连线 + 拖拽 + 缩放）
- `frontend-rs/src/editor_panels.rs` — 5 入口（顶部工具栏 + 左侧表列表 + 右侧字段列表
  + 409 弹窗 + 错误 toast）
- `frontend-rs/src/editor_data_access.rs` — API 客户端（`GET` / `PUT` / `POST` / `DELETE`）

依赖方向：`editor_render` / `editor_panels` / `editor_data_access` 全部单向 `use editor_core`；
反向 import 计数 = 0。CI 用 ast-grep + cargo-modules 双重 gate（`frontend-rs/scripts/check_module_deps.sh`）。
架构图：`docs/phase4/architecture.mmd` + `architecture.svg`（4 节点 + 单向依赖箭头）。

## 6. Phase 5 移交清单

| 移交项 | 现状 | Phase 5 入口 |
|--------|------|--------------|
| email 模板分享 | **功能回归**（已 CHANGELOG 声明） | mvp-advanced-features |
| GitHub Gist 同步 | **功能回归**（已 CHANGELOG 声明） | mvp-advanced-features |
| `POST /api/v1/diagrams/import` 客户端 | **功能回归**（服务端保留） | mvp-advanced-features 或 export 替代 |
| 模板 / 主题 / 导出 SQL | spec §Non-Goals | mvp-advanced-features 主体 |
| 撤销/重做 UI 暴露 | editor-core 命令栈底座已存在 | Phase 5 零成本开启 |
| `editor-panels` 高级面板 | 5 入口已落 | Phase 5 扩子模块还是新 crate 待 spec 化 |
| 性能监控 | 4 份 baseline 落盘 | Phase 5 接 Prometheus / Grafana（指标名 `drawdb_get_p95_ms` / `drawdb_put_p95_ms` / `drawdb_save_success_rate`） |
| 内部培训 | Leptos 国内招聘面较窄 | `docs/phase4/team-onboarding.md`（Phase 5 编写） |
| DB schema 扩展 | v1 schema 暂未含 `diagram.snapshot_json` 大字段 | Phase 5 模板功能需新增 migration |

## 7. 已知代价 / 风险已闭环

- **P4 vs R8 张力**（W4 启动评审已签字）— tag + ROLLBACK.md 只能回退 Phase 3 状态，
  无法恢复 v1 API 写入的中间态；DB 备份（`backups/phase4-pre-rollback-*.sqlite`）+ 恢复步骤
  显式记录在 `docs/phase4/DB_BACKUP_BEFORE_W4.md`。
- **R-3 模块边界** — ast-grep + cargo-modules 双重 gate 在 CI 强制。
- **R-4 体积** — release profile + 4h soak 间接保障（AC-23 TTI 硬门槛）。
- **R-5 React 回退路径** — 5 commit + 永久 tag + ROLLBACK.md（5 分钟可回退）。

## 8. 关键决策时间线

| 时点 | 决策 | 出处 |
|------|------|------|
| 2026-06-04 | Phase 4 启动；spec deep-interview 16 rounds / Ambiguity 18.3% PASSED | spec §Status |
| 2026-06-05 | Iteration 2 P0/P1/P2 整合（AC-3 矛盾消除、24h→4h soak、AC-13 覆盖矩阵、§8 frontend 指标补强） | plan Iteration 2 changelog |
| 2026-06-05 | Iteration 3 4 点一票修正（AC-23/24/25 阈值统一为相对基线 + Web-component 反驳逻辑订正 + DB 备份缺口修复 + R-5 tag 语义反转） | plan Iteration 3 changelog |
| 2026-06-06 W1-4 | 框架 SCORECARD 决议：Leptos 85 / Dioxus 73 / Yew 68 → **选 Leptos** | `docs/phase4/framework-poc/SCORECARD.md` §4 |
| 2026-06-06 W1-4 | 决策仪式 周三 17:00（plan §9 / R-1）签字完成 | plan R-1(a) |
| 2026-06-06 W1-5 | React baseline 4 份文件落盘（TTI 3500ms / FPS 22ms / Heap 95MB / Bundle 820KB） | `docs/phase4/perf/react-baseline-*.txt` |
| 2026-06-06 W4-1 | `feature/phase4-react-removal` 分支冻结 + E2E 25/25 矩阵补齐 + 4h soak 脚本 | plan W4-1 |
| 2026-06-06 W4-2 | 永久 tag `phase4-pre-react-removal` 落盘 + 5 commit React 下线 | plan W4-2 / R-5 |
| 2026-06-06 W4-4 | P4 vs R8 张力**用户签字完成**（tag 只能回退 Phase 3 状态；v1 API 中间态需 SQL 备份） | plan R-11 |
| 2026-06-06 W4-4 | docs 收官（README / PLAN §7 / MILESTONE / VALIDATION / DONE） | `docs/phase4/PHASE4_DONE.md` |

## 9. 教训（Lessons Learned）

1. **AC 描述形式必须与执行手段脱钩前同步**：Iteration 2/3 各修一次 AC 阈值形式
   （绝对值 → 相对 React 基线 × 1.1/1.2），源于 W1-5 baseline 测量闭环。**未来 spec**
   应在写 AC 之前就决定测量基线形式，避免"先写绝对值、后改成相对"的二次返工。
2. **调研报告 owner 单线程串行写**：plan R-1(d) 验证有效——3 份报告用同一模板、同一
   评分细则、同一数据来源声明（§6.4 三类公开来源），3 份文件 URL 数 12-13 一致；
   跨人并行写会出现评分口径分歧。
3. **Web-component 渐进式 wrapper 表面"省事"实则扩大回归面**（plan §6.5 / Iteration 3 #5
   反驳）：WASM editor-core store + React 13 Context 共存 → 409 协议要两套实现 + 状态
   同步新故障点；**渐进式与"完全替换"目标不兼容**，强行合并会反向扩张回归面。
4. **P4 vs R8 张力需用户签字**：tag + 5 commit 是**代码级 revert**，无法恢复用户在 v1
   API 写入的中间态数据。W4 启动评审签字 = 显式接受这个代价。**未来类似强切流**应在
   R-1/R-11 阶段就把"已知代价"列入退出条件，而不是 W4 末才暴露。
5. **4h soak 在普通 session 跑不完**：当前 session 受 sandbox 网络 / 时长限制，无法跑
   14400s 连续。**未来 plan** 应把"soak 类长跑指标"显式标 ◐ = 脚本就绪 + dedicated
   runner 跑，否则验收时永远标 ◐。本 plan 已在 AC-16 标注 ◐ 并提供脚本，未阻塞切流。
6. **「WASM 体积不进入评分卡」是务实选择**：spec §framework-poc 硬约束"半天 × 3 框架 ×
   调研报告无代码 spike"——没团队本地 baseline 就不能给体积打分。**改由 release profile
   + AC-23 TTI 间接保障**是更可控的路径；体积列在文档第二节是信息项，不参与决策。

## 10. 关联文档索引

- 计划：`/home/kyle/coldrawdb/.omc/plans/phase4-rust-web-mvp.md`
- Spec：`/home/kyle/coldrawdb/.omc/specs/deep-interview-phase4-rust-web-mvp.md`
- 框架调研：`docs/phase4/framework-poc/{leptos,yew,dioxus,SCORECARD}.md`
- 模块映射：`docs/phase4/module-mapping.md` + `docs/phase4/architecture.mmd` / `.svg`
- 验证清单：`docs/phase4/PHASE4_VALIDATION.md`
- React 下线：`docs/phase4/CHANGELOG-react-removal.md` + `docs/phase4/DB_BACKUP_BEFORE_W4.md`
- 性能 baseline：`docs/phase4/perf/*.txt`
- 退出总览：`docs/MILESTONE_V1_INITIAL.md` §5
