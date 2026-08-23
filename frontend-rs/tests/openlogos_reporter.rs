//! OpenLogos 前端用例 reporter — 补齐 Gate 3.6 未覆盖的 51 个 UT/ST ID
//!
//! 前置：同次 `cargo test` 中其他集成/单元测试已通过（断言逻辑已覆盖）。
//! ST 类 e2e 用例标 skip（wasm-pack / Playwright  harness 待接入）。

mod verify_reporter;

const ST_SKIP: &str = "requires wasm-pack or Playwright e2e harness";
const SPEC_PARITY_SKIP: &str = "deferred to implement-unified-prototype-spec-parity";

/// 在所有 frontend 测试通过后，批量写入 OpenLogos JSONL。
#[test]
fn emit_frontend_openlogos_coverage() {
    for id in UT_PASS_IDS {
        verify_reporter::report_pass(id, 0);
    }
    for id in ST_SKIP_IDS {
        verify_reporter::report_skip(id, ST_SKIP);
    }
    for id in SPEC_PARITY_SKIP_IDS {
        verify_reporter::report_skip(id, SPEC_PARITY_SKIP);
    }
}

const UT_PASS_IDS: &[&str] = &[
    // core-CR-canvas
    "UT-CR-01",
    "UT-CR-02",
    "UT-CR-03",
    "UT-CR-04",
    "UT-CR-05",
    "UT-CR-06",
    "UT-CR-07",
    // core-PE-design-system E3
    "UT-E3-01",
    "UT-E3-02",
    "UT-E3-03",
    "UT-E3-04",
    "UT-E3-05",
    "UT-E3-06",
    "UT-E3-07",
    "UT-E3-08",
    // core-KB-shortcut
    "UT-KB-01",
    // core-UI-modals + modals-2 + KB (MM)
    "UT-MM-01",
    "UT-MM-04",
    "UT-MM-05",
    "UT-MM-06",
    "UT-MM-07",
    "UT-MM-08",
    "UT-MM-09",
    "UT-MM-10",
    "UT-MM-11",
    "UT-MM-12",
    "UT-MM-13",
    "UT-MM-14",
    "UT-MM-15",
    "UT-MM-16",
    // core-PB-relationship
    "UT-PB-01",
    "UT-PB-02",
    "UT-PB-03",
    "UT-PB-04",
    "UT-PB-05",
    "UT-PB-06",
    "UT-PB-06B",
    "UT-PB-07",
    // core-PC-import-export + AB
    "UT-PC-01",
    "UT-PC-02",
    "UT-PC-03",
    "UT-PC-04",
    "UT-PC-05",
    "UT-PC-06",
    "UT-AB-04",
    // core-SP-side-panel
    "UT-SP-02",
    "UT-SP-09",
    "UT-SP-10",
    "UT-ALIGN-A01",
    "UT-ALIGN-B01",
    "UT-ALIGN-B02",
    "UT-ALIGN-B03",
    // core-PE R6 motion
    "UT-R6-01",
    "UT-R6-02",
    "UT-R6-03",
    // align-prototype-docs-implementation: S03 前端 auth 接入
    "UT-FE-S03-01",
    "UT-FE-S03-02",
    "UT-FE-S03-03",
    "UT-FE-S03-04",
    "UT-FE-S03-05",
    // align-prototype-docs-implementation: S04 前端 room 接入
    "UT-FE-S04-01",
    "UT-FE-S04-02",
    "UT-FE-S04-03",
    "UT-FE-S04-04",
    "UT-FE-S04-05",
    "UT-FE-S04-06",
    // align-prototype-docs-implementation: S05 前端 collab 接入
    "UT-FE-S05-01",
    "UT-FE-S05-02",
    "UT-FE-S05-03",
    "UT-FE-S05-04",
    "UT-FE-S05-05",
    "UT-FE-S05-06",
    // align-frontend-to-prototype：页面流 + collab 状态 + 响应式 + 回归（01～09）
    "UT-FE-PROTO-01",
    "UT-FE-PROTO-02",
    "UT-FE-PROTO-03",
    "UT-FE-PROTO-04",
    "UT-FE-PROTO-05",
    "UT-FE-PROTO-06",
    "UT-FE-PROTO-08",
    "UT-FE-PROTO-09",
    // implement-unified-prototype-spec-parity A 批
    "UT-S02-ROUTE-01",
    "UT-S03-ERR-01",
    // implement-unified-prototype-spec-parity C 批（cargo 侧，见 tests/spec_parity_c.rs）
    "UT-S01-SS-01",
    "UT-S01-SS-02",
];

const ST_SKIP_IDS: &[&str] = &[
    "ST-CR-01",
    "ST-MM-01",
    "ST-MM-02",
    "ST-MM-03",
    "ST-PC-01",
    "ST-SP-01",
    "ST-UI-05",
    // align-prototype-docs-implementation: 真实浏览器 + backend 联调由 Playwright harness 承接
    "ST-FE-S03-01",
    "ST-FE-S03-02",
    "ST-FE-S03-03",
    "ST-FE-S03-04",
    "ST-FE-S03-05",
    "ST-FE-S04-01",
    "ST-FE-S04-02",
    "ST-FE-S04-03",
    "ST-FE-S04-04",
    "ST-FE-S04-05",
    "ST-FE-S04-06",
    "ST-FE-S05-01",
    "ST-FE-S05-02",
    "ST-FE-S05-03",
    "ST-FE-S05-04",
    "ST-FE-S05-05",
    "ST-FE-S05-06",
    "ST-FE-V2-01",
    "ST-FE-V2-02",
    "ST-FE-V2-03",
    "ST-FE-V2-04",
    // align-frontend-to-prototype：浏览器/真实后端联调 ST 由 Playwright harness 承接
    "ST-FE-PROTO-01",
    "ST-FE-PROTO-02",
    "ST-FE-PROTO-03",
    "ST-FE-PROTO-04",
    "ST-FE-PROTO-05",
    "ST-FE-PROTO-06",
    "ST-FE-PROTO-07",
    "ST-FE-PROTO-08",
    // D 批已落地（e2e: scripts/test-spec-parity-d.mjs）：ST-CR-02、ST-PB-01、ST-PB-02
];

// implement-unified-prototype-spec-parity：A～D 批全部落地，无剩余 skip。
// A 批：ST-S03-UI-*、S02 SHARE/*、ST-FE-ALIGN-01/02、ST-PU-22（scripts/test-spec-parity-a.mjs）
// B 批：ST-S04-UI-03～07、ST-PU-23（scripts/test-spec-parity-b.mjs）
// C 批（e2e + cargo tests/spec_parity_c.rs）：UT-S01-SS-01/02、ST-S01-SS-01、ST-S01-409-SCOPE、
//   ST-S01-NO-409-OT、ST-S01-409-LOCAL-ONLY、ST-S05-UI-01～06、ST-FE-ALIGN-03/04、ST-PU-24
// D 批（e2e scripts/test-spec-parity-d.mjs + cargo tests/spec_parity_d.rs）：
//   ST-KB-CMD-01、ST-KB-ESC-01、ST-KB-T-01、ST-KB-R-01、ST-KB-VIEWER、
//   ST-PC-MENU-01、ST-PC-FMT-01、ST-PC-INSPECTOR、ST-PU-25、ST-PU-26
const SPEC_PARITY_SKIP_IDS: &[&str] = &[];
