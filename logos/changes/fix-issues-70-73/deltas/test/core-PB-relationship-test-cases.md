# Delta — core-PB-relationship-test-cases.md（修改）

> 模块：core | 提案：fix-issues-70-73 | 关联 issue：#73
> 场景：S01；以下用例均须真实执行后接入 OpenLogos reporter。

## ADDED — UT-PB-38 — 关系显示名与空值兜底

- **位置**：`frontend-rs/src/editor_core.rs` 的 `relation_display_name` 纯函数测试。
- **GIVEN**：分别构造 id 为 `r1`、名称为 `user_posts`、空串 `""`、纯空白 `"   "` 的三条 Reference。
- **WHEN**：逐条调用 `relation_display_name`。
- **THEN**：
  1. 具名关系返回原始名称 `user_posts`。
  2. 空串与纯空白均返回且仅返回 `未命名关系（r1）`。
  3. 空值兜底不得返回 `ref-r1`，且调用前后 `Reference.name` 保持原值，不得把 placeholder 写回模型。
- **reporter**：测试真实运行后登记 `UT-PB-38` 到 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。

## ADDED — ST-PB-22 — 关系名称真实编辑、持久化与只读

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：在现有 parity 浏览器链路中预置两张表及一条 id 为 `r1`、原始 `name=""` 的关系；通过真实页面进入关系 Inspector。测试可使用现有隔离 API/房间夹具，但不得用静态源码锚点、直接修改 store 或声明式结果代替浏览器交互。
- **WHEN / THEN**：
  1. **原始值与 placeholder**：`inspector-ref-name` 的 value 为 `""`，placeholder 为 `未命名关系（r1）`；placeholder 不得成为输入值或写入文档。
  2. **真实编辑与 trim**：向输入框填入 `"  user_posts  "` 并触发真实 blur；等待既有保存链完成，持久化文档中的 `r1.name == "user_posts"`，输入框 value 同步为 trim 后的值。
  3. **同值不写**：再次以 `user_posts`（可带首尾空白）执行 blur；与步骤 2 完成时相比，不得新增文档 PUT 或协作 op，不得重复置脏。
  4. **保存重载与显示联动**：重载页面并重新选中 `r1`，名称仍为 `user_posts`；选中端点表时，既有相关关系列表显示 `user_posts`；开启既有关系标签后，`r1` 的标签实际绘制内容使用 `user_posts`，不得继续显示未命名兜底。标签文本可通过既有探针或测试侧画布文本采集断言，不得只断言 `visible_ids`。
  5. **清空并再次重载**：将名称改为纯空白并 blur，等待第二次保存完成后重载；持久化 `r1.name == ""`，输入框 value 为 `""`、placeholder 为 `未命名关系（r1）`；相关关系列表与 `r1` 标签均回退该统一显示名。
  6. **只读**：切换到 Viewer 或分享只读上下文，`inspector-ref-name` 必须 disabled；记录当前文档与保存/op 计数，尝试编辑或触发 blur 后名称不变且不产生新增写入。
- **reporter**：上述步骤全部真实通过后登记一次 `ST-PB-22` 到 `logos/resources/verify/test-results.jsonl`；任一步骤失败不得写入 PASS，不得以测试存在或源码匹配替代执行结果。

## ADDED — 用例登记追加

| TC ID | 描述 | 实现位置 |
|---|---|---|
| UT-PB-38 | 具名显示与空串/纯空白的统一未命名兜底（#73） | `frontend-rs/src/editor_core.rs` 纯函数测试 |
| ST-PB-22 | 真实 blur 编辑、trim、同值不写、保存重载、列表/标签、清空与只读（#73） | `frontend-rs/scripts/test-spec-parity-d.mjs` |
