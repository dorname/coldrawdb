# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — §4.4 提交行为改为本地解析合并进当前画布（不再新建游离 diagram / 不再跳转）+ 成功/失败反馈口径
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — AppBar 房间徽章名称缩略 + 「← 空间」显性返回入口
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — AppBar 布局补返回入口节点
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型 AppBar 对齐：名称缩略 + 返回入口（importModel 本地合并已一致）
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — ST-PC-01 改写为合并语义 + 新增导入合并 UT 用例
- [x] 产出 delta 文件到 `deltas/test/core-S04-test-cases.md` — ST-S04-UI-10 返回入口 + 名称缩略 e2e/UT 锚点用例

## [code] 代码实现
- [x] `frontend-rs/src/editor_panels.rs`：ImportDrawer 提交改为本地合并（新增 `merge_import_into_store` 纯函数：避让布局 + 追加表/关系 + 选中导入结果），移除提交路径中的 bridge import 调用与跳转
- [x] `frontend-rs/src/editor_panels.rs` + `styles.css`：AppBar 图表标题 / room-badge 名称缩略（max-width + ellipsis + title）；room-badge 左侧新增「← 空间」返回按钮
- [x] 编写/更新 UT 与 e2e 对齐代码，含 OpenLogos reporter 写入（UT-PC-09 / UT-S04-UI-12 / ST-PC-01 / ST-S04-UI-12）
