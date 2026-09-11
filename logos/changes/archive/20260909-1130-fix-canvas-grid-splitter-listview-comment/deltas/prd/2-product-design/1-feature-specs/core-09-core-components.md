## MODIFIED — ### 14.2 交互

1. **拖动**：`pointerdown` 在分隔条上时 `set_pointer_capture`，`pointermove` 实时改写宽度，`pointerup` / `pointercancel` 结束并持久化。
2. **拖动方向语义**：宽度增减跟随「分隔条相对面板的边缘位移」，而非统一取 `+dx`——
   - `splitter-inspector`（右侧面板**左缘**）：向左拖（dx<0）面板变宽，向右拖（dx>0）面板变窄，即 `width = start_w - dx`；
   - `splitter-list-tree`（左侧树列**右缘**）：向右拖（dx>0）树列变宽，向左拖变窄，即 `width = start_w + dx`。
   - 键盘方向键维持数值语义（ArrowLeft 减宽 / ArrowRight 加宽，与 `aria-valuenow` 一致），不随实例位置翻转。
3. **hover 态**：分隔条宽 6px，默认透明（叠在两列间隙上）；hover 或拖动中显示品牌色高亮（`--cdb-brand` 或 `--cdb-color-primary`，2px 内缘指示条）。
4. **双击复位**：双击分隔条恢复默认宽度（330 / 230），并写入 localStorage。
5. **键盘可达**：分隔条为 `role="separator"`，`aria-orientation="vertical"`，`aria-valuenow` 反映当前宽度；方向键 ±16px（Shift ±64px），Home/End 跳 min/max，Esc 复位默认。
