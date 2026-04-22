// File: crates/jag-editor/src/fallback.rs
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

pub struct SoftwareRenderer {
    pixmap: Pixmap,
}

impl SoftwareRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = Pixmap::new(width.max(1), height.max(1)).unwrap();
        Self { pixmap }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(new_pixmap) = Pixmap::new(width.max(1), height.max(1)) {
            self.pixmap = new_pixmap;
        }
    }

    pub fn render(&mut self) {
        // Clear to #24273a (Catppuccin Macchiato Base)
        self.pixmap.fill(Color::from_rgba8(36, 39, 58, 255));

        // Draw Gutter Background
        let mut paint = Paint::default();
        paint.set_color_rgba8(30, 32, 48, 255); // Slightly darker for gutter
        
        let gutter_rect = Rect::from_xywh(0.0, 0.0, 50.0, self.pixmap.height() as f32).unwrap();
        self.pixmap.fill_rect(gutter_rect, &paint, Transform::identity(), None);
    }

    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }
}
