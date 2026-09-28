# Delta — core-PB-relationship-test-cases.md（fix-issue-46-hover-perf-and-corner-summary / #46）

## ADDED — UT-PB-21 — hover 守卫与角落定位锚点

- **类型**：前端锚点/纯函数测试（`frontend-rs` lib）
- **断言**：
  1. `hover_ref` 信号类型为 `Option<String>`（仅 ref_id，不含坐标）——锚点：`create_rw_signal(None::<String>)`（R-PERF-HOV-01）
  2. pointermove hover 路径含同 ref 守卫：命中与当前值相同则不 set（锚点：`hover_ref.get_untracked().as_deref() == hit.as_deref()` 或等价比较后跳过）（R-PERF-HOV-02）
  3. tooltip 节点**不含** `style=format!("left:{}px;top:{}px;"` client 坐标绑定；CSS `.cdb-rel-hover-tooltip` 为 `position: absolute` 且含 `left`/`bottom` 角落锚定（R-HOV-03 B1）
  4. hover 命中经 rAF 节流：存在帧合并守卫（pending 标记 + `request_animation_frame`）（R-PERF-HOV-03）
  5. hover 路径无 `schedule_paint` 调用（R-PERF-HOV-04 绘制解耦锚点）
- reporter 登记 `UT-PB-21`

## MODIFIED — ST-PB-12 — 悬浮摘要 e2e（口径修订：角落摘要区）

原断言「tooltip 位于光标右下（client 坐标 +12）」废止。修订后断言：

1. 悬停关系线 → `rel-hover-tooltip` 出现且文案 `t1.f1 → t2.f2`（不变）
2. tooltip 位于画布容器**左下角摘要区**：其 boundingBox 与画布容器左下邻域重合（left/bottom 锚定），**不随**指针微动而位移（同一命中下连续两次采样 box 相等——R-PERF-HOV-01/02 抖动回归）
3. 悬停不改选中态（`sel_ref === false`，不变）；移开 tooltip 消失（不变）
- reporter 登记 `ST-PB-12`

## MODIFIED — 用例 ID 清单（附录）追加

| ID | 标题 |
|---|---|
| UT-PB-21 | hover 守卫/信号瘦身/角落定位/rAF 节流锚点（#46 R-PERF-HOV） |
| ST-PB-12 | （修订）悬浮摘要 e2e：B1 角落摘要区 + 同命中零位移 + 不改选中态 |
