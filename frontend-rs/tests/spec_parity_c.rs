//! implement-unified-prototype-spec-parity C 批 — 保存态与协作 409 单元用例
//!
//! Spec: `logos/resources/test/core-S01-test-cases.md`（UT-S01-SS-01/02）
//! 对齐实现：`editor_panels.rs::save_chip_state` + `editor_core.rs::CollabOtState::snapshot_conflict_shows_modal`
//!
//! 覆盖用例：
//! - UT-S01-SS-01 dirty → saving → saved 文案与主原型 saveText 表一致；成功后 revision 推进（源码断言）
//! - UT-S01-SS-02 重试耗尽 → save-state=Error；本地 dirty 不丢（源码断言）

use frontend_rs::editor_core::{CollabConnectionState, CollabOtState};
use frontend_rs::editor_panels::save_chip_state;

/// UT-S01-SS-01：保存态派生与主原型 saveText 表逐字一致
#[test]
fn ut_s01_ss_01_save_chip_state_matches_prototype_save_text() {
    // 主原型 saveText：saved=已保存 / dirty=有未保存更改 / saving=保存中… / error=保存失败
    assert_eq!(
        save_chip_state(false, false, true),
        ("dirty", "有未保存更改", "cdb-save-dot--dirty"),
        "UT-S01-SS-01: 编辑后 dirty 态文案"
    );
    assert_eq!(
        save_chip_state(true, false, true),
        ("saving", "保存中…", "cdb-save-dot--saving"),
        "UT-S01-SS-01: debounce 触发 PUT 后 saving 态文案"
    );
    assert_eq!(
        save_chip_state(false, false, false),
        ("saved", "已保存", "cdb-save-dot--saved"),
        "UT-S01-SS-01: PUT 成功后 saved 态文案"
    );
    // saving 优先级最高（PUT 进行中即使出错标记残留也先显示保存中）
    assert_eq!(save_chip_state(true, true, true).0, "saving");
    assert_eq!(save_chip_state(true, false, false).0, "saving");

    // 成功路径 revision 推进：schedule_save Ok 分支必须写回服务器 rev 并清 dirty
    let panels_src = include_str!("../src/editor_panels.rs");
    assert!(
        panels_src.contains("store.revision.set(resp.revision);"),
        "UT-S01-SS-01: PUT 成功后 revision-display 必须 +1（revision 写回缺失）"
    );
    let ok_arm = panels_src
        .split("Ok(resp) => {")
        .nth(1)
        .expect("schedule_save Ok 分支存在");
    assert!(
        ok_arm.contains("store.dirty.set(false);"),
        "UT-S01-SS-01: PUT 成功后必须清 dirty（saved 态来源）"
    );
}

/// UT-S01-SS-02：PUT 网络失败重试耗尽 → error 态；本地 dirty 不丢
#[test]
fn ut_s01_ss_02_save_failure_keeps_dirty_and_shows_error() {
    assert_eq!(
        save_chip_state(false, true, true),
        ("error", "保存失败", "cdb-save-dot--error"),
        "UT-S01-SS-02: 重试耗尽后 save-state=Error 文案"
    );

    let panels_src = include_str!("../src/editor_panels.rs");
    // 失败分支：save_offline 置位 + 错误文案，但不得清 dirty（保留本地未保存标记）
    let err_arm = panels_src
        .split("Err(_) => {")
        .nth(1)
        .expect("schedule_save Err(_) 分支存在");
    assert!(
        err_arm.contains("save_offline.set(true);"),
        "UT-S01-SS-02: 失败必须置 save_offline（error 态来源）"
    );
    assert!(
        !err_arm.contains("dirty.set(false)"),
        "UT-S01-SS-02: 失败分支禁止清 dirty（不丢本地未保存更改）"
    );
    // 重试机制存在：指数退避 PUT（初始 + 至多 3 次重试）
    let access_src = include_str!("../src/editor_data_access.rs");
    assert!(
        access_src.contains("SAVE_RETRY_DELAYS_MS") && access_src.contains("save_with_retry"),
        "UT-S01-SS-02: 指数退避重试（save_with_retry）必须保留"
    );
}

/// ST-S01-409-SCOPE / ST-S01-409-LOCAL-ONLY 支撑：409 模态抑制判定矩阵
#[test]
fn s01_409_modal_suppression_matrix() {
    // 协作已连接 + 非仅本地 → 禁止模态（服务器合并）
    let connected = CollabOtState::connected(7);
    assert!(
        !connected.snapshot_conflict_shows_modal(),
        "协作 Connected 态 409 不得弹 S01 模态"
    );

    // 协作已连接 + 用户选择仅本地 → 允许模态
    let mut local_only = CollabOtState::connected(7);
    local_only.enter_local_only();
    assert!(
        local_only.snapshot_conflict_shows_modal(),
        "仅本地态 409 必须允许 S01 模态"
    );

    // 重连中 / 离线 / 只读 / 无协作（默认） → 允许模态（非 OT 合并路径）
    for conn in [
        CollabConnectionState::Reconnecting,
        CollabConnectionState::Offline,
        CollabConnectionState::ReadOnly,
        CollabConnectionState::Connecting,
    ] {
        let mut st = CollabOtState::default();
        st.connection = conn;
        assert!(
            st.snapshot_conflict_shows_modal(),
            "非 Connected 协作态 409 必须允许 S01 模态"
        );
    }
    assert!(CollabOtState::default().snapshot_conflict_shows_modal());
}
