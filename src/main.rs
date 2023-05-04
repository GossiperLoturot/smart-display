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
    staging_belt: wgpu::util::StagingBelt,
    glyph_blush: wgpu_glyph::GlyphBrush<()>,
}

impl Renderer {
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
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../assets/fonts/Inconsolata-Bold.ttf"
        ))
        .unwrap();
        let glyph_blush = wgpu_glyph::GlyphBrushBuilder::using_font(font)
            .build(&device, surface_capability.formats[0]);

        Self {
            window,
            surface,
            device,
            queue,
            config,
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
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
