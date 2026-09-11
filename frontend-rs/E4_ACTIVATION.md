# E4 Monaco 激活指南

> 关联提案：redesign-phase-e-design-system-migration（E4 子批次）
> 关联规格：`logos/resources/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
> 关联代码：`frontend-rs/src/code_view.rs` + `frontend-rs/src/command_palette.rs`
> 关联样式：`frontend-rs/src/styles.css` 中 `.cdb-monaco-container` / `.cdb-command-palette` / `.cdb-code-view__copy`

## 1. 现状（V1 skeleton）

E4 在 2026-06-15 Phase E 合并中以 **V1 skeleton** 形式落地：

| 维度 | V1 状态 |
|---|---|
| Rust 骨架 | `code_view.rs` 含 `CodeView` / `ViewMode` / `ViewModeToggle`；`command_palette.rs` 含 `CommandPalette` / `setup_command_palette_shortcut` |
| Props 签名 | 完整（与规格 §2 / §7 一一对应） |
| 视觉占位 | 渲染 "待 wasm-pack 环境激活" 占位文案（V1） |
| CSS 样式 | `.cdb-monaco-container` / `.cdb-command-palette` / `.cdb-code-view__copy` 就位 |
| 静态 UT | `tests/code_view.rs` 7 个测试 PASS（验证文件存在 + Props + Enum + CSS 选择器） |
| Monaco mount | ❌ 未实现（需 wasm-pack） |
| DBML 注入 | ❌ 未实现（需 Monaco 运行时） |
| 浏览器 e2e | ❌ 未实现（需浏览器） |

## 2. 沙箱限制原因

| 工具 | 沙箱状态 | 影响 |
|---|---|---|
| `cargo install wasm-pack` | 不可用 | Monaco 二进制无法编译进 WASM |
| `rustup target add wasm32-unknown-unknown` | 未装 | WASM 目标不可用 |
| 浏览器（Playwright/Chromium） | 未装 | e2e 与真实交互验证不可用 |
| `node_modules` ~ 30MB Monaco | 不可下载 | npm 依赖缺失 |

## 3. 激活前置环境

### 3.1 工具链

```bash
# 1. Rust 工具链
rustup --version                # ≥ 1.74
rustup target add wasm32-unknown-unknown

# 2. wasm-pack
cargo install wasm-pack          # ~5 分钟
wasm-pack --version

# 3. Node + npm
node --version                   # ≥ 18
npm --version                    # ≥ 9

# 4. 验证
rustc --print target-list | grep wasm32
```

### 3.2 依赖安装

```bash
cd frontend-rs
npm install --save monaco-editor monaco-editor-wasm
ls node_modules/monaco-editor/    # 应有 esm/、min/ 等目录
```

### 3.3 Docker 镜像（如 CI 复现）

```dockerfile
FROM rust:1.78-bookworm
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack
RUN apt-get update && apt-get install -y nodejs npm
WORKDIR /app
```

## 4. 激活步骤

### 步骤 1：启用 Cargo.toml 依赖

`frontend-rs/Cargo.toml` 当前：

```toml
# E4 Monaco 依赖（V1 占位：需 wasm-pack + 浏览器环境激活）
# monaco-editor-wasm = "0.45"  # uncomment when wasm32 target + wasm-pack available
# wasm-bindgen-futures = "0.4" # uncomment with monaco
# js-sys = "0.3"                # 已有
```

修改为：

```toml
# E4 Monaco 依赖（V1 占位：需 wasm-pack + 浏览器环境激活）
monaco-editor-wasm = "0.45"  # activated 2026-XX-XX
wasm-bindgen-futures = "0.4" # activated with monaco
js-sys = "0.3"                # 已有
```

或 sed：

```bash
cd frontend-rs
sed -i 's/^# monaco-editor-wasm = "0.45"$/monaco-editor-wasm = "0.45"/' Cargo.toml
sed -i 's/^# wasm-bindgen-futures = "0.4"$/wasm-bindgen-futures = "0.4"/' Cargo.toml
```

### 步骤 2：补全 code_view.rs（按规格 §3.2–§3.4）

参考 `core-0a-code-editor.md` §3 实现以下：

1. **`#[wasm_bindgen] extern "C" 块`**（绑定 monaco JS API）
   ```rust
   #[wasm_bindgen]
   extern "C" {
       #[wasm_bindgen(js_namespace = monaco)]
       pub type Monaco;
       #[wasm_bindgen(method, js_namespace = monaco)]
       pub fn editor(this: &Monaco) -> Editor;
       // ... 完整 monaco.editor / monaco.languages / set_theme 绑定
   }
   ```

2. **lazy import('monaco-editor/...')**
   ```rust
   create_effect(move |_| {
       if visible.get() {
           // 首次进入时按需加载 ~30MB bundle
           let _ = import("monaco-editor/esm/vs/editor/editor.api").then(|_| {
               // 调用 monaco.editor.create(container, options)
           });
       }
   });
   ```

3. **DBML setup**（§3.3）
   - `monaco.languages.register(&"dbml".into())`
   - `monaco.languages.set_monarch_tokens_provider(&"dbml".into(), &DBML_TOKEN_PROVIDER)`
   - `monaco.editor.set_theme(&theme.into())` 根据 `THEME_MODE`（E5 阶段接入）

4. **复制按钮**（§3.4）
   - 绝对定位 `right-6 bottom-2 z-10`（CSS 已就位）
   - `navigator.clipboard().write_text(&monaco.get_value())` + Toast

### 步骤 3：补全 command_palette.rs（按规格 §7）

1. **真实模糊搜索**：6 种对象 × 模糊匹配（表/区域/枚举/便签/关系/类型）
2. **Ctrl+K / Cmd+K 全局监听**：
   ```rust
   window::event_listener(keydown, move |ev| {
       if ev.key() == "k" && (ev.ctrl_key() || ev.meta_key()) {
           visible.update(|v| *v = !*v);
       }
   });
   ```
3. **Enter 跳转 + 选中 + 滚动到视口**（与 `editor_panels.rs` 接线）
4. **↑/↓ 键盘导航** + highlight RwSignal

### 步骤 4：编译 WASM

```bash
cd frontend-rs
wasm-pack build --target web --release
# 产出：
#   pkg/frontend_rs_bg.wasm     (~30MB)
#   pkg/frontend_rs.js          (loader)
#   pkg/frontend_rs.d.ts        (TypeScript types)
#   pkg/package.json
```

### 步骤 5：trunk 集成

```bash
# 1. 修改 frontend-rs/index.html 引用 pkg 输出
# <script type="module">
#   import init from './pkg/frontend_rs.js';
#   await init();
# </script>

# 2. 修改 trunk.toml 包含 monaco-editor 的 esm 资源
# [build]
# public_url = "/"
# [serve]
# address = "0.0.0.0"
# port = 8080

# 3. 启动 dev server
trunk serve --release
```

### 步骤 6：浏览器 e2e（ST-PE-08）

```bash
# 1. 安装 Playwright
cd tests
npm install -D playwright
npx playwright install chromium

# 2. 编写 e2e
cat > code-view-e2e.spec.ts <<'TS'
import { test, expect } from '@playwright/test';

test('ST-PE-08: CodeView Monaco load + copy', async ({ page }) => {
  await page.goto('http://localhost:8080/editor');
  await page.click('[data-testid="btn-code-view"]');
  await page.waitForSelector('.cdb-monaco-container .monaco-editor');
  await page.click('.cdb-code-view__copy');
  const text = await page.evaluate(() => navigator.clipboard.readText());
  expect(text).toMatch(/CREATE TABLE/);
});
TS

# 3. 运行
npx playwright test code-view-e2e.spec.ts
```

### 步骤 7：浏览器手动验证清单

- [ ] 访问 `/editor` 加载正常
- [ ] network 面板无 `monaco-editor` 请求（V1 优化：lazy load）
- [ ] 点击 AppBar `btn-code-view` → 切换到 Code 模式
- [ ] Tool Rail / Inspector / IO 抽屉全部隐藏
- [ ] Monaco bundle ~30MB 加载完成
- [ ] SQL / DBML / JSON 三个 Tab 切换正常
- [ ] DBML 语法高亮（关键字、类型、关系）
- [ ] 复制按钮 → 剪贴板含当前 SQL/DBML
- [ ] 暗色模式 → Monaco 主题自动切到 `vs-dark`
- [ ] Ctrl+K → CommandPalette 唤起
- [ ] 输入 "users" → 模糊匹配 Tables 列表
- [ ] Enter → 跳转并选中 users 表
- [ ] Esc → 关闭 CommandPalette

## 5. 验证矩阵（激活后必跑）

| 测试 | 命令 | 期望 |
|---|---|---|
| `cargo check` | `cd frontend-rs && cargo check` | 0 errors |
| `cargo test` | `cd frontend-rs && cargo test` | 26+ UT 全 PASS |
| `wasm-pack build` | `cd frontend-rs && wasm-pack build --target web` | 产出 `pkg/*.wasm` |
| WASM bundle 体积 | `ls -la pkg/*.wasm` | ~30MB |
| ST-PE-08 浏览器 e2e | `npx playwright test code-view-e2e.spec.ts` | PASS |

## 6. 失败模式与回滚

| 失败 | 原因 | 回滚 |
|---|---|---|
| `cargo check` E0412 / E0432 | monaco 依赖未装 | `cargo install wasm-pack` + 重装依赖 |
| `wasm-pack build` link error | wasm32 target 缺失 | `rustup target add wasm32-unknown-unknown` |
| `pkg/frontend_rs_bg.wasm` > 50MB | monaco 完整 bundle | 改用 monaco-editor-core（仅核心 API）+ 手动加载语言 |
| Playwright timeout | Monaco 首次加载慢 | 调高 `await page.waitForSelector` timeout 到 30s |
| 浏览器 SQL 高亮缺失 | DBML setup 未注入 | 检查 `monaco.languages.register` 是否在 mount 后调用 |

## 7. 预计工作量

| 阶段 | 工作量 | 前提 |
|---|---|---|
| 沙箱内补完整骨架（不含 Monaco mount 验证） | 1-2 小时 | 不需要 wasm-pack |
| 步骤 2-3 补 Rust 代码 | 4-6 小时 | 参考规格 §3.2 / §7 |
| 步骤 4 wasm-pack 编译 | 5-10 分钟 | 工具链就绪 |
| 步骤 5 trunk 集成 | 30 分钟 | 已有 trunk.toml |
| 步骤 6 浏览器 e2e 调试 | 1-2 天 | 第一次 monaco 集成需要调通网络 + bundle 加载 |
| 步骤 7 手动验证 | 30 分钟 | 7 项清单 |

**总预计**：1-2 天（含调试） / 不含调试 4-6 小时。

## 8. 不在 E4 范围

- ❌ 代码视图**双向编辑**（粘贴 SQL 应用回画布）— V2+
- ❌ Monaco IntelliSense / autocomplete（V1 仅语法高亮）
- ❌ 多 Tab 同时打开（V1 单视图）
- ❌ Monaco 主题自定义（V1 跟随 `THEME_MODE` 切 vs/vs-dark）

## 9. 相关链接

- 规格：`logos/resources/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
- 测试用例：`logos/resources/test/core-PE-design-system-test-cases.md` §5
- 测试代码：`frontend-rs/tests/code_view.rs`
- 骨架代码：`frontend-rs/src/code_view.rs` + `frontend-rs/src/command_palette.rs`
- Cargo 占位：`frontend-rs/Cargo.toml`（注释的 monaco 行）
- main 分支 CodeEditor 参考：`origin/main:src/components/CodeEditor/index.jsx` + `setUpDBML.js`

---

**最后更新**：2026-06-15（Phase E 合并时落地 V1 skeleton）
**下次激活**：等 wasm-pack + wasm32 + 浏览器环境就绪
