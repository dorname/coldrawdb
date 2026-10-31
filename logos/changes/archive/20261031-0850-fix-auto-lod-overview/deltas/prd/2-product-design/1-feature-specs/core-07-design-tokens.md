## MODIFIED — §15.4 `canvas.lod.tier1-max-zoom`

恢复 `canvas.lod.tier1-max-zoom = 0.5`（亮/暗主题相同），仅用于 Auto 维度：小于等于阈值进入拓扑，高于阈值进入详情；显式 Table/Field 不消费该 token。本次不恢复 ±0.03 滞回。`cdb.view-dimension` 的可选值修订为 `auto` / `table` / `field`，默认 Auto。
