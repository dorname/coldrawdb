## MODIFIED — UT-CR-LOD-01 维度模式与可读性补偿纯函数

- 存储合同改为：`from_stored(None)` / 非法值 / `"auto"` → Auto；旧 `"table"` / `"field"` 原样保留；`next` 按 Auto → Table → Field → Auto 环回；自动标签为“自动”。
- 显式 Table → Topology、Field → Detail 在 zoom 0.1 / 0.35 / 1.0 / 2.0 下不变。删除“缺省 Field、所有维度与 zoom 无关”的旧预期。
- 维度内字号/线宽补偿、精灵指纹、表级锚点与拓扑注释规则不变。

## MODIFIED — ST-CR-LOD-01 显式维度切换 e2e

- 默认改为 Auto；“字段维度 35% 仍展开”必须先显式切到 Field，再验证 35% / 100% / 200% 不串档。
- 三入口循环顺序和指示同步覆盖 Auto → Table → Field → Auto；刷新后持久化当前显式模式。
- 表维度补偿、拓扑注释、表级锚定既有断言保留。

## ADDED — UT-CR-LOD-02 自动 LOD 阈值纯函数

- **位置**：`frontend-rs/src/editor_render.rs`。
- **断言**：Auto 在 zoom=0.5 / 0.1 时为 Topology，在 zoom=0.5001 / 1.0 时为 Detail；非法/防御性 zoom 不 panic；显式 Table/Field 在阈值两侧保持固定档位。
- **reporter**：登记 `UT-CR-LOD-02` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — ST-CR-LOD-02 全景自动拓扑与显式覆盖 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **前置**：默认 Auto，预置 16 表 / 19 关系。
- **步骤**：缩放到 zoom≈0.1，恢复 zoom=1；依次切换 Table、Field、Auto 并跨阈值缩放，刷新验证偏好。
- **断言**：Auto 全景时 `tier=topology`、`anchor_mode=table`，放大后恢复 detail/field；Field 在 zoom≈0.1 仍保持字段详情，Table 在高 zoom 仍保持拓扑；回到 Auto 后重新按 zoom 判档；localStorage 与指示一致。
- **reporter**：登记 `ST-CR-LOD-02` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-LOD-02 | 自动 LOD 阈值与显式覆盖纯函数 | `editor_render.rs::tier_for_dimension` |
| ST-CR-LOD-02 | 全景自动拓扑与显式覆盖 e2e | `frontend-rs/scripts/test-spec-parity-d.mjs` |
