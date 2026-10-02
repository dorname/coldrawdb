# Delta: core-PB-relationship-test-cases.md — UT-PB-32 + 修订 30/31

## MODIFIED — UT-PB-30 / UT-PB-31

- 低缩放带宽口径改为 **14** 屏幕像素：`rel_line_hit_px(0.25)==14`；zoom=0.25 世界 56 命中 / 60 不命中；近线优先同口径。

## ADDED — UT-PB-32 叠线簇轮选

- **覆盖**：`resolve_ref_hit_cluster` / R-HIT-08
- **断言**：无 prev → 最近；prev 在簇内 → 下一根；环回；prev 不在簇 → 最近；单候选稳定
- **reporter**：`UT-PB-32`

### 附录追加

| ID | 标题 |
|---|---|
| UT-PB-32 | 叠线簇点击轮选（精准消歧） |
