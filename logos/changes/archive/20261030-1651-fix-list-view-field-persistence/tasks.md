# 实现任务

## [code] 代码实现

- [x] 用 Issue #58 路径复现并定位列表字段编辑在真实保存 / 协作 op 链路中的断点，记录失败证据——真实 backend + WS 回归通过，未复现故障；既有 Mock 用例也通过
- [x] 评估 `frontend-rs/src/editor_panels.rs` 与 `frontend-rs/src/collab_client.rs`——现行提交、diff、op 上行与物化链路正确，无需业务代码修改
- [x] 强化 `ST-SP-LIST-02`：新增 `frontend-rs/tests/e2e/specs/listview-persistence.spec.ts`，覆盖全屏 ListView 的字段名、注释、PK / NN / UNIQUE / 自增、类型与参数编辑，并断言真实服务端快照及重新加载一致
- [x] 在同一真实用例中验证 Viewer 控件禁用与强制 `change` 不落账；复跑既有 `ST-SP-LIST-02`，撤销 / 增删移动回归通过
- [x] 更新 Playwright OpenLogos reporter 映射并运行相关 ST，保留真实通过证据；完整验收待用户授权 `openlogos verify`
