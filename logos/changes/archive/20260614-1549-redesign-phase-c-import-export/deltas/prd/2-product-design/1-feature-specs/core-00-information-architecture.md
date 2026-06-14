# Delta — core-00-information-architecture.md

## ADDED — §9 V2 增量：IO 抽屉（Phase C）

> merge 时在主文档末尾追加。与 Phase A 归档 delta（`redesign-phase-a-layout`）互补；若主文档 §1 仍为 V1 双顶栏，本节独立描述 V2 IO 区域。

### 9.1 主体栅格（含 IO 抽屉）

```
.cdb-main {
  display: grid;
  grid-template-columns: 48px 1fr auto auto;
  /* ToolRail | Canvas | Inspector? | IoDrawer? */
}
```

| 状态 | grid-template-columns |
|------|------------------------|
| 默认（Inspector 开） | `48px 1fr 320px 0` |
| Inspector 折叠 | `48px 1fr 0 0` |
| IO 抽屉开 | `48px 1fr 0 400px`（Inspector 强制折叠） |
| IO + Inspector 均关 | `48px 1fr 0 0` |

### 9.2 z-index

| 层级 | Phase C 内容 |
|------|--------------|
| L3 | Inspector **或** IoDrawer（互斥，同层） |
| L4 | 模态（New / 冲突 / 删除确认）— IO 抽屉不升级至 L4 |

### 9.3 Phase 边界更新

| 能力 | Phase A | Phase B | Phase C |
|------|---------|---------|---------|
| 导入/导出侧边抽屉 | ❌ | ❌ | ✅ |
| SQL/DBML 全屏视图 | ❌ | ❌ | ❌（Phase D） |
| Command Palette | ❌ | ❌ | ❌（Phase D） |
