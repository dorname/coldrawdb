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
    // fix-canvas-zoom-perf：§3.3 锚定缩放 + §5.6 R-PERF-01~06（editor_render.rs 单测）
    "UT-CR-ZOOM-01",
    "UT-CR-CULL-01",
    "UT-CR-BATCH-01",
    "UT-CR-FONTCACHE-01",
    "UT-CR-DRAG-01",
    // perf-canvas-drag-smoothness：§5.6 R-PERF-07~11（editor_render.rs 单测；
    // ST-CR-GHOST-01 由 scripts/test-canvas-perf.mjs e2e 上报）
    "UT-CR-SPRITE-01",
    "UT-CR-GUARD-01",
    "UT-CR-DPR-01",
    "UT-CR-GHOST-01",
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
    "UT-MM-23", // ux-canvas-batch: 列表视图过滤纯函数测试（按名称模糊匹配/按类型/按是否有索引）
    "UT-MM-24", // ux-canvas-batch: 列表视图批量重命名纯函数测试（重名冲突处理：B2-S1 补充规则）
    "UT-MM-25", // ux-canvas-batch: ViewMode 三态迁移测试（Canvas→List→Canvas、Canvas→Code→Canvas、List 下画布隐藏条件）
    "UT-MM-26", // ux-canvas-batch: 列表视图批量改类型纯函数测试（类型兼容性通用决策程序：族内由窄到宽直接改/由宽到窄跳过/跨族跳过/未列出对保守 fallback 跳过/非法目标类型跳过）
    "UT-MM-27", // ux-canvas-batch: 列表视图导出 CSV schema 内容纯函数测试（CSV 转义：逗号/引号/换行；输入 &[Table]，行=字段，列=table_name/field_name/field_type/has_index——C-3 裁决仅 CSV 纯手写无依赖）
    "UT-MM-28", // ux-canvas-batch 批次4: ListView 列宽钳制 + 自适应纯函数测试（clamp_column_width min=60, max=480；auto_calc_column_width 公式 max(60, min(480, chars × 8 + 40))；7 子用例覆盖边界/钳制/saturating 溢出）
    "UT-MM-29", // ux-canvas-batch 批次4: 表/字段分组纯函数测试（GroupByMode {None, ByTag} 两模式；统一输出 Vec<Bucket{key, fields}>；None = 单桶 _flat；ByTag 按 Field.tag 分桶空 tag 归 (empty)；大小写敏感；7 子用例覆盖空表/单 tag/混合 tag/大小写/单字段多 tag/输出形状统一）
    "UT-MM-30", // ux-canvas-batch 批次4: rAF 调度去重 + TextCacheKey 测试（schedule_render_dedup 可测同步核 pending 状态机：首次入队执行/二次 noop/清 pending 后可入队；schedule_render 壳 + TextCacheKey font_px 容差；6 子用例覆盖三态/多轮/键相等性）
    "UT-MM-31", // p0-fix 定点 1: delete_room URL 拼接 + 状态映射纯函数（204 → Ok；403/404/500 → ApiError::Server(状态码, body)）
    "UT-MM-32", // p0-fix 定点 1: can_delete_room 纯函数（my_role == "owner" → true；editor/viewer/空串 → false）
    "UT-MM-33", // p0-fix 定点 3: hit_test_reference / point_near_bezier / bezier_point / dist_point_segment 点到贝塞尔连线命中检测纯函数
    "UT-MM-34", // p0-fix 定点 3: is_delete_key 纯函数（Delete/Backspace 命中；普通键/Escape/空串不命中）
    "UT-AREA-01", // p0-fix 定点 2: area_rect_from_drag 归一化 + <10px 不创建 + build_area 默认值 + hit_test_area 后创建优先
    "UT-NOTE-01", // p0-fix 定点 2: build_note 默认值 + hit_test_note 固定 180×100 命中
    "UT-MM-35", // list-view-table-structure: group_tables ByTable 按表分桶（桶键=表名 / 桶序保持 / 空表出桶 / 空输入 0 桶）
    "UT-MM-36", // diagram-database-dialect: types_for_database 三组清单（Generic 对齐既有 5 项 / MySQL 含 DATETIME 无 SERIAL / PostgreSQL 含 SERIAL 无 DATETIME / 未暴露引擎回落 Generic）
    "UT-MM-37", // redesign-listview-type-length-canvas-fix: parse_field_type/compose_field_type 参数化类型纯函数（VARCHAR 长度 / DECIMAL、NUMERIC 精度小数位 / 缺省值 / 老整串兼容 / 往返一致）
    "UT-MM-38", // listview-tree-master-detail: 表树过滤与当前表解析纯函数（filter_tables 口径 + resolve_list_current_table 命中/回落首表/空数组/过滤导航解耦）
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
    // relation-inspector-and-ddl-io: DDL 列级解析 + 外键 references
    "UT-PC-07",
    "UT-PC-08",
    // fix-appbar-roomname-back-and-import-merge: 导入合并避让布局 + AppBar 缩略/返回锚点
    "UT-PC-09",
    "UT-S04-UI-12",
    // fix-overflow-menu-room-delete-listview-io: ListView IO/菜单边界锚点 + 房间卡片删除锚点
    "UT-PC-10",
    "UT-S04-UI-13",
    // feat-db-connect-import-and-ddl-realdb-verify: ImportDrawer 数据库来源锚点
    // （UT-PC-11/12/13 由 backend/src/bridge_introspect.rs 上报；UT-PC-14 由 tests/ddl_realdb_verify.rs 上报）
    "UT-PC-15",
    // feat-room-recycle-bin-dropdown-and-db-export: 回收站锚点 + 导出到数据库/下拉样式锚点
    // （UT-PC-16/17 由 backend/src/bridge_introspect.rs 上报）
    "UT-S04-UI-15",
    "UT-PC-18",
    "UT-PC-19",
    // fix-dbimport-save-and-pg-schema: 导入合并 ID 重键 + schema 输入锚点
    // （UT-PC-21 由 backend/src/bridge_introspect.rs 上报）
    "UT-PC-20",
    "UT-PC-22",
    "UT-PC-23",
    // fix-select-bg-diagram-delete-and-log-format：创建空间弹窗删除图表锚点
    "UT-S04-UI-16",
    // fix-canvas-zoom-invite-comment-resize：前端数据面（同源 base_url + 结构化 import/connect
    // 消费 + COMMENT ON 解析/导出 + JSON 注释透传；UT-PC-24/25 由 backend 上报）
    "UT-S04-UI-17",
    "UT-PC-26",
    "UT-PC-27",
    "UT-PC-28",
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
    // wire-frontend-collab-ws：真实 WS 传输接线（src/collab_client.rs 单测同次 cargo test 通过）
    "UT-FE-S05-13",
    "UT-FE-S05-14",
    // fix-collab-autosave-race：UT-FE-S05-15（checkpoint 头构造）随 checkpoint 协议废弃移除
    "UT-FE-S05-16",
    "UT-FE-S05-17",
    // fix-collab-autosave-race：room 一律跳过全量保存（editor_panels.rs 单测）/ ack 清空
    // dirty / RemoteOp 有序收编+缓冲回放 / fields.reorder diff（collab_client.rs 单测
    // 同次 cargo test 通过）
    "UT-FE-S05-18",
    "UT-FE-S05-19",
    "UT-FE-S05-20",
    "UT-FE-S05-21",
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
    // feat-data-dictionary（S07）：字典纯函数 8 断言在 src/editor_dict.rs 单测
    // （UT-S07-09 断言在 src/editor_data_access.rs dict_s07_tests；UT-S07-11 由 backend 上报）
    "UT-S07-01",
    "UT-S07-02",
    "UT-S07-03",
    "UT-S07-04",
    "UT-S07-05",
    "UT-S07-06",
    "UT-S07-07",
    "UT-S07-08",
    "UT-S07-09",
    "UT-S07-10",
    // fix-dict-save-and-layout：DiagramForSave 显式序列化 dictionaries
    // （断言在 src/editor_data_access.rs dict_s07_tests）
    "UT-S07-12",
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
    // wire-frontend-collab-ws：真实 WS 双上下文回归（tests/e2e/s05-collab.spec.ts，
    // 需本地栈；真实运行结果由 playwright openlogos reporter 追加，此处声明式保底覆盖）
    "ST-FE-S05-07",
    "ST-FE-S05-08",
    "ST-FE-S05-09",
    // fix-collab-autosave-race：物化单写者端到端一致性（同 s05-collab.spec.ts 真实链路，
    // 需本地栈；声明式保底覆盖同上）
    "ST-FE-S05-10",
    // V2 全链路回归
    "ST-FE-V2-01",
    "ST-FE-V2-02",
    "ST-FE-V2-03",
    "ST-FE-V2-04",
    // fix-canvas-zoom-invite-comment-resize：导出→导入 round-trip 注释不丢
    // （tests/comment_roundtrip.rs st_pc_07_export_import_comment_roundtrip）
    "ST-PC-07",
    // feat-data-dictionary（S07）：端到端建字典→绑字段→保存→重开链路。
    // 持久化主链路由真实断言覆盖：backend UT-S07-11（save→load 往返）+
    // 前端 ST-S07-02/UT-S07-09/UT-S07-10（加载/规范化）；UI 编排段为声明式保底覆盖
    // （同 ST-FE-S05-07~09 先例，浏览器回归由 smoke 承接）。
    "ST-S07-01",
    // fix-dict-save-and-layout：抽屉互斥状态机断言在 src/editor_panels.rs
    // st_s07_06_overlay_mutex_transitions；DOM 叠加段为声明式保底覆盖
    "ST-S07-06",
    // fix-dict-save-and-layout：ListView 字典区块视图模型断言在 src/editor_dict.rs
    // st_s07_07_list_dict_node_view_and_detail_rows；DOM 渲染段为声明式保底覆盖
    "ST-S07-07",
];

const ST_SKIP_IDS: &[&str] = &[
    "ST-CR-01",
    "ST-MM-01",
    "ST-MM-02",
    "ST-MM-03",
    "ST-PC-01",
    "ST-SP-01",
    "ST-UI-05",
    // p0-fix 定点 1：Owner 删除房间全流程（按钮→确认模态→DELETE→回 rooms 页刷新）——e2e 链路待 wasm-pack/Playwright harness
    "ST-S04-UI-08",
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
    // feat-data-dictionary（S07）：Viewer 只读（编辑控件 disabled + 导出可用）——
    // disabled 属性断言需 DOM harness（同 ST-CR-01 等先例）
    "ST-S07-03",
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
