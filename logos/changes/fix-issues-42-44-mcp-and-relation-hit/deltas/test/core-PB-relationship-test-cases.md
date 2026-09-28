# Delta — core-PB-relationship-test-cases.md（fix-issues-42-44-mcp-and-relation-hit）

## ADDED — UT 用例（#43/#44）

| ID | 场景 | 预期 |
|---|---|---|
| UT-PB-18 | 构造两条近平行贝塞尔关系，点击点同时落在两条线的 8px 带宽内 → `hit_test_reference_tier` | 返回**距离最小**的关系 id（而非数组序靠前者）；点击点距所有线 > 8px 时返回 None（R-HIT-02/03） |
| UT-PB-19 | `line_type=orthogonal` 关系：点击点落在折线拐角附近（贝塞尔几何不经过、折线几何经过处） | 命中该 orthogonal 关系（R-HIT-01 线型同源）；反之贝塞尔曲线上一点不误命中 orthogonal 线 |
| UT-PB-20 | tooltip 文案纯函数：详情档 → `源表.源字段 → 目标表.目标字段`；拓扑档 → 至少含 `源表 → 目标表` | 文案构成正确（R-HOV-02） |

## ADDED — ST 用例（#43/#44）

| ID | 场景 | 预期 |
|---|---|---|
| ST-PB-11 | e2e：构造三条近平行关系线（字段维度），连续点击各线中段 | 每次选中的关系（Inspector）与点击目标线一致；点击两线正中间超阈值空白 → 不误选任何关系 |
| ST-PB-12 | e2e：pointermove 悬浮关系线中段 → 出现 tooltip 含两端 `表.字段`；悬浮前后选中态不变；移开 tooltip 消失 | tooltip 内容与关系端点一致；`selected_ref_id` 不因悬停改变（R-HOV-04） |

- **reporter**：上述 UT/ST 均写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PB-18 | UT | 关系线命中：最近距离优先 + 阈值外不命中（#44） |
| UT-PB-19 | UT | 关系线命中：线型同源（orthogonal 折线几何）（#44） |
| UT-PB-20 | UT | 悬浮 tooltip 文案纯函数（#43） |
| ST-PB-11 | ST | e2e：密集线点击目的一致 + 空白不误选（#44） |
| ST-PB-12 | ST | e2e：悬浮出 tooltip 且不改选中态（#43） |
