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
