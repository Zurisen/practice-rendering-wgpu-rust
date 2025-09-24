use cgmath::Rotation3;
use renderer_backend::instance::Instance;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::renderer_backend::{
    self,
    material::{self, Material},
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
    pub material: Material,
    pub instances: Vec<Instance>,
    pub instances_buffer: wgpu::Buffer,
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

        let mesh_size: f32 = 0.1;
        let mesh = mesh_builder::create_mesh(&device, &mesh_size);
        let material = Material::new(&device, &queue, "textures/texture_diamond.jpg");

        // Create Instances of the mesh
        const NUM_INSTANCES: u16 = 8;

        let mut init_position = -mesh_size * (NUM_INSTANCES - 1) as f32;
        let instances = (0..NUM_INSTANCES)
            .map(|i| {
                let position = cgmath::Vector3 {
                    x: init_position,
                    y: 0.0,
                    z: 0.0,
                };
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    cgmath::Deg(30.0 * (i as f32)),
                );
                init_position = init_position + 2.0 * mesh_size;
                Instance { position, rotation }
            })
            .collect::<Vec<_>>();

        let instances_raw = instances
            .iter()
            .map(|inst| inst.to_raw())
            .collect::<Vec<_>>();

        let instances_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instances Buffer"),
            contents: bytemuck::cast_slice(&instances_raw),
            usage: wgpu::BufferUsages::VERTEX,
        });

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
            pipeline_builder.add_bind_group_layout(&material.bind_group_layout);
            pipeline_builder.add_vertex_buffer_layout(Instance::desc());
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
            material,
            instances,
            instances_buffer,
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
            render_pass.set_vertex_buffer(1, self.instances_buffer.slice(..));
            render_pass.set_bind_group(0, &self.material.bind_group, &[]);
            render_pass.draw_indexed(
                0..self.mesh.num_indices as u32,
                0,
                0..self.instances.len() as u32,
            );
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
