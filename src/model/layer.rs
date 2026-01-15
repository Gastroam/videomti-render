use super::transform::LayerTransform;
use super::types::BlendMode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum LayerSource {
    Video { resource_id: Uuid },
    Image { resource_id: Uuid },
    Color { color: [f32; 4] },
    // Future: Text, Procedural
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub source: LayerSource,
    pub transform: LayerTransform,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    // Effect stack placeholder for now
    #[serde(default)]
    pub effect_stack: Vec<String>,
}

impl Layer {
    pub fn new_color(id: Uuid, color: [f32; 4]) -> Self {
        Self {
            id,
            source: LayerSource::Color { color },
            transform: LayerTransform::default(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            effect_stack: vec![],
        }
    }
}
