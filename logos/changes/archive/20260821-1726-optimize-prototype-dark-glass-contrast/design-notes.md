# Delta — core-01-editor-prototype.html 暗色玻璃态对比度优化

> module: core | proposal: optimize-prototype-dark-glass-contrast

## MODIFIED — `html[data-mode="dark"]` 色板

将暗色模式色板整体向更高对比度、更低灰度漂移方向调整：背景更暗、表面层更不透明、文字更亮、辅助文字更清晰。玻璃态保留 `backdrop-filter`，但增加表面不透明度与边框强度，避免 aurora 背景干扰阅读。

```css
    html[data-mode="dark"] {
      --bg:#050f13;--bg-deep:#08191f;--surface:rgba(16,38,45,.86);--surface-solid:#10262d;
      --surface-soft:rgba(22,48,56,.78);--surface-hover:rgba(34,68,78,.96);--line:rgba(194,232,238,.16);
      --line-strong:rgba(194,232,238,.30);--text:#f2fdfe;--text-2:#b8d2d8;--text-3:#86a3ab;
      --brand:#5ee9dc;--brand-strong:#8cf0e6;--brand-soft:rgba(79,209,197,.18);--accent:#b9a0ff;
      --green:#5ee2aa;--amber:#f5c45c;--red:#ff8a8a;--blue:#7ab8f5;--shadow:0 24px 90px rgba(0,0,0,.45);
      --shadow-soft:0 12px 42px rgba(0,0,0,.32);
    }
```

## ADDED — 暗色模式高对比度覆盖规则

在 `<style>` 末尾、`@media` 查询之前新增以下规则，对玻璃卡片、表单、按钮、标签、画布元素做针对性对比度强化。

```css
    /* 暗色高对比度覆盖（optimize-prototype-dark-glass-contrast） */
    html[data-mode="dark"] .glass {
      background:var(--surface);
      border-color:var(--line-strong);
      box-shadow:var(--shadow-soft);
    }
    html[data-mode="dark"] .auth-story {
      background:linear-gradient(155deg,color-mix(in srgb,var(--brand) 14%,var(--surface)),color-mix(in srgb,var(--accent) 10%,var(--surface)));
      border-color:var(--line-strong);
    }
    html[data-mode="dark"] .feature {
      background:rgba(16,38,45,.72);
      border-color:var(--line-strong);
    }
    html[data-mode="dark"] .feature span { color:var(--text-3) }
    html[data-mode="dark"] .input,
    html[data-mode="dark"] .select,
    html[data-mode="dark"] .textarea {
      background:var(--surface-soft);
      border-color:var(--line-strong);
      color:var(--text);
    }
    html[data-mode="dark"] .input:hover,
    html[data-mode="dark"] .select:hover,
    html[data-mode="dark"] .textarea:hover { border-color:var(--brand) }
    html[data-mode="dark"] .field label,
    html[data-mode="dark"] .field-label { color:var(--text-2) }
    html[data-mode="dark"] .btn {
      background:var(--surface-soft);
      border-color:var(--line-strong);
      color:var(--text-2);
    }
    html[data-mode="dark"] .btn:hover:not(:disabled) {
      background:var(--surface-hover);
      border-color:var(--line-strong);
      color:var(--text);
    }
    html[data-mode="dark"] .btn--primary {
      background:linear-gradient(135deg,var(--brand),var(--brand-strong));
      color:#050f13;
      box-shadow:0 8px 26px color-mix(in srgb,var(--brand) 32%,transparent);
    }
    html[data-mode="dark"] .btn--danger { color:var(--red); border-color:color-mix(in srgb,var(--red) 35%,transparent) }
    html[data-mode="dark"] .auth-tabs { background:var(--surface-soft); border-color:var(--line-strong) }
    html[data-mode="dark"] .auth-tabs button.is-active { background:var(--surface-solid); color:var(--text) }
    html[data-mode="dark"] .demo-note { background:var(--brand-soft); color:var(--text-2) }
    html[data-mode="dark"] .tag { background:var(--surface-soft); border-color:var(--line); color:var(--text-2) }
    html[data-mode="dark"] .tag--brand { background:var(--brand-soft); color:var(--brand-strong); border-color:color-mix(in srgb,var(--brand) 32%,transparent) }
    html[data-mode="dark"] .room-card { background:var(--surface); border-color:var(--line-strong) }
    html[data-mode="dark"] .room-card:hover { border-color:color-mix(in srgb,var(--brand) 40%,transparent) }
    html[data-mode="dark"] .new-room-card { background:rgba(16,38,45,.58); border-color:var(--line-strong) }
    html[data-mode="dark"] .db-table { background:rgba(16,38,45,.94); border-color:var(--line-strong) }
    html[data-mode="dark"] .table-field { border-bottom-color:rgba(194,232,238,.10) }
    html[data-mode="dark"] .field-type { color:var(--text-3) }
    html[data-mode="dark"] .canvas-note { background:rgba(242,184,75,.24); color:var(--text); border-color:color-mix(in srgb,var(--amber) 38%,transparent) }
    html[data-mode="dark"] .canvas-area { border-color:color-mix(in srgb,var(--accent) 55%,transparent); background:color-mix(in srgb,var(--accent) 8%,transparent); color:var(--accent) }
    html[data-mode="dark"] .banner { background:rgba(242,184,75,.18); color:var(--text); border-color:color-mix(in srgb,var(--amber) 40%,transparent) }
    html[data-mode="dark"] .banner--danger { background:rgba(224,88,88,.16); border-color:color-mix(in srgb,var(--red) 40%,transparent) }
    html[data-mode="dark"] .menu-item { color:var(--text-2) }
    html[data-mode="dark"] .menu-item:hover { background:var(--surface-hover); color:var(--text) }
    html[data-mode="dark"] .command-item kbd { color:var(--text-3); border-color:var(--line) }
    html[data-mode="dark"] .activity-item time { color:var(--text-3) }
    html[data-mode="dark"] .diagnostic { background:var(--surface-soft) }
    html[data-mode="dark"] .preview,
    html[data-mode="dark"] .code-area { background:#061217; color:#d8f0f2 }
```

## 第 2 轮调整（艺术总监审核反馈）

> 审核方式：艺术总监代理 Playwright 真渲染（1440×900@2x）+ 73 组计算样式 WCAG 比值实测。verdict：需调整（轻度），2 项 P0。

### ADDED — `.remote-label` 暗色文字覆盖（P0，AA 硬性不达标修复）

远端光标名字牌白字压 `--accent:#b9a0ff` 实色实测仅 2.20:1。在暗色覆盖块末尾新增：

```css
    html[data-mode="dark"] .remote-label{color:#241547}
```

深字 `#241547` 压 `#b9a0ff` 约 7.9:1，与 `.btn--primary`「深字压亮品牌色」手法一致；light 模式保持白字不变。

### MODIFIED — invite 页主标题断行修复（P0，排版缺陷）

`renderInvitePage()` 中 h1 字面量 `"一起把模型\n推向下一版。"` / `"这张邀请卡\n已经失效。"` 的 `\n` 被 HTML 折叠为空格，导致「版。」孤字悬行。改为与 auth 页一致的 `<br>` 断行：

```html
<h1>${state.inviteExpired ? "这张邀请卡<br>已经失效。" : "一起把模型<br>推向下一版。"}</h1>
```

### 暂缓项（并入后续打磨批次，本提案不处理）

1. P1 头像 initials 白字压浅色填充（2.35–2.66:1）：总监定性为装饰性标识、从宽处理。
2. P2 微字号地板（field-type/constraint 9px → 10px）：实测均 ≥5.96:1 达标，属可读性打磨。
3. P2 invite h1 增加品牌色 span 点缀：视觉一致性打磨。

## 设计决策说明

1. **背景压暗**：`--bg` 从 `#07151a` 调整为 `#050f13`，让玻璃面板更突出。
2. **表面层更不透明**：`--surface` 从 `.72` 提升到 `.86`，`--surface-soft` 从 `.62` 提升到 `.78`，降低 aurora 光斑对文字可读性的干扰。
3. **文字提亮并拉大层级**：主文字 `#f2fdfe`、次级 `#b8d2d8`、辅助 `#86a3ab`，三者与深色背景的对比度均超过 WCAG AA 的 4.5:1。
4. **边框增强**：`--line-strong` 从 `.22` 提升到 `.30`，让玻璃卡片和输入框边界在复杂背景下更清晰。
5. **强调色同步提亮**：`--brand` 与功能色略微提亮，确保在暗色表面上依然醒目。
6. **组件覆盖规则**：针对表单、按钮、卡片、标签、画布便签等使用背景色的组件统一应用新的 surface 变量，避免局部硬编码颜色导致对比度失控。
