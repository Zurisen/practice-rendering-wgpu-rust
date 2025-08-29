use glfw::{Action, Context, Key, Window, fail_on_errors};
use wgpu::{Backends, Texture, TextureViewDimension};

struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut Window,
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

        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        };

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

        surface.configure(&device, &config);

        Self {
            instance: instance,
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            size: size,
            window: window,
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
                    r: 0.25,
                    g: 0.0,
                    b: 0.5,
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

        command_encoder.begin_render_pass(&render_pass_descriptor);
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
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true);
                }
                _ => {}
            }
        }

        state.render();

        state.window.swap_buffers();
    }
}

fn main() {
    pollster::block_on(run())
}
