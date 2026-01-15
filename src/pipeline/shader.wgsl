// Vertex shader structure
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct LayerUniforms {
    transform: mat4x4<f32>,
    opacity: f32, // Passed but need padding handling on CPU side?
    // In WGSL, uniform buffers usually need specific alignment.
    // crevice std140 will handle it.
};

@group(0) @binding(0)
var<uniform> uniforms: LayerUniforms;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>, // Changed to vec3 to match VideoVertex
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.transform * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.uv);
    color.a = color.a * uniforms.opacity;
    return color;
}
