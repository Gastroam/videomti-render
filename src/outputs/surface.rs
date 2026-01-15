use super::RenderSink;
use crate::core::{RenderContext, RenderError};
use wgpu::{Surface, SurfaceConfiguration, SurfaceTexture, TextureView};

pub struct SurfaceSink<'a> {
    pub surface: Surface<'a>,
    pub config: SurfaceConfiguration,
    // We hold the SurfaceTexture temporarily until present is called
    current_surface_texture: Option<wgpu::SurfaceTexture>,
}

impl<'a> SurfaceSink<'a> {
    pub fn new(ctx: &RenderContext, surface: Surface<'a>, width: u32, height: u32) -> Self {
        let caps = surface.get_capabilities(&ctx.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *format,
            width,
            height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&ctx.device, &config);

        Self {
            surface,
            config,
            current_surface_texture: None,
        }
    }

    pub fn resize(&mut self, ctx: &RenderContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&ctx.device, &self.config);
        }
    }
}

impl<'a> RenderSink for SurfaceSink<'a> {
    fn prepare_frame(&mut self) -> Result<TextureView, RenderError> {
        let texture = self
            .surface
            .get_current_texture()
            .map_err(RenderError::SurfaceError)?;

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.current_surface_texture = Some(texture);
        Ok(view)
    }

    fn present(&mut self, _ctx: &RenderContext) {
        if let Some(texture) = self.current_surface_texture.take() {
            texture.present();
        }
    }
}
