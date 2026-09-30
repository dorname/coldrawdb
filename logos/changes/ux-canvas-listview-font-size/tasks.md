# 实现任务

## [delta] 规格变更
- [x] 更新 `core-04-side-panel-tabs.md` §10.5，补充 ListView 字体层级说明

## [code] 代码实现
- [x] 调整 `frontend-rs/src/styles.css` 中 ListView 相关字号
- [x] 新增 `UT-LV-FONT-01` Rust 单元测试，锁定 styles.css 字号合同
- [x] 在 `frontend-rs/tests/openlogos_reporter.rs` 注册新 UT ID

## [verify] 验证
- [ ] 运行 `cargo test` 通过
- [ ] 运行 `openlogos verify` 通过
