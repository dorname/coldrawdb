# Delta — core-09-core-components.md（Splitter 分隔条）

> 提案：fix-canvas-zoom-invite-comment-resize（问题4：布局面板可拖动调整）
> 目标主文档：`logos/resources/prd/2-product-design/1-feature-specs/core-09-core-components.md`

## ADDED — Splitter 分隔条

### 13.4 Splitter 分隔条

> 提案：fix-canvas-zoom-invite-comment-resize（问题4：布局面板可拖动调整）

#### 13.4.1 适用范围

布局面板的宽度拖动组件，落地两处：

| 实例 | 位置 | 拖动目标 | 默认宽度 | localStorage key |
|---|---|---|---|---|
| `splitter-inspector` | `.cdb-main` 第 2/3 列之间（Inspector 左缘） | CSS 变量 `--cdb-inspector-w` | 330px | `cdb.inspector.width` |
| `splitter-list-tree` | ListView `.cdb-list-view-body` 第 1/2 列之间（树列右缘） | 树列内联 `grid-template-columns` | 230px | `cdb.list-tree.width` |

#### 13.4.2 交互

1. **拖动**：`pointerdown` 在分隔条上时 `set_pointer_capture`，`pointermove` 实时改写宽度，`pointerup` / `pointercancel` 结束并持久化。
2. **hover 态**：分隔条宽 6px，默认透明（叠在两列间隙上）；hover 或拖动中显示品牌色高亮（`--cdb-brand` 或 `--cdb-color-primary`，2px 内缘指示条）。
3. **双击复位**：双击分隔条恢复默认宽度（330 / 230），并写入 localStorage。
4. **键盘可达**：分隔条为 `role="separator"`，`aria-orientation="vertical"`，`aria-valuenow` 反映当前宽度；方向键 ±16px（Shift ±64px），Home/End 跳 min/max，Esc 复位默认。

#### 13.4.3 约束

1. **宽度钳制**：Inspector 240–520px，且不超过 `视口宽度 - 200px`；树列 160–420px。
2. **生效断点**：仅 ≥1024px 视口渲染/可拖动；≤720px（Inspector 为绝对覆盖层）与 721–1023px（全屏遮罩态）下分隔条 `display:none`，宽度变量不生效。
3. **拖动期禁过渡**：`.cdb-main` 存在 `transition: grid-template-columns 160ms`，拖动期间在容器上加临时 class（如 `is-resizing`）将 `transition` 置 `none`，`pointerup` 后移除恢复过渡。
4. **Inspector 折叠态**：`inspector_open=false`（第 3 列归零）时分隔条不可见（列宽为 0 无间隙），重开后恢复持久化宽度。

#### 13.4.4 实现约定（CSS 变量直写，绕过响应式）

1. 宽度存 `RwSignal<u32>` 仅作初始值与持久化来源；**拖动期间不更新 signal**，直接在 `pointermove` 回调中调用 `document.document_element().style().set_property("--cdb-inspector-w", format!("{}px", w))`，避免 Leptos 每帧重渲染。
2. `pointerup` 时一次性写回 signal 并持久化 localStorage。
3. ListView 树列同法：拖动期间直写树容器 `style` 的 `grid-template-columns: {w}px minmax(0,1fr)`。
4. 初始化（mount）时读 localStorage，合法值（数字且在钳制范围内）直写 CSS 变量，非法/缺失用默认值。

#### 13.4.5 testid 与验收锚点

| 元素 | `data-testid` |
|---|---|
| Inspector 分隔条 | `splitter-inspector` |
| ListView 树列分隔条 | `splitter-list-tree` |

- 拖动期间每次 `pointermove` 后 `--cdb-inspector-w` 即时变化（无 160ms 过渡延迟）。
- 钳制：拖过 520 / 240 边界时变量停在边界值。
- 刷新后宽度恢复（见 UT-PU-23）。
- 画布无需额外处理：grid 重排 + RAF 每帧比对 `clientWidth/clientHeight`（`editor_render.rs`）已覆盖拖动期画布跟随（见 ST-PU-29）。
