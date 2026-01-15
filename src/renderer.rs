use crate::core::RenderContext;
use crate::model::FrameDescription;
use crate::outputs::RenderSink;
use crate::pipeline::{CompositionPipeline, LayerUniforms, QUAD_INDICES, QUAD_VERTICES};
use crate::resources::TextureManager;
use crevice::std140::AsStd140;
pub use glam::Mat4; // Exposed for internal use, though tests should use glam dependency directly
use wgpu::util::DeviceExt;
use wgpu::{CommandEncoder, Device, Queue, TextureView};

pub struct Renderer {
    pipeline: CompositionPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
}

impl Renderer {
    pub fn new(context: &RenderContext) -> Self {
        // Init pipeline
        let format = wgpu::TextureFormat::Rgba8Unorm; // Match BufferSink
        let pipeline = CompositionPipeline::new(&context.device, format);

        // Init geometry buffers
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Index Buffer"),
                contents: bytemuck::cast_slice(QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Init Sampler (Linear for smooth scaling)
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest, // Wait, error said `expected MipmapFilterMode, found FilterMode`?
            // In wgpu 0.28, is it MipmapFilterMode? Or did I use FilterMode enum?
            // Usually it is FilterMode. Let's check if there is a separate enum.
            // `wgpu::FilterMode` has `Nearest`, `Linear`.
            // If error says `expected MipmapFilterMode`, then wgpu 0.28 split it.
            // Let's try `wgpu::MipmapFilterMode::Nearest`.

            // Also fixing RenderPassDescriptor missing multiview_mask
            ..Default::default()
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            sampler,
        }
    }

    pub fn render(
        &mut self,
        context: &RenderContext,
        texture_manager: &TextureManager,
        composition: &FrameDescription,
        sink: &mut dyn RenderSink,
    ) -> Result<(), crate::core::RenderError> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let output_view = sink.prepare_frame()?;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Composition Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: composition.background_color[0] as f64,
                            g: composition.background_color[1] as f64,
                            b: composition.background_color[2] as f64,
                            a: composition.background_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for layer in &composition.layers {
                // 1. Get Texture
                let texture_resource = match &layer.source {
                    crate::model::LayerSource::Video { resource_id }
                    | crate::model::LayerSource::Image { resource_id } => {
                        texture_manager.get_resource(resource_id)
                    }
                    _ => None, // Color layers not supported yet in texture pipeline
                };

                if let Some(res) = texture_resource {
                    // 2. Prepare Uniforms
                    // Projection Matrix: We need to map pixel coordinates to Normalized Device Coordinates (-1 to 1)
                    // Or, simpler for now: We assume the camera is over the canvas.
                    // Let's create an Orthographic Projection.
                    // 0,0 top-left -> width,height bottom-right vs -1,1 -> 1,-1

                    let projection = glam::Mat4::orthographic_rh(
                        0.0,
                        composition.dimensions.0 as f32,
                        composition.dimensions.1 as f32,
                        0.0,
                        -1.0,
                        1.0,
                    );

                    let model_matrix = layer.transform.to_matrix();

                    // Final MVP = Projection * Model
                    let transform_final = projection * model_matrix;

                    let uniforms = LayerUniforms {
                        transform: transform_final.to_cols_array_2d().into(), // Convert to mint::ColumnMatrix4 via array
                        opacity: layer.opacity,
                    };

                    // Create temp uniform buffer
                    let uniform_buffer =
                        context
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Temp Uniform Buffer"),
                                contents: uniforms.as_std140().as_bytes(),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });

                    // Create Bind Group 0 (Uniforms)
                    let uniform_bg = context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.pipeline.uniform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: uniform_buffer.as_entire_binding(),
                            }],
                            label: Some("Uniform BG"),
                        });

                    // Create Bind Group 1 (Texture)
                    let view = res
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let texture_bg = context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.pipeline.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                                },
                            ],
                            label: Some("Texture BG"),
                        });

                    render_pass.set_bind_group(0, &uniform_bg, &[]);
                    render_pass.set_bind_group(1, &texture_bg, &[]);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        } // Drop render pass to release borrow

        context.queue.submit(std::iter::once(encoder.finish()));
        sink.present(context); // Handle swapchain presentation or buffer copy

        Ok(())
    }
}
