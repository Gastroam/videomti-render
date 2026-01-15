use videomti_render::core::RenderContext;
use videomti_render::outputs::RenderSink;
use videomti_render::outputs::buffer::BufferSink;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp,
};

#[tokio::test]
async fn test_headless_render() {
    // 1. Initialize Headless Context
    let ctx = RenderContext::new(None)
        .await
        .expect("Failed to create context");

    // 2. Create Sink
    let width = 256;
    let height = 256;
    let mut sink = BufferSink::new(&ctx, width, height);

    // 3. Prepare Frame
    let view = sink.prepare_frame().expect("Failed to prepare frame");

    // 4. Encode Render Pass (Clear to Red)
    let mut encoder = ctx
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Blue Clear Encoder"),
        });

    {
        let _rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }), // Blue
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None, // Added field for RenderPassDescriptor
        });
    }

    // 5. Copy to Buffer
    sink.copy_to_buffer(&mut encoder);

    // 6. Submit
    ctx.queue.submit(Some(encoder.finish()));

    // 7. Read back
    let padded_bytes = sink.read_pixels(&ctx).await.expect("Failed to read pixels");

    // 8. Remove Padding and Save
    let bytes_per_pixel = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;

    let mut unpadded_bytes = Vec::with_capacity((width * height * bytes_per_pixel) as usize);

    for i in 0..height {
        let start = (i * padded_bytes_per_row) as usize;
        let end = start + unpadded_bytes_per_row as usize;
        unpadded_bytes.extend_from_slice(&padded_bytes[start..end]);
    }

    let img_buffer = image::RgbaImage::from_raw(width, height, unpadded_bytes)
        .expect("Failed to create image buffer");
    img_buffer
        .save("headless_output.png")
        .expect("Failed to save image");

    // Basic verification
    assert!(std::path::Path::new("headless_output.png").exists());
}
