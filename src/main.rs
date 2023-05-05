mod picture;
mod text;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Refresh rate [ms]
    #[arg(long, default_value = "1000")]
    interval: u64,
    /// Window width
    #[arg(long, default_value = "800")]
    width: u32,
    /// Window height
    #[arg(long, default_value = "480")]
    height: u32,
    /// Path representing background picture directory
    #[arg(long, default_value = "pictures")]
    path: String,
    /// A time until shuffling background picture [s]
    #[arg(long, default_value = "3600")]
    pic_interval: u64,
}

fn main() {
    use clap::Parser;
    let args = Args::parse();

    let interval = std::time::Duration::from_millis(args.interval);
    let mut instance = std::time::Instant::now();

    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(args.width, args.height))
        .build(&event_loop)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(window));
    let mut rng = rand::thread_rng();
    renderer.set_picture(choise_pic(&args.path, &mut rng));

    use winit::event::Event;
    use winit::event::StartCause;
    use winit::event::WindowEvent;
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            control_flow.set_wait_timeout(interval);
        }
        Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            if args.pic_interval < instance.elapsed().as_secs() {
                renderer.set_picture(choise_pic(&args.path, &mut rng));
                instance = std::time::Instant::now();
            }
            renderer.request_redraw();
            control_flow.set_wait_timeout(interval);
        }
        Event::RedrawRequested(window_id) if renderer.match_window(window_id) => {
            renderer.draw();
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
    picture_pipeline: crate::picture::PicturePipeline,
    text_pipeline: crate::text::TextPipeline,
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
        let inner_size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface.configure(&device, &config);

        let picture_pipeline = picture::PicturePipeline::new(&device, config.format, 800, 480);
        let text_pipeline =
            text::TextPipeline::new(&device, config.format, config.width, config.height);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            picture_pipeline,
            text_pipeline,
        }
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn draw(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.picture_pipeline.draw(&self.device, &self.queue, &view);
        self.text_pipeline.draw(&self.device, &self.queue, &view);

        frame.present();
    }

    fn set_picture(&mut self, img: image::DynamicImage) {
        self.picture_pipeline.set_image(&self.queue, img);
    }

    fn resize(&mut self, new_inner_size: winit::dpi::PhysicalSize<u32>) {
        if 0 < new_inner_size.width && 0 < new_inner_size.height {
            self.config.width = new_inner_size.width;
            self.config.height = new_inner_size.height;
            self.surface.configure(&self.device, &self.config);
            self.text_pipeline
                .resize(new_inner_size.width, new_inner_size.height);
        }
    }

    fn match_window(&self, window_id: winit::window::WindowId) -> bool {
        self.window.id() == window_id
    }
}

fn choise_pic(path: &str, rng: &mut impl rand::Rng) -> image::DynamicImage {
    use rand::seq::IteratorRandom;

    let entry = std::fs::read_dir(path)
        .unwrap()
        .choose(rng)
        .unwrap()
        .unwrap();

    let file = std::fs::File::open(entry.path()).unwrap();
    let reader = std::io::BufReader::new(file);

    image::load(reader, image::ImageFormat::Png).unwrap()
}
