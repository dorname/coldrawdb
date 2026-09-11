# Delta: core-01d-import-export.md

> 提案：fix-appbar-roomname-back-and-import-merge

## MODIFIED — §4.4 提交行为

**merge 时替换 §4.4 提交行为整节（1~6 条）为**：

1. 校验：内容非空；SQL 需选引擎；JSON 需合法
2. **SQL 导入解析（relation-inspector-and-ddl-io：列级 DDL 解析）**——`parse_sql_import_tables` 口径不变（列定义/表级约束/标识符规范化/V1 边界，见前述条款）
3. **本地合并进当前画布（fix-appbar-roomname-back-and-import-merge，替换原 bridge 新建游离 diagram + 跳转语义）**：
   - 编辑器上下文（房间编辑器 / 独立编辑器）下，解析结果经 `merge_import_into_store(store, tables, references)` 纯函数合并：新表**避让现有布局**排布（在现有内容右侧/下方网格落位，不覆盖既有表），references 一并追加
   - 合并后复用既有保存链路（`dirty` + `schedule_save` → PUT 当前 diagram），选中导入结果表并滚动可见
   - **不再**调用 `POST /api/v1/bridge/import/local`、**不再**创建游离 diagram、**不再** `window.location` 跳转；bridge 端点保留（契约不动），导入日志面板仅作历史记录展示
4. 成功 → Toast「已导入 N 张表 / M 条关系」+ 关闭抽屉
5. 失败（解析 Err / 保存失败）→ 抽屉内 inline 错误 + ErrorToast
6. 按钮文案：**导入到画布**；提交中 disabled + `导入中...`

> 兼容性说明：原第 3 条「`POST /api/v1/bridge/import/local`」与第 4 条「跳转 `/editor/{diagramId}`」废弃。原语义产生的游离 diagram 用户无法从房间列表触达，是「导入没有成功」反馈的根因。
