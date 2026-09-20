//! fix-remote-github-issues-7-18（issue #9/#11/#17）— 连线选侧 / ResizeObserver / 幽灵层圆角 UT
//!
//! 覆盖：
//!   UT-PB-09        `pick_port_sides` 按相对位置选侧（目标在左 → 左出右进；在右 → 右出左进；x 相等 → 默认）
//!   UT-PB-10        `calc_path` 消费选侧结果：锚点与贝塞尔控制点方向随侧变化
//!   UT-RP-06        容器尺寸变化经 ResizeObserver 同步 backing store（源码锚点 + 像素口径）
//!   UT-RP-07        分隔条拖动触发重绘且 dpr 同源；卸载 disconnect（源码锚点）
//!   UT-CR-GHOST-02  幽灵层圆角高亮（R-HL-02）：禁 CSS outline，canvas 内 round_rect 描边（源码锚点）
//!
//! 对应规格：`core-01b-relationship.md` §3.4、`core-01-editor-canvas.md` §11.5 / §5.7。
//! 断言通过后按 logos/spec/test-results.md 追加 reporter 记录。

mod verify_reporter;

use frontend_rs::editor_core::types::{Field, Table};
use frontend_rs::editor_render::{calc_path, pick_port_sides, FieldPortSide};
use std::time::Instant;

const RENDER: &str = include_str!("../src/editor_render.rs");

/// 默认表宽（editor_render.rs 私有 const TABLE_WIDTH = 230.0；锚定口径防漂移）
const DEFAULT_TABLE_WIDTH: f64 = 230.0;

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

fn fixture_table(id: &str, x: f64, y: f64) -> Table {
    Table {
        id: id.to_string(),
        name: id.to_string(),
        x,
        y,
        color: String::new(),
        comment: String::new(),
        fields: vec![Field {
            id: format!("{id}-f1"),
            name: "id".to_string(),
            type_: "INT".to_string(),
            default: String::new(),
            check: String::new(),
            primary: true,
            unique: false,
            not_null: true,
            increment: false,
            comment: String::new(),
            tag: String::new(),
            dict_code: String::new(),
        }],
        indices: vec![],
        width: None,
        min_height: None,
    }
}

/// UT-PB-09：`pick_port_sides` 按相对位置选侧（core-01b §3.4）。
#[test]
fn ut_pb_09_pick_port_sides_by_relative_position() {
    let start = Instant::now();

    // 目标在右 → 右出左进（默认口径）
    let a = fixture_table("a", 0.0, 0.0);
    let b_right = fixture_table("b", 400.0, 0.0);
    assert_eq!(
        pick_port_sides(&a, &b_right),
        (FieldPortSide::End, FieldPortSide::Start),
        "目标在右 → (右出, 左进)"
    );

    // 目标在左 → 左出右进
    let b_left = fixture_table("b", -400.0, 0.0);
    assert_eq!(
        pick_port_sides(&a, &b_left),
        (FieldPortSide::Start, FieldPortSide::End),
        "目标在左 → (左出, 右进)"
    );

    // 中心 x 相等 → 稳定性默认（右出左进），避免边界抖动
    let b_same = fixture_table("b", 0.0, 300.0);
    assert_eq!(
        pick_port_sides(&a, &b_same),
        (FieldPortSide::End, FieldPortSide::Start),
        "中心 x 相等 → 默认 (右出, 左进)"
    );

    // 仅依赖几何（x/width）：自定义宽度的表按实际中心判定
    let mut wide = fixture_table("w", 100.0, 0.0);
    wide.width = Some((DEFAULT_TABLE_WIDTH * 3.0) as u32);
    // a 中心 = 0 + W/2；wide 中心 = 100 + 3W/2 → 目标在右
    assert_eq!(
        pick_port_sides(&a, &wide),
        (FieldPortSide::End, FieldPortSide::Start),
        "width 参与中心计算"
    );

    report("UT-PB-09", start);
}

/// UT-PB-10：`calc_path` 消费选侧结果（core-01b §3.4）。
#[test]
fn ut_pb_10_calc_path_consumes_picked_sides() {
    let start = Instant::now();

    // GIVEN：A(0,0) 与 B(-400,0)（B 在 A 左侧），各含字段
    let a = fixture_table("a", 0.0, 0.0);
    let b = fixture_table("b", -400.0, 0.0);

    // WHEN：计算 A→B 路径
    let p = calc_path(&a, "a-f1", &b, "b-f1");

    // THEN：起点 x == A.x（左缘），终点 x == B.x + B.width（右缘）
    assert_eq!(p.x1, a.x, "目标在左：起点应为源表左缘锚点");
    assert_eq!(p.x2, b.x + DEFAULT_TABLE_WIDTH, "目标在左：终点应为目标表右缘锚点");
    // 贝塞尔控制点 c1 < x1（向左伸展）、c2 > x2（向右伸展）
    assert!(p.cx1 < p.x1, "左出控制点应向左伸展：cx1={} x1={}", p.cx1, p.x1);
    assert!(p.cx2 > p.x2, "右进控制点应向右伸展：cx2={} x2={}", p.cx2, p.x2);

    // B 移到 A 右侧后回归右出左进口径（与历史默认视觉一致）
    let b_right = fixture_table("b", 400.0, 0.0);
    let p2 = calc_path(&a, "a-f1", &b_right, "b-f1");
    assert_eq!(p2.x1, a.x + DEFAULT_TABLE_WIDTH, "目标在右：起点应为源表右缘锚点");
    assert_eq!(p2.x2, b_right.x, "目标在右：终点应为目标表左缘锚点");
    assert!(p2.cx1 > p2.x1, "右出控制点应向右伸展");
    assert!(p2.cx2 < p2.x2, "左进控制点应向左伸展");

    // 动态更新：同一对表、拖动后相对位置翻转 → 选侧翻转（无需重建关系）
    let b_moved = fixture_table("b", -400.0, 0.0);
    let p3 = calc_path(&a, "a-f1", &b_moved, "b-f1");
    assert_eq!(p3.x1, a.x, "拖动后按最新几何重算出入侧");

    report("UT-PB-10", start);
}

/// UT-RP-06：容器尺寸变化经 ResizeObserver 同步 backing store（core-01 §11.5，R-DPR-07）。
///
/// ResizeObserver 仅存在于浏览器，native 侧锁源码锚点 + 像素同步口径：
/// 回调内必须按最新 client_width/height × 有效 dpr 写 canvas.width/height 并 schedule_paint。
#[test]
fn ut_rp_06_resize_observer_syncs_backing_store() {
    let start = Instant::now();

    // 1) ResizeObserver 挂载：canvas 父容器 observe
    assert!(
        RENDER.contains("web_sys::ResizeObserver::new"),
        "R-DPR-07：必须创建 ResizeObserver"
    );
    let observe_idx = RENDER.find("web_sys::ResizeObserver::new").expect("observer 存在");
    let mount_block: String = RENDER[observe_idx..].chars().take(600).collect();
    assert!(
        mount_block.contains("canvas.parent_element()") && mount_block.contains("obs.observe(&parent)"),
        "R-DPR-07：observer 必须挂 canvas 父容器"
    );

    // 2) 回调内同步 backing store + 触发重绘（同帧口径）
    let cb_idx = RENDER.find("sync(&canvas_for_cb)").expect("RO 回调体存在");
    let cb_block: String = RENDER[cb_idx..].chars().take(120).collect();
    assert!(
        cb_block.contains("sp();"),
        "R-DPR-07：回调内必须同步 backing store 并 schedule_paint"
    );

    // 3) 像素同步口径（sync_backing_store 闭包）：CSS × 有效 dpr → canvas.width/height
    let sync_idx = RENDER.find("let sync_backing_store").expect("sync 闭包存在");
    let sync_block: String = RENDER[sync_idx..].chars().take(1600).collect();
    assert!(
        sync_block.contains("parent.client_width()") && sync_block.contains("parent.client_height()"),
        "R-DPR-07：必须按最新 client_width/height 计算"
    );
    assert!(
        sync_block.contains("capped_drag_dpr") && sync_block.contains("current_device_pixel_ratio()"),
        "R-DPR-09：有效 dpr 必须走 capped_drag_dpr(current_device_pixel_ratio()) 同源"
    );
    assert!(
        sync_block.contains("canvas.set_width(w)") && sync_block.contains("canvas.set_height(h)"),
        "R-DPR-07：必须写 canvas.width/height（backing store）"
    );

    // 4) 渲染 effect 与 RO 回调共用同一同步闭包（不绕路）
    let effect_idx = RENDER.find("sync_backing_store(&canvas)").expect("effect 消费 sync 闭包");
    assert!(
        effect_idx > sync_idx,
        "渲染 effect 必须消费 sync_backing_store（同源同步）"
    );

    report("UT-RP-06", start);
}

/// UT-RP-07：分隔条拖动触发重绘且 dpr 同源；卸载 disconnect（R-DPR-08/09/10）。
#[test]
fn ut_rp_07_splitter_covered_and_observer_disconnected() {
    let start = Instant::now();

    // 1) R-DPR-10：组件卸载 disconnect（on_cleanup 取句柄 → disconnect）
    let cleanup_idx = RENDER.find("on_cleanup(move").expect("on_cleanup 调用存在");
    let cleanup_block: String = RENDER[cleanup_idx..].chars().take(300).collect();
    assert!(
        cleanup_block.contains("obs.disconnect()"),
        "R-DPR-10：卸载必须断开 ResizeObserver"
    );
    // observer 句柄保存在 holder（非 forget），保证可断开
    assert!(
        RENDER.contains("ro_holder") && RENDER.contains("borrow_mut().take()"),
        "R-DPR-10：observer 句柄必须可取回断开（不得泄漏监听）"
    );

    // 2) R-DPR-08：分隔条只写 CSS 变量、不绕过统一同步（splitter.rs 无重绘调用，
    //    由 ResizeObserver 自然覆盖——布局变化必然触发 RO 回调）
    const SPLITTER: &str = include_str!("../src/splitter.rs");
    assert!(
        !SPLITTER.contains("ResizeObserver"),
        "R-DPR-08：splitter 不得自建 observer（统一走画布 RO）"
    );

    // 3) R-DPR-09：RO 回调使用的有效 dpr 与 R-PERF-10 拖拽降采样同源——
    //    sync 闭包内 drag_active 判定与 capped_drag_dpr 调用（锚点见 UT-RP-06 #3），
    //    且渲染 effect 不再保留独立 dpr 计算副本
    let sync_idx = RENDER.find("let sync_backing_store").expect("sync 闭包存在");
    let sync_block: String = RENDER[sync_idx..].chars().take(1200).collect();
    assert!(
        sync_block.contains("drag_state.get_untracked().is_some()"),
        "R-DPR-09：拖拽活跃期有效 dpr 判定必须与渲染 effect 同源"
    );

    report("UT-RP-07", start);
}

/// UT-CR-GHOST-02：幽灵层圆角高亮（core-01 §5.7，R-HL-01/02）。
#[test]
fn ut_cr_ghost_02_ghost_highlight_is_rounded() {
    let start = Instant::now();

    // 1) create_table_ghost 内不得出现 CSS outline 高亮（R-HL-02 禁令）
    let ghost_idx = RENDER.find("fn create_table_ghost").expect("create_table_ghost 存在");
    let ghost_block: String = RENDER[ghost_idx..].chars().take(1800).collect();
    assert!(
        !ghost_block.contains("outline:"),
        "R-HL-02：幽灵层不得使用 CSS outline 画高亮（不贴合 border-radius）"
    );

    // 2) 改用幽灵 canvas 内 round_rect 描边（draw_table_selection 同一函数：16/14 圆角口径）
    assert!(
        ghost_block.contains("draw_table_selection(&off, table, palette)"),
        "R-HL-01/02：幽灵层必须在 canvas 内用 draw_table_selection 画圆角选中环"
    );
    // set_transform 与 sprite 渲染同参数（环落在表体世界坐标处）
    assert!(
        ghost_block.contains("(sprite.margin - table.x) * scale"),
        "幽灵层选中环 transform 必须与 sprite 渲染同口径"
    );

    // 3) draw_table_selection 自身锚点：round_rect 16（外扩环）+ 14（表体）
    let sel_idx = RENDER.find("fn draw_table_selection").expect("draw_table_selection 存在");
    let sel_block: String = RENDER[sel_idx..].chars().take(700).collect();
    assert!(
        sel_block.contains("round_rect") && sel_block.contains("16.0") && sel_block.contains("14.0"),
        "R-HL-01：选中环必须为 16/14 圆角描边"
    );

    report("UT-CR-GHOST-02", start);
}
