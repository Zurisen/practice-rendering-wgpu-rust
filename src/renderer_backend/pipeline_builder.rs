use std::env::current_dir;
use std::path::PathBuf;
use std::{default, fs};

use wgpu::PipelineCompilationOptions;

pub struct PipelineBuilder {
    shader_filename: String,
    vertex_entry: String,
    fragment_entry: String,
    pixel_format: wgpu::TextureFormat,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        PipelineBuilder {
            shader_filename: "dummy".to_string(),
            vertex_entry: "dummy".to_string(),
            fragment_entry: "dummy".to_string(),
            pixel_format: wgpu::TextureFormat::Rgba8Unorm,
            vertex_buffer_layouts: Vec::new(),
        }
    }

    pub fn add_buffer_layout(&mut self, layout: wgpu::VertexBufferLayout<'static>) {
        self.vertex_buffer_layouts.push(layout);
    }

    pub fn set_shader_module(
        &mut self,
        shader_filename: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) {
        self.shader_filename = shader_filename.to_string();
        self.vertex_entry = vertex_entry.to_string();
        self.fragment_entry = fragment_entry.to_string();
    }

    pub fn set_pixel_format(&mut self, pixel_format: wgpu::TextureFormat) {
        self.pixel_format = pixel_format;
    }

    pub fn build_pipeline(&mut self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let mut filepath: PathBuf = current_dir().unwrap();
        filepath.push("src/");
        filepath.push(self.shader_filename.as_str());
        let filepath: String = filepath.into_os_string().into_string().unwrap();
        let source_code: String = fs::read_to_string(filepath).expect("Cant read source code");

        let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(source_code.into()),
        };

        let shader_module = device.create_shader_module(shader_module_descriptor);

        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };

        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

        let render_targets = [Some(wgpu::ColorTargetState {
            format: self.pixel_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_descripter = wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some(&self.vertex_entry),
                buffers: &self.vertex_buffer_layouts, // we pass custom vertex buffer layout to not hardcode into the wgls
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some(&self.fragment_entry),
                targets: &render_targets,
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        };

        device.create_render_pipeline(&render_pipeline_descripter)
    }
}
