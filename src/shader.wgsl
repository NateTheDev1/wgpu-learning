// Vertex Shader


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};


// Stores the output of the vertex shader
struct VertexOutput {
    // tells WGPU this is the value we want to use as the vertex's clip coordinates
    // vec3<f32> = A 3D vector of 32-bit floats
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};

// marks it as the entry point for the vertex shader
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment Shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // sets the color of the current fragment to be a dark brown
    return vec4<f32>(in.color, 1.0);
}