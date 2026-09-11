//! S07 数据字典 ST 用例（feat-data-dictionary）
//!
//! Spec: `logos/resources/test/core-S07-test-cases.md`
//! - ST-S07-02：旧图（无 dictionaries）加载 → 默认空不报错，编辑后 JSON 新增 dictionaries 键
//! - ST-S07-04：导出文件名 `data-dictionary-{name}.md` + 三段内容与模型一致
//! - ST-S07-05：删除被引用字典 → 单次 undo 字典与字段绑定全部恢复
//!
//! UT-S07-01~10 由 src/editor_dict.rs 单测真实断言、openlogos_reporter.rs 批量登记；
//! UT-S07-11 由 backend 上报；ST-S07-01/03 见 openlogos_reporter.rs 说明。

mod verify_reporter;

use frontend_rs::editor_core::types::{DataDictionary, Diagram, DictionaryItem, Field, Table};
use frontend_rs::editor_core::{CommandStack, EditorStore};
use frontend_rs::editor_dict::{
    cascade_clear_bindings, dict_export_filename, export_dict_markdown, snapshot_bindings,
};
use leptos::{create_rw_signal, SignalGet, SignalSet, SignalUpdate};
use std::cell::RefCell;
use std::rc::Rc;

fn make_field(id: &str, name: &str, dict_code: &str) -> Field {
    Field {
        id: id.into(),
        name: name.into(),
        type_: "INT".into(),
        default: String::new(),
        check: String::new(),
        primary: false,
        unique: false,
        not_null: false,
        increment: false,
        comment: String::new(),
        tag: String::new(),
        dict_code: dict_code.into(),
    }
}

fn make_table(fields: Vec<Field>) -> Table {
    Table {
        id: "t1".into(),
        name: "users".into(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: String::new(),
        fields,
        indices: vec![],
        width: None,
        min_height: None,
    }
}

fn yes_no_dict() -> DataDictionary {
    DataDictionary {
        id: "d1".into(),
        name: "是否".into(),
        code: "yes_no".into(),
        comment: String::new(),
        items: vec![
            DictionaryItem {
                id: "i0".into(),
                value: "0".into(),
                label: "否".into(),
                sort: 0,
            },
            DictionaryItem {
                id: "i1".into(),
                value: "1".into(),
                label: "是".into(),
                sort: 1,
            },
        ],
    }
}

/// ST-S07-02：加载本变更前保存的旧图（无 dictionaries）→ 正常打开、字典空态、
/// 悬空绑定静默置空；编辑（加字典）后 snapshot JSON 新增 `dictionaries` 键。
#[test]
fn st_s07_02_legacy_diagram_loads_then_reserializes_with_dictionaries() {
    // 旧图 JSON：无 dictionaries 键、字段无 dict_code（serde default 兜底）
    let legacy_json = serde_json::json!({
        "id": "d-legacy",
        "name": "旧图",
        "revision": 3,
        "database": "Generic",
        "tables": [{
            "id": "t1", "name": "users", "x": 0.0, "y": 0.0,
            "color": "", "comment": "",
            "fields": [{
                "id": "f1", "name": "status", "type_": "INT",
                "default": "", "check": "", "primary": false, "unique": false,
                "not_null": false, "increment": false, "comment": ""
            }],
            "indices": []
        }],
        "references": [],
        "notes": [],
        "areas": []
    });
    let diagram: Diagram = serde_json::from_value(legacy_json).expect("ST-S07-02: 旧图 JSON 应解析成功");
    assert!(diagram.dictionaries.is_empty(), "ST-S07-02: 缺省 dictionaries 应为空数组");

    let store = EditorStore::new();
    store.load(diagram);
    assert!(store.dictionaries.get().is_empty(), "ST-S07-02: 加载后字典空态");

    // 悬空绑定规范化（dict_code 指向不存在字典 → 静默置空）
    store.tables.update(|ts| ts[0].fields[0].dict_code = "ghost".into());
    let mut tables = store.tables.get();
    let cleared = frontend_rs::editor_dict::sanitize_dangling_bindings(
        &mut tables,
        &store.dictionaries.get(),
    );
    assert_eq!(cleared, 1, "ST-S07-02: 悬空绑定应被置空");

    // 编辑：新建字典 → snapshot JSON 新增 dictionaries 键
    store.dictionaries.set(vec![yes_no_dict()]);
    let snap = store.snapshot("d-legacy".into(), "旧图".into());
    let json = serde_json::to_value(&snap).expect("ST-S07-02: snapshot 应可序列化");
    assert!(
        json.get("dictionaries").is_some(),
        "ST-S07-02: 编辑后 JSON 应新增 dictionaries 键"
    );
    assert_eq!(json["dictionaries"][0]["code"], "yes_no");

    verify_reporter::report_pass("ST-S07-02", 0);
}

/// ST-S07-04：导出 → 文件名 `data-dictionary-{diagram_name}.md`；
/// Markdown 三段（清单/明细/绑定关系）与当前模型一致。
#[test]
fn st_s07_04_export_filename_and_content_match_model() {
    let dicts = vec![
        yes_no_dict(),
        DataDictionary {
            id: "d2".into(),
            name: "性别".into(),
            code: "gender".into(),
            comment: "性别代码".into(),
            items: vec![
                DictionaryItem { id: "im".into(), value: "1".into(), label: "男".into(), sort: 0 },
                DictionaryItem { id: "if".into(), value: "2".into(), label: "女".into(), sort: 1 },
            ],
        },
    ];
    let tables = vec![make_table(vec![
        make_field("f1", "status", "yes_no"),
        make_field("f2", "gender", "gender"),
        make_field("f3", "name", ""),
    ])];

    // 文件名口径（core-01e §4）
    assert_eq!(
        dict_export_filename("电商核心模型"),
        "data-dictionary-电商核心模型.md",
        "ST-S07-04: 文件名应为 data-dictionary-{{name}}.md"
    );

    let md = export_dict_markdown("电商核心模型", &dicts, &tables);
    // 清单 2 行 + 引用字段数
    assert!(md.contains("| 是否 | yes_no | 2 | 1 |  |"), "ST-S07-04: 清单行与模型一致");
    assert!(md.contains("| 性别 | gender | 2 | 1 | 性别代码 |"));
    // 明细按 sort 序
    assert!(md.contains("### 是否（yes_no）"));
    assert!(md.contains("| 0 | 否 |"));
    assert!(md.contains("| 1 | 是 |"));
    // 绑定关系 2 行（未绑定字段不出现）
    assert!(md.contains("| users | status |  | yes_no |"));
    assert!(md.contains("| users | gender |  | gender |"));
    assert!(!md.contains("| users | name |"), "ST-S07-04: 未绑定字段不出现在绑定关系");

    verify_reporter::report_pass("ST-S07-04", 0);
}

/// ST-S07-05：删除被 2 字段引用的字典（确认级联）→ 字典删除 + 绑定置空为
/// **单次 undo 单元**；一次 undo 字典与两个字段绑定全部恢复。
#[test]
fn st_s07_05_delete_referenced_dict_single_undo_restores_all() {
    let store = EditorStore::new();
    store.tables.set(vec![make_table(vec![
        make_field("f1", "status", "yes_no"),
        make_field("f2", "enabled", "yes_no"),
    ])]);
    store.dictionaries.set(vec![yes_no_dict()]);

    // 生产同源通路：apply_dict_mutation 记录单次 DictSnapshot 命令
    let stack_sig = create_rw_signal(Rc::new(RefCell::new(CommandStack::new())));
    frontend_rs::editor_panels::apply_dict_mutation(&store, stack_sig, |ds, tables| {
        assert_eq!(cascade_clear_bindings(tables, "yes_no"), 2);
        ds.retain(|d| d.code != "yes_no");
    });

    assert!(store.dictionaries.get().is_empty(), "ST-S07-05: 字典已删除");
    assert!(
        store.tables.get()[0].fields.iter().all(|f| f.dict_code.is_empty()),
        "ST-S07-05: 两个字段绑定已级联置空"
    );

    // 单次 undo：字典 + 全部绑定恢复
    let cmd = {
        let stack_rc = stack_sig.get();
        let c = stack_rc.borrow_mut().undo();
        c
    }
    .expect("ST-S07-05: 应存在恰好一个 undo 单元");
    CommandStack::revert(&store, &cmd).expect("ST-S07-05: revert 应成功");

    let dicts = store.dictionaries.get();
    assert_eq!(dicts.len(), 1, "ST-S07-05: undo 后字典恢复");
    assert_eq!(dicts[0].code, "yes_no");
    assert_eq!(dicts[0].items.len(), 2, "ST-S07-05: undo 后字典项一并恢复");
    let fields = &store.tables.get()[0].fields;
    assert!(
        fields.iter().all(|f| f.dict_code == "yes_no"),
        "ST-S07-05: undo 后两个字段绑定全部恢复"
    );
    // undo 栈已空 → 不存在第二次 undo（证明级联与删除同属一个单元）
    let no_more = {
        let stack_rc = stack_sig.get();
        let c = stack_rc.borrow_mut().undo();
        c
    };
    assert!(no_more.is_none(), "ST-S07-05: 级联与删除应为单次 undo 单元");

    // redo 回放（对称性）
    let redo_cmd = {
        let stack_rc = stack_sig.get();
        let c = stack_rc.borrow_mut().redo();
        c
    }
    .expect("ST-S07-05: redo 应可取回命令");
    CommandStack::execute(&store, &redo_cmd).expect("ST-S07-05: execute 应成功");
    assert!(store.dictionaries.get().is_empty(), "ST-S07-05: redo 后字典再次删除");

    // snapshot_bindings 形状自检（before/after 捕获口径）
    let bindings = snapshot_bindings(&store.tables.get());
    assert_eq!(bindings.len(), 2);

    verify_reporter::report_pass("ST-S07-05", 0);
}
