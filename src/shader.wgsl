// Vertex Shader

// Stores the output of the vertex shader
struct VertexOutput {
    // tells WGPU this is the value we want to use as the vertex's clip coordinates
    // vec3<f32> = A 3D vector of 32-bit floats
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>
};

// marks it as the entry point for the vertex shader
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;

    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // sets the colo of the current fragment to be a dark brown
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}