//! 协作 op 的发送循环。传输层只需要收发文本帧，便于用假 socket 测试。
//! 落地时用 tokio-tungstenite 实现 CollabSocket。

use serde_json::Value;

use super::{build_op_frame, interpret_frame, Inbound};

pub trait CollabSocket {
    fn send_text(&mut self, text: String) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn recv_text(&mut self) -> impl std::future::Future<Output = Result<Option<String>, String>> + Send;
}

pub async fn submit_ops<S: CollabSocket>(
    socket: &mut S,
    ops: &[Value],
    current_rev: i64,
) -> Result<i64, (String, String)> {
    if ops.is_empty() {
        return Ok(current_rev);
    }
    let mut last = current_rev;
    for (index, op) in ops.iter().enumerate() {
        let frame = build_op_frame((index as i64) + 1, op);
        socket
            .send_text(frame.to_string())
            .await
            .map_err(|message| ("UPSTREAM_UNAVAILABLE".into(), message))?;
        loop {
            let text = socket
                .recv_text()
                .await
                .map_err(|message| ("UPSTREAM_UNAVAILABLE".into(), message))?
                .ok_or_else(|| ("UPSTREAM_ERROR".into(), "协作通道在 ack 前关闭".into()))?;
            let frame: Value = serde_json::from_str(&text).map_err(|_| {
                ("UPSTREAM_ERROR".into(), "协作通道返回了非 JSON 帧".into())
            })?;
            match interpret_frame(&frame) {
                Inbound::Ignore => continue,
                Inbound::Ack { server_rev } => {
                    last = server_rev;
                    break;
                }
                Inbound::Failed { code, message } => return Err((code, message)),
            }
        }
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::VecDeque;

    struct Script {
        inbound: VecDeque<String>,
        sent: Vec<String>,
    }

    impl CollabSocket for Script {
        async fn send_text(&mut self, text: String) -> Result<(), String> {
            self.sent.push(text);
            Ok(())
        }
        async fn recv_text(&mut self) -> Result<Option<String>, String> {
            Ok(self.inbound.pop_front())
        }
    }

    #[tokio::test]
    async fn sends_one_op_and_reads_ack_after_connected() {
        let mut socket = Script {
            inbound: VecDeque::from([
                json!({"type":"connected","serverRev":2}).to_string(),
                json!({"type":"ack","serverRev":3}).to_string(),
            ]),
            sent: Vec::new(),
        };
        let rev = submit_ops(
            &mut socket,
            &[json!({"type":"table.update","targetId":"t1","changes":{"name":"a"}})],
            2,
        )
        .await
        .unwrap();
        assert_eq!(rev, 3);
        assert_eq!(socket.sent.len(), 1);
        let sent: Value = serde_json::from_str(&socket.sent[0]).unwrap();
        assert_eq!(sent["type"], "op");
        assert_eq!(sent["clientRev"], 1);
    }

    #[tokio::test]
    async fn read_only_stops_without_further_sends() {
        let mut socket = Script {
            inbound: VecDeque::from([
                json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"}).to_string(),
            ]),
            sent: Vec::new(),
        };
        let err = submit_ops(&mut socket, &[json!({"type":"table.update","targetId":"t1"})], 1)
            .await
            .unwrap_err();
        assert_eq!(err.0, "READ_ONLY");
        assert_eq!(socket.sent.len(), 1);
    }
}
