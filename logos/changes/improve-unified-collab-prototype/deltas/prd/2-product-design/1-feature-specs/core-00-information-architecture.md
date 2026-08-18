# Delta — core-00-information-architecture.md（修改）

> module: core | proposal: improve-unified-collab-prototype | 2026-08-18
> merge 时将本节追加到正式规格。

## ADDED — §10 S01～S05 唯一主原型

### 10.1 唯一入口

`core-01-editor-prototype.html` 是产品设计评审的唯一主原型。它以一个独立 HTML 文件承载 S01～S05 的页面、样式、SVG 图标、演示数据与交互逻辑，断网直接打开即可运行。

`core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html` 仅保留为对应阶段的历史设计参考；新增功能与视觉变更不再分别维护到这些文件，避免状态模型和设计语言再次分叉。

### 10.2 单文件约束

| 资源 | 约束 |
|---|---|
| CSS | 仅允许主 HTML 内的 `<style>`；不得引用 `core-00-prototype-shared.css` |
| JavaScript | 仅允许主 HTML 内的 `<script>`；不得引用外部脚本或模块 |
| 图标 | 使用内联 SVG symbol；不得用 emoji 充当功能图标 |
| 字体与图片 | 使用系统字体与 CSS 图形；不得依赖 CDN 或远程 URL |
| 数据与网络 | 使用确定性模拟数据；不得发起真实 HTTP、WebSocket 或遥测请求 |
| 状态 | 一个轻量 store 统一驱动路由、编辑命令、权限、浮层、保存与协作状态 |

### 10.3 页面状态流

```text
[登录/注册]
     │ 登录成功 / 演示进入
     ▼
[房间与最近项目]
     ├── 创建房间 ───────────────┐
     ├── 打开已有房间 ───────────┤
     └── 接受邀请 ───────────────┤
                                  ▼
                        [协作 ER 编辑器]
                         ├── 编辑/关系/撤销/保存
                         ├── 导入/导出/代码/分享
                         ├── 成员/角色/邀请
                         └── OT/presence/重连模拟
```

所有视图必须在同一 DOM 应用壳内切换，不刷新页面。浏览器地址仅作为原型状态提示，不要求真实路由服务。

### 10.4 编辑器层级

| 层级 | 内容 | 玻璃态规则 |
|---|---|---|
| L0 | 渐变背景、画布网格 | 不使用 blur，保证性能与坐标清晰 |
| L1 | 表、关系、区域、便签、远端选区 | 半透明实体卡片，选中态保持实线高对比 |
| L2 | AppBar、ToolRail、StatusBar | `backdrop-filter` + 细描边；提供不支持 blur 时的实色回退 |
| L3 | Inspector、IO/成员/Activity SideSheet | 更高不透明度，确保表单可读性 |
| L4 | Tooltip、Popover、Command Palette | 阴影与描边共同区分，不只依赖透明度 |
| L5 | Modal + overlay | 焦点与 Escape 行为清晰；关闭后 overlay 必须从 DOM/交互树退出 |
| L6 | Toast、连接 Banner | 不遮挡主操作，状态色同时配文字和图标 |

### 10.5 响应式行为

- `≥ 1180px`：ToolRail + Canvas + Inspector 同屏，协作控制台以右侧 SideSheet 展示。
- `760px～1179px`：Inspector 默认折叠为抽屉，AppBar 次要动作收入更多菜单。
- `< 760px`：保留画布与底部快捷操作；ToolRail 横向化，Inspector/成员/IO 均以全高抽屉展示。
- 任何宽度下，登录、进入房间、创建表、邀请成员、角色切换、断线恢复和导出均必须可达。

### 10.6 原型与生产边界

主原型用于验证信息架构、状态反馈和交互连贯性，不代表 S03～S05 的生产网络能力已实现。界面中的登录、HTTP、WebSocket、OT transform、剪贴板和文件下载均由本地模拟器驱动；所有模拟入口必须标注「演示」或「模拟」。
