use super::RenderError;
use wgpu::{Adapter, Device, Instance, Queue, Surface};

pub struct RenderContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl RenderContext {
    /// Initializes the GPU context. If a `surface` is provided, the adapter selection is optimized for it.
    pub async fn new(surface_opt: Option<&Surface<'_>>) -> Result<Self, RenderError> {
        // 1. Instance: Entry point to WGPU
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. Adapter: Hardware selection (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface_opt,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| RenderError::AdapterNotFound)?;

        // 3. Device & Queue: Command channel
        let device_result = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("videomti_main_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .await;

        match device_result {
            Ok((device, queue)) => Ok(Self {
                instance,
                adapter,
                device,
                queue,
            }),
            Err(e) => Err(RenderError::DeviceCreationFailed(e)),
        }
    }
}
