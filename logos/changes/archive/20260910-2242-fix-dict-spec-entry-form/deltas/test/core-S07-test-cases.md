# Delta — core-S07-test-cases.md（ST-S07-02/03 措辞对齐入口形态）

## MODIFIED — 3. ST 用例

| 用例 ID | 场景 | 预期结果 |
|---|---|---|
| ST-S07-01 | 建字典 → 加 3 字典项 → 绑 2 字段 → 等自动保存 → 刷新重开 | 字典/字典项/绑定完整恢复；revision 递增无 409 |
| ST-S07-02 | 加载本变更前保存的旧图（无 dictionaries） | 正常打开，字典面板空态，编辑/保存后 JSON 新增 `dictionaries` 键 |
| ST-S07-03 | Viewer 角色打开 | 字典面板可浏览、导出可用；新建/编辑/删除/绑定控件全部 disabled |
| ST-S07-04 | 导出 → 下载 `.md` → 与当前模型对照 | 文件名 `data-dictionary-{name}.md`；内容与模型一致 |
| ST-S07-05 | 删除被引用字典 → undo | 字典与字段绑定一次 undo 全部恢复 |
