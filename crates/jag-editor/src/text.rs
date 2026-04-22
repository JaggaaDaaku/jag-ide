use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextBounds, TextRenderer,
    TextAtlas, Viewport, Cache,
};
use wgpu::{Device, Queue, SurfaceConfiguration};

pub struct EditorTextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub buffer: Buffer,
    pub text_renderer: TextRenderer,
    pub atlas: TextAtlas,
    pub viewport: Viewport,
    pub cache: Cache,
}

impl EditorTextRenderer {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // 1. Initialize Components
        let mut font_system = FontSystem::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, config.format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        // 2. Initialize Text Buffer
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        // 3. Configure initial dimensions
        buffer.set_size(&mut font_system, Some(config.width as f32), Some(config.height as f32));
        let viewport = Viewport::new(device, &cache);
        let swash_cache = SwashCache::new();

        Self {
            font_system,
            swash_cache,
            buffer,
            text_renderer,
            atlas,
            viewport,
            cache,
        }
    }

    pub fn resize(&mut self, _device: &Device, queue: &Queue, width: u32, height: u32) {
        self.buffer.set_size(&mut self.font_system, Some(width as f32), Some(height as f32));
        self.viewport.update(queue, Resolution { width, height });
    }

    pub fn set_text(&mut self, text: &str) {
        // Clear previous buffer state
        self.buffer.lines.clear();

        let mut spans = Vec::new();
        let words = text.split_whitespace();

        for word in words {
            let color = self.token_to_color(word);
            spans.push((
                word.to_string() + " ",
                Attrs::new()
                    .family(Family::Monospace)
                    .color(color),
            ));
        }

        // Note: For multi-line text with highlighting, we should ideally
        // use a more robust parser. For this phase, we just join them.
        self.buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::Monospace).color(Color::rgb(205, 214, 244)),
            Shaping::Advanced,
            None,
        );

        // Advanced: Applying colors to spans within the buffer
        // Note: Buffer::set_text overwrites everything. To do spans, we usually use
        // a specialized layout or update the glyph attributes after shaping.
        // For simplicity in Phase 3.17.2, we will use a single color context
        // and add more granular span support in Phase 4.
    }

    fn token_to_color(&self, token: &str) -> Color {
        match token {
            "fn" | "let" | "mut" | "use" | "pub" | "mod" | "impl" | "struct" | "enum" | "match" => {
                Color::rgb(198, 160, 246) // Mauve (#c6a0f6)
            }
            "true" | "false" | "Some" | "None" | "Ok" | "Err" => {
                Color::rgb(245, 169, 127) // Peach (#f5a97f)
            }
            t if t.contains('(') || t.contains("::") => {
                Color::rgb(139, 213, 202) // Teal (#8bd5ca)
            }
            t if t.starts_with('"') || t.starts_with('\'') => {
                Color::rgb(166, 218, 149) // Green (#a6da95)
            }
            _ => Color::rgb(205, 214, 244), // Text (#cad3f5)
        }
    }

    pub fn draw(&mut self, device: &Device, queue: &Queue, render_pass: &mut wgpu::RenderPass) -> anyhow::Result<()> {
        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 60.0, // Indent for the gutter (50px + 10px padding)
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 10000,
                    bottom: 10000,
                },
                default_color: Color::rgb(205, 214, 244), // Catppuccin Macchiato Text (#cad3f5)
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ).map_err(|e| anyhow::anyhow!("Text preparation failed: {:?}", e))?;
        
        self.text_renderer.render(&self.atlas, &self.viewport, render_pass).map_err(|e| anyhow::anyhow!("Text render failed: {:?}", e))?;

        Ok(())
    }
}
