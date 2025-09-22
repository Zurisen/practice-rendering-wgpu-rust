pub struct PipelineBuilder<'a> {
    pub device: &'a wgpu::Device,
    pub shader_path: String,
    pub vertex_entry: String,
    pub fragment_entry: String,
    pub pixel_format: wgpu::TextureFormat,
    pub bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        device: &'a wgpu::Device,
        shader_path: &str,
        vertex_entry: &str,
        fragment_entry: &str,
        pixel_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            device: device,
            shader_path: shader_path.to_string(),
            vertex_entry: vertex_entry.to_string(),
            fragment_entry: fragment_entry.to_string(),
            pixel_format: pixel_format,
            bind_group_layouts: Vec::new(),
            vertex_buffer_layouts: Vec::new(),
        }
    }

    pub fn add_bind_group_layout(&mut self, layout: &'a wgpu::BindGroupLayout) {
        self.bind_group_layouts.push(layout);
    }

    pub fn add_vertex_buffer_layout(&mut self, layout: wgpu::VertexBufferLayout<'static>) {
        self.vertex_buffer_layouts.push(layout);
    }

    pub fn build_pipeline(&mut self, label: &str) -> wgpu::RenderPipeline {
        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout descriptor"),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[],
        };
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&pipeline_layout_descriptor);

        let mut filepath = std::env::current_dir().unwrap();
        filepath.push("src/");
        filepath.push(self.shader_path.as_str());
        let filepath: String = filepath.into_os_string().into_string().unwrap();
        let source_code = std::fs::read_to_string(filepath).expect("No shader found in that path");
        let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some("Shader module descriptor"),
            source: wgpu::ShaderSource::Wgsl(source_code.into()),
        };
        let shader_module = self.device.create_shader_module(shader_module_descriptor);

        let render_targets = [Some(wgpu::ColorTargetState {
            format: self.pixel_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some(&self.vertex_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &self.vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some(&self.fragment_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &render_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        };
        let render_pipeline = self
            .device
            .create_render_pipeline(&render_pipeline_descriptor);

        render_pipeline
    }
}
