# 验收报告

> 变更提案：`fix-trunk-toml-case-conflict`
> 生成时间：2026-09-20 | 验收基线 commit：`6abdbe6`（`fix(fix-trunk-toml-case-conflict): implement changes`）
> 生成方式：本机未安装 `openlogos` CLI，按 `logos/spec/test-results.md` + `logos.config.json` 的 verify 口径等价执行。

## 一、汇总

| 指标 | 数值 |
|------|------|
| 已定义用例 | 441 |
| 人工用例（不计入分母） | 0 |
| 已执行用例 | 441 |
| 通过 | 405 |
| 失败 | 0 |
| 跳过 | 36 |
| 未覆盖 | 0 |
| 覆盖率 | 100% |
| 通过率 | 100% |
| **Gate 3.5** | **PASS** |

统计口径：`test-results.jsonl` 中同一 `id` 取最后一次结果（last-write-wins），得唯一用例 441 条。

## 二、跳过的用例（36）

ST-CR-01、ST-FE-ALIGN-03、ST-FE-ALIGN-04、ST-FE-PROTO-01、ST-FE-PROTO-02、ST-FE-PROTO-03、ST-FE-PROTO-04、ST-FE-PROTO-05、ST-FE-PROTO-06、ST-FE-PROTO-07、ST-FE-PROTO-08、ST-MM-01、ST-MM-02、ST-MM-03、ST-PU-24、ST-S01-03、ST-S01-409-SCOPE、ST-S01-NO-409-OT、ST-S02-01、ST-S02-02、ST-S02-03、ST-S02-04、ST-S02-05、ST-S02-06、ST-S04-UI-08、ST-S05-UI-01、ST-S05-UI-02、ST-S05-UI-04、ST-S05-UI-06、ST-S07-03、ST-SP-01、ST-UI-05、UT-PC-13、UT-PC-17、UT-PC-21、UT-PC-24

## 三、本次变更专项验收

本提案为构建配置类修复，不新增业务行为，因此未新增 UT/ST 用例（与 `proposal.md`「影响的编排测试：无」一致），专项验收以可复现的断言实测为准。

### 3.1 断言与实测结果

| # | 断言 | 实测结果 |
|---|------|----------|
| 1 | `frontend-rs/trunk.toml` 是合法 TOML | `tomllib.load` 解析成功 |
| 2 | 归一后配置语义完整覆盖原两份 | `SEMANTIC_EQ=True`；`NEW={"build":{"target":"index.html","dist":"dist"},"tools":{"wasm-opt":false},"serve":{"no_autoreload":true},"watch":{"ignore":["tests"]}}`，且 `OLD_UP={"watch":{"ignore":["tests"]}}`、`OLD_LO={"build":{...},"tools":{...},"serve":{...}}` 的每个键值均被包含 |
| 3 | 索引中 trunk 配置仅剩一条 | `git ls-files frontend-rs/` → 仅 `frontend-rs/trunk.toml` |
| 4 | 大小写不敏感平台不再出现「假修改」 | `git status --porcelain` 无 ` M frontend-rs/Trunk.toml`，仅剩未跟踪的 `COLLAB_ONBOARDING.md`（与本变更无关，未纳入） |
| 5 | 代码区无残留大写文件名引用 | `git grep -I "Trunk\.toml" -- ":(exclude)logos"` 唯一命中为新 `trunk.toml` 第 3 行的历史说明注释 |
| 6 | 四处引用均已同步为小写 | `Dockerfile:11`、`.dockerignore:2`、`README.md:107`、`core-01-deployment-plan.md:232` |
| 7 | 全局测试账本自洽、无未登记 ID | `node scripts/validate-openlogos-ledger.mjs` → `{"defined":441,"manual":0,"automated":441,"executed":441,"status":"PASS"}`，EXIT=0 |
| 8 | 无失败用例 | `FAILED=(none)` |

### 3.2 复现命令

```bash
# 断言 1 + 2
python -c "import tomllib,subprocess,json;cur=tomllib.load(open('frontend-rs/trunk.toml','rb'));g=lambda p:tomllib.loads(subprocess.run(['git','show',p],capture_output=True,encoding='utf-8').stdout);a=g('HEAD~1:frontend-rs/Trunk.toml');b=g('HEAD~1:frontend-rs/trunk.toml');print('SEMANTIC_EQ=',all(cur.get(k,{}).get(kk)==vv for src in (a,b) for k,sec in src.items() for kk,vv in sec.items()))"

# 断言 3 + 4 + 5
git ls-files frontend-rs/ | Select-String "runk"
git status --porcelain
git grep -n -I "Trunk\.toml" -- ":(exclude)logos"

# 断言 7
node scripts/validate-openlogos-ledger.mjs
```

## 四、环境限制声明（重要）

1. **官方预跑未执行**：`logos.config.json` 配置 `verify.pre_run_command = "bash scripts/run-verify-tests.sh"`，该脚本需依次运行 `backend` / `frontend-rs` / `mcp-server` 的 `cargo test`，多批 Playwright 回归，最后执行账本校验。本机实测无法运行：
   - `bash.exe` 实为 WSL 启动器，`/bin/bash` 不存在（`WSL ... execvpe(/bin/bash) failed: No such file or directory`）；
   - `frontend-rs/node_modules` 不存在，Playwright 依赖未安装。
2. **本报告复用的账本未变更**：`logos/resources/verify/test-results.jsonl`（47,572 字节，mtime `2026/9/18 17:08:06`）自上次验收后未被改写，仍是最近一次完整预跑的产物；本次仅对其执行读取与校验，未做任何写入。
3. **`trunk` CLI 未安装**，`trunk build --release` 未实测；本变更不修改任何 Rust/TS 源码，不改变任何被现有 441 个用例覆盖的运行时行为。
4. 上述限制不影响 Gate 3.5 的判定口径（覆盖率与失败数），但**若需以「本变更后重新全量预跑」作为验收证据，请在具备 bash + cargo + Playwright 的 Linux 环境重跑 `bash scripts/run-verify-tests.sh` 后再执行 `openlogos verify`**。
