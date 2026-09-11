# 实现任务

## 本批覆盖 UT 用例

- UT-S02-01 ~ UT-S02-04（`lib.rs` location_tests — `?share=` 解析）
- UT-S01-09（`editor_data_access::save_retry_tests` — 重试间隔）
- UT-E4-08 / UT-E4-09（`command_palette` 过滤与构建）

## [code] 代码实现

- [x] `lib.rs`：`parse_share_param` + `diagram_id_from_location`
- [x] `editor_data_access.rs`：`save_with_retry`（3s/6s/12s）
- [x] `command_palette.rs`：Ctrl+K + 搜索 + Enter 选中
- [x] `code_view.rs`：Code View 全屏 + Tab + 复制
- [x] `editor_panels.rs`：AppRoot 接线 + 离线保存 UI + share 冷加载
- [x] `styles.css`：`.cdb-main.cdb-is-hidden`

## [verify] 验收

- [x] `cargo test -p frontend-rs`（71 passed + 7 E4 + …）
- [ ] 用户授权 `openlogos verify align-sync-tech-plan-code`
- [ ] 用户授权 `openlogos archive align-sync-tech-plan-code`
