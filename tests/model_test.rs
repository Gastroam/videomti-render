use glam::{Vec2, Vec4};
use uuid::Uuid;
use videomti_render::model::{FrameDescription, Layer, LayerTransform};

#[test]
fn test_matrix_generation() {
    // 1. Setup a transform
    let transform = LayerTransform {
        position: Vec2::new(100.0, 200.0),
        scale: Vec2::new(2.0, 2.0),
        rotation: std::f32::consts::PI / 2.0, // 90 degrees
        anchor: Vec2::new(0.5, 0.5),          // Center
    };

    // 2. Generate Matrix
    let matrix = transform.to_matrix();

    // 3. Verify Logic
    // Apply matrix to a point at Origin (0,0) in local space?
    // Wait, our local space logic relies on "unit quad" assumptions.
    // If we assume the quad is drawn at (-0.5, -0.5) to (0.5, 0.5),
    // then the center is at (0,0) local.
    //
    // A point at (0,0) local (the visual center) should be mapped to `position` (100, 200).
    // Let's test this.

    let center_local = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let center_world = matrix * center_local;

    // Allow small float error
    assert!((center_world.x - 100.0).abs() < 0.001);
    assert!((center_world.y - 200.0).abs() < 0.001);

    // Test Rotation: A point at (0.5, 0.0) local (Right edge of unit quad, if centered)
    // Rotated 90 deg (CCW) -> should point UP (0.0, 0.5)
    // Scaled * 2 -> (0.0, 1.0)
    // Translated to 100, 200 -> (100, 201)

    let point_right = Vec4::new(0.5, 0.0, 0.0, 1.0);
    let point_transformed = matrix * point_right;

    assert!(
        (point_transformed.x - 100.0).abs() < 0.001,
        "Expected X=100, Got {}",
        point_transformed.x
    );
    // Y should be 200 + 1 = 201
    assert!(
        (point_transformed.y - 201.0).abs() < 0.001,
        "Expected Y=201, Got {}",
        point_transformed.y
    );
}

#[test]
fn test_serialization() {
    let id = Uuid::new_v4();
    let mut frame = FrameDescription::new(1920, 1080, [0.0, 0.0, 0.0, 1.0]);

    let layer = Layer::new_color(id, [1.0, 0.0, 0.0, 1.0]);
    frame.layers.push(layer);

    // Serialize
    let json = serde_json::to_string_pretty(&frame).expect("Failed to serialize");
    println!("JSON:\n{}", json);

    // Deserialize
    let frame_back: FrameDescription = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(frame_back.dimensions, (1920, 1080));
    assert_eq!(frame_back.layers.len(), 1);
    assert_eq!(frame_back.layers[0].id, id);
}
