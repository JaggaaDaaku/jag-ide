use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use actix_ws::Message;
use jag_core::types::WorkspaceId;
use std::time::{Duration, Instant};
use tracing::{error, info, debug};
use futures_util::{StreamExt as _, SinkExt as _};

use crate::AppState;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[get("/api/workspaces/{id}/sync")]
pub async fn workspace_sync(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let workspace_id = path.into_inner();
    
    // In a real implementation, we'd verify the auth token from query params or a ticket
    // For now, we just upgrade the connection.

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let mut rx = state.broadcast_tx.subscribe();
    
    actix_web::rt::spawn(async move {
        info!("WebSocket connection established for workspace {}", workspace_id);
        
        let mut last_heartbeat = Instant::now();
        let mut interval = actix_web::rt::time::interval(HEARTBEAT_INTERVAL);

        loop {
            tokio::select! {
                // Heartbeat
                _ = interval.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                        info!("WebSocket client heartbeat failed, disconnecting!");
                        break;
                    }
                    
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
                
                // Broadcast messages from other parts of the app
                Ok(msg) = rx.recv() => {
                    // Send to client
                    if let Ok(text) = serde_json::to_string(&msg) {
                        if session.text(text).await.is_err() {
                            break;
                        }
                    }
                }
                
                // Messages from client
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        Message::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Message::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                        Message::Text(text) => {
                            debug!("Received WS message: {}", text);
                            // Here we'd handle cursor movement, artifact updates etc.
                        }
                        Message::Close(reason) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }
                
                else => break,
            }
        }
        
        info!("WebSocket connection closed for workspace {}", workspace_id);
    });

    Ok(response)
}
