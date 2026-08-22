# 设计说明 — polish-dark-mode-micro-contrast

> module: core | proposal: polish-dark-mode-micro-contrast
> 本文档记录每轮审核调整，实现载体为全量原型 delta `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

## 第 1 轮调整（2026-08-21）

### 1. ADDED — `.avatar` 暗色 initials 深色字（P1）

上一变更审核实测：白字压 `#4fd1c5` / `#aa8cff` / `#68aef2` / `#f2b84b` 仅 1.7–2.66:1。在暗色覆盖块 `.remote-label` 之后新增：

```css
    html[data-mode="dark"] .avatar{color:rgba(20,16,40,.85)}
```

深字对四档成员色与品牌渐变（#5ee9dc→#b9a0ff）均 ≥ 4.5:1；light 模式渐变填充为深色（#1e8393→#7c5ce7），保持白字不动。

### 2. MODIFIED — 微字号地板 10px（P2）

基础规则（两种模式共用）：

- `.field-type`：`font-size:9px → 10px`
- `.constraint`：`font-size:9px → 10px`

### 3. MODIFIED — invite h1 品牌色强调（P2）

`renderInvitePage()` h1 第二行包 `<span>`，复用 `.hero-copy h1 span{color:var(--brand)}`：

```html
<h1>${state.inviteExpired ? "这张邀请卡<br><span>已经失效。</span>" : "一起把模型<br><span>推向下一版。</span>"}</h1>
```

### 待总监裁决项（本轮未动）

1. `.field-key`（PK 标识，9px/900 amber）：符号性标记，是否一并升 10px。
2. 其余 9px 辅助位：`activity-item time`、`command-item kbd`、`menu-item .shortcut`——均为时间戳/快捷键提示类，是否纳入地板。
3. light 模式下内联成员色填充（#4fd1c5 等浅色 tint）白字 initials 同样不达标（模式无关缺陷），是否用 `.avatar[style*="background"]` 选择器在两种模式统一修复，或另立提案处理。

## 第 1 轮审核结论与裁决落实（2026-08-22）

艺术总监代理真渲染复审（24 张截图目检 + report.json 实测矩阵）：**审核通过**。三项打磨全部达标——initials × 成员色矩阵最低 5.53:1（A on #aa8cff）、field-type/constraint 双模式恰为 10px 无截断、invite span dark 13.07:1 / light 4.03:1（大字阈值 3:1 达标）；八组视图零溢出、零 JS 错误。

裁决落实：

1. **A `.field-key` 维持 9px**：实测 9.82:1 余量大，徽标化标识与 10px 数据文本形成层级差，不并入地板。
2. **B 其余 9px 辅助位维持从宽**：activity time 6.08 / command kbd 6.09 / menu shortcut 5.89 均达标。附带发现命令面板激活态 kbd 4.11:1 边缘值 → 下一打磨批（`--text-3 → --text-2`）。
3. **C light 内联浅色填充 initials 在本提案收口**（已落实）：新增 `html:not([data-mode="dark"]) .avatar[style*="background"]{color:rgba(20,16,40,.85)}`——选择器限定 light 侧且仅命中内联浅色填充，light 渐变深底头像（无 inline background）保持白字不误伤。总监附带发现：light 渐变头像白字在 #1e8393 端 4.45:1 边缘值（差 0.05），留待后续批次微调渐变起点（如 #177382）。

## 第 2 轮定向复测结论（2026-08-22）

light 侧规则复测 **审核通过**：内联填充头像 9 组合实测 5.53–7.76:1 全部达标；渐变深底头像（rooms/appbar/invite option-card）计算 color 保持 `rgb(255,255,255)` 零误伤；dark 通道 12 实例零回归；全程 0 JS 错误。

两轮累计实测覆盖：dark initials 矩阵 9 组合 + light 内联矩阵 9 组合 + 10px 地板双模式 30 实例 + invite span 双模式双态 + 四视图双模式回归。

**遗留（另项处理，不阻断本提案）**：light 渐变头像白字 4.45:1 边缘值（微调渐变起点收口）；命令面板激活态 kbd 4.11:1（`--text-3 → --text-2`）。
