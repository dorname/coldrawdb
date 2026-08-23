# core-PU 单文件一体化原型验收矩阵

> module: core | proposal: improve-unified-collab-prototype | type: prototype acceptance

## 1. 范围与说明

> module: core | proposal: implement-unified-prototype-spec-parity | type: prototype acceptance

唯一现行主原型：`core-01-editor-prototype.html`。本矩阵验证 **auth → rooms → invite → room-editor** 完整交互与视觉基线；历史 `core-03/04/05-*-prototype.html` 不纳入现行验收。

静态原型不调用生产 API。生产语义对齐见 `core-V2-production-frontend-test-cases.md`。ST-PU-01～21 继续作为主原型回归；ST-PU-22～26 由本提案落实自动化（或带缺口说明的显式 skip）。

实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）。生产对齐完成前仍为「后端已实现；生产前端部分接入」。

## 2. 用例

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

## 3. PU-AC 追溯

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
| 重复刷新回归 | PASS | 新增表、创建关系、批量导入均仅 1 次主视图重建和 1 个渲染批次；保存阶段 Canvas DOM 保持；revision 12→15；ST-PU-19 reporter 为 PASS |

浏览器验证期间发现并修复两项问题：AppBar Popover 被 Inspector 截获点击；移动端关闭 Inspector 后桌面三列规则覆盖单列布局。修复后已重跑对应完整链路。

## 6. 自动化执行约束与审计基线

- 每个用例使用独立或显式重置的 browser context，避免状态串扰。
- 失败必须记录步骤、可见锚点和脱敏错误；截图写测试产物目录，不嵌入 reporter。
- 2026-08-18 完整性审计结果：ST-PU-01～ST-PU-19 共 19/19 PASS。
- 历史 S04/S05 独立原型的未绑定控件不纳入现行验收；资源索引必须明确其历史属性。

## 关键 `data-testid` 锚点合同（与主原型一致）

| 页面 / 区域 | 关键 `data-testid` |
|---|---|
| auth | `login-form` / `register-form` |
| rooms | `rooms-list-page` / `room-list` / `btn-create-room` |
| invite | `invite-accept-page` / `btn-accept-invite` |
| room-editor 壳 | `room-editor-page` / `app-bar` / `tool-rail` / `editor-canvas` / `inspector` / `status-bar` |
| 保存 / 协作 | `save-state` / `revision-display` / `ws-status` / `ot-rev` / `room-presence` / `reconnect-banner` |
| 浮层 / IO | `tool-search` / `code-view-modal` / `btn-more-menu` / `btn-import` / `btn-export` / `room-members-panel` |
| 关系 / 主题 | `rel-rubber-band` / `btn-theme-toggle` |

## 统一原型对齐用例增量与修订索引

| ID | 变更 | 说明 | 状态 |
|---|---|---|---|
| ST-PU-01～04 | MODIFIED | 明确页面流 auth→rooms→room-editor；断言关键 testid 存在 | 既有回归 |
| ST-PU-05 | MODIFIED | 原型松手网格为演示 `GRID=12`；**生产合同**为 `GRID_SIZE=20`（见 CR）；拖动中关系 `path[d]` 跟手 | 既有回归 |
| ST-PU-06 / 20 | 保留 | 点击两点与拖线（`DRAG_THRESHOLD=4`）双路径 | 既有回归 |
| ST-PU-09 | MODIFIED | 主题切换后无残留 overlay；暗色 token 生效 | 既有回归；D 批对照生产 |
| ST-PU-17 | MODIFIED | 桌面 + ≤720px：Inspector/成员/IO/模态关键操作可达，无横向溢出 | 既有回归；D 批对照生产 |
| ST-PU-18 | 保留 | `prefers-reduced-motion` | 既有回归 |
| ST-PU-22 | ADDED | 未登录打开主原型默认入口 → 仅 auth；不出现私有 `room-list` 数据 | 本提案 A 批实现 |
| ST-PU-23 | ADDED | 邀请失效态：`invite-accept-page` 无加入按钮 | 本提案 B 批实现 |
| ST-PU-24 | ADDED | room-editor 可见 `ws-status`、`ot-rev`、`room-presence`（演示值须标「演示」） | 本提案 C 批实现 |

## 视觉 / 主题 / 响应式基线

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-PU-22 | 冷启动 | 打开主 HTML | 落在 auth；关键 login/register testid 存在 | 本提案 A 批实现 |
| ST-PU-23 | 邀请失效演示 | 打开 invite 失效路径 | 说明文案可见；无 accept 主按钮 | 本提案 B 批实现 |
| ST-PU-24 | 已进入 room-editor | 观察 StatusBar / AppBar | `ws-status`、`ot-rev`、`room-presence` 可见；演示控件标注演示 | 本提案 C 批实现 |
| ST-PU-25 | 编辑器 | 切换主题 | `data-mode` 切换；画布/壳层对比度可读；无半透明残留层 | 本提案 D 批实现 |
| ST-PU-26 | 720px | 开关 Inspector 与 IO | 以抽屉呈现；可关闭；不锁定背景滚动（或关闭后恢复） | 本提案 D 批实现 |

## PU-AC 追溯补充

| 验收标准 | 覆盖用例 |
|---|---|
| PU-AC-09 页面流四态 | ST-PU-22～24、02、04、10 |
| PU-AC-10 主题与响应式 | ST-PU-09、17、18、25、26 |

## 统一原型验收边界声明

- 主原型可演示 ≠ 生产前端逐项完成。
- 生产对齐由本提案 `implement-unified-prototype-spec-parity` 执行；完成前状态仍为「后端已实现；生产前端部分接入」。
- ST-PU-22～26 必须写入 reporter；不得静默缺失。
