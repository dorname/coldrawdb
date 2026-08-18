# Delta — core-PU-unified-prototype-test-cases.md（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 1. 范围与说明

本矩阵验证静态产品原型，不调用生产 API，因此不产生 API 编排 JSON。ST-PU-01～ST-PU-19 全部由 Playwright 自动回归，并向 OpenLogos 结果文件写入独立 reporter。脚本从仓库正式测试目录运行，不依赖 `.understand-anything/tmp`。人工视觉复核可保留，但不再是功能完整性的唯一证据。生产 S01/S02/S04/S05 编排测试保持不变。

## MODIFIED — 2. 用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-PU-01 | 断网 | 直接打开主 HTML | 无资源错误；登录页样式、SVG 图标和交互完整 |
| ST-PU-02 | 登录页 | 输入合法邮箱和 8 位密码并提交 | loading 后进入房间列表，出现欢迎 Toast |
| ST-PU-03 | 注册页 | 输入不一致密码后修正并提交 | 先显示字段错误；修正后进入房间列表 |
| ST-PU-04 | 房间列表 | 创建「数据模型评审」并提交 | 新房间出现并进入编辑器，AppBar 显示房间名 |
| ST-PU-05 | Editor 角色 | 新建表、添加字段、修改字段类型并拖拽表 | Canvas、Inspector、Activity 与保存状态一致 |
| ST-PU-06 | 已有两张表 | 开启关系工具，依次选择源/目标字段 | 画布出现关系线，关系计数与 Activity 更新 |
| ST-PU-07 | 已发生编辑 | 点击撤销、重做并等待自动保存 | 模型正确回退/恢复；状态从未保存→保存中→已保存，rev +1 |
| ST-PU-08 | 编辑器 | 打开导入，输入 SQL 并执行；再打开导出 | 新表生成；SQL/DBML/JSON 预览随模型更新 |
| ST-PU-09 | 编辑器 | 打开代码视图和命令面板，执行主题切换 | 浮层正常关闭；主题变化；无遗留 overlay |
| ST-PU-10 | Owner | 生成 Viewer 邀请，打开预览并接受 | 邀请 URL 可见；接受后以 Viewer 进入同一房间 |
| ST-PU-11 | Owner | 打开成员面板，将 Bob 改 Viewer 后移除 | 权限标签即时更新；确认后成员消失 |
| ST-PU-12 | 协作连接正常 | 模拟 Alice 光标与创建 `orders` | 光标移动；表、Activity、server rev 同步更新 |
| ST-PU-13 | 连接正常 | 模拟断线后本地修改，再恢复 | 队列计数增加；恢复后清零，出现同步成功反馈 |
| ST-PU-14 | 重连失败 | 选择仅本地编辑 | StatusBar 显示离线/409 风险；本地编辑仍可进行 |
| ST-PU-15 | Viewer | 点击新建表、修改字段、邀请成员 | 写操作被阻止并提示只读；远端操作仍可显示 |
| ST-PU-16 | 编辑器 | 模拟 Token 过期 | 会话指示进入续期并恢复，当前编辑状态不丢失 |
| ST-PU-17 | 桌面与 720px 视口 | 依次打开 Inspector、成员、IO 与模态 | 关键操作可达；内容无不可恢复遮挡 |
| ST-PU-18 | 系统 reduced-motion | 触发光标、Toast、抽屉 | 无非必要动画；信息仍完整可感知 |
| ST-PU-19 | 协作编辑器 | 分别新增表、完成关系、批量导入并观察至自动保存结束 | 每次编辑最多重建 1 次 `#app` 主视图；保存状态阶段不替换 Canvas DOM；同视图不重复播放入场动画；revision 正常递增 |

## ADDED — 6. 自动化执行约束与审计基线

- 每个用例使用独立或显式重置的 browser context，避免状态串扰。
- 失败必须记录步骤、可见锚点和脱敏错误；截图写测试产物目录，不嵌入 reporter。
- 2026-08-18 完整性审计结果：ST-PU-01～ST-PU-19 共 19/19 PASS。
- 历史 S04/S05 独立原型的未绑定控件不纳入现行验收；资源索引必须明确其历史属性。
