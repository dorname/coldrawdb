# Code Editor 规格（E4 — Monaco 集成）

## 0. 事实基线与实现分层

唯一现行交互基线：`core-01-editor-prototype.html` 的代码视图。

| 层 | 要求 |
|---|---|
| 原型 / 规格验收 | 画布壳内全屏 `.code-view`；只读 `<textarea class="code-area">` 演示 SQL / DBML / JSON；复制 + 返回画布 |
| 生产增强（可选） | 可用 Monaco（或等价）替换 textarea，提供语法高亮；**不得改变**入口、格式切换、复制、返回画布与布局互斥语义 |

本规格不再将「必须引入 monaco-editor-wasm」作为文档对齐的前置条件；Monaco 属于生产实现选项，由后续代码变更验收。

## 1. 概述

Code View 是 room-editor 内的只读代码表面，由 AppBar `data-testid="btn-code-view"`（`toggle-code`）进入/退出。展示由当前 diagram 实时生成的 SQL、DBML 或 JSON；不支持双向粘贴回写画布（仍为 Out of Scope）。

## 2. CodeView 组件

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

## 3. Monaco 集成

### 3.1 依赖

| 包 | 版本 | 来源 |
|---|---|---|
| `monaco-editor` | `^0.45.0` | npm（前端依赖） |
| `monaco-editor-wasm` | `^0.45.0` | npm |
| `wasm-bindgen` | `^0.2` | Rust → JS 绑定 |

**Cargo.toml 增量**：
```toml
[dependencies]
monaco-editor-wasm = "0.45"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
```

**package.json 增量**：
```json
{
  "dependencies": {
    "monaco-editor": "^0.45.0",
    "monaco-editor-wasm": "^0.45.0"
  }
}
```

### 3.2 挂载流程

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = monaco)]
    pub type Monaco;
    #[wasm_bindgen(js_namespace = monaco)]
    pub fn editor() -> Monaco;
    #[wasm_bindgen(js_namespace = monaco, js_name = editor)]
    pub type Editor;
    #[wasm_bindgen(method)]
    pub fn create(this: &Monaco, container: &HtmlElement, options: &JsValue) -> Editor;
    // ...
}

#[component]
pub fn CodeView(/* ... */) -> impl IntoView {
    let container_ref = NodeRef::<HtmlElement>::new();
    
    create_effect(move |_| {
        if visible.get() {
            // lazy import：首次进入时按需加载 ~30MB Monaco bundle
            let _ = import("monaco-editor/esm/vs/editor/editor.api").then(|_| {
                // 调用 monaco.editor.create(container, options)
            });
        }
    });
    
    view! {
        <Modal visible=visible width=ModalWidth::XLarge>
            <div node_ref=container_ref class="cdb-monaco-container" />
        </Modal>
    }
}
```

### 3.3 DBML Setup（对齐 main `setUpDBML.js`）

```rust
fn setup_dbml(monaco: &Monaco, database: &Database) {
    // 注册 DBML 语言
    monaco.languages.register(&"dbml".into());
    
    // 配置语法高亮（来自 main setUpDBML.js 简化版）
    monaco.languages.set_monarch_tokens_provider(&"dbml".into(), &DBML_TOKEN_PROVIDER);
    
    // 主题：浅色 vs 暗色（E5 切换）
    let theme = match settings.mode {
        Light => "vs",
        Dark => "vs-dark",
    };
    monaco.editor.set_theme(&theme.into());
}
```

### 3.4 复制按钮（对齐 main `CodeEditor`）

```rust
view! {
    <div class="cdb-monaco-container" node_ref=container_ref>
        {show_copy.then(|| view! {
            <Button
                class="cdb-code-view__copy"
                variant=ButtonVariant::Secondary
                on_click=move |_| {
                    let value = monaco.get_value();
                    navigator.clipboard().write_text(&value);
                    toast.success("已复制到剪贴板");
                }
            >
                <IconCopy />
            </Button>
        })}
    </div>
}
```

**视觉**（对齐 main `CodeEditor` styles）：
```css
.cdb-code-view__copy {
  position: absolute;
  right: var(--cdb-space-6);  /* 24px */
  bottom: var(--cdb-space-2); /* 8px */
  z-index: var(--cdb-z-modal); /* 50，悬浮在 Monaco 上 */
}
```

## 4. 复制行为

| 步骤 | 行为 |
|---|---|
| 1 | 用户点击 `<Button icon=IconCopy>` |
| 2 | 调用 `navigator.clipboard.writeText(monaco.get_value())` |
| 3 | 成功 → toast 提示 "已复制到剪贴板"（右下角 L6 `--cdb-z-notification`） |
| 4 | 失败 → toast 错误 + console.error |

## 5. 关闭行为

| 触发 | 行为 |
|---|---|
| ESC 键 | Modal 关闭（E3 Modal `esc_closable`） |
| 点击遮罩 | Modal 关闭（`mask_closable`） |
| 右上角 × | Modal 关闭 |
| View → Code View 菜单 | 切换 `ViewMode::Canvas` |

## 6. ViewMode 互斥

```rust
pub enum ViewMode { Canvas, Code }
let view_mode: RwSignal<ViewMode> = create_signal(ViewMode::Canvas);

// 进入 Code 模式
let open_code_view = move |_| {
    view_mode.set(ViewMode::Code);
    // 隐藏 Tool Rail / Inspector / IO 抽屉
};

// 返回 Canvas 模式
let close_code_view = move |_| {
    view_mode.set(ViewMode::Canvas);
};
```

**布局变化**：

| ViewMode | AppBar | ToolRail | Inspector | IO 抽屉 | CodeView |
|---|---|---|---|---|---|
| `Canvas` | 显示 | 显示 | 选中态显示 | 互斥 | 隐藏 |
| `Code` | 显示 | **隐藏** | **隐藏** | **隐藏** | 显示 |

**AppBar** 在两种模式都显示，AppBar 末尾的 `btn-code-view` 按钮文案随模式切换：
- `ViewMode::Canvas` → "代码" + `<IconCode />`
- `ViewMode::Code` → "返回" + `<IconArrowLeft />`

## 7. Command Palette（Phase D 收尾）

> Phase D `core-01e-command-palette.md`（archive/）规格的 E3 Modal 升级版

```rust
// frontend-rs/src/command_palette.rs
#[component]
pub fn CommandPalette(
    visible: RwSignal<bool>,
) -> impl IntoView
```

**触发**：
- 键盘：`Ctrl+K`（Windows/Linux）/ `Cmd+K`（macOS）
- 菜单：File → "命令面板…"
- 入口按钮：AppBar 隐藏时由 `Ctrl+K` 唤起

**Props / 行为**：
- 居中浮层（Modal width=`Small 400px` 或 `Medium 640px`）
- 顶部：`<Input placeholder="搜索表/区域/枚举/便签/关系/类型..." />` + `<IconSearch />`
- 列表：模糊搜索结果
  - Tables：`{name} ({field_count} 字段)` + `<IconAddTable />`
  - Areas：`{name} ({table_count} 表)` + `<IconAddArea />`
  - Enums：`{name} ({value_count} 值)` + `<IconEnum />`
  - Notes：`{title or content[:30]}` + `<IconAddNote />`
  - Relationships：`{start_table}.{start_field} → {end_table}.{end_field}` + `<IconRelationship />`
  - Types：`{name} ({kind})` + `<IconType />`
- Enter 选中：聚焦画布对象 + 滚动到视口 + 关闭 Palette
- Esc 关闭
- ↑/↓ 键导航结果列表

**z-index**：`--cdb-z-modal`（L5，与 CodeView 同层互斥）

**视觉**：白底 + `--cdb-shadow-xl`（最高浮层）+ `--cdb-radius-xl`

## 8. 验收约束

- 存在 `btn-code-view` 与 `code-view-modal`
- SQL / DBML / JSON 三段切换可见且内容随 diagram 变化
- 复制触发成功反馈（Toast）
- 返回画布后 code-view 节点移除，可继续编辑画布
- 若生产接入 Monaco：关闭时销毁编辑器实例；主题跟随 `data-mode`（见 `core-0b`）

## 9. 不在 E4 范围

- 代码视图**双向编辑**（粘贴 SQL 应用回画布）— V2+
- Monaco IntelliSense / autocomplete 配置（V1 仅语法高亮）
- 多 Tab 同时打开（V1 单视图）

## 代码视图格式与内容

| 格式 | 生成规则（原型语义） |
|---|---|
| SQL | 各表 `CREATE TABLE …` |
| DBML | `Table` 块 + `Ref:` 关系 |
| JSON | `diagram` 对象 pretty-print |

切换：`code-format` + `data-format`；保持只读。

## 代码视图复制与返回

| 动作 | 行为 |
|---|---|
| 复制 | 读取当前格式文本；原型以 Toast 模拟成功（可不写真实剪贴板）；生产应 `clipboard.writeText` + Toast「已复制…」 |
| 返回画布 | `codeView=false`，卸载 `.code-view`，恢复 Inspector/模拟器可见性规则 |
| Esc（生产建议） | 与返回画布等价；原型以按钮为主 |

## 与 Command Palette 的关系

命令面板可含「打开代码视图」项（原型 ⌘E 提示）；Code View 与 Command（z=55）互斥——打开命令时不要求保持 code 层焦点冲突。Code View **不是** E3 Modal 居中对话框，而是画布区域覆盖层（`code-view-modal` testid 保留历史名）。

## Code View 非强制实现边界

- `monaco-editor` / `monaco-editor-wasm` Cargo/npm 依赖表不再作为本文件合并后的强制条款
- 「全屏 Modal width=XLarge」布局描述改为画布内 `.code-view` 覆盖
- 复制按钮「右下角绝对定位」改为工具栏「复制」按钮（对齐主原型）
