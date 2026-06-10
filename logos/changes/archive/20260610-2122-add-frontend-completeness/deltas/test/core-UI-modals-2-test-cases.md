# 顶部菜单剩余 5 模态测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-UI-modals-2-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §5.3/5.4/5.5/5.6/5.9

## 1. 范围

B5 模态补全（5 个剩余）：
- `ImportModal`：粘贴 SQL → 调用 `/api/v1/bridge/import/local`（B5 解析后端 stub）
- `ImportSourceModal`：选择 local / remote（V1 仅 local 实际生效）
- `LanguageModal`：切换 zh / en（V1 提示 toast）
- `SetTableWidthModal`：批量设置表宽（0 = auto）
- `ConfigureCustomTypesModal`：增删改自定义类型（V1 仅前端 session state）

**对应实现**：`frontend-rs/src/editor_panels.rs::modals`（B4 子模块扩展）

## 2. UT 用例

### UT-MM-10 — Import 模态 SQL 解析（纯函数）

- **位置**：`frontend-rs/src/editor_panels.rs::modals::parse_sql_statements`
- **步骤**：传入 `text = "CREATE TABLE a (id INT); INSERT INTO a VALUES (1);"`
- **断言**：
  - 返回 `Ok(vec!["CREATE TABLE a (id INT)".to_string(), "INSERT INTO a VALUES (1)".to_string()])`
  - 空字符串 → `Ok(vec![])`（无语句不报错）
  - 包含 `-- comment` 单行注释 → 注释被去除

### UT-MM-11 — SetTableWidth 模态宽度解析

- **位置**：`frontend-rs/src/editor_panels.rs::modals::parse_table_width`
- **步骤**：
  - 传入 `"200"` → 200
  - 传入 `"0"` → 0（auto）
  - 传入 `"abc"` → Err
  - 传入 `""` → Err
- **断言**：
  - `parse_table_width("200") == Ok(200)`
  - `parse_table_width("0") == Ok(0)`
  - `parse_table_width("abc").is_err()`
  - `parse_table_width("").is_err()`

### UT-MM-12 — Language 模态验证

- **位置**：`frontend-rs/src/editor_panels.rs::modals::validate_language`
- **步骤**：传入 `"en"` / `"zh"` / `"fr"`
- **断言**：
  - `validate_language("en") == Ok(())`
  - `validate_language("zh") == Ok(())`
  - `validate_language("fr").is_err()`（V1 仅双语）

### UT-MM-13 — ConfigureCustomTypes 增删（纯数据层）

- **位置**：`frontend-rs/src/editor_panels.rs::modals::{add_custom_type, remove_custom_type}`
- **步骤**：
  - 初始 `vec![]`
  - `add_custom_type(&mut v, "uuid", "VARCHAR(36)")` → `v == [("uuid", "VARCHAR(36)")]`
  - 重复 add 同名 → 替换（不重复）
  - `remove_custom_type(&mut v, "uuid")` → `v.is_empty()`
  - `remove_custom_type` 不存在的 name → no-op
- **断言**：
  - add 后 Vec 长度正确
  - add 已存在则替换
  - remove 后 Vec 为空
  - remove 不存在不 panic

### UT-MM-14 — ImportSource 模态选择解析

- **位置**：`frontend-rs/src/editor_panels.rs::modals::resolve_import_source`
- **步骤**：传入 `"local"` / `"remote"` / `"http"`
- **断言**：
  - `resolve_import_source("local") == Ok(SourceKind::Local)`
  - `resolve_import_source("remote") == Ok(SourceKind::Remote)`
  - `resolve_import_source("http").is_err()`（V1 不支持）

## 3. ST 用例

### ST-MM-02 — 端到端 Import 模态 SQL 解析（e2e）

- **位置**：`frontend-rs/tests/wasm/ui.rs`（B5 接入）
- **类型**：wasm-pack test --headless --chrome
- **步骤**：
  1. 打开 File → Import 模态
  2. 粘贴 `"CREATE TABLE users (id INT PRIMARY KEY);"`
  3. 点 OK
  4. 验证调用 `POST /api/v1/bridge/import/local` + 跳转新 diagram
- **B5 标记 skip**：完整 e2e 跑在 B5 wasm-pack test 接入后

### ST-MM-03 — ConfigureCustomTypes 关闭后跨刷新保留

- **位置**：`frontend-rs/tests/wasm/ui.rs`（B5 接入）
- **类型**：wasm-pack test --headless --chrome
- **B5 标记 skip**：完整 e2e 跑在 B5 wasm-pack test 接入后（V1 限制：仅 session state，不跨刷新）

## 4. V1 边界

- ❌ Remote Import 真实接入（V1 仅 UI 入口）
- ❌ Language 实际 i18n 文案切换（V1 切换后只 toast 提示）
- ❌ ConfigureCustomTypes 跨刷新保留（V1 仅 session state — spec §5.9 限制）
- ❌ SetTableWidth 实时应用（V1 仅提交后批量更新）

## 5. 对齐参考源

- `core-05-top-menu-modals.md` §5.3 / §5.4 / §5.5 / §5.6 / §5.9
- `frontend-rs/src/editor_panels.rs::modals`（B4 子模块扩展）

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-MM-10 | Import 模态 SQL 解析 | `editor_panels.rs::modals::parse_sql_statements` |
| UT-MM-11 | SetTableWidth 模态宽度解析 | `editor_panels.rs::modals::parse_table_width` |
| UT-MM-12 | Language 模态验证 | `editor_panels.rs::modals::validate_language` |
| UT-MM-13 | ConfigureCustomTypes 增删 | `editor_panels.rs::modals::{add_custom_type,remove_custom_type}` |
| UT-MM-14 | ImportSource 模态选择解析 | `editor_panels.rs::modals::resolve_import_source` |
| ST-MM-02 | 端到端 Import 模态 SQL 解析 | `frontend-rs/tests/wasm/ui.rs`（B5） |
| ST-MM-03 | ConfigureCustomTypes 关闭后跨刷新保留 | `frontend-rs/tests/wasm/ui.rs`（B5） |
