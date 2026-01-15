use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Add,
    // Add more as needed
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}
