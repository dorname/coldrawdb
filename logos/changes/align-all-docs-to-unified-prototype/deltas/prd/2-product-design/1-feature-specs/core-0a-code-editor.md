# Delta — core-0a-code-editor.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线与实现分层

唯一现行交互基线：`core-01-editor-prototype.html` 的代码视图。

| 层 | 要求 |
|---|---|
| 原型 / 规格验收 | 画布壳内全屏 `.code-view`；只读 `<textarea class="code-area">` 演示 SQL / DBML / JSON；复制 + 返回画布 |
| 生产增强（可选） | 可用 Monaco（或等价）替换 textarea，提供语法高亮；**不得改变**入口、格式切换、复制、返回画布与布局互斥语义 |

本规格不再将「必须引入 monaco-editor-wasm」作为文档对齐的前置条件；Monaco 属于生产实现选项，由后续代码变更验收。

## MODIFIED — 1. 概述

Code View 是 room-editor 内的只读代码表面，由 AppBar `data-testid="btn-code-view"`（`toggle-code`）进入/退出。展示由当前 diagram 实时生成的 SQL、DBML 或 JSON；不支持双向粘贴回写画布（仍为 Out of Scope）。

## MODIFIED — 2. CodeView 组件

```text
.code-view[data-testid=code-view-modal]  // position:absolute; inset:0; z-index:5
├─ .code-toolbar
│  ├─ .segmented → SQL | DBML | JSON
│  ├─ .tag.tag--brand「实时生成」
│  ├─ 复制（copy-code）
│  └─ 返回画布（toggle-code）
└─ textarea.code-area[readonly][aria-label=代码内容]
```

- 进入 Code：`workspace` 增加 `is-code`；Inspector 隐藏（opacity/pointer-events）；协作模拟器收起
- AppBar / StatusBar 仍可见；ToolRail 与画布被 code-view 覆盖
- 代码区视觉：深底（原型 `#08171c` / 暗色 `#061217`）+ 等宽 12px/1.7；生产应映射为 token

## ADDED — 统一原型对齐补充：格式与内容

| 格式 | 生成规则（原型语义） |
|---|---|
| SQL | 各表 `CREATE TABLE …` |
| DBML | `Table` 块 + `Ref:` 关系 |
| JSON | `diagram` 对象 pretty-print |

切换：`code-format` + `data-format`；保持只读。

## ADDED — 统一原型对齐补充：复制与返回

| 动作 | 行为 |
|---|---|
| 复制 | 读取当前格式文本；原型以 Toast 模拟成功（可不写真实剪贴板）；生产应 `clipboard.writeText` + Toast「已复制…」 |
| 返回画布 | `codeView=false`，卸载 `.code-view`，恢复 Inspector/模拟器可见性规则 |
| Esc（生产建议） | 与返回画布等价；原型以按钮为主 |

## ADDED — 与 Command Palette 的关系

命令面板可含「打开代码视图」项（原型 ⌘E 提示）；Code View 与 Command（z=55）互斥——打开命令时不要求保持 code 层焦点冲突。Code View **不是** E3 Modal 居中对话框，而是画布区域覆盖层（`code-view-modal` testid 保留历史名）。

## ADDED — 统一原型对齐补充：强制 Monaco / 依赖清单作为规格真值

- `monaco-editor` / `monaco-editor-wasm` Cargo/npm 依赖表不再作为本文件合并后的强制条款
- 「全屏 Modal width=XLarge」布局描述改为画布内 `.code-view` 覆盖
- 复制按钮「右下角绝对定位」改为工具栏「复制」按钮（对齐主原型）

## MODIFIED — 8. 验收约束

- 存在 `btn-code-view` 与 `code-view-modal`
- SQL / DBML / JSON 三段切换可见且内容随 diagram 变化
- 复制触发成功反馈（Toast）
- 返回画布后 code-view 节点移除，可继续编辑画布
- 若生产接入 Monaco：关闭时销毁编辑器实例；主题跟随 `data-mode`（见 `core-0b`）
