# Delta: remove-legacy-docs — core-01-requirements.md

> 目标主文档：`logos/resources/prd/1-product-requirements/core-01-requirements.md`
> 提案：remove-legacy-docs | 日期：2026-09-11

## MODIFIED — 6. 验收条件

## 6. 验收条件

V1 通过条件：
- [ ] Phase 4 CI 全绿（W4 perf 已记录）
- [x] drawdb 主分支能力对齐（已于 V1 收官时核验；原能力清单 `docs/drawdb-capability-checklist.md` 已随提案 remove-legacy-docs 移除，内容可溯 git 历史）
- [ ] 11 张表可读写无错
- [ ] 7 引擎 SQL 导入导出可演示
- [ ] 409 revision 冲突可演示

### 6.1 V2 工作空间验收条件（规格级）

在既有 V1 验收条件之外，V2 规格收口通过条件：

- [ ] 需求/设计/技术场景对 `auth → rooms → room-editor` 叙述一致
- [ ] S03～S05 状态不再写成「❌ V2 / 范围外」或「仅后端、前端完全未接入」的过时表述
- [ ] 主原型关键 `data-testid` 已映射到测试/验收文档
- [ ] 明确区分：已有部分生产接入 vs 下一变更 `implement-unified-prototype-spec-parity` 待实现的逐项对齐
- [ ] 演示器行为不写入 API/DB 强制契约，除非场景时序已推导

### 6.2 S07 数据字典验收条件（规格级）

- [ ] 字典 CRUD + 字典项编辑在侧栏字典 Tab 可演示
- [ ] 字段绑定字典后，字典 Tab 可见引用计数；删除被引用字典出现确认
- [ ] 字典随自动保存持久化，刷新/重开后完整恢复；无字典的旧图加载不报错
- [ ] Markdown 导出内容与当前模型一致（字典项序、绑定关系）
- [ ] Viewer 只读：字典编辑与绑定控件禁用，导出可用
