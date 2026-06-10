# Delta — core-04-side-panel-tabs.md
# 模块：core | 提案：add-frontend-completeness
# 路径：`logos/changes/add-frontend-completeness/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
# 对齐参考源：tasks.md B2「6 Tab + 搜索筛选」+ 新建 `core-SP-side-panel-test-cases.md`

## MODIFIED — §11 测试用例 ID 索引（追加 2 个新 ID）

> **追加** 到主文档 `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
> §11 测试用例 ID 索引表格的末尾。merge 时在表格最后追加 2 行（保持原 9 行不变）。

| TC ID | 描述 |
|---|---|
| UT-SP-01 | 单击 Tables Tab 表项 → 画布高亮 + 滚动 |
| UT-SP-02 | 搜索 "user" → 列表过滤只含 user* |
| UT-SP-03 | 双击枚举名 → 进入重命名 |
| UT-SP-04 | 引用检查：删除被引用的枚举 → 弹确认 |
| UT-SP-05 | Issues Tab：表名重复 → 错误项出现 |
| UT-SP-06 | Issues Tab：单击错误 → 跳转 + 画布闪烁 |
| UT-SP-07 | DBML Editor：编辑后 Apply → 解析成功 → Diagram 更新 |
| UT-SP-08 | DBML Editor：编辑非法 DBML → 错误消息 + 不应用 |
| ST-SP-01 | 端到端：编辑 5 表 → Issues Tab 显示 0 error |
| **UT-SP-09** | **6 业务 Tab 切换**（点击 Tab A→B→C，验证激活态 + 内容区切换）— B2 范围 |
| **UT-SP-10** | **全局搜索跨 Tab 过滤**（spec §10，搜索框过滤 Tables/Areas/Enums 等多 Tab 列表）— B2 范围 |

> **详细定义** 见 `logos/resources/test/core-SP-side-panel-test-cases.md`（与本 delta 同步新增）。
> 修正 tasks.md B2 覆盖说明（删去错误引用的 UT-S02-03/04，替换为 UT-SP-09/10）。
