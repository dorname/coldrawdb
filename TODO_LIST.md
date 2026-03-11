# drawDB 项目待办事项汇总

> 含 take-todos 提取项与手动维护项。最后更新：2026-03-11

---

## 一、执行计划级待办（可选，其他分支）

> 当前分支仅保留 Legacy 接口。以下为历史/其他分支的规划项，仅供参考。

| # | 状态 | 待办项 | 说明 |
|---|------|--------|------|
| 1 | ⬜ 待做 | Rust Web MVP 可跑主流程 | 技术选型 PoC + MVP 功能 (建表、加字段、建关联、保存、加载) |
| 2 | ⬜ 待做 | 灰度与旧链路下线 | 灰度发布、回滚预案归档 |

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
| 1 | **前端后端联调** | `src/components/Workspace.jsx` | 主保存路径走 Legacy API / Dexie | 中 |
| 2 | **Bug** | `backend/src/entity/vo/template_vo.rs`、`backend/src/templates/mod.rs` | `TemplateVo::from` 与调用处类型不匹配（Template Entity vs template::Model），编译 E0308/E0609 | 高 |
| 3 | **Missing** | `backend` 初始化 | `init_templates` 为占位，未将前端 6 个模板 seeds 写入 `template` 表（custom=0 默认模板） | 中 — 需定 seeds 来源后实现幂等插入 |
| 4 | **已移除** | `backend/src/notes`、`backend/src/indices` | 已从本分支移除（未挂载路由、无调用） | — |

---

## 四、其他分支 / 历史规划（仅供参考）

Phase 4/5 相关子任务（Rust Web MVP、灰度、切流等）为其他分支规划，当前分支不保留详细文档。上述项适用于引入新 API 或 Rust Web 时的其他分支。
