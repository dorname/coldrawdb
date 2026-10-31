## ADDED — UT-CR-SPRITE-02 同字体桶不同几何的缓存失效

- **位置**：`frontend-rs/src/editor_render.rs`（`table_sprite_geometry_fingerprint` / `effective_font_bucket` / `lod_table_size_with`）。
- **前置**：同一多字段 Table，Field/Detail，字号倍率 1.5；取 zoom=0.46 与 0.48。
- **断言**：两者 `sprite_zoom_bucket` 与 `effective_font_bucket` 相同，但 `lod_table_size_with` 高度不同；几何指纹必须不同，相同输入重复计算保持稳定；坐标不属于几何指纹输入。
- **reporter**：`UT-CR-SPRITE-02` 写入 `logos/resources/verify/test-results.jsonl`。

## ADDED — ST-CR-SPRITE-02 缩放后卡体精灵与选中环几何一致

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs` + `window.__cdb_table_geometry_probe`。
- **前置**：预置多字段 `asset_object` 表并选中，显式 Field，字号 100%；先缩放到同一粗字体桶内的较低 zoom 建立精灵，再反向缩放。
- **断言**：实际精灵 `body_width/body_height` 与选中环 `selection_width/selection_height` 相等，且表 id 一致；随后在 40%～150% 区间双向缩放、切换 150% 字号、注释显示模式和 Table/Field 维度后仍一致，不出现旧尺寸卡体与新选中环组合。
- **reporter**：`ST-CR-SPRITE-02` 写入 `logos/resources/verify/test-results.jsonl`。

### 用例登记追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-SPRITE-02 | 同字体桶不同几何的缓存失效 | `editor_render.rs::table_sprite_geometry_fingerprint` |
| ST-CR-SPRITE-02 | 缩放后卡体精灵与选中环几何一致 | e2e + `__cdb_table_geometry_probe` |
