use crevice::std140::AsStd140;

#[derive(AsStd140)]
pub struct LayerUniforms {
    pub transform: mint::ColumnMatrix4<f32>,
    pub opacity: f32,
    // Padding to strict 16-byte alignment is handled by crevice/mint usually,
    // but opacity is f32 (4 bytes). Next might be start of struct or padding.
    // crevice handles padding.
}

// Need to add mint to dependencies since crevice uses it for types
