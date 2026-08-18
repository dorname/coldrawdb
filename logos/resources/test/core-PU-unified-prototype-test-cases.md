# core-PU 单文件一体化原型验收矩阵

> module: core | proposal: improve-unified-collab-prototype | type: prototype acceptance

## 1. 范围与说明

本矩阵验证静态产品原型，不调用生产 API，因此不产生 API 编排 JSON，也不写 OpenLogos 运行时测试 reporter。生产 S01/S02/S04/S05 编排测试保持不变。浏览器验收使用本地 `file://` 或静态服务器打开 delta HTML，并执行可见交互与只读诊断。

## 2. 用例

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

## 3. PU-AC 追溯

| 验收标准 | 覆盖用例 |
|---|---|
| PU-AC-01 单文件 | ST-PU-01 |
| PU-AC-02 连续主链 | ST-PU-02、04、05、06、10、12、08 |
| PU-AC-03 编辑完整性 | ST-PU-05～08 |
| PU-AC-04 协作完整性 | ST-PU-10～16 |
| PU-AC-05 浮层完整性 | ST-PU-08、09、17 |
| PU-AC-06 视觉质量 | ST-PU-01、09、17、18 |
| PU-AC-07 可访问性 | ST-PU-03、09、17、18 |
| PU-AC-08 可诊断性 | `window.__cdbPrototype.diagnose()` 全部检查项通过 |

## 4. 自动诊断契约

主原型暴露只读对象 `window.__cdbPrototype`：

- `diagnose()`：返回 `{ pass, checks[] }`，检查外部依赖、关键 testid、重复 ID、可见 overlay、store 不变量。
- `snapshot()`：返回深拷贝的当前 view、role、connection、serverRev、pendingOps、tables、relations 与 openLayer。
- `demo(action)`：只接受白名单动作 `remote-cursor`、`remote-table`、`disconnect`、`reconnect`、`reconnect-fail`、`viewer`，供验收驱动界面。

诊断接口不得返回密码、Token 或可变 store 引用。

## 5. 本次 delta 验证结果（2026-08-18）

| 验证组 | 结果 | 实测摘要 |
|---|---|---|
| 静态单文件 | PASS | JavaScript 语法有效；无外部 CSS/JS/字体/图片依赖；52 个唯一 `data-testid` 文本锚点 |
| 编辑与协作主链 | PASS | 2→6 张表、1→2 条关系；server revision 同步至 48；断线队列恢复后为 0；浏览器错误 0 |
| 房间与浮层链 | PASS | 注册校验、创建房间、Viewer 邀请、成员改权/移除、DBML、命令导出全部通过 |
| 内置诊断 | PASS | 单文件依赖、DOM ID、角色、revision、队列、数据模型、浮层状态共 7 项通过 |
| 窄屏布局 | PASS | 720px 视口下 Canvas/ToolRail/Drawer 宽度均为 708px；页面 `scrollWidth=720px`，无横向溢出 |

浏览器验证期间发现并修复两项问题：AppBar Popover 被 Inspector 截获点击；移动端关闭 Inspector 后桌面三列规则覆盖单列布局。修复后已重跑对应完整链路。
