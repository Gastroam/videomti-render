use super::geometry::VideoVertex;
use wgpu::{Device, PipelineLayout, RenderPipeline, TextureFormat};

pub struct CompositionPipeline {
    pub pipeline: RenderPipeline,
    pub layout: PipelineLayout,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl CompositionPipeline {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        // 1. Shaders
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // 2. Bind Group Layouts
        // Group 0: Uniforms (Transform + Opacity)
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, // crevice size
                    },
                    count: None,
                }],
            });

        // Group 1: Texture + Sampler
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
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

        // 3. Pipeline Layout
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Composition Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            // Error said "missing immediate_size". wgpu 0.28.
            immediate_size: 0,
        });

        // 4. Render Pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Composition Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[VideoVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(), // Simplification
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            layout,
            uniform_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}
