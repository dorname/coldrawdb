# Delta — core-PU-unified-prototype-test-cases.md

> 模块：core | 提案：optimize-canvas-connect-and-drag

## MODIFIED — 2. 用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-PU-01 | 断网 | 直接打开主 HTML | 无资源错误；登录页样式、SVG 图标和交互完整 |
| ST-PU-02 | 登录页 | 输入合法邮箱和 8 位密码并提交 | loading 后进入房间列表，出现欢迎 Toast |
| ST-PU-03 | 注册页 | 输入不一致密码后修正并提交 | 先显示字段错误；修正后进入房间列表 |
| ST-PU-04 | 房间列表 | 创建「数据模型评审」并提交 | 新房间出现并进入编辑器，AppBar 显示房间名 |
| ST-PU-05 | Editor 角色 | 新建表、添加字段、修改字段类型并拖拽表 | Canvas、Inspector、Activity 与保存状态一致；**拖动过程中**已有关系的 SVG `path[d]` 随表位置更新；松手后表 `x/y` 为 12 的倍数 |
| ST-PU-06 | 已有两张表 | 开启关系工具，**依次点击**源/目标字段 | 画布出现关系线，关系计数与 Activity 更新（点击两点路径必须保留） |
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
| ST-PU-20 | 已有两张表、关系工具开启 | 从源字段 pointerdown 拖到目标字段后松开（位移 ≥ 4px） | `rel-rubber-band` 在拖动中可见；松手后 `relations.length + 1`；拖动过程不重建 `#app` |
| ST-PU-21 | 编辑器已有 users→posts 关系 | 拖动 `users` 表头至少 40px，在 pointerup **之前**读取关系 path | `path[data-relation="rel-users-posts"]` 的 `d` 已相对按下时变化；松手后 `users.x`、`users.y` 均为 12 的倍数 |

## MODIFIED — 3. PU-AC 追溯

| 验收标准 | 覆盖用例 |
|---|---|
| PU-AC-01 单文件 | ST-PU-01 |
| PU-AC-02 连续主链 | ST-PU-02、04、05、06、10、12、08 |
| PU-AC-03 编辑完整性 | ST-PU-05～08、ST-PU-19、ST-PU-20、ST-PU-21 |
| PU-AC-04 协作完整性 | ST-PU-10～16 |
| PU-AC-05 浮层完整性 | ST-PU-08、09、17 |
| PU-AC-06 视觉质量 | ST-PU-01、09、17、18 |
| PU-AC-07 可访问性 | ST-PU-03、09、17、18 |
| PU-AC-08 可诊断性 | `window.__cdbPrototype.diagnose()` 全部检查项通过；ST-PU-19 验证渲染次数不变量 |
