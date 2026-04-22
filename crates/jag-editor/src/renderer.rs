use std::sync::Arc;
use tokio::sync::RwLock;
use winit::window::Window;
use crate::vertex::Vertex;
use crate::fallback::SoftwareRenderer;
use crate::text::EditorTextRenderer;
use crate::server::prepare_frame_payload;
use wgpu::util::DeviceExt;

pub enum RenderBackend {
    Gpu {
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        pipeline: wgpu::RenderPipeline,
        vertex_buffer: Arc<RwLock<wgpu::Buffer>>,
        text_renderer: Box<EditorTextRenderer>,
    },
    Cpu {
        software_renderer: SoftwareRenderer,
    },
}

pub struct EditorRenderer {
    backend: RenderBackend,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
}

impl EditorRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        // 1. Try Initialize GPU
        match Self::init_gpu(window.clone()).await {
            Ok((device, queue, surface, config, pipeline, vertex_buffer, text_renderer)) => {
                tracing::info!("GPU Backend initialized successfully");
                Ok(Self {
                    backend: RenderBackend::Gpu {
                        device,
                        queue,
                        surface,
                        config,
                        pipeline,
                        vertex_buffer,
                        text_renderer: Box::new(text_renderer),
                    },
                    size,
                    scale_factor,
                })
            }
            Err(e) => {
                tracing::warn!("GPU initialization failed: {}. Falling back to CPU.", e);
                Ok(Self {
                    backend: RenderBackend::Cpu {
                        software_renderer: SoftwareRenderer::new(size.width, size.height),
                    },
                    size,
                    scale_factor,
                })
            }
        }
    }

    async fn init_gpu(
        window: Arc<Window>,
    ) -> Result<
        (
            wgpu::Device,
            wgpu::Queue,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
            wgpu::RenderPipeline,
            Arc<RwLock<wgpu::Buffer>>,
            EditorTextRenderer,
        ),
        String,
    > {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).map_err(|e| e.to_string())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| "No suitable GPU adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Jag Editor Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    experimental_features: Default::default(),
                    trace: Default::default(),
                }
            )
            .await
            .map_err(|e| e.to_string())?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Load Shaders
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Jag Editor Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
        });

        let vertex_buffer = Arc::new(RwLock::new(Self::create_gutter_buffer(
            &device,
            size.width,
            window.scale_factor() as f32,
        )));

        // Initialize Text Renderer
        let mut text_renderer = EditorTextRenderer::new(&device, &queue, &config);
        text_renderer.set_text("fn main() {\n    let msg = \"Hello Jag IDE!\";\n    println!(\"{}\", msg);\n}");

        Ok((device, queue, surface, config, pipeline, vertex_buffer, text_renderer))
    }

    fn create_gutter_buffer(device: &wgpu::Device, width: u32, scale_factor: f32) -> wgpu::Buffer {
        let gutter_width_logical = 50.0;
        let gutter_width_ndc = (gutter_width_logical * scale_factor * 2.0 / width as f32) - 1.0;
        
        let vertices = [
            Vertex { position: [-1.0, 1.0, 0.0], color: [0.118, 0.125, 0.188] }, 
            Vertex { position: [gutter_width_ndc, 1.0, 0.0], color: [0.118, 0.125, 0.188] },
            Vertex { position: [-1.0, -1.0, 0.0], color: [0.118, 0.125, 0.188] },
            
            Vertex { position: [-1.0, -1.0, 0.0], color: [0.118, 0.125, 0.188] },
            Vertex { position: [gutter_width_ndc, 1.0, 0.0], color: [0.118, 0.125, 0.188] },
            Vertex { position: [gutter_width_ndc, -1.0, 0.0], color: [0.118, 0.125, 0.188] },
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            match &mut self.backend {
                RenderBackend::Gpu { device, queue, surface, config, vertex_buffer, text_renderer, .. } => {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(device, config);
                    
                    let new_vb = Self::create_gutter_buffer(device, new_size.width, self.scale_factor as f32);
                    if let Ok(mut vb) = vertex_buffer.try_write() {
                        *vb = new_vb;
                    }

                    text_renderer.resize(device, queue, new_size.width, new_size.height);
                }
                RenderBackend::Cpu { software_renderer } => {
                    software_renderer.resize(new_size.width, new_size.height);
                }
            }
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.resize(self.size);
    }

    pub async fn capture_frame(&mut self, frame_id: u64) -> Result<Vec<u8>, String> {
        if let RenderBackend::Gpu { device, queue, surface, config, .. } = &self.backend {
            let output = match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    surface.configure(device, config);
                    return Err("Surface outdated/lost during capture".to_string());
                }
                _ => return Err("Surface unavailable during capture".to_string()),
            };
            let texture = &output.texture;
            let width = texture.width();
            let height = texture.height();

            let bytes_per_pixel = 4;
            let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let unpadded_bytes_per_row = width * bytes_per_pixel;
            let padding = (alignment - unpadded_bytes_per_row % alignment) % alignment;
            let padded_bytes_per_row = unpadded_bytes_per_row + padding;

            let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: (padded_bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Capture Encoder"),
            });

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            queue.submit(std::iter::once(encoder.finish()));

            let buffer_slice = staging_buffer.slice(..);
            let (tx, rx) = tokio::sync::oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
                tx.send(v).ok();
            });

            device.poll(wgpu::PollType::Poll).map_err(|e| e.to_string())?;
            rx.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

            let data = buffer_slice.get_mapped_range();
            let mut result = Vec::with_capacity((width * height * 4) as usize);
            
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                result.extend_from_slice(&data[start..end]);
            }

            drop(data);
            staging_buffer.unmap();
            
            let payload = prepare_frame_payload(frame_id, width, height, &result);
            Ok(payload)
        } else {
            Err("Capture only supported on GPU backend".to_string())
        }
    }

    pub fn render(&mut self) -> Result<(), String> {
        match &mut self.backend {
            RenderBackend::Gpu { device, queue, surface, pipeline, vertex_buffer, text_renderer, config, .. } => {
                let output = match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) => t,
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(device, config);
                        return Ok(());
                    }
                    _ => return Ok(()),
                };
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                // Encoder 1: Geometry Pass (Gutter)
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let vb_guard = vertex_buffer.try_read();
                    
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.141, // #24273a (Catppuccin Macchiato Base)
                                    g: 0.153,
                                    b: 0.227,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                        multiview_mask: None,
                    });

                    render_pass.set_pipeline(pipeline);
                    if let Ok(vb) = &vb_guard {
                        render_pass.set_vertex_buffer(0, vb.slice(..));
                        render_pass.draw(0..6, 0..1);
                    }
                }

                // Encoder 2: Text Pass
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Text Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                        multiview_mask: None,
                    });

                    text_renderer.draw(device, queue, &mut render_pass).map_err(|e| e.to_string())?;
                }

                queue.submit(std::iter::once(encoder.finish()));
                output.present();
                
                Ok(())
            }
            RenderBackend::Cpu { software_renderer } => {
                let mut _software_renderer = software_renderer;
                Ok(())
            }
        }
    }
}
