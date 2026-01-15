use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerTransform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32, // in radians
    pub anchor: Vec2,  // Pivot point (0.0-1.0), default usually 0.5,0.5
}

impl Default for LayerTransform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            anchor: Vec2::new(0.5, 0.5),
        }
    }
}

impl LayerTransform {
    /// Generates the model matrix for the shader.
    /// Order: Translate(pos) * Rotate(rot) * Translate(-anchor * size?) * Scale(scale)
    /// Actually, anchor logic usually depends on the size of the object, effectively changing the origin.
    ///
    /// For a normalized quad (-0.5 to 0.5) or (0 to 1), anchor logic shifts it.
    ///
    /// If we assume a standard unit quad centered at 0,0 (-0.5, 0.5), and anchor being the pivot:
    /// But user request says: "Pivot point (0.5, 0.5 is the center)".
    ///
    /// Standard transformation pipeline:
    /// 1. Anchor Offset: Move so anchor is at (0,0)
    /// 2. Scale
    /// 3. Rotate
    /// 4. Translate: Move to final position
    ///
    /// However, if we don't know the pixel size here (only normalized or relative),
    /// we might treat `anchor` as an offset in local space.
    ///
    /// Let's stick to the user's simplified provided implementation logic, but refined for anchors if needed.
    /// User provided:
    /// `Mat4::from_translation(self.position.extend(0.0)) * Mat4::from_rotation_z(self.rotation) * Mat4::from_scale(self.scale.extend(1.0))`
    /// This ignores anchor. We should probably add anchor support properly.
    ///
    /// If we assume the underlying geometry is a quad from -0.5 to 0.5 (W=1, H=1):
    /// Anchor 0.5,0.5 -> Center. No offset needed if geometry is already centered.
    /// Anchor 0.0,0.0 -> Top-Left? Or Bottom-Left?
    ///
    /// Let's implement the standard TRS for now as requested, and note anchor as a TODO if not explicitly mathematically defined yet.
    /// Wait, the user specifically included `anchor` in the struct definition in the prompt.
    ///
    /// Let's try to incorporate it.
    /// Effect of anchor: The point `anchor` in local space stays fixed at `position` in parent space.
    /// Transformation = T * R * S * T_anchor_inverse?
    ///
    /// Actually, let's keep it simple first as per the requested code snippet, but add the anchor field as requested.
    /// I will implement `to_matrix` to respect the anchor if possible, or start with simple TRS.
    ///
    /// Let's assume the vertex shader draws a quad from -0.5 to 0.5.
    /// Anchor (0.5, 0.5) means pivot is at center.
    /// Offset = (0.5 - anchor.x, 0.5 - anchor.y) * Size(1.0)?
    ///
    /// Let's implement the suggested simple version first to pass "Hito 2" prompt spec, then Refine.
    pub fn to_matrix(&self) -> Mat4 {
        // Simple TRS
        let trs = Mat4::from_translation(self.position.extend(0.0))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_scale(self.scale.extend(1.0));

        // If we want to support anchor:
        // We need to shift the geometry so the anchor point is at the origin *before* scaling/rotating.
        // Assuming unit quad centered at 0 (from -0.5 to 0.5).
        // Anchor 0.5, 0.5 is 0,0 in local coords.
        // Anchor 0,0 is -0.5, -0.5 in local coords.
        // Shift = (0.5 - anchor.x, 0.5 - anchor.y)?
        let shift_x = 0.5 - self.anchor.x;
        let shift_y = 0.5 - self.anchor.y; // Y axis direction matters

        // This translation applies PRE-rotation/scale.
        let anchor_transform = Mat4::from_translation(Vec3::new(shift_x, shift_y, 0.0));

        trs * anchor_transform
    }
}
