# 实现任务

## [code] 代码实现（纯测试基建，无 delta）
- [x] #68：`frontend-rs/tests/e2e/playwright.config.ts` 增加全局字体 abort（fonts.googleapis.com / fonts.gstatic.com），对齐 parity 系列口径
- [x] #67：`frontend-rs/scripts/test-spec-parity-e.mjs` ST-PU-27 在进房与首次拖拽之间增加稳态等待（轮询无 in-flight 请求 + 首帧稳定），断言语义不变
- [x] 本地定向回归：`SPEC_PARITY_ONLY=ST-PU-27` 隔离跑 PASS；E 批全量 6/6 PASS；G 批 goto 挂起假失败已消除（ST-PB-05/ST-RP-04 通过）。G 批剩余 5 个失败为真实产品缺陷——矮视口下悬浮控件被状态栏遮挡（已提 issue #69，属产品代码修复，不在本提案范围）
