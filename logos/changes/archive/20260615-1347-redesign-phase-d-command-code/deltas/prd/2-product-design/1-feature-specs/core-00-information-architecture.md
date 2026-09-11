# Delta — core-00-information-architecture.md

## ADDED — §9 V2 增量：Command Palette 与代码视图（Phase D）

> merge 时在主文档末尾（§8 IO 抽屉之后）追加。

### 9.1 视图模式（完整）

| 模式 | 入口 | 布局变化 |
|------|------|----------|
| **Canvas**（默认） | `/editor/{id}` | Phase A/C 布局（Tool Rail + Canvas + Inspector/IO） |
| **Code** | `btn-code-view` | 隐藏 Tool Rail / Canvas / Inspector / IO；全屏代码区 |
| **Command Palette**（叠加） | `Ctrl+K` | 居中浮层，不改变底层 view_mode |

### 9.2 z-index 更新

| 层级 | Phase D 内容 |
|------|--------------|
| L3 | Inspector **或** IoDrawer（互斥） |
| L4 | 阻塞模态（New / 冲突） |
| L4.5 | Command Palette |
| L5 | Toast |

### 9.3 Phase 边界更新

| 能力 | Phase C | Phase D |
|------|---------|---------|
| Command Palette | ❌ | ✅ |
| SQL/DBML 全屏视图 | ❌ | ✅（只读） |
| 左栏 7 Tab UI | ❌ | ❌（永久移除，Palette 替代） |
| 代码视图双向编辑 | ❌ | ❌（后续） |

## MODIFIED — §8.3 Phase 边界表

> 将 `SQL/DBML 全屏视图 | ❌（Phase D）` 更新为 `✅（Phase D 只读）`；新增 `Command Palette | ✅`。
