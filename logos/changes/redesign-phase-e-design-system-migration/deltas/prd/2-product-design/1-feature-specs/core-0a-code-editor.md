# Delta — core-0a-code-editor.md（新文件）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E4）
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
> 对齐参考源：main `src/components/CodeEditor/index.jsx` + `setUpDBML.js`、Phase D 已归档的 `core-01f-code-view.md`（archive 目录）
> 最后更新：2026-06-15

# Code Editor 规格（E4 — Monaco 集成）

## 1. 概述

E4 用 Monaco Editor 替换 Phase D V1 计划的 `<textarea readonly>` 方案，实现 `core-01f-code-view.md`（archive/）的全屏 SQL/DBML 代码视图。**同时实现 Command Palette**（Phase D `core-01e-command-palette.md` archive/）—— 两者都用 E3 Modal 容器（`--cdb-z-modal` L5）承载。

**E4 是 Phase D 的代码收尾**：Phase D 规格已 archive，本规格作为 Phase D 的 Monaco 升级版落地。Phase D archive/ 目录中的 `core-01e` / `core-01f` 不再单独实现。

## 2. CodeView 组件

```rust
// frontend-rs/src/code_view.rs
#[component]
pub fn CodeView(
    visible: RwSignal<bool>,
    language: RwSignal<CodeLanguage>,
    #[prop(default = true)] show_copy: bool,
    #[prop(default = true)] readonly: bool,
) -> impl IntoView

pub enum CodeLanguage { Sql, Dbml, Json }
```

**Props 行为**：
- `visible`：RwSignal 控制全屏视图显示
- `language`：当前展示的代码语言（SQL / DBML / JSON）
- `show_copy=true`：右下角显示 `<Button variant=Secondary icon=IconCopy />`（main `absolute right-6 bottom-2 z-10`）
- `readonly=true`：只读模式（V1 不支持编辑）

**视觉**：
- 全屏 modal（`Modal width=XLarge 1200px` 或 `width=Full`）
- 顶部 Tab 切换：`<Tab>SQL</Tab>` | `<Tab>DBML</Tab>` | `<Tab>JSON</Tab>`
- 右上角：`<Button variant=Tertiary icon=IconClose on_click=close>` 关闭
- 主体：Monaco editor 容器
- 右下角：复制按钮（绝对定位）

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

- `frontend-rs/Cargo.toml` 含 `monaco-editor-wasm = "0.45"` 依赖
- `frontend-rs/src/code_view.rs` 存在
- `frontend-rs/src/command_palette.rs` 存在
- `frontend-rs/src/editor_panels.rs` 含 `ViewMode` 信号 + `btn-code-view` 接线
- `Ctrl+K` / `Cmd+K` 唤起 CommandPalette（UT-E4-01）
- CodeView 复制按钮成功复制 SQL/DBML 文本（UT-E4-02）
- CodeView 关闭时 Monaco 销毁（避免内存泄漏，UT-E4-03）
- Monaco lazy load：首次进入前 `network` 面板无 `monaco-editor` 请求（UT-E4-04）
- ST-PE-08：Playwright 加载 Code View，截图含 SQL 高亮

## 9. 不在 E4 范围

- 代码视图**双向编辑**（粘贴 SQL 应用回画布）— V2+
- Monaco IntelliSense / autocomplete 配置（V1 仅语法高亮）
- 多 Tab 同时打开（V1 单视图）
