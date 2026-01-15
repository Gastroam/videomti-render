use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to find a suitable graphics adapter")]
    AdapterNotFound,
    #[error("Failed to create device: {0}")]
    DeviceCreationFailed(#[from] wgpu::RequestDeviceError),
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}
