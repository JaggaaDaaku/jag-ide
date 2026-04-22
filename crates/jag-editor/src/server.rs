// File: crates/jag-editor/src/server.rs
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use axum::body::Bytes;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::protocol::{EditorMessage, FrameHeader, FrameCompression};
use lz4_flex::compress_prepend_size;
use serde_json;

pub struct StreamingServer {
    pub frame_tx: broadcast::Sender<Arc<Vec<u8>>>,
    pub port: u16,
}

impl StreamingServer {
    pub fn new(port: u16) -> Self {
        let (frame_tx, _) = broadcast::channel(10);
        Self { frame_tx, port }
    }

    pub async fn start(self: Arc<Self>) {
        let server = self.clone();
        let app = Router::new().route("/ws", get(move |ws| handle_ws(ws, server.clone())));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .unwrap();
        tracing::info!("Streaming server listening on ws://localhost:{}", self.port);
        axum::serve(listener, app).await.unwrap();
    }
}

async fn handle_ws(ws: WebSocketUpgrade, server: Arc<StreamingServer>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server))
}

async fn handle_socket(socket: WebSocket, server: Arc<StreamingServer>) {
    let (mut sink, mut stream) = socket.split();
    let mut frame_rx = server.frame_tx.subscribe();

    tokio::select! {
        // Send frames to client
        _ = async {
            while let Ok(frame) = frame_rx.recv().await {
                if sink.send(Message::Binary(Bytes::from(frame.to_vec()))).await.is_err() {
                    break;
                }
            }
        } => (),
        // Receive events from client
        _ = async {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    #[allow(clippy::collapsible_if)]
                    if let Ok(editor_msg) = serde_json::from_str::<EditorMessage>(&text) {
                        tracing::info!("AI completion generated: {:?}", editor_msg);
                        // Forward to renderer event queue
                    }
                }
            }
        } => (),
    }
}

pub fn prepare_frame_payload(frame_id: u64, width: u32, height: u32, rgba_pixels: &[u8]) -> Vec<u8> {
    let compressed = compress_prepend_size(rgba_pixels);
    let header = FrameHeader {
        frame_id,
        width,
        height,
        compression: FrameCompression::Lz4,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    
    let header_json = serde_json::to_string(&header).unwrap();
    let header_bytes = header_json.as_bytes();
    
    let mut payload = Vec::with_capacity(4 + header_bytes.len() + compressed.len());
    payload.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    payload.extend_from_slice(header_bytes);
    payload.extend_from_slice(&compressed);
    payload
}
