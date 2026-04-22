// File: crates/jag-editor/src/protocol.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Priority {
    High,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum InputEvent {
    KeyPress { key: String, code: String },
    MouseScroll { delta_x: f32, delta_y: f32 },
    MouseClick { x: f32, y: f32, button: u8 },
    Resize { width: u32, height: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EditorMessage {
    #[serde(rename = "input")]
    Input {
        #[serde(flatten)]
        event: InputEvent,
        priority: Priority,
    },
    #[serde(rename = "ack")]
    FrameAck {
        frame_id: u64,
        priority: Priority,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameCompression {
    None,
    Lz4,
    Jpeg(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameHeader {
    pub frame_id: u64,
    pub width: u32,
    pub height: u32,
    pub compression: FrameCompression,
    pub timestamp: u64,
}
