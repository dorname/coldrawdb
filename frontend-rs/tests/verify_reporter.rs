//! OpenLogos reporter（frontend-rs 集成测试专用）
//! 输出：`logos/resources/verify/test-results.jsonl`

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const RELATIVE_PATH: &str = "logos/resources/verify/test-results.jsonl";

static FILE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static TRUNCATE_DONE: AtomicBool = AtomicBool::new(false);

fn lock() -> &'static Mutex<()> {
    FILE_LOCK.get_or_init(|| Mutex::new(()))
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn result_path() -> PathBuf {
    // 沙箱 (bwrap --ro-bind workspace) 让 CARGO_MANIFEST_DIR 指向沙箱副本，
    // reporter 用它解析 jsonl 路径 → 沙箱销毁后丢失。
    // 允许通过环境变量 COLDRAWDB_JSONL_PATH 覆盖为绝对路径。
    if let Ok(p) = std::env::var("COLDRAWDB_JSONL_PATH") {
        return PathBuf::from(p);
    }
    project_root().join(RELATIVE_PATH)
}

fn append_only_mode() -> bool {
    std::env::var("OPENLOGOS_APPEND").ok().as_deref() == Some("1")
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = (rem / 3600) as u32;
    let minute = ((rem % 3600) / 60) as u32;
    let second = (rem % 60) as u32;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

fn civil_from_days(z: i64) -> (u32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z / 146_097 } else { (z - 146_096) / 146_097 };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y } as u32;
    (y, m, d)
}

fn ensure_parent(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
}

fn id_is_valid(id: &str) -> bool {
    if id.len() < 7 {
        return false;
    }
    let Some((kind, rest)) = id.split_once('-') else {
        return false;
    };
    if kind != "UT" && kind != "ST" {
        return false;
    }
    let parts: Vec<&str> = rest.split('-').collect();
    parts.len() >= 2 && parts.iter().all(|p| !p.is_empty())
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn append_record(id: &str, status: &str, duration_ms: Option<u128>, error: Option<&str>) {
    if !id_is_valid(id) {
        return;
    }
    let path = result_path();
    ensure_parent(&path);
    let _guard = lock().lock().unwrap();
    if !append_only_mode() && !TRUNCATE_DONE.swap(true, Ordering::SeqCst) {
        let _ = fs::write(&path, "");
    }
    let mut record = format!(
        "{{\"id\":\"{}\",\"status\":\"{}\",\"timestamp\":\"{}\"",
        escape_json(id),
        status,
        now_iso8601()
    );
    if let Some(ms) = duration_ms {
        record.push_str(&format!(",\"duration_ms\":{}", ms));
    }
    if let Some(err) = error {
        record.push_str(&format!(",\"error\":\"{}\"", escape_json(err)));
    }
    record.push_str("}\n");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("open test-results.jsonl");
    file.write_all(record.as_bytes()).expect("write jsonl");
    let _ = file.flush();
}

pub fn report_pass(id: &str, duration_ms: u128) {
    append_record(id, "pass", Some(duration_ms), None);
}

pub fn report_skip(id: &str, reason: &str) {
    append_record(id, "skip", None, Some(reason));
}
