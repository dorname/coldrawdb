# 顶部菜单剩余 5 模态测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-UI-modals-2-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §5.3/5.4/5.5/5.6/5.9

## 1. 范围

剩余模态 / 历史边界：ImportSource、Language、SetTableWidth、ConfigureCustomTypes 等。主路径 IO 已迁抽屉后，历史 Import 模态不得再标为唯一入口。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED

| ID | 变更 | 合同 |
|---|---|---|
| UT-MM-10～14 | 保留 | 纯函数解析仍有效 |
| ST-MM-HIST-01（ADDED） | ADDED | 文档/用例须标注：历史 Import 模态为边界能力；现行主路径=更多菜单→IO 抽屉 |
| ST-MM-ESC-02（ADDED） | ADDED | 任一剩余模态 Esc/遮罩关闭后无残留层 |
| ST-MM-SCOPE（ADDED） | ADDED | remote import / 未支持语言等 V1 边界保持 Err；不得标完成 |

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

## 与主原型关系

主原型未演示的次要模态：规格可保留，但第二阶段验收优先级低于 auth/rooms/editor 主链。

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
| UT-MM-17 | SetTableMinHeight 模态最小高度解析（feat-table-resize，对称 width "0=auto"）| `editor_panels.rs::modals::parse_table_height` |
| UT-MM-18 | cardinality 推导纯函数测试（字段已参与关系计数：s==1&&e==1→1:1, s>1&&e==1→1:N, s==1&&e>1→N:1, s>1&&e>1→N:N） | `editor_panels.rs::modals::infer_cardinality` |
| UT-MM-19 | flip_reference_endpoints 翻转后重新推导 cardinality（s/e 互换） | `editor_panels.rs::modals::flip_reference_endpoints` |
| UT-MM-20 | build_reference 使用推导值而非用户必选下拉值 | `editor_panels.rs::modals::build_reference` |
| UT-MM-21 | 列表视图排序纯函数测试（按表维度属性排序：表名/字段数/类型/是否有索引） | `editor_panels.rs::sort_tables` |
| UT-MM-22 | 列表视图 tab 切换测试 | `editor_panels.rs::ListView` |
| UT-MM-23 | 列表视图过滤纯函数测试（按名称模糊匹配/按类型/按是否有索引；与 SortColumn::Type 首字段类型口径对齐） | `editor_panels.rs::filter_tables` |
| UT-MM-24 | 列表视图批量重命名纯函数测试（重名冲突处理：B2-S1 补充规则——冲突判定以改名前快照为准/处理顺序按旧名字典序/同一新名多旧名映射字典序靠前者得名其余跳过） | `editor_panels.rs::batch_rename_tables` |
| UT-MM-25 | ViewMode 三态迁移测试（Canvas→List→Canvas、Canvas→Code→Canvas、List 下画布隐藏条件） | `code_view.rs::ViewMode` |
| ST-MM-02 | 端到端 Import 模态 SQL 解析 | `frontend-rs/tests/wasm/ui.rs`（B5） |
| ST-MM-03 | ConfigureCustomTypes 关闭后跨刷新保留 | `frontend-rs/tests/wasm/ui.rs`（B5） |
| UT-MM-28 | ListView 列宽钳制 + 自适应 + 列宽结构测试（clamp_column_width min=60, max=480；auto_calc_column_width 公式 max(60, min(480, chars × 8 + 40))；max_chars_for_column 按列名实际内容算最长字符数；ColumnWidths 结构 get/set 通路；15 子用例覆盖边界/钳制/saturating 溢出/混合字符数） | `editor_panels.rs::clamp_column_width` + `auto_calc_column_width` + `max_chars_for_column` + `ColumnWidths` |
| UT-MM-29 | 表/字段分组纯函数测试（GroupByMode {None, ByTag} 两模式；统一输出 Vec<Bucket{key, fields: Vec<(table_id, field_id)>}>；None = 单桶 _flat 含所有字段；ByTag 按 Field.tag 分桶空 tag 归 (empty) 兜底，BTreeMap 字典序；大小写敏感；7 子用例覆盖空表/单 tag/混合 tag/大小写/单字段多 tag/输出形状统一） | `editor_panels.rs::group_tables` |
| UT-MM-30 | rAF 调度去重 + TextCacheKey 键结构测试（schedule_render_dedup 可测同步核 pending 状态机：首次入队执行/二次 noop/清 pending 后可入队/多轮入队各执行；TextCacheKey font_px 容差 0.01 相等 + text 不等；6 子用例覆盖三态/多轮/键相等性） | `editor_render.rs::schedule_render_dedup` + `TextCacheKey` |
