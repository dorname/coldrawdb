# [BUG] 区域画完后无法调整区域大小

## Describe the bug

画布上用「添加区域」工具拖框创建区域后，只能平移位置，**无法再调整宽高**。选中区域后边缘/角点无缩放手柄；Inspector 仅提供名称、颜色与删除，也没有宽度/高度输入。尺寸定错只能删了重画，体验差。

对照：表已有 `feat-table-resize` 一类改宽/最小高度能力；区域目前只有创建拖框一次定尺寸 + 后续整框拖动。

## To Reproduce

1. 打开编辑器，选择 ToolRail「添加区域」
2. 在画布上拖框创建一个区域并松开
3. 选中该区域（点击区域或从 Areas 列表选中）
4. 尝试拖边缘/角点改变大小，或在 Inspector 中寻找宽高字段
5. 观察：只能拖动整体位置；无法改 `width` / `height`；Inspector 仅有名称 / 颜色 / 删除

## Expected behavior

区域创建后应可调整大小，建议至少支持：

1. **画布交互**：选中区域后显示边缘/四角 resize 手柄（或边框拖拽改尺寸），最小宽高有合理下限
2. **Inspector**：可编辑宽度、高度（或 x/y/w/h），改后即时反映到画布并持久化
3. **撤销/协作**：尺寸变更进入 CommandStack / OT（若适用），与现有区域拖动落账路径一致

## Screenshots

（可选补充：选中区域后无手柄、Inspector 无宽高字段的截图）

## Desktop

- OS: Windows / WSL2
- Browser: 待补充
- Theme: 不限

## Additional context

- 代码侧现状（对照当前工作树）：
  - 创建：`area_rect_from_drag` 拖框一次写入 `x/y/width/height`
  - 移动：`area_drag` 支持整框平移
  - Inspector：`inspector-area-form` 仅 `name` / `color` / 删除（`editor_panels.rs`）
  - 未见区域边/角 resize 命中与落账路径（表侧已有 `feat-table-resize`）
- 相关能力：区域创建 / 选中 / 拖动 / Inspector 编辑（p0-fix 定点 2）
- 建议验收：
  1. 选中区域后可拖边/角改变宽高，松手后尺寸持久化（刷新仍在）
  2. Inspector 改宽高与画布一致
  3. 只读模式不可 resize
  4. 最小尺寸约束生效（拖框创建阈值逻辑可复用或对齐）
