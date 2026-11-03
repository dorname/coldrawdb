## ADDED — 2.x UT 用例（存量非空图首次物化保真）

### UT-C-11 — 存量非空图首次物化保持实体 ID、关系四端点与附属对象

- **位置**：`backend/src/collab/mod.rs` 或邻近既有后端协作测试模块，测试函数名使用 `ut_c11_*` 前缀。
- **关联**：fix-issues-70-73 / issue #72；验证导入后的实际 ID 经房间首次物化不会被二次 remap。
- **前置**：创建并绑定 room 的存量非空 diagram，关系存储中至少包含 2 张表、每张表至少 1 个字段、1 条连接两表字段的 reference、1 个 area 和 1 个 note；为五类实体及关系四端点使用可逐值识别的非空 ID，且 `doc_json=NULL`。
- **操作**：记录首次物化前的完整持久化快照；通过真实 `append_op` 链路提交一个有效 op，触发首次物化；随后读取物化 `doc_json`，并与物化前快照及首个 op 的预期目标变更逐项对账。
- **断言**：
  1. 首次物化以存量非空快照为基线，既有 table / field / reference / area / note 均存在且各自 ID 与物化前逐值一致；不得出现替换既有 ID 的新 `auto-` ID。
  2. 既有 reference 自身 ID 不变，`start_table_id`、`end_table_id`、`start_field_id`、`end_field_id` 四端点与物化前逐值一致，字段仍分别归属于指定起止表。
  3. areas 与 notes 不丢失，其 ID 和内容保持；除首个 op 明确修改的目标属性外，其他既有实体内容及数组顺序不变。
  4. 首个 op 的效果正常落账，`head.server_rev` 按既有语义递增；再次 GET 物化读与 op log 重放结果一致。
  5. 物化路径不执行导入式重键，不根据名称、顺序或端点相似度猜测映射，也不修复、改写或清理历史悬空关系。
- **OpenLogos reporter**：本用例必须随真实 backend 测试执行，并由现有 OpenLogos reporter 按实际断言结果写入 `logos/resources/verify/test-results.jsonl`。禁止仅登记用例标题、声明式补录或以固定 0ms 结果兜底入账；测试失败或未实际运行时不得记录 PASS。
