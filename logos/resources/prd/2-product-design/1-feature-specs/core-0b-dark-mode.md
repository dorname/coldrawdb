# Dark Mode 规格（E5）

## 1. 概述

E5 实现 drawdb-web 的暗色模式，对齐 main `settings.mode === "dark"` 全局切换（`darkBgTheme = "#16161A"`）。E1 阶段已定义 `<html data-mode="light|dark">` DOM 接口与 token 覆盖规则（`core-07-design-tokens.md` §14/§15），E5 填充具体映射值与 JS 切换逻辑。

**E5 目标**：
- `<html data-mode="dark">` 时全局 token 切换为暗色映射
- 用户偏好持久化到 `localStorage["cdb-mode"]`
- 首次访问跟随 `prefers-color-scheme` 媒体查询
- AppBar 提供 Theme toggle 按钮（E3 §1）+ View → Theme 子菜单

## 2. Token 暗色映射

完整 token 列表见 `core-07-design-tokens.md` §2–§14。本节定义 light → dark 的映射规则。

| Light token | 值 | Dark 值 | 备注 |
|---|---|---|---|
| `--cdb-color-primary` | `#175e7a` | `#4ba3c4` | 暗色下提高亮度，对比度 |
| `--cdb-color-primary-hover` | `#134c63` | `#6cb8d4` | — |
| `--cdb-color-primary-soft` | `#e6f1f5` | `#1a3a48` | 暗色下加深而非提亮 |
| `--cdb-color-text-0` | `#1f2937` | `#f9fafb` | 反转 |
| `--cdb-color-text-1` | `#374151` | `#e5e7eb` | — |
| `--cdb-color-text-2` | `#6b7280` | `#9ca3af` | — |
| `--cdb-color-text-3` | `#9ca3af` | `#6b7280` | — |
| `--cdb-color-bg-0` | `#ffffff` | `#16161a` | **main `darkBgTheme`** |
| `--cdb-color-bg-1` | `#f9fafb` | `#1f1f23` | toolbar / sidesheet |
| `--cdb-color-bg-2` | `#f3f4f6` | `#2a2a2e` | popover 嵌套 |
| `--cdb-color-bg-3` | `#e5e7eb` | `#0e0e10` | 画布背景（最深） |
| `--cdb-color-border` | `#e5e7eb` | `#2a2a2e` | — |
| `--cdb-color-border-strong` | `#d1d5db` | `#3a3a3e` | — |
| `--cdb-color-warning-soft` | `#fef3c7` | `#5c4a0e` | 暗色下加深 |
| `--cdb-color-success-soft` | `#d1fae5` | `#0e4f3a` | — |
| `--cdb-color-error-soft` | `#fee2e2` | `#5c1a1a` | — |
| `--cdb-color-info-soft` | `#dbeafe` | `#1a3a5c` | — |
| `--cdb-shadow-sm` | `rgba(0,0,0,0.05)` | `rgba(0,0,0,0.3)` | 暗色阴影更深 |
| `--cdb-shadow-md` | `rgba(0,0,0,0.07)` | `rgba(0,0,0,0.4)` | — |
| `--cdb-shadow-lg` | `rgba(0,0,0,0.1)` | `rgba(0,0,0,0.5)` | — |
| `--cdb-shadow-xl` | `rgba(0,0,0,0.1)` | `rgba(0,0,0,0.6)` | — |

## 3. CSS 实现

`[data-mode="dark"]` 与 `@media (prefers-color-scheme: dark)` 内 `:root:not([data-mode="light"]):not([data-mode="dark"])` **必须**包含相同 Token 集合（primary / grey / semantic / text / bg / border / shadow / surface），不得仅覆写部分 Token。

新增 surface Token 暗色映射：

| Token | Dark 值 |
|---|---|
| `--cdb-color-canvas-grid` | `rgba(255, 255, 255, 0.06)` |
| `--cdb-color-inspector-edge` | `rgba(255, 255, 255, 0.04)` |
| `--cdb-color-focus-error` | `rgba(248, 113, 113, 0.2)` |
| `--cdb-color-error-hover-bg` | `rgba(248, 113, 113, 0.12)` |

```css
/* styles.css — 节选 */

[data-mode="dark"] {
  color-scheme: dark;
  --cdb-color-primary: #4ba3c4;
  /* ... 完整 dark token，含 surface 四组 ... */
  --cdb-color-canvas-grid: rgba(255, 255, 255, 0.06);
  --cdb-color-inspector-edge: rgba(255, 255, 255, 0.04);
  --cdb-color-focus-error: rgba(248, 113, 113, 0.2);
  --cdb-color-error-hover-bg: rgba(248, 113, 113, 0.12);
}

@media (prefers-color-scheme: dark) {
  :root:not([data-mode="light"]):not([data-mode="dark"]) {
    color-scheme: dark;
    /* 与 [data-mode="dark"] 相同 token 全集（R3） */
  }
}
```

## 4. JS 切换逻辑

```rust
// frontend-rs/src/settings.rs（扩展）
use wasm_bindgen::prelude::*;

pub enum ThemeMode { Light, Dark, System }

pub static THEME_MODE: Lazy<RwSignal<ThemeMode>> = Lazy::new(|| {
    let initial = read_local_storage("cdb-mode")
        .and_then(|s| match s.as_str() {
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            "system" => Some(ThemeMode::System),
            _ => None,
        })
        .unwrap_or(ThemeMode::System);
    
    let signal = create_rw_signal(initial);
    
    // 立即应用
    apply_theme_mode(signal.get());
    
    // 监听变化
    create_effect(move |_| {
        let mode = signal.get();
        write_local_storage("cdb-mode", &format!("{:?}", mode).to_lowercase());
        apply_theme_mode(mode);
    });
    
    signal
});

pub fn apply_theme_mode(mode: ThemeMode) {
    let html = document().document_element().unwrap();
    let resolved = match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::System => {
            if web_sys::window()
                .unwrap()
                .match_media("(prefers-color-scheme: dark)").unwrap()
                .matches()
            { "dark" } else { "light" }
        }
    };
    html.set_attribute("data-mode", resolved).unwrap();
}
```

## 5. UI 入口

### 5.1 AppBar Theme Toggle 按钮（E3 §1 已实现类名）

```rust
<Button
    class="cdb-btn cdb-btn--tertiary"
    data-testid="btn-theme-toggle"
    on_click=toggle_theme
>
    {move || match THEME_MODE.get() {
        ThemeMode::Light => view! { <IconSun /> },
        ThemeMode::Dark => view! { <IconMoon /> },
        ThemeMode::System => view! { <IconSun /> <IconMoon /> },
    }}
</Button>
```

### 5.2 View → Theme 子菜单

```
View ▼
  └─ Theme ▶
       ├─ ☀ 浅色 (Light)
       ├─ 🌙 暗色 (Dark)
       └─ 💻 跟随系统 (System)
```

用 E3 `<Dropdown trigger=Click>` + `<DropdownItem active=... on_click=set_mode>`。

## 6. 持久化与跨标签页

| 行为 | 实现 |
|---|---|
| 持久化 | `localStorage["cdb-mode"]` = `"light" \| "dark" \| "system"` |
| 跨标签页同步 | `window.addEventListener("storage", cb)` 监听其他标签页的修改 |
| 跟随系统变化 | `matchMedia("(prefers-color-scheme: dark)").addEventListener("change", cb)`（仅在 mode=System 时响应） |

## 7. Monaco 主题同步（E4 衔接）

E4 CodeView 在 `setup_dbml()` 中根据当前 mode 设置 Monaco 主题：
- `ThemeMode::Light` → `monaco.editor.set_theme("vs")`
- `ThemeMode::Dark` → `monaco.editor.set_theme("vs-dark")`

切换主题时调用 `monaco.editor.set_theme()` 实时更新（E4 `UT-E4-05`）。

## 8. 验收约束

- `frontend-rs/src/settings.rs` 含 `THEME_MODE` 全局信号 + `apply_theme_mode` 函数
- `<html>` 元素 `data-mode` 属性在 light/dark 间切换
- `localStorage["cdb-mode"]` 持久化用户选择
- `prefers-color-scheme: dark` 在 System 模式下生效
- AppBar `btn-theme-toggle` 点击循环切换 Light → Dark → System → Light
- View → Theme 子菜单点击立即应用并持久化
- Monaco 主题实时跟随切换（UT-E5-04）
- 跨标签页 storage 事件触发同步（UT-E5-05）
- ST-PE-06：Playwright 切换暗色模式，截图含暗色 token

## 9. 不在 E5 范围

- 暗色 + 高对比度（accessibility）模式 — V2+
- 用户自定义主题色（替换 `--cdb-color-primary`）— V2+
- 自动切换（按时间 19:00 切暗色）— V2+

