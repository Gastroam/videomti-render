use std::collections::HashMap;
use uuid::Uuid;
use wgpu::{Device, Extent3d, Queue, Texture, TextureDescriptor, TextureFormat, TextureUsages};

pub struct TextureResource {
    pub texture: Texture,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

pub struct TextureManager {
    // Active resource map
    pub resources: HashMap<Uuid, TextureResource>,
    // orphaned texture pool (Future Optimization)
    // pool: Vec<Texture>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Uploads bytes to GPU texture.
    pub fn update_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        id: Uuid,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        let entry = self.resources.entry(id).or_insert_with(|| {
            // If doesn't exist, create texture on GPU
            let size = Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&TextureDescriptor {
                label: Some(&format!("texture_{}", id)),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb, // Standard for video
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            TextureResource {
                texture,
                width,
                height,
                format: TextureFormat::Rgba8UnormSrgb,
            }
        });

        // Simple dimension check - in prod we should recreate texture if size changed
        if entry.width != width || entry.height != height {
            eprintln!(
                "Warning: Texture size mismatch for ID {}. Expected {}x{}, got {}x{}",
                id, entry.width, entry.height, width, height
            );
            // TODO: Recreate texture
            return;
        }

        // WGPU 0.28: ImageCopyTexture -> TexelCopyTextureInfo
        // ImageDataLayout -> TexelCopyBufferLayout
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width), // 4 bytes per pixel (RGBA)
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_resource(&self, id: &Uuid) -> Option<&TextureResource> {
        self.resources.get(id)
    }
}
