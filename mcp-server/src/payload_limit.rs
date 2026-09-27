//! fix-open-issues-26-33（2026-09-27，#26）：update_diagram 参数体积软上限。
//!
//! 大图整量 update_diagram 容易超时/413 且失败后无任何进度落库。本模块在
//! adapter 侧本地估算参数体积，超过软上限（默认 256 KiB，可用
//! COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES 调整）时直接返回可诊断的
//! PAYLOAD_TOO_LARGE（details 含当前体积 / 上限 / 分片建议），不发送上游请求；
//! 官方分批路径为 update_table / update_field / update_reference / layout_diagram
//! 串行携带最新 expected_revision（见 core-S06-mcp-service-design.md §4.4）。

use serde_json::{json, Value};

use crate::error::ToolError;

/// 软上限默认值：256 KiB（与 mcp-tools.yaml configuration.default 一致）。
pub const DEFAULT_SOFT_LIMIT_BYTES: usize = 256 * 1024;
/// 软上限允许的最小配置值：16 KiB（与 mcp-tools.yaml configuration.minimum 一致）。
pub const MIN_SOFT_LIMIT_BYTES: usize = 16 * 1024;
/// 配置环境变量名。
pub const ENV_SOFT_LIMIT: &str = "COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES";

/// 解析软上限配置。返回 (生效值, 回退警告)。
/// - None / 空白 → 默认值，无警告；
/// - 合法整数且 ≥ MIN_SOFT_LIMIT_BYTES → 该值；
/// - 非整数 / < MIN_SOFT_LIMIT_BYTES → 回退默认值并给出警告（调用方写 stderr）。
pub fn parse_soft_limit(raw: Option<&str>) -> (usize, Option<String>) {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return (DEFAULT_SOFT_LIMIT_BYTES, None);
    };
    match raw.parse::<usize>() {
        Ok(value) if value >= MIN_SOFT_LIMIT_BYTES => (value, None),
        _ => (
            DEFAULT_SOFT_LIMIT_BYTES,
            Some(format!(
                "{ENV_SOFT_LIMIT}={raw} 非法（须为 ≥ {MIN_SOFT_LIMIT_BYTES} 的整数），\
                 回退默认值 {DEFAULT_SOFT_LIMIT_BYTES}"
            )),
        ),
    }
}

/// 估算 update_diagram diagram 参数的序列化体积（字节）。
/// 与实际上游 PUT body 中的 diagram 字段同口径（serde_json 紧凑序列化）。
pub fn estimate_payload_bytes(diagram: &Value) -> usize {
    serde_json::to_string(diagram)
        .map(|serialized| serialized.len())
        .unwrap_or(usize::MAX)
}

/// 分批建议构成：统计表/关系数量并按主导实体推荐分批工具。
/// - 无表且无关系（纯布局类变更）→ layout_diagram；
/// - 关系多于表 → update_reference；
/// - 其余（含表主导、表关系同数）→ update_table。
pub fn batch_hint(diagram: &Value) -> Value {
    let count = |key: &str| {
        diagram
            .get(key)
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    };
    let tables = count("tables");
    let references = count("references");
    let suggested_batch = if tables == 0 && references == 0 {
        "layout_diagram"
    } else if references > tables {
        "update_reference"
    } else {
        "update_table"
    };
    json!({"tables": tables, "references": references, "suggested_batch": suggested_batch})
}

/// 构造超限错误：code=PAYLOAD_TOO_LARGE、retryable=false，
/// details 含 payload_bytes / soft_limit_bytes / suggestion / batch_hint（UT-MCP-30）。
pub fn payload_too_large_error(
    payload_bytes: usize,
    soft_limit_bytes: usize,
    diagram: &Value,
) -> ToolError {
    let mut error = ToolError::new(
        "PAYLOAD_TOO_LARGE",
        format!(
            "update_diagram 参数体积 {payload_bytes} 字节超过软上限 {soft_limit_bytes} 字节；\
             本次未发送任何写入，请按 details.batch_hint 分批后串行重试"
        ),
        false,
    );
    error.details = Some(json!({
        "payload_bytes": payload_bytes,
        "soft_limit_bytes": soft_limit_bytes,
        "suggestion": "大图请分批写入：按表 update_table、按字段 update_field、\
                       按关系 update_reference、布局 layout_diagram；\
                       每批携带最新 expected_revision 严格串行（并行分批必然 409）",
        "batch_hint": batch_hint(diagram),
    }));
    error
}
