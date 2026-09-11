# Delta — core-S07-test-cases.md（新增文件）

> 目标主文档：`logos/resources/test/core-S07-test-cases.md`（新文件，merge 时整体创建）

## ADDED — 全文（新文件）

# S07 数据字典 — 测试用例

> 场景时序：`core-S07-data-dictionary.md`；功能规格：`core-01e-data-dictionary.md`。
> 全部为前端用例（无新 API，无编排测试）；实现在 `frontend-rs`（wasm-bindgen-test / 纯函数单测）+ OpenLogos reporter。

## 1. 测试边界

- 覆盖：字典 CRUD、字典项编辑校验、字段绑定与级联、Markdown 导出、持久化兼容、Viewer 只读
- 不覆盖：S05 OT 合并细节（S05 用例已覆盖 op 通路，S07 仅验证字典编辑走同一 commit/op 通路）、Word 导出/代码生成（边界外）

## 2. UT 用例

| 用例 ID | 输入 / 操作 | 预期结果 |
|---|---|---|
| UT-S07-01 | 点击「+ 新建字典」 | 列表追加 `dict_N` / `code_N`，空 items；进 undo 栈 |
| UT-S07-02 | 将字典编码改为已存在编码 | 红标提示，不落账、不触发保存 |
| UT-S07-03 | 同字典内两个字典项 value 相同 | 行内红标；其余编辑不受影响 |
| UT-S07-04 | Inspector 绑定字段到 `gender` | `field.dict_code="gender"`；字段卡显示摘要 Tag `1=男 2=女` |
| UT-S07-05 | 修改字典编码 `gender→sex`（已被 2 字段引用） | 两字段 `dict_code` 同步改写为 `sex`；单次 undo 恢复 |
| UT-S07-06 | 删除被 2 字段引用的字典并确认 | 字典删除 + 两字段 `dict_code` 置空；单次 undo 整体恢复 |
| UT-S07-07 | 对含 2 字典、2 绑定的图执行导出 | Markdown 含三段：清单 2 行、明细按 sort 序、绑定关系 2 行 |
| UT-S07-08 | 无字典时打开导出 | 导出按钮禁用 + title 提示 |
| UT-S07-09 | diagram JSON 反序列化（无 `dictionaries` 键） | 解析成功，`dictionaries=[]` |
| UT-S07-10 | 字段 `dict_code` 指向不存在字典 → 加载规范化 | 静默置空 + warn；加载不中断 |

## 3. ST 用例

| 用例 ID | 场景 | 预期结果 |
|---|---|---|
| ST-S07-01 | 建字典 → 加 3 字典项 → 绑 2 字段 → 等自动保存 → 刷新重开 | 字典/字典项/绑定完整恢复；revision 递增无 409 |
| ST-S07-02 | 加载本变更前保存的旧图（无 dictionaries） | 正常打开，字典 Tab 空态，编辑/保存后 JSON 新增 `dictionaries` 键 |
| ST-S07-03 | Viewer 角色打开 | 字典 Tab 可浏览、导出可用；新建/编辑/删除/绑定控件全部 disabled |
| ST-S07-04 | 导出 → 下载 `.md` → 与当前模型对照 | 文件名 `data-dictionary-{name}.md`；内容与模型一致 |
| ST-S07-05 | 删除被引用字典 → undo | 字典与字段绑定一次 undo 全部恢复 |

## 4. 验收标准追溯

| FR | 覆盖用例 |
|---|---|
| FR-25 字典 CRUD | UT-S07-01/02/03, ST-S07-01 |
| FR-26 字段绑定 | UT-S07-04/05, ST-S07-01 |
| FR-27 快照持久化 | UT-S07-09, ST-S07-01/02 |
| FR-28 引用删除确认 | UT-S07-06, ST-S07-05 |
| FR-29 Markdown 导出 | UT-S07-07/08, ST-S07-04 |
| Viewer 只读（FR-20 延伸） | ST-S07-03 |

## 5. Reporter 契约

- 所有 UT/ST 用例执行后写入 `logos/resources/verify/test-results.jsonl`，字段遵循 `logos/spec/test-results.md`（case_id / scenario=S07 / result / duration_ms 等）
- 跳过（skip）必须注明原因；失败必须附断言输出
