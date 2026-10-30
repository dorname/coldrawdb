## MODIFIED — UT-PB-20 悬浮 tooltip 文案纯函数

详情档保留 `源表.源字段 → 目标表.目标字段`，拓扑档保留 `源表 → 目标表`，两种档位均追加“若线条重叠，重复点击可切换关系”；未知 ref 仍返回 None。

## ADDED — UT-PB-33 完全重合路径逐条轮选

- **位置**：`frontend-rs/src/editor_render.rs`。
- **前置**：两个表、三条使用相同端点和线型的关系，路径完全重合。
- **操作**：在详情和拓扑档位下，对同一路径中点连续调用 `hit_test_reference_tier`，将上次选中 id 作为 `prev_selected`。
- **断言**：首次选中首条；后续按 refs 顺序选中第二、第三条并环回首条；两种档位覆盖一致，不修改命中算法。
- **reporter**：登记 `UT-PB-33` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — ST-PB-17 大图完全重合关系提示与点击可达性

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **前置**：预置 16 表 / 19 关系，其中四条关系共享同一对表和字段端点，几何完全重合。
- **步骤**：悬浮重合线中点，随后在同一屏幕坐标重复点击四次，等待每次选中探针更新。
- **断言**：tooltip 包含“若线条重叠，重复点击可切换关系”；四次点击依次选中四条关系且可继续环回；选中关系工具与默认选择工具至少一条路径覆盖；全程无新增 PUT/op 落账。
- **reporter**：登记 `ST-PB-17` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — 用例登记追加

| ID | 层级 | 说明 |
|---|---|---|
| UT-PB-33 | UT | 详情/拓扑完全重合路径逐条轮选（#60） |
| ST-PB-17 | ST | 16 表 / 19 关系叠线提示与点击可达性（#60） |
