use super::RenderSink;
use crate::core::{RenderContext, RenderError};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Extent3d, MapMode, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};

pub struct BufferSink {
    pub texture: Texture,
    pub output_buffer: Buffer,
    pub dimensions: (u32, u32),
}

impl BufferSink {
    pub fn new(ctx: &RenderContext, width: u32, height: u32) -> Self {
        let texture_desc = TextureDescriptor {
            label: Some("BufferSink Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm, // Standard format
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = ctx.device.create_texture(&texture_desc);

        // Calculate buffer size with padding for alignment
        let bytes_per_pixel = 4; // Rgba8
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;
        let buffer_size = (padded_bytes_per_row * height) as u64;

        let output_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            texture,
            output_buffer,
            dimensions: (width, height),
        }
    }

    /// Read the buffer content back to CPU. This is async.
    pub async fn read_pixels(&self, ctx: &RenderContext) -> Result<Vec<u8>, RenderError> {
        let slice = self.output_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();

        slice.map_async(MapMode::Read, move |v| {
            let _ = tx.send(v);
        });

        ctx.instance.poll_all(true);

        // await the channel, then check the result of mapping
        match rx.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => panic!("Map failed: {:?}", e),
            Err(_) => panic!("Channel closed"),
        }

        let data = slice.get_mapped_range();
        let result = data.to_vec();

        drop(data);
        self.output_buffer.unmap();

        Ok(result)
    }

    // Helper to encode copy
    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder) {
        let (width, height) = self.dimensions;
        let bytes_per_pixel = 4;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

impl RenderSink for BufferSink {
    fn prepare_frame(&mut self) -> Result<TextureView, RenderError> {
        Ok(self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default()))
    }

    fn present(&mut self, ctx: &RenderContext) {
        // Create encoder and copy texture to buffer
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("BufferSink Copy Encoder"),
            });

        self.copy_to_buffer(&mut encoder);

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }
}
