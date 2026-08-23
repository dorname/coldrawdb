# Delta — core-09-core-components.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线

唯一现行组件行为与视觉基线：`core-01-editor-prototype.html`。本 delta 对齐 Button / Popover / Modal / SideSheet(Drawer) / Tag / Banner / Toast；其余（Dropdown 菜单项、Tooltip、Collapse）沿用主原型等价行为。

统一约束：视觉引用 `core-07` token；键盘可达；遮罩/浮层关闭后 **DOM 与交互层不得残留**（无透明拦截、无遗留 `pointer-events`、无僵尸 overlay）。

## MODIFIED — 2. Button

对齐主原型 `.btn` 族：

| 变体 | 行为摘要 |
|---|---|
| 默认 / soft | `--surface-soft` 底 + `--line` 边；hover 上浮 1px |
| `--primary` | brand 渐变；浅色模式白字，暗色模式近黑字 `#050f13` |
| `--danger` | `--red` 字色与淡红边 |
| `--ghost` / `--icon` | 透明底；图标按钮方形 |
| `--sm` | 紧凑高度（画布工具、Banner 动作） |

- `disabled`：不可点、无 hover 位移
- `loading` / `aria-busy`：Auth 提交显示 spinner +「正在验证…」
- 过渡：`.18s var(--ease)`；active `scale(.98)`

## MODIFIED — 6. Popover

对齐 `.popover`（z≈46）：

- 触发：AppBar 更多菜单、用户菜单、rooms 用户菜单（`state.layer` 切换）
- 视觉：玻璃态、宽约 `230px`、圆角 `14px`；`.menu-item` 高 `39px`，可带 shortcut
- 关闭：再次点击触发器、选择菜单项、打开 Modal/Drawer、Esc（与全局 layer 清理一致）
- 关闭后菜单节点不渲染，不得留下可点击幽灵层

## MODIFIED — 3. Modal

对齐 `.overlay`（z=50）+ `.modal`：

| 属性 | 事实 |
|---|---|
| 遮罩 | `rgba(2,12,16,.54)` + `blur(8px)`；`data-overlay` |
| 宽度 | 常规 `min(520px,100%)`；宽版 `.modal--wide` → `min(720px,100%)` |
| 结构 | `modal-head` / `modal-body` / `modal-foot` |
| a11y | `role="dialog"` `aria-modal="true"` + `aria-labelledby` |
| 关闭 | 关闭按钮 `close-layer`；点击遮罩（`event.target` 为 overlay）；表单取消 |
| 入场 | `fade` `.18s` + `modal-in` `.22s` |

**遮罩关闭后不残留（强制）**：

1. `layer` 置空后整段 overlay 从渲染树移除（主原型：条件渲染返回空串）。
2. 不得使用 `visibility:hidden` / `opacity:0` 却保留 `position:fixed; inset:0` 拦截点击。
3. 打开 Drawer / Code / Command 时与 Modal 互斥；关闭路径必须清空对应状态。
4. 生产实现若使用延迟卸载动画，动画结束后必须真正卸载；超时失败亦须强制移除。

典型 Modal：`modal-create-room`、`modal-invite`、分享、偏好设置、删除确认、原型诊断。

## MODIFIED — 9. SideSheet

对齐 `.drawer`（z=35，宽 `min(420px, calc(100% - 72px))`）：

| Drawer | `data-testid` |
|---|---|
| 成员 | `room-members-panel` |
| 活动 | `activity-feed` |
| 导入 | `import-drawer` |
| 导出 | `export-drawer` |

- 入场：`drawer-in` `.24s`（translateX 25px → 0）
- 关闭：`close-drawer` / 打开 Modal 时 `drawer=null`
- ≤760px：全宽 + 圆角收紧
- 关闭后同样不得残留遮挡画布的透明层（Drawer 无全屏 mask 时，也不得留下不可见 hit-area）

## ADDED — Tag

对齐 `.tag` / `.tag--brand` / `.tag--warn`：

- 胶囊、11px 粗体、可内嵌状态点
- 用途：角色、在线人数、待同步计数、代码「实时生成」、房间徽章

## ADDED — Banner

对齐画布顶部 `.banner`（z=12）：

| 态 | 样式 | 用途 |
|---|---|---|
| 默认（警告） | amber soft | 重连中 / 同步中（`reconnect-banner`） |
| `--danger` | red soft | 离线 / 仅本地编辑 |

含文案 + 可选 `.banner-actions` 按钮；连接恢复为 `connected` 时 Banner **整段卸载**。

## ADDED — Toast

对齐 `#toast-region.toast-region`（z=60，右上）：

- 结构：图标列 + 标题/正文 + 关闭按钮；玻璃态；`toast-in` `.25s`
- 错误：`.is-error` + info 图标
- 区域：`aria-live="polite"`；约 3600ms 自动消失；可手动 `dismiss-toast`
- `pointer-events:none` 在 region，单项 toast `pointer-events:auto`，避免挡住整页

## REMOVED / 降级

- 以 SemiUI Modal/SideSheet API 像素对齐作为唯一验收 → 改为主原型行为与层级
- Warning 变体按钮若与主原型 `--danger` 冲突，以 danger 语义为准
- Tooltip 黑底白字强制 → 主原型 ToolRail tip 为表面色玻璃 tip（可保留生产增强，但不得与暗色对比度冲突）

## MODIFIED — 11. 验收约束

- Button / Popover / Modal / Drawer / Tag / Banner / Toast 均可在主原型对应路径演示
- 关闭 Modal/Command/Popover/Drawer/Banner/Toast 后，无残留 fixed 遮罩或拦截层（可用诊断「浮层状态」或 DOM 断言）
- 组件颜色/阴影仅通过 token；硬编码禁止规则见 `core-07`
