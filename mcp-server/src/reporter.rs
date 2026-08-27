use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Utc;
use serde_json::json;

static REPORT_LOCK: Mutex<()> = Mutex::new(());

fn result_path() -> PathBuf {
    // 沙箱 (bwrap --ro-bind workspace) 让 CARGO_MANIFEST_DIR 指向沙箱副本，
    // reporter 用它解析 jsonl 路径 → 沙箱销毁后丢失。
    // 允许通过环境变量 COLDRAWDB_JSONL_PATH 覆盖为绝对路径。
    if let Ok(p) = std::env::var("COLDRAWDB_JSONL_PATH") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("logos/resources/verify/test-results.jsonl")
}

pub fn report(id: &str, result: Result<(), String>, duration_ms: u128) {
    let _guard = REPORT_LOCK.lock().unwrap();
    let path = result_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let (status, message) = match result {
        Ok(()) => ("pass", None),
        Err(message) => ("fail", Some(redact(&message))),
    };
    let mut record = json!({"id":id,"status":status,"duration_ms":duration_ms,"timestamp":Utc::now().to_rfc3339(),"module":"core","scenario":"S06"});
    if let Some(message) = message {
        record["error"] = json!(message);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{record}");
    }
}

pub fn redact(message: &str) -> String {
    let mut output = message.to_string();
    if let Ok(token) = std::env::var("COLDRAWDB_ACCESS_TOKEN") {
        if !token.is_empty() {
            output = output.replace(&token, "[REDACTED]");
        }
    }
    output
}
