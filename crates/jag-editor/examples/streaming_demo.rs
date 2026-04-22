// File: crates/jag-editor/examples/streaming_demo.rs
use jag_editor::renderer::EditorRenderer;
use jag_editor::server::StreamingServer;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 1. Initialize Window & Renderer
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Jag IDE - Streaming Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .with_visible(false) // Run headless-ish for demo
        .build(&event_loop)
        .unwrap());

    let mut renderer = EditorRenderer::new(window.clone()).await.unwrap();

    // 2. Start Streaming Server
    let server = Arc::new(StreamingServer::new(9999));
    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.start().await;
    });

    // 3. Main Loop: Render & Stream
    let mut frame_id = 0;
    let mut ticker = interval(Duration::from_millis(16)); // ~60 FPS

    println!("Demo started! Connect to ws://localhost:9999/ws and expect LZ4 frames.");

    loop {
        ticker.tick().await;
        
        // Render
        renderer.render().unwrap();
        
        // Capture & Stream
        if let Ok(payload) = renderer.capture_frame(frame_id).await {
            let _ = server.frame_tx.send(Arc::new(payload));
            frame_id += 1;
            if frame_id % 60 == 0 {
                println!("Streamed 60 frames...");
            }
        }
    }
}
