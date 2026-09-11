//! editor-dict: S07 数据字典（PDManer 式代码映射表）纯函数层。
//!
//! Spec: `core-01e-data-dictionary.md` / 时序 `core-S07-data-dictionary.md`。
//! 所有函数不依赖 Leptos/DOM，可在 host 单测；UI 层（editor_panels）只组装信号与事件。
//!
//! 关键口径：
//! - 绑定为**软引用**（`Field.dict_code` 存字典编码），字典改名不影响绑定；
//!   改编码时 `propagate_code_rename` 同步改写引用字段。
//! - 删除被引用字典 → `cascade_clear_bindings` 级联置空（与删除同一 undo 单元由调用方保证）。
//! - 加载时 `sanitize_dangling_bindings` 静默置空悬空绑定（EX-7.4）。

use crate::editor_core::types::{DataDictionary, Field, Table};

/// 字典编码在同图内是否已被其他字典占用（UT-S07-02）。
pub fn dict_code_taken(dicts: &[DataDictionary], code: &str, except_id: &str) -> bool {
    dicts.iter().any(|d| d.id != except_id && d.code == code)
}

/// 字典项 value 在字典内是否重复（UT-S07-03）。
pub fn item_value_taken(dict: &DataDictionary, value: &str, except_id: &str) -> bool {
    dict.items.iter().any(|i| i.id != except_id && i.value == value)
}

/// 字典被多少字段引用（引用计数，core-01e §2.1）。
pub fn dict_ref_count(tables: &[Table], code: &str) -> usize {
    tables
        .iter()
        .flat_map(|t| t.fields.iter())
        .filter(|f| f.dict_code == code)
        .count()
}

/// 删除字典时级联置空所有引用字段的绑定，返回置空数量（UT-S07-06）。
pub fn cascade_clear_bindings(tables: &mut [Table], code: &str) -> usize {
    let mut cleared = 0;
    for t in tables.iter_mut() {
        for f in t.fields.iter_mut() {
            if f.dict_code == code {
                f.dict_code = String::new();
                cleared += 1;
            }
        }
    }
    cleared
}

/// 字典编码改写时同步更新所有引用字段（UT-S07-05）。
pub fn propagate_code_rename(tables: &mut [Table], old_code: &str, new_code: &str) {
    for t in tables.iter_mut() {
        for f in t.fields.iter_mut() {
            if f.dict_code == old_code {
                f.dict_code = new_code.to_string();
            }
        }
    }
}

/// 加载规范化：字段 dict_code 指向不存在的字典 → 静默置空（EX-7.4 / UT-S07-10）。
/// 返回置空数量。
pub fn sanitize_dangling_bindings(tables: &mut [Table], dicts: &[DataDictionary]) -> usize {
    let mut cleared = 0;
    for t in tables.iter_mut() {
        for f in t.fields.iter_mut() {
            if !f.dict_code.is_empty() && !dicts.iter().any(|d| d.code == f.dict_code) {
                f.dict_code = String::new();
                cleared += 1;
            }
        }
    }
    cleared
}

/// 字段卡映射摘要 Tag：`1=男 2=女`；超过 4 项折叠为「共 N 项」（core-01e §3）。
pub fn dict_summary(dict: &DataDictionary) -> String {
    if dict.items.len() > 4 {
        return format!("共 {} 项", dict.items.len());
    }
    let mut items: Vec<&crate::editor_core::types::DictionaryItem> = dict.items.iter().collect();
    items.sort_by_key(|i| i.sort);
    items
        .iter()
        .map(|i| format!("{}={}", i.value, i.label))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 按 code 查字典。
pub fn find_dict<'a>(dicts: &'a [DataDictionary], code: &str) -> Option<&'a DataDictionary> {
    dicts.iter().find(|d| d.code == code)
}

/// Markdown 导出（CAP-DICT-04 / UT-S07-07）：字典清单 + 字典明细 + 字段绑定关系三段。
pub fn export_dict_markdown(diagram_name: &str, dicts: &[DataDictionary], tables: &[Table]) -> String {
    let mut out = format!("# 数据字典 — {}\n\n## 字典清单\n\n| 字典名称 | 编码 | 项数 | 引用字段数 | 说明 |\n|---|---|---|---|---|\n", diagram_name);
    for d in dicts {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            d.name,
            d.code,
            d.items.len(),
            dict_ref_count(tables, &d.code),
            d.comment
        ));
    }
    out.push_str("\n## 字典明细\n");
    for d in dicts {
        out.push_str(&format!("\n### {}（{}）\n\n| 值 | 含义 |\n|---|---|\n", d.name, d.code));
        let mut items: Vec<&crate::editor_core::types::DictionaryItem> = d.items.iter().collect();
        items.sort_by_key(|i| i.sort);
        for i in items {
            out.push_str(&format!("| {} | {} |\n", i.value, i.label));
        }
    }
    out.push_str("\n## 字段绑定关系\n\n| 表 | 字段 | 字段注释 | 绑定字典 |\n|---|---|---|---|\n");
    for t in tables {
        for f in &t.fields {
            if !f.dict_code.is_empty() {
                out.push_str(&format!("| {} | {} | {} | {} |\n", t.name, f.name, f.comment, f.dict_code));
            }
        }
    }
    out
}

/// 新建字典的默认名称/编码（`dict_N` / `code_N`，N 从 1 递增避开占用）。
pub fn next_dict_defaults(dicts: &[DataDictionary]) -> (String, String) {
    let mut n = dicts.len() + 1;
    loop {
        let code = format!("code_{}", n);
        if !dicts.iter().any(|d| d.code == code) {
            return (format!("dict_{}", n), code);
        }
        n += 1;
    }
}

/// 全量字段绑定快照（含空串）：DictSnapshot 命令 before/after 用（core-01e §6）。
pub fn snapshot_bindings(tables: &[Table]) -> Vec<(String, String, String)> {
    tables
        .iter()
        .flat_map(|t| {
            t.fields
                .iter()
                .map(|f| (t.id.clone(), f.id.clone(), f.dict_code.clone()))
        })
        .collect()
}

/// 无字典时导出禁用（UT-S07-08 / core-01e §4）。
pub fn dict_export_enabled(dicts: &[DataDictionary]) -> bool {
    !dicts.is_empty()
}

/// fix-dict-save-and-layout（core-01e §3.1）：ListView 树区字典节点 (title, meta) 视图模型（ST-S07-07）。
/// title = `名称（编码）`；meta = `N 项 · 引用 M`（M = 绑定该字典的字段数，dict_ref_count 口径）。
pub fn list_dict_node_view(dict: &DataDictionary, ref_count: usize) -> (String, String) {
    (
        format!("{}（{}）", dict.name, dict.code),
        format!("{} 项 · 引用 {}", dict.items.len(), ref_count),
    )
}

/// fix-dict-save-and-layout（core-01e §3.1）：ListView 字典明细行 (id, value, label)，按 sort 升序（ST-S07-07）。
pub fn list_dict_detail_rows(dict: &DataDictionary) -> Vec<(String, String, String)> {
    let mut items: Vec<&crate::editor_core::types::DictionaryItem> = dict.items.iter().collect();
    items.sort_by_key(|i| i.sort);
    items
        .iter()
        .map(|i| (i.id.clone(), i.value.clone(), i.label.clone()))
        .collect()
}

/// 导出文件名 `data-dictionary-{diagram_name}.md`（core-01e §4）；非法字符转 `_`。
pub fn dict_export_filename(diagram_name: &str) -> String {
    let safe: String = diagram_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("data-dictionary-{}.md", safe)
}

#[allow(clippy::too_many_arguments)]
pub fn make_field(id: &str, name: &str, type_: &str, dict_code: &str) -> Field {
    Field {
        id: id.into(),
        name: name.into(),
        type_: type_.into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_core::types::{DataDictionary, DictionaryItem, Table};

    fn dict(id: &str, name: &str, code: &str, items: Vec<(&str, &str, &str)>) -> DataDictionary {
        DataDictionary {
            id: id.into(),
            name: name.into(),
            code: code.into(),
            comment: String::new(),
            items: items
                .into_iter()
                .enumerate()
                .map(|(ix, (id, value, label))| DictionaryItem {
                    id: id.into(),
                    value: value.into(),
                    label: label.into(),
                    sort: ix as i64,
                })
                .collect(),
        }
    }

    fn table_with(fields: Vec<Field>) -> Table {
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

    #[test]
    fn ut_s07_02_dict_code_conflict_detected() {
        let dicts = vec![dict("d1", "是否", "yes_no", vec![]), dict("d2", "性别", "gender", vec![])];
        assert!(dict_code_taken(&dicts, "yes_no", "d2"));
        assert!(!dict_code_taken(&dicts, "yes_no", "d1")); // 自身豁免
        assert!(!dict_code_taken(&dicts, "active", "d2"));
    }

    #[test]
    fn ut_s07_03_item_value_conflict_detected() {
        let d = dict("d1", "性别", "gender", vec![("i1", "1", "男"), ("i2", "2", "女")]);
        assert!(item_value_taken(&d, "1", "i2"));
        assert!(!item_value_taken(&d, "1", "i1"));
        assert!(!item_value_taken(&d, "3", "i1"));
    }

    #[test]
    fn ut_s07_05_code_rename_propagates_to_fields() {
        let mut tables = vec![table_with(vec![
            make_field("f1", "status", "INT", "yes_no"),
            make_field("f2", "gender", "INT", "gender"),
        ])];
        propagate_code_rename(&mut tables, "yes_no", "yn");
        assert_eq!(tables[0].fields[0].dict_code, "yn");
        assert_eq!(tables[0].fields[1].dict_code, "gender");
    }

    #[test]
    fn ut_s07_06_cascade_clear_on_dict_delete() {
        let mut tables = vec![table_with(vec![
            make_field("f1", "status", "INT", "yes_no"),
            make_field("f2", "enabled", "INT", "yes_no"),
            make_field("f3", "gender", "INT", "gender"),
        ])];
        assert_eq!(dict_ref_count(&tables, "yes_no"), 2);
        let cleared = cascade_clear_bindings(&mut tables, "yes_no");
        assert_eq!(cleared, 2);
        assert_eq!(tables[0].fields[0].dict_code, "");
        assert_eq!(tables[0].fields[1].dict_code, "");
        assert_eq!(tables[0].fields[2].dict_code, "gender");
    }

    #[test]
    fn ut_s07_10_dangling_binding_sanitized_on_load() {
        let mut tables = vec![table_with(vec![
            make_field("f1", "status", "INT", "ghost_dict"),
            make_field("f2", "gender", "INT", "gender"),
        ])];
        let dicts = vec![dict("d2", "性别", "gender", vec![])];
        let cleared = sanitize_dangling_bindings(&mut tables, &dicts);
        assert_eq!(cleared, 1);
        assert_eq!(tables[0].fields[0].dict_code, "");
        assert_eq!(tables[0].fields[1].dict_code, "gender");
    }

    #[test]
    fn ut_s07_04_summary_tag_format_and_fold() {
        let d = dict("d1", "性别", "gender", vec![("i1", "1", "男"), ("i2", "2", "女")]);
        assert_eq!(dict_summary(&d), "1=男 2=女");
        let big = dict(
            "d2",
            "状态",
            "status",
            vec![("a", "0", "草稿"), ("b", "1", "待审"), ("c", "2", "发布"), ("d", "3", "归档"), ("e", "4", "废弃")],
        );
        assert_eq!(dict_summary(&big), "共 5 项");
    }

    #[test]
    fn ut_s07_07_markdown_export_three_sections() {
        let dicts = vec![
            dict("d1", "是否", "yes_no", vec![("i0", "0", "否"), ("i1", "1", "是")]),
            dict("d2", "性别", "gender", vec![("im", "1", "男"), ("if", "2", "女")]),
        ];
        let tables = vec![table_with(vec![
            make_field("f1", "status", "INT", "yes_no"),
            make_field("f2", "gender", "INT", "gender"),
        ])];
        let md = export_dict_markdown("电商核心模型", &dicts, &tables);
        assert!(md.contains("# 数据字典 — 电商核心模型"));
        assert!(md.contains("## 字典清单"));
        assert!(md.contains("| 是否 | yes_no | 2 | 1 |  |"));
        assert!(md.contains("### 性别（gender）"));
        assert!(md.contains("| 1 | 男 |"));
        assert!(md.contains("## 字段绑定关系"));
        assert!(md.contains("| users | status |  | yes_no |"));
        assert!(md.contains("| users | gender |  | gender |"));
        // 明细按 sort 序
        let yn = md.find("### 是否").unwrap();
        let p0 = md[yn..].find("| 0 | 否 |").unwrap();
        let p1 = md[yn..].find("| 1 | 是 |").unwrap();
        assert!(p0 < p1);
    }

    #[test]
    fn ut_s07_01_next_defaults_skip_taken_codes() {
        let dicts = vec![dict("d1", "是否", "code_2", vec![])];
        let (name, code) = next_dict_defaults(&dicts);
        assert_eq!(name, "dict_3"); // len+1 = 2 被占用 → 3
        assert_eq!(code, "code_3");
    }

    #[test]
    fn ut_s07_08_export_disabled_when_no_dicts() {
        assert!(!dict_export_enabled(&[]));
        let dicts = vec![dict("d1", "是否", "yes_no", vec![])];
        assert!(dict_export_enabled(&dicts));
    }

    #[test]
    fn st_s07_07_list_dict_node_view_and_detail_rows() {
        // 树节点视图模型：title = 名称（编码）；meta = N 项 · 引用 M
        let d = dict("d1", "性别", "gender", vec![("i1", "1", "男"), ("i2", "2", "女")]);
        let (title, meta) = list_dict_node_view(&d, 3);
        assert_eq!(title, "性别（gender）");
        assert_eq!(meta, "2 项 · 引用 3");

        // 明细行按 sort 升序（乱序 sort 输入验证排序而非输入序）
        let mut d2 = dict("d2", "状态", "status", vec![("a", "0", "草稿"), ("b", "1", "发布")]);
        d2.items[0].sort = 5;
        d2.items[1].sort = 2;
        let rows = list_dict_detail_rows(&d2);
        assert_eq!(
            rows,
            vec![
                ("b".to_string(), "1".to_string(), "发布".to_string()),
                ("a".to_string(), "0".to_string(), "草稿".to_string()),
            ],
            "ST-S07-07: 明细行必须按 sort 升序"
        );
    }

    #[test]
    fn st_s07_04_filename_sanitized() {
        assert_eq!(dict_export_filename("电商核心模型"), "data-dictionary-电商核心模型.md");
        assert_eq!(dict_export_filename("a/b c"), "data-dictionary-a_b_c.md");
    }
}
