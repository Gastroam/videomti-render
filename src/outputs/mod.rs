use crate::core::{RenderContext, RenderError};
use wgpu::TextureView;

pub trait RenderSink {
    fn prepare_frame(&mut self) -> Result<TextureView, RenderError>;
    fn present(&mut self, ctx: &RenderContext);
}

pub mod buffer;
pub mod surface;

pub use buffer::BufferSink;
pub use surface::SurfaceSink;
