# Delta — core-04-side-panel-tabs.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3）

## MODIFIED — §1 侧边栏布局（Phase A 废弃说明 + Tool Rail 图标 E2 替换）

**merge 时替换** §1 段，更新为：

### §1 侧边栏布局（V2 — Tool Rail + Issues Collapse）

V1 的 280px 左栏 7 Tab 已在 **Phase A** 中废弃（`redesign-phase-a-layout` 已合并）。V2 改为：

- **48px Tool Rail**（左侧图标轨）：5 个核心动作按钮 + Issues 徽章
- **Issues 折叠面板**（位于 AppBar 下方的全宽条带）：仅在 issue 计数 > 0 时展开
- **浏览能力**：通过 E4 Command Palette（`Ctrl+K`）恢复，不在侧栏

```
+--48px--+ +--------------------------------+  +--可变--+
|        | |                                |  |        |
| Tool   | |       EditorCanvas             |  |Inspector|
| Rail   | |                                |  | (L3)   |
| (L2.5) | |                                |  |        |
|        | +--------------------------------+  |        |
| [+Tbl] | | [Issues 5]  ▼ Collapse  (E3 组件)  |        |
| [+Area]| +--------------------------------+  |        |
| [+Note]|                                  |        |
| [ Rel] |                                  |        |
| [ Pan] |                                  |        |
| (5 btns)|                                  |        |
+--------+                                  +--------+
```

**Tool Rail 按钮清单**（E2 图标替换原 emoji）：

| 按钮 | 图标（E2） | 提示 | data-testid |
|---|---|---|---|
| 新建表 | `<IconAddTable />` | "新建表 (T)" | `cdb-tool-rail-add-table` |
| 新建区域 | `<IconAddArea />` | "新建区域" | `cdb-tool-rail-add-area` |
| 新建便签 | `<IconAddNote />` | "新建便签" | `cdb-tool-rail-add-note` |
| 关系工具 | `<IconRelationship />` | "关系工具" | `cdb-tool-rail-relationship` |
| 平移 | `<IconPan />` | "平移画布" | `cdb-tool-rail-pan` |

**Issues 折叠面板**（E3 Collapse 组件）：

| 项 | 规格 |
|---|---|
| 容器 | `<Collapse lazyRender keepDOM={false} bordered={Default}>` |
| Header | `<Tag color=Warning><IconWarning />{count}</Tag>` + 标题"问题"（来自 main `Issues.jsx`） |
| 折叠状态 | issue 计数 = 0 时默认折叠；> 0 时展开 |
| 动画 | 收起/展开 `--cdb-duration-base` |
| 内容 | 最多 160px 高，超出滚动 |

**z-index**：Tool Rail `--cdb-z-side-rail`（L2.5），Issues Collapse 跟随 AppBar 层（L2）

## MODIFIED — §2 Tables Tab（Phase A 标记废弃）

**merge 时在 §2 顶部插入废弃说明**：

> ⚠️ **V1 280px 左栏 7 Tab 已在 Phase A 废弃**。Tables Tab 的浏览/搜索能力迁移至：
> - **快捷跳转**：`Ctrl+K` 调出 Command Palette（E4）
> - **创建入口**：Tool Rail `<IconAddTable />`（§1）
> - **属性编辑**：画布选中表 → Inspector 抽屉（`core-01-editor-canvas.md` §6）
>
> 本节 §2 内容**仅作 V1 行为记录**，不构成 V2 规范。V2 行为以 `core-01-editor-canvas.md` 与 `core-09-core-components.md` §8 Collapse 为准。

## MODIFIED — §3–§7 Areas / Enums / Notes / Relationships / Types Tab（同上废弃说明）

**merge 时在 §3、§4、§5、§6、§7 顶部各插入相同的废弃说明**（与 §2 相同，列表项指向对应 Tool Rail 按钮）。

## MODIFIED — §8 Issues Tab（升级为 E3 Collapse）

**merge 时替换** §8 Issues 段，更新为：

### §8 Issues 折叠面板（E3 Collapse）

V1 Issues 是 7 Tab 之一。V2 升级为 AppBar 下方全宽折叠条带，由 E3 `<Collapse>` 组件承载。

**Props**：
```rust
<Collapse
  lazy_render={true}
  keep_dom={false}
  bordered={CollapseBordered::Default}
>
  <CollapsePanel
    header=view! {
      <Tag color=TagColor::Warning size=TagSize::Small>
        <IconWarning />
        {move || count.get()}
      </Tag>
      <span class="cdb-ms-2">"问题"</span>
    }
    item_key="issues"
  >
    {move || issues_list.get().into_iter().map(|i| view! { <div class="cdb-py-2">{i}</div> }).collect_view()}
  </CollapsePanel>
</Collapse>
```

**行为**（对齐 main `Issues.jsx`）：
- issue 计数 = 0 → 默认折叠 + Tag 不显示
- issue 计数 > 0 → 默认展开 + Tag 显示计数（`overflowCount=99`）
- 严格模式（`settings.strictMode=true`）→ 显示"严格模式开启，无问题"占位

**视觉**：
- header 高度 40px，hover `--cdb-color-grey-1`
- 列表项 `--cdb-font-size-sm`，`color: var(--cdb-color-text-1)`
- Tag `color=Warning` 背景 `--cdb-color-warning-soft`
