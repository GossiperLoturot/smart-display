fn main() {
    let interval = std::time::Duration::from_secs(1);

    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 480))
        .build(&event_loop)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(window));

    use winit::event::Event;
    use winit::event::StartCause;
    use winit::event::WindowEvent;
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            control_flow.set_wait_timeout(interval);
        }
        Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            renderer.request_redraw();
            control_flow.set_wait_timeout(interval);
        }
        Event::RedrawRequested(window_id) if renderer.match_window(window_id) => {
            renderer.redraw();
        }
        Event::WindowEvent { window_id, event } if renderer.match_window(window_id) => {
            match event {
                WindowEvent::Resized(new_inner_size) => {
                    renderer.resize(new_inner_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(*new_inner_size);
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                _ => {}
            }
        }
        _ => {}
    });
}

struct Renderer {
    window: winit::window::Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    nums_indices: u32,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    staging_belt: wgpu::util::StagingBelt,
    glyph_blush: wgpu_glyph::GlyphBrush<()>,
}

impl Renderer {
    #[rustfmt::skip]
    const VERTICES: &[Vertex] = &[
        Vertex { position: [-1.0, -1.0, 0.0], texcoord: [0.0, 0.0] },
        Vertex { position: [1.0, -1.0, 0.0], texcoord: [1.0, 0.0] },
        Vertex { position: [1.0, 1.0, 0.0], texcoord: [1.0, 1.0] },
        Vertex { position: [-1.0, 1.0, 0.0], texcoord: [0.0, 1.0] },
    ];
    const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

    async fn new(window: winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();
        let surface_capability = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: window.inner_size().width,
            height: window.inner_size().height,
            format: surface_capability.formats[0],
            present_mode: surface_capability.present_modes[0],
            alpha_mode: surface_capability.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // create resource
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Self::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let nums_indices = Self::INDICES.len() as u32;
        let img = image::load_from_memory(include_bytes!("../assets/textures/main.png")).unwrap();
        let texture_size = wgpu::Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1,
        };
        let tex = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &img.to_rgba8(),
        );
        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../assets/shaders/main.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            multiview: None,
        });
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../assets/fonts/Inconsolata-Bold.ttf"
        ))
        .unwrap();
        let glyph_blush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            vertex_buffer,
            index_buffer,
            nums_indices,
            bind_group,
            pipeline,
            staging_belt,
            glyph_blush,
        }
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn redraw(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // draw
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw_indexed(0..self.nums_indices, 0, 0..1);
        drop(pass);
        let utc = chrono::Local::now();
        let date_text = utc.format("%Y/%m/%d %a\n").to_string();
        let time_text = utc.format("%H:%M:%S\n").to_string();
        self.glyph_blush.queue(
            wgpu_glyph::Section::default()
                .add_text(
                    wgpu_glyph::Text::new(&date_text)
                        .with_scale(32.0)
                        .with_color([1.0, 1.0, 1.0, 1.0]),
                )
                .add_text(
                    wgpu_glyph::Text::new(&time_text)
                        .with_scale(128.0)
                        .with_color([1.0, 1.0, 1.0, 1.0]),
                )
                .with_screen_position((
                    frame.texture.width() as f32 * 0.5,
                    frame.texture.height() as f32 * 0.5,
                ))
                .with_layout(
                    wgpu_glyph::Layout::default()
                        .h_align(wgpu_glyph::HorizontalAlign::Center)
                        .v_align(wgpu_glyph::VerticalAlign::Center),
                ),
        );
        self.glyph_blush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                frame.texture.width(),
                frame.texture.height(),
            )
            .unwrap();
        self.staging_belt.finish();

        self.queue.submit([encoder.finish()]);
        frame.present();

        self.staging_belt.recall();
    }

    fn resize(&mut self, new_inner_size: winit::dpi::PhysicalSize<u32>) {
        if 0 < new_inner_size.width && 0 < new_inner_size.height {
            self.config.width = new_inner_size.width;
            self.config.height = new_inner_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn match_window(&self, window_id: winit::window::WindowId) -> bool {
        self.window.id() == window_id
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    texcoord: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: &[wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}
