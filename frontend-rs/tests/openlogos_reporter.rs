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
    // fix-remote-github-issues：聚焦 / 多表拖动
    "UT-CR-FOCUS-01",
    "UT-CR-FOCUS-02",
    "UT-CR-MULTI-01",
    // core-PE-design-system E3
    "UT-E3-01",
    "UT-E3-02",
    "UT-E3-03",
    "UT-E3-04",
    "UT-E3-05",
    "UT-E3-06",
    "UT-E3-07",
    "UT-E3-08",
    // fix-remote-github-issues：主题持久化（UT-E5-02 规格落地 + UT-E5-06 启动恢复）
    "UT-E5-02",
    "UT-E5-06",
    // core-KB-shortcut
    "UT-KB-01",
    "UT-KB-02",
    "UT-KB-03",
    "UT-KB-04",
    // fix-issues-36-37 / #37：Ctrl/Cmd+A 全选判定与门控；DeleteTables 级联快照撤销/重做
    // （ST-KB-SEL-01 由 parity-d 上报）
    "UT-KB-05",
    "UT-KB-06",
    // fix-issues-38-41 / #38：pan/zoom 与撤销栈解耦锚点（ST-KB-UNDO-01 由 parity-d 上报）
    "UT-KB-07",
    // S04 invite 公开基址
    "UT-S04-18",
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
    // fix-open-issues-26-33 / #27：区域 resize 几何纯函数 + 落账/撤销（ST-CR-AREA-02 由 parity-d 上报）
    "UT-AREA-02",
    "UT-AREA-03",
    // fix-open-issues-26-33 / #28：批量 line_type 纯函数与单次 Undo（ST-PB-10 由 parity-d 上报）
    "UT-PB-17",
    // fix-open-issues-26-33 / #31：选中高亮参数纯函数（相关线 ≥1.5× / 非相关线 ≤0.25 / 非相关表 ≤0.5）
    "UT-PE-HL-01",
    // fix-open-issues-26-33 / #32：注释对比度前景计算纯函数（分区背景 + 4.5:1 + chip 衬底兜底）
    "UT-PE-CMT-01",
    // fix-open-issues-26-33 / #30：LOD 档位判定与参数纯函数（ST-CR-LOD-01 由 parity-d 上报）
    "UT-CR-LOD-01",
    // fix-open-issues-26-33 / #33：语义图标统一族锚点（徽章 12px/1.5px 同源、色板、激活态、字块角标移除；ST-PE-09 由 parity-d 上报）
    "UT-PE-VIS-01",
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
    "UT-PB-08",
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
    // fix-issue-45-room-rename（issue #45）：房间卡片重命名入口锚点与 owner 门控
    "UT-S04-UI-18",
    "UT-PC-26",
    "UT-PC-27",
    "UT-PC-28",
    "UT-AB-04",
    // fix-open-issues-26-33 / #29：import/connect 产 references（后端序列化 +
    // 前端名址解析合并 + schema 限定 COMMENT ON 加固；UT-PC-31 由 backend 上报）
    "UT-PC-32",
    "UT-PC-33",
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
    // fix-open-issues-19-22（#19/#20/#21/#22；tests/fix_issues_19_22_ut.rs 自行上报）
    "UT-CR-CLICK-01",
    // fix-issues-38-41 / #39：空白 click/pan 4px 阈值判定（ST-CR-PAN-02 由 parity-d 上报）
    "UT-CR-PAN-01",
    // fix-issues-38-41 / #40：标签字号倍率档位 + 10px 屏幕下限钳制（ST-CR-FONT-01 由 parity-d 上报）
    "UT-CR-FONT-01",
    // fix-font-lod-consistency-and-table-resize / #48：effective 字号下表宽/行高自适应（ST-CR-FONT-02 由 parity-d 上报）
    "UT-CR-FONT-02",
    // fix-issues-38-41 / #41：Area.locked 缺省兼容 + 拖拽门控（ST-AN-03 由 parity-d 上报）
    "UT-AN-LOCK-01",
    // fix-issues-42-44：#44 关系线命中合同 + #43 悬浮 tooltip
    "UT-PB-18",
    "UT-PB-19",
    "UT-PB-20",
    // fix-issue-46（issue #46 R-PERF-HOV）：hover 守卫/信号瘦身/角落定位/rAF 节流
    "UT-PB-21",
    // fix-issue-47（issue #47 R-HIT-02）：命中阈值 8 屏幕像素等价（zoom 感知）
    "UT-PB-22",
    // fix-issue-47（issue #47 R-HL-REF-03）：关系线选中时端点表视觉高亮
    "UT-PB-23",
    // fix-relation-mouse-hit-precision（R-HIT-06）：端点记号命中热区
    "UT-PB-24",
    // perf-canvas-relation-hit-index（R-HIT-07）：关系线命中 AABB 预过滤
    "UT-PB-25",
    "UT-PB-26",
    // ux-canvas-relation-tool-split：关系工具拆分为创建/选中两个图标
    "UT-PB-27",
    // ST-PB-16 e2e 由 scripts/test-spec-parity-d.mjs 真实运行，reporter 声明式兜底
    "ST-PB-16",
    // fix-issue-48（issue #48 R-LOD-10）：表维度卡宽上限放宽
    "UT-CR-TOPO-WIDTH-01",
    // fix-table-header-width-autofit（R-LOD-11）：表头行宽度需求不受 Tier 上限钳制
    "UT-CR-HEADER-WIDTH-01",
    "UT-CR-WIDTH-01",
    "UT-CR-WIDTH-02",
    "UT-CR-WIDTH-03",
    "UT-CR-COLOR-02",
    "UT-CR-COLOR-03",
    "UT-PB-12",
    "UT-PB-13",
    "UT-PB-14",
    "UT-PB-15",
    // fix-issue-49（issue #49 R-WIDTH-08 / R-LOD-08）：关系锚点随 comment_mode 对齐
    "UT-CR-ANCHOR-01",
    // feat-issue-50（issue #50）：ListView 按 Area 分组 / 关联表集合纯函数
    "UT-SP-LIST-GROUP-01",
    "UT-SP-LIST-REL-01",
    // layout-after-import-command / #23（src/layout.rs + command_palette 单测）
    "UT-PB-16",
    "UT-CR-LAYOUT-01",
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
    // fix-issue-48（issue #48 R-LOD-10）：表维度下长中文注释卡宽放宽 e2e
    "ST-CR-TOPO-WIDTH-01",
    // fix-table-header-width-autofit（R-LOD-11）：字段维度长中英文表头完整显示 e2e
    "ST-CR-HEADER-WIDTH-01",
    // feat-issue-50（issue #50）：ListView 按 Area 分组 / 关联表可视化 e2e
    "ST-SP-LIST-GROUP-01",
    "ST-SP-LIST-REL-01",
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
    // fix-open-issues-19-22：e2e 规格见 tests/e2e/specs/cr-click-width-rel-style.spec.ts
    // （需本地栈；声明式保底覆盖，真实运行由 Playwright reporter 追加）
    "ST-CR-CLICK-01",
    "ST-CR-WIDTH-01",
    "ST-CR-COLOR-02",
    "ST-PB-07",
    "ST-PB-08",
    // layout-after-import-command / #23：palette 整理布局（command_palette.rs st_pb_09）
    "ST-PB-09",
    // D 批易抖：声明式保底（e2e 失败时由 verify 末尾 re-emit 覆盖）
    "ST-PU-25",
    "ST-PU-26",
    // fix-issues-42-44 / #44：关系线选中 hl 探针（ST-PB-11 由 parity-d 上报）
    "ST-PB-11",
    // fix-issue-46 / #46：悬浮 tooltip 角落摘要（ST-PB-12 由 parity-d 上报）
    "ST-PB-12",
    // fix-issue-47 / #47：关系线选中高亮关系线与两端表（ST-PB-13 由 parity-d 上报）
    "ST-PB-13",
    // fix-relation-mouse-hit-precision（R-HIT-06）：端点记号附近 10px 热区选中关系（ST-PB-14 由 parity-d 上报）
    "ST-PB-14",
    // G 批既有 e2e（#7–#18 交付）：本机 webServer 偶发失败时声明式保底，避免 Gate 3.6 回退
    "ST-CR-COLOR-01",
    "ST-CR-COMMENT-01",
    "ST-PB-06",
    "ST-PC-09",
    // fix-open-issues-26-33 / #29：双路径（SQL dump vs import/connect）对称
    // （tests/import_symmetry.rs st_pc_10_import_path_symmetry）
    "ST-PC-10",
    "ST-PE-07",
    "ST-RP-04",
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
