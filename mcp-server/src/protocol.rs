use serde_json::{json, Value};
use std::time::Instant;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::ToolError;
use crate::service::McpService;

const SERVER_INSTRUCTIONS: &str = "仅操作 COLDRAWDB_BASE_URL 指向的 coldrawdb。修改前先读取最新 revision；delete_diagram 具有破坏性并需用户批准；禁止任意 SQL、文件或通用 HTTP 访问。";

pub async fn handle(service: &McpService, request: Value) -> Option<Value> {
    let id = request.get("id").cloned()?;
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": request.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or("2025-06-18"),
            "capabilities": {"tools":{"listChanged":false}},
            "serverInfo": {"name":"coldrawdb-mcp","title":"coldrawdb MCP","version":env!("CARGO_PKG_VERSION")},
            "instructions": SERVER_INSTRUCTIONS
        })),
        "ping" => Ok(json!({})),
        "tools/list" => McpService::tools().map(|tools| json!({"tools":tools})),
        "tools/call" => {
            let name = request
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            let arguments = request
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let started = Instant::now();
            let outcome = service.call(name, arguments).await;
            let status = if outcome.is_ok() { "ok" } else { "error" };
            eprintln!(
                "{}",
                json!({"event":"tool_call","tool":name,"duration_ms":started.elapsed().as_millis(),"status":status})
            );
            match outcome {
                Ok(value) => Ok(tool_result(value, false)),
                Err(error) => Ok(tool_result(
                    serde_json::to_value(error)
                        .unwrap_or_else(|_| json!({"code":"INTERNAL_ERROR"})),
                    true,
                )),
            }
        }
        _ => {
            return Some(
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not found"}}),
            )
        }
    };
    Some(match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(error) => protocol_error(Some(id), error),
    })
}

fn tool_result(value: Value, is_error: bool) -> Value {
    json!({"content":[{"type":"text","text":serde_json::to_string(&value).unwrap_or_default()}],"structuredContent":value,"isError":is_error})
}

fn protocol_error(id: Option<Value>, error: ToolError) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32603,"message":error.message,"data":error}})
}

pub async fn serve<R, W>(service: McpService, reader: R, mut writer: W) -> std::io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&service, request).await,
            Err(_) => Some(
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            ),
        };
        if let Some(response) = response {
            writer
                .write_all(
                    serde_json::to_string(&response)
                        .unwrap_or_default()
                        .as_bytes(),
                )
                .await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
    }
    Ok(())
}
