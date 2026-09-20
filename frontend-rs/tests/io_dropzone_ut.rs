//! fix-remote-github-issues-7-18（issue #7/#8）— IO 文件拖放 UT
//!
//! 覆盖：
//!   UT-PC-31  drop 事件读取文件文本写入导入内容并触发解析摘要（源码锚点：drop → data_transfer.files[0]
//!             → read_as_text → content.set + format.set；解析摘要由既有 import_parse_summary 响应式消费）
//!   UT-PC-32  扩展名白名单与 Tab 映射纯函数 import_format_for_filename（.ddl 归 SQL；非白名单拒绝）
//!   UT-PC-33  点击 dropzone 触发隐藏 io-file-input（accept 含 .ddl）
//!
//! 对应规格：`core-01d-import-export.md` §4.3（合并自 fix-remote-github-issues-7-18）。
//! 断言通过后按 logos/spec/test-results.md 追加 reporter 记录。

mod verify_reporter;

use frontend_rs::editor_panels::{
    import_format_for_filename, import_parse_summary, ImportFormat, IMPORT_ACCEPT_EXTS,
    IMPORT_MAX_SIZE_KB,
};
use std::time::Instant;

const PANELS: &str = include_str!("../src/editor_panels.rs");
const CSS: &str = include_str!("../src/styles.css");

fn report(id: &str, start: Instant) {
    verify_reporter::report_pass(id, start.elapsed().as_millis());
}

/// 取 CSS 规则块（`.x { ... }` 首层括号内容）。
fn css_block(css: &str, selector: &str) -> String {
    let start = css
        .find(selector)
        .unwrap_or_else(|| panic!("缺少 CSS 规则：{selector}"));
    let open = css[start..]
        .find('{')
        .map(|i| start + i)
        .expect("规则块必须有 {");
    let close = css[open..]
        .find('}')
        .map(|i| open + i)
        .expect("规则块必须有 }");
    css[open..=close].to_string()
}

/// 从 `start` 起取至多 `n` 个字符的安全切片（源码含中文，避免字节切片切到 UTF-8 边界内）。
fn safe_slice(src: &str, start: usize, n: usize) -> String {
    src[start..].chars().take(n).collect()
}

/// UT-PC-32：扩展名白名单与 Tab 映射（.ddl 归 SQL；非白名单拒绝）。
#[test]
fn ut_pc_32_import_format_for_filename_whitelist_mapping() {
    let start = Instant::now();

    // .sql / .ddl → Sql（.ddl 视为 SQL DDL 文本，与 .sql 同路径解析）
    assert_eq!(import_format_for_filename("a.sql"), Some(ImportFormat::Sql));
    assert_eq!(import_format_for_filename("schema.ddl"), Some(ImportFormat::Sql));
    // 大小写不敏感（真实文件常为 .SQL / .DDL）
    assert_eq!(import_format_for_filename("B.DDL"), Some(ImportFormat::Sql));
    assert_eq!(import_format_for_filename("B.SQL"), Some(ImportFormat::Sql));
    // .dbml → Dbml；.json → Json
    assert_eq!(import_format_for_filename("c.dbml"), Some(ImportFormat::Dbml));
    assert_eq!(import_format_for_filename("d.json"), Some(ImportFormat::Json));
    assert_eq!(import_format_for_filename("d.JSON"), Some(ImportFormat::Json));
    // 非白名单拒绝
    assert_eq!(import_format_for_filename("e.txt"), None);
    assert_eq!(import_format_for_filename("f.csv"), None);
    assert_eq!(import_format_for_filename("g"), None);
    assert_eq!(import_format_for_filename("h.sql.bak"), None);

    // accept 常量与白名单一一对应（UT-PC-33 的 accept 断言同源）
    assert_eq!(IMPORT_ACCEPT_EXTS, ".sql,.ddl,.dbml,.json");

    // 大小上限常量锚点（core-01d §4.3：V1 硬编码 5120 KB）
    assert_eq!(IMPORT_MAX_SIZE_KB, 5120);

    report("UT-PC-32", start);
}

/// UT-PC-31：drop 事件读取文件文本写入导入内容并触发解析摘要（生产源码锚点）。
///
/// wasm 交互路径无法 native 驱动（DragEvent/DataTransfer/File 仅存在于浏览器），
/// 锚点断言锁定 §4.3 交互合同的接线：drop → DataTransfer.files[0] → 白名单 →
/// read_as_text → content.set + format.set；解析摘要由 import_parse_summary 响应式消费
/// （content 是摘要渲染块的依赖信号，写入即触发更新）。
#[test]
fn ut_pc_31_drop_reads_file_text_and_updates_summary() {
    let start = Instant::now();

    // 1) drop 事件接线：preventDefault + DataTransfer.files[0]
    assert!(PANELS.contains("on:drop="), "dropzone 必须绑定 drop 事件");
    let drop_idx = PANELS.find("on:drop=").expect("drop 绑定存在");
    let drop_block = safe_slice(PANELS, drop_idx, 700);
    assert!(drop_block.contains("ev.prevent_default()"), "drop 必须 preventDefault");
    assert!(
        drop_block.contains("data_transfer()") && drop_block.contains("files()"),
        "drop 必须取 DataTransfer.files"
    );

    // 2) 读取路径：白名单校验 → read_as_text → 写 content + 切 Tab
    let read_idx = PANELS
        .find("let read_import_file")
        .expect("必须有 read_import_file 闭包");
    let read_block = safe_slice(PANELS, read_idx, 1800);
    assert!(
        read_block.contains("import_format_for_filename(&name)"),
        "读取前必须过扩展名白名单"
    );
    assert!(
        read_block.contains("IMPORT_MAX_SIZE_KB"),
        "读取前必须过大小上限"
    );
    assert!(
        read_block.contains("gloo::file::futures::read_as_text"),
        "必须用 File.text() 等价物（gloo read_as_text）读取文本"
    );
    assert!(
        read_block.contains("content.set(text)") && read_block.contains("format.set(fmt)"),
        "读取成功必须写入导入内容 signal 并切换 Tab"
    );
    // 非白名单 inline 提示口径（§4.3：不写内容）
    assert!(
        read_block.contains("仅支持 .sql / .ddl / .dbml / .json"),
        "非白名单必须 inline 提示且不写内容"
    );

    // 3) dragover 高亮态接线（§4.3 事件表：dragenter/dragover 附加 is-dragover，dragleave 移除）
    assert!(PANELS.contains("class:is-dragover="), "dropzone 必须绑定 is-dragover 高亮态");
    assert!(PANELS.contains("on:dragenter=") && PANELS.contains("on:dragover=") && PANELS.contains("on:dragleave="),
        "dragenter/dragover/dragleave 事件必须齐全");
    let hover_block = css_block(CSS, ".cdb-io-dropzone.is-dragover");
    assert!(hover_block.contains("border-color"), "is-dragover 必须有高亮边框");

    // 4) 摘要触发链回归：content 写入后 import_parse_summary 给出非 0 摘要（ST-PC-09 的纯函数侧）
    let ddl = "CREATE TABLE users (id INT PRIMARY KEY);\nCREATE TABLE posts (id INT);";
    let summary = import_parse_summary(ImportFormat::Sql, ddl).expect("ddl 按 SQL 解析");
    assert_eq!(summary, "2 条语句", ".ddl 拖入后摘要应非 0");

    // 5) ST-PC-09 数据链路：MySQL 行内表选项 COMMENT='用户表' 必须回填 Table.comment
    // （导入侧此前丢弃表选项注释；导出侧早已产出该形态，round-trip 缺口）
    let ddl_commented = "CREATE TABLE cms_users (\n  id INT PRIMARY KEY COMMENT '主键'\n) COMMENT='用户表';";
    let (tables, _) = frontend_rs::editor_panels::parse_sql_import_tables(ddl_commented)
        .expect("带表选项 COMMENT 的 DDL 应解析成功");
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].name, "cms_users");
    assert_eq!(tables[0].comment, "用户表", "行内表选项 COMMENT 必须回填 Table.comment");
    assert_eq!(tables[0].fields[0].comment, "主键", "列级 COMMENT 回填回归");

    report("UT-PC-31", start);
}

/// UT-PC-33：点击 dropzone 触发隐藏 io-file-input（accept 含 .ddl）。
#[test]
fn ut_pc_33_click_dropzone_triggers_hidden_file_input() {
    let start = Instant::now();

    // 1) 隐藏 file input 存在：testid + accept 白名单（含 .ddl）+ display:none
    assert!(
        PANELS.contains("data-testid=\"io-file-input\""),
        "必须有隐藏 io-file-input"
    );
    let input_idx = PANELS.find("data-testid=\"io-file-input\"").expect("input 存在");
    let input_block = safe_slice(PANELS, input_idx, 500);
    assert!(
        input_block.contains("accept=IMPORT_ACCEPT_EXTS"),
        "accept 必须来自白名单常量"
    );
    assert!(
        IMPORT_ACCEPT_EXTS.contains(".ddl"),
        "accept 必须含 .ddl（issue #8）"
    );
    assert!(
        input_block.contains("display:none"),
        "file input 必须隐藏（仅作兜底触发器）"
    );

    // 2) 点击 dropzone → input.click()（node_ref 接线）
    assert!(PANELS.contains("node_ref=io_file_input_ref"), "input 必须挂 node_ref");
    let click_idx = PANELS.find("io_file_input_ref.get()").expect("点击触发存在");
    let click_block = safe_slice(PANELS, click_idx, 200);
    assert!(click_block.contains("input.click()"), "点击 dropzone 必须触发 input.click()");

    // 3) change 选中后与 drop 共用同一读取路径（§4.3：走与 drop 相同的读取路径）
    let change_idx = PANELS[input_idx..]
        .find("on:change=")
        .map(|i| input_idx + i)
        .expect("io-file-input 必须绑定 change");
    let change_block = safe_slice(PANELS, change_idx, 700);
    assert!(
        change_block.contains("read_import_file(file)"),
        "file input change 必须走与 drop 相同的读取路径"
    );

    report("UT-PC-33", start);
}
