use cgmath::prelude::*;
use glfw::{Action, Context, Key, Window, fail_on_errors};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgc::instance;
mod controls;
mod renderer_backend;

use crate::controls::camera_controller::CameraController;
use crate::renderer_backend::bind_group_layout;
use crate::renderer_backend::camera::{Camera, CameraUniform};
use crate::renderer_backend::instance::Instance;
use crate::renderer_backend::instance::Instance as CustomInstance;
use crate::renderer_backend::material::Material;
use crate::renderer_backend::mesh_builder::{self, Mesh};
use crate::renderer_backend::pipeline_builder::PipelineBuilder;

struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut Window,
    render_pipeline: wgpu::RenderPipeline,
    mesh: Mesh,
    material: Material,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_controller: CameraController,
    camera_bind_group: wgpu::BindGroup, // TODO: Camera class with all the functionality
    instances: Vec<CustomInstance>,
    instance_buffer: wgpu::Buffer,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut Window) -> Self {
        let size = window.get_size();
        let instance_descriptor = wgpu::InstanceDescriptor::default();

        let instance = wgpu::Instance::new(&instance_descriptor);
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.unwrap();
        let surface = unsafe { instance.create_surface_unsafe(target) }.unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        // Set adapter, represents a physical or virtual GPU (graphics device) available on your system.
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        };

        // with the adapter abstaction we get the Device: This is a logical connection to the GPU. and Queue: This is how you send (submit) commands buffers to the GPU for execution.
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

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
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // configure drawable surface (the "frame canvas")
        surface.configure(&device, &config);

        // build a mesh (for dummy, a quad)
        let mesh = mesh_builder::make_quad(&device);

        // build bind group layouts for material textures
        let material_bind_group_layout: wgpu::BindGroupLayout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        // Camera bind group and bind group layouts creation
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Instances
        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;
                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    CustomInstance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // build the pipeline, setting the shader module, pixel format and buffer layouts
        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = PipelineBuilder::new(&device);
            builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_buffer_layout(CustomInstance::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&camera_bind_group_layout);
            render_pipeline = builder.build_pipeline("Render Pipeline");
        }

        let quad_material = Material::new(
            "img/texture_diamond.jpg",
            &device,
            &queue,
            &material_bind_group_layout,
        );
        let camera_controller = CameraController::new(0.05);

        // Buffer layouts describe how your vertex data is structured in memory (e.g., position, color, stride, offsets).
        Self {
            instance: instance,
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            size: size,
            window: window,
            render_pipeline: render_pipeline,
            mesh: mesh,
            material: quad_material,
            camera: camera,
            camera_uniform: camera_uniform,
            camera_buffer: camera_buffer,
            camera_controller: camera_controller,
            camera_bind_group: camera_bind_group,
            instances: instances,
            instance_buffer: instance_buffer,
        }
    }

    fn render(&mut self) {
        let drawable = self.surface.get_current_texture().unwrap();
        let image_view_descriptor = wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        };

        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };

        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.1,
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
            render_pass.set_bind_group(0, &self.material.bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..self.instances.len() as _);
            //render_pass.draw(0..6, 0..1); // only if not using index optimization
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();
    }

    fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = self.size.0 as u32;
            self.config.height = self.size.1 as u32;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_camera(&mut self) {
        // Update camera position based on controller
        self.camera_controller.update_camera(&mut self.camera);

        // Update the camera uniform and write to GPU
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
    fn update_surface(&mut self) {
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&self.window) }.unwrap();

        self.surface = unsafe { self.instance.create_surface_unsafe(target) }.unwrap();
    }
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "WGPU Project", glfw::WindowMode::Windowed)
        .unwrap();

    let mut state = State::new(&mut window).await;

    state.window.set_key_polling(true);
    state.window.make_current();

    while !state.window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            state.camera_controller.process_events(&event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true);
                }
                _ => {}
            }
        }

        state.update_camera();
        state.render();

        state.window.swap_buffers();
    }
}

// Bind group layout builder --> Builder struct (bind_group_layout.rs)
// Pipeline builder takes bind group layout
// Engine has bind group layout and pipeline uses it
// Bind group builder --> Builder struct (bind_group.rs)
// Texture class
// Engine has textures
// Shader has textures
fn main() {
    pollster::block_on(run())
}
