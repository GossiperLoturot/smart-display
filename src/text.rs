pub struct TextPipeline {
    staging_belt: wgpu::util::StagingBelt,
    glyph_blush: wgpu_glyph::GlyphBrush<()>,
    target_width: u32,
    target_height: u32,
}

impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        target_width: u32,
        target_height: u32,
    ) -> Self {
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../assets/fonts/Inconsolata-Bold.ttf"
        ))
        .unwrap();
        let glyph_blush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font).build(device, target_format);

        Self {
            staging_belt,
            glyph_blush,
            target_width,
            target_height,
        }
    }

    pub fn resize(&mut self, target_width: u32, target_height: u32) {
        self.target_width = target_width;
        self.target_height = target_height;
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.staging_belt.recall();

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
                    self.target_width as f32 * 0.5,
                    self.target_height as f32 * 0.5,
                ))
                .with_layout(
                    wgpu_glyph::Layout::default()
                        .h_align(wgpu_glyph::HorizontalAlign::Center)
                        .v_align(wgpu_glyph::VerticalAlign::Center),
                ),
        );
        self.glyph_blush
            .draw_queued(
                device,
                &mut self.staging_belt,
                encoder,
                view,
                self.target_width,
                self.target_height,
            )
            .unwrap();

        self.staging_belt.finish();
    }
}
