// File: crates/jag-editor/src/lib.rs
pub mod renderer;
pub mod vertex;
pub mod fallback;
pub mod text;
pub mod server;
pub mod protocol;

use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    keyboard::{Key, NamedKey},
};
use crate::renderer::EditorRenderer;

pub async fn run() {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Jag IDE - GPU Editor (Standalone)")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap());

    let mut renderer = EditorRenderer::new(window.clone()).await.expect("Failed to create renderer");

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput {
                    event: key_event,
                    ..
                } => {
                    if key_event.state == ElementState::Pressed && key_event.logical_key == Key::Named(NamedKey::Escape) {
                        elwt.exit();
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                    window.request_redraw();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    let new_inner_size = window.inner_size();
                    renderer.resize(new_inner_size);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    match renderer.render() {
                        Ok(_) => {}
                        Err(e) if e.contains("Lost") => renderer.resize(renderer.size),
                        Err(e) if e.contains("OutOfMemory") => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
