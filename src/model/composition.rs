use super::layer::Layer;
use serde::{Deserialize, Serialize};

/// The entry point for rendering a single frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameDescription {
    pub dimensions: (u32, u32),
    pub layers: Vec<Layer>,
    pub background_color: [f32; 4],
}

impl FrameDescription {
    pub fn new(width: u32, height: u32, bg_color: [f32; 4]) -> Self {
        Self {
            dimensions: (width, height),
            layers: vec![],
            background_color: bg_color,
        }
    }
}
