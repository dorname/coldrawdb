//! fix-global-entity-id-uniqueness 回归用例
//!
//! 背景：table/field/reference/area/note 的 id 在后端 DB 是**全局单列主键**，
//! 旧的「图内 max+1 计数器」id（`auto-1` / `ref-0` …）在任何新 diagram 上都会
//! 与其他 diagram 已占用的全局 id 冲突 → PUT 保存 500（新账户建表保存失败）。
//!
//! 修复：`editor_core::new_entity_id` 产出 `{prefix}-{16位hex}` 全局唯一 id。
//! 本文件覆盖 UT-ID-GLOBAL-01；UT-ID-GLOBAL-02 的断言在
//! `src/editor_panels.rs` 测试模块（signal 依赖），由 openlogos_reporter 批量上报。

mod verify_reporter;

use frontend_rs::editor_core::new_entity_id;
use std::collections::HashSet;

/// UT-ID-GLOBAL-01：4 类前缀各生成 1000 个实体 id，全部互不重复，
/// 且格式保持 `{prefix}-{16位hex}`（前缀兼容 data-testid / OT op / 字符串断言）。
#[test]
fn ut_id_global_01_entity_ids_globally_unique() {
    let prefixes = ["auto", "ref", "area", "note"];
    let mut seen = HashSet::new();
    for round in 0..1000u32 {
        for p in prefixes {
            let id = new_entity_id(p);
            let (head, tail) = id
                .rsplit_once('-')
                .unwrap_or_else(|| panic!("第 {round} 轮：id {id} 缺少前缀-后缀分隔符"));
            assert_eq!(head, p, "第 {round} 轮：前缀应保留为 {p}，实际 {id}");
            assert_eq!(tail.len(), 16, "第 {round} 轮：后缀应为 16 位 hex，实际 {id}");
            assert!(
                tail.bytes().all(|b| b.is_ascii_hexdigit()),
                "第 {round} 轮：后缀应为 hex 字符，实际 {id}"
            );
            assert!(seen.insert(id.clone()), "第 {round} 轮：id 重复 {id}");
        }
    }
    assert_eq!(seen.len(), 4000);
    verify_reporter::report_pass("UT-ID-GLOBAL-01", 0);
}
