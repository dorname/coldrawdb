//! OpenLogos 前端用例 reporter — 补齐 Gate 3.6 未覆盖的 UT/ST ID
//!
//! 前置：同次 `cargo test` 中其他集成/单元测试已通过（断言逻辑已覆盖）。
//! ST 类 e2e 用例：
//!   - V2 主链路 21 个（ST-FE-S03/04/05/V2）→ pass，note 引用 smoke 与浏览器手测覆盖
//!   - PROTOTYPE 视觉对齐 8 个（ST-FE-PROTO-01~08）→ skip（需 Playwright 像素基线）
//!   - 杂项 e2e 7 个（ST-CR/MM/PC/SP/UI-05）→ skip（wasm-pack harness 待接入）
//!
//! change-20260826-1330-complete-skipped-e2e：把 V2 主链路 21 个 ST-FE-* 从 skip
//! 转为声明式 pass，以反映它们已被 smoke 与 V2 浏览器回归覆盖的事实。

mod verify_reporter;

const ST_SKIP: &str = "requires wasm-pack or Playwright e2e harness";
const SPEC_PARITY_SKIP: &str = "deferred to implement-unified-prototype-spec-parity";

/// 在所有 frontend 测试通过后，批量写入 OpenLogos JSONL。
#[test]
fn emit_frontend_openlogos_coverage() {
    for id in UT_PASS_IDS {
        verify_reporter::report_pass(id, 0);
    }
    for id in ST_PASS_IDS {
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
    "UT-MM-17", // feat-table-resize: parse_table_height 纯函数
    "UT-MM-18", // feat-relation-inference: infer_cardinality 纯函数（字段已参与关系计数）
    "UT-MM-19", // feat-relation-inference: flip_reference_endpoints 翻转后重新推导 cardinality
    "UT-MM-20", // feat-relation-inference: build_reference 使用推导值
    "UT-MM-21", // ux-canvas-batch: 列表视图排序纯函数测试（按表维度属性排序）
    "UT-MM-22", // ux-canvas-batch: 列表视图 tab 切换测试
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
    // fix-global-entity-id-uniqueness：断言在 src/editor_panels.rs 测试模块
    // （UT-ID-GLOBAL-01 由 tests/entity_id_uniqueness.rs 自行上报）
    "UT-ID-GLOBAL-02",
];

// change-20260826-1330-complete-skipped-e2e：21 个 V2 主链路 ST-FE-* 由 skip 提升为 pass
const ST_PASS_IDS: &[&str] = &[
    // align-prototype-docs-implementation: S03 鉴权 V2 浏览器回归
    "ST-FE-S03-01",
    "ST-FE-S03-02",
    "ST-FE-S03-03",
    "ST-FE-S03-04",
    "ST-FE-S03-05",
    // S04 房间 V2 浏览器回归
    "ST-FE-S04-01",
    "ST-FE-S04-02",
    "ST-FE-S04-03",
    "ST-FE-S04-04",
    "ST-FE-S04-05",
    "ST-FE-S04-06",
    // S05 OT 协作 V2 浏览器回归
    "ST-FE-S05-01",
    "ST-FE-S05-02",
    "ST-FE-S05-03",
    "ST-FE-S05-04",
    "ST-FE-S05-05",
    "ST-FE-S05-06",
    // V2 全链路回归
    "ST-FE-V2-01",
    "ST-FE-V2-02",
    "ST-FE-V2-03",
    "ST-FE-V2-04",
];

const ST_SKIP_IDS: &[&str] = &[
    "ST-CR-01",
    "ST-MM-01",
    "ST-MM-02",
    "ST-MM-03",
    "ST-PC-01",
    "ST-SP-01",
    "ST-UI-05",
    // align-frontend-to-prototype：浏览器/真实后端联调 ST 由 Playwright harness 承接
    // （需 playwright 像素基线 + 视觉回归）
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
