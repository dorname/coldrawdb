# Delta — core-00-information-architecture.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`

## MODIFIED — §1 顶层布局 V2（E1 增量）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E1）
> 对齐 `core-07-design-tokens.md` §13 z-index 体系

**merge 时替换** `core-00-information-architecture.md` 当前 §1 顶层布局段，更新为：

### §1 顶层布局（V2 — E1 z-index 扩展）

```
+---------------------------------------------------------------+
| AppBar（48px, --cdb-z-app-bar=20）                            |
|  项目名 / 保存状态 / 导入 / 导出 / 撤销 / 重做 / 分享 / 主题  |
+--------+-------------------------------------+----------------+
|        |                                     |                |
| Tool   |                                     |                |
| Rail   |        EditorCanvas                  |  Inspector     |
| 48px   |   （Table / Area / Note /            |  (L3, --cdb-z- |
| (--cdb |    Relationship / Canvas）           |   inspector=30)│
| -z-    |                                     |                |
| side-  |                                     |                |
| rail=  |                                     |                |
| 25)    |                                     |                |
|        |                                     |                |
+--------+-------------------------------------+----------------+
| StatusBar（28px, --cdb-z-app-bar）                            |
+---------------------------------------------------------------+
```

**z-index 层级（来自 `core-07-design-tokens.md` §13）**：

| 层级 | Token | 值 | 内容 |
|---|---|---|---|
| L0 | `--cdb-z-base` | 1 | 默认 |
| L1 | `--cdb-z-canvas-overlay` | 10 | 画布选中框、连线 hover |
| L2 | `--cdb-z-app-bar` | 20 | AppBar、StatusBar |
| L2.5 | `--cdb-z-side-rail` | 25 | Tool Rail 悬浮按钮 |
| L3 | `--cdb-z-inspector` / `--cdb-z-drawer` | 30 | Inspector 抽屉 / IO 抽屉（互斥） |
| L4 | `--cdb-z-tooltip` | 40 | Tooltip |
| L4.5 | `--cdb-z-popover` | 45 | Popover、Dropdown |
| L5 | `--cdb-z-modal` | 50 | Modal、Command Palette（E4） |
| L6 | `--cdb-z-notification` | 60 | Toast |

**E1 增量**：新增 L4–L6 三层（Tooltip / Popover / Modal / Notification），原 V1 §1 无浮层与模态的层级说明。

## MODIFIED — §3 4 模块前端（E1 增量：暗色模式 token 接口预留）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E1）
> 对齐 `core-07-design-tokens.md` §14 / §15 主题切换接口

**merge 时替换** `core-00-information-architecture.md` 当前 §3 "4 模块前端（Phase 4 架构）" 段，更新为：

### §3 4 模块前端（Phase 4 架构，E1 增量主题接口）

```
┌─────────────────────────────────────────┐
│           frontend-rs crate             │
│  (Leptos 0.x + WASM + trunk)            │
├─────────────────────────────────────────┤
│  lib.rs                                 │
│  ├── mount_to_body                      │
│  ├── <html data-mode="light|dark"> 主题  │
│  └── 监听 prefers-color-scheme          │
├─────────────────────────────────────────┤
│  editor_data_access（无依赖）           │
│  └── HTTP 客户端（diagrams/bridge）     │
│        + debounce 1s 自动保存          │
├─────────────────────────────────────────┤
│  editor_core（依赖 data_access）         │
│  └── 状态机（diagram / undo / redo）     │
├─────────────────────────────────────────┤
│  editor_panels（依赖 core）              │
│  └── 侧栏（Tables / Areas / Notes ...）  │
├─────────────────────────────────────────┤
│  editor_render（依赖 core）              │
│  └── Canvas 渲染（Table/Field/连线）    │
├─────────────────────────────────────────┤
│  styles.css（E1：~100 token）            │
│  ├── :root { --cdb-* }                  │
│  ├── [data-mode="dark"] { --cdb-* 覆盖 }│ ← E5 填充具体值
│  └── prefers-color-scheme: dark 媒体查询│ ← E5 接入
└─────────────────────────────────────────┘
```

**主题切换接口**（E1 定义接口，E5 填充实现）：

| 接口 | 类型 | 来源 | 实现阶段 |
|---|---|---|---|
| `<html data-mode="light\|dark">` | DOM 属性 | `core-07-design-tokens.md` §15 | E1 预留 |
| `prefers-color-scheme: dark` | 媒体查询 | 同上 | E5 接入 |
| `localStorage["cdb-mode"]` | 持久化 | 同上 | E5 实现 |

E1 阶段 `lib.rs` 不实现主题切换逻辑，**仅在 mount 时设置 `<html data-mode="light">` 作为初始值**。E5 阶段补全 JS 切换 + 持久化 + 媒体查询监听。
