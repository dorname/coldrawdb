# Delta — core-01b-relationship.md（fix-issue-46-hover-perf-and-corner-summary / #46）

## MODIFIED — §4.8 R-HOV-03 位置（光标跟随 → 固定角落摘要区）

原文：`- **R-HOV-03 位置**：tooltip 跟随光标右下偏移（约 12px），不遮挡 Inspector；纯 DOM 浮层，不进画布绘制层。`

改为：

```
- **R-HOV-03 位置（fix-issue-46 修订为 B1 固定角落摘要区）**：tooltip 为画布容器（`.cdb-canvas-stack`）内**左下角固定摘要区**（`position:absolute; left:12px; bottom:12px`），不跟随光标、不绑定任何指针坐标；不遮挡 Inspector；纯 DOM 浮层，不进画布绘制层。
  - 选型理由（issue #46 作者认可）：密集关系场景下固定角落稳定可预期、不挡线、不依赖线旁几何；坐标不来自指针事件，从根上消除「视口 client 坐标当容器局部坐标」的错位 bug 类。
  - 演进路径：待密集场景选中可靠性与线旁展示遮挡问题解决后，可评估 B2（线旁跟随）或 B3（混合降级）；本版锁定 B1。
```

## ADDED — §4.8 追加 R-PERF-HOV（悬停性能合同，#46 P0 抖动消除）

```
- **R-PERF-HOV-01 信号瘦身**：hover 信号只携带 `ref_id`（Option<String>），不携带指针坐标——坐标每事件变化是抖动主因，移出信号后命中坐标不再驱动渲染。
- **R-PERF-HOV-02 写守卫**：pointermove 命中结果与当前 hover 值相同（同 ref_id）时**不得** set 信号；仅在「命中变化 / 命中消失 / 进入拖拽态清除」三种迁移时写信号。文案不变 → DOM 不重建。
- **R-PERF-HOV-03 命中节流**：hover 命中检测按 rAF 合并（每帧至多一次 `hit_test_reference_tier`），密集关系大图下避免每事件全量遍历 refs；拖拽/点击路径不节流（交互正确性优先）。
- **R-PERF-HOV-04 绘制解耦**：tooltip 为纯 DOM 节点，任何 hover 路径不得触发 `schedule_paint` / canvas 重绘；空闲（无悬停、无拖拽）时画布完全静止。
```
