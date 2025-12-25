// Vertex Shader
struct ViewUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> view: ViewUniform;

struct InstanceInput {
    @location(0) position: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    // Quad coords (2 triangles)
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0)
    );
    let uv = uvs[in_vertex_index];

    // Scale by radius and translate by instance position
    // We assume 2D visualization in XY plane for now, or 3D view
    let world_pos = instance.position + vec3<f32>(uv * instance.radius, 0.0);
    
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.uv = uv;
    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Round circle
    let dist = length(in.uv);
    if (dist > 1.0) {
        discard;
    }
    // Antialiasing soft edge
    let alpha = 1.0 - smoothstep(0.9, 1.0, dist);
    return vec4<f32>(in.color, alpha);
}
