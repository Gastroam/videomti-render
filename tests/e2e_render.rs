use glam::vec2;
use std::path::Path;
use uuid::Uuid;
use videomti_render::core::{RenderContext, RenderError}; // Import RenderError
use videomti_render::model::{FrameDescription as Composition, Layer, LayerSource, LayerTransform};
use videomti_render::outputs::BufferSink;
use videomti_render::renderer::Renderer;
use videomti_render::resources::TextureManager;

fn create_test_rgba_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = if (x / 50 + y / 50) % 2 == 0 { 255 } else { 0 };

            buffer.push(r);
            buffer.push(g);
            buffer.push(b);
            buffer.push(255);
        }
    }
    buffer
}

#[tokio::test]
async fn test_full_composition_render() {
    let context = RenderContext::new(None)
        .await
        .expect("Failed to create RenderContext");
    let mut texture_manager = TextureManager::new();
    let mut renderer = Renderer::new(&context);

    let width = 1280;
    let height = 720;
    // Explict type for sink to help compiler?
    let mut sink = BufferSink::new(&context, width, height);

    let video_uuid = Uuid::new_v4();
    let test_pattern = create_test_rgba_pattern(640, 360);
    texture_manager.update_texture(
        &context.device,
        &context.queue,
        video_uuid,
        640,
        360,
        &test_pattern,
    );

    let composition = Composition::new(width, height, [0.1, 0.1, 0.1, 1.0]);
    let mut frame = composition.clone();

    frame.layers.push(Layer {
        id: Uuid::new_v4(),
        source: LayerSource::Video {
            resource_id: video_uuid,
        },
        transform: LayerTransform {
            position: vec2(width as f32 / 2.0, height as f32 / 2.0),
            scale: vec2(1.5, 1.5),
            rotation: 45.0f32.to_radians(),
            anchor: vec2(0.5, 0.5),
        },
        opacity: 0.8,
        effect_stack: vec![],
        blend_mode: Default::default(),
    });

    renderer
        .render(&context, &texture_manager, &frame, &mut sink)
        .expect("Render failed");

    context.instance.poll_all(true);

    // Explicitly handle Result
    let result: Result<Vec<u8>, RenderError> = sink.read_pixels(&context).await;
    let image_data = result.expect("Failed to read pixels");

    let path = Path::new("output_audit.png");
    image::save_buffer(path, &image_data, width, height, image::ColorType::Rgba8).unwrap();

    assert!(path.exists());
}
