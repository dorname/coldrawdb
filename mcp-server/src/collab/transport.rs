//! tokio-tungstenite 传输。落地到 mcp-server 时与 session::submit_ops 对接。

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

use super::redact_transport_error;
use super::session::{submit_ops, CollabSocket};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct LiveSocket {
    write: futures_util::stream::SplitSink<WsStream, Message>,
    read: futures_util::stream::SplitStream<WsStream>,
}

impl CollabSocket for LiveSocket {
    async fn send_text(&mut self, text: String) -> Result<(), String> {
        self.write
            .send(Message::Text(text.into()))
            .await
            .map_err(|error| error.to_string())
    }

    async fn recv_text(&mut self) -> Result<Option<String>, String> {
        loop {
            match self.read.next().await {
                None => return Ok(None),
                Some(Ok(Message::Text(text))) => return Ok(Some(text.to_string())),
                Some(Ok(Message::Close(_))) => return Ok(None),
                Some(Ok(_)) => continue,
                Some(Err(error)) => return Err(error.to_string()),
            }
        }
    }
}

pub async fn submit_over_websocket(
    ws_url: &str,
    ops: &[Value],
    current_rev: i64,
) -> Result<i64, (String, String)> {
    let (stream, _) = connect_async(ws_url).await.map_err(|error| {
        (
            "UPSTREAM_UNAVAILABLE".to_string(),
            redact_transport_error(ws_url, &error.to_string()),
        )
    })?;
    let (write, read) = stream.split();
    let mut socket = LiveSocket { write, read };
    submit_ops(&mut socket, ops, current_rev)
        .await
        .map_err(|(code, message)| (code, redact_transport_error(ws_url, &message)))
}
