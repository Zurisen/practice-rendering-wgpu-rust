use crate::renderer_backend::{
    mesh_builder::{self, Mesh, Vertex},
    pipeline_builder,
};

pub struct State<'a> {
    pub window: &'a mut glfw::Window,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub mesh: Mesh,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a mut glfw::PWindow) -> Self {
        // Standard Device and Surface configuration //
        let size = window.get_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.unwrap();
        let surface = unsafe { instance.create_surface_unsafe(target) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("GPU Device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);
        // ------------------------------------ //

        let mesh = mesh_builder::make_mesh(&device);

        // Create Render Pipeline
        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut pipeline_builder = pipeline_builder::PipelineBuilder::new(
                &device,
                "shaders/shader.wgsl",
                "vs_main",
                "fs_main",
                config.format,
            );
            pipeline_builder.add_vertex_buffer_layout(Vertex::desc());
            render_pipeline = pipeline_builder.build_pipeline("Render Pipeline");
        }

        Self {
            window,
            device,
            queue,
            surface,
            config,
            render_pipeline,
            mesh,
        }
    }

    pub fn render(&mut self) {
        let drawable = self.surface.get_current_texture().unwrap();

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        };
        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let image_view_descriptor = wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        };
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.9,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        drawable.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
