# drawDB 项目待办事项汇总

> 含 take-todos 提取项与手动维护项。最后更新：2026-03-11

---

## 一、执行计划级待办（Phase 4 / 5 / 收尾）

来自 `docs/phase0/EXECUTION_PLAN.md`：

| # | 状态 | 待办项 | 说明 |
|---|------|--------|------|
| 1 | ⬜ 待做 | Phase 4: Rust Web MVP 可跑主流程 | 技术选型 PoC (Leptos/Yew/Dioxus) + MVP 功能 (建表、加字段、建关联、保存、加载) |
| 2 | ⬜ 待做 | Phase 5: 灰度到 100% 并满足 SLA | 小流量灰度 5%→20%→50%→100%，保存成功率 >= 99.95% |
| 3 | ⬜ 待做 | 旧链路下线 + 回滚预案归档 | 切流复盘与回滚手册 |

---

## 二、代码级 TODO

| # | 文件 | 行号 | 内容 |
|---|------|------|------|
| 1 | `backend/src/diagrams/mod.rs` | L86-91 | **update_diagram 事务不完整**：缺少删除/重建关联关系的逻辑（表关联、引用关联、更新引用） |
| 2 | `src/utils/importSQL/oraclesql.js` | L75 | **Oracle SQL 导入 default 值重建**：当 parser 支持后需要实现 |

### 详情：update_diagram 事务缺失逻辑

```
// backend/src/diagrams/mod.rs L86-91
// TODO：
// 1、删除与表的关联关系
// 2、删除与引用的关联关系
// 3、重新构建与表的关联关系
// 4、重新构建与引用的关联关系
// 5、更新图表
// 6、更新引用
```

---

## 三、已知代码缺陷 / 遗留问题

> 详细 Bug 列表见 [BUGS.md](BUGS.md)。

| # | 类型 | 文件 | 说明 | 建议优先级 |
|---|------|------|------|-----------|
| 1 | **前端后端联调** | `src/components/Workspace.jsx` | 主保存路径仍走 Legacy API / Dexie，Phase 4 时接入 `/api/v1/diagrams` | 中 — Phase 4 时处理 |
| 2 | **Bug** | `backend/src/entity/vo/template_vo.rs`、`backend/src/templates/mod.rs` | `TemplateVo::from` 与调用处类型不匹配（Template Entity vs template::Model），编译 E0308/E0609 | 高 |
| 3 | **Missing** | `backend` 初始化 | `init_templates` 为占位，未将前端 6 个模板 seeds 写入 `template` 表（custom=0 默认模板） | 中 — 需定 seeds 来源后实现幂等插入 |
| 4 | **未使用模块** | `backend/src/notes`、`backend/src/indices` | 已 `mod` 但未挂载路由，且无 crate 内调用；待 Phase 4 或 Legacy 扩展时接入或移除 | 低 |

---

## 四、Phase 4 具体子任务

来自 `RUST_WEB_REFACTOR_PLAN.md` 与 `docs/phase0/EXECUTION_PLAN.md`：

### 4.1 技术选型 PoC（预计 2 周）

- [ ] Leptos / Yew / Dioxus 三选一
- [ ] 评分维度：编辑器交互适配 (40)、性能 (25)、工程可维护性 (20)、团队学习成本 (15)
- [ ] 输出 PoC 评估结论文档

### 4.2 MVP 功能开发（预计 2-3 周）

- [ ] 最小 Rust 页面打通"加载 + 保存"链路
- [ ] 建表
- [ ] 加字段
- [ ] 建关联
- [ ] 保存到后端 API
- [ ] 从后端 API 加载

### 4.3 API 集成与冲突处理（预计 1 周）

- [ ] 接入 `/api/v1/diagrams` CRUD
- [ ] 409 冲突提示用户刷新/覆盖
- [ ] 与后端联调报告

### 4.4 基础体验补齐（预计 1 周）

- [ ] 撤销重做 (undo/redo)
- [ ] 缩放平移
- [ ] 最小工具栏

### 4.5 配套文档

- [ ] 准备 Rust Web MVP PoC 的对接接口清单
- [ ] 制定双写窗口（最多 4 周）与回滚演练计划

---

## 五、Phase 5 子任务（Phase 4 完成后执行）

- [ ] 小流量灰度 5% → 20% → 50% → 100%
- [ ] 观察指标：保存成功率、P95、冲突率、导入失败率
- [ ] 异常触发自动回退策略
- [ ] 完成旧链路下线与文档归档
- [ ] 灰度发布记录
- [ ] 指标看板截图/报表
- [ ] 切流复盘与回滚手册
