use std::process::ExitCode;

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::protocol;
use coldrawdb_mcp::{Config, McpService};
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}: {}", error.code, error.message);
            return ExitCode::from(2);
        }
    };
    // diagram-api-auth：后端开启强制鉴权（COLDRAWDB_DIAGRAMS_AUTH=on）时，
    // 未配置 COLDRAWDB_ACCESS_TOKEN 会让全部工具调用收到上游 401 —— 启动期探测并明确报错。
    // 探测口径：匿名 GET /api/v1/diagrams/{不存在 id}，flag=on → 401，flag=off → 404。
    // 后端离线/网络异常不阻断启动（工具调用期再按上游错误如实上报）。
    if config.access_token.is_none() {
        let auth_forced = match config.endpoint("/api/v1/diagrams/__mcp_probe__") {
            Ok(url) => match reqwest::Client::builder().no_proxy().build() {
                Ok(client) => client
                    .get(url.as_str())
                    .timeout(config.timeout)
                    .send()
                    .await
                    .map(|resp| resp.status() == reqwest::StatusCode::UNAUTHORIZED)
                    .unwrap_or(false),
                Err(_) => false,
            },
            Err(_) => false,
        };
        if auth_forced {
            eprintln!("CONFIG_INVALID: 后端已开启强制鉴权（COLDRAWDB_DIAGRAMS_AUTH=on），必须配置 COLDRAWDB_ACCESS_TOKEN（POST /api/v1/auth/login 签发的 accessToken）");
            return ExitCode::from(2);
        }
    }
    let api = match ApiClient::new(config) {
        Ok(api) => api,
        Err(error) => {
            eprintln!("{}: {}", error.code, error.message);
            return ExitCode::from(2);
        }
    };
    let service = McpService::new(api);
    let stdin = BufReader::new(tokio::io::stdin());
    if let Err(error) = protocol::serve(service, stdin, tokio::io::stdout()).await {
        eprintln!("INTERNAL_ERROR: stdio 失败: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
